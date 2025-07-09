#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>

// --- Basic Type Definitions ---
typedef uint64_t u64;

typedef struct {
    u64 mask;
    u64 magic;
    u64* ptr;
    int shift;
} SMagic;


// --- Rook-specific Data ---
SMagic mRookTbl[64];
u64 rook_attack_table[102400]; // Rooks require a larger attack table

// --- Constant Masks ---
const u64 board_edges = 0xFF818181818181FFULL;

// --- Helper Functions ---
unsigned int prng_state;

void seed_prng(unsigned int seed) {
    prng_state = seed;
}

unsigned int rand_prng() {
    prng_state ^= prng_state << 13;
    prng_state ^= prng_state >> 17;
    prng_state ^= prng_state << 5;
    return prng_state;
}

u64 rand64_prng() {
    u64 r1 = (u64)rand_prng();
    u64 r2 = (u64)rand_prng();
    return r1 | (r2 << 32);
}

u64 sparse_rand_u64() {
    return rand64_prng() & rand64_prng() & rand64_prng();
}

int popcount(u64 bb) {
    int count = 0;
    while (bb) {
        bb &= bb - 1;
        count++;
    }
    return count;
}

u64 generate_rook_attacks(int square, u64 blockers) {
    u64 attacks = 0ULL;
    int r, f, tr = square / 8, tf = square % 8;
    for (r = tr + 1; r <= 7; r++) { attacks |= (1ULL << (r * 8 + tf)); if ((1ULL << (r * 8 + tf)) & blockers) break; }
    for (r = tr - 1; r >= 0; r--) { attacks |= (1ULL << (r * 8 + tf)); if ((1ULL << (r * 8 + tf)) & blockers) break; }
    for (f = tf + 1; f <= 7; f++) { attacks |= (1ULL << (tr * 8 + f)); if ((1ULL << (tr * 8 + f)) & blockers) break; }
    for (f = tf - 1; f >= 0; f--) { attacks |= (1ULL << (tr * 8 + f)); if ((1ULL << (tr * 8 + f)) & blockers) break; }
    return attacks;
}

u64 generate_rook_blocker_mask(int square) {
    // For rooks, the relevant blockers are on the rank and file, excluding the edges.
    u64 attacks = 0ULL;
    int r, f, tr = square / 8, tf = square % 8;
    for (r = tr + 1; r < 7; r++) attacks |= (1ULL << (r * 8 + tf));
    for (r = tr - 1; r > 0; r--) attacks |= (1ULL << (r * 8 + tf));
    for (f = tf + 1; f < 7; f++) attacks |= (1ULL << (tr * 8 + f));
    for (f = tf - 1; f > 0; f--) attacks |= (1ULL << (tr * 8 + f));
    return attacks;
}


// --- Magic Finding Function (with logging) ---
void init_magic_rook_attacks() {
    u64* attack_table_ptr = rook_attack_table;
    // Using a known-good set of seeds for rooks
    int seeds[] = {8977, 44560, 54343, 38998, 5731, 95205, 104912, 17020};
    
    printf("Starting magic number generation for rooks...\n");

    for (int s = 0; s < 64; s++) {
        printf("Finding magic for square %d...\n", s);

        int epoch[4096] = {0};
        int cnt = 0;

        mRookTbl[s].mask = generate_rook_blocker_mask(s);
        mRookTbl[s].ptr = attack_table_ptr;
        
        int relevant_bits = popcount(mRookTbl[s].mask);
        mRookTbl[s].shift = 64 - relevant_bits;
        int size = 1 << relevant_bits;
        
        u64 occupancy[4096];
        u64 reference[4096];
        
        u64 b = 0;
        for(int i = 0; i < size; i++) {
            occupancy[i] = b;
            reference[i] = generate_rook_attacks(s, b);
            b = (b - mRookTbl[s].mask) & mRookTbl[s].mask;
        }

        seed_prng(seeds[s % 8]);
        
        int attempts = 0;
        while (1) {
            attempts++;
            mRookTbl[s].magic = sparse_rand_u64();
            if (popcount((mRookTbl[s].magic * mRookTbl[s].mask) >> 56) < 6) continue;
            
            ++cnt;
            int i;
            for (i = 0; i < size; ++i) {
                unsigned idx = (occupancy[i] * mRookTbl[s].magic) >> mRookTbl[s].shift;
                if (epoch[idx] < cnt) {
                    epoch[idx] = cnt;
                    mRookTbl[s].ptr[idx] = reference[i];
                } else if (mRookTbl[s].ptr[idx] != reference[i]) {
                    break;
                }
            }

            if (i == size) {
                printf("  -> Found magic for square %d in %d attempts!\n", s, attempts);
                break;
            }

            // Failsafe to prevent a true infinite loop
            if (attempts > 100000000) {
                 printf("\nERROR: Could not find magic for square %d after 100 million attempts. Aborting.\n", s);
                 exit(1);
            }
        }
        attack_table_ptr += size;
    }
    printf("\nSuccessfully generated all rook magic numbers!\n");
}


// --- Main Function to Run the Test ---
int main() {
    init_magic_rook_attacks();
    return 0;
}