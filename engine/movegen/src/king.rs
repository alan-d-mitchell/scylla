use types::{bitboard::Bitboard, square::Square};
use crate::constants::*;
use std::sync::OnceLock;

static KING_ATTACKS: OnceLock<[Bitboard; 64]> = OnceLock::new();

#[inline(always)]
pub fn get_king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS.get_or_init(|| init_king_attacks())[sq.0 as usize]
}

fn mask_king_attacks(sq: Square) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let bb = Bitboard(1 << sq.0);

    // North, South, East, West
    attacks |= bb << 8;                                // North
    attacks |= bb >> 8;                                // South
    attacks |= (bb << 1) & NOT_FILE_A;                 // East
    attacks |= (bb >> 1) & NOT_FILE_H;                 // West

    // Diagonals
    attacks |= (bb << 9) & NOT_FILE_A;                 // North-East
    attacks |= (bb << 7) & NOT_FILE_H;                 // North-West
    attacks |= (bb >> 7) & NOT_FILE_A;                 // South-East
    attacks |= (bb >> 9) & NOT_FILE_H;                 // South-West

    attacks
}

fn init_king_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard::EMPTY; 64];

    for sq in 0..64 {
        table[sq] = mask_king_attacks(Square::new(sq as u8));
    }

    table
}
