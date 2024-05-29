use chrono::NaiveDateTime;
use sqlx::postgres::PgPoolOptions;
use sqlx::Error;
use sqlx::Row;

pub struct Database {
    pool: sqlx::Pool<sqlx::Postgres>,
    table_name: String,
}

impl Database {
    // Initialize the database connection pool
    pub async fn new(database_url: &str, table_name: &str) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self {
            pool,
            table_name: table_name.to_string(),
        })
    }

    // Method to create a table with a given name
    pub async fn create_table(&self) -> Result<(), Error> {
        let query = format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                id SERIAL PRIMARY KEY,
                action TEXT NOT NULL,
                file_size BIGINT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            "#,
            self.table_name
        );
        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }

    // Method to add an action in the table
    pub async fn add(&self, action: &str) -> Result<(), Error> {
        let query = format!("INSERT INTO {} (action) VALUES ($1)", self.table_name);
        sqlx::query(&query).bind(action).execute(&self.pool).await?;
        Ok(())
    }

    // Method to record file size with a specified action
    pub async fn filesize(&self, size: i64) -> Result<(), Error> {
        let query = format!(
            "INSERT INTO {} (action, file_size) VALUES ($1, $2)",
            self.table_name
        );
        sqlx::query(&query)
            .bind("file_downloaded")
            .bind(size)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Method to get a value by action from the table
    pub async fn get(&self, action: &str) -> Result<Option<(i32, String, NaiveDateTime)>, Error> {
        let query = format!(
            "SELECT id, action, created_at FROM {} WHERE action = $1",
            self.table_name
        );
        let row = sqlx::query(&query)
            .bind(action)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let id: i32 = row.get("id");
            let action: String = row.get("action");
            let created_at: NaiveDateTime = row.get("created_at");
            Ok(Some((id, action, created_at)))
        } else {
            Ok(None)
        }
    }

    // Method to drop the table
    pub async fn drop_table(&self) -> Result<(), Error> {
        let query = format!("DROP TABLE IF EXISTS {}", self.table_name);
        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    #[ignore]
    async fn test_add_and_get() {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let table_name = "kv_store_test";
        let db = Database::new(&database_url, table_name).await.unwrap();

        db.create_table().await.unwrap();
        db.add("foo").await.unwrap();
        let value = db.get("foo").await.unwrap();
        if let Some((id, action, created_at)) = value {
            println!("id: {}, action: {}, created_at: {}", id, action, created_at);
            assert_eq!(action, "foo");
        } else {
            panic!("No value found");
        }

        // Test the filesize method
        db.filesize(1068845).await.unwrap();

        // Drop the table after the test
        db.drop_table().await.unwrap();
    }
}
