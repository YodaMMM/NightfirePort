//! Bond's trigger (`Player_WeaponFiring` 0x1a5460, called from `Player_Update`): shots per pull, the refire interval
//! and the clip, plus the weapon-switch state machine (`Player_WeaponChange` 0x1a6f80 → `Player_SetWeaponAnimObj`
//! 0x1a2e38). Spec: docs/systems/player.md "Aim and bullets" + docs/systems/pickups.md "Bond's inventory".
//!
//! `Player_ResetWalkAnim` 0x195a88 (16 B, `Player_Weapon` 0x1a24c8:162's equip edge, the bit0==0
//! arm): `*(u8*)(*(obj+0xE0) + 0x954) = 0xFF` [M 0x195a88:6]. BL+0x954 is write-only across the
//! whole decompile (bldata.md:186 — no reader; the other writer is 0x190630), so there is nothing
//! to observe; recorded here at the weapon-switch code that would carry it.

/// The trigger state of the selected weapon: clip (`BLData+0x1B8 + w·0xC`), spare rounds of its ammo type
/// (`BLData+0x170`), the refire timer (`+0x92C`), the shots left in this pull (`+0x936`).
#[derive(Clone, Debug)]
pub struct Gun {
    pub weapon: u8,
    pub clip: u32,
    pub spare: u32,
    pub timer: f32,
    pub pending: i32,
    held: bool,
    /// `anim+0xF4` reload states: 0 = idle, 5 = reload start (+0xA8), 6 = shell loop (+0xA4, one shell),
    /// 7 = reload end (+0xAC). The view-script block in `Sim::run` drives these [M Player_SetWeaponAnimObj].
    pub reload_state: u8,
    /// Ticks remaining in the current reload script (5/6/7). When it hits 0, the state transitions.
    pub reload_ticks: u32,
    /// Set by `reload_tick` when the shell loop continues (state 6 stays 6 but the +0xA4 script must replay
    /// from frame 0 for the next shell). The view-script block in `Sim::run` checks this to re-queue +0xA4.
    pub reload_loop_restarted: bool,
    /// `anim+0x63`: the switch-request weapon id (set by `Player_WeaponChange`, consumed by
    /// `Player_SetWeaponAnimObj`). When it differs from `weapon`, the holster/draw cycle runs.
    pub switch_request: u8,
    /// `anim+0x64`: the saved weapon id (set by `Player_WeaponNone` for the toggle-back from hands) [M 0x1a9858].
    pub saved_weapon: u8,
    /// `BLData+0x930`: the last weapon (for the toggle-back when cycling from a gadget to a weapon) [M 0x1a6f80].
    pub prev_weapon: u8,
    /// `BLData+0x932`: the last gadget (for the toggle-back when cycling from a weapon to a gadget) [M 0x1a6f80].
    pub prev_gadget: u8,
    /// `BLData+0x959` / `anim+0xF4`: the view-script state machine's phase.
    /// 0 = idle, 1 = holstering, 2 = holster-done (swap), 3 = drawing, 4 = draw-done, 8 = variant-swap.
    pub switch_state: u8,
    /// View-anim state 9 (`anim+0xF4 = 9`): a fire script is playing (`Player_SetFiringAnim` 0x1a3c70:52
    /// writes the state before the script is queued). While it holds, `Player_WeaponChange` refuses every
    /// switch and the trigger can only refire through the state-9 gate [M 0x1a6f80:33-47, 0x1a5460:631-648].
    pub fire_state: bool,
    /// `BLData+0x95C`: the bullet-spawned latch. Cleared when a pull starts; set when the round spawns —
    /// on the press tick for `fire_anim_len == 0` weapons [M 0x1a5460:688-690], else at the release frame
    /// (`fire_anim_len <= script frame`, `Player_SetWeaponAnimObj` case 9 0x1a2e38:416-420).
    pub latch: bool,
    /// `BLData+0x958`: the fire-script alternation toggle. `Player_SetFiringAnim` (0x1a3c70:40-49) flips it
    /// per shot for flag-0x40000 weapons and picks +0xB0 / +0xB4 by its parity (the game reuses the same byte
    /// for the flag-0x1000 reload's frame-6 toggle, which the port's reload math covers separately).
    pub alt_fire_toggle: u8,
    /// `BLData+0x1BC + base·0xC`: the twin-variant cycle marker — (equipped twin id − base id), written by
    /// `Player_WeaponSelect` 0x1a2cc0:15-16/27-28 on every same-base switch (0 when the base itself is
    /// equipped). `Player_WeaponChange` 0x1a6f80:180-195 reads it during the cycle walk: from a selectable,
    /// owned candidate `cand` the cursor first offers `cand + skip[cand]` (the twin) and lands there when
    /// `Player_WeaponHasAmmo` passes for it; only when the twin is dry (and skip > 0) does it fall back to
    /// `cand`, clearing the marker. So with the P2K silencer equipped (`BL+0x1BC[0x02] = 1`) cycling to the
    /// pistol hands the SILENCED 0x03 twin while it has ammo [M]. Indexed by weapon id (base only).
    pub twin_skip: Vec<i8>,
    /// `BLData+0x964`: the sight-laser latch. `Player_SetWeaponAnim` 0x1a9338:21-26 writes it from
    /// `weapon_data[w]+0x68 & 0x8` on every completed draw/switch (the port sets it wherever the
    /// swap commits the new weapon); `Player_LaserPointer` 0x1a65d8:93 returns while it is 0, so
    /// the SG5's beam lives only while the semi twin 0x17 (the record with the bit) is held [D].
    /// The anim-script toggle (`AnimProcessScriptCmds` 0x1c1b38 case 1) can flip it outside the
    /// draw path — the port does not port that command (no scripted user known); [S] noted.
    pub laser_on: bool,
    /// weapon-handling: the running reload was started by the reload key (branch A — +0xA4 from
    /// frame 0) rather than by the empty-clip auto-reload (branch B — frame 6 for +0x68 & 0x1000).
    pub reload_by_key: bool,
    /// weapon-handling: the caller's view-state gate for this tick (`Player_WeaponFiring`
    /// 0x1a5460:631-648 — state 9 with `+0x68 & 0x8000`, or a state outside {0, 9, 0xB}): the
    /// countdown runs, the shot does not.
    pub refire_blocked: bool,
    /// Set by `update_with` on the tick `Player_WeaponFiring` 0x1a5460 reaches
    /// `Player_HandleHasNoAmmo`: trigger up, no round to fire, view idle, and
    /// `Player_ReloadAmmoType(_, 1)` refused [M 0x1a5460 release arm]. The Sim answers it with
    /// `Gun::no_ammo_request`.
    pub dry: bool,
}

impl Gun {
    /// `Player_WeaponChange` (0x1a6f80): the weapon cycle (`param_3` = 0) or the gadget cycle (`param_3` = 1).
    /// The walk [M 0x1a6f80:145-199]: the cursor is the REQUEST byte `+0x63` (so a press during a holster or
    /// draw walks on from the last request), pulled to the low bound when it sits outside the mode's
    /// `[lo, hi]` (0..0x48, 0x49..0x5F), then to its record's BASE (`weapon_data[+0x63]+2`) — a held twin
    /// walks from its family, so the step down from the silenced 0x03 leaves the P2K instead of re-offering
    /// it. Each step adds `dir` and wraps: `>= hi` → `lo + 1`, `<= lo` → `hi − 1`.
    /// The game does NOT check the first-person model here (that's `Player_WeaponSelect`'s job); weapon 1
    /// (fists) is a valid cycle target.
    /// The twin step [M 0x1a6f80:179-198]: a selectable candidate `cand` first offers `cand + twin_skip[cand]`
    /// (the equipped twin) and lands there when it has ammo; when the twin is dry the walk falls back to `cand`
    /// (only for a POSITIVE marker) and CLEARS the marker (the game's `BL+0x1BC[cand] = 0`).
    /// Returns the id to switch to, or `None` when the walk runs out (the game then writes `+0x63 = +0x62`,
    /// no change [M :200]).
    pub fn cycle(&mut self, dir: i8, mode: CycleMode, owned: &[bool], has_ammo: impl Fn(u8) -> bool, selectable: impl Fn(u8) -> bool, base_of: impl Fn(u8) -> u8) -> Option<u8> {
        let dir = dir.signum() as i32;
        if dir == 0 { return None; }
        let (first, last) = mode.bounds();
        let (lo, hi) = (first - 1, last + 1);
        let mut w = self.switch_request as i32;
        if w < lo || w > hi { w = lo; }
        w = base_of(w as u8) as i32;
        for _ in 0..0x72 {
            w += dir;
            if w >= hi { w = lo + 1; }
            if w <= lo { w = hi - 1; }
            let wu = w as u8;
            if !selectable(wu) { continue; }
            if !owned.get(wu as usize).copied().unwrap_or(false) { continue; }
            // The twin step [M 0x1a6f80:180-195]: the cursor first points at `cand + twin_skip[cand]`
            // (the equipped twin) and lands there when it has ammo; when the twin is dry the walk
            // falls back to the candidate, CLEARING the marker, and lands when the candidate has
            // ammo; when both are dry the walk continues past (`cursor = cand`, advance).
            let skip = *self.twin_skip.get(wu as usize).unwrap_or(&0);
            let twin_step = skip != 0 && wu != 0x54;
            let offer = if twin_step { (wu as i32 + skip as i32) as u8 } else { wu };
            if has_ammo(offer) { return Some(offer); }
            if twin_step && skip > 0 && has_ammo(wu) {
                self.twin_skip[wu as usize] = 0;
                return Some(wu);
            }
        }
        None
    }

