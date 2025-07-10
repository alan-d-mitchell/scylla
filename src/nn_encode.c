#include "nn_encode.h"
#include <string.h>

// This is a simplified encoding. A full implementation would be more complex.
// For now, we just map from/to squares. This is enough to get started.
// Total possible moves: 64 squares * 64 squares = 4096. Promotions add more.
// We will use a simple linear mapping for this example.
#define POLICY_SIZE 4672

void move_to_policy_vector(Move move, float* policy_vector) {
    // Initialize the entire vector to 0.0
    memset(policy_vector, 0, sizeof(float) * POLICY_SIZE);

    int index = -1;
    // Simple mapping: from_square * 64 + to_square
    // Note: This doesn't handle promotions yet, but is fine for bootstrapping.
    if (move.from >= 0 && move.from < 64 && move.to >= 0 && move.to < 64) {
        index = move.from * 64 + move.to;
    }

    if (index != -1 && index < POLICY_SIZE) {
        policy_vector[index] = 1.0f; // Set the chosen move's probability to 1.0
    }
}