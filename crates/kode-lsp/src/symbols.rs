use std::io;

use lsp_types::{
    DocumentSymbolParams, DocumentSymbolResponse, SymbolInformation, TextDocumentIdentifier, Uri,
    WorkspaceSymbolParams,
};

use crate::client::LspClient;

/// Flattened symbol info for display.
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: lsp_types::SymbolKind,
    pub range: lsp_types::Range,
    pub detail: Option<String>,
}

impl LspClient {
    /// Get document symbols.
    pub async fn document_symbols(&mut self, uri: &str) -> io::Result<Vec<SymbolInfo>> {
        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse::<Uri>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self
            .send_request("textDocument/documentSymbol", Some(value))
            .await?;

        if result.is_null() {
            return Ok(vec![]);
        }

        let response: DocumentSymbolResponse = serde_json::from_value(result)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(flatten_symbols(response))
    }

    /// Search workspace symbols.
    pub async fn workspace_symbols(
        &mut self,
        query: &str,
    ) -> io::Result<Vec<SymbolInformation>> {
        let params = WorkspaceSymbolParams {
            query: query.to_string(),
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self
            .send_request("workspace/symbol", Some(value))
            .await?;

        if result.is_null() {
            return Ok(vec![]);
        }

        serde_json::from_value(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

fn flatten_symbols(response: DocumentSymbolResponse) -> Vec<SymbolInfo> {
    let mut result = Vec::new();
    match response {
        DocumentSymbolResponse::Flat(symbols) => {
            for sym in symbols {
                result.push(SymbolInfo {
                    name: sym.name,
                    kind: sym.kind,
                    range: sym.location.range,
                    detail: None,
                });
            }
        }
        DocumentSymbolResponse::Nested(symbols) => {
            flatten_document_symbols(&symbols, &mut result);
        }
    }
    result
}

fn flatten_document_symbols(
    symbols: &[lsp_types::DocumentSymbol],
    out: &mut Vec<SymbolInfo>,
) {
    for sym in symbols {
        out.push(SymbolInfo {
            name: sym.name.clone(),
            kind: sym.kind,
            range: sym.range,
            detail: sym.detail.clone(),
        });
        if let Some(children) = &sym.children {
            flatten_document_symbols(children, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{DocumentSymbol, SymbolKind};

    #[test]
    fn flatten_nested_symbols() {
        let response = DocumentSymbolResponse::Nested(vec![DocumentSymbol {
            name: "MyClass".into(),
            detail: None,
            kind: SymbolKind::CLASS,
            tags: None,
            deprecated: None,
            range: lsp_types::Range::default(),
            selection_range: lsp_types::Range::default(),
            children: Some(vec![DocumentSymbol {
                name: "myMethod".into(),
                detail: None,
                kind: SymbolKind::METHOD,
                tags: None,
                deprecated: None,
                range: lsp_types::Range::default(),
                selection_range: lsp_types::Range::default(),
                children: None,
            }]),
        }]);

        let flat = flatten_symbols(response);
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].name, "MyClass");
        assert_eq!(flat[1].name, "myMethod");
    }
}
