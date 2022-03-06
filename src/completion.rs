use std::collections::HashMap;

use crate::chumsky::{Expr, Func, Spanned};
/// return (need_to_continue_search, founded reference)
pub fn completion(ast: &HashMap<String, Func>, ident_offset: usize) -> Vec<Spanned<String>> {
    let mut vector = Vec::new();
    for (_, v) in ast.iter() {
        if v.name.1.end < ident_offset {
            vector.push(v.name.clone());
        }
    }

    log::debug!("this is completion from body offset {}", ident_offset);
    for (name, v) in ast.iter() {
        if v.span.end > ident_offset && v.span.start < ident_offset {
            log::debug!("this is completion from body {}", name);
            vector.extend(v.args.clone());
            get_completion_of(&v.body, &mut vector, ident_offset);
        }
    }
    vector
}

pub fn get_completion_of(
    expr: &Spanned<Expr>,
    definition_ass_list: &mut Vec<Spanned<String>>,
    ident_offset: usize,
) -> bool {
    match &expr.0 {
        Expr::Error => true,
        Expr::Value(_) => true,
        // Expr::List(exprs) => exprs
        //     .iter()
        //     .for_each(|expr| get_definition(expr, definition_ass_list)),
        Expr::Local(local) => {
            if ident_offset >= local.1.start && ident_offset < local.1.end {
                false
            } else {
                true
            }
        }
        Expr::Let(name, lhs, rest, name_span) => {
            definition_ass_list.push((name.clone(), name_span.clone()));
            match get_completion_of(lhs, definition_ass_list, ident_offset) {
                true => get_completion_of(rest, definition_ass_list, ident_offset),
                false => return false,
            }
        }
        Expr::Then(first, second) => {
            match get_completion_of(first, definition_ass_list, ident_offset) {
                true => get_completion_of(second, definition_ass_list, ident_offset),
                false => false,
            }
        }
        Expr::Binary(lhs, op, rhs) => {
            match get_completion_of(lhs, definition_ass_list, ident_offset) {
                true => get_completion_of(rhs, definition_ass_list, ident_offset),
                false => false,
            }
        }
        Expr::Call(callee, args) => {
            match get_completion_of(callee, definition_ass_list, ident_offset) {
                true => {}
                false => return false,
            }
            for expr in &args.0 {
                match get_completion_of(&expr, definition_ass_list, ident_offset) {
                    true => continue,
                    false => return false,
                }
            }
            true
        }
        Expr::If(test, consequent, alternative) => {
            match get_completion_of(test, definition_ass_list, ident_offset) {
                true => {}
                false => return false,
            }
            match get_completion_of(consequent, definition_ass_list, ident_offset) {
                true => {}
                false => return false,
            }
            get_completion_of(alternative, definition_ass_list, ident_offset)
        }
        Expr::Print(expr) => get_completion_of(expr, definition_ass_list, ident_offset),
        Expr::List(lst) => {
            for expr in lst {
                match get_completion_of(expr, definition_ass_list, ident_offset) {
                    true => continue,
                    false => return false,
                }
            }
            true
        }
    }
}
