//! level-16 interactables: one live test per interactive KIND on 07000016 (the missile base,
//! launch centre) through the real input path (`nf_view::load` + the Walker; teleports only to
//! REACH a spot — every operation itself is the real input). Evidence tags in every assert;
//! docs/levels/07000016.md §7 is the census (707 placements/39 kinds, main map m63).
#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    // A fresh Bond with only the carried gates: the save Trigger_Activate would make [M 0x1b2318:97-116].
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn load16() -> Walker {
    // The arrival from 07000015 carries the 07000014/15 gate chain [D — the walkreal_07000016
    // pattern]: 0x47/0x48/0x4a/0x54/0xce/0x4b.
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).unwrap());
    assert!(w.wait_enabled(3000, "the launch-centre arrival"), "the intro releases Bond");
    w.wait(200);
    w
}

/// Teleport-reach + the real E: aim the LOOK at `at` and press use on an edge until `done`.
/// The use dispatch is the game's `Player_Activate` 0x1a5170 (the E path).
fn press_use_until(w: &mut Walker, at: [f32; 3], label: &str, max: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) -> bool {
    for k in 0..max {
        if done(w.scene.sim.as_ref().unwrap()) { return true; }
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (at[0] - sim.bond.pos[0]).atan2(at[2] - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let to = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
        let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.08, 0.08),
            look: (want_pitch - sim.bond.pitch).clamp(-0.06, 0.06),
            ..Default::default()
        };
        w.tick(input, k % 6 == 0); // one edge per 6 ticks (the use_door rhythm)
    }
    eprintln!("[16] {label}: the E never landed in {max} ticks");
    false
}

/// THE FINALE through the game's own writer: the KikoDeathButton #693 (7.02, 1.16, −155.61) is
/// the level's kind-0x29 switch — param 0 = ch 0x90, param 5 = 0x8d the gate [D]. Bond
/// teleport-reaches the tower room (the corridor touch #616 that arms ch 0x8d is deliberately
/// NOT crossed — the gate INHIBITS the switch while its channel is on [M Switch_Activate
/// 0x1b02a8:22]), presses E, and the chain runs: ch 0x90 → mux #618 → ch 100 → MissionState 6 →
/// the driving hand-off 0x7300017 [D placements; M 0x182c60:111-112].
#[test]
fn the_kiko_death_button_completes_the_final_mission() {
    let mut w = load16();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x90), "sister: ch 0x90 off before the press [D]");
        assert!(!sim.drones.channels.on(0x8d), "sister: the gate ch 0x8d is off on the reach (touch #616 not crossed) — the switch is usable [M 0x1b02a8:22]");
        assert!(sim.drones.mission.status() != 6, "sister: the mission is not yet complete");
    }
    // Reach: the tower room floor by the button (its placement y 1.16 [D #693]).
    w.scene.sim.as_mut().unwrap().bond.pos = [7.02, 1.2, -154.0];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let pressed = press_use_until(&mut w, [7.02, 1.46, -155.61], "the KikoDeathButton", 180,
        |sim| sim.drones.channels.on(0x90));
    assert!(pressed, "the E on the KikoDeathButton never wrote ch 0x90 — Switch_Activate 0x1b02a8 must toggle param-0 [D #693 p0 = 0x90]");
    // ch 0x90 → mux #618 (out 0x64 = ch 100 ← 0x90) → Mission_Update state 6 [D #618; M 0x1827d8:155-160].
    let mut complete = false;
    for _ in 0..900u32 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission.status() == 6 { complete = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(100), "mux #618 (out 0x64 = ch 100 ← 0x90) fires [D #618]");
        assert!(complete, "the final mission completes (MissionState 6 [M 0x182c60:85-116])");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0017, "the driving hand-off [M 0x182c60:111-112]");
    }
}

