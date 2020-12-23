use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, tag, take_while1};
use nom::character::is_alphabetic;
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::multi::{many0, many1, separated_list0};
use nom::number::complete::float;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{
  character::complete::{char, multispace0},
  combinator::value,
};
use nom::{
  error::{context, VerboseError},
  IResult,
};

type ParseResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

use crate::ast::{
  Assignment, Bag, BagEntry, Expression, Pattern, Statement, Table, TableEntry, TableRow,
};

fn parse_string_literal(input: &str) -> ParseResult<String> {
  context(
    "string literal",
    delimited(
      char('"'),
      map(
        opt(escaped_transform(
          nom::bytes::complete::is_not("\\\""),
          '\\',
          alt((
            value("\\", tag("\\")),
            value("\"", tag("\"")),
            value("\n", tag("\n")),
          )),
        )),
        |literal: Option<String>| literal.unwrap_or_else(|| String::from("")),
      ),
      char('"'),
    ),
  )(input)
}

fn ws<'a>(input: &'a str) -> ParseResult<()> {
  let (input, _) = multispace0(input)?;
  Ok((input, ()))
}

pub fn parse_bag_entry<'a>(input: &'a str) -> ParseResult<BagEntry> {
  let (input, (_, weight, _, value)) =
    context("bag entry", tuple((ws, opt(float), ws, parse_expression)))(input)?;

  let value = Box::new(value);

  Ok((input, BagEntry { weight, value }))
}

pub fn parse_bag<'a>(input: &'a str) -> ParseResult<Bag> {
  let (input, (_, _, items)) = context(
    "bag",
    tuple((
      tag("bag"),
      ws,
      delimited(
        tag("["),
        separated_list0(tag(","), terminated(parse_bag_entry, ws)),
        tag("]"),
      ),
    )),
  )(input)?;

  Ok((input, Bag { items }))
}

/*pub fn is_allowed_letter(ch: char) -> bool {
  ch.is_alphabetic()
}*/

pub fn parse_identifier(input: &str) -> ParseResult<&str> {
  let first_letter = nom::character::complete::satisfy(|ch| ch.is_alphabetic());
  let identifier = preceded(first_letter, crate::nom_unicode::complete::alphanumeric0);
  recognize(identifier)(input)
}

pub fn parse_pattern(input: &str) -> ParseResult<Pattern> {
  let (input, expressions) = delimited(
    char('{'),
    many0(delimited(multispace0, parse_expression, multispace0)),
    char('}'),
  )(input)?;

  Ok((input, Pattern { parts: expressions }))
}

pub fn parse_table_header(input: &str) -> ParseResult<Vec<String>> {
  let (input, columns) = context(
    "table header",
    delimited(
      char('['),
      separated_list0(
        tag(","),
        delimited(
          multispace0,
          preceded(char('.'), take_while1(|c| is_alphabetic(c as u8))),
          multispace0,
        ),
      ),
      char(']'),
    ),
  )(input)?;

  Ok((input, columns.into_iter().map(String::from).collect()))
}

pub fn parse_table_entry(input: &str) -> ParseResult<TableEntry> {
  let (input, placeholder) = opt(char('_'))(input)?;

  if placeholder.is_some() {
    let (input, _) = ws(input)?;
    return Ok((input, TableEntry::Hole));
  }

  let (input, prefix): (&str, Option<char>) = opt(char('+'))(input)?;
  let (input, _) = ws(input)?;
  let is_append = prefix.is_some();
  let (input, expression) = parse_expression(input)?;

  Ok((
    input,
    match is_append {
      true => TableEntry::Append(Box::new(expression)),
      false => TableEntry::Literal(Box::new(expression)),
    },
  ))
}

pub fn parse_table_row(input: &str) -> ParseResult<TableRow> {
  let (input, _) = ws(input)?;
  let (input, weight) = opt(float)(input)?;
  let (input, _) = ws(input)?;

  let (input, items) = delimited(
    char('['),
    separated_list0(
      tag(","),
      delimited(
        multispace0,
        context("table entry", parse_table_entry),
        multispace0,
      ),
    ),
    char(']'),
  )(input)?;

  Ok((input, TableRow { items, weight }))
}

pub fn parse_table(input: &str) -> ParseResult<Table> {
  let (input, (columns, rows)) = context(
    "table",
    preceded(
      tag("table"),
      preceded(
        ws,
        delimited(
          char('['),
          delimited(
            ws,
            tuple((
              terminated(parse_table_header, tuple((ws, char(','), ws))),
              separated_list0(char(','), parse_table_row),
            )),
            ws,
          ),
          char(']'),
        ),
      ),
    ),
  )(input)?;

  Ok((input, Table { columns, rows }))
}

pub fn parse_property_access(input: &str) -> ParseResult<Expression> {
  // TODO: Support other expressions.
  let (input, identifier) = parse_identifier(input)?;
  let expression = Expression::VariableE(String::from(identifier));

  let (input, property) = preceded(char('.'), parse_identifier)(input)?;

  Ok((
    input,
    Expression::PropertyAccessE(Box::new(expression), String::from(property)),
  ))
}

