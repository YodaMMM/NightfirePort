//! Hurt volumes — placement type 0xE4 (`Hurt_Create` 0x17e870, `Hurt_Update` 0x17e5d0,
//! `Hurt_GetDamage` 0x17ea18, `Hurt_GetType` 0x17ea50), the game's contact damage volumes
//! (docs/systems/hurt-volumes.md).
//!
//! The param map [M Hurt_Create 0x17e870: the creator's record fields, `tag+0x2C+4k` = param k —
//! the convention every ported kind uses; pinned by 07000002 #13 (gate 141, sound 0xffff) and
//! 07000016 #309 (gate 243, damage 1000) [D probes]]:
//! - param 0 byte → ext+0x14: the pain type `Hurt_GetType` 0x17ea50 returns
//!   (`Player_HandlePain` 0x1902c8:84-104 gives types 1..7 armourFraction 0 — armour cannot
//!   absorb a hurt volume; the port's `PlayerHealth::pain_generic` mirrors that switch).
//! - param 1 int→float → ext+0x10: the damage `Hurt_GetDamage` 0x17ea18 returns.
//! - param 2 u16 → ext+0x18: the gate channel (`Hurt_Update` reads `switch_channels[gate]`).
//! - param 3 u16 → ext+0x1a (0 → 0xffff): the activation sound id.
//! - param 4 u16 → ext+0x16: the WHO mask the state-0 arm ANDs with the contact classes.
//! - param 5 byte → ext+0x15: the one-shot flag (state 1's expiry parks state 2 → retire).
//!
//! `Hurt_Update` 0x17e5d0's state machine [M, the decompile + disasm]:
//! - the gate maintenance is a LIVE level, every tick: with a gate set, the channel LOW sets
//!   `obj+0xF0 |= 0x20` and the channel UP clears it (`0x17e5d0:26-37`), and state 1's entry
//!   re-checks it (`:109-116` — a falling gate parks state 0 with the bit set). `Hurt_Create`'s
//!   tail sets the bit at create when the channel reads low [M 0x17e870 tail; the channels start
//!   zeroed, so every gated volume loads disabled].
//! - state 0 (armed) computes the who-mask `uVar5` (`:51-102`): gated → `uVar5 =
//!   switch_channels[gate]` — the channel VALUE is the mask, so a live channel starts the
//!   volume; ungated with who 0 → `uVar5 = 1` (starts at once); ungated with who ≠ 0 → walk the
//!   object's `obj+0xD0` hit chain OR-ing the contact classes (class 3 → 8, class 2 → 0x10,
//!   class 5 → 2; classes 1/4 collect nothing) then `uVar5 = who & collected`. `uVar5 != 0` →
//!   state 1. So who 1 collects nothing, ever — the configured-inert volumes (07000005
//!   #328–330/#334–340, 07000006 #373 [D params]); the player brings 0x10.
//! - state 1 (animating): the sound at t 0 + the spline pose; duration 0 parks (`:106-108` —
//!   every 07000005 hurt reads 0 keys [D]); a one-shot expires into state 2, which overwrites
//!   `obj+0xFF = 1` and retires the volume (`:129-132`, `:41-47`).
//! - THE DAMAGE does not flow through `Hurt_Update`: the player's object collision applies it —
//!   `Player_DealWithObjHit` 0x194458 (the class-0x27 arm) calls
//!   `Player_HandlePain(impact + Hurt_GetDamage(obj), …, Hurt_GetType(obj), −1)` with NO state
//!   or who check of its own; NPCs get the same treatment through `NDrone2_DealWithObjHit`
//!   0x13fff0:308-334, and the chain walk that reaches them is unconditional
//!   (`Player_CollisionHandler` 0x1949a8:139-145 calls the deal for every obj+0xD0 entry). The
//!   contact gate is ONLY the disabled bit: an inactive volume (the 0x20 bit set) must not
//!   produce the contact — that is what keeps 07000016's #309 "HurtPoly-ShuttleLaunch" (gate
//!   0xf3, 1000 damage [D]) from killing Bond on the silo floor during the chaser fight, before
//!   the launch raises ch 243. The who param plays no part in the damage: the port's first cut
//!   folded the state-0 START mask into the touch (an ungated who 1 volume returned no contact)
//!   — the three damage sites refute that fold (fixed 2026-11-08, hurt-volume-flight-2; the
//!   who-1 volumes 07000005 #328-330/#334-340 and 07000006 #373 never START [M :51-102] but a
//!   body that touches one takes the damage).
//!
//! The port's contact is the standing capsule against the placement's own collision triangles —
//! the glist `Control_SetGList` links at create [M 0x17e870:44], resolved like the trigger
//! builder's (the same source) and tested with the trigger walk's capsule-vs-mesh contact
//! (triggers::capsule_hits_tris). The exact BL object-collision pick is the port's approximation
//! [S]; the class arm and the numbers are [M]. Both victim passes are wired: the player's
//! (`Player_DealWithObjHit` 0x194458) and the drones' (`NDrone2_DealWithObjHit` 0x13fff0 gives
//! NPCs the same treatment) — the who masks that name the drone class bit 8, e.g. 07000008's
//! #42–44 who 0x18 (classes 3+2, the kill-floor quads, type 6, 1000 damage [D params]), are
//! delivered for it. The drone contact capsule is the trigger walk's drone body
//! ([pos−1.0 … head], r 0.4 — the Toucher law drone.rs builds for `Trigger_Calc`); the delivery
//! is the explosion path's flat subtract (`health -= damage · damage_taken`, `last_damage`
//! stamped — the drones' own update's health ≤ 0 selector picks the death).
use crate::triggers::capsule_hits_tris;
use crate::world::V3;
use nf_formats::level::Level;

