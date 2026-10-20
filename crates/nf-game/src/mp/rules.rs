//! The per-scenario MP rules (402 mp-match-rules) — every rule here carries its game address in
//! `docs/systems/mp-match-rules.md`; this file implements, in order: kill scoring
//! (`MP_PlayerKilled` 0x188f70), the assassin reset (0x18aa70), the friendly-fire gate
//! (`MP_RegisterBulletHit` 0x1857c0), the respawn rules (`MP_ReSpawn` 0x185368), the who-won
//! decision (`MP_SortOutWhoWon` 0x1848d0), the round restart (`MP_RestartScenario` 0x183fc0), the
//! mode-object creation picks (`MP_Start` 0x183a50:77-114) and the per-frame scenario updates
//! (`MP_FlagUpdate` 0x1861a8, `MP_UplinkUpdate` 0x186d98, `MP_KOHUpdate` 0x186ae0,
//! `MP_DemolitionProtectionUpdate` 0x1870a0, `MP_GoldenEyeUpdate` 0x187568,
//! `MP_BluePrintUpdate` 0x1882d8 / `MP_BluePrintReachedBase` 0x188970).
//!
//! Timers are **frames** at the fixed 60 Hz (`FRAME_RATE_INT`); the drip scores use the game's
//! `REC_FRAME_RATE * 0.2` = 0.2 points/s. Touch events (`touch_*`) stand in for the game's
//! `MP_HitBy` 0x1830e0 overlap walk — the rules (team filters, state gates) are here, the spatial
//! test is the caller's.

use super::{scenario, MpEndStatus, MpMatch};
use crate::drone::Rng;

/// 60 Hz — the fixed tick (`FRAME_RATE_INT`; the game's timers count frames [M 0x1861a8:226-229]).
pub const FRAME_RATE_INT: u32 = 60;
/// `FRAME_RATE_INT * 0x1e` — the 30 s dropped-object return [M 0x1861a8:226-229].
pub const RETURN_FRAMES: u32 = FRAME_RATE_INT * 30;
/// The capture/base reach: sq-dist < 2.0 [M 0x1861a8:74-86, disasm `c.olt.s` 2.0].
pub const CAPTURE_SQ_DIST: f32 = 2.0;
/// mp-scenarios-online: the player's world-sphere radius (`M_CONTROL+0x8C`, 1.0576 [L ee_start_05
/// — `Bond::sphere_centre`'s doc]) — the body half of the port's `MP_HitBy` contact stand-in.
pub const PLAYER_SPHERE_RADIUS: f32 = 1.0576;
/// The player's world-sphere centre height relative to `Bond::pos` (`M_CONTROL+0x80` y = −0.1267
/// [L ee_start_05]; the small horizontal offset is yaw-dependent and the match keeps no yaw — [S]).
pub const PLAYER_SPHERE_DY: f32 = -0.1267;
/// The GoldenEye strike's flight length on the arenas whose script carries it — 07000023's
/// `0600003e` entry is 299 ticks [D the archive entry's length field; the live load reads the
/// arena's own copy through `level.nis`, so this constant is only the headless rigs' value].
pub const GE_STRIKE_FRAMES: u16 = 299;

/// mp-scenarios-online: an MP object's model bounds — the placement's celglist record (`obj+0xD8`):
/// +0x10 the local sphere centre, +0x1C its radius (`Control_BuildWorldSph` 0x133070 → obj+0x80 /
/// +0x8C [M]), +0x20/+0x30 the local box min/max (`MP_KOHUpdate` 0x186ae0:88-93 and
/// `MP_isPosOnHill` 0x18bc20 test the hill against it) — `nf_formats::level::MapObject`'s
/// `centre`/`radius`/`min`/`max` [D]. Records registered without bounds (the headless rigs) have no
/// contact geometry: only the explicit `touch_*` events move their objects.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MpObjBounds {
    pub centre: [f32; 3],
    pub radius: f32,
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// mp-hitby-contacts: the port's derived field — the local sphere centre through the
    /// placement's rotation (`cel+0x10 · M(obj+0x90)`, the translation row excluded):
    /// `Control_BuildWorldSph` 0x133070:25-36 through `RotMatrix` 0x1e2368 [M]. Computed once in
    /// `register_mp_object_bounds`; `[0,0,0]` for an unrotated placement or a bare registration.
    /// The contact sphere's world centre is `pos + sphere_off` [M 0x133070].
    pub sphere_off: [f32; 3],
}

/// One MP object's live state — the port's stand-in for `obj_tag+0xF4/+0xF6` and the
/// `MPOBJECT+0x02/+0x04/+0x08/+0x50/+0x52` fields [M 0x185de8:129-131, 0x1861a8, 0x1870a0:81-91].
#[derive(Clone, Debug, Default)]
pub struct MpObjState {
    /// `obj+0xF4` (i16): the scenario state word — per-type meaning (flag 0/1/2, uplink 2/0/1,
    /// site HP 2000.., GE item 0/1/2/3, blueprint 0/1/2).
    pub state: i16,
    /// `obj+0xF6` (i16): the sub-index / done flag (GE item 0/1, site done = 1).
    pub sub: i16,
    /// `MPOBJECT+0x04` (u16): the return counter (frames, counts up past `RETURN_FRAMES`).
    pub counter: u16,
    /// `MPOBJECT+0x08` / `obj+0x1C`: the carrying player's slot.
    pub carrier: Option<usize>,
    /// `MPOBJECT+0x50` (i16): the last hitter slot (the demolition site).
    pub last_hitter: i16,
    /// `MPOBJECT+0x52` (i16): the controlling player slot (uplink), −1 = none.
    pub controller: i16,
    /// `MPOBJECT+0x02` (u16): the team word (the flag's owner; the GE/blueprint carrying team).
    pub team: u16,
    /// `obj+0xC0`: the object's current world position (the drop spot when dropped).
    pub pos: [f32; 3],
    /// The home/spawn position (`MPOBJECT+0x10`'s matrix translation) — where the object returns
    /// to [M 0x1861a8:99-112].
    pub home: [f32; 3],
    /// mp-scenarios-online: the model bounds (the contact sphere / the hill box); `None` = no
    /// contact geometry (the explicit `touch_*` events only).
    pub bounds: Option<MpObjBounds>,
}

/// The scenario objects' live state — the port's `Flags`[2] @0x317210, `Bases`[2] @0x317330,
/// `Uplinks`[8] @0x317450, `Hill` @0x318CE0, the demolition/protection site, `GoldenEye`[2]
/// @0x317FF0/0x318080, `BluePrint` @0x318830 and `EsponageBase`[2] [M 0x185de8:133-178].
/// The getter family over these records: `MP_getFlagObj` 0x18b978 / `MP_getBaseObj` 0x18b9a0 /
/// `MP_getUplinkObj` 0x18b9c8 / `MP_getHillObj` 0x18bb50 / `MP_getDemolitionObj` 0x18bab8 /
/// `MP_getProtectionObj` 0x18bac8 / `MP_getGoldenEyeObj` 0x18baf0 (gates on state ∈ {0, 2}) /
/// `MP_getBlueprintObj` 0x18bad8 / `MP_getEsponageBaseObj` 0x18a338 / `MP_getObjExtFromMPOBJECT`
/// 0x18a210. `MP_recalcObjExtPaths` 0x18b888's position-refresh half is the carrier updates
/// below; its AI-emitter re-walk is not ported.
#[derive(Clone, Debug, Default)]
pub struct MpObjectsRt {
    /// The two CTF flags (index = owning team).
    pub flags: [Option<MpObjState>; 2],
    /// The two CTF bases' capture points (the base obj's +0xC0 position).
    pub bases: [Option<[f32; 3]>; 2],
    /// The uplink terminals (≤ 8 [M 0x185990:55-63]).
    pub uplinks: Vec<MpObjState>,
    /// The KOTH hill.
    pub hill: Option<MpObjState>,
    /// The one demolition/protection site.
    pub site: Option<MpObjState>,
    /// The two GoldenEye items (sub 0/1).
    pub ge: [Option<MpObjState>; 2],
    /// The GoldenEye strike's target slot (global from `MP_GetTarget` at launch [M 0x187568:362-367]).
    pub ge_target: Option<usize>,
    /// The blueprint item.
    pub blueprint: Option<MpObjState>,
    /// The two espionage bases (the blueprint delivery points).
    pub esp_bases: [Option<[f32; 3]>; 2],
    /// mp-scenarios-online: the CTF bases' / espionage bases' model bounds (the contact sphere the
    /// blueprint's `EsponageBase` HITDATA walk stands in for [M 0x1882d8:68-77]).
    pub bases_bounds: [Option<MpObjBounds>; 2],
    pub esp_bases_bounds: [Option<MpObjBounds>; 2],
    /// mp-scenarios-online: the frames the GoldenEye strike has been in flight (state 3) — the
    /// strike completes at [`MpObjectsRt::ge_flight_ticks`] (the SP's state 2 [M 0x187568:96]).
    pub ge_strike_timer: u16,
    /// mp-goldeneye-strike: the strike's flight length in ticks — the SP script 0x0600003e's own
    /// length (299 on 07000023 [D the archive entry]; `SP_Update` 0x1dbda8 sends the state to 5
    /// when `Script_IsPlaying` 0x1db630 reads the play byte done, and `SP_GetState` 0x1dc900
    /// reports 2 — the game's landed test). The live load fills it from the arena's `level.nis`;
    /// 0 = the arena carries no strike script: the ScriptPlayer never plays and the strike never
    /// lands (the game's own failed-lookup soft-lock [M Script_Load 0x1d8248]).
    pub ge_flight_ticks: u16,
    /// mp-goldeneye-strike: the tracked beam position — Some(the item's pos) at the launch
    /// (`SP_Create`'s pos [M 0x187568:383-384]), Some((target.x, target.y − 1.0, target.z)) on
    /// every in-flight tick (`MP_GoldenEyeUpdate`'s `SP_SetPos` [M :87-95]), None at the impact
    /// (the strike global clears [M :121-122]). Hashed with [`MpObjectsRt::ge_target`].
    pub ge_strike_pos: Option<[f32; 3]>,
}

