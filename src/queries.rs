//! Helper functions for SQL queries

// A lot of duplicate code here could be merged by using macros
// but that would make adding different fields more troublesome

/// Date and time range specifying ranges for creation and update
pub struct SelectRange {
    /// Creation time range
    pub created: (Option<i32>, Option<i32>),
    /// Update time range
    pub updated: (Option<i32>, Option<i32>),
}

/// Query string for SELECT queries
#[derive(Deserialize)]
pub struct SelectQuery {
    /// Left creation bounder timestamp
    pub cf: Option<i32>,
    /// Right creation bounder timestamp
    pub ct: Option<i32>,
    /// Left update bounder timestamp
    pub uf: Option<i32>,
    /// Right update bounder timestamp
    pub ut: Option<i32>,
    /// Query size limit
    pub limit: Option<i64>,
    /// Whether to sort the results in ascending order
    pub asc: Option<bool>,
    /// Whether to sort the results by creation date
    pub created: Option<bool>,
}

/// Filters for SELECT queries
pub struct SelectFilters {
    /// Creation and update date and time ranges
    pub range: SelectRange,
    /// Query size limit
    pub limit: Option<i64>,
    /// Whether to sort the results in ascending order
    pub order_asc: bool,
    /// Whether to sort the results by creation date
    pub order_created: bool,
}

impl From<SelectQuery> for SelectFilters {
    fn from(query: SelectQuery) -> Self {
        SelectFilters {
            range: SelectRange {
                created: (query.cf, query.ct),
                updated: (query.uf, query.ut),
            },
            limit: query.limit,
            order_asc: query.asc.unwrap_or(false),
            order_created: query.created.unwrap_or(false),
        }
    }
}

/// Code common to all select functions
macro_rules! common_select {
    ($q:expr, $f:expr) => {
        if let Some(cf) = $f.range.created.0 {
            $q = $q.filter(created.ge(cf));
        }
        if let Some(ct) = $f.range.created.1 {
            $q = $q.filter(created.lt(ct));
        }
        if let Some(uf) = $f.range.updated.0 {
            $q = $q.filter(updated.ge(uf));
        }
        if let Some(ut) = $f.range.updated.1 {
            $q = $q.filter(updated.lt(ut));
        }

        if let Some(limit) = $f.limit {
            $q = $q.limit(limit);
        }

        $q = match ($f.order_asc, $f.order_created) {
            (false, false) => $q.order(updated.desc()),
            (true, false) => $q.order(updated.asc()),
            (false, true) => $q.order(created.desc()),
            (true, true) => $q.order(created.asc()),
        };
    };
}

/// SELECT a single entry given its id
macro_rules! find {
    ($n:ident, $t:ty) => {
        pub fn find(f_id: i32, pool: Data<Pool>) -> QueryResult<$t> {
            let conn: &SqliteConnection = &pool.get().unwrap();
            $n.find(f_id).first::<$t>(conn)
        }
    };
}

/// DELETE an entry
macro_rules! delete {
    ($n:ident) => {
        pub fn delete(d_id: i32, pool: Data<Pool>) -> QueryResult<()> {
            let conn: &SqliteConnection = &pool.get().unwrap();
            diesel::delete($n.find(d_id)).execute(conn)?;

            Ok(())
        }
    };
}

/// Queries affecting the `files` table
pub mod files {
    use crate::models::files::*;
    use crate::queries::SelectFilters;
    use crate::schema::files::dsl::*;
    use crate::schema::files::table;
    use crate::Pool;

    use actix_web::web::Data;
    use diesel::prelude::*;
    use diesel::result::QueryResult;

    /// SELECT multiple file entries
    pub fn select(filters: SelectFilters, pool: Data<Pool>) -> QueryResult<Vec<File>> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let mut query = files.into_boxed();
        common_select!(query, filters);
        query.load::<File>(conn)
    }

    find!(files, File);

    /// REPLACE a file entry
    pub fn replace(r_id: i32, r_filepath: &str, pool: Data<Pool>) -> QueryResult<File> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let new_file = NewFile {
            id: r_id,
            filepath: r_filepath,
        };
        diesel::replace_into(table)
            .values(&new_file)
            .execute(conn)?;

        find(r_id, pool)
    }

    /// UPDATE a file entry
    pub fn update(
        u_id: i32,
        new_id: Option<i32>,
        new_filepath: Option<&str>,
        pool: Data<Pool>,
    ) -> QueryResult<File> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let file = find(u_id, pool)?;
        let query = diesel::update(&file);
        let time_update = updated.eq(chrono::Utc::now().timestamp() as i32);
        match (new_id, new_filepath) {
            (Some(new_id), Some(new_filepath)) => {
                query
                    .set((id.eq(new_id), filepath.eq(new_filepath), time_update))
                    .execute(conn)?;
            }
            (Some(new_id), None) => {
                query.set((id.eq(new_id), time_update)).execute(conn)?;
            }
            (None, Some(new_filepath)) => {
                query
                    .set((filepath.eq(new_filepath), time_update))
                    .execute(conn)?;
            }
            (None, None) => {
                return Ok(file);
            }
        }

        Ok(file)
    }

    delete!(files);
}

