use std::io::Write;
use std::path::PathBuf;
use std::process;

use clap::Parser;
use rnadraw_core::svg::{DEFAULT_NUCLEOTIDE_COLORS, Legend, SvgOptions};

#[derive(Clone, Copy, clap::ValueEnum)]
enum Format {
    Svg,
    Json,
}

/// RNA secondary structure SVG renderer
#[derive(Parser)]
#[command(name = "rnadraw", version)]
struct Cli {
    /// Dot-bracket-plus structure notation
    #[arg(short, long)]
    structure: String,

    /// RNA sequence (e.g. GGGAAACCC)
    #[arg(short = 'q', long)]
    sequence: Option<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = Format::Svg)]
    format: Format,

    /// Per-base equilibrium probabilities as comma-separated floats (0.0–1.0).
    /// Uses probability colormap. Mutually exclusive with --nucleotide.
    #[arg(short, long, value_delimiter = ',', conflicts_with = "nucleotide")]
    probabilities: Option<Vec<f64>>,

    /// Color by nucleotide type (A=green, U=red, G=black, C=blue).
    /// Used when --probabilities is not given.
    #[arg(short = 'c', long)]
    nucleotide: bool,

    /// Hide 3' direction arrows
    #[arg(long)]
    no_arrows: bool,

    /// Disable stem auto-alignment
    #[arg(long)]
    no_align: bool,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let output = match cli.format {
        Format::Json => {
            let json = rnadraw_core::draw_structure(&cli.structure);
            if json.is_empty() {
                eprintln!("error: invalid structure or empty result");
                process::exit(1);
            }
            json
        }
        Format::Svg => {
            let mut opts = SvgOptions {
                show_arrows: !cli.no_arrows,
                align_stem: !cli.no_align,
                ..SvgOptions::default()
            };

            if cli.probabilities.is_some() {
                opts.probabilities = cli.probabilities;
            } else if cli.nucleotide {
                opts.base_colors = Some(DEFAULT_NUCLEOTIDE_COLORS.map(String::from));
                opts.legend = Legend::Nucleotide;
            } else {
                eprintln!("error: specify --probabilities or --nucleotide for SVG color mode");
                process::exit(1);
            }

            let svg = rnadraw_core::draw_svg(&cli.structure, cli.sequence.as_deref(), &opts);
            if svg.is_empty() {
                eprintln!("error: invalid structure or empty result");
                process::exit(1);
            }
            svg
        }
    };

    if let Some(path) = cli.output {
        if let Err(e) = std::fs::write(&path, &output) {
            eprintln!("error: failed to write {}: {e}", path.display());
            process::exit(1);
        }
    } else {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        if let Err(e) = handle.write_all(output.as_bytes()) {
            eprintln!("error: write failed: {e}");
            process::exit(1);
        }
    }
}
