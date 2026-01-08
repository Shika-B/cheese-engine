#!/bin/sh

DIR="executables/"

# Get the two most recently modified files
LATEST_FILES=$(ls -t "$DIR" | head -n 2)
FILE1=$(echo "$LATEST_FILES" | sed -n '1p')
FILE2=$(echo "$LATEST_FILES" | sed -n '2p')

echo "Latest engine: $DIR$FILE1"
echo "Second latest engine: $DIR$FILE2"

fastchess \
    -engine cmd="$DIR$FILE1" name="NewEngine" \
    -engine cmd="$DIR$FILE2" name="OldEngine" \
    -pgnout file="games.pgn" \
    -openings file=8moves_v3.pgn format=pgn order=random \
    -each tc=10+0.1 \
    -rounds 100 -repeat \
    -concurrency 8 \
    -recover \
    -sprt elo0=0 elo1=5 alpha=0.05 beta=0.1

