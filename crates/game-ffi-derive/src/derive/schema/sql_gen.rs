//! SQL DDL generation from DbSchemaInfo (Plan 082)
//!
//! Generates:
//! - `CREATE TABLE IF NOT EXISTS` statements with columns, foreign keys, and unique constraints
//! - `CREATE INDEX IF NOT EXISTS` statements from `#[db_index]` attributes
//! - `COLUMN_DEFS_SQL` — column definitions without CREATE TABLE wrapper (for `#[db_flatten]` composition)
//! - Rust `const` strings and `fn` for embeddable SQL via `quote`
//!
//! When `#[db_flatten]` is used on a field:
//! - `CREATE_TABLE_SQL` becomes `fn create_table_sql() -> String` (runtime composition)
//! - `COLUMN_DEFS_SQL` const contains only own non-flattened column definitions
//! - The flattened type must also have `#[db_table]` (so it has `COLUMN_DEFS_SQL`)

use proc_macro2::TokenStream;
use quote::quote;

use super::crud_gen::generate_crud_impl;
use super::types::DbSchemaInfo;
use crate::derive::attributes::{DbForeignKeyAttr, DbIndexColumns};

// ============================================================================
// SQL DDL Generation
// ============================================================================

/// Get column SQL lines for non-flattened fields (no indentation, no commas).
fn own_column_sql_lines(schema: &DbSchemaInfo) -> Vec<String> {
    schema
        .fields
        .iter()
        .filter(|f| !f.is_flatten())
        .filter_map(|f| f.column_sql())
        .collect()
}

/// Generate column definitions SQL string (no indentation, no commas, no CREATE TABLE wrapper).
///
/// Each line is a single column definition. Flattened fields are excluded.
///
/// Example output:
/// ```text
/// id UUID PRIMARY KEY DEFAULT gen_random_uuid()
/// name VARCHAR(255) NOT NULL
/// is_enabled BOOLEAN NOT NULL DEFAULT true
/// ```
pub fn generate_column_defs_sql(schema: &DbSchemaInfo) -> String {
    own_column_sql_lines(schema).join("\n")
}

/// Generate a `CREATE TABLE IF NOT EXISTS` SQL statement from a `DbSchemaInfo`.
///
/// Only includes non-flattened fields. For structs with `#[db_flatten]`,
/// use the generated `create_table_sql()` function instead.
///
/// Output example:
/// ```sql
/// CREATE TABLE IF NOT EXISTS shops (
///     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
///     name VARCHAR(255) NOT NULL,
///     shop_type VARCHAR(50) NOT NULL DEFAULT 'general',
///     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
/// )
/// ```
pub fn generate_create_table_sql(schema: &DbSchemaInfo) -> String {
    let mut lines: Vec<String> = own_column_sql_lines(schema)
        .into_iter()
        .map(|l| format!("    {}", l))
        .collect();

    // Foreign key constraints
    for fk in &schema.foreign_keys {
        lines.push(format!("    {}", foreign_key_sql(fk)));
    }

    // Unique constraints
    for uc in &schema.unique_constraints {
        let cols = uc.columns.join(", ");
        lines.push(format!("    CONSTRAINT {} UNIQUE ({})", uc.name, cols));
    }

    format!(
        "CREATE TABLE IF NOT EXISTS {} (\n{}\n)",
        schema.table_name,
        lines.join(",\n")
    )
}

/// Generate all `CREATE INDEX IF NOT EXISTS` statements from a `DbSchemaInfo`.
///
/// Combines table-level and field-level indexes.
pub fn generate_create_indexes_sql(schema: &DbSchemaInfo) -> String {
    let indexes = schema.all_indexes();
    if indexes.is_empty() {
        return String::new();
    }

    let mut stmts: Vec<String> = Vec::new();
    for idx in indexes {
        stmts.push(index_sql(
            &idx.name,
            &idx.on,
            &idx.condition,
            &schema.table_name,
        ));
    }

    stmts.join(";\n") + ";"
}

/// Generate the combined schema SQL (CREATE TABLE + CREATE INDEX).
#[allow(dead_code)]
pub fn generate_full_schema_sql(schema: &DbSchemaInfo) -> String {
    let table_sql = generate_create_table_sql(schema);
    let index_sql = generate_create_indexes_sql(schema);

    if index_sql.is_empty() {
        table_sql
    } else {
        format!("{};\n\n{}", table_sql, index_sql)
    }
}

// ============================================================================
// Rust Code Generation (quote)
// ============================================================================

