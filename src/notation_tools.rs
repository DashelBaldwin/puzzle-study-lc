// notation_tools.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceName {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    name: PieceName,
    color: PieceColor
}

#[derive(Debug, Clone)]
pub struct Board {
    contents: [[Option<Piece>; 8]; 8],
    en_passant_target: Option<(usize, usize)>
}

#[derive(Clone)]
struct PieceLocator {
    origin: (usize, usize),
    target: Piece,
    search_direction: (i32, i32),
    scope_restriction: (Option<usize>, Option<usize>),
    board: Board,
    is_jumper: bool
}

impl PieceLocator {
    fn new(
        origin: (usize, usize),
        target: Piece,
        search_direction: (i32, i32),
        scope_restriction: (Option<usize>, Option<usize>),
        board: &Board,
        is_jumper: bool,
    ) -> Self {
        Self {
            origin,
            search_direction,
            target,
            scope_restriction,
            board: board.clone(),
            is_jumper,
        }
    }

    fn locate(&self) -> Option<(usize, usize)> {
        let mut rank = self.origin.0 as i32;
        let mut file = self.origin.1 as i32;

        rank += self.search_direction.0;
        file += self.search_direction.1;

        while (0..8).contains(&rank) && (0..8).contains(&file) {
            if let Some(current) = self.board.contents[rank as usize][file as usize] {
                if current == self.target {
                    if let Some(rank_restriction) = self.scope_restriction.0 {
                        if rank as usize != rank_restriction { break; }
                    } else if let Some(file_restriction)= self.scope_restriction.1 {
                        if file as usize != file_restriction { break; }
                    }
                    Some((rank as usize, file as usize));
                }
                break; 
            }

            if self.is_jumper { break; }

            rank += self.search_direction.0;
            file += self.search_direction.1;
        }
        None
    }
}


impl Default for Board {
    fn default() -> Self {
        let mut board: [[Option<Piece>; 8]; 8] = [[None; 8]; 8];

        let create_piece = |name, color| Some(Piece { name, color });

        let standard_setup = vec![
            PieceName::Rook, PieceName::Knight, PieceName::Bishop, PieceName::Queen, 
            PieceName::King, PieceName::Bishop, PieceName::Knight, PieceName::Rook
        ];

        for i in 0..8 {
            board[0][i] = create_piece(standard_setup[i], PieceColor::White);
            board[1][i] = create_piece(PieceName::Pawn, PieceColor::White);
            board[6][i] = create_piece(PieceName::Pawn, PieceColor::Black);
            board[7][i] = create_piece(standard_setup[i], PieceColor::Black);
        }

        Self { contents: board, en_passant_target: None }
    }
}

impl Board {
    pub fn print(&self) {
        for row in self.contents.iter().rev() {
            for square in row.iter() {
                match square {
                    Some(piece) => {
                        let symbol = match (&piece.name, &piece.color) {
                            (PieceName::King, PieceColor::White) => "K",
                            (PieceName::Queen, PieceColor::White) => "Q",
                            (PieceName::Rook, PieceColor::White) => "R",
                            (PieceName::Bishop, PieceColor::White) => "B",
                            (PieceName::Knight, PieceColor::White) => "N",
                            (PieceName::Pawn, PieceColor::White) => "P",
                            (PieceName::King, PieceColor::Black) => "k",
                            (PieceName::Queen, PieceColor::Black) => "q",
                            (PieceName::Rook, PieceColor::Black) => "r",
                            (PieceName::Bishop, PieceColor::Black) => "b",
                            (PieceName::Knight, PieceColor::Black) => "n",
                            (PieceName::Pawn, PieceColor::Black) => "p",
                        };
                        print!("{} ", symbol);
                    }
                    None => print!(". "),
                }
            }
            println!();
        }
    }

    fn normal_movement(&mut self, from: (usize, usize), to: (usize, usize)) {
        let piece = self.contents[from.0][from.1].unwrap();
        self.contents[from.0][from.1] = None;
        self.contents[to.0][to.1] = Some(piece);
    }

    fn pawn_movement(&mut self, from: (usize, usize), to: (usize, usize), promotion: Option<Piece>) {
        self.normal_movement(from, to);

        let distance_moved = if to.0 > from.0 {to.0 - from.0} else {from.0 - to.0};

        if let Some(target) = self.en_passant_target {
            if to == target {
                match to.0 {
                    2 => self.contents[to.0 - 1][to.1] = None,
                    5 => self.contents[to.0 + 1][to.1] = None,
                    _ => println!("ep target not in valid ep location")
                }
            }
        } else if let Some(new_piece) = promotion {
            self.contents[to.0][to.1] = Some(new_piece);

        } else if distance_moved == 2 {
            self.en_passant_target = if to.0 == 3 {Some((2, to.1))} else {Some((5, to.1))}
        } else {
            self.en_passant_target = None;
        }
    } 

    fn king_movement(&mut self, from: (usize, usize), to: (usize, usize)) {
        let distance_moved = if to.1 > from.1 {to.1 - from.1} else {from.1 - to.1};

        if distance_moved > 1 {
            match to.1 {
                2 => self.normal_movement((from.0, 0), (from.0, 3)),
                7 => self.normal_movement((from.0, 7), (from.0, 6)),
                _ => println!("castling king landed at incorrect location")
            }
        } else {
            self.normal_movement(from, to);
        }
    }

    pub fn find_origin_of_move(
        &self,
        end_square: (usize, usize), 
        piece_name: PieceName, 
        piece_color: PieceColor, 
        scope_restriction: (Option<usize>, Option<usize>)
    ) -> Option<(usize, usize)> {
        let piece = Piece { name: piece_name, color: piece_color };
        let hv_locators: Vec<PieceLocator> = vec![
            PieceLocator::new(end_square, piece, (0, 1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (0, -1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (1, 0), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-1, 0), scope_restriction, &self.clone(), false)
        ];
        let diag_locators: Vec<PieceLocator> = vec![
            PieceLocator::new(end_square, piece, (1, 1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (1, -1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-1, 1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-1, -1), scope_restriction, &self.clone(), false)
        ];
        let kn_locators: Vec<PieceLocator> = vec![
            PieceLocator::new(end_square, piece, (1, 2), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (1, -2), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (2, 1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (2, -1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-1, 2), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-1, -2), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-2, 1), scope_restriction, &self.clone(), false),
            PieceLocator::new(end_square, piece, (-2, -1), scope_restriction, &self.clone(), false)
        ];
        let q_locators: Vec<PieceLocator> = [diag_locators.as_slice(), hv_locators.as_slice()].concat();

        match piece_name {
            PieceName::Rook => if let Some(origin) = find_piece_location(hv_locators) {
                Some(origin);
            }
            PieceName::Bishop => if let Some(origin) = find_piece_location(diag_locators) {
                Some(origin);
            }
            PieceName::Queen => if let Some(origin) = find_piece_location(q_locators) {
                Some(origin);
            }
            PieceName::Knight => if let Some(origin) = find_piece_location(kn_locators) {
                Some(origin);
            }
            _ => {
                println!("King or Pawn searched for origin square as if it were another piece")
            }
        }
        println!("Err: No piece was located...");
        None
    }
}

fn find_piece_location(locators: Vec<PieceLocator>) -> Option<(usize, usize)> {
    locators.into_iter()
        .find_map(|locator| locator.locate())
}

// 2 am trash coding session derived function that works correctly on the first test goes so hard
// if you're reading this, pray for me that this never breaks
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

    // waow 
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