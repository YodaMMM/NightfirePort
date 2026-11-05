//! audit-effect-calls live test (NF_ISO): the footstep call is the FULL
//! `Effect_Create(…, foot | 0x42)` [M AnimObjectUpdate 0x1befd8:196-201] — flag 0x2 runs
//! `Effect_Player` 0x1cec48:59-140's decal arm alongside the 0x40 sound: one footprint decal per
//! step, gfx = `EffectInfo[mat]+0x8` on foot 4 / `+0xc` on foot 8 [M 0x1cec48:84-89], the foot
//! alternating every step [M 0x1befd8:186-198 `ScriptFootstepFoot` flip → :196-198], the decal's
//! y = the ground hit's y + 0.05 [M 0x1cec48:118-124], hosted on the ground cel [M 0x1cec48:107+].
//!
//! The walk is REAL input from the level start (no teleport): `Walker::walk_to` over
//! `Sim::step_bond`. The port's print x/z sits at the body's own ground point [S — the game puts
//! it at the foot bone (`obj+0xdc → +0xb8` bones +0xdc0/+0xcc0, 0x1cec48:89-96), whose world
//! matrices the port does not build for first-person Bond]; the draw is the shared decal path
//! (no new drawing). Opt-in: `NF_ISO`.
//!
//! Run: `NF_ISO=… cargo test -p nf-view --test footprint_decals_live -- --test-threads=1`

#[path = "common/walker.rs"]
mod walker;

use nf_game::effect::DecalHost;
use walker::{Walker, Wp};

#[test]
fn walking_leaves_the_games_footprint_decals() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    nf_view::settle_start_draw(w.scene.sim.as_mut().unwrap());

    let (start, yaw) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.pos, sim.bond.yaw)
    };
    let dir = [yaw.sin(), 0.0, yaw.cos()];
    // The floor under the start: the same ray the port's step block casts (`p → p − 1.6347 y`).
    let (mat, floor_y) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let probe = [start[0], start[1] - 1.6347, start[2]];
        match sim.data().world.segment_surface_host(start, probe, true) {
            Some((t, _, m, _, _)) => (m, start[1] - 1.6347 * t),
            None => panic!("no floor under the 07000005 start"),
        }
    };
    let (m4, m8) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let rec = &sim
            .surfaces
            .as_ref()
            .expect("the surface table")
            .effects[(mat & 0x3f) as usize];
        (rec.foot_decal_4, rec.foot_decal_8)
    };
    assert!(
        m4 != 0 || m8 != 0,
        "material {mat} under the 07000005 start carries no foot decals — the test needs a printed surface"
    );

    let before = w.scene.sim.as_ref().unwrap().drones.effects.decals.len();
    // ~4 steps at the game's 1.65 stride [M 0x1befd8:165].
    let target = [start[0] + dir[0] * 7.0, start[1], start[2] + dir[2] * 7.0];
    w.walk_to(Wp::new(target[0], target[1], target[2]), "the print walk");

    let sim = w.scene.sim.as_ref().unwrap();
    // The prints among the new decals (a responding guard's holes carry the hole meshes —
    // filtered by the material's two foot columns; on stone-family floors 0x02000126 is shared
    // with the hole set, so the count assert keeps a margin).
    // fx-npc-footsteps: the level's NPCs lay their OWN prints now (the same `Effect_Create`), so
    // the filter also keeps only the decals ON Bond's walked segment — a patrolling guard's step
    // on the same surface family shares the two meshes but not the path.
    let seg_dist = |p: [f32; 3]| {
        let (sx, sz) = (start[0], start[2]);
        let dx = target[0] - sx;
        let dz = target[2] - sz;
        let len2 = dx * dx + dz * dz;
        let t = (((p[0] - sx) * dx + (p[2] - sz) * dz) / len2).clamp(0.0, 1.0);
        let (cx, cz) = (sx + dx * t, sz + dz * t);
        ((p[0] - cx) * (p[0] - cx) + (p[2] - cz) * (p[2] - cz)).sqrt()
    };
    let prints: Vec<_> = sim.drones.effects.decals.iter().skip(before)
        .filter(|d| d.mesh == m4 || d.mesh == m8)
        .filter(|d| seg_dist(d.pos) < 1.0)
        .collect();
    assert!(
        prints.len() >= 2,
        "walking ~7 units at the 1.65 stride leaves ≥2 prints, got {} (m4={m4:#x} m8={m8:#x} mat={mat})",
        prints.len()
    );
    // The foot alternates from foot 8 (the first flip makes `ScriptFootstepFoot` 1 → foot 8 →
    // the +0xc column) [M 0x1befd8:186-198, 0x1cec48:84-89].
    for (k, dcl) in prints.iter().enumerate() {
        let want = if k % 2 == 0 { m8 } else { m4 };
        assert_eq!(dcl.mesh, want, "print {k} must carry the foot-{} column", if k % 2 == 0 { "8" } else { "4" });
    }
    // y = the ground hit + 0.05 [M 0x1cec48:118-124]; host = the ground cel.
    for dcl in &prints {
        assert!(
            (dcl.pos[1] - (floor_y + 0.05)).abs() < 0.3,
            "print y {} must sit at the walked floor + 0.05 ({} [M 0x1cec48:118-124])",
            dcl.pos[1],
            floor_y + 0.05
        );
        assert!(matches!(dcl.host, DecalHost::Cel(_)), "the print hosts on its ground cel [M 0x1cec48:107+]");
    }
}
