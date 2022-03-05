use tower_lsp::lsp_types::SemanticTokenType;

pub const LEGEND_TYPE: [SemanticTokenType; 6] = [
    SemanticTokenType::FUNCTION,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::STRING,
    SemanticTokenType::COMMENT,
    SemanticTokenType::NUMBER,
    SemanticTokenType::KEYWORD,
];
