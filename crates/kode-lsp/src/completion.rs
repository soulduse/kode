use std::io;

use lsp_types::{
    CompletionItem, CompletionParams, CompletionResponse, Position,
    TextDocumentIdentifier, TextDocumentPositionParams, Uri,
};
use serde_json::Value;

use crate::client::LspClient;

impl LspClient {
    /// Request completions at a position.
    pub async fn completion(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> io::Result<Vec<CompletionItem>> {
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: parse_uri(uri)?,
                },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self.send_request("textDocument/completion", Some(value)).await?;

        parse_completion_response(result)
    }

    /// Resolve a completion item for more details.
    pub async fn resolve_completion(
        &mut self,
        item: CompletionItem,
    ) -> io::Result<CompletionItem> {
        let value = serde_json::to_value(item)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self.send_request("completionItem/resolve", Some(value)).await?;
        serde_json::from_value(result)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

fn parse_completion_response(value: Value) -> io::Result<Vec<CompletionItem>> {
    if value.is_null() {
        return Ok(vec![]);
    }

    // Try CompletionList first, then Vec<CompletionItem>
    if let Ok(resp) = serde_json::from_value::<CompletionResponse>(value.clone()) {
        match resp {
            CompletionResponse::Array(items) => Ok(items),
            CompletionResponse::List(list) => Ok(list.items),
        }
    } else {
        Ok(vec![])
    }
}

fn parse_uri(uri: &str) -> io::Result<Uri> {
    uri.parse::<Uri>().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_completion() {
        let result = parse_completion_response(Value::Null).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_completion_array() {
        let json = serde_json::json!([
            {"label": "println", "kind": 3},
            {"label": "print", "kind": 3}
        ]);
        let result = parse_completion_response(json).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].label, "println");
    }

    #[test]
    fn parse_completion_list() {
        let json = serde_json::json!({
            "isIncomplete": false,
            "items": [{"label": "foo"}]
        });
        let result = parse_completion_response(json).unwrap();
        assert_eq!(result.len(), 1);
    }
}
