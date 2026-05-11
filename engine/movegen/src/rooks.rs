use types::bitboard::Bitboard;
use types::square::Square;
use crate::magics::{ROOK_MAGICS, ROOK_SHIFTS};
use crate::masks::get_rook_mask;
use std::sync::OnceLock;
use std::time::Instant;
use std::io::Write;

// 102,400 is the exact number of table entries needed to store all 
// non-colliding magic hashes for a rook across all 64 squares.
static ROOK_ATTACKS: OnceLock<[Bitboard; 102400]> = OnceLock::new();

// We need an offset array to know where in the 102,400-length array 
// a specific square's sub-table begins.
static ROOK_OFFSETS: OnceLock<[usize; 64]> = OnceLock::new();

#[inline(always)]
pub fn get_rook_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    let relevant = blockers & get_rook_mask(sq);
    let hash = (relevant.0.wrapping_mul(ROOK_MAGICS[sq.0 as usize])) >> ROOK_SHIFTS[sq.0 as usize];
    
    let offset = ROOK_OFFSETS.get().unwrap()[sq.0 as usize];
    
    // Look up the pre-calculated attack board
    ROOK_ATTACKS.get().unwrap()[offset + hash as usize]
}

fn classical_rook_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let rank = (sq.0 / 8) as i8;
    let file = (sq.0 % 8) as i8;

    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // N, S, E, W

    for (df, dr) in directions {
        let mut r = rank;
        let mut f = file;

        loop {
            r += dr;
            f += df;
            if r < 0 || r > 7 || f < 0 || f > 7 { break; }
            
            let target = (r * 8 + f) as u8;
            attacks |= Bitboard(1u64 << target);
            
            if blockers.contains(target) { break; } // Stop if we hit a piece
        }
    }

    attacks
}

pub fn init_rook_magic_table() {
    let mut attacks = [Bitboard::EMPTY; 102400];
    let mut offsets = [0; 64];
    let mut current_offset = 0;

    for sq in 0..64 {
        offsets[sq] = current_offset;
        let mask = get_rook_mask(Square::new(sq as u8));
        let shift = ROOK_SHIFTS[sq];
        
        // The Carry-Rippler Trick
        let mut subset = 0u64;
        loop {
            // 1. Hash this specific subset of blockers
            let hash = (subset.wrapping_mul(ROOK_MAGICS[sq])) >> shift;
            let index = current_offset + hash as usize;
            
            // 2. Calculate the true attacks and store them at the hash index
            attacks[index] = classical_rook_attacks(Square::new(sq as u8), Bitboard(subset));

            // 3. Ripple to the next subset
            subset = subset.wrapping_sub(mask.0) & mask.0;
            if subset == 0 { break; }
        }
        
        // Advance the offset by the number of possible hashes for this square
        current_offset += 1 << (64 - shift);
    }
    
    // Store arrays in our global thread-safe variables
    ROOK_ATTACKS.set(attacks).expect("Rook attacks already initialized");
    ROOK_OFFSETS.set(offsets).expect("Rook offsets already initialized");
}


// ======================================================================
// BRUTE FORCER LOGIC
// ======================================================================

struct XorShift {
    state: u64,
}

impl XorShift {
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x1234567890ABCDEF } else { seed } }
    }

    fn rand(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn rand_fewbits(&mut self) -> u64 {
        self.rand() & self.rand() & self.rand()
    }
}

fn find_rook_magic(sq: u8, mask: u64, shift: usize) -> u64 {
    let mut prng = XorShift::new(1070372 + sq as u64);
    
    let bits = mask.count_ones();
    let num_subsets = 1 << bits;
    let mut subsets = vec![0u64; num_subsets];
    let mut attacks = vec![0u64; num_subsets];
    
    let mut subset = 0u64;
    for i in 0..num_subsets {
        subsets[i] = subset;
        attacks[i] = classical_rook_attacks(Square::new(sq), Bitboard(subset)).0;
        subset = subset.wrapping_sub(mask) & mask; 
    }

    let mut used = vec![u64::MAX; 1 << (64 - shift)];

    loop {
        let magic = prng.rand_fewbits();
        used.fill(u64::MAX); 
        let mut fail = false;

        for i in 0..num_subsets {
            let hash = (subsets[i].wrapping_mul(magic)) >> shift;
            let index = hash as usize;

            if used[index] == u64::MAX {
                used[index] = attacks[i];
            } else if used[index] != attacks[i] {
                fail = true;
                break;
            }
        }

        if !fail {
            return magic;
        }
    }
}

/// Call this function once to generate and print your custom arrays.
pub fn generate_and_print_rook_magics() {
    println!("Brute-forcing custom Rook Magics...");
    let start = Instant::now();

    let mut magics = [0u64; 64];
    let mut shifts = [0usize; 64];

    for sq in 0..64 {
        let mask = get_rook_mask(Square::new(sq as u8)).0;
        let bits = mask.count_ones() as usize;
        
        // Dynamically shift based on your exact mask
        let shift = 64 - bits; 
        
        shifts[sq] = shift;
        magics[sq] = find_rook_magic(sq as u8, mask, shift);
        
        print!(".");
        let _ = std::io::stdout().flush();
    }

    println!("\nDone in {:?}", start.elapsed());

    println!("\n// COPY THESE INTO YOUR magics.rs FILE");
    println!("pub const ROOK_SHIFTS: [usize; 64] = [");
    for (i, &s) in shifts.iter().enumerate() {
        print!("{:2}, ", s);
        if (i + 1) % 8 == 0 { println!(); }
    }
    println!("];\n");

    println!("pub const ROOK_MAGICS: [u64; 64] = [");
    for (i, &m) in magics.iter().enumerate() {
        print!("0x{:016x}, ", m);
        if (i + 1) % 4 == 0 { println!(); }
    }
    println!("];");
}
