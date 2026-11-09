//! level-07-guest-hostile: 07000007's two voice-0x10 party guests. The census: voice 0x10 =
//! skin 0x05000026, exactly two drones — #22 (pl#426, at (80.71, −50.47), 0.05 from kind-8 path
//! 3's node 7) and #55 (pl#501, at (88.25, −55.63), 0.03 from path 7's node 0), both team 3 with
//! props 0x33/0x1D/0x3C [L guest census probe]. Both boot in WaitSwitch: #22's switch channel is
//! 23 (TouchOnce #415) with the (21, tag 100) flush pair, #55's is 21 (Drake's reveal) with
//! (0x48, tag 100) [D placements]. Prop 0x1D walkers run CivilianMission 0x16, and it is
//! `NDrone2_DSTATE_CivilianMission` 0x15d860 case 3 that carries the voice-0x10 flip (the same
//! shape as CivilianPatrol 0x15d610:53-55, which the party's non-walkers never reach):
//! `NearArmedPerson` (the radius arg dead [M 0x1787c8]) → `+0x4F8 |= 0x8000000`; prop 0x33's
//! once-shout (`+0x4FC |= 1`, `DroneAlertToObject(20, 20, 0.7625, msg 0x11, Bond)`,
//! `+0x4F8 |= 0x200000`); voice 0x10 → `+0x44 = 1` (the team byte) and SetState(0x56 ATTACK);
//! every other voice → SetState(0x12 CivilianScared) [M 0x15d860:69-90]. The port's
//! CIVILIAN_MISSION UPDATE arm is that arm line for line (drone.rs).
//!
//! THE STORY ANSWER the walk pins: the weapon lock [M 0x1a24c8:242-275] holds Bond on the fists
//! until ch 0x48 — and ch 0x48 is exactly the channel that removes guest 55 (and drone 50),
//! while ch 21 (the reveal, forced before the library door) removes guest 22. The flush
//! (`NDrone2_PreDroneControl` 0x148db0:53-99) has NO state gate — it takes the started walkers
//! too — and these drones' alternate state +0x5A4 is DeleteMe 0xb4 [L probe: alt_state 180 on
//! 22/55/50/9/28/29; the disc's mode-0x15 row]. So through the level's own data the armed sight
//! and the voice-0x10 guests never coexist: guest 22 patrols path 3 only between ch 23 and ch
//! 21, guest 55 patrols path 7 only between ch 21 and ch 0x48, Bond is unarmed through BOTH
//! windows, and the flip arm is story-dead. The walk pins the real behaviour end to end: the
//! wake plates, the patrol lanes (route paths 3/7), the unarmed crossing of guest 22's lane at
//! sight range without a flip (the drawn-weapon gate [M 0x1787c8:38-42 via 0x17a4b8 +
//! 0x178768]), the plate-timed departures, and the armed sweep of both lanes finding no hostile
//! guest. The flip arm itself — team 1 + Attack + the once-shout, the voice-0x12 sister to
//! CivilianScared, the unarmed sister to nothing — is pinned at the unit level by nf-game's
//! `civilian_mission_flips_voice_0x10_to_the_hostile_team_on_the_armed_sight`.
//!
//! No teleports after the spawn; no channel/state writes anywhere.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::state;
use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn guest(w: &Walker, i: usize) -> (u8, u8, f32, bool) {
    let g = &w.scene.sim.as_ref().unwrap().drones.list[i];
    (g.state, g.team, walker::dxz(g.pos, w.scene.sim.as_ref().unwrap().bond.pos), g.seen)
}

fn crate_delete_me() -> u8 { nf_game::kiko::DELETE_ME }

