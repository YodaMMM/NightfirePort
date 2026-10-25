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
/// (−26.65, −368.36, 37.44 [D]), whose proximity arm opens it as Bond walks up ([D straddle
/// param 4 = 1; probe: state 0x2 while the leg closes] — the E toggle is the game's second arm,
/// [M Door_Activate]) and walks the room south to the shelves.
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
