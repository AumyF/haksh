use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use haksh::parser::parse_line;

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    loop {
        let readline = rl.readline("haksh >> ");
        match readline {
            Ok(line) => {
                let result = parse_line(&line);
                match result {
                    Ok(t) => {
                        println!("Parsed: {:?}", t);
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                    },
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
