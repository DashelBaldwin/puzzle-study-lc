// post_overwrite.rs

use std::error::Error;

use super::json_objects::Puzzle;

use serde::Serialize;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;

const PAT: &str = "lip_4706DGPceC0b3H9YRO5x";

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

pub async fn post_overwrite(study_id: &str, mut puzzles: Vec<Puzzle>) -> Result<(), Box<dyn Error>> {
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
