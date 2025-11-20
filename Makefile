.PHONY: all test run clean build

# Default target: run the application
all: run

# Run the application
run:
	cargo run

# Run all tests
test:
	cargo test

# Build the project
build:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

