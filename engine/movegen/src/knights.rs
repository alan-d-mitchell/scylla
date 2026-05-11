use types::{bitboard::Bitboard, square::Square};
use crate::constants::*;
use std::sync::OnceLock;

static KNIGHT_ATTACKS: OnceLock<[Bitboard; 64]> = OnceLock::new();


/// The public function to grab a knight's attacks.
/// It initializes the table the very first time it's called, and then 
/// just returns the pre-calculated bitboard instantly every time after.
#[inline(always)]
pub fn get_knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS.get_or_init(|| init_knight_attacks())[sq.0 as usize]
}

fn mask_knight_attacks(sq: Square) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let bb = Bitboard(1 << sq.0);

    // --- Rightward Jumps (Must not wrap to A or B files) ---
    // North-North-East (+17)
    attacks |= (bb << 17) & NOT_FILE_A;
    // East-North-East (+10)
    attacks |= (bb << 10) & NOT_FILE_AB;
    // East-South-East (-6)
    attacks |= (bb >> 6) & NOT_FILE_AB;
    // South-South-East (-15)
    attacks |= (bb >> 15) & NOT_FILE_A;

    // --- Leftward Jumps (Must not wrap to G or H files) ---
    // North-North-West (+15)
    attacks |= (bb << 15) & NOT_FILE_H;
    // West-North-West (+6)
    attacks |= (bb << 6) & NOT_FILE_GH;
    // West-South-West (-10)
    attacks |= (bb >> 10) & NOT_FILE_GH;
    // South-South-West (-17)
    attacks |= (bb >> 17) & NOT_FILE_H;

    attacks
}

pub fn init_knight_attacks() -> [Bitboard; 64] {
    let mut table = [Bitboard::EMPTY; 64];

    for sq in 0..64 {
        table[sq] = mask_knight_attacks(Square::new(sq as u8));
    }

    table
}
