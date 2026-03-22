use std::collections::HashMap;
use std::io;

use lsp_types::{
    ClientCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
    ServerCapabilities, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    Uri, VersionedTextDocumentIdentifier,
};
use serde_json::Value;
use tokio::sync::oneshot;

use crate::capabilities::ServerCaps;
use crate::jsonrpc::{Message, Notification, Request};
use crate::transport::LspTransport;

/// An LSP client communicating with a single language server.
pub struct LspClient {
    transport: LspTransport,
    pending: HashMap<i64, oneshot::Sender<Value>>,
    next_id: i64,
    server_caps: Option<ServerCaps>,
    language_id: String,
}

fn parse_uri(uri: &str) -> io::Result<Uri> {
    uri.parse::<Uri>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
}

impl LspClient {
    /// Create a new client by spawning an LSP server process.
    pub async fn start(
        language_id: &str,
        command: &str,
        args: &[&str],
    ) -> io::Result<Self> {
        let transport = LspTransport::spawn(command, args).await?;
        Ok(Self {
            transport,
            pending: HashMap::new(),
            next_id: 1,
            server_caps: None,
            language_id: language_id.to_string(),
        })
    }

    /// Send initialize request and wait for response.
    #[allow(deprecated)]
    pub async fn initialize(&mut self, root_uri: &str) -> io::Result<ServerCapabilities> {
        let params = InitializeParams {
            root_uri: root_uri.parse::<Uri>().ok(),
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let result = self.send_request("initialize", Some(value)).await?;

        let init_result: InitializeResult = serde_json::from_value(result)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.server_caps = Some(ServerCaps::new(init_result.capabilities.clone()));

        // Send initialized notification
        self.send_notification("initialized", Some(serde_json::json!({})))
            .await?;

        Ok(init_result.capabilities)
    }

    /// Shut down the LSP server gracefully.
    pub async fn shutdown(&mut self) -> io::Result<()> {
        let _ = self.send_request("shutdown", None).await;
        self.send_notification("exit", None).await?;
        self.transport.shutdown().await
    }

    /// Notify server that a document was opened.
    pub async fn did_open(
        &mut self,
        uri: &str,
        language_id: &str,
        version: i32,
        text: &str,
    ) -> io::Result<()> {
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: parse_uri(uri)?,
                language_id: language_id.to_string(),
                version,
                text: text.to_string(),
            },
        };
        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.send_notification("textDocument/didOpen", Some(value))
            .await
    }

    /// Notify server of document changes (full sync).
    pub async fn did_change(
        &mut self,
        uri: &str,
        version: i32,
        text: &str,
    ) -> io::Result<()> {
        let params = DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: parse_uri(uri)?,
                version,
            },
            content_changes: vec![TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: text.to_string(),
            }],
        };
        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.send_notification("textDocument/didChange", Some(value))
            .await
    }

    /// Notify server that a document was closed.
    pub async fn did_close(&mut self, uri: &str) -> io::Result<()> {
        let params = DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: parse_uri(uri)?,
            },
        };
        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.send_notification("textDocument/didClose", Some(value))
            .await
    }

    /// Notify server that a document was saved.
    pub async fn did_save(&mut self, uri: &str) -> io::Result<()> {
        let params = DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier {
                uri: parse_uri(uri)?,
            },
            text: None,
        };
        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.send_notification("textDocument/didSave", Some(value))
            .await
    }

    /// Send a request and return the result value.
    pub async fn send_request(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> io::Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let req = Request::new(id, method, params);
        self.transport.send_request(&req).await?;

        // Process incoming messages until we get our response
        loop {
            match self.transport.recv().await {
                Some(Message::Response(resp)) => {
                    let resp_id = resp.id.unwrap_or(-1);
                    if resp_id == id {
                        if let Some(err) = resp.error {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("LSP error {}: {}", err.code, err.message),
                            ));
                        }
                        return Ok(resp.result.unwrap_or(Value::Null));
                    }
                    // Response for a different request
                    if let Some(sender) = self.pending.remove(&resp_id) {
                        let _ = sender.send(resp.result.unwrap_or(Value::Null));
                    }
                }
                Some(Message::Notification(notif)) => {
                    self.handle_server_notification(notif);
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::ConnectionAborted,
                        "LSP server disconnected",
                    ));
                }
            }
        }
    }

    /// Send a notification (no response expected).
    pub async fn send_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> io::Result<()> {
        let notif = Notification::new(method, params);
        self.transport.send_notification(&notif).await
    }

    /// Process any pending incoming messages without blocking.
    pub fn process_incoming(&mut self) {
        while let Some(msg) = self.transport.try_recv() {
            match msg {
                Message::Response(resp) => {
                    let id = resp.id.unwrap_or(-1);
                    if let Some(sender) = self.pending.remove(&id) {
                        let _ = sender.send(resp.result.unwrap_or(Value::Null));
                    }
                }
                Message::Notification(notif) => {
                    self.handle_server_notification(notif);
                }
            }
        }
    }

    fn handle_server_notification(&self, notif: Notification) {
        tracing::debug!("Server notification: {}", notif.method);
    }

    pub fn server_caps(&self) -> Option<&ServerCaps> {
        self.server_caps.as_ref()
    }

    pub fn language_id(&self) -> &str {
        &self.language_id
    }
}
