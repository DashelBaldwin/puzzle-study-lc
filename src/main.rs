// main.rs

// TODO: restructure project

// TODO: make directly imported puzzles correctly say they were imported from id, instead of from puzzle history
// TODO: start working on the cli application

// Possible TODO: also allow pasting chess.com puzzle exported pgns into cli as input for convenience
// Possible TODO: make auto generation skip puzzles manually imported into the same set by user to prevent duplicates

use serde::Serialize;
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;

mod notation_utils;
mod api_requests;

use api_requests::json_objects::Puzzle;
use api_requests::json_objects::DirectPuzzle;
use api_requests::json_objects::DirectPuzzleGameData;
use api_requests::json_objects::PuzzleAttempt;
use api_requests::json_objects::parse_direct_puzzle;
use api_requests::json_objects::parse_puzzle;

const PAT: &str = "lip_4706DGPceC0b3H9YRO5x";
const PAGE_SIZE: i32 = 50;

async fn get_puzzle_from_id(id: &str) -> Result<Puzzle, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("https://lichess.org/api/puzzle/{}", id))
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        match parse_direct_puzzle(&body) {
            Ok(direct_puzzle) => {
                let puzzle = Puzzle {
                    id: id.to_string(),
                    rating: direct_puzzle.puzzle.rating,
                    solution: direct_puzzle.puzzle.solution,
                    themes: direct_puzzle.puzzle.themes,
                    fen: notation_utils::pgn_to_fen::pgn_to_fen(&direct_puzzle.game.pgn)
                };
                println!("{}, {}", puzzle.fen, puzzle.id);
                Ok(puzzle)
            }
            Err(e) => {
                eprintln!("Failed to parse puzzle: {}", e);
                Err(Box::from(e))
            }
        }

    } else {
        Err(Box::from(format!("API request error: {}", response.status())))
    }

}


async fn get_puzzles_from_ids(ids: Vec<&str>) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let mut puzzles: Vec<Puzzle> = Vec::new();

    for id in ids {
        puzzles.push(get_puzzle_from_id(id).await?);
    }

    Ok(puzzles)
}


async fn get_puzzle_history_incorrect_page(max: i32, before_date: i64) -> Result<(Vec<Puzzle>, i64), Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let mut query = vec![("max", i64::from(max))];

    if before_date > -1 {
        query.push(("before", before_date));
    }

    let response = client
        .get("https://lichess.org/api/puzzle/activity")
        .headers(headers)
        .query(&query)
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let puzzle_attempt_strings: Vec<&str> = body.lines().collect();
        let mut incorrect_puzzles: Vec<Puzzle> = Vec::new();
        let mut last_date: i64 = 0;

        for puzzle_attempt_string in puzzle_attempt_strings {
            match parse_puzzle(puzzle_attempt_string) {
                Ok(puzzle_attempt) => {
                    if !puzzle_attempt.win {
                        incorrect_puzzles.push(puzzle_attempt.puzzle);
                        last_date = puzzle_attempt.date;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse puzzle: {}", e);
                }
            }
        }
        Ok((incorrect_puzzles, last_date))

    } else {
        Err(Box::from(format!("API request error: {}", response.status())))
    }
}


async fn get_last_n_incorrect_puzzles(n: usize) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let mut incorrect_puzzles: Vec<Puzzle> = Vec::new();
    let mut size: usize = 0;
    let mut before_date = -1;
    let mut page_number = 1;

    while size < n {
        println!("Getting page {}", page_number);
        let page_data = get_puzzle_history_incorrect_page(PAGE_SIZE, before_date).await?;
        let page = page_data.0;
        before_date = page_data.1;

        if page.is_empty() {break;}

        for puzzle in page {
            incorrect_puzzles.push(puzzle);
            size += 1;
        }

        page_number += 1;
    }

    Ok(incorrect_puzzles[0..n].to_vec())
}


