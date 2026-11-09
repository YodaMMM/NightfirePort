//! debris-fade-draw: a debris piece draws with its alpha byte obj+0x106 as the draw's alpha
//! factor, so it fades over its life instead of drawing solid until it vanishes.
//!
//! The game [M]: `View_DrawObjList` pass 0 calls `View_SetupRenderModes_Tweak(viewer, obj)` per
//! object (0x1e69a0:77), which passes `psiSetTweakARGB(A = obj+0x106, R, G, B)` (0x1e5888:46-49);
//! `psiSetTweakARGB` 0x124770 stores TAmbient = (R, G, B, A)/255, the VU1 lighting matrix's ambient
//! term (`FillMatrixChainRot` 0x122310:241). The debris byte starts at 0xff (`Debris_CreateEx`
//! 0x1ac808:27) and `Debris_Update` 0x1aac00:18-26 steps it down one per tick. No night-sight
//! quarter: that needs obj+0xF0 & 0x20000 (0x1e5888:12-17); the creators OR 0x20000020
//! (0x1ac808:45, 0x1aaa00:53) and no debris mesh's glist flags carry 0x20000 [D every level].
//! Every debris mesh batch blends (PRIM ABE set, ALPHA_1 0x44 or 0x7f00000048) [D every level],
//! so the factor reaches the pixel; the port carries it as the attachment colour slot's alpha
//! (the GPU palette) and the posed vertex alpha (the CPU path) — the gas / swoosh / script
//! entities' same factor. RGB stays the white tweak (`debris_shading_live`).
//!
//! Real path: the level loads, Bond stands at the level's own MGL pickup facing the storeroom's
//! steel shelf, one contact round fires through the real fire input, the sim's ticks age the pieces.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Loads 07000011, fires one MGL round at the shelf and walks `ticks` ticks after the trigger.
fn shelf_blast(ticks: u32) -> nf_view::scene::Scene {
    let iso = iso().unwrap();
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load 07000011");
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let at = scene.sim.as_ref().unwrap().drones.pickups.iter().find(|p| p.weapon == 0x2a)
        .expect("07000011 places an MGL").start;
    scene.sim.as_mut().unwrap().place_bond(at, 0.35);
    {
        let sim = scene.sim.as_mut().unwrap();
        let w = 0x2ausize;
        if w < sim.drones.inventory.owned.len() { sim.drones.inventory.owned[w] = true; }
        sim.drones.inventory.clip[w] = 6;
        sim.gun.weapon = w as u8;
        sim.gun.switch_request = w as u8;
        sim.gun.switch_state = 0;
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.gun.reload_state = 0;
        sim.gun.fire_state = false;
        sim.gun.clip = 6;
        sim.gun.pending = 0;
    }
    let mut input = Input::default();
    input.fire = true;
    for t in 0..ticks {
        scene.sim.as_mut().unwrap().step_bond(if t == 0 { input } else { Input::default() }, t == 0, false, false);
    }
    scene
}

/// The debris attachments whose slot holds a live piece of that mesh, with the piece's byte.
fn live_debris(scene: &nf_view::scene::Scene) -> Vec<(usize, u8)> {
    let s = scene.sim.as_ref().unwrap();
    let map = nf_game::effect::debris_slot_map(&s.drones.effects.debris, &s.drones.breaks.shatter.debris, &s.drones.blast_debris);
    scene.attachments.iter().enumerate().filter_map(|(k, a)| {
        let Some((2, slot, mesh)) = a.effect else { return None };
        let p = match map.get(slot).copied().flatten()? {
            (0, i) => s.drones.effects.debris.get(i).map(|d| (d.mesh, d.alpha)),
            (1, i) => s.drones.breaks.shatter.debris.get(i).map(|d| (d.model, d.fade)),
            (_, i) => s.drones.blast_debris.get(i).map(|d| (d.model, d.fade)),
        }?;
        (p.0 == mesh).then_some((k, p.1))
    }).collect()
}

#[test]
fn a_late_blast_piece_draws_at_its_alpha_byte_through_the_palette_and_the_cpu_path() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    // 100 ticks after the trigger: the pieces (spawned ~t17..t35, life Rand(90)+90) are mid-fade.
    let scene = shelf_blast(100);
    let live = live_debris(&scene);
    assert!(!live.is_empty(), "the MGL blast's metal pieces are still alive at t100");
    for &(_, b) in &live {
        assert!(b < 0xff - 50 && b > 1, "a t100 piece has aged at least 50 ticks off 0xff, got {b:#x}");
    }
    // The GPU palette (the path the window draws with): the slot's colour alpha = byte / 255,
    // RGB the white tweak.
    let (layout, _attrs) = scene.skin_layout();
    let (_pal, cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    for &(k, b) in &live {
        let c = cols[layout.col_att as usize + k];
        assert_eq!([c[0], c[1], c[2]], [1.0; 3], "debris slot {k} keeps the white tweak RGB [M 0x1e5888:46-49]");
        assert!((c[3] - b as f32 / 255.0).abs() < 1e-6,
            "debris slot {k}'s draw alpha = obj+0x106 / 255 = {} [M 0x1e5888:46-49, 0x124770], got {}", b as f32 / 255.0, c[3]);
    }
    // The CPU path (`posed_vertices`): each of the slot's vertices carries rgba.a × byte / 255.
    let posed = scene.posed_vertices(None, 240.0, Default::default());
    let mut checked = 0;
    for &(k, b) in &live {
        for (i, cv) in scene.char_verts.iter().enumerate() {
            if cv.attachment != Some(k as u32) { continue; }
            let base = scene.vertices[scene.char_vertex_start + i].rgba[3] as f32;
            let want = ((base * b as f32 / 255.0).round() as i32).clamp(0, 255) as u8;
            assert_eq!(posed[i].rgba[3], want, "debris slot {k} vertex {i}: alpha {base} × {b}/255");
            checked += 1;
        }
    }
    assert!(checked > 0, "the live debris slots carry vertices");
}

#[test]
fn a_fresh_piece_draws_solid_and_a_free_slot_keeps_alpha_one() {
    if iso().is_none() { eprintln!("NF_ISO not set — skipping"); return; }
    // Just after the first pieces spawn (the blast's first debris tick): bytes near 0xff.
    let mut scene = shelf_blast(1);
    let mut t = 1;
    while live_debris(&scene).is_empty() && t < 80 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        t += 1;
    }
    let live = live_debris(&scene);
    assert!(!live.is_empty(), "the blast spawned pieces by t{t}");
    let (layout, _attrs) = scene.skin_layout();
    let (_pal, cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    for &(k, b) in &live {
        assert!(b >= 0xfd, "a piece on its first ticks holds ~0xff [M 0x1ac808:27], got {b:#x}");
        assert!((cols[layout.col_att as usize + k][3] - b as f32 / 255.0).abs() < 1e-6);
    }
    for (k, a) in scene.attachments.iter().enumerate() {
        if !matches!(a.effect, Some((2, _, _))) || live.iter().any(|&(j, _)| j == k) { continue; }
        assert_eq!(cols[layout.col_att as usize + k][3], 1.0, "a debris slot with no live piece of its mesh keeps alpha 1");
    }
}
