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

/// Every placement's model resolves on this level — zero missing hashes/indices of the 707
/// placements across all 64 maps [D parse].
#[test]
fn every_placement_resolves_its_model() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let mut unresolved = 0;
    for m in &lv.maps {
        for p in &m.placements {
            let ok = if p.by_hash() { lv.by_hash.contains_key(&p.hashcode) }
                else { (p.index as usize) < m.objects.len() };
            if !ok { unresolved += 1; }
        }
    }
    assert_eq!(unresolved, 0, "every placement's model resolves [D the 07000016 parse]");
}

/// The player-triggered SimpleScript kind: the four Door_corridor_graphic ScriptPlayers
/// (#431/#432/#501/#502 — kind 0xd9, script 0x06000999, mode 0x40, mask 1, out = the door's
/// lock channel 0x27/0x28 [D census #431-432]) START on the player's panel E. The writer is the
/// mask-0x04 activate panel #684 (3.36, 0.84, −141.50, out 0x27 [D]) beside door[3] #364
/// (1.83, −0.10, −141.88, lock p3 = 0x27 [D]): the E raises ch 0x27, and `SP_Update` 0x1dbda8's
/// state-0 arm (mask 1 = the out-channel bit, `SP_Hit` 0x1dc470:16-18 feeds `switch_channels[p6]`)
/// loads + starts each graphic SP's script [M 0x1dbda8:64-106 → load]. The same edge unlocks
/// door[3], which the E then opens. Sister: ch 0x27 off and the 0x27 graphic SPs rest at create
/// state 0, script unloaded, before the E.
#[test]
fn the_door_panel_e_starts_the_corridor_graphic_script_players() {
    let mut w = load16();
    let graphic27 = |sim: &nf_view::scene::Sim| sim.drones.scripts.players.iter()
        .filter(|p| p.scripts[0] == 0x0600_0999 && p.out == 0x27)
        .map(|p| (p.placement, p.state, p.info[0].is_some(), p.live_matrix[3])).collect::<Vec<_>>();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(graphic27(sim).len(), 2, "the two ch-0x27 graphic SPs exist [D #432/#501 — the port indexes SP placements by the main map: #85/#154 = the census's #432/#501]");
        assert!(!sim.drones.channels.on(0x27), "sister: ch 0x27 off before the panel E [D]");
        for (pl, st, loaded, _) in graphic27(sim) {
            assert_eq!(st, 1, "sister: SP #{pl} rests at the create load beat's state 1 — mode 0x40 ∉ {{0,1}} loads + stops at create [M 0x1db9a8:81-86]");
            assert!(loaded, "sister: the create beat loaded the script, stopped [M 0x1db9a8:81-86]");
        }
    }
    // Reach door[3]'s north-east approach (the real route's own side: walkreal presses the panel
    // from z −140) and press E on the panel #684 until its ch 0x27 lands.
    w.scene.sim.as_mut().unwrap().bond.pos = [3.4, 0.0, -140.2];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let pressed = press_use_until(&mut w, [3.36, 0.84, -141.50], "door[3]'s panel #684", 180,
        |sim| sim.drones.channels.on(0x27));
    assert!(pressed, "the E on the panel never wrote ch 0x27 — the mask-0x04 activate trigger [D #684; M Trigger_Activate 0x1b2318]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for (pl, st, loaded, live) in graphic27(sim) {
            assert!(loaded, "the panel E keeps graphic SP #{pl}'s script 0x0600999 loaded [M SP_Update 0x1dbda8 case 1 → start]");
            assert_eq!(st, 2, "the panel E started graphic SP #{pl} (create state 1 → playing state 2 while ch 0x27 holds — the arm-1 start) [M 0x1dbda8:64-106 case 1]");
            eprintln!("[16] graphic SP #{pl} state {st} live ({:.2},{:.2},{:.2})", live[0], live[1], live[2]);
        }
    }
    // The same channel unlocks door[3]: the E now opens it (the door kind's lock consumer,
    // walkreal-12's two-latch fix).
    let (_, di) = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::door_near(sim, [1.83, -0.1, -141.88]).expect("door[3] #364 in reach [D]")
    };
    w.use_door(di, "door[3] unlocked by the panel");
    for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.doors.list[di];
    assert!(d.state == nf_game::door::OPEN || d.state == nf_game::door::OPENING,
        "door[3] opens after its panel E (state {:#x}, pos01 {:.3})", d.state, d.pos01);
    assert!(d.pos01 > 0.5, "the leaf travelled (pos01 {:.3})", d.pos01);
}

