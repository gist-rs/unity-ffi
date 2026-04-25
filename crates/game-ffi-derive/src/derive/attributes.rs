//! Attribute parsing for GameComponent derive macro
//!
//! This module provides utilities for parsing struct and field attributes
//! used by the GameComponent derive macro.
//!
//! ## Plan 082: Schema-Driven Database Attributes
//!
//! The following attributes enable auto-generation of SQL DDL and CRUD:
//! - `#[db_table("table_name")]` — marks struct as a database table
//! - `#[primary_key]` — marks a field as the primary key
//! - `#[db_column(TYPE, CONSTRAINTS)]` — SQL column type and constraints
//! - `#[db_default("value")]` — database default value expression
//! - `#[db_index(name = "...", on = "...", condition = "...")]` — index definition
//! - `#[db_foreign_key(column, references = "table.col", on_delete = "...")]` — FK constraint
//! - `#[db_unique_constraint("name", columns = ["..."])]` — unique constraint

use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse::ParseStream, Attribute, Expr, ExprLit, Ident, Lit, LitBool, LitFloat, LitInt, LitStr,
    Meta, MetaNameValue,
};

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

// ============================================================================
// Plan 082: Database Schema Attribute Types
// ============================================================================

/// Parsed `#[db_table("name")]` attribute
#[derive(Debug, Clone)]
pub struct DbTableAttr {
    /// The database table name
    pub table_name: String,
}

/// Parsed `#[db_column(TYPE, CONSTRAINTS)]` attribute
#[derive(Debug, Clone)]
pub struct DbColumnAttr {
    /// SQL column type (e.g., "VARCHAR(255)", "BIGINT")
    pub sql_type: Option<String>,
    /// Additional constraints (e.g., "NOT NULL", "UNIQUE")
    pub constraints: Vec<String>,
}

/// Parsed `#[db_default("value")]` attribute
#[derive(Debug, Clone)]
pub enum DbDefaultAttr {
    /// String default value (e.g., "'general'", "NOW()")
    String(String),
    /// Numeric default value (e.g., 24, 0)
    Number(i64),
    /// Float default value (e.g., 125.0, 0.5)
    Float(f64),
    /// Boolean default value
    Bool(bool),
}

/// Parsed `#[db_index(name = "...", on = "...", condition = "...")]` attribute
#[derive(Debug, Clone)]
pub struct DbIndexAttr {
    /// Index name
    pub name: String,
    /// Column(s) to index on (single or composite)
    pub on: DbIndexColumns,
    /// Optional WHERE condition for partial index
    pub condition: Option<String>,
}

/// Columns for an index — single or composite
#[derive(Debug, Clone)]
pub enum DbIndexColumns {
    /// Single column index
    Single(String),
    /// Composite column index
    Composite(Vec<String>),
}

/// Parsed `#[db_foreign_key(column, references = "table.col", on_delete = "...")]` attribute
#[derive(Debug, Clone)]
pub struct DbForeignKeyAttr {
    /// The local column name
    pub column: String,
    /// The referenced table.column (e.g., "shops.id")
    pub references: String,
    /// On delete action (e.g., "CASCADE", "SET NULL")
    pub on_delete: Option<String>,
}

/// Parsed `#[db_unique_constraint("name", columns = ["..."])]` attribute
#[derive(Debug, Clone)]
pub struct DbUniqueConstraintAttr {
    /// Constraint name
    pub name: String,
    /// Column names in the constraint
    pub columns: Vec<String>,
}

// ============================================================================
// Struct and Field Attribute Types (Extended for Plan 082)
// ============================================================================

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
    /// Whether to skip FFI extern "C" function generation
    pub skip_ffi: bool,
    // Plan 082: Database schema attributes
    /// Database table name from `#[db_table("name")]`
    pub db_table: Option<DbTableAttr>,
    /// Table-level indexes from `#[db_index(...)]`
    pub db_indexes: Vec<DbIndexAttr>,
    /// Foreign key constraints from `#[db_foreign_key(...)]`
    pub db_foreign_keys: Vec<DbForeignKeyAttr>,
    /// Unique constraints from `#[db_unique_constraint(...)]`
    pub db_unique_constraints: Vec<DbUniqueConstraintAttr>,
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
    // Plan 082: Database schema field attributes
    /// Whether this field is a primary key
    pub primary_key: bool,
    /// SQL column type and constraints
    pub db_column: Option<DbColumnAttr>,
    /// Database default value
    pub db_default: Option<DbDefaultAttr>,
    /// Field-level index definition
    pub db_index: Option<DbIndexAttr>,
}

