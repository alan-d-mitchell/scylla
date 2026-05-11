use std::ops::{
    BitAnd, BitAndAssign, BitOr, 
    BitOrAssign, BitXor, BitXorAssign,
    Not, Shl, ShlAssign, 
    Shr, ShrAssign
};

use std::fmt;
use crate::square::Square;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Bitboard(pub u64);

impl Bitboard {

    pub const EMPTY: Bitboard = Bitboard(0);
    pub const UNIVERSE: Bitboard = Bitboard(!0);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn contains(self, sq: u8) -> bool {
        (self.0 & (1 << sq)) != 0
    }

    /// Returns the square of the Least Significant Bit (LSB) and clears it from the bitboard.
    /// This is the fastest way to iterate over occupied squares.
    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }

        // trailing_zeros() perfectly maps to our square indices (0 for A1, 63 for H8)
        let sq = self.0.trailing_zeros() as u8;
        
        // Clear the lowest set bit
        self.0 &= self.0 - 1; 
        
        Some(Square::new(sq))
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl Shr<usize> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl BitAndAssign for Bitboard {

    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOrAssign for Bitboard {

    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXorAssign for Bitboard {

    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl ShlAssign<usize> for Bitboard {

    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs;
    }
}

impl ShrAssign<usize> for Bitboard {

    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs;
    }
}


impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..=7).rev() {
            write!(f, "{} |", rank + 1)?;
            for file in 0..=7 {
                let sq = Square::new(rank * 8 + file);
                // If the bit at this square is 1, print an 'X', else a dot
                let symbol = if self.contains(sq.0) { " X" } else { " ." };
                write!(f, "{}", symbol)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  +----------------")?;
        writeln!(f, "    a b c d e f g h")?;
        Ok(())
    }
}
