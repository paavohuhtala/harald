mod ast;
pub mod eval;
pub mod parser;

pub fn compile_script(script: &str) -> eval::CompiledScript {
    let (_input, parsed) = parser::parse_program(script).unwrap();
    // TODO: Warn about unused input?
    eval::compile_script(parsed)
}
