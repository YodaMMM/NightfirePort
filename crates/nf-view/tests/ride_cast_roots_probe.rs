//! level-4a-ride-bond-float probe: the landing cut-scene's three anim scripts
//! (0x060006d0 = s3 guard, 0x060006d1 = s2 Bond, 0x060006d2 = s1 officer — skins
//! 0x0500004/0x0500007/0x0500006 [D 0700004a-audit §2]) — every opcode-0 window and each
//! windowed sequence's root channels (ch0/1/2) at its first and last frames. Answers where the
//! cast's roots are car-local and where world. Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn dump_the_cast_scripts_windows_and_roots() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    w.tick(nf_game::player::Input::default(), false);
    let sim = w.scene.sim.as_ref().unwrap();
    for (name, hash) in [("s3 guard", 0x0600_06d0u32), ("s2 BOND", 0x0600_06d1), ("s1 officer", 0x0600_06d2)] {
        let Some(sc) = sim.scripts.get(&hash) else { eprintln!("{name} {hash:08x}: MISSING"); continue };
        eprintln!("== {name} {hash:08x} length {} looping {}", sc.length, sc.looping);
        for (a, b, q) in sc.sequences() {
            let Some(seq) = sim.seqs.get(&q) else { eprintln!("  win {a}..{b} seq {q:08x} MISSING"); continue };
            let v = |ch: usize, fr: u32| seq.value(ch, fr);
            eprintln!(
                "  win {a}..{b} seq {q:08x} flags {:x} chans {} | ch0 f{a} {:.3} f{b} {:.3} | ch1 {:.3} {:.3} | ch2 {:.3} {:.3}",
                seq.flags, seq.channels.len(), v(0, a as u32), v(0, b as u32), v(1, a as u32), v(1, b as u32), v(2, a as u32), v(2, b as u32));
        }
    }
}
