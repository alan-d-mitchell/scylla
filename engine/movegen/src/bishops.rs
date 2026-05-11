use types::{bitboard::Bitboard, square::Square};
use crate::magics::{BISHOP_MAGICS, BISHOP_SHIFTS};
use crate::magics;
use crate::masks::get_bishop_mask;
use std::sync::OnceLock;

// 5,248 is the exact number of entries needed for bishops.
static BISHOP_ATTACKS: OnceLock<[Bitboard; 5248]> = OnceLock::new();
static BISHOP_OFFSETS: OnceLock<[usize; 64]> = OnceLock::new();

#[inline(always)]
pub fn get_bishop_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    let relevant = blockers & get_bishop_mask(sq);
    let hash = (relevant.0.wrapping_mul(BISHOP_MAGICS[sq.0 as usize])) >> BISHOP_SHIFTS[sq.0 as usize];
    
    let offset = BISHOP_OFFSETS.get().unwrap()[sq.0 as usize];
    
    BISHOP_ATTACKS.get().unwrap()[offset + hash as usize]
}

pub fn classical_bishop_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let rank = (sq.0 / 8) as i8;
    let file = (sq.0 % 8) as i8;

    let directions = [(1, 1), (-1, 1), (1, -1), (-1, -1)]; // NE, NW, SE, SW

    for (df, dr) in directions {
        let mut r = rank;
        let mut f = file;
        loop {
            r += dr;
            f += df;
            if r < 0 || r > 7 || f < 0 || f > 7 { break; }
            
            let target = (r * 8 + f) as u8;
            attacks |= Bitboard(1u64 << target);
            
            if blockers.contains(target) { break; } 
        }
    }

    attacks
}

pub fn init_bishop_magic_table() {
    let mut attacks = [Bitboard::EMPTY; 5248];
    let mut offsets = [0; 64];
    let mut current_offset = 0;

    for sq in 0..64 {
        offsets[sq] = current_offset;
        let mask = get_bishop_mask(Square::new(sq as u8));
        let bits = mask.0.count_ones();
        
        // Use the Carry-Rippler to fill the table using our hardcoded magics
        let mut subset = 0u64;
        loop {
            // We use the constants we just verified
            let hash = (subset.wrapping_mul(BISHOP_MAGICS[sq])) >> BISHOP_SHIFTS[sq];
            let index = current_offset + hash as usize;

            let correct_attacks = classical_bishop_attacks(Square::new(sq as u8), Bitboard(subset));
            if attacks[index] != Bitboard::EMPTY && attacks[index] != correct_attacks {
                panic!(
                    "FATAL COLLISION on square {}!\nSubset: {}\nHash: {}\nMagic is invalid.", 
                    sq, subset, hash
                );
            }
            
            attacks[index] = correct_attacks;

            subset = subset.wrapping_sub(mask.0) & mask.0;
            if subset == 0 { break; }
        }

        // Advance the offset by 2^bits
        current_offset += 1 << bits;
    }

    BISHOP_ATTACKS.set(attacks).expect("init failed");
    BISHOP_OFFSETS.set(offsets).expect("init failed");
}
