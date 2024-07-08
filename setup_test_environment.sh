#!/bin/bash

# Cleanup previous test environment
echo "Cleaning up previous test environment..."
if [ -d "test_folder" ]; then
    rm -rf test_folder
    echo "Removed test_folder"
fi

# if [ -f "rules/example.yaml" ]; then
#     rm -f rules/example.yaml
#     echo "Removed rules/example.yaml"
# fi

# Setup new test environment
echo "Setting up new test environment..."
mkdir -p test_folder/Desktop
mkdir -p test_folder/Downloads
mkdir -p test_folder/Music
mkdir -p test_folder/Videos
mkdir -p test_folder/Documents
mkdir -p test_folder/Pictures/Wallpapers
mkdir -p test_folder/Pictures/Screenshots

# Create test files on Desktop with different timestamps
touch -t 202401010101.01 test_folder/Desktop/file1.txt
touch -t 202302020202.02 test_folder/Desktop/file2.mp3
touch -t 202303030303.03 test_folder/Desktop/file3.mp4
touch -t 202304040404.04 test_folder/Desktop/file4.pdf
touch -t 202305050505.05 test_folder/Desktop/file5.jpg
touch -t 202306060606.06 test_folder/Desktop/file6.png
touch -t 202307070707.07 test_folder/Desktop/file7.gif
touch -t 202308080808.08 test_folder/Desktop/file8.flac
touch -t 202309090909.09 test_folder/Desktop/file9.mov
touch -t 202310101010.10 test_folder/Desktop/file10.docx
touch -t 202311111111.11 test_folder/Desktop/file11.aac
touch -t 202312121212.12 test_folder/Desktop/wallpaper1.jpg
touch -t 202301010101.01 test_folder/Desktop/wallpaper2.png
touch -t 202211111111.11 test_folder/Desktop/screenshot1.png
touch -t 202210101010.10 test_folder/Desktop/screenshot2.jpg
touch -t 202212121212.12 test_folder/Desktop/clearshot1.jpg
touch -t 202213131313.13 test_folder/Desktop/clearshot2.png
touch -t 202212121212.12 test_folder/Desktop/infinite_loop.txt

echo "Setup complete."
