//! gun-smoke-haze: the pistol's smoke on 07000004 at the play-start pose, pinned to the game's
//! numbers.
//!
//! Four P2K shots 10 ticks apart from (-3.869, 5.455, 30.303) yaw 1.658 at the wall ahead. Per
//! shot the game creates exactly two gas objects [L PCSX2 DynamicObjList walk of class 0xE,
//! audits render-1109-haze]:
//! - the muzzle puff: `Player_MuzzleFlash` 0x1a4298 → `Gas_Init(0.05, 0.1, cel, datum 0, rot,
//!   NULL, 0x2000039, 7, 0x48403080, 1)` life 128 [M 0x1a4298; life 0x80 at 0x1a43f8]. Live:
//!   kind 7, spawned at x -3.010 y 6.009 z 30.498, scale 0.0508 two ticks in, scale rate
//!   0.00039, life 126, rgba (70, 63, 47, 126), rgba rate (-0.562, -0.5, -0.375, -0.992);
//! - the wall impact puff: `Effect_Bullet` 0x1ce8a8's only gas row for this wall's material,
//!   mesh 0x2000124, kind 5, scale 1.5 → 2.5, life 4, rgba 0x7f7f007f → 0x7f7f0040 (the player's
//!   own shot skips the `Rand_Rand(2)` coin [M 0x1ce94c]). Live: at (9.922, 6.155, 29.097).
//! 15 ticks after the 4th shot the four muzzle puffs are alive at scale 0.0566 / 0.0605 /
//! 0.0645 / 0.0684 and alpha 111 / 101 / 91 / 81 [L] — the port's pool carries the same values.
//! The quad (`smoke_1_dust`, ±2.86 × ±2.99) draws additive: its leaf's ALPHA_1 is 0x48
//! ((Cs − 0)·As + Cd) with ZMSK set [D the glist's GS words].
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn settle(scene: &mut nf_view::scene::Scene) {
    for _ in 0..6000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { return; }
    }
    panic!("the intro scenes never settled");
}

const MUZZLE: u32 = nf_game::gas::SMOKE_QUAD;
const IMPACT: u32 = 0x0200_0124;

/// A muzzle puff `age` ticks after its spawn (the pool updates it in the spawn step, so age 1 is
/// the first read), per `Gas_Init` 0x1aacc8's args and `gases_smoke` 0x1ab2d0's tail:
/// scale 0.05 + age·(0.1 − 0.05)/128, alpha 128 + age·(1 − 128)/128, life 128 − age.
fn muzzle_expect(age: f32) -> (f32, f32, f32) {
    (0.05 + age * (0.1 - 0.05) / 128.0, 128.0 + age * (1.0 - 128.0) / 128.0, 128.0 - age)
}

fn matches_muzzle(g: &nf_game::gas::Gas) -> bool {
    let (s, a, _) = muzzle_expect(128.0 - g.life);
    g.mesh == MUZZLE && g.kind == 7 && (g.scale - s).abs() < 1e-4 && (g.rgba[3] - a).abs() < 0.05
        && (g.scale_rate - 0.05 / 128.0).abs() < 1e-7
        && (g.rgba_rate[3] - (1.0 - 128.0) / 128.0).abs() < 1e-5
}

