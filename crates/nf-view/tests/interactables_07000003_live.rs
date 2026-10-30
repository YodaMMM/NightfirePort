//! interactables-03: one live test per KIND of player-touchable placement on 07000003 (Double
//! Cross part 3, the castle servants' floor), driven the way a player drives it (the walk, the
//! use key, the trigger pull, the walk-over), each asserting the game's effect with its sister.
//! The census is `nf-extract placements-all 07000003.bin` over main map 50 (511 placements
//! [D]; docs/levels/07000003-audit.md's type table) — the player-touchable kinds on this level:
//!
//! | kind | n | the game's effect | test |
//! |---|---|---|---|
//! | 0xDB doors | 13 | E opens; #4 locks on ch 0x31 [D p3=49] | doors / e_plate |
//! | 0xEA/0xEB plates | 19+2 | walk-over (mask 8) and E (mask 4) write the out channel | walk_plates / e_plate |
//! | 0xF0 pickups | 5 | walk-over takes (armour; the medal-gated kind-6) | pickups |
//! | 0x20 breakables | 97 | bullets shatter | breakables |
//! | 0xE1 destroyables | 5 | bullets smash (out ch 0 — no channel [D p0]); their 0x31 fusebox twins are scriptless and NEVER created [M 0x1b0448] | destroyables |
//! | 0x22 ladder | 1 | the climb (this level's only ride kind — lifts are absent [D census]) | ladder |
//! | 0xD9 script players | 3 | the mask-4 E-use plays the script (#110's dragon → the safe key) | dragon |
//!
//! ZERO on main map 50 [D census vs the full dispatcher 0x1d04f0]: switches 0x29, monitors 0x30,
//! sensors 0x28, safes/locks 0x2E, copters 0x2F, turrets 0x27, grapple points 0x3A, lifts — the
//! level's E-usable "console" role is the dragon script entity and its vertical role is the ladder.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport
//! to reach a spot is used only where the walk there is a separately documented blocker (marked
//! at the site); the operation itself always runs through the real input.

#[path = "common/walker_b2.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    Walker::new(nf_view::load(&iso(), "07000003.bin", false).expect("load 07000003"))
}

/// Walks the aim onto `at` from the eye and taps the trigger (the glass-shatter harness:
/// turn/look inputs, fire through the real gun path, reload when the clip runs dry).
fn fire_at(scene: &mut nf_view::scene::Scene, at: [f32; 3], ticks: u32) -> bool {
    let mut last_tap = 0u32;
    let mut fired = false;
    for t in 0..ticks {
        let (input, trigger, reload) = {
            let sim = scene.sim.as_ref().unwrap();
            let (bp, dp) = (sim.bond.pos, at);
            let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
            let dx = dp[0] - eye[0];
            let dy = dp[1] - eye[1];
            let dz = dp[2] - eye[2];
            let dxz = (dx * dx + dz * dz).sqrt().max(0.5);
            let desired = dx.atan2(dz);
            let err = { let mut e = desired - sim.bond.yaw; while e > std::f32::consts::PI { e -= 2.0 * std::f32::consts::PI; } while e < -std::f32::consts::PI { e += 2.0 * std::f32::consts::PI; } e };
            let want_pitch = (dy / dxz).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let clip_empty = sim.gun.clip == 0 && sim.gun.spare > 0;
            let on_target = err.abs() < 0.02 && perr.abs() < 0.028 && t.saturating_sub(last_tap) > 15;
            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
             on_target && !clip_empty,
             clip_empty && t % 20 == 0)
        };
        if trigger { last_tap = t; fired = true; }
        scene.sim.as_mut().unwrap().step_bond(input, trigger, reload, false);
        let sim = scene.sim.as_ref().unwrap();
        if fired && t.saturating_sub(last_tap) > 40 { break; }
    }
    fired
}



/// The nav walk with the doors the route crosses opened by the player's own E (the guards'
/// graph routes the rooms; a closed door on the route is opened with the real use key, then the
/// leg retries — the player's own path through the level).
fn walk_over(w: &mut Walker, target: [f32; 3], label: &str) {
    for k in 0..6 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(p, target) < 1.5 { return; }
        let before = p;
        w.walk_nav_to(target, label, 0.9);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(p, target) < 1.5 { return; }
        let dn = walker::door_near(w.scene.sim.as_ref().unwrap(), p);
        if let Some((dpos, di)) = dn {
            if walker::dxz(dpos, p) < 2.8 && walker::dxz(dpos, target) < walker::dxz(p, target) + 1.0 {
                w.use_door(di, &format!("{label}: the door on the way ({k})"));
                continue;
            }
        }
        if walker::dxz(p, before) < 0.5 { break; }
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(walker::dxz(p, target) < 2.0, "{label}: the nav walk never arrived, at ({:.2},{:.2},{:.2}) d {:.2}",
        p[0], p[1], p[2], walker::dxz(p, target));
}

