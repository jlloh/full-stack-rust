use crate::schema::queue;
use crate::ANONYMOUS;
use anyhow::{anyhow, Context, Result as AnyhowResult};
use chrono::prelude::*;
use common::UserInfo;
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
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = queue)]
pub struct InsertQueueItem {
    pub user: String,
    pub is_selected: bool,
    pub is_processed: bool,
    pub is_abandoned: bool,
    pub updated_at: NaiveDateTime,
}

/// Insert new entry into queue. Fails if user already exists in queue
pub fn insert_into_queue(con: &mut SqliteConnection, user: String) -> AnyhowResult<()> {
    let current_time = Utc::now().naive_utc();
    let new_queue_item = InsertQueueItem {
        user,
        is_selected: false,
        is_processed: false,
        is_abandoned: false,
        updated_at: current_time,
    };

    diesel::insert_into(queue::table)
        .values(new_queue_item)
        .execute(con)
        .context("Failed to insert row into database")?;
    Ok(())
}

#[derive(Queryable)]
pub struct QueueRow {
    pub id: i32,
    pub user: String,
    pub is_selected: bool,
    pub is_processed: bool,
    pub is_abandoned: bool,
    pub updated_at: NaiveDateTime,
}

// Given user's email, try to obtain current assigned queue
pub fn get_user_assigned_queue(
    con: &mut SqliteConnection,
    provided_user: &str,
) -> AnyhowResult<Option<i32>> {
    // If user is anonymous, skip database check
    if provided_user == ANONYMOUS {
        return Ok(None);
    }

    let results = queue::table
        .filter(queue::user.eq(provided_user))
        .filter(queue::is_processed.eq(false))
        .filter(queue::is_abandoned.eq(false))
        .load::<QueueRow>(con)
        .context("Failed to query queue table")?;

    match results.len() {
        0 => Ok(None),
        1 => Ok(Some(results[0].id)),
        _ => Err(anyhow!(
            "Expected 0 or 1 row. Found {} rows instead.",
            results.len()
        )),
    }
}

/// Helper function
pub fn get_or_insert(con: &mut SqliteConnection, user_struct: UserInfo) -> AnyhowResult<UserInfo> {
    let x = get_user_assigned_queue(con, &user_struct.email)?;

    match x {
        Some(number) => {
            // Already has a number, we return
            let mut result = user_struct.clone();
            result.assigned_number = Some(number);
            Ok(result)
        }
        None => {
            // No number. So we assign a number
            insert_into_queue(con, user_struct.email.clone())?;
            // recurse function
            get_or_insert(con, user_struct)
        }
    }
}

/// Abandon assigned_number
pub fn set_to_abandoned(con: &mut SqliteConnection, user_struct: UserInfo) -> AnyhowResult<()> {
    if let Some(number) = get_user_assigned_queue(con, &user_struct.email)? {
        diesel::update(queue::table)
            .filter(queue::id.eq(number))
            .set(queue::is_abandoned.eq(true))
            .execute(con)
            .context("Failed to set to abandoned")?;
    }
    Ok(())
}

/// Get current queue
pub fn get_selected_queue(con: &mut SqliteConnection) -> AnyhowResult<Option<i32>> {
    let results = queue::table
        .filter(queue::is_selected.eq(true))
        .filter(queue::is_processed.eq(false))
        .filter(queue::is_abandoned.eq(false))
        .load::<QueueRow>(con)
        .context("Failed to query queue table")?;

    match results.len() {
        0 => Ok(None),
        1 => Ok(Some(results[0].id)),
        _ => Err(anyhow!(
            "Expected 0 or 1 row. Found {} rows instead.",
            results.len()
        )),
    }
}