/// The locked door + its mask-0x04 activate trigger, both through the real path: door[0] #351
/// (1.84, −0.10, −114.23, ch_lock 0x24 [D]) refuses Bond's E while ch 0x24 is off; the writer is
/// the SecurityStation activate trigger #597 (−15.20, 6.52, −94.56, mask 0x4 [D]) in the west
/// security room — walkreal-c's door[0] blocker answer. The E there raises 0x24, and the lock's
/// rising edge lets the same E open door[0] [M Door_Update's lock path].
#[test]
fn door0_unlocks_from_the_security_station_and_opens() {
    let mut w = load16();
    // Reach the door's north approach; the station E is not pressed yet.
    w.scene.sim.as_mut().unwrap().bond.pos = [2.3, 0.0, -112.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let (_, di) = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::door_near(sim, [1.84, -0.1, -114.23]).expect("door[0] #351 in reach")
    };
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x24), "sister: ch 0x24 off before the station [D]");
    w.use_door(di, "door[0] while locked");
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0xc, 0,
        "sister: the E refuses door[0] while ch 0x24 is off [D #351 p3 = 0x24; M Door_Update's lock path]");
    // The SecurityStation E (the activate-trigger kind): teleport-reach the west security room's
    // upper floor and press E at the trigger's centre.
    let centre = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH_ONCE, 0x24)
        .expect("the SecurityStation activate trigger #597 [D]");
    w.scene.sim.as_mut().unwrap().bond.pos = [centre[0], centre[1] - 0.6, centre[2] + 1.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let pressed = press_use_until(&mut w, centre, "the SecurityStation", 180,
        |sim| sim.drones.channels.on(0x24));
    assert!(pressed, "the E on the SecurityStation never wrote ch 0x24 — the mask-0x04 activate trigger [D #597; M Trigger_Activate 0x1b2318]");
    // The lock's rising edge unlocks: the same E now opens door[0].
    w.scene.sim.as_mut().unwrap().bond.pos = [2.3, 0.0, -112.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    w.use_door(di, "door[0] unlocked");
    for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.doors.list[di];
    assert!(d.state == nf_game::door::OPEN || d.state == nf_game::door::OPENING,
        "door[0] opens after the station E (state {:#x}, pos01 {:.3})", d.state, d.pos01);
    assert!(d.pos01 > 0.5, "the leaf travelled (pos01 {:.3})", d.pos01);
}

/// The pickups take on the walk-over: the grenade cluster #701–705 (0.4–0.8, 0.64,
/// −132.5..−133.7 [D]) sits in the door[1]–door[3] corridor on the walked route.
/// [M Pickup_Handler 0x18e650 — the take consumes the pickup.]
#[test]
fn the_corridor_pickup_takes_on_the_walk() {
    let mut w = load16();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [0.6, 0.64, -133.1]) < 1.2),
            "sister: the grenade cluster sits in the corridor [D #701-705]");
    }
    // Reach the corridor north of the cluster (past door[0]/door[1]'s line), then the real walk
    // through it.
    w.scene.sim.as_mut().unwrap().bond.pos = [0.6, 0.0, -134.6];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    w.walk_to(Wp::new(0.6, 0.0, -132.2).r(0.9), "through the grenade cluster");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [0.6, 0.64, -133.1]) < 1.2),
        "the walk-over takes the grenade [M Pickup_Handler 0x18e650]");
}

/// The destroyable: the Fire-extinguisher #508 (−13.69, 7.13, −86.99, kind 6, blast damage
/// 0xc8 [D]) smashes under Bond's fire — the bullet's damage pool drains the tag's health
/// [M Destroy_CollisionHandler 0x12bd48; Destroy_Create 0x12c318].
#[test]
fn the_fire_extinguisher_smashes_under_fire() {
    let mut w = load16();
    let idx = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.destroy.iter().position(|d| d.lo[0] < -13.69 && d.hi[0] > -13.69 && d.lo[2] < -86.99 && d.hi[2] > -86.99)
            .expect("the security-room extinguisher's bounds [D #508]")
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.props.destroy[idx].dead, "sister: the extinguisher stands [D]");
        assert!(sim.drones.channels.on(0x8e), "the spawn plate fired on the arrival [D #599]");
    }
    // Reach the security room floor below the mount; the shots are the real fire.
    w.scene.sim.as_mut().unwrap().bond.pos = [-13.69, 6.4, -85.3];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let smashed = w.shoot_point([-13.69, 7.13, -86.99], "the extinguisher",
        |sim| sim.drones.props.destroy[idx].dead);
    assert!(smashed, "the extinguisher never smashed — the bullet damage pool must drain the tag [M 0x12bd48]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.props.destroy[idx].health <= 0.0, "the tag's health exhausted [M 0x12c318:36]");
}

