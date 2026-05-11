use board::Board;
use crate::{constants::*, pawn, knights, bishops, rooks, queen, king};
use types::{
    bitboard::Bitboard, color::Color, piece::PieceType, square::Square, mov::*, 
    BK_CASTLE, BQ_CASTLE, WK_CASTLE, WQ_CASTLE
};
use crate::moves::MoveList;

pub fn generate_pseudo_legal_moves(board: &Board) -> MoveList {
    let mut list = MoveList::new();
    let us = board.side_to_move;
    
    // The squares we are NOT allowed to move to (our own pieces)
    let friendly_pieces = board.colors[us as usize];
    let all_pieces = board.colors[Color::White as usize] | board.colors[Color::Black as usize];
    let enemy_pieces = board.colors[us.flip() as usize];

    generate_pawn_moves(board, &mut list, us, all_pieces, enemy_pieces);
    generate_knight_moves(board, &mut list, us, friendly_pieces);
    generate_slider_moves(board, &mut list, us, friendly_pieces, all_pieces);
    generate_king_moves(board, &mut list, us, friendly_pieces, all_pieces);

    list
}

fn generate_knight_moves(board: &Board, list: &mut MoveList, us: Color, friendly_pieces: Bitboard) {
    let mut knights = board.pieces[us as usize][PieceType::Knight as usize];
    let enemy_pieces = board.colors[us.flip() as usize];

    while let Some(from_sq) = knights.pop_lsb() {
        let attacks = knights::get_knight_attacks(from_sq);
        let mut valid_moves = attacks & !friendly_pieces;

        while let Some(to_sq) = valid_moves.pop_lsb() {
            // Correctly identify captures
            let flag = if enemy_pieces.contains(to_sq.0) { FLAG_CAPTURE } else { FLAG_QUIET };
            list.push(Move::new(from_sq, to_sq, flag));
        }
    }
}

fn generate_king_moves(board: &Board, list: &mut MoveList, us: Color, friendly_pieces: Bitboard, all_pieces: Bitboard) {
    let mut kings = board.pieces[us as usize][PieceType::King as usize];
    let enemy_color = us.flip();
    
    while let Some(from_sq) = kings.pop_lsb() {
        let attacks = king::get_king_attacks(from_sq);
        let mut valid_moves = attacks & !friendly_pieces;
        
        while let Some(to_sq) = valid_moves.pop_lsb() {
            let flag = if board.colors[enemy_color as usize].contains(to_sq.0) { FLAG_CAPTURE } else { FLAG_QUIET };
            list.push(Move::new(from_sq, to_sq, flag));
        }

        // --- Castling with Path Safety ---
        let empty_squares = !all_pieces;

        if us == Color::White {
            // Kingside (e1 to g1)
            if (board.castling & WK_CASTLE) != 0 && (WHITE_KS_EMPTY & empty_squares) == WHITE_KS_EMPTY {
                if !is_square_attacked(board, Square::new(4), enemy_color) && // e1
                   !is_square_attacked(board, Square::new(5), enemy_color) && // f1
                   !is_square_attacked(board, Square::new(6), enemy_color)    // g1
                {
                    list.push(Move::new(from_sq, Square::new(6), FLAG_KINGSIDE_CASTLE));
                }
            }
            // Queenside (e1 to c1)
            if (board.castling & WQ_CASTLE) != 0 && (WHITE_QS_EMPTY & empty_squares) == WHITE_QS_EMPTY {
                if !is_square_attacked(board, Square::new(4), enemy_color) && // e1
                   !is_square_attacked(board, Square::new(3), enemy_color) && // d1
                   !is_square_attacked(board, Square::new(2), enemy_color)    // c1
                {
                    list.push(Move::new(from_sq, Square::new(2), FLAG_QUEENSIDE_CASTLE));
                }
            }
        } else {
            // Black Kingside (e8 to g8)
            if (board.castling & BK_CASTLE) != 0 && (BLACK_KS_EMPTY & empty_squares) == BLACK_KS_EMPTY {
                if !is_square_attacked(board, Square::new(60), enemy_color) && // e8
                   !is_square_attacked(board, Square::new(61), enemy_color) && // f8
                   !is_square_attacked(board, Square::new(62), enemy_color)    // g8
                {
                    list.push(Move::new(from_sq, Square::new(62), FLAG_KINGSIDE_CASTLE));
                }
            }
            // Black Queenside (e8 to c8)
            if (board.castling & BQ_CASTLE) != 0 && (BLACK_QS_EMPTY & empty_squares) == BLACK_QS_EMPTY {
                if !is_square_attacked(board, Square::new(60), enemy_color) && // e8
                   !is_square_attacked(board, Square::new(59), enemy_color) && // d8
                   !is_square_attacked(board, Square::new(58), enemy_color)    // c8
                {
                    list.push(Move::new(from_sq, Square::new(58), FLAG_QUEENSIDE_CASTLE));
                }
            }
        }
    }
}

