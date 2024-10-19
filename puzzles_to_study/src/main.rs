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

        let pgn_moves = fen_to_pgn(self.fen.clone(), self.solution.clone());
        
        let mut pgn_output: String;

        // waow lazy code alert!
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


// fn fen_to_pgn(fen: String, ambiguous_moves: Vec<String>) -> Vec<String> {
//     let mut board: Vec<Vec<String>> = vec![vec![".".to_string(); 8]; 8];

//     let fen_regions: Vec<&str> = fen
//         .split(|c: char| c == '/' || c.is_whitespace())
//         .collect();

//     let fen_position_regions = &fen_regions[..8];

//     let mut rank: usize = 0;
//     for region in fen_position_regions {
//         let mut file: usize = 0;
//         for c in region.chars() {
//             if let Some(digit) = c.to_digit(10) {
//                 file += digit as usize - 1;
//             } else {
//                 let piece_c: String = c.to_uppercase().collect();
//                 board[rank][file] = piece_c;
//             }
//             file += 1;
//         }
//         rank += 1;
//     }

//     let mut ep_file: i32 = -1; // TODO change to the ep flag if provided in FEN

//     let mut moves: Vec<String> = Vec::new();
//     for ambiguous_move in ambiguous_moves {
//         let start_file = ambiguous_move.chars().nth(0).unwrap();
//         let start_rank = ambiguous_move.chars().nth(1).unwrap();
//         let end_file = ambiguous_move.chars().nth(2).unwrap();
//         let end_rank = ambiguous_move.chars().nth(3).unwrap();

//         let start_file_index = start_file as usize - 'a' as usize;
//         let end_file_index = end_file as usize - 'a' as usize;

//         let start_rank_index = 8 - (start_rank.to_digit(10).unwrap() as usize);
//         let end_rank_index = 8 - (end_rank.to_digit(10).unwrap() as usize);

//         let piece = board[start_rank_index][start_file_index].clone();
//         board[start_rank_index][start_file_index] = ".".to_string();

//         let mut was_double_pawn_move = false;

//         // waow 
//         if piece == "K" && (start_rank_index == 0 || start_rank_index == 7) && start_file_index == 4 {
//             if end_file_index == 6 {
//                 moves.push("O-O".to_string());
//                 board[start_rank_index][7] = ".".to_string();
//                 board[start_rank_index][5] = "R".to_string();
//             } else if end_file_index == 2 {
//                 moves.push("O-O-O".to_string());
//                 board[start_rank_index][0] = ".".to_string();
//                 board[start_rank_index][3] = "R".to_string();
//             }
//         } else if piece == "P" {
//             let mut pawn_move: String;
//             if start_file_index != end_file_index {
//                 pawn_move = format!("{}x{}{}", start_file, end_file, end_rank);
//                 if end_file_index as i32 == ep_file {
//                     if end_rank_index as i32 == 2 {
//                         board[3][end_file_index] = ".".to_string();
//                     } else {
//                         board[4][end_file_index] = ".".to_string();
//                     }
//                 }
//             } else {
//                 if (end_rank_index as i32 - start_rank_index as i32).abs() == 2 {
//                     was_double_pawn_move = true;
//                 }
//                 pawn_move = format!("{}{}", end_file, end_rank);
//             }

//             if end_rank_index == 0 || end_rank_index == 7 {
//                 let promotion: String = ambiguous_move.chars().nth(4).unwrap().to_uppercase().collect();
//                 pawn_move = format!("{}={}", pawn_move, promotion);
//             }
//             moves.push(pawn_move);
//         } else if board[end_rank_index][end_file_index] != "." {
//             moves.push(format!("{}x{}{}", piece, end_file, end_rank));
//         } else {
//             moves.push(format!("{}{}{}", piece, end_file, end_rank));
//         }
//         board[end_rank_index][end_file_index] = piece;

//         if was_double_pawn_move {
//             ep_file = end_file_index as i32;
//         } else {
//             ep_file = -1;
//         }
//     }

//     return moves;
// }

#[derive(Clone)]
struct Board {
    squares: Vec<Vec<String>>,
}

impl Board {
    fn new() -> Self {
        Board {
            squares: vec![vec![".".to_string(); 8]; 8],
        }
    }

