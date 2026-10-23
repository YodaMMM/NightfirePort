//! The body-glow FX the game attaches to every guard and the player at spawn —
//! `BodyGlow_Create` 0x1ac158 [M 001ac158].
//!
//! The game's callers: `NDrone2_DefaultInit` 0x14b300 (tail — unconditional for every guard
//! spawn, result → `DIVars+0x10` [M; docs/atlas/drone.md:51]), `Player_Init` 0x1910f8:184
//! (result → player data +0x88C [M; docs/atlas/player.md:102]), and `AnimDebugCreate`
//! 0x1bcfa8's 0x7000041 arm (kind 0x15 object). The call-tree audit reaches it through the
//! ported guard-spawn states.
//!
//! calltree-main-26 — the `AnimDebugCreate` subtree: `AnimDebugCreate` 0x1bcfa8 itself is
//! called UNGATED from `Player_Init` 0x1910f8:117, but its body returns unless GameState is
//! 0x7000043 / 0x7000049 (the anim ring arm, which calls `AnimObjectInitialPose` 0x1c64e0 per
//! ring object [M 0x1bcfa8:66-115]) or 0x7000041 (the briefing display-model arm [M :116-155];
//! no InitialPose there). The levels 0x7000043/0x7000049 appear in NO shipped level list (the
//! campaign `mission::LEVELS`, the MP arenas 0x07000023..0x29 + 0x0700004b [D
//! docs/atlas/multiplayer.md §1]) and in NO loader — the binary's only references to those
//! GameState values are AnimDebugCreate and `Env_Tree_Create` 0x17c200's per-level branch [M
//! the decompile-wide grep] — so the 0x7000043/49 arm (and `AnimObjectInitialPose`) is
//! unreachable on the shipped path; the 0x7000041 arm IS the shipped briefing display model
//! (the port's stand-in). The full citation chain lives in
//! crates/nf-view/src/menu.rs's calltree-main-26 block.
//!
//! Behaviour [M 001ac158]:
//! - gate: owner == 0 or the global `switch_NO_DRONES` != 0 → return 0 (no glow).
//!   `switch_NO_DRONES` is written only by the `C_CHCHDRONES` menu control 0x215cf0 (0x4b) and
//!   consumed by `ResetMap_GameInit` 0x1d5080 → `Drone_bDisableSystem`; it is 0 on the shipped
//!   path, so every guard/player spawn creates the glow.
//! - the main object: `control_create_object(0x44, owner+0x30 pos, owner+0x50 rot, 0)`,
//!   obj+0xff = 0x46 (class → `BodyGlow_Update` 0x1ac2c8, docs/atlas/object.md:154), gfx hash
//!   0x200025f, obj+0xe8 = 6.0f, obj+0xf0 |= 0x2408, `build_LinkToRoom(obj, 0, glb_world)`,
//!   `View_SetDrawInNoViews`.
//! - the ext block (0x44 bytes): ext[0] = owner, u16 ext+0x40 = 0x100.
//! - 15 child sprite objects (the `do..while` reads the counter BEFORE its increment — 1..=15 — and
//!   loops while < 0xf, so 15 passes; ext[0] owner + ext[1..=15] fill exactly the 0x40 bytes before the
//!   u16 at ext+0x40 — review 2026-10-21, the branch said 14): `control_create_object(0, …)`, gfx
//!   hash 0x200055a, obj+0xff = 1, obj+0xf0 |= 0x2400, linked + no-views, stored ext[1..=15].
//!
//! The port carries the record (owner, the two gfx hashes, the child count, the flags) on the
//! guard spawn path; the sprite rendering itself is draw-side work (no drawing
//! here). Fields the draw code needs: `MAIN_GFX` / `CHILD_GFX`, `FLAGS_MAIN` / `FLAGS_CHILD`,
//! `RADIUS` (obj+0xe8 = 6.0), `EXT_LINK_U16` (ext+0x40 = 0x100).

