#!/usr/bin/env python3

import os
import re
import subprocess
import shutil
from pathlib import Path

def parse_filename(filename):
    """Parse filename format: {seq1}-{seq2}-wnd{window}-w{width_ratio}.{format}"""
    pattern = r'^(.+?)-(.+?)-wnd(\d+)-w(\d+(?:\.\d+)?)\.(.+)$'
    match = re.match(pattern, filename)
    if match:
        seq1, seq2, window, width_ratio, format_ext = match.groups()
        return {
            'seq1': seq1,
            'seq2': seq2,
            'window': int(window),
            'width_ratio': float(width_ratio),
            'format': format_ext,
            'filename': filename
        }
    return None

def find_input_file(seq_name, input_dir):
    """Find corresponding input file for sequence name"""
    possible_files = [f"{seq_name}.fa", f"{seq_name}.fasta"]
    for filename in possible_files:
        path = input_dir / filename
        if path.exists():
            return path
    return None

def generate_dotplot(seq1_file, seq2_file, window, width_ratio, output_file, is_self_comparison=False, output_format="png"):
    """Generate dotplot using the dnadotplot binary"""
    cmd = ["./target/release/dnadotplot", "--revcompl"]
    
    cmd.extend(["-1", str(seq1_file)])
    if not is_self_comparison:
        cmd.extend(["-2", str(seq2_file)])
    
    cmd.extend(["-w", str(width_ratio), "--window", str(window), "-o", str(output_file)])
    
    if output_format.lower() == "svg":
        cmd.append("--svg")
    
    print(f"Running: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"Error generating {output_file}: {result.stderr}")
        return False
    return True

def generate_html_page(comparisons, output_dir):
    """Generate HTML comparison page"""
    html_content = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Dotplot Comparison</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }
        .comparison {
            margin-bottom: 40px;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .comparison h2 {
            color: #333;
            border-bottom: 2px solid #007acc;
            padding-bottom: 10px;
        }
        .image-container {
            display: flex;
            gap: 20px;
            flex-wrap: wrap;
        }
        .image-section {
            flex: 1;
            min-width: 300px;
        }
        .image-section h3 {
            color: #555;
            margin-bottom: 10px;
        }
        .image-section img {
            max-width: 100%;
            height: auto;
            border: 1px solid #ddd;
            border-radius: 4px;
        }
        .params {
            background-color: #f8f9fa;
            padding: 10px;
            border-radius: 4px;
            font-family: monospace;
            margin-bottom: 15px;
        }
        .status {
            padding: 5px 10px;
            border-radius: 3px;
            font-weight: bold;
            display: inline-block;
            margin-bottom: 10px;
        }
        .success { background-color: #d4edda; color: #155724; }
        .error { background-color: #f8d7da; color: #721c24; }
    </style>
</head>
<body>
    <h1>Dotplot Comparison Results</h1>
    <p>Comparison between reference images and generated images using dnadotplot.</p>
"""

    for comp in comparisons:
        params = comp['params']
        status_class = "success" if comp['generated'] else "error"
        status_text = "Generated successfully" if comp['generated'] else "Generation failed"
        
        seq_info = f"{params['seq1']} vs {params['seq2']}" if params['seq1'] != params['seq2'] else f"{params['seq1']} (self-comparison)"
        
        html_content += f"""
    <div class="comparison">
        <h2>{seq_info}</h2>
        <div class="status {status_class}">{status_text}</div>
        <div class="params">
            Parameters: window={params['window']}, width_ratio={params['width_ratio']}, format={params['format']}
        </div>
        <div class="image-container">
            <div class="image-section">
                <h3>Reference Image</h3>
                <img src="img/{params['filename']}" alt="Reference: {params['filename']}">
            </div>"""
        
        if comp['generated']:
            html_content += f"""
            <div class="image-section">
                <h3>Generated Image</h3>
                <img src="generated-imgs/{comp['generated_filename']}" alt="Generated: {comp['generated_filename']}">
            </div>"""
        else:
            html_content += f"""
            <div class="image-section">
                <h3>Generated Image</h3>
                <p style="color: #721c24;">Failed to generate image</p>
            </div>"""
        
        html_content += """
        </div>
    </div>"""

    html_content += """
</body>
</html>"""

    with open(output_dir / "index.html", "w") as f:
        f.write(html_content)

def main():
    project_root = Path(".")
    input_dir = project_root / "input"
    ref_img_dir = project_root / "tests" / "page" / "img"
    generated_img_dir = project_root / "tests" / "page" / "generated-imgs"
    
    # Ensure generated images directory exists
    generated_img_dir.mkdir(parents=True, exist_ok=True)
    
    # Build the project first
    print("Building dnadotplot...")
    result = subprocess.run(["cargo", "build", "--release"], capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Build failed: {result.stderr}")
        return
    
    # Parse reference images
    ref_images = list(ref_img_dir.glob("*"))
    comparisons = []
    
    for ref_img_path in ref_images:
        if ref_img_path.is_file():
            params = parse_filename(ref_img_path.name)
            if not params:
                print(f"Skipping {ref_img_path.name}: doesn't match expected format")
                continue
            
            print(f"Processing {ref_img_path.name}...")
            
            # Find input files
            seq1_file = find_input_file(params['seq1'], input_dir)
            seq2_file = find_input_file(params['seq2'], input_dir)
            
            if not seq1_file:
                print(f"Warning: Could not find input file for {params['seq1']}")
                comparisons.append({'params': params, 'generated': False, 'generated_filename': None})
                continue
            
            is_self_comparison = params['seq1'] == params['seq2']
            if not is_self_comparison and not seq2_file:
                print(f"Warning: Could not find input file for {params['seq2']}")
                comparisons.append({'params': params, 'generated': False, 'generated_filename': None})
                continue
            
            # Generate output filename (keep original format)
            generated_filename = ref_img_path.name
            generated_path = generated_img_dir / generated_filename
            
            # Generate the dotplot
            success = generate_dotplot(
                seq1_file, 
                seq2_file if not is_self_comparison else seq1_file,
                params['window'],
                params['width_ratio'],
                generated_path,
                is_self_comparison,
                params['format']
            )
            
            comparisons.append({
                'params': params,
                'generated': success,
                'generated_filename': generated_filename if success else None
            })
    
    # Generate HTML comparison page
    print("Generating HTML comparison page...")
    generate_html_page(comparisons, project_root / "tests" / "page")
    
    print(f"Done! Generated {len([c for c in comparisons if c['generated']])} images")
    print("Open tests/page/index.html to view the comparison")

if __name__ == "__main__":
    main()