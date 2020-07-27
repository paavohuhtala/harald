use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{alpha1, char, multispace0};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::combinator::{all_consuming, map, opt, recognize};
use nom::multi::{many0, many1, separated_list};
use nom::number::complete::float;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use crate::ast::{
    Assignment, Bag, BagEntry, Expression, Pattern, Statement, TableDict, TableDictEntry,
    TableDictRow,
};

fn parse_string_literal<'a>(input: &'a str) -> IResult<&str, &'a str> {
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

pub fn parse_identifier(input: &str) -> IResult<&str, &str> {
    let first_letter = alpha1;
    let rest = take_while(|b| is_alphanumeric(b as u8) || b == '_');
    let identifier = preceded(first_letter, rest);
    recognize(identifier)(input)
}

pub fn parse_pattern(input: &str) -> IResult<&str, Pattern> {
    let (input, expressions) = delimited(
        char('{'),
        many0(delimited(multispace0, parse_expression, multispace0)),
        char('}'),
    )(input)?;

    Ok((input, Pattern { parts: expressions }))
}

pub fn parse_table_dict_header(input: &str) -> IResult<&str, Vec<String>> {
    let (input, columns) = delimited(
        char('['),
        separated_list(
            tag(","),
            delimited(
                multispace0,
                preceded(char('.'), take_while1(|c| is_alphabetic(c as u8))),
                multispace0,
            ),
        ),
        char(']'),
    )(input)?;

    Ok((input, columns.into_iter().map(String::from).collect()))
}

pub fn parse_table_dict_entry(input: &str) -> IResult<&str, TableDictEntry> {
    let (input, placeholder) = opt(char('_'))(input)?;

    if placeholder.is_some() {
        let (input, _) = ws(input)?;
        return Ok((input, TableDictEntry::Hole));
    }

    let (input, prefix): (&str, Option<char>) = opt(char('+'))(input)?;
    let (input, _) = ws(input)?;
    let is_append = prefix.is_some();
    let (input, expression) = parse_expression(input)?;

    Ok((
        input,
        match is_append {
            true => TableDictEntry::Append(Box::new(expression)),
            false => TableDictEntry::Literal(Box::new(expression)),
        },
    ))
}

pub fn parse_table_dict_row(input: &str) -> IResult<&str, TableDictRow> {
    let (input, weight) = opt(float)(input)?;
    let (input, _) = ws(input)?;

    let (input, items) = delimited(
        char('['),
        separated_list(
            tag(","),
            delimited(multispace0, parse_table_dict_entry, multispace0),
        ),
        char(']'),
    )(input)?;

    Ok((input, TableDictRow { items, weight }))
}

pub fn parse_table_dict(input: &str) -> IResult<&str, TableDict> {
    let (input, (columns, rows)) = preceded(
        tag("table_dict"),
        preceded(
            ws,
            delimited(
                char('['),
                delimited(
                    ws,
                    tuple((
                        terminated(parse_table_dict_header, tuple((ws, char(','), ws))),
                        separated_list(char(','), parse_table_dict_row),
                    )),
                    ws,
                ),
                char(']'),
            ),
        ),
    )(input)?;

    Ok((input, TableDict { columns, rows }))
}

pub fn parse_property_access(input: &str) -> IResult<&str, Expression> {
    // TODO: Support other expressions.
    let (input, identifier) = parse_identifier(input)?;
    let expression = Expression::VariableE(String::from(identifier));

    let (input, property) = preceded(char('.'), parse_identifier)(input)?;

    Ok((
        input,
        Expression::PropertyAccessE(Box::new(expression), String::from(property)),
    ))
}

pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        map(parse_pattern, Expression::PatternE),
        map(parse_string_literal, |s| {
            Expression::LiteralE(String::from(s))
        }),
        map(parse_table_dict, Expression::TableDictE),
        map(parse_bag, Expression::BagE),
        parse_property_access,
        map(parse_identifier, |s| Expression::VariableE(String::from(s))),
    ))(input)
}

