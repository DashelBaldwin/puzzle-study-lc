// get_last_n_incorrect.rs 

use std::error::Error;

use super::json_objects::Puzzle;
use super::json_objects::parse_puzzle;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

const PAGE_SIZE: i32 = 50;

async fn get_puzzle_history_incorrect_page(client: &reqwest::Client, pat: String, max: i32, before_date: i64, ignore: &Vec<String>) -> Result<(Vec<Puzzle>, i64, usize), Box<dyn Error>> {
    let mut headers = HeaderMap::new();
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
        let mut duplicates: usize = 0;

        for puzzle_attempt_string in puzzle_attempt_strings {
            match parse_puzzle(puzzle_attempt_string) {
                Ok(mut puzzle_attempt) => {
                    if !puzzle_attempt.win {
                        if !ignore.contains(&puzzle_attempt.puzzle.id) {
                            puzzle_attempt.puzzle.imported_directly = None;
                        incorrect_puzzles.push(puzzle_attempt.puzzle);
                        last_date = puzzle_attempt.date;
                        } else {
                            duplicates += 1;
                        }
                    }
                }
                Err(e) => {
                    return Err(Box::from(e));
                }
            }
        }
        Ok((incorrect_puzzles, last_date, duplicates))

    } else {
        Err(Box::from(format!("Couldn't access the puzzle history of the user associated with '{}'; was this token entered correctly?", pat)))
    }
}

pub async fn get_last_n_incorrect(pat: String, n: usize, ignore: Vec<String>) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut incorrect_puzzles: Vec<Puzzle> = Vec::new();
    let mut size: usize = 0;
    let mut before_date: i64 = -1;
    let mut total_duplicates: usize = 0;

    while size < n {
        let page_data = get_puzzle_history_incorrect_page(&client, pat.clone(), PAGE_SIZE, before_date, &ignore).await?;
        let page = page_data.0;
        before_date = page_data.1;

        total_duplicates += page_data.2;

        if page.is_empty() {break;}

        for puzzle in page {
            incorrect_puzzles.push(puzzle);
            size += 1;
        }
    }

    let plural_char = if total_duplicates == 1 { "" } else { "s" };
    if total_duplicates > 0 { println!("\nSkipping {} duplicate ID{}", total_duplicates, plural_char); }

    Ok(incorrect_puzzles[0..n].to_vec())
}
