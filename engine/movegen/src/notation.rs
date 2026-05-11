use board::Board;
use types::{mov::Move, piece::PieceType};
use crate::generator;

pub fn move_to_san(board: &mut Board, m: Move) -> String {
    let mut san = String::new();
    let us = board.side_to_move;
    let from = m.from_sq();
    let to = m.to_sq();
    
    let pt = board.piece_at(from).unwrap_or(PieceType::Pawn);
    let is_capture = board.piece_at(to).is_some() || (pt == PieceType::Pawn && from.0 % 8 != to.0 % 8);

    // 1. Castling
    if pt == PieceType::King && (from.0 as i8 - to.0 as i8).abs() == 2 {
        if to.0 % 8 == 6 { san.push_str("O-O"); } 
        else { san.push_str("O-O-O"); }
    } else {
        // 2. Piece Type
        if pt != PieceType::Pawn {
            let char = match pt {
                PieceType::Knight => 'N', PieceType::Bishop => 'B',
                PieceType::Rook => 'R', PieceType::Queen => 'Q',
                PieceType::King => 'K', _ => unreachable!(),
            };
            san.push(char);

            // 3. Disambiguation
            let moves = generator::generate_pseudo_legal_moves(board);
            let mut possible_froms = Vec::new();

            for alt_m in moves.as_slice() {
                if alt_m.to_sq() == to && alt_m.from_sq() != from {
                    if board.piece_at(alt_m.from_sq()) == Some(pt) {
                        let undo = board.make_move(*alt_m);
                        let king_sq = board.get_king_square(us);
                        if !generator::is_square_attacked(board, king_sq, us.flip()) {
                            possible_froms.push(alt_m.from_sq());
                        }
                        board.unmake_move(*alt_m, undo);
                    }
                }
            }

            if !possible_froms.is_empty() {
                let mut same_file = false;
                let mut same_rank = false;
                for alt_from in possible_froms {
                    if alt_from.0 % 8 == from.0 % 8 { same_file = true; }
                    if alt_from.0 / 8 == from.0 / 8 { same_rank = true; }
                }
                if !same_file { san.push((b'a' + (from.0 % 8) as u8) as char); } 
                else if !same_rank { san.push((b'1' + (from.0 / 8) as u8) as char); } 
                else {
                    san.push((b'a' + (from.0 % 8) as u8) as char);
                    san.push((b'1' + (from.0 / 8) as u8) as char);
                }
            }
        } else if is_capture {
            san.push((b'a' + (from.0 % 8) as u8) as char);
        }

        // 4. Capture 'x'
        if is_capture { san.push('x'); }

        // 5. Destination Square
        san.push((b'a' + (to.0 % 8) as u8) as char);
        san.push((b'1' + (to.0 / 8) as u8) as char);

        // 6. Promotions
        if m.is_promo() {
            san.push('=');
            let promo_char = match m.flags() & 0b0011 {
                0 => 'N', 1 => 'B', 2 => 'R', 3 => 'Q', _ => unreachable!(),
            };
            san.push(promo_char);
        }
    }

    // 7. Check (+) and Checkmate (#)
    let undo = board.make_move(m);
    let enemy = us.flip();
    let enemy_king_sq = board.get_king_square(enemy);
    
    if generator::is_square_attacked(board, enemy_king_sq, us) {
        let enemy_moves = generator::generate_pseudo_legal_moves(board);
        let mut has_legal = false;
        
        for em in enemy_moves.as_slice() {
            let e_undo = board.make_move(*em);
            let e_king_sq = board.get_king_square(enemy);
            if !generator::is_square_attacked(board, e_king_sq, us) {
                has_legal = true;
                board.unmake_move(*em, e_undo);
                break;
            }
            board.unmake_move(*em, e_undo);
        }

        if has_legal { san.push('+'); } 
        else { san.push('#'); }
    }
    
    board.unmake_move(m, undo);
    san
}
