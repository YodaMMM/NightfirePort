//! The mounted-gun implementation (placement type 0x34, class 0x47).
//!
//! `parsemap_create_dynamic_objects` 0x1d04f0 case 0x34 → `GunImp_Create` 0x131f20; the object's
//! class byte is 0x47 [M 0x131f20 `sb zero,255(a0)`-family: `*(char *)(iVar5 + 0xff) = 0x47`].
//! 0700000d carries exactly one: #225 `PowerStationGunImpBase` (137.27, 9.33, −324.66), the laser
//! prototype's mount — the level's "Destroy door C5 with mounted laser" objective [D placements].
//!
//! The deployable's ground placement — `Collide_PlaceObject` 0x1ecfb8 (the +0.05 y bump, the
//! 3.0 down ray via `Collide_RayIntersect` flags 0x1D, the normal-offset reseat, the 0x800/0x2000
//! two-pass triangle pick, the −0.13/0.2/0.96 support gates and the `Mat_Align2Up` fit test [M
//! 0x1ecfb8:5-70]) — is not ported: its only caller is `GT_DeployMiniGun` 0x181560:57, which the
//! port has not needed (the mounted turrets spawn placed by their placements) [S].
//!
//! The turret hardware is the `TurretGuns` table [D ACTION.ELF @0x26f520, 12 × 0x1c bytes; LOAD
//! vaddr 0x107100 = file 0x1000, so file 0x169420]: per entry a 12-byte muzzle offset, the gun
//! art hash (+0x0c), the weapon id (+0x10), the fire interval in TICKS as a float bit pattern
//! (+0x14 — the update counts FRAME_RATE_MUL = 1.0 a tick down [M 0x1320d8:107-112, :324]), the
//! fire sound (+0x18). Entry 0xb is the laser: muzzle (0, 0, 1.71), art
//! 0x02000b50, weapon **0x67**, interval 1.0, sound 1307. `GunImp_Create` picks the entry from
//! MPSettings in the decompile — the single-player arm reads 0xb [M 0x131f20:52-63]; the port
//! (single player only) hard-sets 0xb and pins the laser entry by hash.
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
//!   (base = the placement transform — `Quat_QuatToMat` on the +0x24 quaternion with the
//!   position via `Mat_SetTrans` [M 0x131f20:66-67], overwriting the created object's matrix at
//!   `Control_CreateObjEx` 0x1343c8:18-23; #225's quat is the identity [D placements, and every
//!   0x34 disc row is yaw-only with a quat-consistent euler — placement_rot_census] — composed
//!   RotY(yaw_s)·RotX(pitch_s)) applied to `TurretGuns[0xb]`'s (0, 0, 1.71) offset; the
//!   direction its +Z.
//!   The bullet is the player's (the game passes the player object as the owner), so it flies
//!   the player-bullet flight and its class-5 contacts damage the C5 `Destroy` leaves through
//!   `Destroy_CollisionHandler` 0x12bd48 — the door death writes ch 0x1f (objective 6).
//!   Direct `Bullet_init` — no `PlrStat_LogShotFired` (that logs inside `Player_WeaponInitBullet`'s
//!   pellet loop [M 0x1a4bf0:258], which this path never reaches) and no fire-mode bookkeeping.
//!
//! Beam visuals: the 0x20005cd quad `PositionBeam(muzzle → ray hit)` and the 0x200026e impact
//! flare are presentation — the port exposes the getters the draw code calls (`muzzle`,
//! `barrel_dir`, `heat`, `hot`) — no drawing here. The ray itself is NOT presentation: the fire
//! arm casts the 1000-unit `Collide_RayIntersect` [M 0x1320d8:372-384], clamps the beam at the
//! hit [M :385-387], and the 0x67 arm runs `Explode_Propagate(4.0, 5.0, hit, cel, the imp obj)`
//! at it [M :433-436] — the sweep's case 0x21 smashes a destroyable it reaches with ≥ 1.0
//! damage [M 0x17c518:141-177] and `Destroy_Smash` 0x12bb60:40 writes the leaf's channel. On
//! 0700000d that is C5's ch 0x1f: the mounted laser kills its objective door on the first beam
//! contact (drone.rs `run_beam_explodes`; the 50-a-contact bullet drain
//! [Destroy_CollisionHandler 0x12bd48:31-35] is the slow second path).
//!
//! Tags: [M] the addresses above, [D] the disc bytes, [L] the snapshot reads, [S] suggested.

