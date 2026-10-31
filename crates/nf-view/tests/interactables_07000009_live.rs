//! interactables-09: the player-touchable kinds of 07000009 (Night Shift 1, Tower A) no other
//! live test of the level drives yet — the per-kind census battery, modelled on
//! interactables_0700000b_live.rs (one live test per KIND, driven the way a player drives it,
//! each asserting the game's effect with a sister).
//!
//! The kind census (m39 placements [D census_07000009 dump + docs/levels/07000009.md]):
//! - ALREADY DRIVEN elsewhere: the alarm Switches 0x29 on ch 0x7C/0x7D (+ the no-press sister),
//!   the breakable glass 0x20 (the real fire path + the 0x30-filter refusal), the E-doors 0xdb
//!   (+ the locked shutter), the armour pickup (take + parked-short sisters), the Load Point's
//!   E-refusal at the closed doors, the liftside touch plate ch 0xAB and the guard kill —
//!   interact_07000009_live.rs; the arrival plate ch 0x51, the elevator panel activation
//!   (SP #327 → ch 0x39 → obj 2) and the exit chain (ch 0x32 → 0700000a) —
//!   walkreal_07000009_live.rs; the script-lift call stations (SP #278/#279 through their
//!   sub-script entities: light, ride, doors, out-0 silence) — garage_lift_calls_live.rs; the
//!   lift indicator art — lift_indicators_09_live.rs.
//! - ADDED HERE (the kinds nothing on 09 drove yet): the kind-6 BOND_BONUS pickups (2 [D
//!   #525/#526] — the gold-medal-gated creation + the walk-over take), the hint markers 0x33
//!   (2 [D #539/#540] — the walk-in INFO-pane fire), and the LIFTDOORS SS pair's positive
//!   channel-gated open (focused: the matrices slide on ch 0x39 [D params; M SS_Update]).
//! - ZERO player-touchable placements on 09 [D census; M 0x1d04f0 case list]: locks 0x2E,
//!   monitors 0x30 (no 0x30 placement in any of the 40 maps — the security-room wall is baked
//!   art), grapple 0x3A, gadget targets 0x35-0x38, copters 0x2F, turrets/sensors (no kind).
//!   NOT player-touchable: the muxes 0xEC/0xEF, the SoundTriggers 0xF9 (audio presentation
//!   riding the already-pinned channels), the FuseBoxes 0x31 (23 boxes, NO params on 09 — no
//!   channels [D]), rain 0x32/0xDF, skylines 0x2A, CEL rooms 0x47, the rotor 0x71, AI infra
//!   0xE9/0xF5, scenery 0x19, the player start 0x24.
//!
//! Transits: the spawn→lobby walk is walkreal_07000009_live's proven route; the per-kind
//! battery does not re-run that multi-thousand-tick walk per test (the accepted battery
//! precedent, interactables_07000002_live.rs header) — the teleports below only REACH the
//! spots, and every operation after the settle runs through the real input.
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The fresh-boot load (the same defaults nf_view::load uses — difficulty 1, bonus 0), settled
/// through the intro until Bond is free.
fn load_09(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000009.bin", false).expect("load 07000009");
    settle_intro(&mut scene);
    scene
}

/// The medal-carrying load: `load_with_carry_medals` with `bits` as the profile's `bonus[0]`
/// word — the game's `bonus[]` global survives every map load and the kind-6 gate reads it on
/// every level [M 0x18dda0:93-95; the port's pickup-medal-gate]. Difficulty 1 = the replay-1
/// default of the direct-load wrappers (the same difficulty nf_view::load keeps).
fn load_09_bits(iso: &str, bits: u64) -> nf_view::scene::Scene {
    let mut scene =
        nf_view::load_with_carry_medals(iso, "07000009.bin", false, None, 1, bits).expect("load 07000009 with medals");
    settle_intro(&mut scene);
    scene
}

fn settle_intro(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");
}

/// The gold-medal `bonus[0]` word for BaseMap 0x07000009 [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm; the 0x0700005/9/11 kind-4 group's rank 1
/// [L 0x2bc920 row 5; M 0x199210]] — the same word the 0b battery pins for its 09-BaseMap
/// pickups. pickup-medal-gate.
fn gold_bits() -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(0x0700_0009, 3, 0);
    p.bonus[0]
}

