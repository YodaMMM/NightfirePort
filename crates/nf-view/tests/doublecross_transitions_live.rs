//! doublecross-transitions: every Double Cross (mission 0x07000001, parts 07000001→07000002→
//! 07000003→07000004) level transition, on the real path — docs/systems/doublecross-transitions.md.
//!
//! The chain [D: the four levels' 0xE8 placements, `nf-extract placements`]: Castle A's exit #18
//! (auto gate ch 0x10, Touch #180 = Bond AND the skin-0x4F contact) loads 07000002; Castle B's
//! exit #206 (key gate 0x4C, E-key) loads 07000003; Castle C's exit #451 (key gate 0x52, E-key)
//! loads 07000004; Castle D has NO exit — the mission layer ends it (the intro script chain
//! enables the assassin objective, the kill raises its channel, `Mission_Update` maps the end to
//! the 0x07300004 outro [M 0x182c60 case 6]). The between-levels sequence [M]: the exit's
//! `ResetMap_LevelToLoad(next, 1, 0)` WRAPS the part code to its 0x0710000N briefing code
//! [M 0x1d3800:27-101], the 300-tick fade runs (level-exit-fade), the loader's phase 2 strips the
//! wrap back [M 0x1d3560:97-98] — the briefing codes 0x07100002/03/04 are NOT in `UsingMovie`
//! 0x107eb8, so no movie plays and the straight load stands — then the loading screen (the
//! Double Cross art group 0x030000d2 [M 0x1d5258]) and `Player_Init`'s 45-tick fade-in from black
//! [M 0x1910f8:45]. Run with NF_ISO set; each test carries a sister that can fail.

use nf_formats::iso9660::IsoImage;
use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load(iso: &str, level: &str) -> Scene {
    nf_view::load(iso, level, false).unwrap_or_else(|e| panic!("load {level}: {e}"))
}

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// The wrap→resolve round trip the drain runs [M 0x1d3800 + 0x1d3560; nf_game::triggers].
fn hop_lands_on(next: u32) -> u32 {
    let wrapped = nf_game::triggers::wrap_intro(next);
    assert_eq!(nf_game::triggers::resolve_exit_code(wrapped), next, "phase 2 strips the wrap back to the part");
    wrapped
}

/// None of the Double Cross exits may reference the never-entered wrapper codes [D: every 0xE8
/// param 0 on the disc scanned — nothing loads 0x07000041/43/46/49; the decomp has no loader
/// constant for them [M — the only ELF hits are the MapMusic rows 0x24fe0c-0x24fef0]].
fn assert_no_wrapper_target(sim: &nf_view::scene::Sim, level: &str) {
    for t in &sim.drones.triggers {
        if t.kind == nf_game::triggers::kind::LOAD_LEVEL {
            assert!(!matches!(t.next_level, 0x0700_0041 | 0x0700_0043 | 0x0700_0046 | 0x0700_0049),
                "{level}: an exit targets a wrapper code 0x{:08x} — the retail chain never does [D]", t.next_level);
        }
    }
}