#[derive(Clone, Debug)]
pub struct Hurt {
    pub placement: usize,
    /// ext+0x14 = param 0 byte: the pain type handed to `Player_HandlePain` [M 0x17e870, 0x17ea50].
    pub kind: u8,
    /// ext+0x10 = param 1 as int→float: the damage [M 0x17e870:24-31, 0x17ea18].
    pub damage: f32,
    /// ext+0x18 = param 2: the gate channel — `Hurt_Update` parks the volume while it reads low
    /// (the live 0x20 bit) [M 0x17e5d0:26-37]. 0 = ungated.
    pub gate: u16,
    /// ext+0x1a = param 3 (0 → 0xffff): the activation sound id [M 0x17e5d0:119-123; the state-1
    /// `Sound_Play3D(100.0, 1.0, 1.0, id, obj+0x30, …)`]. Delivered for the sound pass; the port
    /// does not fire it (no level in the survey sets one [D]).
    pub sound: u16,
    /// ext+0x16 = param 4: the WHO mask `Hurt_Update` 0x17e5d0's state 0 ANDs with the contact
    /// classes' bits (class 3 → 8, class 2 = the player → 0x10, class 5 → 2) when there is no
    /// gate [M 0x17e5d0:57-95]. 0 starts at once; who 1 collects nothing at all (the
    /// configured-inert volumes, e.g. 07000006 #373).
    pub who: u16,
    /// ext+0x15 = param 5: the one-shot flag — state 1's expiry parks state 2, which retires the
    /// volume [M 0x17e5d0:129-132 + :41-47].
    pub one_shot: bool,
    /// ext+0xC: the last key index (`level_tag+2` − 1, 0 when the count is 0 [M 0x17e870:26-30]).
    /// State 1 returns at once on 0, so a zero-duration volume never reaches the one-shot expiry
    /// [M 0x17e5d0:106-108, :129-132].
    pub duration: u16,
    /// The volume's cels: the placement's object collision triangles in world space (the glist
    /// `Control_SetGList` links at create [M 0x17e870:44]; the same source the trigger builder
    /// reads — `level_tag` object collision, placement matrix applied).
    pub tris: Vec<[V3; 3]>,
    pub lo: [f32; 3],
    pub hi: [f32; 3],
    /// The obj+0xF0 0x20 disabled bit: set at create when the gate channel reads low (the
    /// channels start zeroed) [M 0x17e870 tail], maintained every tick by `Hurt_Update`
    /// [M 0x17e5d0:26-37 — channel low sets it, channel up clears it].
    pub disabled: bool,
    /// The one-shot spent (the state-2 retire) [M 0x17e5d0:41-47].
    pub fired: bool,
    /// The port's mechanism witness: the damage applications this volume made (the live tests'
    /// "the volumes stay armed" pin reads it; the game's equivalent is the mirrored hit on
    /// obj+0xD0).
    pub hits: u32,
    /// The placement's raw +0x40 u16 — the path's key count [M the loader 0x1d1b68:88-92 stores
    /// it at level_tag+0x02, Hurt_Create 0x17e870 stores `count − 1` at ext+0x0C]. 0 = a static
    /// volume (the loader leaves level_tag+0x08 null; state 1 parks [M 0x17e5d0:106-108]).
    pub count: u16,
    /// The placement's raw +0x42 u16 — the flight rate: `ext[1] = (count−1) · (1/60) · (60/rate)`
    /// per tick [M 0x17e870 disasm 0x17e968-0x17e998; the constants 0x30d0dc = 1/60 and
    /// 0x30d0d0 = 60.0 [D ACTION.ELF]], so one traversal takes `rate` ticks.
    pub rate: u16,
    /// The resolved `m_paths` entry [M 0x1d1b68:88-92 — level_tag+0x08 = `m_paths[raw+0x44]`,
    /// Hurt_Create stores it at ext+0x08], the block-0x19 keys (28-byte KEYED_POSROT records,
    /// `door::parse_keys`). None = no path (every non-07000046 volume [D census]).
    pub path: Option<Vec<crate::door::Key>>,
    /// ext[0]: the flight clock [M 0x17e5d0:119-131 — advanced by ext[1] every state-1 tick,
    /// reset to 0 when it passes count−1].
    pub t: f32,
    /// `obj+0xF4 == 1`: state 1, the flight (the port's flag; the game enters it from state 0's
    /// who-mask law [M 0x17e5d0:51-102] and retires through state 2 [M :129-132]).
    pub flying: bool,
    /// The object-space collision (the glist tris before the pose) — state 1 re-poses them every
    /// tick: `Spline_Interp` writes obj+0x90 directly [M 0x17e5d0:121-123].
    pub local_tris: Vec<[V3; 3]>,
    /// obj+0x90: the current world pose — the placement's until state 1 [M 0x17e870:17], then the
    /// spline's matrix (world-space keys [D: 07000046 path 14's key 0 = the Icicle's hang spot]).
    pub pose: [[f32; 4]; 4],
    /// The glist collision's shape key in the world's hashed shape map (level-05-basin-floor-3):
    /// the target object's hashcode, or the anonymous `(map, object)` synthetic key
    /// `0x8000_0000 | (map << 16) | object` when the hashcode is MAX — the same keys
    /// `CollisionWorld::build` indexes [M world.rs's shapes build]. 0 = the placement resolves to
    /// no collision (the volume carries no solid side). The gated planes' per-tick dynamic parts
    /// pose THIS object's glist (the same source `set_dynamic`'s doors/SS/panes ride).
    pub shape_key: u32,
}

