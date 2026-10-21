//! The GUNTURRET special object — `GT_Create` 0x17ffa0 (class 0x36 '6' → `GT_Update` 0x180498,
//! docs/atlas/object.md) and the mounted-minigun deploy `GT_DeployMiniGun` 0x181560 [M both
//! decompiles].
//!
//! `GT_Create`'s callers [M]:
//! - `parsemap_create_dynamic_objects` 0x1d04f0 **case 0x27** (line 115, spawn-flag gated) —
//!   41 placements disc-wide: 0700000a ×6 (sky-bridge turrets), 0700000b ×5 (terminal/exec
//!   floor), 2 per MP map [D script-census.md:141; docs/levels/0700000a.md:109,
//!   docs/levels/0700000b.md:160].
//! - `GT_DeployMiniGun` 0x181560 ← `DroneWeap_FireWeapon` 0x178f68:123 (the guard weapon-datum
//!   char 'R' deploy, see below) and `Player_WeaponInitBullet` 0x1a4bf0:282 (the 0x52 briefcase).
//!
//! Creator behaviour [M 0017ffa0:69-160]:
//! - gate: rec+0x34 (param 2) == 0 or rec+0x38 (param 3) == 0 → return 0 (no turret);
//! - `Control_CreateObjEx(1.0, 0x100, pos, rot, 0, parent, 0, 1, 4)`; memset ext 0x100;
//!   obj+0xff = 0x36; obj+0xf0 |= 0x810 (bit 0x10 = hidden until deployed — the draw gate
//!   `crates/nf-view/src/scene.rs` already honours);
//! - ext+0xec (s16) = param 0 (the facing mode: 0 → `Mat_GetDir`, else `Mat_GetUp`+negate);
//!   ext+0xcc = 100.0f; ext+0xee (u16) = param 1; ext+0xe8 (u16) = param 4; ext+0xea (u16) =
//!   param 5; ext+0xb8 = the parent object (param_6) or the new object itself;
//!   ext+0xf0 (u16) = 200 when param 0 == 0, else 0xffff (HP / invulnerable);
//!   ext+0xf2 (u8) = 0; ext+0xe4 = 0;
//! - `Script_Load(param 2 → ext+0xb0)` and `Script_Load(param 3 → ext+0xb4)` with the
//!   per-object callback `GT_Register` 0x181538; either load failing → obj+0xfe |= 1 (the
//!   object deletes itself) and return 0;
//! - `Script_Play`/`Script_Update` both; obj+0xf4 = 0; the script-1 object bbox's max.z × 1.5
//!   → ext+0xd0 (the activation radius);
//! - MP-only (MPSettings != 0, param 0 == 0): deletes sibling kind-'6' turrets with the same
//!   ext+0xb8 parent via `Explode_Create(5.0, 5.0, 50.0, …, 0x6000052, …)`.
//!
//! The port stores the placement-derived record; the script VM (`Script_Load/Play/Update` per
//! object) and the turret behaviours (`GT_Update` 0x180498, `GT_Track` 0x17f9c8,
//! `GT_ApplyDamage` 0x1816d8, `GT_Deploy`/`TakeControl` — the full function map in
//! docs/reports/script-ops-unported.md:78-91) are the tracked gap: the turret records carry
//! the data those ports (and the draw code) need.
//!
//! `GT_DeployMiniGun` 0x181560 [M 00181560 — Ghidra spills the stack args; the flow]: copy the
//! parent matrix, `Collide_PlaceObject(0.5, 0.6, pos, &mat, parent)` → 0 = no room, return 0;
//! else `Mat_GetDir`+`Vec_DirToRot` → rot vec, `Quat_MatToQuat(mat)`, the glist
//! `hashtable_hashcode_to_celglist(0x2000359)` (the 02000359 minigun — not a placement's), then
//! `GT_Create(pos, rotvec, quat, &mat, celglist, parent)`. Its guard-AI caller is
//! `DroneWeap_FireWeapon` 0x178f68:92-145: when the guard's weapon datum char
//! (BOT_vars+0x768) == 'R', `BOTSTATE_getStateType() == 4` (combat), the weapon id not
//! 0xdd/0xde, `AmmoInGun >= 1` and not `tooCloseForWeapon` (+0x170 gate), the game deploys at
//! **(x + sin(yaw), y, z + sin(yaw + 1.5707964))** [M 00178f68:129-137], yaw = obj+0x54; on
//! success `BOTWEAP_decrRounds(…, 1000)` + `BOTSTATE_changeWeapon(…, 0, 1, 0)` and the normal
//! bullet path never runs. The port does not yet model the weapon datum char (BOT_vars+0x768),
//! so the fire_weapon branch cannot select yet; the deploy position math is ported here and
//! unit-tested.

/// The minigun glist hash `GT_DeployMiniGun` fetches [M 00181560].
pub const MINIGUN_GLIST: u32 = 0x2000_359;
/// The `Collide_PlaceObject` radii [M 00181560].
pub const DEPLOY_RADIUS: (f32, f32) = (0.5, 0.6);
/// The deploy yaw offset: sin(yaw + π/2) for the z term [M 00178f68:137].
pub const HALF_PI: f32 = 1.570_796_4;

