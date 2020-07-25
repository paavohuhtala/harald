use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, multispace0};
use nom::character::is_alphanumeric;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1, separated_list};
use nom::number::complete::float;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use crate::ast::{Assignment, Bag, BagEntry, Expression, Pattern, Statement};

fn string_literal<'a>(input: &'a str) -> IResult<&str, &'a str> {
    delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)
}

fn ws<'a>(input: &'a str) -> IResult<&str, ()> {
    let (input, _) = multispace0(input)?;
    Ok((input, ()))
}

pub fn parse_bag_entry<'a>(input: &'a str) -> IResult<&'a str, BagEntry> {
    let (input, ()) = ws(input)?;
    let (input, weight): (&str, Option<f32>) = opt(float)(input)?;
    let (input, ()) = ws(input)?;
    let (input, value) = parse_expression(input)?;
    let value = Box::new(value);

    Ok((input, BagEntry { weight, value }))
}

pub fn parse_bag<'a>(input: &'a str) -> IResult<&'a str, Bag> {
    let (input, (_, _, items)) = tuple((
        tag("bag"),
        ws,
        delimited(
            tag("["),
            separated_list(tag(","), terminated(parse_bag_entry, ws)),
            tag("]"),
        ),
    ))(input)?;

    Ok((input, Bag { items }))
}

pub fn parse_variable(input: &str) -> IResult<&str, String> {
    let (input, name) = take_while1(|c| is_alphanumeric(c as u8))(input)?;
    Ok((input, String::from(name)))
}

pub fn parse_pattern(input: &str) -> IResult<&str, Pattern> {
    let (input, expressions) = delimited(
        tag("{"),
        many0(delimited(multispace0, parse_expression, multispace0)),
        tag("}"),
    )(input)?;

    Ok((input, Pattern { parts: expressions }))
}

pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        map(parse_pattern, |p| Expression::PatternE(p)),
        map(string_literal, |s| Expression::LiteralE(String::from(s))),
        map(parse_bag, |bag| Expression::BagE(bag)),
        map(parse_variable, Expression::VariableE),
    ))(input)
}

pub fn parse_assignment(input: &str) -> IResult<&str, Assignment> {
    let (input, (name, _, _, _, value)) =
        tuple((parse_variable, ws, tag("="), ws, parse_expression))(input)?;

    Ok((
        input,
        Assignment {
            name,
            value: Box::new(value),
        },
    ))
}

pub fn parse_assignment_statement(input: &str) -> IResult<&str, Statement> {
    map(parse_assignment, Statement::AssignmentS)(input)
}

pub fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, _) = ws(input)?;
    // let (input, statement) = alt((parse_assignment_statement,))(input)?;
    let (input, statement) = parse_assignment_statement(input)?;
    let (input, _) = tuple((ws, char(';')))(input)?;
    Ok((input, statement))
}

pub fn parse_program(input: &str) -> IResult<&str, Vec<Statement>> {
    let (input, statements) = many1(parse_statement)(input)?;
    // Consume trailing whitespace
    let (input, _) = ws(input)?;
    Ok((input, statements))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_string_literal() {
        use super::string_literal;
        assert_eq!(
            string_literal(r#""Hello, world!""#),
            Ok(("", "Hello, world!"))
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
}
