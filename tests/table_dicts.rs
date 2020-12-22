use harald::run_script;

#[test]
fn table_holes() {
  let output = run_script(include_str!("./table_holes.hd")).unwrap();
  assert_eq!(output, "a b");
}
