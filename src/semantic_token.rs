use tower_lsp::lsp_types::{SemanticTokenType, SemanticToken};

pub const LEGEND_TYPE: [SemanticTokenType; 7] = [
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::COMMENT,
    SemanticTokenType::NUMBER,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR
];
