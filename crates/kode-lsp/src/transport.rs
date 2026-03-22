use std::io;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::jsonrpc::{self, Message, Notification, Request};

/// LSP transport managing a child server process over stdio.
pub struct LspTransport {
    child: Child,
    sender: mpsc::Sender<Vec<u8>>,
    receiver: mpsc::Receiver<Message>,
}

impl LspTransport {
    /// Spawn an LSP server process and wire up stdio channels.
    pub async fn spawn(command: &str, args: &[&str]) -> io::Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "failed to capture stdin")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "failed to capture stdout")
        })?;

        // Channel for outgoing messages (to stdin)
        let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(64);

        // Channel for incoming messages (from stdout)
        let (in_tx, in_rx) = mpsc::channel::<Message>(64);

        // Writer task: send bytes to stdin
        let mut stdin = stdin;
        tokio::spawn(async move {
            while let Some(data) = out_rx.recv().await {
                if stdin.write_all(&data).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        // Reader task: parse messages from stdout
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                match read_one_message(&mut reader).await {
                    Ok(msg) => {
                        if in_tx.send(msg).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child,
            sender: out_tx,
            receiver: in_rx,
        })
    }

    /// Send a JSON-RPC request.
    pub async fn send_request(&self, req: &Request) -> io::Result<()> {
        let value = serde_json::to_value(req)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let bytes = jsonrpc::encode_message(&value);
        self.sender
            .send(bytes)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "stdin channel closed"))
    }

    /// Send a JSON-RPC notification.
    pub async fn send_notification(&self, notif: &Notification) -> io::Result<()> {
        let value = serde_json::to_value(notif)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let bytes = jsonrpc::encode_message(&value);
        self.sender
            .send(bytes)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "stdin channel closed"))
    }

    /// Receive the next incoming message.
    pub async fn recv(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }

    /// Try to receive without blocking.
    pub fn try_recv(&mut self) -> Option<Message> {
        self.receiver.try_recv().ok()
    }

    /// Kill the child process.
    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.child.kill().await
    }
}

/// Read one complete LSP message from the stream (Content-Length framing).
async fn read_one_message<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> io::Result<Message> {
    // Read headers
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break; // end of headers
        }
        if let Some(val) = trimmed.strip_prefix("Content-Length:") {
            if let Ok(len) = val.trim().parse::<usize>() {
                content_length = Some(len);
            }
        }
    }

    let length = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length")
    })?;

    // Read body
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).await?;

    let value: Value = serde_json::from_slice(&body)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Classify message
    if value.get("id").is_some() && value.get("method").is_none() {
        let resp: jsonrpc::Response = serde_json::from_value(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Message::Response(resp))
    } else {
        let notif: jsonrpc::Notification = serde_json::from_value(value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Message::Notification(notif))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_one_message_from_bytes() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"capabilities":{}}}"#;
        let framed = format!("Content-Length: {}\r\n\r\n{}", json.len(), json);
        let cursor = std::io::Cursor::new(framed.into_bytes());
        let mut reader = BufReader::new(cursor);

        let msg = read_one_message(&mut reader).await.unwrap();
        match msg {
            Message::Response(r) => assert_eq!(r.id, Some(1)),
            _ => panic!("expected response"),
        }
    }
}