use crate::world::V3;
use nf_formats::anim::Mat4;

/// The `TurretGuns` table [D ACTION.ELF @0x26f520 (file 0x169420), 12 entries × 0x1c].
pub struct TurretGun {
    /// The muzzle offset in the barrel's frame (+0x00, 3 floats).
    pub muzzle: [f32; 3],
    /// The mounted-gun art object hash (+0x0c).
    pub art: u32,
    /// The weapon id (+0x10) — `weapon_data + w·0x10c` is the round's datum.
    pub weapon: u8,
    /// The fire interval in ticks (+0x14, float bits — the update counts 1.0 a tick down
    /// [M 0x1320d8:107-112, :324]).
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

/// The distinct gun-child art hashes of `TURRET_GUNS` (the draw keeps one slot per art, so the
/// MP weapon set's pick shows the right mesh — see `GunImp::art_slot`).
pub const TURRET_ARTS: [u32; 7] = [0x0200_0b05, 0x0200_0a3b, 0x0200_0b06, 0x0200_0b08, 0x0200_0b07, 0x0200_0565, 0x0200_0b50];

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
    /// The firing beam this shot leaves behind: `(muzzle, hit)` for the game's `0x20005cd` quad
    /// (`PositionBeam(muzzle → ray hit)`, `GunImp_Update` 0x1320d8:394) and the `0x200026e` flare at
    /// the hit. `Some` on the shot tick, `None` otherwise; the scene stretches the quad along it.
    pub beam: Option<(V3, V3)>,
    /// `ext+0x9d` — the `TurretGuns` entry: 0xb in SP, the MP weapon set (Random: a `Rand_Rand(10)`
    /// pick) [M GunImp_Create 0x131f20:52-63,87].
    pub turret: u8,
    /// The seat offset along the base's forward axis (ext+0x68; the offset vector's x/y are
    /// zeroed at create [M GunImp_Create 0x131f20:78-79]). Its z = `*(float *)(celglist + 0x28)
    /// * 0.75` [M 0x131f20:77,81] — the glist's bounds-block word at +0x28: the entity-params
    /// bounds copy into the runtime glist at the same +8 shift the verified rows show (centre
    /// payload+8 → glist+0x10, radius payload+0x14 → glist+0x1c [M glist.md], so min at
    /// payload+0x18 → glist+0x20 and **glist+0x28 = min.z**) — for #225 min.z = −1.4362069
    /// [D placements box], seat = −1.0771552. Residual [S]: the payload→runtime +8 shift for
    /// the bounds block is inferred from the verified centre/radius rows, not read live.
    pub seat: f32,
    /// The placement object's gfx-sphere radius — the game's obj+0x8C world-sphere radius source
    /// (`Control_BuildWorldSph` 0x133070:48 takes the glist bound radius × |scale|) — the use-key
    /// pick's candidate radius [M Collide_PickObj 0x1eae80:219-228, `1.0 + obj+0x8C`].
    pub radius: f32,
    /// `ext+0x98` — the looped fire sound's handle: `Sound_Play3D(TurretGuns[+0x18], obj+0x30)`
    /// stored the first fire tick while the trigger holds (0x67/0x70/0x61 arms), `Sound_Stop`ed
    /// on release or overheat [M GunImp_Update 0x1320d8:261-269, :281-298], and on dismount
    /// [M GunImp_Deactivate 0x132c18:62-63]. 0 = not playing.
    pub fire_handle: u32,
}

impl GunImp {
    /// The drawn barrel matrix's +Z [M GunImp_Update 0x1320d8:325 — `Mat_GetDir(dir, cel+0x90)`,
    /// and Mat_GetDir 0x1e3398 reads the matrix's third row (+0x20..+0x28) un-negated — the
    /// disassembly]. The cel is the matrix the aim drives and the draw poses
    /// (`barrel_matrix`), so the fire direction reads from that same matrix.
    pub fn barrel_dir(&self) -> V3 {
        let m = self.barrel_matrix();
        let f = [m[2][0], m[2][1], m[2][2]];
        let l = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt().max(1e-9);
        [f[0] / l, f[1] / l, f[2] / l]
    }