/// Unity-specific configuration for a field
#[derive(Debug, Default, Clone)]
#[allow(dead_code)] // Will be used when field-level attributes are implemented
pub struct FieldUnityConfig {
    /// Custom field name in Unity
    pub name: Option<String>,
    /// Whether this is a header field (for inspector ordering)
    pub header_field: bool,
    /// Whether this field is read-only in Unity inspector
    pub read_only: bool,
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
                        } else if ident == "skip_ffi" {
                            result.skip_ffi = true;
                        }
                        if !input.is_empty() {
                            input.parse::<syn::Token![,]>()?;
                        }
                    }
                    Ok(())
                });
            }
        } else if attr.path().is_ident("db_table") {
            // #[db_table("table_name")] — Plan 082
            result.db_table = Some(parse_db_table_attr(attr)?);
        } else if attr.path().is_ident("db_index") {
            // #[db_index(name = "...", on = "...")] — Plan 082
            result.db_indexes.push(parse_db_index_attr(attr)?);
        } else if attr.path().is_ident("db_foreign_key") {
            // #[db_foreign_key(column, references = "...", on_delete = "...")] — Plan 082
            result
                .db_foreign_keys
                .push(parse_db_foreign_key_attr(attr)?);
        } else if attr.path().is_ident("db_unique_constraint") {
            // #[db_unique_constraint("name", columns = ["..."])] — Plan 082
            result
                .db_unique_constraints
                .push(parse_db_unique_constraint_attr(attr)?);
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
        } else if attr.path().is_ident("primary_key") {
            // #[primary_key] — Plan 082
            result.primary_key = true;
        } else if attr.path().is_ident("db_column") {
            // #[db_column(TYPE, CONSTRAINTS)] — Plan 082
            result.db_column = Some(parse_db_column_attr(attr)?);
        } else if attr.path().is_ident("db_default") {
            // #[db_default("value")] — Plan 082
            result.db_default = Some(parse_db_default_attr(attr)?);
        } else if attr.path().is_ident("db_index") {
            // #[db_index(name = "...", on = "...")] — Plan 082
            result.db_index = Some(parse_db_index_attr(attr)?);
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
                } else if key == "read_only" {
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

// ============================================================================
// Plan 082: Database Schema Attribute Parsing Functions
// ============================================================================

/// Parse `#[db_table("table_name")]` attribute
fn parse_db_table_attr(attr: &Attribute) -> Result<DbTableAttr, syn::Error> {
    // Handle both #[db_table = "name"] and #[db_table("name")] formats
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }),
            ..
        }) => Ok(DbTableAttr {
            table_name: s.value(),
        }),
        Meta::List(list) => {
            let s: LitStr = list.parse_args()?;
            Ok(DbTableAttr {
                table_name: s.value(),
            })
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "Expected #[db_table(\"table_name\")] or #[db_table = \"table_name\"]",
        )),
    }
}

/// Parse `#[db_column(TYPE, CONSTRAINTS)]` attribute
///
/// Examples:
/// - `#[db_column(VARCHAR(255), NOT NULL)]`
/// - `#[db_column(BIGINT)]`
/// - `#[db_column(TEXT, NOT NULL, UNIQUE)]`
fn parse_db_column_attr(attr: &Attribute) -> Result<DbColumnAttr, syn::Error> {
    let mut sql_type = None;
    let mut constraints = Vec::new();

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                // Try to parse as string literal first
                if input.peek(LitStr) {
                    let s: LitStr = input.parse()?;
                    if sql_type.is_none() {
                        sql_type = Some(s.value());
                    } else {
                        constraints.push(s.value());
                    }
                } else {
                    // Parse as identifier(s) — e.g., VARCHAR, NOT, NULL
                    let ident: Ident = input.parse()?;
                    let mut token = ident.to_string();

                    // Handle parenthesized types like VARCHAR(255)
                    if input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in input);
                        let inner: syn::Lit = content.parse()?;
                        let inner_str = match &inner {
                            Lit::Int(i) => i.base10_digits().to_string(),
                            Lit::Str(s) => s.value(),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    inner,
                                    "Expected literal for type parameter",
                                ))
                            }
                        };
                        token = format!("{}({})", token, inner_str);
                    }

                    if sql_type.is_none() {
                        sql_type = Some(token);
                    } else {
                        // Collect constraint tokens: "NOT NULL" needs to be combined
                        if token == "NOT" && input.peek(Ident) {
                            let next: Ident = input.parse()?;
                            constraints.push(format!("NOT {}", next));
                        } else {
                            constraints.push(token);
                        }
                    }
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    Ok(DbColumnAttr {
        sql_type,
        constraints,
    })
}

