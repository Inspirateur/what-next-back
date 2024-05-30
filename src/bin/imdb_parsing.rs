use std::collections::HashMap;
use diesel::Connection;
use what_next_back::{add_imdb_oeuvre_no_check, add_tag, establish_connection, get_imdb_oeuvre_id, models::{Medium, NewOeuvre}, update_rating};

fn main() {
    seed_imdb_oeuvres();
    seed_imdb_crews();
    seed_imdb_ratings();
}

// Download "title.basics.tsv" from https://datasets.imdbws.com/ before running this (and set the file path accordingly)
// NOTE: The file is BIG, run it in release mode
fn seed_imdb_oeuvres() {
    let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b'\t').from_path("data/title.basics.tsv") else {
        println!("couldn't find or open title.basics.tsv");
        return;
    };

    println!("Reading IMDb oeuvres");
    let conn = &mut establish_connection();
    // do it all in a single transaction for efficiency
    let _ = conn.transaction(|conn| {
        let mut i: u32 = 0;
        let mut skipped = 0;
        for record_res in rdr.records() {
            i += 1;
            if i.trailing_zeros() == 18 {
                println!("{i} - (skipped {skipped})");
                skipped = 0;
            }
            let Ok(record) = record_res else {
                skipped += 1;
                continue;
            };
            let imdb_id = &record[0];
            let medium = &record[1];
            let title = &record[2];
            let genres: Vec<_> = record[8].split(",").collect();
            let medium = if medium == "tvSeries" {
                Medium::Series
            } else if medium == "movie" {
                if genres.iter().any(|genre| *genre == "Animation") {
                    Medium::AnimationMovie
                } else {
                    Medium::Movie
                }
            } else {
                skipped += 1;
                continue;
            };
    
            let new_oeuvre = NewOeuvre {
                title: title,
                medium: medium,
                rating: None,
                synopsis: None,
                picture: None
            };
            if let Ok(oeuvre_id) = add_imdb_oeuvre_no_check(conn, new_oeuvre, imdb_id) {
                for genre in genres.into_iter().filter(|g| *g != "Animation") {
                    if genre == "\\N" { continue; }
                    let _ = add_tag(conn, oeuvre_id, format!("genre:{genre}"));
                }
            } else {
                skipped += 1;
            }
        }
        diesel::result::QueryResult::Ok(())
    });
}

// Download "name.basics.tsv" & "title.crew.tsv" from https://datasets.imdbws.com/ before running this (and set the file pathes accordingly)
// NOTE: it takes some RAM
fn seed_imdb_crews() {
    let Some(name_map) = get_imdb_name_map() else {
        return;
    };

    let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b'\t').from_path("data/title.crew.tsv") else {
        println!("couldn't find or open title.crew.tsv");
        return;
    };

    println!("Reading IMDb crews");
    let conn = &mut establish_connection();
    // do it all in a single transaction for efficiency
    let _ = conn.transaction(|conn| {
        let mut i: u32 = 0;
        let mut skipped = 0;
        for record_res in rdr.records() {
            i += 1;
            if i.trailing_zeros() == 18 {
                println!("{i} - (skipped {skipped})");
                skipped = 0;
            }
            let Ok(record) = record_res else {
                skipped += 1;
                continue;
            };
            let imdb_id = &record[0];
            let Ok(oeuvre_id) = get_imdb_oeuvre_id(conn, imdb_id) else {
                skipped += 1;
                continue;
            };

            if let Some(director_id) = record[1].split(",").next() {
                if let Some(director) = name_map.get(director_id) {
                    let _ = add_tag(conn, oeuvre_id, format!("director:{director}"));
                }
            };
            if let Some(writer_id) = record[2].split(",").next() {
                if let Some(writer) = name_map.get(writer_id) {
                    let _ = add_tag(conn, oeuvre_id, format!("writer:{writer}"));
                };
            };
        }
        diesel::result::QueryResult::Ok(())
    });
}

fn get_imdb_name_map() -> Option<HashMap<String, String>> {
    let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b'\t').from_path("data/name.basics.tsv") else {
        println!("couldn't find or open name.basics.tsv");
        return None;
    };
    println!("Reading IMDb name map");
    let mut i: u32 = 0;
    let mut skipped = 0;
    let mut name_map = HashMap::new();
    for record_res in rdr.records() {
        i += 1;
        if i.trailing_zeros() == 18 {
            println!("{i} - (skipped {skipped})");
            skipped = 0;
        }
        let Ok(record) = record_res else {
            skipped += 1;
            continue;
        };
        if !record[4].contains("writer") && !record[4].contains("director") {
            skipped += 1;
            continue;
        }
        name_map.insert(record[0].to_string(), record[1].to_string());
    }
    Some(name_map)
}

// Download "title.ratings.tsv" from https://datasets.imdbws.com/ before running this (and set the file pathes accordingly)
fn seed_imdb_ratings() {
    // empirical number that biases rating towards really popular oeuvre
    const VIRTUAL_RATINGS: f32 = 1000.;

    let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b'\t').from_path("data/title.ratings.tsv") else {
        println!("couldn't find or open title.ratings.tsv");
        return;
    };

    println!("Reading IMDb ratings");
    let conn = &mut establish_connection();
    let _ = conn.transaction(|conn| {
        let mut i: u32 = 0;
        let mut skipped = 0;
        for record_res in rdr.records() {
            i += 1;
            if i.trailing_zeros() == 18 {
                println!("{i} - (skipped {skipped})");
                skipped = 0;
            }
            let Ok(record) = record_res else {
                skipped += 1;
                continue;
            };
            let imdb_id = &record[0];
            let Ok(avg_on_10) = record[1].parse::<f32>() else {
                skipped += 1;
                continue;
            };
            let Ok(n) = record[2].parse::<i32>().map(|n| n as f32) else {
                skipped += 1;
                continue;
            };
            let Ok(oeuvre_id) = get_imdb_oeuvre_id(conn, imdb_id) else {
                skipped += 1;
                continue;
            };
            // we take into account the amount of votes n by re-averaging with 10 virtual ratings of 50%
            // (same principle as this https://www.youtube.com/watch?v=8idr1WZ1A7Q)
            let rating_on_10 = (avg_on_10*n + VIRTUAL_RATINGS*5.)/(n+VIRTUAL_RATINGS);
            let rating_on_100 = (rating_on_10*10.).round() as i32;
            update_rating(conn, oeuvre_id, rating_on_100)?;
        }
        diesel::result::QueryResult::Ok(())
    });
}