impl MpObjectsRt {
    fn obj_at(records: &[super::MpObjectRecord], i: usize) -> [f32; 3] {
        [records[i].matrix[3][0], records[i].matrix[3][1], records[i].matrix[3][2]]
    }

    fn bounds_at(records: &[super::MpObjectRecord], i: usize) -> Option<MpObjBounds> {
        records[i].bounds
    }

    fn config_team(records: &[super::MpObjectRecord], i: usize) -> u16 {
        (records[i].config[1] & 0xffff) as u16
    }
}

/// The who-won decision of `MP_SortOutWhoWon` 0x1848d0 (numbers only — the game's texts are the
/// renderer's). `Winner::None` for team games: the function never compares the team scores there
/// [M 0x1848d0:87-92]; the scoreboard judges.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Winner {
    /// Nobody qualified / the generic team text.
    None,
    /// The unique winner's slot.
    Slot(usize),
    /// The draw text (a tie at the minimum, or 0/≥2 qualifiers).
    Draw,
}

fn sq_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// mp-goldeneye-strike: the strike's draw state (`MpMatch::ge_strike_view`) — the phase IS the
/// Option: None = no strike in flight, Some = the script 0x0600003e is playing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeStrikeView {
    /// The target slot (`MP_GetTarget`'s random opposing pick [M 0x187568:362-367]).
    pub target: usize,
    /// The tracked beam position — the target's `(x, y − 1.0, z)` from the first update tick,
    /// the item obj's `+0x30` on the launch tick itself [M 0x187568:87-95, :383-384].
    pub pos: [f32; 3],
    /// The ticks left of the flight (the script's length minus the in-flight count [D the
    /// length field; `SP_GetState` 0x1dc900 reports the impact at 0]).
    pub ticks_left: u16,
}

impl MpMatch {
    // ---------------------------------------------------------------- kill scoring

    /// `MP_PlayerKilled` 0x188f70 — the full scoring path. `killer` is the resolved credit slot
    /// (the game walks `HitByList`; the caller resolves), `explosive` = the killing hurt type was
    /// 4..7 [M 0x188f70:193-198] (preloads the carried objects' return counters), `golden_gun` =
    /// the bot golden-gun special (+2.0 [M :170-178]).
    pub fn player_killed(&mut self, victim: usize, killer: Option<usize>, explosive: bool, golden_gun: bool, rng: &mut Rng) {
        if self.settings.master == 0 {
            return; // [M 0x188f70:84-86]
        }
        self.game.slots[victim].deaths += 1; // every scenario [M :92-94]
        if self.settings.scenario == scenario::DEATH_LIMIT_FFA {
            self.game.slots[victim].score -= 1.0; // the lives copy [M :96-98]
        }
        self.game.slots[victim].dead = true; // the port's obj+0xF4 ∈ {2,3} stand-in
        // The kill message to the last shooter when that is not the victim [M :100-107]. The
        // port's credited killer stands in for the `+0x20` last-shooter word: the port registers
        // only some hits through `register_bullet_hit` (bot-vs-bot rounds do not), so the word
        // can be stale where the game's never is [S].
        if let Some(k) = killer.filter(|&k| k != victim && k < super::MP_MAX_SLOTS) {
            if self.kill_msgs.len() >= 64 { self.kill_msgs.remove(0); }
            self.kill_msgs.push((k, victim));
        }
        let mut delta = 0.0f32;
        if let Some(k) = killer {
            if k == victim {
                let s = &mut self.game.slots[victim];
                s.streak = 0;
                s.kills -= 1;
                s.last_killer = 0xffff; // [M :151-155]
            } else {
                let same_team = self.settings.players[victim].team == self.settings.players[k].team;
                if self.settings.team_game == 0 || !same_team {
                    self.game.slots[k].kills += 1; // [M :157-163]
                    self.game.slots[k].streak += 1;
                }
                self.game.slots[victim].last_killer = k as u16; // [M :164]
            }
            delta = if k == victim {
                -1.0 // self-kill [M :167-169]
            } else if golden_gun {
                2.0 // [M :170-178]
            } else {
                1.0
            };
            // the personal gate: bit30 == 0 AND one of the four scenarios [M :179-184]
            let s = self.settings.scenario;
            if self.settings.bit30 == 0
                && (s == scenario::DEATHMATCH
                    || s == scenario::TEAM_DEATHMATCH
                    || s == scenario::KOTH_FFA
                    || s == scenario::KOTH_TEAM)
            {
                self.game.slots[k].score += delta;
            }
            // the team effects — scenario 0x20000002 only [M :246-274]
            if self.settings.scenario == scenario::TEAM_DEATHMATCH {
                let vt = self.settings.players[victim].team as usize;
                if self.settings.players[k].team == self.settings.players[victim].team {
                    // same team (incl. self): the victim's team pays |Δ| [M :248-252]
                    self.game.team_scores[vt] -= delta.abs();
                } else {
                    // enemy kill: the opposing team gains |Δ| [M :261-273]
                    self.game.team_scores[self.settings.players[k].team as usize] += delta.abs();
                }
            }
        } else {
            // no player credit (uVar16 == 0): the victim's team pays |Δ| = 0 [M :254-259,273]
            if self.settings.scenario == scenario::TEAM_DEATHMATCH {
                self.game.team_scores[self.settings.players[victim].team as usize] -= 0.0;
            }
        }
        // the assassin block — scenario 0x400 [M :223-245]
        if self.settings.scenario == scenario::ASSASSIN {
            let killer_slot = killer.filter(|k| *k != victim);
            if let Some(t) = self.assassin_target {
                if victim == t && killer_slot == self.assassin {
                    self.game.slots[killer_slot.unwrap()].score += 5.0; // [M :229-233]
                }
            }
            if self.assassin == Some(victim) {
                if let (Some(t), Some(k)) = (self.assassin_target, killer_slot) {
                    if k == t {
                        self.game.slots[k].score += 3.0; // [M :234-242]
                    }
                }
                self.assassin_reset(rng); // [M :243]
            }
        }
        // the carried objects drop [M :276-303]
        self.drop_carried(victim, explosive, rng);
    }