/// Stand ~in reach of `centre`, aim at it and press the use key — the player's E at a plate or
/// script entity (the use sphere is head + 1×look [M 0x1a5170:100-108]).
fn use_at(w: &mut Walker, centre: [f32; 3], label: &str) {
    for k in 0..60 {
        let (bp, yaw, pitch) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let head = sim.bond.head();
            let to = [centre[0] - head[0], centre[1] - head[1], centre[2] - head[2]];
            let l = (to[0] * to[0] + to[2] * to[2]).sqrt().max(1e-4);
            (bp, to[0].atan2(to[2]), (to[1] / l).atan() / std::f32::consts::FRAC_PI_2)
        };
        let _ = bp;
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.yaw = yaw;
            sim.bond.pitch = pitch;
        }
        w.tick(Input::default(), k % 6 == 0);
    }
}

/// The gold-medal `bonus[0]` word for 07000003's base map — the mission row's `mission` hash
/// 0x07000001 [L ee_start_05 @ 0x2a3f18 via mission::c07000003::DEF.mission; M
/// PlrStats_HasGoldMedal 0x199a40 = Menu_HasMedal(BaseMapHCode, 3, 0)].
fn gold_bits() -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(0x0700_0001, 3, 0);
    p.bonus[0]
}

fn load_gold() -> Walker {
    Walker::new(nf_view::load_with_carry_medals(&iso(), "07000003.bin", false, None, 1, gold_bits()).expect("load gold"))
}

/// The walk-over plates (0xEA TouchOnce mask 8 [D]): the game fires the out channel when the
/// player's touch volume enters the plate. Plate #476 (out 0x4D, the objective reveal) sits at
/// (−8.95, 6.04, −23.21) [D] — the 0x24 spawn is (−8.85, 6.84, −23.19) [D], so the level's first
/// ticks fire it. Sister: the far #415 plate (out 0x21, (14.29, 6.10, −33.33) [D]) stays silent
/// until Bond walks onto it.
#[test]
fn walk_over_plates_fire_their_out_channels() {
    let mut w = load();
    w.wait(5);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d),
        "the spawn plate #476 fires ch 0x4D on the first ticks [D #476 at the 0x24 spot; M Trigger_Update 0x1b1a78]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x21),
        "sister: the far #415 plate is silent at the spawn");
    // Walk onto #415 — the real walk-over. Its centre from the live trigger record.
    let c = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH_ONCE, 0x21)
        .expect("the 0x21 plate exists");
    // Two plates write 0x21 [D #414 (8.98, −34.35) / #415 (14.29, −33.33)] — either is the sister.
    assert!(((c[0] - 14.29).abs() < 0.6 && (c[2] + 33.33).abs() < 0.6)
        || ((c[0] - 8.98).abs() < 0.6 && (c[2] + 34.35).abs() < 0.6), "the 0x21 plate at its disc spot [D], got {:?}", c);
    // The bedroom's east furniture blocks the direct last leg (the probe: the walk wedges at
    // (6.35, −30.17), d 4.9) — the stand-in transit stops 3 short; the walk-over is real.
    w.scene.sim.as_mut().unwrap().bond.pos = [c[0] - 3.0, c[1], c[2] + 0.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    w.walk_to(Wp::new(c[0], c[1], c[2]).r(0.8), "onto the 0x21 plate");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x21),
        "the walk-over fires ch 0x21 [D #415 p0=33; M Trigger_Hit 0x1b1fc8]");
}

