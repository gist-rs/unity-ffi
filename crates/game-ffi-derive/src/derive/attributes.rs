//! Attribute parsing for GameComponent derive macro
//!
//! This module provides utilities for parsing struct and field attributes
//! used by the GameComponent derive macro.

use syn::{parse::ParseStream, Attribute, Expr, ExprLit, Ident, Lit, Meta, MetaNameValue};

/// Hash mode for UUID generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HashMode {
    /// Default: hash struct name + fields + breaking attributes
    #[default]
    Default,
    /// All: hash full signature including all attributes (strict)
    All,
    /// Name: hash struct name only (loose, for prototyping)
    Name,
}

/// Parsed struct-level attributes
#[derive(Debug, Default, Clone)]
pub struct StructAttributes {
    /// UUID for the type (optional, auto-generated if not provided)
    pub uuid: Option<String>,
    /// Hash mode for UUID generation
    pub hash_mode: HashMode,
    /// Unity-specific configuration
    pub unity: Option<UnityConfig>,
    /// Unreal-specific configuration
    pub unreal: Option<UnrealConfig>,
    /// Whether to skip validation generation
    pub skip_validation: bool,
    /// Whether to skip Default implementation generation
    pub skip_default: bool,
    /// Whether to skip zero-copy implementation generation
    pub skip_zero_copy: bool,
}

/// Unity-specific configuration for a struct
#[derive(Debug, Default, Clone)]
pub struct UnityConfig {
    /// Custom name in Unity (defaults to struct name)
    pub name: Option<String>,
    /// Whether the struct is read-only in Unity
    pub read_only: bool,
}

/// Unreal-specific configuration for a struct
#[derive(Debug, Default, Clone)]
pub struct UnrealConfig {
    /// Custom class name in Unreal (defaults to struct name)
    pub class: Option<String>,
    /// Whether the struct should be BlueprintType
    pub blueprint_type: bool,
}

/// Parsed field-level attributes
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when field-level attributes are implemented
#[derive(Default)]
pub struct FieldAttributes {
    /// Whether to skip this field from public API
    pub skip: bool,
    /// Minimum value validation
    pub min: Option<Expr>,
    /// Maximum value validation
    pub max: Option<Expr>,
    /// Breaking attributes that affect UUID generation
    pub breaking_attributes: Vec<String>,
    /// Unity-specific configuration
    pub unity: Option<FieldUnityConfig>,
    /// Unreal-specific configuration
    pub unreal: Option<FieldUnrealConfig>,
}

/// Unity-specific configuration for a field
#[derive(Debug, Default, Clone)]
#[allow(dead_code)] // Will be used when field-level attributes are implemented
pub struct FieldUnityConfig {
    /// Custom field name in Unity
    pub name: Option<String>,
    /// Whether this is a header field (for inspector ordering)
    pub header_field: bool,
}

/// Unreal-specific configuration for a field
#[derive(Debug, Default, Clone)]
#[allow(dead_code)] // Will be used when field-level attributes are implemented
pub struct FieldUnrealConfig {
    /// Custom field name in Unreal
    pub name: Option<String>,
    /// Whether this field is replicated over network
    pub replicated: bool,
    /// Edit mode: InstanceOnly, DefaultOnly, or Anywhere
    pub edit_mode: Option<String>,
}

/// Parse struct-level attributes
pub fn parse_struct_attributes(attrs: &[Attribute]) -> Result<StructAttributes, syn::Error> {
    let mut result = StructAttributes::default();

    for attr in attrs {
        if attr.path().is_ident("uuid") {
            // #[uuid = "..."]
            result.uuid = Some(parse_string_attr(attr)?);
        } else if attr.path().is_ident("hash") {
            // #[hash = "all"] or #[hash = "name"]
            result.hash_mode = parse_hash_mode(attr)?;
        } else if attr.path().is_ident("unity") {
            // #[unity(name = "...", read_only)]
            result.unity = Some(parse_unity_config(attr)?);
        } else if attr.path().is_ident("unreal") {
            // #[unreal(class = "...", blueprint_type)]
            result.unreal = Some(parse_unreal_config(attr)?);
        } else if attr.path().is_ident("game_ffi") {
            // #[game_ffi(skip_validation, skip_default)]
            if let Meta::List(list) = &attr.meta {
                let _ = list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
                    while !input.is_empty() {
                        let ident: Ident = input.parse()?;
                        if ident == "skip_validation" {
                            result.skip_validation = true;
                        } else if ident == "skip_default" {
                            result.skip_default = true;
                        } else if ident == "skip_zero_copy" {
                            result.skip_zero_copy = true;
                        }
                        if !input.is_empty() {
                            input.parse::<syn::Token![,]>()?;
                        }
                    }
                    Ok(())
                });
            }
        }
    }

    Ok(result)
}

/// Parse a string attribute value
fn parse_string_attr(attr: &Attribute) -> Result<String, syn::Error> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }),
            ..
        }) => Ok(s.value()),
        _ => Err(syn::Error::new_spanned(attr, "Expected string value")),
    }
}

