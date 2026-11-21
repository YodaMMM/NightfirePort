//! The driving engine's POV weapon mode — the opening of Paris Prelude (MIS01), where Bond
//! shoots the Sniper Rifle from the helicopter while it flies a spline, before the hand-over to
//! the Vanquish. Write-up: docs/systems/driving-mis01-opening.md.
//!
//! Four pieces, all DRIVING.ELF [M] (EE addresses, decompile under `work/decomp/DRIVING.ELF/`)
//! unless tagged [D] (disc / ELF data bytes) or [S] (suggested):
//! - [`WEAPONS`]: the 32-entry weapon table `FUN_0020f2b8` builds at 0x4165c0 (stride 0x54);
//!   `FUN_0020bbe0` copies it into the weapon manager (`DAT_0033a530`, table at +0x10) on every
//!   reset, so the `MSet` "objective channels" of drive-mission-scripts.md are this inventory.
//! - [`Loadout`]: the manager — `FUN_0020bbe0` reset, `FUN_00205498` + `FUN_0020d378` the MSet
//!   registration, the head of `FUN_0020c078` (the POV weapon swap), the POV branch of the fire
//!   `FUN_0020de20`, the reloads `FUN_0020e868` / `FUN_0020e978` / `FUN_0020c078`'s empty-clip
//!   loop, and `ECustomTransition` `FUN_0014b020` (the hand-over loadout).
//! - [`PovGunner`]: Bond's gunner actor (`AICharacter`, 0x170 bytes, vtable 0x3696e0) built by
//!   `FUN_00123950`; its per-frame update `FUN_00118af0` dispatches on state +0x4c to the vtable
//!   slot `0x44 + 8*state` — states 1/2/8/9/10/11/12/13 are `FUN_00124240`/`00124348`/
//!   `001243f0`/`00124440`/`00124570`/`00124650`/`00124770`/`00124800`.
//! - [`PovSession`]: the mode flag (weapon manager `DAT_0033a024`+0x350, written only through
//!   `FUN_00206398`) driven by the scripted events `ESetAutoDriveCamOn`/`Off`(+`Movie`).
//!
//! Not ported (stated plainly): the aim/camera matrix `FUN_00124848` (the scope sway and the
//! recoil jitter), the bullet hit test, the anim player itself (the caller reports "anim done"
//! and the anim time), the zoom-fraction blend (+0x160 from `FUN_001073a8`/`001073d8`/
//! `001073c0`, not decomposed), the camera turn input, and the scope object's +0x24/+0x2c
//! guards in the zoom input `FUN_001be038`.

use crate::driving::mission::MissionSet;

/// One weapon table entry as `FUN_0020f2b8` stores it (fields this port uses).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponDef {
    /// +0x04: the name (entry 0 = "Gun" 0x398a40, entry 23 = "P2K" 0x398ba8) [M+D].
    pub name: &'static str,
    /// +0x1c: the loaded rounds the table starts with.
    pub clip: i32,
    /// +0x20: the clip size.
    pub clip_size: i32,
    /// +0x34: the value the MSet's third array overrides (`FUN_0020d378` arg 5); its consumer
    /// is not traced.
    pub p34: i32,
    /// +0x38: set → each shot calls `FUN_002068e0(mgr, 1)` (a shot counter, `FUN_0020de20`).
    pub counted: bool,
    /// +0x44: the sound stem (`"SFX_" + stem + "Reload"` in `FUN_0020e868`).
    pub stem: &'static str,
}

const fn w(name: &'static str, clip: i32, clip_size: i32, p34: i32, counted: bool, stem: &'static str) -> WeaponDef {
    WeaponDef { name, clip, clip_size, p34, counted, stem }
}

/// The weapon table [M `FUN_0020f2b8`, entry `i` at 0x4165c0 + i*0x54].
pub const WEAPONS: [WeaponDef; 32] = [
    w("Gun", 1, 1, 10, true, "mntmach"),
    w("Rocket", 1, 1, 0xfa, true, "Rocket"),
    w("Missile", 4, 4, 0xfa, true, "Missile"),
    w("Q-Booster", 1, 1, 0, false, "Boost"),
    w("Time Mine", 1, 1, 100, true, ""),
    w("Prox Mine", 1, 1, 100, true, "Hmine"),
    w("Rem Mine", 1, 1, 500, true, ""),
    w("Floating Mine", 1, 1, 1000, true, ""),
    w("Detonator", 1, 1, 0, false, ""),
    w("Spikes", 1, 1, 0x19, false, ""),
    w("Q-Slick", 1, 1, 0, false, ""),
    w("Q-Smoke", 1, 1, 0, false, "smkscrn"),
    w("Q-Worm", 1, 1, 0x19, false, ""),
    w("Laser", 1, 1, 0x19, false, ""),
    w("Q-Pulse", 1, 1, 0x32, true, "EMP"),
    w("Radar", 1, 1, 0, false, "qlocate"),
    w("Cannon", 1, 1, 0xfa, true, "cannon"),
    w("Q-Vision", 1, 1, 0, false, ""),
    w("2 Wheels", 1, 1, 0, false, "TwoWheel"),
    w("Grenade", 1, 1, 500, true, "grenade"),
    w("Q-Charge", 1, 1, 5000, false, "Hmine"),
    w("Torpedo", 2, 2, 0xfa, true, "vtorpedo"),
    w("Remote Torpedo", 2, 2, 0xfa, true, "vGtorp"),
    w("P2K", 10, 10, 10, true, "HHandGun"),
    w("Snowmobile gun", 0x32, 0x32, 0x32, true, "snomach"),
    w("Snowmobile Rocket", 1, 1, 0xfa, true, "grenade"),
    w("Sniper Rifle", 0x1e, 0x1e, 10, true, "sniper"),
    w("Ultralight gun", 10, 10, 0x1e, true, "Umntmach"),
    w("Ultralight rocket", 10, 10, 0xfa, true, "Umissile"),
    w("FOD Gun", 0x32, 0x32, 0x32, true, "fingergun"),
    w("FOD Rocket", 1, 1, 0xfa, true, "finger"),
    w("CH-6 Remote Rocket", 10, 10, 0xfa, true, "Rocket"),
];

