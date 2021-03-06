use crate::{
    compilation::{compile_validators, context::CompilationContext, JSONSchema},
    error::{error, no_error, CompilationError, ErrorIterator, ValidationError},
    keywords::{format_validators, CompilationResult, Validators},
    validator::Validate,
};
use regex::Regex;
use serde_json::{Map, Value};
use std::{collections::BTreeSet, iter::FromIterator};

pub(crate) struct AdditionalPropertiesValidator {
    validators: Validators,
}
impl AdditionalPropertiesValidator {
    #[inline]
    pub(crate) fn compile(schema: &Value, context: &CompilationContext) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesValidator {
            validators: compile_validators(schema, context)?,
        }))
    }
}
impl Validate for AdditionalPropertiesValidator {
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.validators.iter().all(move |validator| {
            instance_value
                .values()
                .all(move |value| validator.is_valid(schema, value))
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.validators
                .iter()
                .flat_map(move |validator| {
                    instance_value
                        .values()
                        .flat_map(move |value| validator.validate(schema, value))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesValidator {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub(crate) struct AdditionalPropertiesFalseValidator {}
impl AdditionalPropertiesFalseValidator {
    #[inline]
    pub(crate) fn compile() -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesFalseValidator {}))
    }
}
impl Validate for AdditionalPropertiesFalseValidator {
    #[inline]
    fn build_validation_error<'a>(&self, instance: &'a Value) -> ValidationError<'a> {
        ValidationError::false_schema(instance)
    }

    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value.is_empty()
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }
}
impl ToString for AdditionalPropertiesFalseValidator {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

pub(crate) struct AdditionalPropertiesNotEmptyFalseValidator {
    properties: BTreeSet<String>,
}
impl AdditionalPropertiesNotEmptyFalseValidator {
    #[inline]
    pub(crate) fn compile(properties: &Value) -> CompilationResult {
        if let Value::Object(properties) = properties {
            Ok(Box::new(AdditionalPropertiesNotEmptyFalseValidator {
                properties: BTreeSet::from_iter(properties.keys().cloned()),
            }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}
impl Validate for AdditionalPropertiesNotEmptyFalseValidator {
    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value
            .keys()
            .all(|property| self.properties.contains(property))
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        _: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        instance_value
            .keys()
            .filter_map(|property| {
                if self.properties.contains(property) {
                    None
                } else {
                    // No extra properties are allowed
                    let property_value = Value::String(property.to_string());
                    Some(error(
                        ValidationError::false_schema(&property_value).into_owned(),
                    ))
                }
            })
            .next()
            .unwrap_or_else(no_error)
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesNotEmptyFalseValidator {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

pub(crate) struct AdditionalPropertiesNotEmptyValidator {
    validators: Validators,
    properties: BTreeSet<String>,
}
impl AdditionalPropertiesNotEmptyValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        properties: &Value,
        context: &CompilationContext,
    ) -> CompilationResult {
        if let Value::Object(properties) = properties {
            Ok(Box::new(AdditionalPropertiesNotEmptyValidator {
                properties: BTreeSet::from_iter(properties.keys().cloned()),
                validators: compile_validators(schema, context)?,
            }))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}
impl Validate for AdditionalPropertiesNotEmptyValidator {
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.validators.iter().all(move |validator| {
            instance_value
                .iter()
                .filter(move |(property, _)| !self.properties.contains(*property))
                .all(move |(_, value)| validator.is_valid(schema, value))
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.validators
                .iter()
                .flat_map(move |validator| {
                    instance_value
                        .iter()
                        .filter(move |(property, _)| !self.properties.contains(*property))
                        .flat_map(move |(_, value)| validator.validate(schema, value))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesNotEmptyValidator {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub(crate) struct AdditionalPropertiesWithPatternsValidator {
    validators: Validators,
    pattern: Regex,
}
impl AdditionalPropertiesWithPatternsValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        pattern: Regex,
        context: &CompilationContext,
    ) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesWithPatternsValidator {
            validators: compile_validators(schema, context)?,
            pattern,
        }))
    }
}
impl Validate for AdditionalPropertiesWithPatternsValidator {
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.validators.iter().all(move |validator| {
            instance_value
                .iter()
                .filter(move |(property, _)| !self.pattern.is_match(property))
                .all(move |(_, value)| validator.is_valid(schema, value))
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.validators
                .iter()
                .flat_map(move |validator| {
                    instance_value
                        .iter()
                        .filter(move |(property, _)| !self.pattern.is_match(property))
                        .flat_map(move |(_, value)| validator.validate(schema, value))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesWithPatternsValidator {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub(crate) struct AdditionalPropertiesWithPatternsFalseValidator {
    pattern: Regex,
}
impl AdditionalPropertiesWithPatternsFalseValidator {
    #[inline]
    pub(crate) fn compile(pattern: Regex) -> CompilationResult {
        Ok(Box::new(AdditionalPropertiesWithPatternsFalseValidator {
            pattern,
        }))
    }
}
impl Validate for AdditionalPropertiesWithPatternsFalseValidator {
    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value
            .keys()
            .all(|property| self.pattern.is_match(property))
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        _: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        instance_value
            .keys()
            .find(|property| !self.pattern.is_match(property))
            .map_or_else(no_error, |property| {
                error(
                    ValidationError::false_schema(&Value::String(property.to_string()))
                        .into_owned(),
                )
            })
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesWithPatternsFalseValidator {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

pub(crate) struct AdditionalPropertiesWithPatternsNotEmptyValidator {
    validators: Validators,
    properties: BTreeSet<String>,
    pattern: Regex,
}
impl AdditionalPropertiesWithPatternsNotEmptyValidator {
    #[inline]
    pub(crate) fn compile(
        schema: &Value,
        properties: &Value,
        pattern: Regex,
        context: &CompilationContext,
    ) -> CompilationResult {
        if let Value::Object(properties) = properties {
            Ok(Box::new(
                AdditionalPropertiesWithPatternsNotEmptyValidator {
                    validators: compile_validators(schema, context)?,
                    properties: BTreeSet::from_iter(properties.keys().cloned()),
                    pattern,
                },
            ))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}
impl Validate for AdditionalPropertiesWithPatternsNotEmptyValidator {
    #[inline]
    fn is_valid_object(
        &self,
        schema: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        self.validators.iter().all(move |validator| {
            instance_value
                .iter()
                .filter(move |(property, _)| {
                    !self.properties.contains(*property) && !self.pattern.is_match(property)
                })
                .all(move |(_, value)| validator.is_valid(schema, value))
        })
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        schema: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        Box::new(
            self.validators
                .iter()
                .flat_map(move |validator| {
                    instance_value
                        .iter()
                        .filter(move |(property, _)| {
                            !(self.properties.contains(*property)
                                || self.pattern.is_match(property))
                        })
                        .flat_map(move |(_, value)| validator.validate(schema, value))
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesWithPatternsNotEmptyValidator {
    fn to_string(&self) -> String {
        format!(
            "additionalProperties: {}",
            format_validators(&self.validators)
        )
    }
}

pub(crate) struct AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    properties: BTreeSet<String>,
    pattern: Regex,
}
impl AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    #[inline]
    pub(crate) fn compile(properties: &Value, pattern: Regex) -> CompilationResult {
        if let Value::Object(properties) = properties {
            Ok(Box::new(
                AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
                    properties: BTreeSet::from_iter(properties.keys().cloned()),
                    pattern,
                },
            ))
        } else {
            Err(CompilationError::SchemaError)
        }
    }
}
impl Validate for AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    #[inline]
    fn is_valid_object(
        &self,
        _: &JSONSchema,
        _: &Value,
        instance_value: &Map<String, Value>,
    ) -> bool {
        instance_value
            .keys()
            .all(|property| self.properties.contains(property) || self.pattern.is_match(property))
    }
    #[inline]
    fn is_valid(&self, schema: &JSONSchema, instance: &Value) -> bool {
        if let Value::Object(instance_value) = instance {
            self.is_valid_object(schema, instance, instance_value)
        } else {
            true
        }
    }

    #[inline]
    fn validate_object<'a>(
        &self,
        _: &'a JSONSchema,
        _: &'a Value,
        instance_value: &'a Map<String, Value>,
    ) -> ErrorIterator<'a> {
        instance_value
            .keys()
            .find(|property| {
                !self.properties.contains(*property) && !self.pattern.is_match(property)
            })
            .map_or_else(no_error, |property| {
                error(
                    ValidationError::false_schema(&Value::String(property.to_string()))
                        .into_owned(),
                )
            })
    }
    #[inline]
    fn validate<'a>(&self, schema: &'a JSONSchema, instance: &'a Value) -> ErrorIterator<'a> {
        if let Value::Object(instance_value) = instance {
            self.validate_object(schema, instance, instance_value)
        } else {
            no_error()
        }
    }
}
impl ToString for AdditionalPropertiesWithPatternsNotEmptyFalseValidator {
    fn to_string(&self) -> String {
        "additionalProperties: false".to_string()
    }
}

#[inline]
pub(crate) fn compile(
    parent: &Map<String, Value>,
    schema: &Value,
    context: &CompilationContext,
) -> Option<CompilationResult> {
    let properties = parent.get("properties");
    if let Some(patterns) = parent.get("patternProperties") {
        if let Value::Object(obj) = patterns {
            let pattern = obj.keys().cloned().collect::<Vec<String>>().join("|");
            match Regex::new(&pattern) {
                Ok(re) => {
                    match schema {
                        Value::Bool(true) => None, // "additionalProperties" are "true" by default
                        Value::Bool(false) => match properties {
                            Some(properties) => Some(
                                AdditionalPropertiesWithPatternsNotEmptyFalseValidator::compile(
                                    properties, re,
                                ),
                            ),
                            None => {
                                Some(AdditionalPropertiesWithPatternsFalseValidator::compile(re))
                            }
                        },
                        _ => match properties {
                            Some(properties) => {
                                Some(AdditionalPropertiesWithPatternsNotEmptyValidator::compile(
                                    schema, properties, re, context,
                                ))
                            }
                            None => Some(AdditionalPropertiesWithPatternsValidator::compile(
                                schema, re, context,
                            )),
                        },
                    }
                }
                Err(_) => Some(Err(CompilationError::SchemaError)),
            }
        } else {
            Some(Err(CompilationError::SchemaError))
        }
    } else {
        match schema {
            Value::Bool(true) => None, // "additionalProperties" are "true" by default
            Value::Bool(false) => match properties {
                Some(properties) => Some(AdditionalPropertiesNotEmptyFalseValidator::compile(
                    properties,
                )),
                None => Some(AdditionalPropertiesFalseValidator::compile()),
            },
            _ => match properties {
                Some(properties) => Some(AdditionalPropertiesNotEmptyValidator::compile(
                    schema, properties, context,
                )),
                None => Some(AdditionalPropertiesValidator::compile(schema, context)),
            },
        }
    }
}