/// Settle Bond at a spot (a teleport TO the observation point; everything after is real input).
fn settle_near(scene: &mut nf_view::scene::Scene, spot: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let floor = sim.world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
    if let Some(f) = floor { sim.bond.pos = [spot[0], f + 0.1, spot[2]]; } else { sim.bond.pos = spot; }
    sim.bond.vel = [0.0; 3];
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
}

fn face(scene: &mut nf_view::scene::Scene, at: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let b = sim.bond.pos;
    sim.bond.yaw = (at[0] - b[0]).atan2(at[2] - b[2]);
}

// ---------------------------------------------------------------------------
// KIND Pickup, kind 6 BOND_BONUS — #525 "Pickup 007" at (−0.82, −369.14, 91.62) and #526 at
// (−23.27, −376.16, 37.48), params (0:6, 1:6, 4:0xffff, 6:0x020004a) [D placements]. The
// kind-6 row is created ONLY with the BaseMap's gold medal recorded [M Pickup_Create 0x18dda0:
// 93-95], 07000009 plays under BaseMap 0x07000009 [L ee_start_05.bin @0x2a4354]. The walk-over
// takes it through the shared Pickup_Handler take path — the kind-6 arm consumes the pickup
// [M Pickup_Handler 0x18e650:234-236] (the spin art is the kind's own [M 0x18e650 kind-6 arm;
// pickup.rs spin = kind::BOND_BONUS]).
// ---------------------------------------------------------------------------

/// The gold-medal load spawns both bonuses; walking over the car-park one consumes it. Sister:
/// the lobby one still sits after the take (its own walk is the lobby route's, not re-run here).
#[test]
fn the_gold_medal_spawns_the_bonus_pickups_and_the_walk_over_takes_one() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09_bits(&iso, gold_bits());
    // Both bonuses exist at load, at their disc spots [D #525/#526].
    {
        let sim = scene.sim.as_ref().unwrap();
        let at = |x: f32, z: f32| sim.drones.pickups.iter()
            .find(|p| p.kind == nf_game::pickup::kind::BOND_BONUS
                && (p.m[3][0] - x).abs() < 0.5 && (p.m[3][2] - z).abs() < 0.5);
        assert!(at(-0.82, 91.62).is_some(), "the lobby bonus #525 exists at its disc spot [D]");
        assert!(at(-23.27, 37.48).is_some(), "the car-park bonus #526 exists at its disc spot [D]");
    }
    // The walk-over: stand short of #526 (the car park, ~27 west of the spawn), then real
    // forward input until the take lands.
    settle_near(&mut scene, [-22.0, -376.0, 37.48]);
    face(&mut scene, [-23.27, -376.16, 37.48]);
    let mut taken = false;
    for _ in 0..300 {
        {
            let sim = scene.sim.as_ref().unwrap();
            if sim.drones.pickups.iter()
                .any(|p| p.kind == nf_game::pickup::kind::BOND_BONUS && (p.m[3][0] + 23.27).abs() < 0.5 && p.gone) {
                taken = true; break;
            }
        }
        scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(taken, "the walk-over consumes the car-park bonus [M 0x18e650:234-236]");
    assert!(sim.drones.pickups.iter()
        .any(|p| p.kind == nf_game::pickup::kind::BOND_BONUS && (p.m[3][0] + 0.82).abs() < 0.5 && !p.gone && !p.hidden),
        "sister: the lobby bonus #525 still sits after the car-park take [D #525]");
}

/// Sister (the creation gate): the fresh-boot bonus word (0) creates NO kind-6 at all, while
/// the armour kind-3 row still exists — the same load built the rest of the pickups, so the
/// absence is the gate's own [M 0x18dda0:93-95].
#[test]
fn without_the_gold_medal_the_bonus_kind_is_not_created() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = load_09_bits(&iso, 0);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.pickups.iter().all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → no kind-6 bonus created [M 0x18dda0:93-95]");
    assert!(sim.drones.pickups.iter().any(|p| p.kind == 3 && (p.m[3][0] + 22.57).abs() < 0.5),
        "positive control: the armour kind-3 row exists on the same load [D #527]");
}

