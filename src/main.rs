use std::{error::Error, io::stdin};

use lirs::{parse, Evaluator};
fn main() -> Result<(), Box<dyn Error>> {
    let stdin = stdin();
    let eval = Evaluator::new();
    for line in stdin.lines() {
        let line = line?;
        let e = match parse(&line) {
            Ok(exp) => exp,
            Err(e) => {
                eprintln!("Parse Error: {}", e);
                continue;
            }
        };
        let e = match eval.eval(e) {
            Ok(exp) => exp,
            Err(e) => {
                eprintln!("Eval Error: {}", e);
                continue;
            }
        };
        println!("Result: {}", e);
    }
    Ok(())
}
