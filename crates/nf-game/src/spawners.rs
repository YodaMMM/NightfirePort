//! Drone spawners (placement type 0xF1, `DroneSpawner_Create` 0x1378D0 / `DroneSpawner_Control` 0x137E68;
//! docs/systems/triggers.md "Spawners"): the drones of a group are templates that a spawner brings in.
//!
//! Param 7 (+0x14) is the **KILL channel** [L-verified 07000005, docs/fields/hitdata-spawner-ally.md
//! "Live result"]: `Control` latches `ch[p7]` into the byte at +0x20 (0x137e68:50-52) and the latched
//! path skips the spawn machinery entirely and runs the cleanup block (0x137e68:235-288 — the disasm
//! 0x137fbc-0x1380b4): spawned drones are removed (`obj+0xFE |= 1`), the slots are zeroed, and the
//! spawner disables itself — with a level split: on levels outside {0x07000011..0x07000013, 0x07000016,
//! 0x0700004A} every spawned drone is removed; on those levels only drones whose `obj+0xFC` byte (the
//! draw counter — 0 = not drawn recently) is 0 are removed and the spawner waits for the rest. While a
//! script has `Drone_EnableAll(0)` active (`NPCGlobals == 0`) `Control` returns at its first line
//! (0x137e68:21-23), so the latch only fires once the script frees the NPCs — measured live: ch 0x43's
//! wall-hug script held the latch off until `Script_Free`.

use crate::triggers::Channels;

#[derive(Clone, Debug)]
pub struct Spawner {
    pub placement: usize,
    /// 2: spawn while alive < max; 3: only when none is alive.
    pub mode: u32,
    pub group: u32,
    pub max_alive: usize,
    pub waves: u32,
    pub start: u32,
    pub stop: u32,
    pub done: u32,
    pub min_dist: f32,
    /// Drone indices of the group's templates and which of them were used this epoch.
    pub templates: Vec<usize>,
    pub used: Vec<bool>,
    /// Drones it spawned (indices) — the game's live-slot array [max_alive] [M 0x137aa0:46-51].
    pub spawned: Vec<usize>,
    pub active: bool,
    pub immediate: bool,
    /// +0x1C, the spawn latch: the mode arm decides it and the wave rollover alone clears it
    /// [M 0x137e68:145-172, :176-210]. Mode 2 re-decides it every tick (alive < max); mode 3 only
    /// sets it (the initial fill, or nobody alive), so one empty moment opens a whole wave: the
    /// latch stays up while the wave's first drones stand, until `this_wave` reaches `max_alive`.
    pub spawn_on: bool,
    /// p7 (+0x14): the **kill channel** [L-verified; M 0x137e68:50-52]. `Control` latches `ch[p7]` into
    /// +0x20; the latched path runs the cleanup block and never spawns again. (fix-actors' "ARM channel"
    /// reading was refuted by the live measurement.)
    pub kill: u32,
    /// +0x20, the p7 latch [M 0x137e68:50-52; L: +0x20 → 1 only after the ch-0x43 script freed the NPCs].
    pub kill_latch: bool,
    pub wave: u32,
    pub this_wave: usize,
    pub gone: bool,
    /// The mode-3 re-arm stinger request (the `Drone_SfxInfo` INDEX 0xD6/0xD7), drained by the
    /// caller after `update` [M 0x137e68:191-200]. Set every tick the branch runs — the
    /// `Rand_Rand(2)` draw is consumed even when the entry's repeat delay then suppresses the
    /// sound — and it can fire on a tick whose spawn attempt is screened out, so it cannot ride
    /// `Action`.
    pub sfx_request: Option<u32>,
}

/// What a spawner asks of the drone list this frame.
#[derive(Clone, Debug, PartialEq)]
pub enum Action { None, Spawn(usize), Remove(Vec<usize>) }

