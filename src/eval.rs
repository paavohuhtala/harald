use crate::ast;
use rand::distributions::weighted::alias_method::WeightedIndex;
use std::{borrow::Cow, collections::HashMap};

enum Value<'a> {
    StringV(Cow<'a, str>),
}

#[derive(Debug)]
pub struct Pattern {
    parts: Vec<Expression>,
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

/* impl RuntimeBag {
    fn from_ast(id: usize, bag: ast::Bag) -> Self {
        /*let weights = bag
            .items
            .iter()
            .map(|item| item.weight.unwrap_or(1.0))
            .collect::<Vec<_>>();
        let items = bag.items.into_iter().map(|item| item.value);
        let distribution = WeightedIndex::new(weights).unwrap();

        RuntimeBag {
            id,
            name: None,
            items,
            distribution,
        }*/

        todo!()
    }
}*/

pub trait StringWritable {
    fn append_str(&mut self, s: &str);
}

impl StringWritable for String {
    fn append_str(&mut self, s: &str) {
        self.push_str(s);
    }
}

/*struct BagContext {
    bag_counter: usize,
    bags: Vec<RuntimeBag>,
}

struct BagHandle(usize);

impl BagContext {
    fn create_bag_from_ast(&mut self, bag: ast::Bag) -> BagHandle {
        todo!()
    }

    fn sample<'a>(&'a mut self, handle: BagHandle) -> Value<'a> {
        todo!()
    }
}

struct ExecutionContext {
    bag_ctx: BagContext,
}*/

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

    pub fn run(&self, output: &mut impl StringWritable) {
        let entry = self
            .variables
            .get("result")
            .expect("Expected result to be defined.");

        self.eval_expression(entry, output);
    }

    pub fn eval_expression(&self, expression: &Expression, output: &mut impl StringWritable) {
        match expression {
            Expression::LiteralE(literal) => {
                output.append_str(literal);
            }
            Expression::VariableE(variable) => {
                let expression = self
                    .variables
                    .get(variable)
                    .expect(&format!("Unknown variable: {}", variable));

                self.eval_expression(expression, output)
            }
            Expression::PatternE(pattern) => {
                for part in &pattern.parts {
                    self.eval_expression(part, output);
                }
            }
            Expression::BagE(bag) => {
                self.eval_expression(&bag.items[0], output);
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
    #[test]
    fn test_eval_literal() {
        use super::{ast, compile_script};

        let compiled = compile_script(vec![ast::Statement::AssignmentS(ast::Assignment {
            name: String::from("result"),
            value: Box::new(ast::Expression::LiteralE(String::from("Hello, world!"))),
        })]);

        let mut output = String::new();
        compiled.run(&mut output);

        assert_eq!(output, "Hello, world!");
    }
}
