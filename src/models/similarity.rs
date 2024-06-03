use std::collections::HashMap;
use rusqlite::{Connection, Result};
use crate::{AppRating, RatingOn100};
use super::Medium;

fn order_user_id(user1_id: i32, user2_id: i32) -> (i32, i32) {
    if user1_id < user2_id {
        (user1_id, user2_id)
    } else {
        (user2_id, user1_id)
    }
}

fn update_similarity(conn: &Connection, user1_id: i32, user2_id: i32, delta: i32) -> Result<()> {
    let (user1_id, user2_id) = order_user_id(user1_id, user2_id);

    conn.execute(
        "INSERT INTO users_similarity(user1_id, user2_id, score) VALUES(?1, ?2, ?3) 
        ON CONFLICT(user1_id, user2_id) DO UPDATE SET score = score + ?3", 
        [user1_id, user2_id, delta]
    )?;
    Ok(())
}

pub fn on_rating_update(conn: &Connection, user_id: i32, oeuvre_id: i32, old_rating: AppRating, new_rating: AppRating) -> Result<()> {
    let other_ratings: Vec<(i32, AppRating)> = conn
        .prepare("SELECT user_id, rating FROM user_ratings WHERE oeuvre_id = ?1 AND user_id != ?2")?
        .query_map([oeuvre_id, user_id], |row| Ok((row.get::<usize, i32>(0)?, AppRating(row.get::<usize, i32>(1)?))))?
        .collect::<Result<Vec<_>>>()?;
    
    for (other_user_id, other_rating) in other_ratings {
        if other_user_id == user_id { continue; }
        let delta = AppRating::similarity(new_rating, other_rating) 
            - AppRating::similarity(old_rating, other_rating);
        update_similarity(conn, user_id, other_user_id, delta)?;
    }
    Ok(())
}

pub fn on_rating_add(conn: &Connection, user_id: i32, oeuvre_id: i32, rating: AppRating) -> Result<()> {
    let other_ratings: Vec<(i32, AppRating)> = conn
        .prepare("SELECT user_id, rating FROM user_ratings WHERE oeuvre_id = ?1 AND user_id != ?2")?
        .query_map([oeuvre_id, user_id], |row| Ok((row.get::<usize, i32>(0)?, AppRating(row.get::<usize, i32>(1)?))))?
        .collect::<Result<Vec<_>>>()?;

    for (other_user_id, other_rating) in other_ratings {
        if other_user_id == user_id { continue; }
        let delta = AppRating::similarity(rating, other_rating);
        update_similarity(conn, user_id, other_user_id, delta)?;
    }
    Ok(())
}

pub fn on_rating_remove(conn: &Connection, user_id: i32, oeuvre_id: i32, old_rating: AppRating) -> Result<()> {
    let other_ratings: Vec<(i32, AppRating)> = conn
        .prepare("SELECT user_id, rating FROM user_ratings WHERE oeuvre_id = ?1 AND user_id != ?2")?
        .query_map(
            [oeuvre_id, user_id], 
            |row| Ok((row.get::<usize, i32>(0)?, AppRating(row.get::<usize, i32>(1)?))))?
        .collect::<Result<Vec<_>>>()?;

    for (other_user_id, other_rating) in other_ratings {
        if other_user_id == user_id { continue; }
        let delta = -AppRating::similarity(old_rating, other_rating);
        update_similarity(conn, user_id, other_user_id, delta)?;
    }
    Ok(())
}

pub struct Reco {
    pub oeuvre_id: i32,
    pub score: RatingOn100,
}

struct SimilarUser {
    user_id: i32,
    similarity: i32,
}

struct RatedOeuvre {
    oeuvre_id: i32,
    rating: AppRating,
    overall_rating: RatingOn100,
}

struct PopularOeuvre {
    oeuvre_id: i32,
    overall_rating: RatingOn100,
}

