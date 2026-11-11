//! ssys-wire: the security subsystems count down and fire in play.
//!
//! Before this task crates/nf-game/src/ssys.rs ported `SSys_Init` 0x1b01b0 / `SSys_Monitor`
//! 0x1ad998 / `SSys_Msg` 0x1ad878 / `SSys_RamLoad` 0x1b0208 with nothing calling them (grep: the
//! only `Ssys` references outside the file were none) — the 240 s countdowns, their announcements
//! and their channel writes never ran. The wiring, each [M]:
//! - `Switch_Create` 0x1adcc0:29-31 — the switch-create channel store (`attach_level_id`);
//! - `SSys_Monitor` from `Game_Run` 0x1c9790:32-34 — the per-tick pass (tick_inner, after
//!   `Env_Update`, before the object-handler passes, `ScriptCam == 0` [M 0x1ad998:8]);
//! - `SSys_RamLoad` from `Player_RamLoad` 0x1957a8:13 — the warm-load block restore
//!   (`load_with_carry`), whose block `Player_RamSave` 0x195670:9 wrote.
//!
//! The census [D `ssys_switch_census`, all 20 story main maps]: exactly two switches carry a
//! non-zero placement param 6 (`level_tag`+0x44) — 07000009 #71 (ch 0x37 → item 1, laser 1) and
//! 0700000a #643 (ch 0x3b → item 2, laser 2, gate ch 0x3c). The load snapshot proves the store
//! live: [L ee_start_0700000a.bin @0x2befd0] `SSysItems` = item 0 {0,0,0}; item 1
//! {−1.0, 0xff, 0}; **item 2 {−1.0, 0x3b, 0}**; items 3/4 {−1.0, 0xff, 0}; `SSysDelayTimer` 0.0.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test ssys_wire_live -- --test-threads=1`

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The `nis::World` the use-pick needs: the level's glist bounds (`Control_BuildWorldSph`
/// 0x133070's inputs — the entity target's sphere), read straight off the parsed level.
struct GlistWorld { models: std::collections::HashMap<u32, nf_game::pickup::Model> }
impl nf_game::nis::World for GlistWorld {
    fn anim_length(&self, _script: u32) -> Option<u16> { None }
    fn discrete0(&self, _skin: u32, _script: u32, _frame: f32) -> Option<nf_formats::anim::Mat4> { None }
    fn glist_sphere(&self, hash: u32) -> Option<([f32; 3], f32)> {
        self.models.get(&hash).map(|m| (m.centre, m.radius))
    }
}

fn level_models(iso: &str, archive: &str) -> (Level, GlistWorld) {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse");
    let models = nf_game::pickup::models(&level);
    (level, GlistWorld { models })
}

/// The switch-create channel store, through the real load: a cold 0700000a reads `SSysItems[2]`
/// = {−1.0, 0x3b, 0} with every other item still at the `SSys_Init` 0x1b01b0 0xff — the disc's
/// own load snapshot shape [L ee_start_0700000a.bin @0x2befd0].
#[test]
fn the_switch_create_store_lands_in_ssysitems() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    // 0700000a: switch #643 (param 6 = 2, param 0 = 0x3b) arms item 2's watch channel [D census].
    let scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    let s = &scene.sim.as_ref().unwrap().drones.ssys;
    assert_eq!(s.items[2], nf_game::ssys::SsysItem { timer: -1.0, channel: 0x3b, state: 0 },
        "SSysItems[2] reads the 0a load snapshot [L ee_start_0700000a.bin @0x2befd0; M Switch_Create 0x1adcc0:29-31]");
    // Sister (can fail): the SSys_Init'd items keep 0xff — the store hits item 2 only.
    for i in [1usize, 3, 4] {
        assert_eq!(s.items[i].channel, 0xff, "item {i} stays SSys_Init'd [L @0x2befd0 +{i}·8]");
    }
    // 07000009: switch #71 (param 6 = 1, param 0 = 0x37) arms item 1 [D census].
    let scene = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009");
    let s = &scene.sim.as_ref().unwrap().drones.ssys;
    assert_eq!(s.items[1].channel, 0x37, "09's laser switch feeds item 1 [D #71 params]");
    assert_eq!(s.items[2].channel, 0xff, "no item-2 switch on 09 [D census]");
}

