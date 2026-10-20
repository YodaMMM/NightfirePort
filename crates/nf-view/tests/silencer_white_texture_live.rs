//! silencer-white-artifact: the suppressed weapon's suppressor mesh drew TWICE on the silenced
//! twin's view model — once as the fp skin's rigid extra part (bone 43) and once as the record's
//! +0x7C datum-0 attach (fix-silencer-visual) — and both copies carry the mesh's ADDITIVE sheen
//! batch (ALPHA_1 0x7f00000048 [D 07000005.bin map 45 obj 3 0x02000842 leaf 0 batch 1]), so the
//! two adds stacked into the user's "weird white thing" (2026-10-19).
//!
//! The game's chain [M]: `Player_WeaponFiring` 0x1a5460:96-110 attaches the +0x7C mesh at DATUM 0
//! every tick while the held record's flags (+0x68) carry 0x800 (the twin arm 0x1a5460:111-118
//! clears datum 0 for the bare record) — and the function opens by hiding the weapon-slot objects
//! every tick (32 `View_SetDrawInNoViews` calls over BL+0x7ec..+0x86c and +0x86c; the slots come
//! from `Player_InitWeapon` 0x1a7ef8). The fp skin carries the SAME mesh as a rigid extra on bone
//! 43 [D 07000005.bin: skin 05000088 extra bone 43 = 0x02000842 'Suppressor'; likewise
//! 05000047→0x020001f2, 05000050→0x020005b2, 050000a8→0x0200099d, 050000a9→0x020009a3], and
//! `AnimObjectDraw` 0x1c4490 draws only the main skinned list (+0x2c/+0x4) and the datum entities
//! (+0x2f/+0x20) — never the extra glists (AnimProcessSkinData 0x1bd500 puts them at +0x8, which
//! only `AnimObjectNew` 0x1be910:151-172's child objects read). The fix skips the skin extra whose
//! hash is the record's +0x7C: the datum attach is the mesh's only draw — the bare twin stays bare,
//! the silenced twin's sheen adds once.
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str) -> nf_view::scene::Scene {
    // The P2K pair's assets merge like the dev give does; 05 carries them natively either way.
    nf_view::load_with_carry_weapons(iso, "07000005.bin", false, None, &[2, 3]).expect("load 07000005")
}

fn settle(scene: &mut nf_view::scene::Scene) {
    for _ in 0..600 { if !scene.sim.as_ref().unwrap().disabled { break; } scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "the intro never released Bond");
}

/// Presses the mode button (the real B press) with the machine idle, up to 3 attempts — fire_modes3's
/// harness verbatim.
fn mode_press(scene: &mut nf_view::scene::Scene) -> u8 {
    for _ in 0..3 {
        for _ in 0..40 {
            let sim = scene.sim.as_ref().unwrap();
            if sim.gun.switch_state == 0 && sim.gun.reload_state == 0 && !sim.gun.fire_state { break; }
            scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        }
        scene.sim.as_mut().unwrap().step_bond(Input { mode_switch: true, ..Default::default() }, false, false, false);
        for _ in 0..200 {
            scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
            let sim = scene.sim.as_ref().unwrap();
            if sim.gun.switch_state == 0 && !sim.gun.fire_state { break; }
        }
        let w = scene.sim.as_ref().unwrap().gun.weapon;
        let idle = { let sim = scene.sim.as_ref().unwrap(); sim.gun.switch_state == 0 && !sim.gun.fire_state };
        if idle { return w; }
    }
    scene.sim.as_ref().unwrap().gun.weapon
}

/// Rigid attachments of a view-model character as (bone, local) content — fix_silencer_visual's
/// `rigid_content` (no sim-driven kinds; the extras and the silencer are this set).
fn rigid_content(scene: &nf_view::scene::Scene, vm: usize) -> Vec<(usize, [[f32; 4]; 4])> {
    let mut v: Vec<(usize, [[f32; 4]; 4])> = scene.attachments.iter().filter(|a| {
        a.character as usize == vm
            && a.flash.is_none() && a.tracer.is_none() && a.projectile.is_none()
            && !a.dropped && a.script.is_none() && a.aims_light.is_none() && a.gas.is_none() && a.effect.is_none()
    }).map(|a| (a.bone, a.local)).collect();
    v.sort_by_key(|(b, _)| *b);
    v
}

