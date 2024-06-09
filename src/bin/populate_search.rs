use regex::Regex;
use what_next_back::{add_token, establish_connection, DatabaseKind};

fn main() {
    let mut conn = establish_connection(DatabaseKind::PROD).unwrap();
    let tx = conn.transaction().unwrap();
    tx.execute("DELETE FROM search_tokens", []).unwrap();
    {
        let mut stmt = tx.prepare_cached("SELECT id, title FROM oeuvres").unwrap();
        let reg = Regex::new(r"\w+").unwrap();
        let mut i: u32 = 0;
        for res in stmt.query_map([], |row| Ok((row.get::<usize, i32>(0)?, row.get::<usize, String>(1)?))).unwrap() {
            let (id, text) = res.unwrap();
            for token in reg
                .find_iter(&text.to_lowercase())
                .map(|token| token.as_str().trim_end_matches("s"))
                .filter(|token| token.len() > 0) 
            {
                i += 1;
                add_token(&tx, id, token).unwrap();
                if i.trailing_zeros() == 18 {
                    println!("{i}");
                }
            }
        }    
    }
    let _ = tx.commit();
}