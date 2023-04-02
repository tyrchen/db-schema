# README

This crate provides a set of functions to generate SQL statements for various PostgreSQL schema objects, such as tables, views, materialized views, functions, triggers, and indexes. The generated SQL statements can be useful for schema introspection, documentation, or migration purposes.

## Features

The module provides a `PgSchema` struct that accepts a namespace (schema name) as input and exposes methods for generating SQL statements for the following schema objects:

- Enums
- Composite types
- Tables
- Views
- Materialized views
- Functions
- Triggers
- Indexes

## Usage

1. Create an instance of `PgSchema` with the desired namespace (schema name).

rust

```rust
use db_schema::PgSchema;

let schema = PgSchema::new("your_schema_name");
```

2. Use the available methods to generate SQL statements for the desired schema objects.

rust

```rust
// Get the SQL statements for all enums in the schema
let enums_sql = schema.enums();

// Get the SQL statements for all composite types in the schema
let types_sql = schema.types();

// Get the SQL statements for all tables in the schema
let tables_sql = schema.tables();

// Get the SQL statements for all views in the schema
let views_sql = schema.views();

// Get the SQL statements for all materialized views in the schema
let mviews_sql = schema.mviews();

// Get the SQL statements for all functions in the schema
let functions_sql = schema.functions();

// Get the SQL statements for all triggers in the schema
let triggers_sql = schema.triggers();

// Get the SQL statements for all indexes in the schema
let indexes_sql = schema.indexes();
```

You can also use the `get_*` methods to generate SQL statements for the desired schema objects. These methods accept a `PgPool` instance as input and return a `Result<Vec<String>, sqlx::Error>`.

## Example

Here's an example of how to retrieve the SQL statements for all schema objects in a given namespace (schema name):

rust

```rust
use db_schema::PgSchema;
use sqlx::PgPool;

async fn generate_sql_statements_for_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    let schema = PgSchema::new("your_schema_name");

    let enums = schema.get_enums(pool).await?;
    let types = schema.get_types(pool).await?;
    let tables = schema.get_tables(pool).await?;
    let views = schema.get_views(pool).await?;
    let mviews = schema.get_mviews(pool).await?;
    let functions = schema.get_functions(pool).await?;
    let triggers = schema.get_triggers(pool).await?;
    let indexes = schema.get_indexes(pool).await?;

    println!("Enums: {:?}", enums);
    println!("Types: {:?}", types);
    println!("Tables: {:?}", tables);
    println!("Views: {:?}", views);
    println!("Materialized Views: {:?}", mviews);
    println!("Functions: {:?}", functions);
    println!("Triggers: {:?}", triggers);
    println!("Indexes: {:?}", indexes);

    Ok(())
}
```

## Tests

The code also includes tests to validate the functionality of the `PgSchema` struct. To run the tests, execute the following command:

sh

```sh
cargo nextest run
```
