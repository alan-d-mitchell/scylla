#ifndef BOARD_H
#define BOARD_H

#include "defs.h"

typedef struct {
    int captured_piece;
    int enpassant_square;
    int castling_rights;
} UndoInfo;

typedef struct {
    u64 piece_bitboards[12];
    u64 occupancies[3]; // 0 for white, 1 for black, 2 for both
    int side_to_move;
    int enpassant_square; // -1 if no en passant square
    int castle_rights;
    int ply;
    UndoInfo history[256];
} Board;

void make_move(Board* board, Move move);
void undo_move(Board* board, Move move);

#endif