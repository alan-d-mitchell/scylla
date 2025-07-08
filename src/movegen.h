#ifndef MOVEGEN_H
#define MOVEGEN_H

#include "bitboard.h"
#include "move.h"

// --- Pre-computed Attack Tables ---
extern u64 pawn_attacks[2][64];
extern u64 knight_attacks[64];
extern u64 king_attacks[64];
extern u64 bishop_relevant_blockers[64];
extern u64 rook_relevant_blockers[64];

// --- Function Declarations (Prototypes) ---

// Initialization function
void init_attack_tables();

// Real move generation function
void generate_all_knight_moves(const Board* board, MoveList* move_list);

#endif // MOVEGEN_H