//! MP character data (mp-character-data): the `MP_skins` per-character table and the third-person
//! body's animation-set descriptors, straight from ACTION.ELF's own data.
//!
//! - `MP_skins` @0x26d488 (ELF symbol, 464 = 29 × 0x10 bytes) — the character index → skin table
//!   `Player_Init` 0x1910f8 reads on the MP path (`MPSettings+slot*0x30+0x24` selects the record;
//!   disasm 0x1913c0-0x1913e0 loads +0x00 into `AnimObjectNew` and +0x08 into `BLData+0x965`)
//!   [M 0x1910f8 lines 108-117].
//! - The 16 animation-set descriptors @0x2b91b0..0x2b9430 (stride 0x28; `PunchSkin` starts right
//!   after at 0x2b9430 [M-sym]) — the static id tables `PlayerAnimSetInitNormal` 0x18fcd8 /
//!   `PlayerAnimSetInitCrouch` 0x18fea0 hand to `AnimSetAppend` per anim-set class.
//!
//! Everything here is numbers/ids only — the drawing is the app's.

use crate::buf::{invalid, Buf, Result};
use crate::ee::Machine;
use crate::weapon::WeaponData;

/// Records in `MP_skins` (`MP_NeedSkin` 0x120c48's `while (iVar2 < 0x1d)` loop bound) [M].
pub const MP_SKINS_COUNT: usize = 29;

/// One `MP_skins` record (0x10 bytes) [D ELF 0x26d488].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MpSkinEntry {
    /// +0x00: the character's skin (anim object) hash, 0x0500xxxx — `AnimObjectNew`'s argument on
    /// the MP path [M 0x1910f8 disasm 0x1913d4 `lw $s0, ($v0)` after `sll $v0, $v0, 4`].
    pub skin: u32,
    /// +0x04: the skin's DATA file hash, 0x0100xxxx (`MP_NeedFile` 0x120ca0's column, base
    /// 0x26d48c) [M]. Carried by every MP arena's map pack [D scan of all 99 containers].
    pub file: u32,
    /// +0x08: the sleeve variant (4..7 for MP) — copied to `BLData+0x965` [M 0x1910f8 disasm
    /// 0x1913e0 `sb $a2, 0x965($s1)`], which `Player_SetWeaponAnim` 0x1a9338 feeds to
    /// `AnimObjectSetSleeveType` (the first-person arms, not the body).
    pub sleeve: u32,
    /// +0x0c: the runtime "selected this match" flag — 0 on the disc [D]; `MP_setLoadingSkins`
    /// 0x1261e8 sets it to 1 for every selected character; `MP_NeedSkin`/`MP_NeedFile` return it.
    pub selected: u32,
}

/// `MP_skins` — the 29 MP characters, index = the MP character menu's list order (`Menu_GetMPSkins`
/// 0x1ffc98 walks the same 29 indexes; the menu's character index is the table index).
pub struct MpSkins {
    pub entries: Vec<MpSkinEntry>,
}

impl MpSkins {
    /// Read the table from ACTION.ELF (the ELF symbol carries the exact extent) [D].
    pub fn from_action_elf(data: &[u8]) -> Result<MpSkins> {
        let elf = crate::elf::Elf::parse(data)?;
        let syms = elf.symbols(data)?;
        let (va, size) = syms
            .iter()
            .find(|s| s.name == "MP_skins")
            .map(|s| (s.value, s.size as usize))
            .ok_or_else(|| crate::buf::FormatError::Invalid("ACTION.ELF has no symbol MP_skins".into()))?;
        if size != MP_SKINS_COUNT * 0x10 {
            return invalid(format!("MP_skins size {size} is not 29 × 0x10"));
        }
        let m = Machine::from_elf(data)?;
        let bytes = m.read(va, size)?;
        let entries = bytes
            .chunks_exact(0x10)
            .map(|r| {
                let b = Buf(r);
                MpSkinEntry { skin: b.u32(0).unwrap_or(0), file: b.u32(4).unwrap_or(0), sleeve: b.u32(8).unwrap_or(0), selected: b.u32(0xc).unwrap_or(0) }
            })
            .collect();
        Ok(MpSkins { entries })
    }