fn similar_users(conn: &Connection, user_id: i32, similarity_threshold: i32) 
    -> Result<Vec<SimilarUser>> 
{
    conn.prepare("SELECT user1_id, user2_id, score FROM users_similarity WHERE score > ?1 AND (user1_id = ?2 OR user2_id = ?2)")?
        .query_map(
            [similarity_threshold, user_id], 
            |row| {
                let user1_id = row.get::<usize, i32>(0)?;
                let user2_id = row.get::<usize, i32>(1)?;
                Ok(SimilarUser {
                    user_id: if user1_id == user_id { user2_id } else { user1_id },
                    similarity: row.get::<usize, i32>(2)?
                })
            })?.collect::<Result<Vec<_>>>()
}

fn unseen_popular_oeuvre(conn: &Connection, user_id: i32, medium: Medium) -> Result<Option<PopularOeuvre>> {
    // TODO: Return the most popular oeuvre in medium that user hasn't rated
    todo!()
}

fn recommendable_oeuvres(conn: &Connection, recommender_id: i32, recommendee_id: i32, medium: Medium) 
    -> Result<Vec<RatedOeuvre>> 
{
    // TODO: Return all the oeuvres in medium that recomender has given a positive rating to and that recomendee hasn't rated
    todo!()
}

// TODO: also create fake users representing TAGS of popular oeuvres that like every oeuvre that has their tag
// these users should have the PHC field empty so no one can log into them
pub fn get_reco(conn: &Connection, user_id: i32, medium: Medium) -> Result<Option<Reco>> {
    let similar_users = similar_users(conn, user_id, 1)?;
    let max_similiarity = similar_users.iter().map(|su| su.similarity).max().unwrap_or(0);
    // Compute a softmax of similarities with 1 phatom user at 0 similarity that gives every oeuvre their overall rating
    // This is necessary for a user that has rated few oeuvres
    let softmax_total: f32 = similar_users.iter()
        .map(|su| ((su.similarity-max_similiarity) as f32).exp())
        .sum::<f32>() 
        + (-max_similiarity as f32).exp();
    let phantom_weight = (-max_similiarity as f32).exp()/softmax_total;
    // Contains weighted user ratings (AppRating), the oeuvre with the max value will be recommended
    let mut oeuvres_scored: HashMap<i32, f32> = HashMap::new();
    // Caching the overall ratings (/100) to avoid re-querying them
    let mut oeuvres_overall_rating: HashMap<i32, f32> = HashMap::new();
    if let Some(popular_oeuvre) = unseen_popular_oeuvre(conn, user_id, medium)? {
        oeuvres_scored.insert(popular_oeuvre.oeuvre_id, 0.);
        oeuvres_overall_rating.insert(popular_oeuvre.oeuvre_id, popular_oeuvre.overall_rating.0 as f32);
    }
    for similar_user in similar_users.into_iter() {
        let user_weight = ((similar_user.similarity-max_similiarity) as f32).exp()/softmax_total;
        for new_oeuvre in recommendable_oeuvres(conn, similar_user.user_id, user_id, medium)? {
            *oeuvres_scored.entry(new_oeuvre.oeuvre_id).or_insert(0.) += new_oeuvre.rating.0 as f32*user_weight;
            oeuvres_overall_rating.insert(new_oeuvre.oeuvre_id, new_oeuvre.overall_rating.0 as f32);
        }
    }
    // Factor in the overall rating into the recommendation
    let Some((best_oeuvre_id, best_score)) = oeuvres_scored
        .into_iter()
        .map(|(oeuvre_id, score)| (oeuvre_id, (AppRating::to_rating_on_100(score) + *oeuvres_overall_rating.get(&oeuvre_id).unwrap_or(&50.)*phantom_weight) as i32))
        .max_by(|(_, score_a), (_, score_b)| score_a.cmp(score_b)) else 
    {
        return Ok(None);
    };
    Ok(Some(Reco { oeuvre_id: best_oeuvre_id, score: RatingOn100(best_score) }))
}