#[derive(Serialize)]
struct ImportPgnRequest {
    name: String,
    pgn: String,
    orientation: String,
    variant: String,
    mode: String
}


fn concatenate_pgn(puzzles: Vec<Puzzle>, offset_index: bool) -> String {
    let index_offset = if offset_index {2} else {1};
    let pgn_strings: Vec<String> = puzzles
        .iter()
        .enumerate()
        .map(|(index, puzzle)| puzzle.build_pgn(index + index_offset)) 
        .collect();
    
    pgn_strings.join("\n\n")
}


async fn post_puzzles_to_study(study_id: &str, puzzles: Vec<Puzzle>, offset_index: bool) -> Result<(), Box<dyn Error>> {
    let client: Client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let pgn_strings = concatenate_pgn(puzzles, offset_index);

    let form_puzzle_name = if !offset_index {"Puzzle 1".to_string()} else {"Puzzle 2".to_string()};

    let form = ImportPgnRequest {
        name: form_puzzle_name,
        pgn: pgn_strings,
        orientation: "default".to_string(),
        variant: "fromPosition".to_string(),
        mode: "gamebook".to_string()
    };

    let response = client
        .post(format!("https://lichess.org/api/study/{}/import-pgn", study_id))
        .headers(headers)
        .form(&form)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Successfully imported PGN!");
    } else {
        println!("Failed to import PGN: {:?}", response.text().await?);
    }

    Ok(())
}


async fn get_study_chapter_ids(study_id: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let response = client
        .get(format!("https://lichess.org/api/study/{}.pgn", study_id))
        .headers(headers)
        .query(&[("clocks", false), ("comments", false), ("variations", false)])
        .send()
        .await?;

    let mut ids : Vec<String> = Vec::new();

    if response.status().is_success() {
        let body: String = response.text().await?;
        
        for line in body.lines() {
            if line.starts_with("[Site ") {          
                if let Some(id) = line.split('/').last() {
                    if let Some(trimmed) = id.strip_suffix("\"]") {
                        ids.push(trimmed.to_string());
                    }
                }
            }
        }
    } else {
        println!("Error: failed to get study chapters")
    }

    Ok(ids)
}


async fn clear_chapter(study_id: &str, id: String) -> Result<(), Box<dyn Error>> {
    let client: Client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let response = client
        .delete(format!("https://lichess.org/api/study/{}/{}", study_id, id))
        .headers(headers)
        .send()
        .await?;
    
        if !response.status().is_success() {
            eprintln!("Failed to delete chapter ID={} with status: {}", id, response.status());
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to delete chapter ID={} with status: {}", id, response.status()))));
        }

    Ok(())
}


async fn clear_study(study_id: &str, mut ids: Vec<String>, include_first_chapter: bool) -> Result<(), Box<dyn Error>> {
    let chapters_left = if include_first_chapter {0} else {1};

    while ids.len() > chapters_left {
        let id = ids.pop().expect("Expected String in ids");
        clear_chapter(study_id, id).await?;
    }

    Ok(())
}


async fn clear_and_upload(study_id: &str, mut puzzles: Vec<Puzzle>) -> Result<(), Box<dyn Error>> {
    println!("Getting study chapter IDs...");
    let chapter_ids = get_study_chapter_ids(study_id).await?;
    let minimum_chapter_id = (&chapter_ids[0]).to_string();
    let first_puzzle = puzzles.remove(0);

    println!("Attempting to clear study...");
    clear_study(study_id, chapter_ids, false).await?;
    println!("Swapping first chapter...");
    post_puzzles_to_study(study_id, vec![first_puzzle], false).await?;
    clear_chapter(study_id, minimum_chapter_id).await?;
    println!("Uploading all chapters...");
    post_puzzles_to_study(study_id, puzzles, true).await?;

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let puzzles = get_last_n_incorrect_puzzles(64).await?;
    clear_and_upload("l2Yn1iSK", puzzles).await?;

    Ok(())
}
