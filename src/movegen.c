// movegen.c

#include <inttypes.h>
#include <stdio.h>
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

// Magic numbers for bishops (pre-computed and well-known)
const u64 bishop_magic_numbers[64] = {
    0x40040844404084ULL, 0x2004208a004208ULL, 0x101010101010101ULL, 0x202020202020202ULL,
    0x404040404040404ULL, 0x808080808080808ULL, 0x400804020100400ULL, 0x200100200408080ULL,
    0x804002401000804ULL, 0x80042008100420ULL, 0x1000102004010ULL, 0x200100040080402ULL,
    0x400080200100104ULL, 0x80004001002008ULL, 0x800820040010010ULL, 0x408820010004002ULL,
    0x2000401004004ULL, 0x10004008002008ULL, 0x2002008004010ULL, 0x20080040010020ULL,
    0x40040020008010ULL, 0x800800400020010ULL, 0x1001000400020008ULL, 0x2042008001000ULL,
    0x401004002000800ULL, 0x8020010040080ULL, 0x1008020010040ULL, 0x20040800200100ULL,
    0x4002010040080ULL, 0x80010040020080ULL, 0x1004002001008ULL, 0x208401000200ULL,
    0x41040020008080ULL, 0x82080040020080ULL, 0x10100400020008ULL, 0x20200800400100ULL,
    0x40400800200100ULL, 0x80800400100200ULL, 0x10010020004008ULL, 0x2002041000800ULL,
    0x40010400200080ULL, 0x8002080040010ULL, 0x1004010020004ULL, 0x2008020040010ULL,
    0x4008010020004ULL, 0x80040200100080ULL, 0x1008040020010ULL, 0x2008040010020ULL,
    0x4088100020004ULL, 0x8108200040010ULL, 0x1020100400080ULL, 0x2040200800401ULL,
    0x4080200400100ULL, 0x8040400200100ULL, 0x1080800200040ULL, 0x2010840010002ULL,
    0x402082004000100ULL, 0x8408200100040ULL, 0x1082000400100ULL, 0x2082008400102ULL,
    0x41020008200401ULL, 0x8204001080080ULL, 0x1040002080100ULL, 0x2080004102008ULL
};

const u64 bishop_magic_numbers[64] = {
    0x40040844404084ULL, 0x2004208A004208ULL, 0x101010101010101ULL, 0x202020202020202ULL,
    0x404040404040404ULL, 0x808080808080808ULL, 0x400804020100400ULL, 0x200100200408080ULL,
    0x804002401000804ULL, 0x80042008100420ULL, 0x1000102004010ULL,   0x200100040080402ULL,
    0x400080200100104ULL, 0x80004001002008ULL,  0x800820040010010ULL, 0x408820010004002ULL,
    0x2000401004004ULL,   0x10004008002008ULL,  0x2002008004010ULL,   0x20080040010020ULL,
    0x40040020008010ULL,  0x800800400020010ULL, 0x1001000400020008ULL,0x2042008001000ULL,
    0x401004002000800ULL, 0x8020010040080ULL,   0x1008020010040ULL,   0x20040800200100ULL,
    0x4002010040080ULL,   0x80010040020080ULL,  0x1004002001008ULL,   0x208401000200ULL,
    0x41040020008080ULL,  0x82080040020080ULL,  0x10100400020008ULL,  0x20200800400100ULL,
    0x40400800200100ULL,  0x80800400100200ULL,  0x10010020004008ULL,  0x2002041000800ULL,
    0x40010400200080ULL,  0x8002080040010ULL,   0x1004010020004ULL,   0x2008020040010ULL,
    0x4008010020004ULL,   0x80040200100080ULL,  0x1008040020010ULL,   0x2008040010020ULL,
    0x4088100020004ULL,   0x8108200040010ULL,   0x1020100400080ULL,   0x2040200800401ULL,
    0x4080200400100ULL,   0x8040400200100ULL,   0x1080800200040ULL,   0x2010840010002ULL,
    0x402082004000100ULL, 0x8408200100040ULL,   0x1082000400100ULL,   0x2082008400102ULL,
    0x41020008200401ULL,  0x8204001080080ULL,   0x1040002080100ULL,   0x2080004102008ULL
};

