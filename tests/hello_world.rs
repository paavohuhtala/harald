use harald_lang::compile_script;

#[test]
fn test_hello_world() {
    let script = compile_script(include_str!("./hello_world.hd"));
    let mut output = String::new();
    script.run(&mut output);
    assert_eq!(output, "Hello, world!");
}