    /// `Player_WeaponChange` toggle-back path [M 0x1a6f80 lines 66-121]. When cycling weapons (mode=0) from a
    /// gadget, if `prev_weapon` (+0x930) is set and has ammo, switch directly to it — the game gates on the
    /// REMEMBERED ID's BASE (`weapon_data[+0x930].+2`): the base's selectable (+4) and the base's owned slot
    /// (`BL+0x1BA + base·0xC` — the twins share it, `Player_EquipWeapon` 0x1a8a58:39-41), then
    /// `Player_WeaponHasAmmo` for the remembered id itself, and restores +0x930 VERBATIM (line 113) — so the
    /// SILENCED twin 0x03 remembered in +0x930 comes back silenced [M]. Mirrors for +0x932 (the 0x55→0x54 fixup).
    /// Returns the toggle target, or `None` if no toggle applies (fall through to the cycle).
    pub fn toggle_back(&self, mode: CycleMode, owned: &[bool], has_ammo: impl Fn(u8) -> bool, selectable: impl Fn(u8) -> bool, base_of: impl Fn(u8) -> u8) -> Option<u8> {
        let cur = self.weapon;
        match mode {
            CycleMode::Weapons => {
                if cur >= 0x4a && cur <= 0x5e && self.prev_weapon != 0 {
                    let w = self.prev_weapon;
                    let b = base_of(w) as usize;
                    if selectable(b as u8) && owned.get(b).copied().unwrap_or(false) && has_ammo(w) {
                        return Some(w);
                    }
                }
            }
            CycleMode::Gadgets => {
                if cur >= 1 && cur <= 0x47 && self.prev_gadget != 0 {
                    let w = self.prev_gadget;
                    let b = base_of(w) as usize;
                    if selectable(b as u8) && owned.get(b).copied().unwrap_or(false) && has_ammo(w) {
                        return Some(w);
                    }
                }
            }
        }
        None
    }

    /// `Player_WeaponHasAmmo` (0x1a16c0): whether the player has ammo for weapon `w` — carried rounds of its
    /// ammo type, or rounds in the clip (`Player_AmmoInGun`), or (for `ammo_type == 0`) always true (melee /
    /// unlimited weapons), or (for flag 0x20000 weapons) a matching ammo pickup on the ground. Weapon 0x45
    /// (the grenade-belt pseudo-weapon) checks `(i16)weapon_data[0x45].clip_size (100, = +0x48CE) <=
    /// BLData+0x4F4` — and `+0x4F4` is not a separate field: it is `clip[0x45]` (`0x1B8 + 0x45·0xC`), the
    /// carried grenade count [M 0x1a16c0:14-15; player.md "`+0x4F4` is NOT a separate field"].
    /// The 0x20000 arm's object scan needs the world's bullets: `placed_keeps_ammo`.
    /// `clip_slot` is `Player_AmmoInGun` 0x1a9190's pick (see `ammo_in_gun_slot`) — the game calls
    /// `Player_AmmoInGun` for this clip half [M 0x1a16c0:22-24], so every caller that holds the weapon
    /// table passes the shared-slot resolver.
    pub fn has_ammo(w: u8, carried: &[i16], clip: &[i16], ammo_type: impl Fn(u8) -> usize, _flags: impl Fn(u8) -> u32, clip_slot: impl Fn(u8) -> usize) -> bool {
        if w == 0x45 {
            // `(short)weapon_data._18638_2_ (= weapon_data[0x45].clip_size = 100) <= *(short*)(BL+0x4F4)`
            // [M 0x1a16c0:14-15]; G-029 previously stood in with `true`.
            return 100i16 <= clip.get(0x45usize).copied().unwrap_or(0);
        }
        let t = ammo_type(w);
        if carried.get(t).copied().unwrap_or(0) != 0 { return true; }
        // `Player_AmmoInGun`: rounds in the clip (the shared-slot pick, 0x1a9190).
        if clip.get(clip_slot(w)).copied().unwrap_or(0) != 0 { return true; }
        // `ammo_type == 0`: melee / unlimited weapons (fists, hands, gadgets) always have ammo [M 0x1a16c0].
        if t == 0 { return true; }
        // Flag 0x20000's object scan: `placed_keeps_ammo` (no object list in Gun).
        false
    }

    /// `Player_WeaponHasAmmo` 0x1a16c0's last arm [M :28-51]: a dry `+0x68 & 0x20000` record (the detonators
    /// 0x38/0x39, 0x5a, 0x5c) with a non-zero ammo type still has ammo while one of the player's own bullets
    /// of its family is in the world — a placed mine keeps its detonator live. `placed` lists those bullets'
    /// record bases.
    pub fn placed_keeps_ammo(ammo_type: usize, flags: u32, base: u16, placed: &[u16]) -> bool {
        ammo_type != 0 && flags & 0x20000 != 0 && placed.contains(&base)
    }

    /// `Player_AmmoInGun` 0x1a9190's clip-slot pick: a twin record (`+0x05 != 0`) reads the BASE's
    /// slot (record `+0x02`) unless the `+0x90` ammo-shared bytes differ (then its own); a non-twin
    /// (whose `+0x02` bases to itself [L ee_start_05.bin +0x02 sweep]) reads its own. The Phoenix
    /// Samurai's give fills `clip[0x33]` (the base 0x33's slot — both records' +0x90 is 0x1f [L
    /// ee_start_05.bin @ 0x2bf150]), so the Overcharge twin 0x32's dry check reads [0x33]: without
    /// this the mode press to 0x32 is forever inert and the Samurai fires nothing anywhere
    /// (weapon-matrix4 red #1).
    pub fn ammo_in_gun_slot(w: u8, base_of: impl Fn(u8) -> u8, shared90_of: impl Fn(u8) -> u8) -> usize {
        let b = base_of(w);
        if b != w && shared90_of(w) == shared90_of(b) { b as usize } else { w as usize }
    }

    /// `Player_GetBestWeapon` (0x1a9680): scans the `BestWeapon` table (0x2bede0, 18 u16) for the first owned,
    /// has-ammo family. An entry is a `weapon_data` index; its family is the record's `+2` base, and the
    /// candidate is base + the family's twin marker (`BL+0x1BC + base·0xC`, written by `Player_WeaponSelect`) —
    /// except the AIMS-20 0x1A, which always comes back on its rifle side [M 0x1a9680:18-23]. So a silenced
    /// P2K comes back silenced. The owned byte is the base's; the ammo check is the candidate's.
    pub fn get_best_weapon(&self, best: &[u16], base_of: impl Fn(u8) -> u8, owned: &[bool], has_ammo: impl Fn(u8) -> bool) -> Option<u8> {
        for &w in best {
            let base = base_of(w as u8);
            let id = if base == 0x1a { base } else {
                (base as i32 + *self.twin_skip.get(base as usize).unwrap_or(&0) as i32) as u8
            };
            if owned.get(base as usize).copied().unwrap_or(false) && has_ammo(id) {
                return Some(id);
            }
        }
        None
    }

