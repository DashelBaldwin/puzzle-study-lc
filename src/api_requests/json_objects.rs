// json_objects.rs

use serde::Deserialize;
use crate::notation_utils;

#[derive(Deserialize)]
#[derive(Clone)]
pub struct Puzzle {
    pub id: String,
    pub rating: i32,
    pub solution: Vec<String>,
    pub themes: Vec<String>,
    pub fen: String,
}

#[derive(Deserialize)]
pub struct PuzzleAttempt {
    pub win: bool,
    pub puzzle: Puzzle, 
    pub date: i64
}

#[derive(Deserialize)]
pub struct DirectPuzzleData {
    pub game: DirectPuzzleGameData,
    pub puzzle: DirectPuzzle
}

#[derive(Deserialize)]
pub struct DirectPuzzle {
    pub id: String,
    pub rating: i32,
    pub solution: Vec<String>,
    pub themes: Vec<String>,
}

#[derive(Deserialize)]
pub struct DirectPuzzleGameData {
    pub pgn: String
}

impl Puzzle {
    pub fn info_comment(&self) -> String {
        let link: String = format!("https://lichess.org/training/{}", self.id);
        let comment: String = format!(
            "{} (from puzzle history)\nRating - {}\nThemes - {}",
            link, self.rating, self.themes.join(", ")
        );

        return comment;
    }

    pub fn build_pgn(&self, puzzle_num: usize) -> String {
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

        let pgn_moves = notation_utils::fen_to_pgn::fen_to_pgn(self.fen.clone(), self.solution.clone());
        
        let mut pgn_output: String;

        // lazy alert!
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

pub fn parse_puzzle(json_str: &str) -> serde_json::Result<PuzzleAttempt> {
    let puzzle_attempt: PuzzleAttempt = serde_json::from_str(json_str)?;
    Ok(puzzle_attempt)
}

pub fn parse_direct_puzzle(json_str: &str) -> serde_json::Result<DirectPuzzleData> {
    let direct_puzzle_data: DirectPuzzleData = serde_json::from_str(json_str)?;
    Ok(direct_puzzle_data)
}
