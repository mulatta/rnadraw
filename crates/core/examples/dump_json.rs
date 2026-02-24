fn main() {
    let json = rnadraw_core::draw_structure("(((...)))");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let bases = &v["layout"]["bases"];
    println!("=== BASES ===");
    for (i, b) in bases.as_array().unwrap().iter().enumerate() {
        println!(
            "base[{}]: x={:.4} y={:.4} xt={:.4} yt={:.4} a1={:.4} a2={:.4} l1={:.4} l2={:.4}",
            i,
            b["x"],
            b["y"],
            b["xt"],
            b["yt"],
            b["angle1"],
            b["angle2"],
            b["length1"],
            b["length2"]
        );
    }
    println!("\n=== PAIRS ===");
    println!("{:?}", v["pairs"]);
    println!("\n=== NICKS ===");
    println!("{:?}", v["nicks"]);
    println!("\n=== SEGMENTS ===");
    for (i, s) in v["segments"].as_array().unwrap().iter().enumerate() {
        println!("seg[{}]: {:?}", i, s);
    }
}
