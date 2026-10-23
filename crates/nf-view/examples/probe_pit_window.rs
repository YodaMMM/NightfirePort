//! level-16-jetpack-escape-2 probe: play the launch chain in the port's sim with Bond parked at
//! the pit floor (probe-only placement) and log the window: ch 243 (the hurt gate), ch 240 (the
//! pit door), ch 80 (the shuttle-bay lift summon), the lift platform entity 02000a79's keyed
//! matrix, and Bond's health. The launch script's stream 6 writes ch 243=1 at stream tick 1611
//! and =0 at 2170 (absolute waits [M nis.rs:446-451; D dump]); stream 21 writes ch 80=1 at 2240;
//! stream 7 writes ch 240=1 at 2180. Opt-in via NF_ISO.
use nf_game::player::Input;

fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn main() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).expect("load");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        // Probe-only: stand Bond on the ch-0x81 bridge slab so the launch chain fires (the real
        // walk crosses it on foot; the probe watches the chain, not the approach).
        sim.bond.pos = [2.4, 1.2, -175.5];
    }
    let mut fired = [false; 4]; // 0x81, 243, 240, 80
    for t in 0..3600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let ch = |c: u16| sim.drones.channels.on(c as u32);
        if ch(0x81) && !fired[0] { fired[0] = true; eprintln!("t{t}: ch 0x81 fired (the launch chain head)"); }
        if ch(243) && !fired[1] { fired[1] = true; eprintln!("t{t}: ch 243 RISES (the volume goes live)"); }
        if !ch(243) && fired[1] && !fired[2] { fired[2] = true; eprintln!("t{t}: ch 243 DROPS (the pulse ends)"); }
        if ch(240) && !fired[2] { fired[2] = true; eprintln!("t{t}: ch 240 rises (the pit door / group-6 arm)"); }
        if ch(80) && !fired[3] { fired[3] = true; eprintln!("t{t}: ch 80 rises (the lift summon)"); }
        if t % 150 == 0 {
            let lift = sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
                nf_game::nis::Obj::Entity { hash: 0x0200_0a79, matrix, .. } => Some(matrix[3]),
                _ => None,
            });
            eprintln!("t{t}: bond({:.2},{:.2},{:.2}) hp {:.1} grounded {} lift {:?} ch243 {} ch240 {} ch80 {}",
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.drones.health.health, sim.bond.grounded,
                lift.map(|m| format!("({:.2},{:.2},{:.2})", m[0], m[1], m[2])), ch(243) as u8, ch(240) as u8, ch(80) as u8);
        }
        if sim.drones.health.health <= 0.0 { eprintln!("t{t}: BOND DEAD"); break; }
    }
}