    /// `MP_setLoadingSkins` 0x1261e8: clear every record's +0x0c, then set 1 for each human slot's
    /// character (`MPSettings+slot*0x30+0x24`, `slot < MPSettings+0x1AC`) and each bot record's
    /// character (bot table: count byte @0x2deea9, 0x12-byte records @0x2deeba, char at +0x10)
    /// [M]. The port passes the same indexes its slots carry.
    pub fn set_loading_skins(&mut self, char_indexes: impl Iterator<Item = u32>) {
        for e in self.entries.iter_mut() { e.selected = 0; }
        for idx in char_indexes {
            if let Some(e) = self.entries.get_mut(idx as usize) { e.selected = 1; }
        }
    }

    /// `MP_NeedSkin` 0x120c48: in MP (the caller checks the arena) the skin-hash column's record
    /// flag; a hash on no record → 1 (the game loads unrecognised skins unfiltered).
    pub fn need_skin(&self, skin_hash: u32) -> bool {
        self.entries.iter().find(|e| e.skin == skin_hash).map_or(true, |e| e.selected != 0)
    }

    /// `MP_NeedFile` 0x120ca0: the same over the file-hash column.
    pub fn need_file(&self, file_hash: u32) -> bool {
        self.entries.iter().find(|e| e.file == file_hash).map_or(true, |e| e.selected != 0)
    }
}

// ─── The animation-set descriptors ──────────────────────────────────────────────────────────────

/// One animation-set descriptor (0x28 bytes @0x2b91b0 + i*0x28) [D]. Layout, from
/// `AnimSetInit` 0x1c6a48 + `AnimSetUpdate` 0x1c0938's reads:
/// `u32 loco[countA] , 0 , u32 strafe[4] , 0` — locomotion = `[idle, walk, run]` (the crouch
/// sets carry walk == run), strafe = `[left_a, left_b, right_a, right_b]` (the pairs duplicate;
/// `AnimSetUpdate` reads word[countA+1] for dir 1 and word[countA+3] for dir 2 [M 0x1c0938
/// disasm 0x1c0f2c/0x1c0f48]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MpAnimSet {
    pub idle: u32,
    pub walk: u32,
    pub run: u32,
    /// dir 1 (the strafe word `AnimSetUpdate` appends for positive strafe).
    pub strafe_left: u32,
    /// dir 2 (negative strafe).
    pub strafe_right: u32,
}

/// The 16 descriptors (8 normal + 8 crouch), each 0x28 bytes @0x2b91b0 + i*0x28 [D].
pub const ANIM_SET_BASE: u32 = 0x2b91b0;
pub const ANIM_SET_STRIDE: u32 = 0x28;
pub const ANIM_SET_COUNT: usize = 16;

/// `PlayerAnimSetInitNormal` 0x18fcd8's selector → descriptor mapping (the same switch in
/// `PlayerAnimSetInitCrouch` 0x18fea0 at +0x28) [M]. The selector: SP (`MPSettings._384_4_` == 0)
/// → 999 [M 0x18fd08-0x18fd18: `movz $v1, $t1, $a0` with $t1 = 999]; MP →
/// `weapon_data[weapon]*0x10C + 0x84` (the weapon's anim-set class byte) [M 0x18fd04].
/// `default` covers class 0 and anything > 6 [M the switch's fall-through @0x2b9250].
/// The crouch descriptor is always the normal one + 0x28 [M 0x18fea0's switch values].
pub fn anim_set_index(class: u32, crouch: bool) -> usize {
    let normal = match class {
        999 => 0,   // 0x2b91b0 [M 0x18fcd8]
        1 => 6,     // 0x2b92a0
        2 => 14,    // 0x2b93e0
        3 => 2,     // 0x2b9200
        4 => 10,    // 0x2b9340
        5 => 8,     // 0x2b92f0
        6 => 12,    // 0x2b9390
        _ => 4,     // 0x2b9250 (default)
    };
    normal + crouch as usize
}

/// The anim-set class byte for a weapon: `weapon_data[w*0x10C + 0x84]` [M 0x18fd04 disasm
/// `lb $v0, 0x62($a0)` × `*0x10c + 0x84`]. SP forces the 999 selector before this is consulted
/// (`movz`, [M 0x18fd18]).
pub fn weapon_anim_class(weapon_data: &WeaponData, weapon: u8) -> u32 {
    weapon_data.records.get(weapon as usize).map_or(0, |r| r[0x84] as u32)
}

/// The stand→crouch transition script, per class (`PlayerAnimStand2Crouch` 0x190198's switch;
/// type 3 when `MPSettings._384_4_` == 0 [M 0x1901c0/0x1901d0 `movn`]) [M].
pub fn stand2crouch_script(class: u32) -> u32 {
    match class {
        1 => 0x6000_9dd,
        2 => 0x6000_9b1,
        3 => 0x6000_1dc,
        4 => 0x6000_1d0,
        5 => 0x6000_1e8,
        6 => 0x6000_9b9,
        _ => 0x6000_1f3,
    }
}