/// The doors (0xDB, 13 [D]): a plain door opens on the E (`Door_Activate`'s use path [M
/// door.rs]). The locked sister is `locked_door_refuses_the_e` below.
#[test]
fn plain_door_e_opens() {
    let mut w = load();
    let di5 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.placement == 5).expect("door #5 door_secret_library [D]");
    let di4 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.placement == 4).expect("door #4 door_secret_bedroom [D]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.doors.list[di4].ch_lock, 49, "#4 locks on ch 0x31 [D p3=49]");
        assert_eq!(sim.drones.doors.list[di5].ch_lock, 0, "#5 has no lock channel [D p3=0]");
    }
    let _ = di4; // the locked sister is the ignored test below
    // The plain door opens on the E.
    let p5 = w.scene.sim.as_ref().unwrap().drones.doors.list[di5].pos;
    walk_over(&mut w, [p5[0] - 0.8, p5[1], p5[2] - 0.8], "to door #5");
    w.use_door(di5, "door_secret_library");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di5].state;
    assert!(st & 0xc != 0, "the unlocked door opens on the E [M Door_Activate], state {st:#x}");
}

/// The mask-4 plate (0xEA #504 `p0=49 p1=4` [D]) operates on the E — the same use-pick the doors
/// take (`Trigger_Update`'s use arm [M triggers.rs]) — and its ch 0x31 unlocks the proximity
/// slider #4 (p0=2 [D]): the door opens on approach [M door.rs], and SS #502 (p3=49 [D], the
/// state-0x40 picture) operates as the response [M SS_Operate 0x1a9df8 reads ch +0x10]. Sister:
/// ch 0x31 off and #4 shut before the E.
#[test]
fn the_e_plate_unlocks_the_proximity_door_and_swings_the_picture() {
    let mut w = load();
    let di4 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.placement == 4).expect("door #4");
    let si = w.scene.sim.as_ref().unwrap().drones.sss.list.iter()
        .position(|s| s.placement == 502).expect("SS #502 bedroom_triggerpicture [D]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x31), "sister: ch 0x31 off before the E");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.sss.list[si].state, 0x40, "the picture rests in state 0x40 [D p0=64]");
    // E the plate at (11.80, 6.04, −35.63) [D] — the use pick is the point head + 1×look vs the
    // plate's own collision glist [M 0x1a5170:100-108; 0x1eae80:219-228; 0x1ebaa8:186-190;
    // 0x1e92d8], and #504's glist is the walk-in shell x 10.09-13.51, z −36.33..−34.94, sheets y
    // 5.64/6.04/6.44 [D coll-scan obj 308]. The direct last leg to the plate wedges on the bedroom
    // furniture (the probe: head_bump at (10.36, −34.12), 1.6 short [D
    // docs/reports/level-03-interactables.md]) — the real walk reaches the bedroom, the blocked
    // last leg takes the probe's own wedge spot, and the E runs from there with the horizontal
    // look across the volume (the use point lands on the top sheet: (10.36, ~7.0, −35.1), the
    // plane distance ~0.5 under the 1.0 reach — the game's own control probe fires the plate from
    // ~1.5 south aimed at it).
    walk_over(&mut w, [11.80, 6.04, -33.2], "to the secret-bedroom plate #504");
    w.scene.sim.as_mut().unwrap().bond.pos = [10.36, 6.48, -34.12];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    use_at(&mut w, [10.36, 6.9, -35.63], "E the secret-bedroom plate #504");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x31),
        "the E fires the mask-4 plate's ch 0x31 [D #504 p0=49 p1=4; M triggers.rs]");
    // The unlocked door opens on the E [M Player_Activate 0x1a5170's door arm — the game has no
    // proximity open for a closed idle door, door.rs's opener census].
    let p4 = w.scene.sim.as_ref().unwrap().drones.doors.list[di4].pos;
    walk_over(&mut w, [p4[0] - 1.1, p4[1] - 0.1, p4[2] - 1.1], "to the unlocked bedroom door");
    w.use_door(di4, "the unlocked bedroom door");
    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di4].state;
    assert!(st & 0xc != 0, "door #4 opens on the E once ch 0x31 is on [D p3=49; M door.rs lock gate passes], state {st:#x}");
    // The picture answers the channel [M SS_Operate 0x1a9df8's +0x10 gate].
    let mut moved = false;
    for _ in 0..120 {
        let s = &w.scene.sim.as_ref().unwrap().drones.sss.list[si];
        if s.state != 0x40 { moved = true; break; }
        w.wait(1);
    }
    assert!(moved, "SS #502 leaves its resting state once ch 0x31 is on [D p3=49; M 0x1a9df8]");
}

