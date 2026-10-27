//! 993-cutscene-door: 07000014's control-room scene 0x06000833 opens its door.
//!
//! Evidence base:
//! - The scene's door is driven by the SCRIPT itself: 07800014.bin's NIS 06000833 (1400 ticks) has
//!   entity streams 10/11 — op-07 Entity hashes **0x02000891 / 0x02000892, obj_type 0xff**, no parent,
//!   keys 173..227 / 228..293 — leaf A slides x 27.7915 → 27.0697 (min at t539) and back, leaf B
//!   mirrors 27.7895 → 28.6815 (t546), both at y 1.2965 z 49.5721, closed again by t1000
//!   [D decode, host-float: `07800014.bin!06000833`, streams at pack+0xabde0].
//! - The door is the kind-0xdb placement pair **#116/#117 at (27.788, 1.203, 49.671)** — the key
//!   spot [D main-map placements, block 0x1A]. The scene's actors (skins 0x05000087/0x05000090,
//!   Rook 0x05000066) stand at (26.3/29.3, 1.08, 48.6) right behind it [D].
//! - The game drives those keys through `Script_CreateEntity` 0x1d8968's default arm: a plain
//!   object bound to the hash's gfx (`hashtable_set_object_to_entity_gfx` 0x1ca6d8), moved by
//!   `Script_Interp` case 1 (0x1d9a08:99-111 writes obj+0x90 straight from the interp result, the
//!   parent chain concats, then the tail `Concat(si+0x00, ·)` :133). The player's base si+0x00 is
//!   the IDENTITY from the camera op on: `Script_CameraStart` 0x1d9140 opens with
//!   `Mat_IdentityT()` on it — so the keys are the world matrix, no SP #276 placement compose
//!   (52.1566, 2.6365, 46.6214 [D placement]). THE PORT BUG: only the camera case skipped the
//!   base, so the door movers sat at (79.95, 3.93, 96.19) — the door never moved.
//! - The draw slots: the scene walk seeds one attachment per Entity cmd resolved through
//!   `level.by_hash` (scene.rs), so the movers draw at the sim matrix once the base is identity.
//!
//! Opt-in via `NF_ISO`.

use nf_formats::iso9660::IsoImage;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

/// The disc key endpoints of the two door leaves [D 06000833 streams 10/11].
const LEAF_A_CLOSED: f32 = 27.7915;
const LEAF_A_OPEN: f32 = 27.0697;
const LEAF_B_OPEN: f32 = 28.6815;
const DOOR_Y: f32 = 1.2965;
const DOOR_Z: f32 = 49.5721;

fn leaf_positions(sim: &nf_view::scene::Sim) -> Vec<(u32, [f32; 3])> {
    let mut out = Vec::new();
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
            if *hash == 0x0200_0891 || *hash == 0x0200_0892 {
                out.push((*hash, [matrix[3][0], matrix[3][1], matrix[3][2]]));
            }
        }
    }
    out
}

