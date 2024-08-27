use derivative::Derivative;

use crate::generated::{Cst, Node, NodeRef};
use crate::parser::Span;

// This is a simple macro named `say_hello`.
macro_rules! syntax_node_define {
    // `()` indicates that the macro takes no argument.
    ($t:tt) => {
        #[derive(Derivative)]
        #[derivative(Debug, Clone)]
        pub struct $t<'a> {
            #[derivative(Debug = "ignore")]
            cst: &'a Cst<'a>,
            children: Vec<NodeRef>,
            span: Span,
        }
    };
}
syntax_node_define!(File);

#[derive(Derivative)]
#[derivative(Debug, Clone)]
enum FnOrStruct<'a> {
    Fn(Fn<'a>),
    Struct(Struct<'a>),
}

syntax_node_define!(Fn);

syntax_node_define!(ParamList);

syntax_node_define!(Params);

syntax_node_define!(Param);

syntax_node_define!(TypeExpr);

syntax_node_define!(ReturnType);

syntax_node_define!(Block);

#[derive(Derivative)]
#[derivative(Debug, Clone)]
enum Stmt<'a> {
    Expr(StmtExpr<'a>),
    Let(StmtLet<'a>),
    Block(BlockStmt<'a>),
    ReturnStmt(StmtReturn<'a>),
}

syntax_node_define!(StmtExpr);

syntax_node_define!(StmtLet);

syntax_node_define!(BlockStmt);

syntax_node_define!(StmtReturn);

syntax_node_define!(StmtIf);

#[derive(Derivative)]
#[derivative(Debug, Clone)]
enum Expr<'a> {
    MemberExpr(MemberExpr<'a>),
    StructConstructor(StructConstructor<'a>),
    CallExpr(CallExpr<'a>),
    NameExpr(ExprName<'a>),
    LitExpr(ExprLiteral<'a>),
    ParenExpr(ParenExpr<'a>),
}

syntax_node_define!(MemberExpr);

syntax_node_define!(StructConstructor);

syntax_node_define!(CallExpr);

syntax_node_define!(ExprName);

syntax_node_define!(ExprLiteral);

syntax_node_define!(ParenExpr);

syntax_node_define!(Struct);

syntax_node_define!(StructField);

#[derive(Default, Clone, Debug)]
pub enum SyntaxNode<'a> {
    Token(Span),
    #[default]
    Empty,
    Error(Span),
    File(File<'a>),
    Fn(Fn<'a>),
    ParamList(ParamList<'a>),
    Params(Params<'a>),
    Param(Param<'a>),
    ReturnType(ReturnType<'a>),
    TypeExpr(TypeExpr<'a>),
    Block(Block<'a>),
    Struct(Struct<'a>),
    StructField(StructField<'a>),
    Stmt(Stmt<'a>),
    StmtExpr(StmtExpr<'a>),
    StmtLet(StmtLet<'a>),
    StmtReturn(StmtReturn<'a>),
    StmtIf(StmtIf<'a>),
    // Alter,
    // Consequent,
    Expr(Expr<'a>),
    // Factor,
    // Prime,
    ExprCall(CallExpr<'a>),
    StructConstructor(StructConstructor<'a>),
    // StructConstructorField,
    // ArgList(),
    // Arguments,
    ExprLiteral(ExprLiteral<'a>),
    ExprName(ExprName<'a>),
    MemberExpr(MemberExpr<'a>),
    ExprParen(ParenExpr<'a>),
}

pub fn init_syntax_tree<'a>(cst: &'a Cst<'a>) -> Vec<SyntaxNode<'a>> {
    let mut vec = vec![SyntaxNode::default(); cst.nodes.len()];
    to_syntax_tree(cst, NodeRef(0), &mut vec);
    vec
}

pub fn to_syntax_tree<'a>(
    cst: &'a Cst<'a>,
    index: NodeRef,
    syntax_nodes: &mut Vec<SyntaxNode<'a>>,
) {
    let node = cst.get(index);
    let Some(span) = cst.get_span(index) else {
        return;
    };

    let children_iter = cst.children(index);
    let mut children = vec![];
    for child in children_iter {
        let child_node = cst.get(child);
        match child_node {
            Node::Rule(_, _) => {}
            Node::Token(token_idx) => match cst.tokens[token_idx] {
                crate::parser::Token::Whitespace => continue,
                _ => {}
            },
        }
        children.push(child);
        to_syntax_tree(cst, child, syntax_nodes);
    }
    let syntax_node = match node {
        Node::Rule(rule, offset) => match rule {
            crate::generated::Rule::Error => SyntaxNode::Error(span),
            crate::generated::Rule::File => {
                let file = File {
                    cst,
                    children,
                    span,
                };
                SyntaxNode::File(file)
            }
            crate::generated::Rule::Fn => {
                let r#fn = Fn {
                    cst,
                    children,
                    span,
                };
                SyntaxNode::Fn(r#fn)
            }
            crate::generated::Rule::ParamList => SyntaxNode::ParamList(ParamList {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::Params => SyntaxNode::Params(Params {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::Param => SyntaxNode::Param(Param {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::ReturnType => SyntaxNode::ReturnType(ReturnType {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::TypeExpr => SyntaxNode::TypeExpr(TypeExpr {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::Block => SyntaxNode::Block(Block {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::Struct => SyntaxNode::Struct(Struct {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::StructField => SyntaxNode::StructField(StructField {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::Stmt => todo!(),
            crate::generated::Rule::StmtExpr => SyntaxNode::StmtExpr(StmtExpr {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::StmtLet => SyntaxNode::StmtLet(StmtLet {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::StmtReturn => todo!(),
            crate::generated::Rule::StmtIf => todo!(),
            crate::generated::Rule::Alter => todo!(),
            crate::generated::Rule::Consequent => todo!(),
            crate::generated::Rule::Expr => todo!(),
            crate::generated::Rule::Factor => todo!(),
            crate::generated::Rule::Prime => todo!(),
            crate::generated::Rule::ExprCall => todo!(),
            crate::generated::Rule::StructConstructor => todo!(),
            crate::generated::Rule::StructConstructorField => todo!(),
            crate::generated::Rule::ArgList => todo!(),
            crate::generated::Rule::Arguments => todo!(),
            crate::generated::Rule::ExprLiteral => SyntaxNode::ExprLiteral(ExprLiteral {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::ExprName => SyntaxNode::ExprName(ExprName {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::MemberExpr => SyntaxNode::MemberExpr(MemberExpr {
                cst,
                children,
                span,
            }),
            crate::generated::Rule::ExprParen => todo!(),
        },
        Node::Token(token_index) => SyntaxNode::Token(span),
    };
    syntax_nodes[index.0] = syntax_node;
}
