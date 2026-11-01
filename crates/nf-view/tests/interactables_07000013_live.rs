//! level-13 interactables: one live test per interactive KIND on 07000013 (HT Tower2 C) through
//! the real input path (`nf_view::load` + the Walker; no channel stand-ins except where noted).
//! Evidence tags in every assert; docs/levels/07000013.md §7 is the census.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The spawn touches + the intro SP: the two spawn TouchOnces fire (0x28, the 0x4A enable), the
/// mission announces the entrance objective, and the intro ScriptPlayer #65 (0x060006D7, mode 1 →
/// state 7, plays at load [M 0x1db9a8's mode-1 arm]) releases Bond.
#[test]
fn spawn_touches_fire_and_the_intro_releases() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    assert!(w.wait_enabled(3000, "the 13 intro"), "the intro releases Bond");
    w.wait(120);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x28), "the spawn touch #58 writes ch 0x28 [D placement params]");
    assert!(sim.drones.channels.on(0x4a), "the spawn touch #153 writes ch 0x4A [D]");
    assert_eq!(sim.drones.mission.states[18][0], 3,
        "'Raise security gate and escape through front entrance' announces on the 0x4A enable [L]");
    // Sister (fails without the spawn): the mid-lobby 0x56 touches have NOT fired standing here.
    assert!(!sim.drones.channels.on(0x56), "the lobby's 0x56 touches stay off at the spawn [D]");
}

/// The mid-lobby TouchOnce pair (#140/#141, out 0x56, at the desk block's west/east lanes [D])
/// fires on the REAL walk past the central desks.
#[test]
fn the_mid_lobby_touches_write_0x56_on_the_walk() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x56),
        "sister: ch 0x56 off before the walk [D]");
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(5.0, -368.4, 27.2).r(0.8), "13: into touch #141");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x56),
        "walking the east lane fires touch #141 (out 0x56) [D placement params; volume (4.18,−368.89,26.45)..(6.90,−367.91,27.50)]");
}

/// The pickups take on the walk-over: the Spas12 (#429) and the BodyArmourLight (#414) — kind
/// 0xF0, `Pickup_Create` 0x18dda0 placements. The route is the game's real one (the walkreal
/// route): the lobby's west "furniture maze" blocks every direct lane to the security room (the
/// fake-lift doors' baked mesh x −14.6..−12.9 / z 31.3..32.9 inside the glass band
/// x −18.4..−10.6 / z 27.5..35.3, plus the room's own east wall [D stuck dumps]), so the walked
/// approach rounds the desk block's north side to the unlocked north-west door #355
/// (−26.65, −368.36, 37.44 [D]), whose E opens it ([M Door_Activate 0x135be0's toggle]; the level
/// doc §7's old "proximity arm" prose misread `Door_Update` 0x1363c8's open-door-only scan — the
/// door test in this file is the correction) and walks the room south to the shelves.
///
/// Sister note: the old "the east-lane 0x56 touch stayed off" sister is gone. The disc's two
/// 0x56 TouchOnces (#140/#141) sit at the desk block's two lanes — volumes x −8.33..−5.61 and
/// 4.18..6.90, both z 26.25..27.50 [D trigger dump] — and every walked route to the security
/// room crosses one (the direct diagonal measured 0x56 ON; the west gap lane clipped #140's
/// west edge; the east strip's mouth is solid). A player walking to the security room fires the
/// mid-lobby touch; the assert was an artifact of the wedged walker this task un-wedged.
#[test]
fn the_pickups_take_on_the_walk() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let spas = sim.drones.inventory.owned[0x1c]
            || sim.gun.weapon == 0x1c;
        assert!(!spas, "sister: the Spas12 is not owned before the walk");
        assert!(!sim.drones.channels.on(0x56), "sister: the mid-lobby 0x56 touch idles at the spawn [D]");
    }
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "13: E the security-room door");
    w.wait(120); // the sliding door finishes [probe: state 0x8 → 0x4 within the wait]
    w.walk_to(Wp::new(-27.0, -368.4, 37.0).r(1.0), "13: through the doorway");
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "13: inside the security room");
    w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
    // The security room's south shelf: the BodyArmourLight (−27.66, −368.51, 25.57) and the Spas12
    // (−20.48, −368.86, 25.38) sit on the room's south strip [D].
    w.walk_to(Wp::new(-20.48, -368.4, 25.38).r(1.2), "13: the Spas12 shelf");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let owned = sim.drones.inventory.owned[0x1c] || sim.gun.weapon == 0x1c;
        assert!(owned, "the Spas12 pickup (#429) takes on the walk-over [M Pickup_Handler 0x18e650]");
    }
    w.walk_to(Wp::new(-27.66, -368.4, 25.57).r(1.2), "13: the BodyArmour shelf");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The armour take is the BLData armour float, not an inventory bit: `Pickup_Handler` case
        // 3 refuses while +0x8B0 >= 50, else sets +0x8B0 = 50.0 [M 0x18e650:220-225] — the old
        // `owned[0x32]` assert read a slot the take never writes (0x32 is the Frinesi's weapon
        // id). The pickup itself is consumed on the take [M 0x18e650:338-348].
        assert!(sim.drones.health.armour >= 50.0,
            "the BodyArmourLight pickup (#414) takes on the walk-over — the armour float to 50 [M 0x18e650:220-225; got {:.1}]", sim.drones.health.armour);
        let taken = sim.drones.pickups.iter()
            .any(|p| walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], [-27.66, -368.51, 25.57]) < 1.5 && p.gone);
        assert!(taken, "the BodyArmourLight pickup is consumed by the take [M 0x18e650:338-348]");
    }
}

