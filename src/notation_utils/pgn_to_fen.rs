// pgn_to_fen.rs

#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceName {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PieceColor {
    White,
    Black
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Piece {
    name: PieceName,
    color: PieceColor
}

#[derive(Debug, Clone)]
struct Board {
    contents: [[Option<Piece>; 8]; 8],
    en_passant_target: Option<(usize, usize)>
}

#[derive(Debug, Clone)]
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
                if (current.name == self.target.name) && (current.color == self.target.color) {
                    if let Some(rank_restriction) = self.scope_restriction.0 {
                        if rank as usize != rank_restriction { break; }
                    } else if let Some(file_restriction)= self.scope_restriction.1 {
                        if file as usize != file_restriction { break; }
                    }
                    return Some((rank as usize, file as usize));
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
    fn to_fen(&self, color: PieceColor) -> String {
        let mut fen = String::new();

        for row in self.contents.iter().rev() {
            let mut empty_count = 0;

            for &cell in row {
                match cell {
                    Some(piece) => {
                        if empty_count > 0 {
                            fen.push_str(&empty_count.to_string());
                            empty_count = 0; // Reset the empty count
                        }

                        let piece_char = match piece {
                            Piece { name: PieceName::Pawn, color } => if color == PieceColor::White { 'P' } else { 'p' },
                            Piece { name: PieceName::Knight, color } => if color == PieceColor::White { 'N' } else { 'n' },
                            Piece { name: PieceName::Bishop, color } => if color == PieceColor::White { 'B' } else { 'b' },
                            Piece { name: PieceName::Rook, color } => if color == PieceColor::White { 'R' } else { 'r' },
                            Piece { name: PieceName::Queen, color } => if color == PieceColor::White { 'Q' } else { 'q' },
                            Piece { name: PieceName::King, color } => if color == PieceColor::White { 'K' } else { 'k' },
                        };
                        fen.push(piece_char);
                    }
                    None => {
                        empty_count += 1;
                    }
                }
            }

            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            fen.push('/');
        }

        if fen.ends_with('/') {
            fen.pop();
        }

        let move_char = if color == PieceColor::White {'w'} else {'b'};

        let additional_info = format!(" {} - - 0 1", move_char);
        fen.push_str(&additional_info);

        fen
    }

    fn _debug_print(&self) {
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
                    _ => println!("ERROR ep target not in valid ep location")
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
                6 => self.normal_movement((from.0, 7), (from.0, 5)),
                _ => println!("ERROR castling king landed at incorrect location")
            }
        } 
        self.normal_movement(from, to);
    }

    fn find_origin_of_move(
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
            PieceLocator::new(end_square, piece, (1, 2), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (1, -2), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (2, 1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (2, -1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-1, 2), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-1, -2), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-2, 1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-2, -1), scope_restriction, &self.clone(), true)
        ];
        let kg_locators: Vec<PieceLocator> = vec![
            PieceLocator::new(end_square, piece, (1, 1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (1, -1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-1, 1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-1, -1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (0, 1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (0, -1), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (1, 0), scope_restriction, &self.clone(), true),
            PieceLocator::new(end_square, piece, (-1, 0), scope_restriction, &self.clone(), true)
        ];
        let q_locators: Vec<PieceLocator> = [diag_locators.as_slice(), hv_locators.as_slice()].concat();

        match piece_name {
            PieceName::Rook => if let Some(origin) = find_piece_location(hv_locators) {
                return Some(origin);
            }
            PieceName::Bishop => if let Some(origin) = find_piece_location(diag_locators) {
                return Some(origin);
            }
            PieceName::Queen => if let Some(origin) = find_piece_location(q_locators) {
                return Some(origin);
            }
            PieceName::Knight => {
                if let Some(origin) = find_piece_location(kn_locators) {
                    return Some(origin);
                }
            }
            PieceName::King => {
                if let Some(origin) = find_piece_location(kg_locators) {
                    return Some(origin);
                }
            }
            PieceName::Pawn => {
                println!("Pawn searched for origin square as if it were another piece")
            }
        }
        println!("find_origin_of_move: ERROR No piece was located...");
        None
    }

    fn find_origin_of_pawn_move(
        &self,
        end_square: (usize, usize), 
        piece_color: PieceColor, 
        file_restriction: Option<usize>
    ) -> Option<(usize, usize)> {
        let pawn = Piece { name: PieceName::Pawn, color: piece_color };
        let search_direction = if piece_color == PieceColor::White {(-1, 0)} else {(1, 0)};
        let search_from = if let Some(file) = file_restriction {(end_square.0, file)} else {end_square};
        if let Some(origin) = PieceLocator::new(
            search_from, 
            pawn, 
            search_direction, 
            (None, None), 
            &self.clone(), 
            false
        ).locate() {
            return Some(origin);
        }
        println!("find_origin_of_pawn_move: No pawn move found");
        None
    }
}

fn find_piece_location(locators: Vec<PieceLocator>) -> Option<(usize, usize)> {
    for locator in locators {
        if let Some(coords) = locator.locate() {
            return Some(coords);
        }
    }
    None
}

fn piecename_from_char(c: char) -> PieceName {
    match c {
        'K' => PieceName::King,
        'N' => PieceName::Knight,
        'B' => PieceName::Bishop,
        'Q' => PieceName::Queen,
        'R' => PieceName::Rook,
        _ => {
            println!("piecename_from_char: no matching piece for char {}", c);
            PieceName::King
        }
    }
}

fn file_idx_from_char(c: char) -> usize {
    if ('a'..='h').contains(&c) {
        (c as usize) - ('a' as usize)
    } else {
        println!("file_idx_from_char: no matching file for char {}", c);
        0
    }
}

pub fn pgn_to_fen(pgn_string: &str) -> String {
    let plys: Vec<String> = pgn_string
        .split_whitespace()
        .map(|s| {
            let mut cleaned = s.to_string();
            if cleaned.ends_with('+') || cleaned.ends_with('#') {
                cleaned.pop();
            }
            cleaned
        })
        .collect();

    let mut board = Board::default();
    let mut turn = PieceColor::White;

    for ply in plys {
        match ply.as_str() {
            "O-O" => match turn {
                PieceColor::White => board.king_movement((0, 4), (0, 6)),
                PieceColor::Black => board.king_movement((7, 4), (7, 6)),
            },
            "O-O-O" => match turn {
                PieceColor::White => board.king_movement((0, 4), (0, 2)),
                PieceColor::Black => board.king_movement((7, 4), (7, 2)),
            },
            _ => {
                let mut ply_chars: Vec<char> = ply.chars().collect();
                let to: (usize, usize);

                if ply_chars[0].is_lowercase() {
                    let mut promotion: Option<Piece> = None;

                    if ply_chars[ply_chars.len() - 2] == '=' {
                        promotion = Some(Piece {
                            name: piecename_from_char(ply_chars[ply_chars.len() - 1]),
                            color: turn,
                        });
                    }

                    if ply_chars[1] == 'x' {
                        let r = ply_chars[3].to_digit(10)
                            .expect("pgn_to_fen: failed to convert digit to usize") as usize;
                        to = (r - 1, file_idx_from_char(ply_chars[2]));
                        let search_file = file_idx_from_char(ply_chars[0]);
                        
                        if let Some(from) = board.find_origin_of_pawn_move(to, turn, Some(search_file)) {
                            board.pawn_movement(from, to, promotion);
                        }
                    } else {
                        let r = ply_chars[1].to_digit(10)
                            .expect("pgn_to_fen: failed to convert digit to usize") as usize;
                        to = (r - 1, file_idx_from_char(ply_chars[0]));

                        if let Some(from) = board.find_origin_of_pawn_move(to, turn, None) {
                            board.pawn_movement(from, to, promotion);
                        }
                    }
                } else {
                    ply_chars.retain(|&c| c != 'x');
                    let piece = Piece {
                        name: piecename_from_char(ply_chars[0]),
                        color: turn,
                    };
                    let r = ply_chars[ply_chars.len() - 1].to_digit(10)
                        .expect("pgn_to_fen: failed to convert digit to usize") as usize;
                    to = (r - 1, file_idx_from_char(ply_chars[ply_chars.len() - 2]));

                    let mut scope_restriction: (Option<usize>, Option<usize>) = (None, None);

                    match ply_chars.len() {
                        5 => {
                            scope_restriction.0 = Some(ply_chars[2].to_digit(10)
                                .expect("pgn_to_fen: failed to convert digit to usize") as usize);
                            scope_restriction.1 = Some(file_idx_from_char(ply_chars[1]));
                        }
                        4 => {
                            let token = ply_chars[1];
                            if token.is_digit(10) {
                                scope_restriction.0 = Some(token.to_digit(10)
                                    .expect("pgn_to_fen: failed to convert digit to usize") as usize);
                            } else {
                                scope_restriction.1 = Some(file_idx_from_char(token));
                            }
                        }
                        _ => {}
                    }

                    if let Some(from) = board.find_origin_of_move(to, piece.name, piece.color, scope_restriction) {
                        board.normal_movement(from, to);
                    }
                }
            }
        }

        turn = if turn == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        };
    }

    board.to_fen(turn)
}
