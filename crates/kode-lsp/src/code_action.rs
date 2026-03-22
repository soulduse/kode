use std::io;

use lsp_types::{
    CodeActionContext, CodeActionOrCommand, CodeActionParams, Diagnostic, Range,
    TextDocumentIdentifier, Uri, WorkspaceEdit,
};

use crate::client::LspClient;

impl LspClient {
    /// Request code actions for a range.
    pub async fn code_actions(
        &mut self,
        uri: &str,
        range: Range,
        diagnostics: Vec<Diagnostic>,
    ) -> io::Result<Vec<CodeActionOrCommand>> {
        let params = CodeActionParams {
            text_document: TextDocumentIdentifier {
                uri: uri.parse::<Uri>()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
            },
            range,
            context: CodeActionContext {
                diagnostics,
                only: None,
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self
            .send_request("textDocument/codeAction", Some(value))
            .await?;

        if result.is_null() {
            return Ok(vec![]);
        }

        serde_json::from_value(result).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

/// Apply a workspace edit by collecting the text edits per document.
pub fn collect_workspace_edits(edit: &WorkspaceEdit) -> Vec<(String, Vec<lsp_types::TextEdit>)> {
    let mut result = Vec::new();
    if let Some(changes) = &edit.changes {
        for (uri, edits) in changes {
            result.push((uri.as_str().to_string(), edits.clone()));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_empty_edits() {
        let edit = WorkspaceEdit::default();
        let result = collect_workspace_edits(&edit);
        assert!(result.is_empty());
    }
}
