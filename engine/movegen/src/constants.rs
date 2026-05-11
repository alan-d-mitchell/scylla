use types::bitboard::Bitboard;

pub const FILE_A: Bitboard = Bitboard(0x0101010101010101);
pub const FILE_B: Bitboard = Bitboard(0x0202020202020202);
pub const FILE_G: Bitboard = Bitboard(0x4040404040404040);
pub const FILE_H: Bitboard = Bitboard(0x8080808080808080);

pub const NOT_FILE_A: Bitboard = Bitboard(!FILE_A.0);
pub const NOT_FILE_AB: Bitboard = Bitboard(!(FILE_A.0 | FILE_B.0));
pub const NOT_FILE_H: Bitboard = Bitboard(!FILE_H.0);
pub const NOT_FILE_GH: Bitboard = Bitboard(!(FILE_G.0 | FILE_H.0));

pub const RANK_3: Bitboard = Bitboard(0x0000000000FF0000); // Where White's single push lands
pub const RANK_6: Bitboard = Bitboard(0x0000FF0000000000); // Where Black's single push lands
pub const RANK_2: Bitboard = Bitboard(0x000000000000FF00);
pub const RANK_7: Bitboard = Bitboard(0x00FF000000000000);

// Castling Empty Square Masks (The squares between King and Rook)
pub const WHITE_KS_EMPTY: Bitboard = Bitboard(0x0000000000000060); // f1, g1
pub const WHITE_QS_EMPTY: Bitboard = Bitboard(0x000000000000000E); // b1, c1, d1
pub const BLACK_KS_EMPTY: Bitboard = Bitboard(0x6000000000000000); // f8, g8
pub const BLACK_QS_EMPTY: Bitboard = Bitboard(0x0E00000000000000); // b8, c8, d8
