//! Database models

/// Models from the `files` table
pub mod files {
    use crate::schema::files;

    /// An entry from the `files` table
    #[derive(Queryable, Identifiable, Serialize)]
    pub struct File {
        /// Primary key, its radix 36 value is used as an url
        pub id: i32,
        /// Path to the file to serve relative to the static files root
        pub filepath: String,
        /// Creation date and time as a UNIX timestamp
        pub created: i32,
    }

    /// A new entry to the `files` table
    #[derive(Insertable)]
    #[table_name = "files"]
    pub struct NewFile<'a> {
        /// Primary key, its radix 36 value is used as an url
        pub id: i32,
        /// Path to the file to serve relative to the static files root
        pub filepath: &'a str,
    }
}

/// Models from the `links` table
pub mod links {
    use crate::schema::links;

    /// An entry from the `links` table
    #[derive(Queryable, Identifiable, Serialize)]
    pub struct Link {
        /// Primary key, its radix 36 value is used as an url
        pub id: i32,
        /// URL this link forwards to
        pub forward: String,
        /// Creation date and time as a UNIX timestamp
        pub created: i32,
    }

    /// A new entry to the `links` table
    #[derive(Insertable)]
    #[table_name = "links"]
    pub struct NewLink<'a> {
        /// Primary key, its radix 36 value is used as an url
        pub id: i32,
        /// URL this link forwards to
        pub forward: &'a str,
    }
}

/// Models from the `texts` table
pub mod texts {
    use crate::schema::texts;

    /// An entry from the `texts` table
    #[derive(Queryable, Identifiable, Serialize)]
    pub struct Text {
        /// Primary key, its radix 36 value is used as an url
        pub id: i32,
        /// Text contents
        pub contents: String,
        /// Creation date and time as a UNIX timestamp
        pub created: i32,
    }

    /// A new entry to the `texts` table
    #[derive(Insertable)]
    #[table_name = "texts"]
    pub struct NewText<'a> {
        /// Primary key, its radix 36 value is used as an url
        pub id: i32,
        /// Text contents
        pub contents: &'a str,
    }
}
