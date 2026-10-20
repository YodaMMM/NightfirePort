//! Scratch probe (not committed): which `MP_skins` hashcodes (ACTION.ELF 0x26d488, 29 x 16 B) resolve in an MP arena's
//! loaded skins. Usage: cargo run --release -p nf-view --example probe_mp_skins -- <iso> 07000023
const MP_SKINS: [u32; 29] = [
    0x5000089, 0x50000a5, 0x500008e, 0x5000092, 0x5000095, 0x5000096, 0x5000097, 0x5000098, 0x500009a, 0x50000c6,
    0x500009e, 0x50000a0, 0x50000a4, 0x500008b, 0x50000bb, 0x500001c, 0x500001d, 0x500001f, 0x5000056, 0x5000060,
    0x5000063, 0x5000086, 0x5000081, 0x5000082, 0x5000084, 0x500006b, 0x5000085, 0x500007b, 0x50000b9,
];

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let scene = nf_view::load(&a[1], &format!("{}.bin", a[2]), false).expect("load");
    let sim = scene.sim.as_ref().expect("sim");
    let mut have = 0;
    for (i, h) in MP_SKINS.iter().enumerate() {
        let s = sim.skins.get(h);
        if s.is_some() { have += 1; }
        println!("{i:2} {h:#010x} {}", s.map_or("MISSING".to_string(), |s| format!("ok skeleton {:?}", s.skeleton)));
    }
    println!("{have}/29 resolve; characters in scene: {}", scene.characters.len());
}
