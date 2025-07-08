#include <stdio.h>
#include <string.h>
#include <inttypes.h>

#include "bitboard.h"

// Private helper function, not needed in the .h file
void set_bit(u64* bitboard, int square, int action) {
    if (action) {
        *bitboard |= (1ULL << square); // Set bit
    } else {
        *bitboard &= ~(1ULL << square); // Clear bit
    }
}

void print_bitboard(u64 bitboard) {
    printf("Bitboard repr: %" PRIu64 "\n", bitboard);

    for (int rank = 7; rank >= 0; rank--) {
        printf(" %d |", rank + 1);
        for (int file = 0; file < 8; file++) {
            int square = rank * 8 + file;
            if ((bitboard >> square) & 1) {
                printf(" X");
            } else {
                printf(" .");
            }
        }
        printf("\n");
    }
    printf("   +----------------\n     a b c d e f g h\n\n");
}

void parse_fen(Board* board, const char* fen) {
    // Reset board state
    memset(board->piece_bitboards, 0, sizeof(board->piece_bitboards));
    memset(board->occupancies, 0, sizeof(board->occupancies));
    board->side_to_move = 0;
    board->enpassant_square = -1;
    board->castle_rights = 0;
    
    // Use a mutable copy of the FEN string for strtok
    char fen_copy[256];
    strncpy(fen_copy, fen, 255);
    fen_copy[255] = '\0';

    char* token;

    // 1. Piece Placement
    token = strtok(fen_copy, " ");
    int rank = 7;
    int file = 0;
    for (size_t i = 0; i < strlen(token); i++) {
        char c = token[i];
        if (c == '/') {
            rank--;
            file = 0;
        } else if (c >= '1' && c <= '8') {
            file += c - '0';
        } else {
            int square = rank * 8 + file;
            int piece_type = -1;
            switch(c) {
                case 'P': piece_type=P; break; case 'N': piece_type=N; break;
                case 'B': piece_type=B; break; case 'R': piece_type=R; break;
                case 'Q': piece_type=Q; break; case 'K': piece_type=K; break;
                case 'p': piece_type=p; break; case 'n': piece_type=n; break;
                case 'b': piece_type=b; break; case 'r': piece_type=r; break;
                case 'q': piece_type=q; break; case 'k': piece_type=k; break;
            }
            if (piece_type != -1) {
                set_bit(&board->piece_bitboards[piece_type], square, 1);
            }
            file++;
        }
    }

    // 2. Side to Move
    token = strtok(NULL, " ");
    board->side_to_move = (strcmp(token, "w") == 0) ? WHITE : BLACK;

    // 3. Castling Rights
    token = strtok(NULL, " ");
    for (size_t i = 0; i < strlen(token); i++) {
        switch (token[i]) {
            case 'K': board->castle_rights |= WK; break;
            case 'Q': board->castle_rights |= WQ; break;
            case 'k': board->castle_rights |= BK; break;
            case 'q': board->castle_rights |= BQ; break;
        }
    }

    // 4. En Passant Square
    token = strtok(NULL, " ");
    if (strcmp(token, "-") != 0) {
        int ep_file = token[0] - 'a';
        int ep_rank = token[1] - '1';
        board->enpassant_square = ep_rank * 8 + ep_file;
    }

    // Populate occupancy bitboards
    for (int piece = P; piece <= K; piece++) board->occupancies[WHITE] |= board->piece_bitboards[piece];
    for (int piece = p; piece <= k; piece++) board->occupancies[BLACK] |= board->piece_bitboards[piece];
    board->occupancies[2] = board->occupancies[WHITE] | board->occupancies[BLACK];
}