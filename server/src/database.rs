use crate::schema::queue;
use anyhow::{Context, Result as AnyhowResult};
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use std::env;

pub fn establish_connection_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    let database_url = env::var(crate::DATABASE_URL_KEY).unwrap_or_else(|_| {
        panic!(
            "Missing the {} environment variable.",
            crate::DATABASE_URL_KEY
        )
    });
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
    // SqliteConnection::establish(&database_url).expect("Expected to connect to database")
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = queue)]
pub struct QueueItem {
    pub user: String,
    pub is_selected: bool,
    pub is_processed: bool,
    pub updated_at: NaiveDateTime,
}

pub fn insert_into_queue(con: &mut SqliteConnection, user: String) -> AnyhowResult<()> {
    let current_time = Utc::now().naive_utc();
    let new_queue_item = QueueItem {
        user,
        is_selected: false,
        is_processed: false,
        updated_at: current_time,
    };

    diesel::insert_into(queue::table)
        .values(new_queue_item)
        .execute(con)
        .context("Failed to insert row into database")?;
    Ok(())
}
