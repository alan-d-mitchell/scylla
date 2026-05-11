use board::Board;
use movegen::generator;
use types::piece::PieceType;
use std::time::Instant;

pub fn run_perft(board: &mut Board, depth: usize) {
    let start = Instant::now();
    let mut total_nodes = 0;

    let moves = generator::generate_pseudo_legal_moves(board);

    println!("\nperft results (depth {}):", depth);
    println!("---------------------------");

    for m in moves.as_slice() {
        let san = movegen::notation::move_to_san(board, *m);
        let undo = board.make_move(*m);

        let us = board.side_to_move.flip();
        let king_sq = board.get_king_square(us);
        
        // Only recurse if the move was legal
        if !generator::is_square_attacked(board, king_sq, board.side_to_move) {
            let nodes = perft_recursive(board, depth - 1);
            println!("{:<5} ({}): {}", m.to_string(), san, nodes);
            total_nodes += nodes;
        }

        board.unmake_move(*m, undo);
    }

    let duration = start.elapsed();
    let nps = if duration.as_secs_f64() > 0.0 {
        (total_nodes as f64 / duration.as_secs_f64()) as u64
    } else { 0 };

    println!("---------------------------");
    println!("total nodes: {}", total_nodes);
    println!("time:        {:?}", duration);
    println!("nps:         {} ({:.2} mnps)", nps, nps as f64 / 1_000_000.0);
}

fn perft_recursive(board: &mut Board, depth: usize) -> u64 {
    if depth == 0 { return 1; }

    let mut nodes = 0;
    let moves = generator::generate_pseudo_legal_moves(board);

    let enemy_king_bb = board.pieces[board.side_to_move.flip() as usize][PieceType::King as usize].0;

    for m in moves.as_slice() {
        if (enemy_king_bb & (1u64 << m.to_sq().0)) != 0 {
            continue;
        }

        let hash_before = board.zobrist_key;

        let undo = board.make_move(*m);

        let us = board.side_to_move.flip();
        let king_sq = board.get_king_square(us);
        
        if !generator::is_square_attacked(board, king_sq, board.side_to_move) {
            nodes += perft_recursive(board, depth - 1);
        }

        board.unmake_move(*m, undo);

        assert_eq!(
            hash_before, 
            board.zobrist_key, 
            "FATAL ZOBRIST DESYNC! move: {}, hash before: {:x}, hash after: {:x}", 
            m, hash_before, board.zobrist_key
        );
    }

    nodes
}
