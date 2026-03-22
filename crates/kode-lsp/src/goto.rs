use std::io;

use lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, Location, Position,
    TextDocumentIdentifier, TextDocumentPositionParams, Uri,
};
use serde_json::Value;

use crate::client::LspClient;

impl LspClient {
    /// Go to definition at a position.
    pub async fn goto_definition(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> io::Result<Vec<Location>> {
        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: parse_uri(uri)?,
                },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self
            .send_request("textDocument/definition", Some(value))
            .await?;

        parse_goto_response(result)
    }

    /// Go to implementation at a position.
    pub async fn goto_implementation(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> io::Result<Vec<Location>> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character}
        });
        let result = self
            .send_request("textDocument/implementation", Some(params))
            .await?;
        parse_goto_response(result)
    }

    /// Find all references at a position.
    pub async fn find_references(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> io::Result<Vec<Location>> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri},
            "position": {"line": line, "character": character},
            "context": {"includeDeclaration": true}
        });
        let result = self
            .send_request("textDocument/references", Some(params))
            .await?;
        parse_goto_response(result)
    }
}

fn parse_goto_response(value: Value) -> io::Result<Vec<Location>> {
    if value.is_null() {
        return Ok(vec![]);
    }

    if let Ok(loc) = serde_json::from_value::<Location>(value.clone()) {
        return Ok(vec![loc]);
    }

    if let Ok(resp) = serde_json::from_value::<GotoDefinitionResponse>(value.clone()) {
        match resp {
            GotoDefinitionResponse::Scalar(loc) => Ok(vec![loc]),
            GotoDefinitionResponse::Array(locs) => Ok(locs),
            GotoDefinitionResponse::Link(links) => {
                Ok(links
                    .into_iter()
                    .map(|link| Location {
                        uri: link.target_uri,
                        range: link.target_selection_range,
                    })
                    .collect())
            }
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
    fn parse_null_goto() {
        let result = parse_goto_response(Value::Null).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_single_location() {
        let json = serde_json::json!({
            "uri": "file:///test.kt",
            "range": {
                "start": {"line": 10, "character": 5},
                "end": {"line": 10, "character": 15}
            }
        });
        let result = parse_goto_response(json).unwrap();
        assert_eq!(result.len(), 1);
    }
}
