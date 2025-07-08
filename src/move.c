




void generate_knight_attacks(const Board* board, MoveList* move_list) {
    // Determine whose turn it is and get relevant bitboards
    int side = board->side_to_move;
    u64 friendlies = board->occupancies[side];
    u64 enemies = board->occupancies[!side];

    // Get the bitboard of our knights
    u64 knights = board->piece_bitboards[side == 0 ? N : n];

    // Loop through each of our knights
    while (knights) {
        // Get the starting square of one knight
        int from_square = __builtin_ctzll(knights);
        
        // Get the attack bitboard for this knight from our pre-computed table
        u64 attacks = knight_attacks[from_square];
        
        // A knight can move to any of its attack squares that are NOT occupied by a friendly piece
        u64 valid_moves = attacks & ~friendlies;
        
        // Loop through each valid destination square
        while (valid_moves) {
            int to_square = __builtin_ctzll(valid_moves);
            
            // Create the move object and add it to the list
            Move move;
            move.from = from_square;
            move.to = to_square;
            move.piece = side == 0 ? N : n;
            move.promotion = 0;
            // Check if the destination square is occupied by an enemy piece
            move.is_capture = (enemies & (1ULL << to_square)) ? 1 : 0;
            move.is_enpassant = 0;
            move.is_castle = 0;

            move_list->moves[move_list->count++] = move;

            // Remove this move from the bitboard to process the next one
            valid_moves &= valid_moves - 1;
        }

        // Remove the knight we just processed from the bitboard
        knights &= knights - 1;
    }
}