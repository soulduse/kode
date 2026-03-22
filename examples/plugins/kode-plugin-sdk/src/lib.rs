//! Kode Plugin SDK — shared types and utilities for WASM plugins.

use serde::{Deserialize, Serialize};
use std::alloc::{alloc, dealloc, Layout};

// ── Host imports ──

extern "C" {
    pub fn host_log(ptr: i32, len: i32);
    pub fn host_get_buffer_content(uri_ptr: i32, uri_len: i32, out_ptr: i32, out_max: i32) -> i32;
    pub fn host_get_buffer_line_count(uri_ptr: i32, uri_len: i32) -> i32;
}

// ── Memory management (exported to host) ──

static mut RESPONSE_BUF: Vec<u8> = Vec::new();

#[unsafe(no_mangle)]
pub extern "C" fn kode_alloc(size: i32) -> i32 {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { alloc(layout) as i32 }
}

#[unsafe(no_mangle)]
pub extern "C" fn kode_dealloc(ptr: i32, size: i32) {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe {
        dealloc(ptr as *mut u8, layout);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kode_plugin_get_response_len() -> i32 {
    unsafe { RESPONSE_BUF.len() as i32 }
}

// ── Convenience functions ──

/// Log a message to the host.
pub fn log(msg: &str) {
    unsafe {
        host_log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

/// Set the response buffer and return a pointer to it.
pub fn set_response(response: &PluginResponse) -> i32 {
    let json = serde_json::to_vec(response).unwrap_or_default();
    unsafe {
        RESPONSE_BUF = json;
        RESPONSE_BUF.as_ptr() as i32
    }
}

// ── Types ──

#[derive(Debug, Deserialize)]
pub struct PluginEvent {
    pub event_type: String,
    pub uri: Option<String>,
    pub language: Option<String>,
    pub line: Option<u32>,
    #[serde(default)]
    pub content_changed: bool,
}

#[derive(Debug, Serialize, Default)]
pub struct PluginResponse {
    pub decorations: Vec<Decoration>,
}

#[derive(Debug, Serialize)]
pub struct Decoration {
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_end: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub style: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
}
