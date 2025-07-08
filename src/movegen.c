// movegen.c

#include "movegen.h"
#include "move.h"

// Piece attack tables
u64 pawn_attacks[2][64];
u64 knight_attacks[64];
u64 king_attacks[64];

// Sliding piece blocker masks
u64 bishop_relevant_blockers[64];
u64 rook_relevant_blockers[64];

// --- Constant Masks ---

// Masks for preventing wrap-around on files A/H
const u64 not_a_file = 18374403900871474942ULL; // ~0x0101010101010101
const u64 not_h_file = 9187201950435737471ULL;  // ~0x8080808080808080
const u64 not_hg_file = 4557430888798830399ULL; // ~0xC0C0C0C0C0C0C0C0
const u64 not_ab_file = 18229723555195321596ULL;// ~0x0303030303030303

// Masks for board edges (used for sliding piece blocker masks)
const u64 rank_1 = 0x00000000000000FFULL;
const u64 rank_8 = 0xFF00000000000000ULL;
const u64 file_a = 0x0101010101010101ULL;
const u64 file_h = 0x8080808080808080ULL;
const u64 board_edges = rank_1 | rank_8 | file_a | file_h;

// --- Main Initialization Functions ---

void generate_pawn_attacks() {
    for (int square = 0; square < 64; square++) {
        u64 start_bitboard = 1ULL << square;
        u64 white_attacks = 0ULL;
        u64 black_attacks = 0ULL;

        if (((start_bitboard << 9) & not_a_file) != 0) white_attacks |= (start_bitboard << 9);
        if (((start_bitboard << 7) & not_h_file) != 0) white_attacks |= (start_bitboard << 7);
        if (((start_bitboard >> 9) & not_h_file) != 0) black_attacks |= (start_bitboard >> 9);
        if (((start_bitboard >> 7) & not_a_file) != 0) black_attacks |= (start_bitboard >> 7);
        
        pawn_attacks[0][square] = white_attacks;
        pawn_attacks[1][square] = black_attacks;
    }
}

void generate_knight_attacks() {
    for (int square = 0; square < 64; square++) {
        u64 start_bitboard = 1ULL << square;
        u64 attacks = 0ULL;

        if (((start_bitboard << 17) & not_a_file) != 0) attacks |= (start_bitboard << 17);
        if (((start_bitboard << 15) & not_h_file) != 0) attacks |= (start_bitboard << 15);
        if (((start_bitboard << 10) & not_ab_file) != 0) attacks |= (start_bitboard << 10);
        if (((start_bitboard << 6) & not_hg_file) != 0) attacks |= (start_bitboard << 6);
        if (((start_bitboard >> 17) & not_h_file) != 0) attacks |= (start_bitboard >> 17);
        if (((start_bitboard >> 15) & not_a_file) != 0) attacks |= (start_bitboard >> 15);
        if (((start_bitboard >> 10) & not_hg_file) != 0) attacks |= (start_bitboard >> 10);
        if (((start_bitboard >> 6) & not_ab_file) != 0) attacks |= (start_bitboard >> 6);

        knight_attacks[square] = attacks;
    }
}

void generate_king_attacks() {
    for (int square = 0; square < 64; square++) {
        u64 start_bitboard = 1ULL << square;
        u64 attacks = 0ULL;

        if (((start_bitboard << 9) & not_a_file) != 0) attacks |= (start_bitboard << 9);
        if (((start_bitboard << 8)) != 0)               attacks |= (start_bitboard << 8);
        if (((start_bitboard << 7) & not_h_file) != 0) attacks |= (start_bitboard << 7);
        if (((start_bitboard << 1) & not_a_file) != 0) attacks |= (start_bitboard << 1);
        if (((start_bitboard >> 9) & not_h_file) != 0) attacks |= (start_bitboard >> 9);
        if (((start_bitboard >> 8)) != 0)               attacks |= (start_bitboard >> 8);
        if (((start_bitboard >> 7) & not_a_file) != 0) attacks |= (start_bitboard >> 7);
        if (((start_bitboard >> 1) & not_h_file) != 0) attacks |= (start_bitboard >> 1);
        
        king_attacks[square] = attacks;
    }
}

// --- Helper Functions for Sliding Piece Mask Generation ---

// Generates bishop attacks on an empty board.
u64 generate_bishop_attacks_empty_board(int square) {
    u64 attacks = 0ULL;
    int r, f;
    int tr = square / 8;
    int tf = square % 8;

    // Generate rays in all 4 diagonal directions
    for (r = tr + 1, f = tf + 1; r <= 7 && f <= 7; r++, f++) attacks |= (1ULL << (r * 8 + f));
    for (r = tr + 1, f = tf - 1; r <= 7 && f >= 0; r++, f--) attacks |= (1ULL << (r * 8 + f));
    for (r = tr - 1, f = tf + 1; r >= 0 && f <= 7; r--, f++) attacks |= (1ULL << (r * 8 + f));
    for (r = tr - 1, f = tf - 1; r >= 0 && f >= 0; r--, f--) attacks |= (1ULL << (r * 8 + f));
    
    return attacks;
}

// Generates rook attacks on an empty board.
u64 generate_rook_attacks_empty_board(int square) {
    u64 attacks = 0ULL;
    int r, f;
    int tr = square / 8;
    int tf = square % 8;

    // Generate rays in all 4 cardinal directions
    for (r = tr + 1; r <= 7; r++) attacks |= (1ULL << (r * 8 + tf));
    for (r = tr - 1; r >= 0; r--) attacks |= (1ULL << (r * 8 + tf));
    for (f = tf + 1; f <= 7; f++) attacks |= (1ULL << (tr * 8 + f));
    for (f = tf - 1; f >= 0; f--) attacks |= (1ULL << (tr * 8 + f));
    
    return attacks;
}

void generate_bishop_masks() {
    for (int square = 0; square < 64; square++) {
        bishop_relevant_blockers[square] = generate_bishop_attacks_empty_board(square);
    }
}

void generate_rook_masks() {
    for (int square = 0; square < 64; square++) {
        rook_relevant_blockers[square] = generate_rook_attacks_empty_board(square);
    }
}

// The single, main initialization function to be called from main()
void init_attack_tables() {
    generate_pawn_attacks();
    generate_knight_attacks();
    generate_king_attacks();
    generate_bishop_masks();
    generate_rook_masks();
}

void generate_all_knight_moves(const Board* board, MoveList* move_list) {
    int side = board->side_to_move;

    u64 friendlies = board->occupancies[side];
    u64 enemies = board->occupancies[!side];
    u64 knights = board->piece_bitboards[side == WHITE ? N : n];

     while (knights) {
        int from_square = __builtin_ctzll(knights);
        u64 attacks = knight_attacks[from_square];
        u64 valid_moves = attacks & ~friendlies;
        
        while (valid_moves) {
            int to_square = __builtin_ctzll(valid_moves);
            Move move = {
                .from = from_square,
                .to = to_square,
                .piece = side == WHITE ? N : n,
                .promotion = 0,
                .is_capture = (enemies & (1ULL << to_square)) ? 1 : 0,
                .is_enpassant = 0,
                .is_castle = 0
            };

            move_list->moves[move_list->count++] = move;
            valid_moves &= valid_moves - 1;
        }
        
        knights &= knights - 1;
    }
}
