use diesel::prelude::*;
use super::{Tag, NewTag, OeuvreTag};
use crate::schema::{oeuvres, tags, oeuvre_tags};

pub fn add_tag(conn: &mut SqliteConnection, oeuvre_id: i32, tag_label: String) -> diesel::result::QueryResult<()> {
    let tag: Tag = if let Some(tag) = tags::table.filter(tags::label.eq(tag_label.clone())).first(conn).optional()? {
        tag
    } else {
        let new_tag = NewTag { label: tag_label };
        diesel::insert_into(tags::table)
            .values(&new_tag)
            .returning(Tag::as_returning())
            .get_result(conn)?
    };

    let tagged_oeuvre = OeuvreTag {
        oeuvre_id: oeuvre_id,
        tag_id: tag.id,
    };

    diesel::insert_into(oeuvre_tags::table)
        .values(&tagged_oeuvre)
        .execute(conn)?;

    diesel::result::QueryResult::Ok(())
}

pub fn update_rating(conn: &mut SqliteConnection, oeuvre_id: i32, rating_on_100: i32) -> diesel::result::QueryResult<()> {
    diesel::update(oeuvres::table.find(oeuvre_id)).set(oeuvres::rating.eq(rating_on_100)).execute(conn)?;
    diesel::result::QueryResult::Ok(())
}