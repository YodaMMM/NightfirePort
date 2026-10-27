//! The use-object classes of `Player_Activate` (0x1a5170) that the Tower mission (map-tower)
//! needs, beyond the doors/triggers the port already had:
//!
//! - **PCQWorm** (placement type **0x35** → `PCQWorm_Create` 0x1b07a8, class 0x49): the Q-Worm
//!   target computer. `PCQWorm_Update` 0x1aee20 state 0 waits for a class-5 object (a BULLET,
//!   `Bullet_init` 0x12c5a0 sets `obj+0xFF = 5`) in the PC's contact list whose weapon reads
//!   **0x58** (the Q-Worm gadget, weapon-data table [L]) and then, next update, writes
//!   `switch_channels[param1] = 1` and advances to state 2 (param 0 = the keycode count; both
//!   Tower PCs carry 0 → no keycode message, one-tick install). `PCQWorm_Activate` 0x1b0840 (the
//!   E path) starts the install only when param 2 != 0 (both Tower PCs carry 0 → E does nothing).
//! - **Switch** (placement type **0x29** → `Switch_Create` 0x1adcc0, class 0x39): param 0 = the
//!   switch channel, 2 = auto-toggle delay, 3 = the rest value, 4 = busy tracking, 5 = the gate
//!   channel. `Switch_Activate` 0x1b02a8 (reached from `SP_Activate` 0x1dc338 through the
//!   switch's owner link, or from the game's own use dispatch): toggles the channel when the
//!   gate is on (or absent), the stored value matches the channel, and the switch's script is
//!   idle. `Switch_Update` 0x1adea8 flips the stored value back after `param2` frames (none of
//!   the Tower switches set it). A class-3 activator can never toggle ch 0x61 [M 0x1b02a8].
//! - **ScriptPlayer E-activation** (`SP_Activate` 0x1dc338): the game's ONLY route to an SP from
//!   the use key is a hit class-0 script ENTITY carrying `obj+0xE4` (the owner-SP handle
//!   `Script_EntityStart` 0x1d8d78:108-110 stores) — `Player_Activate` 0x1a5170:121-122 dispatches
//!   `SP_Activate` on exactly that object, and the class walk (0x1a5170:126-175) has no 0x1B case,
//!   so the placed SP body (class 0x1B, `obj+0xE4` = 0 [L ee_start_05.bin ×8]) is inert. The pick
//!   target is the entity's world sphere (obj+0x80/0x8C, `Control_BuildWorldSph` 0x133070: the
//!   model's local sphere through the entity's script matrix). `SP_Activate` then: a link of class
//!   0x36 → `GT_Activate`; a link of class 0x39 → the facing dot (`Mat_GetDir(entity+0x90)`,
//!   `dot < 0` refuses) then `Switch_Activate` on the link; otherwise the mask gates — bit 2 (4)
//!   required, and when bit 1 (1) is also set the OUT channel must already read on — then
//!   `ext+0x2C = 1` (the `activated` flag `SP_Update` 0x1dbda8 consumes) and the current
//!   SCRIPTINFO's `+0x9C = 0` (restart). On the Tower levels this is the elevator panel
//!   (`SCRIPTPLAYER_Elevators`, 07000009 #70: mode-64 preload creates the panel entity at load,
//!   out ch 0x39 — objective 2's AND mux #274 waits for it) and the security keyboard (0700000a
//!   #60, same preload mechanism — docs/systems/elevator-button.md). The port resolves the same
//!   entity spheres (`Player::entity_use_targets`); an SP with NO live entity sphere is simply not
//!   use-activatable — the model-sphere and placement-matrix fallbacks the older tasks kept are
//!   removed (use-pick-sp-standins: nothing in 0x1a5170/0x1dc338 reads either).
//!
//! Every fix cites the game address above; spec: docs/levels/0700000{9,a,b}.md.

use crate::triggers::Channels;
use crate::world::V3;

/// A test World: no anims; the model table only knows hash 0x11 (a panel at (5,0,0), r 0.5)
/// and 0x12 (the same panel parked far away at (50,0,0), for an out-of-reach twin).
struct TestWorld;
impl crate::nis::World for TestWorld {
    fn anim_length(&self, _script: u32) -> Option<u16> { None }
    fn discrete0(&self, _skin: u32, _script: u32, _frame: f32) -> Option<nf_formats::anim::Mat4> { None }
    fn glist_sphere(&self, hash: u32) -> Option<([f32; 3], f32)> {
        match hash { 0x11 => Some(([5.0, 0.0, 0.0], 0.5)), 0x12 => Some(([50.0, 0.0, 0.0], 0.5)), _ => None }
    }
}

/// Whether the segment a→b passes within `r` of `c` (the bullet-through-contact test).
fn seg_sphere(a: V3, b: V3, c: V3, r: f32) -> bool {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if len2 <= 0.0 { 0.0 } else { (ab[0] * ac[0] + ab[1] * ac[1] + ab[2] * ac[2]).clamp(0.0, len2) / len2 };
    let p = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
    (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2) <= r * r
}

/// One Q-Worm target PC (`PCQWorm_Create` 0x1b07a8's ext: `[0]=param0 keycodes, [1]=param1
/// channel, [2]=param2 E-usable, [3]=param3`).
pub struct PcQworm {
    /// The object's world position (the placement transform; the Tower PCs carry real ones [D]).
    pub pos: V3,
    /// param 1: the channel `PCQWorm_Update` writes when the install completes [M 0x1aee20].
    pub ch: u8,
    /// param 2: non-zero makes `PCQWorm_Activate` (the E path) start the install [M 0x1b0840].
    pub e_usable: u8,
    /// param 3: `PCQWorm_Update` 0x1aee20 scans for the bullet only when it is non-zero
    /// (0x1aee20:49-51); both Tower PCs carry 1, the decoy office PCs carry 0 with no channel.
    pub armed: bool,
    /// `obj+0xF4`: 0 idle, 1 installing, ≥2 done [M 0x1aee20].
    pub state: u8,
    /// obj+0x8C: the object's world-sphere radius — `Control_BuildWorldSph` 0x133070's posed value
    /// (the model's local sphere radius, `MapObject.radius` = glist+0x2C, × |obj+0xE8|; the
    /// model-less create arm's 0.1). 0 until `set_pc_spheres` poses it — `Player_Activate`
    /// 0x1a5170 picks sphere-vs-sphere against the hit object's world sphere, not a point.
    pub radius: f32,
    /// obj+0x80: the posed world-sphere centre — `Control_BuildWorldSph` 0x133070 puts the model's
    /// LOCAL sphere centre (`MapObject.centre` = glist+0x20) through the object's matrix (the
    /// placement's rotation+translation), NOT at the placement translation. Defaults to `pos`
    /// (the model-less arm puts obj+0x80 = obj+0xC0, the matrix translation) until
    /// `set_pc_spheres` poses it.
    pub sph_centre: V3,
}

