use std::collections::HashMap;

use sqlx::{Connection, PgPool, Row, postgres::PgConnection};

use crate::database::{
    ConstraintType, DatabaseSchema, DatabaseTable, LoadState, TableColumn, TableConstraint,
    TableIndex,
};

#[derive(Clone)]
pub struct PostgresDriver {
    pool: PgPool,
}

impl PostgresDriver {
    pub async fn test_connection(connection_uri: &str) -> Result<String, sqlx::Error> {
        let mut conn = PgConnection::connect(connection_uri).await?;
        let row = sqlx::query("SELECT version();")
            .fetch_one(&mut conn)
            .await?;
        Ok(row.get(0))
    }

    pub async fn list_schemas(&self) -> Result<Vec<DatabaseSchema>, sqlx::Error> {
        sqlx::query("SELECT schema_name FROM information_schema.schemata")
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| DatabaseSchema {
                        name: row.get(0),
                        tables: LoadState::NotLoaded,
                    })
                    .collect()
            })
    }

    pub async fn list_tables(&self, schema_name: &str) -> Result<Vec<DatabaseTable>, sqlx::Error> {
        self.load_tables_for_schema(schema_name).await
    }

    async fn load_tables_for_schema(
        &self,
        schema_name: &str,
    ) -> Result<Vec<DatabaseTable>, sqlx::Error> {
        let tables: Vec<DatabaseTable> =
            sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = $1")
                .bind(schema_name)
                .fetch_all(&self.pool)
                .await
                .map(|rows| {
                    rows.into_iter()
                        .map(|row| DatabaseTable {
                            name: row.get(0),
                            columns: Vec::new(),
                            constraints: Vec::new(),
                            indexes: Vec::new(),
                        })
                        .collect()
                })?;

        let tables = self.load_columns_for_tables(schema_name, tables).await?;
        let tables = self
            .load_constraints_for_tables(schema_name, tables)
            .await?;
        self.load_indexes_for_tables(schema_name, tables).await
    }

    async fn load_columns_for_tables(
        &self,
        schema_name: &str,
        mut tables: Vec<DatabaseTable>,
    ) -> Result<Vec<DatabaseTable>, sqlx::Error> {
        let rows = sqlx::query("SELECT table_name, column_name, column_default, is_nullable, data_type, character_maximum_length, is_identity
        FROM information_schema.columns
        WHERE table_schema = $1
        order by ordinal_position").bind(schema_name).fetch_all(&self.pool).await?;
        let mut columns_by_table: HashMap<String, Vec<TableColumn>> = HashMap::new();

        for row in rows {
            let table_name: String = row.get("table_name");

            let raw_is_identity: String = row.get("is_identity");
            let column_default: Option<String> = row.get("column_default");

            let is_auto_increment = raw_is_identity == "YES"
                || column_default
                    .as_deref()
                    .is_some_and(|default| default.starts_with("nextval("));

            let table_column = TableColumn {
                name: row.get("column_name"),
                default: row.get("column_default"),
                is_nullable: row.get::<String, _>("is_nullable") == "YES",
                data_type: row.get("data_type"),
                character_maximum_length: row
                    .try_get::<Option<i32>, _>("character_maximum_length")?
                    .map(|value| value as u32),
                is_auto_increment,
            };
            columns_by_table
                .entry(table_name)
                .or_default()
                .push(table_column);
        }
        for table in &mut tables {
            table.columns = columns_by_table.remove(&table.name).unwrap_or_default();
        }
        Ok(tables)
    }

    async fn load_constraints_for_tables(
        &self,
        schema_name: &str,
        mut tables: Vec<DatabaseTable>,
    ) -> Result<Vec<DatabaseTable>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT constraint_name, table_name, constraint_type, is_deferrable, nulls_distinct
        FROM information_schema.table_constraints
        WHERE table_schema = $1;",
        )
        .bind(schema_name)
        .fetch_all(&self.pool)
        .await?;
        let mut constraints_by_table: HashMap<String, Vec<TableConstraint>> = HashMap::new();

        for row in rows {
            let table_name: String = row.get("table_name");

            let table_constraint = TableConstraint {
                name: row.get("constraint_name"),
                constraint_type: ConstraintType::from_str(row.get("constraint_type")),
                is_deferrable: row.get::<String, _>("is_deferrable") == "YES",
                nulls_distinct: row
                    .try_get::<Option<String>, _>("nulls_distinct")?
                    .map(|v| v == "YES"),
            };
            constraints_by_table
                .entry(table_name)
                .or_default()
                .push(table_constraint);
        }
        for table in &mut tables {
            table.constraints = constraints_by_table.remove(&table.name).unwrap_or_default();
        }
        Ok(tables)
    }

    async fn load_indexes_for_tables(
        &self,
        schema_name: &str,
        mut tables: Vec<DatabaseTable>,
    ) -> Result<Vec<DatabaseTable>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT table_class.relname AS table_name,
                    index_class.relname AS index_name,
                    index_metadata.indisunique AS is_unique,
                    index_metadata.indisprimary AS is_primary,
                    pg_get_indexdef(index_metadata.indexrelid) AS definition
             FROM pg_catalog.pg_index AS index_metadata
             JOIN pg_catalog.pg_class AS table_class
               ON table_class.oid = index_metadata.indrelid
             JOIN pg_catalog.pg_class AS index_class
               ON index_class.oid = index_metadata.indexrelid
             JOIN pg_catalog.pg_namespace AS namespace
               ON namespace.oid = table_class.relnamespace
             WHERE namespace.nspname = $1
             ORDER BY table_class.relname, index_class.relname",
        )
        .bind(schema_name)
        .fetch_all(&self.pool)
        .await?;
        let mut indexes_by_table: HashMap<String, Vec<TableIndex>> = HashMap::new();

        for row in rows {
            let table_name: String = row.get("table_name");
            indexes_by_table
                .entry(table_name)
                .or_default()
                .push(TableIndex {
                    name: row.get("index_name"),
                    definition: row.get("definition"),
                    is_unique: row.get("is_unique"),
                    is_primary: row.get("is_primary"),
                });
        }

        for table in &mut tables {
            table.indexes = indexes_by_table.remove(&table.name).unwrap_or_default();
        }
        Ok(tables)
    }

    pub async fn new(connection_uri: String) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(&connection_uri).await?;
        Ok(Self { pool })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
