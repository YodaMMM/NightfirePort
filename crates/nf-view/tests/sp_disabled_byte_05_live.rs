//! sp-disabled-byte-05: 07000005's copter ScriptPlayers — the SP +0x36 disabled byte's setter.
//!
//! The game freezes a copter's looping SP through `Copter_Create` 0x17d340:73-74: the SP is
//! created at load (`SP_Create` 0x1dc7e0 → `SP_CreateScriptPlayer` 0x1db9a8, modes 8,8, mask 0 →
//! state 6, primed looping) and immediately `SP_HideObj(sp, 1)` — SP+0x36 = 1 — when the copter's
//! gate channel (placement param 6, copter+0x70) is armed and off. `Copter_Update` 0x17dbb8's head
//! unhides it (`SP_HideObj(sp, 0)`, copter+0x70 = 0) on the gate's first ON tick; `SP_Update`
//! 0x1dbda8:57-59 returns while SP+0x36 is set, so the primed pair holds its pose. On 07000005 the
//! two type-0x2F placements (#241 gate 0, #242 gate 66 = 0x42 [D the census,
//! interact_07000005_live]) are exactly the snapshot's twin looping pairs: #242's SP reads
//! +0x36 = 1 (obj 0x16ddb40, SI0 primed with entity streams [L ee_start_05.bin]) and #241's reads
//! 0 (0x16df360) and plays. The old port kept the pair out of the world entirely and played a
//! synthetic one-shot player on the gate edge — the stand-in this task removes
//! (docs/systems/copter-sp-hide.md).
//!
//! Opt-in: `NF_ISO`. Every game claim cites [M]/[D]/[L]; the ch 0x42 drive in the unhide test is
//! the documented harness stand-in for walking to the SWITCH_Helicopterstart plate (the game's
//! channel writer).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn load(iso: &str) -> Walker {
    let mut w = Walker::new(nf_view::load(iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    w
}

/// The pair indices (the two players carrying the copter's script pair).
fn copter_sps(sim: &nf_view::scene::Sim) -> Vec<usize> {
    (0..sim.drones.scripts.players.len())
        .filter(|&i| sim.drones.scripts.players[i].scripts == [0x0600_0046, 0x0600_0047])
        .collect()
}

/// The two copter SPs exist from the create beat, primed (mode 8, mask 0 → state 6 [M 0x1dc7e0 →
/// 0x1db9a8]), and the +0x36 rule holds: the gate-0 copter's SP (#241) plays, the gate-66 one's
/// (#242) sits frozen with a loaded script — the snapshot's twins [L ee_start_05.bin]. FAILS on
/// the stand-in: no player for the pair exists at load there (the synthetic one-shot spawns only
/// on the gate edge).
#[test]
fn the_copter_sps_exist_and_the_gated_one_is_disabled() {
    let mut w = load(&iso());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let sps = copter_sps(sim);
        assert_eq!(sps.len(), 2, "both copters' ScriptPlayers exist from the create beat [M 0x17d340:66-70]");
        for &i in &sps {
            let p = &sim.drones.scripts.players[i];
            assert_eq!(p.modes, [8, 8], "SP {i}: modes 8,8 [M 0x1dc7e0 fills both mode slots with param 5; L ee_start_05]");
            assert_eq!(p.state, 6, "SP {i}: mask 0 → state 6 [M 0x1db9a8:72-76; L ee_start_05]");
            assert!(p.info[p.cur].is_some(), "SP {i}: primed — the SI exists even while disabled [L ee_start_05: 0x16ddb40's SI0 0x16dd070 holds play 0x9 with entity streams]");
        }
        // The hide rule: the SP whose copter's gate is armed and off at create reads disabled, the
        // gate-0 one does not [M 0x17d340:73-74; L +0x36 = 1 on 0x16ddb40, 0 on the twin].
        let gate_of = |sim: &nf_view::scene::Sim, i: usize| {
            sim.drones.copters.iter()
                .find(|c| c.placement == sim.drones.scripts.players[i].placement)
                .map(|c| c.gate)
                .unwrap_or(u16::MAX)
        };
        let frozen = sps.iter().copied().find(|&i| sim.drones.scripts.players[i].hidden);
        let playing = sps.iter().copied().find(|&i| !sim.drones.scripts.players[i].hidden);
        assert_eq!(frozen.map(|i| gate_of(sim, i)), Some(66),
            "the gate-66 copter's SP (#242) is the disabled one [M 0x17d340:73-74; L +0x36 = 1]");
        assert_eq!(playing.map(|i| gate_of(sim, i)), Some(0),
            "the gate-0 copter's SP (#241) plays from load [M 0x17d340:73-74's gate!=0 guard; L +0x36 = 0]");
        assert!(playing.is_some() && gate_of(sim, playing.unwrap()) == 0);
    }
    // The frozen SP's clock does not advance: SP_Update returns while +0x36 is set [M 0x1dbda8:57-59].
    let clock = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        let fi = copter_sps(sim).into_iter()
            .find(|&i| sim.drones.scripts.players[i].hidden)
            .expect("the frozen SP persists");
        sim.drones.scripts.players[fi].info[sim.drones.scripts.players[fi].cur].as_ref().unwrap().clock
    };
    let a = clock(&w);
    w.wait(60);
    let b = clock(&w);
    assert_eq!(a, b, "the disabled SP's script clock holds while +0x36 is set [M 0x1dbda8:57-59]");
    // The playing twin's clock advances over the same window.
    let pclock = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        let pi = copter_sps(sim).into_iter()
            .find(|&i| !sim.drones.scripts.players[i].hidden)
            .expect("the playing twin persists");
        sim.drones.scripts.players[pi].info[sim.drones.scripts.players[pi].cur].as_ref().unwrap().clock
    };
    let c = pclock(&w);
    w.wait(60);
    let d = pclock(&w);
    assert!(d > c, "the twin pair ticks (clock {c} -> {d}) [L ee_start_05: the twin plays]");
}