/// The real-path pin on the census level: Bond's use dispatch reaches the lobby laser switch
/// through its embedded show's live entity (the game's only route to a scripted-position switch
/// [M Player_Activate 0x1a5170:121-122 + SP_Activate 0x1dc338:44-53 + Switch_Create 0x1adcc0:44-46]),
/// `Switch_Activate` 0x1b02a8 toggles ch 0x37, `SSys_Monitor` 0x1ad998 arms item 1 at 240 s
/// (`Game_Run` 0x1c9790:32), the announcements play at 180/120/60/6 s (`SSys_Msg` 0x1ad878's
/// laser-1 table [L ee_start_05 @0x2bef60]), the channel drops at 0 — and the drop itself
/// unpauses the show again (`Switch_Update` 0x1adea8:19-24's mirror arm): the AMBER art
/// (`SCRIPT_Lasers_AMBER` 0x02000a29 [D script 0x06000667 stream 0 t31]) comes up.
#[test]
fn the_09_laser_switch_arms_the_countdown_announces_and_drops() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000009.bin", false).expect("load 07000009");
    let (_level, w) = level_models(&iso, "07000009.bin");
    let sim = scene.sim.as_mut().unwrap();
    // Nothing arms while ch 0x37 is off (the load beat [L snapshot shape]): items idle.
    let mut input = Input::default();
    for _t in 1..=30u32 {
        sim.step_bond(input, false, false, false); input = Input::default();
        // The game's per-frame sound maintenance (`Sound_UpdateSounds` 0x1dda70) — step_bond is
        // update-only; headless the driver holds no voices, so `playing` = false frees each node
        // after its START_GRACE (the pool must drain over a 240 s run).
        sim.drones.audio.update(None, ([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), &|_| false);
    }
    assert_eq!(sim.drones.ssys.items[1].state, 0, "no arm while the channel is off [M 0x1ad998:15-21]");
    assert_eq!(sim.drones.channels.v[0x37], 0, "the laser channel starts off [L ee_start_0700000a analog; D the 09 dumps' non-writers]");

    // The trip: the use dispatch's switch-through-SP route. The live entity target comes from the
    // show's ON art (created at script t0 [D 0x06000667 stream 0]); the pick point `head + look`
    // lands on the posed sphere and the body stands on the facing side (identity matrix → dir +z).
    let spi = sim.drones.useobjs.switches.iter().find(|s| s.ch == 0x37)
        .and_then(|s| s.sp).expect("the 09 laser switch's embedded player [M Switch_Create 0x1adcc0:44-46]");
    let targets = sim.drones.scripts.players[spi].entity_use_targets_dir(&sim.drones.scripts.objects, &w);
    let Some((c, _r, dir, epos)) = targets.iter().find(|(_, r, _, _)| *r > 0.0).copied() else {
        panic!("no live laser entity target after 30 ticks — the ON art never mounted [D 0x06000667 t0]");
    };
    let _ = epos;
    let body = [c[0], c[1], c[2] + (dir[2].signum() * 1.0).max(1.0)];
    let head = [c[0], c[1] + 0.5, c[2]];
    let look = [c[0] - head[0], c[1] - head[1], c[2] - head[2]];
    let used = sim.drones.useobjs.player_activate(
        &mut sim.drones.scripts, &mut sim.drones.channels, head, look, sim.drones.frame, &w, body);
    assert!(used, "the use dispatch reached the laser switch through its show entity");
    assert_eq!(sim.drones.channels.v[0x37], 1, "Switch_Activate 0x1b02a8 toggled ch 0x37 on");
    assert_eq!(sim.drones.ssys.items[1].channel, 0x37, "item 1 watches exactly the toggled channel [D #71 param 6 = 1]");

    // The countdown, through the real tick loop (the monitor rides Game_Run's slot in tick_inner).
    // Thresholds [L ee_snapshot @0x30cd84..0x30cd90 — 180/120/60/6]; the arm is 240.0 [M 0x1ad998
    // 0x43700000]; the laser-1 sound column indexed by the msg id [L ee_start_05 @0x2bef60 —
    // msg 6 = 0x2a2 … msg 1 = 0x2a3].
    const MSG6: u32 = 0x2a2; const MSG5: u32 = 0x295; const MSG4: u32 = 0x296;
    const MSG3: u32 = 0x297; const MSG2: u32 = 0x298; const MSG1: u32 = 0x2a3;
    let mut arm_tick = None;
    let mut seen: Vec<(u32, u32)> = Vec::new(); // (tick, sound id)
    let mut drop_tick = None;
    for t in 1..=14_700u32 {
        sim.step_bond(input, false, false, false);
        input = Input::default();
        // The observations read the just-started nodes (state 0) BEFORE the update promotes them
        // (`update` moves a fresh node to state 1 on its first pass — sound.rs's deferred start).
        if arm_tick.is_none() && sim.drones.ssys.items[1].state == 6 {
            arm_tick = Some(t);
            assert!((sim.drones.ssys.items[1].timer - 240.0).abs() < 0.01, "the arm writes 240.0 [M 0x43700000]");
            assert!(sim.drones.audio.active_ids().contains(&MSG6), "the arm announcement plays [M SSys_Msg(i, 6)]");
        }
        let ids = sim.drones.audio.active_ids();
        for (id, m) in [(MSG5, 5u32), (MSG4, 4), (MSG3, 3), (MSG2, 2)] {
            if ids.contains(&id) && !seen.iter().any(|(_, mm)| *mm == m) {
                seen.push((t, id));
                let at = t - arm_tick.expect("armed before any announcement");
                // 240−180 = 60 s → tick 3600 after the arm, each next threshold 3600 later, the
                // 6 s one 234·60 = 14040 after the arm [L MIN3/MIN2/MIN1/MIN01 @0x30cd84..90].
                // The window is ±8: the monitor subtracts the f32 1/60 and the accumulated
                // rounding drifts a few ticks at 3600 (the unit pin allows ±2 per 480).
                let want = match m { 5 => 3600, 4 => 7200, 3 => 10800, _ => 14040 };
                assert!((at as i64 - want as i64).abs() <= 8,
                    "msg {m} at +{at} ticks after the arm, want ~{want} [L MIN3/MIN2/MIN1/MIN01 @0x30cd84..90]");
            }
        }
        if sim.drones.channels.v[0x37] == 0 && arm_tick.is_some() && drop_tick.is_none() {
            drop_tick = Some(t);
            let at = t - arm_tick.unwrap();
            assert!((at as i64 - 14400).abs() <= 16, "the channel drops 240 s after the arm, got +{at} [M 0x43700000 arm, REC_FRAME_RATE]");
            assert_eq!(sim.drones.ssys.items[1].state, 0, "state 0 after the fire [M 0x1adc70]");
            assert_eq!(sim.drones.ssys.items[1].timer, -1.0, "timer −1 after the fire [M 0x1adc74]");
            assert!(ids.contains(&MSG1), "the fire announcement plays [M SSys_Msg(i, 1)]");
            break;
        }
        sim.drones.audio.update(None, ([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), &|_| false);
    }
    let arm_tick = arm_tick.expect("the item armed after the trip");
    let drop_tick = drop_tick.expect("the channel dropped at the countdown's end");
    // Every announcement happened, in order [M SSys_Monitor 0x1ad998's state switch].
    let order: Vec<u32> = seen.iter().map(|(_, id)| *id).collect();
    assert_eq!(order, vec![MSG5, MSG4, MSG3, MSG2], "180/120/60/6 s announcements in order");

    // The drop's own reaction: the channel change unpauses the embedded show again
    // (`Switch_Update` 0x1adea8:19-24) — the 983-tick show resumes from its t2 hold, so the
    // t31 AMBER swap lands ~31 ticks later [D 0x06000667 stream 0: AMBER 0x02000a29 at t31].
    for _t in drop_tick + 1..=drop_tick + 60 {
        sim.step_bond(input, false, false, false);
        input = Input::default();
        sim.drones.audio.update(None, ([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), &|_| false);
    }
    let ents = sim.drones.scripts.entities();
    let laser: Vec<u32> = ents.iter().filter(|(sh, ..)| *sh == 0x0600_0667).map(|&(_, _, hash, _, _)| hash).collect();
    assert!(laser.contains(&0x0200_0a29),
        "the drop unpauses the show and the AMBER art comes up (got {laser:?}) [M Switch_Update 0x1adea8:19-24; D 0x06000667 t31]");
    let _ = arm_tick;
}

/// The save side: a warm load restores the block (`Player_RamLoad` 0x1957a8:13 → `SSys_RamLoad`
/// 0x1b0208 — the block `Player_RamSave` 0x195670:9 wrote), and the 0a skip keeps item 2 standing
/// ([M 0x1b0208:13] — the port plants the carried record first, then `ram_load`'s skip leaves it).
#[test]
fn the_warm_load_restores_the_ssys_block_and_0a_keeps_item_2() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let block = [
        nf_game::ssys::SsysItem { timer: 33.0, channel: 0x51, state: 4 },
        nf_game::ssys::SsysItem { timer: 118.5, channel: 0x3b, state: 4 },
        nf_game::ssys::SsysItem { timer: -1.0, channel: 0x52, state: 0 },
        nf_game::ssys::SsysItem { timer: 240.0, channel: 0x53, state: 6 },
    ];
    let mut ssys = nf_game::ssys::Ssys::new();
    for k in 0..4 { ssys.items[k + 1] = block[k]; }
    let ch = nf_game::triggers::Channels::default();
    let save = nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
        .with_ssys(&ssys);
    // 07000009: every record copies verbatim [M 0x1b0208:10-18].
    let scene = nf_view::load_with_carry(&iso, "07000009.bin", false, Some(&save)).expect("warm load 09");
    let s = &scene.sim.as_ref().unwrap().drones.ssys;
    for k in 0..4 {
        assert_eq!(s.items[k + 1], block[k], "warm load restores item {} [M SSys_RamLoad 0x1b0208]", k + 1);
    }
    assert_eq!(s.delay_timer, 0.0, "SSysDelayTimer = 0 [M 0x1b0208:21]");
    // 0700000a: item 2 is NOT loaded — the carried record stands in its place (the game's .data
    // persistence, the exit save being the last writer before the load) [M 0x1b0208:13].
    let scene = nf_view::load_with_carry(&iso, "0700000a.bin", false, Some(&save)).expect("warm load 0a");
    let s = &scene.sim.as_ref().unwrap().drones.ssys;
    assert_eq!(s.items[2], block[1], "the skip keeps the carried record 2 on 0a [M 0x1b0208:13]");
    assert_eq!(s.items[1], block[0], "item 1 still loads on 0a [M 0x1b0208:13's item-2-only skip]");
    assert_eq!(s.items[3], block[2], "item 3 still loads on 0a");
    // The death-replay keep: the fail serve carries the LIVE item 2 in the block's record-2 slot
    // (`fail_restart_save`), so the reload's pre-seed plants the death-time record where the
    // game's .data kept it.
    let scene = nf_view::load_with_carry(&iso, "0700000a.bin", false, Some(&save)).expect("warm load 0a for the death serve");
    let mut sim = scene.sim.unwrap();
    // Mid-play: the Mission_Init latch stamps `level_current` on the first tick [M 0x1d5080:6],
    // and a death happens in play, never at tick 0.
    let mut input = Input::default();
    for _t in 1..=5u32 { sim.step_bond(input, false, false, false); input = Input::default(); }
    let entry = nf_view::entry_checkpoint_save(&sim);
    let dying = nf_game::ssys::SsysItem { timer: 55.25, channel: 0x3b, state: 5 };
    sim.drones.ssys.items[2] = dying;
    let serve = nf_view::fail_restart_save(Some(&entry), &sim);
    let served = serve.ssys.expect("the serve carries the block");
    assert_eq!(served[1], dying, "the death serve keeps the death-time item 2 [M 0x1b0208:13's skip]");
    // Item 1 rode the 5 live ticks too: its saved state 4 with the channel off hit the monitor's
    // disarm arm on the first tick — state 0, timer −1.0, the msg-0 sound [M 0x1ad998:15-21] —
    // and the entry checkpoint (the serve's base) carries exactly that post-tick record.
    assert_eq!(served[0], nf_game::ssys::SsysItem { timer: -1.0, channel: 0x51, state: 0 },
        "the monitor's channel-off disarm ran through the real ticks [M 0x1ad998:15-21]");
}