pub fn parse_assignment(input: &str) -> IResult<&str, Assignment> {
    let (input, (name, _, _, _, value)) =
        tuple((parse_identifier, ws, tag("="), ws, parse_expression))(input)?;

    Ok((
        input,
        Assignment {
            name: name.to_string(),
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
    let (input, statements) = all_consuming(terminated(many1(parse_statement), ws))(input)?;
    Ok((input, statements))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_string_literal() {
        use super::parse_string_literal;
        assert_eq!(
            parse_string_literal(r#""Hello, world!""#),
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

    #[test]
    fn test_parse_table_dict_entry() {
        use super::{parse_table_dict_entry, Expression, TableDictEntry};

        assert_eq!(
            parse_table_dict_entry(r#""Harald""#),
            Ok((
                "",
                TableDictEntry::Literal(Box::new(Expression::LiteralE(String::from("Harald"))))
            ))
        );

        assert_eq!(
            parse_table_dict_entry(r#"+"in""#),
            Ok((
                "",
                TableDictEntry::Append(Box::new(Expression::LiteralE(String::from("in"))))
            ))
        );
    }

    #[test]
    fn test_parse_table_dict_row() {
        use super::{parse_table_dict_row, Expression, TableDictEntry, TableDictRow};

        assert_eq!(
            parse_table_dict_row(r#"["unicorn", "unicorns"]"#),
            Ok((
                "",
                TableDictRow {
                    weight: None,
                    items: vec![
                        TableDictEntry::Literal(Box::new(Expression::LiteralE(String::from(
                            "unicorn"
                        )))),
                        TableDictEntry::Literal(Box::new(Expression::LiteralE(String::from(
                            "unicorns"
                        ))))
                    ]
                }
            ))
        );

        assert_eq!(
            parse_table_dict_row(r#"["unicorn", +"s"]"#),
            Ok((
                "",
                TableDictRow {
                    weight: None,
                    items: vec![
                        TableDictEntry::Literal(Box::new(Expression::LiteralE(String::from(
                            "unicorn"
                        )))),
                        TableDictEntry::Append(Box::new(Expression::LiteralE(String::from("s"))))
                    ]
                }
            ))
        );

        assert_eq!(
            parse_table_dict_row(r#"[  "unicorn"  , + "s"   ]"#),
            Ok((
                "",
                TableDictRow {
                    weight: None,
                    items: vec![
                        TableDictEntry::Literal(Box::new(Expression::LiteralE(String::from(
                            "unicorn"
                        )))),
                        TableDictEntry::Append(Box::new(Expression::LiteralE(String::from("s"))))
                    ]
                }
            ))
        );
    }

    #[test]
    fn test_parse_table_dict() {
        use super::{parse_table_dict, Expression, TableDict, TableDictEntry, TableDictRow};

        let table_dict = r#"table_dict [
            [.base, .plural],
            ["unicorn", "unicorns"],
            ["kitten", +"s"]
        ]"#;

        assert_eq!(
            parse_table_dict(table_dict),
            Ok((
                "",
                TableDict {
                    columns: vec![String::from("base"), String::from("plural")],
                    rows: vec![
                        TableDictRow {
                            weight: None,
                            items: vec![
                                TableDictEntry::Literal(Box::new(Expression::LiteralE(
                                    String::from("unicorn")
                                ))),
                                TableDictEntry::Literal(Box::new(Expression::LiteralE(
                                    String::from("unicorns")
                                )))
                            ]
                        },
                        TableDictRow {
                            weight: None,
                            items: vec![
                                TableDictEntry::Literal(Box::new(Expression::LiteralE(
                                    String::from("kitten")
                                ))),
                                TableDictEntry::Append(Box::new(Expression::LiteralE(
                                    String::from("s")
                                )))
                            ]
                        }
                    ]
                }
            ))
        )
    }
}
