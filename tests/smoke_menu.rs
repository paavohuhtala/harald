use harald::compile_script;

#[test]
fn menu_sample_1000() {
  let script = compile_script(include_str!("../programs/menu.hd")).unwrap();

  for _ in 0..1000 {
    script.run().unwrap();
  }
}
