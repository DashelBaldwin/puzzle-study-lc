// main.rs

use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;

mod notation_tools;

//lip_NfKGhBFyeXxX0yerqoCi
const PAT: &str = "lip_NfKGhBFyeXxX0yerqoCi";

#[derive(Deserialize)]
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
    puzzle: Puzzle
}


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


async fn get_puzzle_history(max: i32) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let response = client
        .get("https://lichess.org/api/puzzle/activity")
        .headers(headers)
        .query(&[("max", max)])
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        let puzzle_attempt_strings: Vec<&str> = body.lines().collect();
        let mut incorrect_puzzles: Vec<Puzzle> = Vec::new();

        for puzzle_attempt_string in puzzle_attempt_strings {
            match parse_puzzle(puzzle_attempt_string) {
                Ok(puzzle_attempt) => {
                    if !puzzle_attempt.win {
                        incorrect_puzzles.push(puzzle_attempt.puzzle);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse puzzle: {}", e);
                }
            }
        }
        Ok(incorrect_puzzles)

    } else {
        Err(Box::from(format!("API request error: {}", response.status())))
    }
}


#[derive(Serialize)]
struct ImportPgnRequest {
    name: String,
    pgn: String,
    orientation: String,
    variant: String,
    mode: String
}


fn concatenate_pgn(puzzles: Vec<Puzzle>) -> String {
    let pgn_strings: Vec<String> = puzzles
        .iter()
        .enumerate()
        .map(|(index, puzzle)| puzzle.build_pgn(index + 1)) 
        .collect();
    
    pgn_strings.join("\n\n")
}


async fn post_puzzles_to_study(study_id: &str, puzzles: Vec<Puzzle>) -> Result<(), Box<dyn Error>> {
    let client: Client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let pgn_strings = concatenate_pgn(puzzles);

    let form = ImportPgnRequest {
        name: "Puzzle 1".to_string(),
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


// async fn clear_study


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
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to delete chapter ID={} with status: {}", id, response.status()))));  // Return an error
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // let puzzle_history = get_puzzle_history_blocking(10)?;
    // for puzzle in &puzzle_history {
    //     println!("{}", puzzle.build_pgn(0));
    // }
    // post_puzzles_to_study("mP8agodj", puzzle_history)?;

    let chapter_ids = get_study_chapter_ids("mP8agodj").await?;
    clear_study("mP8agodj", chapter_ids, false).await?;

    Ok(())
}