pub mod kind;

use internment::Intern;

use crate::utils::span::{Span, Spanned};

pub type Program<'p> = Vec<Declaraction<'p>>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Declaraction<'decl> {
    pub kind: DeclaractionKind<'decl>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeclaractionKind<'decl> {
    Function {
        name: Intern<String>,
        args: &'decl [Intern<String>],
        body: &'decl Expression<'decl>,
        span: Span,
    },
}

impl Spanned for Declaraction<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Expression<'exp> {
    pub kind: ExpressionKind<'exp>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExpressionKind<'exp> {
    Block {
        body: &'exp [Declaraction<'exp>],
    },
    Arithmetic {
        lhs: &'exp Expression<'exp>,
        rhs: &'exp Expression<'exp>,
        kind: kind::Arithmetic,
    },
    MatchExpression {
        cond: &'exp Expression<'exp>,
        arms: &'exp [(Literal, Expression<'exp>)],
    },
    Call {
        name: Intern<String>,
        args: &'exp [Expression<'exp>],
    },
}

impl Spanned for Expression<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LiteralKind {
    Number(f64),
    String(Intern<String>),
    Boolean(bool),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Literal {
    kind: LiteralKind,
    pub span: Span,
}

impl Spanned for Literal {
    fn span(&self) -> Span {
        self.span
    }
}
