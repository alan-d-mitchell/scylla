use std::cmp::Reverse;
use board::Board;
use types::mov::Move;
use types::piece::PieceType;
use movegen::{generator, notation};
use eval::evaluate;
use tt::{TranspositionTable, FLAG_EXACT, FLAG_ALPHA, FLAG_BETA};
use std::time::Instant;

pub const INFINITY: i32 = 2_000_000;
const MATE_VALUE: i32 = 100_000;
const MATE_THRESHOLD: i32 = 90_000;

// =========================================
// MOVE SCORING (MVV-LVA)
// =========================================

fn piece_val(pt: PieceType) -> i32 {
    match pt {
        PieceType::Pawn => 100, PieceType::Knight => 300,
        PieceType::Bishop => 300, PieceType::Rook => 500,
        PieceType::Queen => 900, PieceType::King => 0,
    }
}

pub fn score_move(board: &Board, m: Move) -> i32 {
    let mut score = 0;
    if m.is_capture() {
        let attacker = board.piece_at(m.from_sq()).unwrap_or(PieceType::Pawn);
        let victim = board.piece_at(m.to_sq()).unwrap_or(PieceType::Pawn); 
        score += 10000 + (piece_val(victim) * 10) - piece_val(attacker);
    }
    if m.is_promo() {
        score += 9000;
    }
    score
}

// =========================================
// THE SEARCHER STRUCT
// =========================================

pub struct Searcher<'a> {
    pub tt: &'a mut TranspositionTable,
    pub killers: [[Option<Move>; 2]; 128],
    pub start_time: Instant,
    pub time_limit: u128,
    pub nodes: u64,
    pub aborted: bool,
}

impl<'a> Searcher<'a> {
    pub fn new(tt: &'a mut TranspositionTable, time_limit: u128) -> Self {
        Self {
            tt,
            killers: [[None; 2]; 128],
            start_time: Instant::now(),
            time_limit,
            nodes: 0,
            aborted: false,
        }
    }

    pub fn search_position(&mut self, board: &mut Board, depth: u8, ply: u8) -> Option<Move> {
        let mut best_move = None;
        let mut alpha = -INFINITY;
        let beta = INFINITY;

        let pseudo_moves = generator::generate_pseudo_legal_moves(board);
        let mut moves = pseudo_moves.as_slice().to_vec();

        let tt_move = self.tt.probe(board.zobrist_key).and_then(|entry| entry.best_move);

        moves.sort_unstable_by_key(|&m| {
            if Some(m) == tt_move {
                Reverse(30000) 
            } else {
                Reverse(score_move(board, m)) 
            }
        });

        let us = board.side_to_move;
        let enemy_king_bb = board.pieces[us.flip() as usize][PieceType::King as usize].0;

        for m in moves {
            if (enemy_king_bb & (1u64 << m.to_sq().0)) != 0 { continue; }

            let undo = board.make_move(m);
            let king_sq = board.get_king_square(us);
            
            if !generator::is_square_attacked(board, king_sq, us.flip()) {
                
                let score = -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha);
                
                board.unmake_move(m, undo);

                if self.aborted { return None; }

                if score > alpha {
                    alpha = score;
                    best_move = Some(m);
                }
            } else {
                board.unmake_move(m, undo);
            }
        }

        if !self.aborted {
            self.tt.record(board.zobrist_key, depth, alpha, FLAG_EXACT, best_move);
        }

