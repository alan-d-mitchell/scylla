use std::fmt;

use types::{
    piece::PieceType, color::Color, square::Square, bitboard::Bitboard,
    WK_CASTLE, WQ_CASTLE, BK_CASTLE, BQ_CASTLE,
    mov::{Move, FLAG_CAPTURE, FLAG_EN_PASSANT, FLAG_KINGSIDE_CASTLE, FLAG_QUEENSIDE_CASTLE, FLAG_DOUBLE_PUSH}
};

use crate::zobrist;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    pub pieces: [[Bitboard; 6]; 2],
    pub colors: [Bitboard; 2],
    pub side_to_move: Color,
    pub en_passant: Option<Square>,
    pub castling: u8,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub history: Vec<u64>,
}

#[derive(Copy, Clone, Debug)]
pub struct UndoInfo {
    pub castling: u8,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub captured_piece: Option<PieceType>,
    pub moved_piece: PieceType,
    pub zobrist_key: u64,
}

const CASTLING_PERMS: [u8; 64] = [
    0xD, 0xF, 0xF, 0xF, 0xC, 0xF, 0xF, 0xE, // Rank 1 (White)
    0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF,
    0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF,
    0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF,
    0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF,
    0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF,
    0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF, 0xF,
    0x7, 0xF, 0xF, 0xF, 0x3, 0xF, 0xF, 0xB, // Rank 8 (Black)
];

impl Board {

    pub fn empty() -> Self {
        Self {
            pieces: [[Bitboard::EMPTY; 6]; 2],
            colors: [Bitboard::EMPTY; 2],
            side_to_move: Color::White,
            en_passant: None,
            castling: 0,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_key: 0,
            history: Vec::new(),
        }
    }

    pub fn occupancy(&self) -> u64 {
        self.colors[0].0 | self.colors[1].0
    }

    pub fn set_piece(&mut self, color: Color, piece: PieceType, sq: Square) {
        let bit = Bitboard(1 << sq.0);

        self.pieces[color as usize][piece as usize] |= bit;
        self.colors[color as usize] |= bit;
    }

    /// Adds a piece to the bitboards
    pub fn add_piece(&mut self, piece: PieceType, color: Color, sq: Square) {
        let mask = 1u64 << sq.0;
        self.pieces[color as usize][piece as usize].0 |= mask;
        self.colors[color as usize].0 |= mask;

        self.zobrist_key ^= zobrist::KEYS.pieces[color as usize][piece as usize][sq.0 as usize];
    }

    /// Removes a piece from the bitboards
    pub fn remove_piece(&mut self, piece: PieceType, color: Color, sq: Square) {
        let mask = !(1u64 << sq.0);
        self.pieces[color as usize][piece as usize].0 &= mask;
        self.colors[color as usize].0 &= mask;

        self.zobrist_key ^= zobrist::KEYS.pieces[color as usize][piece as usize][sq.0 as usize];
    }

    /// Returns the square of the king for a given color
    pub fn get_king_square(&self, color: Color) -> Square {
        let bits = self.pieces[color as usize][PieceType::King as usize].0;
        Square::new(bits.trailing_zeros() as u8)
    }

    /// Find which piece (if any) is on a square
    pub fn piece_at(&self, sq: Square) -> Option<PieceType> {
        let mask = 1u64 << sq.0;

        for p in 0..6 {
            // Check white then black
            if (self.pieces[0][p].0 & mask) != 0 || (self.pieces[1][p].0 & mask) != 0 {
                return Some(match p {
                    0 => PieceType::Pawn, 1 => PieceType::Knight, 2 => PieceType::Bishop,
                    3 => PieceType::Rook, 4 => PieceType::Queen, 5 => PieceType::King,
                    _ => unreachable!(),
                });
            }
        }

        None
    }

