use kode_plugin_sdk::{set_response, Decoration, PluginEvent, PluginResponse};

const RAINBOW: &[&str] = &["#f9e2af", "#fab387", "#f38ba8", "#cba6f7", "#89b4fa", "#a6e3a1"];

#[unsafe(no_mangle)]
pub extern "C" fn kode_plugin_init() -> i32 {
    kode_plugin_sdk::log("bracket-colorizer initialized");
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

    if event.event_type != "buffer_open" && event.event_type != "buffer_change" {
        return 0;
    }

    let uri = match &event.uri {
        Some(u) => u,
        None => return 0,
    };

    let mut buf = vec![0u8; 1024 * 64];
    let len = unsafe {
        kode_plugin_sdk::host_get_buffer_content(
            uri.as_ptr() as i32,
            uri.len() as i32,
            buf.as_mut_ptr() as i32,
            buf.len() as i32,
        )
    };

    if len <= 0 {
        return 0;
    }

    let content = std::str::from_utf8(&buf[..len as usize]).unwrap_or("");
    let response = colorize_brackets(content);
    set_response(&response)
}

fn colorize_brackets(content: &str) -> PluginResponse {
    let mut decorations = Vec::new();
    let mut depth: i32 = 0;

    for (line_idx, line) in content.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            match ch {
                '(' | '[' | '{' => {
                    let color = RAINBOW[depth as usize % RAINBOW.len()];
                    decorations.push(Decoration {
                        line: line_idx as u32,
                        col_start: Some(col as u32),
                        col_end: Some((col + 1) as u32),
                        color: Some(color.to_string()),
                        style: "foreground".to_string(),
                        annotation: None,
                        side: None,
                    });
                    depth += 1;
                }
                ')' | ']' | '}' => {
                    depth = (depth - 1).max(0);
                    let color = RAINBOW[depth as usize % RAINBOW.len()];
                    decorations.push(Decoration {
                        line: line_idx as u32,
                        col_start: Some(col as u32),
                        col_end: Some((col + 1) as u32),
                        color: Some(color.to_string()),
                        style: "foreground".to_string(),
                        annotation: None,
                        side: None,
                    });
                }
                _ => {}
            }
        }
    }

    PluginResponse { decorations }
}
