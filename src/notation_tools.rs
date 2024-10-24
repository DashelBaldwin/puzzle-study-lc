// notation_tools.js

// yes I know this is lazy but it works and I tested it and I don't want to attempt
// to refactor it again because weird edge cases break and they don't break here somehow 

// this converts an FEN string and the following non-descriptive positional notation moves (e.g. "f7h8")
// into a minimal PGN format; adding check + checkmate annotations and only adding disambiguations when
// necessary is not required for lichess to parse the file properly. however, creating a board to handle
// other special cases like en passant and promotion is still required, hence why this is so messy

pub fn fen_to_pgn(fen: String, ambiguous_moves: Vec<String>) -> Vec<String> {
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

    let mut ep_file: i32 = if fen_regions[10] != "-" {
        (fen_regions[10].chars().nth(0).unwrap() as usize -('a' as usize)) as i32
    } else {
        -1
    };

    let mut moves: Vec<String> = Vec::new();
    for ambiguous_move in ambiguous_moves {
        let start_file = ambiguous_move.chars().nth(0).unwrap();
        let start_rank = ambiguous_move.chars().nth(1).unwrap();
        let end_file = ambiguous_move.chars().nth(2).unwrap();
        let end_rank = ambiguous_move.chars().nth(3).unwrap();

        let start_file_index = start_file as usize - 'a' as usize;
        let start_rank_index = 8 - (start_rank.to_digit(10).unwrap() as usize);
        let end_file_index = end_file as usize - 'a' as usize;
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
            let mut pawn_move: String;
            if start_file_index != end_file_index {
                pawn_move = format!("{}x{}{}", start_file, end_file, end_rank);
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
                pawn_move = format!("{}{}", end_file, end_rank);
            }

            if end_rank_index == 0 || end_rank_index == 7 {
                let promotion: String = ambiguous_move.chars().nth(4).unwrap().to_uppercase().collect();
                pawn_move = format!("{}={}", pawn_move, promotion);
            }
            moves.push(pawn_move);
        } else if board[end_rank_index][end_file_index] != "." {
            moves.push(format!("{}{}{}x{}{}", piece, start_file, start_rank, end_file, end_rank));
        } else {
            moves.push(format!("{}{}{}{}{}", piece, start_file, start_rank, end_file, end_rank));
        }
        board[end_rank_index][end_file_index] = piece;

        if was_double_pawn_move {
            ep_file = end_file_index as i32;
        } else {
            ep_file = -1;
        }
    }
    moves
}