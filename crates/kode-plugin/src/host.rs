use std::path::Path;

use wasmtime::*;
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;

use crate::abi::{PluginEvent, PluginResponse};
use crate::manifest::PluginManifest;

/// Host state accessible from WASM imports.
pub struct HostState {
    pub wasi: WasiP1Ctx,
    pub limits: StoreLimits,
    pub log_buffer: Vec<String>,
    /// Fuel limit per call (for timeout).
    pub fuel_limit: u64,
}

/// A loaded and instantiated WASM plugin.
pub struct PluginInstance {
    store: Store<HostState>,
    instance: Instance,
    pub manifest: PluginManifest,
    pub enabled: bool,
    consecutive_failures: u32,
}

const MAX_CONSECUTIVE_FAILURES: u32 = 5;
const FUEL_PER_MS: u64 = 10_000;

impl PluginInstance {
    /// Load a plugin from its manifest.
    pub fn load(engine: &Engine, manifest: PluginManifest) -> Result<Self> {
        let wasm_path = manifest.wasm_path();
        if !wasm_path.exists() {
            anyhow::bail!("WASM file not found: {}", wasm_path.display());
        }

        let module = Module::from_file(engine, &wasm_path)?;

        let wasi = WasiCtxBuilder::new()
            .build_p1();

        let fuel_limit = manifest.limits.timeout_ms * FUEL_PER_MS;

        let memory_bytes = (manifest.limits.max_memory_mb as usize) * 1024 * 1024;
        let limits = StoreLimitsBuilder::new()
            .memory_size(memory_bytes)
            .build();

        let mut store = Store::new(
            engine,
            HostState {
                wasi,
                limits,
                log_buffer: Vec::new(),
                fuel_limit,
            },
        );

        store.limiter(|state| &mut state.limits);

        // Add initial fuel
        store.set_fuel(fuel_limit)?;

        let mut linker = Linker::new(engine);

        // Add WASI imports
        wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |state: &mut HostState| {
            &mut state.wasi
        })?;

        // Add host imports
        linker.func_wrap(
            "kode",
            "host_log",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                if let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                    let mut buf = vec![0u8; len as usize];
                    if memory.read(&caller, ptr as usize, &mut buf).is_ok() {
                        if let Ok(msg) = std::str::from_utf8(&buf) {
                            caller.data_mut().log_buffer.push(msg.to_string());
                        }
                    }
                }
            },
        )?;

        linker.func_wrap(
            "kode",
            "host_get_buffer_content",
            |_caller: Caller<'_, HostState>,
             _uri_ptr: i32,
             _uri_len: i32,
             _out_ptr: i32,
             _out_max: i32|
             -> i32 {
                // TODO: implement buffer content access via callback
                0
            },
        )?;

        linker.func_wrap(
            "kode",
            "host_get_buffer_line_count",
            |_caller: Caller<'_, HostState>, _uri_ptr: i32, _uri_len: i32| -> i32 {
                // TODO: implement line count access via callback
                0
            },
        )?;

        let instance = linker.instantiate(&mut store, &module)?;

        // Call init
        if let Some(init) = instance.get_typed_func::<(), i32>(&mut store, "kode_plugin_init").ok()
        {
            let result = init.call(&mut store, ())?;
            if result != 0 {
                anyhow::bail!("Plugin init returned error: {}", result);
            }
        }

        Ok(Self {
            store,
            instance,
            manifest,
            enabled: true,
            consecutive_failures: 0,
        })
    }

    /// Send an event to the plugin and get decorations back.
    pub fn handle_event(&mut self, event: &PluginEvent) -> Option<PluginResponse> {
        if !self.enabled {
            return None;
        }

        if !self.manifest.subscribes_to(&event.event_type) {
            return None;
        }

        // Reset fuel for this call
        let fuel = self.store.data().fuel_limit;
        let _ = self.store.set_fuel(fuel);

        match self.handle_event_inner(event) {
            Ok(response) => {
                self.consecutive_failures = 0;
                Some(response)
            }
            Err(e) => {
                self.consecutive_failures += 1;
                tracing::warn!(
                    "Plugin '{}' error ({}/{}): {}",
                    self.manifest.plugin.name,
                    self.consecutive_failures,
                    MAX_CONSECUTIVE_FAILURES,
                    e
                );
                if self.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                    tracing::error!(
                        "Plugin '{}' disabled after {} consecutive failures",
                        self.manifest.plugin.name,
                        MAX_CONSECUTIVE_FAILURES
                    );
                    self.enabled = false;
                }
                None
            }
        }
    }

    fn handle_event_inner(&mut self, event: &PluginEvent) -> Result<PluginResponse> {
        let event_json = serde_json::to_vec(event)?;
        let event_len = event_json.len() as i32;

        // Allocate memory in WASM for the event
        let alloc = self
            .instance
            .get_typed_func::<i32, i32>(&mut self.store, "kode_alloc")?;
        let event_ptr = alloc.call(&mut self.store, event_len)?;

        // Write event JSON into WASM memory
        let memory = self
            .instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("no memory export"))?;
        memory.write(&mut self.store, event_ptr as usize, &event_json)?;

        // Call the handler
        let handle = self
            .instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "kode_plugin_handle_event")?;
        let response_ptr = handle.call(&mut self.store, (event_ptr, event_len))?;

        if response_ptr == 0 {
            return Ok(PluginResponse::default());
        }

        // Get response length
        let get_len = self
            .instance
            .get_typed_func::<(), i32>(&mut self.store, "kode_plugin_get_response_len")?;
        let response_len = get_len.call(&mut self.store, ())? as usize;

        if response_len == 0 {
            return Ok(PluginResponse::default());
        }

        // Read response from WASM memory
        let mut response_buf = vec![0u8; response_len];
        memory.read(&self.store, response_ptr as usize, &mut response_buf)?;

        let response: PluginResponse = serde_json::from_slice(&response_buf)?;
        Ok(response)
    }

    pub fn name(&self) -> &str {
        &self.manifest.plugin.name
    }

    pub fn drain_logs(&mut self) -> Vec<String> {
        std::mem::take(&mut self.store.data_mut().log_buffer)
    }
}

/// Create a wasmtime engine configured for plugins.
pub fn create_engine() -> Result<Engine> {
    let mut config = Config::new();
    config.consume_fuel(true);
    Engine::new(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_engine_works() {
        let engine = create_engine();
        assert!(engine.is_ok());
    }
}