    /// `Player_HandleHasNoAmmo` (0x1a2ba8), the request it writes. The fists 0x01, the Samurai pair 0x32/0x33
    /// and the recharging gadgets 0x4A..0x51 stay in hand; a `+0x68 & 0x10` record first offers its twin
    /// (`w + (+0x05)` — the AIMS-20's dry grenade side goes back to the rifle, the thrown-out Remote Mine goes
    /// to its detonator while a mine is down); everything else takes `Player_GetBestWeapon`. The deployed
    /// Ronin's 'R' arm sits with its own state in the Sim. `None` = the request stays.
    pub fn no_ammo_request(&self, twin: i8, flags68: u32, best: &[u16], base_of: impl Fn(u8) -> u8, owned: &[bool], has_ammo: impl Fn(u8) -> bool) -> Option<u8> {
        let w = self.weapon;
        if matches!(w, 1 | 0x32 | 0x33 | 0x4a..=0x52) { return None; }
        if flags68 & 0x10 != 0 {
            let t = (w as i32 + twin as i32) as u8;
            if has_ammo(t) { return Some(t); }
        }
        self.get_best_weapon(best, base_of, owned, has_ammo)
    }

    /// `Player_WeaponNone` (0x1a9858): Bond goes to "hands" (no weapon, weapon id 0x47). Saves the current weapon
    /// to `saved_weapon` (`anim+0x64`, for the toggle-back), sets `weapon = 0x47`, `switch_request = 0x47` (no
    /// switch triggered), stops the view script [M].
    pub fn go_to_hands(&mut self) {
        self.saved_weapon = self.weapon; // `anim+0x64 = anim+0x62` (save for toggle-back)
        self.weapon = 0x47;
        self.switch_request = 0x47; // `anim+0x63 = 0x47` — no switch triggered
        self.switch_state = 0;
    }

    /// Whether a weapon switch is in progress (the holster/draw cycle).
    pub fn switching(&self) -> bool { self.switch_state != 0 }

    /// Whether a reload is in progress (states 5/6/7).
    pub fn reloading(&self) -> bool { self.reload_state != 0 }

    /// Whether the view script needs to be restarted this tick (the shell loop continued — the +0xA4 script
    /// must replay from frame 0 for the next shell, matching `AnimScriptAdd(+0xA4)` in case 6 [M 0x1a2e38]).
    pub fn reload_loop_restarted(&self) -> bool { self.reload_state == 6 && self.reload_ticks == 0 }

    /// Advance the reload state machine one tick. Called by `Sim::run` after advancing the view script.
    /// `trigger_held` = the fire button is currently held (firing interrupt). `p` = the weapon params.
    /// Returns the new reload state (for the view-script block to pick the right script).
    pub fn reload_tick(&mut self, trigger_held: bool, p: GunParams) -> u8 {
        self.reload_loop_restarted = false;
        if self.reload_state == 0 { return 0; }
        if self.reload_ticks > 0 { self.reload_ticks -= 1; }
        if self.reload_ticks > 0 { return self.reload_state; }
        // The current script has finished. Transition.
        match self.reload_state {
            5 => {
                // Reload start done → queue +0xA4, go to state 6 [M case 5].
                self.reload_state = 6;
                self.reload_ticks = p.reload_loop_ticks.max(1);
            }
            6 => {
                // The +0xA4 script stopped: the rounds move [M case 6: Player_ReloadAmmoType(_, 0)].
                let n = if p.one_at_a_time { 1 } else { (p.clip - self.clip).min(self.spare) };
                let n = n.min(self.spare);
                self.clip += n;
                self.spare -= n;
                if p.one_at_a_time {
                    // Shell weapons (flags68 & 0x100): probe for more shells [M case 6:
                    // Player_ReloadAmmoType(_, 1)]. More and no fresh fire press → queue +0xA4 again,
                    // stay 6 (disasm 0x1a3620); none or a fire press → queue +0xAC, state 7.
                    let more = self.spare > 0 && self.clip < p.clip;
                    if more && !trigger_held {
                        self.reload_loop_restarted = true;
                        self.reload_ticks = p.reload_loop_ticks.max(1);
                    } else {
                        self.reload_state = 7;
                        self.reload_ticks = p.reload_end_ticks.max(1);
                    }
                } else {
                    // Non-shell weapons: straight to idle — case 6 falls to LAB_001a3678: the +0xA0 idle
                    // script, state 0, Input_ClearAction(9); +0xAC is never played (retail scan [L
                    // ee_start_05.bin]: only SPAS-12 0x1c/0x1d has +0xAC) [M 0x1a3678-0x1a369c].
                    self.reload_state = 0;
                    self.reload_ticks = 0;
                    self.held = false; // Input_ClearAction(9)
                    self.pending = 0;
                }
            }
            7 => {
                // Reload end done → go to idle [M case 7]. Clear the trigger action.
                self.reload_state = 0;
                self.reload_ticks = 0;
                self.held = false; // Input_ClearAction(9)
                self.pending = 0;
            }
            _ => {}
        }
        self.reload_state
    }
}

/// The cycle mode: weapons (1..0x47) or gadgets (0x4A..0x5E) [M `Player_WeaponChange` 0x1a6f80].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CycleMode {
    /// `param_3` = 0: cycle weapons (ids 1..0x47). Triggered by L2/R2 or Left/Right on the d-pad.
    Weapons,
    /// `param_3` = 1: cycle gadgets (ids 0x4A..0x5E). Triggered by Up/Down on the d-pad.
    Gadgets,
}

impl CycleMode {
    /// The cycle bounds [M disassembly: `slti v0,a0,71` (0x47) for weapons; `li s6,73`/`li v1,95` (0x49/0x5F)
    /// for gadgets, wrapping 0x4A..0x5E].
    pub fn bounds(&self) -> (i32, i32) {
        match self {
            CycleMode::Weapons => (1, 0x47),
            CycleMode::Gadgets => (0x4a, 0x5e),
        }
    }
}

/// The outcome of one press of the fire-mode button (`Player_Weapon` 0x1a24c8's ○ block).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModeSwitch {
    /// A twin-variant switch passed the has-ammo gate: the id to write to `anim+0x63`
    /// (the normal switch request — the holster/draw or variant-swap machinery runs).
    pub new_weapon: Option<u8>,
    /// The mode byte's new value (`BL + w*0xC + 0x1BB`); `None` = leave it.
    pub new_mode: Option<u8>,
}

/// `Player_Weapon` 0x1a24c8 (lines 204-260): the ○ press (`Input_Action(slot, 0xc, 4)`; for the
/// watch variants 0x54/0x55 the fire press ORs in) with the view anim idle and not
/// (aiming ∧ weapon flag 0x40):
/// - weapon_data[w]+0x05 == 0 → cycle the mode byte, wrapping at the record's mode count
///   (`+0x2E`, u8): `mode = mode + 1; if (mode == count) mode = 0`. Records with count 1 (every
///   gun; the table's +0x2E is 1 except the grenades 0x60/0x61 [L ee_start_05.bin]) don't move.
/// - weapon_data[w]+0x05 ≠ 0 (s8) → the twin-variant switch: `new = w + twin`; when
///   `Player_WeaponHasAmmo` (0x1a16c0 — carried rounds of the record's ammo type, rounds in the
///   clip, ammo type 0, or a matching ground pickup for flag-0x20000 weapons; it does NOT read the
///   owned byte) passes, write it to `obj+0x63` and reset the zoom floats. This is the P2K's
///   silencer (0x02 'Semi' ↔ 0x03 'Silenced'), the AT-600 Scorpion's heat-seeker (0x2E
///   'Heatseeker' ↔ 0x2F 'Unguided' — only 0x2E's `+0x6C` carries the 0x20000 homing bit [L]),
///   the SG5's 0x16 ↔ 0x17 and the watch's laser/taser pair.
pub fn fire_mode_switch(weapon: u8, twin: i8, mode_count: u8, mode: u8, has_ammo: impl Fn(u8) -> bool) -> ModeSwitch {
    if twin == 0 {
        if mode_count != 1 {
            let m = mode + 1;
            ModeSwitch { new_weapon: None, new_mode: Some(if m == mode_count { 0 } else { m }) }
        } else {
            ModeSwitch { new_weapon: None, new_mode: None }
        }
    } else {
        let new = (weapon as i32 + twin as i32) as u8;
        if has_ammo(new) {
            ModeSwitch { new_weapon: Some(new), new_mode: None }
        } else {
            ModeSwitch { new_weapon: None, new_mode: None }
        }
    }
}

