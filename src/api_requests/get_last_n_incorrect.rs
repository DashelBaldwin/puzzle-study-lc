// get_last_n_incorrect.rs 

use std::error::Error;

use super::json_objects::Puzzle;
use super::json_objects::parse_puzzle;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

const PAGE_SIZE: i32 = 50;

async fn get_puzzle_history_incorrect_page(client: &reqwest::Client, pat: String, max: i32, before_date: i64) -> Result<(Vec<Puzzle>, i64), Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    println!("Attempting to get {} puzzles as '{}'", max, pat);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", pat))?);

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
                Ok(mut puzzle_attempt) => {
                    if !puzzle_attempt.win {
                        puzzle_attempt.puzzle.imported_directly = None;
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

pub async fn get_last_n_incorrect(pat: String, n: usize) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut incorrect_puzzles: Vec<Puzzle> = Vec::new();
    let mut size: usize = 0;
    let mut before_date = -1;
    let mut page_number = 1;

    while size < n {
        println!("Getting page {}", page_number);
        let page_data = get_puzzle_history_incorrect_page(&client, pat.clone(), PAGE_SIZE, before_date).await?;
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