/// One switch (`Switch_Create` 0x1adcc0's ext: +0x12 = param0 channel, +0x10 = param2 auto-delay,
/// +0x16 = param3 rest value, +0x19 = param4 busy, +0x14 = param5 gate).
pub struct SwitchObj {
    pub pos: V3,
    /// param 0: the channel this switch toggles.
    pub ch: u8,
    /// param 5: the inhibit gate — usable only while this channel is OFF (0 = always allowed)
    /// [M 0x1b02a8:22 `(*(ushort *)(iVar2 + 0x14) == 0 || (switch_channels[+0x14] == '\0'))`].
    pub gate: u8,
    /// param 3: the rest value the auto-return flips toward [M 0x1adea8].
    pub rest: u8,
    /// param 4: the busy flag — with it set, the switch activates only while its channel has
    /// never changed (`Switch_Activate` 0x1b02a8:12-16 returns when +0x18 != 0; +0x18 is bumped
    /// on every channel change in `Switch_Update` 0x1adea8:22,29) — a one-shot latch, not a
    /// re-armable cooldown.
    pub busy: bool,
    /// param 2: the auto-toggle delay in frames (0 = none) [M 0x1adea8].
    pub auto_delay: u16,
    /// `obj+0xF4`: the stored value [M 0x1adcc0 creates it from param 3].
    pub value: u8,
    /// `inst+0x18`: the change latch — bumped on every channel change [M 0x1adea8:22,29].
    pub latch: u8,
    /// The auto-return frame counter (`obj+0xF6`) [M 0x1adea8].
    pub auto_ct: u16,
    /// The switch's main-map placement index (the walk order `UseObjects::create` consumed).
    pub placement: usize,
    /// The embedded ScriptPlayer's index in `Drones::scripts.players` — `Switch_Create` 0x1adcc0:44-46
    /// stores `SP_Create(pos, rot, param 1, 0, 8, 0, obj, 0)`'s handle at ext+0x0C. `None` = the
    /// switch carries no script param (the SP handle is then 0 and every `SP_*` call on it is a
    /// no-op [M SP_GetState 0x1dc900:8-10]). Set by `Drones::attach_level` (crate::drone) after the
    /// players are mounted (laser-scripts).
    pub sp: Option<usize>,
}

/// The level's use objects, in placement order.
#[derive(Default)]
pub struct UseObjects {
    pub pcs: Vec<PcQworm>,
    pub switches: Vec<SwitchObj>,
}

impl UseObjects {
    /// `Switch_GetNearest` (0x1b0398) + `Switch_GetSwitchChannel` (0x1b0430): the nearest switch
    /// object's channel (squared 3-D distance to the given position; the game walks the SwitchList in
    /// creation order). 0 when there is no switch.
    pub fn nearest_switch_channel(&self, pos: V3) -> u8 {
        let mut best = None;
        for s in &self.switches {
            let d = (s.pos[0] - pos[0]).powi(2) + (s.pos[1] - pos[1]).powi(2) + (s.pos[2] - pos[2]).powi(2);
            if best.is_none_or(|(b, _)| d < b) { best = Some((d, s.ch)); }
        }
        best.map_or(0, |(_, ch)| ch)
    }

    /// `Switch_Create` 0x1adcc0:39-43's load beat: `switch_channels[param0] = param3` (+ the time
    /// stamp) at level load. The Tower switches all carry rest 0 (no-op) — ported for fidelity.
    pub fn init_channels(&self, ch: &mut Channels, frame: u32) {
        for s in &self.switches {
            if s.ch != 0 && s.rest != 0 {
                ch.set(s.ch as u32, s.rest, frame);
                ch.time[s.ch as usize] = frame;
            }
        }
    }

