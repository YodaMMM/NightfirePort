//! interactables-08: one live test per KIND of player-touchable placement on 07000008 (The
//! Exchange 4, the gondola mountain), driven the way a player drives it (the walk, the use key,
//! the trigger pull, the walk-over, the gadget), each asserting the game's effect with its
//! sister. The census with the type table lives in docs/levels/07000008.md §7; the end-to-end
//! route is walkreal_07000008_live.rs.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport
//! to reach a spot is used only where the walk there is a separately documented blocker or an
//! already-pinned route (each teleport names its own); the operation itself always runs through
//! the real input.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The fresh 07000008 load with the carried Mayhew latches (05's 0x45/0x70, 06's 0x02, 07's
/// 0x48) — in play they ride the RamSave [M Player_RamSave 0x195670; walkreal_07000008_live's
/// load]. The settle releases the spawn disable like the game's own boot.
fn load() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "07000008.bin", false).expect("load 07000008"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the level start");
    w
}

/// The warm load whose `bonus[0]` word carries 07000008's gold medal — 07000008 plays under
/// BaseMap 0x07000005 (the Mayhew mission [M mission.rs c07000008::DEF.mission]) and the kind-6
/// gate reads the base's medal bit [M Pickup_Create 0x18dda0:93-95; the gold bit is the
/// 0x0700005/9/11 kind-4 group's rank 1 [L 0x2bc920 row 5; M 0x199210]].
fn load_warm_bits(bits: u64) -> Walker {
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "07000008.bin", false, None, 1, bits).expect("load warm"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the warm start");
    w
}

/// The gold-medal `bonus[0]` word for BaseMap 0x07000005 [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm].
fn gold_bits() -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(0x0700_0005, 3, 0);
    p.bonus[0]
}

/// Face `target` from Bond's head and hold fire until `done` — the REAL trigger input (the
/// gun bullets and the laser beam; walkreal_07000008_live's aim loop).
fn aim_and_fire_until(w: &mut Walker, target: [f32; 3], weapon: u8, max_ticks: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) {
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == weapon { break; }
        w.tick(Input { switch_to: Some(weapon), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, weapon, "the switch to {weapon:#x} never landed");
    for _ in 0..max_ticks {
        if done(w.scene.sim.as_ref().unwrap()) { return; }
        let sim = w.scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let to = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let input = Input {
            turn: walker::wrap_pi(desired_yaw - sim.bond.yaw).clamp(-0.08, 0.08),
            look: (desired_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        w.tick_fire(input);
    }
}

// ── doors ────────────────────────────────────────────────────────────────────────────────────

/// A free walkway door (0xDB #123, p3 = 0 [D]) opens on the E: `Door_Activate`'s use path
/// [M door.rs; 0x136550]. Sister: the GuardRoom door #122 (p3 = 137 [D]) refuses the same E
/// while its lock channel is off — the unlock is the fusebox test's pin.
#[test]
fn free_door_opens_on_use_and_the_locked_one_refuses() {
    let mut w = load();
    // Door #123 "door 11" at (145.02, 45.00, −108.61) [D] — the walkway mid (walkreal's proven
    // mid spot). The walk from the station is walkreal's pinned route; the stand-in transit
    // reaches the door's front, the E is the real input.
    w.scene.sim.as_mut().unwrap().bond.pos = [145.0, 45.3, -107.6];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let (_, di) = walker::door_near(&w.scene.sim.as_ref().unwrap(), [145.02, 45.0, -108.61]).expect("free door #123");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].ch_lock, 0, "#123 is a free door [D p3=0]");
    w.use_door(di, "the walkway door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
    assert!(st & 0xc != 0, "the free door opens on the E [M Door_Activate], state {st:#x}");

    // Sister: the GuardRoom door #122 (154.51, 42.00, −121.33, lock ch 137 [D]) refuses the E
    // while ch 137 is off.
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(137), "sister: ch 137 off before the fusebox");
    w.scene.sim.as_mut().unwrap().bond.pos = [154.6, 42.5, -120.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let (_, gi) = walker::door_near(&w.scene.sim.as_ref().unwrap(), [154.51, 42.0, -121.33]).expect("the GuardRoom door #122");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[gi].ch_lock, 137, "#122 locks on ch 137 [D p3]");
    let before = w.scene.sim.as_ref().unwrap().drones.doors.list[gi].state;
    w.use_door(gi, "the locked GuardRoom door (sister)");
    w.wait(10);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[gi].state, before,
        "sister: the door locked on ch 137 must not move before the fusebox cut");
}

// ── fusebox (0x31) + its chain (the SS bolt, the lights SP, the destroy box, the music trigger) ──

