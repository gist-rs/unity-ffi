//! Database schema types and Rust-to-PostgreSQL type mapping (Plan 082)
//!
//! Provides:
//! - Type mapping from Rust types to PostgreSQL column types
//! - `DbFieldInfo` — parsed info about a struct field for SQL generation
//! - `DbSchemaInfo` — parsed info about a struct for SQL DDL generation

use crate::derive::attributes::{
    DbColumnAttr, DbDefaultAttr, DbForeignKeyAttr, DbIndexAttr, DbUniqueConstraintAttr,
};

// ============================================================================
// Type Mapping: Rust → PostgreSQL
// ============================================================================

/// Map a Rust type string to the default PostgreSQL column type.
///
/// Returns `None` for types that have no sensible default mapping.
///
/// | Rust Type | PostgreSQL Type |
/// |-----------|-----------------|
/// | `uuid::Uuid` | `UUID` |
/// | `String` | `TEXT` |
/// | `i32` | `INTEGER` |
/// | `i64` | `BIGINT` |
/// | `u32` | `INTEGER` |
/// | `u64` | `BIGINT` |
/// | `f32` | `REAL` |
/// | `f64` | `DOUBLE PRECISION` |
/// | `bool` | `BOOLEAN` |
/// | `DateTime<Utc>` | `TIMESTAMPTZ` |
/// | `Option<T>` | nullable version of T's mapping |
/// | `Vec<u8>` | `BYTEA` |
/// | `serde_json::Value` | `JSONB` |
pub fn map_rust_type_to_sql(rust_type: &str) -> Option<String> {
    // Strip whitespace
    let ty = rust_type.trim();

    // Handle Option<T> — strip outer Option, result is nullable
    if let Some(inner) = ty.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
        return map_rust_type_to_sql(inner);
    }

    // Direct type mappings
    match ty {
        "uuid::Uuid" | "Uuid" => Some("UUID".to_string()),
        "String" => Some("TEXT".to_string()),
        "i16" | "i8" => Some("SMALLINT".to_string()),
        "i32" => Some("INTEGER".to_string()),
        "i64" => Some("BIGINT".to_string()),
        "u8" => Some("SMALLINT".to_string()),
        "u16" => Some("INTEGER".to_string()),
        "u32" => Some("BIGINT".to_string()),
        "u64" => Some("BIGINT".to_string()),
        "f32" => Some("REAL".to_string()),
        "f64" => Some("DOUBLE PRECISION".to_string()),
        "bool" => Some("BOOLEAN".to_string()),
        "DateTime<Utc>" | "DateTime<FixedOffset>" => Some("TIMESTAMPTZ".to_string()),
        "NaiveDateTime" => Some("TIMESTAMP".to_string()),
        "NaiveDate" => Some("DATE".to_string()),
        "NaiveTime" => Some("TIME".to_string()),
        "Vec<u8>" => Some("BYTEA".to_string()),
        "serde_json::Value" | "Value" => Some("JSONB".to_string()),
        _ => None,
    }
}

/// Check if a Rust type string represents `Option<T>`.
pub fn is_option_type(rust_type: &str) -> bool {
    rust_type.trim().starts_with("Option<")
}

/// Extract the inner type from `Option<T>`, or return the type as-is.
pub fn unwrap_option_type(rust_type: &str) -> &str {
    let ty = rust_type.trim();
    ty.strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(ty)
}

// ============================================================================
// Schema Info Types
// ============================================================================

/// Information about a single struct field, parsed for SQL generation.
#[derive(Debug, Clone)]
pub struct DbFieldInfo {
    /// Rust field name (used as SQL column name by default)
    pub name: String,
    /// Rust type string (e.g., "uuid::Uuid", "Option<String>")
    pub rust_type: String,
    /// Whether this field is a primary key
    pub primary_key: bool,
    /// Explicit SQL column type override from `#[db_column(TYPE)]`
    pub sql_type_override: Option<String>,
    /// Additional SQL constraints from `#[db_column(_, CONSTRAINTS)]`
    pub constraints: Vec<String>,
    /// Database default value from `#[db_default(...)]`
    pub default_value: Option<DbDefaultAttr>,
    /// Field-level index from `#[db_index(...)]`
    pub index: Option<DbIndexAttr>,
    /// Whether this field's type columns should be flattened into the parent table
    pub flatten: bool,
}

impl DbFieldInfo {
    /// Build a `DbFieldInfo` from raw field data and parsed attributes.
    pub fn new(
        name: String,
        rust_type: String,
        primary_key: bool,
        db_column: Option<&DbColumnAttr>,
        db_default: Option<&DbDefaultAttr>,
        db_index: Option<&DbIndexAttr>,
        flatten: bool,
    ) -> Self {
        let (sql_type_override, constraints) = match db_column {
            Some(col) => (col.sql_type.clone(), col.constraints.clone()),
            None => (None, Vec::new()),
        };

        Self {
            name,
            rust_type,
            primary_key,
            sql_type_override,
            constraints,
            default_value: db_default.cloned(),
            index: db_index.cloned(),
            flatten,
        }
    }

