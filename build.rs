use proffer::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize)]
pub enum PrimitiveType {
    String,
    Boolean,
    Integer,
    Double,
    Timestamp,
    Json,
    Long,
}
impl PrimitiveType {
    pub fn as_rust_ty(&self) -> &str {
        match self {
            PrimitiveType::String => "String",
            PrimitiveType::Boolean => "bool",
            PrimitiveType::Integer => "i32",
            PrimitiveType::Double => "f32",
            PrimitiveType::Timestamp => "u32",
            PrimitiveType::Json => "Value",
            PrimitiveType::Long => "u32",
        }
    }
}
impl Default for PrimitiveType {
    fn default() -> PrimitiveType {
        PrimitiveType::String
    }
}

#[derive(Serialize, Deserialize)]
pub enum UpdateType {
    Mutable,
    Immutable,
    Conditional,
}
impl Default for UpdateType {
    fn default() -> UpdateType {
        UpdateType::Immutable
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Property {
    #[serde(alias = "Required")]
    required: bool,
    #[serde(alias = "Documentation", default)]
    documentation: String,
    #[serde(alias = "PrimitiveType", default)]
    primitive_type: PrimitiveType,
    #[serde(alias = "UpdateType")]
    update_type: UpdateType,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PropertyType {
    #[serde(alias = "Documentation", default)]
    documentation: String,
    #[serde(alias = "Properties", default)]
    properties: HashMap<String, Property>,
}

type PropertyTypes = HashMap<String, PropertyType>;

/// Get the last segment of the module path, which is the struct name.
pub fn struct_mod_n_name(path: &str) -> (String, String) {
    let s = path.split("::").collect::<Vec<&str>>();
    if s.len() > 2 {
        (
            s[s.len() - 2].replace('.', ""),
            s[s.len() - 1].replace('.', ""),
        )
    } else {
        ("DEFAULT".to_owned(), s[s.len() - 1].replace('.', ""))
    }
}

pub fn build_property_types(prop_types: &PropertyTypes) -> impl Iterator<Item = Module> + '_ {
    let mut modules: HashMap<String, Module> = HashMap::new();

    prop_types.iter().for_each(|(prop_type_name, prop_type)| {
        let (mod_name, struct_name) = struct_mod_n_name(prop_type_name);

        let mut _struct = Struct::new(struct_name).set_is_pub(true).to_owned();

        for (prop_name, prop) in &prop_type.properties {
            _struct.add_field(
                Field::new(prop_name, &prop.primitive_type.as_rust_ty().to_string())
                    .set_is_pub(true)
                    .add_doc(format!(
                        "/// Official documentation: [{}]({})",
                        prop.documentation, prop.documentation
                    ))
                    .to_owned(),
            );
        }

        let module = modules
            .entry(mod_name.clone())
            .or_insert(
                Module::new(mod_name.clone())
                    .add_use_statement("use serde_json::Value;")
                    .set_is_pub(true)
                    .to_owned(),
            )
            .add_struct(_struct)
            .to_owned();
        modules.insert(mod_name, module.to_owned());
    });
    modules.into_iter().map(|(_, m)| m)
}

fn main() {
    let spec_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/definitions",
        "/CloudFormationResourceSpecification.json"
    );
    let spec_bytes = fs::read(spec_path).unwrap();
    let spec: Value = serde_json::from_slice(&spec_bytes).unwrap();

    let property_types: PropertyTypes =
        serde_json::from_value(spec["PropertyTypes"].clone()).unwrap();

    let mut module = Module::new("AWS").set_is_pub(true).to_owned();

    for s in build_property_types(&property_types) {
        module.add_submodule(s);
    }

    let src_code = module.generate();

    let _ = fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/src", "/aws.rs"),
        src_code,
    )
    .unwrap();
}
