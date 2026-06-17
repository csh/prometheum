use nom::character::complete::space0;
use nom::sequence::preceded;
use nom::{
    bytes::{tag, take_while, take_while1}, character::char,
    error::context,
    sequence::{delimited, separated_pair, terminated},
    IResult,
    Parser,
};

/// Parsed [`NSLOCTEXT`](https://dev.epicgames.com/documentation/unreal-engine/text-localization-in-unreal-engine)
#[derive(Debug)]
pub struct LocalizableString {
    pub namespace: String,

    pub key: String,

    pub display: String,
}

/// Parsed [`FNameProperty`](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/CoreUObject/FNameProperty)
#[derive(Debug)]
pub struct FNameProperty {
    pub key: String,
    pub value: String,
}

fn quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(
        context("opening double quote", char('"')),
        take_while(|c: char| c != '"'),
        context("closing double quote", char('"')),
    )
    .parse(input)
}

pub fn parse_nsloctext(input: &str) -> IResult<&str, LocalizableString> {
    let (input, (namespace, key, display)) = preceded(
        tag("NSLOCTEXT"),
        delimited(
            context("opening parenthesis", char('(')),
            (
                context(
                    "namespace",
                    terminated(quoted_string, (space0, char(','), space0)),
                ),
                context(
                    "key",
                    terminated(quoted_string, (space0, char(','), space0)),
                ),
                context("value", quoted_string),
            ),
            context("closing parenthesis", char(')')),
        ),
    )
    .parse(input)?;

    Ok((
        input,
        LocalizableString {
            namespace: namespace.into(),
            key: key.into(),
            display: display.into(),
        },
    ))
}

pub fn parse_fname_property(input: &str) -> IResult<&str, FNameProperty> {
    let (input, (key, value)) = delimited(
        context("opening parenthesis", char('(')),
        separated_pair(
            context(
                "key",
                take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            ),
            context("assignment operator", (space0, char('='), space0)),
            context("value", quoted_string),
        ),
        context("closing parenthesis", char(')')),
    )
    .parse(input)?;

    Ok((
        input,
        FNameProperty {
            key: key.into(),
            value: value.into(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nsloctext_silent_blade_display_name() {
        let input =
            r#"NSLOCTEXT("D_Talents", "Knife_Increased_Sneak-DisplayName", "Silent Blade")"#;
        let (rest, ns) = parse_nsloctext(input).unwrap();
        assert_eq!(rest, "", "parser must consume the whole input");
        assert_eq!(ns.namespace, "D_Talents");
        assert_eq!(ns.key, "Knife_Increased_Sneak-DisplayName");
        assert_eq!(ns.display, "Silent Blade");
    }

    #[test]
    fn nsloctext_longer_description() {
        let input = r#"NSLOCTEXT("D_Talents", "Knife_Increased_Sneak-Description", "Increased sneak while holding a knife")"#;
        let (rest, ns) = parse_nsloctext(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(ns.display, "Increased sneak while holding a knife");
    }

    #[test]
    fn nsloctext_extra_whitespace_around_commas() {
        // Some game data has extra spaces; the parser must tolerate them.
        let input = "NSLOCTEXT(\"NS\" ,  \"Key\"  ,  \"Value\")";
        let (rest, ns) = parse_nsloctext(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(ns.namespace, "NS");
        assert_eq!(ns.key, "Key");
        assert_eq!(ns.display, "Value");
    }

    #[test]
    fn nsloctext_empty_display_string() {
        let input = r#"NSLOCTEXT("NS", "Key", "")"#;
        let (rest, ns) = parse_nsloctext(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(ns.display, "");
    }

    #[test]
    fn nsloctext_reject_incomplete_line() {
        let input = r#"NSLOCTEXT("NS", "#;
        let res = parse_nsloctext(input);
        assert!(res.is_err(), "incomplete NSLOCTEXT should fail parsing");
    }

    #[test]
    fn fname_basic_stat_key() {
        let input = r#"(Value="BasePickaxeMeleeDamage_+%")"#;
        let (rest, prop) = parse_fname_property(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(prop.key, "Value");
        assert_eq!(prop.value, "BasePickaxeMeleeDamage_+%");
    }

    #[test]
    fn fname_property_with_spaces_around_equals() {
        let input = r#"(Value = "BasePickaxeMeleeDamage_+%")"#;
        let (rest, prop) = parse_fname_property(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(prop.value, "BasePickaxeMeleeDamage_+%");
        assert_eq!(prop.key, "Value")
    }

    #[test]
    fn fname_empty_value() {
        let input = r#"(Value="")"#;
        let (rest, prop) = parse_fname_property(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(prop.value, "");
    }

    #[test]
    fn fname_different_property_name() {
        let input = r#"(RowName="Stone_Tools_Reroute")"#;
        let (rest, prop) = parse_fname_property(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(prop.key, "RowName");
        assert_eq!(prop.value, "Stone_Tools_Reroute");
    }

    #[test]
    fn fname_reject_incomplete_line() {
        let input = r#"(RowName="Foobar"#;
        let res = parse_fname_property(input);
        assert!(res.is_err(), "incomplete FNameProperty should fail parsing");
    }
}