/// Parse `#[db_default("value")]` attribute
///
/// Examples:
/// - `#[db_default("'general'")]` — string SQL default
/// - `#[db_default(24)]` — numeric default
/// - `#[db_default(true)]` — boolean default
/// - `#[db_default("gen_random_uuid()")]` — SQL function default
/// - `#[db_default("NOW()")]` — SQL function default
fn parse_db_default_attr(attr: &Attribute) -> Result<DbDefaultAttr, syn::Error> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit { lit, .. }),
            ..
        }) => parse_db_default_lit(lit),
        Meta::List(list) => {
            // Handle #[db_default("value")] or #[db_default(42)] or #[db_default(125.0)] formats
            let nested: TokenStream2 = list.tokens.clone();
            // Try to parse as string literal
            let parsed: Result<LitStr, _> = syn::parse2(nested.clone());
            if let Ok(s) = parsed {
                return Ok(DbDefaultAttr::String(s.value()));
            }
            // Try to parse as float (before int, since 125.0 parses as LitFloat)
            let parsed: Result<LitFloat, _> = syn::parse2(nested.clone());
            if let Ok(f) = parsed {
                return Ok(DbDefaultAttr::Float(f.base10_parse()?));
            }
            // Try to parse as integer
            let parsed: Result<LitInt, _> = syn::parse2(nested.clone());
            if let Ok(n) = parsed {
                return Ok(DbDefaultAttr::Number(n.base10_parse()?));
            }
            // Try to parse as boolean
            let parsed: Result<LitBool, _> = syn::parse2(nested);
            if let Ok(b) = parsed {
                return Ok(DbDefaultAttr::Bool(b.value));
            }
            Err(syn::Error::new_spanned(
                attr,
                "Expected db_default(\"value\") or db_default(42) or db_default(125.0) or db_default(true)",
            ))
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "Expected db_default = \"value\" or db_default(\"value\")",
        )),
    }
}

/// Parse a literal value into a DbDefaultAttr
fn parse_db_default_lit(lit: &Lit) -> Result<DbDefaultAttr, syn::Error> {
    match lit {
        Lit::Str(s) => Ok(DbDefaultAttr::String(s.value())),
        Lit::Int(n) => Ok(DbDefaultAttr::Number(n.base10_parse()?)),
        Lit::Float(f) => Ok(DbDefaultAttr::Float(f.base10_parse()?)),
        Lit::Bool(b) => Ok(DbDefaultAttr::Bool(b.value)),
        _ => Err(syn::Error::new_spanned(
            lit,
            "Expected string, integer, float, or boolean for db_default",
        )),
    }
}

