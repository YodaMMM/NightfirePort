//! One-shot census: for the 07000007 BondTalk tables, does each line's facial script carry an
//! op-1 sound command, and does its sfx id equal the entry's sfxId? [M AnimScriptPeekSfx
//! 0x1c6ca8 walks the same commands]. Opt-in: NF_ISO.
use nf_game::player::Input;

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let voices = sim.voices.as_deref().expect("voices");
    // the AnimScriptPeekSfx fold: the first op-1 command's sfx id [M 0x1c6ca8]
    let op1 = |h: u32| -> u32 {
        sim.scripts.get(&h).and_then(|s| s.commands.iter()
            .find(|c| c.op == 1)
            .and_then(|c| c.args.get(1)).copied()).unwrap_or(0) as u32
    };
    let ranges: [(usize, usize, &str); 5] = [
        (0x58, 0x84, "set 0xd guests (0x58..0x83)"),
        (0x84, 0x8d, "set 0xe (0x84..0x8c)"),
        (0x8e, 0x8f, "set 0x11 (0x8e)"),
        (0x95, 0x97, "set 9 (0x95/0x96)"),
        (0x97, 0x9b, "set 0x10 (0x97..0x9a)"),
    ];
    for (lo, hi, name) in ranges {
        println!("== {name}");
        for i in lo..hi {
            let l = &voices[i];
            let present = sim.scripts.contains_key(&l.facial);
            let o = op1(l.facial);
            println!("line 0x{i:02x} sfx {:>4} facial 0x{:08x} present {present:>5} op1_sfx {o:>4} eq {}", l.sfx, l.facial, o == l.sfx as u32);
        }
    }
}
