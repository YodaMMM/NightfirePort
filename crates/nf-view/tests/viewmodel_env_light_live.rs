//! viewmodel-light: the first-person weapons' chrome layers are ENVIRONMENT-MAPPED, and the view
//! model's tint follows the room Bond stands in.
//!
//! The game [M]: each batch's GIF tag carries a VU1 handler selector in the low half of its fourth
//! word (REGS fields 8-11, unused under NREG 3); the microprogram dispatch `ILW.w vi04, 0(vi01)` +
//! the per-object light/tweak mode (data 0x16.x) jumps through the table at micro 0x08-0x16
//! (.vutext micro 0x023-0x025). Selector 8 is the transform with the light/tweak variants; selector
//! 0x10 branches to micro 0x2e2 for every mode — the environment-map handler, which builds
//! ST = 0.5 + (row · n)/256 from the vertex normal with the camera rows `FillMatrixChainSkin`
//! 0x122788 / `FillMatrixChainRot` 0x122310 upload at data 0x10-0x13 (viewer +0x120 / +0x130 ×
//! 1/256, + 0.5) and colours the vertex `TAmbient ⊙ rgba` with no point lights (micro 0x304
//! `MUL vf18, vf12, vf17`). The sniper rifle's silencer is a full-[0,1]-UV additive sheen batch
//! with selector 0x10 over a dark base [D 0700000c.bin object 0x0200072c 'Case']: drawn with its
//! disc UVs it landed on the stripe texture's bright band and read near-white; the PS2 draws the
//! banded chrome (audits/render-1031-viewmodel side-by-sides).
//!
//! The tint: the view model is a 0x800 object whose obj+0x100..0x102 `Lights_CalcAmbientLight`
//! 0x1cb178 steps toward Bond's linked cel by `ClrStep(…, 0.1)` per tick; in the 0700000d ST-3
//! building that cel's ambient is (36, 36, 36) [D], so the gun settles dark there.
//!
//! Opt-in via NF_ISO.

use nf_formats::ps2gfx::{HANDLER_ENV, HANDLER_LIT};
use nf_game::player::Input;
use nf_view::scene::{ENV_OBJECT, ENV_SKINNED};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn level(iso: &str, archive: &str) -> nf_formats::level::Level {
    let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive in FILES.BIN");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    nf_formats::level::Level::parse(&data).unwrap()
}

/// Every batch of 0700000c's sniper body ('Case' 0x0200072c) is lit-handler except its additive
/// sheen batches, which are env-mapped — and the archive holds no third selector.
#[test]
fn the_sniper_sheen_batches_carry_the_env_handler() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let lvl = level(&iso, "0700000c.bin");
    let case = lvl.maps.iter().flat_map(|m| &m.objects).find(|o| o.hashcode == 0x0200_072c).expect("the sniper body 'Case' [D]");
    let (mut env_add, mut lit_normal) = (0, 0);
    for b in case.gfx.leaves.iter().flat_map(|l| &l.batches) {
        let additive = b.alpha & 0xff == 0x48;
        if additive { assert_eq!(b.handler, HANDLER_ENV, "an additive sheen batch is env-mapped"); env_add += 1; }
        else { assert_eq!(b.handler, HANDLER_LIT, "a base batch is the lit handler"); lit_normal += 1; }
    }
    assert!(env_add >= 3 && lit_normal >= 20, "both kinds present (env {env_add}, lit {lit_normal})");
    // Sister: across the archive only the two table entries the microcode uses occur, and both do.
    let mut seen = std::collections::BTreeMap::<u16, usize>::new();
    for o in lvl.maps.iter().flat_map(|m| &m.objects) { for b in o.gfx.leaves.iter().flat_map(|l| &l.batches) { *seen.entry(b.handler).or_default() += 1; } }
    assert_eq!(seen.keys().copied().collect::<Vec<_>>(), vec![HANDLER_LIT, HANDLER_ENV], "selector census {seen:x?}");
}