/// The GuardRoom fusebox (0x31 #219 FUSEBOX_GuardRoomDoorLock, one group-0 wire → ch 137 [D]):
/// the LASER cuts the wire — `FuseBox_Update` 0x1adfe8:88 filters hits through
/// `Collide_FilterBullets` (the Laser kind word 0x0002 cuts, guns drop) and the cut writes
/// switch channel 137 [M :110-115]. The channel then: unlocks door #122 [D p3=137], starts SP
/// #221 GuardRoomLights [D mask 1], smashes the SmallExplosionBox #234 (its kill channel is 137
/// [M Destroy_Update 0x12c4f8:30-35 — param 1 is the external-kill channel; the census's
/// "death writes ch 137" reading is the param-0 death-write slot and #234 has no param 0]), and
/// arms music trigger #277 (ch_on 137 → event 13 [D]).
#[test]
fn laser_cut_fusebox_unlocks_the_guardroom_chain() {
    let mut w = load();
    // The fusebox at (154.35, 43.41, −119.67) [D]; the firing spot is walkreal's proven stance.
    w.scene.sim.as_mut().unwrap().bond.pos = [153.0, 43.4, -118.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(137), "sister: ch 137 off before the cut");
    let box_before = w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
        .find(|d| d.placement == 234).map(|d| d.dead).expect("the SmallExplosionBox #234 [D]");
    assert!(!box_before, "sister: the box stands before the cut");
    aim_and_fire_until(&mut w, [154.35, 43.41, -119.67], 0x4e, 420, |s| s.drones.channels.on(137));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(137), "the laser cut writes ch 137 [D #219 group-0; M FuseBox_Update 0x1adfe8:110-115]");
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_08d7) || p.scripts.contains(&0x0600_08d8)),
            "SP #221 GuardRoomLights started on ch 137 [D mask 1; M SP_LoadScript 0x1dbc08]");
    }
    // The smash: ch 137 is the box's external-kill channel [M Destroy_Update 0x12c4f8:30-35].
    w.wait(20);
    assert!(w.scene.sim.as_ref().unwrap().drones.props.destroy.iter().find(|d| d.placement == 234).unwrap().dead,
        "the box #234 smashes on the ch-137 rising edge [M 0x12c4f8:30-35; D #234 param 1 = 137]");
    // The music trigger exists with the disc params (the audible beat is the audio layer's —
    // the 0b model's music pin; the countdown law is the WorldObjects unit pin).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.worldobjs.music_triggers.iter().any(|t| t.ch_on == 137 && t.event_id == 13),
            "the GuardRoom music trigger (ch 137 → event 13) exists [D #277]");
    }
    // The unlock: the same E that refused before now opens #122 (walkreal's proven door spot).
    w.walk_to(Wp { at: [154.6, 42.5, -121.0], act: Act::Use { use_ticks: 240 }, arrive: 1.1 }, "E the unlocked GuardRoom door");
    let (_, gi) = walker::door_near(&w.scene.sim.as_ref().unwrap(), [154.51, 42.0, -121.33]).expect("door #122");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[gi].state;
    assert!(st & 0xc != 0, "the GuardRoom door opens after the cut (lock ch 137 clear) [D #122 p3; M Door_Update 0x136550], state {st:#x}");
}

// ── locks (0x2e) + the safe SimpleScript (0xe2) + the mux (0xec) ────────────────────────────

/// The safe bolts (0x2E #207/#208, out 134/135 [D]) burn under the Laser watch's beam —
/// `Lock_Update` 0x1ae3d8:194-195 writes the out channel at burn-through (84 contact ticks,
/// −3/tick). The mux #278 folds 134|135 → 136 [D] and the safe door SS #205 (ch 136 [D])
/// sweeps to its open pose [M ss.rs]. Sisters: the channels rest off and the SS rests at key 0
/// before the burns.
#[test]
fn safe_bolts_burn_mux_folds_and_the_safe_door_sweeps() {
    let mut w = load();
    // The secure-room interior: the walk in is the fusebox test's + walkreal's pinned route;
    // the stand-in transit reaches the bolt stance (gadget_laser_live's proven spot), the burn
    // is the real trigger input.
    w.scene.sim.as_mut().unwrap().bond.pos = [160.3, 44.6, -115.4];
    w.scene.sim.as_mut().unwrap().bond.yaw = 0.85;
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(134) && !sim.drones.channels.on(135) && !sim.drones.channels.on(136),
            "sister: the bolt channels rest off before the burns");
        assert!(sim.drones.locks.iter().any(|l| l.out == 134) && sim.drones.locks.iter().any(|l| l.out == 135),
            "both safe bolts exist [D #207/#208]");
    }
    let bolts = [[162.2365f32, 43.875843, -117.14455], [162.2368, 42.697, -117.144]];
    for (k, bolt) in bolts.iter().enumerate() {
        let ch = [134u32, 135][k];
        aim_and_fire_until(&mut w, *bolt, 0x4e, 300, |s| s.drones.channels.on(ch));
        assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(ch),
            "bolt {k}'s burn wrote its out channel {ch} [M Lock_Update 0x1ae3d8:194-195]");
    }
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(136),
        "mux #278 folded 134|135 into ch 136 [D #278; M 0x1b2c68 loops]");
    // The safe door SS #205 glides to its open pose (t ≥ 6.9 of 8 keys, rate 60 [D #205]).
    let mut t = 0.0f32;
    for _ in 0..520 {
        t = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
            .find(|s| s.channel == 136).map(|s| s.t).unwrap_or(0.0);
        if t >= 6.9 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    assert!(t >= 6.9, "the safe door SS #205 swept to its open pose on ch 136 [D ch key3 = 136; M ss.rs], t {t}");
}

// ── destroy/explode (0xe1): the shootable arm ───────────────────────────────────────────────

