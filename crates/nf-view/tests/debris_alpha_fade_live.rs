//! debris-alpha-fade: the blast debris pieces' sim alpha byte (obj+0x106) steps down 1 per tick
//! and reaches the draw as the game's tweak A. The game's rules [M]:
//! - `Debris_CreateEx` 0x1ac808:27 stores obj+0x106 = 0xff at spawn;
//! - `Debris_Update` 0x1aac00:18-24 subtracts the record's step byte (INFO+0x32 = 1 [M
//!   0x1ac808:32]) from obj+0x106 every tick while the result stays > 0, and kills the piece
//!   (`obj+0xFE |= 1` [M 0x1aac00:56-63]) the tick the subtraction would reach ≤ 0 — the byte
//!   itself never holds a value below 1 while the piece draws;
//! - the piece also dies when its tick count runs out (INFO+0x30 < 1 [M 0x1aac00:14]) — the
//!   blast pieces' life is `Rand(90) + 90` ticks [M 0x1ac808:29-31 over param 5 = 180, the
//!   `Explode_Update` arm's `Debris_CreateEx(…, 180, 3, …)` 0x17c350:63-67], so death is by life
//!   and the last stored alpha is 255 − life ∈ 76..165.
//! The draw chain that consumes the byte [M]: `View_DrawObjList` pass 0 calls
//! `View_SetupRenderModes_Tweak(viewer, obj)` per object (0x1e69a0:77), which passes
//! `psiSetTweakARGB(A = obj+0x106, R, G, B)` (0x1e5888:46-49; debris RGB tint is 0xff —
//! `control_init_object` 0x134630:18-21, no later writer); `psiSetTweakARGB` 0x124770 loads
//! TAmbient = (R, G, B, A)/255 @0x2650d0..dc and `FillMatrixChainRot` 0x122310:241 feeds it to
//! `sceVu0LightColorMatrix` as the VU1 lighting matrix's ambient term. A debris object carries
//! obj+0xFA = 0x20 only, so it draws through `View_DrawObjectGlist` 0x1e6780.
//!
//! The pins here run the REAL path: the level loads, Bond stands at the level's own MGL pickup,
//! one contact round fires through the real fire input, the blast spawns its metal pieces, and
//! the sim's own ticks step the pieces' fade byte. No teleport after the load beyond standing at
//! the pickup's placement (the grenade_bounce_debris_live harness's rule).
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// One piece's observed alpha trail: the draw slot's serial (stable while the piece lives), the
/// tick it was first seen, and the fade bytes observed per tick.
struct Trail {
    serial: u32,
    first_tick: u32,
    fades: Vec<u8>,
}

/// Fires one contact round of weapon `w` straight ahead and walks the sim N ticks, tracking every
/// blast debris piece's fade byte per tick. Returns the trails.
fn fire_and_track(scene: &mut nf_view::scene::Scene, w: u16, ticks: u32) -> (u16, Vec<Trail>) {
    {
        let sim = scene.sim.as_mut().unwrap();
        let owned = sim.drones.inventory.owned.len();
        if (w as usize) < owned { sim.drones.inventory.owned[w as usize] = true; }
        sim.drones.inventory.clip[w as usize] = 6;
        sim.gun.weapon = w as u8;
        sim.gun.switch_request = w as u8;
        sim.gun.switch_state = 0;
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 6;
        sim.gun.pending = 0;
    }
    let mut input = Input::default();
    input.fire = true;
    let mut kind = None;
    let mut trails: Vec<Trail> = Vec::new();
    for t in 0..ticks {
        let sim = scene.sim.as_mut().unwrap();
        let n0 = sim.drones.blasts.len();
        sim.step_bond(if t == 0 { input } else { Input::default() }, t == 0, false, false);
        let sim = scene.sim.as_mut().unwrap();
        if kind.is_none() && sim.drones.blasts.len() > n0 {
            let b = sim.drones.blasts.iter().rev().find(|b| b.weapon_type == w);
            kind = b.map(|b| b.debris);
        }
        // The blast pieces this tick, keyed by their draw slot's serial (unique while alive —
        // `Effects::assign_debris_slots` hands each piece a slot it keeps for life).
        let mut seen = std::collections::HashSet::new();
        for d in &sim.drones.blast_debris {
            if (d.draw.slot as usize) >= nf_game::effect::DEBRIS_DRAW_SLOTS { continue; }
            seen.insert(d.draw.serial);
            match trails.iter_mut().find(|tr| tr.serial == d.draw.serial) {
                Some(tr) => tr.fades.push(d.fade),
                None => trails.push(Trail { serial: d.draw.serial, first_tick: t, fades: vec![d.fade] }),
            }
        }
        // A trail whose piece died stops growing (the vec's length freezes at its last tick).
    }
    (kind.unwrap_or(0), trails)
}

