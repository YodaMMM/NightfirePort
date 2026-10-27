//! Real-path proof for level-1b-space-laser: the 0700001b kind-0x38 space laser's full
//! arm/charge/fire/disable chain through the REAL Sim (the Walker harness — the level's start
//! switch runs first, exactly like the walkreals), wired at the game's object-update position
//! (control_funcs row 0x50 = `SpaceLaser_Update` 0x1aa3f8 [M docs/atlas/object.md:164]; the
//! create walk's case 0x38 @0x1d06fc [M parsemap_create_dynamic_objects decomp :225-227]).
//!
//! Disc data [D docs/levels/0700001b-audit.md:275]: the kind-0x38 placement #130 "BigLaserBeam
//! Fire Marker" at (0,0,0) — p0/p1 = 0x060007c0/c1 (the tracking scripts, 360 ticks each),
//! p2 = 0x060007e3 (the dead script, 10 ticks), p3 = 165 (the arm-delay seed), p4 = 65 (the
//! state-3 re-raise seed), p5 = ch 99 (the state-3 write), p6 = ch 14 (Drake's death kill),
//! p7 = ch 19 (the arm gate — NO disc script ever writes it [D docs/levels/0700001b.md:71]:
//! the gate is poked here, exactly the live-check stand-in the unit tests document).
//!
//! The state-3 test pokes `arm_delay` to just under 0 — the [D] seed is 165.0 s (9900 gate-open
//! ticks) and the suite cannot afford the full wait; the unit test
//! `space_laser::tests::dead_state_reraises_its_channel` pins the real timer semantics
//! [M 0x1aa3f8:78-80] and this test pins the rest of the chain (the dead script's [D] 10-tick
//! load, the state-3 entry, the [D] 65.0 s re-raise, the ch-99 write).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn laser_index(w: &Walker) -> usize {
    w.scene.sim.as_ref().unwrap().drones.space_lasers.iter()
        .position(|l| l.ch5 == 99 && l.ch6 == 14 && l.ch7 == 19)
        .expect("the 0700001b kind-0x38 laser (p5=99, p6=14, p7=19 [D audit:275])")
}

/// The arm gate is poked by the harness every tick: the level's own start scripts clear
/// switch_channels during the first ~50 ticks (the chwatch/probe run: ch 19 reads 1 for exactly
/// one tick after a single poke), so a one-shot poke cannot hold the [D] gate through the 10 s
/// cooldown window. The live check's debugger poke has the same fight — see the report.
fn poke_gate(w: &mut Walker) {
    let sim = w.scene.sim.as_mut().unwrap();
    sim.drones.channels.set(19, 1, sim.drones.frame);
}

