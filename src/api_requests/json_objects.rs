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
    pub imported_directly: Option<bool>
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
pub struct DirectPuzzle { // removed "pub id: String" because compiler flagged it as unused
    pub rating: i32,
    pub solution: Vec<String>,
    pub themes: Vec<String>,
}

#[derive(Deserialize)]
pub struct DirectPuzzleGameData {
    pub pgn: String
}

impl Puzzle {
    fn formatted_themes(&self) -> String {
        self.themes.iter().map(|s| {
            let mut result = String::new();
            let mut was_lower = false;

            for (i, ch) in s.chars().enumerate() {
                if i > 0 && ch.is_uppercase() && was_lower {
                    result.push(' ');
                }
                if i == 0 {
                    result.push(ch.to_ascii_uppercase());
                } else {
                    result.push(ch.to_ascii_lowercase());
                }
                was_lower = ch.is_lowercase();
            }
        result
        }).collect::<Vec<_>>().join(", ")
    }

    pub fn info_comment(&self) -> String {
        let link: String = format!("https://lichess.org/training/{}", self.id);
        let source: &str;
        if self.imported_directly == Some(true) {
            source = "(from ID)";
        } else {
            source = "(from puzzle history)"
        };
        let comment: String = format!(
            "{} {}\nRating - {}\nThemes - {}",
            link, source.to_string(), self.rating, self.formatted_themes()
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

        let fen_regions: Vec<&str> = self.fen
            .split(|c: char| c == '/' || c.is_whitespace())
            .collect();

        let ep_flag = fen_regions[10];
        
        let last_move = if ep_flag != "-" {
            let (file, rank) = ep_flag.split_at(1);
            let target_rank = rank.parse::<u8>().unwrap();
            let move_rank = if target_rank == 3 { 4 } else { 5 };
            format!("(Last move: {}{})\n", file, move_rank)
        } else {
            "".to_string()
        };

        // lazy alert!
        if puzzle_color == "w" {
            pgn_output = format!("{{ White to move \n{}}}\n", last_move).to_string();
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
            pgn_output = format!("{{ Black to move \n{}}}\n", last_move).to_string();
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