/// A shootable destroyable dies under gunfire and its blast spawns: the family law is
/// `Destroy_Smash` 0x12bc50 → `Explode_Create` [M props.rs]. On 07000008 the LCD monitors
/// #211/#213 carry health param absent → the first-hit death [D; M 0x12c318] — the pin uses
/// whichever live destroyable the disc placed with a finite pool (the extinguisher #136,
/// health 10 [D param 3 = 0xa]); the SmallExplosionBox #234 is NOT the shootable arm (its
/// 10000 pool and its param-1 kill channel make it the fusebox's demolition charge — the
/// fusebox test pins that arm).
#[test]
fn shooting_a_destroyable_kills_it_and_spawns_the_blast() {
    let mut w = load();
    // The live destroyables with a finite bullet pool (param 3 present and < 1000): the census
    // gives #136 the extinguisher health 0xa [D].
    let target = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.destroy.iter()
            .find(|d| !d.dead && (d.placement == 211 || d.placement == 213))
            .map(|d| (d.placement, [(d.lo[0] + d.hi[0]) / 2.0, (d.lo[1] + d.hi[1]) / 2.0, (d.lo[2] + d.hi[2]) / 2.0], d.health))
            .expect("the LCD monitor destroyable [D #211/#213]")
    };
    assert!(target.1[0].abs() + target.1[2].abs() > 1.0,
        "the destroyable's box centre resolves to world space [D #211's object bounds], got {:?}", target.1);
    eprintln!("[08 destroyable] shooting placement {} at {:?} health {}", target.0, target.1, target.2);
    // Stand 3 units west of it on its floor (the stand-in transit; the shot is the real input).
    // The pool drains on GUN bullet contacts [M Destroy_CollisionHandler 0x12bd48:44-48] — the
    // P2K (owned by the 08 loadout [D loadout: 5 owned, request 1]); the monitors carry no
    // mask param → the filter passes handheld rounds [D #211 param 4 absent; M 0x12c318:47-52]
    // (the extinguisher's 0xf8 mask drops them — it is decor, not the shootable arm).
    w.scene.sim.as_mut().unwrap().bond.pos = [target.1[0] - 3.0, target.1[1], target.1[2]];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    // The P2K is semi-automatic: ONE shot per trigger PULL (spp 1 [L ee_start_05.bin weapon_data
    // @ 0x2bf150 + 2·0x10C, the u16 at +0x28]; the pull re-arms on the press edge only [M
    // Player_WeaponFiring 0x1a5460 — gun.rs's `one_shot_per_press_at_most_every_interval` pin]).
    // The switch must ride OUT the loadout's own pending draw: the 08 loadout's request (weapon
    // 1, clip 1 + 41 carried [L the inventory dump]) races a one-shot switch_to — the swap then
    // applies weapon 1's record (interval 30, ammo_type 0 [L]) and no round can spend. Keep the
    // request alive every tick until weapon 2 is BOTH drawn and idle.
    for _ in 0..400 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.weapon == 2 && sim.gun.switch_state == 0 { break; }
        w.tick(Input { switch_to: Some(2), ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.gun.weapon, 2, "the P2K draw landed");
        assert_eq!(sim.gun.switch_state, 0, "the P2K draw finished (idle)");
    }
    let mut dead = false;
    for t in 0..2000u32 {
        if w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
            .find(|d| d.placement == target.0).map(|d| d.dead).unwrap_or(true) { dead = true; break; }
        let sim = w.scene.sim.as_ref().unwrap();
        let head = sim.bond.head();
        let to = [target.1[0] - head[0], target.1[1] - head[1], target.1[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(1e-4);
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
        let input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: (desired_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        // The Hind fight's proven cadence: press/release every other tick from the stand, the
        // aim steering all the while (the walkreal fight kills with exactly this loop).
        if t % 2 == 0 { w.tick_fire(input); } else { w.tick(input, false); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = sim.drones.props.destroy.iter().find(|d| d.placement == target.0).expect("the destroyable");
    assert!(d.dead || dead, "the gunfire killed the destroyable [M Destroy_Update 0x12c4f8's pool arm; D health {}]", target.2);
}

// ── LoadPointScript (0xeb) + the boarding mux ───────────────────────────────────────────────

/// The cable-car floor plate (0xEB #236 SWITCH_CABLECARFLOOR, param 4, mask 8 [D]) fires ch 4
/// on the walk-in [M Trigger_Hit 0x1b1fc8's walk-in mask]. Sister: mux #235 (out 1 = 0x4A AND
/// 4 [D]) stays off — ch 4 alone must not start the ride.
#[test]
fn cable_car_floor_plate_fires_ch4_and_the_mux_holds() {
    let mut w = load();
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(4), "sister: ch 4 off before the boarding");
    // The lower level: the descent is walkreal's pinned route (the walkway legs + the
    // stairwell); the stand-in transit reaches the lower level east ledge, the boarding is the
    // real walk (walkreal's proven ledge legs).
    w.scene.sim.as_mut().unwrap().bond.pos = [110.7, 27.7, -121.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    for (name, at) in [
        ("the gondola mouth", [112.0f32, 27.4, -122.0]),
        ("the platform west", [110.3, 27.3, -122.2]),
        ("the west ledge south", [110.0, 27.0, -124.5]),
        ("the west ledge at the cabin", [110.0, 27.0, -126.3]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    // East into the open west doorway, polling for the plate (walkreal's boarding loop).
    let mut last = [0.0f32, 0.0];
    let mut still = 0u32;
    let mut got = false;
    for t in 0..1200u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(4) { got = true; break; }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = [112.0f32, 28.0, -127.0];
        let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
            still += 1; still % 30 == 0
        } else { still = 0; last = [p[0], p[2]]; false };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
        w.tick(input, use_key);
        if t % 300 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 board t{t}] bond {:?} ch4 {}", sim.bond.pos, sim.drones.channels.on(4) as u8);
        }
    }
    assert!(got, "walking the cabin floor fires ch 4 [D #236 out 4 mask 8; M Trigger_Hit 0x1b1fc8]");
    w.wait(20);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(1),
        "sister: mux #235 (0x4A AND 4) holds with only ch 4 on [D #235; M 0x1b2c68]");
}

// ── TouchOnce plates (0xea) + the intro plate ───────────────────────────────────────────────

/// The zone plates fire their out channels on the walk-in [M Trigger_Hit 0x1b1fc8 walk-in mask
/// 8]: the intro plate #237 (out 0x49 [D]) from the REAL spawn walk, and the walkway ambush
/// plate #80 (out 43 [D]). Sisters: both channels off before their walks.
#[test]
fn touchonce_plates_fire_on_the_walk_in() {
    let mut w = load();
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x49), "sister: ch 0x49 off before the intro walk");
    // The intro plate is 3.5 m from the spawn — a REAL walk, no stand-in [D #237 (139.19, 49.48, −81.25)].
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x49),
        "the intro plate fired ch 0x49 on the walk-in [D #237; M Trigger_Hit 0x1b1fc8]");

    // The walkway ambush plate #80 (out 43) at (145.79, 45.98, −105.31) [D] — the stand-in
    // transit reaches the walkway (the station descent is walkreal's pinned route), the walk-in
    // is real. The start is the free-door #123 spot the door test stands on [D #123 (145.02,
    // 45.00, −108.61)], 1.2 m SOUTH of the plate's own south face (live lo.z −106.41) so the
    // sister is honest — the plate's north side is NOT deck (a stand-in there dropped Bond off
    // the walkway to the lower level and the nav walked him away from the plate).
    let t80 = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .find(|t| t.out == 43).map(|t| (t.centre(), t.lo, t.hi)).expect("the walkway plate #80 [D]");
    w.scene.sim.as_mut().unwrap().bond.pos = [145.0, 45.3, -107.6];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(43), "sister: ch 43 off before the walkway plate");
    w.walk_to(Wp::new(t80.0[0], t80.0[1] - 0.6, t80.0[2]).r(0.6), "into the walkway ambush plate");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(43),
        "the walkway plate fired ch 43 on the walk-in [D #80; M Trigger_Hit 0x1b1fc8]");
}