impl Hurt {
    /// `Hurt_Create` 0x17e870 over the main map's placements (placement order, like the game's
    /// `parsemap_create_dynamic_objects` 0x1d04f0 case 0xE4).
    pub fn create(level: &Level) -> Vec<Hurt> {
        let Some(m) = level.main_map() else { return Vec::new() };
        let map = &level.maps[m];
        let mut out = Vec::new();
        for (i, p) in map.placements.iter().enumerate() {
            if p.flags & 0xff != 0xe4 { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < map.objects.len() { Some((m, p.index as usize)) } else { None };
            // Hurt_Create builds obj+0x90 with CCO(0) → View_RotTransMatrix
            // [M 0x17e870:17, 0x1344e0:26-29].
            let mx = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
            let tris: Vec<[V3; 3]> = target
                .and_then(|(om, oi)| level.maps[om].objects[oi].collision.as_ref())
                .map(|c| c.triangles().into_iter().map(|t| t.map(|v| transform(&mx, v))).collect())
                .unwrap_or_default();
            let mut lo = [f32::MAX; 3];
            let mut hi = [-f32::MAX; 3];
            for t in &tris { for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } } }
            let gate = param(2) as u16;
            // The flight triple [M the loader 0x1d1b68:88-92 + Hurt_Create 0x17e870:18-27]: the raw
            // +0x40/+0x42/+0x44 u16s (extra[7] low/high, extra[8] low); the +0x44 index resolves
            // through the main map's 0x19 blocks (the game's m_paths table) only when count != 0 —
            // the loader skips the store otherwise and the pointer stays null.
            let (count, rate, path_idx) = ((p.extra[7] & 0xffff) as u16, (p.extra[7] >> 16) as u16, p.extra[8] & 0xffff);
            let path = if count != 0 {
                level.path_blocks().get(path_idx as usize).map(|b| crate::door::parse_keys(b))
            } else { None };
            let local_tris: Vec<[V3; 3]> = target
                .and_then(|(om, oi)| level.maps[om].objects[oi].collision.as_ref())
                .map(|c| c.triangles().into_iter().map(|t| t.map(|v| [v[0], v[1], v[2]])).collect())
                .unwrap_or_default();
            // The create pose [M 0x17e870:17 — CCO(0) → View_RotTransMatrix; obj+0x90 until state 1].
            let pose = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
            // The shape key (level-05-basin-floor-3): the target object's hashcode, or the
            // anonymous synthetic key when the hashcode is MAX — the keys the world's `shapes`
            // map indexes (world.rs's build). 0 when the target has no collision (no solid side).
            let shape_key = match target {
                Some((om, oi)) if level.maps[om].objects[oi].collision.is_some() => {
                    let h = level.maps[om].objects[oi].hashcode;
                    if h == u32::MAX { 0x8000_0000 | ((om as u32) << 16) | oi as u32 } else { h }
                }
                _ => 0,
            };
            out.push(Hurt {
                placement: i,
                kind: param(0) as u8,
                damage: param(1) as i32 as f32,
                gate,
                sound: param(3) as u16,
                who: param(4) as u16,
                one_shot: param(5) & 0xff == 1,
                duration: ((p.extra[7] & 0xffff) as u16).saturating_sub(1),
                tris, lo, hi,
                // Hurt_Create's tail [M]: the gate set and low at create → obj+0xF0 |= 0x20. The
                // switch channels start zeroed, so a gated volume always loads disabled.
                disabled: gate != 0,
                fired: false,
                hits: 0,
                count,
                rate,
                path,
                t: 0.0,
                flying: false,
                local_tris,
                pose,
                shape_key,
            });
        }
        out
    }

    /// The AABB quick reject + the capsule-mesh contact (the trigger `contains` rule). The
    /// capsule is the standing player segment (feet `a` → head `b`, radius `r`).
    pub fn contains(&self, a: V3, b: V3, r: f32) -> bool {
        if self.tris.is_empty() { return false; }
        if (0..3).any(|k| (a[k] < self.lo[k] - r && b[k] < self.lo[k] - r) || (a[k] > self.hi[k] + r && b[k] > self.hi[k] + r)) { return false; }
        capsule_hits_tris(a, b, r, &self.tris)
    }

    /// `ext[1]`, the per-tick flight step [M Hurt_Create 0x17e870 disasm 0x17e968-0x17e998:
    /// `(count−1) · (1/60) · (60/rate)` — the constants 0x30d0dc = 1/60, 0x30d0d0 = 60.0
    /// [D ACTION.ELF] — so the whole path takes `rate` ticks]. The count−1 term is ext+0x0C.
    pub fn step(&self) -> f32 {
        let countm1 = self.count.saturating_sub(1) as f32;
        countm1 * (1.0 / 60.0) * (60.0 / self.rate as f32)
    }

    /// State 0 → 1 [M Hurt_Update 0x17e5d0:51-102]: `uVar5 = who & collected` non-zero arms the
    /// flight. `collected` here is the class-5 bit (2): the caller feeds a player bullet's flight
    /// segment against the volume's tris (the game adds the contact to the volume's obj+0xD0 hit
    /// chain [M Collide_Update 0x1ea398]). The seven 07000046 volumes carry who 0x2 — bullet-only
    /// starters [D params]. No-op without a path or already flying/retired/disabled.
    pub fn start_flight(&mut self) {
        if self.flying || self.fired || self.disabled { return; }
        if self.path.is_some() && self.count >= 2 { self.flying = true; }
    }

    /// State 1's per-tick body [M Hurt_Update 0x17e5d0:104-132]: pose at `t` (`Spline_Interp(t,
    /// ext+0x08, 0, ext+0x0C, obj+0x90, 0, 2)` — the Catmull-Rom/slerp tail the door port pins),
    /// then `t += ext[1]`; passing ext+0x0C resets t to 0 and a one-shot parks state 2 (retire).
    /// ext+0x0C is the placement's count − 1 [M Hurt_Create 0x17e870:28-32 — `*(short *)
    /// (param_3 + 2) - 1`, the door's +0x6E convention], so both the Spline last index and the
    /// wrap threshold are count−1 and the traversal is `rate` ticks. The `t == 0` sound arm
    /// (:119-123) is not ported (no surveyed volume sets a sound id [D]; the port's hurt sound
    /// note stands). Re-poses the world tris + AABB from the spline matrix.
    pub fn tick_flight(&mut self) {
        if !self.flying { return; }
        let Some(keys) = self.path.clone() else { return; };
        let countm1 = self.count.saturating_sub(1);
        let old = self.t;
        if let Some(m) = crate::door::interp(&keys, 0, countm1, old, true) {
            self.pose = m;
            let mut lo = [f32::MAX; 3];
            let mut hi = [-f32::MAX; 3];
            self.tris = self.local_tris.iter().map(|t| {
                let w = t.map(|v| transform(&m, v));
                for v in &w { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } }
                w
            }).collect();
            self.lo = lo;
            self.hi = hi;
        }
        self.t = old + self.step();
        if self.t > countm1 as f32 {
            self.t = 0.0;
            if self.one_shot {
                self.flying = false;
                self.fired = true;
            }
        }
    }

    /// `Hurt_Update` 0x17e5d0's gate maintenance + state-0 start law, and the class-0x27 contact
    /// damage (`Player_DealWithObjHit` 0x194458's arm through the caller's `Player_HandlePain`
    /// pipeline — `PlayerHealth::pain_generic`: the difficulty factor, no armour for types 1..7
    /// [M 0x1902c8:84-104]). Returns Some((damage, pain type)) on a contact tick.
    ///
    /// `gate_on` is the gate channel's live value (the caller reads `channels.on(gate)`; a zero
    /// gate passes anything — the law below skips it). The contact gate is the state machine's:
    /// a disabled volume (the 0x20 bit the gate maintenance keeps) and an ungated who whose mask
    /// no contact class satisfies produce no contact, so no damage.
    pub fn update(&mut self, gate_on: bool, bond_pos: V3) -> Option<(f32, u8)> {
        // The standing player capsule [M Player_DealWithObjHit 0x194458 — the body the collide
        // pass tests]: feet at the position, 1.8 tall, r 0.55 [M Player_CollLadder's +0x7CC].
        let a = [bond_pos[0], bond_pos[1], bond_pos[2]];
        let b = [bond_pos[0], bond_pos[1] + 1.8, bond_pos[2]];
        self.update_touch(gate_on, a, b, 0.55, false)
    }

    /// The object-hit delivery (level-05-basin-floor-4): the state side of `update_touch`
    /// without the contains test — the contact already happened in the collide pass (the
    /// world's push log; the game's `Player_DealWithObjHit` 0x194458:94-103 fires from the
    /// per-contact hit, so a capsule standing ON a solid plane is hit). Same law as the
    /// contact arm: the disabled bit, the one-shot expiry [M `Hurt_Update` 0x17e5d0:129-132 →
    /// :41-47, :106-108], the hit witness.
    pub fn deliver_contact(&mut self) -> Option<(f32, u8)> {
        if self.disabled || self.fired { return None; }
        if self.one_shot && !self.flying && self.duration != 0 { self.fired = true; }
        self.hits += 1;
        Some((self.damage, self.kind))
    }

    /// The victim-generic contact law: `update` for the player (class 2), the drones' pass for a
    /// drone (class 3). `a`..`b` is the victim's capsule, `r` its radius. The damage arm has NO
    /// who-mask: `Player_DealWithObjHit` 0x194458:94-101 applies `Hurt_GetDamage` on every
    /// class-0x27 contact and `NDrone2_DealWithObjHit` 0x13fff0:308-334 the same on the drone
    /// side — the who param's only law is the state-0 START mask [M Hurt_Update 0x17e5d0:51-102],
    /// which lives in the starters (the class-5 bullet starter feeding `start_flight`; a static
    /// volume's start is unobservable — no path, state 1 returns at once [M :104-105]).
    /// `Hurt_Update` 0x17e5d0:26-37's gate maintenance, extracted for the scene's every-tick
    /// pass (level-05-basin-floor-5): the channel low sets obj+0xF0|0x20, the channel up clears
    /// it — a LIVE level the game's object control pass runs EVERY tick, including while the
    /// player's own update is cutscene-paused (the launch NIS on 07000016 parks the player and
    /// the pit pulse's plane still arms/drops with ch 243 [M the decompile: no player-state
    /// guard]). A zero gate never disables.
    pub fn maintain_gate(&mut self, gate_on: bool) {
        if self.gate != 0 { self.disabled = !gate_on; }
    }

    pub fn update_touch(&mut self, gate_on: bool, a: V3, b: V3, r: f32, drone: bool) -> Option<(f32, u8)> {
        self.maintain_gate(gate_on);
        if self.disabled || self.fired { return None; }
        // The collide contact: the victim's capsule against the volume's glist tris — the damage
        // arm's only condition beyond the disabled bit [M 0x194458:94-101].
        if !self.contains(a, b, r) { return None; }
        // The one-shot expiry [M 0x17e5d0:129-132 → :41-47]: state 2 retires the volume, and only
        // a volume with keys gets there [M :106-108]: 07000015's death plates and 07000025's
        // one-shots carry none [D] and stay armed. A FLYING keyed volume retires on the state-1
        // wrap alone (`tick_flight`; the contact arm never touches the state machine); a keyed
        // volume not yet flying parks on its first contact pass [S, the port's stand-in].
        if self.one_shot && !self.flying && self.duration != 0 { self.fired = true; }
        self.hits += 1;
        Some((self.damage, self.kind))
    }
}

