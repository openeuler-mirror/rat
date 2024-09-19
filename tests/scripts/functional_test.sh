#!/bin/bash

generate_random_file() {
    filename=$1
    size=$2
    dd if=/dev/urandom of="$filename" bs=1 count="$size" status=none
}


RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Function to compare the output of programs rat and cat
compare_rat_and_cat() {
    # Store all passed arguments to be used by the programs
    args=("$@")

    # Print the arguments before running the test, with a dividing line
    echo -e "${YELLOW}========================================"
    echo -e "Running test with arguments: ${args[*]}"
    echo -e "========================================${NC}"

    # Create temporary files to store the output
    # tmp_rat=$(mktemp)
    # tmp_cat=$(mktemp)
    tmp_rat=a
    tmp_cat=b
    # Execute rat program and store its output in the temporary file
    target/debug/rat "${args[@]}" > "$tmp_rat"
    exit_code_rat=$?  # Capture the exit code of the rat program

    # Execute cat program and store its output in the temporary file
    cat "${args[@]}" > "$tmp_cat"
    exit_code_cat=$?  # Capture the exit code of the cat program

    # Check if the exit codes of both programs are the same
    if [ $exit_code_rat -ne $exit_code_cat ]; then
        echo -e "${RED}Error: Programs exited with different codes.${NC}"
        echo -e "${RED}rat exit code: $exit_code_rat, cat exit code: $exit_code_cat${NC}"
        rm "$tmp_rat" "$tmp_cat"
        return 1
    fi

    # Compare the contents of the temporary files
    if ! cmp -s "$tmp_rat" "$tmp_cat"; then
        echo -e "${RED}Error: Outputs differ.${NC}"
        echo -e "${YELLOW}--- rat output ---${NC}"
        cat "$tmp_rat"
        echo -e "${YELLOW}--- cat output ---${NC}"
        cat "$tmp_cat"
        rm "$tmp_rat" "$tmp_cat"
        return 1
    else
        echo -e "${GREEN}Success: Outputs are identical.${NC}"
    fi

    # Clean up temporary files
    rm "$tmp_rat" "$tmp_cat"
    return 0
}

generate_random_file "random_file" 2048
compare_rat_and_cat random_file

generate_random_file "random_file" $(expr 1024)
compare_rat_and_cat random_file

generate_random_file "random_file" $(expr 10 \* 1024 \* 1024)
compare_rat_and_cat random_file

compare_rat_and_cat random_file -A
compare_rat_and_cat random_file -b
compare_rat_and_cat random_file -e
compare_rat_and_cat random_file -E
compare_rat_and_cat random_file -n
compare_rat_and_cat random_file -s
compare_rat_and_cat random_file -t
compare_rat_and_cat random_file -T
compare_rat_and_cat random_file -v

compare_rat_and_cat random_file -A -b -e -E
compare_rat_and_cat random_file -n -s -t -T
compare_rat_and_cat random_file -v -A -b -e -E -n
compare_rat_and_cat random_file -s -t -T -v -A -b
compare_rat_and_cat random_file -e -E -n -s -t -T -v
compare_rat_and_cat random_file -A -b -e -E -n -s -t -T -v