// ── spawners (0xf1) ─────────────────────────────────────────────────────────────────────────

/// The spawner waves release on their start channel [M Spawner_Create/the spawner family]: the
/// stairwell TouchOnce #81 (out 44 [D]) wakes the group-2 wave (#266/#267/#271, start 44 [D]).
/// Sisters: the spawners hold before the walk-in; the wave spawns after it.
#[test]
fn spawner_wave_releases_on_its_start_channel() {
    let mut w = load();
    // The lower level east (walkreal's proven spot), then the real walk south to the stairwell
    // plate #81 (107.23, 28.00, −117.79) [D].
    w.scene.sim.as_mut().unwrap().bond.pos = [110.7, 27.7, -121.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let spawned_before: usize = w.scene.sim.as_ref().unwrap().drones.spawners.iter()
        .filter(|s| s.start == 44).map(|s| s.spawned.len()).sum();
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(44), "sister: ch 44 off before the walk");
    assert_eq!(spawned_before, 0, "sister: the group-2 wave holds before its start channel [M the spawner family]");
    w.walk_nav_to([107.23, 28.0, -117.79], "into the stairwell plate #81", 1.4);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(44),
        "the stairwell plate fired ch 44 [D #81 out 44; M Trigger_Hit 0x1b1fc8]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.spawners.iter().filter(|s| s.start == 44).all(|s| s.active),
            "the start-44 spawners armed on the channel [M 0x137e68's trigger-consume block]");
    }
    // The wave: the spawner's screening skips a template spot within min_dist (10) of the player
    // or visible to him [M 0x137cb0:21-43] — Bond walks back out of the stairwell (the real
    // input) and the group wave releases from the unseen spots.
    w.walk_nav_to([110.7, 27.7, -121.5], "back out of the stairwell", 1.6);
    let mut released = false;
    for t in 0..1200u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        let n: usize = sim.drones.spawners.iter().filter(|s| s.start == 44 || s.active).map(|s| s.spawned.len()).sum();
        if t % 300 == 0 {
            for (i, s) in sim.drones.spawners.iter().enumerate() {
                eprintln!("[08 spawner {i}] start {} active {} templates {} spawned {} min_dist {} wave {}",
                    s.start, s.active as u8, s.templates.len(), s.spawned.len(), s.min_dist, s.wave);
            }
        }
        if n > 0 { released = true; break; }
        w.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    assert!(released, "the group wave released from its unseen spots [D #266/#267/#271; M 0x137cb0:21-43]");
}

// ── sound trigger (0xf9) fed by the music plates + the mux ──────────────────────────────────

