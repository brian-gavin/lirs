use std::{cell::RefCell, fmt, rc::Rc};

use crate::env::Env;

pub type Symbol = String;

#[derive(Debug, Clone)]
pub enum Atom {
    Symbol(Symbol),
    Number(f64),
}

impl From<String> for Atom {
    fn from(s: String) -> Self {
        Atom::Symbol(s)
    }
}

impl From<f64> for Atom {
    fn from(f: f64) -> Self {
        Atom::Number(f)
    }
}

pub type List = Vec<Expr>;

#[derive(Debug, Clone)]
pub enum Expr {
    Atom(Atom),
    List(List),
    Builtin(Builtin),
    Procedure(Procedure),
}

impl Expr {
    pub fn of<T: Into<Atom>>(t: T) -> Expr {
        Expr::Atom(t.into())
    }

    pub fn tru() -> Expr {
        Expr::of("#t".to_string())
    }

    pub fn fals() -> Expr {
        Expr::of("#f".to_string())
    }

    pub fn truthy(&self) -> bool {
        match self {
            Expr::Atom(Atom::Symbol(s)) if s == "#f" => false,
            _ => true,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Atom(Atom::Number(n)) => write!(f, "{}", n),
            Expr::Atom(Atom::Symbol(s)) => write!(f, "{}", s),
            _ => write!(f, ""),
        }
    }
}

#[derive(Clone)]
pub struct Builtin(pub fn(List, &Rc<RefCell<Env>>) -> Result<Expr, String>);

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Builtin").field(&(self.0 as usize)).finish()
    }
}

#[derive(Clone, Debug)]
pub struct Procedure {
    pub(crate) body: Box<Expr>,
    pub(crate) params: Vec<String>,
    pub(crate) capture: Rc<RefCell<Env>>,
}

impl Procedure {
    pub fn new(body: Expr, params: Vec<String>, capture: Rc<RefCell<Env>>) -> Procedure {
        let body = Box::new(body);
        Procedure {
            body,
            params,
            capture,
        }
    }
}