// Attack table, indexed by the magic index. Needs to be large enough.
u64 bishop_attack_table[5248];

// Array of pointers. Each square will have a pointer that points to
// the start of its dedicated section within the main bishop_attack_table.
u64* bishop_attacks[64];

const int bishop_relevant_bits[64] = {
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6
};

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

// Generates bishop attacks
u64 generate_bishop_attacks(int square, u64 blockers) {
    u64 attacks = 0ULL;
    int r, f;
    int tr = square / 8;
    int tf = square % 8;

    // Generate rays and stop at the first blocker
    for (r = tr + 1, f = tf + 1; r <= 7 && f <= 7; r++, f++) {
        attacks |= (1ULL << (r * 8 + f));
        if ((1ULL << (r * 8 + f)) & blockers) break;
    }
    for (r = tr + 1, f = tf - 1; r <= 7 && f >= 0; r++, f--) {
        attacks |= (1ULL << (r * 8 + f));
        if ((1ULL << (r * 8 + f)) & blockers) break;
    }
    for (r = tr - 1, f = tf + 1; r >= 0 && f <= 7; r--, f++) {
        attacks |= (1ULL << (r * 8 + f));
        if ((1ULL << (r * 8 + f)) & blockers) break;
    }
    for (r = tr - 1, f = tf - 1; r >= 0 && f >= 0; r--, f--) {
        attacks |= (1ULL << (r * 8 + f));
        if ((1ULL << (r * 8 + f)) & blockers) break;
    }
    return attacks;
}

void generate_bishop_masks() {
    for (int square = 0; square < 64; square++) {
        bishop_relevant_blockers[square] = generate_bishop_attacks(square, 0ULL) & ~board_edges;
    }
}

void init_magic_bishop_attacks() {
    u64* attack_table_ptr = bishop_attack_table; // Start at beginning of our mem block

    for (int s = 0; s < 64; s++) {
        // Set the pointer for the current square to the current position in the attack table
        bishop_attacks[s] = attack_table_ptr;

        u64 blocker_mask = bishop_relevant_blockers[s];
        int relevant_bits_count = bishop_relevant_bits[s];
        int occupancy_indices = 1 << relevant_bits_count;

        // Loop through all possible blocker configs for current square
        for (int i = 0; i < occupancy_indices; i++) {
            u64 temp_mask = blocker_mask;
            u64 current_blockers = 0ULL;

            // Generate specific blocker permutation
            for (int bit_index = 0; bit_index < relevant_bits_count; bit_index++) {
                int lsb_index = __builtin_ctzll(temp_mask);
                temp_mask &= temp_mask - 1;

                if ((i >> bit_index) & 1) {
                    current_blockers |= (1ULL << lsb_index);
                }
            }

            // Calculate the magic index
            u64 magic_index = (current_blockers * bishop_magic_numbers[s]) >> (64 - bishop_relevant_bits[s]);

            // Generate the attack set for this specific blocker config and store it
            u64 attacks = generate_bishop_attacks(s, current_blockers);
            bishop_attacks[s][magic_index] = attacks;
        }

        // Advance the attack_table_ptr for the next square's data
        attack_table_ptr += occupancy_indices;
    }
}

u64 get_bishop_attacks(int square, u64 occupancy) {
    occupancy &= bishop_relevant_blockers[square];
    occupancy *= bishop_magic_numbers[square];
    occupancy >>= (64 - bishop_relevant_bits[square]);

    // Use the pointer for the square as a base, then add the magic index
    return bishop_attacks[square][occupancy];
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
    init_magic_bishop_attacks();
}