/// `Player_Zoom` 0x1a1a70, one tick (`FRAME_RATE_MUL` = 1). `cur` is `BLData+0x8D0` (the
/// magnification the view divides its FOV by), `target` is `+0x8D4`, `max` is `+0x8D8` (the held
/// record's `weapon_data+0x88`, written on the aim bit's rise [M Player_Weapon 0x1a24c8:176-182]).
/// `axis` is the zoom action 6 (`Input_Actionf(slot, 6, 1)`), read only while the aim bit is set
/// [M 0x1a1a70:18-19]; the pad's d-pad up gives −1 (zoom in), down +1 (out) [M psiInput_MapInputs
/// style tables: `dpad: (6, 0x1000 → −1, 0x4000 → +1)`].
///
/// - Axis held: `f = 1 + 0.025` (`1/f` when the axis is positive); the target AND the current
///   magnification are scaled by `f` (the current one clamped at the target), then the target
///   takes the current value [M :34-41, :55-69, :70-71] — a geometric 1.025×/tick ramp.
/// - Axis idle: the current magnification eases toward the target by `1 + 0.15` per tick
///   (`1/1.15` from above), clamped at the target [M :21-26, :55-69]; the target is untouched.
/// - The target is clamped to `[1, min(max, 50)]` every tick [M :44-54].
///
/// Returns `Some(cur)` when the axis moved the zoom — the value the game stores as the base
/// record's remembered zoom (`BLData+0x1B4 + base·0xC`) [M :70-74].
pub fn zoom_tick(cur: &mut f32, target: &mut f32, max: f32, axis: f32) -> Option<f32> {
    let f8;
    let f5;
    if axis == 0.0 {
        f8 = *cur - *target;
        f5 = if f8 > 0.0 { 1.0 / (1.0 + 0.15) } else { 1.0 + 0.15 };
    } else {
        f5 = if axis > 0.0 { 1.0 / (1.0 + 0.025) } else { 1.0 + 0.025 };
        *target *= f5;
        f8 = axis;
    }
    if max < *target { *target = max; }
    if *target < 1.0 { *target = 1.0; } else if *target > 50.0 { *target = 50.0; }
    if f8 != 0.0 {
        *cur *= f5;
        let past = if f8 < 0.0 { *target < *cur } else { *cur < *target };
        if past { *cur = *target; }
    }
    if axis != 0.0 {
        *target = *cur;
        Some(*cur)
    } else { None }
}

/// The remembered zoom a base record starts with: `sqrt(weapon_data[w]+0x88)` per record
/// [M Player_InitAmmoWeapons 0x1a17f8:51 — `BL+0x1B4 + w·0xC = SQRT(weapon_data[w]+0x88)`].
/// The aim bit's rise restores it as the target when `PlayerSetting+6` is set (the boot default
/// 1 [M Input_Init 0x17edd8:22; L ee_start_05 0x2a38ce = 1]), clamped to the record's max
/// [M Player_Weapon 0x1a24c8:170-182].
pub fn initial_saved_zoom(max: f32) -> f32 { max.max(0.0).sqrt() }

/// The fields of a `weapon_data` record the trigger uses.
#[derive(Clone, Copy, Debug)]
pub struct GunParams {
    pub shots_per_pull: i32,
    pub interval: f32,
    pub clip: u32,
    pub ammo_per_shot: u32,
    pub reload_ticks: u32,       // legacy: total reload time for non-shotgun weapons
    /// weapon-handling: the reload KEY's time (branch A, `Player_WeaponFiring` disasm 0x1a5fb0-0x1a5fdc):
    /// +0xA4 at 1.35× from frame 0 — the frame-6 start of +0x68 & 0x1000 is the auto-reload's
    /// alone (branch B, 0x1a62e4-0x1a631c). Equal to `reload_ticks` for every other record.
    pub reload_key_ticks: u32,
    pub one_at_a_time: bool,
    pub reload_start_ticks: u32, // state 5: +0xA8 script length (0 if no start script)
    pub reload_loop_ticks: u32,  // state 6: +0xA4 script length per shell
    pub reload_end_ticks: u32,   // state 7: +0xAC script length
    pub ammo_type: u8,           // +0x90: 0 = melee (unlimited, no clip check), 1+ = ranged
    /// `+0x44`: the fire script's release frame — the tick the round spawns (`Player_SetWeaponAnimObj`
    /// case 9 0x1a2e38:416-420; `fire_anim_len == 0` spawns on the press tick [M 0x1a5460:688-690]).
    pub fire_anim_len: u32,
    /// `+0xB0`: the fire script. `Player_SetFiringAnim` 0x1a3c70 enters view state 9 ONLY when a
    /// fire script exists (its tail: `if (fp_model != 0 && hash != 0) { +0xf4 = 9; AnimScriptAdd }`).
    pub fire_script: u32,
    /// `+0xB4`: the fire-last / release script — the other half of the state-9 gate.
    pub fire_last: u32,
}

impl GunParams {
    /// `script_len(hash)`: the length of a first-person script (frames), for the reload time.
    pub fn from_record(w: &[u8], mode: usize, script_len: impl Fn(u32) -> Option<u32>) -> GunParams {
        let u32_at = |o: usize| u32::from_le_bytes(w[o..o + 4].try_into().unwrap());
        let flags68 = u32_at(0x68);
        let len_of = |h: u32| -> u32 {
            if h == 0 { return 0; }
            script_len(h).unwrap_or(60)
        };
        let start_h = u32_at(0xa8);
        let reload_h = u32_at(0xa4);
        let end_h = u32_at(0xac);
        GunParams {
            shots_per_pull: u16::from_le_bytes([w[0x28 + 2 * mode.min(3)], w[0x29 + 2 * mode.min(3)]]) as i32,
            interval: u32_at(0x40) as f32,
            clip: u16::from_le_bytes([w[0x92], w[0x93]]) as u32,
            // `Player_RoundToFire` 0x1a18e0 checks and spends the RAW +0x91 byte
            // (`(short)(param_2 & 0xff)` against the clip, `-(param_3 & 0xff)` on the spend): a 0
            // clip-add is always fireable and spends nothing — the Remote Mine's detonator 0x38/0x39,
            // the Stunner twins 0x4b/0x4d and the Shaver twins 0x5a/0x5c all carry +0x91 = 0
            // [L ee_start_05.bin]. The old `.max(1)` turned the zero-spend into a needs-1-round gate
            // and refused the detonator's fire outright (the F family: sounds, no shot tick).
            ammo_per_shot: w[0x91] as u32,
            reload_ticks: reload_ticks(start_h, reload_h, flags68, &script_len),
            reload_key_ticks: reload_ticks(start_h, reload_h, flags68 & !0x1000, &script_len),
            one_at_a_time: flags68 & 0x100 != 0,
            reload_start_ticks: len_of(start_h),
            reload_loop_ticks: len_of(reload_h),
            reload_end_ticks: len_of(end_h),
            ammo_type: w[0x90],
            fire_anim_len: u32_at(0x44),
            fire_script: u32_at(0xb0),
            fire_last: u32_at(0xb4),
        }
    }
}

/// The reload time (`Player_WeaponFiring` → anim state 6 → `Player_SetWeaponAnimObj` case 6: the rounds move when the
/// reload script stops): the start script `+0xA8` if any, then the reload script `+0xA4`; without a start script and
/// flag 0x200 clear the reload plays at 1.35 (`AnimScriptAddSpeed(0x3FACCCCD)`), from frame 6 when flag 0x1000
/// (`AnimListBuild` … `+0x90` = 6.0). P2K: (155 − 6) / 1.35 → 111 ticks (*suggested* to ±1: the stop rule of the
/// script player is not read). Unknown lengths fall back to 60.
pub fn reload_ticks(start: u32, reload: u32, flags68: u32, script_len: &impl Fn(u32) -> Option<u32>) -> u32 {
    let len = |h: u32| if h == 0 { Some(0) } else { script_len(h) };
    let (Some(a), Some(b)) = (len(start), len(reload)) else { return 60 };
    if start != 0 || flags68 & 0x200 != 0 { return a + b; }
    let from = if flags68 & 0x1000 != 0 { 6.0 } else { 0.0 };
    ((b as f32 - from).max(0.0) / 1.35).ceil() as u32
}

