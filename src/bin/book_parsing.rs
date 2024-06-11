use what_next_back::{add_oeuvre, add_tag, establish_connection, isbn::add_isbn_oeuvre, DatabaseKind, Medium, NewOeuvre, RatingOn100};

fn main() {
    seed_books();
    // println!("{}", isbn13to10("9999999999999").unwrap());
}

fn isbn13to10(isbn13: &str) -> Result<String, ()> {
    let isbn9 = &isbn13.get(3..12).ok_or(())?;
    let mut checksum = 0;
    let mut mult = 10;
    for c in isbn9.chars() {
        let d = c.to_digit(10).ok_or(())?;
        checksum += mult*d;
        mult -= 1;
    }
    checksum = 11 - (checksum % 11);
    let mut res = match checksum {
        10 => "X".to_string(),
        _ => checksum.to_string()
    };
    res.insert_str(0, isbn9);
    Ok(res)
}

// Download "books_1.Best_Books_Ever.csv" from https://zenodo.org/records/4265096 before running this (and set the file path accordingly)
fn seed_books() {
    // empirical number that biases ratings in favor of really popular work
    const VIRTUAL_RATINGS: f32 = 50_000.;
    const MISSING_ISBN10: &str = "9999999999";

    let Ok(mut rdr) = csv::ReaderBuilder::new().from_path("data/books_1.Best_Books_Ever.csv") else {
        println!("couldn't find or open books_1.Best_Books_Ever.csv");
        return;
    };

    println!("Reading Books");
    let mut conn = establish_connection(DatabaseKind::PROD).unwrap();
    let tx = conn.transaction().unwrap();
    let mut i: u32 = 0;
    let mut skipped = 0;
    for record_res in rdr.records() {
        i += 1;
        if i.trailing_zeros() == 12 {
            println!("{i} - (skipped {skipped})");
        }
        let record = record_res.unwrap();
        let isbn13 = &record[7];
        let Ok(isbn10) = isbn13to10(isbn13) else {
            skipped += 1;
            continue;
        };
        let title = &record[1];
        let author = &record[3].split(", ").next().unwrap().replace(" (Goodreads Author)", "");
        let rating_on_5 = record[4].parse::<f32>().unwrap_or(0.);
        let num_ratings = record[17].parse::<i32>().unwrap_or(0) as f32;
        // we take into account the amount of votes n by re-averaging with 10 virtual ratings of 50%
        // (same principle as this https://www.youtube.com/watch?v=8idr1WZ1A7Q)
        let rating_on_5 = (rating_on_5*num_ratings + VIRTUAL_RATINGS*2.5)/(num_ratings+VIRTUAL_RATINGS);
        let rating_on_100 = RatingOn100((rating_on_5*20.).round() as i32);
        let synopsis = &record[5];
        let picture = &record[21];
        let new_oeuvre = NewOeuvre {
            title: title,
            medium: Medium::Book,
            rating: rating_on_100,
            synopsis,
            picture
        };
        let oeuvre_id = if isbn10 == MISSING_ISBN10 {
            add_oeuvre(&tx, new_oeuvre).unwrap()
        } else {
            add_isbn_oeuvre(&tx, new_oeuvre, &isbn10).unwrap()
        };
        add_tag(&tx, oeuvre_id, format!("author:{author}")).unwrap();
        for genre in record[8][1..(record[8].len()-1)].split(", ").into_iter().filter_map(|genre| genre.get(1..(genre.len()-1))) {
            add_tag(&tx, oeuvre_id, format!("genre:{genre}")).unwrap();
        }
    }
    let _ = tx.commit();
}