use std::str::FromStr;
use rocket::request::FromParam;
use serde::{Serialize, Deserialize};
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString)]
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

impl<'a> FromParam<'a> for Medium {
    type Error = <Self as ::core::str::FromStr>::Err;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Medium::from_str(param)
    }
}

impl From<i32> for Medium {
    fn from(value: i32) -> Self {
        unsafe {::std::mem::transmute(value)}
    }
}

#[derive(Serialize)]
pub struct Oeuvre {
    pub id: i32,
    pub medium: Medium,
    pub title: String,
    pub rating: RatingOn100,
    pub synopsis: String,
    pub picture: String,
    pub tags: Vec<String>,
    pub user_rating: Option<AppRating>
}

pub struct NewOeuvre<'a> {
    pub medium: Medium,
    pub title: &'a str,
    pub rating: RatingOn100,
    pub synopsis: &'a str,
    pub picture: &'a str
}

// Raw app rating, values can be: -2, -1, 1, 2
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct AppRating(pub i32);

impl AppRating {
    pub fn similarity(rating1: AppRating, rating2: AppRating) -> i32 {
        match (rating1.0-rating2.0).abs() {
            0 => 1,
            1 => 0,
            2 => -1,
            3 | 4 => -2,
            _ => unreachable!()
        }
    }

    pub fn to_rating_on_100(rating: f32) -> f32 {
        (rating + 2.) * 25.
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct RatingOn100(pub i32);