/// The pillar glass (kind 0x20, `Break_Create` 0x12c230) shatters under the cold load's held
/// weapon (id 0x10, the P2K — word 0x0008, contact damage 10.0 [L ee_start_05.bin weapon_data
/// @0x2bf150 + 0x10·0x10C: +0x10 = 0x0008; the sim's own record read agrees]). The disc's
/// pillarglass placements carry NO param 2 (#373–#376 "pillarglass(1)" params [0:0x2, 1:0x6]
/// [D census] — param 1 0x6 is the HEALTH 6.0, not a filter: the old "param 2 = 0x6 → the
/// exclude filter !6" note misread it) → tag+0x08 = 0 [M Break_Create 0x12c230's param-2==0
/// arm], so the filter mask is 0 | 0x106 = 0x106 [M Break_Update 0x12bfd8:15] and the round's
/// word ANDs it to zero → the contact reaches the damage walk [M Collide_FilterBullets
/// 0x1ece30:24-33]. 10.0 against health 6.0: the pane dies on the first round [M 0x12bfd8:44-48
/// + Break_Kill 0x12b970].
///
/// The stand: north of the pillar — the glass box itself (the four sheets x 6.46..8.02 at
/// z 35.96/37.53, y −369.25..−354.12 [D parse]) is solid to the capsule, so the old stand
/// (7.9, −368.4, 36.72) sat inside the enclosure and the leg never arrived (the 6000-tick
/// wedge); the walked spot is just north of the north sheet.
///
/// The fire is PULSED: the P2K is a per-pull semi-auto — shots_per_pull 1 in every mode
/// (weapon_data+0x28+2·mode [L ee_start_05.bin @0x2bf150 + 0x10·0x10C]; the port's record read
/// [M GunParams]) latches on the press edge (`pressed && !held` [M the pull latch]), so a HELD
/// trigger fired exactly once (clip 7→6, then no refire for 900 ticks — the old loop's own
/// silence) and the real player taps fire.
#[test]
fn the_pillarglass_shatters_under_fire() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    let brk = {
        let sim = w.scene.sim.as_ref().unwrap();
        // The east desk pair's first panel (#373 at (6.46,−361.69,36.72) [D]) — its north sheet
        // (sim bounds lo (6.59,−369.25,37.53) hi (7.86,−354.12,37.53) [D parse]).
        (0, sim.drones.channels.on(0x56))
    };
    assert!(!brk.1, "sister baseline: the lobby touches idle");
    // Stand north of the panel's north face and fire up at a face point of that sheet.
    let glass_at = [7.24f32, -365.0, 37.53]; // the north sheet's face point, 3.9 above the floor
    w.walk_to(Wp::new(6.72, -368.4, 38.6).r(0.9), "13: north of the east pillarglass");
    let start = w.total_ticks;
    let mut loop_t = 0u32;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let dead = sim.drones.breaks.list.iter().any(|b| b.dead && (b.lo[0] - glass_at[0]).abs() < 1.0
            && (b.lo[2] - glass_at[2]).abs() < 1.0);
        if dead { break; }
        assert!(w.total_ticks - start < 900, "the pillarglass never broke under P2K fire");
        let p = sim.bond.pos;
        let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
        let dx = glass_at[0] - p[0];
        let dz = glass_at[2] - p[2];
        let dy = glass_at[1] - eye;
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / walker::dxz(p, glass_at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.05, 0.05),
            look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        loop_t += 1;
        if loop_t % 6 == 0 { w.tick_fire(input); } else { w.tick(input, false); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.breaks.list.iter().any(|b| b.dead), "the pillarglass died under fire [M Break_Kill 0x12b970]");
}