// ---------------------------------------------------------------------------
// KIND Hint marker (0x33) — #539 at (−20.90, −368.61, 26.37) and #540 at (−18.01, −368.61,
// 26.34), param 0 = 0x01000291 (the use-key hint text [D placements; USATxt.dat]), param 1 = 0
// (silent [D]), stage param 2 = 0 (no gate [D]). `Hint_Create` 0x1b3248 builds the marker off
// the placement (parsemap case 0x33 [M 0x1d04f0:194-198]); `Hint_Update` 0x1b3380 fires it once
// with the player in its chain: the type-1 INFO-pane message `Text_AddMsg(..., label, 0, 0xb4)`
// and the delete bit [M 0x1b3380:24-42].
// ---------------------------------------------------------------------------

/// The markers register with the disc label and the walk-in fires one: the message lands on the
/// INFO pane queue and the marker is spent. Sister: the other marker (2.9 east) stays unfired —
/// the walk stops between them.
#[test]
fn the_hint_markers_carry_the_disc_label_and_fire_on_the_walk_in() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.hints.len(), 2, "the census: two 0x33 placements [D #539/#540]");
        assert!(sim.hints.iter().all(|h| h.label == 0x0100_0291),
            "the use-key hint label [D #539/#540 param 0]");
        assert!(sim.hints.iter().all(|h| h.sound == 0), "no hint sound on 09 [D param 1 = 0]");
        assert!(sim.hints.iter().all(|h| h.stage == 0), "no stage gate on 09 [D param 2 = 0]");
    }
    // West marker #539: settle 1.3 west of its centre, then a real walk east into it. The lobby
    // teleports in on the walkreal precedent; the walk into the sphere is the real input.
    let hc = scene.sim.as_ref().unwrap().hints.iter()
        .find(|h| (h.centre[0] + 20.90).abs() < 0.3).map(|h| h.centre)
        .expect("the west hint marker #539 [D]");
    settle_near(&mut scene, [hc[0] - 1.3, hc[1] - 0.6, hc[2]]);
    face(&mut scene, hc);
    let mut fired_i: Option<usize> = None;
    for _ in 0..240 {
        {
            let sim = scene.sim.as_ref().unwrap();
            if let Some(hi) = sim.hint_fired.iter().position(|&f| f) { fired_i = Some(hi); break; }
        }
        scene.sim.as_mut().unwrap().step_bond(Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let hi = fired_i.expect("the walk-in fires the west marker [M Hint_Update 0x1b3380:24-42]");
    assert!((sim.hints[hi].centre[0] + 20.90).abs() < 0.3, "the fired one is the west marker #539");
    // The INFO pane holds the hint: the drain binds the label into `info_msgs` per tick (the
    // queue itself empties every frame — scene.rs's drain is the game's post-time bind [M
    // Txt_BindLabel inside Hint_Update 0x1b3380:31]), life 180 ticks [M Text_AddMsg life 0xb4].
    assert!(sim.hud.info_msgs.iter().any(|m| m.label == 0x0100_0291),
        "the INFO pane holds the hint label 0x01000291 [D #539/#540 param 0; M 0x1b3380:31]");
    let other = sim.hints.iter().position(|h| (h.centre[0] + 18.01).abs() < 0.3)
        .expect("the east marker #540 [D]");
    assert!(!sim.hint_fired[other], "sister: the east marker stays unfired until its own walk");
}

// ---------------------------------------------------------------------------
// KIND SimpleScript (0xE2) — the LIFTDOORS_LEFT/RIGHT_ENDLEVELDOORS pair (#260 LEFT at
// (−4.48, −368.19, 13.19), #259 RIGHT at (−3.40, −368.19, 13.23)), params (0:0x40, 1:1, 2:1,
// 3:0x39, 4:0xffff, 5:0, 6:1, 7:0) [D placements]: state 0x40 waits for `SS_Operate()` =
// `switch_channels[57]` (param 3 = 57, param 2 = 1) [M SS_Create 0x1a9c58, SS_Operate 0x1a9df8,
// SS_Update 0x1a9948]; the panel press writes ch 0x39 [M 0x1dbda8:221-229] and the doors play
// their open spline (param 6 = 1 [D]). The E-refusal negative (mask = param 2 = 1 → `SS_Activate`
// needs ext+0x1E & 4 [M 0x1a9ea8]) is pinned by interact_07000009_live; this is the kind's
// POSITIVE arm, focused.
// ---------------------------------------------------------------------------

/// The panel press writes ch 0x39 and the SS doors' matrices slide apart. Sister (in-test):
/// before the press both doors rest at their load pose with ch 0x39 off.
#[test]
fn the_lift_doors_slide_open_on_the_panel_channel() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_09(&iso);
    // The panel stance (the lobby by the west desk — the walkreal transit reaches it; the
    // battery teleports in). The panel is knee-high (centre y −368.23 [D gfx bounds]); the use
    // pick is the 1.0 sphere at head+view [M 0x1a5170:100-108], so the aim pitches down.
    const PANEL: [f32; 3] = [-18.079, -368.227, 25.015];
    settle_near(&mut scene, [-16.6, -369.0, 27.5]);
    fn ss(sim: &nf_view::scene::Sim, left: bool) -> &nf_game::ss::Ss {
        // LEFT at x −4.48, RIGHT at x −3.40 [D placements; the sim's ids are the map-local #3/#2
        // the doc carries] — the level holds exactly two 0xE2 SS, so x disambiguates.
        let (mut a, mut b) = (None, None);
        for s in &sim.drones.sss.list { if a.is_none() { a = Some(s); } else { b = Some(s); } }
        let (a, b) = (a.expect("SS LEFT [D]"), b.expect("SS RIGHT [D]"));
        let l = if a.matrix[3][0] < b.matrix[3][0] { a } else { b };
        let r = if a.matrix[3][0] < b.matrix[3][0] { b } else { a };
        if left { l } else { r }
    }
    let before = {
        let sim = scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x39), "sister: ch 0x39 off before the press");
        [(ss(sim, true).matrix[3][0], ss(sim, true).matrix[3][2], ss(sim, true).state),
         (ss(sim, false).matrix[3][0], ss(sim, false).matrix[3][2], ss(sim, false).state)]
    };
    // The real E pulses at the panel (the walkreal press loop): walk the last 0.9 and pulse E.
    let mut fired = false;
    for k in 0..720 {
        {
            let sim = scene.sim.as_mut().unwrap();
            if sim.drones.channels.on(0x39) { fired = true; break; }
            let b = &mut sim.bond;
            let dy = PANEL[1] - (b.pos[1] + b.eye);
            let dx = PANEL[0] - b.pos[0];
            let dz = PANEL[2] - b.pos[2];
            let dxz = (dx * dx + dz * dz).sqrt();
            b.yaw = dx.atan2(dz);
            b.pitch = (dy / dxz).atan() / std::f32::consts::FRAC_PI_2;
            let forward = if dxz > 0.9 { 1.0 } else { 0.0 };
            sim.step_bond(Input { forward, ..Default::default() }, false, false, dxz <= 1.2 && k % 6 == 0);
        }
    }
    assert!(fired, "the E pulses at the panel write ch 0x39 [M 0x1dbda8:221-229]");
    // The doors play their open spline on the channel edge [M SS_Update 0x1a9948 state 0x40 → 1].
    let mut slid = 0.0f32;
    for _ in 0..160 {
        {
            let sim = scene.sim.as_mut().unwrap();
            let dx = (ss(sim, true).matrix[3][0] - before[0].0).abs()
                + (ss(sim, false).matrix[3][0] - before[1].0).abs();
            slid = dx;
            if slid > 0.2 { break; }
            sim.step_bond(Input::default(), false, false, false);
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    let l = ss(sim, true);
    let r = ss(sim, false);
    eprintln!("[ss] L x {:.2}→{:.2} state {}→{} | R x {:.2}→{:.2} state {}→{}",
        before[0].0, l.matrix[3][0], before[0].2, l.state, before[1].0, r.matrix[3][0], before[1].2, r.state);
    assert!(slid > 0.2, "the door matrices slide apart on ch 0x39 [M SS_Update 0x1a9948], slid {slid:.3}");
    assert!(l.state != before[0].2 || r.state != before[1].2,
        "the SS left the wait state 0x40 [M 0x1a9948 state 0x40 → 1 on the channel event]");
}
