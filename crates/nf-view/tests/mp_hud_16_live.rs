//! mp-hud-16 — the in-match MP HUD with 16 participants (1 human + 15 bots on 07000023), numbers
//! only (quad counts per texture, layout numbers — no pixel goldens). The data side is
//! nf-game/tests/mp_hud_16.rs; docs/systems/mp-hud-data.md §7. Opt-in via NF_ISO.

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

/// Quads per texture index in a HUD frame (`HudFrame::runs` carry the texture slot; 0 = solid).
fn quads_with(frame: &nf_view::gpu::HudFrame, slot: usize) -> usize {
    frame.runs.iter().filter(|r| r.0 as usize == slot).map(|r| r.3 as usize / 6).sum()
}

/// 16 participants: the radar draws its ring and one blip per other participant (15) — the
/// game's 24-blip cap holds them all [M 0x19f6e8:108,153]. Sister: the viewer's radar toggle off
/// hides the ring and empties the blips [M 0x189840:22, 0x19f6e8:102-105].
#[test]
fn sixteen_participants_draw_fifteen_radar_blips() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, settings(15)).expect("load");
    let m = scene.sim.as_ref().unwrap().mp.as_ref().unwrap();
    assert_eq!(m.bots.len(), 15, "15 bots created");
    let slot = |h: u32| scene.hud_textures.iter().position(|(k, _)| *k == h).map(|i| i + 1);
    let (ring, blip) = (slot(0x0300_0088).expect("the radar ring texture loads in MP"), slot(0x0300_016f).expect("the blip sheet loads in MP"));
    // The ring's template [L RadarSprInfo @0x2bdd60, read from the disc's ACTION.ELF]: texture
    // 0x03000088, layer 0x1e, flags 0 (plain blend), colour 0x7f7f7fff.
    let t = scene.hud.as_ref().expect("hud tables");
    assert_eq!(t.radar.len(), 1);
    assert_eq!((t.radar[0].texture, t.radar[0].layer, t.radar[0].flags, t.radar[0].colour), (0x0300_0088, 0x1e, 0, 0x7f7f_7fff));
    assert_eq!(t.mp_score.len(), 7, "MPScoreInfo holds 7 sprites [L docs/fields/hud-panes.md]");
    let frame = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    assert_eq!(quads_with(&frame, ring), 1, "one ring sprite");
    assert_eq!(quads_with(&frame, blip), 15, "a blip for each of the 15 other participants");
    // The ring's colour is the template's 0x7f7f7f on the GS scale (0x7f/0x80 ≈ 0.99) — sister to
    // the raw-127 multiplier that blew the dark-red ring out to white.
    let r = frame.runs.iter().find(|r| r.0 as usize == ring).unwrap();
    let c = frame.v[r.2 as usize].c;
    assert!(c.iter().take(3).all(|&k| (k - 127.0 / 128.0).abs() < 1e-3), "ring colour {c:?}");
    // Sister: the toggle off.
    let sim = scene.sim.as_mut().unwrap();
    let me = sim.mp_local_slot;
    sim.mp.as_mut().unwrap().settings.players[me].radar_on = 0;
    let off = nf_view::hud::overlay(&scene, 4.0 / 3.0);
    assert_eq!(quads_with(&off, ring), 0, "the ring hides with the toggle");
    assert_eq!(quads_with(&off, blip), 0, "no blips with the toggle off");
}

/// The standings layout [S port layout]: ≤ 8 participants keep the HUD font's own size and list
/// everyone; 9–16 shrink to a constant on-screen size (the lobby's 720/view_h rule) — at 720p
/// 16 rows do not fit the 182-unit band, so the list is cut (the viewer always kept, nf-game's
/// board test); at 1440p every row fits. Sister: the 720p compact scale differs from the ≤ 8 one.
#[test]
fn the_standings_layout_scales_past_eight() {
    let lh = 16.0;
    let avail = 182.0;
    let (s8, p8, r8) = nf_view::hud::mp_board_layout(8, lh, 720.0, avail);
    assert_eq!((s8, r8), (1.0, 8), "8 participants: full-size rows, all listed");
    assert!(p8 * 8.0 <= avail, "8 full rows fit the band");
    let (s16, p16, r16) = nf_view::hud::mp_board_layout(16, lh, 720.0, avail);
    assert!(s16 < 1.0, "compact past 8");
    assert!(r16 < 16 && (r16 as f32 + 1.0) * p16 <= avail, "cut at 720p with room for the +N line: {r16} rows");
    let (s16h, _, r16h) = nf_view::hud::mp_board_layout(16, lh, 1440.0, avail);
    assert_eq!(r16h, 16, "every row fits at 1440p");
    assert!((s16h * 1440.0 - s16 * 720.0).abs() < 1e-3, "the same on-screen size at both heights");
    // Window height doesn't touch the ≤ 8 layout (the game's HUD scales with the window).
    assert_eq!(nf_view::hud::mp_board_layout(8, lh, 1440.0, avail), (s8, p8, r8));
}
