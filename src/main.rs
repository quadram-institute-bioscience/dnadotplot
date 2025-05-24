use clap::{Arg, Command};
use bio::io::fasta;
use image::{ImageBuffer, Luma};
use std::fs::File;
use std::io::{BufReader, Read};
use flate2::read::GzDecoder;
use std::path::Path;

#[derive(Debug)]
struct Config {
    first_file: String,
    second_file: Option<String>,
    first_name: Option<String>,
    second_name: Option<String>,
    output: String,
    width: f64,
    window: usize,
    revcompl: bool,
    svg: bool,
}

fn main() {
    let matches = Command::new("dnadotplot")
        .version("0.1.0")
        .about("A DNA dot plot generator")
        .arg(Arg::new("first-file")
            .short('1')
            .long("first-file")
            .value_name("FILE")
            .help("Path to the first FASTA file (can be gzipped)")
            .required(true))
        .arg(Arg::new("second-file")
            .short('2')
            .long("second-file")
            .value_name("FILE")
            .help("Path to the second FASTA file (if omitted, self alignment of the first)"))
        .arg(Arg::new("first-name")
            .short('f')
            .long("first-name")
            .value_name("STR")
            .help("Name of the FASTA sequence in the first file"))
        .arg(Arg::new("second-name")
            .short('s')
            .long("second-name")
            .value_name("STR")
            .help("Name of the FASTA sequence in the second file"))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("FILE_PNG")
            .help("Path to the output file (PNG)")
            .required(true))
        .arg(Arg::new("width")
            .short('w')
            .long("width")
            .value_name("FLOAT")
            .help("Image size: if >1 use as pixels, if <1 use as fraction of longest sequence")
            .default_value("0.3"))
        .arg(Arg::new("window")
            .long("window")
            .value_name("INT")
            .help("Window size for matching (default: 10)")
            .default_value("10"))
        .arg(Arg::new("revcompl")
            .short('r')
            .long("revcompl")
            .help("Also look for reverse complement matches")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("svg")
            .long("svg")
            .help("Output as SVG with coordinate axes instead of PNG")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let config = Config {
        first_file: matches.get_one::<String>("first-file").unwrap().clone(),
        second_file: matches.get_one::<String>("second-file").map(|s| s.clone()),
        first_name: matches.get_one::<String>("first-name").map(|s| s.clone()),
        second_name: matches.get_one::<String>("second-name").map(|s| s.clone()),
        output: matches.get_one::<String>("output").unwrap().clone(),
        width: matches.get_one::<String>("width").unwrap().parse().expect("Invalid width value"),
        window: matches.get_one::<String>("window").unwrap().parse().expect("Invalid window value"),
        revcompl: matches.get_flag("revcompl"),
        svg: matches.get_flag("svg"),
    };

    if let Err(e) = run(config) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let seq1 = read_fasta(&config.first_file, config.first_name.as_deref())?;
    let seq2 = if let Some(ref file) = config.second_file {
        read_fasta(file, config.second_name.as_deref())?
    } else {
        seq1.clone()
    };

    let dotplot = generate_dotplot(&seq1, &seq2, config.width, config.window, config.revcompl)?;
    if config.svg {
        save_svg(&dotplot, &config.output, seq1.len(), seq2.len())?;
    } else {
        save_image(&dotplot, &config.output)?;
    }
    
    println!("Dot plot saved to {}", config.output);
    Ok(())
}

fn read_fasta(filepath: &str, seq_name: Option<&str>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let file = File::open(filepath)?;
    let reader: Box<dyn Read> = if filepath.ends_with(".gz") {
        Box::new(GzDecoder::new(BufReader::new(file)))
    } else {
        Box::new(BufReader::new(file))
    };

    let mut fasta_reader = fasta::Reader::new(reader);
    
    for result in fasta_reader.records() {
        let record = result?;
        if let Some(name) = seq_name {
            if record.id() == name {
                return Ok(record.seq().to_vec());
            }
        } else {
            return Ok(record.seq().to_vec());
        }
    }
    
    Err(format!("Sequence not found in {}", filepath).into())
}

fn window_match(seq1: &[u8], seq2: &[u8], pos1: usize, pos2: usize, window: usize) -> bool {
    if pos1 + window > seq1.len() || pos2 + window > seq2.len() {
        return false;
    }
    
    for i in 0..window {
        if seq1[pos1 + i].to_ascii_uppercase() != seq2[pos2 + i].to_ascii_uppercase() {
            return false;
        }
    }
    true
}

