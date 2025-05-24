# DNA Dot Plot Generator

A Rust tool for generating dot plot alignments between DNA sequences from FASTA files.

![DNA Dot plot](./tests/page/generated-imgs/seq1-seq2-wnd10-w0.3.png)

## Installation

Currently, the project requires Rust and Cargo to build. Follow these steps to install:

```bash
cargo build --release
```

## Usage

```bash
./target/release/dnadotplot -1 <file1.fa> [-2 <file2.fa>] -o <output.png> [options]
```

### Options

- `-1`, `--first-file`: First FASTA file (required)
- `-2`, `--second-file`: Second FASTA file (optional, defaults to self-alignment)
- `-o`, `--output`: Output PNG file (required)
- `-s`, `--img-size`: Image size - pixels if >1, fraction of sequence length if <1 (default: 0.3)
- `-w`, `--window`: Window size for alignment (default: 10)
- `--revcompl`: Use reverse complement of the first sequence
- `-f`, `--first-name`: Specific sequence name from first file
- `-n`, `--second-name`: Specific sequence name from second file

## Examples

```bash
# Self-alignment of E. coli genome
./target/release/dnadotplot -1 input/ecoli.fa -o ecoli_self.png -s 1000

# Cross-alignment between two genomes
./target/release/dnadotplot -1 input/bbreve.fa -2 input/ecoli.fa -o comparison.png

# Use fraction of sequence length for image size
./target/release/dnadotplot -1 genome.fa -o plot.png -s 0.5
```

The output is a grayscale PNG where black pixels indicate exact nucleotide matches and white pixels indicate no matches.

## Testing and Comparison System

To regenerate test images and create a visual comparison with reference images:

```bash
# Generate comparison images and HTML report
python3 generate_comparison.py
```

This command will:

1. Parse reference images in `tests/page/img/` with format `{seq1}-{seq2}-wnd{window}-w{width_ratio}.{format}`
2. Generate corresponding images using dnadotplot with `--revcompl` flag
3. Save generated images to `tests/page/generated-imgs/`
4. Create an HTML comparison page at `tests/page/index.html`

The system is flexible - new reference images following the naming convention will automatically be included in the comparison when the script is run.