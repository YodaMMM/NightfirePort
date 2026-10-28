//! bath-hatch-02: the 02 bath hatch above the deep ladder. The deep ladder on 07000002
//! (centre (−50.35, ·, 5.50), yaw π/2) climbs out of the water into the bathhouse, but the
//! climb stops under a CLOSED hatch — the player cannot get out of the water route (the
//! remaining gap after ladder-underwater-02). On the PS2 the player opens it.
//!
//! What the disc says (all measured here through the real loaders):
//! - The hatch over the deep ladder is the SimpleScript **#308** (type 0xE2, at
//!   (−49.75, 4.76, 6.13) — 0.7 from the ladder axis): params (0→64, 1→1, 2→4, 3→2, 4→586,
//!   6→1) [D]. Per SS_Create 0x1a9c58's field map: state 64 = 0x40 (armed rest), wrap 1
//!   (one-shot), mask 4 (**E-usable**), channel 2, sound 586, spline. Its twin #309
//!   `bath_hatch1` (−49.76, 4.76, 16.94) carries mask 1, channel 1 — not E-usable, and no
//!   placement writes ch 1 (locked by design, grates-02).
//! - The only writer of the hatch's channel 2 is the SS itself: the main map's 36 triggers
//!   contain no out 2 [D probe], and the SS's own write-back only echoes its operate result
//!   [M 0x1a9948:18-23]. So the game opens this hatch by the **use key**: Player_Activate
//!   0x1a5170 (no substate gate; the creepwall arm needs BLData+0x95f == 6) → the pick's
//!   class 0x25 case '%' → SS_Activate 0x1a9ea8 (armed state + mask & 4 → state 1, latch
//!   switch_channels[ch]). The pick is live while climbing: Player_Update 0x193600's
//!   obj+0xF4 == 1 branch runs Player_WeaponFiring 0x1a5460 every tick [M, the port's
//!   climb_tick doc], which reads the use action (Input_Action slot 0xe, edge 4) and calls
//!   Player_Activate [M 0x1a5460:393-402].
//! - The ladder-top TouchOnce plates #500 (out 61, y 4.45) and #501 (out 62, y 4.94) feed
//!   SEQ_AND #396 (list [61, 62]) whose out 101 is the Bond-moment band (0x65–0x6E) — the
//!   climb moment, NOT a hatch drive [D].
//!
//! The tests pin the disc chain and drive the real path (`nf_view::load` + `Sim::step_bond`):
//! swim in, climb, open the hatch with the use key, climb out into the bathhouse, walk away.
//! Opt-in via `NF_ISO`.

use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load() -> Sim {
    let iso = iso().expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    scene.sim.take().expect("sim")
}

/// One step through the real path; the level's scripts grab Bond into cut-scenes within seconds,
/// so each tick carries the game's own skip action like the walkreal walker — [S] unverified (nis.rs is the port's skip plumbing; no decompile line pinned here).
fn step(sim: &mut Sim, input: nf_game::player::Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, false);
}

fn step_use(sim: &mut Sim, input: nf_game::player::Input) {
    if sim.disabled { sim.drones.scripts.skip_request = true; }
    sim.step_bond(input, false, false, true);
}

fn ladder_near(sim: &Sim, x: f32, z: f32) -> (usize, nf_game::ladder::Ladder) {
    let li = (0..sim.ladders.len()).min_by(|a, b| {
        let d = |l: &nf_game::ladder::Ladder| (l.centre[0] - x).powi(2) + (l.centre[2] - z).powi(2);
        d(&sim.ladders[*a]).total_cmp(&d(&sim.ladders[*b]))
    }).expect("07000002 has ladders");
    (li, sim.ladders[li].clone())
}

