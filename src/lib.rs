#[macro_use]
extern crate diesel;

use diesel::r2d2::{self, ConnectionManager};
use diesel::sqlite::SqliteConnection;

pub mod models;
pub mod queries;
pub mod schema;

/// SQLite database connection pool
pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

/// Functions used for the initial setup
pub mod setup {
    use crate::Pool;

    use diesel::r2d2::{self, ConnectionManager};
    use diesel::sqlite::SqliteConnection;

    /// Creates a SQLite database connection pool
    pub fn create_pool(url: &str) -> Pool {
        let manager = ConnectionManager::<SqliteConnection>::new(url);
        r2d2::Pool::builder()
            .build(manager)
            .expect("Can't create pool.")
    }
}