/// `Player_Init`'s entry fade [M 0x1910f8:45] — the RAW state and the DRAW read are two
/// different claims, and both are pinned here:
/// - RAW: armed (45, 45) at the load, falls exactly 1 per tick, gone after 45. The game
///   advances the fade object per Game_Run tick — `Camera_UpdateAll` 0x1bbd68 loops every
///   viewer and runs unconditionally in Game_Run 0x1c9790:43, including while a script camera
///   owns the view; [L] FRAME_RATE_MUL = 1.0 (ee_snapshot 0x0030d0d8), so a 45-tick rate is
///   45 ticks.
/// - VIEW (`spawn_fade_view`, what the draw overlay reads): the fade object lives on the
///   PLAYER's viewer [M Camera_SetFade 0x1bbdd8:9 indexes `(&glb_viewer)[param_2]`;
///   Player_Init 0x1910f8:44-45 passes the player's viewer id], and a script camera DISABLES
///   the gameplay viewers — `Script_CameraStart` 0x1d9140 runs `Camera_Enable(v, 0, 0, 0)` for
///   v 0..6 (clearing the +0x2a draw flag [M Camera_Enable 0x1b9920:31-32]) and enables viewer
///   4, which never carries the entry fade (the SCRIPT fade arms there — Script_FadeStart
///   0x1db3d0:10). `Game_Draw` draws `psiFadeView` per viewer from that viewer's own fade
///   bytes [M 0x1e6f50:186-212], and PushStates/PopStates save only +0x29/+0x2a [M
///   0x1bbc60/0x1bbcd0]. So the view read is None exactly while a script camera owns the view:
///   the PS2 shows 07000002's opening cut-scene at full brightness from +3 [L, the main
///   agent's GDB-build capture, 2026-10-19]. On a load with no auto-play intro — Castle C's
///   script bank is mode-0/64 only [D, the §4 census] — the player viewer stays drawn and the
///   fade-up really shows for its 45 ticks.
/// Sisters that can fail: the exact per-tick raw ramp (a tick drift fails), the per-level
/// camera expectation from tick 1 on (a viewer-rule or script regression fails; tick 0 is the
/// pre-tick read — the port's intro beat is the first control walk, see the n == 0 arm), and
/// the exact drawn alpha ramp.
fn assert_entry_fade_runs(scene: &mut Scene, level: &str, view_drawn: bool) {
    {
        let sim = scene.sim.as_mut().unwrap();
        let (left, total) = sim.drones.spawn_fade.expect("the entry fade is armed at the load");
        assert_eq!((left, total), (45, 45), "{level}: Player_Init arms a 45-tick fade [M 0x1910f8:45]");
    }
    for n in 0..46u32 {
        let sim = scene.sim.as_mut().unwrap();
        let cam = sim.drones.scripts.camera().is_some();
        let view = sim.drones.spawn_fade_view();
        let raw_left = sim.drones.spawn_fade.map(|(l, _)| l);
        if n < 45 {
            assert_eq!(raw_left, Some(45 - n),
                "{level}: the raw fade falls 1/tick even under the cut-scene [M 0x1bada0:18 + 0x1c9790:43]");
        } else {
            assert_eq!(sim.drones.spawn_fade, None, "{level}: the raw fade ends at tick 45");
        }
        if view_drawn {
            assert!(!cam, "{level}: tick {n} must be camera-free (Castle C has no auto-play intro [D])");
            if n < 45 {
                let v = view.expect("{level}: the fade draws while no script camera owns the view [M 0x1e6f50:186-212]");
                assert!((v.alpha - (45 - n) as f32 / 45.0).abs() < 1e-6,
                    "{level}: drawn alpha must be (45-{n})/45, got {}", v.alpha);
            } else {
                assert!(view.is_none(), "{level}: the expired fade draws nothing");
            }
        } else if n == 0 {
            // Tick 0 reads BEFORE the first step: the port's load does not tick, so the auto-play
            // intro has not started yet — the game runs SP_LoadScript's mode dispatch (mode 1 →
            // `Script_Play`) during the level load itself [M 0x1dbc08:41-55 via SP_CreateScriptPlayer
            // 0x1db9a8], the port's beat lands on the first control walk (step 1) [S: port-only beat
            // shift, the script machinery byte-identical across the held-merge base; every DRAWN
            // frame matches — the window steps before it draws (lib.rs 4696) and the [L] capture
            // shows the cut-scene at full brightness from +3]. The pre-tick read pins the armed,
            // unticked state: the raw fade at (45, 45) and the fade view still on the player's
            // viewer (no script camera exists yet to take it off).
            assert_eq!(raw_left, Some(45),
                "{level}: the entry fade is armed and unticked before the first step [M 0x1910f8:45]");
            let v = view.expect("{level}: the fade view is the player viewer's before the intro starts");
            assert!((v.alpha - 1.0).abs() < 1e-6, "{level}: the pre-tick fade is at black, got {}", v.alpha);
        } else {
            assert!(cam, "{level}: tick {n} (after {n} steps) must sit under the auto-play intro's script camera [D mode-1 SP]");
            assert!(view.is_none(),
                "{level}: the entry fade must NOT draw while the cut-scene owns the view [M 0x1d9140 + 0x1b9920:31-32 + 0x1e6f50:186-212; L the PS2 capture]");
        }
        sim.step_bond(Default::default(), false, false, false);
    }
}

