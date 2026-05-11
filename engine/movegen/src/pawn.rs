use types::{bitboard::Bitboard, color::Color, square::Square};
use crate::constants::*;
use std::sync::OnceLock;

static PAWN_ATTACKS: OnceLock<[[Bitboard; 64]; 2]> = OnceLock::new();

#[inline(always)]
pub fn get_pawn_attacks(color: Color, sq: Square) -> Bitboard {
    PAWN_ATTACKS.get_or_init(|| init_pawn_attacks())[color as usize][sq.0 as usize]
}

fn mask_pawn_attacks(color: Color, sq: Square) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let bb = Bitboard(1 << sq.0);

    if color == Color::White {
        // White captures North-West (+7) and North-East (+9)
        attacks |= (bb << 7) & NOT_FILE_H;
        attacks |= (bb << 9) & NOT_FILE_A;
    } else {
        // Black captures South-East (-7) and South-West (-9)
        // Note: From Black's perspective looking down the board, 
        // -7 is SE and -9 is SW in Little-Endian Rank-File mapping.
        attacks |= (bb >> 7) & NOT_FILE_A;
        attacks |= (bb >> 9) & NOT_FILE_H;
    }

    attacks
}

fn init_pawn_attacks() -> [[Bitboard; 64]; 2] {
    let mut table = [[Bitboard::EMPTY; 64]; 2];

    for sq in 0..64 {
        table[Color::White as usize][sq] = mask_pawn_attacks(Color::White, Square::new(sq as u8));
        table[Color::Black as usize][sq] = mask_pawn_attacks(Color::Black, Square::new(sq as u8));
    }

    table
}