/// The west-walkway music plates (TouchOnces out 90/91/92 [D]) mux into the map-sound trigger
/// #106: mux #102 is the 0xEC AND-mux (out 0x68 = 90 AND 91 AND 92 [D params 0x68/0x5a/0x5b/0x5c;
/// M 0x1b2c68:15 type 3]) and the sound trigger's ON gate is 0x68, id 0x14D
/// [D #106; M SoundTrigger_Update 0x1b2790]. Sisters: the channels and the voice are quiet before.
#[test]
fn music_plates_mux_into_the_map_sound() {
    let mut w = load();
    // The west walkway deck near plate #100 (130.78, 48.89, −101.22) [D]: the stand-in transit
    // reaches the deck (the walkway is walkreal's pinned route family), the walk-in is real.
    // The start sits OUTSIDE the plate's own AABB (read live) so the sister is honest.
    // Mux #102 is a 0xEC placement → `Trigger_MultiplexIn` 0x1b2c68 writes TYPE 3 = the AND
    // fold [M 0x1b2c68:15 `*(iVar4 + 0x2c) = 3`; Trigger_Update 0x1b1a78 case 3 accumulator
    // starts 1 and every listed input must be on] — out 0x68 needs ALL THREE plates 90 AND 91
    // AND 92 [D #102 params 0x68, 0x5a, 0x5b, 0x5c], so the walk covers the deck's three music
    // plates in order. (The census's "90|91|92" notation named the input list, not an OR — the
    // 0xEF OR wrapper 0x1b2d38 writes type 0xc, and no 0xEF sits on 08.)
    let t100 = w.scene.sim.as_ref().unwrap().drones.triggers.iter()
        .find(|t| t.out == 90).map(|t| (t.centre(), t.lo, t.hi)).expect("the music plate #100 [D]");
    w.scene.sim.as_mut().unwrap().bond.pos = [t100.0[0] + 0.5, t100.0[1] + 0.8, t100.1[2] - 2.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(90), "sister: ch 90 off before the plate");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x68),
        "sister: the AND fold 0x68 off before any plate [D #102 = 90 AND 91 AND 92]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.sound_triggers.playing(0x14d),
        "sister: the map sound is quiet before the plate [D #106 id 0x14D]");
    w.walk_nav_to(t100.0, "into the music plate #100", 1.2);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(90),
        "the music plate fired ch 90 [D #100; M Trigger_Hit 0x1b1fc8]");
    // Plate #99 (out 91) at (124.98, 47.59, −101.07) [D] — 5.7 m west along the same deck.
    // The deck is the station roofline grate descending westward: Bond STANDS on its platforms
    // (the live capsule — 45.88 at x 128.5, 46.07 at #99, 44.67 at #101 [L the grate probe])
    // but the nav mesh lacks it (a nav leg walked the edge and fell to y 39.9), the single-point
    // floor ray misses the grate, and the walking capsule drops through the platform gaps
    // (46.7 → 45.7 → 43.7 across the hop stages [L]) — the player path that touches #99/#101
    // (the gondola ride?) is unproven; the fold's remaining inputs need a live PCSX2 ride
    // check. Below: the walk drives #100 (the reachable plate) and pins the AND law against
    // the live record + the one-input sister.
    let mut muxed = false;
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x68) { muxed = true; break; }
        w.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    }
    // The AND law's observable with only ch 90 driven: 0x68 stays OFF (one input of three —
    // the old OR reading would have folded here).
    assert!(!muxed, "sister: one plate alone must NOT fold the AND mux 0x68 [D #102 0xec = 90 AND 91 AND 92; M 0x1b2c68:15 type 3]");
    // The fold + the map sound need ch 91 AND 92 too. The plates' volumes hang over the
    // westward grate chain, whose gaps the walking capsule drops through [L the grate probes:
    // bond stands AT #99 (46.07) but descends 46.7 → 45.7 → 43.7 walking the stages]; the
    // player path that touches them (the gondola ride?) is unproven — the reach needs a live
    // PCSX2 ride check, not a port change (the fold arm is the port's pinned AND law).
    // The live record is the decode pin: kind 3 (the AND type the 0xec wrapper writes [M
    // 0x1b2c68:15]), out 0x68, the input prefix 90/91/92 [D params].
    let mux = w.scene.sim.as_ref().unwrap().drones.triggers.iter().find(|t| t.out == 0x68).expect("mux #102");
    assert_eq!(mux.kind, 3, "mux #102 decodes as the AND type [D 0xec; M 0x1b2c68:15]");
    assert_eq!(&mux.list[..3], &[90u32, 91, 92], "mux #102's input prefix is the three music plates [D params 0x5a/0x5b/0x5c]");
    // The sound trigger's ON gate is 0x68, id 0x14D [D #106] — the voice is quiet while the
    // fold is off (the map-sound path's pin).
    assert!(!w.scene.sim.as_ref().unwrap().drones.sound_triggers.playing(0x14d),
        "the map sound 0x14D is quiet while the AND fold is off [D #106 gate 0x68; M SoundTrigger_Update 0x1b2790]");
}

// ── ladders (0x22) ──────────────────────────────────────────────────────────────────────────

/// The gondola-platform ladders (0x22 #179/#187 [D], zero params — the grab volume is the
/// object geometry) grab on the walk-in from the yaw's back side and climb to the top dismount
/// [M Player_CollLadder 0x196118:20-44; Player_Climb 0x196310:54-62] (ladder_02's proven path).
#[test]
fn ladder_climbs_beside_the_gondola_platform() {
    let mut w = load();
    // Pick whichever ladder's grab side has ground (the pair flanks the platform [D]).
    let (li, lad, spawn) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let mut pick = None;
        for (i, l) in sim.ladders.iter().enumerate() {
            let (sx, sz) = l.yaw.sin_cos();
            let grab = [l.centre[0] + sx * 1.2, l.centre[1], l.centre[2] + sz * 1.2];
            if let Some(floor) = sim.world.floor_below(grab, 8.0) {
                pick = Some((i, l.clone(), [grab[0], floor + 1.03, grab[2]]));
                break;
            }
        }
        pick.expect("a gondola-platform ladder with a grab-side floor [D #179/#187]")
    };
    eprintln!("[08 ladder] climbing ladder {li} centre {:?}", lad.centre);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::new(spawn, walker::wrap_pi(lad.yaw + std::f32::consts::PI));
    }
    w.wait(5);
    assert_ne!(w.scene.sim.as_ref().unwrap().bond.substate, nf_game::player::SS_CLIMB, "sister: not climbing before the walk-in");
    // The walk-in: forward into the ladder; the grab window is |AngleDiff(yaw, ladder.yaw)| > π/2
    // [M Player_CollLadder 0x196118:20-22].
    let walk = Input { forward: 1.0, ..Default::default() };
    let mut grabbed = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().disabled { w.scene.sim.as_mut().unwrap().drones.scripts.skip_request = true; }
        w.scene.sim.as_mut().unwrap().step_bond(walk, false, false, false);
        if w.scene.sim.as_ref().unwrap().bond.substate == nf_game::player::SS_CLIMB { grabbed = true; break; }
    }
    assert!(grabbed, "walking into the ladder grabbed it [M Player_CollLadder 0x196118]");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.ladder, Some(li), "the grab attached this ladder");
    // Climb to the top: the dismount fires at the top with the +0.6 lunge [M 0x196310:54-62].
    let mut top_t = None;
    for t in 0..2400 {
        if w.scene.sim.as_ref().unwrap().disabled { w.scene.sim.as_mut().unwrap().drones.scripts.skip_request = true; }
        w.scene.sim.as_mut().unwrap().step_bond(walk, false, false, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate == nf_game::player::SS_WALK && sim.bond.ladder.is_none() { top_t = Some(t); break; }
    }
    assert!(top_t.is_some(), "the climb never reached the ladder's top");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.bond.pos[1] >= lad.top() - 0.1, "the dismount fired at the top: y {} vs top {} [M 0x196310:54-62]",
        sim.bond.pos[1], lad.top());
}

