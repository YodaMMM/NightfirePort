//! One-shot: the shape of the talk scripts the guests attach (0600037d was seen on d7,
//! 06000113 on d23): sequences() entries, seq flags/channels/frames, live weights at frames.
//! Opt-in: NF_ISO.
fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let sim = scene.sim.as_ref().unwrap();
    let voices = sim.voices.as_deref().unwrap();
    for (i, l) in voices.iter().enumerate() {
        if l.facial != 0x0600_037d && l.facial != 0x0600_0113 { continue; }
        println!("line {i} (0x{i:02x}) sfx {} facial {:08x}", l.sfx, l.facial);
        let sc = sim.scripts.get(&l.facial).expect("script present");
        for (a, b, q) in sc.sequences() {
            match sim.seqs.get(&q) {
                Some(sq) => {
                    let samples: Vec<String> = [1u32, 5, 10, 20, 30].iter()
                        .map(|f| format!("f{f}:live{}", (0..sq.channels.len()).filter(|c| sq.value(*c, *f).abs() > 0.01).count()))
                        .collect();
                    println!("  win start {a} b {b} seq {q:08x} flags {:x} ch {} frames {} [{}]", sq.flags, sq.channels.len(), sq.frames, samples.join(" "));
                }
                None => println!("  win start {a} b {b} seq {q:08x} MISSING from scene seqs"),
            }
        }
        println!("  commands: {} ops {:?}", sc.commands.len(), sc.commands.iter().map(|c| c.op).collect::<Vec<_>>());
    }
}
