use serde::{Deserialize, Serialize};
use validator::{
    register::FieldName,
    rule::{custom, Required, RuleExt, StartWith},
    ser::Value,
    Validator,
};

#[derive(Serialize, Deserialize, Debug)]
struct Person {
    name: &'static str,
    age: u8,
}

#[test]
fn test_validator() {
    let validator = Validator::new()
        .rule("name", Required.and(StartWith("hello")))
        .rule("age", custom(age_limit))
        .message([
            ("name.required", "name is required"),
            ("name.start_with", "name should be starts with `hello`"),
        ]);

    let person = Person {
        name: "li",
        age: 18,
    };

    let res = validator.validate(person).unwrap_err();

    assert!(res.len() == 2);
    assert!(res.contains(&(
        vec![FieldName::Literal("age".into())].into(),
        vec!["age should be between 25 and 45".to_string()],
    )));
    assert!(res.contains(&(
        vec![FieldName::Literal("name".into())].into(),
        vec!["name should be starts with `hello`".to_string()],
    )));

    //println!("{res:?}");
}

fn age_limit(v: &mut Value) -> Result<(), String> {
    if let Value::Int8(n) = v {
        if *n >= 25 && *n <= 45 {
            return Ok(());
        }
    }
    Err("age should be between 25 and 45".to_owned())
}

#[test]
fn test_has_tuple() {
    let validator = Validator::new()
        .rule(0, StartWith("hello"))
        .message([("0.start_with", "first item should be start with `hello`")]);

    #[derive(Serialize, Deserialize, Debug)]
    struct Foo(&'static str, &'static str);

    let res = validator.validate(Foo("heoo", "bar")).unwrap_err();
    assert!(res.len() == 1);

    assert!(res.contains(&(
        vec![FieldName::Tuple(0)].into(),
        vec!["first item should be start with `hello`".to_string()],
    )));
}

#[test]
fn test_has_array() {
    let validator = Validator::new().rule([1], StartWith("hello"));

    let res = validator.validate(vec!["foo", "bar"]).unwrap_err();

    assert!(res.len() == 1);
    assert!(res.contains(&(
        vec![FieldName::Array(1)].into(),
        vec!["this field must be start with {}".to_string()],
    )));
}
