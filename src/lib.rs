#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;

use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;

pub mod models;
pub mod queries;
pub mod schema;
pub mod setup;

/// SQLite database connection pool
pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
