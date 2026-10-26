//! level-07-urn-break: `SP_Hit` 0x1dc470's bullet arm — a player bullet that flies through a
//! mask-bit-2 script player's CURRENT stream object starts its break script.
//!
//! The disc census [D dump_sp_hit_census]: exactly three mask-bit-2 script players ship —
//! 07000007 #312 (OBJECT_UrnBreak, script 0x0600007A, modes [64, 0], hp 0, filter 0),
//! 07000012 #660 (the sky bridge, 0x06000530, modes [64, 0], **hp 120**, filter 0) and
//! 07000006 #807 (the radio, 0x06000055+0x06000056, modes [8, 1], hp 0, filter 0). None sets
//! the `+0x37` weapon filter. The urn's script [D dump_sp_script]: stream 9 carries the INTACT
//! urn (glist 0x020004EE, model sphere r 0.911) from t0 to its t23 EntityEnd; stream 11 a
//! 1-tick piece (0x020004E5); on the replay streams 1-8 carry the debris pieces (stream 1 =
//! 0x020004EA), stream 12 the crash sound.
//!
//! The pin is the game's one-frame order [M `control_movement_object_handler` 0x1336f8:238 —
//! `Collide_Update` runs AFTER the control walk, so `SP_Hit` reads the chains one walk after
//! the contact]: the tick that records the contact still shows state 1, the next tick's walk
//! consumes it (state 1 → 2 [M 0x1dbda8 case 1]).
//!
//! Fire-path note: the radio (07000006) and the sky-bridge (07000012) tests drive Bond's real
//! trigger (`Sim::step_bond` fire → `Gun::update_with` → the shot block → the bullet pass), the
//! shape bullet_range_live/fix_06_fire_live pin. On 07000007 the headless harness has no
//! first-person camera feed for Bond's own spawn (the round spawns from a stale camera), so the
//! urn test hands the pass the fired round DIRECTLY (`player_bullets.push` — the same
//! `PlayerBullet` the shot block produces) and pins the pass → chain rotation → `SP_Hit` →
//! state machine chain behind it. Run with NF_ISO set.
//!
//! ```bash
//! NF_ISO=… cargo test -p nf-view --test urn_break_live
//! ```

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The intact urn's script entity: script 0x0600007A stream 9, glist 0x020004EE [D dump].
const URN_SCRIPT: u32 = 0x0600_007a;
const URN_STREAM: u8 = 9;
const URN: u32 = 0x0200_04ee;
const DEBRIS_STREAM: u8 = 1;
const DEBRIS: u32 = 0x0200_04ea;

/// The script entity `src`'s index and world matrix.
fn find_entity(sim: &nf_view::scene::Sim, src: u32, stream: u8, glist: u32) -> Option<(usize, [[f32; 4]; 4])> {
    sim.drones.scripts.objects.list.iter().position(|o| matches!(o,
        Some(nf_game::nis::Obj::Entity { hash, src: (s, st), .. }) if *s == src && *st == stream && *hash == glist))
        .map(|i| (i, match &sim.drones.scripts.objects.list[i] {
            Some(nf_game::nis::Obj::Entity { matrix, .. }) => *matrix,
            _ => unreachable!(),
        }))
}

/// The glist's local bounding sphere through the entity matrix — the game's
/// `Control_BuildWorldSph` volume [M 0x133070:36-48]; no model record → the translation with
/// the no-record radius 0.1 [M :19-27, :61-66].
fn sphere(sim: &nf_view::scene::Sim, glist: u32, m: &[[f32; 4]; 4]) -> ([f32; 3], f32) {
    match sim.models.get(&glist) {
        Some(md) => (
            [m[0][0] * md.centre[0] + m[1][0] * md.centre[1] + m[2][0] * md.centre[2] + m[3][0],
             m[0][1] * md.centre[0] + m[1][1] * md.centre[1] + m[2][1] * md.centre[2] + m[3][1],
             m[0][2] * md.centre[0] + m[1][2] * md.centre[1] + m[2][2] * md.centre[2] + m[3][2]],
            md.radius),
        None => ([m[3][0], m[3][1], m[3][2]], 0.1),
    }
}

