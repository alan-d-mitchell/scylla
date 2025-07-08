#ifndef BITBOARD_H
#define BITBOARD_H

#include <stdint.h>

// A more convenient type for our 64-bit bitboards.
typedef uint64_t u64;

// Enums for better readability
enum { P, N, B, R, Q, K, p, n, b, r, q, k };
enum { WHITE, BLACK };
enum { WK = 1, WQ = 2, BK = 4, BQ = 8 }; // For castling rights

// Enum for board squares
enum {
    a1, b1, c1, d1, e1, f1, g1, h1,
    a2, b2, c2, d2, e2, f2, g2, h2,
    a3, b3, c3, d3, e3, f3, g3, h3,
    a4, b4, c4, d4, e4, f4, g4, h4,
    a5, b5, c5, d5, e5, f5, g5, h5,
    a6, b6, c6, d6, e6, f6, g6, h6,
    a7, b7, c7, d7, e7, f7, g7, h7,
    a8, b8, c8, d8, e8, f8, g8, h8
};

// The main struct to hold all information about the chess position.
typedef struct {
    u64 piece_bitboards[12];
    u64 occupancies[3]; // 0 for white, 1 for black, 2 for both

    int side_to_move;
    int enpassant_square; // -1 if no en passant square
    int castle_rights;

} Board;

// --- Function Declarations (Prototypes) ---

void print_bitboard(u64 bitboard);
void parse_fen(Board* board, const char* fen);

#endif // BITBOARD_H