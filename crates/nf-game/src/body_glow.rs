//! The body-glow FX the game attaches to every guard and the player at spawn —
//! `BodyGlow_Create` 0x1ac158 [M 001ac158].
//!
//! The game's callers: `NDrone2_DefaultInit` 0x14b300 (tail — unconditional for every guard
//! spawn, result → `DIVars+0x10` [M; docs/atlas/drone.md:51]), `Player_Init` 0x1910f8:184
//! (result → player data +0x88C [M; docs/atlas/player.md:102]), and the debug anim viewer
//! `AnimDebugCreate` 0x1bcfa8 (kind 0x15 object, not the shipped path). The call-tree audit
//! reaches it through the ported guard-spawn states.
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
//! - 14 child sprite objects (`do..while` counter, 0..0xe): `control_create_object(0, …)`, gfx
//!   hash 0x200055a, obj+0xff = 1, obj+0xf0 |= 0x2400, linked + no-views, stored ext[1..14].
//!
//! The port carries the record (owner, the two gfx hashes, the child count, the flags) on the
//! guard spawn path; the sprite rendering itself is the main agent's draw work (no drawing
//! here). Fields the draw code needs: `MAIN_GFX` / `CHILD_GFX`, `FLAGS_MAIN` / `FLAGS_CHILD`,
//! `RADIUS` (obj+0xe8 = 6.0), `EXT_LINK_U16` (ext+0x40 = 0x100).

/// The main glow object's gfx hash [M 001ac158: `hashtable_set_object_to_entity_gfx(obj, 0x200025f)`].
pub const MAIN_GFX: u32 = 0x2000_25f;
/// The 14 child sprites' gfx hash [M 001ac158: 0x200055a per child].
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

/// The glow record one spawn creates (the main object + its 14 children, ext[0] = owner).
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
    Some(BodyGlow { owner, children: 14 })
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 001ac158] the shipped path creates the record; the do-while makes exactly 14 children.
    #[test]
    fn glow_record_matches_the_game() {
        let g = create(3, false).expect("shipped path glows");
        assert_eq!(g.owner, 3);
        assert_eq!(g.children, 14);
        assert_eq!(MAIN_GFX, 0x2000_25f);
        assert_eq!(CHILD_GFX, 0x2000_55a);
        assert_eq!(CLASS, 0x46);
        assert_eq!(CHILD_CLASS, 1);
        assert_eq!(EXT_LINK_U16, 0x100);
        assert_eq!(FLAGS_MAIN, 0x2408);
        assert_eq!(FLAGS_CHILD, 0x2400);
        // Sister (can fail): the child count is the loop bound — 13 or 15 would mean the
        // counter test was misread.
        assert_ne!(g.children, 13);
        assert_ne!(g.children, 15);
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
}
