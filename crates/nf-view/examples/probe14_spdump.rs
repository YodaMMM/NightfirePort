//! Scratch probe (level-14-lid-and-climb): dump every ScriptPlayer placement in 07000014 —
//! BOTH script slots, modes, mask/out/gate, position — and fully decode the lid scripts'
//! streams and key tables (the keys are what Script_GetInterp poses the created entity with).
//! Assertion-free.
use nf_formats::nis::Cmd;

fn dump_script(sim: &nf_view::scene::Sim, hash: u32) {
    let Some(nis) = sim.drones.scripts.lib.get(&hash) else {
        println!("  script {hash:08x}: NOT in lib");
        return;
    };
    println!(
        "  script {hash:08x}: camera {} loop_start {} length {} streams {} keys {}",
        nis.camera,
        nis.loop_start,
        nis.length,
        nis.streams.len(),
        nis.keys.len()
    );
    for (si, st) in nis.streams.iter().enumerate() {
        println!(
            "   stream {si}: keys {}..{} commands {}",
            st.first_key, st.last_key, st.commands.len()
        );
        for c in st.decode() {
            println!("     {c:?}");
        }
        // The stream's key slice: what Script_GetInterp samples into the entity matrix.
        let n = nis.keys.len() as u16;
        if st.first_key < n || st.last_key < n {
            let nu = nis.keys.len();
            let lo = (st.first_key as usize).min(nu);
            let hi = ((st.last_key as usize) + 1).min(nu);
            for k in &nis.keys[lo..hi] {
                println!(
                    "       key t{:.2}: pos ({:.3}, {:.3}, {:.3}, {:.3}) scale {:.3} w28 {:.3}",
                    k.time, k.pos[0], k.pos[1], k.pos[2], k.pos[3], k.scale, k.w28
                );
            }
        }
    }
}

fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();

    println!("== all ScriptPlayers ==");
    let mut script_hashes = std::collections::BTreeSet::new();
    for (i, p) in sim.drones.scripts.players.iter().enumerate() {
        let (x, y, z) = (p.matrix[3][0], p.matrix[3][1], p.matrix[3][2]);
        println!(
            "players[{i}]: scripts {:?} modes {:?} mask {:#06x} out {} gate {} stay {} at ({x:.3}, {y:.3}, {z:.3})",
            p.scripts, p.modes, p.mask, p.out, p.gate, p.stay
        );
        if p.mask & 4 != 0 || p.scripts.contains(&0x0600_0a3b) {
            script_hashes.extend(p.scripts.iter().copied().filter(|h| *h != 0));
        }
    }

    println!("== the mask-4 / lid scripts, fully ==");
    for h in &script_hashes {
        dump_script(sim, *h);
    }

    // The prime tick, then the created entities near the slot (z~63.66) and the mouths (z~42.16).
    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    println!("-- after 1 prime tick --");
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let nf_game::nis::Obj::Entity { hash, kind, matrix, .. } = o {
            let (x, y, z) = (matrix[3][0], matrix[3][1], matrix[3][2]);
            if (30.0..50.0).contains(&x) && (35.0..70.0).contains(&z) {
                println!("  entity {hash:08x} kind {kind:#04x} at ({x:.3}, {y:.3}, {z:.3})");
            }
        }
    }
}
