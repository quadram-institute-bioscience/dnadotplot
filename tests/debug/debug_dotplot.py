#!/usr/bin/env python3

# Simple debug script to check window matching logic
sequence = "CTTGGTCATTTAGAGGAAGTAAAAGTCGTAACAAGGTTTCCGTAGGTGAACCTGCGGAAGGATCATTAAAGAAATTTAATAATT"
window_size = 10

print(f"Sequence length: {len(sequence)}")
print(f"Window size: {window_size}")
print(f"Expected matches: 0 to {len(sequence) - window_size}")

# Check every possible self-match position
missing_positions = []
for pos in range(len(sequence) - window_size + 1):
    # Check if window at position matches itself
    window = sequence[pos:pos + window_size]
    if window == window:  # This should always be true for self-comparison
        continue
    else:
        missing_positions.append(pos)

if missing_positions:
    print(f"Missing positions: {missing_positions}")
else:
    print("All positions should match - no gaps expected in diagonal")

# Let's specifically check positions around 45 and 55
print("\nChecking specific positions:")
for pos in [44, 45, 46, 54, 55, 56]:
    if pos <= len(sequence) - window_size:
        window = sequence[pos:pos + window_size]
        print(f"Position {pos}: {window}")
    else:
        print(f"Position {pos}: Beyond sequence end")