use uci::interface;
use clap::Parser;
use board::Board;
use movegen::generator;
use std::time::Instant;
use search::Searcher;
use tt::TranspositionTable;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Print pseudo-legal moves for a given position
    #[arg(short, long)]
    debug_moves: bool,

    /// Custom FEN string to use for debug commands (defaults to startpos)
    #[arg(short, long, default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
    fen: String,

    /// Run a Perft test to a specific depth
    #[arg(short, long)]
    perft: Option<usize>,

    /// Search the position to the given depth
    #[arg(short, long)]
    search: Option<u8>,
}

fn main() {
    movegen::init_all();

    let raw_args: Vec<String> = std::env::args().collect();
    if raw_args.len() == 1 || raw_args.get(1).map(|s| s.as_str()) == Some("uci") {
        interface::start_loop();
        return; // Exit main so clap doesn't panic
    }

    let args = Args::parse();

    if args.debug_moves {
        run_debug_moves(&args.fen);
    } else if let Some(depth) = args.perft {
        let mut board: Board = args.fen.parse().expect("failed to parse FEN");
        perft::run_perft(&mut board, depth);
    } else if let Some(depth) = args.search {
        let mut board: Board = args.fen.parse().expect("failed to parse FEN");
        let mut tt = TranspositionTable::new(256);
        
        println!("engine analyzing position at depth {}", depth);

        let start = Instant::now();

        let mut searcher = Searcher::new(&mut tt, 10_000_000);
        let best_move = searcher.search_position(&mut board, depth, 0);

        let elapsed = start.elapsed();

        match best_move {
            Some(m) => {
                let pv = search::extract_pv(&mut board, &tt, depth);
                let pv_string = pv.join(" ");

                println!("best move: {} (found in {:?})", m, elapsed);
                println!("PV: {}", pv_string);
            },
            None => println!("no legal moves found! (checkmate or stalemate)"),
        }
    } else {
        interface::start_loop();
    }

}

fn run_debug_moves(fen: &str) {
    println!("loading FEN: {}", fen);
    
    let board: Board = fen.parse().expect("failed to parse FEN string");

    // Print the board state to verify the pieces are where we think they are
    println!("--- board state ---");
    println!("{}", board);

    // Also print the occupancy bitboard to see if it matches the pieces
    let all_pieces = board.colors[0] | board.colors[1];
    println!("occupancy bitboard:");
    for rank in (0..8).rev() {
        for file in 0..8 {
            let mask = 1u64 << (rank * 8 + file);
            if (all_pieces.0 & mask) != 0 {
                print!("1 ");
            } else {
                print!(". ");
            }
        }
        println!();
    }
    println!("-------------------");
    
    let start_time = Instant::now();
    let pseudo_moves = generator::generate_pseudo_legal_moves(&board);

    let mut legal_moves = Vec::new();

    for m in pseudo_moves.as_slice() {
        let mut temp_board = board.clone();
        temp_board.make_move(*m);

        let king_sq = temp_board.get_king_square(board.side_to_move);
        if !generator::is_square_attacked(&temp_board, king_sq, board.side_to_move.flip()) {
            legal_moves.push(*m);
        }
    }

    let elapsed = start_time.elapsed();

    println!("found {} pseudo-legal moves ({} legal) in {:?}:", 
        pseudo_moves.count, 
        legal_moves.len(), 
        elapsed
    );

    let mut san_board = board.clone();

    for m in legal_moves {
        let san = movegen::notation::move_to_san(&mut san_board, m);
        println!("- {:<5} ({})", m.to_string(), san);
    }
}
