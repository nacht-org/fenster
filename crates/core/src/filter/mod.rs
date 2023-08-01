use serde::Deserialize;
use std::collections::BTreeMap;

mod fields;

pub use fields::{Check, Choice, ChoiceField, FieldGroup, RangeField, SelectField, TextField};

type VerifyResult = Result<(), String>;

pub trait InputField {
    type Type;
    fn verify_input(&self, value: &Self::Type) -> VerifyResult;
}

pub type FieldMap = BTreeMap<String, Field>;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Field {
    Text(fields::TextField),
    Select(fields::SelectField),
    Range(fields::RangeField),
    Choice(fields::ChoiceField),
    Group(fields::FieldGroup<FieldMap>),
}