impl Spawner {
    /// From a placement (`p0` > 1 required, `p1` group non-zero — `Control`'s early-outs at
    /// 0x137e68:44-46/38-40) and the indices of the drones in its group.
    pub fn new(placement: usize, param: impl Fn(u32) -> u32, templates: Vec<usize>, ch: &Channels) -> Option<Spawner> {
        if param(0) <= 1 { return None; }
        if param(1) == 0 { return None; }
        let n = templates.len();
        let max = param(2) as usize;
        let start = param(4);
        // +0x1F "no stealth checks": Init sets it when (p4 = 0 or on) and (p5 = 0 or off) [M 0x137aa0:33-45];
        // cleared at wave rollover [M 0x137e68:148]. GAPS G-025's "mode-3 wave-delay byte" label is a
        // misread — it is the initial-fill flag: while set it overrides mode 3's nobody-alive rule
        // (0x137e68:184) and skips `SpawnDrone`'s min-distance + visibility screens (0x137cb0:30/:43).
        // There is no delay value anywhere in the spawner family; the gap between mode-3 waves is
        // purely the alive-scan == 0 wait.
        let immediate = (start == 0 || ch.on(start)) && !ch.on(param(5));
        // +0x18: `cvt.s.w` of p8, 0 → 10.0 [M 0x1379c0-0x137a1c] — a number on the disc, not float bits.
        let min = param(8) as i32 as f32;
        Some(Spawner {
            placement, mode: param(0), group: param(1), max_alive: if max == 0 || max > n { n } else { max },
            waves: if param(3) == 0 { 32000 } else { param(3) }, start, stop: param(5), done: param(6),
            kill: param(7), kill_latch: false,
            min_dist: if min == 0.0 { 10.0 } else { min },
            used: vec![false; n], templates, spawned: Vec::new(), active: start == 0, immediate, spawn_on: false, wave: 0, this_wave: 0,
            // No template on this difficulty: `DroneSpawner_Init` deletes the spawner [M 0x137aa0:16-19].
            gone: n == 0,
            sfx_request: None,
        })
    }