    /// `parsemap_create_dynamic_objects` 0x1d04f0 cases 0x35 / 0x29, in placement order.
    pub fn create(ps: &[nf_formats::placement::Placement]) -> UseObjects {
        let mut u = UseObjects::default();
        for (pi, p) in ps.iter().enumerate() {
            if p.flags & 0xff != 0x35 && p.flags & 0xff != 0x29 { continue; }
            let par = |k: u32| p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v);
            let pos = p.translation;
            if p.flags & 0xff == 0x35 {
                // PCQWorm_Create 0x1b07a8: ext[1] = param 1 (the channel), ext[2] byte = param 2.
                u.pcs.push(PcQworm { pos, ch: par(1) as u8, e_usable: par(2) as u8, armed: par(3) != 0, state: 0, radius: 0.0,
                    sph_centre: pos });
            } else {
                // Switch_Create 0x1adcc0: +0x12 = param 0, +0x10 = param 2, +0x16 = param 3,
                // +0x19 = param 4, +0x14 = param 5; obj+0xF4 starts at param 3.
                u.switches.push(SwitchObj {
                    pos, ch: par(0) as u8, gate: par(5) as u8, rest: par(3) as u8, busy: par(4) != 0,
                    auto_delay: par(2) as u16, value: par(3) as u8, auto_ct: 0,
                    latch: 0,
                    placement: pi,
                    sp: None,
                });
            }
        }
        u
    }

    /// Pose each PC's world sphere exactly as `Control_BuildWorldSph` 0x133070 does for the placed
    /// object [M]: centre obj+0x80 = the model's LOCAL sphere centre (`MapObject.centre`, the
    /// glist+0x20 input the game reads at model+0x10) through the object's matrix (the placement's
    /// rotation+translation, `Control_GetObjMatrix` 0x134210's default arm); radius obj+0x8C =
    /// `MapObject.radius` (glist+0x2C, read at model+0x1C) × |scale| (:48). A placement whose map
    /// object the level data does not carry takes the model-less create arm (:23-31): obj+0x80 =
    /// obj+0xC0 (the matrix translation), obj+0x8C = 0.1. This replaces the half-diagonal of the
    /// posed AABB the older task stood in (that box is world-axis-aligned, so a rotated object's
    /// half-diagonal over-shoots the game's posed sphere, and its centre drifts off obj+0x80).
    /// The bullet-contact test keeps `pos` — the game's class-5 contact reads the glist box, whose
    /// port stand-in is anchored to the placement (715c's probe, unchanged here). Called once after
    /// `create` by the level build (the placements alone carry no model bounds).
    pub fn set_pc_spheres(&mut self, level: &nf_formats::level::Level, m: usize) {
        let mut it = self.pcs.iter_mut();
        let Some(first) = it.next() else { return };
        let mut pcs: Vec<&mut PcQworm> = vec![first];
        pcs.extend(it);
        let mut k = 0usize;
        for p in &level.maps[m].placements {
            if p.flags & 0xff != 0x35 { continue; }
            let Some(pc) = pcs.get_mut(k) else { break };
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(map, object)| &level.maps[map].objects[object]) }
                else { level.maps[m].objects.get(p.index as usize) };
            match obj {
                Some(o) => {
                    // 0x133070:34-48 — the local sphere centre through the object matrix, the
                    // local radius × |obj+0xE8| (the placement's scale triple product here; the
                    // Tower PCs carry scale 1 [D placements extra]). The matrix is the created
                    // object's obj+0x90: CCO(0) → View_RotTransMatrix [M 0x1b07a8:12-13].
                    let mx = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
                    let s: f32 = p.scale().iter().map(|v| v.abs()).product();
                    pc.sph_centre = nf_formats::placement::transform(&mx, o.centre);
                    pc.radius = o.radius * s;
                }
                None => {
                    // 0x133070:23-31 — no model: obj+0x80 = obj+0xC0 (the matrix translation,
                    // the placement translation), obj+0x8C = 0.1.
                    pc.sph_centre = p.translation;
                    pc.radius = 0.1;
                }
            }
            k += 1;
        }
    }

    /// `PCQWorm_Update` 0x1aee20 + `Switch_Update` 0x1adea8, once per tick. `bullets` are this
    /// tick's player-bullet shots as (from, to, weapon) — the game reads the PC's class-5 contact
    /// list (a bullet with weapon 0x58 entering the object's box); the port's equivalent is the
    /// segment-vs-sphere test over each bullet's path [M 0x1aee20's 0x58 test]. `scripts` carries
    /// the level's ScriptPlayers so the switch's embedded one can be unpaused (laser-scripts).
    pub fn tick(&mut self, bullets: &[(V3, V3, u16)], ch: &mut Channels, frame: u32, scripts: &mut crate::nis::Scripts) {
        for pc in self.pcs.iter_mut() {
            match pc.state {
                0 => {
                    // State 0: a Q-Worm bullet (weapon 0x58) reaching the PC begins the install
                    // [M 0x1aee20: the class-5 toucher with ext+0x44 == 0x58] — only when param 3
                    // is set (0x1aee20:49-51). The contact box is the PC's glist; the port stands
                    // in the placement sphere r = 0.7 (the Q-WORM_PC_1 box's circumscribed radius,
                    // [D placements 0x35]).
                    if pc.armed {
                        let hit = bullets.iter().any(|&(a, b, w)| w == 0x58
                            && seg_sphere(a, b, pc.pos, 0.7));
                        if hit { pc.state = 1; }
                    }
                }
                1 => {
                    // State 1: the keycode message (param 0 keys; presentation) and the channel
                    // write, then state 2 [M 0x1aee20 LAB_001aefec]. The Tower PCs carry param 0
                    // = 0 → the write lands the tick after the hit.
                    if pc.ch != 0 { ch.set(pc.ch as u32, 1, frame); }
                    pc.state = 2;
                }
                _ => {}
            }
        }
        for s in self.switches.iter_mut() {
            // The stored value follows the channel; every change bumps the latch [M 0x1adea8:20-23].
            let cur = ch.v.get(s.ch as usize).copied().unwrap_or(0);
            if s.value != cur {
                s.value = cur;
                s.latch = s.latch.saturating_add(1);
                // The channel-mirror arm unpauses the embedded player [M 0x1adea8:23
                // `SP_UnPause(inst+0x0C)` = the +0x2c flag plus `SCRIPTINFO+0x9c = 0`,
                // SP_UnPause 0x1dca58:10-14] — the laser show resumes from its hold.
                if let Some(idx) = s.sp {
                    if let Some(p) = scripts.players.get_mut(idx) {
                        p.activated = true;
                        if let Some(sc) = p.info[p.cur].as_mut() { sc.delay = 0; }
                    }
                }
            }
            // The auto-return [M 0x1adea8's second block]: after param2 frames off the rest
            // value, flip the channel back. Its fire unpauses the player too [M 0x1adea8:29-30].
            if s.auto_delay != 0 && cur != s.rest {
                s.auto_ct += 1;
                if s.auto_ct > s.auto_delay {
                    let v = if cur != 0 { 0u8 } else { 1u8 };
                    ch.set(s.ch as u32, v, frame);
                    if let Some(idx) = s.sp {
                        if let Some(p) = scripts.players.get_mut(idx) {
                            p.activated = true;
                            if let Some(sc) = p.info[p.cur].as_mut() { sc.delay = 0; }
                        }
                    }
                    s.auto_ct = 0;
                }
            } else {
                s.auto_ct = 0;
            }
        }
    }

    /// The E-use dispatch (`Player_Activate` 0x1a5170's class walk): SPs (mask & 4, via
    /// `SP_Activate` 0x1dc338) and switches (class 0x39, `Switch_Activate` 0x1b02a8) within the
    /// use sphere (radius 1.0 at head + look [M 0x1a5170's `Collide_SphereIntersect(1.0, …)`]).
    /// `scripts` are the level's ScriptPlayers (`Drones::scripts`); `w` is the level data (for the
    /// entity sphere lookups); `body` is the activator object's `obj+0xC0` (the player object's
    /// world-matrix translation — `SP_Activate` 0x1dc338:51-53's facing-dot input; the RAY origin
    /// stays the HEAD, `Player_GetHeadPos` [M 0x1a5170:100]). Returns whether anything activated
    /// (the game's `bVar3` — it gates nothing else in the port).
    ///
    /// The SP reach: the game dispatches `SP_Activate` only on a hit object with
    /// `obj+0xE4 != 0 && class == 0` [M 0x1a5170:121-122] — a script ENTITY of that SP
    /// (`Script_EntityStart` 0x1d8d78:108-110 stores the owner handle). The SP body (class 0x1B)
    /// is inert to the pick — the class walk has no 0x1B case (0x1a5170:126-175). So each mask-4
    /// SP tests ONLY its live entity spheres (`Player::entity_use_targets`) [M]: the model-sphere
    /// and placement-matrix fallbacks the older tasks kept are removed (use-pick-sp-standins —
    /// nothing in 0x1a5170/0x1dc338 reads either; an SP whose script has created no entity is
    /// simply not use-activatable, which on the real levels never happens for the E-flagged SPs:
    /// their mode-64 preloads create the entities at level load, elevator-button.md).
    pub fn player_activate(
        &mut self, scripts: &mut crate::nis::Scripts, ch: &mut Channels, head: V3, look: V3, frame: u32,
        w: &dyn crate::nis::World, body: V3,
    ) -> bool {
        let p = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
        let near = |q: V3| (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2) <= 1.0;
        let in_sphere = |c: [f32; 3], r: f32| {
            let d = (p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2);
            d <= (1.0 + r) * (1.0 + r)
        };
        let mut used = false;
        // SP_Activate 0x1dc338: mask & 4, the gate channel, and the out-channel gate when
        // mask & 1; then `ext+0x2C = 1` — the SP's own update plays it and holds its out.
        for spi in 0..scripts.players.len() {
            {
                let sp = &mut scripts.players[spi];
                if sp.deleted || sp.mask & 4 == 0 { continue; }
                if sp.gate != 0 && !ch.on(sp.gate as u32) { continue; }
                if sp.mask & 1 != 0 && !ch.on(sp.out as u32) { continue; }
                // The entity targets — the ONLY pick targets [M 0x1a5170:121-122 +
                // 0x1d8d78:108-110 + 0x133070's posed spheres]. No model/placement fallback.
                let hit = sp.entity_use_targets(&scripts.objects, w)
                    .into_iter()
                    .any(|(c, r)| in_sphere(c, r));
                if hit {
                    sp.activated = true;
                    // SP_Activate 0x1dc338:84-86's second write: the current SCRIPTINFO's
                    // `+0x9C = 0` (the port's `delay`). Without it a script held by an event 3
                    // (`+0x9C = p0 − 1` [M 0x1d94c0:98-117]) never resumes: 0x0600059B's stream 1
                    // carries `Event{kind 3, p0 = 0}` at t=0 [D nis dump], so the preload leaves
                    // the hold latched and the activation must clear it for the FX to play.
                    if let Some(s) = sp.info[sp.cur].as_mut() { s.delay = 0; }
                    used = true;
                }
            }
        }
        // The switch-through-SP route [M Switch_Create 0x1adcc0:60-66 + SP_Activate 0x1dc338:44-53]:
        // Switch_Create builds each switch's embedded ScriptPlayer at the switch's own position
        // with the switch object as its LINK (record[0]), so a use-pick hit on that SP's class-0
        // entity dispatches `SP_Activate` — whose linked-type-0x39 arm runs the facing dot-check
        // and then `Switch_Activate` on the switch, RETURNING before any mask read or activation
        // tail. This is the only pick route to a switch in `Player_Activate` 0x1a5170 (its class
        // walk has no 0x39 case), so a scripted-position switch — 07000009's laser switch #71
        // sits at the world origin [D placement] while its show positions the art — is reached
        // through the SP's live entity targets only [M]; the model / placement-matrix fallbacks
        // are removed with the SP arm's (use-pick-sp-standins).
        for sidx in 0..self.switches.len() {
            let Some(spi) = self.switches[sidx].sp else { continue };
            let sp = &scripts.players[spi];
            if sp.deleted { continue; }
            // The entity targets, each with its facing basis and its obj+0xC0 [M 0x1dc338:50-61]:
            // `Mat_GetDir(entity+0x90)` dotted with `activator+0xC0 − entity+0xC0` — `< 0`
            // (behind the panel) refuses, before any sphere read. The dot positions are the
            // OBJECT translation rows (the player's `body`, the entity's matrix translation),
            // not the posed sphere centres and not the head.
            let hit = sp.entity_use_targets_dir(&scripts.objects, w).into_iter().any(|(c, r, dir, epos)| {
                let to_p = [body[0] - epos[0], body[1] - epos[1], body[2] - epos[2]];
                dir[0] * to_p[0] + dir[1] * to_p[1] + dir[2] * to_p[2] >= 0.0 && in_sphere(c, r)
            });
            if !hit { continue; }
            // `Switch_Activate` 0x1b02a8's gate chain — identical to the direct walk below
            // (the game's two routes share it). The value gate also debounces the double
            // dispatch: after the toggle the direct walk's value match fails.
            let s = &mut self.switches[sidx];
            if s.busy && s.latch != 0 { continue; }
            // The inhibit gate: usable while the gate channel is OFF [M 0x1b02a8:22].
            if s.gate != 0 && ch.on(s.gate as u32) { continue; }
            if s.ch == 0 { continue; } // [M 0x1b02a8:24]
            if s.ch == 0x61 { continue; } // [M 0x1b02a8:20-21 — a class-3 activator never toggles the alarm]
            // The script-idle gate [M 0x1b02a8:23 `SP_GetState(inst+0x0C) == 0`]: toggle
            // allowed while the embedded script holds (delay ≠ 0) or is not loaded.
            if let Some(idx) = s.sp {
                let busy_running = scripts.players.get(idx)
                    .and_then(|p| p.info[p.cur].as_ref())
                    .is_some_and(|sc| sc.delay == 0);
                if busy_running { continue; }
            }
            let cur = ch.v.get(s.ch as usize).copied().unwrap_or(0);
            if s.value != cur { continue; }
            let v = if cur != 0 { 0u8 } else { 1u8 };
            ch.set(s.ch as u32, v, frame);
            s.value = v;
            s.latch = s.latch.saturating_add(1);
            // The mirror arm's unpause lands here, where the game's next `Switch_Update`
            // mirror read does it [M 0x1adea8:20-24 `SP_UnPause`; SP_UnPause 0x1dca58:10-14].
            if let Some(idx) = s.sp {
                if let Some(p) = scripts.players.get_mut(idx) {
                    p.activated = true;
                    if let Some(sc) = p.info[p.cur].as_mut() { sc.delay = 0; }
                }
            }
            used = true;
        }
        // Switch_Activate 0x1b02a8: gate on, value matches the channel, idle; a class-3
        // activator never toggles ch 0x61 — the port's use path is always the player (class 3),
        // so the alarm channel is excluded here exactly as the game's check does.
        for s in self.switches.iter_mut() {
            if s.busy && s.latch != 0 { continue; }
            // The inhibit gate: usable while the gate channel is OFF [M 0x1b02a8:22].
            if s.gate != 0 && ch.on(s.gate as u32) { continue; }
            if s.ch == 0 { continue; } // [M 0x1b02a8:24 — a channel-less switch never activates]
            if s.ch == 0x61 { continue; }
            // The script-idle gate [M 0x1b02a8:23 `SP_GetState(inst+0x0C) == 0`, SP_GetState
            // 0x1dc900:13-20 = `SCRIPTINFO+0x9c == 0` → toggle allowed only while the embedded
            // script is NOT running its delay-0 update path — i.e. held (delay ≠ 0) or not yet
            // loaded (SP_GetState's null-SCRIPTINFO arm returns false = allowed)].
            if let Some(idx) = s.sp {
                let busy_running = scripts.players.get(idx)
                    .and_then(|p| p.info[p.cur].as_ref())
                    .is_some_and(|sc| sc.delay == 0);
                if busy_running { continue; }
            }
            let cur = ch.v.get(s.ch as usize).copied().unwrap_or(0);
            if s.value != cur { continue; }
            if near(s.pos) {
                let v = if cur != 0 { 0u8 } else { 1u8 };
                ch.set(s.ch as u32, v, frame);
                s.value = v;
                // The toggle's mirror mismatch bumps +0x18 next Update [M 0x1adea8:20-23]; the
                // port syncs the mirror here, so it bumps the latch at the same point — and
                // unpauses the embedded player here, where the game's next Switch_Update mirror
                // arm does it (0x1adea8:23 `SP_UnPause`; SP_UnPause 0x1dca58:10-14).
                s.latch = s.latch.saturating_add(1);
                if let Some(idx) = s.sp {
                    if let Some(p) = scripts.players.get_mut(idx) {
                        p.activated = true;
                        if let Some(sc) = p.info[p.cur].as_mut() { sc.delay = 0; }
                    }
                }
                used = true;
            }
        }
        // PCQWorm_Activate 0x1b0840: only when param 2 != 0 (the Tower PCs: 0 → E does nothing).
        // The pick is the game's sphere-vs-sphere [M 0x1a5170]: the view sphere (r 1.0) against the
        // PC's own world sphere (obj+0x80 = the posed local-sphere centre, obj+0x8C = the local
        // radius × |scale| — `set_pc_spheres`' Control_BuildWorldSph pose; before that refine the
        // object reads as a point at the placement translation, the create-time state).
        for pc in self.pcs.iter_mut() {
            if pc.e_usable == 0 || pc.state != 0 { continue; }
            if in_sphere(pc.sph_centre, pc.radius) {
                pc.state = 1;
                used = true;
            }
        }
        used
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chans(on: &[u8]) -> Channels {
        let mut c = Channels::default();
        for &k in on { c.set(k as u32, 1, 0); }
        c
    }

    fn placement(flags: u32, pos: [f32; 3], params: &[(u32, u32)]) -> nf_formats::placement::Placement {
        nf_formats::placement::Placement {
            index: 0, hashcode: 0, flags, translation: pos, rotation: [0.0; 3],
            params: params.to_vec(), extra: [0; 9],
        }
    }

    /// The Q-Worm PC: a weapon-0x58 bullet reaching it writes its channel one tick later
    /// [M 0x1aee20]; a normal bullet does nothing; E does nothing when param 2 = 0 [M 0x1b0840].
    #[test]
    fn pcqworm_installs_on_qworm_bullet_only() {
        let ps = [placement(0x35, [10.0, 1.0, 10.0], &[(0, 0), (1, 0x48), (2, 0), (3, 1)])];
        let mut scripts = crate::nis::Scripts::default();
        let mut scripts2 = crate::nis::Scripts::default();
        let mut u = UseObjects::create(&ps);
        let mut ch = chans(&[]);
        let bullets = [( [10.0, 1.0, 10.4], [10.0, 1.0, 12.0], 0x58u16 )];
        u.tick(&bullets, &mut ch, 1, &mut scripts);
        u.tick(&[], &mut ch, 2, &mut scripts);
        assert!(ch.on(0x48), "the Q-Worm bullet installs the virus (ch 0x48) [M 0x1aee20]");
        // Sister: a rifle bullet must not.
        let mut u2 = UseObjects::create(&ps);
        let mut ch2 = chans(&[]);
        let rifle = [( [10.0, 1.0, 10.4], [10.0, 1.0, 12.0], 2u16 )];
        u2.tick(&rifle, &mut ch2, 1, &mut scripts2);
        u2.tick(&[], &mut ch2, 2, &mut scripts2);
        assert!(!ch2.on(0x48), "a normal bullet does not install [M 0x1aee20's 0x58 test]");
        // E with param 2 = 0 does nothing either.
        let mut u3 = UseObjects::create(&ps);
        let mut ch3 = chans(&[]);
        u3.player_activate(&mut scripts, &mut ch3, [9.0, 1.0, 10.0], [1.0, 0.0, 0.0], 0, &TestWorld, [9.0, 1.0, 10.0]);
        assert!(!ch3.on(0x48) && u3.pcs[0].state == 0, "E cannot install when param 2 = 0 [M 0x1b0840]");
    }

    /// The switch: E toggles its channel when the gate allows and the value matches
    /// [M 0x1b02a8]. The alarm pulls (ch 0x7C/0x7D) are player-usable — that is how the alarm
    /// raises — while a switch whose channel IS 0x61 refuses the class-3 player activator.
    #[test]
    fn switch_toggles_channel_and_skips_alarm() {
        let ps = [
            placement(0x29, [0.0, 0.0, 0.0], &[(0, 9), (1, 0), (2, 0), (3, 0), (4, 1), (5, 0), (6, 0)]),
            placement(0x29, [5.0, 0.0, 0.0], &[(0, 0x7c), (1, 0), (2, 0), (3, 0), (4, 1), (5, 0), (6, 0)]),
            placement(0x29, [10.0, 0.0, 0.0], &[(0, 0x61), (1, 0), (2, 0), (3, 0), (4, 1), (5, 0), (6, 0)]),
        ];
        let mut u = UseObjects::create(&ps);
        let mut scripts = crate::nis::Scripts::default();
        let mut ch = chans(&[]);
        // E on the keycard lock (ch 9) — the direct walk keeps its placement-origin reach
        // (not ported here).
        assert!(u.player_activate(&mut scripts, &mut ch, [0.0, 1.6, 0.0], [0.0, -1.0, 0.0], 1, &TestWorld, [0.0, 0.0, 0.0]));
        assert!(ch.on(9), "the switch toggles ch 9 [M 0x1b02a8]");
        assert!(!ch.on(0x7c), "the alarm pull is out of reach here");
        // E at the alarm pull (ch 0x7C): the player CAN raise the alarm with it.
        assert!(u.player_activate(&mut scripts, &mut ch, [5.0, 1.6, 0.0], [0.0, -1.0, 0.0], 2, &TestWorld, [5.0, 0.0, 0.0]));
        assert!(ch.on(0x7c), "the alarm pull toggles ch 0x7C for the player [M 0x1b02a8]");
        // A switch whose channel IS 0x61 refuses the class-3 player [M 0x1b02a8's first gate].
        assert!(!u.player_activate(&mut scripts, &mut ch, [10.0, 1.6, 0.0], [0.0, -1.0, 0.0], 3, &TestWorld, [10.0, 0.0, 0.0]));
        assert!(!ch.on(0x61), "the player cannot toggle a ch-0x61 switch");
    }

    /// SP E-activation through the REAL target — the mode-64 preload's class-0 entity
    /// [M 0x1a5170:121-122 + 0x1d8d78:108-110, the elevator-buttons-09 mechanism]: mask & 4 SPs
    /// activate when the use sphere meets the entity's world sphere, gated/out-of-reach twins
    /// refuse, and the activated SP's own update holds its OUT channel [M 0x1dc338 + 0x1dbda8].
    #[test]
    fn sp_activation_gates_and_fires() {
        // Two mask-4 SPs, each with its preload entity: the first's is the panel at (5,0,0)
        // (TestWorld 0x11), the twin's the same art parked at (50,0,0) (0x12) — out of reach.
        let (p0, o0) = sp_with_entity(0x39, 0, 4, 0x11);
        let (p1, o1) = sp_with_entity(0x40, 0x50, 4, 0x12);
        let mut scripts = crate::nis::Scripts::default();
        scripts.players = vec![p0, p1];
        scripts.objects = o0;
        for (i, o) in o1.list.into_iter().enumerate() { scripts.objects.list.push(o); let _ = i; }
        let ps: [nf_formats::placement::Placement; 0] = [];
        let mut u = UseObjects::create(&ps);
        let mut ch = chans(&[]);
        // In front of the panel, looking at it.
        let dbg_targets = scripts.players[0].entity_use_targets(&scripts.objects, &TestWorld);
        eprintln!("DBG sp0 targets: {:?}", dbg_targets);
        eprintln!("DBG objects: {:?}", scripts.objects.list);
        eprintln!("DBG sp0 cur {} info0 {}", scripts.players[0].cur, scripts.players[0].info[0].is_some());
        assert!(u.player_activate(&mut scripts, &mut ch, [5.0, 0.3, 0.4], [0.0, 0.0, 1.0], 1, &TestWorld, [5.0, 0.0, 0.4]));
        assert!(scripts.players[0].activated, "the mask-4 SP activates through its entity sphere [M 0x1dc338]");
        assert!(!scripts.players[1].activated, "the twin's entity is out of reach anyway");
        // The gated twin, its own entity in reach, gate ch 0x50 OFF: refused [M 0x1dc338:72-81].
        let (p2, o2) = sp_with_entity(0x40, 0x50, 4, 0x11);
        let mut scripts2 = crate::nis::Scripts::default();
        scripts2.players = vec![p2];
        scripts2.objects = o2;
        let mut u2 = UseObjects::create(&ps);
        let mut ch2 = chans(&[]);
        u2.player_activate(&mut scripts2, &mut ch2, [5.0, 0.3, 0.4], [0.0, 0.0, 1.0], 1, &TestWorld, [5.0, 0.0, 0.4]);
        assert!(!scripts2.players[0].activated, "gate ch 0x50 off → no activation [M 0x1dc338]");
        // And with the gate ON: activates.
        let mut ch3 = chans(&[0x50]);
        u2.player_activate(&mut scripts2, &mut ch3, [5.0, 0.3, 0.4], [0.0, 0.0, 1.0], 1, &TestWorld, [5.0, 0.0, 0.4]);
        assert!(scripts2.players[0].activated, "gate on → activates");
    }

    /// The entity sphere is the ONLY SP pick target [M 0x1a5170:121-122 — the dispatch needs a
    /// class-0 obj+0xE4 hit; the class walk has no 0x1B case]. Fails on the older trees, whose
    /// model-sphere and placement-matrix stand-ins activated an SP with NO live entity.
    #[test]
    fn sp_without_an_entity_is_never_use_activatable() {
        // model 0x11 = the panel at (5,0,0) r 0.5 (TestWorld), but the SP's script created
        // nothing — the model art is NOT a pick target.
        let mut scripts = crate::nis::Scripts::default();
        scripts.players = vec![sp_with_model(0x39, 0, 4, 0x11)];
        let ps: [nf_formats::placement::Placement; 0] = [];
        let mut u = UseObjects::create(&ps);
        let mut ch = chans(&[]);
        assert!(!u.player_activate(&mut scripts, &mut ch, [5.0, 0.3, 0.4], [0.0, 0.0, 1.0], 1, &TestWorld, [5.0, 0.0, 0.4]),
            "the model-baked panel is not a use target without a class-0 entity [M 0x1a5170:121-122]");
        assert!(!scripts.players[0].activated);
        // Sister: an SP with no model at all is not activatable at its placement-matrix origin
        // either (the old `near(matrix origin)` stand-in activated it there).
        let mut scripts2 = crate::nis::Scripts::default();
        scripts2.players = vec![sp_with_model(0x39, 0, 4, 0)];
        let mut u2 = UseObjects::create(&ps);
        let mut ch2 = chans(&[]);
        assert!(!u2.player_activate(&mut scripts2, &mut ch2, [0.0, 0.5, -0.5], [0.0, 0.0, 1.0], 1, &TestWorld, [0.0, 0.0, -0.5]),
            "the placement matrix is not a use target [M 0x1a5170's hit-list walk]");
        assert!(!scripts2.players[0].activated);
        // Control: the SAME SP with the preload entity activates at the panel.
        let (p3, o3) = sp_with_entity(0x39, 0, 4, 0x11);
        let mut scripts3 = crate::nis::Scripts::default();
        scripts3.players = vec![p3];
        scripts3.objects = o3;
        assert!(u.player_activate(&mut scripts3, &mut ch, [5.0, 0.3, 0.4], [0.0, 0.0, 1.0], 1, &TestWorld, [5.0, 0.0, 0.4]),
            "the entity sphere reaches where the model/placement stand-ins used to");
    }

    /// The switch-through-SP route [M Switch_Create 0x1adcc0:60-66 + SP_Activate 0x1dc338:44-53]:
    /// a switch whose embedded ScriptPlayer carries the pick target (the preload's class-0
    /// entity here; 09's #71 has the placement at the world origin [D]) toggles through the SP
    /// hit — SP_Activate's facing dot (entity+0x90 dir vs activator+0xC0 − entity+0xC0 [M
    /// 0x1dc338:50-61]) then Switch_Activate's gate chain [M 0x1b02a8]. Sisters: behind the
    /// panel refuses; the inhibit gate blocks; ch 0x61 refuses.
    #[test]
    fn switch_embedded_sp_route_toggles_and_gates() {
        // The switch sits at the ORIGIN (like #71 [D]); its SP's preload entity is the panel at
        // (5,0,0) (TestWorld 0x11, entity matrix identity → obj+0xC0 = the origin).
        let (sp, objs) = sp_with_entity(0, 0, 1, 0x11);
        let mut u = UseObjects::default();
        u.switches.push(SwitchObj { pos: [0.0, 0.0, 0.0], ch: 0x37, gate: 0, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: Some(0) });
        let mut scripts = crate::nis::Scripts::default();
        scripts.players = vec![sp];
        scripts.objects = objs;
        let mut ch = chans(&[]);
        // E at the panel, body in FRONT of it (the entity's dir is +z: dot ≥ 0). The embedded
        // show is PARKED at delay −1 (the real laser shows' event kind 3, p0 = 0 write [M
        // 0x1d94c0:98-117; the 09 #71 / 0a #643 shows freeze two ticks in]) — SP_GetState
        // 0x1dc900:15 returns false, the switch-idle gate allows the toggle [M 0x1b02a8:23].
        if let Some(sc) = scripts.players[0].info[0].as_mut() { sc.delay = -1; }
        assert!(u.player_activate(&mut scripts, &mut ch, [5.0, 0.3, 0.9], [0.0, 0.0, -1.0], 1, &TestWorld, [5.0, 0.0, 0.9]),
            "the SP-routed hit toggles the switch");
        assert!(ch.on(0x37), "ch 0x37 toggles through the SP route [M 0x1dc338 → 0x1b02a8]");
        assert_eq!(u.switches[0].value, 1, "the stored value follows the toggle");
        assert!(scripts.players[0].activated, "the show's mirror unpause lands on the embedded player");
        // Sister 1: the toggle is per press [M 0x1b02a8's value-match + flip]. The first toggle's
        // mirror-unpause cleared the delay (the show resumed); the game's looping show re-parks
        // at its next event-3 hold — re-parked here to isolate the value gate from the idle gate.
        if let Some(sc) = scripts.players[0].info[0].as_mut() { sc.delay = -1; }
        assert!(u.player_activate(&mut scripts, &mut ch, [5.0, 0.3, 0.9], [0.0, 0.0, -1.0], 2, &TestWorld, [5.0, 0.0, 0.9]),
            "the second press re-toggles [M 0x1b02a8]");
        assert!(!ch.on(0x37) && u.switches[0].value == 0, "the toggle is per press");
        // Sister 2: BEHIND the panel the facing dot refuses before any sphere read
        // [M 0x1dc338:59-61] — the pick point is inside the entity's sphere here.
        let (sp4, objs4) = sp_with_entity(0, 0, 1, 0x11);
        let mut u4 = UseObjects::default();
        u4.switches.push(SwitchObj { pos: [0.0, 0.0, 0.0], ch: 0x37, gate: 0, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: Some(0) });
        let mut scripts4 = crate::nis::Scripts::default();
        scripts4.players = vec![sp4];
        scripts4.objects = objs4;
        if let Some(sc) = scripts4.players[0].info[0].as_mut() { sc.delay = -1; }
        let mut ch4 = chans(&[]);
        assert!(!u4.player_activate(&mut scripts4, &mut ch4, [5.0, 0.3, -0.4], [0.0, 0.0, -1.0], 1, &TestWorld, [5.0, 0.0, -0.4]),
            "behind the panel the facing dot refuses [M 0x1dc338:59-61]");
        assert!(!ch4.on(0x37), "no toggle from behind");
        // Sister 3: a RUNNING (unheld, delay 0) show keeps the switch busy — SP_GetState
        // 0x1dc900:15 returns true and Switch_Activate 0x1b02a8:23 refuses [M].
        let (sp5, objs5) = sp_with_entity(0, 0, 1, 0x11);
        let mut u5 = UseObjects::default();
        u5.switches.push(SwitchObj { pos: [0.0, 0.0, 0.0], ch: 0x37, gate: 0, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: Some(0) });
        let mut scripts5 = crate::nis::Scripts::default();
        scripts5.players = vec![sp5];
        scripts5.objects = objs5;
        let mut ch5 = chans(&[]);
        assert!(!u5.player_activate(&mut scripts5, &mut ch5, [5.0, 0.3, 0.9], [0.0, 0.0, -1.0], 1, &TestWorld, [5.0, 0.0, 0.9]),
            "the running show keeps the switch busy [M 0x1b02a8:23 + 0x1dc900:15]");
        assert!(!ch5.on(0x37), "no toggle while the show runs");
        // Sister 4: the inhibit gate ON blocks the SP route [M 0x1b02a8:22].
        let (sp2, objs2) = sp_with_entity(0, 0, 1, 0x11);
        let mut u2 = UseObjects::default();
        u2.switches.push(SwitchObj { pos: [0.0, 0.0, 0.0], ch: 0x37, gate: 7, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: Some(0) });
        let mut scripts2 = crate::nis::Scripts::default();
        scripts2.players = vec![sp2];
        scripts2.objects = objs2;
        let mut ch2 = chans(&[7]);
        if let Some(sc) = scripts2.players[0].info[0].as_mut() { sc.delay = -1; }
        assert!(!u2.player_activate(&mut scripts2, &mut ch2, [5.0, 0.3, 0.9], [0.0, 0.0, -1.0], 1, &TestWorld, [5.0, 0.0, 0.9]),
            "the gate ON refuses the SP-routed toggle");
        assert!(!ch2.on(0x37), "no channel write under the gate");
        // Sister 5: a ch-0x61 switch refuses through the SP route too [M 0x1b02a8:20-21].
        let (sp3, objs3) = sp_with_entity(0, 0, 1, 0x11);
        let mut u3 = UseObjects::default();
        u3.switches.push(SwitchObj { pos: [0.0, 0.0, 0.0], ch: 0x61, gate: 0, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: Some(0) });
        let mut scripts3 = crate::nis::Scripts::default();
        scripts3.players = vec![sp3];
        scripts3.objects = objs3;
        if let Some(sc) = scripts3.players[0].info[0].as_mut() { sc.delay = -1; }
        let mut ch3 = chans(&[]);
        assert!(!u3.player_activate(&mut scripts3, &mut ch3, [5.0, 0.3, 0.9], [0.0, 0.0, -1.0], 1, &TestWorld, [5.0, 0.0, 0.9]),
            "the alarm channel refuses the player activator");
        assert!(!ch3.on(0x61), "ch 0x61 never toggles from the player's E");
    }

    /// A Player literal (the SP test's constructor, field on its own line per the merge
    /// discipline). The `model` arg is gone with the removed SP-body use-target field.
    fn sp_with_model(out: u16, gate: u16, mask: u16, _model: u32) -> crate::nis::Player {
        crate::nis::Player {
            placement: 0, scripts: [1, 0], modes: [64, 0], stay: false, mask, out, gate,
            scale: 1.0, matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            cur: 0, state: 1, activated: false, countdown: 0, info: [None, None], deleted: false,
            live_matrix: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
            subs: Vec::new(),
            hit_points: 0.0,
            damage: 0.0,
            tick_damage: 0.0,
            // +0x36: nothing hides a placed/fusebox/useobj SP at create [M 0x1db9a8].
            hidden: false,
            bullet_filter: 0,
            bullet_hits: Vec::new(),
            tick_bullet_hits: Vec::new(),
        }
    }

    /// A mask-4-capable Player whose current script ran the op-7 Entity command for `model` once
    /// (the mode-64 preload's t=0 beat [M SP_LoadScript 0x1dbc08:41-56 + Script_EntityStart
    /// 0x1d8d78]) — the class-0 entity exists in the returned `Objects`, so
    /// `Player::entity_use_targets` resolves it through the World's glist sphere. The entity
    /// matrix is the create's identity [M nis.rs create], so the posed centre is the model's own
    /// (TestWorld 0x11: the panel at (5,0,0) r 0.5).
    fn sp_with_entity(out: u16, gate: u16, mask: u16, model: u32) -> (crate::nis::Player, crate::nis::Objects) {
        // The op-7 Entity command, no params: [0x07, hash LE, b0..b3 = 0/0/0/0x1f, n = 0]
        // (the disc's command words are little-endian [M nf_formats::nis Stream::decode's rd]).
        let mut cmd = vec![0x07];
        cmd.extend(model.to_le_bytes());
        cmd.extend([0, 0, 0, 0x1f, 0]);
        let nis = std::sync::Arc::new(nf_formats::nis::Nis {
            hashcode: 0x0600_059B, camera: false, loop_start: 0, length: 100,
            streams: vec![nf_formats::nis::Stream { first_key: 0, last_key: 0, commands: cmd }],
            keys: Vec::new(),
        });
        let mut script = crate::nis::Script::load(nis, [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]], 1.0);
        script.start(false);
        let mut objs = crate::nis::Objects::default();
        let mut ch = chans(&[]);
        let mut subs: Vec<Option<crate::nis::Script>> = Vec::new();
        let mut reqs = Vec::new();
        // Two ticks like the preload's play+update+stop beat [M 0x1dbc08:41-56] — the op-7
        // Entity runs inside them and the stream settles (kind 1, object set).
        for f in 0..2u32 {
            script.update(&mut subs, &Default::default(), &mut objs, &mut ch, &TestWorld, f, &mut reqs);
        }
        assert!(objs.list.iter().flatten().any(|o| matches!(o, crate::nis::Obj::Entity { .. })),
            "the helper's Entity op must create the class-0 entity");
        let mut p = sp_with_model(out, gate, mask, model);
        p.info[0] = Some(script);
        (p, objs)
    }

    /// docs/fields/switch.md +0x14: the inhibit gate is usable while the channel is OFF
    /// [M 0x1b02a8:22]. Fail-first: the old port required the channel ON.
    #[test]
    fn switch_inhibit_gate_blocks_only_while_on() {
        let mut u = UseObjects::default();
        u.switches.push(SwitchObj { pos: [0.0, 0.5, -0.5], ch: 9, gate: 7, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: None });
        // Gate channel OFF: the switch activates.
        let mut scripts = crate::nis::Scripts::default();
        let mut ch = Channels::default();
        let used = u.player_activate(&mut scripts, &mut ch, [0.0, 0.5, -0.5], [0.0, 0.0, 1.0], 1, &TestWorld, [0.0, 0.0, -0.5]);
        assert!(used && ch.on(9), "gate channel OFF → the switch activates");
        // Sister: the same switch with its gate channel ON never activates.
        let mut u2 = UseObjects::default();
        u2.switches.push(SwitchObj { pos: [0.0, 0.5, -0.5], ch: 9, gate: 7, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: None });
        let mut scripts2 = crate::nis::Scripts::default();
        let mut ch2 = chans(&[7]);
        let used2 = u2.player_activate(&mut scripts2, &mut ch2, [0.0, 0.5, -0.5], [0.0, 0.0, 1.0], 1, &TestWorld, [0.0, 0.0, -0.5]);
        assert!(!used2 && !ch2.on(9), "gate channel ON → blocked");
        // Sister 2: gate 0 always allows.
        let mut u3 = UseObjects::default();
        u3.switches.push(SwitchObj { pos: [0.0, 0.5, -0.5], ch: 9, gate: 0, rest: 0, busy: false,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: None });
        let mut scripts3 = crate::nis::Scripts::default();
        let mut ch3 = Channels::default();
        assert!(u3.player_activate(&mut scripts3, &mut ch3, [0.0, 0.5, -0.5], [0.0, 0.0, 1.0], 1, &TestWorld, [0.0, 0.0, -0.5]),
            "no gate → activates");
    }

    /// docs/fields/switch.md +0x18: with param 4 (busy) set the switch is a ONE-SHOT — its latch
    /// (+0x18) bumps on the first channel change and never resets [M 0x1adea8:22,29; gate
    /// 0x1b02a8:12-16]. Fail-first: the old port re-armed after a 60-tick cooldown.
    #[test]
    fn switch_busy_is_one_shot_never_rearms() {
        let mut u = UseObjects::default();
        u.switches.push(SwitchObj { pos: [0.0, 0.5, -0.5], ch: 9, gate: 0, rest: 0, busy: true,
            auto_delay: 0, value: 0, latch: 0, auto_ct: 0, placement: 0, sp: None });
        let mut scripts = crate::nis::Scripts::default();
        let mut ch = Channels::default();
        let head = [0.0, 0.5, -0.5]; let look = [0.0, 0.0, 1.0];
        assert!(u.player_activate(&mut scripts, &mut ch, head, look, 1, &TestWorld, [0.0, 0.0, -0.5]), "first use activates");
        // Let the tick see the channel change (the game's mirror test bumps +0x18).
        u.tick(&[], &mut ch, 2, &mut scripts);
        assert_eq!(u.switches[0].latch, 1, "the channel change bumped the latch");
        assert!(!u.player_activate(&mut scripts, &mut ch, head, look, 3, &TestWorld, [0.0, 0.0, -0.5]), "second use blocked");
        // After 61 ticks the game is still dead (the latch never resets).
        for f in 4..70 { u.tick(&[], &mut ch, f, &mut scripts); }
        assert!(!u.player_activate(&mut scripts, &mut ch, head, look, 70, &TestWorld, [0.0, 0.0, -0.5]),
            "a busy switch never re-arms");
    }
}