/// The crouch→stand transition script, per class (`PlayerAnimCrouch2Stand` 0x190068's switch)
/// [M]. The game plays it at ±4× speed (script+0x98 = 0x40800000/0xc0800000) with root-motion
/// flags |= 0x0d000000 [M 0x1900b8-0x1900d0].
pub fn crouch2stand_script(class: u32) -> u32 {
    match class {
        1 => 0x6000_9da,
        2 => 0x6000_9ae,
        3 => 0x6000_1dd,
        4 => 0x6000_1d1,
        5 => 0x6000_1e9,
        6 => 0x6000_9b6,
        _ => 0x6000_1f4,
    }
}

/// The death body scripts (`Player_ChangeSubState` 0x190630 cases 0xd/0xe): the gravity-Y
/// global `DAT_002d8894` == 0.0 picks 0x60008e0, anything else 0x6000151 [M]. `Player_Init`
/// initialises it to −9.8 (0xc11ccccd, `WldGravity.y`) [M 0x1910f8:152-154], so the normal death
/// body is 0x6000151; the 0x60008e0 variant is the zero-gravity death [M — the global is the gravity
/// vector's Y; the old "water-level / −9.9" reading was wrong, daily audit 2026-10-19].
pub const DEATH_SCRIPT: u32 = 0x6000_151;
pub const DEATH_SCRIPT_WATER_ZERO: u32 = 0x6000_8e0;

/// The MP spawn body script (`Player_Init` 0x1910f8 MP branch: `AnimScriptAdd(obj, 0x60001d4)`
/// — class 3's idle) [M 0x1910f8 lines 155-158].
pub const MP_SPAWN_SCRIPT: u32 = 0x6000_1d4;

/// The animation-set tables read from ACTION.ELF.
pub struct MpAnimSets {
    /// The 16 raw descriptors in ELF order [D].
    pub sets: [MpAnimSet; ANIM_SET_COUNT],
}

impl MpAnimSets {
    /// Read the descriptors from ACTION.ELF @0x2b91b0 (no ELF symbol — the vaddr is
    /// `PlayerAnimSetInitNormal`'s switch targets) [D].
    pub fn from_action_elf(data: &[u8]) -> Result<MpAnimSets> {
        let m = Machine::from_elf(data)?;
        let bytes = m.read(ANIM_SET_BASE, (ANIM_SET_COUNT as u32 * ANIM_SET_STRIDE) as usize)?;
        let mut sets = [MpAnimSet { idle: 0, walk: 0, run: 0, strafe_left: 0, strafe_right: 0 }; ANIM_SET_COUNT];
        for (i, set) in sets.iter_mut().enumerate() {
            let r = &bytes[i * ANIM_SET_STRIDE as usize..][..ANIM_SET_STRIDE as usize];
            let words: Vec<u32> = r.chunks_exact(4).map(|w| u32::from_le_bytes(w.try_into().unwrap())).collect();
            // countA = the words until the 0 separator [M AnimSetInit 0x1c6a48's group count].
            let count_a = words.iter().position(|&w| w == 0).unwrap_or(3);
            *set = MpAnimSet {
                idle: words[0],
                walk: words.get(1).copied().unwrap_or(words[0]),
                run: words.get(2).copied().unwrap_or(words[0]),
                strafe_left: words.get(count_a + 1).copied().unwrap_or(0),
                strafe_right: words.get(count_a + 3).copied().unwrap_or(0),
            };
        }
        Ok(MpAnimSets { sets })
    }

    /// The set a body runs: `anim_set_index(class, crouch)` [M 0x18fcd8/0x18fea0; the crouch pick
    /// is `PlayerAnimSetInit` 0x195b90's `obj+0xf6 == 4`].
    pub fn set(&self, class: u32, crouch: bool) -> MpAnimSet {
        self.sets[anim_set_index(class, crouch)]
    }
}

// ─── The body pose (pure) ───────────────────────────────────────────────────────────────────────

