//! The weather draw against the game's `DrawDrops` 0x18cb90 (NF_ISO, opt-in).
//!
//! Rain (DropType 1): every live drop draws the `Raindrop` mesh 0x02000290 through
//! `psiDrawObjectMatrix` 0x124708 right after Game_Draw's `psiSetTweakARGB(0xff, 0xff, 0xff, 0xff)`
//! [M 0x1e6f50:148-151] — a white tweak, no cel/character ambient. On 07000009..0b a second pass
//! draws 0x02000039 per drop with the viewer's rotation under `psiSetTweakARGB(0x24, 0x20, 0x20, 0x20)`
//! [M 0x18cb90]. The port dimmed the streaks by Bond's ambient and had no second pass.
//!
//! Snow (DropType 0): `InitDrops` 0x18bf68 → 4096 flakes [M]; the flake size is
//! `nf_game::env::flake_half_extent` (PS2SnowSize per level [M 0x11e2e8], boot value [D .sdata 0x30c974]).
//! The rain caps are 0x80 on 09..0b and 0x40 elsewhere [M 0x18bf68].

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn tick(scene: &mut nf_view::scene::Scene, n: u32) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..n { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
}

/// The attachment indices of effect kind `k`.
fn kind_slots(scene: &nf_view::scene::Scene, k: u8) -> Vec<usize> {
    scene.attachments.iter().enumerate().filter(|(_, a)| matches!(a.effect, Some((kk, _, _)) if kk == k)).map(|(i, _)| i).collect()
}

#[test]
fn rain_0b_streaks_keep_the_white_tweak_and_the_second_pass_draws() {
    let mut scene = nf_view::load(&iso(), "0700000b.bin", false).expect("load 0b");
    tick(&mut scene, 120);
    let env = scene.sim.as_ref().unwrap().drones.env.as_ref().expect("0b weather");
    assert_eq!(env.kind, 1, "[D] 0b's live 0xDF has p0 = 1 (rain)");
    assert_eq!(env.drops.len(), 0x80, "[M 0x18bf68] rain cap 0x80 on 09..0b");
    let alive = env.alive.iter().filter(|&&a| a).count();
    assert!(alive > 64, "the start view sits inside active boxes: {alive} live drops");

    let streaks = kind_slots(&scene, 7);
    let haze = kind_slots(&scene, 8);
    assert_eq!(streaks.len(), nf_view::scene::RAIN_SLOTS);
    assert_eq!(haze.len(), nf_view::scene::RAIN_SLOTS, "[M 0x18cb90] the 0x02000039 pass on 09..0b");
    assert!(scene.attachments[haze[0]].effect.unwrap().2 == 0x0200_0039);

    let (chars, _, attach) = scene.colour_mults_dense();
    for &i in &streaks { assert_eq!(attach[i], [1.0; 3], "streak {i}: the 0xff tweak [M 0x1e6f50:148]"); }
    let t = (32.0 / 255.0) * (36.0 / 255.0);
    for &i in &haze { assert!((attach[i][0] - t).abs() < 1e-6, "haze {i}: tweak 0x20 rgb × 0x24 alpha"); }
    // Sister: Bond's own multiplier at this night start is NOT white — the old rule (character 0's
    // ambient) would have failed the streak check.
    assert!(chars[0].iter().any(|&c| (c - 1.0).abs() > 0.05), "character 0's ambient here: {:?}", chars[0]);
}

#[test]
fn rain_04_has_no_second_pass_and_the_0x40_cap() {
    let mut scene = nf_view::load(&iso(), "07000004.bin", false).expect("load 04");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [8.0, 5.5, 31.5];
        sim.bond.yaw = 3.14;
    }
    tick(&mut scene, 120);
    let env = scene.sim.as_ref().unwrap().drones.env.as_ref().expect("04 weather");
    assert_eq!(env.kind, 1);
    assert_eq!(env.drops.len(), 0x40, "[M 0x18bf68] rain cap 0x40 outside 09..0b");
    assert!(env.alive.iter().filter(|&&a| a).count() > 32, "the courtyard boxes hold the drops");
    assert_eq!(kind_slots(&scene, 7).len(), nf_view::scene::RAIN_SLOTS);
    // Sister: the level gate (GameState+12 − 0x07000009 < 3) keeps the second pass off 04.
    assert!(kind_slots(&scene, 8).is_empty(), "no 0x02000039 pass on 04");
    let (_, _, attach) = scene.colour_mults_dense();
    for i in kind_slots(&scene, 7) { assert_eq!(attach[i], [1.0; 3]); }
}

/// The half-extent of the drawn snow quads (the first quad's corner spread along `right`).
fn quad_half(scene: &nf_view::scene::Scene) -> (usize, f32) {
    let q = scene.sim.as_ref().unwrap().snow_quads([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let n = q.len() / 6;
    (n, (q[1].pos[0] - q[0].pos[0]) / 2.0)
}

#[test]
fn snow_05_and_06_draw_4096_flakes_at_the_level_size() {
    for (lvl, code, want) in [("07000005.bin", 0x0700_0005u32, 0.19674f32), ("07000006.bin", 0x0700_0006, 0.18746)] {
        let mut scene = nf_view::load(&iso(), lvl, false).expect("load");
        tick(&mut scene, 1);
        let env = scene.sim.as_ref().unwrap().drones.env.as_ref().expect("snow");
        assert_eq!(env.kind, 0, "{lvl}: [D] snow");
        assert_eq!(env.drops.len(), 0x1000, "{lvl}: [M 0x18bf68] 4096 flakes");
        // InitBodge (05..06 [M 0x18bf68]): the first update fills the whole cube (every slot live,
        // y within eye ± 15 [M 0x18c580 InitBodge arm]); the next update kills those under eye − 7.5.
        let alive = env.alive.iter().filter(|&&a| a).count();
        assert_eq!(alive, 0x1000, "{lvl}: the bodge fill");
        let (n, h) = quad_half(&scene);
        assert_eq!(n, alive, "{lvl}: one quad per live flake");
        assert!((h - want).abs() < 1e-3, "{lvl}: half-extent {h} (want {want})");
        assert!((h - nf_game::env::flake_half_extent(code)).abs() < 1e-5);
        // Sister: the old fixed 0.1 stand-in fails this.
        assert!((h - 0.1).abs() > 0.05, "{lvl}: not the old 0.1");
    }
}
