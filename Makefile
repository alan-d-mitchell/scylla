# Makefile for the Scylla Chess Engine

# --- Variables ---

# Compiler to use
CC = gcc

# Compiler flags:
# -Wall -Wextra: Enable all common warnings
# -O2: Standard optimization level
# -g: Include debugging information
# -Isrc: Tell the compiler to look for headers in the 'src' directory
CFLAGS = -Wall -Wextra -O2 -g -Isrc

# Directories
SRC_DIR = src
TEST_DIR = tests

# Source files for the main program
# Note: We now specify the directory for each file
SRCS = scylla.c $(SRC_DIR)/bitboard.c

# Source files for the test suite
TEST_SRCS = $(TEST_DIR)/bitboard_test.c $(SRC_DIR)/bitboard.c

MOVEGEN_TEST_TARGET = run_movegen_tests
MOVEGEN_TEST_SRCS = tests/movegen_test.c src/movegen.c src/bitboard.c

# Name of the final executable for the main program
TARGET = scylla

# Name of the final executable for the test suite
TEST_TARGET = run_tests


# --- Rules ---

# The default rule, executed when you just type "make"
all: $(TARGET)

# Rule to build the main program executable
$(TARGET): $(SRCS)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRCS)

# Rule to build the test suite executable
$(TEST_TARGET): $(TEST_SRCS)
	$(CC) $(CFLAGS) -o $(TEST_TARGET) $(TEST_SRCS)

# A special rule to run the tests.
test: $(TEST_TARGET)
	./$(TEST_TARGET)

$(MOVEGEN_TEST_TARGET): $(MOVEGEN_TEST_SRCS)
	$(CC) $(CFLAGS) -o $(MOVEGEN_TEST_TARGET) $(MOVEGEN_TEST_SRCS)

movegen_test: $(MOVEGEN_TEST_TARGET)
	./$(MOVEGEN_TEST_TARGET)


# Rule to clean up all compiled files
clean:
	rm -f $(TARGET) $(TEST_TARGET) $(MOVEGEN_TEST_TARGET)

# Phony targets are rules that don't produce a file with the same name.
.PHONY: all test clean
