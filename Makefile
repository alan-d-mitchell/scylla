# Makefile for the Scylla Chess Engine

# --- Variables ---

# Compiler to use
CC = gcc

# Compiler flags
CFLAGS = -Wall -Wextra -O2 -g -Isrc

# --- Directories ---
# NEW: Define the output directory for binaries
BIN_DIR = bin
SRC_DIR = src
TEST_DIR = tests

# --- File Paths ---

# Source files for the main program
SRCS = scylla.c $(SRC_DIR)/bitboard.c

# Source files for the move generation test suite
MOVEGEN_TEST_SRCS = $(TEST_DIR)/movegen_test.c $(SRC_DIR)/movegen.c $(SRC_DIR)/bitboard.c

# --- Target Executables ---
# UPDATED: Prepend the binary directory path to all targets
TARGET = $(BIN_DIR)/scylla
MOVEGEN_TEST_TARGET = $(BIN_DIR)/run_movegen_tests


# --- Rules ---

# The default rule, builds the main program
all: $(TARGET)

# Rule to build the main program executable
# UPDATED: It now depends on the bin directory existing
$(TARGET): $(SRCS) | $(BIN_DIR)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRCS)

# Rule to build the move generation test suite
# UPDATED: It also depends on the bin directory existing
$(MOVEGEN_TEST_TARGET): $(MOVEGEN_TEST_SRCS) | $(BIN_DIR)
	$(CC) $(CFLAGS) -o $(MOVEGEN_TEST_TARGET) $(MOVEGEN_TEST_SRCS)

# NEW: A rule to create the bin directory.
# The '-p' flag means it won't error if the directory already exists.
# This is an "order-only prerequisite", so it only runs if the directory is missing.
$(BIN_DIR):
	mkdir -p $(BIN_DIR)

# Rule to run the tests
movegen_test: $(MOVEGEN_TEST_TARGET)
	./$(MOVEGEN_TEST_TARGET)

# Rule to clean up all compiled files
# UPDATED: Now removes the entire bin directory
clean:
	rm -rf $(BIN_DIR)

# Phony targets are rules that don't produce a file with the same name.
.PHONY: all movegen_test clean