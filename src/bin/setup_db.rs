use what_next_back::setup_db;

fn main() {
    if let Err(e) = setup_db() {
        eprintln!("{}", e);
    }
}