// main.rs

// Possible TODO: make auto generation skip puzzles manually imported into the same set by user to prevent duplicates
// FIX: one-move puzzles don't generate info comments

use notation_tools::Board;
use notation_tools::PieceName;
use notation_tools::PieceColor;
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;

mod notation_tools;

const PAT: &str = "lip_4706DGPceC0b3H9YRO5x";
const PAGE_SIZE: i32 = 50;

#[derive(Deserialize)]
#[derive(Clone)]
struct Puzzle {
    id: String,
    rating: i32,
    solution: Vec<String>,
    themes: Vec<String>,
    fen: String,
}

#[derive(Deserialize)]
struct PuzzleAttempt {
    win: bool,
    puzzle: Puzzle, 
    date: i64
}

// #[derive(Deserialize)]
// struct DirectPuzzleData {
//     game: DirectPuzzleGameData,
//     puzzle: DirectPuzzle
// }

// #[derive(Deserialize)]
// struct DirectPuzzle {
//     id: String,
//     rating: i32,
//     solution: Vec<String>,
//     themes: Vec<String>,
// }

// #[derive(Deserialize)]
// struct DirectPuzzleGameData {
//     pgn: String
// }


impl Puzzle {
    fn info_comment(&self) -> String {
        let link: String = format!("https://lichess.org/training/{}", self.id);
        let comment: String = format!(
            "{} (from puzzle history)\nRating - {}\nThemes - {}",
            link, self.rating, self.themes.join(", ")
        );

        return comment;
    }

    fn build_pgn(&self, puzzle_num: usize) -> String {
        let headers: String = format!(
            "[Event \"Puzzle {}\"]\n\
             [Result \"*\"]\n\
             [Variant \"From Position\"]\n\
             [ECO \"?\"]\n\
             [Opening \"?\"]\n\
             [FEN \"{}\"]\n\
             [SetUp \"1\"]\n\
             [ChapterMode \"gamebook\"]",
            puzzle_num, self.fen
        );

        let fen_regions: Vec<&str> = self.fen.split_whitespace().collect(); 
        let puzzle_color = fen_regions[1];

        let pgn_moves = notation_tools::fen_to_pgn(self.fen.clone(), self.solution.clone());
        
        let mut pgn_output: String;

        // lazy code alert!
        if puzzle_color == "w" {
            pgn_output = "{ White to move }\n".to_string();
            for (i, mv) in pgn_moves.iter().enumerate() {
                let move_number = i / 2 + 1;
                let is_player_move = i % 2 == 0;
    
                if i == self.solution.len() - 1 {
                    pgn_output.push_str(&format!("{}.", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(&format!(" {{ {} }} ", self.info_comment()));
                } else if is_player_move {
                    pgn_output.push_str(&format!("{}.", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(" { Correct } ");
                } else {
                    pgn_output.push_str(&format!("{}...", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(" { White to move } ");
                }
            }
            return format!("{}\n\n{}", headers, pgn_output);
        } else {
            pgn_output = "{ Black to move }\n".to_string();
            for (i, mv) in pgn_moves.iter().enumerate() {
                let move_number = (i + 1) / 2 + 1;
                let is_player_move = i % 2 == 1;
    
                if i == self.solution.len() - 1 {
                    pgn_output.push_str(&format!("{}...", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(&format!(" {{ {} }} *", self.info_comment()));
                } else if is_player_move {
                    pgn_output.push_str(&format!("{}.", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(" { Black to move } ");
                } else {
                    pgn_output.push_str(&format!("{}...", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(" { Correct } ");
                }
            }
            return format!("{}\n\n{}", headers, pgn_output);
        }
    }
}


fn parse_puzzle(json_str: &str) -> serde_json::Result<PuzzleAttempt> {
    let puzzle_attempt: PuzzleAttempt = serde_json::from_str(json_str)?;
    Ok(puzzle_attempt)
}


// fn parse_direct_puzzle(json_str: &str) -> serde_json::Result<DirectPuzzleData> {
//     let direct_puzzle_data: DirectPuzzleData = serde_json::from_str(json_str)?;
//     Ok(direct_puzzle_data)
// }


// async fn get_puzzle_from_id(id: String) -> Result<Puzzle, Box<dyn Error>> {
//     let client = reqwest::Client::new();

//     let response = client
//         .get(format!("https://lichess.org/api/puzzle/{}", id))
//         .send()
//         .await?;

//     if response.status().is_success() {
//         let body = response.text().await?;
//         match parse_puzzle(&body) {
//             Ok(puzzle_attempt) => {
//                 Ok(puzzle_attempt.puzzle)
//             }
//             Err(e) => {
//                 eprintln!("Failed to parse puzzle: {}", e);
//                 Err(Box::from(e))
//             }
//         }

//     } else {
//         Err(Box::from(format!("API request error: {}", response.status())))
//     }

// }


// async fn get_puzzles_from_ids(ids: Vec<String>) -> Result<Vec<Puzzle>, Box<dyn Error>> {
//     let mut puzzles: Vec<Puzzle> = Vec::new();

//     for id in ids {
//         puzzles.push(get_puzzle_from_id(id).await?);
//     }

//     Ok(puzzles)
// }


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
    // clear_and_upload("mP8agodj", puzzle_history).await?;

    let board = Board::default();
    board.print();

    if let Some(coords) = Board::find_origin_of_move(&board, (5, 2), PieceName::Knight, PieceColor::White, (None, None)) {
        println!("{}, {}", coords.0, coords.1);
    }

    Ok(())
}