#[test]
fn four_pistol_shots_make_the_games_puffs_at_the_04_start() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso(), "07000004.bin", false).expect("load");
    settle(&mut scene);
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.gun.weapon, 2, "the level's starting P2K");
        let y = sim.bond.pos[1];
        sim.bond = nf_game::player::Bond::new([-3.869, y, 30.303], 1.658);
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let mut muzzles = 0;
    let mut impacts = 0;
    let mut seen: Vec<(u32, u32)> = Vec::new();
    for t in 0..=45u32 {
        let fire = t % 10 == 0 && t < 40;
        scene.sim.as_mut().unwrap().step_bond(Input { fire, ..Default::default() }, fire, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for g in sim.drones.gas.puffs.iter().filter(|g| g.mesh == MUZZLE || g.mesh == IMPACT) {
            // A puff's identity: its mesh + its spawn-time spin draw (unique per Gas_Init).
            let key = (g.mesh, g.spin.to_bits());
            if seen.contains(&key) { continue; }
            seen.push(key);
            if g.mesh == MUZZLE {
                muzzles += 1;
                eprintln!("t{t} muzzle pos {:?} vel {:?} scale {} rate {} rgba {:?} rate {:?} life {}", g.pos, g.vel, g.scale, g.scale_rate, g.rgba, g.rgba_rate, g.life);
                assert!(matches_muzzle(g), "the muzzle puff carries Gas_Init(0.05, 0.1, 7, 0x48403080 → 1, life 128) [M 0x1a4298]: {g:?}");
                assert_eq!(g.life, 127.0, "read one tick after its spawn [L 126 at a two-tick read]");
                for (k, s) in [0x48 as f32, 0x40 as f32, 0x30 as f32].into_iter().enumerate() {
                    let want = s - s / 128.0;
                    assert!((g.rgba[k] - want).abs() < 1e-3, "rgb {k}: {} want {want} (0x48403080 fading to 0x00000001)", g.rgba[k]);
                }
                assert!((g.pos[0] + 3.01).abs() < 0.05 && (g.pos[1] - 6.0).abs() < 0.05 && (g.pos[2] - 30.5).abs() < 0.05,
                    "the muzzle puff spawns at the gun's datum 0 [L (-3.010, 6.009, 30.498)], got {:?}", g.pos);
            } else {
                impacts += 1;
                eprintln!("t{t} impact pos {:?} scale {} rate {} rgba {:?} life {} kind {}", g.pos, g.scale, g.scale_rate, g.rgba, g.life, g.kind);
                assert_eq!(g.kind, 5, "the hard-surface puff is kind 5 [D EffectInfo row; L kind 5]");
                assert!((g.scale_rate - 0.25).abs() < 1e-6, "1.5 → 2.5 over life 4 [D; L rate 0.25]");
                assert!((g.pos[0] - 9.922).abs() < 0.05 && (g.pos[1] - 6.15).abs() < 0.05 && (g.pos[2] - 29.097).abs() < 0.05,
                    "the impact sits on the wall ahead [L (9.922, 6.155, 29.097)], got {:?}", g.pos);
            }
        }
    }
    assert_eq!(muzzles, 4, "one muzzle puff per shot [L: one class-0xE kind-7 object per R1 press]");
    assert_eq!(impacts, 4, "one impact puff per shot [L: one kind-5 0x2000124 object per hit]");
    // 15 ticks after the 4th shot: the four live puffs at their ages.
    let sim = scene.sim.as_ref().unwrap();
    let mut live: Vec<&nf_game::gas::Gas> = sim.drones.gas.puffs.iter().filter(|g| g.mesh == MUZZLE && g.kind == 7).collect();
    live.sort_by(|a, b| b.life.total_cmp(&a.life));
    assert_eq!(live.len(), 4, "all four muzzle puffs are alive 15 ticks after the last shot");
    for (g, age) in live.iter().zip([16.0f32, 26.0, 36.0, 46.0]) {
        let (s, a, l) = muzzle_expect(age);
        eprintln!("t45 puff pos {:?} scale {} alpha {} life {}", g.pos, g.scale, g.rgba[3], g.life);
        assert_eq!(g.life, l);
        assert!((g.scale - s).abs() < 1e-4 && (g.rgba[3] - a).abs() < 0.05, "age {age}: scale {} alpha {}", g.scale, g.rgba[3]);
    }
    // [L] one tick later on the PS2: scale 0.0566 / 0.0605 / 0.0645 / 0.0684, alpha 111 / 101 / 91 / 81.
    for (age, (ls, la)) in [17.0f32, 27.0, 37.0, 47.0].into_iter().zip([(0.0566f32, 111.0f32), (0.0605, 101.0), (0.0645, 91.0), (0.0684, 81.0)]) {
        let (s, a, _) = muzzle_expect(age);
        assert!((s - ls).abs() < 2e-4 && (a.floor() - la).abs() <= 1.0, "the formula reproduces the live read at age {age}: {s} {a} vs {ls} {la}");
    }
    // Sister: the old life-8 growth rate or a doubled alpha fails the check.
    let mut bad = (*live[0]).clone();
    bad.scale_rate = 0.05 / 8.0;
    assert!(!matches_muzzle(&bad), "the checker rejects a wrong growth rate");
    let mut bad = (*live[0]).clone();
    bad.rgba[3] *= 2.0;
    assert!(!matches_muzzle(&bad), "the checker rejects a wrong alpha");
}

/// The draw side [D]: `smoke_1_dust` 0x2000039 is one 4-vertex quad of ±2.86 × ±2.99 with vertex
/// colour (254, 254, 254, 128), drawn with ALPHA_1 0x48 (additive, (Cs − 0)·As + Cd) and ZBUF_1
/// ZMSK set; the port's decode of those words must give blend 1 (additive) and no depth write.
#[test]
fn the_smoke_quad_is_an_additive_no_zwrite_quad() {
    let iso_img = nf_formats::iso9660::IsoImage::open(iso()).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso_img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000004.bin")).expect("07000004");
    let archive = iso_img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("archive");
    let level = nf_formats::level::Level::parse(&archive).expect("level");
    let &(m, o) = level.by_hash.get(&MUZZLE).expect("smoke_1_dust resolves in 07000004");
    let obj = &level.maps[m].objects[o];
    let batches: Vec<_> = obj.gfx.leaves.iter().flat_map(|l| &l.batches).collect();
    assert_eq!(batches.len(), 1);
    let b = batches[0];
    assert_eq!(b.verts.len(), 4);
    assert_eq!(b.alpha & 0xff, 0x48, "ALPHA_1 A=0 B=2 C=0 D=1: additive");
    assert_eq!(b.zbuf >> 32 & 1, 1, "ZMSK: no depth write");
    let ext = b.verts.iter().fold([0.0f32; 2], |e, v| [e[0].max(v.pos[0].abs()), e[1].max(v.pos[1].abs())]);
    assert!((ext[0] - 2.8588).abs() < 1e-3 && (ext[1] - 2.9926).abs() < 1e-3, "the quad's half extents {ext:?}");
    for v in &b.verts { assert_eq!(v.rgba, [254, 254, 254, 128]); }
    let d = nf_view::scene::Draw::from_gs(5, 0, b.alpha, b.test, b.zbuf);
    assert_eq!(d.blend, 1, "the port decodes 0x48 as additive");
    assert!(!d.zwrite, "the port decodes ZMSK as no depth write");
    // Sister: the normal-alpha word decodes differently.
    assert_ne!(nf_view::scene::Draw::from_gs(5, 0, 0x44, b.test, b.zbuf).blend, 1);
}
