use diesel::{deserialize::{self, FromSqlRow}, expression::AsExpression, prelude::*, serialize, sql_types::Integer, sqlite::{Sqlite, SqliteValue}};

#[derive(Debug, FromSqlRow, AsExpression, Clone, Copy)]
#[diesel(sql_type = Integer)]
#[repr(i32)]
pub enum Medium {
    // Note: don't reorder it, only append new variants at the end (it's serialized as an int)
    Movie = 1,
    Series,
    AnimationMovie,
    Anime,
    Book,
    VideoGame
}

impl serialize::ToSql<Integer, Sqlite> for Medium {
    fn to_sql<'b>(&self, out: &mut serialize::Output<Sqlite>) -> serialize::Result {
        out.set_value(*self as i32);
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<Integer, Sqlite> for Medium {
    fn from_sql(bytes: SqliteValue) -> deserialize::Result<Self> {
        let value = <i32 as deserialize::FromSql<Integer, Sqlite>>::from_sql(bytes)?;
        Ok(unsafe {::std::mem::transmute(value)})
    }
    
    fn from_nullable_sql(bytes: Option<SqliteValue>) -> deserialize::Result<Self> {
        match bytes {
            Some(bytes) => Self::from_sql(bytes),
            None => Err(Box::new(diesel::result::UnexpectedNullError)),
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::oeuvres)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Oeuvre {
    pub id: i32,
    pub medium: Medium,
    pub title: String,
    pub rating: Option<i32>,
    pub synopsis: Option<String>,
    pub picture: Option<String>
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::oeuvres)]
pub struct NewOeuvre<'a> {
    pub medium: Medium,
    pub title: &'a str,
    pub rating: Option<i32>,
    pub synopsis: Option<String>,
    pub picture: Option<String>
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::tags)]
pub struct Tag {
    pub id: i32,
    pub label: String
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tags)]
pub struct NewTag {
    pub label: String
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::oeuvre_tags)]
pub struct OeuvreTag {
    pub oeuvre_id: i32,
    pub tag_id: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::imdb_map)]
pub struct ImdbMap {
    pub oeuvre_id: i32,
    pub imdb_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::imdb_map)]
pub struct NewImdbMap<'a> {
    pub oeuvre_id: i32,
    pub imdb_id: &'a str,
}