//! level-07-party-door-lock: the door back to the main party room locks at the control-taking
//! scene, through the player's own input.
//!
//! THE DOOR [D placements census]: #251 "Door_Wooden_8" (85.55, 43.00, −35.12) is the only door on
//! 07000007 wired to a story channel — starts-open (p8|p9 = 1|36 → flags 0x25: 0x01 spline |
//! 0x04 starts-open | 0x20 keep-open), open channel p2 = 130 (0x82), lock channel p3 = 200, hit
//! sound p7 = 320. [M Door_Create 0x136100:96-99] a starts-open door is created OPEN (state 4)
//! and forces its open channel on — ch 130 reads 1 from the first tick.
//!
//! THE STORY POINT [D the 07000007 script decode]: the hall walk-in TouchOnce #412 (94.27, 43.61,
//! −45.92, out ch 0x48) fires SP #420 — script 0x06000073 (the control-taking cut-scene, streamed
//! from pack 07800007.bin [M lib.rs's loadable-pack pass; SP_LoadScript 0x1dbc08]). Its stream 8
//! carries the level's ONLY script channel write: event 0x0D [0x82, 0] — ch 130 ← 0 [D stream
//! decode; M Script_EventHandler 0x1d94c0 case 0xd `switch_channels[p0] = p1`].
//!
//! THE LOCK [M Door_Update 0x1363c8]: with the open channel off and the door open, the channel arm
//! activates the door (close) once no player stands in its blocking zone; #251 is then CLOSED with
//! lock channel 200 — and nothing on the level writes ch 200 [D: the script lib's only channel
//! event is [130, 0]; no trigger's out is 200 (the census); no writer family carries 200 (docs/
//! systems/switch-channel-writers.md)]. NOTE (level-07-party-door-lock-2): that no-writer claim is
//! a [D] scan result, NOT an [M] proof, so it is NOT asserted here — the test logs ch 200's state
//! as an observation and pins only the [M]-backed behaviour (the E refusal, the free door still
//! opening). [M Door_Activate 0x135be0 LAB_00135cf8] refuses every later E press on the closed
//! door while ch 200 is off, with the hit sound (320 [D p7]). The locked door gives no
//! text hint at all [M Door_DoMsg 0x135f18:20-24 — the lock gate; the "locked" feedback is the
//! hit sound only], and the "Door Unlocked" message (0x2000003) belongs to ch 200's rising edge
//! [M 0x1363c8:88-107], which never fires on this level.
//!
//! The story point is unavoidable on the real route: the plate #412's box (x 93.31..95.23,
//! z −46.43..−45.41 [D trigger bounds]) IS the hall→north passage — the probe's plate-free skirt
//! at x 92.4 walks off the hall's north ledge and falls (the port's own collision). So the door's
//! pre-story state is pinned where the player sees it — standing OPEN through the whole walk up to
//! the plate — and the passage itself is never crossed before the scene.
//!
//! The other ch-200 doors (#252/#253/#254/#255/#256/#304, and the slide #342 — no hit sound,
//! p7 = 0xffff) are locked from boot the same way and stay locked: level boundary doors, the
//! U-044 shape. The free doors (#242-#248, p3 = 0) cannot lock [M Door_Activate's lock arm needs
//! +0x68 ≠ 0] — the sister pins one opening after the story point.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn door_i(w: &Walker, placement: usize) -> usize {
    w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.placement == placement).unwrap_or_else(|| panic!("door #{placement} created"))
}

