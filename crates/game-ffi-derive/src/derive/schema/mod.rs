//! Database schema generation from GameComponent derive attributes (Plan 082)
//!
//! This module generates SQL DDL and CRUD code from `#[db_table]` and related attributes:
//!
//! - `CREATE TABLE` statements from struct definitions
//! - `CREATE INDEX` statements from `#[db_index]` attributes
//! - Foreign key constraints from `#[db_foreign_key]` attributes
//! - Unique constraints from `#[db_unique_constraint]` attributes
//! - Type mapping between Rust types and PostgreSQL types
//! - CRUD methods (`insert`, `find_by_id`, `update`, `delete`, etc.)

pub mod crud_gen;
pub mod sql_gen;
pub mod types;