/// Parse hash mode attribute
fn parse_hash_mode(attr: &Attribute) -> Result<HashMode, syn::Error> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }),
            ..
        }) => match s.value().as_str() {
            "all" => Ok(HashMode::All),
            "name" => Ok(HashMode::Name),
            "default" => Ok(HashMode::Default),
            other => Err(syn::Error::new_spanned(
                s,
                format!(
                    "Invalid hash mode '{}'. Expected 'all', 'name', or 'default'",
                    other
                ),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            attr,
            "Expected hash attribute value, e.g., #[hash = \"all\"]",
        )),
    }
}

/// Parse Unity config attribute
fn parse_unity_config(attr: &Attribute) -> Result<UnityConfig, syn::Error> {
    let mut config = UnityConfig::default();

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                let key: Ident = input.parse()?;

                if key == "read_only" {
                    config.read_only = true;
                    if !input.is_empty() {
                        input.parse::<syn::Token![,]>()?;
                    }
                } else {
                    input.parse::<syn::Token![=]>()?;

                    if key == "name" {
                        let value: syn::Lit = input.parse()?;
                        if let Lit::Str(s) = value {
                            config.name = Some(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &value,
                                "Expected string value for name",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(&key, "Unknown Unity config key"));
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    Ok(config)
}

/// Parse Unreal config attribute
fn parse_unreal_config(attr: &Attribute) -> Result<UnrealConfig, syn::Error> {
    let mut config = UnrealConfig::default();

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                let key: Ident = input.parse()?;

                if key == "blueprint_type" {
                    config.blueprint_type = true;
                    if !input.is_empty() {
                        input.parse::<syn::Token![,]>()?;
                    }
                } else {
                    input.parse::<syn::Token![=]>()?;

                    if key == "class" {
                        let value: syn::Lit = input.parse()?;
                        if let Lit::Str(s) = value {
                            config.class = Some(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &value,
                                "Expected string value for class",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(&key, "Unknown Unreal config key"));
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    Ok(config)
}

/// Parse field-level attributes
#[allow(dead_code)] // Will be used when field-level attributes are implemented
pub fn parse_field_attributes(attrs: &[Attribute]) -> Result<FieldAttributes, syn::Error> {
    let mut result = FieldAttributes::default();

    for attr in attrs {
        if attr.path().is_ident("field") {
            // #[field(skip, min = 0, max = 100)]
            parse_field_config(attr, &mut result)?;
        } else if attr.path().is_ident("unity") {
            // #[unity(name = "...", header_field)]
            result.unity = Some(parse_field_unity_config(attr)?);
        } else if attr.path().is_ident("unreal") {
            // #[unreal(name = "...", replicated)]
            result.unreal = Some(parse_field_unreal_config(attr)?);
        }
    }

    Ok(result)
}

/// Parse field config attribute
#[allow(dead_code)] // Will be used when field-level attributes are implemented
fn parse_field_config(attr: &Attribute, result: &mut FieldAttributes) -> Result<(), syn::Error> {
    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                let key: Ident = input.parse()?;

                if key == "skip" {
                    result.skip = true;
                    // skip is a breaking attribute
                    result.breaking_attributes.push("skip".to_string());
                    if !input.is_empty() {
                        input.parse::<syn::Token![,]>()?;
                    }
                } else {
                    input.parse::<syn::Token![=]>()?;

                    if key == "min" {
                        result.min = Some(input.parse()?);
                    } else if key == "max" {
                        result.max = Some(input.parse()?);
                    } else {
                        return Err(syn::Error::new_spanned(&key, "Unknown field config key"));
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    Ok(())
}

/// Parse field Unity config
#[allow(dead_code)] // Will be used when field-level attributes are implemented
fn parse_field_unity_config(attr: &Attribute) -> Result<FieldUnityConfig, syn::Error> {
    let mut config = FieldUnityConfig::default();

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                let key: Ident = input.parse()?;

                if key == "header_field" {
                    config.header_field = true;
                    if !input.is_empty() {
                        input.parse::<syn::Token![,]>()?;
                    }
                } else {
                    input.parse::<syn::Token![=]>()?;

                    if key == "name" {
                        let value: syn::Lit = input.parse()?;
                        if let Lit::Str(s) = value {
                            config.name = Some(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &value,
                                "Expected string value for name",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            &key,
                            "Unknown field Unity config key",
                        ));
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    Ok(config)
}

/// Parse field Unreal config
#[allow(dead_code)] // Will be used when field-level attributes are implemented
fn parse_field_unreal_config(attr: &Attribute) -> Result<FieldUnrealConfig, syn::Error> {
    let mut config = FieldUnrealConfig::default();

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                let key: Ident = input.parse()?;

                if key == "replicated" {
                    config.replicated = true;
                    if !input.is_empty() {
                        input.parse::<syn::Token![,]>()?;
                    }
                } else {
                    input.parse::<syn::Token![=]>()?;

                    if key == "name" {
                        let value: syn::Lit = input.parse()?;
                        if let Lit::Str(s) = value {
                            config.name = Some(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &value,
                                "Expected string value for name",
                            ));
                        }
                    } else if key == "edit_mode" {
                        let value: syn::Lit = input.parse()?;
                        if let Lit::Str(s) = value {
                            config.edit_mode = Some(s.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                &value,
                                "Expected string value for edit_mode",
                            ));
                        }
                    } else {
                        return Err(syn::Error::new_spanned(
                            &key,
                            "Unknown field Unreal config key",
                        ));
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    Ok(config)
}
