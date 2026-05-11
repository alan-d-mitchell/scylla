use crate::square::Square;
use std::fmt;

pub const FLAG_QUIET: u16 = 0;
pub const FLAG_DOUBLE_PUSH: u16 = 1;
pub const FLAG_KINGSIDE_CASTLE: u16 = 2;
pub const FLAG_QUEENSIDE_CASTLE: u16 = 3;
pub const FLAG_CAPTURE: u16 = 4;
pub const FLAG_EN_PASSANT: u16 = 5;

pub const FLAG_PROMOTION_KNIGHT: u16 = 8;
pub const FLAG_PROMOTION_BISHOP: u16 = 9;
pub const FLAG_PROMOTION_ROOK: u16 = 10;
pub const FLAG_PROMOTION_QUEEN: u16 = 11;

pub const FLAG_PROMOTION_KNIGHT_CAPTURE: u16 = 12;
pub const FLAG_PROMOTION_BISHOP_CAPTURE: u16 = 13;
pub const FLAG_PROMOTION_ROOK_CAPTURE: u16 = 14;
pub const FLAG_PROMOTION_QUEEN_CAPTURE: u16 = 15;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move(pub u16);

impl Move {

    const FROM_MASK: u16 = 0x3F;
    const TO_MASK: u16 = 0xFC0;
    const FLAG_MASK: u16 = 0xF000;

    pub fn new(from: Square, to: Square, flags: u16) -> Self {
        Self((from.0 as u16) | ((to.0 as u16) << 6) | (flags << 12))
    }

    pub fn from_sq(self) -> Square {
        Square((self.0 & Self::FROM_MASK) as u8)
    }

    pub fn to_sq(self) -> Square {
        Square(((self.0 & Self::TO_MASK) >> 6) as u8)
    }

    pub fn flags(self) -> u16 {
        (self.0 & Self::FLAG_MASK) >> 12
    }

    /// Returns true if the move involves any pawn promotion
    pub fn is_promo(self) -> bool {
        (self.flags() & 0b1000) != 0
    }

    /// Returns true if the move is a capture or en passant
    pub fn is_capture(self) -> bool {
        (self.flags() & 0b0100) != 0
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = self.from_sq().0;
        let to = self.to_sq().0;
        
        let from_file = (b'a' + (from % 8)) as char;
        let from_rank = (b'1' + (from / 8)) as char;
        let to_file = (b'a' + (to % 8)) as char;
        let to_rank = (b'1' + (to / 8)) as char;

        let promo_suffix = match self.flags() {
            FLAG_PROMOTION_KNIGHT | FLAG_PROMOTION_KNIGHT_CAPTURE => "n",
            FLAG_PROMOTION_BISHOP | FLAG_PROMOTION_BISHOP_CAPTURE => "b",
            FLAG_PROMOTION_ROOK   | FLAG_PROMOTION_ROOK_CAPTURE   => "r",
            FLAG_PROMOTION_QUEEN  | FLAG_PROMOTION_QUEEN_CAPTURE  => "q",
            _ => "",
        };
        
        write!(f, "{}{}{}{}{}", from_file, from_rank, to_file, to_rank, promo_suffix)
    }
}
