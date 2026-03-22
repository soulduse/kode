use kode_plugin_sdk::{set_response, Decoration, PluginEvent, PluginResponse};

#[unsafe(no_mangle)]
pub extern "C" fn kode_plugin_init() -> i32 {
    kode_plugin_sdk::log("git-blame initialized");
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn kode_plugin_info(_ptr: i32) -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn kode_plugin_handle_event(event_ptr: i32, event_len: i32) -> i32 {
    let event_bytes =
        unsafe { std::slice::from_raw_parts(event_ptr as *const u8, event_len as usize) };

    let event: PluginEvent = match serde_json::from_slice(event_bytes) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    // Only handle cursor_move with a line number
    let line = match event.line {
        Some(l) => l,
        None => return 0,
    };

    // In a real implementation, we would call host_git_blame here.
    // For now, return a placeholder annotation showing the concept.
    let response = PluginResponse {
        decorations: vec![Decoration {
            line,
            col_start: None,
            col_end: None,
            color: Some("#6c7086".to_string()),
            style: "foreground".to_string(),
            annotation: Some(format!("line {} — git blame pending", line + 1)),
            side: Some("right".to_string()),
        }],
    };

    set_response(&response)
}
