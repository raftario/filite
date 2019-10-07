//! Database models

/// Models from the `files` table
pub mod files {
    use crate::schema::files;

    /// An entry from the `files` table
    #[derive(Queryable, Identifiable)]
    pub struct File {
        pub id: i32,
        pub filepath: String,
        pub created: i32,
        pub updated: i32,
    }

    /// A new entry to the `files` table
    #[derive(Insertable)]
    #[table_name = "files"]
    pub struct NewFile<'a> {
        pub id: i32,
        pub filepath: &'a str,
    }
}

/// Models from the `links` table
pub mod links {
    use crate::schema::links;

    /// An entry from the `links` table
    #[derive(Queryable, Identifiable)]
    pub struct Link {
        pub id: i32,
        pub forward: String,
        pub created: i32,
        pub updated: i32,
    }

    /// A new entry to the `links` table
    #[derive(Insertable)]
    #[table_name = "links"]
    pub struct NewLink<'a> {
        pub id: i32,
        pub forward: &'a str,
    }
}

/// Models from the `texts` table
pub mod texts {
    use crate::schema::texts;

    /// An entry from the `texts` table
    #[derive(Queryable, Identifiable)]
    pub struct Text {
        pub id: i32,
        pub contents: String,
        pub created: i32,
        pub updated: i32,
    }

    /// A new entry to the `texts` table
    #[derive(Insertable)]
    #[table_name = "texts"]
    pub struct NewText<'a> {
        pub id: i32,
        pub contents: &'a str,
    }
}
