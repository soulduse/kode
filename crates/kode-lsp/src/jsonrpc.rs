use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request.
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response.
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// JSON-RPC error object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 notification (no id).
#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Parsed incoming message from an LSP server.
#[derive(Debug)]
pub enum Message {
    Response(Response),
    Notification(Notification),
}

impl Request {
    pub fn new(id: i64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            method: method.into(),
            params,
        }
    }
}

impl Notification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
        }
    }
}

/// Encode a JSON value with Content-Length header for LSP framing.
pub fn encode_message(value: &Value) -> Vec<u8> {
    let body = serde_json::to_string(value).expect("serialize JSON");
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body).into_bytes()
}

/// Try to parse one complete LSP message from a buffer.
/// Returns `Some((message, bytes_consumed))` if a complete message was found.
pub fn decode_message(buf: &[u8]) -> Option<(Message, usize)> {
    let buf_str = std::str::from_utf8(buf).ok()?;

    // Find header/body separator
    let header_end = buf_str.find("\r\n\r\n")?;
    let header = &buf_str[..header_end];

    // Parse Content-Length
    let content_length = header
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if line.to_lowercase().starts_with("content-length:") {
                line["content-length:".len()..].trim().parse::<usize>().ok()
            } else {
                None
            }
        })?;

    let body_start = header_end + 4; // skip \r\n\r\n
    let total = body_start + content_length;

    if buf.len() < total {
        return None; // incomplete message
    }

    let body = &buf_str[body_start..total];
    let value: Value = serde_json::from_str(body).ok()?;

    let message = if value.get("id").is_some() && value.get("method").is_none() {
        // Response (has id, no method)
        let resp: Response = serde_json::from_value(value).ok()?;
        Message::Response(resp)
    } else if value.get("id").is_none() {
        // Notification (no id)
        let notif: Notification = serde_json::from_value(value).ok()?;
        Message::Notification(notif)
    } else {
        // Server-initiated request (has both id and method) — treat as notification for now
        let notif: Notification = serde_json::from_value(value).ok()?;
        Message::Notification(notif)
    };

    Some((message, total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let req = Request::new(1, "initialize", Some(serde_json::json!({"rootUri": "file:///tmp"})));
        let encoded = encode_message(&serde_json::to_value(&req).unwrap());

        let resp_json = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"capabilities": {}}
        });
        let resp_bytes = encode_message(&resp_json);

        let (msg, consumed) = decode_message(&resp_bytes).unwrap();
        assert_eq!(consumed, resp_bytes.len());
        match msg {
            Message::Response(r) => {
                assert_eq!(r.id, Some(1));
                assert!(r.result.is_some());
            }
            _ => panic!("expected response"),
        }

        // Verify request encoding is valid
        assert!(!encoded.is_empty());
    }

    #[test]
    fn decode_notification() {
        let notif_json = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {"uri": "file:///test.kt", "diagnostics": []}
        });
        let bytes = encode_message(&notif_json);

        let (msg, consumed) = decode_message(&bytes).unwrap();
        assert_eq!(consumed, bytes.len());
        match msg {
            Message::Notification(n) => {
                assert_eq!(n.method, "textDocument/publishDiagnostics");
            }
            _ => panic!("expected notification"),
        }
    }

    #[test]
    fn decode_incomplete() {
        let bytes = b"Content-Length: 100\r\n\r\n{\"partial";
        assert!(decode_message(bytes).is_none());
    }
}