/// The suppressor mesh draws ONCE on the silenced twin: the skin's bone-43 extra copy is gone from
/// BOTH twins (the game hides the weapon-slot objects every tick [M 0x1a5460] and owns the mesh via
/// the datum-0 attach), the datum-0 entry (bone 38, the datum's q/t) stays on the silenced twin only,
/// and the body extras (bones 38-42) are untouched on both — the sister set.
#[test]
fn the_silenced_p2k_draws_the_suppressor_mesh_once() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso);
    settle(&mut scene);
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(sim.gun.weapon, 2, "Bond starts on the bare P2K [L the 05 spawn]");
    let (vm2, vm3) = (*scene.view_models.get(&2).expect("the P2K view model"),
                      *scene.view_models.get(&3).expect("the silenced twin's view model"));
    let dt0 = scene.characters[vm3].skin.datums.iter().find(|d| d.id == 0 && d.bone >= 0)
        .expect("the P2K fp skin has a datum 0").clone();

    let w = mode_press(&mut scene);
    assert_eq!(w, 0x03, "the mode press landed the silenced twin");

    // The datum-0 attach is the suppressor's only draw: no bone-43 attachment (the skin extra's copy,
    // ident local on the suppressor bone [D skin 05000088 extra bone 43]) survives on either twin.
    // FAILING-FIRST: on the pre-fix tree both view models carry the bone-43 extra.
    let bare = rigid_content(&scene, vm2);
    let silenced = rigid_content(&scene, vm3);
    assert_eq!(bare.iter().filter(|&&(b, _)| b == 43).count(), 0,
        "no bone-43 attachment on the bare twin's view model (the game hides the weapon-slot objects [M 0x1a5460])");
    assert_eq!(silenced.iter().filter(|&&(b, _)| b == 43).count(), 0,
        "no bone-43 attachment on the silenced twin's view model (the extra copy doubled the additive sheen)");

    // The datum-0 entry: on the silenced twin only, at the datum's bone with the datum's q/t [M
    // AnimDatumSetEntity(vm, 0, +0x7C) 0x1a5460:107 → AnimObjectDraw's QuatTransToMat].
    let dt_entry = silenced.iter().filter(|&&(b, l)| b == dt0.bone as usize && l == nf_formats::anim::quat_trans_to_mat(dt0.q, dt0.t)).count();
    assert_eq!(dt_entry, 1, "the silenced twin carries the datum-0 suppressor attach");
    assert_eq!(bare.iter().filter(|&&(b, l)| b == dt0.bone as usize && l == nf_formats::anim::quat_trans_to_mat(dt0.q, dt0.t)).count(), 0,
        "the bare twin stays bare (the twin-clear arm [M 0x1a5460:111-118])");

    // Sister: the body extras (Grip/Slide/hammer/Trigger/Clip on bones 38-42, ident local) are
    // unchanged on BOTH twins — the fix touched only the +0x7C mesh's extra copy.
    for vm in [vm2, vm3] {
        let content = rigid_content(&scene, vm);
        for bone in 38..=42usize {
            assert!(content.iter().any(|&(b, _)| b == bone),
                "bone {bone}'s body part still draws on view model {vm} (the fix skipped only the +0x7C mesh)");
        }
    }
}

/// The suppressor mesh's blend modes and texture resolution [D 07000005.bin map 45 obj 3]: batch 0
/// (the tube) ALPHA_1 0x44 normal, batch 1 (the sheen) ALPHA_1 0x7f00000048 additive with no depth
/// write — and the mesh's two textures (map 45 indices 0 and 3) resolve into the scene, so the view
/// model keeps no white-fallback material: the user's white thing was the doubled additive sheen,
/// not a missing texture.
#[test]
fn the_suppressor_mesh_batches_carry_the_games_blend_modes_and_resolve() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    // Decode the mesh straight from the archive [D]: the level parse gives map 45's object 3.
    let scene = load(&iso);
    // The scene's texture set holds the suppressor mesh's maps (the load merges the weapon maps; 05
    // carries map 45 natively [D]). No material falls back to the white 1×1 texture.
    let untextured = scene.materials.iter().filter(|m| m.key.is_none()).count();
    assert_eq!(untextured, 0, "no material falls back to the white 1×1 texture on 07000005");
    // The additive sheen exists as blend-1 material(s) in the view-model pass 7 [D batch 1 alpha
    // 0x7f00000048 → Draw::from_gs blend 1]. Pre-fix the doubled copies made TWO overlapping adds;
    // the decode itself was never wrong, so the pin is the state, not a count.
    let additive_vm = scene.materials.iter().filter(|m| m.draw.pass == 7 && m.draw.blend == 1).count();
    assert!(additive_vm >= 1, "the view model keeps its additive sheen material(s) (blend 1, pass 7)");
    // The subtractive/normal mix of the view model stays as decoded (sister: the body parts' normal
    // blend materials exist — the fix removed a mesh, not a blend mode).
    let normal_vm = scene.materials.iter().filter(|m| m.draw.pass == 7 && m.draw.blend == 0).count();
    assert!(normal_vm >= 1, "the view model keeps its normal-blend body materials");
}
