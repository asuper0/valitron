//! register rules

use std::collections::HashMap;

use crate::{
    rule::{IntoRuleList, RuleList},
    ser::{Serializer, ValueMap},
};

mod field_name;
mod lexer;

pub use field_name::FieldName;

#[derive(Default)]
pub struct Validator<'a> {
    rules: HashMap<Vec<FieldName>, RuleList>,
    message: HashMap<(Vec<FieldName>, String), &'a str>,
}

macro_rules! panic_on_err {
    ($expr:expr) => {
        match $expr {
            Ok(x) => x,
            Err(err) => panic!("{err}"),
        }
    };
}

impl<'a> Validator<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn rule<R: IntoRuleList>(mut self, field: &'a str, rule: R) -> Self {
        let names = panic_on_err!(field_name::parse(field));
        self.rules.insert(names, rule.into_list());
        self
    }

    pub fn message<const N: usize>(mut self, list: [(&'a str, &'a str); N]) -> Self {
        self.message = HashMap::from_iter(
            list.map(|(k_str, v)| {
                let key = panic_on_err!(field_name::parse_message(k_str));
                panic_on_err!(self.exit_message(&k_str, &key));
                (key, v)
            })
            .into_iter(),
        );
        self
    }

    pub fn validate<T>(self, data: T) -> Result<(), Vec<(Vec<FieldName>, Vec<String>)>>
    where
        T: serde::ser::Serialize,
    {
        let value = data.serialize(Serializer).unwrap();
        let mut value_map: ValueMap = ValueMap::new(value);
        let mut message = Vec::new();

        for (names, rules) in self.rules.iter() {
            value_map.index(names.clone());
            let rule_resp = rules.clone().call(&mut value_map);

            let mut field_msg = Vec::new();
            for (rule, msg) in rule_resp.into_iter() {
                let final_msg = match self.get_message(&(names.clone(), rule.to_string())) {
                    Some(s) => s.to_string(),
                    None => msg,
                };
                field_msg.push(final_msg);
            }

            if !field_msg.is_empty() {
                message.push((names.clone(), field_msg));
            }
        }

        if message.is_empty() {
            Ok(())
        } else {
            Err(message)
        }
    }

    fn rule_get(&self, names: &Vec<FieldName>) -> Option<&RuleList> {
        self.rules.get(names)
    }

    fn rules_name(&self, names: &Vec<FieldName>) -> Option<Vec<&'static str>> {
        self.rule_get(names).map(|rule| rule.get_rules_name())
    }

    fn exit_message(
        &self,
        k_str: &str,
        (names, rule_name): &(Vec<FieldName>, String),
    ) -> Result<(), String> {
        let point_index = k_str
            .rfind('.')
            .ok_or(format!("no found `.` in the message index"))?;
        let names = self.rules_name(names).ok_or(format!(
            "the field \"{}\" not found in validator",
            &k_str[..point_index]
        ))?;

        if names.contains(&rule_name.as_str()) {
            Ok(())
        } else {
            Err(format!("rule \"{rule_name}\" is not found in rules"))
        }
    }

    fn get_message(&self, key: &(Vec<FieldName>, String)) -> Option<&&str> {
        self.message.get(key)
    }
}

// #[test]
// fn test_message() {
//     let ruler = Ruler::new().message([
//         ("name.required", "name mut not be null"),
//         ("password.password", "password mut not very simple"),
//     ]);
// }
