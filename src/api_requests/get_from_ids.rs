// get_puzzles_from_ids.rs

use std::error::Error;

use super::json_objects::Puzzle;
use super::json_objects::parse_direct_puzzle;

use crate::notation_utils;

async fn get_puzzle_from_id(client: &reqwest::Client, id: String) -> Result<Puzzle, Box<dyn Error>> {
    let response = client
        .get(format!("https://lichess.org/api/puzzle/{}", id))
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        match parse_direct_puzzle(&body) {
            Ok(direct_puzzle) => {
                let puzzle = Puzzle {
                    id: id,
                    rating: direct_puzzle.puzzle.rating,
                    solution: direct_puzzle.puzzle.solution,
                    themes: direct_puzzle.puzzle.themes,
                    fen: notation_utils::pgn_to_fen::pgn_to_fen(&direct_puzzle.game.pgn),
                    imported_directly: Some(true)
                };
                Ok(puzzle)
            }
            Err(e) => {
                Err(Box::from(e))
            }
        }

    } else {
        Err(Box::from(format!("Couldn't find https://lichess.org/training/{}; was this ID entered correctly?\n", id)))
    }

}


pub async fn get_from_ids(ids: Vec<String>, ignore: Vec<String>) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut puzzles: Vec<Puzzle> = Vec::new();
    let mut total_duplicates: usize = 0;

    for id in ids {
        if !ignore.contains(&id) && !puzzles.iter().any(|puzzle| puzzle.id == id) {
            puzzles.push(get_puzzle_from_id(&client, id).await?);
        } else {
            total_duplicates += 1;
        }
    }

    let plural_char = if total_duplicates == 1 { "" } else { "s" };
    if total_duplicates > 0 { println!("Skipping {} duplicate ID{}", total_duplicates, plural_char); }

    Ok(puzzles)
}