/// The entrance shutter's SS (#149, `Shutters2`): the disc's own raise mechanic — state 0x40 at
/// create, one-shot sweep key 0 → key 1 (+4.824 y) over the 240-tick rate when ch 0x23 turns on
/// [D placement params (0:0x40 1:0x1 2:0x1 3:0x23 6:0x1) + the parsed keys; M SS_Update 0x1a9948's
/// 0x40 arm + SS_Operate 0x1a9df8's channel arm]. The poke is the HARNESS's, not a claimed game
/// writer: the disc's ch-0x23 writer is the task's open item (see docs/levels/07000013.md §7).
#[test]
fn the_shutter_ss_sweeps_up_on_ch_0x23() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(30);
    {
        // Sister: no channel, no sweep — the shutter holds key 0 (the closed pose, [D]).
        let sim = w.scene.sim.as_ref().unwrap();
        let ss = &sim.drones.sss.list[0];
        assert_eq!(ss.placement, 149, "the level's one SS is the Shutters2 placement [D census]");
        assert!((ss.t - 0.0).abs() < 1e-4, "the playhead rests at key 0 closed [D]");
        assert!((ss.pos()[1] - 0.0).abs() < 1e-3, "the posed matrix sits at the baked (closed) pose");
    }
    let frame = { w.scene.sim.as_ref().unwrap().drones.channels.time[0x23] };
    w.scene.sim.as_mut().unwrap().drones.channels.set(0x23, 1, frame + 1);
    for _ in 0..400 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let ss = &sim.drones.sss.list[0];
    assert!(ss.t > 1.0, "the sweep ran past key 1 over 400 ticks (rate 240 [D]; got {:.3})"        , ss.t);
    assert!((ss.pos()[1] - 4.824).abs() < 0.35,
        "the posed matrix rose to the key-1 +4.824 y open pose [D keys; got {:.3}]", ss.pos()[1]);
}

/// The gold-medal `bonus[0]` word for the level's BaseMap [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm]. 07000013 plays under BaseMap 0x07000011 — the
/// mission/base pair sits at ee_start_05.bin @0x2a45a8 as the two equal words 0x07000011
/// 0x07000011, the same record shape as 0700000b's 0x07000009 0x07000009 @0x2a4490 [L], and the
/// mission's level list follows (07000012, 0700004a, 07000013 [L @0x2a45c8..]). pickup-medal-gate.
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}