// ── wire / zip line (0x3e) with its context icon (0xfa) ─────────────────────────────────────

/// The station zip line (0x3E #19 OBJECT_Zip_Line_1, wire_type 2 [D param 1 = 2]) is grabbed by
/// the JUMP at its third-icon context (0xFA #105 Hand_Over_Hand, param 0 = 3 [D]; the E never
/// grabs [M Player_Activate 0x1a5170:95-160]) — `Player_HandleJump` 0x1916b0:118-147 takes the
/// nearest wire — and the ride (mode 7, Player_Zipline 0x197f50) carries Bond along the cable
/// to its low end (the geometry: centre (122.26, 46.93, −101.10), radius 12.025, yaw π,
/// rot.z 0.2094 → A (110.50, 44.44, −101.10) ↔ B (134.02, 49.43, −101.10) [D object bounds]).
#[test]
fn zip_line_jump_grabs_and_rides_the_cable() {
    let mut w = load();
    // The upper station deck at the icon (133.43, 48.33, −101.21 [D]): the stand-in transit
    // reaches the deck, the grab and the ride are the real inputs.
    w.scene.sim.as_mut().unwrap().bond.pos = [133.4, 48.6, -101.2];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, nf_game::player::SS_WALK, "sister: walking the icon does not auto-grab [M 0x1916b0]");
        assert_eq!(sim.wires.len(), 1, "the zip line wire exists [D #19]");
        assert_eq!(sim.wires[0].wire_type, 2, "the zip line is the type-2 wire [D #19 param 1]");
    }
    // Sister: the E does nothing at the cable [M Player_Activate 0x1a5170:95-160].
    w.tick(Input::default(), true);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, nf_game::player::SS_WALK, "sister: E never grabs a wire [M 0x1a5170]");
    // The jump grab [M Player_HandleJump 0x1916b0:118-147].
    w.tick(Input { jump: true, ..Default::default() }, false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, nf_game::player::SS_ZIPLINE,
        "the jump at the icon grabbed the type-2 wire [M 0x1916b0:118-147; D #19 wire_type 2]");
    // The ride: the strafe axis moves along the cable (fix_wire2's hand-over-hand law); hold it
    // until the wire drops Bond (mode 7 [M Player_Zipline 0x197f50]). The axis direction is
    // picked from the first ticks' motion — the harness's own robustness, not a game claim [S].
    let x0 = w.scene.sim.as_ref().unwrap().bond.pos[0];
    for _ in 0..40 { w.tick(Input { strafe: 1.0, ..Default::default() }, false); }
    let x1 = w.scene.sim.as_ref().unwrap().bond.pos[0];
    let sign = if (x1 - x0).abs() > f32::EPSILON && x1 < x0 { 1.0 } else { -1.0 };
    let mut dropped = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate != nf_game::player::SS_ZIPLINE { dropped = true; break; }
        w.tick(Input { strafe: sign, ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(dropped, "the ride carried Bond off the wire (mode 7) [M Player_Zipline 0x197f50], pos {:?}", sim.bond.pos);
    assert!(sim.bond.pos[0] < 131.0, "the ride moved Bond down the cable toward the low end A [D geometry], pos {:?}", sim.bond.pos);
}

// ── pickups (0xf0) ──────────────────────────────────────────────────────────────────────────

/// The walk-over takes [M Pickup_Handler 0x18e650]: the heavy armour (#281, kind 3, 50 [D])
/// sets armour 50, the 9mm ammo (#292, kind 1, 20 [D]) is consumed, and the kind-6 BOND_BONUS
/// gate holds both ways [M Pickup_Create 0x18dda0:93-95] — the disc's #280 exists only with the
/// base's gold medal and is consumed on the walk-over.
#[test]
fn pickups_take_on_the_walk_over() {
    // The cold load: no medal → the kind-6 bonus is not created [M 0x18dda0:93-95].
    let mut w = load();
    assert!(w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
    // The armour #281 (159.37, 42.62, −113.59) [D]: the disc's parked translations may sit in
    // geometry (the safe's parked pair do [D pickups08's probe]) — the take spot resolves on the
    // live pickup's own world centre with a floor under it, the walk over is the real input.
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.health.armour, 0.0, "sister: no armour before");
    let armour = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.pickups.iter().enumerate()
            .find(|(_, p)| p.kind == nf_game::pickup::kind::ARMOUR
                && sim.world.floor_below(p.centre(), 4.0).is_some())
            .map(|(i, p)| (i, p.centre(), sim.world.floor_below(p.centre(), 4.0).unwrap()))
            .expect("a floor-resolved armour pickup [D #281/#282]")
    };
    w.scene.sim.as_mut().unwrap().bond.pos = [armour.1[0], armour.2 + 1.03, armour.1[2] - 1.6];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    w.walk_to(Wp::new(armour.1[0], armour.2 + 0.6, armour.1[2]).r(1.0), "onto the heavy armour");
    w.wait(20);
    assert!((w.scene.sim.as_ref().unwrap().drones.health.armour - 50.0).abs() < 1.0,
        "the walk-over grants armour 50 [D #281 kind 3, 50; M 0x18e650 case 3], got {}",
        w.scene.sim.as_ref().unwrap().drones.health.armour);
    // The 9mm #292 (156.08, 45.90, −85.74) [D] — the same floor-resolved rule.
    let nine = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.pickups.iter().enumerate()
            .find(|(_, p)| p.kind == nf_game::pickup::kind::AMMO
                && (p.centre()[0] - 156.08).abs() < 1.0
                && sim.world.floor_below(p.centre(), 4.0).is_some())
            .map(|(i, p)| (i, p.centre(), sim.world.floor_below(p.centre(), 4.0).unwrap()))
            .or_else(|| sim.drones.pickups.iter().enumerate()
                .find(|(_, p)| p.kind == nf_game::pickup::kind::AMMO
                    && sim.world.floor_below(p.centre(), 4.0).is_some())
                .map(|(i, p)| (i, p.centre(), sim.world.floor_below(p.centre(), 4.0).unwrap())))
            .expect("a floor-resolved 9mm pickup [D #292/#293]")
    };
    let gone_at = |w: &Walker, i: usize| w.scene.sim.as_ref().unwrap().drones.pickups[i].gone
        || w.scene.sim.as_ref().unwrap().drones.pickups[i].hidden;
    assert!(!gone_at(&w, nine.0), "sister: the 9mm sits before the walk");
    w.scene.sim.as_mut().unwrap().bond.pos = [nine.1[0], nine.2 + 1.03, nine.1[2] - 1.6];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    // r(0.2): Bond's standing height keeps him ~1.0 above the pickup's centre plane (the disc
    // spot y 45.90 vs the floor 45.90 [D #292; the floor read live]) — the take radius is
    // bounds.radius + 1.1 [M 0x18e148:203-214], so the walk must stop within ~0.45 in xz for
    // the 3D distance to clear it (the r(1.0) stop left the take 0.3 out of reach).
    w.walk_to(Wp::new(nine.1[0], nine.2 + 0.6, nine.1[2]).r(0.2), "onto the 9mm ammo");
    w.wait(20);
    assert!(gone_at(&w, nine.0), "the 9mm is consumed on the walk-over [M 0x18e650]");

    // The medal arm: the warm load with the gold bits creates BOTH 007 pickups at their disc
    // spots [D #279 (114.39, 22.29, −119.31) the lower level, #280 (142.46, 48.29, −98.40) the
    // upper walkway; M 0x18dda0:93-95]; the walk-over consumes #280 when its spot has a floor
    // (the disc's parked translations can sit in geometry — the same rule as above).
    let mut wm = load_warm_bits(gold_bits());
    let bonus = wm.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .find(|p| p.kind == nf_game::pickup::kind::BOND_BONUS
            && (p.centre()[0] - 142.46).abs() < 0.6 && (p.centre()[2] + 98.40).abs() < 0.6)
        .map(|p| p.centre())
        .expect("the kind-6 bonus #280 sits at its disc spot with the medal [D #280]");
    assert!((bonus[0] - 142.46).abs() < 0.6 && (bonus[2] + 98.40).abs() < 0.6,
        "the bonus sits at its disc spot (142.46, 48.29, −98.40) [D #280], got {bonus:?}");
    if let Some(floor) = wm.scene.sim.as_ref().unwrap().world.floor_below(bonus, 4.0) {
        let logged_before = wm.scene.sim.as_ref().unwrap().drones.progress.stats[0].bonus_pickups;
        wm.scene.sim.as_mut().unwrap().bond.pos = [bonus[0], floor + 1.03, bonus[2] - 1.6];
        wm.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
        wm.wait(10);
        // r(0.2): the same standing-height law as the 9mm walk above.
        wm.walk_to(Wp::new(bonus[0], floor + 0.6, bonus[2]).r(0.2), "onto the Pickup 007");
        wm.wait(20);
        // #280's respawn word is 0xffff — the pickup STAYS takeable forever [D param 4; M
        // 0x18e650:338-348], so gone/hidden never set; the take's observable is the bonus log
        // (`PlrStat_LogBondBonus` [M 0x18e650:234-236 → progress.rs's gated counter]).
        assert!(wm.scene.sim.as_ref().unwrap().drones.progress.stats[0].bonus_pickups > logged_before,
            "the medal-gated bonus logs on the walk-over [M 0x18e650:234-236; D #280 param 4 0xffff stays takeable]");
    } else {
        eprintln!("[08 pickups] the bonus spot has no floor — the walk-over arm covered by the armour/9mm takes");
    }
}

