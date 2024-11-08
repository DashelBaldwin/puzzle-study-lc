// post_overwrite.rs

use std::io::{self, Write};
use std::error::Error;

use super::json_objects::Puzzle;

use serde::Serialize;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use crate::utils::progress_bar::{inner_progress_bar, PROGRESS_BAR_WIDTH};

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

async fn post_puzzles_to_study(client: &reqwest::Client, pat: String, study_id: &str, puzzles: Vec<Puzzle>, offset_index: bool) -> Result<(), Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", pat))?);

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

    if !response.status().is_success() {
        eprintln!("Failed to import PGN: {:?}", response.text().await?);
    } 

    Ok(())
}

async fn get_study_chapter_ids(client: &reqwest::Client, pat: String, study_id: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", pat))?);

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
        return Err(Box::from(format!("Couldn't access study '{}' on behalf of the user associated with '{}'; were these tokens entered correctly?", study_id, pat)))
    }

    Ok(ids)
}

async fn clear_chapter(client: &reqwest::Client, pat: String, study_id: &str, id: String) -> Result<(), Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", pat))?);

    let response = client
        .delete(format!("https://lichess.org/api/study/{}/{}", study_id, id))
        .headers(headers)
        .send()
        .await?;
    
        if !response.status().is_success() {
            return Err(Box::from(format!("Couldn't modify study '{}' on behalf of the user associated with '{}'; were these tokens entered correctly?", study_id, pat)))
        }

    Ok(())
}

async fn clear_study(client: &reqwest::Client, pat: String, study_id: &str, mut ids: Vec<String>) -> Result<(), Box<dyn Error>> {
    let initial_size = ids.len();

    print!("Clearing study [{}] ", inner_progress_bar(0.0, PROGRESS_BAR_WIDTH)); 
    io::stdout().flush().unwrap();

    while ids.len() > 1 {
        let progress = 1.0 - ids.len() as f32 / initial_size as f32;
        let id = ids.pop().unwrap();
        clear_chapter(client, pat.clone(), study_id, id).await?;
        print!("\x1b[0GClearing study [{}] ", inner_progress_bar(progress, PROGRESS_BAR_WIDTH)); 
        io::stdout().flush().unwrap();
    }
    println!("\x1b[0GClearing study [{}] done! ", inner_progress_bar(1.0, PROGRESS_BAR_WIDTH)); 
    io::stdout().flush().unwrap();

    Ok(())
}

pub async fn post_overwrite(pat: String, study_id: &str, mut puzzles: Vec<Puzzle>) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    println!("Getting study chapter IDs");
    let chapter_ids = get_study_chapter_ids(&client, pat.clone(), study_id).await?;
    let minimum_chapter_id = (&chapter_ids[0]).to_string();
    let first_puzzle = puzzles.remove(0);

    clear_study(&client, pat.clone(), study_id, chapter_ids).await?;
    post_puzzles_to_study(&client, pat.clone(), study_id, vec![first_puzzle], false).await?;
    clear_chapter(&client, pat.clone(), study_id, minimum_chapter_id).await?;
    println!("Uploading staged puzzles");
    post_puzzles_to_study(&client, pat.clone(), study_id, puzzles, true).await?;

    Ok(())
}