fn generate_slider_moves(board: &Board, list: &mut MoveList, us: Color, friendly_pieces: Bitboard, all_pieces: Bitboard) {
    let enemy_pieces = board.colors[us.flip() as usize]; // Grab enemy pieces for capture checks

    // Bishops
    let mut bishops = board.pieces[us as usize][PieceType::Bishop as usize];
    while let Some(from_sq) = bishops.pop_lsb() {
        let attacks = bishops::get_bishop_attacks(from_sq, all_pieces);
        let mut valid_moves = attacks & !friendly_pieces;
        while let Some(to_sq) = valid_moves.pop_lsb() {
            let flag = if enemy_pieces.contains(to_sq.0) { FLAG_CAPTURE } else { FLAG_QUIET };
            list.push(Move::new(from_sq, to_sq, flag));
        }
    }

    // Rooks
    let mut rooks = board.pieces[us as usize][PieceType::Rook as usize];
    while let Some(from_sq) = rooks.pop_lsb() {
        let attacks = rooks::get_rook_attacks(from_sq, all_pieces);
        let mut valid_moves = attacks & !friendly_pieces;
        while let Some(to_sq) = valid_moves.pop_lsb() {
            let flag = if enemy_pieces.contains(to_sq.0) { FLAG_CAPTURE } else { FLAG_QUIET };
            list.push(Move::new(from_sq, to_sq, flag));
        }
    }

    // Queens
    let mut queens = board.pieces[us as usize][PieceType::Queen as usize];
    while let Some(from_sq) = queens.pop_lsb() {
        let attacks = queen::get_queen_attacks(from_sq, all_pieces);
        let mut valid_moves = attacks & !friendly_pieces;
        while let Some(to_sq) = valid_moves.pop_lsb() {
            let flag = if enemy_pieces.contains(to_sq.0) { FLAG_CAPTURE } else { FLAG_QUIET };
            list.push(Move::new(from_sq, to_sq, flag));
        }
    }
}