/// The per-weapon refire time in seconds, `f32` bits [D DRIVING.ELF data 0x33a030..0x33a0b0;
/// its only reader is `FUN_0020bbe0`, which stores `(int)(v * fps)` at entry +0x3c].
pub const REFIRE_BITS: [u32; 32] = [
    0x3e2aaaab, 0x3e800000, 0x3f2aaaab, 0x40800000, 0x3f800000, 0x3f800000, 0x3f800000, 0x3f800000,
    0x3f800000, 0x40000000, 0x40000000, 0x40000000, 0x40000000, 0x40000000, 0x3f800000, 0x3f800000,
    0x3f800000, 0x40000000, 0x40000000, 0x3f800000, 0x3f800000, 0x40400000, 0x3e800000, 0x3daaaaab,
    0x3daaaaab, 0x3daaaaab, 0x3daaaaab, 0x3d4ccccd, 0x3f800000, 0x3daaaaab, 0x40a00000, 0x3daaaaab,
];

/// The Sniper Rifle's weapon index (its table id 0x1a) [M `FUN_0020f2b8`].
pub const SNIPER_RIFLE: usize = 0x1a;
/// The Missile (id 2) [M].
pub const MISSILE: usize = 2;
/// The first POV weapon id; a POV kind is `id - POV_BASE` (1..=7) [M `FUN_00123950`'s
/// `(kind + 0x17) * 0x54`, `FUN_0020c078`'s `kind + 0x17 != current`].
pub const POV_BASE: usize = 0x17;
/// MSet counts at or above this are skipped [M `FUN_00205498`: `< 0x2711`].
pub const MSET_COUNT_LIMIT: i32 = 0x2711;

/// One inventory slot (the manager's copy of a table entry).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponSlot {
    /// +0x14: bit 0 owned, bit 2 the clip ran dry (wait for the auto reload), bit 5 set when
    /// the gunner switches to it (`FUN_0020c078`, unless bit 6).
    pub flags: u32,
    /// +0x18: the rounds carried (the ammo count).
    pub total: i32,
    /// +0x1c: the rounds in the clip.
    pub clip: i32,
    /// +0x20: the clip size.
    pub clip_size: i32,
    /// +0x24: the last shot's tick (the table's -1000 start). Every POV weapon's shot pattern
    /// row at 0x33a0b0 holds one entry [D], so its timestamp ring never leaves slot 0.
    pub last_fire: i32,
    /// +0x34.
    pub p34: i32,
    /// +0x3c: the refire time in ticks.
    pub refire: i32,
    /// +0x50: the shots fired.
    pub shots: i32,
}

impl WeaponSlot {
    pub fn owned(&self) -> bool { self.flags & 1 != 0 }
}

/// What a fire request did (`FUN_0020de20`'s POV branch).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fire {
    /// No rounds carried: the manager's firing flag (+0x30) drops.
    NoAmmo,
    /// The Sniper Rifle with the dry-fire gate set and the scope not up: "SFX_DryFire" in the
    /// "POV Weapons" group, no shot.
    DryFire,
    /// The switch cooldown, the refire time or a busy gunner held the shot.
    Held,
    /// A shot; `emptied` = the clip ran dry and the slot waits for the auto reload.
    Shot { emptied: bool },
}

/// The weapon manager (`DAT_0033a530`).
#[derive(Clone, Debug, PartialEq)]
pub struct Loadout {
    pub slots: [WeaponSlot; 32],
    /// +0x04: the current weapon [M `FUN_0020d488`].
    pub current: usize,
    /// +0x0c: the weapon stashed while the POV mode swaps in its own [M `FUN_0020c078`].
    pub other: usize,
    /// +0x30: the firing flag.
    pub firing: bool,
    /// +0x34: a reload is waiting for the gunner.
    pub reload_pending: bool,
    /// +0x50: the switch cooldown tick (fire and the gunner switch wait past it).
    pub cooldown: i32,
    /// The shot counter `FUN_002068e0(mgr, n)` feeds (the manager's stat; not traced further).
    pub counted_shots: i32,
}

impl Loadout {
    /// `FUN_0020bbe0`: copy the table, refire = `(int)(seconds * fps)`, weapon 0 owned,
    /// current/stash/cooldown zeroed.
    pub fn reset(fps: i32) -> Loadout {
        let mut slots = [WeaponSlot { flags: 0, total: 0, clip: 0, clip_size: 0, last_fire: -1000, p34: 0, refire: 0, shots: 0 }; 32];
        for (i, s) in slots.iter_mut().enumerate() {
            let d = &WEAPONS[i];
            s.clip = d.clip;
            s.clip_size = d.clip_size;
            s.p34 = d.p34;
            s.refire = (f32::from_bits(REFIRE_BITS[i]) * fps as f32) as i32;
        }
        slots[0].flags |= 1;
        Loadout { slots, current: 0, other: 0, firing: false, reload_pending: false, cooldown: 0, counted_shots: 0 }
    }

    /// `FUN_0020d378(mgr, i, count, clip_size, p34)`: owned; in POV mode the clip size is
    /// overridden (when > 0) and the clip is `min(count, clip size)`; `p34` overridden when > 0;
    /// the count adds to the carried rounds.
    pub fn register(&mut self, i: usize, count: i32, clip_size: i32, p34: i32, pov: bool) {
        let s = &mut self.slots[i];
        s.flags |= 1;
        if clip_size > 0 && pov { s.clip_size = clip_size; }
        if p34 > 0 { s.p34 = p34; }
        if pov { s.clip = count.min(s.clip_size); }
        s.total += count;
    }

    /// `FUN_00205498` (the MSet branch): reset, register every enabled weapon (index 0x17 and
    /// counts >= 0x2711 skipped), select the first enabled weapon past index 0 with a count
    /// >= 1 (none: register weapon 0 empty and select it); the Ultralight gun 0x1b becomes the
    /// Ultralight rocket 0x1c on `jungleb_mis13b` when 0x1c is enabled with a count.
    pub fn from_mset(ms: &MissionSet, pov: bool, fps: i32, level_key: &str) -> Loadout {
        let mut l = Loadout::reset(fps);
        let mut sel: Option<usize> = None;
        for i in 0..32 {
            if i == POV_BASE || !ms.enabled[i] || ms.target[i] >= MSET_COUNT_LIMIT { continue; }
            l.register(i, ms.target[i], ms.param2[i], ms.param3[i], pov);
            if sel.is_none() && i != 0 && ms.target[i] >= 1 { sel = Some(i); }
        }
        l.current = match sel {
            None => { l.register(0, 0, 0, 0, pov); 0 }
            Some(0x1b) if ms.enabled[0x1c] && ms.target[0x1c] > 0 && level_key == "jungleb_mis13b" => 0x1c,
            Some(s) => s,
        };
        l
    }

