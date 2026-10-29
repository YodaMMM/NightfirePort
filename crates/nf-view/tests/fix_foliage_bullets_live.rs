//! fix-foliage-bullets: the trees' leaves seemed bulletproof (07000005, the small trees near the
//! start). What the game's data and code actually say:
//!
//! - The foliage pass machinery is real [M]: `Collide_Intersect` 0x1ebaa8 ORs `HITDATA+0x51` bit 1
//!   from the glist node's flags bit 0x40 (cel+0x94 = the placement's `celglist+0x44` [M
//!   parsemap_block_map_data_static 0x1d1998:103]) or the surface byte's bit 0x40 (disasm
//!   0x1ebf78-0x1ebf80), `Collide_Update` 0x1ea398:216-229 walks the tracer past pass-flagged
//!   hits, and the bit-0x8 queries (HITTEST+0x98) drop them outright (disasm 0x1ebff0-0x1ec010).
//!   The bit-0x8 family is the AIM/LOS traces: the player's aim trace 0x20a/0x208
//!   (`Player_WeaponInitBullet` 0x1a4bf0:154-159) and `Bullet_homing`'s LOS 8 (0x12f358:92).
//! - The 07000005 tree objects (map 50, obj 96/98) carry glist flags 0x41 [D] — node-flagged —
//!   and the leaf cards (obj 21/109/117/144, flags 0x10) carry the 0x40 surface byte [D]. The
//!   separate wood core obj 97 (30 tris, mat 0x04, flags 0) is NOT flagged [D].
//! - The bullets' FLIGHT runs the same pick+walk as every moving object: Bullet_update arms the
//!   HITTEST every flying tick (+0x94 = 0x101/0x201, +0x9a = the surface mask 0x20 for guns
//!   without weapon_data+0x6c & 0x800 [M 0x12e7b0 disasm 0x12f030-0x12f058, 0x12f134/0x12f154]),
//!   and the sorted contact list is walked past every pass-flagged hit to the first solid one
//!   [M Collide_Update 0x1ea398:216-229]. So a flagged TREE stops the round on its UNFLAGGED wood
//!   core (obj 97, mat 0x04 [D]) — the shell itself never stops anything. (An earlier reading of
//!   this file took the +0x94 = 0 at Bullet_init 0x12c668 as "the pick never arms" and pinned the
//!   flight to the shell's first face; the flight's own per-tick mask writes settle it the other
//!   way — the walk rule governs. The bulletproof LOOK is unchanged: the core's decals sit inside
//!   the canopy where no player sees them.)
//!
//! So the round stops INSIDE the tree, on the unflagged core, whose EffectInfo record has NO decal
//! and NO gas — only debris chips [D EffectInfo] — and the canopy hides the impact from outside:
//! it reads as "bulletproof" next to every wall that shows a bullet hole. These tests pin the
//! flight's walk rule and the aim-trace family's drop, so the distinction survives.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test fix_foliage_bullets_live`

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The tree cluster east of the spawn: inst 80 of obj 96's bbox (the port's level build).
const TREE_LO: [f32; 3] = [89.43, 23.61, 88.77];
const TREE_HI: [f32; 3] = [93.52, 30.72, 92.87];

fn in_tree_span_x(x: f32) -> bool { x >= TREE_LO[0] - 0.2 && x <= TREE_HI[0] + 0.2 }