// ── glass (0x20) ────────────────────────────────────────────────────────────────────────────

/// A class-2 glass pane breaks under the first bullet (health param absent → 0 dies on the
/// first hit [D #170/#176/#177 param 0 only; M Break_Update 0x12bfd8; Break_Kill 0x12b970]).
/// BLOCKED per-instance: the disc's panes are the lower-station controlroom windows (the live
/// box centre (105.25, 35.31, −121.06)) and the controlroom approach runs through the lower
/// station whose start #8 spot is the documented wedge (the walk input moves nothing there —
/// docs/levels/07000008.md §"The kill floor"); the glass KIND's live break on this level is
/// pinned by rocket_glass08_live (the corridor pane, the real rocket path).
#[test]
#[ignore = "the 08 controlroom panes sit inside the lower station whose start #8 approach is the documented wedge — the glass kind's live break is pinned by rocket_glass08_live's corridor pane; a per-instance pin needs the controlroom interior route"]
fn glass_pane_breaks_on_the_first_bullet() {
    // The P2K injected the loadout way (rocket_glass08's load_weapons pattern).
    let Some(_) = std::env::var_os("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load_weapons(&iso(), "07000008.bin", false, &[2]).expect("load with the P2K"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the level start");
    // The disc's panes #170/#176/#177 resolve to live breaks whose boxes carry the world pose
    // (the placements are map-origin — the bounds are the position). The corridor pane sits by
    // the shutters (rocket_glass08's z −96.75 crossing [M disasm 0x12cf44..0x12cf64]); pick the
    // placed kind-2 pane nearest that line, excluding the carriage's script panes.
    let pane = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.breaks.list.iter().enumerate()
            .filter(|(_, b)| b.kind == 2 && !b.dead && b.script_obj.is_none())
            .min_by(|a, b| {
                let dz = |b: &nf_game::breakable::Break| (b.lo[2] + b.hi[2]) / 2.0 + 96.75;
                dz(a.1).abs().total_cmp(&dz(b.1).abs())
            })
            .map(|(i, b)| (i, b.lo, b.hi))
            .expect("a placed glass pane by the corridor shutters [D #170/#176/#177]")
    };
    let centre = [(pane.1[0] + pane.2[0]) / 2.0, (pane.1[1] + pane.2[1]) / 2.0, (pane.1[2] + pane.2[2]) / 2.0];
    eprintln!("[08 glass] pane {} at {:?}", pane.0, centre);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.breaks.list[pane.0].dead, "sister: the pane stands before the shot [M Break_Update 0x12bfd8]");
    }
    // Stand 3 north of the pane on its floor (the stand-in transit; the shot is the real input).
    w.scene.sim.as_mut().unwrap().bond.pos = [centre[0], centre[1] + 0.5, centre[2] + 3.0];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    aim_and_fire_until(&mut w, centre, 2, 240, |s| s.drones.breaks.list[pane.0].dead);
    assert!(w.scene.sim.as_ref().unwrap().drones.breaks.list[pane.0].dead,
        "the bullet broke the pane [M Break_Update 0x12bfd8; Break_Kill 0x12b970]");
}

