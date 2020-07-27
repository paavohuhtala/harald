use crate::ast;
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
    TableDictV(&'a TableDict),
}

impl<'a> Value<'a> {
    pub fn get_type_name(&self) -> &'static str {
        match self {
            Value::StringV(_) => "string",
            Value::BagV(_) => "bag",
            Value::TableDictV(_) => "table_dict",
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
            Value::TableDictV(table_dict) => {
                if let Some(name_hint) = &table_dict.name_hint {
                    write!(f, "table_dict ({:?})", name_hint)
                } else {
                    write!(f, "table_dict (anonymous)")
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
pub struct TableDict {
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
    TableDictE(TableDict),
    PropertyAccessE(Box<Expression>, String),
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

    #[error("table_dict with columns {columns:?} has no key \"{key}\"")]
    TableDictMissingProperty { columns: Vec<String>, key: String },
}

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("A table_dict must have at least one column")]
    EmptyTableDict,

    #[error("A bag must have at least one item")]
    EmptyBag,

    #[error("Error on table_dict row {row_number}: expected {} columns ({expected_columns:?}), found {} values ({values:?})", .expected_columns.len(), values.len())]
    InvalidTableDictRow {
        expected_columns: Vec<String>,
        values: Vec<ast::TableDictEntry>,
        row_number: usize,
    },

    #[error("The fist item in a table_dict must be a literal (was {0:?}).")]
    NonLiteralTableDictFirstPattern(ast::TableDictEntry),
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
                    WeightedError::NoItem => CompilerError::EmptyBag,
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
            ast::Expression::TableDictE(table_dict) => {
                if table_dict.columns.len() < 1 {
                    return Err(CompilerError::EmptyTableDict);
                }

                let mut items_per_column = vec![Vec::new(); table_dict.columns.len()];

                for (row_number, row) in table_dict.rows.into_iter().enumerate() {
                    if row.items.len() != table_dict.columns.len() {
                        return Err(CompilerError::InvalidTableDictRow {
                            row_number,
                            expected_columns: table_dict.columns.clone(),
                            values: row.items.clone(),
                        });
                    }

                    let base_item = row.items[0].clone();

                    let base_item = match base_item {
                        ast::TableDictEntry::Literal(s) => {
                            self.transform_expression(*s, name_hint)?
                        }
                        _ => Expression::LiteralE(String::new()),
                    };

                    for (column_number, item) in row.items.into_iter().enumerate() {
                        let maybe_expr = match item {
                            ast::TableDictEntry::Hole => None,
                            ast::TableDictEntry::Literal(expr) => {
                                Some(self.transform_expression(*expr, name_hint)?)
                            }
                            ast::TableDictEntry::Append(expr) => {
                                let expr = self.transform_expression(*expr, name_hint)?;

                                Some(Expression::PatternE(Pattern {
                                    // TODO: Avoid this clone?
                                    parts: vec![base_item.clone(), expr],
                                }))
                            }
                        };

                        match maybe_expr {
                            Some(expr) => {
                                items_per_column[column_number].push(expr);
                            }
                            None => {}
                        }
                    }
                }

                let mut bags = HashMap::new();

                for (items, column) in items_per_column.into_iter().zip(table_dict.columns) {
                    self.id_counter += 1;
                    let id = self.id_counter;

                    // TODO: implement weights
                    let distribution = WeightedIndex::new(vec![1.0; items.len()]).unwrap();

                    let bag = Bag {
                        id,
                        distribution,
                        items,
                        name_hint: name_hint.clone(),
                    };

                    bags.insert(column, bag);
                }

                Ok(Expression::TableDictE(TableDict {
                    name_hint: name_hint.clone(),
                    bags,
                }))
            }
            ast::Expression::PropertyAccessE(expression, property) => {
                let expression = self.transform_expression(*expression, name_hint)?;
                Ok(Expression::PropertyAccessE(Box::new(expression), property))
            }
            _ => todo!("unsupported expression"),
        }
    }

    pub fn run(&self) -> Result<String, InterpreterError> {
        let entry = self
            .variables
            .get("result")
            .expect("Expected result to be defined.");

        self.eval_expression(entry)?
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
            Expression::TableDictE(table_dict) => Ok(Value::TableDictV(table_dict)),
            Expression::PropertyAccessE(expression, property) => {
                let value = self.eval_expression(expression)?;

                match value {
                    Value::TableDictV(table_dict) => {
                        let bag = table_dict.bags.get(property);

                        match bag {
                            None => Err(InterpreterError::TableDictMissingProperty {
                                columns: table_dict.bags.keys().map(String::from).collect(),
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
    fn test_eval_literal() {
        let compiled = compile_script(vec![ast::Statement::AssignmentS(ast::Assignment {
            name: String::from("result"),
            value: Box::new(ast::Expression::LiteralE(String::from("Hello, world!"))),
        })])
        .unwrap();

        let output = compiled.run().unwrap();
        assert_eq!(output, "Hello, world!");
    }
}