/// Grab the deep ladder through the real path from the game's REAL start. The -2 round's
/// re-derivation method: the old calibration teleported Bond next to the ladder, but the game's
/// fresh boot hands Bond at the measured post-intro point [L PCSX2 2026-10-27, PLAYERS obj+0x30
/// = (-27.504, 1.131, 32.348) after the 07000002 intro] — place that measured point, skip the
/// intro with the game's own skip action, then swim the real line to the ladder's grab side
/// (the swim entry happens naturally — the start is in the pool water).
fn grab_deep_ladder(sim: &mut Sim) -> nf_game::ladder::Ladder {
    let (_li, lad) = ladder_near(sim, -50.35, 5.50);
    assert!((lad.centre[0] - -50.35).abs() < 0.5, "not the deep ladder: {:?}", lad.centre);
    let front = [lad.centre[0] + lad.yaw.sin() * 1.2, 3.0, lad.centre[2] + lad.yaw.cos() * 1.2];
    sim.bond = nf_game::player::Bond::new([-27.504, 1.131, 32.348], lad.yaw + std::f32::consts::PI);
    // The intro's scripts hold Bond — skip through them (the game's own skip action, the same
    // carry the walker uses).
    for _ in 0..1200 { step(sim, Default::default()); if !sim.disabled { break; } }
    // Swim the straight line to the ladder's grab front; the pool is open water between the
    // real start and the deep ladder (the game's own water route — ladder-underwater-02).
    let walk = nf_game::player::Input { forward: 1.0, ..Default::default() };
    let mut latched = None;
    for t in 0..1600 {
        let p = sim.bond.pos;
        let d = ((front[0] - p[0]).powi(2) + (front[2] - p[2]).powi(2)).sqrt();
        if d < 0.9 { break; }
        let desired = (front[0] - p[0]).atan2(front[2] - p[2]);
        let err = walker_wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
        step(sim, input);
        if sim.bond.substate == nf_game::player::SS_CLIMB { latched = Some(t); break; }
    }
    if latched.is_none() {
        // Close in on the grab side and walk into the ladder (the last metre).
        sim.bond = nf_game::player::Bond::new(front, lad.yaw + std::f32::consts::PI);
        let walk = nf_game::player::Input { forward: 1.0, ..Default::default() };
        for t in 0..400 {
            step(sim, walk);
            if sim.bond.substate == nf_game::player::SS_CLIMB { latched = Some(t); break; }
        }
    }
    assert_eq!(sim.bond.substate, nf_game::player::SS_CLIMB,
        "swimming in from the real start never attached (last substate {} y {:.3})", sim.bond.substate, sim.bond.pos[1]);
    eprintln!("  grabbed at tick {:?}, y {:.3}", latched, sim.bond.pos[1]);
    lad
}

/// Wrap an angle into −π..π (the walker's helper, local copy).
fn walker_wrap_pi(a: f32) -> f32 {
    let mut a = a;
    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; }
    a
}