    fn update_from_fen(&mut self, fen: &str) {
        let fen_regions: Vec<&str> = fen.split(|c: char| c == '/' || c.is_whitespace()).collect();
        let fen_position_regions = &fen_regions[..8];

        for (rank, region) in fen_position_regions.iter().enumerate() {
            let mut file: usize = 0;
            for c in region.chars() {
                if let Some(digit) = c.to_digit(10) {
                    file += digit as usize;
                } else {
                    let piece: String = c.to_uppercase().collect();
                    self.squares[rank][file] = piece;
                    file += 1;
                }
            }
        }
    }

    fn move_piece(&mut self, start: (usize, usize), end: (usize, usize)) -> String {
        let piece = self.squares[start.0][start.1].clone();
        self.squares[start.0][start.1] = ".".to_string();
        self.squares[end.0][end.1] = piece.clone();
        piece
    }

    fn index_to_square((rank, file): (usize, usize)) -> String {
        format!("{}{}", (file as u8 + b'a') as char, 8 - rank)
    }

    fn capture_piece(&mut self, start: (usize, usize), end: (usize, usize)) -> String {
        let piece = self.move_piece(start, end);
        format!("{}x{}", piece, Board::index_to_square(end))
    }

    
}


#[derive(Clone)]
struct Move {
    start_square: (usize, usize),
    end_square: (usize, usize),
    promotion: Option<char>,
}

impl Move {
    fn from_ambiguous(m: &str) -> Self {
        let start_file = m.chars().nth(0).unwrap();
        let start_rank = m.chars().nth(1).unwrap();
        let end_file = m.chars().nth(2).unwrap();
        let end_rank = m.chars().nth(3).unwrap();

        let start_square = (8 - start_rank.to_digit(10).unwrap() as usize, start_file as usize - 'a' as usize);
        let end_square = (8 - end_rank.to_digit(10).unwrap() as usize, end_file as usize - 'a' as usize);

        let promotion = m.chars().nth(4);
        Move {
            start_square,
            end_square,
            promotion,
        }
    }
}


fn handle_castling(board: &mut Board, start_square: (usize, usize), end_square: (usize, usize)) -> String {
    if end_square.1 == 6 {
        board.move_piece((start_square.0, 7), (start_square.0, 5));
        "O-O".to_string()
    } else {
        board.move_piece((start_square.0, 0), (start_square.0, 3));
        "O-O-O".to_string()
    }
}


fn handle_pawn_move(board: &mut Board, m: &Move, ep_file: i32) -> String {
    let (_start_rank, start_file) = m.start_square;
    let (end_rank, end_file) = m.end_square;

    let mut move_notation = if start_file != end_file {
        format!("{}x{}", (start_file as u8 + b'a') as char, Board::index_to_square(m.end_square))
    } else {
        Board::index_to_square(m.end_square)
    };

    if start_file != end_file && end_file as i32 == ep_file {
        board.squares[end_rank + 1][end_file] = ".".to_string();
    }

    if end_rank == 0 || end_rank == 7 {
        if let Some(promotion) = m.promotion {
            move_notation = format!("{}={}", move_notation, promotion.to_uppercase());
        }
    }

    move_notation
}


fn fen_to_pgn(fen: String, ambiguous_moves: Vec<String>) -> Vec<String> {
    let mut board = Board::new();
    board.update_from_fen(&fen);

    let mut moves: Vec<String> = Vec::new();
    let mut ep_file: i32 = -1;

    for ambiguous_move in ambiguous_moves {
        let m = Move::from_ambiguous(&ambiguous_move);
        let piece = board.squares[m.start_square.0][m.start_square.1].clone();

        let move_notation = match piece.as_str() {
            "K" if m.start_square.1 == 4 && (m.start_square.0 == 0 || m.start_square.0 == 7) => {
                handle_castling(&mut board, m.start_square, m.end_square)
            }
            "P" => {
                handle_pawn_move(&mut board, &m, ep_file)
            }
            _ if board.squares[m.end_square.0][m.end_square.1] != "." => {
                board.capture_piece(m.start_square, m.end_square)
            }
            _ => format!("{}{}", piece, Board::index_to_square(m.end_square)),
        };

        moves.push(move_notation);
        board.move_piece(m.start_square, m.end_square);

        ep_file = if piece == "P" && (m.start_square.0 as i32 - m.end_square.0 as i32).abs() == 2 {
            m.end_square.1 as i32
        } else {
            -1
        };
    }

    moves
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