    /// Get the effective SQL type for this field.
    ///
    /// Priority:
    /// 1. Explicit override from `#[db_column(TYPE)]`
    /// 2. Auto-mapped from Rust type
    pub fn sql_type(&self) -> Option<String> {
        if let Some(ref override_type) = self.sql_type_override {
            return Some(override_type.clone());
        }
        map_rust_type_to_sql(&self.rust_type)
    }

    /// Whether this field is nullable (Rust `Option<T>`).
    pub fn is_nullable(&self) -> bool {
        is_option_type(&self.rust_type)
    }

    /// Format the column default SQL fragment, e.g., `DEFAULT 'general'` or `DEFAULT 24`.
    pub fn default_sql(&self) -> Option<String> {
        self.default_value.as_ref().map(|v| match v {
            DbDefaultAttr::String(s) => format!("DEFAULT {}", s),
            DbDefaultAttr::Number(n) => format!("DEFAULT {}", n),
            DbDefaultAttr::Float(f) => format!("DEFAULT {}", f),
            DbDefaultAttr::Bool(b) => format!("DEFAULT {}", b),
        })
    }

    /// Whether this field should be skipped from direct column SQL generation
    /// (flattened fields are expanded from their embedded type's columns).
    pub fn is_flatten(&self) -> bool {
        self.flatten
    }

    /// Build the full column definition SQL fragment for a CREATE TABLE statement.
    ///
    /// Example output: `"id UUID PRIMARY KEY DEFAULT gen_random_uuid()"`
    pub fn column_sql(&self) -> Option<String> {
        let sql_type = self.sql_type()?;

        let mut parts = vec![self.name.clone(), sql_type];

        // Primary key
        if self.primary_key {
            parts.push("PRIMARY KEY".to_string());
        } else {
            // NOT NULL constraint (only for non-Option types without explicit nullable)
            if !self.is_nullable() && !self.constraints.contains(&"NOT NULL".to_string()) {
                parts.push("NOT NULL".to_string());
            }
        }

        // Default value
        if let Some(default_sql) = self.default_sql() {
            parts.push(default_sql);
        }

        // Additional constraints from #[db_column(_, ...)]
        for constraint in &self.constraints {
            if !parts.iter().any(|p| p == constraint) {
                parts.push(constraint.clone());
            }
        }

        Some(parts.join(" "))
    }
}

/// Full schema information parsed from a struct with `#[db_table]`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DbSchemaInfo {
    /// The struct name (Rust type name)
    #[allow(dead_code)]
    pub struct_name: String,
    /// Database table name from `#[db_table("name")]`
    pub table_name: String,
    /// All fields with their database metadata
    pub fields: Vec<DbFieldInfo>,
    /// Table-level indexes from `#[db_index(...)]` on the struct
    pub table_indexes: Vec<DbIndexAttr>,
    /// Foreign key constraints from `#[db_foreign_key(...)]`
    pub foreign_keys: Vec<DbForeignKeyAttr>,
    /// Unique constraints from `#[db_unique_constraint(...)]`
    pub unique_constraints: Vec<DbUniqueConstraintAttr>,
    /// Skip CRUD generation (schema-only: TABLE_NAME, CREATE_TABLE_SQL, CREATE_INDEXES_SQL)
    pub skip_crud: bool,
}

impl DbSchemaInfo {
    /// Create a new `DbSchemaInfo` from parsed struct data.
    pub fn new(
        struct_name: String,
        table_name: String,
        fields: Vec<DbFieldInfo>,
        table_indexes: Vec<DbIndexAttr>,
        foreign_keys: Vec<DbForeignKeyAttr>,
        unique_constraints: Vec<DbUniqueConstraintAttr>,
        skip_crud: bool,
    ) -> Self {
        Self {
            struct_name,
            table_name,
            fields,
            table_indexes,
            foreign_keys,
            unique_constraints,
            skip_crud,
        }
    }

    /// Find the primary key field, if any.
    pub fn primary_key(&self) -> Option<&DbFieldInfo> {
        self.fields.iter().find(|f| f.primary_key)
    }

    /// Get all field-level indexes.
    pub fn field_indexes(&self) -> Vec<&DbIndexAttr> {
        self.fields
            .iter()
            .filter_map(|f| f.index.as_ref())
            .collect()
    }

    /// Get all indexes (table-level + field-level).
    pub fn all_indexes(&self) -> Vec<&DbIndexAttr> {
        let mut indexes: Vec<&DbIndexAttr> = self.table_indexes.iter().collect();
        indexes.extend(self.field_indexes());
        indexes
    }

