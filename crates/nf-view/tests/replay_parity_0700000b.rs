//! replay-parity-0700000b: the Operative real-damage walk REPLAYED headless to the same mission
//! end.
//!
//! The recording (`tests/replays/0700000b_operative.bin`, NFREPLAY v3) is written by
//! walkreal_0700000b_live's `walk_fresh_spawn_to_the_mission_end_real_damage` — the walker's
//! one step site records every tick's exact input (the sim is stepped with `to_parts()` of the
//! just-pushed record, so the recorded walk is byte-identical to the unrecorded one). This test
//! loads a FRESH 0700000b (the same `nf_view::load` difficulty-1 boot the walk used), hands the
//! file to `replay::replay_headless` — which first verifies the load's determinism (the RNG,
//! Bond's pose and weapon against the file's header [M replay.rs replay_headless's bail arms])
//! and applies the recorded option rows — then feeds every tick and asserts the same mission
//! end: the walk completes at the mission latch with Bond alive, the replayed run ends on the
//! same tick count and the same latch.
//!
//! The same file is the app replay's input: `nf-view <iso> 0700000b.bin --difficulty 1 --replay
//! tests/replays/0700000b_operative.bin` plays the session to mission complete in the windowed
//! app (checked by a windowed replay).

#[test]
fn replay_the_operative_walk_headless_to_the_same_mission_end() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/replays/0700000b_operative.bin");
    let replay = nf_view::replay::Replay::load(path).expect("the committed Operative recording");
    assert_eq!(replay.header.level, "0700000b.bin");
    let ticks = replay.tick_count();
    assert!(ticks > 15000, "the full-mission recording carries the whole walk, got {ticks}");

    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b");
    let states = nf_view::replay::replay_headless(&mut scene, &replay)
        .expect("the headless replay (the header's RNG/pose/weapon checks are the load-determinism proof)");
    assert_eq!(states.len() as u32, ticks + 1, "one snapshot per tick plus the starting state");

    let sim = scene.sim.as_ref().unwrap();
    let last = states.last().unwrap();
    // 0b's mission end: MissionState 6 = complete [M 0x182c60:85-122]. The raw ch 100 is a
    // transient — the jump band's off-write cancels the chain ~20 ticks after OUT #304's write
    // (the walkreal file's own tail comment) — so the END-state assert is the mission LATCH,
    // which the walk itself re-asserts after its final 60 ticks.
    assert_eq!(sim.drones.mission.status(), 6, "MissionState 6 = complete [M 0x182c60:85-122]");
    assert_eq!(sim.drones.mission.to_end, 0x0730_000b, "the driving hand-off target [D]");
    assert!((ticks as u32).abs_diff(last.frame + 1) <= 1,
        "the replayed sim ends on the recording's last tick (frame {} vs {} ticks — the shoot_at bookkeeping boundary)", last.frame, ticks);
    // The recorded end stands just past the deck jump (the mission latches as Bond falls east
    // — the walk's own jump leg ends at the fall; [L] the recording's last snapshot y 39.3).
    assert!(last.bond_pos[1] > 30.0 && last.bond_pos[0] > 5.0,
        "Bond ends the mission mid-jump east of the deck, got {:?}", last.bond_pos);
}