/// The walk pins the story's real behaviour end to end (see the header): the wake plates, the
/// patrol lanes, the unarmed crossing without a flip, the plate-timed departures, the armed sweep
/// finding no hostile guest.
#[test]
fn the_voice_0x10_guests_patrol_their_lanes_between_their_plates_and_never_meet_armed_bond() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (s22, t22, _, _) = guest(&w, 22);
        let (s55, t55, _, _) = guest(&w, 55);
        assert_eq!((sim.drones.list[22].voice_type, sim.drones.list[55].voice_type), (0x10, 0x10),
            "the census's two voice-0x10 guests [L guest census probe]");
        assert_eq!((t22, t55), (3, 3), "both guests boot at team 3 [L guest census probe]");
        assert_eq!((s22, s55), (state::WAIT_SWITCH, state::WAIT_SWITCH), "both boot in WaitSwitch [D placements]");
    }

    // ── the unarmed walk to the hall (walkreal_07000007's route): the group plates fire on the way
    for (name, at) in [
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall west mouth", [76.6, 43.6, -50.7]),
        ("the hall floor", [83.1, 43.6, -50.5]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        let ch6 = w.scene.sim.as_ref().unwrap().drones.channels.on(6);
        let ch23 = w.scene.sim.as_ref().unwrap().drones.channels.on(23);
        eprintln!("[07 guest] after {name}: ch6 {ch6} ch23 {ch23}");
    }
    {
        // The ch-6 and ch-23 plates fired on the walked route: #22's gate is up, so he patrols.
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(6), "the ch-6 group plate #476 fired on the walked route [D census §4]");
        assert!(sim.drones.channels.on(23), "the ch-23 plate #415 fired on the walked route [D census §4]");
        let (s22, t22, d22, _) = guest(&w, 22);
        assert_eq!(t22, 3, "guest 22 stays team 3");
        assert!(matches!(s22, state::CIVILIAN_MISSION | state::WAIT_SWITCH),
            "guest 22 patrols (CivilianMission) or still waits, got {s22:#04x}");
        let path22 = sim.drones.list[22].route.as_ref().map(|r| r.path);
        assert_eq!(path22, Some(3), "guest 22 walks kind-8 path 3 (his spawn sits 0.05 from node 7 [L probe])");
        eprintln!("[07 guest] at the hall floor unarmed: guest22 state {s22:#04x} path {path22:?} dist {d22:.1}");
    }

    // ── Drake's reveal (ch 21): guest 55's gate — the walkreal route's own helper. ch 21 is also
    //    guest 22's flush channel: his (21, tag 100) pair is consumed with alt_state 0xb4
    //    (DeleteMe) — he leaves here, mid-patrol [M 0x148db0:53-99; L probe].
    walker::drakes_reveal_07(&mut w);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(21), "the reveal's ch 21 [D placements]");
        let (s55, t55, _, _) = guest(&w, 55);
        assert_eq!(t55, 3, "guest 55 stays team 3 through the reveal");
        assert!(matches!(s55, state::CIVILIAN_MISSION | state::WAIT_SWITCH),
            "guest 55 patrols (CivilianMission) after ch 21, got {s55:#04x}");
        let path55 = sim.drones.list[55].route.as_ref().map(|r| r.path);
        assert_eq!(path55, Some(7), "guest 55 walks kind-8 path 7 (his spawn sits 0.03 from node 0 [L probe])");
        let g22 = &sim.drones.list[22];
        assert!(!g22.present || g22.state == crate_delete_me(),
            "guest 22 left at his flush channel ch 21 (the alt state is DeleteMe 0xb4 [M 0x148db0:65-99; L probe]), got present {} state {:#04x}",
            g22.present, g22.state);
    }

    // ── the hall walk-in #412 (ch 0x48): the lock lifts; the scene hands Bond back
    w.walk_nav_to([93.3, 43.6, -50.4], "the hall east", 1.5);
    w.walk_to(Wp { at: [94.6, 43.6, -47.4], act: Act::Use { use_ticks: 240 }, arrive: 0.7 }, "E the hall door");
    {
        let target = [94.3f32, 43.6, -45.9];
        let mut last = [0.0f32, 0.0];
        let mut still = 0u32;
        for t in 0..1200u32 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let door_closed = sim.drones.doors.list.iter()
                .filter(|d| walker::dxz(d.pos, p) < 3.0)
                .any(|d| d.state & 0xc == 0);
            let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
                still += 1; door_closed && still % 30 == 0
            } else { still = 0; last = [p[0], p[2]]; false };
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, use_key);
            let _ = t;
        }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x48), "the hall walk-in plate #412 fired ch 0x48 [D placements]");
    w.wait_enabled(1200, "the control-taking scene");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0073) && p.state >= 3),
            "the control-taking cut-scene ran through the real trigger [M levels.md §5.1]");
        // ch 0x48 is guest 55's and drone 50's flush channel: both leave here, at the same plate
        // that lifts the weapon lock [M 0x148db0:53-99; L probe].
        for &gi in &[55usize, 50] {
            let g = &sim.drones.list[gi];
            assert!(!g.present || g.state == crate_delete_me(),
                "drone {gi} left at ch 0x48 (the alt state is DeleteMe 0xb4 [M 0x148db0:65-99; L probe]), got present {} state {:#04x}",
                g.present, g.state);
        }
    }

    // ── the unarmed half pin, restated where it matters: through EVERY step above, both
    //    voice-0x10 guests stayed team 3 and out of ATTACK/Scared (the per-step asserts).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // By the armed phase BOTH voice-0x10 guests are taken by their flush plates — guest 22
        // at ch 21 (the reveal block), guest 55 and drone 50 at ch 0x48 (the scene block). That
        // IS the unarmed story: no flip, no scare, the plates just collect them.
        for &gi in &[22usize, 55] {
            let g = &sim.drones.list[gi];
            assert_ne!(g.team, 1, "guest {gi} never flipped while Bond was unarmed");
            assert!(!g.present || g.state == crate_delete_me(),
                "guest {gi} is gone (or still deleting) by the armed phase, got present {} state {:#04x}",
                g.present, g.state);
        }
    }

    // ── the armed phase: the player cycles to a gun (ch 0x48 lifted the lock; the real
    //    weapon_cycle input) and walks BOTH patrol lanes. The story's answer: the guests are
    //    already gone. The flush (`NDrone2_PreDroneControl` 0x148db0:53-99 — no state gate: it
    //    runs for the started walkers too) consumed their (channel, tag) pairs at ch 21 (guest
    //    22) and ch 0x48 (guest 55/50), and their alternate state +0x5A4 is DeleteMe 0xb4
    //    [L probe: alt_state 180 for drones 22/55/28/29/9/50; the disc's mode-0x15 row], so each
    //    leaves at his plate — the same flush that takes the library door guards off at ch 21
    //    (drakes_reveal_07's own assert). The weapon lock [M 0x1a24c8:242-275] holds Bond on the
    //    fists until ch 0x48 — the very channel that removes guest 55 — so through the level's
    //    own data the armed sight and the voice-0x10 guests never coexist: the flip arm is
    //    story-dead, and what the walk pins is the windows plus the unarmed no-flip.
    assert!(w.switch_to_armed("the party hall"), "Bond owns a gun with ammo (RamLoad) [M 0x1a7ef8]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let wpn = sim.gun.weapon;
        assert!((2..=0x47).contains(&wpn), "Bond holds a drawn gun ({wpn:#x}) — the armed gate [M 0x1787c8:38-42]");
        eprintln!("[07 guest] Bond armed: weapon {wpn:#x} at {:?}", sim.bond.pos);
        for &gi in &[22usize, 55, 50] {
            let g = &sim.drones.list[gi];
            assert!(!g.present || g.team == 3,
                "guest {gi}: gone at his plate, or never hostile through the story (team {})", g.team);
            assert_ne!(g.team, 1, "guest {gi} never flipped through the story walk");
        }
    }
    // Walk guest 22's lane (path 3: (79.75,−50.45) → the north row west) and guest 55's lane
    // (path 7: (88.26,−55.60) → the north row east) with the drawn gun — the lanes the two
    // voice-0x10 guests walked during their windows.
    for (name, at) in [
        ("path 3's hall node", [80.2f32, 43.6, -50.44]),
        ("path 3's north row west", [74.4, 43.6, -45.0]),
        ("the hall floor again", [83.1, 43.6, -50.5]),
        ("path 7's hall node", [88.3, 43.6, -55.6]),
        ("path 7's north row east", [101.7, 43.6, -56.0]),
    ] {
        w.walk_nav_to(at, name, 1.5);
        let sim = w.scene.sim.as_ref().unwrap();
        let hostile = sim.drones.list.iter().enumerate()
            .filter(|(_, g)| g.present && g.voice_type == 0x10 && g.team == 1)
            .count();
        assert_eq!(hostile, 0, "no voice-0x10 guest is hostile anywhere on the armed walk [M the flush 0x148db0 + the ch 0x48 lock]");
    }

    // ── the unarmed no-flip, restated: through EVERY step above, both voice-0x10 guests stayed
    //    team 3 while Bond crossed their lanes — first with the fists (the armed gate
    //    [M 0x1787c8:38-42 via 0x17a4b8 + 0x178768]), then with the gun (they were gone at their
    //    plates). The flip arm itself (team 1 + Attack + the once-shout, the voice-0x12 sister
    //    to CivilianScared) is pinned at the unit level by
    //    nf-game's `civilian_mission_flips_voice_0x10_to_the_hostile_team_on_the_armed_sight`.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for &gi in &[22usize, 55] {
            let g = &sim.drones.list[gi];
            assert_ne!(g.team, 1, "guest {gi} never flipped through the real walk");
        }
    }
}
