//! Probe: the 07000014 opening scene's door entity streams (0x06000833 streams 10/11).
//! Logs the sim's Obj::Entity for hashes 0x02000891/92, the scene's draw slots for them,
//! and whether `level.by_hash` resolves the hashes at all.
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let slots: Vec<(u32, u8, u32, u8)> = scene.attachments.iter()
        .filter_map(|a| a.script.map(|(s, st, h, k)| (s, st, h, k)))
        .filter(|&(s, _, h, _)| s == 0x0600_0833 && (0x0200_0890..=0x0200_0893).contains(&h))
        .collect();
    println!("draw slots for 0x06000833's door entities: {slots:?}");
    println!("total attachments for script 0x06000833: {}",
        scene.attachments.iter().filter(|a| a.script.is_some_and(|(s, _, _, _)| s == 0x0600_0833)).count());
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [40.7, 2.6, 39.0];
    for t in 0..1500u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if t % 250 == 0 || t == 260 || t == 300 || t == 540 || t == 1000 {
            let mut ents = Vec::new();
            for o in sim.drones.scripts.objects.list.iter().flatten() {
                if let nf_game::nis::Obj::Entity { hash, kind, matrix, .. } = o {
                    if (0x0200_0890..=0x0200_0893).contains(hash) {
                        ents.push((*hash, *kind, [matrix[3][0], matrix[3][1], matrix[3][2]]));
                    }
                }
            }
            println!("t{t}: door entities {ents:?}");
        }
    }
}