    /// The carried-object drop of `MP_PlayerKilled` 0x188f70:276-303 — flag (type 0), blueprint
    /// (type 5), GE items (types 6/7) drop on their carrier's death (the object lands at the
    /// death spot — the game's floor-snapped `obj+0xC0` [M 0x1861a8:129-136]); the return counter
    /// preloads to `RETURN_FRAMES` when the kill was explosive.
    fn drop_carried(&mut self, victim: usize, explosive: bool, rng: &mut Rng) {
        let drop_pos = self.game.slots[victim].pos;
        for f in self.objects_rt.flags.iter_mut().flatten() {
            if f.carrier == Some(victim) {
                f.state = 2;
                f.carrier = None;
                f.pos = drop_pos.unwrap_or(f.pos);
                if explosive {
                    f.counter = RETURN_FRAMES as u16; // [M 0x188f70:276-299]
                }
            }
        }
        // mp-scenarios-online: the GoldenEye item's died path is NOT a drop where the carrier
        // fell — `MP_GoldenEyeUpdate(.., died=1)` floor-snaps the carry spot, then re-picks a
        // random pool spawn (`Rand_Rand(GoldenEyeSpawnCounts[type])` + the home/obj matrix copy)
        // and sets state 2 [M 0x187568:262-306]. The port dropped it at the death spot (no draw).
        for i in 0..2 {
            let carried = self.objects_rt.ge[i].as_ref().is_some_and(|it| it.carrier == Some(victim));
            if !carried {
                continue;
            }
            let n = self.objects.goldeneye[i].len();
            let pos = if n > 0 {
                let pick = rng.rand(n as u32) as usize; // [M 0x187568:281-283]
                Some(MpObjectsRt::obj_at(&self.objects.goldeneye[i], pick))
            } else {
                drop_pos
            };
            let it = self.objects_rt.ge[i].as_mut().unwrap();
            it.state = 2; // [M :304]
            it.carrier = None;
            if let Some(p) = pos {
                it.pos = p;
                it.home = p;
            }
            if explosive {
                it.counter = RETURN_FRAMES as u16;
            }
        }
        if let Some(bp) = &mut self.objects_rt.blueprint {
            if bp.carrier == Some(victim) {
                bp.state = 2;
                bp.carrier = None;
                bp.pos = drop_pos.unwrap_or(bp.pos);
                if explosive {
                    bp.counter = RETURN_FRAMES as u16;
                }
            }
        }
        self.game.slots[victim].status_bits &= !1; // the carrying bit clears [M 0x18a518]
    }

    /// `MP_assassinReset` 0x18aa70 — the new assassin is the dead assassin's last-killer when
    /// there is one (the kill just wrote it), else a random pick; the new target is a random pick
    /// among the other present slots (`MP_GetTarget` 0x1832b8 + `Rand_Rand`); the assassin's
    /// settings record team word = 1, everyone else's 0 [M 0x18aa70:12-41].
    fn assassin_reset(&mut self, rng: &mut Rng) {
        if self.assassin.is_none() {
            return;
        }
        self.assassin_reset_impl(true, rng);
    }

    /// mp-scenarios-online: `MP_Start` 0x183a50:166-167 — scenario 0x400 runs
    /// `MP_assassinReset(0)` once every player and bot exists (the port's caller: the load, right
    /// after `create_bots`). Without it the port never had an assassin or a target, so
    /// Assassination scored nothing at all.
    pub fn start_assassin(&mut self, rng: &mut Rng) {
        if self.settings.master == 0 || self.settings.scenario != scenario::ASSASSIN {
            return;
        }
        self.assassin_reset_impl(false, rng);
    }

    /// `MP_GetTarget(team, exclude, 0)` 0x1832b8 with the check flag off: every present slot
    /// (`MPGame+slot·0x30+0x1c != 0`) whose team word matches (2 = any) except `exclude`, then
    /// `Rand_Rand(n)` — drawn whenever n > 0 — picks one [M 0x1832b8:24-66].
    fn get_target(&self, team: u32, exclude: Option<usize>, rng: &mut Rng) -> Option<usize> {
        let cands: Vec<usize> = (0..self.game.slots.len())
            .filter(|&i| {
                self.game.slots[i].pos.is_some()
                    && (team == 2 || self.settings.players[i].team == team)
                    && Some(i) != exclude
            })
            .collect();
        if cands.is_empty() {
            return None;
        }
        Some(cands[rng.rand(cands.len() as u32) as usize])
    }

    /// The body of `MP_assassinReset` 0x18aa70 [M :12-41], in the game's order: every team word
    /// zeroes; `param_1 != 0` and the old assassin's last-killer word (+0x28, as a signed short)
    /// `>= 0` → that killer becomes the assassin; the target = `MP_GetTarget(0, new assassin else
    /// the OLD TARGET, 0)`; with no assassin yet, the assassin = `MP_GetTarget(0, target, 0)`; the
    /// assassin's team word = 1. (The port's 402 version picked the assassin first and the target
    /// second, excluding the wrong slot — a different draw order from the game's.)
    fn assassin_reset_impl(&mut self, inherit: bool, rng: &mut Rng) {
        for p in &mut self.settings.players {
            p.team = 0; // the team words zero [M :12-19]
        }
        let old = self.assassin.take(); // [M :16,20]
        let mut new_a = None;
        if inherit {
            if let Some(o) = old {
                let lk = self.game.slots[o].last_killer as i16;
                if lk >= 0 && (lk as usize) < self.game.slots.len() && self.game.slots[lk as usize].pos.is_some() {
                    new_a = Some(lk as usize); // [M :24-28]
                }
            }
        }
        let exclude = new_a.or(self.assassin_target); // [M :29-32]
        let target = self.get_target(0, exclude, rng); // [M :33-34] (all team words are 0 now)
        self.assassin_target = target;
        if new_a.is_none() {
            new_a = self.get_target(0, target, rng); // [M :35-37]
        }
        self.assassin = new_a;
        if let Some(a) = self.assassin {
            self.settings.players[a].team = 1; // [M :38-41]
        }
    }

    // --------------------------------------------------------------- friendly fire

    /// `MP_RegisterBulletHit` 0x1857c0 — returns whether the hit is allowed. MP off → allowed; no
    /// attacker → blocked; the team game's friendly-fire gate (`friendly_fire == 0` blocks, the
    /// warn timers at +0x22); the victim's last-shooter word (+0x20) records the attacker in every
    /// allowed case [M 0x1857c0:33-57].
    pub fn register_bullet_hit(&mut self, victim: usize, attacker: usize) -> bool {
        if self.settings.master == 0 {
            return true; // [M 0x1857c0:15-17]
        }
        if self.game.slots[attacker].pos.is_none() {
            return false; // [M :18-20]
        }
        if self.settings.team_game != 0 {
            let same = self.settings.players[victim].team == self.settings.players[attacker].team;
            if same && victim != attacker {
                if self.settings.friendly_fire == 0 {
                    return false; // [M :43-45]
                }
                if self.game.slots[victim].timers[0] == 0 {
                    self.game.slots[victim].timers[0] = (3 * FRAME_RATE_INT) as u16; // [M :56]
                }
            }
        }
        self.game.slots[victim].status = attacker as u16; // [M :34]
        true
    }

    // ------------------------------------------------------------------- respawn

    /// `MP_ReSpawn` 0x185368:16-20 — the gates only (the position/health reset is
    /// [`MpMatch::do_respawn`]; the 5 s death delay is the caller's `Player_HandleDeath` gate [M
    /// 0x190bb0:41-54]).
    pub fn respawn_allowed(&self, slot: usize) -> bool {
        if self.settings.master == 0 || self.game.state != 0 {
            return false;
        }
        if self.settings.scenario == scenario::DEATH_LIMIT_FFA
            && self.game.slots[slot].deaths >= self.settings.frag_limit
        {
            return false; // out of lives [M 0x185368:16-20]
        }
        true
    }

    /// `MP_ReSpawn` 0x185368 — the whole per-slot respawn the live host calls: the gates
    /// ([`MpMatch::respawn_allowed`], [M 0x185368:16-20]), the spawn pick through the policy
    /// ([`MpMatch::get_spawn_point`] [M 0x185368:31 — the game passes the slot's team word from
    /// `MPSettings + slot·0x30 + 0x20`]) and the apply ([`MpMatch::do_respawn`] — the position
    /// reset, the streak/last-shooter/status words, the alive flag). Returns
    /// `Some((spawn_idx, health))` when the respawn ran, `None` on the gates. The caller moves the
    /// body and ends its death sequence (the game's `Player_SetHealth`/`Player_ChangeState(0)`/
    /// `obj+0xff = 3` half [M 0x185368:76-108] is the host's player pipeline).
    pub fn mp_respawn(&mut self, slot: usize, rng: &mut Rng) -> Option<(u32, f32)> {
        if !self.respawn_allowed(slot) {
            return None;
        }
        let team = self.settings.players[slot].team as u16;
        let idx = self.get_spawn_point(team, slot, rng);
        let hp = self.do_respawn(slot, idx);
        Some((idx, hp))
    }

    /// `MP_ReSpawn` 0x185368:22-91 — applies the spawn reset at `spawn_idx` (chosen by the
    /// caller's `MP_GetSpawnPoint`) and returns the human health (`armor_bonus + 100.0` [M
    /// :65-73]).
    pub fn do_respawn(&mut self, slot: usize, spawn_idx: u32) -> f32 {
        let p = self.spawns.points[spawn_idx as usize].pos;
        let s = &mut self.game.slots[slot];
        s.pos = Some([p[0], p[1], p[2]]);
        s.streak = 0; // [M 0x185368:55-57]
        s.status = 0xfffe;
        s.status_bits = 0;
        s.dead = false;
        s.spawn_tick = self.tick; // [M 0x185368:90 — obj+0xEC = GameState+0x52]
        self.settings.players[slot].armor_bonus as f32 + 100.0
    }

