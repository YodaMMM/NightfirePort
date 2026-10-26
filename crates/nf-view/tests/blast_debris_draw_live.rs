//! The blast debris draws. `Explode_Update` 0x17c350's debris arm spawns class-0x10 objects through
//! `Debris_CreateEx` 0x1ac808 (the `MetalHash_132` / `StoneHash_133` meshes); they are the same
//! `Debris_Update` 0x1aac00 objects as a breakable's pieces and draw the same way — euler `rot` at
//! obj+0x30, scaled by obj+0xe8 (`Control_GetObjMatrix` 0x134210:25-33). The port ticked them in
//! `Drones::blast_debris` but no draw slot read them.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn a_blast_debris_piece_rides_a_debris_slot() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "07000015.bin", false).expect("load 07000015");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let pos = [1.25f32, 2.5, -3.75];
    {
        let d = &mut scene.sim.as_mut().unwrap().drones;
        d.blast_debris.push(nf_game::shatter::Debris {
            model: nf_game::explode::METAL_HASH_132[0], pos, rot: [0.0; 3], vel: [0.0; 3], spin: [0.0; 3],
            life: 50, fade: 0xff, e8: 1.0,
        });
    }
    let (layout, _attrs) = scene.skin_layout();
    let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    let hit = pal.iter().any(|m| (m[3][0] - pos[0]).abs() < 1e-4 && (m[3][1] - pos[1]).abs() < 1e-4 && (m[3][2] - pos[2]).abs() < 1e-4);
    assert!(hit, "a debris slot poses the blast piece at its obj+0x30 [M 0x1aac00, 0x134210:25-33]");
}
