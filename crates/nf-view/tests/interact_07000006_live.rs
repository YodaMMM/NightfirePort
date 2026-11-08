//! level-06 interactables: the doors, the guardroom radio and the pickups through the REAL input
//! path (`nf_view::load_with_carry` + `Sim::step_bond` walking/turning and the use key).
//!
//! Pins, with the disc rows from the census (docs/levels/07000006.md §5.2/§5.3/§5.6):
//! - **#47 Door_Top_I1** (p3 0 = no lock channel, E-usable row) opens on the use key [M Door_Activate
//!   0x135be0 through Player_Activate 0x1a5170's door arm]; its locked sibling **#369
//!   Locked_MainGate** (p0 2, p3 200) refuses until ch 200 is carried.
//! - **#807 Object_Radio** (0xD9, scripts 0x06000055/56): the class-0 SP E-activates and plays
//!   [M SP_Activate 0x1dc338 → the script start].
//! - **#755 oPickUp_BodyArmour**: walking over it takes it (armour rises) [M Pickup_Update's take
//!   gate]; the sister parked 5 m away never takes it.
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::{Input, RamSave};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn load_06(iso: &str, extra: &[u32]) -> nf_view::scene::Scene {
    let mut gates = vec![0x1d, 0x45, 0x70];
    gates.extend_from_slice(extra);
    let mut scene = nf_view::load_with_carry(iso, "07000006.bin", false, Some(&carry(&gates))).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!sim.disabled, "Bond is free after the intro");
    scene
}

fn settle_near(scene: &mut nf_view::scene::Scene, spot: [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let floor = sim.world.segment(spot, [spot[0], spot[1] - 30.0, spot[2]]).map(|t| spot[1] - 30.0 * t);
    // The game's placement, not a poke (`Player_StandAtNewPosition` 0x190d88 writes obj+0x40 too):
    // a FindCel miss leaves Bond unlinked AT the spot, never shoved back [M 0x133288:63-73].
    let yaw = sim.bond.yaw;
    let p = floor.map_or(spot, |f| [spot[0], f + 0.1, spot[2]]);
    sim.place_bond(p, yaw);
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
}

fn press_use(scene: &mut nf_view::scene::Scene, ticks: u32) {
    for _ in 0..ticks { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, true); }
}

/// KIND Door (the E-doors): #47 opens on the use key. Sister: no use key, the door stays closed.
#[test]
fn the_guardhouse_e_door_opens_on_the_use_key() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    let dpos = {
        let sim = scene.sim.as_ref().unwrap();
        let d = sim.drones.doors.list.iter().find(|d| d.placement == 47).expect("door #47 exists [D]");
        eprintln!("[door] #47 state {} ch_lock {} at {:?}", d.state, d.ch_lock, d.pos);
        assert_eq!(d.state, nf_game::door::CLOSED, "#47 starts closed");
        d.pos
    };
    // Stand INSIDE the guardroom (west of the door plane): the floor east of it drops away.
    settle_near(&mut scene, [dpos[0] - 1.2, dpos[1], dpos[2]]);
    { let sim = scene.sim.as_mut().unwrap();
      eprintln!("[door] bond {:?} (door y {:.2})", sim.bond.pos, dpos[1]);
      assert!((sim.bond.pos[1] - dpos[1]).abs() < 2.0, "Bond settled on the door's floor");
      sim.bond.yaw = (dpos[0] - sim.bond.pos[0]).atan2(dpos[2] - sim.bond.pos[2]); }
    press_use(&mut scene, 60);
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let d = sim.drones.doors.list.iter().find(|d| d.placement == 47).unwrap();
    eprintln!("[door] #47 after use: state {} kind {} ch_open {} straddle {}", d.state, d.kind, d.ch_open, d.straddle);
    assert!(d.state != nf_game::door::CLOSED, "the E-door opens on the use key [M Door_Activate 0x135be0]");
}

/// Sister of the E-door: the same walk with NO use press leaves #47 closed.
#[test]
fn no_use_key_leaves_the_guardhouse_door_closed() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    let dpos = scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .find(|d| d.placement == 47).expect("door #47").pos;
    settle_near(&mut scene, [dpos[0] + 1.2, dpos[1], dpos[2]]);
    for _ in 0..180 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let d = sim.drones.doors.list.iter().find(|d| d.placement == 47).unwrap();
    assert_eq!(d.state, nf_game::door::CLOSED, "standing by an E-door does not open it");
}

