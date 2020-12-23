mod ast;
pub mod eval;
mod nom_unicode;
pub mod parser;
pub mod string_utils;

pub fn compile_script(script: &str) -> Result<eval::CompiledScript, eval::CompilerError> {
  match parser::parse_program(script) {
    Err(nom::Err::Error(err)) => panic!("Parse error: {}", nom::error::convert_error(script, err)),
    Err(err) => panic!("Parse error: {}", err),
    Ok((_, parsed)) => return eval::compile_script(parsed),
  }
}

pub fn run_script(script: &str) -> Result<String, eval::ExecutionError> {
  let script = compile_script(script)?;
  Ok(script.run()?)
}