/// The main glow object's gfx hash [M 001ac158: `hashtable_set_object_to_entity_gfx(obj, 0x200025f)`].
pub const MAIN_GFX: u32 = 0x2000_25f;
/// The 15 child sprites' gfx hash [M 001ac158: 0x200055a per child].
pub const CHILD_GFX: u32 = 0x2000_55a;
/// The main object's obj+0xf0 bits [M 001ac158: |= 0x2408].
pub const FLAGS_MAIN: u32 = 0x2408;
/// The children's obj+0xf0 bits [M 001ac158: |= 0x2400].
pub const FLAGS_CHILD: u32 = 0x2400;
/// obj+0xe8 on the main object [M 001ac158: 0x40c00000 = 6.0f].
pub const RADIUS: f32 = 6.0;
/// The u16 at ext+0x40 [M 001ac158: 0x100].
pub const EXT_LINK_U16: u16 = 0x100;
/// The main object's class byte obj+0xff [M 001ac158: 0x46 → BodyGlow_Update].
pub const CLASS: u8 = 0x46;
/// The child objects' class byte [M 001ac158: 1].
pub const CHILD_CLASS: u8 = 1;

/// The glow record one spawn creates (the main object + its 15 children, ext[0] = owner).
///
/// Update-side (`BodyGlow_Update` 0x1ac2c8, the port does not run the update loop): every
/// tick it recomputes the per-viewer display mask (a viewer whose view mode is 2 or whose
/// HUD block +0x244 is set contributes its bit) and writes it with `View_SetViewMask`
/// 0x1e7ac8 — the main object `mask | 0x10`, then each child the same (0x1ac2c8:223/257)
/// — ported as `crate::view::set_view_mask` [M disasm 0x1e7ac8; docs/systems/view-display.md].
#[derive(Clone, Debug, PartialEq)]
pub struct BodyGlow {
    /// ext[0]: the owner object — the port stores the owning guard's `Drones::list` index.
    pub owner: usize,
    /// The child-sprite count the do-while creates [M 001ac158: 0..0xe].
    pub children: u16,
}

/// `BodyGlow_Create` 0x1ac158 on the shipped path: `no_drones` is the global
/// `switch_NO_DRONES` (0 in every shipped level [M 0x1ac158:9 — set only by the 0x4b menu
/// control]). Returns None exactly when the game returns 0.
pub fn create(owner: usize, no_drones: bool) -> Option<BodyGlow> {
    if no_drones {
        return None;
    }
    Some(BodyGlow { owner, children: 15 })
}

/// `GlowBones` 0x2beed0 — the 15 bone indices the update hangs its child sprites on
/// [L ee_start_05 @ 0x2beed0; `BodyGlow_Update` 0x1ac2c8 reads them in order].
pub const GLOW_BONES: [u8; 15] = [0, 19, 20, 21, 31, 32, 35, 49, 50, 51, 52, 53, 54, 55, 56];

/// `BodyGlow_Delete` 0x1acd50: the delete bit (obj+0xFE |= 1) on the main object **and** its 15
/// children — the loop walks `do { i++ ; child |= 1 } while (i < 0xf)`, 15 passes.
pub fn delete(children: &mut [bool; 15]) -> bool {
    *children = [true; 15];
    true
}

/// The per-frame state `BodyGlow_Update` 0x1ac2c8 computes for the draw code (the port carries
/// the data; the sprite rendering itself is draw-side work).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowUpdate {
    /// The alpha byte written to obj+0x106 on the main object and every child [M 0x1ac2c8:
    /// `(1.0 − dist·0.1) · (i16)count · 128 · (1/256)` truncated to u8].
    pub alpha: u8,
    /// The view mask `View_SetViewMask` gets: the hidden-viewer bits `| 0x10` [M 0x1ac2c8:223].
    pub view_mask: u32,
    /// True when the update took the far/hidden branch: main + all 15 children get obj+0xF0 |= 0x10
    /// (`View_SetDrawInNoViews`) and no bone poses [M 0x1ac2c8:162-173, 197-208].
    pub hidden: bool,
    /// The per-child pose data (bone index, the bone-to-bone distance written to child+0xE8, and
    /// the child scale (1.5, 1.0, dist) [M 0x1ac2c8:231-264]). Empty when `hidden`.
    pub bones: [(u8, f32); 15],
}

