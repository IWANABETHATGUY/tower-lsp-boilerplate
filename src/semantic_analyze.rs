use std::fmt::Display;

use crate::{
    chumsky::{Ast, Expr, Func},
    span::Span,
    symbol_table::SymbolTable,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SemanticError>;
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

pub struct Function {
    pub name: String,
    pub params: Vec<Type>,
}

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

pub fn analyze_program(ast: &Ast) -> Result<SymbolTable> {
    let table = SymbolTable::default();
    let env = im_rc::Vector::new();
    let mut ctx = Ctx { env, table };
    for (_, func) in ast.iter() {
        let name = func.name.0.clone();
        ctx.env.push_back((name, func.name.1.clone()));
        ctx.table.add_symbol(func.name.1.clone());
        analyze_function(&func, &mut ctx)?;
    }
    Ok(ctx.table)
}

fn analyze_function(func: &Func, ctx: &mut Ctx) -> Result<()> {
    analyze_expr(&func.body.0, ctx)
}

fn analyze_expr(expr: &Expr, ctx: &mut Ctx) -> Result<()> {
    match expr {
        Expr::Error => todo!(),
        Expr::Value(value) => {}
        Expr::List(list) => {
            for (i, item) in list.iter().enumerate() {
                analyze_expr(&item.0, ctx)?;
            }
        }
        Expr::Local(name) => {
            let span = match ctx.find_symbol(&name.0) {
                Some(ty) => ty,
                None => {
                    return Err(SemanticError::UndefinedVariable {
                        name: name.0.clone(),
                        span: name.1.clone(),
                    })
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
            let _ = analyze_expr(&first.0, ctx)?;
            analyze_expr(&second.0, ctx)?
        }
        Expr::Binary(lhs, _, rhs) => {
            analyze_expr(&lhs.0, ctx)?;
            analyze_expr(&rhs.0, ctx)?;
        }
        Expr::Call(_, _) => todo!(),
        Expr::If(cond, consequent, alternative) => {
            analyze_expr(&cond.0, ctx)?;
            analyze_expr(&consequent.0, ctx)?;
            analyze_expr(&alternative.0, ctx)?;
        }
        Expr::Print(_) => {}
    };
    Ok(())
}