impl Gun {
    /// Bond's start on 07000005 (`Player_InitWeapon`): weapon 2 with 7 in the clip and 41 spare [L].
    pub fn new(weapon: u8, clip: u32, spare: u32) -> Gun { Gun { weapon, clip, spare, timer: 0.0, pending: 0, held: false, reload_state: 0, reload_ticks: 0, reload_loop_restarted: false, switch_request: weapon, saved_weapon: weapon, prev_weapon: 0, prev_gadget: 0, switch_state: 0, fire_state: false, latch: false, alt_fire_toggle: 0, twin_skip: vec![0i8; 0x72], laser_on: false, reload_by_key: false, refire_blocked: false, dry: false } }

    /// One tick with the trigger (action 9) `pressed` (no reload key, gun idle).
    pub fn update(&mut self, pressed: bool, p: GunParams) -> bool { self.update_with(pressed, false, false, p) }

    /// One tick (`Player_WeaponFiring` 0x1a5460): `reload` = action 14 pressed (R2: activate, else reload),
    /// `busy` = the gun's animation state is not idle (a fire script still playing). A fresh press with the timer run
    /// out arms `shots_per_pull` shots and sets the timer to 1; the timer falls by `FRAME_RATE_MUL` (1) per tick; a shot
    /// needs the timer at 0 or below, a shot left in the pull and a round in the clip (`Player_RoundToFire`), and sets
    /// the timer to max(interval, 1). A reload starts with the reload key, or with the trigger released and no round to
    /// fire — only while the gun is idle and there are spare rounds.
    ///
    /// The reload state machine (cases 5/6/7 of `Player_SetWeaponAnimObj`): state 5 (reload start, +0xA8) → state 6
    /// (shell loop, +0xA4, one shell per iteration) → state 7 (reload end, +0xAC) → state 0 (idle). For one-at-a-time
    /// weapons (flag 0x100), each state-6 iteration loads 1 shell. Firing interrupts: if the trigger is held when a
    /// shell's +0xA4 finishes, the reload aborts to state 7 (+0xAC). The `Sim::run` method in nf-view drives the
    /// view scripts and calls `reload_tick` to advance the state machine.
    pub fn update_with(&mut self, pressed: bool, reload: bool, busy: bool, p: GunParams) -> bool {
        // The order is `Player_WeaponFiring`'s [M 0x1a5460 from the first `Input_Action(9, 1)`]: a running
        // reload is only one more non-idle view state — the refire timer keeps falling through it, and the
        // state gate below refuses the shot; `Sim::run` drives the reload itself via `reload_tick`.
        self.dry = false;
        let view_idle = !busy && self.reload_state == 0;
        let can_reload = self.spare > 0 && self.clip < p.clip;
        // `Player_RoundToFire(_, +0x91, 0)`: ammo type 0 always has a round.
        let round = p.ammo_type == 0 || self.clip >= p.ammo_per_shot;
        if !pressed {
            // Branch A: the reload key (action 14) — read only with the trigger's level at 0 [M 0x1a5460:
            // `if (Input_Action(9, 1) == 0) { if (Input_Action(0xe, 4) …`]; it leaves the refire timer
            // (disasm 0x1a5f50-0x1a5f84 — no +0x92C store).
            if reload && view_idle && can_reload {
                self.start_reload(true, p);
                return false;
            }
            self.pending = 0; // `BL+0x936 = 0`
            if !round {
                self.held = false;
                if !view_idle { return false; } // `if (+0xF4 != 0) return`
                // Branch B: the auto-reload sits in the trigger's RELEASE path: the level read as 0
                // (Input_Update 0x17ee88 writes the action byte 0x01 while held, 0x05 on the fresh
                // press, 0 on release — disasm 0x17f084-0x17f100), so a HELD trigger with an empty clip
                // reloads nothing until it releases [M 0x1a5460 disasm 0x1a5fe8 → 0x1a61d0]. It also
                // zeroes the refire timer (`BLData+0x92C = 0`, 0x1a6244).
                if can_reload {
                    self.timer = 0.0;
                    self.start_reload(false, p);
                } else {
                    self.dry = true; // → `Player_HandleHasNoAmmo`
                }
                return false;
            }
        } else if !round {
            // Held with no round: `if (+0xF6 != 10) return` — before the press read and the countdown.
            self.held = true;
            return false;
        }
        let fresh = pressed && !self.held;
        if fresh && self.timer <= 0.0 { self.pending = p.shots_per_pull; self.timer = 1.0; }
        self.held = pressed;
        // weapon-handling [M 0x1a5460:601-629]: without a fresh press a run-out timer RETURNS before
        // the countdown — a held trigger whose timer ran out on a refused tick (the state gate below)
        // never refires until the next press.
        if self.timer <= 0.0 { return false; }
        self.timer -= 1.0;
        // weapon-handling [M 0x1a5460:631-648]: the view-state gate after the countdown — a state outside
        // {0, 9, 0xB} refuses the shot (the reload here, the rest in `refire_blocked`), and state 9
        // refuses it for `+0x68 & 0x8000` records (the pump Frinesi 0x1c, the snipers 0x1e-0x29: their
        // fire scripts outlast +0x40 [D 07000005 scripts]).
        if self.refire_blocked || self.reload_state != 0 { return false; }
        let mut fired = false;
        if self.timer <= 0.0 {
            self.pending -= 1;
            if self.pending >= 0 {
                // `Player_RoundToFire` (0x1a18e0): for ammo_type == 0 (melee), always round to fire and
                // don't touch the clip [M]. For ranged weapons, check and decrement the clip.
                if p.ammo_type == 0 || self.clip >= p.ammo_per_shot {
                    if p.ammo_type != 0 { self.clip -= p.ammo_per_shot; }
                    self.timer = p.interval.max(1.0);
                    // `Player_SetFiringAnim` (0x1a3c70) enters view state 9 ONLY when the fire script
                    // exists — its tail is `if ((*(int *)(param_3 + 0xdc)) != 0 && iVar7 != 0)
                    // { *(short *)(...+0xf4) = 9; AnimScriptAdd(...); }`, the hash falling back
                    // +0xB4 → +0xB0. A script-less record (the watch pair 0x54/0x55: +0xB0/+0xB4 = 0
                    // [D probe dump]) leaves the view state 0. The port's fire_state is the documented
                    // `anim+0xF4 == 9` proxy — the mode block's `anim+0xf4 == 0` gate reads it
                    // [M Player_Weapon 0x1a24c8:199-201 — `bVar5 != 0 && anim+0xf4 == 0`] — so the
                    // latch gates the same way: the old unconditional latch wedged the camera's mode
                    // block shut forever (the clear needs `view_script == +0xB0`, which never plays).
                    // The spawn latch (+0x95C) is separate and stays: the +0x44 == 0 press-tick spawn
                    // is behind `Player_RoundToFire`, not behind the state-9 store [M 0x1a5460:688-693].
                    if p.fire_script != 0 || p.fire_last != 0 { self.fire_state = true; }
                    self.latch = p.fire_anim_len == 0;
                    fired = true;
                } else { self.pending = 0; }
            } else { self.pending = 0; }
        }
        fired
    }

    /// A tick on which the trigger's level was read but the pull was refused before the press edge
    /// (`Player_WeaponFiring`'s early returns): the edge is spent all the same.
    pub fn hold_trigger(&mut self, pressed: bool) { self.held = pressed; }

    /// A reload starts (`key` = branch A). `Player_WeaponFiring` sets anim+0xF4 = 6 and queues +0xA4 at
    /// 1.35× — from frame 6 for flag-0x1000 weapons on the auto-reload only (AnimListBuild +0x90 = 6.0,
    /// disasm 0x1a6300-0x1a631c; the key's reload plays from frame 0, 0x1a5fb0-0x1a5fdc) — whose tick count
    /// is ceil((len − 6)/1.35): `p.reload_ticks` [M shared tail 0x1a6250-0x1a631c]. With a +0xA8 start script
    /// it sets state 5 instead and queues +0xA8 at 1.0 [M 0x1a6260-0x1a626c]; case 5 then queues +0xA4 at 1.0
    /// for the full length (0x1a2e38 case 5).
    fn start_reload(&mut self, key: bool, p: GunParams) {
        self.reload_state = if p.reload_start_ticks > 0 { 5 } else { 6 };
        self.reload_by_key = key;
        let loop_ticks = if key { p.reload_key_ticks } else { p.reload_ticks };
        self.reload_ticks = if p.reload_start_ticks > 0 { p.reload_start_ticks.max(1) } else { loop_ticks.max(1) };
        self.pending = 0;
        self.held = false;
    }