/// The per-tick state one MP body is posed from. Every field carries its game offset; the caller
/// (the draw) fills it from the slot's live state each frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpBodyState {
    /// `MPSettings._384_4_` (+0x180): MP on [M 0x18fd08]. An MP body is always true; false
    /// reproduces the SP set (999) for tests.
    pub mp: bool,
    /// `ANIMOBJ+0x62`: the current weapon id (the `weapon_data` row) [M 0x18fd04].
    pub weapon: u8,
    /// `obj+0xf6`: the substate — 0 stand/walk, 4 crouch, 0xd ground death, 0xe drown
    /// (`Player_ChangeSubState` 0x190630; the dispatch `Player_Update` 0x193600) [M].
    pub substate: u8,
    /// `BLData+0x18`: the signed forward speed in units/tick [M `Player_Move` 0x191dd0:164-169;
    /// `Player_SSWalk` passes it as `AnimSetUpdate`'s param_2 [M 0x192388-0x1923d8 disasm]].
    pub fwd: f32,
    /// `BLData+0x10`: the strafe speed in units/tick [M 0x191dd0:164-169].
    pub strafe: f32,
    /// `ANIMOBJ+0xdc`: the accumulated walk-cycle distance (init 0.37 at set init, advanced by
    /// `fwd × set+0x28 (0.5)` per tick) [M 0x1c6a48 disasm 0x1c6b58, 0x1c0938:96].
    pub phase: f32,
    /// `BLData+0x960`: the crouch-transition hold counter — `Player_ChangeSubState` sets it to
    /// `(u8)FRAME_RATE` (60) when it fires a transition [M 0x190630 case 0/4]; nonzero holds the
    /// transition pose (`Player_SSWalk`/`SSCrouch` skip `AnimSetUpdate` while set [M 0x192310:34-44]).
    pub hold: u32,
    /// The body's one-shot clock in script frames (the death/transition script's playback time —
    /// the draw advances it; the game's scripts start at time 1 and advance × speed per tick).
    pub one_shot_time: f32,
    /// The gravity vector's Y, `WldGravity.y` = `DAT_002d8894` (0xc11ccccd = −9.8 at `Player_Init`,
    /// after `WldGravity = 0` [M 0x1910f8:152-154]) — the death script pick compares it against
    /// 0.0: zero-g picks 0x60008e0 [M 0x190630 case 0xd/0xe]. (Was misnamed `water_level` / −9.9;
    /// daily audit 2026-10-19.)
    pub gravity_y: f32,
}

/// One body pose: the anim SCRIPT id the game would play (0x06xxxxxx — the ids the descriptors
/// carry; the draw resolves it to kind-4 sequences through the level's script table), the time
/// in that script's own domain, and the blend.
///
/// Time domain: locomotion/idle scripts are distance-driven (`AnimSet_WalkUpdater` 0x1c6a08 /
/// the mode-1 script clock) — `time` is `MpBodyState::phase`, the accumulated walk distance the
/// draw maps through the sequence's distance table (`AnimDistanceTableDistanceToFrame2`
/// 0x1c4270). One-shots (death, the crouch transitions) advance their own script clock — `time`
/// is `MpBodyState::one_shot_time`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpBodyPose {
    pub script: u32,
    pub time: f32,
    /// The run crossfade weight the game writes to script B's +0x88 [M 0x1c0938:112-120]:
    /// 0.0 on idle/one-shots; on locomotion the fraction t (the run companion `set.run` shows at
    /// this weight while the returned walk script plays).
    pub blend: f32,
}

/// The fixed-tick speed constants, NTSC: `FRAME_RATE_MUL` @0x30d0d8 = 1.0 [L ee_start_05.bin;
/// the port's 60 Hz determinism rule pins it] — the maxima `Player_SSWalk` hands to
/// `AnimSetUpdate` are `0.1·FRM` forward and `0.075·FRM` strafe [M 0x192388-0x1923d8 disasm;
/// the same numbers `Player_Move`'s input curve clamps to, 0x191dd0].
pub const FWD_MAX: f32 = 0.1;
pub const STRAFE_MAX: f32 = 0.075;

