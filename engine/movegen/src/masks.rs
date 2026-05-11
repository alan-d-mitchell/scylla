use types::{bitboard::Bitboard, square::Square};
use crate::constants::*;
use std::sync::OnceLock;

static ROOK_MASKS: OnceLock<[Bitboard; 64]> = OnceLock::new();
static BISHOP_MASKS: OnceLock<[Bitboard; 64]> = OnceLock::new();

pub fn get_rook_mask(sq: Square) -> Bitboard {
    ROOK_MASKS.get_or_init(|| init_rook_masks())[sq.0 as usize]
}

pub fn get_bishop_mask(sq: Square) -> Bitboard {
    BISHOP_MASKS.get_or_init(|| init_bishop_masks())[sq.0 as usize]
}

fn init_rook_masks() -> [Bitboard; 64] {
    let mut masks = [Bitboard::EMPTY; 64];

    for sq in 0..64 {
        let mut mask = Bitboard::EMPTY;
        let r = (sq / 8) as i8;
        let f = (sq % 8) as i8;

        // North (stop at rank 6, index 6, because rank 8 is the edge)
        for i in (r + 1)..7 { mask |= Bitboard(1u64 << (i * 8 + f)); }
        // South (stop at rank 2, index 1)
        for i in (1..r).rev() { mask |= Bitboard(1u64 << (i * 8 + f)); }
        // East (stop at file g, index 6)
        for j in (f + 1)..7 { mask |= Bitboard(1u64 << (r * 8 + j)); }
        // West (stop at file b, index 1)
        for j in (1..f).rev() { mask |= Bitboard(1u64 << (r * 8 + j)); }

        masks[sq] = mask;
    }

    masks
}

fn init_bishop_masks() -> [Bitboard; 64] {
    let mut masks = [Bitboard::EMPTY; 64];

    for sq in 0..64 {
        let mut mask = 0u64;
        let r = (sq / 8) as i8;
        let f = (sq % 8) as i8;

        let directions: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        for (dr, df) in directions {
            let mut cur_r = r + dr;
            let mut cur_f = f + df;

            // Keep going until we hit the square BEFORE the edge
            while cur_r > 0 && cur_r < 7 && cur_f > 0 && cur_f < 7 {
                mask |= 1u64 << (cur_r * 8 + cur_f);
                cur_r += dr;
                cur_f += df;
            }
        }

        masks[sq] = Bitboard(mask);
    }

    masks
}