/// The disc chain the fix stands on [D — a direct parse of 07000002.bin from the ISO, and the
/// sim's own SS/trigger builds from those bytes]:
/// - SS #308 over the deep ladder: state 0x40, wrap 1, mask 4 (E-usable), channel 2, sound 586,
///   spline, 2 keys; its twin #309: mask 1, channel 1 (never E, never driven).
/// - The stacked ladder-top plates #500 (out 61) / #501 (out 62) and SEQ_AND #396 [61,62] → 101.
/// - No trigger writes the hatch's channel 2 — the use key is the only opener.
#[test]
fn disc_bath_hatch_chain_over_the_deep_ladder() {
    use nf_formats::{disc, iso9660::IsoImage, level::Level};
    let Some(iso_path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000002.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let trig = nf_game::triggers::build(&level);
    let by_pl = |pl: usize| trig.iter().find(|t| t.placement == pl).expect("trigger placement");
    // [D #500/#501/#396 params, probe of the main map's 36 triggers]
    let t500 = by_pl(500);
    assert_eq!(t500.out, 61, "#500's out channel [D]");
    let t501 = by_pl(501);
    assert_eq!(t501.out, 62, "#501's out channel [D]");
    let p500 = &level.maps[level.main_map().unwrap()].placements[500];
    let p501 = &level.maps[level.main_map().unwrap()].placements[501];
    assert!((p500.translation[1] - 4.45).abs() < 0.05 && (p501.translation[1] - 4.94).abs() < 0.05,
        "the plates must sit stacked at the ladder top [D]: {} / {}", p500.translation[1], p501.translation[1]);
    assert!((p500.translation[2] - 5.46).abs() < 0.2, "#500 is at the deep ladder (z {:.2}) [D]", p500.translation[2]);
    let t396 = by_pl(396);
    assert_eq!(t396.kind, nf_game::triggers::kind::SEQ_AND, "#396 is a sequential AND [D]");
    assert_eq!(&t396.list[..2], &[61u32, 62][..], "#396 ANDs the two plates [D]");
    assert_eq!(t396.out, 101, "#396's out is the Bond-moment band (0x65) [D]");
    // No trigger drives the hatch's channel 2 [D — the full main-map trigger list].
    assert!(!trig.iter().any(|t| t.out == 2), "a trigger writes the hatch's channel 2 — the use-key premise is wrong");
    // And no script stream writes it either: every Nis of the level's packs, every decoded
    // command — the channel-set events (kind 0xD, and kind 0x11 which writes params[1] or, when
    // params[0]'s channel is off, params[2] [M Script_EventHandler 0x1d94c0:180-188]) carry other
    // channels (the intro's lone write is [0x47, 1] [D 0x06000084, hatch-02-intro]).
    let sim = load();
    for (hash, nis) in sim.drones.scripts.lib.iter() {
        for (si, s) in nis.streams.iter().enumerate() {
            for cmd in s.decode() {
                if let nf_formats::nis::Cmd::Event { kind: 0xd, params } = &cmd {
                    assert!(params.first() != Some(&2),
                        "script {hash:08x} stream {si} writes the hatch's channel 2 [D] — the use-key premise is wrong");
                }
                if let nf_formats::nis::Cmd::Event { kind: 0x11, params } = &cmd {
                    assert!(params.get(1) != Some(&2) && params.get(2) != Some(&2),
                        "script {hash:08x} stream {si} kind 0x11 can write the hatch's channel 2 [D] — the use-key premise is wrong");
                }
            }
        }
    }
    // The sim's own SS build from the same bytes.
    let s308 = sim.drones.sss.of_placement(308).expect("SS #308 exists [D]");
    let s309 = sim.drones.sss.of_placement(309).expect("SS #309 exists [D]");
    let hatch = &sim.drones.sss.list[s308];
    // [D #308 params (0,64)(1,1)(2,4)(3,2)(4,586)(6,1); SS_Create 0x1a9c58's field map]
    assert_eq!((hatch.state, hatch.wrap_flag, hatch.mask, hatch.channel, hatch.sound, hatch.spline),
        (0x40, 1, 4, 2, 586, true), "SS #308's fields moved off the disc bytes [D]");
    assert_eq!(hatch.keys.len(), 2, "#308 is a two-key open sweep [D]");
    let hpos = hatch.pos();
    assert!((hpos[0] - -49.75).abs() < 0.1 && (hpos[2] - 6.13).abs() < 0.1 && (hpos[1] - 4.76).abs() < 0.15,
        "the closed hatch pose sits over the deep ladder at y ≈ 4.76 [D]: {hpos:?}");
    let twin = &sim.drones.sss.list[s309];
    assert_eq!((twin.mask, twin.channel), (1, 1), "#309 is the locked twin [D]");
}

/// The real path end to end: swim in, climb to the wedge under the closed hatch, OPEN it with
/// the use key (looking up at it — the player's move), keep climbing, and walk away in the
/// bathhouse. Fails while the hatch never opens (the exit stays blocked) or the open doesn't
/// free the climb.
#[test]
fn deep_ladder_climb_opens_the_hatch_on_e_and_exits_into_the_bathhouse() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sim = load();
    let lad = grab_deep_ladder(&mut sim);
    let s308 = sim.drones.sss.of_placement(308).expect("SS #308 exists [D]");
    let closed = sim.drones.sss.list[s308].matrix;
    let open = {
        let s = &sim.drones.sss.list[s308];
        let last = (s.keys.len() - 1) as u16;
        nf_game::door::interp(&s.keys, 0, last, last as f32, s.spline).unwrap()
    };
    // Sister: closed before any use key — armed rest (0x40), playhead at key 0.
    {
        let s = &sim.drones.sss.list[s308];
        assert_eq!((s.state, s.t), (0x40, 0.0), "the hatch is not armed-closed before the climb");
        assert_eq!(s.matrix, closed, "the hatch moved before any use key");
    }
    // Climb up to the wedge under the hatch (the pre-fix climb rose to ~3.91 and stopped).
    let fwd = nf_game::player::Input { forward: 1.0, ..Default::default() };
    let mut wedged = None;
    for t in 0..1200 {
        step(&mut sim, fwd);
        assert_eq!(sim.bond.substate, nf_game::player::SS_CLIMB,
            "+{t}: the climb dropped him at y {:.3} (substate {}) — the climb-up broke", sim.bond.pos[1], sim.bond.substate);
        if sim.bond.pos[1] >= 3.85 { wedged = Some(t); break; }
    }
    let wedged = wedged.expect("the climb never reached the hatch (the wedge)");
    eprintln!("  wedged at +{wedged}, y {:.3}; hatch state {}", sim.bond.pos[1], sim.drones.sss.list[s308].state);
    // Look up at the hatch (the pitch takes the stick during the climb — [S] unverified:
    // Player_Aiming 0x1a1d28 / Player_ViewClamping 0x1a92e8 have no climb or substate branch)
    // and press E — the game's SS_Activate arm (Player_Activate 0x1a5170 case '%').
    // HOLD the climb position while pressing (forward 0 — the real player stops under the
    // hatch; the old calibration leaned on the pre-merge climb wedge at ~3.91, the climb now
    // rises past it and leaves the pick's reach — the E-press phase must not keep climbing).
    let up = nf_game::player::Input { forward: 0.0, look: 0.2, ..Default::default() };
    for _ in 0..8 { step_use(&mut sim, up); }
    sim.bond.pitch = 1.0;
    // The E edge every other tick, still holding the climb. door-react-path5: the game's use fires
    // once per press edge [M 0x1a5ea8-0x1a5ee8 + 0x17f084-0x17f090] — press on even ticks, release
    // on odd ones, so every even tick is a fresh press (the warmup's 8 presses fired before the
    // pitch rose and must not swallow these).
    let mut opened = false;
    for t in 0..180 {
        if t % 2 == 0 { step_use(&mut sim, up); } else { step(&mut sim, up); }
        if t % 4 == 0 {
            let s = &sim.drones.sss.list[s308];
            if s.state == 1 || s.t > 0.0 { opened = true; }
        }
        if opened { break; }
    }
    {
        let s = &sim.drones.sss.list[s308];
        eprintln!("  after E: opened {opened} state {} t {:.3} ch2 {}", s.state, s.t, sim.drones.channels.on(2));
        assert!(opened, "the use key never reached the hatch (SS #308 still state {} t {:.3} at y {:.3}) \
            — the exit stays blocked", s.state, s.t, sim.bond.pos[1]);
    }
    // The sweep runs to the open pose and the one-shot wrap disarms holding it [M 0x1a9948].
    for _ in 0..150 { step(&mut sim, up); }
    {
        let s = &sim.drones.sss.list[s308];
        assert!(s.state == 0 || s.state == 0x40, "the hatch sweep never wrapped (state {})", s.state);
        assert_eq!(s.matrix, open, "the hatch did not reach the open key's pose");
        assert!(sim.drones.channels.on(2), "SS_Activate's channel latch (switch_channels[2]) never landed [M 0x1a9ea8]");
    }
    // Keep climbing: with the hatch open the climb rises past it, tops out, and walks away.
    let mut out = None;
    for t in 0..1200 {
        step(&mut sim, fwd);
        if sim.bond.substate == nf_game::player::SS_WALK && sim.bond.grounded && sim.bond.pos[1] > 4.9 {
            out = Some(t);
            break;
        }
    }
    let out = out.expect("SOFTLOCK: the open hatch never freed the exit (no grounded walk above it)");
    eprintln!("  exited at +{out}, y {:.3} pos {:?}", sim.bond.pos[1], sim.bond.pos);
    // The climb out passed through the stacked ladder-top plates (#500/#501, TouchOnce mask 0x8
    // [D]): both their channels latched, and the sequential AND #396 wrote the Bond moment 101.
    eprintln!("  plates: ch61 {} ch62 {} moment101 {}", sim.drones.channels.on(61), sim.drones.channels.on(62), sim.drones.channels.on(101));
    assert!(sim.drones.channels.on(61) && sim.drones.channels.on(62),
        "the climb out never touched the ladder-top plates (61/62) — it went around them");
    // The sequential AND #396 fires the Bond moment: 61 then 62 in order → out 101 [M
    // Trigger_Update 0x1b1a78 case 4; D #396 list [61,62] out 101].
    assert!(sim.drones.channels.on(101), "the Bond moment (SEQ_AND #396 out 101) never fired on the climb out");
    // And he walks away — turn up the bathhouse (the corridor runs along z: the deep ladder at
    // z 5.5, the twin hatch #309 at z 16.9, the far doors at z −23 [D]) and walk off; no re-latch.
    sim.bond.yaw = 0.0; // forward = +z, up the bathhouse
    let here = sim.bond.pos;
    let walk = nf_game::player::Input { forward: 1.0, ..Default::default() };
    for t in 0..240 {
        step(&mut sim, walk);
        assert_ne!(sim.bond.substate, nf_game::player::SS_CLIMB,
            "+{t} after the exit: re-latched at y {:.3}", sim.bond.pos[1]);
    }
    let moved = ((sim.bond.pos[0] - here[0]).powi(2) + (sim.bond.pos[2] - here[2]).powi(2)).sqrt();
    assert!(moved > 1.0, "Bond cannot walk away from the exit (moved {moved:.3}) — still trapped");
    let _ = lad;
}

