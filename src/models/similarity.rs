use diesel::prelude::*;
use super::{Medium, Oeuvre, UserSimilarity};

fn order_user_id(user1_id: i32, user2_id: i32) -> (i32, i32) {
    if user1_id < user2_id {
        (user1_id, user2_id)
    } else {
        (user2_id, user1_id)
    }
}

fn update_similarity(conn: &mut SqliteConnection, user1_id: i32, user2_id: i32, delta: i32) -> diesel::result::QueryResult<()> {
    use crate::schema::users_similarity;

    let (user1_id, user2_id) = order_user_id(user1_id, user2_id);

    if let Ok(user_similarity) = users_similarity::table.filter(users_similarity::user1_id.eq(user1_id))
        .filter(users_similarity::user2_id.eq(user2_id))
        .first::<UserSimilarity>(conn) 
    {
        diesel::update(users_similarity::table
            .filter(users_similarity::user1_id.eq(user1_id))
            .filter(users_similarity::user2_id.eq(user2_id)))
            .set(users_similarity::score.eq(user_similarity.score+delta))
            .execute(conn)?;
    } else {
        diesel::insert_into(users_similarity::table)
            .values(&UserSimilarity {
                user1_id, user2_id, score: delta
            })
            .execute(conn)?;
    }
    diesel::result::QueryResult::Ok(())
}

fn similarity(rating1: i32, rating2: i32) -> i32 {
    match (rating1-rating2).abs() {
        0 => 1,
        1 => 0,
        2 => -1,
        _ => -2
    }
}

pub fn on_rating_update(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, old_rating: i32, new_rating: i32) -> diesel::result::QueryResult<()> {
    use crate::schema::user_ratings;

    let other_ratings: Vec<(i32, i32)> = user_ratings::table
        .filter(user_ratings::oeuvre_id.eq(oeuvre_id))
        .select((user_ratings::user_id, user_ratings::rating))
        .load(conn)?;
    for (other_user_id, other_rating) in other_ratings {
        if other_user_id == user_id { continue; }
        let delta = similarity(new_rating, other_rating) + similarity(old_rating, other_rating);
        update_similarity(conn, user_id, other_user_id, delta)?;
    }
    diesel::result::QueryResult::Ok(())
}

pub fn on_rating_add(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, rating: i32) -> diesel::result::QueryResult<()> {
    use crate::schema::user_ratings;

    let other_ratings: Vec<(i32, i32)> = user_ratings::table
        .filter(user_ratings::oeuvre_id.eq(oeuvre_id))
        .select((user_ratings::user_id, user_ratings::rating))
        .load(conn)?;

    for (other_user_id, other_rating) in other_ratings {
        if other_user_id == user_id { continue; }
        let delta = similarity(rating, other_rating);
        update_similarity(conn, user_id, other_user_id, delta)?;
    }

    diesel::result::QueryResult::Ok(())
}

pub fn on_rating_remove(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, old_rating: i32) -> diesel::result::QueryResult<()> {
    use crate::schema::user_ratings;

    let other_ratings: Vec<(i32, i32)> = user_ratings::table
        .filter(user_ratings::oeuvre_id.eq(oeuvre_id))
        .select((user_ratings::user_id, user_ratings::rating))
        .load(conn)?;

    for (other_user_id, other_rating) in other_ratings {
        if other_user_id == user_id { continue; }
        let delta = -similarity(old_rating, other_rating);
        update_similarity(conn, user_id, other_user_id, delta)?;
    }

    diesel::result::QueryResult::Ok(())
}

// TODO: also create fake users representing TAGS of popular oeuvres that like every oeuvre that has their tag
// these user should have the PHC field empty so no one can log into them
pub fn get_reco(conn: &mut SqliteConnection, user_id: i32, medium: Medium) -> diesel::result::QueryResult<Oeuvre> {
    use crate::schema::oeuvres::dsl::*;
    
    oeuvres.order(rating.desc()).first(conn)
}