/// The flight drops the flagged shell and stops on the unflagged core (the walk rule [M
/// 0x1ea398:216-229]); the aim/LOS family (bit 0x8 [M 0x1ebff0-0x1ec010]) drops it at the
/// intersect — both report the core. The fired round's impact lands on the core, with the core's
/// class-2 record — and the canopy hides it from outside (the "bulletproof" look).
#[test]
fn the_flight_and_the_aim_trace_both_stop_on_the_core_behind_the_flagged_shell() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        // Stand 6 units east of the tree, aim at its crown line (the repro shot).
        sim.bond.pos = [97.0, 24.30, 90.80];
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + 0.70, sim.bond.pos[2]];
        let target = [91.0f32, 25.5f32, 90.8f32];
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let dir = [d[0] / l, d[1] / l, d[2] / l];
        sim.bond.yaw = dir[0].atan2(dir[2]);
        sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
        let w = &sim.world;
        let far = [eye[0] + dir[0] * 60.0, eye[1] + dir[1] * 60.0, eye[2] + dir[2] * 60.0];
        // The flat trace (the sight-class query, no drops for 0x13): the shell's 0x13 face inside
        // the tree span. The FLIGHT and the aim family stop deeper, on the core (asserted below
        // through the fired round).
        let (tf, _, mf) = w.segment_surface(eye, far, false).expect("the flat trace stops at the shell");
        let pf = [eye[0] + (far[0] - eye[0]) * tf, eye[1] + (far[1] - eye[1]) * tf, eye[2] + (far[2] - eye[2]) * tf];
        assert!(in_tree_span_x(pf[0]), "flat: the first hit is inside the tree span ({pf:?})");
        assert_eq!(mf & 0x3f, 0x13, "flat: the hit surface byte is the shell's 0x13, got 0x{mf:02x}");
        // The aim-trace family (bit 0x8): the shell drops, the first report is the wood core.
        let (tb, _, mb) = w.segment_surface_bullet(eye, far).expect("the aim trace reaches the core");
        let pb = [eye[0] + (far[0] - eye[0]) * tb, eye[1] + (far[1] - eye[1]) * tb, eye[2] + (far[2] - eye[2]) * tb];
        assert!(in_tree_span_x(pb[0]), "the core sits inside the tree span ({pb:?})");
        assert_eq!(mb & 0x3f, 0x04, "aim: the reported class is the unflagged core's 0x04 [D obj 97], got 0x{:02x}", mb & 0x3f);
        assert!(pb[0] < pf[0], "the core is deeper along the ray than the dropped shell (core {} < shell {})", pb[0], pf[0]);
        eprintln!("[pre] flight/flat stop 0x{mf:02x} at {pf:?}; aim-trace stop 0x{mb:02x} at {pb:?}");
        // Fire one P2K round through the real input path.
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 12;
        sim.gun.pending = 0;
    }
    let mut input = Input::default();
    input.fire = true;
    let mut impacts: Vec<[f32; 3]> = Vec::new();
    for t in 0..40u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(if t == 0 { input } else { Input::default() }, t == 0, false, false);
        let sim = scene.sim.as_mut().unwrap();
        for im in &sim.drones.player_impacts {
            if let nf_game::drone::PlayerImpact::World(p) = im { impacts.push(*p); }
        }
        if sim.drones.player_bullets.is_empty() && t > 0 { break; }
    }
    assert!(!impacts.is_empty(), "the round records a world impact (on the tree)");
    // The impact lands ON the tree (the flight walks past the flagged shell to the unflagged core
    // [M 0x1ea398:216-229]) and never reaches the terrain behind (the 0x0a wall at x≈61.8 the aim
    // trace reports).
    let sim = scene.sim.as_ref().unwrap();
    for p in &impacts {
        assert!(in_tree_span_x(p[0]), "the impact is the tree stop, inside the tree span ({p:?})");
        assert!(p[0] < 92.0, "the flight passed the shell's first face (x >= 92) and stopped on the core behind it: {p:?}");
        let (_, _, m) = sim.world.segment_surface([p[0] + 1.5, p[1], p[2]], [p[0] - 1.5, p[1], p[2]], false)
            .expect("the impact surface is re-queryable");
        assert!(matches!(m & 0x3f, 0x13 | 0x03 | 0x04), "the impact's surface is the tree's 0x13/0x03/0x04, got 0x{m:02x}");
    }
    eprintln!("[fire] world impacts on the tree: {impacts:?}");
}

