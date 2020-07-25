use std::io::Write;

use harald::eval::CompiledScript;
use harald::parser::{parse_expression, parse_statement};

fn main() -> Result<(), anyhow::Error> {
    println!("harald REPL");

    let stdin = std::io::stdin();
    let mut buffer = String::new();

    let mut script = CompiledScript::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        buffer.clear();
        stdin.read_line(&mut buffer)?;

        let command = buffer.trim();

        match command {
            ":q" => {
                break;
            }
            statement if command.ends_with(";") => {
                let statement = parse_statement(statement);
                match statement {
                    Err(err) => {
                        println!("Invalid statement: {}", err.to_string());
                    }
                    Ok((input, statement)) => {
                        if input.len() > 0 {
                            println!("WARNING: Unprocessed input ({})", input);
                        }

                        script.add_statement(statement);
                        println!("OK")
                    }
                }
            }
            expression => {
                let expression = parse_expression(expression);

                match expression {
                    Err(err) => {
                        println!("Invalid expression: {}", err.to_string());
                    }
                    Ok((input, expression)) => {
                        if input.len() > 0 {
                            println!("WARNING: Unprocessed input ({})", input);
                        }

                        let expression = script.transform_expression(expression);

                        let mut result = String::new();

                        match script.eval_expression(&expression, &mut result) {
                            Ok(_) => {
                                println!("< {}", result);
                            }
                            Err(err) => {
                                println!("Error: {}", err);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
