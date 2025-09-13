use log::{info, warn};
use serde::{ser::SerializeMap, Serialize, Serializer};
use serde_yaml::Value;
use std::{collections::HashMap, fs, path::Path};
use upon::{Engine, Syntax};

use crate::cli::error::DSDMError;
use crate::core::global::delims;

#[derive(Debug, Clone)]
pub enum TemplateContext {
    Value(String),
    Nested(HashMap<String, TemplateContext>),
}

impl Serialize for TemplateContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TemplateContext::Value(s) => serializer.serialize_str(s),
            TemplateContext::Nested(map) => {
                let mut ser_map = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    ser_map.serialize_entry(k, v)?;
                }
                SerializeMap::end(ser_map)
            }
        }
    }
}

/// Render (apply) a template given a context (see `TemplateContext`)
pub fn render_template<T>(input: &str, ctx: T) -> Result<String, DSDMError>
where
    T: Serialize,
{
    let dlm = delims()?;
    info!("using delimiters {:?}", dlm);
    let syntax = Syntax::builder().expr(&dlm.open, &dlm.close).build();
    let mut engine = Engine::with_syntax(syntax);

    engine.add_template("tpl", input)?;

    let result = engine.template("tpl").render(ctx).to_string()?;

    Ok(result)
}

/// Render (apply) a template for a file given a context (see `TemplateContext`)
pub fn render_template_file<F, T>(file: F, ctx: T) -> Result<String, DSDMError>
where
    F: AsRef<Path>,
    T: Serialize,
{
    let contents: String = fs::read_to_string(file)?;
    render_template(&contents, ctx)
}

/// Create a `TemplateContext` from a `templates` field deserialized from `mod.yaml`
pub fn build_context(
    template: Option<Value>,
    global: Result<Value, DSDMError>,
) -> Result<TemplateContext, DSDMError> {
    match template {
        Some(Value::Mapping(map)) => {
            let mut context_map = HashMap::new();

            if let Value::Mapping(global_map) = global? {
                info!("registering global templates");
                let global_entry = context_map
                    .entry("global".to_string())
                    .or_insert_with(|| TemplateContext::Nested(HashMap::new()));

                if let TemplateContext::Nested(global_nested) = global_entry {
                    for (k, v) in global_map {
                        let key = k.as_str().ok_or(DSDMError::InvalidKey)?;
                        let value = build_context(Some(v.clone()), Ok(Value::Null))?;
                        global_nested.insert(key.to_string(), value);
                    }
                }
            }

            for (k, v) in map {
                let key = k.as_str().ok_or(DSDMError::InvalidKey)?;
                let value = build_context(Some(v), Ok(Value::Null))?;

                if key.starts_with("global") {
                    warn!("local key contains `global`, this might overwrite global templates",);
                }

                context_map.insert(key.to_string(), value);
            }

            Ok(TemplateContext::Nested(context_map))
        }
        None => {
            info!("no local templates found");

            let mut context_map = HashMap::new();

            if let Value::Mapping(global_map) = global? {
                info!("registering global templates");
                let global_entry = context_map
                    .entry("global".to_string())
                    .or_insert_with(|| TemplateContext::Nested(HashMap::new()));

                if let TemplateContext::Nested(global_nested) = global_entry {
                    for (k, v) in global_map {
                        let key = k.as_str().ok_or(DSDMError::InvalidKey)?;
                        let value = build_context(Some(v.clone()), Ok(Value::Null))?;
                        global_nested.insert(key.to_string(), value);
                    }
                }
            }
            Ok(TemplateContext::Nested(context_map))
        }
        Some(Value::String(s)) => Ok(TemplateContext::Value(s.clone())),
        _ => Err(DSDMError::InvalidValue),
    }
}
