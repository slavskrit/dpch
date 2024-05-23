

use sqlx::postgres::PgPoolOptions;
use sqlx::Error;
use sqlx::Row;
use tokio::runtime::Runtime;

pub struct Database {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Database {
    // Initialize the database connection pool
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }

    // Method to create a table with a given name
    pub async fn create_table(&self, table_name: &str) -> Result<(), Error> {
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
            table_name
        );
        sqlx::query(&query)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Method to set a key-value pair in a table with a given name
    pub async fn set(&self, table_name: &str, key: &str, value: &str) -> Result<(), Error> {
        let query = format!(
            "INSERT INTO {} (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = $2",
            table_name
        );
        sqlx::query(&query)
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Method to get a value by key from a table with a given name
    pub async fn get(&self, table_name: &str, key: &str) -> Result<Option<String>, Error> {
        let query = format!("SELECT value FROM {} WHERE key = $1", table_name);
        let row = sqlx::query(&query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    // Method to drop a table with a given name
    pub async fn drop_table(&self, table_name: &str) -> Result<(), Error> {
        let query = format!("DROP TABLE IF EXISTS {}", table_name);
        sqlx::query(&query)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_set_and_get() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = Database::new(&database_url).await.unwrap();
        let table_name = "kv_store_test";
        db.create_table(table_name).await.unwrap();

        db.set(table_name, "foo", "bar").await.unwrap();
        let value = db.get(table_name, "foo").await.unwrap();
        assert_eq!(value, Some("bar".to_string()));

        // Drop the table after the test
        db.drop_table(table_name).await.unwrap();
    }
}