    /// `ECustomTransition("paris")` `FUN_0014b020`: the MSet copy loses every POV weapon
    /// (enabled 0x17..0x1f cleared); then either weapon 0 gets 500 rounds (flag clear) or the
    /// Missile count is zeroed (flag set), and `FUN_00205498` re-registers from the copy.
    /// `retry_flag` is `DAT_0032e690`: set only on `paris_mis01` when PTP+0x618 == 1 and
    /// PTP+0x984 == 0 and PTP+0xa18 == 0 [M 0x174bf0:269-276]; what that state means is not
    /// traced.
    pub fn custom_transition(ms: &MissionSet, retry_flag: bool, pov: bool, fps: i32, level_key: &str) -> Loadout {
        let mut m = ms.clone();
        for i in POV_BASE..32 { m.enabled[i] = false; }
        if !retry_flag { m.target[0] = 500; m.enabled[0] = true; } else { m.target[2] = 0; }
        Loadout::from_mset(&m, pov, fps, level_key)
    }

    /// The POV swap at the head of `FUN_0020c078`, then the reload completion `FUN_0020e978`,
    /// then the swap back, then the gunner switch (`kind + 0x17 != current` past the cooldown).
    pub fn update(&mut self, pov: bool, now: i32, gunner: Option<&mut PovGunner>) -> Vec<GunnerOut> {
        if pov && self.current < POV_BASE {
            std::mem::swap(&mut self.current, &mut self.other);
            let c = self.current;
            if !(c >= POV_BASE && self.slots[c].total > 0 && self.slots[c].owned()) {
                if let Some(i) = (POV_BASE..32).find(|&i| self.slots[i].total > 0 && self.slots[i].owned()) {
                    self.current = i;
                }
            }
        }
        let mut out = Vec::new();
        let mut gunner = gunner;
        self.complete_reload(pov, gunner.as_deref());
        if !pov && self.current > POV_BASE - 1 {
            std::mem::swap(&mut self.current, &mut self.other);
            let c = self.current;
            if !(c < POV_BASE && self.slots[c].total > 0 && self.slots[c].owned()) {
                if self.slots[0].total >= 1 && self.slots[0].owned() {
                    self.current = 0;
                } else if let Some(i) = (1..POV_BASE).find(|&i| self.slots[i].total > 0 && self.slots[i].owned()) {
                    self.current = i;
                }
            }
        }
        if pov {
            if let Some(g) = gunner.as_deref_mut() {
                if g.kind as usize + POV_BASE != self.current && self.cooldown < now {
                    out.extend(g.switch_to(self.current as i64 - POV_BASE as i64));
                    let s = &mut self.slots[self.current];
                    if s.flags & 0x40 == 0 { s.flags |= 0x20; }
                }
            }
        }
        if let Some(g) = gunner { out.extend(self.auto_reload(pov, now, g)); }
        out
    }

    /// `FUN_0020de20` for the current weapon with the POV gunner present.
    /// `dry_fire_gate` = manager +0x5ec, `scope_up` = the scope object's +0x2c (both not traced
    /// to their writers here); `infinite` = `DAT_0033a008` (no deduction, no clip bookkeeping).
    pub fn fire_pov(&mut self, now: i32, gunner: &mut PovGunner, dry_fire_gate: bool, scope_up: bool, infinite: bool) -> (Fire, Vec<GunnerOut>) {
        let i = self.current;
        if self.slots[i].total < 1 { self.firing = false; return (Fire::NoAmmo, vec![]); }
        if dry_fire_gate && !scope_up && i == SNIPER_RIFLE { return (Fire::DryFire, vec![GunnerOut::Sound("SFX_DryFire")]); }
        if now <= self.cooldown { return (Fire::Held, vec![]); }
        let s = self.slots[i];
        if now < s.last_fire + s.refire { return (Fire::Held, vec![]); }
        if !gunner.ready() { return (Fire::Held, vec![]); }
        let out = gunner.fire();
        self.firing = true;
        let d = &WEAPONS[i];
        let s = &mut self.slots[i];
        s.last_fire = now;
        if d.counted { self.counted_shots += 1; }
        s.shots += 1;
        if infinite { return (Fire::Shot { emptied: false }, out); }
        // Every POV weapon takes the one-round branch (`iVar7 = 1`).
        s.total -= 1;
        s.clip = (s.clip - 1).max(0);
        let emptied = s.total >= 1 && s.clip <= 0;
        if emptied { s.flags |= 4; }
        (Fire::Shot { emptied }, out)
    }

    /// `FUN_0020e868` (the reload request): with the gunner ready, a clip below its size and
    /// below the rounds carried → pending + the gunner's reload.
    pub fn request_reload(&mut self, gunner: &mut PovGunner) -> Vec<GunnerOut> {
        let s = self.slots[self.current];
        if gunner.ready() && s.clip < s.clip_size && s.total > 0 && s.clip < s.total {
            self.reload_pending = true;
            let mut out = vec![GunnerOut::ReloadSound(WEAPONS[self.current].stem)];
            out.extend(gunner.reload());
            return out;
        }
        vec![]
    }

    /// `FUN_0020e978`: POV, rounds carried, pending, gunner ready, the dry flag clear →
    /// clip = min(carried, clip size).
    pub fn complete_reload(&mut self, pov: bool, gunner: Option<&PovGunner>) {
        let Some(g) = gunner else { return };
        let s = &mut self.slots[self.current];
        if pov && s.total > 0 && self.reload_pending && g.ready() && s.flags & 4 == 0 {
            s.clip = s.total.min(s.clip_size);
            self.reload_pending = false;
        }
    }

