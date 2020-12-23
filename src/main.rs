use std::{fs, io::Write, path::PathBuf};

use harald::eval::{CompiledScript, NameHint};
use harald::{
  compile_script,
  parser::{parse_expression, parse_statement},
};

fn run_file(path: &str) -> Result<(), anyhow::Error> {
  let path = PathBuf::from(path);
  let source = fs::read_to_string(path)?;

  let script = compile_script(&source)?;

  for _ in 0..10 {
    let output = script.run()?;
    println!("{}", output);
  }

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
      ":q" | ":exit" => {
        break;
      }
      statement if command.ends_with(";") => {
        let statement = parse_statement(statement);
        match statement {
          Err(err) => {
            println!("Invalid statement: {}", err.to_string());
            continue;
          }
          Ok((input, statement)) => {
            if input.len() > 0 {
              println!("WARNING: Unprocessed input ({})", input);
            }

            match script.add_statement(statement) {
              Err(err) => {
                println!("Failed to compile statement: {}", err);
                continue;
              }
              Ok(()) => println!("OK"),
            }
          }
        }
      }
      expression => {
        let expression = parse_expression(expression);

        match expression {
          Err(err) => {
            println!("Invalid expression: {}", err.to_string());
            continue;
          }
          Ok((input, expression)) => {
            if input.len() > 0 {
              println!("WARNING: Unprocessed input ({})", input);
            }

            let name_hint = Some(NameHint::Repl);
            let expression = script.transform_expression(expression, &name_hint);

            match expression {
              Err(err) => {
                println!("Failed to compile expression: {}", err.to_string());
                continue;
              }
              Ok(expression) => match script.eval_expression(&expression) {
                Ok(result) => {
                  println!("< {}", result);
                }
                Err(err) => {
                  println!("Interpreter error: {}", err);
                }
              },
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