fn window_match_revcompl(seq1: &[u8], seq2: &[u8], pos1: usize, pos2: usize, window: usize) -> bool {
    if pos1 + window > seq1.len() || pos2 + window > seq2.len() {
        return false;
    }
    
    for i in 0..window {
        let base1 = seq1[pos1 + i].to_ascii_uppercase();
        let base2 = seq2[pos2 + window - 1 - i].to_ascii_uppercase();
        let base2_comp = match base2 {
            b'A' => b'T',
            b'T' => b'A',
            b'G' => b'C',
            b'C' => b'G',
            _ => base2,
        };
        if base1 != base2_comp {
            return false;
        }
    }
    true
}

fn generate_dotplot(seq1: &[u8], seq2: &[u8], width_param: f64, window: usize, revcompl: bool) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let max_len = seq1.len().max(seq2.len()) as f64;
    let image_size = if width_param > 1.0 {
        width_param as usize
    } else {
        (max_len * width_param) as usize
    };
    
    let mut matrix = vec![vec![255u8; image_size]; image_size];
    
    // Scan through sequences with sliding window (step size = 1)
    for seq1_pos in 0..=(seq1.len().saturating_sub(window)) {
        for seq2_pos in 0..=(seq2.len().saturating_sub(window)) {
            // Check for forward match
            if window_match(seq1, seq2, seq1_pos, seq2_pos, window) {
                // Map sequence positions to image coordinates with proper rounding
                let img_x = ((seq1_pos as f64 / seq1.len() as f64 * image_size as f64) + 0.5) as usize;
                let img_y = ((seq2_pos as f64 / seq2.len() as f64 * image_size as f64) + 0.5) as usize;
                if img_x < image_size && img_y < image_size {
                    matrix[img_y][img_x] = 0;
                }
            }
            
            // Check for reverse complement match
            if revcompl && window_match_revcompl(seq1, seq2, seq1_pos, seq2_pos, window) {
                let img_x = ((seq1_pos as f64 / seq1.len() as f64 * image_size as f64) + 0.5) as usize;
                let img_y = ((seq2_pos as f64 / seq2.len() as f64 * image_size as f64) + 0.5) as usize;
                if img_x < image_size && img_y < image_size {
                    matrix[img_y][img_x] = 128;
                }
            }
        }
    }
    
    Ok(matrix)
}

fn save_image(matrix: &[Vec<u8>], output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let height = matrix.len() as u32;
    let width = matrix[0].len() as u32;
    
    let mut img = ImageBuffer::new(width, height);
    
    for (y, row) in matrix.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            img.put_pixel(x as u32, y as u32, Luma([pixel]));
        }
    }
    
    img.save(output_path)?;
    Ok(())
}

