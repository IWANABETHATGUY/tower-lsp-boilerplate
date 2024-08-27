use std::mem::transmute;

use codespan_reporting::diagnostic::Diagnostic;
use derivative::Derivative;
use l_lang::generated::{Cst, Parser};
use l_lang::parser::{tokenize, Token};
use logos::Logos;

pub mod chumsky;
pub mod completion;
pub mod jump_definition;
pub mod reference;
pub mod semantic_token;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CstWithSource {
    pub source: String,
    #[derivative(Debug = "ignore")]
    pub cst: Cst<'static>,
}

impl CstWithSource {
    pub fn new(source: String) -> (Self, Vec<Diagnostic<()>>) {
        let mut diags = vec![];
        let (tokens, ranges) = tokenize(Token::lexer(&source), &mut diags);
        (
            Self {
                cst: Parser::parse(
                    // SAFETY: we could ensure that source could live as least as cst
                    unsafe { transmute::<&str, &'static str>(source.as_str()) },
                    tokens,
                    ranges,
                    &mut diags,
                ),
                source,
            },
            diags,
        )
    }
}
