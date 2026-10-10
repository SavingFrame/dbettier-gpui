use futures::TryStreamExt;
use std::{collections::HashMap, time::Instant};

use sqlx::{
    Column, Connection, Either, Executor, PgPool, Row, SqlSafeStr, TypeInfo, ValueRef,
    postgres::{PgConnection, PgRow},
    query,
    types::chrono::{DateTime, Utc},
};

use crate::database::{
    ConstraintType, DatabaseSchema, DatabaseTable, LoadState, TableColumn, TableConstraint,
    TableIndex,
    result::{CellValue, QueryOutput, ResultColumn},
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
        row.try_get(0)
    }

    pub async fn list_schemas(&self) -> Result<Vec<DatabaseSchema>, sqlx::Error> {
        let rows = sqlx::query("SELECT schema_name FROM information_schema.schemata")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(DatabaseSchema {
                    name: row.try_get(0)?,
                    tables: LoadState::NotLoaded,
                })
            })
            .collect()
    }

    pub async fn list_tables(&self, schema_name: &str) -> Result<Vec<DatabaseTable>, sqlx::Error> {
        let rows =
            sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = $1 ORDER BY table_name")
                .bind(schema_name)
                .fetch_all(&self.pool)
                .await?;
        let mut tables: Vec<DatabaseTable> = rows
            .into_iter()
            .map(|row| {
                Ok(DatabaseTable {
                    name: row.try_get(0)?,
                    columns: Vec::new(),
                    constraints: Vec::new(),
                    indexes: Vec::new(),
                })
            })
            .collect::<Result<_, sqlx::Error>>()?;

        self.load_columns_for_tables(schema_name, &mut tables)
            .await?;
        self.load_constraints_for_tables(schema_name, &mut tables)
            .await?;
        self.load_indexes_for_tables(schema_name, &mut tables)
            .await?;
        Ok(tables)
    }

    async fn load_columns_for_tables(
        &self,
        schema_name: &str,
        tables: &mut [DatabaseTable],
    ) -> Result<(), sqlx::Error> {
        let rows = sqlx::query("SELECT table_name, column_name, column_default, is_nullable, data_type, character_maximum_length, is_identity
        FROM information_schema.columns
        WHERE table_schema = $1
        order by ordinal_position").bind(schema_name).fetch_all(&self.pool).await?;
        let mut columns_by_table: HashMap<String, Vec<TableColumn>> = HashMap::new();

        for row in rows {
            let table_name: String = row.try_get("table_name")?;

            let raw_is_identity: String = row.try_get("is_identity")?;
            let column_default: Option<String> = row.try_get("column_default")?;

            let is_auto_increment = raw_is_identity == "YES"
                || column_default
                    .as_deref()
                    .is_some_and(|default| default.starts_with("nextval("));

            let table_column = TableColumn {
                name: row.try_get("column_name")?,
                default: column_default,
                is_nullable: row.try_get::<String, _>("is_nullable")? == "YES",
                data_type: row.try_get("data_type")?,
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
        for table in tables {
            table.columns = columns_by_table.remove(&table.name).unwrap_or_default();
        }
        Ok(())
    }

    async fn load_constraints_for_tables(
        &self,
        schema_name: &str,
        tables: &mut [DatabaseTable],
    ) -> Result<(), sqlx::Error> {
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
            let table_name: String = row.try_get("table_name")?;

            let table_constraint = TableConstraint {
                name: row.try_get("constraint_name")?,
                constraint_type: ConstraintType::from_str(row.try_get("constraint_type")?),
                is_deferrable: row.try_get::<String, _>("is_deferrable")? == "YES",
                nulls_distinct: row
                    .try_get::<Option<String>, _>("nulls_distinct")?
                    .map(|v| v == "YES"),
            };
            constraints_by_table
                .entry(table_name)
                .or_default()
                .push(table_constraint);
        }
        for table in tables {
            table.constraints = constraints_by_table.remove(&table.name).unwrap_or_default();
        }
        Ok(())
    }

    async fn load_indexes_for_tables(
        &self,
        schema_name: &str,
        tables: &mut [DatabaseTable],
    ) -> Result<(), sqlx::Error> {
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
            let table_name: String = row.try_get("table_name")?;
            indexes_by_table
                .entry(table_name)
                .or_default()
                .push(TableIndex {
                    name: row.try_get("index_name")?,
                    definition: row.try_get("definition")?,
                    is_unique: row.try_get("is_unique")?,
                    is_primary: row.try_get("is_primary")?,
                });
        }

        for table in tables {
            table.indexes = indexes_by_table.remove(&table.name).unwrap_or_default();
        }
        Ok(())
    }

    pub async fn execute_query(&self, query: String) -> Result<QueryOutput, sqlx::Error> {
        let started = Instant::now();

        let sql = sqlx::AssertSqlSafe(query).into_sql_str();
        let mut stream = sqlx::raw_sql(sql.clone()).fetch_many(&self.pool);
        let mut columns = None;

        let mut rows = Vec::new();
        let mut rows_affected = 0;

        while let Some(result) = stream.try_next().await? {
            match result {
                Either::Left(result) => rows_affected += result.rows_affected(),
                Either::Right(row) => {
                    columns.get_or_insert_with(|| {
                        row.columns()
                            .iter()
                            .map(|column| ResultColumn {
                                name: column.name().to_owned(),
                                column_type: column.type_info().name().to_owned(),
                            })
                            .collect()
                    });
                    rows.push(decode_row(row)?);
                }
            }
        }
        let columns = match columns {
            Some(columns) => columns,
            None => {
                let description = self.pool.describe(sql).await?;
                description
                    .columns()
                    .iter()
                    .map(|column| ResultColumn {
                        name: column.name().to_owned(),
                        column_type: column.type_info().name().to_owned(),
                    })
                    .collect()
            }
        };
        Ok(QueryOutput {
            rows,
            rows_affected,
            columns,
            elapsed: started.elapsed(),
        })
    }

    pub async fn new(connection_uri: String) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(&connection_uri).await?;
        Ok(Self { pool })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
fn decode_cell(row: &PgRow, index: usize) -> Result<CellValue, sqlx::Error> {
    let value = row.try_get_raw(index)?;

    if value.is_null() {
        return Ok(CellValue::Null);
    }

    let cell = match value.type_info().name() {
        "BOOL" => CellValue::Boolean(row.try_get(index)?),
        "INT2" => CellValue::Integer(i64::from(row.try_get::<i16, _>(index)?)),
        "INT4" => CellValue::Integer(i64::from(row.try_get::<i32, _>(index)?)),
        "INT8" => CellValue::Integer(row.try_get(index)?),
        "FLOAT4" => CellValue::Float(f64::from(row.try_get::<f32, _>(index)?)),
        "FLOAT8" => CellValue::Float(row.try_get(index)?),
        "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => CellValue::Text(row.try_get(index)?),
        "BYTEA" => CellValue::Bytes(row.try_get(index)?),
        "UUID" => CellValue::Uuid(row.try_get::<uuid::Uuid, _>(index)?),
        "JSONB" | "JSON" => CellValue::Json(row.try_get(index)?),
        "TIMESTAMPTZ" => {
            let timestamp = row.try_get::<DateTime<Utc>, _>(index)?;
            CellValue::Timestamp(timestamp.to_string())
        }
        type_name => {
            return Err(sqlx::Error::Decode(
                std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!("Unsupported PostgreSQL type: {type_name}"),
                )
                .into(),
            ));
        }
    };

    Ok(cell)
}

fn decode_row(row: PgRow) -> Result<Vec<CellValue>, sqlx::Error> {
    (0..row.len())
        .map(|index| decode_cell(&row, index))
        .collect()
}
