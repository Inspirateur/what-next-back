use diesel::prelude::*;
use what_next_back::{establish_connection, models::Oeuvre};


fn main() {
    use what_next_back::schema::oeuvres::dsl::*;

    let connection: &mut diesel::prelude::SqliteConnection = &mut establish_connection();
    let results: Vec<Oeuvre> = oeuvres
        .order(rating.desc())
        .limit(10)
        .select(Oeuvre::as_select())
        .load(connection)
        .expect("Error loading oeuvres");

    println!("Displaying {} best rated oeuvres", results.len());
    for oeuvre in results {
        println!("[{:?}] {} {}%", oeuvre.medium, oeuvre.title, oeuvre.rating.unwrap_or(50));
    }
}