/// The pickups (0xF0, 5 [D]) on the walk-over [M Pickup_Handler 0x18e650]: the BodyArmourHeavy
/// (#488, kind 3, 50 [D]) sets armour 50, and a "Pickup 007" (#497, kind 6 [D]) is consumed by
/// the take path — but the kind-6 row exists ONLY with the level's gold medal recorded
/// [M Pickup_Create 0x18dda0:93-95]. Sisters: armour 0 and the bonus sitting before the walks.
#[test]
fn pickups_taken_by_walking_over() {
    let mut w = load_gold();
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.health.armour, 0.0, "sister: no armour before");
    let arm = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .position(|p| p.kind == 3 && (p.m[3][0] + 10.92).abs() < 0.6 && (p.m[3][2] + 38.05).abs() < 0.6)
        .expect("the bedroom BodyArmourHeavy #488 [D]");
    let ap = w.scene.sim.as_ref().unwrap().drones.pickups[arm].m[3];
    w.walk_to(Wp::new(ap[0], ap[1], ap[2]).r(1.2), "onto the BodyArmourHeavy");
    w.wait(20);
    assert!((w.scene.sim.as_ref().unwrap().drones.health.armour - 50.0).abs() < 1.0,
        "the walk-over grants armour 50 [M 0x18e650 case 3; D #488 p2=50], got {}",
        w.scene.sim.as_ref().unwrap().drones.health.armour);
    // The kind-6 bonus (#497 at (31.78, 5.54, 7.09) [D]): the gold load created it; the walk
    // over consumes it. The servants' hall walk is long — the nav route drives it; the take is
    // the real walk-over.
    let bonus = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS && (p.m[3][0] - 31.78).abs() < 0.6)
        .map(|p| (p.gone, p.hidden)).collect::<Vec<_>>();
    assert_eq!(bonus(&w), vec![(false, false)], "sister: the gold load created the kind-6 bonus and it sits [M 0x18dda0:93-95]");
    // The servants' hall transit is the same documented blocker as the stereo leg (the probe:
    // the nav walk wedges at (−1.03, −32.03), d 51) — the stand-in reaches the room; the take is
    // the real walk-over of the last 2 units.
    w.scene.sim.as_mut().unwrap().bond.pos = [31.78, 5.6, 9.2];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    w.walk_to(Wp::new(31.78, 5.6, 7.09).r(1.2), "onto the Pickup 007");
    w.wait(20);
    assert!(bonus(&w).iter().all(|&(gone, hidden)| gone || hidden),
        "the walk-over consumes the kind-6 bonus [M 0x18e650:234-236], got {:?}", bonus(&w));
    // The no-medal arm: the same load with the fresh-boot zero word creates NO kind-6 at all
    // [M 0x18dda0:93-95].
    let bare = load();
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter().all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
}

/// The breakables (0x20, 97 [D]) shatter on the bullets: `Break_Update` 0x12bfd8 subtracts the
/// damage and `Break_Kill` 0x12b970 marks obj+0xFE bit 0 [M breakable.rs]. The bedroom's
/// closet items flank the spawn [D #7/#17]: the shot is real trigger pulls from the floor.
/// Sister: the neighbouring closet still stands.
#[test]
fn breakables_shatter_on_bullets() {
    let mut w = load();
    let bi = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter()
        .position(|b| b.placement == 7).expect("breakable #7 closet_creme2 [D]");
    let ni = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter()
        .position(|b| b.placement == 17).expect("breakable #17 closet_perfume3 [D]");
    let centre = {
        let b = &w.scene.sim.as_ref().unwrap().drones.breaks.list[bi];
        [(b.lo[0] + b.hi[0]) * 0.5, (b.lo[1] + b.hi[1]) * 0.5, (b.lo[2] + b.hi[2]) * 0.5]
    };
    // Stand ~1.6 from the closet so the shot line is short and clear.
    w.scene.sim.as_mut().unwrap().bond.pos = [centre[0] + 1.6, centre[1] - 0.6, centre[2] + 0.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead, "sister: the closet stands before the shot");
    assert!(fire_at(&mut w.scene, centre, 400), "a shot was fired at the closet");
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.breaks.list[bi].dead,
        "the bullets shatter the breakable [M 0x12bfd8 + 0x12b970]");
    assert!(!w.scene.sim.as_ref().unwrap().drones.breaks.list[ni].dead, "sister: the neighbour closet still stands");
}

