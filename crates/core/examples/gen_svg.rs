use rnadraw_core::svg::{DEFAULT_NUCLEOTIDE_COLORS, SvgOptions};

fn main() {
    let structures = [
        ("hairpin", "(((...)))", "GGGAAACCC"),
        ("nick", "((.+.))", "GGAACCC"),
        (
            "multiloop",
            "(((..((...))..((..))...)))",
            "GGGAAGGGAAACCCAGGAACCCCAAACCC",
        ),
    ];

    let opts = SvgOptions {
        show_labels: true,
        base_colors: Some(DEFAULT_NUCLEOTIDE_COLORS.map(|s| s.into())),
        ..SvgOptions::default()
    };

    for (name, structure, seq) in &structures {
        let svg = rnadraw_core::draw_svg(structure, Some(seq), &opts);
        let path = format!("examples/{}.svg", name);
        std::fs::write(&path, &svg).unwrap();
        println!("wrote {} ({} bytes)", path, svg.len());
    }
}
