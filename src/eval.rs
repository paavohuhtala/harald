use crate::{ast, string_utils};
use rand::{
  distributions::{weighted::alias_method::WeightedIndex, WeightedError},
  prelude::Distribution,
};
use std::{borrow::Cow, collections::HashMap, fmt::Display};
use thiserror::Error;

#[derive(Debug)]
pub enum Value<'a> {
  StringV(Cow<'a, str>),
  BagV(&'a Bag),
  TableV(&'a Table),
}

impl<'a> Value<'a> {
  pub fn get_type_name(&self) -> &'static str {
    match self {
      Value::StringV(_) => "string",
      Value::BagV(_) => "bag",
      Value::TableV(_) => "table",
    }
  }

  pub fn try_as_string(self) -> Result<Cow<'a, str>, InterpreterError> {
    match self {
      Value::StringV(s) => Ok(s),
      otherwise => Err(InterpreterError::UnexpectedType {
        expected: "string",
        was: otherwise.get_type_name(),
      }),
    }
  }
}

impl<'a> Display for Value<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Value::StringV(v) => f.write_str(v),
      Value::BagV(bag) => {
        if let Some(name_hint) = &bag.name_hint {
          write!(f, "bag ({:?})", name_hint)
        } else {
          write!(f, "bag (anonymous)")
        }
      }
      Value::TableV(table) => {
        if let Some(name_hint) = &table.name_hint {
          write!(f, "table ({:?})", name_hint)
        } else {
          write!(f, "table (anonymous)")
        }
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Pattern {
  parts: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Table {
  name_hint: Option<NameHint>,
  bags: HashMap<String, Bag>,
}

#[derive(Debug, Clone)]
pub struct Bag {
  id: usize,
  name_hint: Option<NameHint>,
  items: Vec<Expression>,
  distribution: WeightedIndex<f32>,
}

#[derive(Debug, Clone)]
pub enum Expression {
  LiteralE(String),
  VariableE(String),
  PatternE(Pattern),
  BagE(Bag),
  TableE(Table),
  PropertyAccessE(Box<Expression>, String),
  CallE(BuiltInFunction, Vec<Expression>),
}

#[derive(Debug, Clone)]
pub enum BuiltInFunction {
  UpperFirst,
  MaybePrepend,
  MaybeAppend,
}

impl BuiltInFunction {
  pub fn try_parse(s: &str) -> Option<BuiltInFunction> {
    match s {
      "capitalise" => Some(BuiltInFunction::UpperFirst),
      "maybePrepend" => Some(BuiltInFunction::MaybePrepend),
      "maybeAppend" => Some(BuiltInFunction::MaybeAppend),
      _ => None,
    }
  }
}

#[derive(Debug, Clone)]
pub enum FunctionLike {
  BuiltIn(BuiltInFunction),
}

#[derive(Error, Debug)]
pub enum FunctionError {
  #[error("Expected argument #{n} to be {expected}, was {was}")]
  UnexpectedArgumentType {
    n: u8,
    expected: &'static str,
    was: &'static str,
  },
  #[error("Expected {expected} arguments, received {was}")]
  WrongNumberOfArguments { expected: u8, was: u8 },
}

#[derive(Error, Debug)]
pub enum InterpreterError {
  #[error("Unknown variable: {0}")]
  UnknownVariable(String),
  #[error("Unexpected type: expected {expected}, was {was}")]
  UnexpectedType {
    expected: &'static str,
    was: &'static str,
  },
  #[error("Value of type {was} cannot be coerced to {target}")]
  CoercionError {
    target: &'static str,
    was: &'static str,
  },

  #[error("Object of type {was} has no property \"{key}\".")]
  CannotBeIndexed { was: &'static str, key: String },

  #[error("table with columns {columns:?} has no key \"{key}\"")]
  TableMissingProperty { columns: Vec<String>, key: String },

  #[error("Error invoking function {function:?}: {inner}")]
  FunctionError {
    function: BuiltInFunction,
    inner: FunctionError,
  },
}

#[derive(Error, Debug)]
pub enum CompilerError {
  #[error("A table must have at least one column (in {name})")]
  EmptyTable { name: String },

  #[error("A bag must have at least one item (in {name}])")]
  EmptyBag { name: String },

  #[error("Column {column_name} had 0 non-hole entries (in {in_variable})")]
  EmptyTableColumn {
    column_name: String,
    in_variable: String,
  },

  #[error("Error on table row {row_number}: expected {} columns ({expected_columns:?}), found {} values ({values:?})", .expected_columns.len(), values.len())]
  InvalidTableRow {
    expected_columns: Vec<String>,
    values: Vec<ast::TableEntry>,
    row_number: usize,
  },

  #[error("The first item in table {in_variable} on row {row_number} must not be an append entry")]
  AppendInFirstColumn {
    row_number: usize,
    in_variable: String,
  },

  #[error("Function {0} is not defined")]
  UnknownFunction(String),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
  /*#[error("Parser error: {0}")]
  Parser(#[from] nom::Err<(&str, nom::error::ErrorKind)>),*/
  #[error("Compilation error: {0}")]
  Compiler(#[from] CompilerError),
  #[error("Interpreter error: {0}")]
  Interpreter(#[from] InterpreterError),
}

#[derive(Debug, Clone)]
pub enum NameHint {
  InAssignment(String),
  Repl,
}

impl NameHint {
  pub fn get_name(&self) -> String {
    match self {
      NameHint::InAssignment(name) => name.clone(),
      NameHint::Repl => String::from("<repl>"),
    }
  }
}

trait NameHintUtil {
  fn get_name_or_default(&self) -> String;
}

impl NameHintUtil for Option<NameHint> {
  fn get_name_or_default(&self) -> String {
    self
      .as_ref()
      .map(|hint| hint.get_name())
      .unwrap_or_else(|| String::from("<unknown>"))
  }
}

#[derive(Debug)]
pub struct CompiledScript {
  variables: HashMap<String, Expression>,
  id_counter: usize,
}

impl CompiledScript {
  pub fn new() -> Self {
    CompiledScript {
      variables: HashMap::new(),
      id_counter: 0,
    }
  }

  pub fn transform_expression(
    &mut self,
    expression: ast::Expression,
    name_hint: &Option<NameHint>,
  ) -> Result<Expression, CompilerError> {
    match expression {
      ast::Expression::LiteralE(literal) => Ok(Expression::LiteralE(literal)),
      ast::Expression::VariableE(variable) => Ok(Expression::VariableE(variable)),
      ast::Expression::PatternE(pattern) => {
        let parts: Result<Vec<_>, _> = pattern
          .parts
          .into_iter()
          .map(|part| self.transform_expression(part, name_hint))
          .collect();

        let parts = parts?;

        Ok(Expression::PatternE(Pattern { parts }))
      }
      ast::Expression::BagE(bag) => {
        self.id_counter += 1;
        let id = self.id_counter;

        let mut weights = Vec::new();
        let mut items = Vec::new();

        for (weight, expression) in bag.items.into_iter().map(|item| {
          (
            item.weight,
            self.transform_expression(*item.value, name_hint),
          )
        }) {
          let expression = expression?;

          let weight = weight.unwrap_or(1.0);

          weights.push(weight);
          items.push(expression);
        }

        let distribution = WeightedIndex::new(weights).map_err(|err| match err {
          WeightedError::NoItem => CompilerError::EmptyBag {
            name: name_hint.get_name_or_default(),
          },
          _ => panic!("Unhandled WeightedIndex error: {}", err),
        })?;

        let bag = Bag {
          id,
          items,
          name_hint: name_hint.clone(),
          distribution,
        };

        Ok(Expression::BagE(bag))
      }
      ast::Expression::TableE(table) => {
        if table.columns.len() < 1 {
          return Err(CompilerError::EmptyTable {
            name: name_hint.get_name_or_default(),
          });
        }

        let mut items_per_column = vec![Vec::new(); table.columns.len()];

        for (row_number, row) in table.rows.into_iter().enumerate() {
          if row.items.len() != table.columns.len() {
            return Err(CompilerError::InvalidTableRow {
              row_number,
              expected_columns: table.columns.clone(),
              values: row.items.clone(),
            });
          }

          let base_item = row.items[0].clone();

          let base_item = match base_item {
            ast::TableEntry::Literal(s) => self.transform_expression(*s, name_hint)?,
            ast::TableEntry::Hole => Expression::LiteralE(String::new()),
            ast::TableEntry::Append(_) => Err(CompilerError::AppendInFirstColumn {
              row_number,
              in_variable: name_hint.get_name_or_default(),
            })?,
          };

          for (column_number, item) in row.items.into_iter().enumerate() {
            let maybe_expr = match item {
              ast::TableEntry::Hole => None,
              ast::TableEntry::Literal(expr) => Some(self.transform_expression(*expr, name_hint)?),
              ast::TableEntry::Append(expr) => {
                let expr = self.transform_expression(*expr, name_hint)?;

                Some(Expression::PatternE(Pattern {
                  // TODO: Avoid this clone?
                  parts: vec![base_item.clone(), expr],
                }))
              }
            };

            match maybe_expr {
              Some(expr) => {
                items_per_column[column_number].push((row.weight, expr));
              }
              None => {}
            }
          }
        }

        let bags = items_per_column
          .into_iter()
          .zip(table.columns)
          .map(|(items, column)| {
            if items.len() == 0 {
              return Err(CompilerError::EmptyTableColumn {
                column_name: column.clone(),
                in_variable: name_hint.get_name_or_default(),
              });
            }

            self.id_counter += 1;
            let id = self.id_counter;

            let distribution = WeightedIndex::new(
              items
                .iter()
                .map(|(weight, _)| weight.unwrap_or(1.0))
                .collect(),
            )
            .unwrap();

            let bag = Bag {
              id,
              distribution,
              items: items.into_iter().map(|(_, item)| item).collect(),
              name_hint: name_hint.clone(),
            };

            Ok((column, bag))
          })
          .collect::<Result<HashMap<_, _>, CompilerError>>()?;

        Ok(Expression::TableE(Table {
          name_hint: name_hint.clone(),
          bags,
        }))
      }
      ast::Expression::PropertyAccessE(expression, property) => {
        let expression = self.transform_expression(*expression, name_hint)?;
        Ok(Expression::PropertyAccessE(Box::new(expression), property))
      }
      ast::Expression::CallE(name, arguments) => {
        // TODO: Support user defined functions / parameterised patterns
        let function =
          BuiltInFunction::try_parse(&name).ok_or_else(|| CompilerError::UnknownFunction(name))?;

        let arguments = arguments
          .into_iter()
          .map(|expr| self.transform_expression(expr, name_hint))
          .collect::<Result<Vec<_>, _>>()?;

        Ok(Expression::CallE(function, arguments))
      }
    }
  }

  pub fn run(&self) -> Result<String, InterpreterError> {
    let entry = self
      .variables
      .get("result")
      .expect("Expected result to be defined.");

    self
      .eval_expression(entry)?
      .try_as_string()
      .map(Cow::into_owned)
  }

  pub fn try_coerce_to_string<'a>(
    &'a self,
    value: Value<'a>,
  ) -> Result<Cow<'a, str>, InterpreterError> {
    match value {
      Value::StringV(v) => Ok(v),
      Value::BagV(bag) => {
        let value = self.sample_bag(bag)?;
        self.try_coerce_to_string(value)
      }
      otherwise => Err(InterpreterError::CoercionError {
        target: "string",
        was: otherwise.get_type_name(),
      }),
    }
  }

  fn sample_bag<'a>(&'a self, bag: &'a Bag) -> Result<Value<'a>, InterpreterError> {
    let mut rng = rand::thread_rng();
    let i = bag.distribution.sample(&mut rng);
    let expression = &bag.items[i];
    self.eval_expression(expression)
  }

  pub fn eval_expression<'a>(
    &'a self,
    expression: &'a Expression,
  ) -> Result<Value<'a>, InterpreterError> {
    match expression {
      Expression::LiteralE(literal) => Ok(Value::StringV(Cow::from(literal))),
      Expression::VariableE(variable) => {
        let expression = self
          .variables
          .get(variable)
          .ok_or_else(|| InterpreterError::UnknownVariable(variable.clone()))?;

        self.eval_expression(expression)
      }
      Expression::PatternE(pattern) => {
        let mut combined = String::new();

        for part in &pattern.parts {
          let part = self.eval_expression(part)?;
          let part_as_string = self.try_coerce_to_string(part)?;
          combined.push_str(&part_as_string);
        }

        Ok(Value::StringV(Cow::from(combined)))
      }
      Expression::BagE(bag) => Ok(Value::BagV(bag)),
      Expression::TableE(table) => Ok(Value::TableV(table)),
      Expression::PropertyAccessE(expression, property) => {
        let value = self.eval_expression(expression)?;

        match value {
          Value::TableV(table) => {
            let bag = table.bags.get(property);

            match bag {
              None => Err(InterpreterError::TableMissingProperty {
                columns: table.bags.keys().map(String::from).collect(),
                key: property.clone(),
              }),
              Some(bag) => Ok(Value::BagV(bag)),
            }
          }
          otherwise => Err(InterpreterError::CannotBeIndexed {
            was: otherwise.get_type_name(),
            key: property.clone(),
          }),
        }
      }
      Expression::CallE(function, arguments) => {
        self.eval_builtin_function(function, arguments.as_slice())
      }
    }
  }

  fn eval_builtin_function<'a>(
    &'a self,
    function: &BuiltInFunction,
    arguments: &'a [Expression],
  ) -> Result<Value<'a>, InterpreterError> {
    match function {
      BuiltInFunction::UpperFirst => match arguments {
        &[ref inner] => {
          let inner = self.eval_expression(inner)?;
          let inner_as_string = self.try_coerce_to_string(inner)?;
          let capitalised = string_utils::capitalise_first(inner_as_string.as_ref());
          Ok(Value::StringV(Cow::from(capitalised)))
        }
        _ => Err(InterpreterError::FunctionError {
          function: BuiltInFunction::UpperFirst,
          inner: FunctionError::WrongNumberOfArguments {
            expected: 1,
            was: arguments.len() as u8,
          },
        }),
      },
      BuiltInFunction::MaybePrepend => match arguments {
        &[ref prefix, ref condition] => {
          let inner = self.eval_expression(condition)?;
          let inner_as_string = self.try_coerce_to_string(inner)?;

          if inner_as_string.len() > 0 {
            let prefix = self.eval_expression(prefix)?;
            let prefix_as_string = self.try_coerce_to_string(prefix)?;

            let mut prefixed = prefix_as_string.to_string();
            prefixed.push_str(&inner_as_string);
            Ok(Value::StringV(Cow::from(prefixed)))
          } else {
            Ok(Value::StringV(inner_as_string))
          }
        }
        _ => Err(InterpreterError::FunctionError {
          function: BuiltInFunction::MaybePrepend,
          inner: FunctionError::WrongNumberOfArguments {
            expected: 2,
            was: arguments.len() as u8,
          },
        }),
      },
      BuiltInFunction::MaybeAppend => match arguments {
        &[ref condition, ref suffix] => {
          let inner = self.eval_expression(condition)?;
          let inner_as_string = self.try_coerce_to_string(inner)?;

          if inner_as_string.len() > 0 {
            let suffix = self.eval_expression(suffix)?;
            let suffix_as_string = self.try_coerce_to_string(suffix)?;

            let mut suffixed = inner_as_string.to_string();
            suffixed.push_str(&suffix_as_string);
            Ok(Value::StringV(Cow::from(suffixed)))
          } else {
            Ok(Value::StringV(inner_as_string))
          }
        }
        _ => Err(InterpreterError::FunctionError {
          function: BuiltInFunction::MaybePrepend,
          inner: FunctionError::WrongNumberOfArguments {
            expected: 2,
            was: arguments.len() as u8,
          },
        }),
      },
    }
  }

  fn define_variable(&mut self, name: String, value: Expression) {
    self.variables.insert(name, value);
  }

  pub fn add_statement(&mut self, statement: ast::Statement) -> Result<(), CompilerError> {
    match statement {
      ast::Statement::AssignmentS(assignment) => {
        let name_hint = Some(NameHint::InAssignment(assignment.name.clone()));
        let expression = self.transform_expression(*assignment.value, &name_hint)?;
        self.define_variable(assignment.name, expression);
      }
    }

    Ok(())
  }
}

pub fn compile_script(statements: Vec<ast::Statement>) -> Result<CompiledScript, CompilerError> {
  let mut script = CompiledScript::new();

  for statement in statements {
    script.add_statement(statement)?;
  }

  Ok(script)
}

#[cfg(test)]
mod tests {
  use super::{ast, compile_script};

  #[test]
  fn eval_literal() {
    let compiled = compile_script(vec![ast::Statement::AssignmentS(ast::Assignment {
      name: String::from("result"),
      value: Box::new(ast::Expression::LiteralE(String::from("Hello, world!"))),
    })])
    .unwrap();

    let output = compiled.run().unwrap();
    assert_eq!(output, "Hello, world!");
  }

  #[test]
  fn eval_upper_first() {
    let compiled = compile_script(vec![ast::Statement::AssignmentS(ast::Assignment {
      name: String::from("result"),
      value: Box::new(ast::Expression::CallE(
        String::from("capitalise"),
        vec![ast::Expression::LiteralE(String::from("robert"))],
      )),
    })])
    .unwrap();

    let output = compiled.run().unwrap();
    assert_eq!(output, "Robert");
  }
}
