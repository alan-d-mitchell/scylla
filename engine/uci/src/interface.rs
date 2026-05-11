use std::io::{self, BufRead};
use board::Board;

pub fn start_loop() {
    let stdin = io::stdin();
    
    let mut board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        .parse::<Board>()
        .unwrap();

    // Initialize the TT ONCE for the whole game so she remembers moves between turns!
    let mut tt = tt::TranspositionTable::new(256); 

    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        let tokens: Vec<&str> = line.split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name Scylla");
                println!("id author mitchella");
                println!("uciok"); 
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".parse::<Board>().unwrap();
                tt.clear(); // Wipe her memory for the new game
            }
            "position" => {
                parse_position(&tokens, &mut board);
            }
            "go" => {
                let mut wtime: u128 = 0;
                let mut btime: u128 = 0;
                let mut winc: u128 = 0;
                let mut binc: u128 = 0;

                let parts: Vec<&str> = line.split_whitespace().collect();
                for i in 0..parts.len() {
                    match parts[i] {
                        "wtime" => wtime = parts[i + 1].parse().unwrap_or(0),
                        "btime" => btime = parts[i + 1].parse().unwrap_or(0),
                        "winc" => winc = parts[i + 1].parse().unwrap_or(0),
                        "binc" => binc = parts[i + 1].parse().unwrap_or(0),
                        _ => {}
                    }
                }

                let (my_time, my_inc) = if board.side_to_move == types::color::Color::White {
                    (wtime, winc)
                } else {
                    (btime, binc)
                };

                // --- UPGRADED TIME MANAGEMENT ---
                let mut allocated_time = 5000;

                if my_time > 0 {
                    allocated_time = (my_time / 30) + (my_inc * 3 / 4);
                    
                    let hard_limit = (my_time / 7).max(10);
                    allocated_time = allocated_time.min(hard_limit).max(10);

                    let piece_count = board.occupancy().count_ones();
                    if piece_count <= 6 {
                        allocated_time += allocated_time / 2;
                    }

                    allocated_time = allocated_time.min(my_time.saturating_sub(20));
                }

                // Pass time_limit BEFORE tt to match the new Searcher refactor!
                if let Some(best_move) = search::iterative_deepening(&mut board, 100, allocated_time, &mut tt) {
                    println!("bestmove {}", best_move);
                } else {
                    println!("bestmove 0000"); // Checkmate/Stalemate fallback
                }
            }
            "d" | "display" => {
                println!("{}", board);
            }
            "quit" => {
                break;
            }
            _ => {}
        }
    }
}

fn parse_position(tokens: &[&str], board: &mut Board) {
    if tokens.len() < 2 {
        return;
    }

    let mut move_idx = tokens.len();
    if let Some(idx) = tokens.iter().position(|t| *t == "moves") {
        move_idx = idx;
    }

    if tokens[1] == "startpos" {
        *board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".parse::<Board>().unwrap();
    } else if tokens[1] == "fen" {
        let mut fen = tokens[2..move_idx].join(" ");
        
        let parts_count = fen.split_whitespace().count();
        if parts_count == 4 {
            fen.push_str(" 0 1");
        } else if parts_count == 5 {
            fen.push_str(" 1");
        }

        if let Ok(new_board) = fen.parse::<Board>() {
            *board = new_board;
        } else {
            println!("info string engine parsing desync! could not parse FEN: {}", fen);
        }
    }

    // Now apply moves if they exist
    if move_idx < tokens.len() {
        for move_str in &tokens[(move_idx + 1)..] {
            let moves = movegen::generator::generate_pseudo_legal_moves(board);
            let mut found = false;
            
            for m in moves.as_slice() {
                if m.to_string() == *move_str || m.to_string().to_lowercase() == move_str.to_lowercase() {
                    board.make_move(*m);
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("info string engine parsing desync! could not find move: {}", move_str);
            }
        }
    }
}