    /// Get the `struct_name` as a snake_case string suitable for Rust module/const names.
    #[allow(dead_code)]
    pub fn snake_case_name(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.struct_name.char_indices() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Get the uppercase snake_case name for SQL constants.
    #[allow(dead_code)]
    pub fn upper_snake_name(&self) -> String {
        self.snake_case_name().to_uppercase()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_rust_type_to_sql() {
        assert_eq!(map_rust_type_to_sql("uuid::Uuid"), Some("UUID".to_string()));
        assert_eq!(map_rust_type_to_sql("Uuid"), Some("UUID".to_string()));
        assert_eq!(map_rust_type_to_sql("String"), Some("TEXT".to_string()));
        assert_eq!(map_rust_type_to_sql("i32"), Some("INTEGER".to_string()));
        assert_eq!(map_rust_type_to_sql("i64"), Some("BIGINT".to_string()));
        assert_eq!(map_rust_type_to_sql("u32"), Some("BIGINT".to_string()));
        assert_eq!(map_rust_type_to_sql("f32"), Some("REAL".to_string()));
        assert_eq!(
            map_rust_type_to_sql("f64"),
            Some("DOUBLE PRECISION".to_string())
        );
        assert_eq!(map_rust_type_to_sql("bool"), Some("BOOLEAN".to_string()));
        assert_eq!(
            map_rust_type_to_sql("DateTime<Utc>"),
            Some("TIMESTAMPTZ".to_string())
        );
        assert_eq!(map_rust_type_to_sql("Vec<u8>"), Some("BYTEA".to_string()));
        assert_eq!(map_rust_type_to_sql("Unknown"), None);
    }

    #[test]
    fn test_option_type_detection() {
        assert!(is_option_type("Option<Uuid>"));
        assert!(is_option_type("Option<String>"));
        assert!(!is_option_type("Uuid"));
        assert!(!is_option_type("String"));
    }

    #[test]
    fn test_unwrap_option() {
        assert_eq!(unwrap_option_type("Option<Uuid>"), "Uuid");
        assert_eq!(unwrap_option_type("Uuid"), "Uuid");
        assert_eq!(unwrap_option_type("Option<DateTime<Utc>>"), "DateTime<Utc>");
    }

    #[test]
    fn test_option_type_maps_to_same_sql() {
        // Option<T> should map to the same SQL type as T (nullable handled separately)
        assert_eq!(
            map_rust_type_to_sql("Option<Uuid>"),
            Some("UUID".to_string())
        );
        assert_eq!(
            map_rust_type_to_sql("Option<String>"),
            Some("TEXT".to_string())
        );
        assert_eq!(
            map_rust_type_to_sql("Option<i32>"),
            Some("INTEGER".to_string())
        );
    }

    #[test]
    fn test_db_field_info_column_sql() {
        let field = DbFieldInfo {
            name: "id".to_string(),
            rust_type: "Uuid".to_string(),
            primary_key: true,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::String("gen_random_uuid()".to_string())),
            index: None,
            flatten: false,
        };
        assert_eq!(
            field.column_sql(),
            Some("id UUID PRIMARY KEY DEFAULT gen_random_uuid()".to_string())
        );
    }

    #[test]
    fn test_db_field_info_nullable() {
        let field = DbFieldInfo {
            name: "description".to_string(),
            rust_type: "Option<String>".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: None,
            index: None,
            flatten: false,
        };
        assert!(field.is_nullable());
        // Nullable fields should NOT get NOT NULL
        assert_eq!(field.column_sql(), Some("description TEXT".to_string()));
    }

    #[test]
    fn test_db_field_info_with_constraints() {
        let field = DbFieldInfo {
            name: "name".to_string(),
            rust_type: "String".to_string(),
            primary_key: false,
            sql_type_override: Some("VARCHAR(255)".to_string()),
            constraints: vec!["NOT NULL".to_string()],
            default_value: None,
            index: None,
            flatten: false,
        };
        assert_eq!(
            field.column_sql(),
            Some("name VARCHAR(255) NOT NULL".to_string())
        );
    }

    #[test]
    fn test_db_field_info_number_default() {
        let field = DbFieldInfo {
            name: "restock_interval_hours".to_string(),
            rust_type: "i32".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::Number(24)),
            index: None,
            flatten: false,
        };
        assert_eq!(
            field.column_sql(),
            Some("restock_interval_hours INTEGER NOT NULL DEFAULT 24".to_string())
        );
    }

    #[test]
    fn test_db_field_info_bool_default() {
        let field = DbFieldInfo {
            name: "is_enabled".to_string(),
            rust_type: "bool".to_string(),
            primary_key: false,
            sql_type_override: None,
            constraints: vec![],
            default_value: Some(DbDefaultAttr::Bool(true)),
            index: None,
            flatten: false,
        };
        assert_eq!(
            field.column_sql(),
            Some("is_enabled BOOLEAN NOT NULL DEFAULT true".to_string())
        );
    }

    #[test]
    fn test_snake_case_conversion() {
        let schema = DbSchemaInfo::new(
            "ShopInventory".to_string(),
            "shop_inventory".to_string(),
            vec![],
            vec![],
            vec![],
            vec![],
            false,
        );
        assert_eq!(schema.snake_case_name(), "shop_inventory");
        assert_eq!(schema.upper_snake_name(), "SHOP_INVENTORY");
    }
}