/// Castle A (07000001): the real route — the intro script 0x06000087 (mode 1, pack 07800001)
/// writes ch 0xB [D], the contact leaves his wait, Bond AND the contact on Touch #180's plate
/// raise ch 0x10, exit #18 fires `ResetMap_LevelToLoad(0x07000002, 1, 0)` [M 0x1b2318:111].
/// Sister: Bond alone on the plate raises nothing (the touch's mask 0x18 needs the NPC too).
#[test]
fn castle_a_route_fires_the_exit_and_the_hop_runs_the_wrapped_load() {
    let iso_v = iso();
    let mut scene = load(&iso_v, "07000001.bin");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.level, 0x0700_0001);
        let exit = sim.drones.triggers.iter().find(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL)
            .expect("Castle A has exactly one 0xE8 [D]");
        assert_eq!(exit.next_level, 0x0700_0002, "[D 07000001 #18 param 0]");
        assert_eq!(exit.gate, 0x10, "[D #18 param 2 — the auto gate]");
        assert_eq!(exit.key_gate, 0, "[D #18 param 1 — no key gate]");
        assert_no_wrapper_target(sim, "07000001");
    }
    let sim = scene.sim.as_mut().unwrap();
    let plate = sim.drones.triggers.iter().find(|t| t.kind == nf_game::triggers::kind::TOUCH && t.out == 0x10)
        .expect("the LoadPointActivator (Touch out 0x10) [D #180]").clone();
    let centre = [(plate.lo[0] + plate.hi[0]) * 0.5, (plate.lo[1] + plate.hi[1]) * 0.5, (plate.lo[2] + plate.hi[2]) * 0.5];
    // Sister: Bond ALONE on the plate.
    sim.bond.pos = centre;
    for _ in 0..6 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x10), "Bond alone must not arm the AND-touch");
    assert_eq!(sim.drones.level_exit, None);
    // The route: intro → ch 0xB → the contact starts.
    let mut enabled_at = None;
    for t in 0..3000u32 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    enabled_at.expect("the intro script 0x06000087 must write ch 0xB [D]");
    let npc_i = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the skin-0x4F contact [D]");
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    // Both on the plate → ch 0x10 → the exit.
    let mut fired = None;
    for i in 0..12 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.bond.pos = centre;
            sim.drones.list[npc_i].pos = [centre[0] + 0.5, centre[1], centre[2]];
        }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.level_exit.is_some() { fired = Some(i); break; }
    }
    assert_eq!(scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0002),
        "Bond + the contact on the plate must end the level to 07000002 [D]");
    eprintln!("[01→02] exit fired at walk tick {fired:?}");
    // The hop: the drain wraps → no movie → resolves straight back [M].
    let wrapped = hop_lands_on(0x0700_0002);
    assert_eq!(wrapped, 0x0710_0002, "[M 0x1d3800:33-36]");
    assert!(!nf_view::movie::USING_MOVIE.contains(&wrapped),
        "0x07100002 is not in UsingMovie [M 0x107eb8] — no state-6 hold, the straight load stands");
    // The loading screen rows [M 0x1d5258 — the Double Cross group].
    assert_eq!(nf_view::loading::image_for(0x0700_0002), 0x0300_00d2, "the Double Cross art group");
    assert_eq!(nf_view::loading::objective_for(0x0700_0002), Some(0x0400_0008), "[M 0x1d5258's title labels]");
    // The next part loads; the entry fade is armed but its DRAW is None — the opening
    // cut-scene's script camera owns the view from tick 0 (the PS2 shows the cut-scene at full
    // brightness [L]). Raw state still ticks to expiry [M 0x1c9790:43].
    let next = nf_view::level_archive_name(0x0700_0002).to_string();
    let mut scene2 = load(&iso_v, &next);
    assert_eq!(scene2.sim.as_ref().expect("sim").level, 0x0700_0002);
    assert_no_wrapper_target(scene2.sim.as_ref().unwrap(), "07000002");
    assert_entry_fade_runs(&mut scene2, "07000002", false);
}