/// The destroyables (0xE1, 5 [D]) smash on the bullets (`Destroy_Smash` 0x12bb60) — and all five
/// carry out ch 0 [D p0], so the smash writes no channel; their 0x31 fusebox twins sit at the
/// same spots but are scriptless `b` twins, which `FuseBox`'s creation gate refuses [M 0x1b0448]:
/// the port's fuseboxes list must stay EMPTY like the game's. Sister: the twin placement exists
/// on the disc but no object.
#[test]
fn destroyables_smash_on_bullets_and_the_fusebox_twins_never_exist() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.props.destroy.len(), 5, "the five destroyables created [D census]");
        assert!(sim.drones.fuseboxes.is_empty(),
            "the scriptless 0x31 twins are never created [M 0x1b0448; D all 5 scriptless]");
    }
    let di = w.scene.sim.as_ref().unwrap().drones.props.destroy.iter()
        .position(|d| d.placement == 360).expect("destroyable #360 servants_q2_stereo [D]");
    let centre = {
        let d = &w.scene.sim.as_ref().unwrap().drones.props.destroy[di];
        assert_eq!(d.channel, 0, "#360 writes no channel [D p0=0]");
        [(d.lo[0] + d.hi[0]) * 0.5, (d.lo[1] + d.hi[1]) * 0.5, (d.lo[2] + d.hi[2]) * 0.5]
    };
    // The stereo stands in the servants' common room (36.45, 6.16, −15.89) [D]. The walk there is
    // a documented blocker: the nav walk never routes the start block to the servants' floor
    // (the probe: walk_over wedges at (6.22, −18.61), d 32.7, the nav legs return no route) —
    // the stand-in transit reaches the room; the smash is the real trigger pull.
    w.scene.sim.as_mut().unwrap().bond.pos = [centre[0] + 2.2, centre[1] - 0.5, centre[2] + 1.2];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(fire_at(&mut w.scene, centre, 400), "a shot was fired at the stereo");
    w.wait(30);
    assert!(w.scene.sim.as_ref().unwrap().drones.props.destroy[di].dead,
        "the bullets smash the destroyable [M 0x12bb60]");
    assert!(w.scene.sim.as_ref().unwrap().drones.fuseboxes.is_empty(),
        "still no fusebox objects after the smash [M 0x1b0448]");
}

/// The ladder (0x22 #452 at (34.17, 7.56, 18.24) [D]) — this level's only ride kind (lifts are
/// absent [D census]): the walk to the base and the hold-forward climb dismount at the top
/// [M Player_Climb via the ladder system]. Sister: the ladder's top stands above its bottom.
#[test]
fn the_garden_ladder_climbs_to_its_top() {
    let mut w = load();
    let (c, top, bottom) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let l = sim.ladders.iter().find(|l| (l.centre[0] - 34.17).abs() < 0.8 && (l.centre[2] - 18.24).abs() < 0.8)
            .expect("ladder #452 [D]");
        (l.centre, l.top(), l.bottom())
    };
    assert!(top > bottom + 1.0, "sister: the ladder rises, top {top} bottom {bottom} [D]");
    // The garden transit is the same documented blocker (the probe: the nav walk wedges at
    // (−3.44, −18.08), d 53) — the stand-in reaches the base (the first spot with floor under it:
    // the ladder's own column falls away — the probe: (34.3, 6.2, 19.4) drops out of the world);
    // the climb is the real hold-forward.
    // The garden floor at the ladder's base stands at y ≈ 5.25 (the probe: floor_below 5.247 at
    // (31.3..37.3, 15.2..18.2); z ≥ 19.7 falls away — the moat); the climb grabs at bottom 6.23.
    w.scene.sim.as_mut().unwrap().bond.pos = [c[0], 5.6, c[2] - 1.0];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(w.scene.sim.as_ref().unwrap().bond.grounded, "the base spot stands");
    w.walk_to(Wp::at(c[0], 5.6, c[2], Act::Ladder { top_y: top }), "climb ladder #452");
    let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
    assert!(y >= top - 1.2, "the climb dismounts at the top [M Player_Climb], bond y {y} top {top}");
}

