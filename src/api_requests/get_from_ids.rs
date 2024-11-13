use std::error::Error;

use super::json_objects::Puzzle;
use super::json_objects::parse_direct_puzzle;

use crate::notation_utils;

async fn get_puzzle_from_id(client: &reqwest::Client, id: String) -> Result<Puzzle, Box<dyn Error>> {
    let response = client
        .get(format!("https://lichess.org/api/puzzle/{}", id))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!(
            "Couldn't find https://lichess.org/training/{}; was this ID entered correctly?\n",
            id
        ).into());
    }

    let body = response.text().await?;
    let parsed_puzzle = parse_direct_puzzle(&body)?;

    Ok(Puzzle {
        id,
        rating: parsed_puzzle.puzzle.rating,
        solution: parsed_puzzle.puzzle.solution,
        themes: parsed_puzzle.puzzle.themes,
        fen: notation_utils::pgn_to_fen::pgn_to_fen(&parsed_puzzle.game.pgn),
        imported_directly: Some(true),
    })
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