/// Castle B (07000002): the exit #206 is E-key-only behind key gate ch 0x4C (AND #480 = [0x48,
/// 0x4b] — the mission layer's own progress) [D]. Sister: with the gate OFF the E press is the
/// game's locked arm (fallback cooldown, NO load [M 0x1b2318:76-90]). Gate on → the load request.
#[test]
fn castle_b_exit_is_locked_until_the_gate_then_loads_07000003_through_the_wrap() {
    let iso_v = iso();
    let mut scene = load(&iso_v, "07000002.bin");
    let (centre, key_gate) = {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.level, 0x0700_0002);
        assert_no_wrapper_target(sim, "07000002");
        let exit = sim.drones.triggers.iter().find(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL && t.next_level == 0x0700_0003)
            .expect("Castle B's single exit loads 07000003 [D #206]").clone();
        assert_eq!(exit.key_gate, 0x4c, "[D #206 param 1]");
        assert_eq!(exit.gate, 0, "[D #206 param 2 — E-key only]");
        let centre = [(exit.lo[0] + exit.hi[0]) * 0.5, exit.lo[1] - 1.4, (exit.lo[2] + exit.hi[2]) * 0.5];
        (centre, exit.key_gate)
    };
    // Sister: the gate off → the E press must NOT load.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = centre;
        sim.bond.yaw = 0.0;
        sim.bond.pitch = 0.0;
        sim.step_bond(Default::default(), false, false, true);
        assert!(!sim.drones.channels.on(0x4c));
        assert_eq!(sim.drones.level_exit, None, "the locked arm fires no load [M 0x1b2318:76-90]");
    }
    // The gate's own writers are the mission layer's AND #480 [D]; the pin is the exit, so arm it
    // the way the AND's output does and press E on the exit.
    let mut fired = None;
    for i in 0..8 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.drones.set_channel(key_gate);
            sim.bond.pos = centre;
            sim.bond.yaw = 0.0;
            sim.bond.pitch = 0.0;
        }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.level_exit.is_some() { fired = Some(i); break; }
    }
    assert_eq!(scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0003), "[D #206 param 0]");
    eprintln!("[02→03] exit fired at press {fired:?}");
    let wrapped = hop_lands_on(0x0700_0003);
    assert_eq!(wrapped, 0x0710_0003, "[M 0x1d3800:37-40]");
    assert!(!nf_view::movie::USING_MOVIE.contains(&wrapped), "no movie on 02→03 [M 0x107eb8]");
    assert_eq!(nf_view::loading::image_for(0x0700_0003), 0x0300_00d2);
    assert_eq!(nf_view::loading::objective_for(0x0700_0003), Some(0x0400_000a));
    let next = nf_view::level_archive_name(0x0700_0003).to_string();
    let mut scene2 = load(&iso_v, &next);
    assert_eq!(scene2.sim.as_ref().expect("sim").level, 0x0700_0003);
    // 07000003 draws the entry fade: its script bank is mode-0/64 only — no auto-play intro,
    // no script camera in the window [D §4], so the player viewer stays drawn and the game's
    // fade-up really shows [M 0x1e6f50:186-212].
    assert_entry_fade_runs(&mut scene2, "07000003", true);
}

/// Castle C (07000003): exit #451, key gate 0x52 (AND #435 = [0x4e, 0x51]) [D] — same shape,
/// loading 07000004. Sister: the locked E press.
#[test]
fn castle_c_exit_is_locked_until_the_gate_then_loads_07000004_through_the_wrap() {
    let iso_v = iso();
    let mut scene = load(&iso_v, "07000003.bin");
    let (centre, key_gate) = {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.level, 0x0700_0003);
        assert_no_wrapper_target(sim, "07000003");
        let exit = sim.drones.triggers.iter().find(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL && t.next_level == 0x0700_0004)
            .expect("Castle C's single exit loads 07000004 [D #451]").clone();
        assert_eq!(exit.key_gate, 0x52, "[D #451 param 1]");
        let centre = [(exit.lo[0] + exit.hi[0]) * 0.5, exit.lo[1] - 1.4, (exit.lo[2] + exit.hi[2]) * 0.5];
        (centre, exit.key_gate)
    };
    // Sister: the gate off → the E press must NOT load.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = centre;
        sim.bond.yaw = 0.0;
        sim.bond.pitch = 0.0;
        sim.step_bond(Default::default(), false, false, true);
        assert_eq!(sim.drones.level_exit, None, "the locked arm fires no load [M 0x1b2318:76-90]");
    }
    let mut fired = None;
    for i in 0..8 {
        {
            let sim = scene.sim.as_mut().unwrap();
            sim.drones.set_channel(key_gate);
            sim.bond.pos = centre;
            sim.bond.yaw = 0.0;
            sim.bond.pitch = 0.0;
        }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.level_exit.is_some() { fired = Some(i); break; }
    }
    assert_eq!(scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0004), "[D #451 param 0]");
    eprintln!("[03→04] exit fired at press {fired:?}");
    let wrapped = hop_lands_on(0x0700_0004);
    assert_eq!(wrapped, 0x0710_0004, "[M 0x1d3800:41-44]");
    assert!(!nf_view::movie::USING_MOVIE.contains(&wrapped), "no movie on 03→04 [M 0x107eb8]");
    assert_eq!(nf_view::loading::image_for(0x0700_0004), 0x0300_00d2);
    assert_eq!(nf_view::loading::objective_for(0x0700_0004), Some(0x0400_000d));
    let next = nf_view::level_archive_name(0x0700_0004).to_string();
    let mut scene2 = load(&iso_v, &next);
    assert_eq!(scene2.sim.as_ref().expect("sim").level, 0x0700_0004);
    assert_entry_fade_runs(&mut scene2, "07000004", false); // auto-play SP #3 [D §4] — hidden like 02
}

