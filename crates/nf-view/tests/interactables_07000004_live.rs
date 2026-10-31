//! interactables-04: the player-touchable KINDS of 07000004, each driven the way a player drives
//! it (the walk, the trigger pull), each asserting the game's result with a sister that fails
//! without the operation. The census re-run for level-04-interactables-2 over the disc parse [D
//! placements-all 07000004.bin, main map "010000c5": 70 placements, flags&0xff kinds — 0x0F ×1,
//! 0x19 ×40, 0x21 ×4, 0x24 ×1, 0x2A ×3, 0x2B ×2, 0x32 ×3, 0x47 ×2, 0xD8 ×2, 0xD9 ×2, 0xDF ×1,
//! 0xEE ×1, 0xF0 ×4, 0xF3 ×2, 0xFC ×2; NO doors/switches/lifts/locks/monitors/fuseboxes/SS —
//! every number matches docs/levels/07000004.md's 2026-10-17 table; the main map is the only map
//! whose placements the game creates — `parsemap_parsemap` 0x1d1cf0's state-2 epilogue [M]].
//!
//! The kind coverage map (player-touchable kinds only; the world-fx kinds — ripples 0x21,
//! leafgens 0x2B, trees 0xF3, env 0xDF, sky 0x2A, corona 0xFC — are not player-touchable and live
//! in `interact_07000004_live.rs`):
//! - pickups 0xF0 — THIS file: the take's rounds/respawn tail and the armour-cap refusal (the
//!   behaviours `interact_07000004_live.rs`'s take tests do not carry); the base takes are there.
//! - rigid bodies 0xD8 — THIS file: the lamps' create/rest and the shot that tips them (the
//!   census's "UNPORTED" row predates the rigid-body port: the init four [M RB_Init 0x1d7ad8 /
//!   RB_CuboidMomentsOfInertia 0x1d7dc8 / RB_Build 0x1b7ba0 / RB_SetState 0x1d7da0], the evolution
//!   [M RB_Update 0x1d6e38 / RB_EulerEvolve 0x1d7810 / RB_ApplyImpulses 0x1d6ff8] and the blast
//!   arm [M RB_RestingContactRot 0x1d73a0]; the game's collision pass appends the bullet's contact
//!   to the body's `obj+0xD0` chain and `RB_CollisionHandler` 0x1d7a80 walks it into
//!   `RB_RestingContactRot`'s per-hit arm — the port's bullets never reached the bodies before the
//!   `rigid-body-bullet` arm in `update_player_bullets`).
//! - the target drone 0x0F (the assassination shot), the TouchOnce 0xEE mux (#31, out 0x54 → list
//!   [0x64], position-independent — `Trigger_Update` 0x1b1a78 case 5 consults no hit) and the
//!   ScriptPlayers 0xD9 (the intro/story scenes the player skips) are `interact_07000004_live.rs`.
//!
//! Evidence tags: [M] the decompile function + address, [D] the disc parse. A teleport only where
//! the walk is a separately documented blocker; the walk-over and the shot are always the real
//! input path.

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