/// The lift kind: the shuttle-bay lift (ScriptPlayer_ShuttleBayLift #694, kind 0xd9 mode 1,
/// script 0x060009e9, out ch 0x50 [D #694 (2.48, 0.00, −159.93)]) RIDES on the launch chain. The
/// mode-1 load beat starts the script at create, held [M 0x1db9a8/0x1dbda8 mode 1 → load]; the
/// launch cinema's stream-21 event writes [ch 80, 1] [D 0600090a s21 — the dump]; the state-2
/// `SP_UnPause` (mask 1, ch 0x50 on) releases the hold [M 0x1dbda8 case 2]; the cab entities
/// (stream 1 hash 0x02000a79 / stream 2 hash 0x02000a7a [D the dump]) follow the descend keys —
/// stream 1 carries y +1.60 at t0 down to −21.22 at t549, the 22.8-unit ride to the bay floor
/// [D 060009e9 s1 keys — the dump]; `Script_Interp` keys the stream objects every frame [M
/// 0x1d9a08:106-133]; and the lift script's own stream 7 event writes [80, 0] [D 060009e9 s7 —
/// the dump], landing +181 after the summon in the port. Driven through the real path: the
/// walked touch of the launch-room plate #602 (2.37, 1.04, −176.41, out 0x81, mask 8 [D]) starts
/// the chain (SP #609 → 0xb9 → SP #619). Sisters: ch 0x50 cold, the cab at its top key, and the
/// bay drone #634 (skin 0x500005e, wake p3 = 0x50 [D #634]) dormant at its perch.
#[test]
fn the_launch_chain_rides_the_shuttle_bay_lift() {
    let mut w = load16();
    let lift_i = |sim: &nf_view::scene::Sim| sim.drones.scripts.players.iter()
        .position(|p| p.scripts[0] == 0x0600_09e9).expect("the bay-lift SP #694 [D]");
    let cab_y = |sim: &nf_view::scene::Sim| sim.drones.scripts.entity_matrix((0x0600_09e9, 1))
        .map(|m| m[3][1]).expect("the lift cab entity (stream 1) [D 060009e9 s1]");
    let bay_drone = |sim: &nf_view::scene::Sim| sim.drones.list.iter()
        .find(|d| d.skin & 0xffff == 0x5e && walker::dxz(d.pos, [2.45, 1.08, -160.0]) < 2.0)
        .map(|d| (d.state, d.pos));
    let (mut rest_y, mut drone0) = (0.0f32, None);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players[lift_i(sim)].info[0].is_some(),
            "the lift's script loaded at create [M 0x1db9a8 mode-1 load beat]");
        assert!(!sim.drones.channels.on(0x50), "sister: ch 0x50 cold before the chain [D — the only writers are 0600090a s21 [80,1] and 060009e9 s7 [80,0]]");
        rest_y = cab_y(sim);
        assert!((rest_y - 1.60).abs() < 0.5,
            "sister: the cab rests at its top key (y {rest_y:.2}) [D 060009e9 s1 t0 = 1.60]");
        drone0 = bay_drone(sim);
        let (st, pos) = drone0.expect("the bay drone #634 exists [D]");
        assert!(st == 1, "sister: the bay drone #634 dormant at its perch (state {st}) [D #634 p3 = 0x50, unfired]");
        eprintln!("[16] bay drone before: state {st} y {:.2}", pos[1]);
    }
    // The real walk into the launch-room plate (skip off — the real-time order the launch cinema
    // needs; the walkreal real-order leg is the proven pattern).
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [2.4, 1.0, -172.0];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI;
    }
    let mut t81 = None;
    for _ in 0..900u32 {
        if t81.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(0x81) { t81 = Some(w.total_ticks); }
        w.scene.sim.as_mut().unwrap().bond.yaw = std::f32::consts::PI;
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.disabled || sim.bond.pos[1] < -5.0 { break; }
    }
    assert!(t81.is_some(), "the walked touch of the launch-room plate never fired ch 0x81 [D #602 mask 8; M Trigger_TouchOnce 0x1b2b88]");
    // The chain plays out: watch ch 0x50 rise, the cab descend, then the lift script's own
    // [80, 0] clear.
    let (mut t50_on, mut t50_off, mut y_min) = (None, None, rest_y);
    for _ in 0..9000u32 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        y_min = y_min.min(cab_y(sim));
        if t50_on.is_none() && sim.drones.channels.on(0x50) { t50_on = Some(w.total_ticks); }
        if let Some(on) = t50_on {
            if t50_off.is_none() && w.total_ticks > on + 10 && !sim.drones.channels.on(0x50) {
                t50_off = Some(w.total_ticks);
                break;
            }
        }
    }
    let drop = rest_y - y_min;
    eprintln!("[16] lift: ch0x50 on at {t50_on:?}, off at {t50_off:?}, cab drop {drop:.2} (rest {rest_y:.2} → min {y_min:.2})");
    assert!(t50_on.is_some(), "the launch chain never wrote ch 0x50 — the lift summon [D 0600090a s21 event [80,1]; M Script_Update's event writer]");
    assert!(drop > 3.0, "the lift cab never descended (drop {drop:.2}) — the un-held script must key the cab down its 22.8-unit travel [M Script_Interp 0x1d9a08:106-133; D 060009e9 s1 keys 1.60→−21.22]");
    assert!(t50_off.is_some(), "the lift script never wrote its [80, 0] [D 060009e9 s7]");
    // The ride's far end: the descend keys run to the bay floor (t540-549 y −21.22 [D s1]); the
    // port's cab bottoms at −20.46 (the keys' asymptote / the script's loop return at t548) —
    // the ride's observed minimum pins the bay-floor arrival.
    let mut y_bot = 0.0f32;
    for k in 0..1200u32 {
        w.tick(nf_game::player::Input::default(), false);
        let y = cab_y(w.scene.sim.as_ref().unwrap());
        y_bot = y_bot.min(y);
        if k == 0 || k % 100 == 0 { eprintln!("[16] cab y at clear+{k}: {y:.2}"); }
    }
    assert!(y_bot < -20.0,
        "the cab's ride must reach the bay floor (the descend keys' −21.22 approach [D 060009e9 s1 t540-549]), min {y_bot:.2}");
    let (st, pos) = bay_drone(w.scene.sim.as_ref().unwrap()).expect("the bay drone #634 after the chain [D]");
    assert!(st != 1 && pos[1] < drone0.map(|(_, p)| p[1]).unwrap_or(0.74) - 1.0,
        "the bay drone #634 never woke/rode the cab down (state {st}, y {:.2}) [D #634 p3 = 0x50; M the drone wake + the cab keys]", pos[1]);
    eprintln!("[16] bay drone after: state {st} y {:.2}", pos[1]);
}

