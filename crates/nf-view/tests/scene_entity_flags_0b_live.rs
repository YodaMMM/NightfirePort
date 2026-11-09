//! scene-entity-flags-0b: the real-input reproduction of the script-entity scene-hide gap
//! (task level-0b-scene-entity-flags). The game's law, first-hand:
//!
//! - `Script_CreateEntity` 0x1d8968 attaches the command hash's glist to every created object
//!   (typed arms through their creators, the 0x31/0xDB fallthrough through
//!   `hashtable_set_object_to_entity_gfx` 0x1ca6d8:38-40 → `Control_SetGList`).
//! - `Control_SetGList` 0x134960:11-16 copies glist+0x44 into obj+0xF0 and, when the word
//!   carries 0x40000, clears obj+0xF8's display-mask bit 0x10 (the u16 inits to 0x1F,
//!   control_init_object 0x134630:11).
//! - Both dynamic draw walks skip an object when `(obj+0xF8 & object_display_mask) == 0`
//!   [M View_AddObjects 0x1e62d8:30; View_AddForcedObjects 0x1e5fb8:79], and
//!   `View_CaptureScene` 0x1e6df0:15 sets `object_display_mask = 1 << (viewer+0x28 & 0x1f)`
//!   (the layer byte is the viewer index, build_alloc_viewer 0x1c9378:6-7). The cut-scene
//!   camera is viewer 4 (`Script_CameraStart` 0x1d9140 stores 4 into the stream's camera slot
//!   and enables only camera 4), so its captures run with `object_display_mask` 0x10.
//! - Net: an object whose glist flags carry 0x40000 is invisible to the scene camera's pass
//!   and visible in the game views (mask bits 0..3 stay set in 0x0F).
//!
//! Disc [D the sibling census `scene_entity_glist_census_0b`]: the 0b scenes' 0x40000 set is
//! EXACTLY the lift cab 0x02000367 (word 0x40080) and its glass 0x02000421 (0x40081), both
//! created by the lift ScriptPlayer #232's cycle scripts 06000048 + 06000a09 (streams 0/1,
//! type 0x31); the roof cut-scene 060004fa's NIS door 0x02000668 is glist 0x0 (draws).
//!
//! The port applies the same rule to script entities: while the scene camera renders, an
//! entity whose glist carries 0x40000 (0b's lift cab and glass) collapses its draw slots.
//!
//! This test checks it on real input: a fresh boot, walked short of the arrival plates,
//! then a per-tick real-input approach that catches the intro script's scene camera the tick it
//! engages (event 4 player-disable at the scene's tick 0 [D SP #246 = 060004f9, out ch 0x33;
//! the scene runs 690 ticks and finishes with stay → state 5, so a walk that overshoots misses
//! it]). No teleports. Opt-in via NF_ISO.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Walker, Wp};

/// All script-entity palette slots of one entity hash: `Vec<(slot translation)>` over every
/// (script, stream) draw slot the scene build created for it. The lift pair's scripts are the
/// SP #232 pair 06000048 + 06000a09 — only the CURRENT script's slots pose; the other script's
/// slots sit FAR (no live entity carries that src) — so the pin is "ANY slot posed".
fn entity_palette_slots(scene: &nf_view::scene::Scene, hash: u32) -> Vec<[f32; 4]> {
    let (layout, _attrs) = scene.skin_layout();
    let (pal, _cols) = scene.skin_palette(&layout, None, 0.0, nf_view::scene::ViewOpts::default(), None);
    scene.attachments.iter().enumerate()
        .filter(|(_, a)| matches!(a.script, Some((_, _, h, _)) if h == hash))
        .filter_map(|(i, _)| pal.get(layout.attach_base as usize + i).map(|m| m[3]))
        .collect()
}

#[test]
fn the_scene_camera_pass_keeps_drawing_the_0b_lift_cab() {
    let Ok(path) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut w = Walker::new(nf_view::load_with_carry(&path, "0700000b.bin", false, None).expect("load 0700000b"));
    // The first ch-0x33 arrival plate's centre, on the level's own data.
    let plate = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH_ONCE, 0x33)
        .expect("an arrival plate (TouchOnce out ch 0x33) exists [D #165/#247/#284]");
    // Walk to 3 u short of the plate (toward the spawn) so the approach stays real and the
    // scene catch happens on the plate tick, not after the 690-tick scene has finished.
    let spawn = w.scene.sim.as_ref().unwrap().bond.pos;
    let d = [(spawn[0] - plate[0]), (spawn[2] - plate[2])];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt().max(1e-6);
    let pre = [plate[0] + d[0] / len * 3.0, plate[1], plate[2] + d[1] / len * 3.0];
    w.walk_to(Wp::new(pre[0], pre[1], pre[2]), "short of the arrival plates");
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x33),
        "the walk stopped short of the plates (ch 0x33 still off) — the scene catch happens on the approach");
    // Per-tick real-input approach onto the plate; the scene camera engages the tick the
    // TOUCH_ONCE fires ch 0x33 (the intro script's event-4 disable at its tick 0 [D]).
    let mut cam_ticks = None;
    let mut cab_slots = None;
    let mut glass_slots = None;
    for t in 0..1200u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (plate[0] - p[0]).atan2(plate[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
                ..Default::default()
            };
            w.tick(input, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        if !sim.disabled { continue; }
        if sim.script_camera().is_some() {
            let cab = entity_palette_slots(&w.scene, 0x0200_0367);
            let glass = entity_palette_slots(&w.scene, 0x0200_0421);
            if !cab.is_empty() && !glass.is_empty() {
                cam_ticks = Some(t);
                cab_slots = Some(cab);
                glass_slots = Some(glass);
                break;
            }
        }
    }
    let t = cam_ticks.expect("the arrival cut-scene's script camera engages on the walked-in plates [D SP #246 event 4 @0]");
    let cab = cab_slots.expect("the lift cab has script-entity draw slots [M scene.rs's Cmd::Entity arm]");
    let glass = glass_slots.expect("the lift glass has script-entity draw slots [M scene.rs's Cmd::Entity arm]");
    let posed = |slots: &[[f32; 4]]| slots.iter().any(|m| m[1] > -9000.0);
    eprintln!("[0b-scene-flags] scene camera caught at approach tick {t}; cab slots {:?} glass slots {:?}", cab, glass);
    // While the scene camera renders, every cab/glass slot is collapsed (FAR, y -10000): the game's
    // display mask skips glist 0x40080/0x40081 entities there [M Control_SetGList 0x134960:14-16;
    // View_AddObjects 0x1e62d8:30; D the census].
    assert!(!posed(&cab) && !posed(&glass),
        "the lift cab/glass draw while the scene camera renders: cab {cab:?} glass {glass:?}");
}
