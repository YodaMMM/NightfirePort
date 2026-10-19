//! The mounted-gun implementation (placement type 0x34, class 0x47).
//!
//! `parsemap_create_dynamic_objects` 0x1d04f0 case 0x34 → `GunImp_Create` 0x131f20; the object's
//! class byte is 0x47 [M 0x131f20 `sb zero,255(a0)`-family: `*(char *)(iVar5 + 0xff) = 0x47`].
//! 0700000d carries exactly one: #225 `PowerStationGunImpBase` (137.27, 9.33, −324.66), the laser
//! prototype's mount — the level's "Destroy door C5 with mounted laser" objective [D placements].
//!
//! The turret hardware is the `TurretGuns` table [D ACTION.ELF @0x26f520, 12 × 0x1c bytes; LOAD
//! vaddr 0x107100 = file 0x1000, so file 0x169420]: per entry a 12-byte muzzle offset, the gun
//! art hash (+0x0c), the weapon id (+0x10), the fire interval in seconds as a float bit pattern
//! (+0x14), the fire sound (+0x18). Entry 0xb is the laser: muzzle (0, 0, 1.71), art 0x02000b50,
//! weapon **0x67**, interval 1.0, sound 1307. `GunImp_Create` picks the entry from MPSettings in
//! the decompile — the single-player arm reads 0xb [M 0x131f20:52-63]; the port (single player
//! only) hard-sets 0xb and pins the laser entry by hash.
//!
//! Use (`Player_Activate` 0x1a5170's class walk reaching class 0x47 → `GunImp_Activate`
//! 0x132f00): `Player_ChangeSubState(0xc)`, cam mode 0xe, `Player_Disable(player, 1)`,
//! `Player_WeaponNone`, `Input_SetAction(0xe)` — the imp owns the tick, state `obj+0xf4 = 1`.
//!
//! `GunImp_Update` 0x1320d8 while state 1:
//! - aim: yaw `ext+0x74` -= stick-x·FRAME_RATE_MUL·0.015, pitch `ext+0x70` -= stick-y·…·0.015,
//!   pitch clamped to [−π/2, +π/4] [M 0x1320d8:124-135]; both ease toward smoothed mirrors
//!   `ext+0x80`/`ext+0x84` by 0.15 per tick [M 0x1320d8:136-161]. FRAME_RATE_MUL = 1.0 [L, the
//!   script clocks step 1.0/tick — docs/systems/script-player.md].
//! - dismount: `Input_Action(slot, 0xe, 4)` → `GunImp_Deactivate` 0x132c18 (the saved substate
//!   back, cam mode 0, `Player_Enable`).
//! - fire (action 9) when the cooldown `ext+0x94` has drained to ≤ 0: the cooldown refills from
//!   `TurretGuns[0xb]+0x14` (= 1.0 s), the heat `ext+0x90` += 2.0 (≥ 200 sets the hot byte
//!   `ext+0x9c`), and `Bullet_init(slot, player, imp, weapon_data + 0x67·0x10c, muzzle, dir,
//!   0xffff)` spawns the laser round [M 0x1320d8:271-331]. The muzzle is the barrel matrix
//!   (base = the placement transform — #225's rotation is identity — composed RotY(yaw_s)·
//!   RotX(pitch_s)) applied to `TurretGuns[0xb]`'s (0, 0, 1.71) offset; the direction its +Z.
//!   The bullet is the player's (the game passes the player object as the owner), so it flies
//!   the player-bullet flight and its class-5 contacts damage the C5 `Destroy` leaves through
//!   `Destroy_CollisionHandler` 0x12bd48 — the door death writes ch 0x1f (objective 6).
//!   Direct `Bullet_init` — no `PlrStat_LogShotFired` (that logs inside `Player_WeaponInitBullet`'s
//!   pellet loop [M 0x1a4bf0:258], which this path never reaches) and no fire-mode bookkeeping.
//!
//! Beam visuals (the 0x20005cd quad `PositionBeam(muzzle → ray hit)`, the 0x200026e impact flare,
//! `Explode_Propagate` at the hit for 0x67) are presentation: the port exposes the getters the
//! draw code calls (`muzzle`, `barrel_dir`, `heat`, `hot`) — no drawing here.
//!
//! Tags: [M] the addresses above, [D] the disc bytes, [L] the snapshot reads, [S] suggested.

use crate::world::V3;

/// The `TurretGuns` table [D ACTION.ELF @0x26f520 (file 0x169420), 12 entries × 0x1c].
pub struct TurretGun {
    /// The muzzle offset in the barrel's frame (+0x00, 3 floats).
    pub muzzle: [f32; 3],
    /// The mounted-gun art object hash (+0x0c).
    pub art: u32,
    /// The weapon id (+0x10) — `weapon_data + w·0x10c` is the round's datum.
    pub weapon: u8,
    /// The fire interval in seconds (+0x14, float bits).
    pub interval: f32,
    /// The fire sound (+0x18).
    pub sound: u32,
}

