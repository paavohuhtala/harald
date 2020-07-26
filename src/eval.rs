use crate::ast;
use rand::{distributions::weighted::alias_method::WeightedIndex, prelude::Distribution};
use std::{borrow::Cow, collections::HashMap, fmt::Display};
use thiserror::Error;

#[derive(Debug)]
pub enum Value<'a> {
    StringV(Cow<'a, str>),
    BagV(&'a Bag),
}

impl<'a> Value<'a> {
    pub fn get_type_name(&self) -> &'static str {
        match self {
            Value::StringV(_) => "string",
            Value::BagV(_) => "bag",
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
            Value::BagV(bag) => write!(f, "[Bag {}]", bag.id),
        }
    }
}

#[derive(Debug)]
pub struct Pattern {
    parts: Vec<Expression>,
}

#[derive(Debug)]
pub struct TableDict {
    bags: HashMap<String, Bag>,
}

#[derive(Debug)]
pub struct Bag {
    id: usize,
    name: Option<String>,
    items: Vec<Expression>,
    distribution: WeightedIndex<f32>,
}

#[derive(Debug)]
pub enum Expression {
    LiteralE(String),
    VariableE(String),
    PatternE(Pattern),
    BagE(Bag),
}

pub trait StringWritable {
    fn append_str(&mut self, s: &str);
}

impl StringWritable for String {
    fn append_str(&mut self, s: &str) {
        self.push_str(s);
    }
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

    pub fn transform_expression(&mut self, expression: ast::Expression) -> Expression {
        match expression {
            ast::Expression::LiteralE(literal) => Expression::LiteralE(literal),
            ast::Expression::VariableE(variable) => Expression::VariableE(variable),
            ast::Expression::PatternE(pattern) => {
                let parts = pattern
                    .parts
                    .into_iter()
                    .map(|part| self.transform_expression(part))
                    .collect();

                Expression::PatternE(Pattern { parts })
            }
            ast::Expression::BagE(bag) => {
                self.id_counter += 1;
                let id = self.id_counter;

                let mut weights = Vec::new();
                let mut items = Vec::new();

                for (weight, expression) in bag
                    .items
                    .into_iter()
                    .map(|item| (item.weight, self.transform_expression(*item.value)))
                {
                    let weight = weight.unwrap_or(1.0);

                    weights.push(weight);
                    items.push(expression);
                }

                let bag = Bag {
                    id,
                    items,
                    name: None,
                    distribution: WeightedIndex::new(weights).unwrap(),
                };

                Expression::BagE(bag)
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
                    let part = self.eval_expression(part)?.try_as_string()?;
                    combined.push_str(&part);
                }

                Ok(Value::StringV(Cow::from(combined)))
            }
            Expression::BagE(bag) => {
                let mut rng = rand::thread_rng();
                let i = bag.distribution.sample(&mut rng);
                let expression = &bag.items[i];

                self.eval_expression(expression)
            }
        }
    }

    fn define_variable(&mut self, name: String, value: Expression) {
        self.variables.insert(name, value);
    }

    pub fn add_statement(&mut self, statement: ast::Statement) {
        match statement {
            ast::Statement::AssignmentS(assignment) => {
                let expression = self.transform_expression(*assignment.value);
                self.define_variable(assignment.name, expression);
            }
        }
    }
}

pub fn compile_script(statements: Vec<ast::Statement>) -> CompiledScript {
    let mut script = CompiledScript::new();

    for statement in statements {
        script.add_statement(statement);
    }

    script
}

#[cfg(test)]
mod tests {
    use super::{ast, compile_script};

    #[test]
    fn test_eval_literal() {
        let compiled = compile_script(vec![ast::Statement::AssignmentS(ast::Assignment {
            name: String::from("result"),
            value: Box::new(ast::Expression::LiteralE(String::from("Hello, world!"))),
        })]);

        let output = compiled.run().unwrap();
        assert_eq!(output, "Hello, world!");
    }
}