pub fn parse_call(input: &str) -> ParseResult<Expression> {
  let parse_argument_list = separated_list0(delimited(ws, tag(","), ws), parse_expression);

  context(
    "function call",
    map(
      tuple((
        parse_identifier,
        delimited(tag("("), delimited(ws, parse_argument_list, ws), tag(")")),
      )),
      |(function, args)| Expression::CallE(function.to_string(), args),
    ),
  )(input)
}

pub fn parse_expression(input: &str) -> ParseResult<Expression> {
  context(
    "expression",
    alt((
      map(parse_pattern, Expression::PatternE),
      map(parse_string_literal, |s| Expression::LiteralE(s)),
      map(parse_table, Expression::TableE),
      map(parse_bag, Expression::BagE),
      parse_property_access,
      parse_call,
      map(parse_identifier, |s| Expression::VariableE(String::from(s))),
    )),
  )(input)
}

pub fn parse_assignment(input: &str) -> ParseResult<Assignment> {
  let (input, (name, _, _, _, value)) = context(
    "assignment",
    tuple((parse_identifier, ws, tag("="), ws, parse_expression)),
  )(input)?;

  Ok((
    input,
    Assignment {
      name: name.to_string(),
      value: Box::new(value),
    },
  ))
}

pub fn parse_assignment_statement(input: &str) -> ParseResult<Statement> {
  map(parse_assignment, Statement::AssignmentS)(input)
}

pub fn parse_statement(input: &str) -> ParseResult<Statement> {
  let (input, (_, statement, _, _)) = context(
    "statement",
    tuple((ws, parse_assignment_statement, ws, char(';'))),
  )(input)?;
  Ok((input, statement))
}