/// The scene's door leaves sit at the [D] key spot and slide apart during the scene, while the
/// scene runs (before any hand-back) — Kiko's path crosses the plane between them.
#[test]
fn the_control_room_door_leaves_slide_open_during_the_scene() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso().unwrap(), "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    // The vent-drop touch #301 fires the scene on the real path [D placements; the countdown
    // test's trigger spot].
    sim.bond.pos = [40.7, 2.6, 39.0];
    let mut closed_at_start = false;
    let mut max_open: f32 = 0.0;
    let mut at_door = false;
    for _ in 0..1100u32 {
        sim.step_bond(Input::default(), false, false, false);
        let leaves = leaf_positions(sim);
        if !leaves.is_empty() && !closed_at_start {
            let a = leaves.iter().find(|(h, _)| *h == 0x0200_0891).unwrap().1;
            closed_at_start = (a[0] - LEAF_A_CLOSED).abs() < 0.02
                && (a[1] - DOOR_Y).abs() < 0.02 && (a[2] - DOOR_Z).abs() < 0.02;
            assert!(closed_at_start,
                "the leaves must start closed at the [D] key spot ({LEAF_A_CLOSED}, {DOOR_Y}, {DOOR_Z}), got {a:?}");
        }
        for (_, p) in &leaves {
            assert!((p[1] - DOOR_Y).abs() < 0.02 && (p[2] - DOOR_Z).abs() < 0.02,
                "a leaf left the [D] key spot: {p:?} — the SP #276 placement base must NOT compose \
                 into the entity streams [M Script_CameraStart 0x1d9140's Mat_IdentityT on si+0x00]");
        }
        if let (Some(&(ha, pa)), Some(&(hb, pb))) = (leaves.iter().find(|(h, _)| *h == 0x0200_0891),
                                                      leaves.iter().find(|(h, _)| *h == 0x0200_0892)) {
            let gap = pb[0] - pa[0];
            max_open = max_open.max(gap);
            at_door |= (pa[0] - LEAF_A_CLOSED).abs() < 1.5;
        }
    }
    assert!(closed_at_start, "the door entity streams never ran (no leaves at the closed key) — \
        the scene's SP #276 must mount them [D 06000833 streams 10/11]");
    // The full open gap ≈ 28.6815 − 27.0697 = 1.6118 [D keys]; pin a wide-open window.
    assert!(max_open > 1.5,
        "the door never opened: max leaf gap {max_open:.4} vs the [D] keys' 1.6118 (A→{LEAF_A_OPEN}, B→{LEAF_B_OPEN})");
    assert!(at_door, "the leaves never sat at the door plane");
    eprintln!("trace: max leaf gap {max_open:.4}");
}

/// The door leaves must still be WHERE the static door is: the movers' key spot sits on the
/// kind-0xdb placement pair #116/#117 (27.788, 1.203, 49.671 [D]) — this pins that the port's
/// scene seeds draw slots for the two hashes (the movers draw, not just move in the sim).
#[test]
fn the_door_leaves_have_draw_slots_at_the_placement() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load(&iso().unwrap(), "07000014.bin", false).expect("load 07000014");
    let slots: Vec<(u32, u8, u32, u8)> = scene.attachments.iter()
        .filter_map(|a| a.script.map(|(s, st, h, k)| (s, st, h, k)))
        .filter(|&(s, _, h, _)| s == 0x0600_0833 && matches!(h, 0x0200_0891 | 0x0200_0892))
        .collect();
    for h in [0x0200_0891u32, 0x0200_0892] {
        assert!(slots.iter().any(|&(_, _, sh, _)| sh == h),
            "the door entity {h:#010x} has no draw slot — the mover would be invisible and the \
             static placement pair #116/#117 would read as a door that never opens [D decode; \
             Script_CreateEntity 0x1d8968's default arm binds the hash's gfx]");
    }
}

