pub mod pst; // Pull in our new tables

use board::Board;
use types::{piece::PieceType, color::Color};
use pst::*;

const PAWN_VAL: i32 = 100;
const KNIGHT_VAL: i32 = 300;
const BISHOP_VAL: i32 = 300;
const ROOK_VAL: i32 = 500;
const QUEEN_VAL: i32 = 900;

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;

    for color_idx in 0..2 {
        let is_white = color_idx == Color::White as usize;
        let color_multiplier = if is_white { 1 } else { -1 };

        for pt_idx in 0..6 {
            let pt = match pt_idx {
                0 => PieceType::Pawn, 1 => PieceType::Knight, 2 => PieceType::Bishop,
                3 => PieceType::Rook, 4 => PieceType::Queen, 5 => PieceType::King,
                _ => unreachable!(),
            };

            let material_val = match pt {
                PieceType::Pawn => PAWN_VAL, PieceType::Knight => KNIGHT_VAL,
                PieceType::Bishop => BISHOP_VAL, PieceType::Rook => ROOK_VAL,
                PieceType::Queen => QUEEN_VAL, PieceType::King => 0, // King has no material value
            };

            // Get the table for this piece type
            let table = match pt {
                PieceType::Pawn => PAWN_PST, PieceType::Knight => KNIGHT_PST,
                PieceType::Bishop => BISHOP_PST, PieceType::Rook => ROOK_PST,
                PieceType::Queen => QUEEN_PST, PieceType::King => KING_PST,
            };

            // Loop through every piece of this type and color
            let mut bitboard = board.pieces[color_idx][pt_idx].0;
            while bitboard != 0 {
                let sq = bitboard.trailing_zeros() as usize;

                // IMPORTANT: The tables are designed for White (Rank 1 is bottom).
                // If the piece is Black, we have to XOR the square by 56 to flip the board vertically!
                // (e.g., Square 63 (H8) becomes Square 7 (H1)).
                let pst_sq = if is_white { sq } else { sq ^ 56 };

                // Add Material Value + Positional Table Value
                let piece_score = material_val + table[pst_sq];
                score += piece_score * color_multiplier;

                bitboard &= bitboard - 1; // Clear the lowest set bit to move to the next piece
            }
        }
    }

    let white_material = get_material_count(board, Color::White);
    let black_material = get_material_count(board, Color::Black);
    
    let white_king = board.pieces[0][PieceType::King as usize].0.trailing_zeros() as usize;
    let black_king = board.pieces[1][PieceType::King as usize].0.trailing_zeros() as usize;

    if white_material > black_material {
        score += mop_up_eval(white_king, black_king, white_material - black_material);
    } else {
        score -= mop_up_eval(black_king, white_king, black_material - white_material);
    }

    // Negamax return
    if board.side_to_move == Color::Black { -score } else { score }
}

fn mop_up_eval(us_king: usize, them_king: usize, material_diff: i32) -> i32 {
    if material_diff < 200 { return 0; } // Only mop up if we are way ahead

    let mut score = 0;
    
    // 1. Push the enemy king to the edges (Center Manhatten Distance)
    let them_king_rank = them_king / 8;
    let them_king_file = them_king % 8;
    
    let dist_to_center_rank = (them_king_rank as i32 - 3).abs().max((them_king_rank as i32 - 4).abs());
    let dist_to_center_file = (them_king_file as i32 - 3).abs().max((them_king_file as i32 - 4).abs());
    score += (dist_to_center_rank + dist_to_center_file) * 10;

    // 2. Bring our king closer to the enemy king
    let us_king_rank = us_king / 8;
    let us_king_file = us_king % 8;
    
    let king_dist_rank = (us_king_rank as i32 - them_king_rank as i32).abs();
    let king_dist_file = (us_king_file as i32 - them_king_file as i32).abs();
    let king_distance = king_dist_rank + king_dist_file;
    
    score += (14 - king_distance) * 5;

    score
}

fn get_material_count(board: &Board, color: Color) -> i32 {
    let c = color as usize;
    let mut total = 0;
    
    total += board.pieces[c][PieceType::Pawn as usize].0.count_ones() as i32 * PAWN_VAL;
    total += board.pieces[c][PieceType::Knight as usize].0.count_ones() as i32 * KNIGHT_VAL;
    total += board.pieces[c][PieceType::Bishop as usize].0.count_ones() as i32 * BISHOP_VAL;
    total += board.pieces[c][PieceType::Rook as usize].0.count_ones() as i32 * ROOK_VAL;
    total += board.pieces[c][PieceType::Queen as usize].0.count_ones() as i32 * QUEEN_VAL;
    
    total
}