    /// `FUN_0020c078`'s empty-clip loop, POV branch. The loop visits the slots named by manager
    /// +0x00/+0x04/+0x08; the reset leaves +0x00 and +0x08 at -1 and their other writers are
    /// not traced, so the port visits the current weapon (+0x04). Once
    /// `last_fire + refire - 5 < now`, a ready gunner with rounds carried and an empty clip
    /// starts the reload and the dry flag clears (the one-entry ring wraps at once).
    fn auto_reload(&mut self, pov: bool, now: i32, gunner: &mut PovGunner) -> Vec<GunnerOut> {
        let mut out = Vec::new();
        if !pov { return out; }
        {
            let i = self.current;
            let s = self.slots[i];
            if s.flags & 4 == 0 || s.last_fire + s.refire - 5 >= now { return out; }
            if !(gunner.ready() && s.total > 0 && s.clip < 1) { return out; }
            out.extend(gunner.reload());
            self.reload_pending = true;
            self.slots[i].flags &= !4;
        }
        out
    }
}

// ---------------------------------------------------------------------------
// The gunner actor
// ---------------------------------------------------------------------------

/// The per-kind anim tables (kind 0..=7, -1 = none) [D DRIVING.ELF data, read by the code named].
pub mod anim {
    /// 0x3274b0: the ready anim after a switch (`FUN_00123f90`, `FUN_00124240`).
    pub const READY: [i32; 8] = [-1, 1, 2, 4, 0, 1, 0, 1];
    /// 0x3274d0: the idle loop (`FUN_00124348`, replayed when it ends).
    pub const IDLE: [i32; 8] = [-1, 6, 7, 1, 6, 7, 5, 6];
    /// 0x3274f0: the put-away before a switch (`FUN_00123f90`; only the sniper has one).
    pub const PUT_AWAY: [i32; 8] = [-1, -1, -1, 3, -1, -1, -1, -1];
    /// 0x327510: the reload (`FUN_00124170`).
    pub const RELOAD: [i32; 8] = [-1, 8, 9, 5, 8, 9, 7, 8];
    /// 0x327530: the two alternating fire anims per kind (`FUN_001240c8`, `[kind][toggle]`).
    pub const FIRE: [[i32; 2]; 8] = [[-1, -1], [4, 4], [5, 3], [0, 0], [4, 4], [5, 3], [3, 3], [4, 2]];
    /// 0x327570: the scope-up anim (`FUN_00124440`).
    pub const ZOOM_IN: [i32; 8] = [-1, -1, -1, 7, -1, -1, -1, -1];
    /// 0x327590: the scope-down anim (`FUN_00124440`).
    pub const ZOOM_OUT: [i32; 8] = [-1, -1, -1, 6, -1, -1, -1, -1];
    /// 0x3275b0: the anim each kind's actor starts on (`FUN_00123950`).
    pub const SPAWN: [i32; 8] = [-1, 8, 9, 2, 8, 9, 7, 8];
    /// 0x327650: the anim bank (+0x78).
    pub const BANK: [u32; 8] = [0, 0xc, 0xc, 0xe, 0x10, 0x10, 0xf, 0xf];
    /// 0x3275d0: the anim set per kind (`FUN_0010cb88` loads it).
    pub const SET: [&str; 8] = ["", "5.a_ssm.b_ssm.110", "5.a_ssm.b_ssm.110", "6.a_snipe.b_snipe.70",
        "7.a_ul.b_ul.140", "7.a_ul.b_ul.140", "8.a_fod.b_fod.100", "8.a_fod.b_fod.100"];
    /// 0x3275f0: the actor model per kind.
    pub const MODEL: [&str; 8] = ["", "ssm_mg", "ssm_gr", "sniper", "ul_mg", "ul_gr", "fod_mg", "fod_gr"];
    /// 0x327610: the vehicle mount point the actor sits at (0x327630 the matching `_ROT`).
    pub const MOUNT: [&str; 8] = ["", "AIC_HSSNOW_POS", "AIC_HSSNOW_POS", "AIC_HSNIPE_POS",
        "AIC_HULTRA_POS", "AIC_HULTRA_POS", "AIC_HFOD_POS", "AIC_HFOD_POS"];
}

/// The sniper's POV kind (weapon 0x1a - 0x17).
pub const KIND_SNIPER: u32 = 3;

