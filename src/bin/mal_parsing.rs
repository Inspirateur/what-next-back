use what_next_back::{add_tag, establish_connection, mal::{add_mal_oeuvre, get_mal_oeuvre_id}, update_rating, DatabaseKind, Medium, NewOeuvre, RatingOn100};

fn main() {
    seed_animes();
    seed_ratings()
}

// Download "anime.csv" from https://www.kaggle.com/datasets/svanoo/myanimelist-dataset before running this (and set the file path accordingly)
fn seed_animes() {
    let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b'\t').from_path("data/anime.csv") else {
        println!("couldn't find or open anime.csv");
        return;
    };

    println!("Reading MAL oeuvres");
    let mut conn = establish_connection(DatabaseKind::PROD).unwrap();
    let tx = conn.transaction().unwrap();
    let mut i: u32 = 0;
    let mut skipped = 0;
    for record_res in rdr.records() {
        i += 1;
        if i.trailing_zeros() == 10 {
            println!("{i} - (skipped {skipped})");
        }
        let record = record_res.unwrap();
        if &record[5] != "TV" {
            skipped += 1;
            continue;
        }
        let mal_id = record[0].parse::<i32>().unwrap();
        let title = &record[2];
        let synopsis = &record[3];
        let picture = &record[4];
        let new_oeuvre = NewOeuvre {
            title: title,
            medium: Medium::Anime,
            rating: RatingOn100(0),
            synopsis,
            picture
        };
        let oeuvre_id = add_mal_oeuvre(&tx, new_oeuvre, mal_id).unwrap();
        for genre in record[13].split("|").into_iter() {
            let _ = add_tag(&tx, oeuvre_id, format!("genre:{genre}"));
        }
    }
    let _ = tx.commit();
}


// Download "anime.csv" from https://github.com/Hernan4444/MyAnimeList-Database/tree/master/data before running this (and set the file path accordingly)
fn seed_ratings() {
    let Ok(mut rdr) = csv::ReaderBuilder::new().from_path("data/anime_ratings.csv") else {
        println!("couldn't find or open anime_ratings.csv");
        return;
    };

    println!("Reading MAL ratings");
    let mut conn = establish_connection(DatabaseKind::PROD).unwrap();
    let tx = conn.transaction().unwrap();
    let mut i: u32 = 0;
    let mut skipped = 0;
    for record_res in rdr.records() {
        i += 1;
        if i.trailing_zeros() == 10 {
            println!("{i} - (skipped {skipped})");
        }
        let record = record_res.unwrap();
        let mal_id = record[0].parse::<i32>().unwrap();
        let Ok(oeuvre_id) = get_mal_oeuvre_id(&tx, mal_id) else {
            skipped += 1;
            continue;
        };
        let Ok(rating_on_10) = record[2].parse::<f32>() else {
            skipped += 1;
            continue;
        };
        let rating_on_100 = RatingOn100((rating_on_10*10.) as i32);
        let _ = update_rating(&tx, oeuvre_id, rating_on_100);
    }
    let _ = tx.commit();
}