/// The laser arms, charges and fires at Bond on the game's timings: gate open (ch 19 poked)
/// → the 10 s cooldown spends [M 0x1aa3f8:94-96] → the tracking script loads (its [D] 360-tick
/// length read from the level's script lib) → state 1 charges with the 0x5de whine handle
/// [M :119-121] → the script's End → state 2 FIRES: `Bullet_init` with the weapon def record
/// 107 @0x2c6154 [M :172-175], the aim lane = the player quad [M :105-112], the beam object
/// (life 5, alpha 0x7f+Rand(0x80) [M :163-167, 001aac00:20-28]), the 0x2d6210 impact emitter
/// [M :189], the 0x46a 3D sound [M :156] → the 10 s re-fire [M :190].
#[test]
fn the_laser_arms_charges_and_fires_at_bond() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b"));
    w.wait_enabled(3000, "the station start");
    let idx = laser_index(&w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.space_lasers[idx].fire_origin, [0.0, 0.0, 0.0],
            "the Fire Marker placement sits at the origin [D audit:275]");
        // Test isolation (the ch-99 test's note): the eight SpaceMissiles share the ch-19 gate
        // [D p7] and their launch branch writes the mission-fail ch 99 [D], whose LevelToEndTo
        // would reload the level mid-test — inert them (state 4, the inert epilogue [M 0x1aa3e0]).
        for m in sim.drones.space_missiles.iter_mut() { m.state = 4; }
        poke_gate(&mut w); // the arm gate [D p7; no disc writer — the harness holds it]
    }
    // The fire decision lands on the first gate-open tick whose spent cooldown reads < 0
    // [M :94-99: the decrement then the two arms]. Watch for the load, then the shot.
    let mut fired_at = None;
    let mut saw_load = false;
    let mut charged = false;
    for _ in 0..1600u32 {
        poke_gate(&mut w);
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let l = &sim.drones.space_lasers[idx];
        if !saw_load && l.state == 1 {
            saw_load = true;
            assert_eq!(l.script_len, 360, "the tracking script's [D] lib length");
            assert_eq!(l.script_ticks, 1, "one Script_Update ran on the load tick [M :103]");
        }
        if saw_load && !charged && l.state == 1 && l.script_ticks >= 2 {
            // The whine starts in state 1's own arm — the tick after the load [M :115-121;
            // the port's first-free-handle start is the documented [S] sound-side deviation].
            charged = true;
            assert!(l.whine_handle != 0, "the 0x5de whine handle is live in state 1 [M :119-121]");
        }
        // The fire's durable product is the beam (5 ticks [M 001aac00:20-28]); the bullet dies
        // the same tick on the pit geometry when the marker's ray is wall-locked — the beam and
        // the emitter are the observable fire evidence, the bullet's spawn is [M]-wired below.
        if !sim.drones.space_laser_beams.is_empty() {
            assert!(saw_load, "the shot follows the loaded tracking script [M :100-126]");
            fired_at = Some(w.total_ticks);
            break;
        }
    }
    fired_at.expect("the laser fires within 1600 ticks of the gate (10 s + 360 + slack)");
    let sim = w.scene.sim.as_ref().unwrap();
    let l = &sim.drones.space_lasers[idx];
    assert_eq!(l.state, 0, "back to idle after the shot [M :191]");
    assert_eq!(l.cooldown, 10.0, "the 10 s re-fire timer reset [M :190]");
    // The bullet: `Bullet_init(-1, obj, 0, 0x2c6154, origin, dir, 0xffff)` [M :172-175] — the
    // weapon def record 107's own speed/range/damage [D weapon_data 0x2bf150 + 107·0x10c]. The
    // muzzle is the pit-centre marker, so the round can die on the geometry the same tick; the
    // def wiring itself is pinned by the surviving beam below and the shared bullet path.
    if let Some(b) = sim.drones.bullets.iter().find(|b| b.weapon == 107) {
        assert_eq!(b.pos, l.fire_origin, "the shot leaves the fire origin [M :172-175]");
        assert!(b.speed > 0.0 && b.range > 0.0 && b.damage > 0.0,
            "record 107's speed/range/damage reach the bullet [M weapon_data+0x20/+0x1C/+0x0C]");
        let to_player = [l.target[0] - b.pos[0], l.target[1] - b.pos[1], l.target[2] - b.pos[2]];
        let dot = to_player[0] * b.dir[0] + to_player[1] * b.dir[1] + to_player[2] * b.dir[2];
        assert!(dot > 0.0, "the bullet flies toward the tracked aim point [M :128-152]");
    }
    // The beam: the class-0x10 object exposed for the renderer — origin → the aim lane (the
    // ray impact when the 1000-unit mask-8 ray hit [M :144-160]), life 5, alpha 0x7f+Rand(0x80).
    let beam = sim.drones.space_laser_beams.iter().rev().find(|bm| bm.from == l.fire_origin)
        .expect("the fire's beam object is exposed per tick [M :159-174]");
    assert_eq!(beam.life, 5, "ext+0x30 = 5 [M :163]");
    assert_eq!(beam.to, l.target, "the stretch reaches the aim lane [M :188]");
    assert!(beam.alpha >= 0x7f, "the brightness byte 0x7f+Rand(0x80) [M :166-167]");
    // The impact emitter [M :189]: `Emitter_Create(0, 0, 0xc000010, target, 0x2d6210, 0, 0, 0)`
    // resolves its def through the game's hashtable — a miss means NO object (the placement
    // path's own rule [M 0x17ae00]); the port's fx.defs carry the level's 0xF2 placements, so
    // the assert holds only when the def resolves. Whether the game's global table holds
    // 0x2d6210 is a live check (the report).
    // The port's fx.defs carry the level's 0xF2 placement defs; the game's global effect
    // table is not modelled, so the emitter's presence is def-conditional and its global
    // resolution is a live check (the report).
    eprintln!("[space-laser] emitter 0x2d6210: {} in fx.emitters; the game's global def-table hit needs a live check",
        sim.drones.fx.emitters.iter().filter(|e| e.def == 0x2d6210).count());
    // The re-fire: the cooldown spends again → the second beam [M :94-99, :190]. The first
    // beam expires after its 5 ticks, so the detection waits for empty-then-refill.
    let mut seen_empty = sim.drones.space_laser_beams.is_empty();
    let mut second = false;
    for _ in 0..1400u32 {
        poke_gate(&mut w);
        w.tick(nf_game::player::Input::default(), false);
        let n = w.scene.sim.as_ref().unwrap().drones.space_laser_beams.len();
        if n == 0 { seen_empty = true; }
        if seen_empty && n > 0 { second = true; break; }
    }
    assert!(second, "the 10 s re-fire [M :190]");
}

