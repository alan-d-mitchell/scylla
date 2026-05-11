use crate::board::Board;
use std::str::FromStr;
use types::{color::Color, piece::PieceType, square::Square};

impl FromStr for Board {
    type Err = String;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let mut board = Board::empty();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err("invalid FEN: expected exactly 6 parts".to_string());
        }

        let mut rank = 7i8;
        let mut file = 0i8;

        for c in parts[0].chars() {
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }

                '1'..='8' => {
                    let empty_squares = c.to_digit(10).unwrap() as i8;
                    file += empty_squares;
                }

                'p' | 'n' | 'b' | 'r' | 'q' | 'k' | 
                'P' | 'N' | 'B' | 'R' | 'Q' | 'K' => {
                    let color = if c.is_lowercase() {
                        Color::Black
                    } else {
                        Color::White
                    };

                    let piece = match c.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'n' => PieceType::Knight,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        'q' => PieceType::Queen,
                        'k' => PieceType::King,
                        _ => unreachable!(),
                    };

                    let sq = Square::new((rank * 8 + file) as u8);

                    board.set_piece(color, piece, sq);
                    file += 1;
                }

                _ => return Err(format!("invalid FEN character: {}", c)),
            }
        }

        board.side_to_move = if parts[1] == "w" { Color::White } else { Color::Black };

        if parts[2] != "-" {
            for c in parts[2].chars() {
                match c {
                    'K' => board.castling |= 1,
                    'Q' => board.castling |= 2,
                    'k' => board.castling |= 4,
                    'q' => board.castling |= 8,
                    _ => {}
                }
            }
        }

        if parts[3] != "-" {
            let chars: Vec<char> = parts[3].chars().collect();
            let f = chars[0] as u8 - b'a';
            let r = chars[1] as u8 - b'1';
            board.en_passant = Some(Square::new(r * 8 + f));
        }

        board.halfmove_clock = parts[4].parse().unwrap_or(0);
        board.fullmove_number = parts[5].parse().unwrap_or(1);

        board.calculate_zobrist();

        Ok(board)
    }
}