/// The same settle over `nf_view::load_weapons` — the StormUzi 0x12's first-person model must be
/// loaded or `Player_EquipWeapon` refuses the take and `equip_weapon`'s `loaded` gate returns
/// false [M 0x1a8a58; pickup.rs equip_weapon].
fn load_ready_weapons(iso: &str, wids: &[u8]) -> Walker {
    let mut w = Walker::new(nf_view::load_weapons(iso, "07000004.bin", false, wids).expect("load 07000004 + weapons"));
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

/// KIND: pickups 0xF0, the respawn tail. The three PICKUP_StormUzi placements carry (0,0),(1,18),
/// (2,60),(4,319),(5,2) [D — kind 0 weapon 18, 60 rounds, sound 319, respawn 2; `Pickup_Create`
/// 0x18dda0 reads param 5 as the respawn word [M scene.rs `placed_pickup_from_placement`]]. The
/// take parks the pickup hidden in the state-2 wait — `Pickup_Handler` 0x18e650:338-348 writes
/// `obj+0xF0 |= 0x10`, `PICKUPINFO+0x20 = 2` and stamps `obj+0xEC` — and `Pickup_Update`
/// 0x18e148:182-189 re-arms it (`obj+0xF0 &= ~0x10`, state 1) on the first tick where the clock
/// has run past `FRAME_RATE_INT · respawn · 10` = 60·2·10 = 1200 ticks [L FRAME_RATE_INT 60 in
/// SP, docs/systems/pickups.md]. The walk takes the lawn uzi (−10.66, 4.39, 42.66 [D]); the
/// record must be hidden at half the wait and takeable again past it. Sister: the record rests
/// visible (state 1, not hidden) BEFORE the walk — the wait alone never arms the tail.
#[test]
fn the_uzi_respawn_rearms_after_the_state2_wait() {
    let iso = iso();
    let mut w = load_ready_weapons(&iso, &[18]);
    let (at, respawn, rounds) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .expect("the StormUzi pickup");
        assert_eq!(p.respawn, 2, "the disc's respawn word 2 [D param 5]");
        assert_eq!(p.rounds, 60, "the disc's rounds word 60 [D param 2]");
        assert!(!p.hidden && p.state == 1, "sister: the pickup rests visible (state 1) before the walk");
        (p.start, p.respawn, p.rounds)
    };
    let _ = (respawn, rounds);
    eprintln!("[04] walking to the StormUzi at {at:?}");
    w.walk_nav_to(at, "the StormUzi pickup", 0.4);
    let taken_at = w.total_ticks;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.inventory.owned.get(18).copied().unwrap_or(false),
            "the walk-over owns weapon 18 [M Pickup_Handler 0x18e650 type-0 arm]");
        let p = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && (p.start[2] - at[2]).abs() < 1.0)
            .expect("the taken record stays in the list");
        assert!(p.hidden && p.state == 2,
            "the take parks the pickup hidden in the state-2 wait [M 0x18e650:341-347], got hidden {} state {}",
            p.hidden, p.state);
    }
    // Mid-wait: 600 ticks in (half the 1200), still hidden — the re-arm is gated by the elapsed
    // wait, not immediate [M 0x18e148:182-189's `<=` return].
    while w.total_ticks - taken_at < 600 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && (p.start[2] - at[2]).abs() < 1.0)
            .expect("the waiting record");
        assert!(p.hidden && p.state == 2, "600 ticks into the 1200-tick wait the pickup is still hidden [M 0x18e148:183-186], got hidden {} state {}", p.hidden, p.state);
    }
    // Past the wait: the re-arm (hidden bit cleared, state 1 — takeable again).
    while w.total_ticks - taken_at < 1300 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let p = sim.drones.pickups.iter()
        .find(|p| p.kind == 0 && p.weapon == 18 && (p.start[2] - at[2]).abs() < 1.0)
        .expect("the re-armed record");
    assert!(!p.hidden && p.state == 1,
        "past 60·2·10 = 1200 ticks the pickup re-arms takeable [M 0x18e148:182-189], got hidden {} state {}",
        p.hidden, p.state);
}

/// KIND: pickups 0xF0, the armour-cap refusal. #9 PICKUP_BodyArmourHeavy carries (0,3),(2,50),
/// (5,0) [D — kind 3, respawn 0]; `Pickup_Handler` 0x18e650:220-225's kind-3 arm RETURNS while
/// `BLData+0x8B0 >= 50.0` — the shared tail (sound, channel, delete) never runs, so the pickup
/// stays visible and takeable — and otherwise hard-sets 50.0. Bond arrives with the cap poked
/// (the stand-in for a campaign's carried armour): the walk-over must be refused. Then the cap
/// cleared, the SAME standing contact takes it (respawn 0 → deleted [M :338-341]) — the paired
/// sister proving the refusal was the cap arm and not a dead walker.
#[test]
fn the_armour_refuses_the_take_at_the_cap_then_takes() {
    let iso = iso();
    let mut w = load_ready(&iso);
    let at = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
        .expect("the BodyArmourHeavy pickup #9 [D]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.health.armour, 0.0, "the fresh load carries no armour");
        sim.drones.health.armour = 50.0; // the cap [M 0x18e650:220 the >= 50.0 gate]
    }
    w.walk_nav_to(at, "onto the armour at the cap", 0.4);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!((sim.drones.health.armour - 50.0).abs() < 0.001,
            "the cap holds through the walk-over [M 0x18e650:220-225's return], got {}",
            sim.drones.health.armour);
        let p = sim.drones.pickups.iter().find(|p| p.kind == 3)
            .expect("the refused record stays in the list");
        assert!(!p.gone && !p.hidden && p.state == 1,
            "the refused pickup stays visible and takeable — the shared tail never ran [M 0x18e650:220-225], got gone {} hidden {} state {}",
            p.gone, p.hidden, p.state);
    }
    // Sister: the cap cleared, the same standing contact takes it.
    w.scene.sim.as_mut().unwrap().drones.health.armour = 0.0;
    w.wait(90);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!((sim.drones.health.armour - 50.0).abs() < 0.001,
        "the take at armour 0 sets 50.0 [M 0x18e650:222-224], got {}", sim.drones.health.armour);
    assert!(sim.drones.pickups.iter().find(|p| p.kind == 3).map_or(true, |p| p.gone),
        "respawn 0 deletes the taken armour pickup [M 0x18e650:338-341]");
}
