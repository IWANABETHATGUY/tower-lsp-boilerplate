use rust_lapper::{Interval, Lapper};
use std::fmt::Display;

use crate::{
    nrs_lang::{Ast, Expr, Func},
    span::Span,
    symbol_table::{ReferenceId, SymbolId, SymbolTable},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SemanticError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentType {
    Binding(SymbolId),
    Reference(ReferenceId),
}
type IdentRangeLapper = Lapper<usize, IdentType>;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    Null,
    Function,
    List(Box<Type>),
    Unknown,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number => write!(f, "number"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Null => write!(f, "null"),
            Type::Function => write!(f, "function"),
            Type::List(ty) => write!(f, "list<{}>", ty),
            Type::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Error, Debug)]
pub enum SemanticError {
    #[error("Undefined variable {name}")]
    UndefinedVariable { name: String, span: Span },
    #[error("Expect element type: {expect_ty}, but got {actual_ty}")]
    ImConsistentArrayType {
        expect_ty: String,
        actual_ty: String,
        span: Span,
    },
}

impl SemanticError {
    pub fn span(&self) -> Span {
        match self {
            SemanticError::UndefinedVariable { span, .. } => span.clone(),
            SemanticError::ImConsistentArrayType { span, .. } => span.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Semantic {
    pub table: SymbolTable,
    pub ident_range: IdentRangeLapper,
}

impl Semantic {}

pub struct Function {
    pub name: String,
    pub params: Vec<Type>,
}

#[derive(Debug)]
pub struct Ctx {
    env: im_rc::Vector<(String, Span)>,
    table: SymbolTable,
}

impl Ctx {
    fn find_symbol(&self, name: &str) -> Option<Span> {
        self.env
            .iter()
            .rev()
            .find_map(|(n, t)| if n == name { Some(t.clone()) } else { None })
    }
}

pub fn analyze_program(ast: &Ast) -> Result<Semantic> {
    let table = SymbolTable::default();
    let env = im_rc::Vector::new();
    let mut ctx = Ctx { env, table };
    for (func, _) in ast.iter() {
        let name = func.name.0.clone();
        ctx.env.push_back((name, func.name.1.clone()));
        ctx.table.add_symbol(func.name.1.clone());
        analyze_function(func, &mut ctx)?;
    }
    let mut ident_range = IdentRangeLapper::new(vec![]);
    for (symbol_id, range) in ctx.table.symbol_id_to_span.iter_enumerated() {
        ident_range.insert(Interval {
            start: range.start,
            stop: range.end,
            val: IdentType::Binding(symbol_id),
        });
    }
    for (reference_id, reference) in ctx.table.reference_id_to_reference.iter_enumerated() {
        let range = &reference.span;
        ident_range.insert(Interval {
            start: range.start,
            stop: range.end,
            val: IdentType::Reference(reference_id),
        });
    }
    Ok(Semantic {
        table: ctx.table,
        ident_range,
    })
}

fn analyze_function(func: &Func, ctx: &mut Ctx) -> Result<()> {
    analyze_expr(&func.body.0, ctx)
}

fn analyze_expr(expr: &Expr, ctx: &mut Ctx) -> Result<()> {
    match expr {
        Expr::Error => {}
        Expr::Value(_) => {}
        Expr::List(list) => {
            for item in list.iter() {
                analyze_expr(&item.0, ctx)?;
            }
        }
        Expr::Local(name) => {
            let span = match ctx.find_symbol(&name.0) {
                Some(ty) => ty,
                None => {
                    dbg!(&ctx);
                    return Err(SemanticError::UndefinedVariable {
                        name: name.0.clone(),
                        span: name.1.clone(),
                    });
                }
            };
            let symbol_id = *ctx.table.span_to_symbol_id.get(&span).unwrap();
            ctx.table.add_reference(name.1.clone(), Some(symbol_id));
        }
        Expr::Let(name, rhs, then, name_range) => {
            analyze_expr(&rhs.0, ctx)?;
            ctx.table.add_symbol(name_range.clone());
            ctx.env.push_back((name.clone(), name_range.clone()));
            analyze_expr(&then.0, ctx)?;
            ctx.env.pop_back();
        }
        Expr::Then(first, second) => {
            analyze_expr(&first.0, ctx)?;
            analyze_expr(&second.0, ctx)?
        }
        Expr::Binary(lhs, _, rhs) => {
            analyze_expr(&lhs.0, ctx)?;
            analyze_expr(&rhs.0, ctx)?;
        }
        Expr::Call(callee, args) => {
            analyze_expr(&callee.0, ctx)?;
            for arg in args.0.iter() {
                analyze_expr(&arg.0, ctx)?;
            }
        }
        Expr::If(cond, consequent, alternative) => {
            analyze_expr(&cond.0, ctx)?;
            analyze_expr(&consequent.0, ctx)?;
            analyze_expr(&alternative.0, ctx)?;
        }
        Expr::Print(_) => {}
    };
    Ok(())
}