/// Generate token streams for column pushes in struct field order.
///
/// Own fields get a direct `lines.push(...)`, flattened fields get a loop
/// that reads `<Type>::COLUMN_DEFS_SQL`.
fn generate_column_push_tokens(schema: &DbSchemaInfo) -> Vec<TokenStream> {
    let mut pushes = Vec::new();

    for field in &schema.fields {
        if field.is_flatten() {
            // Push lines from flattened type's COLUMN_DEFS_SQL
            match syn::parse_str::<syn::Path>(&field.rust_type) {
                Ok(type_path) => {
                    pushes.push(quote! {
                        for line in #type_path::COLUMN_DEFS_SQL.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                lines.push(trimmed.to_string());
                            }
                        }
                    });
                }
                Err(_) => {
                    let msg = format!(
                        "db_flatten: cannot parse type '{}' as a valid path",
                        field.rust_type
                    );
                    pushes.push(quote! { compile_error!(#msg); });
                }
            }
        } else if let Some(col_sql) = field.column_sql() {
            pushes.push(quote! {
                lines.push(#col_sql.to_string());
            });
        }
        // Fields with no SQL mapping (e.g., unsupported types) are silently skipped
    }

    pushes
}

/// Generate token streams for foreign key constraint pushes.
fn generate_fk_push_tokens(schema: &DbSchemaInfo) -> Vec<TokenStream> {
    schema
        .foreign_keys
        .iter()
        .map(|fk| {
            let fk_sql = foreign_key_sql(fk);
            quote! {
                lines.push(#fk_sql.to_string());
            }
        })
        .collect()
}

/// Generate token streams for unique constraint pushes.
fn generate_unique_push_tokens(schema: &DbSchemaInfo) -> Vec<TokenStream> {
    schema
        .unique_constraints
        .iter()
        .map(|uc| {
            let cols = uc.columns.join(", ");
            let sql = format!("CONSTRAINT {} UNIQUE ({})", uc.name, cols);
            quote! {
                lines.push(#sql.to_string());
            }
        })
        .collect()
}

/// Generate Rust `impl` block with SQL constants and helper methods.
///
/// For structs **without** `#[db_flatten]`:
/// - `CREATE_TABLE_SQL: &'static str` (const) — full CREATE TABLE statement
/// - `COLUMN_DEFS_SQL: &'static str` (const) — column definitions only
///
/// For structs **with** `#[db_flatten]`:
/// - `create_table_sql() -> String` (fn) — runtime composition with flattened types
/// - `COLUMN_DEFS_SQL: &'static str` (const) — own columns only (no flattened)
///
/// Always generates:
/// - `TABLE_NAME`, `CREATE_INDEXES_SQL` (const)
/// - `column_names()`, `column_count()`, `primary_key_field()` methods
pub fn generate_schema_impl(
    struct_name: &proc_macro2::Ident,
    schema: &DbSchemaInfo,
) -> TokenStream {
    let table_name = &schema.table_name;
    let create_table_sql = generate_create_table_sql(schema);
    let create_indexes_sql = generate_create_indexes_sql(schema);
    let column_defs_sql = generate_column_defs_sql(schema);

    // Column names (own non-flattened fields only)
    let column_names: Vec<String> = schema
        .fields
        .iter()
        .filter(|f| !f.is_flatten())
        .map(|f| f.name.clone())
        .collect();
    let column_count = column_names.len();

    // Primary key field name
    let pk_field = schema.primary_key().map(|f| f.name.clone());

    // Check if any field uses #[db_flatten]
    let has_flatten = schema.fields.iter().any(|f| f.is_flatten());

    // Index SQL const (generated regardless of flatten)
    let index_sql_tokens = if create_indexes_sql.is_empty() {
        quote! {}
    } else {
        quote! {
            /// Auto-generated CREATE INDEX SQL
            pub const CREATE_INDEXES_SQL: &'static str = #create_indexes_sql;
        }
    };

    // Primary key method
    let pk_method = if let Some(ref pk) = pk_field {
        quote! {
            /// Get the primary key column name
            pub fn primary_key_field() -> Option<&'static str> {
                Some(#pk)
            }
        }
    } else {
        quote! {
            /// Get the primary key column name (none defined)
            pub fn primary_key_field() -> Option<&'static str> {
                None
            }
        }
    };

    // CREATE TABLE: const for non-flatten, fn for flatten
    let create_table_tokens = if has_flatten {
        let column_pushes = generate_column_push_tokens(schema);
        let fk_pushes = generate_fk_push_tokens(schema);
        let unique_pushes = generate_unique_push_tokens(schema);

        quote! {
            /// Auto-generated CREATE TABLE SQL (runtime composition with flattened types).
            ///
            /// This is a function instead of a const because flattened columns come from
            /// other types' `COLUMN_DEFS_SQL` constants, which can't be concatenated at
            /// compile time.
            pub fn create_table_sql() -> String {
                let mut lines: Vec<String> = Vec::new();
                #(#column_pushes)*
                #(#fk_pushes)*
                #(#unique_pushes)*
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (\n    {}\n)",
                    Self::TABLE_NAME,
                    lines.join(",\n    "),
                )
            }
        }
    } else {
        quote! {
            /// Auto-generated CREATE TABLE SQL
            pub const CREATE_TABLE_SQL: &'static str = #create_table_sql;
        }
    };

    // Column defs doc comment varies based on flatten
    let column_defs_doc = if has_flatten {
        "Column definitions SQL (own non-flattened columns only, one per line).\n\nUsed by parent structs with `#[db_flatten]` to compose full CREATE TABLE SQL."
    } else {
        "Column definitions SQL (one per line, no CREATE TABLE wrapper).\n\nUseful for composing into parent tables via `#[db_flatten]`."
    };

    // Plan 082 Phase 2: CRUD methods
    let crud_impl = if schema.skip_crud {
        quote! {}
    } else {
        generate_crud_impl(struct_name, schema)
    };

    quote! {
        /// Auto-generated database schema implementation
        impl #struct_name {
            /// Database table name
            pub const TABLE_NAME: &'static str = #table_name;

            #create_table_tokens

            #[doc = #column_defs_doc]
            pub const COLUMN_DEFS_SQL: &'static str = #column_defs_sql;

            #index_sql_tokens

            /// Get column names (own non-flattened fields only)
            pub fn column_names() -> &'static [&'static str] {
                &[#(#column_names),*]
            }

            /// Get the number of own non-flattened columns
            pub fn column_count() -> usize {
                #column_count
            }

            #pk_method
        }

        // Plan 082 Phase 2: Auto-generated CRUD operations
        #crud_impl
    }
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Format a single `CREATE INDEX` SQL statement.
fn index_sql(
    name: &str,
    on: &DbIndexColumns,
    condition: &Option<String>,
    table_name: &str,
) -> String {
    let columns = match on {
        DbIndexColumns::Single(col) => col.clone(),
        DbIndexColumns::Composite(cols) => cols.join(", "),
    };

    let mut sql = format!(
        "CREATE INDEX IF NOT EXISTS {} ON {}({})",
        name, table_name, columns
    );

    if let Some(cond) = condition {
        sql.push_str(&format!(" WHERE {}", cond));
    }

    sql
}

/// Format a foreign key constraint SQL fragment for a CREATE TABLE column or table constraint.
///
/// Output: `FOREIGN KEY (column) REFERENCES table.col) ON DELETE action`
fn foreign_key_sql(fk: &DbForeignKeyAttr) -> String {
    let mut sql = format!("FOREIGN KEY ({}) REFERENCES {}", fk.column, fk.references);

    if let Some(on_delete) = &fk.on_delete {
        sql.push_str(&format!(" ON DELETE {}", on_delete));
    }

    sql
}

/// Format a foreign key as an inline column constraint (shorter form).
///
/// Output: `REFERENCES table(col) ON DELETE action`
#[allow(dead_code)]
fn foreign_key_inline_sql(fk: &DbForeignKeyAttr) -> String {
    let mut sql = format!("REFERENCES {}", fk.references);

    if let Some(on_delete) = &fk.on_delete {
        sql.push_str(&format!(" ON DELETE {}", on_delete));
    }

    sql
}

/// Format the default value SQL fragment.
#[allow(dead_code)]
fn default_sql(value: &str) -> String {
    format!("DEFAULT {}", value)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::attributes::{
        DbDefaultAttr, DbForeignKeyAttr, DbIndexAttr, DbIndexColumns, DbUniqueConstraintAttr,
    };
    use crate::derive::schema::types::DbFieldInfo;

    fn make_simple_schema() -> DbSchemaInfo {
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
                    flatten: false,
                },
                DbFieldInfo {
                    name: "name".to_string(),
                    rust_type: "String".to_string(),
                    primary_key: false,
                    sql_type_override: Some("VARCHAR(255)".to_string()),
                    constraints: vec!["NOT NULL".to_string()],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "is_enabled".to_string(),
                    rust_type: "bool".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::Bool(true)),
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "created_at".to_string(),
                    rust_type: "DateTime<Utc>".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
                    index: None,
                    flatten: false,
                },
            ],
            vec![DbIndexAttr {
                name: "idx_shops_is_enabled".to_string(),
                on: DbIndexColumns::Single("is_enabled".to_string()),
                condition: Some("is_enabled = true".to_string()),
            }],
            vec![],
            vec![],
            false,
        )
    }

    fn make_complex_schema() -> DbSchemaInfo {
        DbSchemaInfo::new(
            "ShopInventory".to_string(),
            "shop_inventory".to_string(),
            vec![
                DbFieldInfo {
                    name: "id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: true,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("gen_random_uuid()".to_string())),
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "shop_id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "item_id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "price".to_string(),
                    rust_type: "i64".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "stock_quantity".to_string(),
                    rust_type: "i32".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::Number(2147483647)),
                    index: None,
                    flatten: false,
                },
            ],
            vec![],
            vec![
                DbForeignKeyAttr {
                    column: "shop_id".to_string(),
                    references: "shops.id".to_string(),
                    on_delete: Some("CASCADE".to_string()),
                },
                DbForeignKeyAttr {
                    column: "item_id".to_string(),
                    references: "items.id".to_string(),
                    on_delete: Some("CASCADE".to_string()),
                },
            ],
            vec![DbUniqueConstraintAttr {
                name: "unique_shop_item".to_string(),
                columns: vec!["shop_id".to_string(), "item_id".to_string()],
            }],
            false,
        )
    }

    // ── Existing tests (updated with flatten: false) ──────────────────

    #[test]
    fn test_generate_simple_create_table() {
        let schema = make_simple_schema();
        let sql = generate_create_table_sql(&schema);

        assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS shops"));
        assert!(sql.contains("id UUID PRIMARY KEY DEFAULT gen_random_uuid()"));
        assert!(sql.contains("name VARCHAR(255) NOT NULL"));
        assert!(sql.contains("is_enabled BOOLEAN NOT NULL DEFAULT true"));
        assert!(sql.contains("created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()"));
    }

    #[test]
    fn test_generate_create_table_with_fk_and_unique() {
        let schema = make_complex_schema();
        let sql = generate_create_table_sql(&schema);

        assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS shop_inventory"));
        assert!(sql.contains("FOREIGN KEY (shop_id) REFERENCES shops.id ON DELETE CASCADE"));
        assert!(sql.contains("FOREIGN KEY (item_id) REFERENCES items.id ON DELETE CASCADE"));
        assert!(sql.contains("CONSTRAINT unique_shop_item UNIQUE (shop_id, item_id)"));
    }

    #[test]
    fn test_generate_index_single_column() {
        let schema = make_simple_schema();
        let sql = generate_create_indexes_sql(&schema);

        assert!(
            sql.contains("CREATE INDEX IF NOT EXISTS idx_shops_is_enabled ON shops(is_enabled)")
        );
        assert!(sql.contains("WHERE is_enabled = true"));
    }

    #[test]
    fn test_generate_index_composite() {
        let schema = DbSchemaInfo::new(
            "Test".to_string(),
            "test_table".to_string(),
            vec![],
            vec![DbIndexAttr {
                name: "idx_comp".to_string(),
                on: DbIndexColumns::Composite(vec!["col_a".to_string(), "col_b".to_string()]),
                condition: None,
            }],
            vec![],
            vec![],
            false,
        );
        let sql = generate_create_indexes_sql(&schema);

        assert!(sql.contains("CREATE INDEX IF NOT EXISTS idx_comp ON test_table(col_a, col_b)"));
        assert!(!sql.contains("WHERE"));
    }

    #[test]
    fn test_no_indexes_returns_empty() {
        let schema = DbSchemaInfo::new(
            "Empty".to_string(),
            "empty".to_string(),
            vec![DbFieldInfo {
                name: "id".to_string(),
                rust_type: "Uuid".to_string(),
                primary_key: true,
                sql_type_override: None,
                constraints: vec![],
                default_value: None,
                index: None,
                flatten: false,
            }],
            vec![],
            vec![],
            vec![],
            false,
        );
        let sql = generate_create_indexes_sql(&schema);
        assert!(sql.is_empty());
    }

    #[test]
    fn test_full_schema_sql_combines_table_and_indexes() {
        let schema = make_simple_schema();
        let sql = generate_full_schema_sql(&schema);

        assert!(sql.contains("CREATE TABLE IF NOT EXISTS shops"));
        assert!(sql.contains("CREATE INDEX IF NOT EXISTS idx_shops_is_enabled"));
    }

    #[test]
    fn test_foreign_key_sql() {
        let fk = DbForeignKeyAttr {
            column: "shop_id".to_string(),
            references: "shops.id".to_string(),
            on_delete: Some("CASCADE".to_string()),
        };
        assert_eq!(
            foreign_key_sql(&fk),
            "FOREIGN KEY (shop_id) REFERENCES shops.id ON DELETE CASCADE"
        );

        let fk_no_delete = DbForeignKeyAttr {
            column: "item_id".to_string(),
            references: "items.id".to_string(),
            on_delete: None,
        };
        assert_eq!(
            foreign_key_sql(&fk_no_delete),
            "FOREIGN KEY (item_id) REFERENCES items.id"
        );
    }

    #[test]
    fn test_index_sql_with_condition() {
        let sql = index_sql(
            "idx_active",
            &DbIndexColumns::Single("is_active".to_string()),
            &Some("is_active = true".to_string()),
            "users",
        );
        assert_eq!(
            sql,
            "CREATE INDEX IF NOT EXISTS idx_active ON users(is_active) WHERE is_active = true"
        );
    }

    #[test]
    fn test_index_sql_composite_no_condition() {
        let sql = index_sql(
            "idx_multi",
            &DbIndexColumns::Composite(vec!["a".to_string(), "b".to_string()]),
            &None,
            "my_table",
        );
        assert_eq!(
            sql,
            "CREATE INDEX IF NOT EXISTS idx_multi ON my_table(a, b)"
        );
    }

    #[test]
    fn test_generate_schema_impl_produces_valid_tokens() {
        let schema = make_simple_schema();
        let struct_name = proc_macro2::Ident::new("Shop", proc_macro2::Span::call_site());
        let tokens = generate_schema_impl(&struct_name, &schema);

        let token_str = tokens.to_string();
        assert!(token_str.contains("TABLE_NAME"));
        assert!(token_str.contains("CREATE_TABLE_SQL"));
        assert!(token_str.contains("COLUMN_DEFS_SQL"));
        assert!(token_str.contains("CREATE_INDEXES_SQL"));
        assert!(token_str.contains("column_names"));
        assert!(token_str.contains("primary_key_field"));
    }

    // ── COLUMN_DEFS_SQL tests ─────────────────────────────────────────

    #[test]
    fn test_column_defs_sql_simple() {
        let schema = make_simple_schema();
        let sql = generate_column_defs_sql(&schema);

        // Each column on its own line, no commas, no CREATE TABLE wrapper
        assert!(sql.contains("id UUID PRIMARY KEY DEFAULT gen_random_uuid()"));
        assert!(sql.contains("name VARCHAR(255) NOT NULL"));
        assert!(sql.contains("is_enabled BOOLEAN NOT NULL DEFAULT true"));
        assert!(sql.contains("created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()"));

        // No commas in column defs SQL
        assert!(!sql.contains(","));
        // No CREATE TABLE wrapper
        assert!(!sql.contains("CREATE TABLE"));
    }

    #[test]
    fn test_column_defs_sql_with_fk_and_unique() {
        let schema = make_complex_schema();
        let sql = generate_column_defs_sql(&schema);

        // FK and unique constraints are NOT in column_defs_sql
        assert!(!sql.contains("FOREIGN KEY"));
        assert!(!sql.contains("CONSTRAINT"));
        assert!(!sql.contains("UNIQUE"));

        // But columns are present
        assert!(sql.contains("id UUID PRIMARY KEY"));
        assert!(sql.contains("price BIGINT"));
    }

    // ── #[db_flatten] tests ───────────────────────────────────────────

    fn make_flatten_schema() -> DbSchemaInfo {
        DbSchemaInfo::new(
            "PlayerPositionRecord".to_string(),
            "player_positions".to_string(),
            vec![
                DbFieldInfo {
                    name: "id".to_string(),
                    rust_type: "i64".to_string(),
                    primary_key: true,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "pos".to_string(),
                    rust_type: "Position2D".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: true,
                },
                DbFieldInfo {
                    name: "tick".to_string(),
                    rust_type: "u32".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
                DbFieldInfo {
                    name: "created_at".to_string(),
                    rust_type: "i64".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                    flatten: false,
                },
            ],
            vec![],
            vec![],
            vec![],
            true,
        )
    }

    #[test]
    fn test_flatten_skips_field_from_own_columns() {
        let schema = make_flatten_schema();
        let sql = generate_column_defs_sql(&schema);

        // Own columns are present
        assert!(sql.contains("id BIGINT PRIMARY KEY"));
        assert!(sql.contains("tick BIGINT NOT NULL"));
        assert!(sql.contains("created_at BIGINT NOT NULL"));

        // Flattened field is NOT in own columns
        assert!(!sql.contains("pos"));
        assert!(!sql.contains("Position2D"));
    }

    #[test]
    fn test_flatten_column_names_excludes_flattened() {
        let schema = make_flatten_schema();
        let names: Vec<&str> = schema
            .fields
            .iter()
            .filter(|f| !f.is_flatten())
            .map(|f| f.name.as_str())
            .collect();

        assert_eq!(names, vec!["id", "tick", "created_at"]);
    }

    #[test]
    fn test_flatten_generates_fn_not_const() {
        let schema = make_flatten_schema();
        let struct_name =
            proc_macro2::Ident::new("PlayerPositionRecord", proc_macro2::Span::call_site());
        let tokens = generate_schema_impl(&struct_name, &schema);
        let token_str = tokens.to_string();

        // Should have fn create_table_sql, not const CREATE_TABLE_SQL
        assert!(token_str.contains("create_table_sql"));
        assert!(
            !token_str.contains("CREATE_TABLE_SQL"),
            "flatten structs should use fn create_table_sql(), not const CREATE_TABLE_SQL"
        );

        // Should reference the flattened type's COLUMN_DEFS_SQL
        assert!(
            token_str.contains("COLUMN_DEFS_SQL"),
            "should reference flattened type's COLUMN_DEFS_SQL"
        );

        // Should still have TABLE_NAME and COLUMN_DEFS_SQL as const
        assert!(token_str.contains("TABLE_NAME"));
    }

    #[test]
    fn test_flatten_create_table_sql_has_own_columns() {
        let schema = make_flatten_schema();
        let sql = generate_create_table_sql(&schema);

        // generate_create_table_sql only has own columns (no flattened)
        assert!(sql.contains("id BIGINT PRIMARY KEY"));
        assert!(sql.contains("tick BIGINT NOT NULL"));
        assert!(sql.contains("created_at BIGINT NOT NULL"));
        assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS player_positions"));
    }

    #[test]
    fn test_non_flatten_generates_const() {
        let schema = make_simple_schema();
        let struct_name = proc_macro2::Ident::new("Shop", proc_macro2::Span::call_site());
        let tokens = generate_schema_impl(&struct_name, &schema);
        let token_str = tokens.to_string();

        // Should have const CREATE_TABLE_SQL
        assert!(token_str.contains("CREATE_TABLE_SQL"));
        // Should NOT have fn create_table_sql
        assert!(!token_str.contains("fn create_table_sql"));
    }

    #[test]
    fn test_generate_column_push_tokens_own_field() {
        let schema = DbSchemaInfo::new(
            "Simple".to_string(),
            "simple".to_string(),
            vec![DbFieldInfo {
                name: "id".to_string(),
                rust_type: "i64".to_string(),
                primary_key: true,
                sql_type_override: None,
                constraints: vec![],
                default_value: None,
                index: None,
                flatten: false,
            }],
            vec![],
            vec![],
            vec![],
            false,
        );

        let pushes = generate_column_push_tokens(&schema);
        assert_eq!(pushes.len(), 1);

        let token_str = pushes[0].to_string();
        assert!(token_str.contains("id BIGINT PRIMARY KEY"));
        // quote! renders method calls with spaces: `lines . push (...)`
        assert!(token_str.contains("push"));
    }

    #[test]
    fn test_generate_column_push_tokens_flatten_field() {
        let schema = DbSchemaInfo::new(
            "WithFlatten".to_string(),
            "with_flatten".to_string(),
            vec![DbFieldInfo {
                name: "data".to_string(),
                rust_type: "MyData".to_string(),
                primary_key: false,
                sql_type_override: None,
                constraints: vec![],
                default_value: None,
                index: None,
                flatten: true,
            }],
            vec![],
            vec![],
            vec![],
            false,
        );

        let pushes = generate_column_push_tokens(&schema);
        assert_eq!(pushes.len(), 1);

        let token_str = pushes[0].to_string();
        assert!(token_str.contains("MyData :: COLUMN_DEFS_SQL"));
        assert!(token_str.contains("lines"));
    }
}
