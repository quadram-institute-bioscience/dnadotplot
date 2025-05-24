#!/usr/bin/env python3

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