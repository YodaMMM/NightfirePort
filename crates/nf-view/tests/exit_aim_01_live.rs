//! exit-aim-01: the 01 exit's real use target, proven on the real path (`nf_view::load` + the
//! Walker's real inputs only — no state pokes). The disc's 0xE8 #18 "Load Point" has NO collision
//! — its object record carries a gfx box but `collision None` [D placements dump] — so the game's
//! E pick (a radius-1.0 sphere at head+look vs collision geometry [M 0x1a5170:100-108]) can never
//! reach the 0xE8, and the old stand-in ("aim at the exit trigger's bbox") had nowhere to stand
//! and skipped the E-aim. The exit's real use target is the LoadPointActivator plate #180 (Touch
//! out 0x10 == the 0xE8's gate, 44 collision tris [D]): the player's part is STANDING on it — E
//! on the plate is refused by the game's `mask & 4` gate [M 0x1b2318:23-25; #180 mask 0x18 [D]] —
//! and the exit fires by PRESENCE once BOTH bodies are on the plate [M 0x1b1a78:162-166 case 10 +
//! Trigger_Hit; walkreal_07000001 walks the full escort that brings both].

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_exit_resolves_to_the_gate_plate_and_e_is_refused_there() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // The intro cut-scene (SP #277, script 0x06000087) writes ch 0xB at ~tick 2119 [D decode];
    // wait it out, then the script tail (event 6) re-places Bond (walkreal_07000001's opening).
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    assert!(enabled_at.is_some(), "the intro must write ch 0xB (the contact's enable)");
    w.wait_enabled(6000, "the intro tail (event 6 re-places Bond)");
    w.wait(300);

    // The resolution — fail-first against the old stand-in: the old E-aim targeted the 0xE8's own
    // box, which is unbounded (no mesh), so it skipped. The real target is the gate plate.
    let plate = {
        let sim = w.scene.sim.as_ref().unwrap();
        let exit18 = sim.drones.triggers.iter()
            .find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL && t.next_level == 0x0700_0002)
            .expect("the 01 exit #18 [D placement]");
        assert!(exit18.tris.is_empty(), "the disc's 0xE8 has no collision mesh [D placements dump]");
        assert!(exit18.lo.iter().any(|v| !v.is_finite()),
            "its bbox is unbounded — the old E-aim had nowhere to stand [D]");
        let (target, e_fires) = nf_game::triggers::exit_use_target(&sim.drones.triggers)
            .expect("the gate plate resolves (exit-aim-01)");
        assert_eq!(target.kind, walker::trigger_kind::TOUCH, "the real use target is the Touch plate, not the mesh-less 0xE8");
        assert_eq!(target.out as u32, exit18.gate as u32, "the plate feeds the 0xE8's gate [D #180 out 0x10 == gate ch 0x10]");
        assert!(target.lo.iter().all(|v| v.is_finite()), "the plate's bounds are real [D #180: 44 collision tris]");
        assert!(!e_fires, "E cannot fire the plate [M 0x1b2318:23-25 mask & 4]");
        // The E-path's own request surface, on the disc's real mask: refused before any write.
        let (sets, next) = target.activate_request(&sim.drones.channels);
        assert!(sets.is_empty() && next.is_none(), "activate_request refuses on mask 0x18 [M 0x1b2318:23-25; D #180 p1]");
        walker::trigger_centre(&sim, walker::trigger_kind::TOUCH, 0x10).expect("the AND-touch plate centre")
    };
    // Walk there through the real input path (the proven descent anchors from walkreal_07000001 —
    // the direct nav from the balcony side climbs over the top floor and never enters the stairwell).
    w.walk_nav_to([-30.8, 6.4, 13.1], "the stairwell head", 1.2);
    w.walk_nav_to(plate, "the LoadPointActivator plate", 1.0);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < -32.4 && p[1] < 4.0 && (12.2..18.7).contains(&p[2]),
            "Bond walked onto the plate area through real inputs, got {p:?}");
    }
    // Premise: the contact is not here — the armed guards near his spawn pin him (the game's own
    // AllyLeadHide [M 0x15ec08]), so the AND cannot fire yet [D #180 filter skin 0x4F].
    let ci = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_004f).expect("the contact (skin 0x4F) [D placement]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = walker::dxz(sim.drones.list[ci].pos, sim.bond.pos);
        assert!(d > 6.0, "sister premise: the contact is not on the plate with Bond (dxz {d:.1})");
    }
    // Hold E like a player standing at the exit: NOTHING fires — the plate refuses E [M
    // 0x1b2318:23-25] and one body cannot satisfy the AND [M Trigger_Hit; walkreal's sister pins
    // the same rule without the use key].
    for _ in 0..240 {
        w.tick(Default::default(), true); // the use key held
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x10),
            "E held on the plate must not raise the gate channel [M 0x1b2318:23-25 mask & 4]");
        assert_eq!(sim.drones.level_exit, None,
            "and no exit: one body on the plate is not the AND [M Trigger_Hit 0x1b1fc8]");
        let d = walker::dxz(sim.drones.list[ci].pos, sim.bond.pos);
        eprintln!("[01] after the 240-tick E window: contact dxz {d:.1}, ch 0x10 off, no exit — the E-refusal and the both-bodies rule hold");
    }
}
