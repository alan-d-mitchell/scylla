use types::{bitboard::Bitboard, square::Square};

// We define directions as coordinate offsets (file_offset, rank_offset)
const DIRECTIONS: [(i8, i8); 8] = [
    (0, 1),   // North
    (0, -1),  // South
    (1, 0),   // East
    (-1, 0),  // West
    (1, 1),   // North-East
    (-1, 1),  // North-West
    (1, -1),  // South-East
    (-1, -1), // South-West
];

/// Shoots a ray from a square in a specific direction until it hits a blocker or the edge.
pub fn classical_ray(sq: Square, dir: (i8, i8), blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let mut file = (sq.0 % 8) as i8;
    let mut rank = (sq.0 / 8) as i8;

    loop {
        file += dir.0;
        rank += dir.1;

        // If we fall off the board, stop.
        if file < 0 || file > 7 || rank < 0 || rank > 7 {
            break;
        }

        let target_sq = (rank * 8 + file) as u8;
        attacks |= Bitboard(1 << target_sq);

        // If we hit a piece, the attack stops AFTER including the occupied square
        // (because we can capture the blocking piece).
        if blockers.contains(target_sq) {
            break;
        }
    }

    attacks
}