pub const TURRET_GUNS: [TurretGun; 12] = [
    TurretGun { muzzle: [0.0, 0.27, 0.92], art: 0x0200_0b05, weapon: 0x2e, interval: f32::from_bits(1127481344), sound: 268 },
    TurretGun { muzzle: [0.0, 0.23, 1.65], art: 0x0200_0a3b, weapon: 0x60, interval: f32::from_bits(1092616192), sound: 266 },
    TurretGun { muzzle: [0.17, 0.23, 0.94], art: 0x0200_0b06, weapon: 0x61, interval: f32::from_bits(1084227584), sound: 1255 },
    TurretGun { muzzle: [0.0, 0.20, 1.16], art: 0x0200_0b08, weapon: 0x6f, interval: f32::from_bits(1127481344), sound: 1058 },
    TurretGun { muzzle: [0.0, 0.27, 0.92], art: 0x0200_0b05, weapon: 0x2d, interval: f32::from_bits(1127481344), sound: 268 },
    TurretGun { muzzle: [0.0, 0.21, 0.51], art: 0x0200_0b07, weapon: 0x2a, interval: f32::from_bits(1123024896), sound: 1036 },
    TurretGun { muzzle: [0.0, 0.21, 0.51], art: 0x0200_0b07, weapon: 0x2b, interval: f32::from_bits(1123024896), sound: 1036 },
    TurretGun { muzzle: [0.0, 0.33, 1.61], art: 0x0200_0565, weapon: 0x70, interval: f32::from_bits(1065353216), sound: 1307 },
    TurretGun { muzzle: [0.0, 0.33, 1.61], art: 0x0200_0565, weapon: 0x70, interval: f32::from_bits(1065353216), sound: 1307 },
    TurretGun { muzzle: [0.0, 0.33, 1.61], art: 0x0200_0565, weapon: 0x32, interval: f32::from_bits(1127481344), sound: 1059 },
    TurretGun { muzzle: [0.0, 0.33, 1.61], art: 0x0200_0565, weapon: 0x32, interval: f32::from_bits(1127481344), sound: 1059 },
    TurretGun { muzzle: [0.0, 0.0, 1.71], art: 0x0200_0b50, weapon: 0x67, interval: f32::from_bits(1065353216), sound: 1307 },
];

/// The single-player turret entry [M GunImp_Create 0x131f20:52-55 — the MPSettings+0x180 == 0
/// arm reads 0xb]. The interval field stores raw float bits in the table (the game assigns it to
/// an int cell the update reads back as a float); the decode keeps the bit pattern as f32.
pub const SP_TURRET: usize = 0xb;

#[derive(Clone, Debug)]
pub struct GunImp {
    pub placement: usize,
    /// The placement transform (the base matrix's translation + yaw).
    pub pos: V3,
    pub yaw_base: f32,
    /// The raw aim (ext+0x70 pitch / ext+0x74 yaw), radians.
    pub pitch: f32,
    pub yaw: f32,
    /// The smoothed mirrors (ext+0x80 / ext+0x84).
    pub pitch_s: f32,
    pub yaw_s: f32,
    /// `obj+0xf4`: 0 idle (usable), 1 mounted.
    pub active: bool,
    /// `ext+0x90` — the laser heat, += 2.0 per shot, 200 saturates the hot byte.
    pub heat: f32,
    /// `ext+0x94` — the fire cooldown.
    pub cooldown: f32,
    /// `ext+0x9c` — the hot byte (200 heat reached).
    pub hot: bool,
}

impl GunImp {
    /// The barrel matrix's +Z [M 0x1320d8:162-172 — RotY(yaw_s) after RotX(pitch_s) on the base].
    pub fn barrel_dir(&self) -> V3 {
        let (sp, cp) = self.pitch_s.sin_cos();
        let (sy, cy) = self.yaw_s.sin_cos();
        // RotX(pitch)·(0,0,1) = (0, −sin p, cos p) in the game's matrix sense, then RotY(yaw):
        // (sin y·cos p, −sin p, cos y·cos p) — the port's yaw convention matches Bond's
        // (dx.atan2(dz), walker's aim loop).
        [sy * cp, -sp, cy * cp]
    }

    /// `TurretGuns[0xb]`'s muzzle offset through the barrel matrix, plus the placement origin
    /// [M 0x1320d8:196-211 — the offset vector rotated by the gun object's matrix].
    pub fn muzzle(&self) -> V3 {
        let d = self.barrel_dir();
        let m = TURRET_GUNS[SP_TURRET].muzzle;
        // The offset (0, 0, 1.71) is pure barrel-forward; the rotated form keeps the general shape.
        let along = m[2];
        [self.pos[0] + d[0] * along + m[0], self.pos[1] + d[1] * along + m[1], self.pos[2] + d[2] * along + m[2]]
    }
}
