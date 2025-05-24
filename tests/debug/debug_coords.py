#!/usr/bin/env python3
"""
debug_coords.py - DNA Dot Plot Coordinate Mapping Debugger

This debugging utility validates the coordinate transformation system used in 
DNA dot plot generation. It's part of the testing suite for the DNA Dot Plot 
project, specifically focused on ensuring accurate mapping between biological 
sequence positions and image pixel coordinates.

Functionality:
- Tests coordinate mapping for sliding window analysis (window size = 10)
- Validates that sequence positions map correctly to image coordinates
- Identifies potential gaps or overlaps in the coordinate space
- Ensures the rounding algorithm matches the Rust implementation

The script uses the same coordinate transformation as the main application:
img_coord = int((pos / seq_len * image_size) + 0.5)

This helps identify issues like:
- Missing image coordinates that should be populated
- Coordinate collisions where multiple sequence positions map to same pixel
- Boundary condition problems at sequence edges
"""
# Debug coordinate mapping
seq_len = 84
image_size = 84  # width_param = 1.0, so image_size = 84

print(f"Sequence length: {seq_len}")
print(f"Image size: {image_size}")
print(f"Valid sequence positions for window 10: 0 to {seq_len - 10}")

# Check coordinate mapping for each position
coords_used = set()
missing_coords = []

for pos in range(seq_len - 10 + 1):  # 0 to 74
    # Rust coordinate calculation with rounding fix
    img_coord = int((pos / seq_len * image_size) + 0.5)
    coords_used.add(img_coord)
    print(f"Seq pos {pos:2d} -> img coord {img_coord:2d}")

print(f"\nUnique image coordinates used: {sorted(coords_used)}")
print(f"Total coordinates used: {len(coords_used)}")
print(f"Expected coordinates: 0 to {int((seq_len - 10) / seq_len * image_size)}")

# Check for missing coordinates
expected_coords = set(range(int((seq_len - 10) / seq_len * image_size) + 1))
missing = expected_coords - coords_used
if missing:
    print(f"Missing image coordinates: {sorted(missing)}")
else:
    print("No missing coordinates")