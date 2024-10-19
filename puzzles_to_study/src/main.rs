// main.rs

use serde::Deserialize;
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

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

    fn build_pgn(&self, puzzle_num: i32) -> String {
        let headers: String = format!(
            "[Event \"Puzzle {}\"]\n\
             [Result \"*\"]\n\
             [Variant \"Standard\"]\n\
             [ECO \"?\"]\n\
             [Opening \"?\"]\n\
             [FEN \"{}\"]\n\
             [ChapterMode \"gamebook\"]",
            puzzle_num, self.fen
        );

        let fen_regions: Vec<&str> = self.fen.split_whitespace().collect(); 
        let puzzle_color = fen_regions[1];

        let pgn_moves = notation_tools::fen_to_pgn(self.fen.clone(), self.solution.clone());
        
        let mut pgn_output: String;

        // lazy code alert!
        if puzzle_color == "w" {
            pgn_output = String::from("{ White to move }\n");
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
            pgn_output = String::from("{ Black to move }\n");
            for (i, mv) in pgn_moves.iter().enumerate() {
                let move_number = (i + 1) / 2 + 1;
                let is_player_move = i % 2 == 1;
    
                if i == self.solution.len() - 1 {
                    pgn_output.push_str(&format!("{}...", move_number));
                    pgn_output.push_str(&format!(" {}", mv));
                    pgn_output.push_str(&format!(" {{ {} }} ", self.info_comment()));
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


fn get_puzzle_history_blocking(max: i32) -> Result<Vec<Puzzle>, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", PAT))?);

    let response = client
        .get("https://lichess.org/api/puzzle/activity")
        .headers(headers)
        .query(&[("max", max)])
        .send()?;

    if response.status().is_success() {
        let body = response.text()?;
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




fn main() -> Result<(), Box<dyn Error>> {

    let puzzle_history = get_puzzle_history_blocking(1)?;
    for puzzle in puzzle_history {
        println!("{}", puzzle.build_pgn(1));
    }

    Ok(())
}