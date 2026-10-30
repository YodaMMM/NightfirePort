//! band-settle probe: the SP body anim's root track (the walked-dy candidate).
//! Assertion-free; run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore]
fn probe_sp_body_script_root_tracks() {
    let cold = nf_view::load_with_carry(&iso(), "0700000a.bin", false, None).expect("load");
    let sim = cold.sim.as_ref().unwrap();
    // The SP set [M PlayerAnimSetInitNormal 0x18fcd8:26 — MPSettings+0x384==0 → uVar3=999 →
    // set 0x2b91b0; the ELF array read: all entries 0x06000a00 [D ACTION.ELF @0x2b91b0]].
    let set: [u32; 3] = [0x06000a00; 3];
    for hash in set {
        match sim.scripts.get(&hash) {
            Some(sc) => {
                eprintln!("script 0x{hash:08x}: length {} commands {}", sc.length, sc.commands.len());
                for c in &sc.commands {
                    eprintln!("  op {} args {:?}", c.op, c.args);
                }
            }
            None => eprintln!("script 0x{hash:08x}: NOT LOADED"),
        }
    }
    // Dump the root track of every sequence the script references, plus the near-neighbours.
    let mut seq_hashes: Vec<u32> = sim.seqs.keys().copied().collect();
    seq_hashes.sort();
    eprintln!("{} sequences loaded", seq_hashes.len());
    for h in &seq_hashes {
        if *h != 0x84f { continue; }
        let s = &sim.seqs[h];
        let mut ys = Vec::new();
        for f in 1..=s.frames.min(64) {
            ys.push(nf_game::anim::root_translation_at(s, f as f32)[1]);
        }
        eprintln!("seq 0x{h:08x}: frames {} root-y {:?}", s.frames, ys);
    }
}