    /// `TurretGuns[turret]`'s 12-byte muzzle offset through the drawn barrel matrix, plus its
    /// translation [M GunImp_Update 0x1320d8:196-211 — the offset vector against the gun cel's
    /// matrix rows (its +0x90/+0xA0/+0xB0) with the cel's +0xC0 translation added; :165-172
    /// compose that cel as `base·RotY(yaw_s)·RotX(pitch_s)` — disassembly: `Concat(src, dst)`
    /// writes `dst = src·dst` and `Mat_CopyRot` copies only rows 0-2, keeping the cel's own
    /// translation row, so the cel rides the base translation]. The same matrix the draw uses
    /// (`barrel_matrix`), so the beam and the bullet start on the drawn muzzle.
    pub fn muzzle(&self) -> V3 {
        let m = self.barrel_matrix();
        let o = TURRET_GUNS[(self.turret as usize).min(TURRET_GUNS.len() - 1)].muzzle;
        // The row-vector product o·M (columns of the rotation rows) + the translation row.
        [
            m[3][0] + m[0][0] * o[0] + m[1][0] * o[1] + m[2][0] * o[2],
            m[3][1] + m[0][1] * o[0] + m[1][1] * o[1] + m[2][1] * o[2],
            m[3][2] + m[0][2] * o[0] + m[1][2] * o[1] + m[2][2] * o[2],
        ]
    }

    /// The mounted player pose the game writes every tick [M 0x1320d8:175-194]: the player
    /// object's matrix ← base·RotY(yaw_s) (Bond's yaw follows the turret), the position ←
    /// base·RotY(yaw_s)·(0,0,seat) + base translation (the seat offset rides the matrix rows
    /// [M 0x1320d8:176-192]), the view pitch `BLData+0x8A8 = −pitch_s·0.5` [M 0x1320d8:194].
    /// Returns (pos, yaw, view_pitch).
    pub fn mounted_pose(&self) -> (V3, f32, f32) {
        let yaw = self.yaw_base + self.yaw_s;
        let (sy, cy) = yaw.sin_cos();
        ([self.pos[0] + sy * self.seat, self.pos[1], self.pos[2] + cy * self.seat], yaw, -self.pitch_s * 0.5)
    }

    /// The barrel matrix for the draw: base·RotY(yaw_s)·RotX(pitch_s) with the base
    /// translation [M 0x1320d8:165-172 — the gunimp object's matrix ← the stored base, ·RotY,
    /// copied to the barrel object, ·RotX]. Rows are the port's row-vector axes (row 2 = the
    /// barrel forward, `barrel_dir`); the base is #225's identity-quat placement [D placements;
    /// every 0x34 disc row is yaw-only with a quat-consistent euler, so the composed yaw
    /// `yaw_base + yaw_s` equals the quat base's yaw — placement_rot_census pins it per row], so the
    /// composed yaw is `yaw_base + yaw_s` [D placements; the general non-yaw-only base is not
    /// modelled — 0d carries exactly one mount].
    /// The base object's matrix: the placement turned by the smoothed yaw only [M GunImp_Update
    /// 0x1320d8:162-170 — `obj+0x90 = base · RotY(ext+0x84)`]; the gun child adds the pitch
    /// (`barrel_matrix`, `*ext+0x90 = obj+0x90 · RotX(ext+0x80)` [M :171-173]).
    pub fn base_matrix(&self) -> Mat4 {
        let (sy, cy) = (self.yaw_base + self.yaw_s).sin_cos();
        [[cy, 0.0, -sy, 0.0], [0.0, 1.0, 0.0, 0.0], [sy, 0.0, cy, 0.0], [self.pos[0], self.pos[1], self.pos[2], 1.0]]
    }

    /// The `TURRET_ARTS` index of this mount's gun child (`GunImp_Create` 0x131f20:85-89 creates it
    /// from `TurretGuns[ext+0x9d].art`).
    pub fn art_slot(&self) -> Option<usize> {
        let art = TURRET_GUNS.get(self.turret as usize)?.art;
        TURRET_ARTS.iter().position(|&a| a == art)
    }

    pub fn barrel_matrix(&self) -> Mat4 {
        let yaw = self.yaw_base + self.yaw_s;
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = self.pitch_s.sin_cos();
        // RotX(p) maps (0,0,1) → (0, −sin p, cos p) [M barrel_dir]; rows: right, up, forward.
        [
            [cy, 0.0, -sy, 0.0],
            [sy * sp, cp, cy * sp, 0.0],
            [sy * cp, -sp, cy * cp, 0.0],
            [self.pos[0], self.pos[1], self.pos[2], 1.0],
        ]
    }
}