    // --------------------------------------------------------------- who won / restart

    /// `MP_SortOutWhoWon` 0x1848d0 — the decision (state 3) + the end status (`MpGame::end_status`,
    /// the status sprite's text data [M :21-92]). 0x10: the fewest deaths among the live slots
    /// under the lives limit, with the count/tie quirk [M :23-52]; FFA: exactly one slot at/over
    /// the limit, else a draw [M :55-84]; team games always report the draw (the team winner is
    /// the debriefing's line) [M :87-92]. The `Winner` return is the flow result (402's API); the
    /// status carries the codename for the draw code.
    pub fn sort_out_who_won(&mut self) -> Winner {
        self.game.state = 3; // [M 0x1848d0:20]
        let limit = self.settings.frag_limit;
        if self.settings.scenario == scenario::DEATH_LIMIT_FFA {
            // The survivor count increments BEFORE the min-compare and a tie at the running
            // minimum zeroes it [M 0x1848d0:32-40] — so the draw fires only when the count is
            // zero at the END, i.e. the LAST qualifier tied the minimum and nothing followed
            // (a later qualifier re-increments: [3,3,1] crowns the 1). 402's tie-flag version
            // missed the re-increment; mp-debriefing-data2 settled this from the decomp.
            let mut count: i32 = 0; // iVar10 [M 0x1848d0:23-52]
            let mut best = 30000i32; // iVar2's start [M :23]
            let mut winner: Option<usize> = None;
            for (i, slot) in self.game.slots.iter().enumerate() {
                if slot.pos.is_none() {
                    continue; // the obj gate [M :29]
                }
                let d = slot.deaths;
                if d >= limit {
                    continue; // [M :31]
                }
                count += 1;
                if d < best {
                    best = d;
                    winner = Some(i);
                } else if d == best {
                    count = 0; // the tie quirk [M :38-40]
                }
            }
            self.game.end_status = match winner {
                Some(w) if count != 0 => MpEndStatus::Won(self.settings.players[w].codename.clone()),
                _ => MpEndStatus::Draw, // [M :49-52]
            };
            match winner {
                Some(w) if count != 0 => Winner::Slot(w),
                _ => Winner::Draw,
            }
        } else if self.settings.team_game == 0 {
            // NO obj gate — the game's FFA qualifier loop reads only the slot scores
            // [M 0x1848d0:56-71; the decomp has no `obj != 0` test in this branch]. 402 filtered
            // on `pos.is_some()`; mp-debriefing-data2 removed it (a qualifier needs only
            // `frag_limit <= score`).
            let f_limit = limit as f32;
            let mut count = 0usize;
            let mut winner: Option<usize> = None;
            for (i, slot) in self.game.slots.iter().enumerate() {
                if f_limit <= slot.score {
                    count += 1; // [M :62-67]
                    winner = Some(i);
                }
            }
            self.game.end_status = match winner {
                Some(w) if count == 1 => MpEndStatus::Won(self.settings.players[w].codename.clone()),
                _ => MpEndStatus::Draw, // [M :79-84]
            };
            match winner {
                Some(w) if count == 1 => Winner::Slot(w),
                _ => Winner::Draw,
            }
        } else {
            // the team branch compares nothing — the status is the draw text (the team winner
            // is announced on the debriefing) [M :87-92]; `Winner::None` keeps 402's flow result.
            self.game.end_status = MpEndStatus::Draw;
            Winner::None
        }
    }

    /// `MP_RestartScenario` 0x183fc0:40-82 — the round clock back to 0, a new random site/item
    /// pick (a `Rand_Rand` draw on the shared RNG), the done flags clear, and every present slot's
    /// last-killer/last-shooter words reset to 0xffff. Scores are **not** cleared (multi-round
    /// Demolition/Protection); the players' respawn is the caller's (`MP_ReSpawn` per slot).
    pub fn restart_scenario(&mut self, rng: &mut Rng) {
        self.game.clock = 0.0; // [M 0x183fc0:40-53]
        self.game.restart_timer = 0.0;
        let s = self.settings.scenario;
        if s == scenario::DEMOLITION || s == scenario::PROTECTION {
            let pool: Vec<usize> = match s {
                scenario::DEMOLITION => (0..self.objects.demolition.len()).collect(),
                _ => (0..self.objects.protection.len()).collect(),
            };
            if let Some(site) = &mut self.objects_rt.site {
                site.sub = 0;
                site.state = 2000; // the HP re-inits [M 0x185de8:174]
                site.counter = 0;
                site.last_hitter = -1; // the re-create's 0xffff [M 0x185de8:130-131]
                if !pool.is_empty() {
                    let pick = pool[rng.rand(pool.len() as u32) as usize] as usize; // [M 0x183fc0:40-53]
                    let recs = if s == scenario::DEMOLITION { &self.objects.demolition } else { &self.objects.protection };
                    site.pos = MpObjectsRt::obj_at(recs, pick);
                    site.home = site.pos;
                    site.bounds = MpObjectsRt::bounds_at(recs, pick);
                }
            }
        }
        for slot in &mut self.game.slots {
            slot.last_killer = 0xffff; // [M 0x183fc0:55-82]
            slot.status = 0xffff;
        }
    }

    // ------------------------------------------------------- mode-object creation

    /// `MP_Start` 0x183a50:77-114 — builds the runtime objects from the registered placements.
    /// Flags/bases/uplinks/hill/espionage-bases link directly (the game creates them at
    /// registration); the demolition/protection site, the blueprint item and the two GoldenEye
    /// items pick a random registered place each (`Rand_Rand` — the game's start draws).
    /// Post-load resolve: `MP_initObjExt` (0x18b7a0) + `MP_PostLoad_Init` (0x183718) — the game
    /// re-resolves scripted site objects and re-emits AI paths for the mode objects here; the
    /// port links placements once at start [S partial].
    pub fn create_mode_objects(&mut self, rng: &mut Rng) {
        let s = self.settings.scenario;
        // the direct links from `created` (config word 0 = the type, word 1 = the team)
        // mp-scenarios-online: every MPOBJECT's +0x50 last-hitter / +0x52 controller init 0xffff
        // (−1) [M MP_CreateObject 0x185de8:130-131] — the 402 default was 0 (slot 0), which made a
        // site destroyed with no recorded hitter credit slot 0.
        let fresh = |state: i16, team: u16, pos: [f32; 3], bounds: Option<MpObjBounds>| MpObjState {
            state, team, pos, home: pos, bounds, last_hitter: -1, controller: -1, ..Default::default()
        };
        for i in 0..self.objects.created.len() {
            let (ty, team, pos, bounds) = {
                let r = &self.objects.created[i];
                (r.config[0], (r.config[1] & 0xffff) as u16, MpObjectsRt::obj_at(&self.objects.created, i), r.bounds)
            };
            match ty {
                0 if s == scenario::CAPTURE_THE_FLAG => {
                    self.objects_rt.flags[team as usize % 2] = Some(fresh(0, team, pos, bounds));
                }
                1 if s == scenario::CAPTURE_THE_FLAG => {
                    self.objects_rt.bases[team as usize % 2] = Some(pos);
                    self.objects_rt.bases_bounds[team as usize % 2] = bounds;
                }
                2 if s == scenario::UPLINK => {
                    // the neutral init [M 0x185de8:146]
                    self.objects_rt.uplinks.push(fresh(2, team, pos, bounds));
                }
                4 if s == scenario::BLUEPRINT => {
                    self.objects_rt.esp_bases[team as usize % 2] = Some(pos);
                    self.objects_rt.esp_bases_bounds[team as usize % 2] = bounds;
                }
                9 if s == scenario::KOTH_FFA || s == scenario::KOTH_TEAM => {
                    self.objects_rt.hill = Some(fresh(0, team, pos, bounds));
                }
                _ => {}
            }
        }
        // the random picks [M 0x183a50:77-114]
        if s == scenario::DEMOLITION || s == scenario::PROTECTION {
            let (records, n) = if s == scenario::DEMOLITION {
                (0, self.objects.demolition.len())
            } else {
                (1, self.objects.protection.len())
            };
            if n > 0 {
                let pick = rng.rand(n as u32) as usize;
                let recs = if records == 0 { &self.objects.demolition } else { &self.objects.protection };
                let (pos, bounds) = (MpObjectsRt::obj_at(recs, pick), MpObjectsRt::bounds_at(recs, pick));
                // the HP init 2000 [M 0x185de8:174]
                self.objects_rt.site = Some(fresh(2000, if s == scenario::DEMOLITION { 0 } else { 1 }, pos, bounds));
            }
        }
        if s == scenario::BLUEPRINT && !self.objects.blueprint_items.is_empty() {
            let n = self.objects.blueprint_items.len();
            let pick = rng.rand(n as u32) as usize;
            let pos = MpObjectsRt::obj_at(&self.objects.blueprint_items, pick);
            let bounds = MpObjectsRt::bounds_at(&self.objects.blueprint_items, pick);
            self.objects_rt.blueprint = Some(fresh(0, 0, pos, bounds));
        }
        if s == scenario::GOLDENEYE {
            for i in 0..2 {
                let pool = &self.objects.goldeneye[i];
                if !pool.is_empty() {
                    let pick = rng.rand(pool.len() as u32) as usize; // [M 0x183a50:101-114]
                    let (pos, bounds) = (MpObjectsRt::obj_at(pool, pick), MpObjectsRt::bounds_at(pool, pick));
                    let mut it = fresh(0, 0, pos, bounds);
                    it.sub = i as i16;
                    self.objects_rt.ge[i] = Some(it);
                }
            }
        }
    }

