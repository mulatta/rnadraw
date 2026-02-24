use rnadraw_core::svg::SvgOptions;

fn main() {
    let opts = SvgOptions::default();

    // 1. Large multiloop (5-way junction)
    let multi5 = "(((...)))(((...)))(((...)))(((...)))(((...)))";
    let svg = rnadraw_core::draw_svg(multi5, None, &opts);
    std::fs::write("/tmp/test_multi5.svg", &svg).unwrap();
    println!("multi5: {} bytes, empty={}", svg.len(), svg.is_empty());

    // 2. 8-way junction (stress test)
    let multi8 = "((..))((..))((..))((..))((..))((..))((..))((..))";
    let svg = rnadraw_core::draw_svg(multi8, None, &opts);
    std::fs::write("/tmp/test_multi8.svg", &svg).unwrap();
    println!("multi8: {} bytes, empty={}", svg.len(), svg.is_empty());

    // 3. Pseudoknot notation
    let pk = "((([[[)))]]].";
    let svg = rnadraw_core::draw_svg(pk, None, &opts);
    println!(
        "pseudoknot '((([[[)))]]].': {} bytes, empty={}",
        svg.len(),
        svg.is_empty()
    );

    // Try other pseudoknot brackets
    let pk2 = "((..[[[..))..]]]";
    let svg2 = rnadraw_core::draw_svg(pk2, None, &opts);
    println!(
        "pseudoknot2 '((..[[[..))..]]]': {} bytes, empty={}",
        svg2.len(),
        svg2.is_empty()
    );

    // 4. External loop with dangling ends (5' and 3')
    let dangle = "...(((...))).....";
    let svg = rnadraw_core::draw_svg(dangle, None, &opts);
    std::fs::write("/tmp/test_dangle.svg", &svg).unwrap();
    println!("dangle: {} bytes, empty={}", svg.len(), svg.is_empty());

    // 5' only
    let dangle5 = "..(((...)))";
    let svg = rnadraw_core::draw_svg(dangle5, None, &opts);
    println!("dangle5': {} bytes, empty={}", svg.len(), svg.is_empty());

    // 3' only
    let dangle3 = "(((...)))..";
    let svg = rnadraw_core::draw_svg(dangle3, None, &opts);
    println!("dangle3': {} bytes, empty={}", svg.len(), svg.is_empty());

    // 5. Multi-strand (nick)
    let nick1 = "(((+)))";
    let svg = rnadraw_core::draw_svg(nick1, None, &opts);
    std::fs::write("/tmp/test_nick1.svg", &svg).unwrap();
    println!("nick simple: {} bytes, empty={}", svg.len(), svg.is_empty());

    // Multi-strand with unpaired
    let nick2 = "((..+..))";
    let svg = rnadraw_core::draw_svg(nick2, None, &opts);
    std::fs::write("/tmp/test_nick2.svg", &svg).unwrap();
    println!(
        "nick unpaired: {} bytes, empty={}",
        svg.len(),
        svg.is_empty()
    );

    // 3-strand complex
    let nick3 = "((+.+))";
    let svg = rnadraw_core::draw_svg(nick3, None, &opts);
    std::fs::write("/tmp/test_nick3.svg", &svg).unwrap();
    println!(
        "nick 3-strand: {} bytes, empty={}",
        svg.len(),
        svg.is_empty()
    );

    // 6. Large structure (~120nt)
    let large = "((((((((((....)))))))))).((((((((((.....))))))))))..((((((((((....)))))))))).((((((((((.....))))))))))";
    let svg = rnadraw_core::draw_svg(large, None, &opts);
    std::fs::write("/tmp/test_large.svg", &svg).unwrap();
    println!(
        "large ~100nt: {} bytes, empty={}",
        svg.len(),
        svg.is_empty()
    );

    // 7. Deeply nested
    let deep = "((((((((((((((((((((....))))))))))))))))))))";
    let svg = rnadraw_core::draw_svg(deep, None, &opts);
    std::fs::write("/tmp/test_deep.svg", &svg).unwrap();
    println!("deep nested: {} bytes, empty={}", svg.len(), svg.is_empty());
}
