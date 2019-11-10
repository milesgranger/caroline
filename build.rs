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
    #[serde(alias = "Type")]
    type_: Option<String>,
    #[serde(alias = "ItemType")]
    item_type: Option<String>,
    #[serde(alias = "PrimitiveItemType")]
    primitive_item_type: Option<PrimitiveType>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PropertyType {
    #[serde(alias = "Documentation", default)]
    documentation: String,
    #[serde(alias = "Properties", default)]
    properties: HashMap<String, Property>,
}

type PropertyTypes = HashMap<String, PropertyType>;

pub struct TypeMetadata {
    pub module_path: Vec<String>,
    pub struct_name: String,
    pub is_sub_property: bool,
}

impl<'a> From<&'a str> for TypeMetadata {
    fn from(path: &str) -> Self {
        /*
        Can get this: AWS::EMR::Cluster or something like this: AWS::EMR::Cluster.VolumeSpecification

        Both should have the module path as AWS::EMR::Cluster
        */

        let s = path.split("::").collect::<Vec<&str>>();

        let module_path: Vec<String> = s[0..s.len()]
            .iter()
            .map(|module| {
                // If it's the last element, ie. EMR::Cluster.VolumeSpecification
                // we want the module name for this section to be 'Cluster', this is the module
                // `VolumeSpecification` will be defined.
                if module.contains('.') {
                    module.split('.').take(1).last().unwrap().to_string()
                } else {
                    module.to_string()
                }
            })
            .collect();
        let struct_name = s[s.len() - 1].split('.').last().unwrap().to_string();

        Self {
            module_path,
            struct_name,
            is_sub_property: path.contains('.'),
        }
    }
}

/// Get the last segment of the module path, which is the struct name.

pub fn build_property_types(prop_types: &PropertyTypes) -> Module {
    let mut parent_module = Module::new("AWS")
        .set_is_pub(true)
        .add_attribute("#![allow(unused_imports, non_snake_case)]")
        .to_owned();

    prop_types.iter().for_each(|(prop_type_name, prop_type)| {
        let meta = TypeMetadata::from(prop_type_name.as_str());

        let mut strct = Struct::new(&meta.struct_name)
            .set_is_pub(true)
            .add_attribute("#[derive(Default, Clone)]")
            .to_owned();

        // implement new(...) method
        let mut new_method = Function::new("new")
            .set_is_pub(true)
            .set_return_ty("Self")
            .add_doc(format!("/// Create a new `{}`", &meta.struct_name))
            .to_owned();

        let mut new_method_body = "Self { ".to_string();

        let inner_self = prop_type
            .properties
            .iter()
            .map(|(prop_name, prop)| {
                let mut type_ = match prop.type_.as_ref().map(|v| v.as_str()) {
                    Some("List") => format!(
                        "Vec<{}>",
                        prop.item_type
                            .as_ref()
                            .map(|v| v.as_str())
                            .unwrap_or_else(|| {
                                prop.primitive_item_type
                                    .as_ref()
                                    .map(|v| v.as_rust_ty())
                                    .unwrap_or("String")
                            })
                    ),
                    Some("Map") => format!(
                        "HashMap<String, {}>",
                        prop.item_type
                            .as_ref()
                            .map(|v| v.as_str())
                            .unwrap_or_else(|| {
                                prop.primitive_item_type
                                    .as_ref()
                                    .map(|v| v.as_rust_ty())
                                    .unwrap_or("String")
                            })
                    ),
                    Some(a) => a.to_string(),
                    None => prop.primitive_type.as_rust_ty().to_string(),
                };

                // If this param is not required.
                if !prop.required {
                    type_ = format!("Option<{}>", type_);
                }

                strct.add_field(
                    Field::new(prop_name, &type_)
                        .set_is_pub(true)
                        .add_doc(format!(
                            "/// Official documentation: [{}]({})",
                            prop.documentation, prop.documentation
                        ))
                        .to_owned(),
                );
                new_method.add_parameter(Parameter::new(prop_name, &type_));

                prop_name.as_str()
            })
            .collect::<Vec<&str>>();

        new_method_body.push_str(&inner_self.join(", "));
        new_method_body.push_str("}");
        new_method.set_body(new_method_body);

        // Get or create the nested submodule this struct should live.
        let module = meta
            .module_path
            .iter()
            .fold(&mut parent_module, |md, mod_name| {
                match md.get_submodule(mod_name).is_some() {
                    true => md.get_submodule_mut(mod_name).unwrap(),
                    false => {
                        let mut m = Module::new(mod_name.clone())
                            .add_use_statement("use serde_json::Value;")
                            .add_use_statement("use std::collections::HashMap;")
                            .set_is_pub(true)
                            .to_owned();

                        // `Tag` struct is special
                        if &meta.struct_name != "Tag" {
                            m.add_use_statement("use crate::AWS::Tag::Tag;");
                        }

                        md.add_submodule(m);
                        md.get_submodule_mut(mod_name).unwrap()
                    }
                }
            });

        module.add_struct(strct).add_impl(
            Impl::new(meta.struct_name)
                .add_function(new_method)
                .to_owned(),
        );
    });

    parent_module
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

    let module = build_property_types(&property_types);

    let src_code = module.generate();

    let _ = fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/src", "/aws.rs"),
        src_code,
    )
    .unwrap();
}