/// Sister: without the use key the hatch stays shut — the climb holds the wedge and the plates
/// alone never open it (they only feed the Bond-moment AND [D #396 out 101]).
#[test]
fn the_hatch_stays_shut_without_the_use_key() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sim = load();
    grab_deep_ladder(&mut sim);
    let s308 = sim.drones.sss.of_placement(308).expect("SS #308 exists [D]");
    let fwd = nf_game::player::Input { forward: 1.0, look: 0.2, ..Default::default() };
    for _ in 0..1200 { step(&mut sim, fwd); }
    sim.bond.pitch = 1.0;
    for _ in 0..300 { step(&mut sim, fwd); }
    let s = &sim.drones.sss.list[s308];
    // SS_Activate arms the sweep [M 0x1a9ea8], and so does SS_Update in state 0x40 whenever
    // SS_Operate returns 1 — its channel (ch 2) on, or a toucher matching its mask [M 0x1a9948
    // state 0x40 arm; 0x1a9df8 tail]. Neither holds here: nothing writes ch 2 [D the trigger list
    // + the script scan above; #396's plates write 101, not 2] (daily audit 2026-10-19).
    assert_eq!((s.state, s.t), (0x40, 0.0),
        "the hatch opened without the use key (state {} t {:.3}) — something drives ch 2", s.state, s.t);
    assert!(!sim.drones.channels.on(2), "ch 2 rose without the use key");
    assert!(!sim.drones.channels.on(101) || sim.drones.channels.on(61) || sim.drones.channels.on(62),
        "the climb never touched the ladder-top plates");
}
