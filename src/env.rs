use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    eval::*,
    types::{Builtin, Expr},
};

#[derive(Debug)]
pub struct Env {
    m: HashMap<String, Expr>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub(crate) fn new(outer: Rc<RefCell<Env>>) -> Env {
        Env {
            m: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub(crate) fn get(&self, k: &String) -> Option<Expr> {
        match self.m.get(k) {
            v @ Some(_) => v.cloned(),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(k),
                None => None,
            },
        }
    }

    pub(crate) fn insert(&mut self, k: String, v: Expr) -> Option<Expr> {
        self.m.insert(k, v)
    }

    // pub(crate) fn set(&mut self, k: &String, v: Expr) -> Option<Expr> {
    //     match self.m.get_mut(k) {
    //         Some(x) => Some(replace(x, v)),
    //         None => match &self.outer {
    //             Some(outer) => outer.borrow_mut().set(k, v),
    //             None => None,
    //         },
    //     }
    // }
}

pub(crate) fn global_env() -> Env {
    use std::f64::consts::PI;
    let m = HashMap::from_iter([
        ("pi".into(), Expr::of(PI)),
        ("begin".into(), Expr::Builtin(Builtin(eval_begin))),
        ("define".into(), Expr::Builtin(Builtin(eval_define))),
        ("if".into(), Expr::Builtin(Builtin(eval_if))),
        ("*".into(), Expr::Builtin(Builtin(eval_times))),
        ("-".into(), Expr::Builtin(Builtin(eval_sub))),
        ("=".into(), Expr::Builtin(Builtin(eval_eq))),
        ("lambda".into(), Expr::Builtin(Builtin(eval_lambda))),
        ("quote".into(), Expr::Builtin(Builtin(eval_quote))),
    ]);
    Env { m, outer: None }
}