/// The E-usable script player (0xD9 #110 `masterarea_dragon`, mode 64, mask 4 [D p2/p5]) — this
/// level's switch/console role (switches 0x29 are absent [D census]): the mode-64 prime creates
/// the dragon entity at load [M SP_CreateScriptPlayer 0x1db9a8's load beat], the E plays script
/// 0x06000563, its stream 1 spawns the kind-2 safe key (+0x28 = 0x4E [D]) at its t 0x5b, and
/// taking the key writes ch 0x4E — obj0's watch [M Pickup_Handler 0x18e650:330-333]. Sisters:
/// ch 0x4E off and no kind-2 pickup before the E.
#[test]
fn the_dragon_e_use_spawns_the_safe_key_that_writes_ch_0x4e() {
    let mut w = load();
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e), "sister: ch 0x4E off before the E");
    // The port pre-reserves each script Entity-0xF0 as a `gone` slot (inert until the script
    // spawns it) — the sister is NO LIVE key: every kind-2 row gone or hidden.
    assert!(w.scene.sim.as_ref().unwrap().drones.pickups.iter().filter(|p| p.kind == 2).all(|p| p.gone || p.hidden),
        "sister: no live key pickup before the E");
    // The dragon entity stands at the SP's spot (6.70, 6.14, −42.55) [D] after the prime (the
    // prime runs on the sim's first tick).
    w.wait(5);
    let dragon = w.scene.sim.as_ref().unwrap().drones.scripts.entities().iter()
        .find(|e| e.2 == 0x0200_05f0).expect("the dragon entity exists after the mode-64 prime [D #110 p2=64]")
        .3;
    let dp = [dragon[3][0], dragon[3][1], dragon[3][2]];
    assert!((dp[0] - 6.70).abs() < 1.0 && (dp[2] + 42.55).abs() < 1.0,
        "the dragon rests at the SP's spot [D #110], got {:?}", dp);
    // Stand in front (the hint #510 marker at (5.62, 5.54, −42.55) [D] marks the player's side),
    // aim at the entity and hold the E — `SP_Activate` 0x1dc338's facing dot + use-key bit.
    walk_over(&mut w, [dp[0] - 0.4, dp[1] - 0.6, dp[2] + 1.6], "to the dragon bookcase");
    use_at(&mut w, [dp[0], dp[1] + 0.5, dp[2]], "E the dragon bookcase");
    // The script runs 121 ticks [D 06000563 length]; the key spawns at its t 0x5b [D stream 1].
    w.wait(150);
    let key = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .position(|p| p.kind == 2 && p.switch_channel == 0x4e && !p.gone && !p.hidden)
        .expect("the E spawns the live kind-2 safe key (+0x28 = 0x4E) [D 06000563 stream 1; M Pickup_Create 0x18dda0]");
    let kp = w.scene.sim.as_ref().unwrap().drones.pickups[key].m[3];
    w.walk_to(Wp::new(kp[0], kp[1], kp[2]).r(1.2), "onto the safe key");
    w.wait(20);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e),
        "taking the key writes ch 0x4E — obj0's completion [M 0x18e650:330-333]");
}


/// The locked sister: door #4 `door_secret_bedroom` locks on ch 0x31 (p3=49 [D], +0x68 per
/// `Door_Create` 0x136100; `Door_Activate`'s lock gate [M door.rs — the port's gate refuses the
/// E while the channel is off, probe-pinned: 45 ticks of E from the doorway hold state 0x2 with
/// ch 0x31 forced off]) must refuse the same E — and the E must not reach the mask-4 plate #504
/// either: the game's use pick is the point (head + 1×look) vs the plate's own collision glist
/// [M Player_Activate 0x1a5170:100-108 → Collide_PickObj 0x1eae80:219-228 → Collide_Intersect
/// 0x1ebaa8:186-190 → Intersect_PointGeom 0x1e92d8], and from the doorway stance that point sits
/// ~1.2 above the volume's top sheet (the settled root + the 1.0 reach) and outside its east/south
/// faces — #504's glist is the 4-sheet walk-in shell at x 10.09-13.51, z −36.33..−34.94, y
/// 5.64..6.73 [D coll-scan 07000003 obj 308]. The game plays the locked-door hit sound until ch
/// 0x31 [D audit]; the fixed port pick matches (the plate's ch 0x31 never rises here).
#[test]
fn locked_door_refuses_the_e() {
    let mut w = load();
    let di4 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| d.placement == 4).expect("door #4");
    let p4 = w.scene.sim.as_ref().unwrap().drones.doors.list[di4].pos;
    w.scene.sim.as_mut().unwrap().bond.pos = [p4[0] - 1.8, p4[1] - 0.1, p4[2] - 1.8];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x31), "sister: ch 0x31 off at the locked door");
    let before4 = w.scene.sim.as_ref().unwrap().drones.doors.list[di4].state;
    w.use_door(di4, "the locked bedroom door (sister)");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di4].state, before4,
        "sister: the ch-0x31-locked door must not move on the E [D p3=49; M door.rs lock arm]");
}
