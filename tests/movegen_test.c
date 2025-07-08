// tests/movegen_test.c
// A tester file for testing the pre-computed attack table and masks

#include <stdio.h>
#include "movegen.h"
#include "move.h"

void print_test(const char* title, u64 bitboard) {
    printf("--- %s ---\n", title);

    print_bitboard(bitboard);
}

const char* square_to_algebraic[] = {
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
};

void print_move_list(const MoveList* move_list) {
    printf("Found %d moves:\n", move_list->count);

    for (int i = 0; i < move_list->count; i++) {
        Move move = move_list->moves[i];

        printf("  %s%s%s\n", 
               square_to_algebraic[move.from],
               move.is_capture ? "x" : "-",
               square_to_algebraic[move.to]);
    }
}

int main() {
    // This is the most important step! We must initialize the tables before using them.
    init_attack_tables();
    
    printf("====== Non-Sliding Piece Attack Tests ======\n");
    print_test("Knight attacks from e4", knight_attacks[e4]);
    print_test("White Pawn attacks from d2", pawn_attacks[0][d2]);
    print_test("Black Pawn attacks from g7", pawn_attacks[1][g7]);
    print_test("King attacks from h1", king_attacks[h1]);


    printf("\n====== Sliding Piece Blocker Mask Tests ======\n");
    print_test("Rook blocker mask from a1", rook_relevant_blockers[a1]);
    print_test("Rook blocker mask from d4", rook_relevant_blockers[d4]);
    print_test("Bishop blocker mask from g2", bishop_relevant_blockers[g2]);
    print_test("Bishop blocker mask from f5", bishop_relevant_blockers[f5]);

    
    printf("\n====== Real Knight Move Generation Test ======\n");
    // Set up a board from the complex "Kiwipete" FEN string
    const char* fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    Board board;
    parse_fen(&board, fen);

    // Create an empty move list
    MoveList move_list = { .count = 0 };

    // Generate the knight moves for the position (it's white's turn)
    generate_all_knight_moves(&board, &move_list);

    // Print the results
    printf("--- Testing all knight moves from FEN: %s ---\n", fen);
    print_move_list(&move_list);

    return 0;
}