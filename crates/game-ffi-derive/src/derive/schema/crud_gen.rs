//! CRUD code generation from DbSchemaInfo (Plan 091 - Parameterized Queries)
//!
//! Generates async Rust methods for database CRUD operations using parameterized queries
//! for SQL injection prevention and PgCat compatibility.
//!
//! # Generated Methods
//!
//! - `from_row()` — Convert a `PgRow` to the struct
//! - `insert()` — Insert a new row, return the created record
//! - `find_by_id()` — Find a row by primary key
//! - `find_all()` — Retrieve all rows
//! - `find_many()` — Find rows by a list of primary key values
//! - `paginate()` — Paginated query with offset/limit
//! - `bulk_insert()` — Insert multiple rows at once
//! - `upsert()` — Insert or update on conflict (by primary key)
//! - `update()` — Partial update by primary key
//! - `delete()` — Delete a row by primary key
//!
//! # Changes from Plan 082
//!
//! - All queries now use `sqlx::query()` with `.bind()` for parameterization
//! - No more `sqlx::raw_sql()` or manual string escaping
//! - `find_many()` uses `UNNEST($1::type[])` for IN clause
//! - `from_row()` error handling changed from `Err(RowNotFound)` to `Ok(None)`

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use super::types::{DbFieldInfo, DbSchemaInfo};
use crate::derive::attributes::DbDefaultAttr;
use crate::derive::schema::types::unwrap_option_type;

// ============================================================================
// Public Entry Point
// ============================================================================

/// Generate CRUD method implementations for a database table.
///
/// Returns an empty `TokenStream` if the schema has no primary key.
pub fn generate_crud_impl(struct_name: &Ident, schema: &DbSchemaInfo) -> TokenStream {
    let pk = match schema.primary_key() {
        Some(pk) => pk,
        None => return quote! {},
    };

    let from_row = generate_from_row(struct_name, schema);
    let insert = generate_insert(struct_name, schema, pk);
    let find_by_id = generate_find_by_id(struct_name, schema, pk);
    let find_all = generate_find_all(struct_name, schema);
    let find_many = generate_find_many(struct_name, schema, pk);
    let paginate = generate_paginate(struct_name, schema);
    let bulk_insert = generate_bulk_insert(struct_name, schema, pk);
    let upsert = generate_upsert(struct_name, schema, pk);
    let update = generate_update(struct_name, schema, pk);
    let delete = generate_delete(struct_name, schema, pk);

    quote! {
        #from_row
        #insert
        #find_by_id
        #find_all
        #find_many
        #paginate
        #bulk_insert
        #upsert
        #update
        #delete
    }
}

// ============================================================================
// from_row
// ============================================================================

/// Generate `from_row()` method that converts a `PgRow` to the struct.
///
/// Uses `row.try_get("column")` with type inference from the struct field types.
fn generate_from_row(struct_name: &Ident, schema: &DbSchemaInfo) -> TokenStream {
    let field_assignments: Vec<TokenStream> = schema
        .fields
        .iter()
        .map(|f| {
            let field_ident = Ident::new(&f.name, Span::call_site());
            let field_name_str = &f.name;
            quote! {
                #field_ident: row.try_get(#field_name_str)?
            }
        })
        .collect();

    quote! {
        impl #struct_name {
            /// Convert a PostgreSQL row into Self (Plan 091 - Parameterized Queries).
            pub fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
                use sqlx::Row;
                Ok(Self {
                    #(#field_assignments),*
                })
            }
        }
    }
}

// ============================================================================
// insert
// ============================================================================