pub fn parse_program(input: &str) -> ParseResult<Vec<Statement>> {
  let (input, statements) = context(
    "program",
    all_consuming(terminated(many1(parse_statement), ws)),
  )(input)?;
  Ok((input, statements))
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_parse_string_literal() {
    use super::parse_string_literal;
    assert_eq!(
      parse_string_literal(r#""Hello, world!""#),
      Ok(("", String::from("Hello, world!")))
    );
  }

  #[test]
  fn test_parse_string_literal_empty() {
    use super::parse_string_literal;
    assert_eq!(parse_string_literal(r#""""#), Ok(("", String::from(""))));
  }

  #[test]
  fn test_parse_string_literal_newline() {
    use super::parse_string_literal;
    assert_eq!(
      parse_string_literal("\"Hello\nWorld\""),
      Ok(("", String::from("Hello\nWorld")))
    );
  }

  #[test]
  fn test_parse_bag() {
    use super::{parse_bag, Bag, BagEntry};

    let expected_items: Vec<_> = vec![
      BagEntry::from_string("epic"),
      BagEntry::from_string("awesome"),
      BagEntry::from_string("cool"),
    ];

    assert_eq!(
      parse_bag(r#"bag["epic", "awesome", "cool"]"#),
      Ok((
        "",
        Bag {
          items: expected_items
        }
      ))
    )
  }

  #[test]
  fn test_parse_bag_entry() {
    use super::{parse_bag_entry, BagEntry};

    assert_eq!(
      parse_bag_entry(r#""no weight""#),
      Ok(("", BagEntry::from_string("no weight")))
    );
    assert_eq!(
      parse_bag_entry(r#"3.0 "float weighted""#),
      Ok(("", BagEntry::from_string("float weighted").with_weight(3.0)))
    );
  }

  #[test]
  fn test_parse_assignment() {
    use super::{parse_assignment, Assignment, Bag, BagEntry, Expression};

    assert_eq!(
      parse_assignment(r#"adjective = bag["Friendly", "Unfriendly"]"#),
      Ok((
        "",
        Assignment {
          name: String::from("adjective"),
          value: Box::new(Expression::BagE(Bag {
            items: vec![
              BagEntry::from_string("Friendly"),
              BagEntry::from_string("Unfriendly")
            ],
          })),
        },
      ))
    );
  }

  #[test]
  fn test_parse_assignment_literal() {
    use super::{parse_assignment, Assignment, Expression};

    assert_eq!(
      parse_assignment(r#"secretWord = "hunter2""#),
      Ok((
        "",
        Assignment {
          name: String::from("secretWord"),
          value: Box::new(Expression::LiteralE(String::from("hunter2")))
        }
      ))
    );
  }

  #[test]
  fn test_parse_program() {
    use super::{parse_program, Assignment, Bag, BagEntry, Expression, Statement};

    let program = r#"
            adjective = bag["Friendly", "Unfriendly"];
            result = adjective;
        "#;

    assert_eq!(
      parse_program(program),
      Ok((
        "",
        vec![
          Statement::AssignmentS(Assignment {
            name: String::from("adjective"),
            value: Box::new(Expression::BagE(Bag {
              items: vec![
                BagEntry::from_string("Friendly"),
                BagEntry::from_string("Unfriendly")
              ],
            })),
          }),
          Statement::AssignmentS(Assignment {
            name: String::from("result"),
            value: Box::new(Expression::VariableE(String::from("adjective")))
          })
        ]
      ))
    );
  }

  #[test]
  fn test_parse_pattern() {
    use super::{parse_expression, Expression, Pattern};

    assert_eq!(
      parse_expression(r#"{ "Hello " world }"#),
      Ok((
        "",
        Expression::PatternE(Pattern {
          parts: vec![
            Expression::LiteralE(String::from("Hello ")),
            Expression::VariableE(String::from("world"))
          ]
        })
      ))
    );
  }

  #[test]
  fn test_parse_call_1() {
    use super::{parse_expression, Expression};

    assert_eq!(
      parse_expression(r#"print("Hello, world!")"#),
      Ok((
        "",
        Expression::CallE(
          String::from("print"),
          vec![Expression::LiteralE(String::from("Hello, world!"))]
        )
      ))
    );
  }

  #[test]
  fn test_parse_call_0() {
    use super::{parse_expression, Expression};

    assert_eq!(
      parse_expression(r#"printHello()"#),
      Ok(("", Expression::CallE(String::from("printHello"), vec![])))
    );
  }

  #[test]
  fn test_parse_call_2_pattern() {
    use super::{parse_expression, Expression, Pattern};

    assert_eq!(
      parse_expression(r#"concat( { "Hello" }, {" world!"} )"#),
      Ok((
        "",
        Expression::CallE(
          String::from("concat"),
          vec![
            Expression::PatternE(Pattern {
              parts: vec![Expression::LiteralE(String::from("Hello")),]
            }),
            Expression::PatternE(Pattern {
              parts: vec![Expression::LiteralE(String::from(" world!")),]
            }),
          ]
        )
      ))
    );
  }

  #[test]
  fn test_parse_table_entry() {
    use super::{parse_table_entry, Expression, TableEntry};

    assert_eq!(
      parse_table_entry(r#""Harald""#),
      Ok((
        "",
        TableEntry::Literal(Box::new(Expression::LiteralE(String::from("Harald"))))
      ))
    );

    assert_eq!(
      parse_table_entry(r#"+"in""#),
      Ok((
        "",
        TableEntry::Append(Box::new(Expression::LiteralE(String::from("in"))))
      ))
    );
  }

  #[test]
  fn test_parse_table_row() {
    use super::{parse_table_row, Expression, TableEntry, TableRow};

    assert_eq!(
      parse_table_row(r#"["unicorn", "unicorns"]"#),
      Ok((
        "",
        TableRow {
          weight: None,
          items: vec![
            TableEntry::Literal(Box::new(Expression::LiteralE(String::from("unicorn")))),
            TableEntry::Literal(Box::new(Expression::LiteralE(String::from("unicorns"))))
          ]
        }
      ))
    );

    assert_eq!(
      parse_table_row(r#"["unicorn", +"s"]"#),
      Ok((
        "",
        TableRow {
          weight: None,
          items: vec![
            TableEntry::Literal(Box::new(Expression::LiteralE(String::from("unicorn")))),
            TableEntry::Append(Box::new(Expression::LiteralE(String::from("s"))))
          ]
        }
      ))
    );

    assert_eq!(
      parse_table_row(r#"[  "unicorn"  , + "s"   ]"#),
      Ok((
        "",
        TableRow {
          weight: None,
          items: vec![
            TableEntry::Literal(Box::new(Expression::LiteralE(String::from("unicorn")))),
            TableEntry::Append(Box::new(Expression::LiteralE(String::from("s"))))
          ]
        }
      ))
    );
    assert_eq!(
      parse_table_row(r#"0.5 ["a", "b"]"#),
      Ok((
        "",
        TableRow {
          weight: Some(0.5),
          items: vec![
            TableEntry::Literal(Box::new(Expression::LiteralE(String::from("a")))),
            TableEntry::Literal(Box::new(Expression::LiteralE(String::from("b"))))
          ]
        }
      ))
    );
  }

  #[test]
  fn test_parse_table() {
    use super::{parse_table, Expression, Table, TableEntry, TableRow};

    let table = r#"table [
            [.base, .plural],
            ["unicorn", "unicorns"],
            ["kitten", +"s"]
        ]"#;

    assert_eq!(
      parse_table(table),
      Ok((
        "",
        Table {
          columns: vec![String::from("base"), String::from("plural")],
          rows: vec![
            TableRow {
              weight: None,
              items: vec![
                TableEntry::Literal(Box::new(Expression::LiteralE(String::from("unicorn")))),
                TableEntry::Literal(Box::new(Expression::LiteralE(String::from("unicorns"))))
              ]
            },
            TableRow {
              weight: None,
              items: vec![
                TableEntry::Literal(Box::new(Expression::LiteralE(String::from("kitten")))),
                TableEntry::Append(Box::new(Expression::LiteralE(String::from("s"))))
              ]
            }
          ]
        }
      ))
    )
  }
}