    pub fn make_move(&mut self, m: Move) -> UndoInfo {
        let us = self.side_to_move;
        let from = m.from_sq();
        let to = m.to_sq();
        let flag = m.flags();
        let piece = self.piece_at(from).expect("no piece");

        let mut undo = UndoInfo {
            castling: self.castling,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            captured_piece: None,
            moved_piece: piece,
            zobrist_key: self.zobrist_key,
        };

        self.history.push(undo.zobrist_key);

        self.zobrist_key ^= zobrist::KEYS.castling[self.castling as usize];
        if let Some(ep_sq) = self.en_passant {
            self.zobrist_key ^= zobrist::KEYS.ep[(ep_sq.0 % 8) as usize];
        }

        if m.is_capture() || piece == PieceType::Pawn {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // --- 1. CAPTURE LOGIC ---
        if m.is_capture() {
            if flag == FLAG_EN_PASSANT {
                undo.captured_piece = Some(PieceType::Pawn);
                let ep_pawn_sq = if us == Color::White { to.0 - 8 } else { to.0 + 8 };
                self.remove_piece(PieceType::Pawn, us.flip(), Square::new(ep_pawn_sq));
            } else {
                let captured = self.piece_at(to).expect("no piece to capture");
                undo.captured_piece = Some(captured);
                self.remove_piece(captured, us.flip(), to);
            }
        }

        // --- 2. PHYSICAL MOVE & PROMOTION ---
        self.remove_piece(piece, us, from);

        let piece_to_add = if m.is_promo() {
            match flag & 0b0011 {
                0 => PieceType::Knight,
                1 => PieceType::Bishop,
                2 => PieceType::Rook,
                3 => PieceType::Queen,
                _ => unreachable!(),
            }
        } else { piece };

        self.add_piece(piece_to_add, us, to);

        // --- 3. REMAINING UPDATES (EP/Castling) ---
        self.en_passant = None;
        self.castling &= CASTLING_PERMS[from.0 as usize] & CASTLING_PERMS[to.0 as usize];

        if flag == FLAG_KINGSIDE_CASTLE {
            let (r_from, r_to) = if us == Color::White { (7, 5) } else { (63, 61) };
            self.remove_piece(PieceType::Rook, us, Square::new(r_from));
            self.add_piece(PieceType::Rook, us, Square::new(r_to));
        } else if flag == FLAG_QUEENSIDE_CASTLE {
            let (r_from, r_to) = if us == Color::White { (0, 3) } else { (56, 59) };
            self.remove_piece(PieceType::Rook, us, Square::new(r_from));
            self.add_piece(PieceType::Rook, us, Square::new(r_to));
        } else if flag == FLAG_DOUBLE_PUSH {
            let ep_sq = if us == Color::White { from.0 + 8 } else { from.0 - 8 };
            self.en_passant = Some(Square::new(ep_sq));
        }

        self.zobrist_key ^= zobrist::KEYS.castling[self.castling as usize];
        if let Some(ep_sq) = self.en_passant {
            self.zobrist_key ^= zobrist::KEYS.ep[(ep_sq.0 % 8) as usize];
        }

        self.zobrist_key ^= zobrist::KEYS.side;
        self.side_to_move = us.flip();

        undo
    }

    pub fn unmake_move(&mut self, m: Move, undo: UndoInfo) {
        let us = self.side_to_move.flip();
        self.side_to_move = us;

        let from = m.from_sq();
        let to = m.to_sq();
        let flag = m.flags();

        self.castling = undo.castling;
        self.en_passant = undo.en_passant;
        self.halfmove_clock = undo.halfmove_clock;

        if flag == FLAG_KINGSIDE_CASTLE {
            let (r_from, r_to) = if us == Color::White { (7, 5) } else { (63, 61) };
            self.remove_piece(PieceType::Rook, us, Square::new(r_to));
            self.add_piece(PieceType::Rook, us, Square::new(r_from));
        } else if flag == FLAG_QUEENSIDE_CASTLE {
            let (r_from, r_to) = if us == Color::White { (0, 3) } else { (56, 59) };
            self.remove_piece(PieceType::Rook, us, Square::new(r_to));
            self.add_piece(PieceType::Rook, us, Square::new(r_from));
        }

        let moved_piece = undo.moved_piece;
        
        let piece_to_remove = if m.is_promo() {
            match flag & 0b0011 {
                0 => PieceType::Knight, 1 => PieceType::Bishop, 2 => PieceType::Rook, 3 => PieceType::Queen,
                _ => unreachable!(),
            }
        } else { moved_piece };

        self.remove_piece(piece_to_remove, us, to);
        self.add_piece(moved_piece, us, from);

        if let Some(captured) = undo.captured_piece {
            if flag == FLAG_EN_PASSANT {
                let ep_sq = if us == Color::White { to.0 - 8 } else { to.0 + 8 };
                self.add_piece(captured, us.flip(), Square::new(ep_sq));
            } else {
                self.add_piece(captured, us.flip(), to);
            }
        }

        self.history.pop();
        self.zobrist_key = undo.zobrist_key;
    }

    /// Calculates the Zobrist key from scratch. 
    /// ONLY call this once after setting up the board from a FEN!
    pub fn calculate_zobrist(&mut self) {
        let mut key = 0;

        // 1. Hash all the pieces currently on the board
        for color in 0..2 {
            for pt in 0..6 {
                let mut bitboard = self.pieces[color][pt].0;
                while bitboard != 0 {
                    let sq = bitboard.trailing_zeros() as usize; // Get the square index
                    key ^= zobrist::KEYS.pieces[color][pt][sq];
                    
                    bitboard &= bitboard - 1; // Clear the least significant bit
                }
            }
        }

        // 2. Hash Castling Rights
        key ^= zobrist::KEYS.castling[self.castling as usize];

        // 3. Hash En Passant file
        if let Some(ep_sq) = self.en_passant {
            key ^= zobrist::KEYS.ep[(ep_sq.0 % 8) as usize];
        }

        // 4. Hash Side to Move (Standard practice: only XOR if it's Black's turn)
        if self.side_to_move == Color::Black {
            key ^= zobrist::KEYS.side;
        }

        self.zobrist_key = key;
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n  +---+---+---+---+---+---+---+---+")?;
        
        // Ranks go from 8 down to 1
        for rank in (0..8).rev() {
            write!(f, "{} |", rank + 1)?;
            
            for file in 0..8 {
                let sq_idx = rank * 8 + file;
                let mask = 1u64 << sq_idx;
                let mut piece_char = '.';

                // Check White pieces (Uppercase)
                for pt_idx in 0..6 {
                    if (self.pieces[Color::White as usize][pt_idx].0 & mask) != 0 {
                        piece_char = match pt_idx {
                            0 => 'P', 1 => 'N', 2 => 'B', 
                            3 => 'R', 4 => 'Q', 5 => 'K',
                            _ => '.',
                        };
                    }
                }

                // Check Black pieces (Lowercase) if no white piece was found
                if piece_char == '.' {
                    for pt_idx in 0..6 {
                        if (self.pieces[Color::Black as usize][pt_idx].0 & mask) != 0 {
                            piece_char = match pt_idx {
                                0 => 'p', 1 => 'n', 2 => 'b', 
                                3 => 'r', 4 => 'q', 5 => 'k',
                                _ => '.',
                            };
                        }
                    }
                }
                write!(f, " {} |", piece_char)?;
            }
            writeln!(f, "\n  +---+---+---+---+---+---+---+---+")?;
        }
        
        // File labels
        writeln!(f, "    a   b   c   d   e   f   g   h")?;

        // Metadata section
        writeln!(f, "\nTurn:       {:?}", self.side_to_move)?;
        
        // Castling Rights string (FEN style: KQkq)
        let mut castle_str = String::new();
        if self.castling & WK_CASTLE != 0 { castle_str.push('K'); }
        if self.castling & WQ_CASTLE != 0 { castle_str.push('Q'); }
        if self.castling & BK_CASTLE != 0 { castle_str.push('k'); }
        if self.castling & BQ_CASTLE != 0 { castle_str.push('q'); }
        if castle_str.is_empty() { castle_str = "-".to_string(); }
        
        writeln!(f, "Castling:   {}", castle_str)?;
        
        let ep_str = match self.en_passant {
            Some(sq) => format!("{:?}", sq), // This assumes Square has Debug/Display
            None => "-".to_string(),
        };
        writeln!(f, "En Passant: {}", ep_str)?;
        writeln!(f, "Fullmove:   {}", self.fullmove_number)?;

        Ok(())
    }
}