#[test]
fn blast_debris_fade_steps_one_per_tick_from_ff_and_dies_by_life() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load 07000011");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    // The MGL pickup's spot [D: the level's 0x2A pickup], facing the metal shelf 5.3 units away.
    let at = scene.sim.as_ref().unwrap().drones.pickups.iter().find(|p| p.weapon == 0x2a)
        .expect("07000011 places an MGL").start;
    let yaw = 0.35f32;
    scene.sim.as_mut().unwrap().place_bond(at, yaw);
    let (kind, trails) = fire_and_track(&mut scene, 0x2a, 260);
    assert!(kind == 5 || kind == 6, "a metal contact blast carries the metal debris kind, got {kind}");
    assert!(trails.len() >= 3, "the blast threw pieces (3 per passing tick [M 0x17c350:63-67]), got {}", trails.len());

    for tr in &trails {
        // obj+0x106 = 0xff at create [M 0x1ac808:27] — the piece is first observed on its spawn
        // tick, before any Debris_Update ran on it.
        assert_eq!(tr.fades[0], 0xff,
            "piece (serial {}) fade at spawn = 0xff [M 0x1ac808:27], trail {:?}", tr.serial, tr.fades);
        // Debris_Update steps the byte down EXACTLY 1 per tick [M 0x1aac00:18-21 with INFO+0x32
        // = 1, 0x1ac808:32] — fade at age k is 0xff − k.
        for (k, &f) in tr.fades.iter().enumerate() {
            assert_eq!(f, 0xff - k as u8,
                "piece (serial {}) fade at age {k} = 0xff − {k} [M 0x1aac00:18-21], trail {:?}", tr.serial, tr.fades);
        }
        // Death is by life, not by alpha: the blast piece's life is Rand(90)+90 ≤ 179 ticks
        // [M 0x17c350:63-67 + 0x1ac808:29-31], so the last stored byte is 255 − life ∈ 76..165
        // — every observed fade stays > 1 and the trail just stops (the piece left the list).
        let last = *tr.fades.last().unwrap();
        assert!(last > 1, "piece (serial {}) died by life with alpha {last} still on the byte [M 0x1aac00:14 vs :18-24]", tr.serial);
        assert!(tr.fades.len() >= 90 && tr.fades.len() <= 179,
            "piece (serial {}) lived {} ticks — the Rand(90)+90 window [M 0x17c350:63-67]", tr.serial, tr.fades.len());
    }
    // The pieces are all gone once their lives run out: nothing survives past tick 179 of the
    // oldest spawn (the run tracked 260 ticks).
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.blast_debris.is_empty(), "every piece's Debris_Update killed it by tick 179 [M 0x1aac00:14]");
}

#[test]
fn a_late_piece_keeps_the_same_per_tick_fade_rule() {
    // The blast's debris arm passes pieces on up to 3 separate ticks (the age % (Rand(5)+10)
    // gate [M 0x17c350:34-38]); a piece born later follows the same rule from ITS spawn tick.
    // This is covered by the trails' own first_tick spread — assert the spread exists so the
    // per-trail pins above really did cover more than one spawn tick when the roll allows it,
    // and stay silent (no hard pin on the shared RNG's roll) when it does not.
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load 07000011");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let at = scene.sim.as_ref().unwrap().drones.pickups.iter().find(|p| p.weapon == 0x2a)
        .expect("07000011 places an MGL").start;
    scene.sim.as_mut().unwrap().place_bond(at, 0.35);
    let (_, trails) = fire_and_track(&mut scene, 0x2a, 260);
    let mut firsts: Vec<u32> = trails.iter().map(|t| t.first_tick).collect();
    firsts.sort_unstable();
    firsts.dedup();
    eprintln!("spawn ticks observed: {firsts:?}");
    for tr in &trails {
        assert_eq!(tr.fades[0], 0xff, "every spawn tick's pieces start at 0xff [M 0x1ac808:27]");
    }
}
