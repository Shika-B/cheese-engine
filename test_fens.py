import subprocess
import sys
import tempfile
import os
from typing import List, Optional

# List of FEN positions to test
TEST_FENS = [
    # Starting position
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",

    # Middlegame positions
    "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
    "rnbqkb1r/ppp2ppp/4pn2/3p4/2PP4/2N2N2/PP2PPPP/R1BQKB1R w KQkq d6 0 5",

    # Endgame positions
    "8/8/8/4k3/8/8/4K3/8 w - - 0 1",
    "8/8/4k3/8/8/4K3/8/7R w - - 0 1",
    "8/8/8/3k4/3P4/3K4/8/8 w - - 0 1",# List of FEN positions to test

    # Tactical positions
    "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5Q2/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
    "rnbqkb1r/pp1p1ppp/2p2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq e6 0 4",
]


def run_game(
    engine_path: str,
    fen: str,
    time_control: str = "tc=10+0.1",
    rounds: int = 1,
    concurrency: int = 1,
    output_pgn: Optional[str] = None,
) -> subprocess.CompletedProcess:
    """
    Run a game using cutechess-cli with the engine playing against itself.

    Args:
        engine_path: Path to the chess engine executable
        fen: FEN string for the starting position
        time_control: Time control in cutechess-cli format (default: 10s + 0.1s increment)
        rounds: Number of rounds to play (default: 1)
        concurrency: Number of games to play concurrently (default: 1)
        output_pgn: Optional path to save PGN output

    Returns:
        CompletedProcess with the result of cutechess-cli
    """
    # Create a temporary EPD file with the FEN position
    with tempfile.NamedTemporaryFile(mode='w', suffix='.epd', delete=False) as f:
        # EPD format is just the first 4 fields of FEN (position, turn, castling, en passant)
        fen_parts = fen.split()
        epd = ' '.join(fen_parts[:4])
        f.write(epd + '\n')
        temp_file = f.name

    try:
        cmd = [
            "cutechess-cli",
            "-engine", f"cmd={engine_path}", "name=Engine1",
            "-engine", f"cmd={engine_path}", "name=Engine2",
            "-each", f"{time_control}", "proto=uci",
            "-rounds", str(rounds),
            "-concurrency", str(concurrency),
            "-wait", "10",  # Wait 10ms between games
            "-resign", "movecount=3", "score=800",  # Resign after 3 moves if down 800cp
            "-draw", "movenumber=40", "movecount=8", "score=10",  # Draw rules
            "-openings", f"file={temp_file}", "format=epd",
            "-repeat",  # Play each opening from both sides
        ]

        if output_pgn:
            cmd.extend(["-pgnout", output_pgn])

        # Add result output
        cmd.extend(["-ratinginterval", "1"])

        result = subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            timeout=300,  # 5 minute timeout per game
        )
        return result
    except subprocess.TimeoutExpired:
        print(f"Game timed out for FEN: {fen[:50]}...", file=sys.stderr)
        raise
    except FileNotFoundError:
        print("Error: cutechess-cli not found. Please install it first.", file=sys.stderr)
        print("Ubuntu/Debian: sudo apt install cutechess-cli", file=sys.stderr)
        print("Arch: sudo pacman -S cutechess", file=sys.stderr)
        print("Or build from source: https://github.com/cutechess/cutechess", file=sys.stderr)
        sys.exit(1)
    finally:
        # Clean up temp file
        if os.path.exists(temp_file):
            os.unlink(temp_file)


def main():
    """Main function to run tests on all FEN positions."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Test chess engine by playing from various FEN positions"
    )
    parser.add_argument(
        "engine",
        help="Path to the chess engine executable",
    )
    parser.add_argument(
        "--fens",
        help="Path to file containing FEN strings (one per line)",
        default=None,
    )
    parser.add_argument(
        "--time",
        help="Time control (e.g., tc=10+0.1 for 10s+0.1s)",
        default="tc=10+0.1",
    )
    parser.add_argument(
        "--rounds",
        type=int,
        default=2,
        help="Number of rounds per position (default: 2, plays each side once)",
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        default=1,
        help="Number of games to play concurrently (default: 1)",
    )
    parser.add_argument(
        "--output",
        help="Output PGN file path (default: test_results.pgn)",
        default="test_results.pgn",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print verbose output",
    )

    args = parser.parse_args()

    # Load FENs
    fens = TEST_FENS
    if args.fens:
        with open(args.fens, 'r') as f:
            fens = [line.strip() for line in f if line.strip() and not line.startswith('#')]

    print(f"Testing engine: {args.engine}")
    print(f"Number of positions: {len(fens)}")
    print(f"Time control: {args.time}")
    print(f"Rounds per position: {args.rounds}")
    print(f"Output PGN: {args.output}")
    print("-" * 60)

    total_games = 0
    failed_games = 0

    for i, fen in enumerate(fens, 1):
        print(f"\n[{i}/{len(fens)}] Testing position: {fen[:50]}...")

        try:
            result = run_game(
                engine_path=args.engine,
                fen=fen,
                time_control=args.time,
                rounds=args.rounds,
                concurrency=args.concurrency,
                output_pgn=args.output,
            )

            if result.returncode == 0:
                print(f"✓ Success")
                if args.verbose:
                    print(result.stdout)
            else:
                print(f"✗ Failed with return code {result.returncode}")
                print("STDERR:", result.stderr)
                failed_games += 1

            total_games += 1

        except subprocess.TimeoutExpired:
            failed_games += 1
            total_games += 1
            continue
        except KeyboardInterrupt:
            print("\n\nInterrupted by user.")
            break

    print("\n" + "=" * 60)
    print(f"Completed: {total_games - failed_games}/{total_games} positions")
    print(f"Failed: {failed_games}/{total_games} positions")
    print(f"Results saved to: {args.output}")


if __name__ == "__main__":
    main()