/// Parse `#[db_index(name = "...", on = "...", condition = "...")]` attribute
///
/// Examples:
/// - `#[db_index(name = "idx_shops_npc_id", on = "npc_id")]`
/// - `#[db_index(name = "idx_shop_category", on = ["shop_id", "category"])]`
/// - `#[db_index(name = "idx_active", on = "is_enabled", condition = "is_enabled = true")]`
fn parse_db_index_attr(attr: &Attribute) -> Result<DbIndexAttr, syn::Error> {
    let mut name = None;
    let mut on = None;
    let mut condition = None;

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            while !input.is_empty() {
                let key: Ident = input.parse()?;
                input.parse::<syn::Token![=]>()?;

                match key.to_string().as_str() {
                    "name" => {
                        let value: LitStr = input.parse()?;
                        name = Some(value.value());
                    }
                    "on" => {
                        // Can be a single string or an array of strings
                        if input.peek(syn::token::Bracket) {
                            let content;
                            syn::bracketed!(content in input);
                            let mut cols = Vec::new();
                            while !content.is_empty() {
                                let s: LitStr = content.parse()?;
                                cols.push(s.value());
                                if !content.is_empty() {
                                    content.parse::<syn::Token![,]>()?;
                                }
                            }
                            on = Some(DbIndexColumns::Composite(cols));
                        } else {
                            let value: LitStr = input.parse()?;
                            on = Some(DbIndexColumns::Single(value.value()));
                        }
                    }
                    "condition" => {
                        let value: LitStr = input.parse()?;
                        condition = Some(value.value());
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            &key,
                            format!("Unknown db_index key: {}", other),
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

    let name =
        name.ok_or_else(|| syn::Error::new_spanned(attr, "db_index requires 'name' parameter"))?;
    let on = on.unwrap_or(DbIndexColumns::Single(name.clone()));

    Ok(DbIndexAttr {
        name,
        on,
        condition,
    })
}

/// Parse `#[db_foreign_key(column, references = "table.col", on_delete = "...")]` attribute
///
/// Examples:
/// - `#[db_foreign_key(shop_id, references = "shops.id", on_delete = "CASCADE")]`
/// - `#[db_foreign_key(item_id, references = "items.id")]`
fn parse_db_foreign_key_attr(attr: &Attribute) -> Result<DbForeignKeyAttr, syn::Error> {
    let mut column = None;
    let mut references = None;
    let mut on_delete = None;

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            // First positional argument is the column name
            if input.peek(Ident) && !input.peek2(syn::Token![=]) {
                let col: Ident = input.parse()?;
                column = Some(col.to_string());

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }

            // Then key-value pairs
            while !input.is_empty() {
                let key: Ident = input.parse()?;
                input.parse::<syn::Token![=]>()?;

                match key.to_string().as_str() {
                    "references" => {
                        let value: LitStr = input.parse()?;
                        references = Some(value.value());
                    }
                    "on_delete" => {
                        let value: LitStr = input.parse()?;
                        on_delete = Some(value.value());
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            &key,
                            format!("Unknown db_foreign_key key: {}", other),
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

    let column = column
        .ok_or_else(|| syn::Error::new_spanned(attr, "db_foreign_key requires column name"))?;
    let references = references.ok_or_else(|| {
        syn::Error::new_spanned(attr, "db_foreign_key requires 'references' parameter")
    })?;

    Ok(DbForeignKeyAttr {
        column,
        references,
        on_delete,
    })
}

/// Parse `#[db_unique_constraint("name", columns = ["..."])]` attribute
///
/// Examples:
/// - `#[db_unique_constraint("unique_shop_item", columns = ["shop_id", "item_id"])]`
fn parse_db_unique_constraint_attr(attr: &Attribute) -> Result<DbUniqueConstraintAttr, syn::Error> {
    let mut name = None;
    let mut columns = Vec::new();

    if let Meta::List(list) = &attr.meta {
        list.parse_args_with(|input: ParseStream| -> syn::Result<()> {
            // First positional argument is the constraint name
            if input.peek(LitStr) {
                let s: LitStr = input.parse()?;
                name = Some(s.value());

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }

            // Then key-value pairs
            while !input.is_empty() {
                let key: Ident = input.parse()?;
                input.parse::<syn::Token![=]>()?;

                if key == "columns" {
                    // Parse array: ["col1", "col2"]
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        let s: LitStr = content.parse()?;
                        columns.push(s.value());
                        if !content.is_empty() {
                            content.parse::<syn::Token![,]>()?;
                        }
                    }
                } else {
                    return Err(syn::Error::new_spanned(
                        &key,
                        format!("Unknown db_unique_constraint key: {}", key),
                    ));
                }

                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
        })?;
    }

    let name = name.ok_or_else(|| {
        syn::Error::new_spanned(attr, "db_unique_constraint requires a name argument")
    })?;

    Ok(DbUniqueConstraintAttr { name, columns })
}
