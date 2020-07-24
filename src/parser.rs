use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{char, space0};
use nom::character::is_alphanumeric;
use nom::combinator::opt;
use nom::multi::separated_list;
use nom::number::complete::float;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use crate::ast::{Assignnment, Bag, BagEntry, Expression};

fn string_literal<'a>(input: &'a str) -> IResult<&str, &'a str> {
    delimited(char('"'), take_while(|c| c != '"'), char('"'))(input)
}

fn ws<'a>(input: &'a str) -> IResult<&str, ()> {
    let (input, _) = space0(input)?;
    Ok((input, ()))
}

pub fn parse_bag_entry<'a>(input: &'a str) -> IResult<&'a str, BagEntry> {
    let (input, weight): (&str, Option<f32>) = opt(float)(input)?;
    let (input, ()) = ws(input)?;
    let (input, value) = string_literal(input)?;
    let value = Box::new(Expression::LiteralE(String::from(value)));

    Ok((input, BagEntry { weight, value }))
}

pub fn parse_bag<'a>(input: &'a str) -> IResult<&'a str, Bag> {
    let (input, items): (&str, Vec<BagEntry>) = preceded(
        tag("bag"),
        delimited(
            tag("("),
            separated_list(tag(","), terminated(parse_bag_entry, ws)),
            tag(")"),
        ),
    )(input)?;

    Ok((input, Bag { items }))
}

pub fn parse_variable(input: &str) -> IResult<&str, String> {
    let (input, name) = take_while(|c| is_alphanumeric(c as u8))(input)?;
    Ok((input, String::from(name)))
}

pub fn parse_assignment(input: &str) -> IResult<&str, Assignnment> {
    let (input, name) = parse_variable(input)?;
    let (input, _) = tuple((ws, tag("="), ws))(input)?;
    let (input, value) = parse_bag(input)?;

    Ok((
        input,
        Assignnment {
            name,
            value: Box::new(Expression::BagE(value)),
        },
    ))
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
            parse_bag(r#"bag("epic", "awesome", "cool")"#),
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
        use super::{parse_assignment, Assignnment, Bag, BagEntry, Expression};

        assert_eq!(
            parse_assignment(r#"adjective = bag("Friendly", "Unfriendly")"#),
            Ok((
                "",
                Assignnment {
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
}
