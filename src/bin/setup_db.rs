use what_next_back::{setup_db, DatabaseKind};

fn main() {
    if let Err(e) = setup_db(DatabaseKind::PROD) {
        eprintln!("{}", e);
    }
}