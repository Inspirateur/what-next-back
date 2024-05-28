use what_next_back::{create_oeuvre, establish_connection, models::{Medium, NewOeuvre}};

fn main() {
    // Just some diesel test code
    let connection = &mut establish_connection();

    let new_oeuvre = NewOeuvre {
        title: "Test Movie".to_string(),
        medium: Medium::Book,
        synopsis: Some("The beginning of a great adventure".to_string()),
        picture: None   
    };

    create_oeuvre(connection, new_oeuvre);
}