pub mod kind;

use internment::Intern;

use crate::utils::span::{Span, Spanned};

pub enum Declaraction<'decl> {
    Function {
        name: Intern<String>,
        args: &'decl[Intern<String>],
        body: &'decl Expression<'decl>,
        span: Span,
    }
}

impl Spanned for Declaraction<'_> {
    fn span(&self) -> Span {
        match self {
            Declaraction::Function { span, .. } => *span,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression<'exp> {
    Block {
        body: &'exp[Expression<'exp>],
        span: Span,
    },
    Arithmetic {
        lhs: &'exp Expression<'exp>,
        rhs: &'exp Expression<'exp>,
        span: Span,
        kind: kind::Arithmetic,
    },
    MatchExpression {
        cond: &'exp Expression<'exp>,
        arms: &'exp[(Literal, Expression<'exp>)],
        span: Span,
    },
    Call {
        name: Intern<String>,
        args: &'exp[Expression<'exp>],
        span: Span,
    }
}

impl Spanned for Expression<'_> {
    fn span(&self) -> Span {
        match self {
            Expression::Block { span, .. } => *span,
            Expression::Arithmetic { span, .. } => *span,
            Expression::MatchExpression { span, .. } => *span,
            Expression::Call { span, .. } => *span,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Literal {
    Number {
        value: f64,
        span: Span,
    },
    String {
        value: Intern<String>,
        span: Span,
    },
    Boolean {
        value: bool,
        span: Span,
    },
}

impl Spanned for Literal {
    fn span(&self) -> Span {
        match self {
            Literal::Number { span, .. } => *span,
            Literal::String { span, .. } => *span,
            Literal::Boolean { span, .. } => *span,
        }
    }
}