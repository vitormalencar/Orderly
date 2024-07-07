#!/bin/bash

# Cleanup previous test environment
echo "Cleaning up previous test environment..."
if [ -d "test_folder" ]; then
    rm -rf test_folder
    echo "Removed test_folder"
fi

if [ -f "rules/example.yaml" ]; then
    rm -f rules/example.yaml
    echo "Removed rules/example.yaml"
fi

# Setup new test environment
echo "Setting up new test environment..."
mkdir -p test_folder/organized
echo "Created directory: test_folder/organized"

touch test_folder/delete_me.png
echo "Created file: test_folder/delete_me.png"

# add test to when a move is needed but the destination folder doesn't exist
touch test_folder/move_me_missing_folder.png

# add test to when a move is needed and the destination folder exists
touch test_folder/move_me.png
echo "Created file: test_folder/move_me.png"

# add test to when a copy is needed but the destination folder doesn't exist
touch test_folder/copy_me_missing_folder.png

# add test to when a copy is needed and the destination folder exists
touch test_folder/copy_me.png
echo "Created file: test_folder/copy_me.png"

echo "Setup complete."
