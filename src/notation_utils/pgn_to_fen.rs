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
    is_jump: bool
}

impl PieceLocator {
    fn new(
        origin: (usize, usize),
        target: Piece,
        search_direction: (i32, i32),
        scope_restriction: (Option<usize>, Option<usize>),
        board: &Board,
        is_jump: bool,
    ) -> Self {
        Self {
            origin,
            search_direction,
            target,
            scope_restriction,
            board: board.clone(),
            is_jump,
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
                    }
                    if let Some(file_restriction) = self.scope_restriction.1 {
                        if file as usize != file_restriction { break; }
                    }
                    return Some((rank as usize, file as usize));
                }
                break; 
            }

            if self.is_jump { break; }

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
                            empty_count = 0;
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

        let additional_info: String;

        if let Some(ep_target) = self.en_passant_target {
            if color == PieceColor::White {
                let file = (b'a' + (ep_target.0) as u8) as char;
            let rank = ep_target.1+1;
            additional_info = format!(" {} KQkq {}{} 0 1", move_char, file, rank);
            } else {
                let file = (b'a' + (7-ep_target.0) as u8) as char;
            let rank = (7-ep_target.1)+1;
            additional_info = format!(" {} KQkq {}{} 0 1", move_char, file, rank);
            }
            
        } else {
            additional_info = format!(" {} KQkq - 0 1", move_char);
        }
        
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
        if piece.name != PieceName::Pawn { self.en_passant_target = None; }
    }

    fn pawn_movement(&mut self, from: (usize, usize), to: (usize, usize), promotion: Option<Piece>) {
        self.normal_movement(from, to);

        let distance_moved = if to.0 > from.0 {to.0 - from.0} else {from.0 - to.0};

        if let Some(target) = self.en_passant_target {
            if to == target {
                match to.0 {
                    2 => self.contents[to.0 + 1][to.1] = None,
                    5 => self.contents[to.0 - 1][to.1] = None,
                    _ => ()
                }
                ()
            }
        } 
        if let Some(new_piece) = promotion {
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
                _ => ()
            }
        } 
        self.normal_movement(from, to);
    }

    fn spawn_locators(
        &self,
        directions: &[(i32, i32)],
        end_square: (usize, usize),
        piece: Piece,
        scope_restriction: (Option<usize>, Option<usize>),
        is_jump: bool,
        board: &Board,
    ) -> Vec<PieceLocator> {
        directions.iter()
            .map(|&(dx, dy)| PieceLocator::new(end_square, piece, (dx, dy), scope_restriction, &board.clone(), is_jump))
            .collect()
    }

    fn find_origin_of_move(
        &self,
        end_square: (usize, usize), 
        piece_name: PieceName, 
        piece_color: PieceColor, 
        scope_restriction: (Option<usize>, Option<usize>)
    ) -> Option<(usize, usize)> {
        let piece = Piece { name: piece_name, color: piece_color };
        let hv_directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let diag_directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        let kn_directions = [(1, 2), (1, -2), (2, 1), (2, -1), (-1, 2), (-1, -2), (-2, 1), (-2, -1)];
        let kg_directions = [(1, 1), (1, -1), (-1, 1), (-1, -1), (0, 1), (0, -1), (1, 0), (-1, 0)];

        let hv_locators = self.spawn_locators(&hv_directions, end_square, piece, scope_restriction, false, self);
        let diag_locators = self.spawn_locators(&diag_directions, end_square, piece, scope_restriction, false, self);
        let kn_locators = self.spawn_locators(&kn_directions, end_square, piece, scope_restriction, true, self);
        let kg_locators = self.spawn_locators(&kg_directions, end_square, piece, scope_restriction, true, self);
        let q_locators = [diag_locators.as_slice(), hv_locators.as_slice()].concat();

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
            PieceName::Pawn => ()
        }
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
        None
    }
}

// This was made before I knew to use results... *chaos ensues*
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
        _ => PieceName::Pawn
    }
}

