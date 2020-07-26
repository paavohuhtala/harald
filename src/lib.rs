mod ast;
pub mod eval;
pub mod parser;

pub fn compile_script(script: &str) -> Result<eval::CompiledScript, eval::CompilerError> {
    let (_input, parsed) = parser::parse_program(script).unwrap();
    // TODO: Warn about unused input?
    eval::compile_script(parsed)
}