/// Castle D (07000004) ends in the mission layer, not a level exit [D — the level has no 0xE8]:
/// the intro 0x06000083 (500 ticks, mode 1) → 0xD [0x38,1] → SP #2 plays 0x06000082 (830 ticks)
/// → 0xD [0x53,1] enables the only objective "Defeat the assassin" (channel 0x54, enable 0x53)
/// [D scripts + L mission/c07000004.rs]; the assassin (the level's only drone, "oDrone" #1,
/// death channel 0x54 [D param 12]) dying through the game's own explosion path raises 0x54 →
/// `Mission_Update` internal 6 sets LevelToEndTo 0x0730004 [M 0x182c60 case 6] → after the
/// 240-tick banner the end posts with next 0x07300004 [M case 4]. Sister: at load the objective's
/// enable channel is still OFF — the chain, not boot, arms it.
#[test]
fn castle_d_intro_enables_the_assassin_and_the_kill_ends_the_mission_to_the_outro() {
    let iso_v = iso();
    let mut scene = load(&iso_v, "07000004.bin");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.level, 0x0700_0004);
        assert!(!sim.drones.triggers.iter().any(|t| t.kind == nf_game::triggers::kind::LOAD_LEVEL),
            "Castle D has no 0xE8 exit [D — the all-maps dump is empty]");
        assert!(!sim.drones.channels.on(0x53), "sister: the enable channel is off at load");
    }
    let sim = scene.sim.as_mut().unwrap();
    let mut enabled_at = None;
    for t in 0..2600u32 {
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.channels.on(0x53) { enabled_at = Some(t); break; }
    }
    let enabled_at = enabled_at.expect("the intro chain 0x06000083 → 0x06000082 must write ch 0x53 [D]");
    eprintln!("[04] objective enabled at tick {enabled_at}");
    // The assassin — the only drone, death channel 0x54 [D param 12].
    let assassin = {
        let sim = scene.sim.as_ref().unwrap();
        let i = sim.drones.list.iter().position(|d| d.death_channel == 0x54)
            .expect("the assassin drone (oDrone #1, death channel 0x54) must exist [D]");
        (i, sim.drones.list[i].pos)
    };
    // Kill him through the game's own grenade path [M Explode_Propagate 0x17c518 → Drone_ExplosiveHit],
    // the walk_07000001 pin's shape: a frag at his feet.
    let (i, pos) = assassin;
    scene.sim.as_mut().unwrap().drones.grenades.push(nf_game::grenade::Grenade {
        pos: [pos[0], pos[1] + 0.5, pos[2]], velocity: [0.0, 0.0, 0.0], speed: 0.0,
        travelled: 0.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34, mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None,
        phase: None,
    });
    let mut objective_at = None;
    for _ in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x54) { objective_at = Some(true); break; }
    }
    assert!(objective_at.is_some(), "the assassin's death channel 0x54 (param 12) must raise [D]");
    // The mission layer ends the mission to the outro [M 0x182c60 case 6], after the 240-tick banner.
    let mut end = None;
    for _ in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if let Some(next) = scene.sim.as_ref().unwrap().drones.mission_end { end = Some(next); break; }
    }
    assert_eq!(end, Some(0x0730_0004), "LevelToEndTo(0x07000004) = 0x0730004 [M 0x182c60 case 6]");
    // The outro's own landing: it IS whitelisted (the movie exists [M 0x107eb8]) and phase 2
    // resolves it to the front end with the results page queued [M 0x1d3560:24-50].
    assert!(nf_view::movie::USING_MOVIE.contains(&0x0730_0004), "the 04 outro plays [M 0x107eb8]");
    assert_eq!(nf_game::triggers::resolve_exit_code(0x0730_0004), 0x0700_0048, "[M 0x1d3560's 0x73 arm]");
}