    /// The release frame (`Player_SetWeaponAnimObj` case 9, 0x1a2e38:416-420): the fire script's current
    /// frame has reached `fire_anim_len` (+0x44) and the round has not spawned yet (+0x95C). `frame` is the
    /// script time in frames (the game reads `script+0x90` after `AnimListBuild`; the port's script sounds
    /// use the same `time + 1` frame convention). Only for `fire_anim_len > 0` weapons — the others spawn
    /// on the press tick (latched there) [M 0x1a5460:688-690].
    pub fn release_frame_reached(&self, frame: f32, fire_anim_len: u32) -> bool {
        fire_anim_len > 0 && self.fire_state && !self.latch && frame + 1.0 >= fire_anim_len as f32
    }

    /// The fire script ended: state 9 → 0, the idle script re-queued [M 0x1a2e38 case 9:449-456].
    pub fn end_fire_state(&mut self) { self.fire_state = false; self.latch = false; }

    /// `Player_ReloadAmmoType(player, 1)` [M 0x1a2e38 case 0xc :523]: one round from the pool into
    /// the clip — the refill the grenade family's 0xb/0xc chain runs at the +0xB4 script's stop,
    /// BEFORE the draw (remote-mine-detonate). The same move reload_tick's state 6 makes; here it
    /// is the single-round form the game's call performs. False when the pool is dry or the clip
    /// is full (the game's failure arm runs Player_HandleHasNoAmmo).
    pub fn reload_one(&mut self, p: GunParams) -> bool {
        if self.clip >= p.clip || self.spare == 0 { return false; }
        self.clip += 1;
        self.spare -= 1;
        true
    }