/// The sniper view model's sheen vertices reach the shader flagged env; its base vertices stay
/// unflagged (the sister). The body 'Case' is a RIGID piece (skin info 0 [D]) drawn through
/// `psiDrawObjectMatrix` → `DrawThisBox` → `FillMatrixChainRot`, so it feeds its object-space normal
/// (ENV_OBJECT); the skinned env meshes on the level (the hazmat heavies' skins, skinned + additive
/// [D 0700000d.bin 0x020007c7 / 0x0200070b]) take ENV_SKINNED, and no view-model vertex does.
#[test]
fn the_sniper_view_model_flags_its_sheen_vertices_env() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load(&iso, "0700000d.bin", false).unwrap();
    let vm = *scene.view_models.get(&0x1e).expect("the sniper (0x1e) view model on 0700000d");
    let (mut env, mut plain) = (0usize, 0usize);
    for m in scene.materials.iter().filter(|m| m.skin == Some((vm as u32, None))) {
        let idx = &scene.indices[m.first_index as usize..(m.first_index + m.index_count) as usize];
        for &i in idx {
            let w = scene.vertices[i as usize].nrm[3];
            if m.draw.blend == 1 { assert_eq!(w, ENV_OBJECT, "the rigid sheen layer is env-mapped in object space"); env += 1; }
            else { assert_eq!(w, 0, "a base layer keeps its disc UV and the lights"); plain += 1; }
        }
    }
    assert!(env > 0 && plain > env, "both layers drawn (env {env}, plain {plain})");
    let skinned_env = scene.vertices.iter().filter(|v| v.nrm[3] == ENV_SKINNED).count();
    assert!(skinned_env > 0, "the level's skinned env meshes carry ENV_SKINNED");
}

/// The silenced P2K (05): its sheen pieces (slide, screw-on suppressor 0x02000842 — box flag 1,
/// rigid) are env-mapped in object space too; the bare record draws no sheen batch the shader would
/// treat as skinned.
#[test]
fn the_p2k_sheen_pieces_are_rigid_env() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load_with_carry_weapons(&iso, "07000005.bin", false, None, &[2, 3]).unwrap();
    let vm = *scene.view_models.get(&3).expect("the silenced P2K (0x03) view model");
    let mut flags = std::collections::BTreeMap::<i8, usize>::new();
    for m in scene.materials.iter().filter(|m| matches!(m.skin, Some((c, _)) if c == vm as u32) && m.draw.blend == 1) {
        let idx = &scene.indices[m.first_index as usize..(m.first_index + m.index_count) as usize];
        for &i in idx { *flags.entry(scene.vertices[i as usize].nrm[3]).or_default() += 1; }
    }
    assert!(flags.contains_key(&ENV_OBJECT), "the P2K's sheen is env-mapped in object space ({flags:?})");
    assert!(!flags.contains_key(&ENV_SKINNED), "never the skinned flag on a rigid piece ({flags:?})");
}

fn settle_intro(scene: &mut nf_view::scene::Scene) {
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..340 {
        if sim.disabled { sim.drones.scripts.skip_request = true; }
        sim.step_bond(Input::default(), false, false, false);
    }
    assert!(!sim.disabled, "the intro released Bond");
}

/// The view-model tint at the 0700000d ST-3 spot (137, 5.02, −300 — the sweep's PS2 capture): once
/// the ramp has run, obj+0x100..0x102 equal the building cel's (36, 36, 36) [D] (no sun inside),
/// and the draw multiplier is (0xff × 36) >> 8 over 255 [M 0x1e5888]. Sister: one tick after the
/// move the bytes are still on their way down from the sunlit start — the comparison can fail, and
/// a shot taken there shows a brighter gun than the game settles to.
#[test]
fn the_view_model_tint_settles_to_the_st3_room() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).unwrap();
    settle_intro(&mut scene);
    let vm = *scene.view_models.get(&0x1e).expect("the sniper view model");
    let sim = scene.sim.as_mut().unwrap();
    let start = sim.char_ambient[vm];
    sim.bond = nf_game::player::Bond::stand_at(&sim.world, [137.0, 5.02, -300.0], 3.14);
    sim.step_bond(Input::default(), false, false, false);
    let one_tick = sim.char_ambient[vm];
    for _ in 0..120 { sim.step_bond(Input::default(), false, false, false); }
    let settled = sim.char_ambient[vm];
    eprintln!("start {start:?}, one tick {one_tick:?}, settled {settled:?}");
    assert_eq!(settled, [36, 36, 36], "the ST-3 cel's ambient [D]");
    let want = ((0xff * 36u32) >> 8) as f32 / 255.0;
    for k in 0..3 { assert!((sim.char_multiplier(vm)[k] - want).abs() < 1e-6, "TAmbient {:?} vs {want}", sim.char_multiplier(vm)); }
    // Sister.
    assert!(start[0] > 36 && one_tick[0] > 36, "the start room is brighter and the ramp is gradual ({start:?} → {one_tick:?})");
    assert_ne!(one_tick, settled);
}
