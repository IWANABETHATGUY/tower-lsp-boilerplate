use std::collections::HashMap;

use chumsky::Span;
use im_rc::Vector;

use crate::chumsky::{Expr, Func, FuncOrStruct, Identifier, Spanned};
#[derive(Debug, Clone)]
pub enum ReferenceSymbol {
    Founded(Identifier),
    Founding(usize),
}
use ReferenceSymbol::*;
pub fn get_reference(
    ast: &HashMap<String, FuncOrStruct>,
    ident_offset: usize,
    include_self: bool,
) -> Vec<Spanned<String>> {
    vec![]
}

pub fn get_reference_of_expr(
    expr: &Spanned<Expr>,
    definition_ass_list: Vector<Spanned<String>>,
    reference_symbol: ReferenceSymbol,
    reference_list: &mut Vec<Spanned<String>>,
    include_self: bool,
) {
    match &expr.0 {
        Expr::Error => {}
        Expr::Value(_) => {}
        Expr::Local((name, span)) => {
            if let Founded((symbol_name, symbol_span)) = reference_symbol {
                if &symbol_name == name {
                    let index = definition_ass_list
                        .iter()
                        .position(|decl| decl.0 == symbol_name);
                    if let Some(symbol) = index.map(|i| definition_ass_list.get(i).unwrap().clone())
                    {
                        if symbol == (symbol_name, symbol_span) {
                            reference_list.push((name.clone(), span.clone()));
                        }
                    };
                }
            }
            // if ident_offset >= local.1.start && ident_offset < local.1.end {
            //     let index = definition_ass_list
            //         .iter()
            //         .position(|decl| decl.0 == local.0);
            //     (
            //         false,
            //         index.map(|i| definition_ass_list.get(i).unwrap().clone()),
            //     )
            // } else {
            //     (true, None)
            // }
        }
        Expr::Let(name, lhs, rest, name_span) => {
            let new_decl = Vector::unit((name.clone(), name_span.clone()));
            let next_symbol = match reference_symbol {
                Founding(ident) if ident >= name_span.start && ident < name_span.end => {
                    let spanned_name = (name.clone(), name_span.clone());
                    if include_self {
                        reference_list.push(spanned_name.clone());
                    }
                    ReferenceSymbol::Founded(spanned_name)
                }
                _ => reference_symbol,
            };

            get_reference_of_expr(
                lhs,
                definition_ass_list.clone(),
                next_symbol.clone(),
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                rest,
                new_decl + definition_ass_list,
                next_symbol,
                reference_list,
                include_self,
            );
        }
        Expr::Then(first, second) => {
            get_reference_of_expr(
                first,
                definition_ass_list.clone(),
                reference_symbol.clone(),
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                second,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
        }
        Expr::Binary(lhs, _op, rhs) => {
            get_reference_of_expr(
                lhs,
                definition_ass_list.clone(),
                reference_symbol.clone(),
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                rhs,
                definition_ass_list,
                reference_symbol,
                reference_list,
                include_self,
            );
        }
        Expr::Call(callee, args) => {
            get_reference_of_expr(
                callee,
                definition_ass_list.clone(),
                reference_symbol.clone(),
                reference_list,
                include_self,
            );
            for expr in &args.0 {
                get_reference_of_expr(
                    expr,
                    definition_ass_list.clone(),
                    reference_symbol.clone(),
                    reference_list,
                    include_self,
                );
            }
        }
        Expr::If(test, consequent, alternative) => {
            get_reference_of_expr(
                test,
                definition_ass_list.clone(),
                reference_symbol.clone(),
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                consequent,
                definition_ass_list.clone(),
                reference_symbol.clone(),
                reference_list,
                include_self,
            );
            get_reference_of_expr(
                alternative,
                definition_ass_list,
                reference_symbol.clone(),
                reference_list,
                include_self,
            );
        }
        Expr::Print(expr) => get_reference_of_expr(
            expr,
            definition_ass_list,
            reference_symbol,
            reference_list,
            include_self,
        ),
        Expr::List(lst) => {
            for expr in lst {
                get_reference_of_expr(
                    expr,
                    definition_ass_list.clone(),
                    reference_symbol.clone(),
                    reference_list,
                    include_self,
                );
            }
        }
    }
}
