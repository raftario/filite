//! Helper functions for SQL queries

/// Query string for SELECT queries
#[derive(Deserialize)]
pub struct SelectQuery {
    /// Left creation bounder timestamp
    pub from: Option<i32>,
    /// Right creation bounder timestamp
    pub to: Option<i32>,
    /// Query size limit
    pub limit: Option<i64>,
    /// Whether to sort the results in ascending order
    pub asc: Option<bool>,
}

/// Filters for SELECT queries
pub struct SelectFilters {
    /// Creation and update date and time ranges
    pub range: (Option<i32>, Option<i32>),
    /// Query size limit
    pub limit: Option<i64>,
    /// Whether to sort the results in ascending order
    pub asc: bool,
}

impl From<SelectQuery> for SelectFilters {
    fn from(query: SelectQuery) -> Self {
        SelectFilters {
            range: (query.from, query.to),
            limit: query.limit,
            asc: query.asc.unwrap_or(false),
        }
    }
}

/// Code common to all select functions
macro_rules! common_select {
    ($q:expr, $f:expr) => {
        if let Some(from) = $f.range.0 {
            $q = $q.filter(created.ge(from));
        }
        if let Some(to) = $f.range.1 {
            $q = $q.filter(created.lt(to));
        }
        if let Some(limit) = $f.limit {
            $q = $q.limit(limit);
        }
        $q = if $f.asc {
            $q.order(created.asc())
        } else {
            $q.order(created.desc())
        };
    };
}

/// SELECT a single entry given its id
macro_rules! find {
    ($n:ident, $t:ty) => {
        pub fn find(f_id: i32) -> diesel::result::QueryResult<$t> {
            let conn: &SqliteConnection = &crate::globals::POOL.get().unwrap();
            $n.find(f_id).first::<$t>(conn)
        }
    };
}

/// DELETE an entry
macro_rules! delete {
    ($n:ident) => {
        pub fn delete(d_id: i32) -> diesel::result::QueryResult<()> {
            let conn: &SqliteConnection = &crate::globals::POOL.get().unwrap();
            diesel::delete($n.find(d_id)).execute(conn)?;
            Ok(())
        }
    };
}

/// Queries affecting the `files` table
pub mod files {
    use crate::{
        globals::POOL,
        models::files::*,
        queries::SelectFilters,
        schema::files::{dsl::*, table},
    };
    use diesel::{prelude::*, result::QueryResult};

    find!(files, File);
    delete!(files);

    /// SELECT multiple file entries
    pub fn select(filters: SelectFilters) -> QueryResult<Vec<File>> {
        let conn: &SqliteConnection = &POOL.get().unwrap();
        let mut query = files.into_boxed();
        common_select!(query, filters);
        query.load::<File>(conn)
    }

    /// REPLACE a file entry
    pub fn replace(r_id: i32, r_filepath: &str) -> QueryResult<File> {
        let conn: &SqliteConnection = &POOL.get().unwrap();
        let new_file = NewFile {
            id: r_id,
            filepath: r_filepath,
        };
        diesel::replace_into(table)
            .values(&new_file)
            .execute(conn)?;
        find(r_id)
    }
}

/// Queries affecting the `links` table
pub mod links {
    use crate::{
        globals::POOL,
        models::links::*,
        queries::SelectFilters,
        schema::links::{dsl::*, table},
    };
    use diesel::{prelude::*, result::QueryResult};

    find!(links, Link);
    delete!(links);

    /// SELECT multiple link entries
    pub fn select(filters: SelectFilters) -> QueryResult<Vec<Link>> {
        let conn: &SqliteConnection = &POOL.get().unwrap();
        let mut query = links.into_boxed();
        common_select!(query, filters);
        query.load::<Link>(conn)
    }

    /// REPLACE a link entry
    pub fn replace(r_id: i32, r_forward: &str) -> QueryResult<Link> {
        let conn: &SqliteConnection = &POOL.get().unwrap();
        let new_link = NewLink {
            id: r_id,
            forward: r_forward,
        };
        diesel::replace_into(table)
            .values(&new_link)
            .execute(conn)?;
        find(r_id)
    }
}

/// Queries affecting the `texts` table
pub mod texts {
    use crate::{
        globals::POOL,
        models::texts::*,
        queries::SelectFilters,
        schema::texts::{dsl::*, table},
    };
    use diesel::{prelude::*, result::QueryResult};

    find!(texts, Text);
    delete!(texts);

    /// SELECT multiple text entries
    pub fn select(filters: SelectFilters) -> QueryResult<Vec<Text>> {
        let conn: &SqliteConnection = &POOL.get().unwrap();
        let mut query = texts.into_boxed();
        common_select!(query, filters);
        query.load::<Text>(conn)
    }

    /// REPLACE a text entry
    pub fn replace(r_id: i32, r_contents: &str, r_highlight: bool) -> QueryResult<Text> {
        let conn: &SqliteConnection = &POOL.get().unwrap();
        let new_text = NewText {
            id: r_id,
            contents: r_contents,
            highlight: r_highlight,
        };
        diesel::replace_into(table)
            .values(&new_text)
            .execute(conn)?;
        find(r_id)
    }
}
