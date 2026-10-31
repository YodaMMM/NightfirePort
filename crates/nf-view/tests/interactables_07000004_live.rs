//! interactables-04: the player-touchable kinds of 07000004 that `interact_07000004_live.rs`
//! does not carry. The census is docs/levels/07000004.md's 2026-10-17 table over the disc parse
//! [D placements-all 07000004.bin: 70 placements, main map 49 only — `parsemap_parsemap`
//! 0x1d1cf0's state-2 epilogue creates the main map's dynamic objects [M]]. Every kind the table
//! lists is covered there except one: the **rigid bodies 0xD8** (#39/#40 "rb_whitelamp") — the
//! census's "UNPORTED" row predates the rigid-body port (`crate::rigid_body`: the init four
//! [M RB_Init 0x1d7ad8 / RB_CuboidMomentsOfInertia 0x1d7dc8 / RB_Build 0x1b7ba0 / RB_SetState
//! 0x1d7da0], the evolution [M RB_Update 0x1d6e38 / RB_EulerEvolve 0x1d7810 /
//! RB_ApplyImpulses 0x1d6ff8] and the blast arm [M RB_RestingContactRot 0x1d73a0]). The kind's
//! player route — the round that hits the lamp — is the port gap this file pins: the game's
//! collision pass appends the bullet's contact to the body's `obj+0xD0` chain (the bullet is
//! class 5) and `RB_CollisionHandler` 0x1d7a80 walks it into `RB_RestingContactRot`'s per-hit arm;
//! the port's bullets never reached the bodies before the `rigid-body-bullet` arm in
//! `update_player_bullets`. Evidence tags: [M] the decompile function + address, [D] the disc
//! parse. A teleport only where the walk is a separately documented blocker; the shot is always
//! the real fire path.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

/// The settled walker (the intro + story scenes skipped the way the player does — the AI
/// re-enables and ch 0x53, the story hand-off, is written [D script 0x06000082 stream 9]).
fn load_ready(iso: &str) -> Walker {
    let mut w = Walker::new(nf_view::load(iso, "07000004.bin", false).expect("load 07000004"));
    let mut ready = false;
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { ready = true; break; }
    }
    assert!(ready, "the intro + story scenes must settle (the AI re-enabled, ch 0x53 written)");
    w
}

/// The bodies' full sim-visible state (the same fields `rigid_body_wiring_live` pins — the
/// rest state is bit-identical over the sim [M 0x1d6e38: the type!=0 arm runs no linear
/// integrate; 0x1d79ac: zero omega skips apply_omega]).
fn snap(w: &Walker) -> Vec<(u16, [u8; 132])> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.rbodies.list.iter().map(|o| {
        let mut b = [0u8; 132];
        let mut k = 0usize;
        for v in o.st.pos.iter().chain(o.st.omega.iter()).chain(o.st.vel.iter())
            .chain(o.st.ang_accel.iter()).chain(o.st.lin_impulse.iter())
            .chain(o.st.tor_impulse.iter()).chain(o.st.trans.iter()) {
            b[k..k + 4].copy_from_slice(&v.to_le_bytes()); k += 4;
        }
        for row in &o.st.matrix { for v in row { b[k..k + 4].copy_from_slice(&v.to_le_bytes()); k += 4; } }
        b[k..k + 4].copy_from_slice(&o.st.s9c.to_le_bytes()); k += 4;
        b[k..k + 4].copy_from_slice(&o.st.s13c.to_le_bytes()); k += 4;
        b[k] = o.flags as u8; b[k + 1] = (o.flags >> 8) as u8;
        (o.placement, b)
    }).collect()
}