fn save_svg(matrix: &[Vec<u8>], output_path: &str, seq1_len: usize, seq2_len: usize) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::fs::File;
    
    let matrix_size = matrix.len();
    let margin = 80.0;
    let plot_size = 400.0;
    let svg_size = plot_size + 2.0 * margin;
    
    let mut file = File::create(output_path)?;
    
    // SVG header
    writeln!(file, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    writeln!(file, "<svg width=\"{:.0}\" height=\"{:.0}\" xmlns=\"http://www.w3.org/2000/svg\">", svg_size, svg_size)?;
    writeln!(file, "<defs>")?;
    writeln!(file, "<style>")?;
    writeln!(file, ".axis {{ stroke: #000; stroke-width: 1; }}")?;
    writeln!(file, ".grid {{ stroke: #ccc; stroke-width: 0.5; }}")?;
    writeln!(file, ".label {{ font-family: Arial, sans-serif; font-size: 12px; fill: #000; }}")?;
    writeln!(file, ".title {{ font-family: Arial, sans-serif; font-size: 14px; fill: #000; text-anchor: middle; }}")?;
    writeln!(file, "</style>")?;
    writeln!(file, "</defs>")?;
    writeln!(file, "")?;
    writeln!(file, "<!-- Background -->")?;
    writeln!(file, "<rect width=\"{:.0}\" height=\"{:.0}\" fill=\"white\"/>", svg_size, svg_size)?;
    writeln!(file, "")?;
    writeln!(file, "<!-- Plot area background -->")?;
    writeln!(file, "<rect x=\"{:.0}\" y=\"{:.0}\" width=\"{:.0}\" height=\"{:.0}\" fill=\"white\" stroke=\"#000\" stroke-width=\"1\"/>", margin, margin, plot_size, plot_size)?;
    
    // Draw dots
    for (y, row) in matrix.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            if pixel < 255 {
                let svg_x = margin + (x as f64 / matrix_size as f64) * plot_size;
                let svg_y = margin + (y as f64 / matrix_size as f64) * plot_size;
                let color = if pixel == 0 { "#ff0000" } else { "#008000" };
                writeln!(file, "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"0.8\" fill=\"{}\"/>", svg_x, svg_y, color)?;
            }
        }
    }
    
    // X-axis ticks and labels
    let num_ticks = 9;
    for i in 0..=num_ticks {
        let frac = i as f64 / num_ticks as f64;
        let x_pos = margin + frac * plot_size;
        let seq_pos = (frac * seq1_len as f64) as usize;
        
        // Tick line
        writeln!(file, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" class=\"axis\"/>", 
                x_pos, margin + plot_size, x_pos, margin + plot_size + 5.0)?;
        
        // Grid line
        if i > 0 && i < num_ticks {
            writeln!(file, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" class=\"grid\"/>", 
                    x_pos, margin, x_pos, margin + plot_size)?;
        }
        
        // Label
        writeln!(file, "<text x=\"{:.1}\" y=\"{:.1}\" class=\"label\" text-anchor=\"middle\">{}</text>", 
                x_pos, margin + plot_size + 20.0, seq_pos / 10 * 10)?;
    }
    
    // Y-axis ticks and labels  
    for i in 0..=num_ticks {
        let frac = i as f64 / num_ticks as f64;
        let y_pos = margin + frac * plot_size;
        let seq_pos = (frac * seq2_len as f64) as usize;
        
        // Tick line
        writeln!(file, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" class=\"axis\"/>", 
                margin - 5.0, y_pos, margin, y_pos)?;
        
        // Grid line
        if i > 0 && i < num_ticks {
            writeln!(file, "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" class=\"grid\"/>", 
                    margin, y_pos, margin + plot_size, y_pos)?;
        }
        
        // Label
        writeln!(file, "<text x=\"{:.1}\" y=\"{:.1}\" class=\"label\" text-anchor=\"end\">{}</text>", 
                margin - 10.0, y_pos + 4.0, seq_pos / 10 * 10)?;
    }
    
    // Axis labels
    writeln!(file, "<text x=\"{:.1}\" y=\"{:.1}\" class=\"title\">Sequence 1 (nucleotide position)</text>", 
            margin + plot_size / 2.0, svg_size - 20.0)?;
    
    writeln!(file, "<text x=\"{:.1}\" y=\"{:.1}\" class=\"title\" transform=\"rotate(-90 {} {})\">Sequence 2 (nucleotide position)</text>", 
            20.0, margin + plot_size / 2.0, 20.0, margin + plot_size / 2.0)?;
    
    // Close SVG
    writeln!(file, "</svg>")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_dotplot_identical() {
        let seq1 = b"ATCGATCGATCG";
        let seq2 = b"ATCGATCGATCG";
        let result = generate_dotplot(seq1, seq2, 4.0, 1, false).unwrap();
        
        assert_eq!(result.len(), 4);
        assert_eq!(result[0][0], 0);
        assert_eq!(result[1][1], 0);
        assert_eq!(result[2][2], 0);
        assert_eq!(result[3][3], 0);
    }

    #[test]
    fn test_generate_dotplot_different() {
        let seq1 = b"AAAAAAAAAAAAA";
        let seq2 = b"CCCCCCCCCCCCC";
        let result = generate_dotplot(seq1, seq2, 4.0, 1, false).unwrap();
        
        assert_eq!(result.len(), 4);
        for row in result {
            for pixel in row {
                assert_eq!(pixel, 255);
            }
        }
    }

    #[test]
    fn test_width_calculation() {
        let seq1 = b"ATCGATCGATCGATCG";
        let seq2 = b"ATCGATCGATCGATCG";
        
        let result_pixels = generate_dotplot(seq1, seq2, 8.0, 1, false).unwrap();
        assert_eq!(result_pixels.len(), 8);
        
        let result_fraction = generate_dotplot(seq1, seq2, 0.5, 1, false).unwrap();
        assert_eq!(result_fraction.len(), 8);
    }
    
    
    #[test]
    fn test_window_match() {
        let seq1 = b"ATCGATCGATCG";
        let seq2 = b"ATCGATCGATCG";
        
        assert!(window_match(seq1, seq2, 0, 0, 3));
        assert!(!window_match(seq1, seq2, 0, 1, 3));
        assert!(window_match(seq1, seq2, 3, 3, 3));
    }
    
    #[test]
    fn test_window_match_revcompl() {
        let seq1 = b"ATG";
        let seq2 = b"CAT";
        
        assert!(window_match_revcompl(seq1, seq2, 0, 0, 3));
        
        let seq1 = b"ATCG";
        let seq2 = b"CGAT";
        assert!(window_match_revcompl(seq1, seq2, 0, 0, 4));
    }
}