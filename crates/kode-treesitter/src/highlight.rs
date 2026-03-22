/// A highlight span in a line of text.
#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start_col: usize,
    pub end_col: usize,
    pub group: HighlightGroup,
}

/// Semantic highlight groups that map to theme colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightGroup {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Punctuation,
    Property,
    Constant,
    Attribute,
    Namespace,
    Label,
    Error,
    None,
}

impl HighlightGroup {
    pub fn from_capture_name(name: &str) -> Self {
        match name {
            "keyword" | "keyword.function" | "keyword.return" | "keyword.operator"
            | "keyword.import" | "keyword.modifier" => Self::Keyword,
            "string" | "string.special" => Self::String,
            "number" | "float" => Self::Number,
            "comment" | "comment.line" | "comment.block" => Self::Comment,
            "function" | "function.method" | "function.call" | "function.builtin" => {
                Self::Function
            }
            "type" | "type.builtin" | "type.definition" => Self::Type,
            "variable" | "variable.parameter" | "variable.builtin" => Self::Variable,
            "operator" => Self::Operator,
            "punctuation" | "punctuation.bracket" | "punctuation.delimiter" => Self::Punctuation,
            "property" | "property.definition" => Self::Property,
            "constant" | "constant.builtin" | "boolean" => Self::Constant,
            "attribute" | "annotation" => Self::Attribute,
            "namespace" | "module" => Self::Namespace,
            "label" => Self::Label,
            "error" => Self::Error,
            _ => Self::None,
        }
    }
}

/// Highlighted line ready for rendering.
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    pub line_idx: usize,
    pub text: String,
    pub spans: Vec<HighlightSpan>,
}