/// Sister (the guard side): a drone-style `Bullet` flying the same ray also walks past the flagged
/// shell (the same armed pick + walk [M 0x12f058/0x12f134, 0x1ea398:216-229]) and dies on the core
/// behind it; the flat sight trace stops on the shell face itself (the guards' sight: flags 0x721,
/// no bit 0x8, no 0x20 [M 0x18e148:214] — the foliage stays solid to sight).
#[test]
fn guard_style_bullets_stop_on_the_core_and_sight_on_the_shell() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let (eye, dir) = {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        sim.bond.pos = [97.0, 24.30, 90.80];
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + 0.70, sim.bond.pos[2]];
        let target = [91.0f32, 25.5f32, 90.8f32];
        let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let dir = [d[0] / l, d[1] / l, d[2] / l];
        (eye, dir)
    };
    let sim = scene.sim.as_ref().unwrap();
    // A guard's rifle round: no fly-by, range past the tree.
    let mut b = nf_game::bullet::Bullet { pos: eye, dir, speed: 8.0, travelled: 0.0, range: 120.0, damage: 1.0, owner: 1, state: 0, weapon: 0x16, flyby: false, bounces: 0, ..nf_game::bullet::Bullet::TAIL };
    let mut rng = nf_game::drone::Rng::new(7);
    let mut died_at: Option<[f32; 3]> = None;
    for _ in 0..60 {
        match b.update(&sim.world, &[], [1e4, 0.0, 1e4], &mut rng) {
            Some(nf_game::bullet::BulletEvent::World(p)) => { died_at = Some(p); break; }
            _ => {}
        }
        if b.state >= 2 { break; }
    }
    let dead = died_at.unwrap_or_else(|| { assert!(b.state >= 2 || b.travelled >= b.range, "the round ends this tick"); b.pos });
    assert!(in_tree_span_x(dead[0]), "the guard round stops inside the tree span ({dead:?})");
    assert!(dead[0] < 92.0, "the guard round passed the flagged shell's first face (x >= 92) and \
        stopped on the core behind it [M 0x1ea398:216-229 + 0x12f058/0x12f134]: {dead:?}");
    // Sight: the flat trace stops on the same shell face.
    let far = [eye[0] + dir[0] * 60.0, eye[1] + dir[1] * 60.0, eye[2] + dir[2] * 60.0];
    let t = sim.world.segment(eye, far).expect("sight still stops at the tree");
    let p = [eye[0] + (far[0] - eye[0]) * t, eye[1] + (far[1] - eye[1]) * t, eye[2] + (far[2] - eye[2]) * t];
    assert!(in_tree_span_x(p[0]) && p[0] >= 92.0, "sight's first hit is the shell's front face ({p:?})");
    let (_, _, m) = sim.world.segment_surface(eye, far, false).expect("sight surface re-query");
    assert_eq!(m & 0x3f, 0x13, "sight stops on the shell's 0x13 surface, got 0x{m:02x}");
}

/// Sister (the leaf cards): the collision-only leaf planes by the spawn (obj 117, glist flags 0x10,
/// surface byte 0x40 [D]) block the flat trace and vanish from the aim/LOS-family trace — the
/// material's pass bit (Collide_Intersect 0x1ebaa8 disasm 0x1ebf78-0x1ebf80) with the bit-0x8 drop.
#[test]
fn the_0x40_leaf_cards_leave_the_aim_trace_only() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Default::default(), false, false, false);
    // The card plane spans (101.4-114.5, 24.75-25.75, 93.4-114.9) [D inst 125 obj 117]; the ray
    // crosses it at (111.89, 25.25, 101.89) with nothing non-pass behind on this line.
    let from = [106.0f32, 25.25f32, 96.0f32];
    let to = [112.0f32, 25.25f32, 102.0f32];
    let (t, _, m) = sim.world.segment_surface(from, to, false).expect("the flat trace stops on the leaf card");
    assert!(m & 0x40 != 0 && m & 0x3f == 0, "the leaf card's surface byte carries the pass bit 0x40 over class 0 (got 0x{m:02x})");
    let p = [from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t, from[2] + (to[2] - from[2]) * t];
    assert!(p[0] >= 101.3 && p[0] <= 114.6 && p[2] >= 93.3 && p[2] <= 115.0, "the flat hit is on the card plane ({p:?})");
    assert!(sim.world.segment_surface_bullet(from, to).is_none(), "the aim/LOS trace drops the pass-flagged card entirely");
}
