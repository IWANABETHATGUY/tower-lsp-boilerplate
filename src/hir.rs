use crate::{
    span::Span,
    symbol_table::{ReferenceId, SymbolId},
};
#[derive(Debug)]
pub enum Literal {
    Null(NullLiteral),
    Bool(BoolLiteral),
    NumeralLiteral(NumeralLiteral),
    StringLiteral(StringLiteral),
}

#[derive(Debug)]
pub struct NullLiteral {
    span: Span,
}

#[derive(Debug)]
pub struct BoolLiteral {
    span: Span,
    value: bool,
}

#[derive(Debug)]
pub struct NumeralLiteral {
    span: Span,
    value: f64,
}

#[derive(Debug)]
pub struct StringLiteral {
    span: Span,
    value: String,
}

#[derive(Debug)]
pub struct List {
    span: Span,
}

pub struct IdentBinding {
    symbol: SymbolId,
    span: Span,
}

pub struct IdentReference {
    span: Span,
    reference_id: ReferenceId,
    symbol: Option<SymbolId>,
}

// #[derive(Debug)]
// pub enum Expr {
//     Error,
//     Value(Value),
//     List(Vec<Spanned<Self>>),
//     Local(Spanned<String>),
//     Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>, Span),
//     Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
//     Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
//     Call(Box<Spanned<Self>>, Spanned<Vec<Spanned<Self>>>),
//     If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
//     Print(Box<Spanned<Self>>),
// }
//

pub enum Expr {
    Literal(Literal),
    List(ListExpr),
    Reference(IdentReference),
    LetExpr(LetExpr),
    /// with this dummy node, we can easily know if the expr is a statement or a expr, without
    /// store `semiColon` for each expr e.g.
    /// ```rs
    /// fn test() {
    ///   let a = 1;
    /// }
    /// ```
    /// the body of `test` should be `vec![Expr::LetExpr(LetExpr { ... }), Expr::Null]`
    /// so we know the function returns None type
    /// `
    Null,
}

pub struct ListExpr {
    span: Span,
    exprs: Vec<Expr>,
}

pub struct LetExpr {}

pub fn test() {}