        best_move
    }

    fn alpha_beta(&mut self, board: &mut Board, depth: u8, ply: u8, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes & 2047 == 0 {
            if self.start_time.elapsed().as_millis() > self.time_limit {
                self.aborted = true;
                return 0; 
            }
        }
        if self.aborted { return 0; }

        if board.halfmove_clock >= 100 || is_repetition(board) {
            return 0; 
        }

        let alpha_orig = alpha;
        if let Some(entry) = self.tt.probe(board.zobrist_key) {
            if entry.depth >= depth {
                let mut score = entry.score;
                if score > MATE_THRESHOLD { score -= ply as i32; }
                else if score < -MATE_THRESHOLD { score += ply as i32; }

                if entry.flag == FLAG_EXACT { return score; }
                if entry.flag == FLAG_ALPHA && score <= alpha { return alpha; }
                if entry.flag == FLAG_BETA && score >= beta { return beta; }
            }
        }

        if depth == 0 {
            return self.quiescence_search(board, alpha, beta);
        }

        let us = board.side_to_move;
        let king_sq = board.get_king_square(us);
        let in_check = generator::is_square_attacked(board, king_sq, us.flip());

        let r = 2; 
        let is_endgame = board.occupancy().count_ones() <= 6;
        if depth >= 3 && !in_check && !is_endgame && board.en_passant.is_none() {
            let original_ep = board.en_passant;
            
            board.side_to_move = board.side_to_move.flip();
            board.en_passant = None;
            
            let null_score = -self.alpha_beta(board, depth - 1 - r, ply + 1, -beta, -beta + 1);
            
            board.side_to_move = board.side_to_move.flip();
            board.en_passant = original_ep;

            if self.aborted { return 0; }
            if null_score >= beta {
                return beta;
            }
        }

        let pseudo_moves = generator::generate_pseudo_legal_moves(board);
        let enemy_king_bb = board.pieces[us.flip() as usize][PieceType::King as usize].0;
        
        let mut moves = pseudo_moves.as_slice().to_vec();
        let tt_move = self.tt.probe(board.zobrist_key).and_then(|entry| entry.best_move);
        let p = (ply as usize).min(127);
        
        // Extract killers to avoid borrowing `self` inside the sorting closure
        let primary_killer = self.killers[p][0];
        let secondary_killer = self.killers[p][1];

        moves.sort_unstable_by_key(|&m| {
            if Some(m) == tt_move {
                Reverse(30000) 
            } else if m.is_capture() || m.is_promo() {
                Reverse(score_move(board, m))
            } else if Some(m) == primary_killer {
                Reverse(9000)
            } else if Some(m) == secondary_killer {
                Reverse(8000)
            } else {
                Reverse(0)
            }
        });

        let mut best_score = -INFINITY;
        let mut best_move = None;
        let mut legal_moves_played = 0;

        for m in moves {
            if (enemy_king_bb & (1u64 << m.to_sq().0)) != 0 { continue; }

            let undo = board.make_move(m);
            let current_king_sq = board.get_king_square(us);
            
            if !generator::is_square_attacked(board, current_king_sq, us.flip()) {
                legal_moves_played += 1;
                
                let score = -self.alpha_beta(board, depth - 1, ply + 1, -beta, -alpha);
                
                board.unmake_move(m, undo);
                
                if self.aborted { return 0; }

                if score > best_score {
                    best_score = score;
                    best_move = Some(m);
                }

                if score > alpha { alpha = score; }
                
                if alpha >= beta {
                    if !m.is_capture() && primary_killer != Some(m) {
                        self.killers[p][1] = self.killers[p][0];
                        self.killers[p][0] = Some(m);
                    }
                    break; 
                } 
            } else {
                board.unmake_move(m, undo);
            }
        }

        if legal_moves_played == 0 {
            if in_check {
                return -MATE_VALUE + ply as i32; 
            } else {
                return 0; 
            }
        }

        let flag = if best_score <= alpha_orig { FLAG_ALPHA }
                   else if best_score >= beta { FLAG_BETA }
                   else { FLAG_EXACT };
                   
        let mut tt_score = best_score;
        if tt_score > MATE_THRESHOLD { tt_score += ply as i32; }
        else if tt_score < -MATE_THRESHOLD { tt_score -= ply as i32; }
                   
        self.tt.record(board.zobrist_key, depth, tt_score, flag, best_move);

        best_score
    }

    fn quiescence_search(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        if self.nodes & 2047 == 0 && self.start_time.elapsed().as_millis() > self.time_limit {
            self.aborted = true;
            return 0;
        }
        if self.aborted { return 0; }

        let stand_pat = evaluate(board);
        
        if stand_pat >= beta { return beta; }
        if stand_pat > alpha { alpha = stand_pat; }

        let pseudo_moves = generator::generate_pseudo_legal_moves(board);
        let us = board.side_to_move;
        let enemy_king_bb = board.pieces[us.flip() as usize][PieceType::King as usize].0;
        
        let mut captures: Vec<Move> = pseudo_moves.as_slice().iter().copied()
            .filter(|m| m.is_capture() || m.is_promo())
            .collect();

        captures.sort_unstable_by_key(|&m| Reverse(score_move(board, m)));

        for m in captures {
            if (enemy_king_bb & (1u64 << m.to_sq().0)) != 0 { continue; }

            let undo = board.make_move(m);
            let king_sq = board.get_king_square(us);
            
            if !generator::is_square_attacked(board, king_sq, us.flip()) {
                let score = -self.quiescence_search(board, -beta, -alpha);
                board.unmake_move(m, undo);
                
                if self.aborted { return 0; }

                if score >= beta { return beta; }
                if score > alpha { alpha = score; }
            } else {
                board.unmake_move(m, undo);
            }
        }

        alpha
    }
}

