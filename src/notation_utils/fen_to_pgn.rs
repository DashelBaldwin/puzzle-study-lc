// fen_to_pgn.rs

pub fn fen_to_pgn(fen: impl Into<String>, ambiguous_moves: impl Into<Vec<String>>) -> Vec<String> {
    let fen: String = fen.into();          
    let ambiguous_moves: Vec<String> = ambiguous_moves.into();

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
        (fen_regions[10].chars().nth(0).unwrap() as usize - ('a' as usize)) as i32
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

        let mut piece = board[start_rank_index][start_file_index].clone();
        board[start_rank_index][start_file_index] = ".".to_string();

        let mut was_double_pawn_move = false;

        if piece == "K" 
            && (start_rank_index == 0 || start_rank_index == 7) 
            && start_file_index == 4 
            && (end_file_index == 6 || end_file_index == 2) 
        {
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
                piece = promotion;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn to_str_vec(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|&s| s.to_string()).collect()
    }

    #[test]
    fn test_fen_to_pgn() {
        // https://lichess.org/training/zOm2u
        assert_eq!(
            fen_to_pgn(
                "r4rk1/ppp3b1/3p3p/3Pp1pn/P1P1P1qn/2NQ1pP1/1P1B1PBP/R1R1N1K1 b - - 0 1",
                to_str_vec(vec!["f3g2", "d3e2", "f8f2", "e2f2", "a8f8", "f2f8", "g8f8"])
            ),
            vec!["fxg2", "Qd3e2", "Rf8xf2", "Qe2xf2", "Ra8f8", "Qf2xf8", "Kg8xf8"]
        );

        // https://lichess.org/training/jlm4M
        assert_eq!(
            fen_to_pgn(
                "2k5/1pp3N1/p3P3/5p2/7p/P5n1/1PP3P1/2K5 b - - 0 1",
                to_str_vec(vec!["c8d8", "c1d2", "f5f4", "d2e1", "d8e7", "e1f2", "e7f8", "e6e7", "f8e7"])
            ),
            vec!["Kc8d8", "Kc1d2", "f4", "Kd2e1", "Kd8e7", "Ke1f2", "Ke7f8", "e7", "Kf8xe7"]
        );
        
        // https://lichess.org/training/qe9En
        assert_eq!(
            fen_to_pgn(
                "8/5ppp/4p3/8/3PP1PP/3K1k2/8/8 b - - 1 1",
                to_str_vec(vec!["f3g4", "d3c4", "f7f5", "d4d5", "e6d5", "e4d5", "f5f4", "d5d6", "f4f3", "d6d7", "f3f2", "d7d8q", "f2f1q"])
            ),
            vec!["Kf3xg4", "Kd3c4", "f5", "d5", "exd5", "exd5", "f4", "d6", "f3", "d7", "f2", "d8=Q", "f1=Q"]
        );
        
        // https://lichess.org/training/euwQI
        assert_eq!(
            fen_to_pgn(
                "1k3r2/8/PPKN4/8/8/4p3/8/8 w - - 0 1",
                to_str_vec(vec!["a6a7", "b8a8", "c6c7", "e3e2", "b6b7", "a8a7", "d6c8", "a7a6", "b7b8q"])
            ),
            vec!["a7", "Kb8a8", "Kc6c7", "e2", "b7", "Ka8xa7", "Nd6c8", "Ka7a6", "b8=Q"]
        );
        
        // https://lichess.org/training/vEK4Z
        assert_eq!(
            fen_to_pgn(
                "8/1Q1R4/pP2p2p/4P3/3B4/1kP1K3/5pqP/8 b - - 0 1",
                to_str_vec(vec!["g2g1", "b7h1", "f2f1n", "e3e2", "g1h1"])
            ),
            vec!["Qg2g1", "Qb7h1", "f1=N", "Ke3e2", "Qg1xh1"]
        );
        
        // https://lichess.org/training/N9l1q
        assert_eq!(
            fen_to_pgn(
                "6k1/5p2/2p3pQ/1p3q1P/2n1pP2/1BP3R1/1P3PK1/r7 b - - 1 1",
                to_str_vec(vec!["e4e3", "h5g6", "f5d5", "g3f3", "e3e2", "h6h7", "g8f8", "g6g7", "f8e7", "g7g8q", "e2e1n", "g2h3", "e1f3"])
            ),
            vec!["e3", "hxg6", "Qf5d5", "Rg3f3", "e2", "Qh6h7", "Kg8f8", "g7", "Kf8e7", "g8=Q", "e1=N", "Kg2h3", "Ne1xf3"]
        );
        
        // https://lichess.org/training/R0zaE
        assert_eq!(
            fen_to_pgn(
                "1n1qk2r/pQ3ppp/2p1p3/2bp4/2P1n1b1/5P2/PP1PP1BP/RNB1K1NR b KQk - 0 1",
                to_str_vec(vec!["d8h4", "e1d1", "e4f2", "d1c2", "g4f5", "e2e4", "e8g8", "b1c3", "d5e4"])
            ),
            vec!["Qd8h4", "Ke1d1", "Ne4f2", "Kd1c2", "Bg4f5", "e4", "O-O", "Nb1c3", "dxe4"]
        );
        
        // https://lichess.org/training/xbnI7
        assert_eq!(
            fen_to_pgn(
                "r3k2r/1p4p1/pp1p4/4q1B1/3nP1Q1/P2P4/1PP3P1/R4RK1 b kq - 1 1",
                to_str_vec(vec!["e5h2", "g1f2", "e8g8", "f2e3", "d4c2", "e3d2", "c2a1"])
            ),
            vec!["Qe5h2", "Kg1f2", "O-O", "Kf2e3", "Nd4xc2", "Ke3d2", "Nc2xa1"]
        );
        
        // https://lichess.org/training/xw2Nb
        assert_eq!(
            fen_to_pgn(
                "r2qk2r/p1pp2pp/1p2p3/2b1Pp2/5PPn/2NQ2K1/PPP4P/R1B2R2 w kq f6 0 1",
                to_str_vec(vec!["e5f6", "d8f6", "c3e4"])
            ),
            vec!["exf6", "Qd8xf6", "Nc3e4"]
        );
        
        // https://lichess.org/training/2csxh
        assert_eq!(
            fen_to_pgn(
                "2kr3N/pppnb1pp/5n2/6B1/2PPp3/7q/PP2BP1N/R2Q1R1K b - - 0 1",
                to_str_vec(vec!["e7d6", "f2f4", "e4f3", "f1f2", "f6e4"])
            ),
            vec!["Be7d6", "f4", "exf3", "Rf1f2", "Nf6e4"]
        );
    }
}
