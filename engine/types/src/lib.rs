pub mod color;
pub mod mov;
pub mod piece;
pub mod square;
pub mod bitboard;

pub const WK_CASTLE: u8 = 1; // 0001
pub const WQ_CASTLE: u8 = 2; // 0010
pub const BK_CASTLE: u8 = 4; // 0100
pub const BQ_CASTLE: u8 = 8; // 1000

pub const WHITE_CASTLING: u8 = WK_CASTLE | WQ_CASTLE;
pub const BLACK_CASTLING: u8 = BK_CASTLE | BQ_CASTLE;