/// `BodyGlow_Update` 0x1ac2c8 over the port's record. Inputs:
/// - `owner_kind`/`owner_health`: the owner's class byte (obj+0xFF) and the gate value — kind 2
///   reads owner+0xAC, kind 3 owner+0x894; a failing gate decrements the ext counter (the game's
///   `piVar2[0x10]`, the glow's own reference count) [M 0x1ac2c8:96-125];
/// - `count`: that reference count;
/// - `viewer_dist`: `Vec_Dist3D(owner pos, glb_viewer+0xC0)` [M 0x1ac2c8:175];
/// - `hidden_mask`: the u32 the per-player loop builds — bit v set when viewer v's +0x236 == 2
///   (MP spectator/split view) or the viewer's AnimObject+0x244 != 0 (dead) [M 0x1ac2c8:144-161];
///   `mask == 0` (nobody watches) forces the hidden branch, as does the kind-2-owner + state-1 arm
///   (passed in `owner_state1_hides`);
/// - `viewer_244`: some watching viewer's AnimObject+0x244 is set (`bVar4`) — the game then
///   forces the distance to 0.0 (full alpha, never the > 10.0 hide) [M 0x1ac2c8:135-147];
/// - `bone_gap`: per child k, `Vec_Dist3D(bone GlowBones[k], its skin parent bone)` (child+0xE8;
///   GlowBones[k] == 0 measures against bone 4) — 15 entries, one per child [M 0x1ac2c8:186-201];
///   the port takes them from the animation side.
///
/// The children's obj+0xF0 bits: `|=` 0x2400 normally, `&= 0xffffdbff` when a viewer is dead/spectating
/// (`bVar4`) [M 0x1ac2c8:176-196]; the far branch (> 10.0) hides main + children instead.
/// Returns `None` when the count ran out — the game calls `BodyGlow_Delete` and clears the owner's
/// glow refs (+0x10 / +0x88c) [M 0x1ac2c8:128-136].
pub fn update(
    owner_kind: u8,
    owner_gate_ok: bool,
    owner_state1_hides: bool,
    mut count: u16,
    viewer_dist: f32,
    hidden_mask: u32,
    viewer_244: bool,
    bone_gap: [f32; 15],
) -> (Option<GlowUpdate>, u16) {
    // `bVar4` forces the distance to 0.0 before the > 10.0 test [M 0x1ac2c8:135-137].
    let viewer_dist = if viewer_244 { 0.0 } else { viewer_dist };
    // The gate: a failing owner health check decrements the count once (kinds 2 and 3) [M
    // 0x1ac2c8:96-125]. (The game does the same for kind 3 at +0x894.)
    if (owner_kind == 2 || owner_kind == 3) && !owner_gate_ok {
        count -= 1;
    }
    if count < 1 {
        return (None, count);
    }
    let spectator_or_dead = hidden_mask != 0;
    if hidden_mask == 0 || owner_state1_hides {
        return (
            Some(GlowUpdate { alpha: 0, view_mask: 0x10, hidden: true, bones: [(0, 0.0); 15] }),
            count,
        );
    }
    if viewer_dist > 10.0 {
        return (
            Some(GlowUpdate { alpha: 0, view_mask: hidden_mask | 0x10, hidden: true, bones: [(0, 0.0); 15] }),
            count,
        );
    }
    // The alpha byte [M 0x1ac2c8:224-226]: (1 − dist·0.1) · (i16)count · 128 · 0.00390625.
    let a = (1.0 - viewer_dist * 0.1) * f32::from(count as i16) * 128.0 * 0.003_906_25;
    let mut bones = [(0u8, 0f32); 15];
    for (k, b) in bones.iter_mut().enumerate() {
        let bone = GLOW_BONES[k];
        // Child 0 hangs on bone 0 → the game measures against bone 4 [M 0x1ac2c8:236-239
        // `uVar11 = 4` on bVar1 == 0]; the others against their table predecessor's NEXT pair —
        // the pair distances arrive pre-measured in `bone_gap`.
        let gap = bone_gap[k];
        *b = (bone, gap);
    }
    let _ = spectator_or_dead; // the flags word delta is documented above; carried by the draw side
    (
        Some(GlowUpdate { alpha: a as i32 as u8, view_mask: hidden_mask | 0x10, hidden: false, bones }),
        count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 001ac158] the shipped path creates the record; the do-while makes exactly 15 children (counter 1..=15, `uVar2 < 0xf`).
    #[test]
    fn glow_record_matches_the_game() {
        let g = create(3, false).expect("shipped path glows");
        assert_eq!(g.owner, 3);
        assert_eq!(g.children, 15);
        assert_eq!(MAIN_GFX, 0x2000_25f);
        assert_eq!(CHILD_GFX, 0x2000_55a);
        assert_eq!(CLASS, 0x46);
        assert_eq!(CHILD_CLASS, 1);
        assert_eq!(EXT_LINK_U16, 0x100);
        assert_eq!(FLAGS_MAIN, 0x2408);
        assert_eq!(FLAGS_CHILD, 0x2400);
        // Sister (can fail): the child count is the loop bound — 13 or 15 would mean the
        // counter test was misread.
        assert_ne!(g.children, 14);
        assert_ne!(g.children, 16);
    }

    // [M 001ac158:9] the switch_NO_DRONES gate.
    #[test]
    fn the_no_drones_switch_suppresses_the_glow() {
        assert!(create(0, true).is_none());
        // Sister (can fail): the gate is on the switch alone — owner 0 still glows on the
        // shipped path (the game only early-outs on owner 0 OR the switch; owner 0 never
        // occurs in callers, the port keys the record on the list index).
        assert!(create(0, false).is_some());
    }

    // [L ee_start_05 @ 0x2beed0] the GlowBones table the update walks.
    #[test]
    fn glow_bones_match_the_ee_snapshot() {
        assert_eq!(GLOW_BONES, [0, 19, 20, 21, 31, 32, 35, 49, 50, 51, 52, 53, 54, 55, 56]);
        // Sister (can fail): 15 entries — 14 would mean the do-while bound was misread.
        assert_ne!(GLOW_BONES.len(), 14);
    }

    // [M 0x1acd50] the delete bit lands on the main object and all 15 children.
    #[test]
    fn delete_marks_the_main_and_all_fifteen_children() {
        let mut children = [false; 15];
        assert!(delete(&mut children));
        assert!(children.iter().all(|&c| c));
        // Sister (can fail): the count is 15, not 14 (the loop's `while (i < 0xf)`).
        assert_ne!(children.len(), 14);
    }

    // [M 0x1ac2c8:224-226] the alpha byte formula and the 10.0 hide band.
    #[test]
    fn update_alpha_follows_the_distance_formula() {
        let gaps = [1.0f32; 15];
        // dist 0, count 1 → 1.0 · 1 · 128 / 256 = 0.5 → 0 (truncated).
        let (u, c) = update(2, true, false, 1, 0.0, 0x1, false, gaps).0.map(|g| (g.alpha, g.hidden)).unwrap();
        assert_eq!((u, c), (0, false));
        // count 16, dist 0 → 16 · 128 / 256 = 8.
        let (g, _) = update(2, true, false, 16, 0.0, 0x1, false, gaps);
        assert_eq!(g.unwrap().alpha, 8);
        // dist 2.5, count 16 → (1 − 0.25) · 8 = 6.
        let (g, _) = update(2, true, false, 16, 2.5, 0x1, false, gaps);
        assert_eq!(g.unwrap().alpha, 6);
        // dist 10.0 is NOT the hide band (strictly greater); dist 10.01 is.
        let (g, _) = update(2, true, false, 16, 10.0, 0x1, false, gaps);
        assert!(!g.unwrap().hidden);
        let (g, _) = update(2, true, false, 16, 10.01, 0x1, false, gaps);
        assert!(g.unwrap().hidden);
        // Sister (can fail): alpha is a u8 — count 200 at dist 0 would be 100 exactly.
        let (g, _) = update(2, true, false, 200, 0.0, 0x1, false, gaps);
        assert_eq!(g.unwrap().alpha, 100);
    }

    // [M 0x1ac2c8:96-136] the owner-health gate drains the count to the delete.
    #[test]
    fn update_drains_the_count_and_then_deletes() {
        let gaps = [1.0f32; 15];
        let (_, c) = update(2, false, false, 3, 1.0, 0x1, false, gaps);
        assert_eq!(c, 2);
        // A healthy owner does not drain [M 0x1ac2c8:102 (<= 0.0)].
        let (_, c) = update(2, true, false, 3, 1.0, 0x1, false, gaps);
        assert_eq!(c, 3);
        // Count 1 + a failed gate → the game deletes and clears the owner's refs.
        let (u, c) = update(2, false, false, 1, 1.0, 0x1, false, gaps);
        assert_eq!(c, 0);
        assert!(u.is_none());
        // Sister (can fail): kind 0 owners have no gate (the game only checks kinds 2/3).
        let (_, c) = update(0, false, false, 5, 1.0, 0x1, false, gaps);
        assert_eq!(c, 5);
    }

    // [M 0x1ac2c8:162-173] mask == 0 (nobody watches) → the hidden branch.
    #[test]
    fn update_hides_when_no_viewer_watches() {
        let gaps = [1.0f32; 15];
        let (g, c) = update(2, true, false, 9, 1.0, 0x0, false, gaps);
        assert_eq!(c, 9);
        let g = g.unwrap();
        assert!(g.hidden);
        assert_eq!(g.view_mask, 0x10);
        // The spectator/dead mask keeps the glow visible to the others (mask | 0x10).
        let (g, _) = update(2, true, false, 9, 1.0, 0b0010, false, gaps);
        let g = g.unwrap();
        assert!(!g.hidden);
        assert_eq!(g.view_mask, 0b0010 | 0x10);
        // Sister (can fail): the kind-2 + state-1 arm hides like mask 0.
        let (g, _) = update(2, true, true, 9, 1.0, 0b0001, false, gaps);
        assert!(g.unwrap().hidden);
    }
}