/// What the gunner asks of the outside world.
#[derive(Clone, Debug, PartialEq)]
pub enum GunnerOut {
    /// `FUN_00107c68(actor, bank, anim)`: start an anim on the current kind's actor.
    Play { kind: u32, anim: i32, bank: u32 },
    /// The actor model's +0x30 visibility word.
    Model { kind: u32, shown: bool },
    /// `FUN_001be570` on the scope object (the scope view comes up).
    ScopeOn,
    /// `FUN_001be660` (the scope view goes down).
    ScopeOff,
    /// `FUN_002f4b10(4, name, "POV Weapons", 0)`.
    Sound(&'static str),
    /// The manager's `"SFX_" + stem + "Reload"` in the weapon's bank (`FUN_0020e868`).
    ReloadSound(&'static str),
}

/// Bond's POV gunner (`AICharacter` with vtable 0x3696e0).
#[derive(Clone, Debug, PartialEq)]
pub struct PovGunner {
    /// +0x4c.
    pub state: u32,
    /// +0x50: what state 1 is waiting on (8 the ready anim, 0xd the put-away).
    pub next: u32,
    /// +0x54: the anim index last started.
    pub anim: i32,
    /// +0x58: the POV kind 1..=7.
    pub kind: u32,
    /// +0x98: whose actor is current (the kind it was built for, `DAT_003276c0[kind]`); a
    /// put-away keeps the old actor while +0x58 already names the new kind.
    pub actor: u32,
    /// +0x78: the anim bank.
    pub bank: u32,
    /// +0x150: a reload asked for while busy.
    pub reload_pending: bool,
    /// +0x154: hide the old kind's model on a switch (1 from the ctor `FUN_00123e28`).
    pub hide_on_switch: bool,
    /// +0x158: a switch asked for while busy (0 = none).
    pub pending_kind: u32,
    /// +0x164: the POV mode shows the gunner (`FUN_00123ef8` / `FUN_00123f10`).
    pub visible: bool,
    /// `DAT_00422420[kind]`: which of the two fire anims plays next.
    pub fire_toggle: [bool; 8],
    /// `DAT_003276e4`: the scope is up.
    pub zoomed: bool,
    /// `DAT_003276e0`: the scope is wanted up (the zoom input writes it).
    pub want_zoom: bool,
}

impl PovGunner {
    /// `FUN_00123950`: the gunner exists when any POV weapon is owned; it starts as the lowest
    /// owned kind, in state 8 on that kind's spawn anim, hidden; scope down and not wanted.
    pub fn spawn(loadout: &Loadout) -> Option<(PovGunner, Vec<GunnerOut>)> {
        let kinds: Vec<u32> = (1..8u32).filter(|&k| loadout.slots[k as usize + POV_BASE].owned()).collect();
        let &kind = kinds.first()?;
        let g = PovGunner {
            state: 8, next: 0, anim: 0, kind, actor: kind, bank: anim::BANK[kind as usize],
            reload_pending: false, hide_on_switch: true, pending_kind: 0, visible: false,
            fire_toggle: [false; 8], zoomed: false, want_zoom: false,
        };
        let mut out = Vec::new();
        for &k in &kinds {
            out.push(GunnerOut::Play { kind: k, anim: anim::SPAWN[k as usize], bank: anim::BANK[k as usize] });
            out.push(GunnerOut::Model { kind: k, shown: false });
        }
        Some((g, out))
    }

    fn play(&mut self, anim: i32) -> GunnerOut {
        self.anim = anim;
        GunnerOut::Play { kind: self.actor, anim, bank: self.bank }
    }

    /// `FUN_001241e0`: free to fire or reload (state 2 without a pending reload).
    pub fn ready(&self) -> bool { self.state == 2 && !self.reload_pending }

    /// `FUN_001240c8`.
    pub fn fire(&mut self) -> Vec<GunnerOut> {
        if self.state != 2 { return vec![]; }
        self.state = 11;
        let k = self.kind as usize;
        let a = anim::FIRE[k][self.fire_toggle[k] as usize];
        self.fire_toggle[k] = !self.fire_toggle[k];
        vec![self.play(a)]
    }

    /// `FUN_00124170`.
    pub fn reload(&mut self) -> Vec<GunnerOut> {
        if self.state == 2 {
            self.state = 12;
            self.reload_pending = false;
            let a = anim::RELOAD[self.kind as usize];
            vec![self.play(a)]
        } else {
            self.reload_pending = true;
            vec![]
        }
    }

    /// `FUN_00123ef8`: the POV mode comes on — show the model.
    pub fn show(&mut self) -> Vec<GunnerOut> {
        self.visible = true;
        vec![GunnerOut::Model { kind: self.actor, shown: true }]
    }

    /// `FUN_00123f10`: the POV mode goes off — a put-away in progress finishes at once, then
    /// the model hides.
    pub fn hide(&mut self) -> Vec<GunnerOut> {
        let mut out = Vec::new();
        if self.state == 1 && self.next == 0xd {
            self.state = 2;
            if self.pending_kind != 0 { self.kind = self.pending_kind; }
            self.pending_kind = 0;
            if self.hide_on_switch { out.push(GunnerOut::Model { kind: self.actor, shown: false }); }
            self.actor = self.kind;
        }
        out.push(GunnerOut::Model { kind: self.actor, shown: false });
        self.visible = false;
        out
    }

    /// `FUN_00123f90`: switch to kind `n` (out of 1..=7 → 1).
    pub fn switch_to(&mut self, n: i64) -> Vec<GunnerOut> {
        let n = if (1..8).contains(&n) { n as u32 } else { 1 };
        if self.state != 2 { self.pending_kind = n; return vec![]; }
        if anim::PUT_AWAY[self.kind as usize] == -1 {
            let mut out = Vec::new();
            self.next = 8;
            self.kind = n;
            self.state = 1;
            self.pending_kind = 0;
            if self.hide_on_switch { out.push(GunnerOut::Model { kind: self.actor, shown: false }); }
            self.actor = n;
            if self.visible { out.push(GunnerOut::Model { kind: n, shown: true }); }
            self.bank = anim::BANK[n as usize];
            let a = anim::READY[n as usize];
            out.push(self.play(a));
            out
        } else {
            self.state = 1;
            self.next = 0xd;
            let a = anim::PUT_AWAY[self.kind as usize];
            let out = vec![self.play(a)];
            self.kind = n;
            self.pending_kind = 0;
            out
        }
    }

    /// The zoom input (`FUN_001be038` cases 0x38 press / 0x3a release, without the scope
    /// object's guards): press raises the scope; in toggle mode (PTP+0xa44 != 0) a press with
    /// the scope up lowers it; in hold mode the release lowers it.
    pub fn zoom_input(&mut self, pressed: bool, toggle_mode: bool) {
        match (pressed, toggle_mode) {
            (true, _) if !self.zoomed => self.want_zoom = true,
            (true, true) => self.want_zoom = false,
            (false, false) => self.want_zoom = false,
            _ => {}
        }
    }

    /// One frame (`FUN_00118af0`'s state switch). `anim_done` = `FUN_0010ac58` on the current
    /// actor, `anim_time` = `FUN_00107388` on the handle the last fire/zoom anim returned.
    pub fn update(&mut self, anim_done: bool, anim_time: f32) -> Vec<GunnerOut> {
        match self.state {
            1 => self.state_switching(anim_done),
            2 => self.state_idle(anim_done),
            8 => self.state_ready(anim_done),
            9 => self.toggle_zoom(),
            10 => self.state_zooming(anim_done, anim_time),
            11 => self.state_firing(anim_done, anim_time),
            12 => self.state_reloading(anim_done),
            13 => self.state_put_away(anim_done),
            _ => vec![],
        }
    }

    /// `FUN_001243f0` (state 8, and state 1 waiting on 8).
    fn state_ready(&mut self, anim_done: bool) -> Vec<GunnerOut> {
        if !anim_done { return vec![]; }
        self.state = 2;
        let mut out = Vec::new();
        if self.pending_kind != 0 { out = self.switch_to(self.pending_kind as i64); }
        self.pending_kind = 0;
        out
    }

    /// `FUN_00124800` (state 0xd, and state 1 waiting on 0xd).
    fn state_put_away(&mut self, anim_done: bool) -> Vec<GunnerOut> {
        if anim_done {
            self.state = 0;
            if self.pending_kind != 0 { self.kind = self.pending_kind; }
            self.pending_kind = 0;
        }
        vec![]
    }

    /// `FUN_00124240` (state 1).
    fn state_switching(&mut self, anim_done: bool) -> Vec<GunnerOut> {
        match self.next {
            8 => self.state_ready(anim_done),
            0xd => {
                let mut out = self.state_put_away(anim_done);
                if self.state != 1 {
                    self.next = 8;
                    self.state = 1;
                    // The old actor (+0x98) hides; the new kind's shows when visible.
                    if self.hide_on_switch { out.push(GunnerOut::Model { kind: self.actor, shown: false }); }
                    self.actor = self.kind;
                    if self.visible { out.push(GunnerOut::Model { kind: self.kind, shown: true }); }
                    self.bank = anim::BANK[self.kind as usize];
                    let a = anim::READY[self.kind as usize];
                    out.push(self.play(a));
                }
                out
            }
            _ => vec![],
        }
    }

    /// `FUN_00124348` (state 2).
    fn state_idle(&mut self, anim_done: bool) -> Vec<GunnerOut> {
        if self.reload_pending { return self.reload(); }
        if self.zoomed != self.want_zoom { return self.toggle_zoom(); }
        if anim_done {
            let a = anim::IDLE[self.kind as usize];
            return vec![self.play(a)];
        }
        vec![]
    }

    /// `FUN_00124440` (state 9, also called directly when the wanted scope differs).
    fn toggle_zoom(&mut self) -> Vec<GunnerOut> {
        let k = self.kind as usize;
        let mut out = Vec::new();
        if !self.zoomed {
            if anim::ZOOM_IN[k] == -1 {
                out.push(GunnerOut::ScopeOn);
            } else {
                self.state = 10;
                out.push(self.play(anim::ZOOM_IN[k]));
            }
        } else {
            if anim::ZOOM_OUT[k] != -1 && self.state != 11 && self.state != 12 {
                self.state = 10;
                out.push(self.play(anim::ZOOM_OUT[k]));
            }
            out.push(GunnerOut::ScopeOff);
            if self.visible { out.push(GunnerOut::Model { kind: self.actor, shown: true }); }
        }
        self.zoomed = !self.zoomed;
        out
    }

    /// `FUN_00124570` (state 10): back to idle when the anim ends; scoping in, the model hides
    /// and the scope view comes up while the anim time is in (4.9, 5.3).
    fn state_zooming(&mut self, anim_done: bool, t: f32) -> Vec<GunnerOut> {
        let mut out = Vec::new();
        if anim_done { self.state = 2; }
        if self.zoomed && 4.9 < t && t < 5.3 {
            out.push(GunnerOut::Model { kind: self.actor, shown: false });
            out.push(GunnerOut::ScopeOn);
        }
        out
    }

    /// The shared head of `FUN_00124650` / `FUN_00124770`: with the scope up (or a kind with no
    /// scope anim) a changed wish toggles the scope at once and ends the frame.
    fn scope_wish_interrupts(&mut self) -> Option<Vec<GunnerOut>> {
        let k = self.kind as usize;
        if !(!self.zoomed && anim::ZOOM_IN[k] != -1) && self.zoomed != self.want_zoom {
            return Some(self.toggle_zoom());
        }
        None
    }

    /// `FUN_00124650` (state 0xb): the sniper's bolt sound while the fire anim time is in
    /// (13.9, 14.1) (that frame skips the end test); back to idle when the anim ends, chaining
    /// a pending reload.
    fn state_firing(&mut self, anim_done: bool, t: f32) -> Vec<GunnerOut> {
        if let Some(out) = self.scope_wish_interrupts() { return out; }
        if self.kind == KIND_SNIPER && 13.9 < t && t < 14.1 {
            return vec![GunnerOut::Sound("SFX_sniperbolt")];
        }
        if anim_done {
            self.state = 2;
            if self.reload_pending { return self.reload(); }
        }
        vec![]
    }

    /// `FUN_00124770` (state 0xc).
    fn state_reloading(&mut self, anim_done: bool) -> Vec<GunnerOut> {
        if let Some(out) = self.scope_wish_interrupts() { return out; }
        if anim_done { self.state = 2; }
        vec![]
    }
}

// ---------------------------------------------------------------------------
// The mode flag and the scripted events that drive it
// ---------------------------------------------------------------------------

use nf_formats::drive_script::{MapScript, Ref, ScriptEvent};

/// The POV mode plus its weapon manager and gunner.
#[derive(Clone, Debug)]
pub struct PovSession {
    /// Weapon manager `DAT_0033a024`+0x350.
    pub pov: bool,
    /// The camera start orientation in turns (`FUN_001bb7c0(angle/360, pitch/360, cam, 1)`).
    pub cam_start_turns: Option<(f32, f32)>,
    pub loadout: Loadout,
    pub gunner: Option<PovGunner>,
    /// The vehicle is on a scripted spline (`EControlToSpline` / `EEndControlToSpline`).
    pub on_spline: Option<String>,
    /// `EChangeStage(n)`: the rule group the next mission update makes current
    /// (manager +0x10 → +0xc, `FUN_00206340`).
    pub next_stage: Option<u32>,
    /// `EResetPlayerCarPos`: the trigger the car is put at.
    pub reset_to_trigger: Option<usize>,
    /// `ESetVideo`: the movie (and subtitle) files.
    pub video: Option<(String, Option<String>)>,
    pub mset: MissionSet,
    pub level_key: String,
    pub fps: i32,
    pub retry_flag: bool,
}

impl PovSession {
    /// Mission load: the MSet registered with the mode off (the load runs before any script
    /// event [S: the call order of `FUN_00205498` against the first trigger fire is not
    /// traced]) and the gunner spawned when a POV weapon is owned.
    pub fn load(mset: MissionSet, level_key: &str, fps: i32, retry_flag: bool) -> (PovSession, Vec<GunnerOut>) {
        let loadout = Loadout::from_mset(&mset, false, fps, level_key);
        let (gunner, out) = match PovGunner::spawn(&loadout) { Some((g, o)) => (Some(g), o), None => (None, vec![]) };
        (PovSession {
            pov: false, cam_start_turns: None, loadout, gunner, on_spline: None, next_stage: None,
            reset_to_trigger: None, video: None, mset, level_key: level_key.to_string(), fps, retry_flag,
        }, out)
    }

    fn set_pov(&mut self, on: bool) -> Vec<GunnerOut> {
        let out = match (&mut self.gunner, on) {
            (Some(g), true) if !self.pov => g.show(),
            (Some(g), false) if self.pov => g.hide(),
            _ => vec![],
        };
        self.pov = on;
        out
    }

    /// Runs one scripted event (only the kinds this phase depends on; the rest are ignored).
    pub fn apply(&mut self, e: &ScriptEvent, script: &MapScript) -> Vec<GunnerOut> {
        match e.kind {
            // ESetAutoDriveCamOn `FUN_001576e8` / ...Movie `FUN_00157838`: only when off.
            77 | 234 => {
                if self.pov { return vec![]; }
                self.cam_start_turns = Some((e.f32(1).unwrap_or(0.0) / 360.0, e.f32(2).unwrap_or(0.0) / 360.0));
                self.set_pov(true)
            }
            // ESetAutoDriveCamOff `FUN_00157460` / ...Movie `FUN_00157540`.
            76 | 233 => self.set_pov(false),
            // ECustomTransition `FUN_0014b020` — only the "paris" name does anything.
            240 => {
                if e.reference(1).and_then(|r| script.string(r)).is_some_and(|s| s.strip_prefix("CHAR::") == Some("paris")) {
                    self.loadout = Loadout::custom_transition(&self.mset, self.retry_flag, self.pov, self.fps, &self.level_key);
                }
                vec![]
            }
            // EChangeStage: the ctor `FUN_00149828` stores the stage.
            41 => { self.next_stage = e.word(1); vec![] }
            202 => { self.on_spline = e.reference(2).and_then(|r| script.string(r)).map(str::to_string); vec![] }
            239 => { self.on_spline = None; vec![] }
            140 => { self.reset_to_trigger = e.reference(1).and_then(|r| script.trigger_of(r)); vec![] }
            78 => {
                let s = |i| e.reference(i).and_then(|r: Ref| script.string(r)).map(|s| s.trim_start_matches("CHAR::").to_string());
                self.video = s(1).map(|m| (m, s(2)));
                vec![]
            }
            _ => vec![],
        }
    }

    /// Runs a whole list in order [S: the event queue `FUN_00175730` runs a list in its
    /// stored order].
    pub fn apply_list(&mut self, list: u16, script: &MapScript) -> Vec<GunnerOut> {
        let evs = script.lists.get(&list).cloned().unwrap_or_default();
        evs.iter().flat_map(|e| self.apply(e, script)).collect()
    }

    /// One weapon-manager frame.
    pub fn update(&mut self, now: i32) -> Vec<GunnerOut> {
        self.loadout.update(self.pov, now, self.gunner.as_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mis01_like() -> MissionSet {
        let mut b = vec![0u8; 760];
        b[..10].copy_from_slice(b"helicopter");
        b[0x2c + 2] = 1;
        b[0x2c + 26] = 1;
        for i in 0..32 {
            b[0x4c + i * 4..0x50 + i * 4].copy_from_slice(&50i32.to_le_bytes());
            b[0xcc + i * 4..0xd0 + i * 4].copy_from_slice(&10i32.to_le_bytes());
        }
        b[0x4c + 8..0x50 + 8].copy_from_slice(&30i32.to_le_bytes());
        b[0x4c + 104..0x50 + 104].copy_from_slice(&24i32.to_le_bytes());
        b[0xcc + 8..0xd0 + 8].copy_from_slice(&4i32.to_le_bytes());
        b[0xcc + 104..0xd0 + 104].copy_from_slice(&30i32.to_le_bytes());
        MissionSet::parse(&b)
    }

    #[test]
    fn table_names_and_clips() {
        assert_eq!(WEAPONS[SNIPER_RIFLE].name, "Sniper Rifle");
        assert_eq!((WEAPONS[SNIPER_RIFLE].clip, WEAPONS[SNIPER_RIFLE].clip_size), (30, 30));
        assert_eq!(WEAPONS[MISSILE].name, "Missile");
        assert_eq!(WEAPONS[MISSILE].clip_size, 4);
        assert_eq!(WEAPONS[0].name, "Gun");
        assert_eq!(WEAPONS[23].name, "P2K");
        // The sniper's refire: (int)(0.083333336 * 60) = 5 ticks.
        assert_eq!(Loadout::reset(60).slots[SNIPER_RIFLE].refire, 5);
        assert_eq!(Loadout::reset(60).slots[MISSILE].refire, 40);
    }

    #[test]
    fn mset_load_selects_the_missile_and_pov_swaps_in_the_rifle() {
        let ms = mis01_like();
        let mut l = Loadout::from_mset(&ms, false, 60, "paris_mis01");
        assert_eq!(l.current, MISSILE);
        assert_eq!(l.slots[SNIPER_RIFLE].total, 24);
        assert_eq!(l.slots[MISSILE].total, 30);
        // Off at load: the table's clip stays (30), not min(24, 30).
        assert_eq!(l.slots[SNIPER_RIFLE].clip, 30);
        let (mut g, _) = PovGunner::spawn(&l).unwrap();
        assert_eq!(g.kind, KIND_SNIPER);
        l.update(true, 1, Some(&mut g));
        assert_eq!(l.current, SNIPER_RIFLE);
        assert_eq!(l.other, MISSILE);
        // Off again: the stash comes back.
        l.update(false, 2, Some(&mut g));
        assert_eq!(l.current, MISSILE);
    }

    #[test]
    fn pov_registration_caps_the_clip_at_the_count() {
        let ms = mis01_like();
        let l = Loadout::from_mset(&ms, true, 60, "paris_mis01");
        assert_eq!(l.slots[SNIPER_RIFLE].clip_size, 30);
        assert_eq!(l.slots[SNIPER_RIFLE].clip, 24);
        assert_eq!(l.slots[MISSILE].clip, 4);
    }

    #[test]
    fn custom_transition_drops_the_rifle_and_gives_the_gun_500() {
        let ms = mis01_like();
        let l = Loadout::custom_transition(&ms, false, true, 60, "paris_mis01");
        assert!(!l.slots[SNIPER_RIFLE].owned());
        assert_eq!(l.slots[0].total, 500);
        assert_eq!(l.slots[MISSILE].total, 30, "a fresh reset, not 60");
        assert_eq!(l.current, MISSILE, "index 0 is never the first pick");
        let r = Loadout::custom_transition(&ms, true, true, 60, "paris_mis01");
        assert_eq!(r.slots[MISSILE].total, 0);
        assert_eq!(r.slots[0].total, 0);
        assert_eq!(r.current, 0, "nothing with a count: weapon 0");
    }

    fn ready_gunner() -> (Loadout, PovGunner) {
        let ms = mis01_like();
        let mut l = Loadout::from_mset(&ms, true, 60, "paris_mis01");
        let (mut g, _) = PovGunner::spawn(&l).unwrap();
        g.update(true, 0.0); // spawn anim ends → idle
        assert_eq!(g.state, 2);
        l.update(true, 1, Some(&mut g));
        (l, g)
    }

    #[test]
    fn fire_spends_a_round_and_waits_for_refire_and_the_gunner() {
        let (mut l, mut g) = ready_gunner();
        let (f, out) = l.fire_pov(10, &mut g, false, true, false);
        assert_eq!(f, Fire::Shot { emptied: false });
        assert_eq!(out, vec![GunnerOut::Play { kind: 3, anim: 0, bank: 0xe }]);
        assert_eq!(g.state, 11);
        assert_eq!(l.slots[SNIPER_RIFLE].total, 23);
        assert_eq!(l.slots[SNIPER_RIFLE].clip, 23);
        // The gunner is busy: held even past the refire.
        assert_eq!(l.fire_pov(20, &mut g, false, true, false).0, Fire::Held);
        // The bolt sound at anim time 14.0 holds the end test that frame.
        assert_eq!(g.update(true, 14.0), vec![GunnerOut::Sound("SFX_sniperbolt")]);
        assert_eq!(g.state, 11);
        g.update(true, 20.0);
        assert_eq!(g.state, 2);
        // Refire: last shot at 10, refire 5 → 14 is held, 15 fires.
        assert_eq!(l.fire_pov(14, &mut g, false, true, false).0, Fire::Held);
        assert!(matches!(l.fire_pov(15, &mut g, false, true, false).0, Fire::Shot { .. }));
    }

    #[test]
    fn dry_fire_gate_needs_the_scope() {
        let (mut l, mut g) = ready_gunner();
        assert_eq!(l.fire_pov(10, &mut g, true, false, false).0, Fire::DryFire);
        assert_eq!(l.slots[SNIPER_RIFLE].total, 24);
        assert!(matches!(l.fire_pov(10, &mut g, true, true, false).0, Fire::Shot { .. }));
    }

    #[test]
    fn an_empty_clip_auto_reloads_after_the_refire_and_refills() {
        let (mut l, mut g) = ready_gunner();
        l.slots[SNIPER_RIFLE].clip = 1;
        let (f, _) = l.fire_pov(100, &mut g, false, true, false);
        assert_eq!(f, Fire::Shot { emptied: true });
        assert_eq!(l.slots[SNIPER_RIFLE].flags & 4, 4);
        g.update(true, 20.0); // fire anim ends
        // last 100 + refire 5 - 5 = 100: not yet at 100, starts at 101.
        assert!(l.update(true, 100, Some(&mut g)).is_empty());
        let out = l.update(true, 101, Some(&mut g));
        assert_eq!(out, vec![GunnerOut::Play { kind: 3, anim: 5, bank: 0xe }]);
        assert_eq!(g.state, 12);
        assert_eq!(l.slots[SNIPER_RIFLE].flags & 4, 0);
        assert_eq!(l.slots[SNIPER_RIFLE].clip, 0);
        g.update(true, 0.0); // reload anim ends
        l.update(true, 102, Some(&mut g));
        assert_eq!(l.slots[SNIPER_RIFLE].clip, 23, "min(carried 23, clip 30)");
        assert!(!l.reload_pending);
    }

    #[test]
    fn scope_up_and_down_follow_the_wish() {
        let (_, mut g) = ready_gunner();
        g.zoom_input(true, false);
        let out = g.update(false, 0.0);
        assert_eq!(out, vec![GunnerOut::Play { kind: 3, anim: 7, bank: 0xe }]);
        assert!(g.zoomed);
        assert_eq!(g.state, 10);
        assert_eq!(g.update(false, 5.0), vec![GunnerOut::Model { kind: 3, shown: false }, GunnerOut::ScopeOn]);
        g.update(true, 6.0);
        assert_eq!(g.state, 2);
        g.zoom_input(false, false);
        let out = g.update(false, 0.0);
        assert_eq!(out, vec![GunnerOut::Play { kind: 3, anim: 6, bank: 0xe }, GunnerOut::ScopeOff]);
        assert!(!g.zoomed);
    }

    #[test]
    fn switching_off_the_rifle_plays_the_put_away_first() {
        let (_, mut g) = ready_gunner();
        let out = g.switch_to(1);
        assert_eq!(out, vec![GunnerOut::Play { kind: 3, anim: 3, bank: 0xe }]);
        assert_eq!((g.state, g.next, g.kind), (1, 0xd, 1));
        let out = g.update(true, 0.0);
        assert_eq!(g.state, 1);
        assert_eq!(g.next, 8);
        assert_eq!(out.last(), Some(&GunnerOut::Play { kind: 1, anim: 1, bank: 0xc }));
        g.update(true, 0.0);
        assert_eq!(g.state, 2);
        // Out-of-range kinds clamp to 1.
        assert_eq!(g.switch_to(-21).len(), 2);
        assert_eq!(g.kind, 1);
    }

    #[test]
    fn a_reload_asked_while_firing_chains_after_the_shot() {
        let (mut l, mut g) = ready_gunner();
        l.fire_pov(10, &mut g, false, true, false);
        assert!(g.reload().is_empty());
        assert!(g.reload_pending);
        let out = g.update(true, 20.0);
        assert_eq!(out, vec![GunnerOut::Play { kind: 3, anim: 5, bank: 0xe }]);
        assert_eq!(g.state, 12);
    }
}
