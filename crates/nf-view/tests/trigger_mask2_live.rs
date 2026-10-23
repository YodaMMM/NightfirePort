//! trigger-mask2-bullets: the real-path probe for the mask & 2 (class-5 BULLET contact) arm on
//! its only disc-reachable uses. The census (trigger_mask2_census_live) shows the reachable set is
//! the TouchOnce/Touch family (the only types whose `Trigger_Update`/`Trigger_Calc` arms consume
//! the `Trigger_Hit` result [M 0x1b1a78:71-93 — the mux cases ignore it]) with real collision
//! volumes: **07000006 #720 (out 0x2e), #721 (out 0x2d), #783 (out 0xc)** — TouchOnce, mask 2,
//! EMPTY filter lists, 12/12/2 glist triangles. `Trigger_Hit` 0x1b1fc8:130-135: an empty filter
//! list returns 1 on any class-5 contact — the port's live grenade (a real `Grenade` ticked
//! through `Drones::tick`'s grenade pass, its per-tick swept segment feeding
//! `bullet_contacts_now` → the next tick's `Trigger_Calc`) must fire the plate's out channel.
//!
//! The filtered mask-2 shapes (a filter entry naming a weapon id) exist on the disc only on mux
//! kinds (3/4/5/12), whose update never reads the hit — pinned instead by the triggers.rs unit
//! tests (the satchel/frag filter semantics).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn a_live_grenade_through_the_07000006_plate_fires_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    let (centre, out_ch) = {
        let sim = scene.sim.as_ref().unwrap();
        // The unfiltered TouchOnce plates with a real volume (the census set).
        let t = sim.drones.triggers.iter()
            .filter(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.mask == 2
                && t.list.iter().all(|&f| f == 0) && !t.tris.is_empty())
            .max_by_key(|t| t.tris.len())
            .expect("07000006 carries the unfiltered TouchOnce mask-2 plates #720/#721");
        (t.centre(), t.out)
    };
    eprintln!("[plate] out {out_ch:#x} centre {centre:?}");
    // Park Bond far from the plate (no player contact; the plate has no 0x18 bits anyway).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [centre[0] + 40.0, centre[1] + 0.5, centre[2] + 40.0];
        sim.step_bond(Default::default(), false, false, false);
        assert!(!sim.drones.channels.on(out_ch as u32), "the plate starts unspent");
        // A live satchel (0x3b) dropped from above the plate's centre: its per-tick segments
        // sweep down through the volume. State 0, straight down, no stick phase.
        sim.drones.grenades.push(nf_game::grenade::Grenade {
            pos: [centre[0], centre[1] + 1.5, centre[2]], velocity: [0.0, -0.5, 0.0], speed: 0.5,
            travelled: 0.0, range: 300.0, damage: 0.0, radius: 0.0, mesh: 0, owner: usize::MAX,
            state: 0, bounces: 0, smoke_popped: false, resting: false, stuck: false,
            seek_target: None, phase: None, weapon_type: 0x3b,
        });
    }
    let mut fired_at = None;
    for t in 1..120u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.channels.on(out_ch as u32) { fired_at = Some(t); break; }
    }
    let fired_at = fired_at.expect("the grenade's flight through the plate fired its out channel");
    eprintln!("[fired] out {out_ch:#x} on tick {fired_at} (the one-tick-late Trigger_Hit phase)");
    // Sister: a different plate's channel must not fire from this grenade (the flight crossed
    // only one volume). The other unfiltered plate (0x2d/0xc) stays off.
    let sim = scene.sim.as_ref().unwrap();
    let others: Vec<u8> = sim.drones.triggers.iter()
        .filter(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.mask == 2 && t.out != out_ch)
        .map(|t| t.out).collect();
    for o in others {
        assert!(!sim.drones.channels.on(o as u32), "plate out {o:#x} must stay off");
    }
}

#[test]
fn a_grenade_nowhere_near_the_plates_fires_nothing() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000006.bin", false).expect("load 07000006");
    let outs: Vec<u8> = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.triggers.iter()
            .filter(|t| t.kind == nf_game::triggers::kind::TOUCH_ONCE && t.mask == 2)
            .map(|t| t.out).collect()
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        // The same satchel, dropped 40 units away from every plate (over open floor).
        sim.drones.grenades.push(nf_game::grenade::Grenade {
            pos: [1.0, 3.0, 1.0], velocity: [0.0, -0.5, 0.0], speed: 0.5,
            travelled: 0.0, range: 300.0, damage: 0.0, radius: 0.0, mesh: 0, owner: usize::MAX,
            state: 0, bounces: 0, smoke_popped: false, resting: false, stuck: false,
            seek_target: None, phase: None, weapon_type: 0x3b,
        });
    }
    for t in 1..120u32 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Input::default(), false, false, false);
        for &o in &outs {
            assert!(!sim.drones.channels.on(o as u32), "tick {t}: plate {o:#x} fired with no bullet near it");
        }
        if sim.drones.grenades.is_empty() { break; }
    }
}