fn file_idx_from_char(c: char) -> usize {
    if ('a'..='h').contains(&c) {
        (c as usize) - ('a' as usize)
    } else {
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
                        let r = ply_chars[3].to_digit(10).unwrap() as usize;
                        to = (r - 1, file_idx_from_char(ply_chars[2]));
                        let search_file = file_idx_from_char(ply_chars[0]);
                        
                        if let Some(from) = board.find_origin_of_pawn_move(to, turn, Some(search_file)) {
                            board.pawn_movement(from, to, promotion);
                        }
                    } else {
                        let r = ply_chars[1].to_digit(10).unwrap() as usize;
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
                    let r = ply_chars[ply_chars.len() - 1].to_digit(10).unwrap() as usize;
                    to = (r - 1, file_idx_from_char(ply_chars[ply_chars.len() - 2]));

                    let mut scope_restriction: (Option<usize>, Option<usize>) = (None, None);

                    match ply_chars.len() {
                        5 => {
                            scope_restriction.0 = Some(ply_chars[2].to_digit(10).unwrap() as usize);
                            scope_restriction.1 = Some(file_idx_from_char(ply_chars[1]));
                        }
                        4 => {
                            let token = ply_chars[1];
                            if token.is_digit(10) {
                                scope_restriction.0 = Some(token.to_digit(10).unwrap() as usize - 1);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgn_to_fen() {
        // https://lichess.org/training/zOm2u
        assert_eq!(
            pgn_to_fen(
                "c4 Nf6 g3 g6 Bg2 Bg7 Nc3 O-O Nf3 d6 d4 Nc6 O-O e5 d5 Ne7 e4 Nh5 \
                 Be3 f5 Qc2 f4 Bd2 h6 a4 g5 Ne1 Ng6 Nd3 f3 Bh1 Bh3 Rfd1 Qd7 Ne1 Qg4 \
                 Qd3 Nh4 Rdc1 Bg2 Bxg2"
            ),
            "r4rk1/ppp3b1/3p3p/3Pp1pn/P1P1P1qn/2NQ1pP1/1P1B1PBP/R1R1N1K1 b KQkq - 0 1"
        );
    
        // https://lichess.org/training/jlm4M
        assert_eq!(
            pgn_to_fen(
                "Nf3 Nc6 d4 d6 Bf4 Nf6 e3 Bg4 Nbd2 Nh5 Bg5 f6 Bh4 g5 Bg3 Nxg3 hxg3 \
                 Qd7 Qe2 O-O-O Qb5 a6 Qb3 Be6 Qa4 Na7 Qxd7+ Bxd7 O-O-O Bb5 Bxb5 \
                 Nxb5 a3 e5 dxe5 dxe5 Nc4 Rxd1+ Rxd1 Bd6 Nxd6+ Nxd6 Nd2 f5 Nb3 h5 \
                 Rd5 Re8 f4 gxf4 gxf4 exf4 exf4 Rf8 Re5 Ne4 Nd4 Ng3 Ne6 Re8 Ng7 \
                 Rxe5 fxe5 h4 e6"
            ),
            "2k5/1pp3N1/p3P3/5p2/7p/P5n1/1PP3P1/2K5 b KQkq - 0 1"
        );
        
        // https://lichess.org/training/qe9En
        assert_eq!(
            pgn_to_fen(
                "d4 e6 c4 Nf6 Nc3 Bb4 Nf3 b6 Bd2 Bb7 e3 c5 Be2 Bxc3 bxc3 cxd4 cxd4 \
                 d5 Ne5 dxc4 O-O Nc6 Nxc6 Bxc6 Bxc4 Bd5 Qa4+ Qd7 Qxd7+ Kxd7 Bxd5 \
                 Nxd5 Rfc1 Rhc8 h3 Rxc1+ Rxc1 Rc8 Rxc8 Kxc8 Kf1 Kb7 Ke2 Ka6 Kd3 \
                 Kb5 Kc2 Nb4+ Bxb4 Kxb4 Kb2 Kc4 Kc2 a5 a3 b5 g4 b4 axb4 axb4 h4 \
                 b3+ Kb2 Kd3 Kxb3 Ke2 Kc4 Kxf2 e4 Kf3 Kd3"
            ),
            "8/5ppp/4p3/8/3PP1PP/3K1k2/8/8 b KQkq - 0 1"
        );
        
        // https://lichess.org/training/euwQI
        assert_eq!(
            pgn_to_fen(
                "e4 e5 Nf3 d6 d4 Bg4 dxe5 Bxf3 Qxf3 dxe5 Bc4 Qe7 Qb3 Qb4+ Nc3 Qxb3 \
                 Bxb3 Nd7 Nd5 O-O-O Be3 Kb8 O-O-O g6 Nc3 h6 Bxf7 Ne7 Rd2 Nc6 Rhd1 \
                 Bd6 Bxg6 Rhg8 Bf5 Rxg2 Bxh6 Nd4 Bxd7 Rxd7 Be3 Nf3 Rd5 a6 Bc5 Kc8 \
                 Bxd6 cxd6 Rxd6 Rxd6 Rxd6 Rxh2 Nd1 Kc7 Rf6 Rh3 c3 a5 Kc2 b5 Kb3 \
                 Nd2+ Ka3 Nxe4 Ra6 b4+ Ka4 bxc3 Nxc3 Nc5+ Kb5 Nxa6 Kxa6 Rf3 Kxa5 \
                 Rxf2 b4 Rc2 Nb5+ Kb8 a4 e4 Nd4 Rc4 Nf5 Rc7 b5 Ra7+ Kb4 Kc7 a5 \
                 Ra8 b6+ Kb7 Kb5 Rd8 a6+ Kb8 Kc6 Rf8 Nd6 e3"
            ),
            "1k3r2/8/PPKN4/8/8/4p3/8/8 w KQkq - 0 1"
        );
        
        // https://lichess.org/training/vEK4Z
        assert_eq!(
            pgn_to_fen(
                "e4 e5 d4 exd4 Nf3 Nc6 Bc4 h6 Nxd4 Bc5 Nxc6 Qf6 O-O dxc6 e5 Qh4 \
                 Qf3 Be6 Bxe6 fxe6 Nc3 O-O-O Ne4 Bb6 a4 a6 c3 Ne7 a5 Rhf8 Qe2 \
                 Rf5 axb6 cxb6 Nd6+ Kc7 Nxf5 Nxf5 Be3 c5 b4 Qe4 Rfe1 Nh4 f3 Qg6 \
                 bxc5 Rf8 cxb6+ Kb8 Qf2 Nxf3+ Kh1 Qh5 gxf3 Rxf3 Qe2 Qh3 Rg1 g5 \
                 Bd4 Kc8 Qg2 Qf5 Raf1 g4 Rxf3 gxf3 Qg3 Qe4 Qg8+ Kd7 Rg7+ Kc6 \
                 Qe8+ Kd5 Rd7+ Kc4 Qc8+ Kb3 Kg1 Qg4+ Kf2 Qg2+ Ke3 f2 Qxb7"
            ),
            "8/1Q1R4/pP2p2p/4P3/3B4/1kP1K3/5pqP/8 b KQkq - 0 1"
        );
        
        // https://lichess.org/training/N9l1q
        assert_eq!(
            pgn_to_fen(
                "Nf3 d5 d4 Nf6 Bg5 Nc6 e3 Bg4 Bb5 a6 Ba4 b5 Bb3 e6 O-O Be7 h3 Bh5 \
                 a4 h6 Bh4 Ne4 Bxe7 Nxe7 axb5 axb5 Rxa8 Qxa8 Nbd2 O-O Qe2 Nxd2 \
                 Qxd2 Bxf3 gxf3 Ng6 Kh1 c6 Rg1 Qc8 Rg3 e5 dxe5 Nxe5 e4 dxe4 \
                 Qxh6 g6 f4 Nc4 c3 Qf5 h4 Ra8 h5 Ra1+ Kg2"
            ),
            "6k1/5p2/2p3pQ/1p3q1P/2n1pP2/1BP3R1/1P3PK1/r7 b KQkq - 0 1"
        );
        
        // https://lichess.org/training/R0zaE
        assert_eq!(
            pgn_to_fen("g4 d5 Bg2 Bxg4 c4 c6 Qb3 Nf6 Qxb7 e6 Qxa8 Bc5 Qb7 Ne4 f3"),
            "1n1qk2r/pQ3ppp/2p1p3/2bp4/2P1n1b1/5P2/PP1PP1BP/RNB1K1NR b KQkq - 0 1"
        );
        
        // https://lichess.org/training/xbnI7
        assert_eq!(
            pgn_to_fen(
                "e4 e5 Nf3 Nf6 d3 Nc6 Nc3 Bc5 Be2 d6 O-O Ng4 a3 a6 h3 h5 hxg4 hxg4 \
                 Ng5 f6 Bxg4 fxg5 Bxc8 Qxc8 Bxg5 Qe6 Nd5 Bb6 Nxb6 cxb6 f4 Nd4 \
                 fxe5 Qxe5 Qg4"
            ),
            "r3k2r/1p4p1/pp1p4/4q1B1/3nP1Q1/P2P4/1PP3P1/R4RK1 b KQkq - 0 1"
        );
        
        // https://lichess.org/training/xw2Nb
        assert_eq!(
            pgn_to_fen(
                "e4 b6 Nc3 Bb7 d4 e6 f4 Bb4 Bd3 Nf6 e5 Ne4 Bxe4 Bxe4 Nf3 Nc6 O-O \
                 Bxf3 Qxf3 Nxd4 Qd3 Nf5 g4 Bc5+ Kg2 Nh4+ Kg3 f5"
            ),
            "r2qk2r/p1pp2pp/1p2p3/2b1Pp2/5PPn/2NQ2K1/PPP4P/R1B2R2 w KQkq f6 0 1"
        );
        
        // https://lichess.org/training/2csxh
        assert_eq!(
            pgn_to_fen(
                "e4 d5 exd5 Qxd5 d4 Nf6 Nf3 Bg4 Nbd2 Nbd7 c4 Qh5 Be2 O-O-O O-O e5 \
                 h3 Bxh3 gxh3 Qxh3 Ng5 Qh4 Ndf3 Qg4+ Kh1 e4 Nh2 Qh4 Nxf7 Be7 \
                 Bg5 Qh3 Nxh8"
            ),
            "2kr3N/pppnb1pp/5n2/6B1/2PPp3/7q/PP2BP1N/R2Q1R1K b KQkq - 0 1"
        );
    }
}
