#[derive(PartialEq, Debug)]
pub struct BagEntry {
    pub weight: Option<f32>,
    pub value: Box<Expression>,
}

impl BagEntry {
    pub fn from_string(x: impl Into<String>) -> BagEntry {
        BagEntry {
            weight: None,
            value: Box::new(Expression::LiteralE(x.into())),
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = Some(weight);
        self
    }
}

#[derive(PartialEq, Debug)]
pub struct Bag {
    pub items: Vec<BagEntry>,
}

#[derive(PartialEq, Debug)]
pub struct Assignment {
    pub name: String,
    pub value: Box<Expression>,
}

#[derive(PartialEq, Debug)]
pub enum Statement {
    AssignmentS(Assignment),
}

#[derive(PartialEq, Debug)]
pub enum Expression {
    LiteralE(String),
    VariableE(String),
    BagE(Bag),
}
