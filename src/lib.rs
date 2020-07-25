mod ast;
pub mod eval;
mod parser;

pub fn compile_script(script: &str) -> eval::CompiledScript {
    let (_, parsed) = parser::parse_program(script).unwrap();
    eval::compile_script(parsed)
}