/// The numbers-only pin (no disc read): the Double Cross row's movie-less hops keep their straight
/// load, the loading-screen rows match `ResetMap_LevelCode2Img` [M 0x1d5258], and the mission-start
/// briefing is the one whitelisted Double Cross code [M 0x107eb8; D /MOVIES/30_FPS 07100001.pss].
#[test]
fn the_double_cross_row_wraps_movie_less_and_keeps_the_straight_load() {
    // Sister first: the mission-start code wraps to the one whitelisted briefing.
    let start = nf_game::triggers::wrap_intro(0x0700_0001);
    assert_eq!(start, 0x0710_0001);
    assert!(nf_view::movie::USING_MOVIE.contains(&start), "0x07100001 plays the Double Cross briefing [M 0x107eb8]");
    // The three hops: wrap → not whitelisted (no state-6 hold, no movie) → resolve == the part.
    for (part, wrapped) in [(0x0700_0002, 0x0710_0002), (0x0700_0003, 0x0710_0003), (0x0700_0004, 0x0710_0004)] {
        assert_eq!(nf_game::triggers::wrap_intro(part), wrapped, "[M 0x1d3800]");
        assert!(!nf_view::movie::USING_MOVIE.contains(&wrapped), "0x{wrapped:08x} has no movie file [M 0x107eb8]");
        assert_eq!(nf_game::triggers::resolve_exit_code(wrapped), part, "the straight load stands [M 0x1d3560:97-98]");
        // And the hop never lands on a wrapper [D].
        assert!(!matches!(nf_game::triggers::resolve_exit_code(wrapped), 0x0700_0041 | 0x0700_0043 | 0x0700_0046 | 0x0700_0049));
    }
    // The loading rows for all four parts [M 0x1d5258: the 0x30000d2 group, title labels 6/8/a/d].
    for (part, img, obj) in [
        (0x0700_0001, 0x0300_00d2, 0x0400_0006),
        (0x0700_0002, 0x0300_00d2, 0x0400_0008),
        (0x0700_0003, 0x0300_00d2, 0x0400_000a),
        (0x0700_0004, 0x0300_00d2, 0x0400_000d),
    ] {
        assert_eq!(nf_view::loading::image_for(part), img, "0x{part:08x}'s Double Cross art");
        assert_eq!(nf_view::loading::objective_for(part), Some(obj), "0x{part:08x}'s title label");
    }
    // The hint tables' sizes [M 0x1d5258: 01 two entries, 02 four, 03 five, 04 two; 05 is the
    // fixed-hint sister [M — pinned in loading.rs's tables]].
    assert_eq!(nf_view::loading::hint_for(0x0700_0002, || 1), Some(0x0100_00c8), "02's own table (rand 1 → entry 1, the coin flip keeps it) [M 0x1d5258]");
    assert_eq!(nf_view::loading::hint_for(0x0700_0004, || 3), Some(0x0100_00d0), "04's own table (rand 3 → entry 1) [M 0x1d5258]");
}

/// The mission-fail side stays separate [M 0x182c60:21-42 — a ch-0x60 death never requests a
/// level]: the port's fail path opens the fail menu OVER the paused level (no load, no movie).
/// Sister on the same disc data: Castle A's contact death raises the fail channel 0x60/0x63 and
/// `level_exit` stays None (walk_07000001 pins the channel; this pins the no-load).
#[test]
fn a_double_cross_death_opens_no_transition() {
    let iso_v = iso();
    let mut scene = load(&iso_v, "07000001.bin");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000u32 { sim.step_bond(Default::default(), false, false, false); if sim.drones.channels.on(0xb) { break; } }
    let npc_i = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    let npc_pos = sim.drones.list[npc_i].pos;
    sim.drones.grenades.push(nf_game::grenade::Grenade {
        pos: [npc_pos[0], npc_pos[1] + 0.5, npc_pos[2]], velocity: [0.0, 0.0, 0.0], speed: 0.0,
        travelled: 0.0, range: 60.0, damage: 100.0, radius: 3.0, weapon_type: 0x34, mesh: 0x0200_0726, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
        stuck: false, seek_target: None,
        phase: None,
    });
    let mut failed = false;
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x63) { failed = true; break; }
    }
    assert!(failed, "the mission-fail channel must rise [M 0x1827d8:104-108]");
    assert_eq!(scene.sim.as_ref().unwrap().drones.level_exit, None, "a death requests no load [M 0x182c60:21-42]");
    let _ = read_archive(&iso_v, "07000001.bin"); // the ISO is present; keep the helper used
}
