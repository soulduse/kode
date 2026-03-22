use kode_plugin_sdk::{set_response, Decoration, PluginEvent, PluginResponse};

#[unsafe(no_mangle)]
pub extern "C" fn kode_plugin_init() -> i32 {
    kode_plugin_sdk::log("todo-highlighter initialized");
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

    // Read buffer content from host
    let uri = match &event.uri {
        Some(u) => u,
        None => return 0,
    };

    let mut buf = vec![0u8; 1024 * 64]; // 64KB max
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
    let response = scan_todos(content);
    set_response(&response)
}

fn scan_todos(content: &str) -> PluginResponse {
    let keywords = ["TODO", "FIXME", "HACK", "XXX"];
    let colors = ["#fab387", "#f38ba8", "#a6e3a1", "#f9e2af"];
    let mut decorations = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        for (i, keyword) in keywords.iter().enumerate() {
            if let Some(col) = line.find(keyword) {
                decorations.push(Decoration {
                    line: line_idx as u32,
                    col_start: Some(col as u32),
                    col_end: Some((col + keyword.len()) as u32),
                    color: Some(colors[i].to_string()),
                    style: "foreground".to_string(),
                    annotation: Some(format!("{} comment", keyword)),
                    side: Some("right".to_string()),
                });
            }
        }
    }

    PluginResponse { decorations }
}