/// KIND: the rigid bodies 0xD8, created at rest. `RB_Create` 0x1d6bf0 (the dispatcher's case
/// 0xD8 [M 0x1d04f0]) runs RB_Init/RB_Build/RB_SetState on each placement's record: #39/#40
/// "rb_whitelamp" carry mass param 2 = 1 and the type halfword 0xFFFF (param 4 [D]) — the
/// sphere-collider table [M 0x1d6bf0:77 — SPHERE_VERTS for every non-zero type] and the
/// rotation-only body (the linear integrate arm is type-0 only [M RB_EulerEvolve 0x1d796c]).
/// Sister: the SECOND lamp untouched — both rest bit-identical over 600 ticks.
#[test]
fn lamps_are_created_and_rest_bit_identical() {
    let iso = iso();
    let mut w = load_ready(&iso);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let lamps: Vec<_> = sim.drones.rbodies.list.iter().filter(|o| o.placement == 39 || o.placement == 40).collect();
        assert_eq!(lamps.len(), 2, "both rb_whitelamp placements create bodies [D #39/#40; M 0x1d6bf0 via 0x1d04f0 case 0xD8]");
        for l in &lamps {
            assert_eq!(l.rb.ty, -1, "the type halfword 0xFFFF [D param 4; M RB_Init stores it at +0xa8]");
            assert!((l.rb.mass - 1.0).abs() < 1e-4, "mass param 2 = 1 [D; M RB_Create's mass read]");
            assert_eq!(l.rb.vert_table, nf_game::rigid_body::SPHERE_VERTS, "the sphere collider table [M 0x1d6bf0:77]");
            assert!((l.st.s13c - 0.0).abs() < 10.0 && l.st.s13c > 0.0, "the body's world-sphere radius is sane, got {}", l.st.s13c);
        }
    }
    let before = snap(&w);
    for _ in 0..600 { w.tick(Default::default(), false); }
    let after = snap(&w);
    assert_eq!(before.len(), after.len());
    for (b, a) in before.iter().zip(after.iter()) {
        assert_eq!(b.0, a.0, "the placement list is stable");
        assert_eq!(b.1, a.1, "the lamp at placement {} moved while resting — a resting body is bit-identical [M 0x1d6e38 type!=0: no linear arm; 0x1d79ac: zero omega skips apply_omega]", b.0);
    }
}

/// KIND: the lamp reacts to the player's shot — the round's contact lands in the body's hit
/// chain and `RB_RestingContactRot`'s arm tips it. `RB_Create`'s bodies hang at y 7.86 [D
/// #40 t (8.01, 7.86, 23.03)]; the walk stands under #40 and `shoot_point` fires the real
/// path (the turn/look inputs, the trigger through `Sim::step_bond`). Sister: #39 (−12.05,
/// 7.86, 23.03 [D]) is never shot — it stays bit-identical while #40 moves.
#[test]
fn shot_lamp_leaves_rest_and_the_sister_stays() {
    let iso = iso();
    let mut w = load_ready(&iso);
    // Under lamp #40: the spawn courtyard floor stands ~4.4 (the pickups' y [D #6-#9]); the
    // lamp hangs 7.86 [D] — ~3 units up and ~3.5 out from the stand.
    w.walk_nav_to([8.0, 4.4, 26.5], "under the whitelamp", 1.2);
    let lamp = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.rbodies.list.iter().find(|o| o.placement == 40)
            .map(|o| (o.st.trans, o.st.pos, o.st.matrix, o.st.omega))
            .expect("lamp #40 exists")
    };
    let at = [lamp.0[0], lamp.0[1], lamp.0[2]];
    let mut hit = false;
    for _ in 0..2 {
        let fired = w.shoot_point(at, "the whitelamp", |sim| {
            let o = sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap();
            o.st.omega != lamp.3 || o.st.matrix != lamp.2 || o.st.pos != lamp.1
        });
        hit |= fired;
        if hit { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let post = sim.drones.rbodies.list.iter().find(|o| o.st.trans[0].abs() != f32::INFINITY)
        .map(|_| ()).expect("the bodies still finite");
    let _ = post;
    let o40 = sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap();
    let moved = o40.st.omega != lamp.3 || o40.st.matrix != lamp.2 || o40.st.pos != lamp.1;
    assert!(moved, "the shot must move lamp #40 — the bullet's contact reaches the body's hit chain [M RB_CollisionHandler 0x1d7a80 → RB_RestingContactRot 0x1d73a0's arm]");
    let _ = hit;
    // Sister: #39 never shot — its state is untouched by #40's rounds.
    let o39 = sim.drones.rbodies.list.iter().find(|o| o.placement == 39).unwrap();
    assert!(o39.st.omega == [0.0; 3] && o39.st.lin_impulse == [0.0; 3] && o39.st.tor_impulse == [0.0; 3],
        "sister: the unshot lamp keeps zero omega and empty accumulators [M 0x1d6e38's rest gate]");
    // The body stays finite after the kick (the evolution's NaN discipline).
    assert!(o40.st.trans.iter().chain(o40.st.omega.iter()).all(|v| v.is_finite()),
        "the kicked lamp's state stays finite [M 0x1d7810 EulerEvolve]");
}
