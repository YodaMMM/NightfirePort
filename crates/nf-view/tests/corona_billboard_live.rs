//! Level coronas draw camera-facing. `Corona_Create` 0x18da00 copies the glist flags word to obj+0xF0
//! [M 0x18da40-48]; every placed corona glist carries bit 0x8 (07000005 Corona_Glow2 / Corona_test1
//! 0x22d, Window_corona 0x229 [D]; the other levels in the second test), and `View_DrawObjectGlist` 0x1e6780 draws an 0x8 object with the
//! viewer's rotation: `Mat_CopyRot(viewer+0x1a0)` + `Mat_SetTrans(obj+0xC0)` [M 0x1e67a4-0x1e6810].
//! The port drew them at their placement matrix, so the 07000005 start's bunker glow (placement 14)
//! and gate-arch glow (placement 15) sat in their authored planes, nearly edge-on to the start view,
//! and read as dim slivers next to the PS2's bright pools. Opt-in: NF_ISO.

use nf_view::scene::ViewOpts;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The bunker corona's drawn width across the view, and its sister: the placement-matrix draw (the old
/// path) measured the same way is far narrower, so the comparison can fail.
#[test]
fn the_05_start_coronas_face_the_camera() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    // 07000005's 13 type-0xFC placements all carry 0x8 [D] → 13 billboard slots.
    assert_eq!(scene.corona_pos.len(), 13, "every 07000005 corona rides a billboard slot");
    let bunker = [90.74978f32, 23.47763, 106.80527]; // placement 14, Corona_Glow2 [D]
    let ci = scene.corona_pos.iter().position(|p| (0..3).all(|k| (p[k] - bunker[k]).abs() < 1e-4))
        .expect("the bunker corona (placement 14) has a slot");
    let slot = scene.corona_slot0 + ci as u16;
    // A camera at the 07000005 start eye looking at the bunker corona.
    let eye = [104.954f32, 26.5, 108.82527];
    let d = [bunker[0] - eye[0], bunker[1] - eye[1], bunker[2] - eye[2]];
    let n = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); [v[0] / l, v[1] / l, v[2] / l] };
    let f = n(d);
    let right = n([f[2], 0.0, -f[0]]);
    let up = [right[1] * f[2] - right[2] * f[1], right[2] * f[0] - right[0] * f[2], right[0] * f[1] - right[1] * f[0]];
    let opts = ViewOpts { right, up, ..Default::default() };
    let width_along = |pts: &[[f32; 3]]| {
        let s: Vec<f32> = pts.iter().map(|p| p[0] * right[0] + p[1] * right[1] + p[2] * right[2]).collect();
        s.iter().cloned().fold(f32::MIN, f32::max) - s.iter().cloned().fold(f32::MAX, f32::min)
    };
    // The CPU path.
    let verts = scene.posed_vertices(None, 0.0, opts);
    let idx: Vec<usize> = scene.char_verts.iter().enumerate().filter(|(_, v)| v.searchlight == Some(slot)).map(|(i, _)| i).collect();
    assert!(!idx.is_empty(), "the bunker corona has vertices on its slot");
    let drawn: Vec<[f32; 3]> = idx.iter().map(|&i| verts[i].pos).collect();
    let w_drawn = width_along(&drawn);
    // Corona_Glow2's quad is 2 × 1.7287612 wide [D]; facing the camera the full width shows.
    assert!((w_drawn - 2.0 * 1.7287612).abs() < 0.01, "the billboard shows the full quad width: {w_drawn}");
    // Every drawn vertex lies in the plane through the corona facing the camera (depth spread ≈ 0).
    let depth: Vec<f32> = drawn.iter().map(|p| (p[0] - bunker[0]) * f[0] + (p[1] - bunker[1]) * f[1] + (p[2] - bunker[2]) * f[2]).collect();
    assert!(depth.iter().all(|z| z.abs() < 0.01), "the quad faces the camera: {depth:?}");
    // The GPU palette slot carries the same billboard.
    let (l, _) = scene.skin_layout();
    let (pal, _) = scene.skin_palette(&l, None, 0.0, opts, None);
    let m = pal[(l.sl_base + slot as u32) as usize];
    for k in 0..3 { assert!((m[0][k] - right[k]).abs() < 1e-5 && (m[1][k] - up[k]).abs() < 1e-5, "palette rows = the camera axes: {m:?}"); }
    for k in 0..3 { assert!((m[3][k] - bunker[k]).abs() < 1e-4, "palette translation = the placement: {m:?}"); }
    // Sister: the placement-matrix draw (the old static path) seen from the same camera.
    let rot = [0.0f32, -0.57524276, 0.0]; // placement 14's rotation [D]
    let pm = nf_formats::placement::object_matrix(rot, bunker);
    let local: Vec<[f32; 3]> = idx.iter().map(|&i| scene.char_verts[i].pos).collect();
    let old: Vec<[f32; 3]> = local.iter().map(|&p| nf_formats::placement::transform(&pm, p)).collect();
    let w_old = width_along(&old);
    assert!(w_old < 0.6 * w_drawn, "sister: the placement-matrix quad reads foreshortened from the start ({w_old} vs {w_drawn})");
}

/// The other levels that place coronas carry 0x8 on every corona glist too (07000004 `corona` 0x29,
/// 07000015 `corona1`/`corona_small`/`corona_red` 0x2d, 07000016 `Corona_Glow2` 0x2d [D]), so every
/// placement rides a billboard slot; sister: levels without type-0xFC placements get none.
#[test]
fn every_placed_corona_gets_a_billboard_slot() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    for (lvl, n) in [("07000004.bin", 2usize), ("07000015.bin", 76), ("07000016.bin", 21), ("07000006.bin", 0), ("0700000c.bin", 0)] {
        let scene = nf_view::load(&iso, lvl, false).expect("load");
        assert_eq!(scene.corona_pos.len(), n, "{lvl}: corona billboard slots");
    }
}
