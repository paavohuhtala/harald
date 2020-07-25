use harald::compile_script;

#[test]
fn test_hello_world() {
    let script = compile_script(include_str!("./hello_world.hd"));
    let mut output = String::new();
    script.run(&mut output).unwrap();
    assert_eq!(output, "Hello, world!");
}

#[test]
fn test_hello_world_pattern() {
    let script = compile_script(include_str!("./hello_pattern.hd"));
    let mut output = String::new();
    script.run(&mut output).unwrap();
    assert_eq!(output, "Hello, world!");
}