/// The shuttle-bay hurt volumes are CONFIGURED-INERT on this level — the game's own answer to the
/// task-767/U-051 item: HurtPoly-LiftCrush #699 (2.47, −22.25, −159.91) carries who mask 0x1
/// (Hurt_Update's contact-class AND matches nothing — the configured-inert family [M 0x17e5d0])
/// and gate ch 0xcb, which is low at load (the disabled 0x20 bit sets at create [M 0x17e870
/// tail]). Standing in the volume hurts nothing; the disc params pin the rest.
#[test]
fn the_shuttle_bay_hurt_volume_is_configured_inert() {
    let mut w = load16();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The LiftCrush sits at z −159.91; its sister ShuttleLaunch #656 at z −164.17 [D] —
        // separate the pair by the volume's centre z.
        let h = sim.hurts.iter().find(|h| (h.lo[2] + h.hi[2]) * 0.5 > -161.5 && (h.lo[2] + h.hi[2]) * 0.5 < -158.0)
            .expect("the LiftCrush hurt volume [D #699]");
        assert_eq!(h.kind, 4, "the LiftCrush pain kind [D #699 p0 = 0x4]");
        assert!((h.damage - 1000.0).abs() < 0.01, "the disc damage 0x3e8 [D #699 p1]");
        assert_eq!(h.gate, 0xcb, "the gate channel [D #699 p2 = 0xcb]");
        assert_eq!(h.who, 1, "the who mask 1 — Hurt_Update's class AND collects nothing [D #699 p4; M 0x17e5d0:57-95]");
        assert!(!sim.drones.death.active, "sister: Bond alive before the bay");
        assert_eq!(sim.drones.health.health, 100.0, "sister: full health before the bay");
    }
    // Reach: into the volume's airspace (the gate ch 0xcb is low — the volume sits disabled).
    w.scene.sim.as_mut().unwrap().bond.pos = [2.47, -20.6, -159.91];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    for _ in 0..240 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.health.health, 100.0,
        "standing/falling in the LiftCrush volume hurts nothing — who mask 1 [M 0x17e5d0:57-95]");
    assert!(!sim.drones.death.active, "no death in the bay volume [M]");
}

/// The trench ladder: Ladder1 #653 (−0.43, −6.21, −86.21 [D]) on the corridor trench's west wall
/// takes the real climb hold — Bond's walk into it engages the climb substate and he rises
/// [M Player_CollLadder].
#[test]
fn the_trench_ladder_takes_the_climb() {
    let mut w = load16();
    // Reach: the trench floor by the ladder's base (the corridor dips to −4.8 across x ≤ 7 at
    // z −88..−102 [D — the walkreal trench bypass]; the base sits deeper on the west wall).
    w.scene.sim.as_mut().unwrap().bond.pos = [0.5, -5.9, -86.21];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.substate != nf_game::player::SS_CLIMB, "sister: not climbing before the hold");
        assert!(walker::ladders(sim).iter().any(|(c, _, _)| walker::dxz(*c, [-0.43, -6.21, -86.21]) < 2.0),
            "the trench ladder exists [D #653]");
    }
    // Face west into the wall (the ladder at x −0.43) and walk in.
    w.scene.sim.as_mut().unwrap().bond.yaw = -std::f32::consts::FRAC_PI_2;
    let mut climbing = false;
    for _ in 0..240 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbing = true; break; }
        let input = nf_game::player::Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    assert!(climbing, "the walk into the trench ladder never engaged the climb [M Player_CollLadder]");
}

/// The spawner kind: spawner #601 (6.44, 0.00, −77.90) carries group 1 with start ch 0x8e — the
/// spawn plate's out [D #601 p4/p1; #599 p0] — so the arrival creates the corridor squad there.
/// [M DroneSpawner_Control 0x137e68 waits on the start channel and spawns the group's drones.]
#[test]
fn the_spawn_plate_spawns_group_one_at_its_spawner() {
    let mut w = load16();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x8e), "the spawn plate #599 fired on the arrival [D]");
        assert!(sim.drones.channels.on(0x8f) == false, "sister: the silo plate's 0x8f has not fired down here [D #600]");
    }
    // A present, live drone at the SPAWNER's own spot — no static placement carries that
    // position [D census: the group-1 statics sit at the corridor posts (2.12, 6.79, −81.12)
    // etc.; (6.44, 0.00, −77.90) is #601's own] — so a drone standing there is spawner output.
    let mut found = None;
    for _ in 0..300u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        found = sim.drones.list.iter().position(|d| d.present && !d.corpse && walker::dxz(d.pos, [6.44, 0.0, -77.90]) < 4.0);
        if found.is_some() { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert!(found.is_some(), "DroneSpawner_Control never created the group-1 squad at spawner #601 [M 0x137e68; D #601 start 0x8e]");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[found.unwrap()];
    assert!(matches!(d.skin & 0xffff, 0x87 | 0x8c | 0xb1),
        "the spawned squad wears the level's guard skins [D census p0 skins]");
}