/// KIND Door (0xdb ×2): the security-room door #355 (−26.65, −368.36, 37.44, no lock channel —
/// param 3 = 0 [D census]) opens on the player's E [M Door_Activate 0x135be0's toggle] and the
/// stairwell door #274 (18.82, −368.35, 38.95; lock channel param 3 = 0xc8 [D census] with NO
/// writer in the level's data [D §7]) refuses the same E — while ch 0xc8 is off the activation
/// only plays the hit sound [M Door_Activate's locked arm; door.rs ch_lock].
///
/// The approach alone opens NEITHER door: `Door_Update` 0x1363c8's sphere scan (an actor within
/// √2·(r_door + r_actor) of the door's live world sphere [M 0x136678-0x136684]) runs only to hold
/// or close an OPEN door — both its arms sit behind a state & 6 != 0 test, and `Door_Activate`'s
/// caller census (the channel arms, the open-timer arm, the player's E) leaves NO closed-door
/// proximity route in the game [M 0x1363c8; door.rs's `straddle_walkin` shim narrows to the same
/// shape]. The level doc §7's "proximity arm opens it as Bond walks up" misread that scan — this
/// test is the correction (the straddle param 4 bit is Door_Link2Portal 0x136788's cel-link flag,
/// not an opener).
#[test]
fn the_security_door_walks_open_and_the_stairwell_door_refuses() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    let (di355, di274) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let find = |x: f32, z: f32| sim.drones.doors.list.iter()
            .position(|d| walker::dxz(d.pos, [x, -368.36, z]) < 2.0);
        (find(-26.65, 37.44).expect("door #355 by its placement spot [D census]"),
         find(18.82, 38.95).expect("door #274 by its placement spot [D census]"))
    };
    {
        // Sister first: the stairwell door is closed and locked (ch 0xc8 has no writer [D §7]).
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.doors.list[di274].state & 0xc, 0,
            "the stairwell door rests closed [D]");
        assert!(!sim.drones.channels.on(0xc8), "the lock channel 0xc8 is off — no writer on the disc [D census]");
    }
    // The same walk-up that opens #355 walks to #274's doorway first: the locked door must not
    // move under the approach and must refuse the E (only the hit sound plays [M]). The route is
    // the lobby's proven north strip (the pickups route's z-40 lane) east to the stairwell
    // corner — the diagonal from the east lane wedges on the desk block's north-east cover row
    // (probe: stuck at (6.9, −368.1, 25.9) heading for (14, 30)).
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(5.0, -368.4, 38.0).r(1.5), "13: the north strip east end");
    w.walk_to(Wp::new(11.0, -368.4, 38.5).r(1.5), "13: the north strip past the desks");
    w.walk_to(Wp::new(16.5, -368.4, 38.5).r(1.2), "13: at the stairwell door");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di274];
        assert_eq!(d.state & 0xc, 0,
            "sister: the locked stairwell door does not proximity-open on the same walk-up [D param 3 = 0xc8; M Door_Activate's locked arm]");
    }
    w.use_door(di274, "13: E the locked stairwell door");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.doors.list[di274].state & 0xc, 0,
            "sister: the stairwell door refuses the E while ch 0xc8 is off [M Door_Activate's locked arm]");
    }
    // The security-room door: the proven walk up to its doorway with NO use key — the door must
    // stay closed (the game's scan never opens a closed door [M 0x1363c8]) — then the E opens it.
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.doors.list[di355].state & 0xc, 0,
            "the approach alone does not open the door — no closed-door proximity route in the game [M 0x1363c8's caller census]");
    }
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "13: E the security-room door");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.doors.list[di355].state & 0xc != 0,
        "the E opens the security-room door [M Door_Activate 0x135be0's toggle]");
}

