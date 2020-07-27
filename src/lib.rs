mod ast;
pub mod eval;
pub mod parser;

pub fn compile_script(script: &str) -> Result<eval::CompiledScript, eval::CompilerError> {
    let (_, parsed) = parser::parse_program(script).unwrap();
    eval::compile_script(parsed)
}

pub fn run_script(script: &str) -> Result<String, eval::ExecutionError> {
    let script = compile_script(script).unwrap();
    Ok(script.run()?)
}
