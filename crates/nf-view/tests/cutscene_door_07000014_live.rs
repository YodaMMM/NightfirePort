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

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

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