// Checks if a given square is attacked by a given side.
int is_square_attacked(int square, int side, const Board* board) {
    // Pawn attacks
    if ((pawn_attacks[!side][square] & board->piece_bitboards[side == WHITE ? P : p])) return 1;
    
    // Knight attacks
    if ((knight_attacks[square] & board->piece_bitboards[side == WHITE ? N : n])) return 1;

    // Bishop attacks
    u64 bishop_attacks = get_bishop_attacks(square, board->occupancies[2]);
    if (bishop_attacks & (board->piece_bitboards[side == WHITE ? B : b] | board->piece_bitboards[side == WHITE ? Q : q])) {
        return 1;
    }
    
    // King attacks
     if ((king_attacks[square] & board->piece_bitboards[side == WHITE ? K : k])) return 1;

    // Rook and Queen attacks (temporary, slow method)
    /* u64 rook_attacks = generate_rook_attacks_empty_board(square);
    if (rook_attacks & (board->piece_bitboards[side == WHITE ? R : r] | board->piece_bitboards[side == WHITE ? Q : q])) {
        return 1;
    } */

    return 0; // Square is not attacked
}

// Generates all legal pawn moves for the current side to move.
void generate_all_pawn_moves(const Board* board, MoveList* move_list) {
    int side = board->side_to_move;

    // Define bitboards and constants based on whose turn it is
    u64 my_pawns = board->piece_bitboards[side == WHITE ? P : p];
    u64 enemy_pieces = board->occupancies[!side];
    u64 all_pieces = board->occupancies[2];
    int push_direction = (side == WHITE) ? 8 : -8;
    u64 rank_7 = (side == WHITE) ? 0x00FF000000000000ULL : 0x000000000000FF00ULL;
    u64 rank_3 = (side == WHITE) ? 0x0000000000FF0000ULL : 0x0000FF0000000000ULL;
    int promotion_piece_start = (side == WHITE) ? N : n;
    int promotion_piece_end = (side == WHITE) ? Q : q;

    // --- 1. Single and Double Pawn Pushes ---
    u64 single_pushes = (side == WHITE ? (my_pawns << 8) : (my_pawns >> 8)) & ~all_pieces;
    u64 double_pushes = (side == WHITE ? ((single_pushes & rank_3) << 8) : ((single_pushes & rank_3) >> 8)) & ~all_pieces;

    // Process single pushes
    u64 pushes = single_pushes;
    while (pushes) {
        int to_square = __builtin_ctzll(pushes);
        int from_square = to_square - push_direction;

        // Check for promotion
        if ((1ULL << from_square) & rank_7) {
            for (int promo_piece = promotion_piece_start; promo_piece <= promotion_piece_end; promo_piece++) {
                Move move = { .from = from_square, .to = to_square, .piece = (side == WHITE ? P : p), .promotion = promo_piece };
                move_list->moves[move_list->count++] = move;
            }
        } else {
            Move move = { .from = from_square, .to = to_square, .piece = (side == WHITE ? P : p) };
            move_list->moves[move_list->count++] = move;
        }
        pushes &= pushes - 1;
    }

    // Process double pushes
    pushes = double_pushes;
    while (pushes) {
        int to_square = __builtin_ctzll(pushes);
        int from_square = to_square - (push_direction * 2);
        Move move = { .from = from_square, .to = to_square, .piece = (side == WHITE ? P : p) };
        move_list->moves[move_list->count++] = move;
        pushes &= pushes - 1;
    }

    // --- 2. Pawn Captures (including promotions) ---
    u64 pawns_for_capture = my_pawns;
    while (pawns_for_capture) {
        int from_square = __builtin_ctzll(pawns_for_capture);
        u64 attacks = pawn_attacks[side][from_square] & enemy_pieces;

        while (attacks) {
            int to_square = __builtin_ctzll(attacks);
            // Check for promotion on capture
            if ((1ULL << from_square) & rank_7) {
                for (int promo_piece = promotion_piece_start; promo_piece <= promotion_piece_end; promo_piece++) {
                    Move move = { .from = from_square, .to = to_square, .piece = (side == WHITE ? P : p), .promotion = promo_piece, .is_capture = 1 };
                    move_list->moves[move_list->count++] = move;
                }
            } else {
                Move move = { .from = from_square, .to = to_square, .piece = (side == WHITE ? P : p), .is_capture = 1 };
                move_list->moves[move_list->count++] = move;
            }
            attacks &= attacks - 1;
        }
        pawns_for_capture &= pawns_for_capture - 1;
    }

    // --- 3. En Passant ---
    if (board->enpassant_square != -1) {
        u64 ep_attackers = pawn_attacks[!side][board->enpassant_square] & my_pawns;
        while (ep_attackers) {
            int from_square = __builtin_ctzll(ep_attackers);
            Move move = { .from = from_square, .to = board->enpassant_square, .piece = (side == WHITE ? P : p), .is_capture = 1, .is_enpassant = 1 };
            move_list->moves[move_list->count++] = move;
            ep_attackers &= ep_attackers - 1;
        }
    }
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

void generate_all_bishop_moves(const Board* board, MoveList* move_list) {
    int side = board->side_to_move;
    u64 friendly_pieces = board->occupancies[side];
    u64 bishops = board->piece_bitboards[side == WHITE ? B : b];

    while (bishops) {
        int from_square = __builtin_ctzll(bishops);
        u64 attacks = get_bishop_attacks(from_square, board->occupancies[2]);
        u64 valid_moves = attacks & ~friendly_pieces;

        while (valid_moves) {
            int to_square = __builtin_ctzll(valid_moves);
            Move move = {
                .from = from_square, .to = to_square, .piece = (side == WHITE ? B : b),
                .is_capture = (board->occupancies[!side] & (1ULL << to_square)) ? 1 : 0
            };
            move_list->moves[move_list->count++] = move;
            valid_moves &= valid_moves - 1;
        }
        bishops &= bishops - 1;
    }
}

void generate_all_king_moves(const Board* board, MoveList* move_list) {
    int side = board->side_to_move;
    u64 friendly_pieces = board->occupancies[side];
    
    // Get the king's starting square
    int from_square = __builtin_ctzll(board->piece_bitboards[side == WHITE ? K : k]);
    
    // --- 1. Normal King Moves ---
    u64 attacks = king_attacks[from_square];
    u64 valid_moves = attacks & ~friendly_pieces;
    
    while (valid_moves) {
        int to_square = __builtin_ctzll(valid_moves);
        Move move = {
            .from = from_square, .to = to_square, .piece = (side == WHITE ? K : k),
            .is_capture = (board->occupancies[!side] & (1ULL << to_square)) ? 1 : 0
        };
        move_list->moves[move_list->count++] = move;
        valid_moves &= valid_moves - 1;
    }

    // --- 2. Castling Moves ---
    // Check king-side castling
    if (side == WHITE) {
        if ((board->castle_rights & WK) && 
            !((board->occupancies[2] >> (f1)) & 1) && !((board->occupancies[2] >> (g1)) & 1)) {
            if (!is_square_attacked(e1, BLACK, board) && !is_square_attacked(f1, BLACK, board)) {
                Move move = { .from = e1, .to = g1, .piece = K, .is_castle = 1 };
                move_list->moves[move_list->count++] = move;
            }
        }
        // Check queen-side castling
        if ((board->castle_rights & WQ) &&
            !((board->occupancies[2] >> (d1)) & 1) && !((board->occupancies[2] >> (c1)) & 1) && !((board->occupancies[2] >> (b1)) & 1)) {
            if (!is_square_attacked(e1, BLACK, board) && !is_square_attacked(d1, BLACK, board)) {
                Move move = { .from = e1, .to = c1, .piece = K, .is_castle = 1 };
                move_list->moves[move_list->count++] = move;
            }
        }
    } else { // Black side
        if ((board->castle_rights & BK) &&
            !((board->occupancies[2] >> (f8)) & 1) && !((board->occupancies[2] >> (g8)) & 1)) {
            if (!is_square_attacked(e8, WHITE, board) && !is_square_attacked(f8, WHITE, board)) {
                Move move = { .from = e8, .to = g8, .piece = k, .is_castle = 1 };
                move_list->moves[move_list->count++] = move;
            }
        }
        if ((board->castle_rights & BQ) &&
            !((board->occupancies[2] >> (d8)) & 1) && !((board->occupancies[2] >> (c8)) & 1) && !((board->occupancies[2] >> (b8)) & 1)) {
            if (!is_square_attacked(e8, WHITE, board) && !is_square_attacked(d8, WHITE, board)) {
                Move move = { .from = e8, .to = c8, .piece = k, .is_castle = 1 };
                move_list->moves[move_list->count++] = move;
            }
        }
    }
}