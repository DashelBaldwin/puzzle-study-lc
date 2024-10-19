use serde::Deserialize;
use std::error::Error;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

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
            "{} (from puzzle history)\nRating - {}\nThemes - '{}'",
            link, self.rating, self.themes.join("', '")
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
        let puzzle_color = fen_regions[fen_regions.len() - 2];

        let pgn_moves = fen_to_pgn(self.fen.clone(), self.solution.clone());
        
        let mut pgn_output: String;

        // wow
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


fn fen_to_pgn(fen: String, ambiguous_moves: Vec<String>) -> Vec<String> {
    let mut board: Vec<Vec<String>> = vec![vec![".".to_string(); 8]; 8];

    let fen_regions: Vec<&str> = fen
        .split(|c: char| c == '/' || c.is_whitespace())
        .collect();

    let fen_position_regions = &fen_regions[..8];

    let mut rank: usize = 0;
    for region in fen_position_regions {
        let mut file: usize = 0;
        for c in region.chars() {
            if let Some(digit) = c.to_digit(10) {
                file += digit as usize - 1;
            } else {
                let piece_c: String = c.to_uppercase().collect();
                board[rank][file] = piece_c;
            }
            file += 1;
        }
        rank += 1;
    }

    let mut ep_file: i32 = -1; // TODO change to the ep flag if provided in FEN

    let mut moves: Vec<String> = Vec::new();
    for ambiguous_move in ambiguous_moves {
        let start_file = ambiguous_move.chars().nth(0).unwrap();
        let start_rank = ambiguous_move.chars().nth(1).unwrap();
        let end_file = ambiguous_move.chars().nth(2).unwrap();
        let end_rank = ambiguous_move.chars().nth(3).unwrap();

        let start_file_index = start_file as usize - 'a' as usize;
        let end_file_index = end_file as usize - 'a' as usize;

        let start_rank_index = 8 - (start_rank.to_digit(10).unwrap() as usize);
        let end_rank_index = 8 - (end_rank.to_digit(10).unwrap() as usize);

        let piece = board[start_rank_index][start_file_index].clone();
        board[start_rank_index][start_file_index] = ".".to_string();

        let mut was_double_pawn_move = false;

        // waow 
        if piece == "K" && (start_rank_index == 0 || start_rank_index == 7) && start_file_index == 4 {
            if end_file_index == 6 {
                moves.push("O-O".to_string());
                board[start_rank_index][7] = ".".to_string();
                board[start_rank_index][5] = "R".to_string();
            } else if end_file_index == 2 {
                moves.push("O-O-O".to_string());
                board[start_rank_index][0] = ".".to_string();
                board[start_rank_index][3] = "R".to_string();
            }
        } else if piece == "P" {
            if start_file_index != end_file_index {
                moves.push(format!("{}x{}{}", start_file, end_file, end_rank));
                if end_file_index as i32 == ep_file {
                    if end_rank_index as i32 == 2 {
                        board[3][end_file_index] = ".".to_string();
                    } else {
                        board[4][end_file_index] = ".".to_string();
                    }
                }
            } else {
                if (end_rank_index as i32 - start_rank_index as i32).abs() == 2 {
                    was_double_pawn_move = true;
                }
                moves.push(format!("{}{}", end_file, end_rank));
            }
        } else if board[end_rank_index][end_file_index] != "." {
            moves.push(format!("{}x{}{}", piece, end_file, end_rank));
        } else {
            moves.push(format!("{}{}{}", piece, end_file, end_rank));
        }
        board[end_rank_index][end_file_index] = piece;

        if was_double_pawn_move {
            ep_file = end_file_index as i32;
        } else {
            ep_file = -1;
        }
    }

    return moves;
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
                        println!("{}", puzzle_attempt.puzzle.build_pgn(1));
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


// fn post_to_study_blocking() -> Result<(), Box<dyn Error>> {
    
// }

// { White to move }
// 1. exf7 { Correct } 1... Kxf7 { White to move } 2. Kd5 { Correct } 2... c4 { White to move } 3. Kxc4 { Correct } 3... Kf6 { White to move } 4. Kc5 { Info comment } *
// 1. d4e2 { Correct } 1... g1f1 { White to move } 2. e2f4 { https://lichess.org/training/biY6X (from puzzle history)

// Given FEN, propagate a Vec<Vec<String>> representing the board.
// Board struct needs a flag for en passant target
// Board impl:
//      get_pgn_moves(solution: Vec<String>) -> Vec<String>
//          returns solution.len() Strings in a vector representing the PGN form 
//          of the ambiguous moves dictated in solution. These are then used in
//          place of solution where study pgns are generated.
//

fn main() -> Result<(), Box<dyn Error>> {

    get_puzzle_history_blocking(1)?;

    Ok(())
}