/// cutscene-door-dup: the game hides an object whose glist flags (celglist+0x44 = MapObject.flags)
/// carry 0x40000 from the CUT-SCENE camera: `Control_SetGList` 0x134960 clears obj+0xF8's bit 0x10
/// for such objects, `Script_CameraStart` 0x1d9140 renders ONLY camera 4 while a scene runs (its
/// viewer+0x28 = 4 → object_display_mask 0x10 [M View_CaptureScene 0x1e6df0 / build_alloc_viewer
/// 0x1c9378]), and both draw walks gate on `obj+0xF8 & object_display_mask` first (View_AddObjects
/// 0x1e62d8:33, View_AddForcedObjects 0x1e5fb8:79). [D] 07000014: the control-room door
/// placements at (27.788, 1.203, 49.671) ride the entity-params packs Door_Slide_3A/3B
/// (block+8 = 0x00040000), so the placed doors vanish while the scene's leaf entities draw; the
/// scene's leaf hashes 0x02000891/92 resolve to Elevator_Door_Slide_3A/3B (flags 0), which keep
/// the bit. Without the rule the port drew the closed placed leaf behind the scene's mover — the
/// "static copy" of the door.
#[test]
fn the_placed_door_hides_from_the_scene_camera_while_the_leaves_move() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso().unwrap(), "07000014.bin", false).expect("load 07000014");
    // The disc's own parse for the [D] assertions (the scene does not carry the level tree).
    let data = read_archive(&iso().unwrap(), "07000014.bin");
    let lv = nf_formats::level::Level::parse(&data).expect("parse 07000014");
    let main = lv.main_map().expect("main map");
    // The control-room pair and a pack-0x80000 door (placement #104's object, entity-params idx
    // 103 [D]) that must stay visible on every camera.
    let (mut pair, mut control) = (Vec::new(), Vec::new());
    for (i, p) in lv.maps[main].placements.iter().enumerate() {
        if p.flags & 0xff != 0xdb { continue; }
        if (p.translation[2] - 49.6707).abs() < 0.01 && (p.translation[0] - 27.7878).abs() < 0.01 { pair.push(i); }
        if (p.translation[2] - (-25.9752)).abs() < 0.01 { control.push(i); }
    }
    assert_eq!(pair.len(), 2, "the control-room door pair at (27.788, 1.203, 49.671) [D block 0x1A]");
    let glist_flags = |pi: usize| {
        let p = &lv.maps[main].placements[pi];
        assert!(!p.by_hash(), "doors place by index [D]");
        lv.maps[main].objects[p.index as usize].flags
    };
    for &pi in &pair {
        assert!(glist_flags(pi) & 0x40000 != 0,
            "the control-room pair's packs (Door_Slide_3A/3B) carry glist flag 0x40000 [D idx 116/117]");
        assert!(scene.door_scene_hidden.contains(&pi), "the pair is scene-hidden [M Control_SetGList 0x134960]");
    }
    for &pi in &control {
        assert!(glist_flags(pi) & 0x40000 == 0,
            "the corridor door's pack (idx 103) carries 0x80000, not 0x40000 [D]");
        assert!(!scene.door_scene_hidden.contains(&pi), "the corridor door draws on every camera");
    }
    // The pair's door slots.
    let pair_slots: Vec<usize> = nf_game::door::door_placements(&lv.maps[main].placements).into_iter()
        .enumerate().filter(|(_, pl)| pair.contains(pl)).map(|(di, _)| di).collect();
    assert_eq!(pair_slots.len(), 2, "both pair leaves ride door slots");
    let far = |m: [[f32; 4]; 4]| m[3][1] < -9999.0;
    let (layout, _attrs) = scene.skin_layout();
    let slot_pose = |scene: &nf_view::scene::Scene, di: usize| {
        let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
        pal[layout.door_base as usize + di]
    };
    // Before any scene: the placed doors draw (closed) — normal play is unaffected.
    assert!(!far(slot_pose(&scene, pair_slots[0])) && !far(slot_pose(&scene, pair_slots[1])),
        "the closed placed pair draws before the scene (camera 0, mask 0x01) [M View_AddForcedObjects 0x1e5fb8:79]");
    // Fire the scene on the real path (the vent-drop touch #301) and wait for the script camera.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [40.7, 2.6, 39.0];
        let mut fired = false;
        for _ in 0..1100u32 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.script_camera().is_some() { fired = true; break; }
        }
        assert!(fired, "the control-room scene never started (no script camera) — the vent-drop trigger [D placement #301]");
    }
    // While the script camera renders (mask 0x10): the placed pair collapses, and the scene's own
    // leaf movers keep animating (exactly one door draws, and it moves with the scene).
    for &di in &pair_slots {
        assert!(far(slot_pose(&scene, di)),
            "the placed door slot must collapse while the script camera renders [M Control_SetGList 0x134960 → \
             obj+0xF8 bit 0x10 clear; View_AddObjects 0x1e62d8:33]");
    }
    // The leaves' open keys sit at scene ticks 539-546 of 1400 [D 06000833 streams 10/11] — watch
    // for the whole opening window, not just the first ticks after the camera op.
    let mut max_gap = 0.0f32;
    for _ in 0..1000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let leaves = leaf_positions(scene.sim.as_ref().unwrap());
        if let (Some(&(_, a)), Some(&(_, b))) = (leaves.iter().find(|(h, _)| *h == 0x0200_0891),
                                                  leaves.iter().find(|(h, _)| *h == 0x0200_0892)) {
            max_gap = max_gap.max(b[0] - a[0]);
            if max_gap > 0.5 { break; }
        }
    }
    assert!(max_gap > 0.5, "the scene's leaf movers must keep animating while the placed pair is hidden (max gap {max_gap:.4}, the [D] open gap 1.6118)");
}