    // ------------------------------------------------------------- per-frame updates

    /// The scenario-object updates of one frame — the port's `MP_ObjectUpdate` 0x18b198 walk.
    /// The game's object system runs every frame regardless of the match state (the Protection
    /// timeout scores the frame after the state-6 latch — the site's own done flag stops it), so
    /// this is gated on the master/started flags only. `on_hill` is the KOTH volume test
    /// ([`MpMatch::hill_mask`] for objects with bounds — the live host's
    /// [`MpMatch::update_objects_live`]); `rng` is drawn only by the GoldenEye/blueprint
    /// re-picks and the strike's target pick.
    ///
    /// mp-scenarios-online: objects that carry model bounds (the live load registers them) run the
    /// game's own contact walk inside their update (`MP_HitBy` 0x1830e0 — the port's stand-in
    /// [`MpMatch::hit_by`]), so a flag/terminal/item/blueprint reacts to the bodies that reach it
    /// on every machine from the shared state alone. Objects without bounds (the headless rigs)
    /// keep the explicit `touch_*` events.
    pub fn update_objects(&mut self, dt: f32, on_hill: &[bool; super::MP_MAX_SLOTS], rng: &mut Rng) {
        if self.settings.master == 0 || !self.settings.started {
            return;
        }
        // mp-hitby-contacts: the frame counter the spawn window reads [M 0x1830e0:36-39] — the
        // port's `GameState+0x52` stand-in, one advance per object walk (the fixed 60 Hz).
        self.tick = self.tick.wrapping_add(1);
        self.uplink_update(dt);
        self.koth_update(dt, on_hill);
        self.site_update();
        self.flag_update();
        self.ge_update(dt, rng);
        self.blueprint_update(dt, rng);
    }

    /// mp-scenarios-online: the live host's per-frame object walk — the hill occupancy from the
    /// hill object's own box ([`MpMatch::hill_mask`]), then [`MpMatch::update_objects`].
    pub fn update_objects_live(&mut self, dt: f32, rng: &mut Rng) {
        let mask = self.hill_mask();
        self.update_objects(dt, &mask, rng);
    }

    /// The slot's world-sphere centre (`obj+0x80`) from the match's live position: the human slot
    /// position is `Bond::pos`, a bot slot's is its eye (`BOT_EYE` = `player::EYE` above the same
    /// standing position) — both map to the body's sphere centre `PLAYER_SPHERE_DY` below the
    /// standing position [S stand-in for `Control_BuildWorldSph` 0x133070].
    pub fn slot_centre(&self, slot: usize) -> Option<[f32; 3]> {
        let p = self.game.slots.get(slot)?.pos?;
        let bot = self.bots.iter().any(|b| b.slot == slot);
        let dy = if bot { -crate::player::EYE } else { 0.0 };
        Some([p[0], p[1] + dy + PLAYER_SPHERE_DY, p[2]])
    }

    /// `MP_HitBy(obj, team, exclude, ..)` 0x1830e0 — the first live player in contact with the
    /// object whose team word matches (`team == 2` = any) [M :32-35], skipping the dead [M
    /// :41-75 — the port's `dead` marker] and any body within the 4-frame spawn window [M
    /// :36-39 — `|GameState+0x52 − obj+0xEC| < 4`; the port's counter is [`MpMatch::tick`], the
    /// stamps `MpSlot::spawn_tick`]. The game's contact is the frame's HITDATA list (`obj+0xD0`,
    /// rebuilt by `Collide_Update` 0x1ea398); the port's contact is the pick's sphere law — the
    /// body's world sphere against the object's (`pos + sphere_off`, the placement rotation
    /// applied [M `Control_BuildWorldSph` 0x133070; `Collide_PickObj` 0x1eae80:221-228, STRICT
    /// `<`]) — tested in slot order [S the frame's sort cannot order all-overlapping contacts:
    /// every body sphere's refined distance is 0, 0x1ebaa8:269-271].
    pub fn hit_by(&self, centre: [f32; 3], radius: f32, team: u32, exclude: Option<usize>) -> Option<usize> {
        let reach = radius + PLAYER_SPHERE_RADIUS;
        for slot in 0..self.game.slots.len() {
            let s = &self.game.slots[slot];
            if s.pos.is_none() || s.dead || Some(slot) == exclude {
                continue;
            }
            if team != 2 && self.settings.players[slot].team != team {
                continue;
            }
            // the spawn window [M 0x1830e0:36-39 — the abs diff against the body's +0xEC stamp]
            let diff = (self.tick as i32).wrapping_sub(s.spawn_tick as i32);
            if diff.abs() < 4 {
                continue;
            }
            let c = self.slot_centre(slot).unwrap();
            if sq_dist(c, centre) < reach * reach {
                return Some(slot);
            }
        }
        None
    }

    /// `MP_HitBy` against one scenario object — the contact sphere's world centre is
    /// `pos + sphere_off` (the local centre through the placement rotation [M
    /// `Control_BuildWorldSph` 0x133070:25-36]), radius `bounds.radius` (the placement scale is
    /// 1.0 [M 0x134630:19, 0x133070:37-38]). `None` when the object has no bounds.
    fn obj_hit_by(&self, o: &MpObjState, team: u32) -> Option<usize> {
        let b = o.bounds?;
        let c = [o.pos[0] + b.sphere_off[0], o.pos[1] + b.sphere_off[1], o.pos[2] + b.sphere_off[2]];
        self.hit_by(c, b.radius, team, None)
    }

    /// The KOTH hill occupancy per slot — `MP_isPosOnHill` 0x18bc20 / `MP_KOHUpdate`
    /// 0x186ae0:88-93: `Intersect_SphereBox(0.0, body centre − hill pos, box min, box max)` with
    /// the hill's celglist box (+0x20/+0x30) and radius **0.0** [M disasm 0x18bc40 `mtc1 zero,
    /// $f12`; 0x186b68/0x186b7c `mtc1 zero,$f20` → `mov.s $f12,$f20`] — the point-in-box test
    /// [M `Intersect_SphereBox` 0x1ed988]. All false without a hill or its bounds.
    pub fn hill_mask(&self) -> [bool; super::MP_MAX_SLOTS] {
        let mut mask = [false; super::MP_MAX_SLOTS];
        let Some(h) = &self.objects_rt.hill else { return mask };
        let Some(b) = h.bounds else { return mask };
        for (slot, on) in mask.iter_mut().enumerate() {
            if let Some(c) = self.slot_centre(slot) {
                let p = [c[0] - h.pos[0], c[1] - h.pos[1], c[2] - h.pos[2]];
                *on = (0..3).all(|k| b.min[k] <= p[k] && p[k] <= b.max[k]);
            }
        }
        mask
    }

    /// The team score as `MP_UplinkUpdate`/`MP_KOHUpdate` recompute it: zeroed, then
    /// `+= (float)(int)score` for every slot on the team — the personal scores are **truncated**
    /// to whole points before the sum [M 0x186ae0:124-136 `(float)(int)*pfVar8`, 0x186d98:70-78
    /// the same]. (402 summed the raw floats, so a team showed fractions the game never shows.)
    fn recompute_team_sum(&mut self, t: usize) {
        let mut sum = 0.0f32;
        for i in 0..self.game.slots.len() {
            if self.settings.players[i].team as usize == t {
                sum += (self.game.slots[i].score as i32) as f32;
            }
        }
        self.game.team_scores[t] = sum;
    }

