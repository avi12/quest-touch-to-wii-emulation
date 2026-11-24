use crate::session::json::{JsonObject, JsonValue, parse_flat_object};

/// Values and how `JSON.stringify` writes them.
const NUMBERS: [(f64, &str); 7] = [
    (0.13, "0.13"),
    (-12.34, "-12.34"),
    (2.0, "2"),
    (-0.0, "0"),
    (1_700_000_000_123.0, "1700000000123"),
    (f64::NAN, "null"),
    (f64::NEG_INFINITY, "null"),
];

const LIST: [f64; 2] = [1.0, -0.5];
const ESCAPED: &str = r#"{"te\"xt\\ \/\né":"a\tb"}"#;

#[test]
fn writes_numbers_like_javascript() {
    for (value, expected) in NUMBERS {
        let mut object = JsonObject::new();
        object.number("value", value);
        assert_eq!(object.finish(), format!("{{\"value\":{expected}}}"));
    }
}

#[test]
fn writes_members_in_order() {
    let mut object = JsonObject::new();
    object.numbers("list", &LIST);
    object.boolean("flag", true);
    object.numbers("empty", &[]);
    assert_eq!(
        object.finish(),
        r#"{"list":[1,-0.5],"flag":true,"empty":[]}"#
    );
    assert_eq!(JsonObject::new().finish(), "{}");
}

#[test]
fn reads_every_value_kind() {
    let members = parse_flat_object(
        r#"{"n":-1.5e2,"t":true,"f":false,"z":null,"a":[1,null],"e":[],"s":"x"}"#,
    )
    .unwrap();
    assert_eq!(members["n"], JsonValue::Number(-150.0));
    assert_eq!(members["t"], JsonValue::Bool(true));
    assert_eq!(members["f"], JsonValue::Bool(false));
    assert_eq!(members["z"], JsonValue::Null);
    assert!(
        matches!(&members["a"], JsonValue::Numbers(values) if values.len() == 2 && values[1].is_nan())
    );
    assert_eq!(members["e"], JsonValue::Numbers(Vec::new()));
    assert_eq!(members["s"], JsonValue::Text("x".to_owned()));
}

#[test]
fn reads_escapes_whitespace_and_repeats() {
    let members = parse_flat_object(ESCAPED).unwrap();
    assert_eq!(
        members["te\"xt\\ /\n\u{e9}"],
        JsonValue::Text("a\tb".to_owned())
    );
    assert!(parse_flat_object(" \t{ } \r\n").unwrap().is_empty());
    let repeated = parse_flat_object(r#"{"k":1,"k":2}"#).unwrap();
    assert_eq!(repeated["k"], JsonValue::Number(2.0));
}

#[test]
fn rejects_what_is_not_one_flat_object() {
    for line in [
        "{",
        "{}}",
        r#"{"k":1,}"#,
        r#"{"k" 1}"#,
        r"{k:1}",
        r#"{"k":tru}"#,
        r#"{"k":[1,]}"#,
        r#"{"k":[[1]]}"#,
        r#"{"k":{}}"#,
        r#"{"k":.5}"#,
        r#"{"k":"\x"}"#,
        r#"{"k":"\ud83d"}"#,
        r#"{"k":"open}"#,
    ] {
        assert_eq!(parse_flat_object(line), None, "{line}");
    }
}