/// KIND Pickup, the remaining two takes: the kind-0 weapon pickups (the three Tripbombs #432-434,
/// weapon 0x3a, 1 round each [D census], clustered just north of the spawn) arm the weapon on the
/// walk-over [M Pickup_Handler 0x18e650 case 0], and the kind-6 BOND_BONUS pair (#420/#421, label
/// 0x200004a [D census]) exists ONLY with the level's gold medal recorded — `Pickup_Create`
/// 0x18dda0:93-95 reads the medal gate — spins [M 0x18dda0:138-144] and is consumed by the take
/// path [M 0x18e650:234-236]. Sister: the same load with the fresh-boot zero medal word creates NO
/// kind-6 pickup at all (0b's exec-floor pattern, BaseMap 0x07000011 [L ee_start_05 @0x2a45a8]).
#[test]
fn the_tripbombs_arm_and_the_bond_bonus_takes_with_the_medal() {
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "07000013.bin", false, None, 1, gold_bits(0x0700_0011)).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.inventory.owned[0x3a], "sister: the Tripbomb is not owned before the walk-over");
        // The kind-6 pair exists with the medal: one spinning pickup at each placement spot [D].
        for (x, z) in [(7.29f32, 57.56f32), (-16.88, 29.30)] {
            let at = sim.drones.pickups.iter()
                .any(|p| p.kind == nf_game::pickup::kind::BOND_BONUS && !p.gone
                    && (p.m[3][0] - x).abs() < 0.5 && (p.m[3][2] - z).abs() < 0.5);
            assert!(at, "the BOND_BONUS pickup sits at ({x}, {z}) with the gold medal [M 0x18dda0:93-95; D #420/#421]");
        }
    }
    // The tripbomb cluster just north of the spawn [D #432 (−0.97, 13.94), #433 (−0.84, 14.46),
    // #434 (−0.39, 14.04)]: a few real steps north take all three.
    w.walk_to(Wp::new(-0.9, -368.4, 13.2).r(1.0), "13: into the tripbomb cluster");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.inventory.owned[0x3a],
            "the Tripbomb (0x3a) arms on the walk-over [M 0x18e650 case 0; D #432-434 weapon 0x3a]");
        let taken = sim.drones.pickups.iter()
            .filter(|p| p.weapon == 0x3a && (p.m[3][2] - 14.2).abs() < 1.2)
            .all(|p| p.gone || p.hidden);
        assert!(taken, "the tripbomb pickups are consumed by the takes [M 0x18e650:338-348]");
    }
    // The lobby BOND_BONUS #421 (−16.88, −369.03, 29.30 [D]) sits at the security-room glass
    // band's west end — the proven room route reaches it; the take is the real walk-over.
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "13: E the security-room door");
    w.wait(120);
    w.walk_to(Wp::new(-27.0, -368.4, 37.0).r(1.0), "13: through the doorway");
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "13: inside the security room");
    w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
    w.walk_to(Wp::new(-16.88, -368.4, 29.30).r(1.2), "13: onto the lobby bonus #421");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let taken = sim.drones.pickups.iter()
            .filter(|p| p.kind == nf_game::pickup::kind::BOND_BONUS && (p.m[3][0] + 16.88).abs() < 0.6)
            .all(|p| p.gone || p.hidden);
        assert!(taken, "the BOND_BONUS #421 is consumed on the walk-over [M 0x18e650:234-236]");
    }
    // Sister: the fresh-boot medal word creates no kind-6 at all [M 0x18dda0:93-95].
    let mut bare = Walker::new(nf_view::load_with_carry_medals(&iso(), "07000013.bin", false, None, 1, 0).unwrap());
    bare.wait_enabled(3000, "the 13 intro");
    bare.wait(60);
    assert!(bare.scene.sim.as_ref().unwrap().drones.pickups.iter()
        .all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonuses are not created [M 0x18dda0:93-95]");
}