/// One kind-0x27 placement's turret record (the ext fields the creator writes [M 0017ffa0]).
#[derive(Clone, Debug, PartialEq)]
pub struct GTurret {
    pub placement: usize,
    /// ext+0xec (s16): param 0 — the facing mode (0 = Mat_GetDir, else Mat_GetUp + negate) and
    /// the HP selector (0 → HP 200, else 0xffff) [M 0017ffa0:83-96,141-145].
    pub facing: i16,
    /// ext+0xee (u16): param 1 [M 0017ffa0:85-87].
    pub param1: u16,
    /// ext+0xb0: `Script_Load(param 2)` — zero means the load failed and the game deletes the
    /// object [M 0017ffa0:100-106].
    pub script_a: u32,
    /// ext+0xb4: `Script_Load(param 3)` [M 0017ffa0:103-105].
    pub script_b: u32,
    /// ext+0xe8 (u16): param 4 [M 0017ffa0:88].
    pub param4: u16,
    /// ext+0xea (u16): param 5 [M 0017ffa0:89-91].
    pub param5: u16,
    /// ext+0xf0 (u16): 200 when param 0 == 0, else 0xffff [M 0017ffa0:92-96].
    pub hp: u16,
    /// obj+0xf4: the update state, 0 at creation [M 0017ffa0:117].
    pub state: u16,
}

fn param(params: &[(u32, u32)], k: u32) -> u32 {
    params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v)
}

/// `GT_Create` 0x17ffa0 from one kind-0x27 placement: None when the gate fails (param 2/3 == 0
/// → the game returns 0 before creating anything [M 0017ffa0:69]).
pub fn from_placement(placement: usize, params: &[(u32, u32)]) -> Option<GTurret> {
    let script_a = param(params, 2);
    let script_b = param(params, 3);
    if script_a == 0 || script_b == 0 {
        return None;
    }
    let facing = param(params, 0) as u16 as i16;
    Some(GTurret {
        placement,
        facing,
        param1: param(params, 1) as u16,
        script_a,
        script_b,
        param4: param(params, 4) as u16,
        param5: param(params, 5) as u16,
        hp: if facing == 0 { 200 } else { 0xffff },
        state: 0,
    })
}

/// `GT_DeployMiniGun` 0x181560's deploy position, as `DroneWeap_FireWeapon` 0x178f68:129-137
/// computes it before the call: (x + sin(yaw), y, z + sin(yaw + 1.5707964)), yaw = obj+0x54.
pub fn deploy_position(pos: [f32; 3], yaw: f32) -> [f32; 3] {
    [
        pos[0] + crate::trig::sinf(yaw),
        pos[1],
        pos[2] + crate::trig::sinf(yaw + HALF_PI),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 0017ffa0:69,83-96] the gate, the ext map, the HP selector.
    #[test]
    fn the_creator_param_map() {
        let ps: Vec<(u32, u32)> = vec![(0, 0), (1, 7), (2, 0x0600_0100), (3, 0x0600_0101), (4, 9), (5, 11)];
        let t = from_placement(2, &ps).expect("scripts present");
        assert_eq!(t.placement, 2);
        assert_eq!((t.facing, t.param1), (0, 7));
        assert_eq!((t.script_a, t.script_b), (0x0600_0100, 0x0600_0101));
        assert_eq!((t.param4, t.param5), (9, 11));
        assert_eq!(t.hp, 200, "param 0 == 0 → HP 200 [M 0017ffa0:92-96]");
        assert_eq!(t.state, 0);
        // Sister (can fail): a nonzero facing flips BOTH the facing mode and the HP rule.
        let ps1: Vec<(u32, u32)> = vec![(0, 3), (1, 7), (2, 0x60), (3, 0x61), (4, 9), (5, 11)];
        let t1 = from_placement(0, &ps1).expect("scripts present");
        assert_eq!(t1.facing, 3);
        assert_eq!(t1.hp, 0xffff, "param 0 != 0 → invulnerable 0xffff [M 0017ffa0:92-96]");
    }

    // [M 0017ffa0:69] either script param zero → the game never creates the object.
    #[test]
    fn the_script_gate() {
        assert!(from_placement(0, &[(2, 0x60), (3, 0)]).is_none());
        assert!(from_placement(0, &[(2, 0), (3, 0x61)]).is_none());
        assert!(from_placement(0, &[]).is_none());
        // Sister (can fail): the gate is BOTH scripts — one present is still None.
        assert!(from_placement(0, &[(2, 0x60)]).is_none());
    }

    // [M 00178f68:129-137] the deploy offset: x+sin(yaw), z+sin(yaw+π/2).
    #[test]
    fn the_deploy_position_offset() {
        let p = deploy_position([10.0, 2.0, 20.0], 0.0);
        assert_eq!(p[1], 2.0, "y untouched");
        assert!((p[0] - 10.0).abs() < 1e-4, "sin(0) = 0");
        assert!((p[2] - 21.0).abs() < 1e-4, "sin(π/2) = 1");
        // Sister (can fail): yaw π/2 moves x by 1, z by 0 — a swapped sin would flip them.
        let q = deploy_position([10.0, 2.0, 20.0], HALF_PI);
        assert!((q[0] - 11.0).abs() < 1e-4);
        assert!((q[2] - 20.0).abs() < 1e-4);
    }
}