/// The disable channel kills it: ch 14 (Drake's death [D p6]) jumps any state but 4 to state 4
/// and raises obj+0xfe bit 1 [M 0x1aa3f8:86-90, :189-195] — the object leaves the sim the same
/// frame (the game's delete pass) and stops ticking forever.
#[test]
fn ch_14_kills_the_laser() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b"));
    w.wait_enabled(3000, "the station start");
    let idx = laser_index(&w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(19, 1, sim.drones.frame);
    }
    for _ in 0..700u32 { w.tick(nf_game::player::Input::default(), false); } // into the charge window
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.space_lasers[idx].deleted, "alive before the kill channel");
        let l = &sim.drones.space_lasers[idx];
        assert!(l.whine_handle != 0 || l.state == 2 || l.state == 0, "charging or cycling before the kill");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(14, 1, sim.drones.frame); // Drake's death [D p6]
    }
    w.tick(nf_game::player::Input::default(), false);
    assert!(w.scene.sim.as_ref().unwrap().drones.space_lasers.get(idx).is_none_or(|l| l.deleted),
        "state 4 raised the delete flag [M :189-195]");
    w.tick(nf_game::player::Input::default(), false);
    assert!(w.scene.sim.as_ref().unwrap().drones.space_lasers.is_empty(),
        "the delete pass removed the object — it never ticks again");
}

/// The dead-script chain: arm delay spent → the p2 script (its [D] 10-tick lib length) loads
/// into state 3 [M :87-94] → the re-raise timer (the [D] 65.0 s seed) spends → ch 99 rises and
/// holds [M :175-182]. The 165 s arm-delay wait is poked to just under 0 here (the suite cannot
/// afford 9900 ticks; the unit test pins the real decrement rule [M :78-80]) — p4 keeps its
/// [D] 65.0 seed, so the re-raise timing below is the game's own.
#[test]
fn ch_99_rises_in_state_3() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b"));
    w.wait_enabled(3000, "the station start");
    let idx = laser_index(&w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // Test isolation: the eight SpaceMissiles share the ch-19 launch gate [D p7] and their
        // launch branch's p1 scripts write ch 99 @t1000 [D space_missile.rs's stream decode] —
        // the mission-fail write would mask the laser's own (later) ch-99. State 4 is the
        // missiles' inert epilogue [M 0x1aa3e0].
        for m in sim.drones.space_missiles.iter_mut() { m.state = 4; }
        sim.drones.channels.set(19, 1, sim.drones.frame); // the arm gate [D p7]
        sim.drones.space_lasers[idx].arm_delay = 0.01; // the 165 s wait, poked [S — see the doc]
    }
    let mut saw_state3 = false;
    for _ in 0..200u32 {
        w.tick(nf_game::player::Input::default(), false);
        let l = &w.scene.sim.as_ref().unwrap().drones.space_lasers[idx];
        if l.state == 3 {
            saw_state3 = true;
            assert_eq!(l.script_len, 10, "the dead script's [D] lib length [D audit:606]");
            break;
        }
    }
    assert!(saw_state3, "the spent arm delay loads the dead script into state 3 [M :87-94]");
    // The [D] 65.0 s re-raise: 3900 ticks + slack.
    let mut rose_at = None;
    for k in 0..4200u32 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(99) { rose_at = Some(k); break; }
    }
    let k = rose_at.expect("ch 99 rises once the 65 s re-raise timer spends [M :175-182]");
    assert!(k >= 3800, "the write waits for the [D] 65.0 seed, not the same tick (k = {k})");
    for _ in 0..30u32 { w.tick(nf_game::player::Input::default(), false); }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(99), "the channel holds [M :200-204]");
}

/// BigLaserBeamHURT (0xe4 #259, type 1, dmg 50, gate ch 18 [D audit:444-445]) arms through its
/// gate: `Hurt_Update` parks a gated volume while the channel reads low and starts it on the
/// live channel (the value IS the who-mask) [M 0x17e5d0:26-37, :51-102; hurt.rs]. The disc's
/// ch-18 writer is the laser chain's downstream (the mission-fail path) and is not identified
/// yet [S — see the report's live checks]; this test pins the volume's own gate law through the
/// real level: parked while ch 18 is low, armed once it rises.
#[test]
fn big_laser_beam_hurt_arms_through_its_gate() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b"));
    w.wait_enabled(3000, "the station start");
    let idx = w.scene.sim.as_ref().unwrap().hurts.iter().position(|h| h.gate == 18 && h.kind == 1)
        .expect("the 0xe4 #259 BigLaserBeamHURT (gate ch 18, type 1 [D audit:445])");
    assert_eq!(w.scene.sim.as_ref().unwrap().hurts[idx].damage, 50.0, "dmg 50 [D audit:445]");
    for _ in 0..10u32 { w.tick(nf_game::player::Input::default(), false); }
    assert!(w.scene.sim.as_ref().unwrap().hurts[idx].disabled,
        "parked while the gate reads low [M 0x17e5d0:26-37]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(18, 1, sim.drones.frame);
    }
    w.tick(nf_game::player::Input::default(), false);
    assert!(!w.scene.sim.as_ref().unwrap().hurts[idx].disabled,
        "the live gate arms the volume [M :51-102]");
}