/// The gate edge: arming ch 0x42 (the SWITCH_Helicopterstart plate's channel — the harness
/// stand-in for the plate's own touch) unhides #242's SP on the next tick and the pair starts
/// ticking; the gate field is consumed (the copter keeps updating after — [M 0x17dbb8 head's
/// `+0x70 = 0`]). The Script_HideObj cascade ends with it: the SP's stream objects read as live
/// again [M 0x1dcbf0:12-14 → 0x1db808].
#[test]
fn the_gate_edge_unhides_the_frozen_sp() {
    let mut w = load(&iso());
    let fi = {
        let sim = w.scene.sim.as_ref().unwrap();
        let fi = copter_sps(sim).into_iter()
            .find(|&i| sim.drones.scripts.players[i].hidden)
            .expect("the gate-66 copter's SP starts frozen [M 0x17d340:73-74]");
        assert!(!sim.drones.scripts.hidden_stream_objects().is_empty(),
            "the frozen SP's stream objects carry the hide set [M 0x1db808's bit 0x10]");
        let placement = sim.drones.scripts.players[fi].placement;
        let cur = sim.drones.scripts.players[fi].cur;
        let a = sim.drones.scripts.players[fi].info[cur].as_ref().unwrap().clock;
        (fi, placement, cur, a)
    };
    // The plate's channel, driven as the documented stand-in.
    w.scene.sim.as_mut().unwrap().drones.set_channel(0x42);
    w.wait(2);
    let (fi, placement, cur, a) = fi;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.scripts.players[fi].hidden, "the gate's ON tick unhid the SP [M 0x17dbb8 head]");
        let b = sim.drones.scripts.players[fi].info[cur].as_ref().unwrap().clock;
        assert!(b > a, "the pair ticks again after the unhide (clock {a} -> {b}) [M 0x1dbda8's +0x36 gate cleared]");
        assert!(sim.drones.copters.iter().filter(|c| c.placement == placement).all(|c| c.gate == 0),
            "the gate field is consumed after the unhide [M 0x17dbb8: +0x70 = 0]");
    }
}

/// The sister: Skyrail's (MP map 0x07000024) mask-0 gondola SP — placement 4, kind 0xD9, script
/// 0x06000058, mode 8, mask 0 [D the mp-map-objects census] — still loads and runs. The old
/// prime-skip stand-in froze exactly these (the playtest 1016-1355's "no gondolas"): the gondola
/// is the original victim of skipping the prime for mask-0 SPs. Guards the prime path the copter
/// SPs now share. Driven on the level's own Scripts (the MP map has no campaign flow to walk).
#[test]
fn the_mask0_gondola_sp_still_loads_and_runs() {
    let path = std::path::PathBuf::from(&iso());
    let isoi = nf_formats::iso9660::IsoImage::open(path.to_str().unwrap()).expect("open ISO");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&isoi).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000024.bin")).expect("skyrail level bin");
    let data = isoi.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read skyrail");
    let level = nf_formats::level::Level::parse(&data).expect("parse skyrail");
    let m = level.main_map().expect("skyrail main map");
    let lib: std::collections::HashMap<u32, std::sync::Arc<nf_formats::nis::Nis>> =
        level.nis.iter().map(|(&h, n)| (h, std::sync::Arc::new(n.clone()))).collect();
    let mut scripts = nf_game::nis::Scripts::new(lib, &level.maps[m].placements);
    let idx = scripts.players.iter().position(|p| p.scripts[0] == 0x0600_0058)
        .expect("the gondola SP exists [D placement 4; M 0x1db9a8's prime]");
    {
        let p = &scripts.players[idx];
        assert_eq!(p.modes[0], 8, "the gondola runs mode 8 [D params]");
        assert_eq!(p.state, 6, "mask 0 → state 6 [M 0x1db9a8:72-76]");
        assert!(!p.hidden, "nothing hides a placed SP at create [M 0x1db9a8 writes no +0x36]");
    }
    struct W;
    impl nf_game::nis::World for W {
        fn anim_length(&self, _script: u32) -> Option<u16> { Some(100) }
        fn discrete0(&self, _skin: u32, _script: u32, _frame: f32) -> Option<nf_formats::anim::Mat4> { None }
        fn level(&self) -> u32 { 0x0700_0024 }
    }
    let mut ch = nf_game::triggers::Channels { v: vec![0u8; 256], time: vec![0u32; 256], writes: None, script_channels: Default::default() };
    let mut w = W;
    let mut a = 0.0f32;
    for frame in 0..120u32 {
        let skip = scripts.begin_tick(&mut ch, &w, frame);
        scripts.update_player(idx, &mut ch, &w, frame, skip, nf_game::nis::SpStart { health_ok: true, mission: 0, music2: 0, blasts: Vec::new() });
        if frame == 1 {
            let p = &scripts.players[idx];
            assert!(p.info[p.cur].is_some(), "the gondola's script loaded [M 0x1dbc08's mode-8 arm]");
            a = p.info[p.cur].as_ref().unwrap().clock;
        }
    }
    let p = &scripts.players[idx];
    let b = p.info[p.cur].as_ref().unwrap().clock;
    assert!(b > a, "the gondola's loop ticks (clock {a} -> {b}) [M SP_Update 0x1dbda8 → Script_Update]");
    assert!(scripts.hidden_stream_objects().is_empty(), "no placed SP is hidden [M 0x1db9a8 writes no +0x36]");
}