/// KIND Door (locked): #369 Locked_MainGate (p0 2, lock ch 200 [D]) refuses the use key while ch
/// 200 is low; with ch 200 carried the same press opens it [M Door_Update's lock gate].
#[test]
fn the_locked_main_gate_refuses_until_the_lock_channel_is_carried() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let dpos = {
        let mut scene = load_06(&iso, &[]);
        let p = scene.sim.as_ref().unwrap().drones.doors.list.iter()
            .find(|d| d.placement == 369).expect("door #369 Locked_MainGate [D]").pos;
        settle_near(&mut scene, [p[0] + 1.2, p[1], p[2]]);
        press_use(&mut scene, 60);
        for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        let sim = scene.sim.as_ref().unwrap();
        let d = sim.drones.doors.list.iter().find(|d| d.placement == 369).unwrap();
        assert_eq!(d.state, nf_game::door::CLOSED, "lock ch 200 low: the gate refuses [M Door_Update's lock gate]");
        p
    };
    // The same press with ch 200 carried opens it.
    let mut scene = load_06(&iso, &[200]);
    settle_near(&mut scene, [dpos[0] + 1.2, dpos[1], dpos[2]]);
    press_use(&mut scene, 60);
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let d = sim.drones.doors.list.iter().find(|d| d.placement == 369).unwrap();
    eprintln!("[door] #369 with ch 200: state {}", d.state);
    assert!(d.state != nf_game::door::CLOSED, "lock ch 200 high: the gate opens");
}

/// KIND ScriptPlayer: the guardroom radio #807 (0xD9, scripts 0x06000055/0x06000056, modes
/// [8,1], mask 2, out 0 [D #807]) — mode A = 8 ∉ {0,1} takes the create-time load beat
/// [M SP_CreateScriptPlayer 0x1db9a8:81-86; port from_placement]: the ambient script PLAYS FROM
/// LOAD (it is not E-usable — mask 2 fails Player_Activate's mask-&-4 gate [M 0x1dc338; port
/// useobj.rs]). Sister (the differential): #810 (mode 0, mask 1) does NOT auto-load — its script
/// is None until its trigger fires.
#[test]
fn the_guardroom_radio_is_an_ambient_sp_that_plays_from_load() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    {
        let sim = scene.sim.as_ref().unwrap();
        let radio = sim.drones.scripts.players.iter().find(|p| p.placement == 807).expect("SP #807 Object_Radio [D]");
        assert_eq!(radio.scripts, [0x06000055, 0x06000056], "the radio's script pair [D #807 p0/p1]");
        assert_eq!((radio.modes[0], radio.mask, radio.out), (8, 2, 0), "mode 8 = the load beat, mask 2 = not E-usable [M from_placement]");
        assert!(radio.info[0].is_some(), "mode 8 ∉ {{0,1}}: the script loads at create [M 0x1db9a8:81-86]");
    }
    // It keeps playing while Bond idles at the spawn (an ambient loop, not a one-shot).
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let radio = sim.drones.scripts.players.iter().find(|p| p.placement == 807).unwrap();
    assert!(radio.info[0].is_some() && !radio.deleted, "the radio is still live after 300 idle ticks");
}

/// Sister of the ambient radio: the trigger-start SP #810 (mode 0, mask 1) holds NO loaded script
/// at load — its dressing scene waits for the LoadPoint's ch 0x83 (walk_07000006_live drives it).
#[test]
fn the_trigger_start_sp_waits_for_its_channel() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 810).expect("SP #810 [D]");
    assert_eq!((sp.modes[0], sp.mask, sp.out), (0, 1, 0x83), "mode 0, mask 1, out ch 0x83 [D #810]");
    assert!(sp.info[0].is_none(), "mode 0 does not take the load beat — no script until the trigger [M 0x1db9a8]");
}

/// KIND Pickup: walking over the body armour #755 (36.06, 40.67, −22.21 [D]) takes it — armour
/// rises. Sister: parked 5 m west, the armour stays on the floor.
#[test]
fn walking_over_the_body_armour_takes_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    settle_near(&mut scene, [36.06, 40.67, -22.21]);
    // Walk in place a little: the take ray needs the contact, the pickup sits at floor height.
    let mut input = Input::default();
    for _ in 0..180 {
        input.forward = 0.4;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[armour] armour {:.1} taken {}", sim.drones.health.armour, sim.drones.taken.len());
    assert!(sim.drones.health.armour > 0.0, "the body armour is taken by the walk [M Pickup_Update's take]");
}

/// Sister: Bond 5 m from #755 for the same duration — armour stays 0, the pickup stays live.
#[test]
fn standing_away_leaves_the_body_armour_on_the_floor() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    settle_near(&mut scene, [31.06, 40.67, -22.21]);
    let mut input = Input::default();
    for _ in 0..180 {
        input.forward = 0.4;
        scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.health.armour, 0.0, "5 m west: the armour is not taken");
    assert!(!sim.drones.taken.iter().any(|(h, _)| *h == 0x0200047f || format!("{h}").contains("755")), "no #755 take");
}

