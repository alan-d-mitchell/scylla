use types::mov::Move;

// TT Flags indicate the "type" of score we stored.
pub const FLAG_NONE: u8 = 0;
pub const FLAG_EXACT: u8 = 1; // We know the exact evaluation of the position
pub const FLAG_ALPHA: u8 = 2; // Upper bound (The position is worse than this score)
pub const FLAG_BETA: u8 = 3;  // Lower bound (The position is better than this score)

/// A single entry in the Transposition Table.
/// Carefully sized to 16 bytes for perfect memory alignment and cache efficiency.
#[derive(Copy, Clone, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
}

impl TTEntry {
    pub fn empty() -> Self {
        Self {
            key: 0,
            best_move: None,
            score: 0,
            depth: 0,
            flag: FLAG_NONE,
        }
    }
}

pub struct TranspositionTable {
    pub entries: Vec<TTEntry>,
    size_mask: usize,
}

impl TranspositionTable {
    /// Creates a new Transposition Table allocated to a specific Megabyte size.
    pub fn new(megabytes: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let num_entries = (megabytes * 1024 * 1024) / entry_size;
        
        // Find the largest power of 2 less than or equal to num_entries.
        // We do this so we can use bitwise AND (&) for indexing instead of modulo (%).
        // Bitwise operations are significantly faster!
        let mut capacity = 1;
        while capacity * 2 <= num_entries {
            capacity *= 2;
        }
        
        let size_mask = capacity - 1;

        Self {
            entries: vec![TTEntry::empty(); capacity],
            size_mask,
        }
    }

    /// Clears the TT (useful between different games or long searches)
    pub fn clear(&mut self) {
        self.entries.fill(TTEntry::empty());
    }

    /// Probes the table for a given Zobrist key.
    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let index = (key as usize) & self.size_mask;
        let entry = self.entries[index];
        
        // Always double-check the key to prevent Hash Collisions
        if entry.key == key && entry.flag != FLAG_NONE {
            Some(entry)
        } else {
            None
        }
    }

    /// Records a new evaluation into the table.
    pub fn record(&mut self, key: u64, depth: u8, score: i32, flag: u8, best_move: Option<Move>) {
        let index = (key as usize) & self.size_mask;
        let entry = &mut self.entries[index];

        // Replacement Strategy: 
        // 1. If the slot is empty or holds a completely different position (hash collision), overwrite it.
        // 2. If it's the exact same position, only overwrite if we searched it to a deeper depth this time.
        if entry.key != key || depth >= entry.depth || entry.flag == FLAG_NONE {
            entry.key = key;
            entry.depth = depth;
            entry.score = score;
            entry.flag = flag;
            
            // Crucial: If our new calculation didn't find a best move, but the old one did, 
            // keep the old best move to help with move ordering later!
            if best_move.is_some() {
                entry.best_move = best_move;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tt_storage_and_retrieval() {
        // Allocate a tiny 1MB table for testing
        let mut tt = TranspositionTable::new(1);
        
        let mock_key_1 = 0x123456789ABCDEF0;
        let mock_key_2 = 0x0FEDCBA987654321;

        // 1. Record an exact evaluation
        tt.record(mock_key_1, 5, 300, FLAG_EXACT, None);

        // 2. Probe for the evaluation
        let result = tt.probe(mock_key_1);
        assert!(result.is_some(), "Failed to retrieve stored TT Entry");
        
        let entry = result.unwrap();
        assert_eq!(entry.score, 300);
        assert_eq!(entry.depth, 5);
        assert_eq!(entry.flag, FLAG_EXACT);

        // 3. Ensure a different key returns None (No hash collision)
        let miss = tt.probe(mock_key_2);
        assert!(miss.is_none(), "TT returned an entry for an unrecorded key");
    }

    #[test]
    fn test_tt_depth_replacement() {
        let mut tt = TranspositionTable::new(1);
        let key = 0x9999999999999999;

        // Engine searches at depth 3 and finds score 150
        tt.record(key, 3, 150, FLAG_EXACT, None);

        // Engine searches same position later at depth 5 and finds score 200
        tt.record(key, 5, 200, FLAG_EXACT, None);

        // TT should return the deeper evaluation
        let entry = tt.probe(key).unwrap();
        assert_eq!(entry.score, 200);
        assert_eq!(entry.depth, 5);
        
        // Engine does a shallow Depth 1 search of the same position
        tt.record(key, 1, 50, FLAG_EXACT, None);
        
        // TT should IGNORE the shallow search and keep the Depth 5 evaluation
        let entry = tt.probe(key).unwrap();
        assert_eq!(entry.score, 200, "TT overwrote a deep search with a shallow one!");
    }
}
