//! mp-hud-real — the in-match MP HUD drawn where the real game draws it (numbers only: quad
//! counts and corner positions per texture, no pixel goldens). The positions are the runtime
//! sprite rects read from PCSX2 EE snapshots of a live 1-human Arena match on 07000023
//! (2026-10-18; docs/systems/mp-hud-data.md §8). Opt-in via NF_ISO.

use nf_game::mp::MpSettings;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn settings(bots: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = bots;
    s.time_limit = 600;
    s
}

fn quads_with(frame: &nf_view::gpu::HudFrame, slot: usize) -> usize {
    frame.runs.iter().filter(|r| r.0 as usize == slot).map(|r| r.3 as usize / 6).sum()
}

/// The first quad's top-left of a texture slot, back in HUD units (4:3 window: the Left anchor
/// maps x → −1 + x·2/512, y → 1 − y/224).
fn top_left(frame: &nf_view::gpu::HudFrame, slot: usize) -> Option<(f32, f32)> {
    let r = frame.runs.iter().find(|r| r.0 as usize == slot)?;
    let v = frame.v[r.2 as usize].p;
    Some(((v[0] + 1.0) * 256.0, (1.0 - v[1]) * 224.0))
}

/// One human + 3 bots: the score pane's gun / skull icons at (16,17) / (16,35), the MP health
/// pane's backing + bar at (16,416), the radar ring at (432,16) — the game's created rects [L];
/// the SP health swirl (sheet 0x0300007a) is NOT drawn in a match (the MP pane list has no SP
/// health pane [L MPPaneList[1] = MPHealthPane]). Sister: an SP level draws the swirl and none
/// of the MP pane textures.
#[test]
fn the_mp_panes_sit_where_the_game_puts_them() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, settings(3)).expect("load");
    let slot = |h: u32| scene.hud_textures.iter().position(|(k, _)| *k == h).map(|i| i + 1);
    let frame = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    let near = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 0.05 && (a.1 - b.1).abs() < 0.05;
    let gun = slot(0x0300_0187).expect("the kills icon loads in MP");
    let skull = slot(0x0300_0186).expect("the deaths icon loads in MP");
    let star = slot(0x0300_0188).expect("the team icon loads in MP");
    assert!(near(top_left(&frame, gun).unwrap(), (16.0, 17.0)), "gun {:?}", top_left(&frame, gun));
    assert!(near(top_left(&frame, skull).unwrap(), (16.0, 35.0)), "skull {:?}", top_left(&frame, skull));
    assert_eq!(quads_with(&frame, star), 0, "FFA hides the team icon [M 0x19ed10:108-112]");
    let back = slot(0x0300_0172).expect("the MP health backing loads");
    let bar = slot(0x0300_0171).expect("the MP health bar loads");
    assert!(near(top_left(&frame, back).unwrap(), (16.0, 416.0)), "backing {:?}", top_left(&frame, back));
    assert!(near(top_left(&frame, bar).unwrap(), (16.0, 416.0)));
    let ring = slot(0x0300_0088).unwrap();
    assert!(near(top_left(&frame, ring).unwrap(), (432.0, 16.0)), "ring {:?}", top_left(&frame, ring));
    let swirl = slot(0x0300_007a);
    assert!(swirl.map_or(true, |s| quads_with(&frame, s) == 0), "no SP health swirl in a match");
    // The ammo strip (0x03000128) belongs to the SP pane only [M 0x19a7f0:247,275 — the MP pane's 3 sprites].
    assert!(slot(0x0300_0128).map_or(true, |s| quads_with(&frame, s) == 0), "no rounds strip in a match");

    // Sister: the SP level.
    let sp = nf_view::load(&iso, "07000005.bin", false).expect("load sp");
    let sp_frame = nf_view::hud::overlay(&sp, 4.0 / 3.0);
    let sp_slot = |h: u32| sp.hud_textures.iter().position(|(k, _)| *k == h).map(|i| i + 1);
    assert!(quads_with(&sp_frame, sp_slot(0x0300_007a).expect("the SP sheet")) > 0, "the SP swirl draws on an SP level");
    assert!(sp_slot(0x0300_0187).map_or(true, |s| quads_with(&sp_frame, s) == 0));
}

/// A kill by the local player shows "Killed <victim>" in the MP info pane: the message lands in
/// the INFO queue with label 0x200004d and the victim's name ([M 0x188f70:100-107]; the bank's
/// "Killed %s" [D USATxt.dat]). Sister: a bot-on-bot kill posts nothing to the local player.
#[test]
fn a_local_kill_posts_the_kill_message() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, settings(3)).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    let me = sim.mp_local_slot;
    {
        let m = sim.mp.as_mut().unwrap();
        let mut rng = nf_game::drone::Rng::new(1);
        m.player_killed(2, Some(3), false, false, &mut rng); // bot on bot
        m.player_killed(1, Some(me), false, false, &mut rng);
    }
    scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
    let sim = scene.sim.as_ref().unwrap();
    let msgs: Vec<(u32, String)> = sim.hud.info_msgs.iter().map(|m| (m.label, m.text.clone())).collect();
    assert_eq!(msgs.iter().filter(|(l, _)| *l == nf_game::mp::hud::KILL_MSG_LABEL).count(), 1, "{msgs:?}");
    let (_, text) = msgs.iter().find(|(l, _)| *l == nf_game::mp::hud::KILL_MSG_LABEL).unwrap();
    assert!(text.starts_with("Killed "), "{text}");
}