/// The real walk: the spawn → through the party (the walkreal anchors) → E the hall door #244 →
/// the hall walk-in plate #412 (the story point) → the control-taking scene (0x06000073) hands
/// Bond back in the north fireplace hall. No teleports. The party-corridor door #251 stands OPEN
/// from the boot to the plate — asserted at both ends.
fn walk_to_the_story_point(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 33.4).abs() < 2.0 && (p[2] - -70.6).abs() < 2.0, "the party start #0 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[door_i(w, 251)];
        assert_eq!(d.ch_open, 130, "door #251's open channel is 130 [D p2]");
        assert_eq!(d.ch_lock, 200, "door #251's lock channel is 200 [D p3]");
        assert_eq!(d.state & 0xff, 4, "door #251 is created OPEN (starts-open flags 0x25 [D]; M 0x136100:96-99)");
        assert!(sim.drones.channels.on(130), "the starts-open door forced its open channel on [M 0x136100 tail]");
        // [D] observation, not an assertion (level-07-party-door-lock-2): ch 200 reading 0 at boot
        // follows from the [D] no-writer scan (docs/systems/switch-channel-writers.md maps every
        // [M] writer family; none of this level's records resolves to 200), not from an [M] proof
        // — logged, kept out of pass/fail.
        eprintln!("[07 lock] observation: ch 200 {} at boot (the [D] no-writer scan says off; not asserted)",
            sim.drones.channels.on(200) as u8);
    }

    // The party's own route to the hall (the walkreal anchors).
    for (name, at) in [
        // drone-bump-collision-3 re-derivation: the old anchors crossed the hall's south band
    // (z ≈ −50.4) — the party crowd's roam band (the kind-8 patrol nodes' z −48.6..−57.7
    // [D the nav dump]; the hall bots boot at z −48.6/−50.4/−50.5 [L probe_07_who]). The
    // crowd is ARMED: enabled bots with running control (the party states write no +0x4F8
    // collision bits [M PartyGirlInit 0x1728c0 / PartyGirl 0x160140]) fill +0x154 on their
    // own control tick [M PreDroneControl 0x148db0:35-48], so DoCollision's near arm fires
    // inside 2.0 [M 0x149ec8:30-33, disasm] and NDrone2_Collision writes kind +0x464 = 0x800
    // [M 0x140e90:95-101] → the Jointy push runs at capsule contact [M 0x1ec180:229-281;
    // drone_bump_live 2/2]. The game shoves Bond off the crowd line (the walker's own
    // 6000-tick wedge at (79.19, −51.21) vs the bot at (79.57, −50.34) — 0.99 apart, just
    // outside the 0.95 contact band the push resolves to). The anchors move to the hall's
    // north lane (z ≈ −46.6, clear of the patrol nodes and of the plate box x 93.31..95.23
    // z −46.43..−45.41 [D trigger bounds]), the line the probe's post-plate skirt already
    // walks. The main agent's live check (does the game's crowd stop Bond mid-floor?) is
    // listed in docs/reports/drone-bump-collision-3.md.
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall north lane", [80.0, 43.6, -46.6]),
        ("the hall north lane east", [90.5, 43.6, -46.6]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    {
        // The story point has not fired yet: the party-corridor door still stands open.
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x48), "the walk up to the hall fired no story channel");
        assert!(sim.drones.channels.on(130), "ch 130 still on before the story point");
        assert_eq!(sim.drones.doors.list[door_i(w, 251)].state & 0xff, 4, "#251 stands open until the story point");
    }
    // E the hall door #244 (free, p3 = 0 [D]) — the walkreal anchor.
    w.walk_to(Wp { at: [94.6, 43.6, -47.4], act: Act::Use { use_ticks: 240 }, arrive: 0.7 }, "E the hall door");
    // Cross the hall walk-in plate #412 (the walkreal input loop: aim, walk, pulse E on a wedged door).
    {
        let target = [94.3f32, 43.6, -45.9];
        let mut still = 0u32;
        let mut last = [0.0f32, 0.0];
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
            if t % 300 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[07 lock t{t}] bond {:?} 0x48 {}", sim.bond.pos, sim.drones.channels.on(0x48) as u8);
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48), "the hall walk-in plate #412 fired ch 0x48 [D placements]");
    }
    // The scene holds Bond (the game's own disable); the walker carries the game's skip action.
    w.wait_enabled(1200, "the control-taking scene");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0073) && p.state >= 3),
            "the control-taking cut-scene (0x06000073) ran through the real trigger [M levels.md §5.1]");
        let p = sim.bond.pos;
        assert!((p[0] - 91.52).abs() < 1.0 && (p[2] + 31.09).abs() < 1.0,
            "Bond handed back in the north fireplace hall [D fix_cutscene_actors_live], got {p:?}");
    }
}

