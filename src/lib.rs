#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;

use chrono::{DateTime, Utc};
use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;

pub mod models;
pub mod queries;
pub mod schema;
pub mod setup;

/// SQLite database connection pool
pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Date and time range
type DateTimeRange = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

/// Date and time range specifying ranges for creation and update
pub struct SelectRange {
    created: DateTimeRange,
    updated: DateTimeRange,
}
