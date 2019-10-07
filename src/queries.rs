//! Helper functions for SQL queries

// A lot of duplicate code here could be merged by using macros

/// Queries affecting the `files` table
pub mod files {
    use crate::models::files::*;
    use crate::schema::files::dsl::*;
    use crate::schema::files::table;
    use crate::Pool;

    use actix_web::web::Data;
    use diesel::prelude::*;
    use diesel::result::QueryResult;

    /// SELECT a single file entry given its id
    pub fn find(g_id: i32, pool: Data<Pool>) -> QueryResult<File> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        files.find(g_id).first::<File>(conn)
    }

    /// INSERT a file entry
    pub fn insert(p_id: i32, p_filepath: &str, pool: Data<Pool>) -> QueryResult<File> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let new_file = NewFile {
            id: p_id,
            filepath: p_filepath,
        };
        diesel::insert_into(table).values(&new_file).execute(conn)?;

        find(p_id, pool)
    }

    /// UPDATE a file entry
    pub fn update(
        p_id: i32,
        new_id: Option<i32>,
        new_filepath: Option<&str>,
        pool: Data<Pool>,
    ) -> QueryResult<File> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let file = find(p_id, pool)?;
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

    /// DELETE a file entry
    pub fn delete(d_id: i32, pool: Data<Pool>) -> QueryResult<()> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        diesel::delete(files.find(d_id)).execute(conn)?;

        Ok(())
    }
}

/// Queries affecting the `links` table
pub mod links {
    use crate::models::links::*;
    use crate::schema::links::dsl::*;
    use crate::schema::links::table;
    use crate::Pool;

    use actix_web::web::Data;
    use diesel::prelude::*;
    use diesel::result::QueryResult;

    /// SELECT a single link entry given its id
    pub fn find(g_id: i32, pool: Data<Pool>) -> QueryResult<Link> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        links.find(g_id).first::<Link>(conn)
    }

    /// INSERT a link entry
    pub fn insert(p_id: i32, p_forward: &str, pool: Data<Pool>) -> QueryResult<Link> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let new_link = NewLink {
            id: p_id,
            forward: p_forward,
        };
        diesel::insert_into(table).values(&new_link).execute(conn)?;

        find(p_id, pool)
    }

    /// UPDATE a link entry
    pub fn update(
        p_id: i32,
        new_id: Option<i32>,
        new_forward: Option<&str>,
        pool: Data<Pool>,
    ) -> QueryResult<Link> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let link = find(p_id, pool)?;
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

    /// DELETE a link entry
    pub fn delete(d_id: i32, pool: Data<Pool>) -> QueryResult<()> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        diesel::delete(links.find(d_id)).execute(conn)?;

        Ok(())
    }
}

/// Queries affecting the `texts` table
pub mod texts {
    use crate::models::texts::*;
    use crate::schema::texts::dsl::*;
    use crate::schema::texts::table;
    use crate::Pool;

    use actix_web::web::Data;
    use diesel::prelude::*;
    use diesel::result::QueryResult;

    /// SELECT a single text entry given its id
    pub fn find(g_id: i32, pool: Data<Pool>) -> QueryResult<Text> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        texts.find(g_id).first::<Text>(conn)
    }

    /// INSERT a text entry
    pub fn insert(p_id: i32, p_contents: &str, pool: Data<Pool>) -> QueryResult<Text> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let new_text = NewText {
            id: p_id,
            contents: p_contents,
        };
        diesel::insert_into(table).values(&new_text).execute(conn)?;

        find(p_id, pool)
    }

    /// UPDATE a text entry
    pub fn update(
        p_id: i32,
        new_id: Option<i32>,
        new_contents: Option<&str>,
        pool: Data<Pool>,
    ) -> QueryResult<Text> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        let text = find(p_id, pool)?;
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

    /// DELETE a text entry
    pub fn delete(d_id: i32, pool: Data<Pool>) -> QueryResult<()> {
        let conn: &SqliteConnection = &pool.get().unwrap();
        diesel::delete(texts.find(d_id)).execute(conn)?;

        Ok(())
    }
}
