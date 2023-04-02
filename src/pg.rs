#[cfg(feature = "db-postgres")]
use paste::paste;
#[cfg(feature = "db-postgres")]
use sqlx::PgPool;

pub struct PgSchema {
    namespace: String,
}

impl PgSchema {
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
        }
    }

    pub fn enums(&self) -> String {
        format!("SELECT
      'CREATE TYPE ' || n.nspname || '.' || t.typname || ' AS ENUM (' || string_agg(quote_literal(e.enumlabel), ', ') || ');' AS sql
    FROM
      pg_catalog.pg_type t
      JOIN pg_catalog.pg_namespace n ON t.typnamespace = n.oid
      JOIN pg_catalog.pg_enum e ON t.oid = e.enumtypid
    WHERE
      n.nspname = '{}'
      AND t.typtype = 'e'
    GROUP BY
      n.nspname, t.typname;", self.namespace)
    }

    pub fn types(&self) -> String {
        format!("SELECT
      'CREATE TYPE ' || n.nspname || '.' || t.typname || ' AS (' || string_agg(a.attname || ' ' || pg_catalog.format_type(a.atttypid, a.atttypmod), ', ') || ');' AS sql
    FROM
      pg_catalog.pg_type t
      JOIN pg_catalog.pg_namespace n ON t.typnamespace = n.oid
      JOIN pg_catalog.pg_class c ON t.typrelid = c.oid
      JOIN pg_catalog.pg_attribute a ON t.typrelid = a.attrelid
    WHERE
      n.nspname = '{}'
      AND t.typtype = 'c'
      AND c.relkind = 'c'
    GROUP BY
      n.nspname, t.typname;", self.namespace)
    }

    pub fn tables(&self) -> String {
        format!("WITH table_columns AS (
          SELECT
            n.nspname AS schema_name,
            c.relname AS table_name,
            a.attname AS column_name,
            pg_catalog.format_type(a.atttypid, a.atttypmod) AS column_type,
            a.attnotnull AS is_not_null,
            a.attnum AS column_position
          FROM
            pg_catalog.pg_attribute a
            JOIN pg_catalog.pg_class c ON a.attrelid = c.oid
            JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid
          WHERE
            a.attnum > 0
            AND NOT a.attisdropped
            AND n.nspname = '{0}'
            AND c.relkind = 'r'
        ),
        table_constraints AS (
          SELECT
            tc.constraint_name,
            tc.table_schema,
            tc.table_name,
            kcu.column_name,
            tc.constraint_type
          FROM
            information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
              ON tc.constraint_catalog = kcu.constraint_catalog
              AND tc.constraint_schema = kcu.constraint_schema
              AND tc.constraint_name = kcu.constraint_name
          WHERE
            tc.constraint_type IN ('PRIMARY KEY', 'FOREIGN KEY', 'UNIQUE')
            AND tc.table_schema = '{0}'
        ),
        formatted_columns AS (
          SELECT
            tc.schema_name,
            tc.table_name,
            tc.column_name,
            tc.column_type,
            tc.is_not_null,
            tc.column_position,
            STRING_AGG(
              tcs.constraint_type || ' (' || tc.column_name || ')',
              ', '
              ORDER BY tcs.constraint_type DESC
            ) AS column_constraints
          FROM
            table_columns tc
            LEFT JOIN table_constraints tcs
              ON tc.schema_name = tcs.table_schema
              AND tc.table_name = tcs.table_name
              AND tc.column_name = tcs.column_name
          GROUP BY
            tc.schema_name,
            tc.table_name,
            tc.column_name,
            tc.column_type,
            tc.is_not_null,
            tc.column_position
        ),
        create_table_statements AS (
          SELECT
            fc.schema_name,
            fc.table_name,
            STRING_AGG(
              fc.column_name || ' ' || fc.column_type || (CASE WHEN fc.is_not_null THEN ' NOT NULL' ELSE '' END) || COALESCE(' ' || fc.column_constraints, ''),
              ', '
              ORDER BY fc.column_position
            ) AS formatted_columns
          FROM
            formatted_columns fc
          GROUP BY
            fc.schema_name,
            fc.table_name
        )
        SELECT
          'CREATE TABLE ' || schema_name || '.' || table_name || ' (' || formatted_columns || ');' AS sql
        FROM
          create_table_statements;", self.namespace)
    }

    pub fn views(&self) -> String {
        format!(
            "SELECT
      'CREATE VIEW ' || n.nspname || '.' || c.relname || ' AS ' || pg_get_viewdef(c.oid) AS sql
    FROM
      pg_catalog.pg_class c
      JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid
    WHERE
      c.relkind = 'v' -- Select views
      AND n.nspname = '{}';",
            self.namespace
        )
    }

    pub fn mviews(&self) -> String {
        format!("SELECT
        'CREATE MATERIALIZED VIEW ' || n.nspname || '.' || c.relname || ' AS ' || pg_get_viewdef(c.oid) AS sql
      FROM
        pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid
      WHERE
        c.relkind = 'm' -- Select materialized views
        AND n.nspname = '{}';", self.namespace)
    }

    pub fn functions(&self) -> String {
        format!("SELECT
      'CREATE OR REPLACE FUNCTION ' || n.nspname || '.' || p.proname || '(' || pg_get_function_arguments(p.oid) || ') RETURNS '
      || pg_get_function_result(p.oid) || ' AS $function_body$ ' || pg_get_functiondef(p.oid) || '$function_body$ LANGUAGE ' || l.lanname || ';' AS sql
    FROM
      pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON p.pronamespace = n.oid
      JOIN pg_catalog.pg_language l ON p.prolang = l.oid
    WHERE
      n.nspname = '{}'
      AND p.prokind = 'f';", self.namespace)
    }

    pub fn triggers(&self) -> String {
        format!(
            "SELECT
      'CREATE TRIGGER ' || t.tgname
      || ' ' || CASE
        WHEN t.tgtype & 2 > 0 THEN 'BEFORE'
        WHEN t.tgtype & 4 > 0 THEN 'AFTER'
        WHEN t.tgtype & 64 > 0 THEN 'INSTEAD OF'
      END
      || ' ' || CASE
        WHEN t.tgtype & 8 > 0 THEN 'INSERT'
        WHEN t.tgtype & 16 > 0 THEN 'DELETE'
        WHEN t.tgtype & 32 > 0 THEN 'UPDATE'
      END
      || ' ON ' || n.nspname || '.' || c.relname
      || ' FOR EACH ' || CASE WHEN t.tgtype & 1 > 0 THEN 'ROW' ELSE 'STATEMENT' END
      || ' EXECUTE FUNCTION ' || np.nspname || '.' || p.proname || '();' AS sql
    FROM
      pg_catalog.pg_trigger t
      JOIN pg_catalog.pg_class c ON t.tgrelid = c.oid
      JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid
      JOIN pg_catalog.pg_proc p ON t.tgfoid = p.oid
      JOIN pg_catalog.pg_namespace np ON p.pronamespace = np.oid
    WHERE
      n.nspname = '{}'
      AND NOT t.tgisinternal;",
            self.namespace
        )
    }

    pub fn indexes(&self) -> String {
        format!("SELECT indexdef || ';' AS sql FROM pg_indexes WHERE schemaname = '{}' ORDER BY tablename, indexname;", self.namespace)
    }
}

#[cfg(feature = "db-postgres")]
#[derive(sqlx::FromRow)]
pub struct SchemaRet {
    sql: String,
}

#[cfg(feature = "db-postgres")]
macro_rules! gen_fn {
  ($($name:ident),*) => {
      $(
        paste! {
          pub async fn [<get_ $name>] (&self, pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
              let sql = self.$name();
              let ret: Vec<SchemaRet> = sqlx::query_as(&sql).fetch_all(pool).await?;
              Ok(ret.into_iter().map(|r| r.sql).collect())
          }
        }
      )*
  };
}

#[cfg(feature = "db-postgres")]
impl PgSchema {
    gen_fn!(enums, types, tables, views, mviews, functions, triggers, indexes);
}

#[cfg(feature = "db-postgres")]
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    #[tokio::test]
    async fn get_tables_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_tables(&pool).await?;
        assert_eq!(items.len(), 4);
        assert_eq!(
          items[0],
            "CREATE TABLE gpt.comments (id integer NOT NULL PRIMARY KEY (id), post_id integer NOT NULL FOREIGN KEY (post_id), user_id integer NOT NULL FOREIGN KEY (user_id), content text NOT NULL, created_at timestamp with time zone NOT NULL, updated_at timestamp with time zone NOT NULL);"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_enums_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_enums(&pool).await?;
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0],
            "CREATE TYPE gpt.login_method AS ENUM ('email', 'google', 'github');"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_types_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_types(&pool).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            "CREATE TYPE gpt.address AS (street character varying(255), city character varying(100), state character(2), postal_code character(5));"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_views_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_views(&pool).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            "CREATE VIEW gpt.posts_with_comments AS  SELECT p.id,\n    p.user_id,\n    p.title,\n    p.content,\n    p.status,\n    p.published_at,\n    p.created_at,\n    p.updated_at,\n    json_agg(json_build_object('id', c.id, 'user_id', c.user_id, 'content', c.content, 'created_at', c.created_at, 'updated_at', c.updated_at)) AS comments\n   FROM (gpt.posts p\n     LEFT JOIN gpt.comments c ON ((c.post_id = p.id)))\n  GROUP BY p.id;"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_mviews_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_mviews(&pool).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            "CREATE MATERIALIZED VIEW gpt.users_with_posts AS  SELECT u.id,\n    u.username,\n    u.email,\n    u.first_name,\n    u.last_name,\n    u.created_at,\n    u.updated_at,\n    json_agg(json_build_object('id', p.id, 'title', p.title, 'content', p.content, 'status', p.status, 'published_at', p.published_at, 'created_at', p.created_at, 'updated_at', p.updated_at)) AS posts\n   FROM (gpt.users u\n     LEFT JOIN gpt.posts p ON ((p.user_id = u.id)))\n  GROUP BY u.id;"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_functions_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_functions(&pool).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            "CREATE OR REPLACE FUNCTION gpt.refresh_users_with_posts() RETURNS trigger AS $function_body$ CREATE OR REPLACE FUNCTION gpt.refresh_users_with_posts()\n RETURNS trigger\n LANGUAGE plpgsql\nAS $function$\nBEGIN\n  REFRESH MATERIALIZED VIEW gpt.users_with_posts;\n  RETURN NULL;\nEND;\n$function$\n$function_body$ LANGUAGE plpgsql;"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_triggers_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_triggers(&pool).await?;
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0],
            "CREATE TRIGGER refresh_users_with_posts AFTER INSERT ON gpt.posts FOR EACH STATEMENT EXECUTE FUNCTION gpt.refresh_users_with_posts();"
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_indexes_should_work() -> Result<()> {
        let schema = PgSchema::new("gpt");
        let tdb = TestPg::default();
        let pool = tdb.get_pool().await;
        let items = schema.get_indexes(&pool).await?;
        assert_eq!(items.len(), 8);
        assert_eq!(
            items[0],
            "CREATE UNIQUE INDEX comments_pkey ON gpt.comments USING btree (id);"
        );

        Ok(())
    }
}