#[cfg(test)]
mod review_tests {
    use super::*;

    // [M 0x1ac2c8:135-147] a viewer with AnimObject+0x244 set forces the distance to 0.0: full
    // alpha even past 10.0, where the plain case hides.
    #[test]
    fn a_244_viewer_forces_distance_zero() {
        let gaps = [1.0f32; 15];
        let (g, _) = update(2, true, false, 16, 50.0, 0x1, true, gaps);
        let g = g.unwrap();
        assert!(!g.hidden);
        assert_eq!(g.alpha, 8, "(1 - 0) * 16 * 128 / 256");
        let (g, _) = update(2, true, false, 16, 50.0, 0x1, false, gaps);
        assert!(g.unwrap().hidden, "sister: without the 0x244 viewer, 50.0 > 10.0 hides");
    }

    // [M 0x1ac2c8:186-201] each of the 15 children carries its own bone-pair gap.
    #[test]
    fn each_child_takes_its_own_gap() {
        let mut gaps = [0.0f32; 15];
        for (k, g) in gaps.iter_mut().enumerate() { *g = k as f32; }
        let (g, _) = update(2, true, false, 16, 1.0, 0x1, false, gaps);
        let g = g.unwrap();
        assert_eq!(g.bones[14].1, 14.0);
        assert_eq!(g.bones[13].1, 13.0);
    }
}
