use std::rc::Rc;

use crate::*;

macro_rules! define_ast {
    ($root_name:ident{$($sub:ident{$($key:ident: $value:ty),*}),+}) => {
        paste::paste! {
            #[derive(Clone, Debug)]
            pub enum $root_name {$($sub {$($key: $value,)*},)*}
            impl $root_name {
                $(pub fn [<$sub:lower>]($($key: $value,)*) -> Box<Self> {
                    Box::new(Self::$sub {$($key,)*})
                })*
            }
        }
    };
}

define_ast!(
    Expr {
        Binary {
            left: Box<Expr>,
            operator: Token,
            right: Box<Expr>
        },
        Call {
            callee: Box<Expr>,
            paren: Token,
            args: Vec<Box<Expr>>
        },
        Grouping {
            expression: Box<Expr>
        },
        Literal {
            value: ValueWrapper
        },
        Logical {
            left: Box<Expr>,
            operator: Token,
            right: Box<Expr>
        },
        Unary {
            operator: Token,
            right: Box<Expr>
        },
        Variable {
            name: Token
        },
        Assign {
            name: Token,
            value: Box<Expr>
        }
    }
);

define_ast!(
    Stmt {
        Block {
            statements: Vec<Stmt>
        },
        Expression {
            expression: Box<Expr>
        },
        BreakStmt {
            location: Token
        },
        Var {
            name: Token,
            inititalizer: Option<Box<Expr>>
        },
        Print {
            expression: Box<Expr>
        },
        IfStmt {
            condition: Box<Expr>,
            then_branch: Box<Stmt>,
            else_branch: Option<Box<Stmt>>
        },
        WhileStmt {
            condition: Box<Expr>,
            body: Box<Stmt>
        },
        Function {
            instance: Rc<dyn Callable>
        },
        ReturnStmt {
            keyword: Token,
            value: Box<Expr>
        }
    }
);

/*
name: Token,
            params: Vec<Token>,
            body: Box<Stmt>, */
