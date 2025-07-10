#ifndef NN_ENCODE_H
#define NN_ENCODE_H

#include "defs.h"
#include <stdio.h>

// This function converts a single move into the 4672-element policy vector format.
// The chosen move gets a probability of 1.0, all others are 0.0.
void move_to_policy_vector(Move move, float* policy_vector);

#endif // NN_ENCODE_H