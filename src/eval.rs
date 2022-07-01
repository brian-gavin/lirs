use std::{cell::RefCell, rc::Rc};

use crate::{
    env::{global_env, Env},
    types::{Atom, List, Procedure},
    Expr,
};

#[derive(Debug)]
pub struct Evaluator {
    global_env: Rc<RefCell<Env>>,
}

impl Evaluator {
    pub fn new() -> Evaluator {
        let global_env = Rc::new(RefCell::new(global_env()));
        Evaluator { global_env }
    }

    pub fn eval(&self, e: Expr) -> Result<Expr, String> {
        eval_env(e, &self.global_env)
    }
}

pub fn eval_env(e: Expr, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    use crate::types::Atom::*;
    use Expr::*;
    // let undefined = |s: &str| format!("undefined symbol: {}", s);
    match e {
        // procedure call
        List(list) => {
            let s = match list.first().ok_or("Expected nonempty list")? {
                Atom(Symbol(s)) => s,
                _ => return Err("Expected symbol as head of list".into()),
            };
            // f must be called *after* the borrow on env is dropped.
            let f = match env.borrow().get(s) {
                Some(e) => e,
                None => return Err(format!("undefined: {}", s)),
            };
            eval_call(f, list, env)
        }
        // constant atom symbol
        Atom(Symbol(ref s)) if s.starts_with("'") => Ok(e),
        // other constants
        Atom(Number(_)) | Builtin(_) | Procedure(_) => Ok(e),
        // variable reference
        Atom(Symbol(ref s)) => match env.borrow().get(s) {
            Some(e) => Ok(e.clone()),
            None => Err(format!("undefined: {}", s)),
        },
    }
}

fn eval_call(e: Expr, list: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    match e {
        Expr::Builtin(f) => f.0(list, env),
        Expr::Procedure(p) => {
            let args = list
                .into_iter()
                .skip(1)
                .map(|e| eval_env(e, env))
                .collect::<Result<List, String>>()?;
            let env = Rc::new(RefCell::new({
                let mut env = Env::new(p.capture.clone());
                for (k, v) in p.params.into_iter().zip(args) {
                    env.insert(k, v);
                }
                env
            }));
            eval_env(*p.body.clone(), &env)
        }
        _ => Err(format!("'{}' is not callable", e)),
    }
}

pub fn eval_times(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    use crate::types::Atom::Number;
    use Expr::Atom;
    Ok(l.into_iter()
        .skip(1)
        .map(|e| {
            eval_env(e, env).and_then(|e| match e {
                Atom(Number(n)) => Ok(n),
                _ => Err("*: all exprs must evaluate to numbers".to_string()),
            })
        })
        .collect::<Result<Vec<f64>, String>>()?
        .into_iter()
        .reduce(|accum, e| accum * e)
        .map(|n| Expr::of(n))
        .unwrap_or(Expr::of(1.)))
}

pub fn eval_begin(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    l.into_iter()
        .skip(1)
        .map(|e| eval_env(e, &env))
        .last()
        .unwrap_or(Ok(Expr::of(0.)))
}

pub fn eval_define(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    let mut l = l.into_iter().skip(1);
    let symbol = match l
        .next()
        .ok_or_else(|| "(define symbol value): expected symbol")?
    {
        Expr::Atom(Atom::Symbol(s)) => s,
        _ => return Err("(define symbol value): symbol must be a symbol.".to_string()),
    };
    let value = eval_env(
        l.next()
            .ok_or_else(|| "(define symbol value): expected value")?,
        env,
    )?;
    env.borrow_mut().insert(symbol, value);
    Ok(Expr::of(String::new()))
}

pub fn eval_if(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    let mut l = l.into_iter().skip(1);
    let test = eval_env(l.next().ok_or("(if test conseq alt): expected test")?, env)?;
    let conseq = l.next().ok_or("(if test conseq alt): expected conseq")?;
    let alt = l.next().ok_or("(if test conseq alt): expected alt")?;
    if test.truthy() {
        eval_env(conseq, env)
    } else {
        eval_env(alt, env)
    }
}

pub fn eval_eq(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    use crate::types::Atom::*;
    use Expr::*;
    let mut l = l
        .into_iter()
        .skip(1)
        .map(|e| {
            eval_env(e, env).and_then(|e| match e {
                Atom(Number(n)) => Ok(n),
                _ => Err("(=) all expressions must be numbers.".to_string()),
            })
        })
        .collect::<Result<Vec<f64>, String>>()?
        .into_iter();
    let first = l.next().unwrap_or(0.);
    if l.all(|n| n == first) {
        Ok(Expr::tru())
    } else {
        Ok(Expr::fals())
    }
}

pub fn eval_lambda(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    use crate::types::Atom::Symbol;
    let report = |s| move || format!("(lambda (<params>) expr): {}", s);
    let mut l = l.into_iter().skip(1);
    let params = match l.next().ok_or_else(report("expected params"))? {
        Expr::List(l) => l,
        _ => return Err(report("params must be a list.")()),
    };
    let params = params
        .into_iter()
        .map(|p| match p {
            Expr::Atom(Symbol(s)) => Some(s),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(report("params must be a list of symbols"))?;
    let body = l.next().ok_or_else(report("expected expression"))?;
    Ok(Expr::Procedure(Procedure::new(body, params, env.clone())))
}

pub fn eval_sub(l: List, env: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    let report = |s| format!("(-): {}", s);
    use crate::types::Atom::Number;
    let diff = l
        .into_iter()
        .skip(1)
        .map(|e| {
            eval_env(e, env).and_then(|e| match e {
                Expr::Atom(Number(n)) => Ok(n),
                _ => Err(report("all expressions must be numbers.")),
            })
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .reduce(|accum, n| accum - n)
        .unwrap_or(0.);
    Ok(Expr::Atom(Number(diff)))
}

pub fn eval_quote(l: List, _: &Rc<RefCell<Env>>) -> Result<Expr, String> {
    let expr = l
        .into_iter()
        .skip(1)
        .next()
        .ok_or_else(|| "(quote expr): expected expression")?;
    Ok(expr)
}
