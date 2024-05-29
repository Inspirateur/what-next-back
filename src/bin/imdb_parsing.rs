use diesel::Connection;
use what_next_back::{add_imdb_oeuvre_no_check, establish_connection, models::{Medium, NewOeuvre}};

// Download "title.basics.tsv" from https://datasets.imdbws.com/ before running this
// NOTE: The file is BIG, run it in release mode
fn main() {
    let connection = &mut establish_connection();

    // The path to "title.basics.tsv"
    let Ok(mut rdr) = csv::ReaderBuilder::new().delimiter(b'\t').from_path("data/title.basics.tsv") else {
        println!("couldn't find or open title.basics.tsv");
        return;
    };

    // do it all in a single transaction for efficiency
    let _ = connection.transaction(|connection| {
        let mut i: u32 = 0;
        let mut skipped = 0;
        for record_res in rdr.records() {
            if i.trailing_zeros() == 15 {
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
            let genres = &record[8];
            let medium = if medium == "tvSeries" {
                Medium::Series
            } else if medium == "movie" {
                if genres.split(",").any(|genre| genre == "Animation") {
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
                synopsis: None,
                picture: None
            };
            if add_imdb_oeuvre_no_check(connection, new_oeuvre, imdb_id).is_err() {
                skipped += 1;
            }
            i += 1;
        }
        diesel::result::QueryResult::Ok(())
    });

}