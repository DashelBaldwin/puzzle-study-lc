// notation_tools.rs

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


pub fn fen_to_pgn(fen: String, ambiguous_moves: Vec<String>) -> Vec<String> {
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