/// Generate `insert()` method using parameterized queries.
///
/// Parameters include all non-auto-generated fields (PK with default and
/// timestamp columns with `NOW()` default are excluded).
fn generate_insert(struct_name: &Ident, schema: &DbSchemaInfo, _pk: &DbFieldInfo) -> TokenStream {
    let insertable: Vec<&DbFieldInfo> = schema
        .fields
        .iter()
        .filter(|f| !is_skip_insert(f))
        .collect();

    if insertable.is_empty() {
        return quote! {};
    }

    // Method parameters
    let params: Vec<TokenStream> = insertable
        .iter()
        .map(|f| {
            let ident = Ident::new(&f.name, Span::call_site());
            let ty = parse_type_tokens(&f.rust_type);
            quote! { #ident: #ty }
        })
        .collect();

    // Column list as a single string literal
    let columns_str: String = insertable
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // Generate placeholders: $1, $2, $3, ...
    let placeholders: String = (1..=insertable.len())
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Generate bind calls
    let bind_calls: Vec<TokenStream> = insertable
        .iter()
        .map(|f| {
            let ident = Ident::new(&f.name, Span::call_site());
            quote! { .bind(#ident) }
        })
        .collect();

    quote! {
        impl #struct_name {
            /// Insert a new row and return the created record (Plan 091 - Parameterized Queries).
            pub async fn insert(
                pool: &sqlx::PgPool,
                #(#params),*
            ) -> Result<Self, sqlx::Error> {
                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                    <#struct_name>::TABLE_NAME,
                    #columns_str,
                    #placeholders
                );
                let row = sqlx::query(&sql)
                    #(#bind_calls)*
                    .fetch_one(pool)
                    .await?;
                <#struct_name>::from_row(&row)
            }
        }
    }
}

// ============================================================================
// find_by_id
// ============================================================================

/// Generate `find_by_id()` method using parameterized query.
///
/// Takes the primary key value and returns `Option<Self>`.
/// Changed error handling from `Err(RowNotFound)` to `Ok(None)`.
fn generate_find_by_id(
    struct_name: &Ident,
    _schema: &DbSchemaInfo,
    pk: &DbFieldInfo,
) -> TokenStream {
    let pk_name = &pk.name;
    let pk_ident = Ident::new(&pk.name, Span::call_site());
    let pk_type = parse_type_tokens(&pk.rust_type);

    quote! {
        impl #struct_name {
            /// Find a row by its primary key (Plan 091 - Parameterized Queries).
            pub async fn find_by_id(
                pool: &sqlx::PgPool,
                #pk_ident: #pk_type
            ) -> Result<Option<Self>, sqlx::Error> {
                let sql = format!(
                    "SELECT * FROM {} WHERE {} = $1",
                    <#struct_name>::TABLE_NAME,
                    #pk_name
                );
                match sqlx::query(&sql)
                    .bind(#pk_ident)
                    .fetch_optional(pool)
                    .await?
                {
                    None => Ok(None),
                    Some(row) => Ok(Some(<#struct_name>::from_row(&row)?)),
                }
            }
        }
    }
}

// ============================================================================
// find_all
// ============================================================================

/// Generate `find_all()` method that retrieves all rows.
fn generate_find_all(struct_name: &Ident, _schema: &DbSchemaInfo) -> TokenStream {
    quote! {
        impl #struct_name {
            /// Retrieve all rows (Plan 091 - Parameterized Queries).
            pub async fn find_all(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
                let sql = format!("SELECT * FROM {}", <#struct_name>::TABLE_NAME);
                let rows = sqlx::query(&sql)
                    .fetch_all(pool)
                    .await?;
                rows.iter()
                    .map(|row| <#struct_name>::from_row(row))
                    .collect()
            }
        }
    }
}

// ============================================================================
// find_many
// ============================================================================

/// Generate `find_many()` method using parameterized UNNEST for IN clause.
///
/// For UUID PKs: uses `WHERE pk = ANY($1::uuid[])`
/// For other types: uses appropriate array type casting
fn generate_find_many(
    struct_name: &Ident,
    _schema: &DbSchemaInfo,
    pk: &DbFieldInfo,
) -> TokenStream {
    let pk_name = &pk.name;
    let pk_inner = unwrap_option_type(&pk.rust_type);
    let pk_type = parse_type_tokens(&pk.rust_type);

    // Determine SQL array type
    let array_type = match pk_inner {
        "Uuid" | "uuid::Uuid" => "uuid[]",
        "i32" => "int[]",
        "i64" => "bigint[]",
        "String" => "text[]",
        _ => "uuid[]", // Default to uuid
    };

    // Build the full SQL string at compile time
    // Note: The string must contain literal {} for the generated format!() call.
    // We can't use format!() here because it would consume the {} placeholders.
    let sql_template = format!("SELECT * FROM {{}} WHERE {{}} = ANY($1::{})", array_type);

    quote! {
        impl #struct_name {
            /// Find rows by a list of primary key values (Plan 091 - Parameterized Queries).
            ///
            /// Uses parameterized `WHERE pk = ANY($1::type[])` clause.
            pub async fn find_many(
                pool: &sqlx::PgPool,
                ids: &[#pk_type]
            ) -> Result<Vec<Self>, sqlx::Error> {
                if ids.is_empty() {
                    return Ok(vec![]);
                }
                let sql = format!(
                    #sql_template,
                    <#struct_name>::TABLE_NAME,
                    #pk_name
                );
                let rows = sqlx::query(&sql)
                    .bind(ids)
                    .fetch_all(pool)
                    .await?;
                rows.iter()
                    .map(|row| <#struct_name>::from_row(row))
                    .collect()
            }
        }
    }
}

// ============================================================================
// paginate
// ============================================================================

/// Generate `paginate()` method using parameterized LIMIT and OFFSET.
///
/// Returns `(items, total_count)` where `total_count` is the total number
/// of rows in the table.
fn generate_paginate(struct_name: &Ident, _schema: &DbSchemaInfo) -> TokenStream {
    quote! {
        impl #struct_name {
            /// Paginated query with offset and limit (Plan 091 - Parameterized Queries).
            ///
            /// Returns `(items, total_count)`.
            /// - `page` is 0-indexed.
            /// - `per_page` is the number of items per page.
            pub async fn paginate(
                pool: &sqlx::PgPool,
                page: i64,
                per_page: i64,
            ) -> Result<(Vec<Self>, i64), sqlx::Error> {
                use sqlx::Row;
                let offset = page * per_page;

                // Count query
                let count_sql = format!("SELECT COUNT(*) as cnt FROM {}", <#struct_name>::TABLE_NAME);
                let count_row = sqlx::query(&count_sql)
                    .fetch_one(pool)
                    .await?;
                let total: i64 = count_row.try_get("cnt")?;

                // Data query with parameterized LIMIT and OFFSET
                let data_sql = format!(
                    "SELECT * FROM {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                    <#struct_name>::TABLE_NAME
                );
                let rows = sqlx::query(&data_sql)
                    .bind(per_page)
                    .bind(offset)
                    .fetch_all(pool)
                    .await?;

                let items: Vec<Self> = rows.iter()
                    .map(|row| <#struct_name>::from_row(row))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((items, total))
            }
        }
    }
}

// ============================================================================
// bulk_insert
// ============================================================================

/// Generate `bulk_insert()` method using batched parameterized inserts.
///
/// For efficiency, this generates a multi-row VALUES clause with parameters.
fn generate_bulk_insert(
    struct_name: &Ident,
    schema: &DbSchemaInfo,
    _pk: &DbFieldInfo,
) -> TokenStream {
    let insertable: Vec<&DbFieldInfo> = schema
        .fields
        .iter()
        .filter(|f| !is_skip_insert(f))
        .collect();

    if insertable.is_empty() {
        return quote! {};
    }

    let columns_str: String = insertable
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let field_count = insertable.len();

    // Generate bind calls for each item's fields
    let bind_calls: Vec<TokenStream> = insertable
        .iter()
        .map(|f| {
            let field_ident = Ident::new(&f.name, Span::call_site());
            quote! { query = query.bind(&item.#field_ident); }
        })
        .collect();

    quote! {
        impl #struct_name {
            /// Bulk insert multiple rows and return all created records (Plan 091 - Parameterized Queries).
            ///
            /// Uses a single multi-row INSERT statement with parameterized values.
            pub async fn bulk_insert(
                pool: &sqlx::PgPool,
                items: &[Self]
            ) -> Result<Vec<Self>, sqlx::Error> {
                if items.is_empty() {
                    return Ok(vec![]);
                }

                // Build placeholders for all items: ($1, $2, $3), ($4, $5, $6), ...
                let field_count = #field_count;
                let mut placeholders: Vec<String> = Vec::with_capacity(items.len());
                for i in 0..items.len() {
                    let row_placeholders: Vec<String> = (1..=field_count)
                        .map(|j| format!("${}", i * field_count + j))
                        .collect();
                    placeholders.push(format!("({})", row_placeholders.join(", ")));
                }

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES {} RETURNING *",
                    <#struct_name>::TABLE_NAME,
                    #columns_str,
                    placeholders.join(", ")
                );

                // Bind all values
                let mut query = sqlx::query(&sql);
                for item in items {
                    #(#bind_calls)*
                }

                let rows = query.fetch_all(pool).await?;
                rows.iter()
                    .map(|row| <#struct_name>::from_row(row))
                    .collect()
            }
        }
    }
}

// ============================================================================
// upsert
// ============================================================================

/// Generate `upsert()` method using parameterized INSERT ... ON CONFLICT.
///
/// Uses `INSERT INTO ... ON CONFLICT (pk) DO UPDATE SET ...` with bound parameters.
fn generate_upsert(struct_name: &Ident, schema: &DbSchemaInfo, pk: &DbFieldInfo) -> TokenStream {
    let insertable: Vec<&DbFieldInfo> = schema
        .fields
        .iter()
        .filter(|f| !is_skip_insert(f))
        .collect();

    if insertable.is_empty() {
        return quote! {};
    }

    // Parameters include ALL user-provided fields (insertable)
    let params: Vec<TokenStream> = insertable
        .iter()
        .map(|f| {
            let ident = Ident::new(&f.name, Span::call_site());
            let ty = parse_type_tokens(&f.rust_type);
            quote! { #ident: #ty }
        })
        .collect();

    let columns_str: String = insertable
        .iter()
        .map(|f| f.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // Generate placeholders: $1, $2, $3, ...
    let placeholders: String = (1..=insertable.len())
        .map(|i| format!("${}", i))
        .collect::<Vec<_>>()
        .join(", ");

    // Generate bind calls
    let bind_calls: Vec<TokenStream> = insertable
        .iter()
        .map(|f| {
            let ident = Ident::new(&f.name, Span::call_site());
            quote! { .bind(#ident) }
        })
        .collect();

    // ON CONFLICT DO UPDATE SET — update all non-PK, non-auto fields
    let updatable: Vec<&DbFieldInfo> = schema
        .fields
        .iter()
        .filter(|f| !is_skip_update(f) && !is_skip_insert(f))
        .collect();

    let set_clauses: Vec<String> = updatable
        .iter()
        .map(|f| format!("{name} = EXCLUDED.{name}", name = f.name))
        .collect();

    // Add updated_at = NOW() if present
    let updated_at_clause = if has_updated_at_field(schema) {
        ", updated_at = NOW()".to_string()
    } else {
        String::new()
    };

    let set_sql = format!("{}{}", set_clauses.join(", "), updated_at_clause);

    let pk_name = &pk.name;

    quote! {
        impl #struct_name {
            /// Insert or update on primary key conflict (Plan 091 - Parameterized Queries).
            ///
            /// Uses `ON CONFLICT (pk) DO UPDATE SET ...` (upsert) with bound parameters.
            pub async fn upsert(
                pool: &sqlx::PgPool,
                #(#params),*
            ) -> Result<Self, sqlx::Error> {
                let sql = format!(
                    "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {} RETURNING *",
                    <#struct_name>::TABLE_NAME,
                    #columns_str,
                    #placeholders,
                    #pk_name,
                    #set_sql
                );
                let row = sqlx::query(&sql)
                    #(#bind_calls)*
                    .fetch_one(pool)
                    .await?;
                <#struct_name>::from_row(&row)
            }
        }
    }
}

// ============================================================================
// update
// ============================================================================

/// Generate `update()` method using parameterized UPDATE ... SET.
///
/// All non-PK, non-auto-updated fields become `Option<T>` parameters.
/// `None` means "don't change". An `updated_at = NOW()` clause is appended
/// automatically when the schema contains an `updated_at` column.
fn generate_update(struct_name: &Ident, schema: &DbSchemaInfo, pk: &DbFieldInfo) -> TokenStream {
    let updatable: Vec<&DbFieldInfo> = schema
        .fields
        .iter()
        .filter(|f| !is_skip_update(f))
        .collect();

    if updatable.is_empty() {
        return quote! {};
    }

    // Method parameters: primary key + Option<T> for each updatable field
    let pk_ident = Ident::new(&pk.name, Span::call_site());
    let pk_type = parse_type_tokens(&pk.rust_type);

    let update_params: Vec<TokenStream> = updatable
        .iter()
        .map(|f| {
            let ident = Ident::new(&f.name, Span::call_site());
            let ty = update_param_type(f);
            quote! { #ident: #ty }
        })
        .collect();

    // Generate SET clause building logic with parameterized placeholders
    let set_building: Vec<TokenStream> = updatable
        .iter()
        .map(|f| {
            let field_ident = Ident::new(&f.name, Span::call_site());
            let field_name = &f.name;
            quote! {
                if let Some(ref v) = #field_ident {
                    set_clauses.push(format!("{} = ${}", #field_name, param_index));
                    param_index += 1;
                }
            }
        })
        .collect();

    // Generate bind calls
    let bind_calls: Vec<TokenStream> = updatable
        .iter()
        .map(|f| {
            let field_ident = Ident::new(&f.name, Span::call_site());
            quote! {
                if let Some(v) = #field_ident {
                    query = query.bind(v);
                }
            }
        })
        .collect();

    // Auto-append updated_at = NOW() if the field exists
    let updated_at_push = if has_updated_at_field(schema) {
        quote! {
            if !set_clauses.is_empty() {
                set_clauses.push("updated_at = NOW()".to_string());
            }
        }
    } else {
        quote! {}
    };

    let pk_name = &pk.name;

    quote! {
        impl #struct_name {
            /// Partial update by primary key (Plan 091 - Parameterized Queries).
            ///
            /// `None` params are ignored (field is unchanged).
            /// `updated_at` is auto-set to `NOW()` when the column exists.
            pub async fn update(
                pool: &sqlx::PgPool,
                #pk_ident: #pk_type,
                #(#update_params),*
            ) -> Result<Option<Self>, sqlx::Error> {
                let mut set_clauses: Vec<String> = Vec::new();
                let mut param_index = 1;

                #(#set_building)*

                if set_clauses.is_empty() {
                    return <#struct_name>::find_by_id(pool, #pk_ident).await;
                }

                #updated_at_push

                let sql = format!(
                    "UPDATE {} SET {} WHERE {} = ${} RETURNING *",
                    <#struct_name>::TABLE_NAME,
                    set_clauses.join(", "),
                    #pk_name,
                    param_index
                );

                let mut query = sqlx::query(&sql);
                #(#bind_calls)*
                query = query.bind(#pk_ident);

                match query.fetch_optional(pool).await? {
                    None => Ok(None),
                    Some(row) => Ok(Some(<#struct_name>::from_row(&row)?)),
                }
            }
        }
    }
}

// ============================================================================
// delete
// ============================================================================

/// Generate `delete()` method using parameterized DELETE.
///
/// Returns the number of affected rows (`u64`).
fn generate_delete(struct_name: &Ident, _schema: &DbSchemaInfo, pk: &DbFieldInfo) -> TokenStream {
    let pk_name = &pk.name;
    let pk_ident = Ident::new(&pk.name, Span::call_site());
    let pk_type = parse_type_tokens(&pk.rust_type);

    quote! {
        impl #struct_name {
            /// Delete a row by primary key (Plan 091 - Parameterized Queries).
            ///
            /// Returns the number of rows affected (0 or 1).
            pub async fn delete(
                pool: &sqlx::PgPool,
                #pk_ident: #pk_type
            ) -> Result<u64, sqlx::Error> {
                let sql = format!(
                    "DELETE FROM {} WHERE {} = $1",
                    <#struct_name>::TABLE_NAME,
                    #pk_name
                );
                sqlx::query(&sql)
                    .bind(#pk_ident)
                    .execute(pool)
                    .await
                    .map(|r| r.rows_affected())
            }
        }
    }
}

// ============================================================================
// Helpers — Type Parsing
// ============================================================================

/// Parse a Rust type string (e.g. `"Option<Uuid>"`) into a `TokenStream`.
fn parse_type_tokens(type_str: &str) -> TokenStream {
    let ty = syn::parse_str::<syn::Type>(type_str)
        .unwrap_or_else(|e| panic!("failed to parse type '{type_str}': {e}"));
    quote! { #ty }
}

// ============================================================================
// Helpers — Field Classification
// ============================================================================

/// Fields to skip in `insert()` parameters.
///
/// - Primary key with a default value (auto-generated by DB)
/// - Timestamp columns ending in `_at` with a `NOW()` default
fn is_skip_insert(field: &DbFieldInfo) -> bool {
    // PK with a default → auto-generated
    if field.primary_key && field.default_value.is_some() {
        return true;
    }
    // Timestamp with NOW() default → auto-generated
    if let Some(DbDefaultAttr::String(s)) = &field.default_value {
        let upper = s.to_uppercase();
        if (upper.contains("NOW()") || upper.contains("CURRENT_TIMESTAMP"))
            && field.name.ends_with("_at")
        {
            return true;
        }
    }
    false
}

/// Fields to skip in `update()` parameters.
///
/// - Primary key (immutable)
/// - `created_at` (immutable)
/// - `updated_at` (auto-set by the update method)
fn is_skip_update(field: &DbFieldInfo) -> bool {
    if field.primary_key {
        return true;
    }
    if field.name == "created_at" {
        return true;
    }
    if field.name == "updated_at" {
        return true;
    }
    false
}

/// Check if the schema has an `updated_at` field.
fn has_updated_at_field(schema: &DbSchemaInfo) -> bool {
    schema.fields.iter().any(|f| f.name == "updated_at")
}

// ============================================================================
// Helpers — Update Parameter Type
// ============================================================================

/// Determine the update-parameter type for a struct field.
///
/// - Non-nullable `T` → `Option<T>` (None = don't change)
/// - Nullable `Option<T>` → `Option<T>` (same type, None = don't change)
fn update_param_type(field: &DbFieldInfo) -> TokenStream {
    if field.is_nullable() {
        // Already Option<T>; use as-is so None = don't change.
        parse_type_tokens(&field.rust_type)
    } else {
        let inner = parse_type_tokens(&field.rust_type);
        quote! { Option<#inner> }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::schema::types::{DbFieldInfo, DbSchemaInfo};

    /// Helper: build a simple schema with `id` (UUID PK) + `name` (String).
    fn make_simple_schema() -> DbSchemaInfo {
        DbSchemaInfo::new(
            "SimpleItem".to_string(),
            "simple_items".to_string(),
            vec![
                DbFieldInfo {
                    name: "id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: true,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("gen_random_uuid()".to_string())),
                    index: None,
                },
                DbFieldInfo {
                    name: "name".to_string(),
                    rust_type: "String".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                },
            ],
            vec![],
            vec![],
            vec![],
            false,
        )
    }

    /// Helper: build a complex schema with nullable fields and timestamps.
    fn make_complex_schema() -> DbSchemaInfo {
        DbSchemaInfo::new(
            "Shop".to_string(),
            "shops".to_string(),
            vec![
                DbFieldInfo {
                    name: "id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: true,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("gen_random_uuid()".to_string())),
                    index: None,
                },
                DbFieldInfo {
                    name: "name".to_string(),
                    rust_type: "String".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                },
                DbFieldInfo {
                    name: "shop_type".to_string(),
                    rust_type: "String".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("'general'".to_string())),
                    index: None,
                },
                DbFieldInfo {
                    name: "is_enabled".to_string(),
                    rust_type: "bool".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::Bool(true)),
                    index: None,
                },
                DbFieldInfo {
                    name: "npc_id".to_string(),
                    rust_type: "Option<Uuid>".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                },
                DbFieldInfo {
                    name: "created_at".to_string(),
                    rust_type: "DateTime<Utc>".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
                    index: None,
                },
                DbFieldInfo {
                    name: "updated_at".to_string(),
                    rust_type: "DateTime<Utc>".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
                    index: None,
                },
            ],
            vec![],
            vec![],
            vec![],
            false,
        )
    }

    // -----------------------------------------------------------------------
    // Field classification tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_skip_insert_pk_with_default() {
        let field = DbFieldInfo {
            name: "id".to_string(),
            rust_type: "Uuid".to_string(),
            primary_key: true,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::String("gen_random_uuid()".to_string())),
            index: None,
        };
        assert!(is_skip_insert(&field));
    }

    #[test]
    fn test_is_skip_insert_pk_without_default() {
        let field = DbFieldInfo {
            name: "id".to_string(),
            rust_type: "Uuid".to_string(),
            primary_key: true,
            sql_type_override: None,
            constraints: vec![],
            default_value: None,
            index: None,
        };
        assert!(!is_skip_insert(&field));
    }

    #[test]
    fn test_is_skip_insert_timestamp_with_now() {
        let field = DbFieldInfo {
            name: "created_at".to_string(),
            rust_type: "DateTime<Utc>".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
            index: None,
        };
        assert!(is_skip_insert(&field));
    }

    #[test]
    fn test_is_skip_insert_regular_field() {
        let field = DbFieldInfo {
            name: "name".to_string(),
            rust_type: "String".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: None,
            index: None,
        };
        assert!(!is_skip_insert(&field));
    }

    #[test]
    fn test_is_skip_update_pk() {
        let field = DbFieldInfo {
            name: "id".to_string(),
            rust_type: "Uuid".to_string(),
            primary_key: true,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::String("gen_random_uuid()".to_string())),
            index: None,
        };
        assert!(is_skip_update(&field));
    }

    #[test]
    fn test_is_skip_update_created_at() {
        let field = DbFieldInfo {
            name: "created_at".to_string(),
            rust_type: "DateTime<Utc>".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
            index: None,
        };
        assert!(is_skip_update(&field));
    }

    #[test]
    fn test_is_skip_update_updated_at() {
        let field = DbFieldInfo {
            name: "updated_at".to_string(),
            rust_type: "DateTime<Utc>".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
            index: None,
        };
        assert!(is_skip_update(&field));
    }

    #[test]
    fn test_is_skip_update_regular_field() {
        let field = DbFieldInfo {
            name: "name".to_string(),
            rust_type: "String".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: None,
            index: None,
        };
        assert!(!is_skip_update(&field));
    }

    // -----------------------------------------------------------------------
    // Type helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_has_updated_at_field() {
        assert!(has_updated_at_field(&make_complex_schema()));
        assert!(!has_updated_at_field(&make_simple_schema()));
    }

    // -----------------------------------------------------------------------
    // Token generation smoke tests (Plan 091 - Parameterized Queries)
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_from_row_simple() {
        let schema = make_simple_schema();
        let struct_name = Ident::new("SimpleItem", Span::call_site());
        let tokens = generate_from_row(&struct_name, &schema);
        let code = tokens.to_string();
        assert!(code.contains("from_row"));
        assert!(code.contains("try_get"));
        assert!(code.contains("\"id\""));
        assert!(code.contains("\"name\""));
    }

    #[test]
    fn test_generate_insert_uses_parameterized_query() {
        let schema = make_simple_schema();
        let pk = schema.primary_key().unwrap();
        let struct_name = Ident::new("SimpleItem", Span::call_site());
        let tokens = generate_insert(&struct_name, &schema, pk);
        let code = tokens.to_string();
        // Should use sqlx::query with .bind()
        assert!(code.contains("sqlx :: query"));
        assert!(code.contains(". bind"));
        // Should NOT contain raw_sql
        assert!(!code.contains("raw_sql"));
        // Should NOT contain manual escaping
        assert!(!code.contains("replace"));
        // Should contain placeholder
        assert!(code.contains("$1"));
    }

    #[test]
    fn test_generate_find_by_id_uses_parameterized_query() {
        let schema = make_simple_schema();
        let pk = schema.primary_key().unwrap();
        let struct_name = Ident::new("SimpleItem", Span::call_site());
        let tokens = generate_find_by_id(&struct_name, &schema, pk);
        let code = tokens.to_string();
        // Should use parameterized query
        assert!(code.contains("sqlx :: query"));
        assert!(code.contains(". bind"));
        assert!(code.contains("$1"));
        // Should handle Ok(None) not Err(RowNotFound)
        assert!(code.contains("Ok (None)"));
        assert!(!code.contains("RowNotFound"));
    }

    #[test]
    fn test_generate_find_many_uses_any_array() {
        let schema = make_simple_schema();
        let pk = schema.primary_key().unwrap();
        let struct_name = Ident::new("SimpleItem", Span::call_site());
        let tokens = generate_find_many(&struct_name, &schema, pk);
        let code = tokens.to_string();
        // Should use ANY($1::type[])
        assert!(code.contains("ANY"));
        assert!(code.contains("$1"));
        assert!(code.contains("uuid[]"));
        assert!(code.contains(". bind"));
    }

    #[test]
    fn test_generate_delete_uses_parameterized_query() {
        let schema = make_simple_schema();
        let pk = schema.primary_key().unwrap();
        let struct_name = Ident::new("SimpleItem", Span::call_site());
        let tokens = generate_delete(&struct_name, &schema, pk);
        let code = tokens.to_string();
        // Should use parameterized query
        assert!(code.contains("sqlx :: query"));
        assert!(code.contains(". bind"));
        assert!(code.contains("$1"));
        assert!(!code.contains("raw_sql"));
    }

    #[test]
    fn test_no_sql_escaping_in_generated_code() {
        let schema = make_complex_schema();
        let pk = schema.primary_key().unwrap();
        let struct_name = Ident::new("Shop", Span::call_site());

        let insert = generate_insert(&struct_name, &schema, pk);
        let update = generate_update(&struct_name, &schema, pk);
        let upsert = generate_upsert(&struct_name, &schema, pk);

        let insert_code = insert.to_string();
        let update_code = update.to_string();
        let upsert_code = upsert.to_string();

        // None of the methods should contain manual escaping
        assert!(!insert_code.contains("replace ('\\\\\\'' , \"''\" )"));
        assert!(!update_code.contains("replace ('\\\\\\'' , \"''\" )"));
        assert!(!upsert_code.contains("replace ('\\\\\\'' , \"''\" )"));
    }

    #[test]
    fn test_generate_crud_impl_no_pk_returns_empty() {
        let schema = DbSchemaInfo::new(
            "NoKey".to_string(),
            "no_keys".to_string(),
            vec![DbFieldInfo {
                name: "value".to_string(),
                rust_type: "i32".to_string(),
                primary_key: false,
                sql_type_override: None,
                constraints: vec![],
                default_value: None,
                index: None,
            }],
            vec![],
            vec![],
            vec![],
            false,
        );
        let struct_name = Ident::new("NoKey", Span::call_site());
        let tokens = generate_crud_impl(&struct_name, &schema);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_update_param_type_non_nullable() {
        let field = DbFieldInfo {
            name: "name".to_string(),
            rust_type: "String".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: None,
            index: None,
        };
        let tokens = update_param_type(&field);
        let code = tokens.to_string();
        assert!(code.contains("Option"));
    }

    #[test]
    fn test_update_param_type_nullable() {
        let field = DbFieldInfo {
            name: "npc_id".to_string(),
            rust_type: "Option<Uuid>".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: None,
            index: None,
        };
        let tokens = update_param_type(&field);
        let code = tokens.to_string();
        // Nullable field type stays as-is
        assert!(code.contains("Option < Uuid >"));
    }
}
