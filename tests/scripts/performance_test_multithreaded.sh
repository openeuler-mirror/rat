#!/bin/bash

# =============================================================================
# performance_test_multithreaded.sh
# =============================================================================
# 
# This script measures and compares the performance of the `rat` and `cat` 
# commands in a multi-threaded environment. It runs both commands with the 
# same set of arguments, captures their execution times, and compares their 
# outputs to ensure correctness.
#

# Define some colors using ANSI escape codes for output formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to generate a random file of specified size
generate_random_file() {
    filename=$1
    size=$2
    base64 /dev/urandom | head -c $size > $filename
}

# Function to measure the execution time and output of a command
measure_time_and_output() {
    params=("$@")
    output_file=$1
    time_file=$2
    num_iterations=$3
    shift 3
    command=("$@")
    total_time=0

    for ((i=1; i<=num_iterations; i++)); do
        start_time=$(date +%s%N)
        eval ${command[@]} > "$output_file" 2>&1
        end_time=$(date +%s%N)
        elapsed_time=$(($end_time - $start_time))  # Convert nanoseconds to milliseconds
        total_time=$((total_time + elapsed_time))
    done

    average_time=$((total_time / num_iterations / 1000000))
    echo "$average_time" > "$time_file"
}

# Function to compare the output of programs rat and cat
compare_rat_and_cat() {
    # Store all passed arguments to be used by the programs
    args=("$@")

    # Define temporary files for storing outputs and execution times
    rat_output_file="rat_output_file"
    cat_output_file="cat_output_file"
    rat_time_file=$(mktemp)
    cat_time_file=$(mktemp)
    log_file="comparison_log.txt"

    # Number of iterations for timing measurements
    num_iterations=5

    # Print the arguments before running the test, with a dividing line
    echo -e "${YELLOW}========================================" | tee -a "$log_file"
    echo -e "Running test with arguments: ${args[*]}" | tee -a "$log_file"
    echo -e "========================================${NC}" | tee -a "$log_file"

    # Measure execution time and capture output for rat
    measure_time_and_output "$rat_output_file" "$rat_time_file" $num_iterations "rat ${args[@]}"
    time_rat=$(cat "$rat_time_file")

    # Measure execution time and capture output for cat
    measure_time_and_output "$cat_output_file" "$cat_time_file" $num_iterations "cat ${args[@]}"
    time_cat=$(cat "$cat_time_file")

    # Print execution times to the log file
    echo -e "${YELLOW}Execution times:${NC}" | tee -a "$log_file"
    echo -e "rat: ${time_rat}ms" | tee -a "$log_file"
    echo -e "cat: ${time_cat}ms" | tee -a "$log_file"

    # Check if the exit codes of both programs are the same
    exit_code_rat=$(grep "exit code:" "$rat_output_file" | awk '{print $3}')
    exit_code_cat=$(grep "exit code:" "$cat_output_file" | awk '{print $3}')

    if [ "$exit_code_rat" != "$exit_code_cat" ]; then
        echo -e "${RED}Error: Programs exited with different codes.${NC}" | tee -a "$log_file"
        echo -e "${RED}rat exit code: $exit_code_rat, cat exit code: $exit_code_cat${NC}" | tee -a "$log_file"
        return 1
    fi

    # Compare the contents of the output files
    if ! cmp -s "$rat_output_file" "$cat_output_file"; then
        echo -e "${RED}Error: Outputs differ.${NC}" | tee -a "$log_file"
        echo -e "${YELLOW}--- rat output ---${NC}" | tee -a "$log_file"
        cat "$rat_output_file" | tee -a "$log_file"
        echo -e "${YELLOW}--- cat output ---${NC}" | tee -a "$log_file"
        cat "$cat_output_file" | tee -a "$log_file"
        return 1
    else
        echo -e "${GREEN}Success: Outputs are identical.${NC}" | tee -a "$log_file"
    fi

    # Clean up temporary files
    rm "$rat_output_file" "$cat_output_file" "$rat_time_file" "$cat_time_file"

    return 0
}

# Generate random files of different sizes for testing
generate_random_file rf_256KB $(expr 256 \* 1024)
generate_random_file rf_32MB $(expr 32 \* 1024 \* 1024)
generate_random_file rf_256MB $(expr 256 \* 1024 \* 1024)

# Run tests with different arguments and file sizes

# Simple cat
# Small file
compare_rat_and_cat rf_256KB
# Medium file
compare_rat_and_cat rf_32MB
# Large file
compare_rat_and_cat rf_256MB

# With argument -n
# Small file
compare_rat_and_cat rf_256KB -n
# Medium file
compare_rat_and_cat rf_32MB -n
# Large file
compare_rat_and_cat rf_256MB -n

# With argument -E
# Small file
compare_rat_and_cat rf_256KB -E
# Medium file
compare_rat_and_cat rf_32MB -E
# Large file
compare_rat_and_cat rf_256MB -E

# With argument -v
# Small file
compare_rat_and_cat rf_256KB -v
# Medium file
compare_rat_and_cat rf_32MB -v
# Large file
compare_rat_and_cat rf_256MB -v

# With argument -T
# Small file
compare_rat_and_cat rf_256KB -T
# Medium file
compare_rat_and_cat rf_32MB -T
# Large file
compare_rat_and_cat rf_256MB -T

# With argument -A
# Small file
compare_rat_and_cat rf_256KB -A
# Medium file
compare_rat_and_cat rf_32MB -A
# Large file
compare_rat_and_cat rf_256MB -A

# Clean up generated files
rm rf_256KB rf_32MB rf_256MB