/// The class-5 starter's narrow phase: the round's flight segment against the volume's current
/// tris — the trigger walk's segment-triangle test at radius 0 (a degenerate capsule = the
/// segment) [S: the port's approximation of the game's obj+0xD0 chain add; the class arm and the
/// who-mask law above are the [M] side].
pub fn segment_hits_tris(a: V3, b: V3, tris: &[[V3; 3]]) -> bool {
    crate::triggers::capsule_hits_tris(a, b, 0.0, tris)
}

fn transform(m: &[[f32; 4]; 4], v: V3) -> V3 {
    [
        m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0],
        m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1],
        m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit hurt whose volume is a flat 2×2 quad at the origin (two tris).
    fn hurt(gate: u16) -> Hurt {
        let tris = vec![
            [[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [1.0, 0.0, 1.0]],
            [[-1.0, 0.0, -1.0], [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0]],
        ];
        Hurt {
            placement: 0, kind: 1, damage: 1.0, gate, sound: 0xffff, who: 0, one_shot: false, duration: 0,
            tris: tris.clone(), lo: [-1.0, 0.0, -1.0], hi: [1.0, 0.0, 1.0], disabled: gate != 0,
            fired: false, hits: 0,
            // The flight fields: a static unit volume.
            count: 0, rate: 0, path: None, t: 0.0, flying: false,
            local_tris: tris, pose: [[0.0; 4]; 4],
            shape_key: 0,
        }
    }

    #[test]
    fn gate_off_disables_the_volume() {
        // [M Hurt_Create 0x17e870 tail + Hurt_Update 0x17e5d0:26-37: the gate channel off parks
        // obj+0xF0|0x20, the channel up clears it — the live every-tick law].
        let mut h = hurt(141);
        assert!(h.update(false, [0.0, 0.0, 0.0]).is_none(), "a gated-off volume never hurts");
        assert!(h.disabled);
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(), "the gate on enables it");
        assert!(!h.disabled);
        assert!(h.update(false, [0.0, 0.0, 0.0]).is_none(), "the channel falling parks it again [M :109-116]");
    }

    #[test]
    fn contact_is_the_capsule_against_the_tris() {
        let mut h = hurt(0);
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some());
        assert!(h.update(true, [0.0, 0.4, 0.0]).is_some(), "the capsule spans 0.4..2.2 — the quad is inside its radius");
        assert!(h.update(true, [0.0, 1.7, 0.0]).is_none(), "feet above the plane out of reach: no contact");
        assert!(h.update(true, [3.0, 0.0, 0.0]).is_none(), "outside the quad's reach");
    }

    #[test]
    fn the_who_mask_gates_the_start_the_damage_arm_is_who_blind() {
        // [M Hurt_Update 0x17e5d0:57-95 gates the state-0 START alone — the port's starters own
        // it (the class-5 bullet starter; start_flight's parks). The damage arm has no who:
        // Player_CollisionHandler 0x1949a8:139-145 walks obj+0xD0 unconditionally and
        // Player_DealWithObjHit 0x194458:94-101 applies Hurt_GetDamage on every class-0x27 hit —
        // a who 1 volume (07000006 #373 [D params]) never STARTS but still hurts a body that
        // touches it. (The first cut asserted the fold — is_none on the touch — which the three
        // damage sites refute; fixed with the addresses, 2026-11-08.)
        let mut h = hurt(0);
        h.who = 1;
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(),
            "who 1's touch hurts (the damage arm is who-blind [M 0x194458:94-101])");
        assert!(!h.flying, "…and no flight started (the who-mask is the starters' law [M 0x17e5d0:51-102])");
        h.who = 0x18;
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(), "who 0x18 carries the player bit");
        h.who = 0;
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(), "who 0 starts at load");
    }

    #[test]
    fn ungated_volumes_never_disable() {
        // #13's gate is 141, but a zero gate must read always-on [M 0x17e5d0:51: the gate==0
        // branch skips the channel test entirely].
        let mut h = hurt(0);
        assert!(h.update(false, [0.0, 0.0, 0.0]).is_some());
    }

    #[test]
    fn the_gated_volume_reads_the_channel_value_not_the_who_param() {
        // [M 0x17e5d0:97-99: gated → uVar5 = switch_channels[gate] — the channel VALUE is the
        // who-mask. 07000016's #309 carries who 1 AND gate 0xf3: gated, so the who 1 does NOT
        // make it inert — the launch channel does start it; the fight's low channel does not.]
        let mut h = hurt(0xf3);
        h.who = 1;
        assert!(h.update(false, [0.0, 0.0, 0.0]).is_none(), "the low channel parks it");
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(), "the channel up starts it despite who 1");
    }

    #[test]
    fn the_drone_class_bit_is_8_and_the_kill_floor_who_0x18_takes_both_victims() {
        // [M 0x17e5d0:57-95's class bits gate the START alone: class 3 (a drone) → 8, class 2 (the
        // player) → 0x10. The DAMAGE arm has no who-mask — NDrone2_DealWithObjHit 0x13fff0:308-334
        // applies Hurt_GetDamage on every class-0x27 contact like the player's 0x194458:94-101.
        // 07000008's #42-44 carry who 0x18 = 8|0x10 [D params] — the kill floor kills drones AND
        // the player, and a who 0x10 volume still hurts a drone that walks into it (the mask only
        // decides what STARTS a flying volume).]
        let mut h = hurt(0);
        assert!(h.update_touch(true, [0.0, 0.0, 0.0], [0.0, 1.8, 0.0], 0.4, true).is_some(),
            "who 0x18 takes a drone (bit 8) [M]");
        assert_eq!(h.hits, 1, "the witness counts the application (the port's mechanism witness)");
        let mut h2 = hurt(0);
        h2.who = 0x10; // player-only START mask: the drone never starts it…
        assert!(h2.update_touch(true, [0.0, 0.0, 0.0], [0.0, 1.8, 0.0], 0.4, true).is_some(),
            "…but the contact still hurts it — the damage arm reads no who [M 0x13fff0:308-334]");
        assert!(!h2.flying, "and no flight started (the start law is the starters' [M :51-102])");
        // Sister: who 1 never STARTS from a touch (it collects no class bit [M :57-95]) —
        // the damage arm is who-blind either way.
        let mut h3 = hurt(0);
        h3.who = 1;
        assert!(h3.update_touch(true, [0.0, 0.0, 0.0], [0.0, 1.8, 0.0], 0.4, true).is_some(),
            "who 1's touch hurts (no who in the damage arm [M 0x13fff0:308-334])");
        assert!(!h3.flying);
    }

    #[test]
    fn the_flight_step_is_count_minus_one_over_rate_per_tick() {
        // [M Hurt_Create 0x17e870 disasm 0x17e968-0x17e998: ext[1] = (count−1)·(1/60)·(60/rate);
        // the constants 1/60 [D 0x30d0dc] and 60.0 [D 0x30d0d0]. 07000046's Icicles: count 6,
        // rate 0x5A [D raw +0x40/+0x42] → 5/90 per tick, a traversal every 90 ticks.]
        let mut h = hurt(0);
        h.count = 6;
        h.rate = 90;
        assert!((h.step() - 5.0 / 90.0).abs() < 1e-6, "step = (count−1)/rate, got {}", h.step());
    }

    #[test]
    fn the_flight_clock_wraps_at_count_minus_one_and_a_one_shot_retires() {
        // [M Hurt_Update 0x17e5d0:121-132: Spline_Interp at t, then t += ext[1]; past count−1
        // t resets to 0 and param 5 == 1 parks state 2 — the retire.]
        let mut h = hurt(0);
        h.count = 6;
        h.rate = 60; // step 5/60: the f32 sum crosses 5.0 at the 60th add (rate 90's 5/90 may
                     // accumulate to 4.999… and never wrap — a live-check question, see the report)
        h.one_shot = true;
        h.local_tris = h.tris.clone();
        h.path = Some(vec![crate::door::Key { pos: [0.0, 10.0, 0.0], quat: [0.0, 0.0, 0.0, 1.0] }; 6]);
        h.start_flight();
        assert!(h.flying, "the path arms the flight");
        for _ in 0..61 { h.tick_flight(); }
        assert!(h.fired, "the one-shot retired at the first wrap");
        assert_eq!(h.t, 0.0, "the clock reset at the wrap [M]");
        assert!((h.pose[3][1] - 10.0).abs() < 1e-4, "the pose rode the keys (the last key's y)");
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_none(), "the retired volume never hurts again");
    }

    #[test]
    fn the_flight_without_a_path_or_a_short_count_never_arms() {
        // [M the loader 0x1d1b68: the pointer stays null when raw +0x40 == 0; Hurt_Update
        // 0x17e5d0:106-108 parks state 1 on ext+0x0C == 0.]
        let mut h = hurt(0);
        h.count = 0;
        h.start_flight();
        assert!(!h.flying);
        let mut h2 = hurt(0);
        h2.count = 1; // ext+0x0C = 0
        h2.rate = 90;
        h2.path = Some(vec![crate::door::Key { pos: [0.0; 3], quat: [0.0, 0.0, 0.0, 1.0] }]);
        h2.start_flight();
        assert!(!h2.flying, "count 1 parks (ext+0x0C == 0)");
    }

    #[test]
    fn a_one_shot_without_keys_never_retires() {
        // [M 0x17e5d0:106-108: duration 0 returns before the expiry at :129-132.]
        let mut h = hurt(0);
        h.one_shot = true;
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some());
        assert!(!h.fired);
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(), "still armed");
    }

    #[test]
    fn one_shot_retires_after_the_first_pass() {
        // [M 0x17e5d0:129-132 → :41-47: state 2 overwrites obj+0xFF = 1 — the volume retires.]
        let mut h = hurt(0);
        h.one_shot = true;
        h.duration = 5;
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_some(), "the first pass hurts");
        assert!(h.fired);
        assert!(h.update(true, [0.0, 0.0, 0.0]).is_none(), "the retired volume never hurts again");
    }
}