/// The story point: the scene's stream-8 event drops ch 130, #251 closes, and the use key is
/// refused for good (ch 200 has no writer on this level).
#[test]
fn the_party_corridor_door_locks_at_the_control_scene() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    walk_to_the_story_point(&mut w);
    // The close takes ticks: Door_SetState mirrors the OLD state's motion bits to the open
    // channel (leaving OPEN writes 1 [M 0x135618:26]) until the leaf arrives at CLOSED. Wait out
    // the swing, then pin the settled state.
    let mut settled = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.doors.list[door_i(&w, 251)].state == 2 { settled = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(settled, "#251 arrived at CLOSED after the story point");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(130),
            "the scene's stream-8 event 0x0D [130, 0] fired through the real cut-scene and the close's mirror settled [D stream decode]");
        let d = &sim.drones.doors.list[door_i(&w, 251)];
        assert!(d.state & 0xc == 0,
            "the open-channel fall closed #251 (state {:#04x}) [M Door_Update 0x1363c8's channel arm]", d.state);
    }
    // From the hand-back spot, walk to #251's north side (the fireplace hall's south-west) and
    // press E — the player's own use on the door back to the party.
    for (name, at) in [
        ("the fireplace hall's south-west", [87.5f32, 43.6, -33.5]),
        ("against the party-corridor door", [85.5, 43.6, -33.8]),
    ] {
        w.walk_nav_to(at, name, 1.2);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(walker::dxz(sim.bond.pos, [85.55, 43.0, -35.12]) < 3.5,
            "Bond stands at the closed #251, got {:?}", sim.bond.pos);
    }
    for k in 0..3 {
        for t in 0..30 { w.tick(Input::default(), t == 0); }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[07 lock] E press {}: door #251 state {:#04x}", k + 1,
            sim.drones.doors.list[door_i(&w, 251)].state);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[door_i(&w, 251)];
        assert!(d.state & 0xc == 0, "the use key is refused on the locked party door (state {:#04x}) [M Door_Activate 0x135be0 LAB_00135cf8 — ch 200 off, ch_lock 200]", d.state);
        // [D] observation, not an assertion (level-07-party-door-lock-2): "ch 200 never written on
        // this level" is the [D] scan's result (docs/systems/switch-channel-writers.md), not an
        // [M] proof — logged, kept out of pass/fail.
        eprintln!("[07 lock] observation: ch 200 {} after the E presses (the [D] no-writer scan says it stays off; not asserted)",
            sim.drones.channels.on(200) as u8);
        assert_eq!(d.ch_lock, 200, "the lock channel field stays armed (the one-shot unlock edge never fired) [M 0x1363c8:106 clears it only on the edge]");
    }
    eprintln!("[07 lock] the party door locked at tick {}", w.total_ticks);
}

/// Sister: the story point is #251's own channel write — the free doors cannot lock [M the lock
/// arm needs +0x68 ≠ 0] and still answer the use key after the scene. The hall door #244 (free,
/// p3 = 0 [D]) stands on the way back: a locked door IGNORES every press (the state freezes —
/// Door_Activate returns before any state change), a free one toggles.
#[test]
fn sister_the_free_doors_still_open_after_the_lockout() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    walk_to_the_story_point(&mut w);
    // The same settle (the close's mirror bounces ch 130 while the leaf swings).
    let mut settled = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.doors.list[door_i(&w, 251)].state == 2 { settled = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(settled, "the sister's walk saw #251 arrive CLOSED after the story point");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(130), "the story point's channel write reached the sister's walk too");
        assert!(sim.drones.channels.on(0x48), "the hall walk-in fired (the sister shares the route)");
    }
    // Back into the hall (the walkreal's post-scene anchors, re-routed north of the armed
    // crowd — the same drone-bump-collision-3 re-derivation as the walk up) to the free hall
    // door #244: the old "the hall centre" leg dove into the crowd band (z −50.4) the armed
    // party bots shove Bond off of [M the arming chain, see the walk up].
    for (name, at) in [
        ("the hall south node", [95.2f32, 43.6, -44.6]),
        ("the hall east lane", [94.0, 43.6, -46.2]),
        ("against the free hall door", [94.5, 43.6, -47.6]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    {
        let d = &w.scene.sim.as_ref().unwrap().drones.doors.list[door_i(&w, 244)];
        assert_eq!(d.ch_lock, 0, "the hall door #244 carries no lock channel [D p3=0]");
    }
    // Press E until the leaf MOVES (a locked door's state never changes across presses).
    let start = w.scene.sim.as_ref().unwrap().drones.doors.list[door_i(&w, 244)].state;
    let mut moved = false;
    for k in 0..6 {
        for t in 0..40 {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (94.89f32 - p[0]).atan2(-47.25 - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: 0.0, ..Default::default() };
            w.tick(input, t == 0 && k < 3);
            if w.scene.sim.as_ref().unwrap().drones.doors.list[door_i(&w, 244)].state != start { moved = true; break; }
        }
        if moved { break; }
    }
    assert!(moved, "the free hall door #244 still answers the use key after the story point (state {start:#04x} frozen would mean a lock) [M Door_Activate — no lock field]");
    eprintln!("[07 lock] sister: #244 moved {start:#04x} → {:#04x} at tick {}",
        w.scene.sim.as_ref().unwrap().drones.doors.list[door_i(&w, 244)].state, w.total_ticks);
}