// =========================================
// PRINCIPAL VARIATION & HELPERS
// =========================================

pub fn extract_pv(board: &mut Board, tt: &TranspositionTable, max_length: u8) -> Vec<String> {
    let mut pv = Vec::new();
    let mut undo_history = Vec::new();

    for _ in 0..max_length {
        if let Some(entry) = tt.probe(board.zobrist_key) {
            if let Some(best_move) = entry.best_move {
                let san = notation::move_to_san(board, best_move);
                pv.push(san); 

                let undo = board.make_move(best_move);
                undo_history.push((best_move, undo));
                continue;
            }
        }
        break; 
    }

    while let Some((m, undo)) = undo_history.pop() {
        board.unmake_move(m, undo);
    }
    pv
}

fn is_repetition(board: &Board) -> bool {
    let start = board.history.len().saturating_sub(board.halfmove_clock as usize);
    for i in start..board.history.len() {
        if board.history[i] == board.zobrist_key { return true; }
    }
    false
}

// =========================================
// ITERATIVE DEEPENING
// =========================================
pub fn iterative_deepening(board: &mut Board, max_depth: u8, time_limit: u128, tt: &mut TranspositionTable) -> Option<Move> {
    let mut best_move_overall = None;
    
    // Initialize our new Searcher!
    let mut searcher = Searcher::new(tt, time_limit);

    for d in 1..=max_depth {
        let current_best = searcher.search_position(board, d, 0);

        if searcher.aborted { break; }

        best_move_overall = current_best;

        let score = if let Some(entry) = searcher.tt.probe(board.zobrist_key) { entry.score } else { 0 };
        let pv_strings = extract_pv(board, searcher.tt, d);
        let pv_line = pv_strings.join(" ");
        let elapsed = searcher.start_time.elapsed().as_millis();

        let score_string = if score.abs() > MATE_THRESHOLD {
            let mate_in_plies = MATE_VALUE - score.abs();
            let mate_in_moves = (mate_in_plies + 1) / 2;
            let sign = if score > 0 { "" } else { "-" };
            format!("mate {}{}", sign, mate_in_moves)
        } else {
            format!("cp {}", score)
        };

        println!("info depth {} time {} score {} pv {}", d, elapsed, score_string, pv_line);

        if score > MATE_THRESHOLD {
            let mate_in_plies = (MATE_VALUE - score.abs()) as u8;
            if d >= mate_in_plies { break; }
        }

        if elapsed > (time_limit / 2) {
            break; 
        }
    }

    best_move_overall
}