    /// `MP_UplinkUpdate` 0x186d98 — an owned terminal (state 0/1): if no player of the owning team
    /// is on it, an enemy contact flips it (the controller follows) [M :24-52 — a defender on the
    /// terminal blocks the flip]; then the owning team drips 0.2/s and a recorded controller
    /// drips 0.2/s personally, and in team games the owning team's score is recomputed as the
    /// truncated Σ [M :59-80]. A neutral terminal (2) clears its controller every frame, then a
    /// team-0 contact claims it, then a team-1 contact claims it (team 1 wins a tie — the second
    /// test overwrites) [M :83-114].
    fn uplink_update(&mut self, dt: f32) {
        if self.settings.scenario != scenario::UPLINK {
            return;
        }
        for ui in 0..self.objects_rt.uplinks.len() {
            let state = self.objects_rt.uplinks[ui].state;
            if state == 0 || state == 1 {
                let defended = self.obj_hit_by(&self.objects_rt.uplinks[ui], state as u32).is_some();
                if !defended {
                    let enemy = (state == 0) as u32;
                    if let Some(p) = self.obj_hit_by(&self.objects_rt.uplinks[ui], enemy) {
                        let u = &mut self.objects_rt.uplinks[ui];
                        u.sub = 0;
                        u.state = enemy as i16;
                        u.controller = p as i16;
                    }
                }
                let (state, controller) = {
                    let u = &self.objects_rt.uplinks[ui];
                    (u.state, u.controller)
                };
                let t = state as usize;
                let drip = dt * 0.2; // REC_FRAME_RATE * 0.2 [M 0x186d98:59-60]
                self.game.team_scores[t] += drip;
                if controller != -1 {
                    let c = controller as usize;
                    self.game.slots[c].score += drip; // [M :61-65]
                    if self.settings.team_game != 0 {
                        self.recompute_team_sum(t); // [M :66-80]
                    }
                }
            } else if state == 2 {
                self.objects_rt.uplinks[ui].controller = -1; // [M :84]
                if let Some(p) = self.obj_hit_by(&self.objects_rt.uplinks[ui], 0) {
                    let u = &mut self.objects_rt.uplinks[ui];
                    u.state = 0; // [M :85-98]
                    u.controller = p as i16;
                }
                if let Some(p) = self.obj_hit_by(&self.objects_rt.uplinks[ui], 1) {
                    let u = &mut self.objects_rt.uplinks[ui];
                    u.state = 1; // [M :99-113]
                    u.controller = p as i16;
                }
            }
        }
    }

    /// `MP_UplinkUpdate` 0x186d98:24-114 — the touch rule: a neutral terminal is claimed by the
    /// first toucher; an owned terminal flips at once to an enemy toucher (the controller follows).
    pub fn touch_uplink(&mut self, term: usize, slot: usize) {
        if self.settings.scenario != scenario::UPLINK {
            return;
        }
        let team = self.settings.players[slot].team as i16;
        let u = &mut self.objects_rt.uplinks[term];
        if u.state != team {
            u.state = team;
            u.controller = slot as i16; // [M 0x186d98:88-98, 24-52]
            u.sub = 0;
        }
    }

    /// `MP_KOHUpdate` 0x186ae0:78-151 — every alive player on the hill drips 0.2/s personally; in
    /// team games the scoring player's team score is recomputed as the truncated Σ of that team's
    /// personals [M :122-137]. The on-hill status bit 0x10 (+0x26) sets while on the hill and
    /// clears when an alive player leaves it [M :95-115 `MP_setPlayerStatus(.., 0x10, ..)`]; the
    /// 5-point sound is presentation.
    fn koth_update(&mut self, dt: f32, on_hill: &[bool; super::MP_MAX_SLOTS]) {
        let s = self.settings.scenario;
        if s != scenario::KOTH_FFA && s != scenario::KOTH_TEAM {
            return;
        }
        for slot in 0..self.game.slots.len() {
            if self.game.slots[slot].pos.is_none() || self.game.slots[slot].dead {
                continue; // the obj + alive gates [M 0x186ae0:80-86]
            }
            if !on_hill[slot] {
                self.game.slots[slot].status_bits &= !0x10; // [M :94-98]
                continue;
            }
            self.game.slots[slot].status_bits |= 0x10; // [M :115]
            self.game.slots[slot].score += dt * 0.2; // [M :118-121]
            if self.settings.team_game != 0 {
                let t = self.settings.players[slot].team as usize;
                self.recompute_team_sum(t); // [M :122-137]
            }
        }
    }

    /// `MP_DemolitionProtectionUpdate` 0x1870a0:112-124 — the timeout scores the **defending**
    /// team (0 for Demolition, 1 for Protection — the dispatch's param_3 [M 0x18b198:22-24,35-37]).
    fn site_update(&mut self) {
        let s = self.settings.scenario;
        if s != scenario::DEMOLITION && s != scenario::PROTECTION {
            return;
        }
        let defending = if s == scenario::DEMOLITION { 0usize } else { 1usize };
        let done = self.objects_rt.site.as_ref().map(|x| x.sub == 1).unwrap_or(true);
        if done {
            return; // [M 0x1870a0:77-79]
        }
        if self.game.limit > 0.0 && self.game.limit < self.game.clock {
            self.game.team_scores[defending] += 1.0; // [M :112-124]
            if let Some(site) = &mut self.objects_rt.site {
                site.sub = 1;
            }
        }
    }

    /// mp-scenarios-online: the site's world sphere (position + the model's local sphere centre,
    /// radius) while the round is live — the live host's shots test it (`SP_GetHitDamage` on the
    /// site's SP [M 0x1870a0:81-83] stands on the site model's hits; the port's online shot ray
    /// against this sphere is the stand-in [S]).
    pub fn site_sphere(&self) -> Option<([f32; 3], f32)> {
        let s = self.settings.scenario;
        if s != scenario::DEMOLITION && s != scenario::PROTECTION {
            return None;
        }
        let site = self.objects_rt.site.as_ref()?;
        if site.sub == 1 {
            return None;
        }
        let b = site.bounds?;
        Some(([site.pos[0] + b.centre[0], site.pos[1] + b.centre[1], site.pos[2] + b.centre[2]], b.radius))
    }

    /// `MP_DemolitionProtectionUpdate` 0x1870a0:81-165 — the site takes damage (HP = obj+0xF4,
    /// init 2000 [M 0x185de8:174]; the store is `(short)(int)((float)(int)hp − dmg)` [M :81-83]
    /// and a positive hit records the hitter [M `MP_ApplyDamage` 0x18b2c8, :84-91]). HP < 1: the
    /// attacking team +1.0 always; the last hitter's personal delta is +1.0, or −1.0 when the
    /// hitter is a defender — whose team also pays 1.0 (the own-goal REPLACES the hitter's +1,
    /// `fVar10 = -1.0` [M :129-140]; 402 added both and left the hitter at 0); no recorded hitter
    /// (0xffff) scores no one personally [M :131]. The site is done and the round enters state 6
    /// at once [M :141-160].
    pub fn damage_site(&mut self, dmg: f32, hitter: usize) {
        let s = self.settings.scenario;
        if s != scenario::DEMOLITION && s != scenario::PROTECTION {
            return;
        }
        let defending = if s == scenario::DEMOLITION { 0usize } else { 1usize };
        {
            let Some(site) = &mut self.objects_rt.site else { return };
            if site.sub == 1 {
                return;
            }
            site.state = ((site.state as i32) as f32 - dmg) as i32 as i16; // [M 0x1870a0:81-83]
            if dmg > 0.0 {
                site.last_hitter = hitter as i16; // [M :84-91]
            }
            if site.state >= 1 {
                return;
            }
        }
        let attacking = 1 - defending;
        self.game.team_scores[attacking] += 1.0; // [M :125-132]
        let h = self.objects_rt.site.as_ref().unwrap().last_hitter;
        if h >= 0 {
            let h = h as usize;
            let delta = if self.settings.players[h].team as usize == defending {
                self.game.team_scores[defending] -= 1.0; // the own-goal [M :133-137]
                -1.0
            } else {
                1.0
            };
            self.game.slots[h].score += delta; // [M :138-140]
        }
        let site = &mut self.objects_rt.site;
        site.as_mut().unwrap().sub = 1;
        self.game.state = 6; // the round is over [M :160]
    }