/// Stand Bond beside `target`, take the level's own drawn weapon and hold the trigger 3
/// ticks (the release-frame spawn family). Waits out the intro cut-scene and the holster
/// cycle first.
fn fire_at(sim: &mut nf_view::scene::Sim, from: [f32; 3], target: [f32; 3]) {
    for _ in 0..900 {
        if !sim.disabled && sim.gun.switch_state == 0 { break; }
        sim.step_bond(Default::default(), false, false, false);
    }
    sim.bond.pos = from;
    let eye = [from[0], from[1] + 0.70, from[2]];
    let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    let dir = [d[0] / l, d[1] / l, d[2] / l];
    sim.bond.yaw = dir[0].atan2(dir[2]);
    sim.bond.pitch = dir[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
    for t in 0..3u32 {
        let mut input = Input::default();
        input.fire = true;
        sim.step_bond(input, true, false, false);
        let _ = t;
    }
}

/// A fired P2K round the shot block would hand the pass (the PlayerBullet shape) — the urn
/// test's harness step (see the module note).
fn p2k_round(from: [f32; 3], at: [f32; 3]) -> nf_game::drone::PlayerBullet {
    let d = [at[0] - from[0], at[1] - from[1], at[2] - from[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    nf_game::drone::PlayerBullet {
        pos: from, dir: [d[0] / l, d[1] / l, d[2] / l],
        speed: 13.2, travelled: 0.0, range: 50.0, damage: 2.75, weapon: 2,
        first: true, dead: false, bounces: 0, homing: false, target: None,
        gravity: false, guided_cam: false, snd: 0, swoosh: None, born: 0,
    }
}

/// The urn: a P2K round through the intact urn starts the break script — state 1 holds on the
/// contact tick (Collide after the walk [M 0x1336f8:238]) and flips to 2 on the next walk
/// [M SP_Update case 1]; the debris pieces spawn and the intact urn dies at the replay's t23.
#[test]
fn shooting_the_urn_starts_its_break_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let (pi, centre, radius) = {
        let sim = scene.sim.as_ref().unwrap();
        let pi = sim.drones.scripts.players.iter().position(|p| p.placement == 312).expect("the urn SP #312 [D census]");
        let p = &sim.drones.scripts.players[pi];
        assert_eq!(p.state, 1, "the mode-64 prime leaves the urn primed [M 0x1dbc08]");
        assert_eq!(p.mask, 2, "the urn's trigger mask is the bullet bit [D census]");
        let (_, m) = find_entity(sim, URN_SCRIPT, URN_STREAM, URN).expect("the intact urn entity from the load beat [D stream 9]");
        let (c, r) = sphere(sim, URN, &m);
        (pi, c, r)
    };
    eprintln!("[urn] intact entity at {centre:?} r {radius}");

    // The fired round: from 1.5 units south, through the intact urn's BuildWorldSph sphere.
    let from = [centre[0], centre[1] - 0.35, centre[2] + 1.5];
    scene.sim.as_mut().unwrap().drones.player_bullets.push(p2k_round(from, centre));

    // The one-frame order: the pass RECORDS the contact this tick (the SP still primed), the
    // next walk CONSUMES it (state 2).
    let mut recorded_at: Option<u32> = None;
    let mut consumed_at: Option<u32> = None;
    let mut state_at_record = 0u16;
    for t in 1..40u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[pi];
        if recorded_at.is_none() && !p.tick_bullet_hits.is_empty() {
            recorded_at = Some(t);
            state_at_record = p.state;
        }
        if recorded_at.is_some() && !p.bullet_hits.is_empty() {
            consumed_at = Some(t);
            break;
        }
    }
    let rec = recorded_at.expect("the round never touched the urn's stream object — the gather broke");
    assert_eq!(state_at_record, 1,
        "the SP must still be primed on the tick that records the contact — Collide runs AFTER the walk [M 0x1336f8:238]");
    let cons = consumed_at.expect("the recorded contact never rotated into the walk's read set");
    assert_eq!(cons, rec + 1, "the consumption is the NEXT walk after the record — the one-frame order");
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.scripts.players[pi].state, 2,
            "the bullet bit starts the break script [M SP_Update 0x1dbda8 case 1]");
    }
    // The break plays: the debris piece (stream 1, its Entity at the replay's t2) spawns [D
    // script dump]. The intact urn's own EntityEnd sits at the script's last tick — whether
    // the game runs it before the length stop (the intact art vanishing) is NOT pinned here
    // (needs a live boot): the port's SP_Delete keeps the stream objects, so this test only
    // requires the debris, and logs the intact urn's presence.
    let mut saw_debris = false;
    let mut urn_gone = false;
    for _ in 0..70u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if find_entity(sim, URN_SCRIPT, DEBRIS_STREAM, DEBRIS).is_some() { saw_debris = true; }
        if find_entity(sim, URN_SCRIPT, URN_STREAM, URN).is_none() { urn_gone = true; }
        if saw_debris && urn_gone { break; }
    }
    assert!(saw_debris, "the break script's debris stream never spawned");
    eprintln!("[urn] after the replay: intact urn entity still present = {}", !urn_gone);
}

