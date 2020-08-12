use chrono::NaiveDateTime;

pub struct User {
    pub id: String,
    pub password: String,
    pub role: Role,
}

#[derive(sqlx::Type)]
#[repr(i64)]
pub enum Role {
    User = 0,
    Admin = 255,
}

pub struct Filite {
    pub id: String,
    pub ty: Type,
    pub val: String,

    pub creator: String,
    pub created: NaiveDateTime,

    pub visibility: Visibility,
    pub views: i64,
}

#[derive(sqlx::Type)]
#[repr(i64)]
pub enum Type {
    Fi = 0,
    Li = 1,
    Te = 2,
}

#[derive(sqlx::Type)]
#[repr(i64)]
pub enum Visibility {
    Public = 0,
    Protected = 1,
    Private = 2,
}
