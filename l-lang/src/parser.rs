use codespan_reporting::diagnostic::Label;
use logos::Logos;

pub type Span = core::ops::Range<usize>;
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<()>;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LexerError {
    #[default]
    Invalid,
    // TODO: add more errors if required
}

impl LexerError {
    pub fn into_diagnostic(self, span: Span) -> Diagnostic {
        match self {
            Self::Invalid => Diagnostic::error()
                .with_message("invalid token")
                .with_labels(vec![Label::primary((), span)]),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Logos, Debug, PartialEq, Copy, Clone)]
#[logos(error = LexerError)]
pub enum Token {
    EOF,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("return")]
    Return,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("struct")]
    Struct,
    #[token("else")]
    Else,
    #[token("if")]
    If,
    #[token("->")]
    Arrow,
    #[token("(")]
    LPar,
    #[token(")")]
    RPar,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semi,
    #[token("=")]
    Asn,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token(".")]
    Dot,
    #[token("usize")]
    Usize,
    #[token("bool")]
    Boolean,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Name,
    #[regex("[0-9]+")]
    Int,
    #[regex(r"[ \t\n\f]+")]
    Whitespace,
    Error,
}

// TODO: choose type of CstIndex (in some cases 32 bit is enough)
pub type CstIndex = usize;

// TODO: add context information to the parser if required
#[derive(Default)]
pub struct Context<'a> {
    marker: std::marker::PhantomData<&'a ()>,
}

// TODO: extend tokenization (e.g. check for mismatched parentheses)
pub fn tokenize(
    lexer: logos::Lexer<Token>,
    diags: &mut Vec<Diagnostic>,
) -> (Vec<Token>, Vec<std::ops::Range<CstIndex>>) {
    let mut tokens = vec![];
    let mut ranges = vec![];

    for (token, span) in lexer.spanned() {
        match token {
            Ok(token) => {
                tokens.push(token);
            }
            Err(err) => {
                diags.push(err.into_diagnostic(span.clone()));
                tokens.push(Token::Error);
            }
        }
        ranges.push(span.start as CstIndex..span.end as CstIndex);
    }
    (tokens, ranges)
}