/// The pure body pose for one MP body ([M] throughout — `Player_ChangeSubState` 0x190630 +
/// `AnimSetUpdate` 0x1c0938):
///
/// 1. substate 0xd/0xe (death): the gravity-Y pick, no set ([M 0x190630 cases 0xd/0xe —
///    `AnimSetDeleteAll` then the one script]).
/// 2. `hold != 0`: the crouch↔stand transition for the current stance — substate 4 (crouched
///    now) plays stand2crouch, substate 0 plays crouch2stand ([M 0x190630 case 0/4: the set is
///    rebuilt FIRST, then the transition fires when the previous substate was the other stance;
///    `Player_SSWalk`/`SSCrouch` hold it while +0x960 counts down]).
/// 3. otherwise the stance's set (`PlayerAnimSetInit` 0x195b90: obj+0xf6 == 4 → the crouch
///    descriptor) and `AnimSetUpdate`'s pick: t = clamp(|fwd|, 0, 0.1·FRM)/(0.1·FRM); t ≤ 0.0002
///    → idle at a frozen phase; else the walk/run pair crossfaded at t [M 0x1c0938:99-120].
///
/// No jump/fire/pain slots exist in the set — those states keep the locomotion scripts (jump
/// freezes the phase as the speed → 0; firing plays on the invisible weapon obj,
/// `Player_SetWeaponAnimObj` 0x1a2e38) [M — the descriptors have no such slots and
/// `Player_HandlePain` 0x1902c8 adds no body script].
pub fn mp_body_pose(sets: &MpAnimSets, class: u32, st: &MpBodyState) -> MpBodyPose {
    // 1. Death: the substate-0xd/0xe scripts [M 0x190630].
    if st.substate == 0xd || st.substate == 0xe {
        let script = if st.gravity_y == 0.0 { DEATH_SCRIPT_WATER_ZERO } else { DEATH_SCRIPT };
        return MpBodyPose { script, time: st.one_shot_time, blend: 0.0 };
    }
    // 2. The crouch↔stand transition hold [M 0x190630 case 0/4 + 0x192310/0x1924c0's skip].
    if st.hold != 0 {
        let script = if st.substate == 4 { stand2crouch_script(class) } else { crouch2stand_script(class) };
        return MpBodyPose { script, time: st.one_shot_time, blend: 0.0 };
    }
    // 3. The stance set [M 0x18fcd8/0x18fea0/0x195b90] and AnimSetUpdate's pick [M 0x1c0938].
    let set = sets.set(class, st.substate == 4);
    let t = (st.fwd.abs().min(FWD_MAX) / FWD_MAX).clamp(0.0, 1.0);
    if t <= 0.0002 {
        MpBodyPose { script: set.idle, time: st.phase, blend: 0.0 }
    } else {
        MpBodyPose { script: set.walk, time: st.phase, blend: t }
    }
}

/// The MP strafe overlay ([M 0x1c0938 LAB_001c0dc8], MP only — the SP path returns before it):
/// `ratio = strafe/0.075·FRM`; `angle = atan2f(ratio, |fwd|/0.1·FRM)`; a strafe script plays when
/// |strafe| > 0.0002 and |angle·(2/π)| ≥ 0.5 — dir 1 (positive strafe) appends word[countA+1],
/// dir 2 word[countA+3]; the overlay's playback speed (+0x98) is |ratio| and its weight (+0xa8)
/// is |angle·(2/π)|. `None` = no overlay (the game deletes the previous one).
pub fn mp_body_strafe(sets: &MpAnimSets, class: u32, st: &MpBodyState) -> Option<(u32, f32, f32)> {
    if !st.mp || st.hold != 0 || st.substate == 0xd || st.substate == 0xe { return None; }
    let ratio = st.strafe / STRAFE_MAX;
    let angle = ratio.atan2(st.fwd.abs() / FWD_MAX);
    let weight = (angle * std::f32::consts::FRAC_2_PI).abs();
    if !(st.strafe.abs() > 0.0002 && weight >= 0.5) { return None; }
    let set = sets.set(class, st.substate == 4);
    let script = if st.strafe > 0.0 { set.strafe_left } else { set.strafe_right };
    Some((script, ratio.abs(), weight))
}

/// Every distinct script id the descriptors + transitions + death + spawn can put on a body —
/// the id set an MP body's sequence set resolves ([D] the table bytes; [M] the transitions').
impl MpAnimSets {
    pub fn all_body_scripts(&self) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        let mut push = |id: u32| { if id != 0 && !out.contains(&id) { out.push(id); } };
        for set in self.sets.iter() {
            push(set.idle);
            push(set.walk);
            push(set.run);
            push(set.strafe_left);
            push(set.strafe_right);
        }
        // The transitions (both classes' ranges + the defaults) and the death/spawn scripts.
        for class in [1u32, 2, 3, 4, 5, 6, 0, 999] {
            push(stand2crouch_script(class));
            push(crouch2stand_script(class));
        }
        push(DEATH_SCRIPT);
        push(DEATH_SCRIPT_WATER_ZERO);
        push(MP_SPAWN_SCRIPT);
        out
    }
}
