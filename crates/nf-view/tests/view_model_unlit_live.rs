//! view-model tagging: the first-person view model's draws carry `VIEW_MODEL_NODE` (the list-2 pass,
//! drawn after `psiClearZ`). light-viewmodel corrected the old premise that it takes no point lights:
//! `Game_Draw` 0x1e6f50:150 zeroes the slots before list 2, but `AnimObjectDraw` 0x1c4490:117-122
//! re-picks them through `View_SetupRenderModes` 0x1e78b0 (the object's sphere) or
//! `View_SetupRenderModes2` 0x1e7908 (the viewer's camera, radius 2.0) — the renderer lights the
//! tagged draws from the eye. The tag must stay off World draws (node drives the World cel cull).
//! Opt-in: set `NF_ISO` to the disc image.
use nf_view::scene::{Layer, VIEW_MODEL_NODE};

#[test]
fn the_view_model_draws_are_tagged_and_nothing_else_is() {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let scene = nf_view::load_weapons(&iso, "07000008.bin", false, &[0x2d]).expect("load 08");
    let tagged: Vec<_> = scene.materials.iter().filter(|m| m.node == VIEW_MODEL_NODE).collect();
    assert!(!tagged.is_empty(), "the view model's draws carry VIEW_MODEL_NODE");
    assert!(tagged.iter().all(|m| m.layer != Layer::World), "no world draw is tagged (node drives World cel culling)");
    // Sister: the level's other characters (guards, Bond's third-person body) stay untagged and keep their lights.
    let other_chars = scene.materials.iter().filter(|m| m.layer == Layer::Character && m.node != VIEW_MODEL_NODE).count();
    assert!(other_chars > 0, "the non-view-model characters keep untagged draws");
}