    fn flag_home(&mut self, ti: usize) {
        let f = self.objects_rt.flags[ti].as_mut().unwrap();
        f.state = 0;
        f.carrier = None;
        f.counter = 0;
        f.pos = f.home;
    }

    fn flag_take(&mut self, ti: usize, slot: usize) {
        let f = self.objects_rt.flags[ti].as_mut().unwrap();
        f.state = 1;
        f.carrier = Some(slot);
        f.counter = 0;
        self.game.slots[slot].status_bits |= 1; // [M 0x18a518:88-96]
    }

    /// `MP_FlagUpdate` 0x1861a8 — state 0: an enemy contact takes the flag [M :170-190]; state 1:
    /// the flag rides its carrier, and within sq-dist 2.0 of the enemy base +1.0 y the capture
    /// scores the taker's team +1.0 and the carrier +1.0 and the flag resets home [M :55-112];
    /// state 2 (dropped): a teammate contact returns it home [M :195-221], else an enemy contact
    /// takes it [M :223-263], else the return counter counts and past 30 s it goes home [M
    /// :226-259].
    fn flag_update(&mut self) {
        if self.settings.scenario != scenario::CAPTURE_THE_FLAG {
            return;
        }
        for ti in 0..2 {
            let (state, carrier, counter, fteam) = match &self.objects_rt.flags[ti] {
                Some(f) => (f.state, f.carrier, f.counter, f.team),
                None => continue,
            };
            let enemy = (fteam == 0) as u32; // `MP_HitBy(obj, flag team == 0, ..)` [M :172,225]
            match state {
                0 => {
                    if let Some(p) = self.obj_hit_by(self.objects_rt.flags[ti].as_ref().unwrap(), enemy) {
                        self.flag_take(ti, p);
                    }
                }
                1 => {
                    let Some(c) = carrier else { continue };
                    let Some(cpos) = self.game.slots[c].pos else { continue };
                    // the flag rides the carrier (`AnimGetBoneWorldTrans` bone 0x15 [M :57-63];
                    // the port's carrier position)
                    self.objects_rt.flags[ti].as_mut().unwrap().pos = cpos;
                    let Some(base) = self.objects_rt.bases[1 - ti] else { continue };
                    let point = [base[0], base[1] + 1.0, base[2]]; // [M 0x1861a8:74-86]
                    if sq_dist(cpos, point) < CAPTURE_SQ_DIST {
                        self.game.team_scores[1 - ti] += 1.0; // [M :89-93]
                        self.game.slots[c].score += 1.0; // [M :94-98]
                        self.flag_home(ti);
                        self.game.slots[c].status_bits &= !1; // the carrying bit clears [M :87-88]
                    }
                }
                2 => {
                    if self.obj_hit_by(self.objects_rt.flags[ti].as_ref().unwrap(), fteam as u32).is_some() {
                        self.flag_home(ti); // the teammate return [M :195-221]
                        continue;
                    }
                    if let Some(p) = self.obj_hit_by(self.objects_rt.flags[ti].as_ref().unwrap(), enemy) {
                        self.flag_take(ti, p); // [M :223-263]
                        continue;
                    }
                    let t = counter;
                    if t > RETURN_FRAMES as u16 {
                        self.flag_home(ti); // the 30 s return [M :226-259]
                    } else {
                        self.objects_rt.flags[ti].as_mut().unwrap().counter = t + 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// `MP_FlagUpdate` 0x1861a8:170-263 — the touch rule: an enemy takes the flag (from the base
    /// or dropped); a teammate touching a dropped flag returns it at once. The port's stand-in for
    /// the `MP_HitBy` walk with the enemy-team filter.
    pub fn touch_flag(&mut self, slot: usize) {
        if self.settings.scenario != scenario::CAPTURE_THE_FLAG {
            return;
        }
        let my_team = self.settings.players[slot].team;
        for ti in 0..2 {
            let (state, fteam) = match &self.objects_rt.flags[ti] {
                Some(f) => (f.state, f.team),
                None => continue,
            };
            if state == 2 && my_team == fteam as u32 {
                self.flag_home(ti); // the teammate return [M :195-221]
                return;
            }
            if (state == 0 || state == 2) && my_team != fteam as u32 {
                self.flag_take(ti, slot); // the pickup [M :170-190, 223-263]
                return;
            }
        }
    }

    /// `MP_GoldenEyeUpdate` 0x187568 — the strike in flight completes after
    /// [`GE_STRIKE_FRAMES`] (the SP's state 2 [M :86-96], [S] timing); a free item (0) is taken
    /// by any contact [M :389-400]; a carried item rides its carrier; a dropped item (2) is taken
    /// by any contact, else its counter counts and past 30 s it re-picks a random pool spawn [M
    /// :187-228]; then the launch when both items are carried by the same team (each carrier
    /// +1.0; the target is a random opposing player via `MP_GetTarget` [M :356-384]).
    fn ge_update(&mut self, dt: f32, rng: &mut Rng) {
        if self.settings.scenario != scenario::GOLDENEYE {
            return;
        }
        let in_flight = self.objects_rt.ge.iter().flatten().any(|it| it.state == 3);
        if in_flight {
            self.objects_rt.ge_strike_timer = self.objects_rt.ge_strike_timer.saturating_add(1);
            // The strike SP's beam tracks the target: the target obj's position with y − 1.0, the
            // redraw flag set — the game's per-tick `SP_SetPos` before its landed check [M
            // 0x187568:87-95]. Tick 0's launch position is the item obj's `+0x30` (`SP_Create`'s
            // pos [M :383-384]); from the first update tick the tracker owns the position.
            self.objects_rt.ge_strike_pos = self.objects_rt.ge_target.and_then(|t| self.game.slots[t].pos).map(|p| [p[0], p[1] - 1.0, p[2]]);
            // The impact: the SP's script 0x0600003e ends — `SP_Update` 0x1dbda8's state-5 arm
            // (Script_IsPlaying 0x1db630 reads the play byte done), `SP_GetState` 0x1dc900 then
            // reports 2 [M 0x187568:96] — after the script's own length in ticks [D 07000023.bin
            // 0600003e: 299]. ge_flight_ticks 0 = the arena has no script: the ScriptPlayer never
            // plays and the strike never lands [M Script_Load 0x1d8248's failed-lookup arm].
            if self.objects_rt.ge_flight_ticks > 0 && self.objects_rt.ge_strike_timer >= self.objects_rt.ge_flight_ticks {
                self.ge_strike_complete(rng);
            }
            return; // state 3 does nothing else [M :258-261]
        }
        for i in 0..2 {
            let (state, carrier, counter) = match &self.objects_rt.ge[i] {
                Some(it) => (it.state, it.carrier, it.counter),
                None => continue,
            };
            match state {
                0 => {
                    if let Some(p) = self.obj_hit_by(self.objects_rt.ge[i].as_ref().unwrap(), 2) {
                        self.touch_ge_item(i, p);
                    }
                }
                1 => {
                    if let Some(cpos) = carrier.and_then(|c| self.game.slots[c].pos) {
                        self.objects_rt.ge[i].as_mut().unwrap().pos = cpos; // the carry bone [M :306-318]
                    }
                }
                2 => {
                    if let Some(p) = self.obj_hit_by(self.objects_rt.ge[i].as_ref().unwrap(), 2) {
                        self.touch_ge_item(i, p); // [M :187-190]
                        continue;
                    }
                    if counter > RETURN_FRAMES as u16 {
                        let n = self.objects.goldeneye[i].len();
                        if n > 0 {
                            let pick = rng.rand(n as u32) as usize;
                            let pos = MpObjectsRt::obj_at(&self.objects.goldeneye[i], pick);
                            let it = self.objects_rt.ge[i].as_mut().unwrap();
                            it.state = 0;
                            it.carrier = None;
                            it.counter = 0;
                            it.pos = pos;
                            it.home = pos;
                        }
                    } else {
                        self.objects_rt.ge[i].as_mut().unwrap().counter = counter + dt_frame(dt);
                    }
                }
                _ => {}
            }
        }
        // the strike launch [M :356-384]
        let (s0, s1, c0, c1) = match (&self.objects_rt.ge[0], &self.objects_rt.ge[1]) {
            (Some(a), Some(b)) => (a.state, b.state, a.carrier, b.carrier),
            _ => return,
        };
        if s0 == 1 && s1 == 1 {
            let (Some(c0), Some(c1)) = (c0, c1) else { return };
            let ct = self.settings.players[c0].team;
            if ct == self.settings.players[c1].team {
                // the target: `MP_GetTarget(carrying team == 0, 0, 1)` — a random player of the
                // opposing team [M :362-367]; no target = no launch [M :364-366]
                let filter = if ct == 0 { 1u32 } else { 0u32 };
                let cands: Vec<usize> = (0..self.game.slots.len())
                    .filter(|i| {
                        self.game.slots[*i].pos.is_some()
                            && !self.game.slots[*i].dead
                            && self.settings.players[*i].team == filter
                    })
                    .collect();
                if cands.is_empty() {
                    return;
                }
                self.objects_rt.ge_target = Some(cands[rng.rand(cands.len() as u32) as usize]);
                for c in [c0, c1] {
                    self.game.slots[c].score += 1.0; // [M :368-377]
                }
                for i in 0..2 {
                    self.objects_rt.ge[i].as_mut().unwrap().state = 3; // [M :379-384]
                }
                self.objects_rt.ge_strike_timer = 0;
                // The strike ScriptPlayer is created at the carried item obj's `+0x30` position
                // (`SP_Create(.., 0x600003e, ..)` [M :383-384]) — the tracked position until the
                // first update tick re-sets it onto the target.
                self.objects_rt.ge_strike_pos = Some(self.objects_rt.ge[0].as_ref().map_or([0.0; 3], |it| it.pos));
            }
        }
    }

    /// mp-goldeneye-strike: the strike's draw state (docs/systems/mp-goldeneye-strike.md) —
    /// None when no strike is in flight. The blast renders at the last tracked position on the
    /// tick this view vanishes; the beam/light visuals ride the script's own streams (already in
    /// `level.nis` for the renderer).
    pub fn ge_strike_view(&self) -> Option<GeStrikeView> {
        if !self.objects_rt.ge.iter().flatten().any(|it| it.state == 3) {
            return None;
        }
        let target = self.objects_rt.ge_target?;
        let pos = self.objects_rt.ge_strike_pos?;
        Some(GeStrikeView {
            target,
            pos,
            ticks_left: self.objects_rt.ge_flight_ticks.saturating_sub(self.objects_rt.ge_strike_timer),
        })
    }

    /// `MP_GoldenEyeUpdate` 0x187568:86-179 — the strike completes: the target dies through the
    /// normal death path (`Player_Kill` / the bot's health −1 [M :98-108] → `MP_PlayerKilled`
    /// with no credit — the port runs [`MpMatch::player_killed`] with no killer, unless the
    /// target is already dead [M :99]), the **opposite** team's score += 1.0 and the target's
    /// last-shooter word resets [M :110-119], and both items re-pick random pool spawns (a draw
    /// each) as free items [M :120-179].
    pub fn ge_strike_complete(&mut self, rng: &mut Rng) {
        if self.settings.scenario != scenario::GOLDENEYE {
            return;
        }
        self.objects_rt.ge_strike_timer = 0;
        self.objects_rt.ge_strike_pos = None; // the strike global clears [M 0x187568:121-122]
        if let Some(t) = self.objects_rt.ge_target.take() {
            if !self.game.slots[t].dead {
                self.player_killed(t, None, false, false, rng); // [M 0x187568:98-108]
            }
            let tt = self.settings.players[t].team as usize;
            self.game.team_scores[1 - tt.min(1)] += 1.0; // [M 0x187568:110-117]
            self.game.slots[t].status = 0xfffe; // [M :118]
        }
        for i in 0..2 {
            let n = self.objects.goldeneye[i].len();
            if n > 0 {
                let pick = rng.rand(n as u32) as usize;
                let pos = MpObjectsRt::obj_at(&self.objects.goldeneye[i], pick);
                let it = self.objects_rt.ge[i].as_mut().unwrap();
                it.state = 0;
                it.carrier = None;
                it.counter = 0;
                it.pos = pos;
                it.home = pos;
            }
        }
    }

    /// `MP_GoldenEyeUpdate` 0x187568:231-253 — the item touch: any player of any team picks a free
    /// or dropped item (the carrying team is recorded).
    pub fn touch_ge_item(&mut self, item: usize, slot: usize) {
        if self.settings.scenario != scenario::GOLDENEYE {
            return;
        }
        let it = &mut self.objects_rt.ge[item];
        let Some(it) = it else { return };
        if it.state == 0 || it.state == 2 {
            it.state = 1;
            it.carrier = Some(slot);
            it.counter = 0;
            it.team = self.settings.players[slot].team as u16; // the MP_HitBy out-param [M :241]
        }
    }

    /// `MP_BluePrintUpdate` 0x1882d8 + `MP_BluePrintReachedBase` 0x188970 — a free/dropped
    /// blueprint is taken by any contact [M 0x1882d8:124-145,148]; the carrier reaching their
    /// team's base (the base's HITDATA list holding the carrier [M 0x1882d8:68-77] — the port's
    /// sphere contact with the base model when it has bounds, else the capture radius) scores
    /// team +1.0 and carrier +1.0 and respawns the blueprint at a random registered spawn [M
    /// 0x188970:27-55]; the dropped item resets to free where it lies after 30 s [M
    /// 0x1882d8:147-180].
    fn blueprint_update(&mut self, dt: f32, rng: &mut Rng) {
        if self.settings.scenario != scenario::BLUEPRINT {
            return;
        }
        let (state, carrier, counter) = match &self.objects_rt.blueprint {
            Some(bp) => (bp.state, bp.carrier, bp.counter),
            None => return,
        };
        match state {
            0 => {
                if let Some(p) = self.obj_hit_by(self.objects_rt.blueprint.as_ref().unwrap(), 2) {
                    self.touch_blueprint(p);
                }
            }
            1 => {
                let Some(c) = carrier else { return };
                let Some(cpos) = self.game.slots[c].pos else { return };
                self.objects_rt.blueprint.as_mut().unwrap().pos = cpos; // the carry bone [M :50-58]
                let team = self.settings.players[c].team as usize;
                let Some(Some(base)) = self.objects_rt.esp_bases.get(team).copied() else { return };
                let reached = match self.objects_rt.esp_bases_bounds.get(team).copied().flatten() {
                    Some(b) => {
                        let centre = [base[0] + b.sphere_off[0], base[1] + b.sphere_off[1], base[2] + b.sphere_off[2]];
                        let reach = b.radius + PLAYER_SPHERE_RADIUS;
                        self.slot_centre(c).is_some_and(|sc| sq_dist(sc, centre) < reach * reach)
                    }
                    None => sq_dist(cpos, base) < CAPTURE_SQ_DIST,
                };
                if reached {
                    // MP_BluePrintReachedBase [M 0x188970:27-55]
                    self.game.team_scores[team] += 1.0;
                    self.game.slots[c].score += 1.0;
                    self.game.slots[c].status_bits &= !1;
                    let n = self.objects.blueprint_items.len();
                    let bp = self.objects_rt.blueprint.as_mut().unwrap();
                    bp.state = 0;
                    bp.carrier = None;
                    bp.counter = 0;
                    if n > 0 {
                        let pick = rng.rand(n as u32) as usize;
                        bp.pos = MpObjectsRt::obj_at(&self.objects.blueprint_items, pick);
                        bp.home = bp.pos;
                    }
                }
            }
            2 => {
                if let Some(p) = self.obj_hit_by(self.objects_rt.blueprint.as_ref().unwrap(), 2) {
                    self.touch_blueprint(p); // [M 0x1882d8:148-150]
                    return;
                }
                if counter > RETURN_FRAMES as u16 {
                    let bp = self.objects_rt.blueprint.as_mut().unwrap();
                    bp.state = 0; // resets to free where it lies [M 0x1882d8:147-180]
                    bp.carrier = None;
                    bp.counter = 0;
                } else {
                    self.objects_rt.blueprint.as_mut().unwrap().counter = counter + dt_frame(dt);
                }
            }
            _ => {}
        }
    }

    /// `MP_BluePrintUpdate` 0x1882d8:124-145,182-198 — the touch rule: any player picks the free
    /// or dropped blueprint.
    pub fn touch_blueprint(&mut self, slot: usize) {
        if self.settings.scenario != scenario::BLUEPRINT {
            return;
        }
        let Some(bp) = &mut self.objects_rt.blueprint else { return };
        if bp.state == 0 || bp.state == 2 {
            bp.state = 1;
            bp.carrier = Some(slot);
            bp.counter = 0;
            bp.team = self.settings.players[slot].team as u16;
            self.game.slots[slot].status_bits |= 1;
        }
    }
}

/// The frame count a dt contributes (the fixed 60 Hz — dt is 1/60, so one frame).
fn dt_frame(dt: f32) -> u16 {
    let f = (dt * FRAME_RATE_INT as f32).round() as u16;
    if f == 0 {
        1
    } else {
        f
    }
}
