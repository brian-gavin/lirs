use crate::types::{Atom, Expr};

pub type ParseError = String;
pub type ParseResult = Result<Expr, ParseError>;

pub fn parse(program: &str) -> ParseResult {
    let replace = program.replace('(', " ( ").replace(')', " ) ");
    let mut tokens: Vec<_> = replace.split_whitespace().rev().collect();
    dbg!(&tokens);
    read_from_tokens(&mut tokens)
}

fn read_from_tokens(tokens: &mut Vec<&str>) -> ParseResult {
    let eof = || "Unexpected EOF".to_string();
    match tokens.pop().ok_or_else(eof)? {
        "(" => {
            let mut v = Vec::new();
            while tokens.last().ok_or_else(eof)? != &")" {
                v.push(read_from_tokens(tokens)?);
            }
            let _ = tokens
                .pop()
                .expect("expected there to be ')' in token list.");
            Ok(Expr::List(v.into()))
        }
        ")" => Err("Unexpected ')'".to_string()),
        t => Ok(Expr::of(atom(t))),
    }
}

fn atom(tok: &str) -> Atom {
    number(tok).unwrap_or_else(|| tok.to_string().into())
}

fn number(tok: &str) -> Option<Atom> {
    tok.parse::<f64>().map(|f| f.into()).ok()
}
