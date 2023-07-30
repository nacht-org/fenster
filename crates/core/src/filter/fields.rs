use serde::{Deserialize, Serialize};

use super::{InputField, VerifyResult};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "text", tag = "type")]
pub struct TextField {
    pub title: String,
}

impl InputField for TextField {
    type Type = String;

    fn verify_input(&self, _value: &Self::Type) -> VerifyResult {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "select", tag = "type")]
pub struct SelectField {
    pub title: String,
    pub items: Vec<Check>,
}

#[derive(Deserialize, Debug)]
pub struct SelectResult {
    pub value: String,
    #[serde(default)]
    pub remove: bool,
}

impl InputField for SelectField {
    type Type = Vec<SelectResult>;

    fn verify_input(&self, value: &Self::Type) -> VerifyResult {
        for result in value {
            if let None = self.items.iter().find(|item| &item.value == &result.value) {
                return Err(format!("option not found {}", &result.value));
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Check {
    pub label: String,
    pub value: String,
    pub tri: bool,
}

impl Check {
    pub fn new<L, V>(label: L, value: V, tri: bool) -> Self
    where
        L: ToString,
        V: ToString,
    {
        Check {
            label: label.to_string(),
            value: value.to_string(),
            tri,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "range", tag = "type")]
pub struct RangeField {
    pub title: String,
    pub min: f32,
    pub max: f32,
    pub div: f32,
}

#[derive(Deserialize, Debug)]
pub struct RangeResult {
    pub min: f32,
    pub max: f32,
}

impl InputField for RangeField {
    type Type = RangeResult;

    fn verify_input(&self, value: &Self::Type) -> VerifyResult {
        if value.min < self.min {
            Err(format!("min value must not be less than {}", self.min))
        } else if value.max > self.max {
            Err(format!("max value must not be greater than {}", self.max))
        } else if value.min > value.max {
            Err(String::from("min value must not be greater than max"))
        } else if value.min % self.div != 0.0 {
            Err(format!(
                "min value must must be divisible with {}",
                self.div
            ))
        } else if value.max % self.div != 0.0 {
            Err(format!(
                "max value must must be divisible with {}",
                self.div
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "choice", tag = "type")]
pub struct ChoiceField {
    pub title: String,
    pub items: Vec<Choice>,
}

impl InputField for ChoiceField {
    type Type = String;

    fn verify_input(&self, value: &Self::Type) -> VerifyResult {
        if let None = self.items.iter().find(|item| &item.value == value) {
            Err(format!("option not found {}", value))
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub label: String,
    pub value: String,
}

impl Choice {
    pub fn new<L, V>(label: L, value: V) -> Self
    where
        L: ToString,
        V: ToString,
    {
        Self {
            label: label.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "group", tag = "type")]
pub struct FieldGroup<T> {
    pub title: String,
    pub fields: T,
}

impl<T> InputField for FieldGroup<T>
where
    T: InputField,
{
    type Type = <T as InputField>::Type;

    fn verify_input(&self, value: &Self::Type) -> VerifyResult {
        self.fields.verify_input(value)
    }
}
