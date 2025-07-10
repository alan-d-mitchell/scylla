#include <stdio.h>
#include <string.h>
#include <stdlib.h> // <-- Needed for malloc() and free()

#include "datagen.h"
#include "board.h"
#include "movegen.h"
#include "search.h"
#include "nn_encode.h"

#define MAX_GAME_MOVES 512
#define POLICY_SIZE 4672

// A simple struct to hold the data for one position in a game
typedef struct {
    char fen[256];
    float policy[POLICY_SIZE];
} GamePositionData;

void generate_training_data(int num_games) {
    FILE* data_file = fopen("self_play_data.txt", "a");
    if (data_file == NULL) {
        printf("Error: Could not open self_play_data.txt for writing.\n");
        return;
    }

    // --- HEAP ALLOCATION ---
    // Allocate memory for the game history on the heap instead of the stack.
    GamePositionData* game_history = malloc(sizeof(GamePositionData) * MAX_GAME_MOVES);
    if (game_history == NULL) {
        printf("Error: Failed to allocate memory for game history.\n");
        fclose(data_file);
        return;
    }

    printf("Starting data generation for %d games...\n", num_games);

    for (int i = 0; i < num_games; i++) {
        Board board;
        parse_fen(&board, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        int move_count = 0;
        float game_result = 0.0f; // Use float for consistency

        // --- Play a full game ---
        while(move_count < MAX_GAME_MOVES) {
            MoveList move_list;
            generate_all_moves(&board, &move_list);

            if (move_count > 0 && is_draw(&board)) {
                game_result = 0.0f;
                break;
            }

            if (move_list.count == 0) {
                int king_sq = __builtin_ctzll(board.piece_bitboards[board.side_to_move == WHITE ? K : k]);
                if (is_square_attacked(king_sq, !board.side_to_move, &board)) {
                    game_result = (board.side_to_move == WHITE) ? -1.0f : 1.0f; // Checkmate
                } else {
                    game_result = 0.0f; // Stalemate
                }
                break;
            }

            Move best_move = search_position(&board, 4);

            // For now, we are using a placeholder FEN. A full implementation
            // would require a board_to_fen() function.
            strcpy(game_history[move_count].fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            move_to_policy_vector(best_move, game_history[move_count].policy);

            make_move(&board, best_move);
            move_count++;
        }

        // --- Write the collected game data to the file ---
        for (int j = 0; j < move_count; j++) {
            fprintf(data_file, "%s | ", game_history[j].fen);
            for (int k = 0; k < POLICY_SIZE; k++) {
                fprintf(data_file, "%.4f%s", game_history[j].policy[k], (k == POLICY_SIZE - 1) ? "" : ",");
            }
            // Determine the result from White's perspective for this position
            float final_value = game_result;
            // The FEN above is always from white's perspective, so this is simplified.
            // A real implementation would check whose turn it was at game_history[j].
            fprintf(data_file, " | %.1f\n", final_value);
        }
        printf("Finished game %d / %d\n", i + 1, num_games);
    }

    // --- FREE MEMORY ---
    // Release the heap memory when we're done with it.
    free(game_history);
    fclose(data_file);
    printf("Data generation complete.\n");
}

// Helper function to detect draws by repetition or 50-move rule
int is_draw(Board* board) {
    // 50-move rule
    if (board->ply >= 100) return 1;

    // Three-fold repetition check (requires storing board history hashes)
    // This is complex and omitted for now, but is necessary for a full engine.

    return 0;
}