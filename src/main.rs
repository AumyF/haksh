use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use haksh::interpreter::Environment;
use haksh::parser::{parse_file, parse_line};

fn repl() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    let mut env = Environment::new();
    loop {
        let readline = rl.readline("haksh >> ");
        match readline {
            Ok(line) => {
                let result = parse_line(&line);
                match result {
                    Ok(t) => {
                        println!("Parsed: {:?}", t);

                        match t.1.evaluate_for_repl(&env) {
                            Ok((new_env, value)) => {
                                env = new_env;
                                println!("{:?}", value);
                            }
                            Err(e) => println!("Error: {e}"),
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct InterpretError {
    msg: String,
}
impl std::fmt::Display for InterpretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InterpretError: {}", self.msg)
    }
}
impl std::error::Error for InterpretError {}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let file = args.get(1);

    match file {
        Some(file) => {
            let file = std::fs::read_to_string(file).unwrap();
            let (_, file) = parse_file(&file).unwrap();
            let env = Environment::new();
            let _ = file
                .evaluate(&env)
                .map_err(|msg| Box::new(InterpretError { msg }))?;

            Ok(())
        }
        None => Ok(repl().map_err(Box::new)?),
    }
}