/// KIND walk-in hazard: the courtyard well. The mouth is open AROUND barrel #65: the kind-0xD8
/// body at the mouth centre (31.82, 39.79, 17.26 [D #65]) is a solid ~0.46×0.92 prop the player's
/// capsule DOES hit — its glist carries a 94-tri collision block [D the 0x2E block before #65's
/// gfx, extents ±0.23 x / −0.92..0 y / ±0.20 z], and the game's pick chain admits a cel-linked
/// body for the player's walk query: `Player_Collision` 0x194df0:88/:205 builds the kind-0x800
/// query with HITTEST+0x98 = 0, `Collide_Pick` 0x1eb320:314-329 walks the cel's linked objects
/// (skip only on HITTEST+0x98 & 0x10 — absent — or obj+0xF0 & 0x30000000 — RB_Create writes
/// neither [M 0x1d6bf0:49-56: class 0x1a at obj+0xff, obj+0xfa |= 0x44]), `Collide_PickObj`
/// 0x1eae80:88-90 admits when the glist's registered shape list is non-empty (glist+8 != 0 — the
/// 0x2E block; obj+0xfa & 8 is clear, so the Jointy arm is out), and `Collide_Intersect`
/// 0x1ebaa8:89-105 runs the narrow phase over that geometry. Bond over the mouth centre stands ON
/// the barrel (~39.9); 0.7 off, the mouth is open and he drops ~6.2 to the shaft floor (the probe
/// finds it at 33.59) and stands there — no fall-through (the drops at ±0.35/0.7/1.0 in x/z all
/// land grounded; the old −356 fall-through is gone on the current tree). The well's own Hurt
/// volume #373 [D params [6, 1000, 0, 0xffff, 1, 0]: type 6, damage 1000, no gate, who 1] never
/// STARTS — `Hurt_Update`'s state-0 scan collects only the class bits 0x10, 8 and 2, and 1 & those
/// = 0 [M Hurt_Update 0x17e5d0:51-102] — but the start is not the damage: the class-0x27 arm of
/// `Player_DealWithObjHit` applies `Hurt_GetDamage` (ext+0x10, the 1000) on every contact with no
/// state or who check [M 0x194458:94-101, Hurt_GetDamage 0x17ea18], and only the disabled bit
/// (none here: no gate) keeps a volume out of the contact. So the drop into the shaft kills: the
/// volume's 1000 lands on the touch. (An earlier form of this test held the who-mask fold, the
/// volume inert and the drop survivable; that fold was refuted with the hurt-volume touch rule.)
#[test]
fn the_well_shaft_is_open_and_its_hurt_volume_kills_on_the_touch() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    // Over the well mouth (the well at (31.82, 39.79, 17.26) [D #65]): Bond rests on the barrel.
    settle_near(&mut scene, [31.82, 40.5, 17.26]);
    for _ in 0..600 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("[well] centre y {:.2} health {:.1} ch62 {}", sim.bond.pos[1],
        sim.drones.health.health, sim.drones.channels.on(0x62));
    // The stand height: the placement's floor snap puts the root 1.1 over the hit [M 0x190d88:104-116],
    // the same 1.1 a walked floor rests at. The top of the barrel is read from the same down ray.
    let top = sim.world.segment([31.82, 41.0, 17.26], [31.82, 11.0, 17.26]).map(|t| 41.0 - 30.0 * t).expect("the barrel top");
    // The barrel's rounded top keeps him sliding a little, so the band holds y, not the grounded bit.
    assert!((top - 40.04).abs() < 0.1 && (sim.bond.pos[1] - (top + 1.1)).abs() < 0.2,
        "Bond over the mouth centre stands ON body #65, the mouth's own barrel [D the 94-tri block; M 0x1eae80:88-90] (top {top:.2} y {:.2})",
        sim.bond.pos[1]);
    // 0.7 off centre the mouth is open: the drop lands ON the shaft floor (probe 33.59), not through it.
    let mut open = load_06(&iso, &[]);
    settle_near(&mut open, [31.12, 40.5, 17.26]);
    for _ in 0..600 { open.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = open.sim.as_ref().unwrap();
    eprintln!("[well] off-centre y {:.2} health {:.1} ch62 {}", sim.bond.pos[1],
        sim.drones.health.health, sim.drones.channels.on(0x62));
    assert!((33.0..36.0).contains(&sim.bond.pos[1]) && sim.bond.grounded,
        "0.7 off the mouth Bond stands on the shaft floor ~6.2 down [D the shaft floor tris at 33.59] (y {:.2} grounded {})",
        sim.bond.pos[1], sim.bond.grounded);
    // The touch law: #373 (never started, never disabled) deals its 1000 on Bond's contact
    // [M 0x194458:94-101]; the death raises the level's death channel 0x62.
    let well = sim.hurts.iter().find(|h| h.placement == 373).expect("the well's Hurt volume #373 [D]");
    assert!(!well.flying && !well.disabled, "#373 never starts and has no gate [M 0x17e5d0:51-102]");
    assert!(well.hits > 0 && sim.drones.health.health <= 0.0 && sim.drones.channels.on(0x62),
        "the shaft drop touches #373 and its 1000 kills (hits {}, health {:.1}) [M 0x194458:94-101]",
        well.hits, sim.drones.health.health);
}

/// Sister: Bond 3 m east of the well mouth for the same duration — alive, no death channel.
#[test]
fn standing_beside_the_well_lives() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load_06(&iso, &[]);
    settle_near(&mut scene, [34.82, 40.5, 17.26]);
    for _ in 0..600 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.health.health > 0.0 && !sim.drones.channels.on(0x62),
        "beside the well nothing happens");
}
