use std::{fs, io::Write, path::PathBuf};

use harald::eval::CompiledScript;
use harald::{
    compile_script,
    parser::{parse_expression, parse_statement},
};

fn run_file(path: &str) -> Result<(), anyhow::Error> {
    let path = PathBuf::from(path);
    let source = fs::read_to_string(path)?;

    let script = compile_script(&source);

    let mut output = String::new();
    script.run(&mut output)?;
    println!("{}", output);

    Ok(())
}

fn run_repl() -> Result<(), anyhow::Error> {
    let stdin = std::io::stdin();
    let mut buffer = String::new();

    let mut script = CompiledScript::new();

    println!("harald REPL");

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

fn main() -> Result<(), anyhow::Error> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    let file_path = args.get(0);

    match file_path {
        Some(file_path) => run_file(&file_path)?,
        None => run_repl()?,
    };

    Ok(())
}