/// Sister: an unshot urn stays primed — state 1, the intact urn alive, no debris, for 300
/// ticks (5 s of party).
#[test]
fn an_unshot_urn_stays_primed() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let pi = scene.sim.as_ref().unwrap().drones.scripts.players.iter()
        .position(|p| p.placement == 312).expect("the urn SP #312 [D census]");
    for t in 0..300u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[pi];
        assert_eq!(p.state, 1, "the unshot urn drifted out of the primed state at tick {t}");
        assert!(p.tick_bullet_hits.is_empty() && p.bullet_hits.is_empty(), "phantom bullet contacts at tick {t}");
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(find_entity(sim, URN_SCRIPT, URN_STREAM, URN).is_some(), "the intact urn entity is gone without a shot");
    assert!(find_entity(sim, URN_SCRIPT, DEBRIS_STREAM, DEBRIS).is_none(), "debris spawned without a shot");
}

/// The radio (07000006 #807, modes [8, 1]): the looping script A (0x06000055, the radio's
/// class-0 art glist 0x02000408 [D dump]) owns the primed object; the bullet bit switches it
/// to script B (0x06000056) and plays it once — state 6 → 2, cur 0 → 1 [M SP_Update case 6].
/// Bond fires his real trigger from beside the radio.
#[test]
fn shooting_the_radio_swaps_it_to_its_b_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    const RADIO_SCRIPT: u32 = 0x0600_0055;
    const RADIO: u32 = 0x0200_0408;
    let (pi, centre, radius) = {
        let sim = scene.sim.as_ref().unwrap();
        let pi = sim.drones.scripts.players.iter().position(|p| p.placement == 807).expect("the radio SP #807 [D census]");
        let p = &sim.drones.scripts.players[pi];
        assert_eq!(p.state, 6, "the mode-8 prime loops the radio's script A [M 0x1dbc08 case 8]");
        assert_eq!(p.scripts, [RADIO_SCRIPT, 0x0600_0056], "the radio's two scripts [D census]");
        let (_, m) = find_entity(sim, RADIO_SCRIPT, 0, RADIO).expect("the radio's looping art entity");
        let (c, r) = sphere(sim, RADIO, &m);
        (pi, c, r)
    };
    eprintln!("[radio] art at {centre:?} r {radius}");
    {
        let sim = scene.sim.as_mut().unwrap();
        fire_at(sim, [centre[0], centre[1] - 0.2, centre[2] + 1.2], centre);
    }
    let mut hit = false;
    for _ in 0..10u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.scripts.players[pi];
        if p.state == 2 && p.cur == 1 { hit = true; break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let p = &sim.drones.scripts.players[pi];
    assert!(hit, "the shot never moved the radio SP (state {} cur {})", p.state, p.cur);
    assert_eq!(p.scripts[p.cur], 0x0600_0056, "the radio now plays script B [D census]");
}

/// The sky bridge (07000012 #660, hp 120): its primed stream-0 object is a class-0 entity
/// with NO glist record (script 0x06000530 stream 0 carries glist 0 [D dump]) — the game's
/// `Control_BuildWorldSph` no-record sphere is the matrix translation with radius 0.1 [M
/// 0x133070:19-27, :61-66], and that is the object the Collide pass hangs the class-5 record
/// on. Bond's real trigger held 3 ticks starts the break script (state 1 → 2 [M case 1]).
#[test]
fn shooting_the_sky_bridge_starts_its_break_script() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000012.bin", false).expect("load 07000012");
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    const BRIDGE_SCRIPT: u32 = 0x0600_0530;
    let (pi, centre) = {
        let sim = scene.sim.as_ref().unwrap();
        let pi = sim.drones.scripts.players.iter().position(|p| p.placement == 660).expect("the bridge SP #660 [D census]");
        let p = &sim.drones.scripts.players[pi];
        assert_eq!(p.state, 1, "the mode-64 prime leaves the bridge primed [M 0x1dbc08]");
        assert_eq!(p.hit_points, 120.0, "the bridge carries 120 hit points [D census]");
        let (_, m) = find_entity(sim, BRIDGE_SCRIPT, 0, 0).expect("the bridge's primed stream-0 entity");
        let (c, r) = sphere(sim, 0, &m);
        assert!(r <= 0.1001, "the no-glist sphere is the game's 0.1, got {r}");
        (pi, c)
    };
    eprintln!("[bridge] primed root at {centre:?}");
    {
        let sim = scene.sim.as_mut().unwrap();
        fire_at(sim, [centre[0], centre[1], centre[2] + 1.2], centre);
    }
    let mut hit = false;
    for _ in 0..10u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.scripts.players[pi].state == 2 { hit = true; break; }
    }
    assert!(hit, "the shot never started the bridge's break script (state {})",
        scene.sim.as_ref().unwrap().drones.scripts.players[pi].state);
}
