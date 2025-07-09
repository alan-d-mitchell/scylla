#ifndef DEFS_H
#define DEFS_H

#include <stdint.h>

// A more convenient type for our 64-bit bitboards.
typedef uint64_t u64;

// Struct to represent a single chess move
typedef struct {
    int from;   // Starting square
    int to;     // Destination square
    int piece;  // The piece being moved
    int promotion; // The piece to promote to
    int is_capture;
    int is_enpassant;
    int is_castle;
} Move;

// A list to store all generated moves for a position
#define MAX_MOVES 256
typedef struct {
    Move moves[MAX_MOVES];
    int count;
} MoveList;

#endif