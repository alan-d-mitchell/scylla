use types::mov::Move;

#[derive(Clone, Debug)]
pub struct MoveList {
    pub moves: [Move; 256],
    pub count: usize,
}

impl MoveList {
    
    pub fn new() -> Self {
        Self {
            moves: [Move(0); 256],
            count: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    }
    
    /// Returns a slice of just the valid moves generated so far
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[0..self.count]
    }
}
