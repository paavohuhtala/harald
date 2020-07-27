use harald::run_script;

#[test]
fn table_dict_holes() {
  let output = run_script(include_str!("./table_dict_holes.hd")).unwrap();
  assert_eq!(output, "a b");
}