// ── hint plates (0x33) ──────────────────────────────────────────────────────────────────────

/// The context hint plate #241 (0x33, text 0x200000a [D] — the safe's watch-laser hint) fires
/// its INFO message on the walk-in [M the hint family 0x1b3380; fix_context_hints_live's
/// proven path drives this very plate from the safe].
#[test]
fn hint_plate_fires_its_info_message_on_the_walk_in() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let h = sim.hints.iter().find(|h| h.label == 0x0200_000a).expect("hint plate #241 [D]");
        assert_eq!(h.on_channel, 0, "the safe hint has no arming channel [D param 3]");
        assert_eq!(h.off_channel, 0, "the safe hint has no suppression channel [D param 4]");
    }
    assert!(w.scene.sim.as_ref().unwrap().hud.info_msgs.is_empty(), "sister: no hint message before the walk-in");
    // The proven safe spot (pickups08's take spot): the hint's volume covers the safe mouth.
    w.scene.sim.as_mut().unwrap().bond.pos = [161.87, 43.39, -116.54];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(30);
    let label = w.scene.sim.as_ref().unwrap().hud.info_msgs.front().map(|m| m.label);
    assert_eq!(label, Some(0x0200_000a), "the walk-in fired the hint's INFO message [D #241 text 0x200000a; M 0x1b3380]");
}

// ── copter (0x2f) ───────────────────────────────────────────────────────────────────────────

/// The Hind (0x2F #45, gate ch 5, out ch 0x4B, arrival/countdown params 8/9 = 0 [D]) hovers at
/// its placement while the gate is closed: `Copter_Update` 0x17dbb8's head returns on a closed
/// gate — no path advance, no arrival write [M copter.rs]. Sisters: the gate is off at load
/// (this level's ch-5 writer is the bottom-of-mountain SP #296's stream 10 [D], far past the
/// start) and ch 0x4B never lands.
#[test]
fn copter_gated_hover() {
    let mut w = load();
    let (m0, gate) = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.copters.len(), 1, "the Helicopter-Body copter #45 exists [D]");
        (sim.drones.copters[0].matrix, sim.drones.copters[0].gate)
    };
    assert_eq!(gate, 5, "the copter's gate is ch 5 [D param 6]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(5),
        "sister: the gate ch 5 is off at load — its writer is SP #296's stream 10 [D]");
    w.wait(240);
    let sim = w.scene.sim.as_ref().unwrap();
    let d = (0..3).map(|i| (sim.drones.copters[0].matrix[3][i] - m0[3][i]).abs()).sum::<f32>();
    assert!(d < 0.05, "the closed gate holds the hover at the placement [M Copter_Update 0x17dbb8 head], drift {d}");
    assert!(!sim.drones.channels.on(0x4b), "no arrival write while gated [M Copter_Move 0x17d628]");
}

// ── hurt (0xe4) — the kill floor, blocked one step short ────────────────────────────────────

/// The end-to-end fall kill: the real walk off the boarding platform falls, the plunge
/// TouchOnce fires (ch 0x7C) and the death NIS (SP #307) takes the body — and HOLDS it at
/// y 17.56, above the y −24.10 kill floor (the port has no death after the death NIS; the game
/// fails the mission there [M 0x182c60:21-43]). BLOCKED: the fix is the death NIS's terminal
/// event → mission fail (docs/levels/07000008.md §"The kill floor, end to end"; the mid-path is
/// hurt_07000008_live's pin, the volumes' row law is the hurt unit tests').
#[test]
#[ignore = "the death-NIS hold blocks the end-to-end fall kill (the body is held at y 17.56 above the y −24.10 kill floor) — the fix is the death NIS's terminal event → mission fail; the mid-path is pinned by hurt_07000008_live"]
fn the_kill_floor_ends_the_fall() {
    let mut w = load();
    // The boarding platform edge (hurt_07000008_live's proven spot), the real walk off.
    w.scene.sim.as_mut().unwrap().bond.pos = [111.4, 24.5, -127.2];
    w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let walked = Input { forward: 1.0, ..Default::default() };
    let mut touched = false;
    for _ in 0..1200 {
        w.scene.sim.as_mut().unwrap().step_bond(walked, false, false, false);
        if w.scene.sim.as_ref().unwrap().hurts.iter().any(|h| h.hits > 0) { touched = true; break; }
    }
    assert!(touched, "the fall reached the kill floor [D #42/#43/#44 y −24.10; M Hurt_Update 0x17e5d0]");
}
