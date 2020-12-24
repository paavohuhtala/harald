use harald::{
  eval::{CompilerError, ExecutionError},
  run_script,
};

use matches::assert_matches;

#[test]
fn table_holes() {
  let output = run_script(include_str!("./table_holes.hd")).unwrap();
  assert_eq!(output, "a b");
}

#[test]
fn table_trailing_comma() {
  let output = run_script(include_str!("./table_trailing.hd")).unwrap();
  assert_eq!(output, "a b");
}

#[test]
fn table_weight() {
  let output = run_script(include_str!("./table_weight.hd")).unwrap();
  assert_eq!(output, "a a");
}

#[test]
fn empty_table() {
  let output = run_script(include_str!("./empty_table.hd"));
  assert_matches!(
    output,
    Err(ExecutionError::Compiler(CompilerError::EmptyTableColumn {
      column_name: column,
      in_variable: variable
    })) if column == "hasNoEntries" && variable == "testTable"
  );
}