fn generate_pawn_moves(board: &Board, list: &mut MoveList, us: Color, all_pieces: Bitboard, enemy_pieces: Bitboard) {
    let pawns = board.pieces[us as usize][PieceType::Pawn as usize];
    let empty_squares = !all_pieces;
    let them = us.flip();

    if us == Color::White {
        // --- 1. Single Pushes & Promotions ---
        let mut single_pushes = (pawns << 8) & empty_squares;
        while let Some(to_sq) = single_pushes.pop_lsb() {
            let from_sq = Square::new(to_sq.0 - 8);
            if to_sq.0 >= 56 { // Rank 8
                push_promotions(list, from_sq, to_sq, false);
            } else {
                list.push(Move::new(from_sq, to_sq, FLAG_QUIET));
            }
        }

        // --- 2. Double Pushes ---
        let mut double_pushes = (((pawns & RANK_2) << 8) & empty_squares) << 8 & empty_squares;
        while let Some(to_sq) = double_pushes.pop_lsb() {
            list.push(Move::new(Square::new(to_sq.0 - 16), to_sq, FLAG_DOUBLE_PUSH));
        }
    } else {
        // --- Black Pushes & Promotions ---
        let mut single_pushes = (pawns >> 8) & empty_squares;
        while let Some(to_sq) = single_pushes.pop_lsb() {
            let from_sq = Square::new(to_sq.0 + 8);
            if to_sq.0 <= 7 { // Rank 1
                push_promotions(list, from_sq, to_sq, false);
            } else {
                list.push(Move::new(from_sq, to_sq, FLAG_QUIET));
            }
        }

        let mut double_pushes = (((pawns & RANK_7) >> 8) & empty_squares) >> 8 & empty_squares;
        while let Some(to_sq) = double_pushes.pop_lsb() {
            list.push(Move::new(Square::new(to_sq.0 + 16), to_sq, FLAG_DOUBLE_PUSH));
        }
    }

    // --- 3. Captures & Capture-Promotions ---
    let mut capturing_pawns = pawns;
    while let Some(from_sq) = capturing_pawns.pop_lsb() {
        let attacks = pawn::get_pawn_attacks(us, from_sq);
        let mut valid_captures = attacks & enemy_pieces;
        
        while let Some(to_sq) = valid_captures.pop_lsb() {
            if (us == Color::White && to_sq.0 >= 56) || (us == Color::Black && to_sq.0 <= 7) {
                push_promotions(list, from_sq, to_sq, true);
            } else {
                list.push(Move::new(from_sq, to_sq, FLAG_CAPTURE));
            }
        }
    }

    // --- 4. En Passant --- (No promotions possible here)
    if let Some(ep_sq) = board.en_passant {
        let mut ep_attackers = pawn::get_pawn_attacks(them, ep_sq) & pawns;
        while let Some(from_sq) = ep_attackers.pop_lsb() {
            list.push(Move::new(from_sq, ep_sq, FLAG_EN_PASSANT));
        }
    }
}

pub fn is_square_attacked(board: &Board, sq: Square, attacker_color: Color) -> bool {
    let all_pieces = board.colors[Color::White as usize] | board.colors[Color::Black as usize];
    let attackers = &board.pieces[attacker_color as usize];

    // 1. Attacked by Pawns?
    // We check if a pawn of OUR color on THIS square would hit an ENEMY pawn
    if (pawn::get_pawn_attacks(attacker_color.flip(), sq) & attackers[PieceType::Pawn as usize]).0 != 0 {
        return true;
    }

    // 2. Attacked by Knights?
    if (knights::get_knight_attacks(sq) & attackers[PieceType::Knight as usize]).0 != 0 {
        return true;
    }

    // 3. Attacked by King?
    if (king::get_king_attacks(sq) & attackers[PieceType::King as usize]).0 != 0 {
        return true;
    }

    // 4. Attacked by Sliders? (Bishops/Queens)
    let bishop_attacks = bishops::get_bishop_attacks(sq, all_pieces);
    let bishop_type_attackers = attackers[PieceType::Bishop as usize] | attackers[PieceType::Queen as usize];
    if (bishop_attacks & bishop_type_attackers).0 != 0 {
        return true;
    }

    // 5. Attacked by Sliders? (Rooks/Queens)
    let rook_attacks = rooks::get_rook_attacks(sq, all_pieces);
    let rook_type_attackers = attackers[PieceType::Rook as usize] | attackers[PieceType::Queen as usize];
    if (rook_attacks & rook_type_attackers).0 != 0 {
        return true;
    }

    false
}

fn push_promotions(list: &mut MoveList, from: Square, to: Square, capture: bool) {
    let base_flag = if capture { FLAG_CAPTURE } else { 0 }; // We'll combine flags
    list.push(Move::new(from, to, base_flag | FLAG_PROMOTION_QUEEN));
    list.push(Move::new(from, to, base_flag | FLAG_PROMOTION_ROOK));
    list.push(Move::new(from, to, base_flag | FLAG_PROMOTION_BISHOP));
    list.push(Move::new(from, to, base_flag | FLAG_PROMOTION_KNIGHT));
}
