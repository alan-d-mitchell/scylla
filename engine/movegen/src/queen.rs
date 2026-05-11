use types::{bitboard::Bitboard, square::Square};
use crate::rooks::get_rook_attacks;
use crate::bishops::get_bishop_attacks;

#[inline(always)]
pub fn get_queen_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    get_rook_attacks(sq, blockers) | get_bishop_attacks(sq, blockers)
}