    /// One `DroneSpawner_Control` update. `level` is the current level id (the game's `GameState._12_4_`,
    /// read as DAT_002a3774 at 0x137fc4 [M disasm]); `alive(i)`: drone i is in a live slot; `drawn(i)`:
    /// its `obj+0xFC` draw counter is non-zero (the port's per-tick visibility stand-in); `dist_ok(i)`:
    /// template i's spot is at least `min_dist` from the player (a PRE-selection usability test —
    /// the candidate loop only decrements the draw for templates that pass it, 0x137cb0:30);
    /// `visible(i)`: the spot is visible to Bond (a POST-selection abort, 0x137cb0:43 — the attempt
    /// fails and the next empty slot tries); `free(i)`: the port can give template i's new guard a
    /// body (the game's record outlives its guards; the port's draw pool is finite); `rand(n)`: `Rand_Rand`.
    pub fn update(&mut self, ch: &mut Channels, frame: u32, level: u32, alive: impl Fn(usize) -> bool,
                  drawn: impl Fn(usize) -> bool, dist_ok: impl Fn(usize, f32) -> bool,
                  visible: impl Fn(usize) -> bool, free: impl Fn(usize) -> bool,
                  rand: &mut impl FnMut(u32) -> u32) -> Action {
        if self.gone { return Action::None; }
        // The p7 kill latch [M 0x137e68:50-52; L-verified 07000005: +0x20 → 1 once the ch-0x43 script
        // freed the NPCs — the early return below is what held it off while the script ran].
        if self.kill != 0 && ch.on(self.kill) { self.kill_latch = true; }
        if !self.active && !self.kill_latch {
            // The trigger channel (+0x0E = param 4) is consumed when it fires [M: `+0x0E = 0, +0x1E = 1`];
            // the wait is skipped entirely once the kill latch is set [M: the p4 block only runs when
            // +0x20 is clear, 0x137e68:53-64].
            if self.start != 0 && ch.on(self.start) { self.active = true; self.start = 0; }
            else { return Action::None; }
        }
        if self.stop != 0 && ch.on(self.stop) {
            self.active = false; // un-trigger [M 0x137e68: `+0x1E = 0`]
            if self.kill_latch { return Action::None; } // latched: pauses this tick [M :79-81]
            self.gone = true; return Action::None;      // un-latched: deletes itself [M :82]
        }
        if self.kill_latch {
            // The cleanup block [M 0x137e68:235-288; disasm 0x137fbc-0x1380b4; L-verified 07000005:
            // both gate guards removed, the slots zeroed, +0x1E = 0, obj+0xFE |= 1].
            // The level split (the disasm's `s7`, set through the always-executed delay slot at
            // 0x137fe8): kill-all for level < 0x07000011, 0x07000014/0x07000015, and > 0x07000016
            // except 0x0700004A; the obj+0xFC-gated keep for 0x07000011..0x07000013, 0x07000016 and
            // 0x0700004A.
            let kill_all = level < 0x0700_0011
                || (0x0700_0014..=0x0700_0015).contains(&level)
                || (level > 0x0700_0016 && level != 0x0700_004a);
            let mut removed = Vec::new();
            let mut kept = 0usize;
            for &i in &self.spawned {
                if !alive(i) { continue; } // a dead drone's slot is already cleared [M DroneSpawner_DroneDelete 0x13a3b8 / the alive scan]
                if kill_all || !drawn(i) {
                    // obj+0xFC == 0 (not drawn recently) → removed [M `lbu $v0, 0xfc($s0)` 0x13805c;
                    // on 0x07000016 the removal first raises the drone's death channel [M jal 0x13e660
                    // at 0x138074 — the caller writes it, DroneFunc_SetDeathChannel 0x13e660]].
                    removed.push(i);
                } else {
                    kept += 1; // the kept count [M 0x138098: `andi $s6, $v0` with v0 = s6+1]
                }
            }
            self.spawned.retain(|i| !removed.contains(i)); // the slots zeroed [M `sw $zero, ($v1)` 0x138094]
            if kill_all || kept == 0 {
                // every spawned drone gone → the spawner deletes itself [M 0x1380b4-ish: `fe |= 1`,
                // `+0x1E = 0`]; with keep-gated survivors it returns and lingers [M :281-283].
                self.gone = true;
                self.active = false;
            }
            return Action::Remove(removed);
        }
        // The alive scan zeroes the slot of a dead or removed guard [M 0x137e68:96-124].
        self.spawned.retain(|&i| alive(i));
        let living = self.spawned.len();
        if self.wave >= self.waves {
            // All waves spawned: `if ((ushort)puVar1[3] <= (ushort)puVar1[0xb])` — max waves (+0x0C)
            // vs waves completed (+0x2C), a pure counter compare with NO template-exhaustion term
            // [M 0x137e68:126-143; the port's old `|| used.all()` fired one tick before the game's
            // rollover and killed single-template multi-wave spawners — 07000012 placement 647
            // (mode 3, max 2, waves 5, 1 template) died after wave 1 instead of respawning]. The
            // done write waits for the spawned drones to die when the done channel (p6) OR the kill
            // channel (p7) is set — the game's `p6 == 0 && p7 == 0` immediate disable [M :126-143].
            if (self.done != 0 || self.kill != 0) && living > 0 { return Action::None; }
            if self.done != 0 { ch.set(self.done, 1, frame); }
            self.gone = true;
            self.active = false; // +0x1E = 0, +0x1D = 0 [M 0x137e68:140-142]
            return Action::None;
        }
        if self.this_wave >= self.max_alive {
            // Wave rollover [M 0x137e68:145-172]: the epoch counts reset (+0x28 = 0, +0x2C++), +0x1F
            // cleared and every template record's spawned count zeroed — the same templates can spawn
            // again next wave.
            self.wave += 1;
            self.this_wave = 0;
            self.immediate = false;
            self.spawn_on = false;
            for u in self.used.iter_mut() { *u = false; }
            return Action::None;
        }
        // The mode-3 arm's re-arm stinger [M 0x137e68:191-200, INSIDE the mode arm — after the
        // rollover and waves-exhausted early-outs, which return without it]: on the power-station
        // pair only, while the last wave is entirely dead and at least one wave completed, the
        // branch runs EVERY tick — `Rand_Rand(2)` picks Drone_SfxInfo[0xD6] or [0xD7] (sfx
        // 0x549/0x526) and `NDrone2_PlaySFX(1.0, NULL, id, NULL, enforceDelay=1)` plays it through
        // the entry's 600-frame repeat delay. The draw sits between the alive scan and the spawn
        // attempt and is consumed even when the delay then suppresses the sound. The first-ever
        // wave is silent (waves_completed == 0); the initial fill never reaches it (+0x1F clears at
        // the first rollover, so waves_completed > 0 implies the fill is over).
        if self.mode == 3 && living == 0 && self.wave > 0
            && (level == 0x0700_000c || level == 0x0700_000d) {
            self.sfx_request = Some(rand(2) + 0xd6);
        }
        // The mode arm [M 0x137e68:176-210]: mode 2 re-decides the +0x1C latch every tick; mode 3
        // sets it for the initial fill and whenever nobody is alive and otherwise leaves it as it
        // stands, so a wave fills up to `max_alive` (one drone a tick) before the rollover drops
        // the latch and the spawner waits for the wave to die. Other modes never touch it.
        match self.mode {
            2 => self.spawn_on = living < self.max_alive,
            3 => if self.immediate || living == 0 { self.spawn_on = true; },
            _ => {}
        }
        if !self.spawn_on { return Action::None; }
        // The slot walk [M 0x137e68:212-232]: every empty slot of the +0xA-long array calls
        // `DroneSpawner_SpawnDrone` until one succeeds; a failure moves on to the next slot in the
        // same tick (the `DAT_00299092` line-of-sight counter only moves in the sight code, and
        // `Drone_PositionVisible` does not touch it). Each attempt is its own draw.
        for _ in 0..self.max_alive.saturating_sub(living) {
            // Selection [M 0x137cb0:21-43]: one `Rand_Rand(+0x2E)` — the group's TEMPLATE COUNT,
            // set once by `DroneSpawner_Init` 0x137aa0:32 — walked down once per usable template.
            // Usable = unused this epoch AND, with screening on (+0x1F == 0), the spot at least
            // `min_dist` (block+0x18) from the player; a too-close spot is skipped without
            // consuming the draw. A draw past the last usable template falls off the walk and the
            // attempt fails.
            let mut left = rand(self.templates.len() as u32) as i32;
            let mut pick = None;
            for k in 0..self.templates.len() {
                if !self.immediate && !dist_ok(self.templates[k], self.min_dist) { continue; }
                if self.used[k] { continue; }
                left -= 1;
                if left == -1 { pick = Some(k); break; }
            }
            let Some(k) = pick else { continue };
            let i = self.templates[k];
            // The POST-selection abort [M 0x137cb0:43]: with screening on, a spot visible to Bond
            // fails the attempt (returns 0) — the same spot can win a later draw.
            if !self.immediate && visible(i) { continue; }
            // The game's spawn is a new object, so a corpse of the same template lies on untouched;
            // the port adds a guard for a template still in use, while it has a body to draw.
            if !free(i) { continue; }
            self.used[k] = true;
            if !self.spawned.contains(&i) { self.spawned.push(i); } // one record per template
            self.this_wave += 1;
            return Action::Spawn(i);
        }
        Action::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawner(n: usize) -> Spawner {
        // mode 2, group 1, no start/stop channels → active and immediate (+0x1F set, the
        // no-screening arm [M 0x137aa0:33-45]).
        Spawner::new(0, |p| if p == 0 { 2 } else if p == 1 { 1 } else { 0 }, (0..n).collect(),
            &Channels::default()).unwrap()
    }

    /// [M 0x137cb0:21 + 0x137aa0:31-32] the pick draw is `Rand_Rand` over the WHOLE template
    /// list — `+0x2E` and `+0x8` are both the template count `DroneSpawner_Init` writes from the
    /// census table — and only USABLE templates (unused this epoch; plus the distance screen when
    /// +0x1F is clear) decrement the counter (:41). A draw whose rank passes the last usable
    /// template spawns NOTHING this call (:88-96 fall through the loop → `return 0`) — `Control`
    /// retries next tick with a fresh draw. The old port drew `rand(usable.len())` and always
    /// spawned, so waves arrived at the wrong rate.
    #[test]
    fn the_pick_draws_over_all_templates_and_a_missed_rank_spawns_nothing() {
        let mut s = spawner(4);
        s.used[0] = true;
        s.used[1] = true; // 2 of 4 templates usable
        let mut moduli: Vec<u32> = Vec::new();
        let a = s.update(&mut Channels::default(), 0, 0, |_| false, |_| false, |_, _| true, |_| false, |_| true,
            &mut |n| { moduli.push(n); 3 });
        assert_eq!(moduli, vec![4; 4], "each empty slot makes its own attempt, every draw over the whole template list, not the usable count [M 0x137cb0:21, 0x137e68:212-232]");
        assert_eq!(a, Action::None, "rank 3 of 2 usable → nothing spawns this call [M 0x137cb0:88-96]");
    }

    /// The draw's rank indexes the USABLE walk: rank r spawns the (r+1)-th usable template
    /// [M 0x137cb0:41-56 — each usable record decrements, the −1 arm creates].
    #[test]
    fn the_draw_rank_indexes_the_usable_walk() {
        let mut s = spawner(4);
        s.used[0] = true;
        s.used[1] = true; // usable = templates 2, 3
        let mut moduli: Vec<u32> = Vec::new();
        let a = s.update(&mut Channels::default(), 0, 0, |_| false, |_| false, |_, _| true, |_| false, |_| true,
            &mut |n| { moduli.push(n); 1 });
        assert_eq!(moduli, vec![4], "the draw modulus is the whole template list [M 0x137cb0:21]");
        assert_eq!(a, Action::Spawn(3), "rank 1 of usable [2, 3] → the second usable template [M 0x137cb0:41-56]");
        assert!(s.used[3], "the picked template is used this epoch [M :82]");
    }

    /// A rank inside the usable count spawns even with used templates before it — the used ones
    /// are walked past without consuming the rank [M :41-56].
    #[test]
    fn a_rank_zero_spawns_the_first_usable() {
        let mut s = spawner(4);
        s.used[0] = true;
        let a = s.update(&mut Channels::default(), 0, 0, |_| false, |_| false, |_, _| true, |_| false, |_| true,
            &mut |_| 0);
        assert_eq!(a, Action::Spawn(1), "rank 0 → the first usable template [M 0x137cb0:41-56]");
    }
}