/// Queries affecting the `links` table
pub mod links {
    use crate::models::links::*;
    use crate::queries::SelectFilters;
    use crate::schema::links::dsl::*;
    use crate::schema::links::table;
    use crate::Pool;

    use actix_web::web::Data;
    use diesel::prelude::*;
    use diesel::result::QueryResult;

    /// SELECT multiple link entries
    pub fn select(filters: SelectFilters, pool: Data<Pool>) -> QueryResult<Vec<Link>> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let mut query = links.into_boxed();
        common_select!(query, filters);
        query.load::<Link>(conn)
    }

    find!(links, Link);

    /// REPLACE a link entry
    pub fn replace(r_id: i32, r_forward: &str, pool: Data<Pool>) -> QueryResult<Link> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let new_link = NewLink {
            id: r_id,
            forward: r_forward,
        };
        diesel::replace_into(table)
            .values(&new_link)
            .execute(conn)?;

        find(r_id, pool)
    }

    /// UPDATE a link entry
    pub fn update(
        u_id: i32,
        new_id: Option<i32>,
        new_forward: Option<&str>,
        pool: Data<Pool>,
    ) -> QueryResult<Link> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let link = find(u_id, pool)?;
        let query = diesel::update(&link);
        let time_update = updated.eq(chrono::Utc::now().timestamp() as i32);
        match (new_id, new_forward) {
            (Some(new_id), Some(new_forward)) => {
                query
                    .set((id.eq(new_id), forward.eq(new_forward), time_update))
                    .execute(conn)?;
            }
            (Some(new_id), None) => {
                query.set((id.eq(new_id), time_update)).execute(conn)?;
            }
            (None, Some(new_forward)) => {
                query
                    .set((forward.eq(new_forward), time_update))
                    .execute(conn)?;
            }
            (None, None) => (),
        }

        Ok(link)
    }

    delete!(links);
}

/// Queries affecting the `texts` table
pub mod texts {
    use crate::models::texts::*;
    use crate::queries::SelectFilters;
    use crate::schema::texts::dsl::*;
    use crate::schema::texts::table;
    use crate::Pool;

    use actix_web::web::Data;
    use diesel::prelude::*;
    use diesel::result::QueryResult;

    /// SELECT multiple text entries
    pub fn select(filters: SelectFilters, pool: Data<Pool>) -> QueryResult<Vec<Text>> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let mut query = texts.into_boxed();
        common_select!(query, filters);
        query.load::<Text>(conn)
    }

    find!(texts, Text);

    /// REPLACE a text entry
    pub fn replace(r_id: i32, r_contents: &str, pool: Data<Pool>) -> QueryResult<Text> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let new_text = NewText {
            id: r_id,
            contents: r_contents,
        };
        diesel::replace_into(table)
            .values(&new_text)
            .execute(conn)?;

        find(r_id, pool)
    }

    /// UPDATE a text entry
    pub fn update(
        u_id: i32,
        new_id: Option<i32>,
        new_contents: Option<&str>,
        pool: Data<Pool>,
    ) -> QueryResult<Text> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let text = find(u_id, pool)?;
        let query = diesel::update(&text);
        let time_update = updated.eq(chrono::Utc::now().timestamp() as i32);
        match (new_id, new_contents) {
            (Some(new_id), Some(new_contents)) => {
                query
                    .set((id.eq(new_id), contents.eq(new_contents), time_update))
                    .execute(conn)?;
            }
            (Some(new_id), None) => {
                query.set((id.eq(new_id), time_update)).execute(conn)?;
            }
            (None, Some(new_contents)) => {
                query
                    .set((contents.eq(new_contents), time_update))
                    .execute(conn)?;
            }
            (None, None) => (),
        }

        Ok(text)
    }

    delete!(texts);
}