/// Kiko #612 (skin 0x500005e, voice 0x13, wake 0x1b [D]) cannot be hurt here: DefaultInit sets
/// `Drone+0x1B` for voice 0x13 on 0x07000016 [M 0x14b300:699-700] and HandleImpact returns at once
/// on it [M 0x1469b8:23-25]. Bond's own rounds land on her and she keeps walking her route to the
/// tripwire #613, which fails the mission [D #613; M 0x1827d8:108-116]. A shot guard is the sister.
#[test]
fn kiko_takes_no_damage_and_still_reaches_the_tripwire() {
    // A cold load: the fresh kit puts the Delta Repeater in hand [M 0x1a7ef8 case 0x7000016].
    let mut w = Walker::new(nf_view::load(&iso(), "07000016.bin", false).unwrap());
    assert!(w.wait_enabled(3000, "the launch-centre arrival"), "the intro releases Bond");
    w.wait(200);
    let kiko = {
        let sim = w.scene.sim.as_mut().unwrap();
        let i = sim.drones.list.iter().position(|d| d.skin == 0x0500_005e && d.switch_channel == 0x1b).expect("Kiko #612 [D]");
        let hp = sim.drones.list[i].health;
        sim.bond.pos = [14.0, 0.9, -154.9];
        sim.bond.vel = [0.0; 3];
        let frame = sim.drones.frame;
        sim.drones.channels.set(0x1b, 1, frame);
        (i, hp)
    };
    let (i, hp) = kiko;
    let mut landed = false;
    for t in 0..900u32 {
        let (input, shoot) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[i];
            let part = walker::DamageLog::exposed_part(sim, i);
            let at = part.unwrap_or([d.pos[0], d.pos[1] + 0.6, d.pos[2]]);
            let b = sim.bond.pos;
            let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
            let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
            let err = walker::wrap_pi(dx.atan2(dz) - sim.bond.yaw);
            let perr = (dy / dx.hypot(dz).max(0.5)).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
            (nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
             part.is_some() && err.abs() < 0.03 && perr.abs() < 0.04 && t % 8 == 0)
        };
        if shoot { w.tick_fire(input); } else { w.tick(input, false); }
        let sim = w.scene.sim.as_ref().unwrap();
        landed |= sim.drones.list[i].hit.is_some();
        assert_eq!(sim.drones.list[i].health, hp, "Kiko lost health at tick {t} [M 0x1469b8:23-25]");
        if sim.drones.channels.on(0x7b) { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(landed, "no round reached Kiko, so the test proved nothing");
    assert!(sim.drones.channels.on(0x7b), "Kiko never reached the tripwire #613 (at {:?}, state {:#x})", sim.drones.list[i].pos, sim.drones.list[i].state);
}
