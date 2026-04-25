//! SQL DDL generation from DbSchemaInfo (Plan 082)
//!
//! Generates:
//! - `CREATE TABLE IF NOT EXISTS` statements with columns, foreign keys, and unique constraints
//! - `CREATE INDEX IF NOT EXISTS` statements from `#[db_index]` attributes
//! - Rust `const` strings for embeddable SQL constants via `quote`

use proc_macro2::TokenStream;
use quote::quote;

use super::crud_gen::generate_crud_impl;
use super::types::DbSchemaInfo;
use crate::derive::attributes::{DbForeignKeyAttr, DbIndexColumns};

// ============================================================================
// SQL DDL Generation
// ============================================================================

/// Generate a `CREATE TABLE IF NOT EXISTS` SQL statement from a `DbSchemaInfo`.
///
/// Output example:
/// ```sql
/// CREATE TABLE IF NOT EXISTS shops (
///     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
///     name VARCHAR(255) NOT NULL,
///     shop_type VARCHAR(50) NOT NULL DEFAULT 'general',
///     npc_id UUID REFERENCES npcs(id) ON DELETE SET NULL,
///     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
///     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
///     CONSTRAINT unique_shop_name UNIQUE (name)
/// )
/// ```
pub fn generate_create_table_sql(schema: &DbSchemaInfo) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Column definitions
    for field in &schema.fields {
        if let Some(col_sql) = field.column_sql() {
            lines.push(format!("    {}", col_sql));
        }
    }

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
///
/// Output example:
/// ```sql
/// CREATE INDEX IF NOT EXISTS idx_shops_npc_id ON shops(npc_id);
/// CREATE INDEX IF NOT EXISTS idx_shop_inventory_category ON shop_inventory(shop_id, category);
/// CREATE INDEX IF NOT EXISTS idx_active ON shops(is_enabled) WHERE is_enabled = true;
/// ```
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

/// Generate Rust `impl` block with SQL constants and helper methods.
///
/// Generates:
/// - `CREATE_TABLE_SQL` const
/// - `CREATE_INDEXES_SQL` const (if indexes exist)
/// - `TABLE_NAME` const
/// - `primary_key_field()` method
/// - `column_names()` method
pub fn generate_schema_impl(
    struct_name: &proc_macro2::Ident,
    schema: &DbSchemaInfo,
) -> TokenStream {
    let table_name = &schema.table_name;
    let create_table_sql = generate_create_table_sql(schema);
    let create_indexes_sql = generate_create_indexes_sql(schema);

    // Column name constants
    let column_names: Vec<String> = schema.fields.iter().map(|f| f.name.clone()).collect();
    let column_count = column_names.len();

    // Primary key field name
    let pk_field = schema.primary_key().map(|f| f.name.clone());

    // Generate index SQL const only if indexes exist
    let index_sql_tokens = if create_indexes_sql.is_empty() {
        quote! {}
    } else {
        quote! {
            /// Auto-generated CREATE INDEX SQL (Plan 082)
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

    // Plan 082 Phase 2: CRUD methods (from_row, insert, find_by_id, etc.)
    let crud_impl = generate_crud_impl(struct_name, schema);

    quote! {
        /// Auto-generated database schema implementation (Plan 082)
        impl #struct_name {
            /// Database table name
            pub const TABLE_NAME: &'static str = #table_name;

            /// Auto-generated CREATE TABLE SQL (Plan 082)
            pub const CREATE_TABLE_SQL: &'static str = #create_table_sql;

            #index_sql_tokens

            /// Get all column names in struct field order
            pub fn column_names() -> &'static [&'static str] {
                &[#(#column_names),*]
            }

            /// Get the number of columns
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
/// Output: `FOREIGN KEY (column) REFERENCES table(col) ON DELETE action`
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
                },
                DbFieldInfo {
                    name: "name".to_string(),
                    rust_type: "String".to_string(),
                    primary_key: false,
                    sql_type_override: Some("VARCHAR(255)".to_string()),
                    constraints: vec!["NOT NULL".to_string()],
                    default_value: None,
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
                    name: "created_at".to_string(),
                    rust_type: "DateTime<Utc>".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::String("NOW()".to_string())),
                    index: None,
                },
            ],
            vec![DbIndexAttr {
                name: "idx_shops_is_enabled".to_string(),
                on: DbIndexColumns::Single("is_enabled".to_string()),
                condition: Some("is_enabled = true".to_string()),
            }],
            vec![],
            vec![],
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
                },
                DbFieldInfo {
                    name: "shop_id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                },
                DbFieldInfo {
                    name: "item_id".to_string(),
                    rust_type: "Uuid".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                },
                DbFieldInfo {
                    name: "price".to_string(),
                    rust_type: "i64".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: None,
                    index: None,
                },
                DbFieldInfo {
                    name: "stock_quantity".to_string(),
                    rust_type: "i32".to_string(),
                    primary_key: false,
                    sql_type_override: None,
                    constraints: vec![],
                    default_value: Some(DbDefaultAttr::Number(2147483647)),
                    index: None,
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
        )
    }

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
            }],
            vec![],
            vec![],
            vec![],
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

        // Verify the generated token stream contains expected items
        let token_str = tokens.to_string();
        assert!(token_str.contains("TABLE_NAME"));
        assert!(token_str.contains("CREATE_TABLE_SQL"));
        assert!(token_str.contains("CREATE_INDEXES_SQL"));
        assert!(token_str.contains("column_names"));
        assert!(token_str.contains("primary_key_field"));
    }
}
