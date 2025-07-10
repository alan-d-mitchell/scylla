#ifndef DATAGEN_H
#define DATAGEN_H

#include "board.h"

// The main function to start the self-play data generation process.
void generate_training_data(int num_games);

int is_draw(Board* board);

#endif // DATAGEN_H