    /// `Player_SetFiringAnim` (0x1a3c70): which first-person fire script this shot queues, in the game's
    /// order —
    /// 1. flag 0x1000 (reload-anim-from-6) or base weapon 0x11: the LAST round (`clip < 1`, read after the
    ///    shot's decrement) plays the fire-last script +0xB4 [M 0x1a3c70:25-33];
    /// 2. flag 0x80000 (the fists, +0x68 = 0x880A2 [L ee_start_05.bin record 1]): one `Rand(100)` draw per
    ///    shot; > 50 plays +0xB4 — the left/right punch alternation [M 0x1a3c70:34-36];
    /// 3. the fallback is the fire script +0xB0 [M 0x1a3c70:37-39];
    /// 4. flag 0x40000 overrides both: the `BLData+0x958` toggle flips per shot and picks +0xB0 / +0xB4 by
    ///    parity [M 0x1a3c70:40-49].
    /// Returns the script hashcode (0 when the weapon has neither script — the caller leaves the view alone).
    pub fn firing_anim_script(&mut self, flags68: u32, base: u16, clip: u32, fire: u32, fire_last: u32, rng: &mut crate::drone::Rng) -> u32 {
        let mut script = 0u32;
        if flags68 & 0x1000 != 0 || base == 0x11 {
            if clip < 1 { script = fire_last; }
        }
        if flags68 & 0x80000 != 0 && rng.rand(100) > 50 { script = fire_last; }
        if script == 0 { script = fire; }
        if flags68 & 0x40000 != 0 {
            self.alt_fire_toggle = 1 - self.alt_fire_toggle; // `'\x01' - +0x958` [M 0x1a3c70:41-42]
            script = if self.alt_fire_toggle == 0 { fire } else { fire_last };
        }
        script
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Held fire with an AUTOMATIC re-fires at the record's own rate: the fresh press arms the
    /// pull (`shots_per_pull`), each shot re-arms the refire timer at `+0x40` ticks [M
    /// Player_WeaponFiring 0x1a5460:601-655]. Weapon 0x12's record measures spp 999 / interval 6
    /// [L ee_start_05.bin weapon_data @ 0x2bf150, stride 0x10C: the u16 at +0x28 (mode 0) =
    /// 0x3E7, the tick count at +0x40 = 6].
    #[test]
    fn held_automatic_fires_at_the_record_rate() {
        let p = GunParams { shots_per_pull: 999, interval: 6.0, clip: 30, ammo_per_shot: 1, reload_ticks: 0, reload_key_ticks: 0, one_at_a_time: false, reload_start_ticks: 0, reload_loop_ticks: 0, reload_end_ticks: 0, ammo_type: 1, fire_anim_len: 0, fire_script: 0x0600_0900, fire_last: 0 };
        let mut g = Gun::new(0x12, 30, 60);
        let shots: Vec<usize> = (0..37).filter(|&t| g.update(true, p)).collect();
        assert_eq!(shots, [0, 6, 12, 18, 24, 30, 36], "held fire → the record's interval [M 0x1a5460:601-655; L weapon 0x12: spp 999, interval 6]");
        // Sister: the semi-automatic P2K held the same way fires ONCE (the pull re-arms on the
        // press edge only) — pinned on the level above and in fix_06_fire_live.rs.
    }

    #[test]
    fn one_shot_per_press_at_most_every_interval() {
        let p = GunParams { shots_per_pull: 1, interval: 8.0, clip: 7, ammo_per_shot: 1, reload_ticks: reload_ticks(0, 0x0600_0827, 0x1004, &|_| Some(155)), reload_key_ticks: reload_ticks(0, 0x0600_0827, 0x04, &|_| Some(155)), one_at_a_time: false, reload_start_ticks: 0, reload_loop_ticks: 111, reload_end_ticks: 0, ammo_type: 1, fire_anim_len: 0, fire_script: 0x0600_0823, fire_last: 0 };
        assert_eq!(p.reload_ticks, 111);
        let mut g = Gun::new(2, 7, 41);
        // Holding the trigger fires once.
        let held: Vec<bool> = (0..20).map(|_| g.update(true, p)).collect();
        assert_eq!(held.iter().filter(|&&f| f).count(), 1);
        assert!(held[0]);
        // Tapping every 4 ticks: a press only arms once the timer has run out before it, so presses 8 ticks after a
        // shot (timer still 1) are lost and the next one 12 ticks after fires.
        let mut g = Gun::new(2, 7, 41);
        let taps: Vec<usize> = (0..40).filter(|&t| g.update(t % 4 == 0, p)).collect();
        assert_eq!(taps, [0, 12, 24, 36]);
        assert_eq!(g.clip, 3);
        // Sister: an empty clip does not fire, then reloads.
        let mut g = Gun::new(2, 0, 41);
        assert!(!g.update(true, p));
        // Released: the reload starts (state 6, no start script for the P2K).
        assert!(!g.update(false, p));  // trigger released + empty clip → reload starts
        assert_eq!(g.reload_state, 6); // state 6 (shell loop)
        // Run the reload: the P2K (non-shell) reloads all rounds when the +0xA4 script stops and goes
        // straight to idle — LAB_001a3678 plays +0xA0 and sets state 0; +0xAC is never queued [M].
        for _ in 0..p.reload_ticks { g.reload_tick(false, p); }
        assert_eq!(g.reload_state, 0); // → state 0 (idle) directly, no state 7 hop
        assert_eq!((g.clip, g.spare), (7, 34));
    }

    #[test]
    fn cycle_skips_unowned_unselectable_and_empty() {
        // Three weapons: 2 (P2K, owned, has ammo), 0x16 (SG5, owned, has ammo), 0x24 (sniper, owned, no ammo).
        let owned = vec![false; 0x73];
        let mut owned = owned;
        owned[2] = true; owned[0x16] = true; owned[0x24] = true;
        let selectable = |w: u8| w == 2 || w == 0x16 || w == 0x24;
        let has_ammo = |w: u8| w == 2 || w == 0x16; // 0x24 has no ammo
        let mut g = Gun::new(2, 7, 41);
        // Next from P2K → SG5 (0x16).
        assert_eq!(g.cycle(1, CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), Some(0x16));
        // Next from SG5 → wraps to P2K (0x24 skipped: no ammo).
        let mut g2 = Gun::new(0x16, 30, 60);
        assert_eq!(g2.cycle(1, CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), Some(2));
        // Prev from P2K → wraps to SG5 (0x24 skipped).
        assert_eq!(g.cycle(-1, CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), Some(0x16));
        // Sister: with no ammo at all, cycle returns None.
        let no_ammo = |_w: u8| false;
        assert_eq!(g.cycle(1, CycleMode::Weapons, &owned, no_ammo, selectable, |w| w), None);
    }

    #[test]
    fn cycle_weapons_and_gadgets_are_separate() {
        // Weapons: 2 (owned, ammo). Gadgets: 0x4A (owned, ammo), 0x50 (owned, ammo).
        let owned = vec![false; 0x73];
        let mut owned = owned;
        owned[2] = true; owned[0x4a] = true; owned[0x50] = true;
        let selectable = |w: u8| w == 2 || w == 0x4a || w == 0x50;
        let has_ammo = |_w: u8| true;
        let mut g = Gun::new(2, 7, 41);
        // Weapon cycle from P2K (2) → wraps to 2 (only weapon).
        assert_eq!(g.cycle(1, CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), Some(2));
        // Gadget cycle from P2K (2, a weapon) → starts at 2, steps to 0x4A (first gadget).
        assert_eq!(g.cycle(1, CycleMode::Gadgets, &owned, has_ammo, selectable, |w| w), Some(0x4a));
        // Gadget cycle from 0x4A → 0x50.
        let mut g2 = Gun::new(0x4a, 1, 1);
        assert_eq!(g2.cycle(1, CycleMode::Gadgets, &owned, has_ammo, selectable, |w| w), Some(0x50));
        // Sister: weapon cycle from a gadget (0x4A) → wraps to 2 (first weapon).
        assert_eq!(g2.cycle(1, CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), Some(2));
    }

    /// The cycle bounds are pinned to the game's disassembly values [M disassembly 0x1a707c/0x1a7084:
    /// `li s6,73` (0x49) and `li v1,95` (0x5F) for gadgets; `slti v0,a0,71` (0x47) for weapons].
    #[test]
    fn cycle_bounds_match_the_disassembly() {
        assert_eq!(CycleMode::Weapons.bounds(), (1, 0x47));
        assert_eq!(CycleMode::Gadgets.bounds(), (0x4a, 0x5e));
        // Sister: a weapon at 0x48 is NOT in the weapon range (0x48 > 0x47) — it would be skipped.
        assert!(0x48 > CycleMode::Weapons.bounds().1);
        // Sister: a gadget at 0x5F is NOT in the gadget range (0x5F > 0x5E).
        assert!(0x5f > CycleMode::Gadgets.bounds().1);
    }

    /// `Player_WeaponNone` (0x1a9858) + `Player_GetBestWeapon` (0x1a9680): when Bond is at hands (id 0x47),
    /// `GetBestWeapon` scans the `BestWeapon` table for the first owned+has-ammo weapon and returns it [M].
    #[test]
    fn hands_recovers_to_best_weapon() {
        const BEST_WEAPON: [u16; 18] = [0x42, 0x32, 0x1a, 0x18, 0x1c, 0x16, 0x15, 0x08, 0x10, 0x0e, 0x0d, 0x0c, 0x12, 0x06, 0x04, 0x02, 0x0a, 0x01];
        let owned = vec![false; 0x73];
        let mut owned = owned;
        // Bond owns only the P2K (2) with ammo.
        owned[2] = true;
        let has_ammo = |w: u8| w == 2;
        let base_of = |w: u8| w; // identity: BestWeapon entries are weapon_data indices; base(id) = id for the P2K
        let mut g = Gun::new(0x47, 0, 0); // hands
        // GetBestWeapon finds the P2K.
        assert_eq!(g.get_best_weapon(&BEST_WEAPON, base_of, &owned, has_ammo), Some(2));
        // Go to hands saves the current weapon to saved_weapon (anim+0x64), sets weapon=0x47, switch_request=0x47.
        g.weapon = 2;
        g.go_to_hands();
        assert_eq!(g.weapon, 0x47); // hands
        assert_eq!(g.switch_request, 0x47); // no switch triggered [M 0x1a9858]
        assert_eq!(g.saved_weapon, 2); // saved for toggle-back (anim+0x64)
        // Sister: if nothing is owned, GetBestWeapon returns None.
        let empty = vec![false; 0x73];
        assert_eq!(g.get_best_weapon(&BEST_WEAPON, base_of, &empty, has_ammo), None);
    }

    /// `Player_GetBestWeapon` looks up the real weapon id via `weapon_data[entry*0x10c+2]` (the base weapon).
    /// BestWeapon entry 0x15 has base 0x14; entry 0x32 has base 0x33; entry 0x18 has base 0x19 [M 0x1a9680].
    #[test]
    fn get_best_weapon_uses_base_weapon_lookup() {
        const BEST_WEAPON: [u16; 18] = [0x42, 0x32, 0x1a, 0x18, 0x1c, 0x16, 0x15, 0x08, 0x10, 0x0e, 0x0d, 0x0c, 0x12, 0x06, 0x04, 0x02, 0x0a, 0x01];
        // Only weapon 0x14 (the base of BestWeapon entry 0x15) is owned with ammo.
        let owned = vec![false; 0x73];
        let mut owned = owned;
        owned[0x14] = true;
        let has_ammo = |w: u8| w == 0x14;
        // base_of: entry 0x15 → base 0x14 (the variant points at its base).
        let base_of = |w: u8| if w == 0x15 { 0x14 } else { w };
        let g = Gun::new(0x47, 0, 0);
        // GetBestWeapon should find 0x14 (the base of entry 0x15), not 0x15 itself.
        assert_eq!(g.get_best_weapon(&BEST_WEAPON, base_of, &owned, has_ammo), Some(0x14));
        // Sister: if base_of is identity (wrong), 0x15 is checked instead and not owned → None.
        let identity = |w: u8| w;
        assert_eq!(g.get_best_weapon(&BEST_WEAPON, identity, &owned, has_ammo), None);
    }

    /// The toggle-back: cycling weapons from a gadget jumps to the last weapon [M 0x1a6f80 lines 93-121].
    #[test]
    fn toggle_back_from_gadget_to_weapon() {
        let owned = vec![true; 0x73];
        let has_ammo = |_w: u8| true;
        let selectable = |_w: u8| true;
        let mut g = Gun::new(0x4a, 1, 1); // a gadget
        g.prev_weapon = 2; // last weapon was the P2K
        // Cycling weapons from a gadget → toggle to the P2K (not the cycle).
        assert_eq!(g.toggle_back(CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), Some(2));
        // Sister: cycling gadgets from a gadget → no toggle (None).
        assert_eq!(g.toggle_back(CycleMode::Gadgets, &owned, has_ammo, selectable, |w| w), None);
        // Sister: prev_weapon = 0 → no toggle.
        g.prev_weapon = 0;
        assert_eq!(g.toggle_back(CycleMode::Weapons, &owned, has_ammo, selectable, |w| w), None);
    }

    /// The walk's first step is from the REQUEST's base, inside the mode's bounds [M 0x1a6f80:145-162]:
    /// the gadget cycle from a weapon opens on the first gadget, the weapon cycle down from a gadget on
    /// the top of the weapon range, and a twin base above its twin (the Samurai 0x33's 0x32) walks on.
    #[test]
    fn the_walk_starts_at_the_requests_base() {
        let mut owned = vec![false; 0x73];
        for w in [1usize, 2, 0x33, 0x45, 0x4a, 0x5e] { owned[w] = true; }
        let selectable = |w: u8| matches!(w, 1 | 2 | 0x33 | 0x45 | 0x4a | 0x5e);
        let base = |w: u8| if w == 0x03 { 2 } else if w == 0x32 { 0x33 } else { w };
        let yes = |_w: u8| true;
        let mut g = Gun::new(2, 7, 41);
        assert_eq!(g.cycle(1, CycleMode::Gadgets, &owned, yes, selectable, base), Some(0x4a), "first gadget up");
        assert_eq!(g.cycle(-1, CycleMode::Gadgets, &owned, yes, selectable, base), Some(0x5e), "last gadget down");
        let mut g = Gun::new(0x4a, 1, 1);
        assert_eq!(g.cycle(-1, CycleMode::Weapons, &owned, yes, selectable, base), Some(0x45), "the top weapon");
        assert_eq!(g.cycle(1, CycleMode::Weapons, &owned, yes, selectable, base), Some(1), "the bottom weapon");
        let mut g = Gun::new(3, 7, 41);
        g.twin_skip[2] = 1;
        assert_eq!(g.cycle(-1, CycleMode::Weapons, &owned, yes, selectable, base), Some(1), "out of the family");
        // The cursor is the request, not the held id.
        let mut g = Gun::new(1, 1, 0);
        g.switch_request = 2;
        assert_eq!(g.cycle(1, CycleMode::Weapons, &owned, yes, selectable, base), Some(0x33), "on from the request");
        // Only a positive marker falls back to the base [M 0x1a6f80:188, a signed compare].
        let mut g = Gun::new(2, 7, 41);
        g.twin_skip[0x33] = -1;
        let only_base = |w: u8| w != 0x32;
        assert_eq!(g.cycle(1, CycleMode::Weapons, &owned, only_base, selectable, base), Some(0x45), "a negative marker walks past");
    }

    /// `Player_GetBestWeapon`: base + the family marker, the AIMS-20 always on its rifle [M 0x1a9680:18-23].
    #[test]
    fn best_weapon_reads_the_family_marker() {
        let mut owned = vec![false; 0x73];
        owned[0x02] = true;
        owned[0x1a] = true;
        let base = |w: u8| match w { 0x03 => 2, 0x1b => 0x1a, _ => w };
        let mut g = Gun::new(0x16, 0, 0);
        g.twin_skip[2] = 1;
        assert_eq!(g.get_best_weapon(&[0x02], base, &owned, |_| true), Some(3), "the silenced twin");
        assert_eq!(g.get_best_weapon(&[0x02], base, &owned, |w| w == 2), None, "the twin's own ammo decides");
        g.twin_skip[0x1a] = 1;
        assert_eq!(g.get_best_weapon(&[0x1a], base, &owned, |_| true), Some(0x1a), "the AIMS-20 ignores the marker");
    }

    /// `Player_HandleHasNoAmmo`'s exemptions and the `+0x68 & 0x10` twin offer [M 0x1a2ba8].
    #[test]
    fn no_ammo_request_exempts_and_offers_the_twin() {
        let owned = vec![true; 0x73];
        let id = |w: u8| w;
        for w in [1u8, 0x32, 0x33, 0x4a, 0x4e, 0x50, 0x51] {
            assert_eq!(Gun::new(w, 0, 0).no_ammo_request(0, 0, &[2], id, &owned, |_| true), None, "{w:#x} stays");
        }
        assert_eq!(Gun::new(0x34, 0, 0).no_ammo_request(0, 0, &[2], id, &owned, |_| true), Some(2), "the frag is not exempt");
        assert_eq!(Gun::new(0x1b, 0, 0).no_ammo_request(-1, 0x10, &[2], id, &owned, |_| true), Some(0x1a), "the twin first");
        assert_eq!(Gun::new(0x1b, 0, 0).no_ammo_request(-1, 0x10, &[2], id, &owned, |w| w == 2), Some(2), "a dry twin falls through");
        assert_eq!(Gun::new(0x16, 0, 0).no_ammo_request(1, 0, &[2], id, &owned, |_| true), Some(2), "no 0x10: no twin offer");
    }

    /// The trigger's rule on the release arm and the timer through a reload [M 0x1a5460].
    #[test]
    fn the_refire_timer_runs_through_a_reload_and_dry_is_flagged() {
        let p = GunParams { shots_per_pull: 1, interval: 30.0, clip: 9, ammo_per_shot: 1, reload_ticks: 40, reload_key_ticks: 40, one_at_a_time: false, reload_start_ticks: 0, reload_loop_ticks: 40, reload_end_ticks: 0, ammo_type: 3, fire_anim_len: 0, fire_script: 1, fire_last: 0 };
        let mut g = Gun::new(0xf, 5, 9);
        assert!(g.update_with(true, false, false, p));
        assert_eq!(g.timer, 30.0);
        assert!(!g.update_with(false, true, false, p), "the key takes the release");
        assert_eq!(g.reload_state, 6);
        assert_eq!(g.timer, 30.0, "branch A leaves the timer");
        for _ in 0..40 { g.update_with(false, false, false, p); g.reload_tick(false, p); }
        assert_eq!(g.reload_state, 0);
        assert_eq!(g.timer, 0.0, "it ran out during the reload");
        assert!(g.update_with(true, false, false, p), "the first press after the reload fires");
        // Dry: the flag rises on the release, with the view idle only.
        let mut g = Gun::new(0xf, 0, 0);
        g.update_with(true, false, false, p);
        assert!(!g.dry, "the held trigger holds it");
        g.update_with(false, false, true, p);
        assert!(!g.dry, "a busy view holds it");
        g.update_with(false, false, false, p);
        assert!(g.dry);
        // Sister: ammo type 0 always has a round.
        let m = GunParams { ammo_type: 0, ..p };
        let mut g = Gun::new(1, 0, 0);
        g.update_with(false, false, false, m);
        assert!(!g.dry);
    }

    /// Weapon 0x45's has-ammo is the game's literal comparison `weapon_data[0x45].clip_size (100) <=
    /// clip[0x45]` [M 0x1a16c0:14-15] — `+0x4F4` is `clip[0x45]`, the carried grenade count.
    #[test]
    fn grenade_belt_weapon_0x45_follows_the_carried_count() {
        let mut clip = vec![0i16; 0x73];
        let carried = [0i16; 34];
        let ammo_type = |_w: u8| 0usize;
        let flags = |_w: u8| 0u32;
        assert!(!Gun::has_ammo(0x45, &carried, &clip, ammo_type, flags, |w| w as usize), "count < 100: no ammo");
        clip[0x45] = 100;
        assert!(Gun::has_ammo(0x45, &carried, &clip, ammo_type, flags, |w| w as usize), "count ≥ 100 [M]");
        // Sister: other weapons ignore clip[0x45] entirely.
        clip[0x45] = 0;
        assert!(Gun::has_ammo(1, &carried, &clip, |_w| 0usize, flags, |w| w as usize), "ammo_type 0 = melee: always true");
    }
    /// `Player_Zoom` 0x1a1a70: the held axis ramps BOTH magnifications ×1.025/tick from where the
    /// current one stands (the sniper 0x24, max 10 [L weapon_data +0x88]: 1 → 10 in 94 ticks),
    /// the idle axis eases ×1.15/tick toward the target, and the max clamps exactly.
    #[test]
    fn zoom_ramps_geometrically_and_eases_back() {
        let (mut cur, mut target) = (1.0f32, 1.0f32);
        let mut n = 0;
        while cur < 10.0 {
            assert!(zoom_tick(&mut cur, &mut target, 10.0, -1.0).is_some());
            assert_eq!(cur, target, "the held axis keeps the target on the current zoom");
            n += 1;
        }
        assert_eq!((n, cur), (94, 10.0), "1.025^93 < 10 <= 1.025^94; the max clamps exactly");
        // Idle: the zoom stays (the target is untouched by an idle tick).
        assert_eq!(zoom_tick(&mut cur, &mut target, 10.0, 0.0), None);
        assert_eq!(cur, 10.0);
        // The aim falls: the target drops to 1.0 and the zoom eases out at 1/1.15 per tick.
        target = 1.0;
        let mut out = 0;
        while cur > 1.0 { zoom_tick(&mut cur, &mut target, 10.0, 0.0); out += 1; }
        assert_eq!(out, 17, "ln 10 / ln 1.15 = 16.5 → 17 ticks, the last clamped onto 1.0");
        // Sister: a positive axis zooms OUT, and never below 1.
        let (mut c, mut t) = (2.0f32, 2.0f32);
        zoom_tick(&mut c, &mut t, 10.0, 1.0);
        assert!(c < 2.0 && (c - 2.0 / 1.025).abs() < 1e-6);
        for _ in 0..100 { zoom_tick(&mut c, &mut t, 10.0, 1.0); }
        assert_eq!(c, 1.0);
        assert_eq!(initial_saved_zoom(16.0), 4.0);
    }
}
