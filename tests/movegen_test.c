// tests/movegen_test.c
// A tester file for testing the pre-computed attack table and masks

#include <stdio.h>
#include "movegen.h"
#include "move.h"

void print_test(const char* title, u64 bitboard) {
    printf("--- %s ---\n", title);

    print_bitboard(bitboard);
}

// Helper array to map squares on the board
const char* square_to_algebraic[] = {
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
};

// Helper array to map piece enum to a character for printing promotions
const char piece_to_char[] = "PNBRQKpnbrqk";

void print_move_list(const MoveList* move_list) {
    printf("Found %d moves:\n", move_list->count);
    for (int i = 0; i < move_list->count; i++) {
        Move move = move_list->moves[i];
        
        // If the move is a promotion
        if (move.promotion) {
            printf("  %s%s%s=%c\n", 
                   square_to_algebraic[move.from],
                   move.is_capture ? "x" : "-",
                   square_to_algebraic[move.to],
                   piece_to_char[move.promotion]);
        } else { // For all other moves
            printf("  %s%s%s\n", 
                   square_to_algebraic[move.from],
                   move.is_capture ? "x" : "-",
                   square_to_algebraic[move.to]);
        }
    }
}

void pawn_move_test() {
    init_attack_tables();

    const char* fen = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/8/P1P1P1PP/RNBQKBNR w KQkq e6 0 1";
    Board board;
    parse_fen(&board, fen);

    MoveList move_list = { .count = 0 };
    generate_all_pawn_moves(&board, &move_list);

    printf("Testing pawn moves from FEN: %s\n", fen);
    print_move_list(&move_list);

}

void knight_move_test() {
    init_attack_tables();

    // the w after the main string with slashes can be changed to a b in order to see the black moves
    const char* fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    Board board;
    parse_fen(&board, fen);

    // Create an empty move list
    MoveList move_list = { .count = 0 };

    // Generate the knight moves for the position (it's white's turn)
    generate_all_knight_moves(&board, &move_list);

    // Print the results
    printf("Testing all knight moves from FEN: %s\n", fen);
    print_move_list(&move_list);
}

void bishop_move_test() {
    init_attack_tables();

    // A position with interesting bishop moves for white
    const char* fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    Board board;
    parse_fen(&board, fen);

    MoveList move_list = { .count = 0 };

    // Generate only the bishop moves
    generate_all_bishop_moves(&board, &move_list);

    printf("Testing bishop moves from FEN: %s\n", fen);
    print_move_list(&move_list);
}

void king_move_test() {
    init_attack_tables();

    // FEN designed for testing castling
    const char* fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    Board board;
    parse_fen(&board, fen);

    MoveList move_list = { .count = 0 };
    generate_all_king_moves(&board, &move_list);

    printf("Testing king moves from FEN: %s\n", fen);
    print_move_list(&move_list);
}

int main() {
    init_attack_tables();

    pawn_move_test();
    knight_move_test();
    bishop_move_test();
    king_move_test();

    return 0;
}