/// KIND Break, the filtered pane: the securityroom_glass #286 (kind 0x20, params [0:0x2, 1:0xa,
/// 2:0x30] [D census]) carries param 2 = 0x30 → `Break_Create` stores ~0x30 = 0xffcf [M
/// 0x12c230:31-39] and `Break_Update`'s mask 0xffcf | 0x106 [M 0x12bfd8:15] — the P2K's round word
/// 0x0008 [L ee_start_05.bin weapon_data @0x2bf150 + 0x10·0x10C + 0x10] ANDs it non-zero, so
/// `Collide_FilterBullets` pulls the contact [M 0x1ece30:24-33] and the round never damages the
/// pane, though its health 10.0 would die to the first passing round [D param 1 = 0xa; M
/// 0x12bfd8:44-48]. The SAME fire path shatters the unfiltered pillarglass (the test above). The
/// 0x30-filter family pin on 07000009 is `interact_07000009_live::
/// the_security_room_glass_refuses_the_same_gun`; this is the level's own pane.
#[test]
fn the_securityroom_glass_refuses_the_p2k_word() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).unwrap());
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    // The pane: the level's one 0x30-filter break (health 10.0 [D param 1 = 0xa]).
    let pane = {
        let sim = w.scene.sim.as_ref().unwrap();
        let (i, b) = sim.drones.breaks.list.iter().enumerate()
            .find(|(_, b)| b.filter == 0xffcf && (b.health - 10.0).abs() < 0.01)
            .expect("the securityroom_glass with filter ~0x30 [D census]");
        assert!(!b.dead, "the pane is alive at load");
        (i, [b.lo, b.hi])
    };
    // The proven walk into the security room; stand south-east of the pane band (its bounds' own
    // x span, a couple of units into the room) and fire the held P2K straight at its face.
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "13: E the security-room door");
    w.wait(120);
    w.walk_to(Wp::new(-27.0, -368.4, 37.0).r(1.0), "13: through the doorway");
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "13: inside the security room");
    let mut shots = 0u32;
    // Approach the pane along the line from the room side (the walk left Bond inside the room)
    // and hold 2.5 m off its centre, then fire the held P2K at the centre.
    let stand = {
        let sim = w.scene.sim.as_ref().unwrap();
        let (lo, hi) = (pane.1[0], pane.1[1]);
        let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
        let from = sim.bond.pos;
        let d = [centre[0] - from[0], 0.0, centre[2] - from[2]];
        let l = (d[0] * d[0] + d[2] * d[2]).sqrt().max(1e-4);
        [centre[0] - 2.5 * d[0] / l, from[1], centre[2] - 2.5 * d[2] / l]
    };
    for t in 0..900u32 {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.breaks.list[pane.0].dead,
            "the filtered pane must survive the P2K — the filter arm died [M 0x1ece30]");
        if t == 899 { break; }
        let (lo, hi) = (pane.1[0], pane.1[1]);
        let target = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
        let p = sim.bond.pos;
        let input = if walker::dxz(p, stand) > 0.6 {
            let dx = stand[0] - p[0];
            let dz = stand[2] - p[2];
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            nf_game::player::Input { turn: err.clamp(-0.05, 0.05), forward: 1.0, ..Default::default() }
        } else {
            let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - p[0];
            let dz = target[2] - p[2];
            let dy = target[1] - eye;
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(p, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            nf_game::player::Input {
                turn: err.clamp(-0.05, 0.05),
                look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
                ..Default::default()
            }
        };
        let fired = t % 6 == 0;
        if fired { shots += 1; }
        if fired { w.tick_fire(input); } else { w.tick(input, false); }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(shots > 100, "the fire loop pulled the trigger ({shots} pulls)");
    assert!(!sim.drones.breaks.list[pane.0].dead,
        "the gun word 0x0008 is filtered out of the 0x30-filter pane [M 0x1ece30:24-33]");
    assert!(!sim.drones.break_kills.iter().any(|&(pl, _, _)| pl == sim.drones.breaks.list[pane.0].placement),
        "no kill effect for the filtered pane [M Break_Kill 0x12b970]");
}

/// KIND Hint plates (0x33 ×2) — the parse-model pin (the port's hints are `fix-context-hints`'
/// parse-time model, `nf_game::player::hints`; the live firing path — `Hint_Update` 0x1b3380's
/// one-shot TXTMSG type 1 + sound + the delete bit — is not in the sim and is the report's port
/// gap). The disc's pair [D census]: #423 (−19.17, −368.60, 26.00) label 0x10001f6, no gate; and
/// #424 (−19.85, −368.63, 26.04) label 0x2000003 whose on-channel is param 3 = 0x23 — the same
/// channel the gate panel's E writes, confirming ch 0x23 reads as "the security system released"
/// [M 0x1b3248:33-37 puts param 3 in inst+0x08, the on-channel; 0x1b3380:17-22 gates on it].
#[test]
fn the_hint_plates_carry_the_disc_params() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000013.bin")).expect("07000013.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let hints = nf_game::player::hints(&lv, 1);
    let by_label = |label: u32| hints.iter().find(|h| h.label == label);
    let h1 = by_label(0x10001f6).expect("hint #423 (label 0x10001f6) [D census]");
    assert_eq!(h1.on_channel, 0, "hint #423 has no arm channel — always armed [D census]");
    let h2 = by_label(0x2000003).expect("hint #424 (label 0x2000003) [D census]");
    assert_eq!(h2.on_channel, 0x23,
        "hint #424's arm channel is the gate panel's ch 0x23 [D census param 3; M 0x1b3248:33-37]");
    assert!((walker::dxz(h2.pos, [-19.85, -368.63, 26.04])) < 0.2,
        "hint #424 sits at its placement spot [D census]");
    assert_eq!(hints.len(), 2, "the level's two hint plates, nothing else [D census]");
}

/// Every placement's model resolves on this level — zero missing hashes/indices of the 439
/// placements across all 42 maps [D parse].
#[test]
fn every_placement_resolves_its_model() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000013.bin")).expect("07000013.bin");
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
    assert_eq!(unresolved, 0, "every placement's model resolves [D the 07000013 parse]");
}
