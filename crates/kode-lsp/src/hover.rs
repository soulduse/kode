use std::io;

use lsp_types::{
    HoverParams, Position, TextDocumentIdentifier, TextDocumentPositionParams, Uri,
};

use crate::client::LspClient;

/// Simplified hover information for display.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<lsp_types::Range>,
}

impl LspClient {
    /// Request hover information at a position.
    pub async fn hover(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> io::Result<Option<HoverInfo>> {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: uri.parse::<Uri>()
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
                },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
        };

        let value = serde_json::to_value(params)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let result = self
            .send_request("textDocument/hover", Some(value))
            .await?;

        if result.is_null() {
            return Ok(None);
        }

        let hover: lsp_types::Hover = serde_json::from_value(result)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let contents = extract_hover_contents(&hover.contents);
        Ok(Some(HoverInfo {
            contents,
            range: hover.range,
        }))
    }
}

fn extract_hover_contents(contents: &lsp_types::HoverContents) -> String {
    use lsp_types::{HoverContents, MarkedString, MarkupContent};

    match contents {
        HoverContents::Scalar(MarkedString::String(s)) => s.clone(),
        HoverContents::Scalar(MarkedString::LanguageString(ls)) => {
            format!("```{}\n{}\n```", ls.language, ls.value)
        }
        HoverContents::Array(arr) => arr
            .iter()
            .map(|ms| match ms {
                MarkedString::String(s) => s.clone(),
                MarkedString::LanguageString(ls) => {
                    format!("```{}\n{}\n```", ls.language, ls.value)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n"),
        HoverContents::Markup(MarkupContent { value, .. }) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{HoverContents, MarkupContent, MarkupKind};

    #[test]
    fn extract_markup_content() {
        let contents = HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: "**fun** main()".into(),
        });
        assert_eq!(extract_hover_contents(&contents), "**fun** main()");
    }

    #[test]
    fn extract_scalar_string() {
        let contents = HoverContents::Scalar(lsp_types::MarkedString::String("hello".into()));
        assert_eq!(extract_hover_contents(&contents), "hello");
    }
}
