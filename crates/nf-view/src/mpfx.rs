//! mp-remote-fx: what this machine SEES and HEARS of the other players and the bots in an MP
//! match — muzzle flash + muzzle light, the bullet trail, the world impact (decal / gas / debris /
//! emitter + the surface's impact sound), the victim's flesh puff, the shot / footstep / jump /
//! death sounds positioned at the body, the local player's hit feedback (the red direction glow
//! + `0x412`), and the death body. docs/systems/mp-remote-fx.md.
//!
//! The combat code (`Sim::step_net_combat` for the humans, `Sim::step_mp_bots` for the bots)
//! records each resolved shot as an [`MpShot`] event — the same events on every machine, since
//! both run in the shared lockstep tick. [`Sim::flush_mp_fx`] turns them into effects at the end
//! of the tick. Everything here is DRAW / AUDIO state: the effect pools (`drones.effects`,
//! `drones.gas`, `drones.fx`, `drones.lights`, `drones.audio`, `drones.health.flash`/`dir_bits`)
//! and `Sim::mp_fx` are outside `Sim::net_state_hash`, and no shared-stream (`drones.rng`) draw
//! happens: each shot's effect draws come from a generator seeded by (frame, shooter, shot index),
//! so every machine draws the same debris for the same shot without touching the lockstep stream
//! [S port design — the PS2 had no online play]. The local slot's own shots are skipped: its own
//! fire pipeline (`step_bond`) already draws and plays them on this machine.

use crate::scene::Sim;
use nf_game::drone::Rng;

type V3 = [f32; 3];

/// What one MP shot hit (the nearest contact the combat resolution chose).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MpShotHit {
    /// Nothing within range.
    Miss,
    /// The world: the surface normal and `HITDATA+0x50` material byte at `MpShot::end`.
    World { normal: V3, material: u8 },
    /// A human slot's body.
    Human(usize),
    /// A bot (its MP slot).
    Bot(usize),
    /// The Demolition/Protection site model.
    Site,
}

/// One resolved shot, recorded by the combat code for the effects (never read back by the sim).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpShot {
    pub shooter: usize,
    /// The shooter is a bot (a drone in the game — `NDrone2_DSTATE_BotAttackFire` 0x16f5e8 fires
    /// through `DroneWeap_Fire` [M]) rather than a human player.
    pub bot: bool,
    pub weapon: u8,
    /// The round's origin (the human's eye / the bot's muzzle-height eye) and unit direction.
    pub from: V3,
    pub dir: V3,
    /// Where the round stopped (the contact, or the range end on a miss).
    pub end: V3,
    pub hit: MpShotHit,
    /// The raw damage dealt (`weapon_data+0x0C`).
    pub damage: f32,
}

/// One cosmetic bullet trail in flight (`GUN_BULLET_TRAIL` 0x02000123).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MpTracer {
    pub pos: V3,
    pub dir: V3,
    /// Units per tick (`weapon_data+0x20`).
    pub speed: f32,
    /// Distance left to the contact.
    pub left: f32,
}

/// Trail draw slots (the newest first).
pub const MP_TRACER_SLOTS: usize = 24;

/// The per-machine draw/audio state of the remote effects. Not hashed.
#[derive(Clone, Debug)]
pub struct MpFx {
    /// This tick's shots (pushed by the combat code, drained by `flush_mp_fx`).
    pub events: Vec<MpShot>,
    /// Sounds the remote bodies' movement core queued this tick (`Bond::sfx_out`: the jump
    /// launch / head bump [M `Player_HandleJump` 0x1916b0]), played at the flush.
    pub body_sounds: Vec<(u32, V3)>,
    /// Per slot, the frame of its latest shot (the muzzle flash lives 2 ticks).
    pub last_shot: [Option<u32>; nf_net::MAX_AGENTS],
    /// The trails in flight.
    pub tracers: Vec<MpTracer>,
    /// Per slot, the footstep distance (`BL+0x924`) and the last body position.
    pub step: [f32; nf_net::MAX_AGENTS],
    pub prev_pos: [Option<V3>; nf_net::MAX_AGENTS],
    /// Per slot, dead at the last flush (the death edge plays the death sound).
    pub was_dead: [bool; nf_net::MAX_AGENTS],
    /// Fire-script sounds still due: (tick, slot, sound id).
    pub due: Vec<(u32, usize, u32)>,
    /// Every sound this module started (frame, id, position, slot) — the tests' and the log's view.
    pub started: Vec<(u32, u32, V3, usize)>,
    /// mp-bot-voices: the bots' bark handle per slot — the game's `BOT_vars+0x73c` voice ref
    /// (`Sound_Stop_Ref` 0x1de370 stops it before the next bark [M 0x1266c8-0x1266d0]; the game
    /// re-stamps it with the new play [M 0x1266fc]). The `Sound_Playing_Ref` 0x1de3a0 gate that
    /// reads it is NOT evaluated — see `Sim::mp_bot_bark`'s doc. Cleared on the bot's respawn
    /// (a fresh drone object, a fresh +0x73c).
    pub bot_bark: [Option<u32>; nf_net::MAX_AGENTS],
    /// Off: the combat still records, the flush only clears (the hash-equality tests).
    pub disabled: bool,
}

impl Default for MpFx {
    fn default() -> Self {
        MpFx {
            events: Vec::new(), body_sounds: Vec::new(), last_shot: [None; nf_net::MAX_AGENTS], tracers: Vec::new(),
            step: [0.0; nf_net::MAX_AGENTS], prev_pos: [None; nf_net::MAX_AGENTS], was_dead: [false; nf_net::MAX_AGENTS],
            due: Vec::new(), started: Vec::new(), bot_bark: [None; nf_net::MAX_AGENTS], disabled: false,
        }
    }
}

fn log_on() -> bool { std::env::var_os("NF_MPFX_LOG").is_some() }

fn rec_u32(r: &[u8], o: usize) -> u32 { u32::from_le_bytes(r[o..o + 4].try_into().unwrap()) }
fn rec_f32(r: &[u8], o: usize) -> f32 { f32::from_le_bytes(r[o..o + 4].try_into().unwrap()) }

impl Sim {
    /// The body of a dead MP slot, posed in the death script until the respawn: (feet position,
    /// yaw, ticks since the death). `Player_CheckForDeath` 0x190810:89-93 switches the dying
    /// player to substate 0xd (0xe in a vehicle) whose body script is `DEATH_SCRIPT`
    /// [M 0x190630 cases 0xd/0xe], and nothing moves the body until `MP_ReSpawn` 0x185368
    /// re-stands it [M]. The bots (drones in the game) die through the drone death states — the
    /// port poses their MP body in the same player death script [S]. None for the local slot (its
    /// own view runs the first-person death sequence) or a living / empty slot.
    pub fn mp_body_death(&self, slot: usize) -> Option<(V3, f32, u32)> {
        let m = self.mp.as_ref()?;
        let humans = m.settings.human_count as usize;
        if slot < humans {
            if slot >= nf_net::MAX_AGENTS || slot == self.mp_local_slot { return None; }
            let t = self.mp_dead_ticks[slot];
            if t == 0 { return None; }
            let b = self.mp_bonds[slot].as_ref()?;
            Some((b.pos, b.yaw, t))
        } else {
            let bot = m.bots.iter().find(|b| b.slot == slot)?;
            if !bot.dead { return None; }
            let p = bot.brain.pos;
            // Both kill paths stamp `respawn_at = frame + 180` at the death.
            let t = self.drones.frame.saturating_sub(bot.respawn_at.saturating_sub(180)) + 1;
            Some(([p[0], p[1] - crate::scene::BOT_EYE, p[2]], bot.brain.yaw, t))
        }
    }

    /// The trail draw slot `k`'s matrix (local +Z along the flight, scaled one tick of travel —
    /// `tracer_matrix`'s convention), None when the slot is free.
    pub fn mp_tracer(&self, k: usize) -> Option<&MpTracer> {
        let n = self.mp_fx.tracers.len();
        if k >= n { return None; }
        self.mp_fx.tracers.get(n - 1 - k)
    }

    fn mpfx_play3d(&mut self, id: u32, pos: V3, slot: usize) -> u32 {
        if id == 0 || id & 0xfffff == 0xffff { return 0; }
        let h = self.drones.audio.play3d(id, pos, None);
        let frame = self.drones.frame;
        self.mp_fx.started.push((frame, id, pos, slot));
        if self.mp_fx.started.len() > 4096 { self.mp_fx.started.drain(..2048); }
        if log_on() {
            let l = self.bond.head();
            let d = ((pos[0] - l[0]).powi(2) + (pos[1] - l[1]).powi(2) + (pos[2] - l[2]).powi(2)).sqrt();
            eprintln!("[mpfx] f{frame} sound {id:#x} at ({:.2}, {:.2}, {:.2}) slot {slot} dist {d:.1}", pos[0], pos[1], pos[2]);
        }
        h
    }

    /// mp-bot-voices: the MP bots' bark — `BOT_soundEffect` 0x126618's production caller. The
    /// trigger's pool comes from `BotBrain::bot_sound_effect` (trigger 1 → 0x154/0x157 + Rand(3)
    /// [M disasm 0x1266a0-0x1266b0]; 0x1F → 0x14D/0x152 [M 0x126680-0x12669c]); the one
    /// `Rand_Rand(n)` draw is on the shared match stream, taken exactly when the game's arm is
    /// reached [M 0x1266b4], so every machine draws identically and the state hash stays equal.
    /// The line then runs `NDrone2_PlaySFX`'s row gates via `Drones::foley_entry` (the bot bark
    /// passes param_5 = 1 [M 0x1266f0]): the 30-frame post-load gate + the row's repeatDelay —
    /// all ten bot rows carry repeat 0 [D ELF `Drone_SfxInfo` 0x2812d0 rows 0x14d..0x159], so the
    /// delay arm is vacuous here and the post-load gate stands. Delivery: the previous bark's
    /// handle stops (`Sound_Stop_Ref` [M 0x1266c8-0x1266d0]) and the row's sfx plays at the bot
    /// (the game's `Sound_Play3D(piVar6[4], …)` inside PlaySFX [M 0x141a50]; the port's MP bots
    /// are not drone objects, so the line rides the presentation channel like the bots' shot
    /// sounds — the drones' hearing list is an SP-drone consumer).
    ///
    /// NOT carried: `Sound_Playing_Ref` 0x1de3a0's "previous bark still playing" gate on the
    /// trigger-0x1F arm [M 0x126668-0x126678]. It reads the sound pool's node (ref word +0x38 vs
    /// `Drone+0x108`, state byte +0x50) — the pool's contents are per-machine presentation (the
    /// local slot's own 0x412 / 0x88 sounds play on one machine only), so a gate on it would make
    /// the shared stream's draw count machine-dependent and desync the match. The port stops the
    /// previous handle and always plays (the same deviation `Drones::bot_handle_pain` carries for
    /// SP), rate-limited only by the row gates above.
    ///
    /// `alt` (the game's `Drone+0x15` byte) is false: the MP bot records carry no sex byte and the
    /// port does not map the MP character skins to one — the male pools stand in. [S] unverified;
    /// the alt arms stay pinned only by `bot_sound_effect`'s unit tests, not by the live path.
    ///
    /// Runs outside `self` so the damage sites can call it while holding the match borrow: the
    /// draw + the row stamp are sim state and happen even when `mp_fx.disabled` (the
    /// hash-equality runs), only the delivery is skipped.
    pub fn mp_bot_bark(
        drones: &mut nf_game::drone::Drones, mp_fx: &mut MpFx,
        voices: Option<&[nf_formats::sfx::VoiceLine]>, bots: &[nf_game::mp::MpBot],
        slot: usize, trigger: u32,
    ) {
        let Some(b) = bots.iter().find(|b| b.slot == slot) else { return };
        let alt = false; // [S] unverified — see the doc
        let Some((base, n)) = b.brain.bot_sound_effect(trigger, alt) else { return };
        let pos = b.brain.pos;
        let id = base as usize + drones.rng.rand(n) as usize; // [M 0x1266b4] the shared stream
        let frame = drones.frame;
        let Some(l) = nf_game::drone::Drones::foley_entry(voices, &mut drones.voice_last, id, frame) else { return };
        if mp_fx.disabled { return; }
        if slot < mp_fx.bot_bark.len() {
            if let Some(h) = mp_fx.bot_bark[slot] { drones.audio.stop(h); } // [M 0x1266c8]
        }
        let h = drones.audio.play3d(l.sfx, pos, None);
        mp_fx.started.push((frame, l.sfx, pos, slot));
        if mp_fx.started.len() > 4096 { mp_fx.started.drain(..2048); }
        if slot < mp_fx.bot_bark.len() {
            mp_fx.bot_bark[slot] = (h != 0).then_some(h); // [M 0x1266fc]
        }
        if log_on() {
            eprintln!("[mpfx] f{frame} bot bark slot {slot} trigger {trigger:#x} line {id:#x} sfx {} at ({:.2}, {:.2}, {:.2})", l.sfx, pos[0], pos[1], pos[2]);
        }
    }

    /// The end-of-tick effects pass: the shots recorded this tick, the due fire-script sounds,
    /// the trails, the footsteps / body sounds and the death edges. Draw/audio-only (see the
    /// module doc); runs after the online combat (`step_net_combat`) or, offline, at the end of
    /// `step_bond`.
    pub fn flush_mp_fx(&mut self) {
        let events = std::mem::take(&mut self.mp_fx.events);
        let body_sounds = std::mem::take(&mut self.mp_fx.body_sounds);
        if self.mp.is_none() || self.mp_fx.disabled { return; }
        let local = self.mp_local_slot;
        let frame = self.drones.frame;
        // 1. The trails already flying move one tick (`Bullet_update` advances by the speed).
        for t in self.mp_fx.tracers.iter_mut() {
            t.pos = [t.pos[0] + t.dir[0] * t.speed, t.pos[1] + t.dir[1] * t.speed, t.pos[2] + t.dir[2] * t.speed];
            t.left -= t.speed;
        }
        self.mp_fx.tracers.retain(|t| t.left > 0.0);
        // 2. This tick's shots.
        for (k, shot) in events.iter().enumerate() {
            self.mpfx_shot(shot, k as u32, local, frame);
        }
        // 3. The fire-script sounds that fall due now (at the shooter's live head).
        let due: Vec<(u32, usize, u32)> = self.mp_fx.due.iter().copied().filter(|d| d.0 <= frame).collect();
        self.mp_fx.due.retain(|d| d.0 > frame);
        for (_, slot, id) in due {
            if let Some(p) = self.mpfx_slot_head(slot) { self.mpfx_play3d(id, p, slot); }
        }
        // 4. The remote bodies' own movement sounds (jump launch / head bump).
        for (id, pos) in body_sounds { self.mpfx_play3d(id, pos, usize::MAX); }
        // 5. Footsteps and the death edges, per slot.
        for slot in 0..nf_net::MAX_AGENTS {
            if slot == local { self.mp_fx.prev_pos[slot] = None; self.mp_fx.was_dead[slot] = false; continue; }
            let dead = self.mp_body_death(slot);
            let was = self.mp_fx.was_dead[slot];
            self.mp_fx.was_dead[slot] = dead.is_some();
            if let Some((p, _, _)) = dead {
                // `Player_CheckForDeath` 0x190810:88: `Sound_Play3D(0x89)` at the dying player
                // [M] — the humans; the bots' cry is the drone pain pool (`BOT_handlePain`'s
                // `Rand_Rand` pick, not ported here — see the doc's open items).
                let human = self.mp.as_ref().is_some_and(|m| slot < m.settings.human_count as usize);
                if !was && log_on() { eprintln!("[mpfx] f{frame} slot {slot} died at ({:.2}, {:.2}, {:.2}): death body", p[0], p[1], p[2]); }
                if !was && human { self.mpfx_play3d(0x89, [p[0], p[1] + 1.0, p[2]], slot); }
                self.mp_fx.prev_pos[slot] = None;
                continue;
            }
            let Some((pos, _, _)) = self.mp_body_state(slot) else { self.mp_fx.prev_pos[slot] = None; continue };
            let Some(prev) = self.mp_fx.prev_pos[slot].replace(pos) else { continue };
            let moved = ((pos[0] - prev[0]).powi(2) + (pos[2] - prev[2]).powi(2)).sqrt();
            if moved > 3.0 { self.mp_fx.step[slot] = 0.0; continue; } // a respawn / teleport
            // The body's grounded/substate (the bots walk the player movement core too).
            let body = self.mp_bonds.get(slot).and_then(|b| b.as_ref()).map(|b| (b.grounded, b.substate))
                .or_else(|| self.mp.as_ref().and_then(|m| m.bots.iter().find(|b| b.slot == slot)).map(|b| (b.brain.body.grounded, b.brain.body.substate)));
            let Some((grounded, substate)) = body else { continue };
            if !grounded || substate == nf_game::player::SS_SWIM { continue; }
            // `AnimObjectUpdate` 0x1befd8:163-198 [M]: the step distance wraps at the 1.65 stride
            // and the floor surface's footstep (`EffectInfo+0x16`) plays at the floor point —
            // the same rule `step_bond` runs for the local body (the climb's 0x286 arm aside).
            self.mp_fx.step[slot] += moved;
            if self.mp_fx.step[slot] > 1.65 {
                self.mp_fx.step[slot] %= 1.65;
                // The body position is `Bond::pos` (the floor + 1.1): `step_bond`'s floor ray, p → p − 1.6347.
                let hit = self.world.segment_surface(pos, [pos[0], pos[1] - 1.6347, pos[2]], true);
                let id = self.surfaces.as_ref().zip(hit).and_then(|(s, (_, _, m))| nf_formats::sfx::SurfaceSounds::get(&s.footstep, m));
                if let (Some(id), Some((t, _, _))) = (id, hit) {
                    self.mpfx_play3d(id, [pos[0], pos[1] - 1.6347 * t, pos[2]], slot);
                }
            }
        }
    }

    /// A slot's head (the human's eye, the bot's eye), for the sounds that follow the shooter.
    fn mpfx_slot_head(&self, slot: usize) -> Option<V3> {
        if slot == self.mp_local_slot { return None; }
        if let Some(b) = self.mp_bonds.get(slot).and_then(|b| b.as_ref()) { return Some(b.head()); }
        self.mp.as_ref()?.bots.iter().find(|b| b.slot == slot).map(|b| b.brain.pos)
    }

    fn mpfx_shot(&mut self, shot: &MpShot, index: u32, local: usize, frame: u32) {
        let Some(rec) = self.weapons.records.get(shot.weapon as usize).cloned() else { return };
        // The per-shot effect generator: the same draws for the same shot on every machine,
        // and never the lockstep stream.
        let mut rng = Rng::new(frame.wrapping_mul(0x0100_0193) ^ (shot.shooter as u32).wrapping_mul(0x9e37_79b9) ^ index.wrapping_mul(0x85eb_ca6b) ^ 0x6d70_6678);
        let flags6c = rec_u32(&rec, 0x6c);
        let flags70 = rec_u32(&rec, 0x70);
        let own = shot.shooter == local;
        if !own {
            if shot.shooter < nf_net::MAX_AGENTS { self.mp_fx.last_shot[shot.shooter] = Some(frame); }
            if shot.bot {
                // `DroneWeap_FireWeapon` 0x178f68: the muzzle light `Light_Create(7.0, 1.0, muzzle,
                // rgb +0x56/+0x55/+0x54, 0, life 0, kind 1)` [M :189] and the shot sound
                // `Sound_Play3D(weapon_data+0x60, drone)`, skipped for 0x5EF [M :275] — the
                // guards' path (drone.rs `handle_firing`).
                self.drones.lights.create(7.0, 1.0, shot.from, [rec[0x56], rec[0x55], rec[0x54]], 0, 0, 1, 0, &mut rng);
                let id = u16::from_le_bytes([rec[0x60], rec[0x61]]) as u32;
                if id != 0x5ef {
                    let at = self.mp_body_state(shot.shooter).map_or(shot.from, |(p, _, _)| p);
                    self.mpfx_play3d(id, at, shot.shooter);
                }
            } else {
                // `Player_MuzzleFlash` 0x1a4298: `Light_Create(radius +0x58, 2.0, the muzzle, rgb,
                // 0, life 0, kind 1)` under the +0xDC/+0x48 gate [M :26-46]; MP skips the smoke
                // puff (`MPSettings+0x180 == 0` gate [M :47-55]).
                if rec_u32(&rec, 0xdc) != 0 && rec_u32(&rec, 0x48) != 0 {
                    self.drones.lights.create(rec_f32(&rec, 0x58), 2.0, shot.from, [rec[0x56], rec[0x55], rec[0x54]], 0, 0, 1, 0, &mut rng);
                }
                // The shot sound: the held weapon's fire script's op-1 `Sound_Play3D` — the
                // player's own gun sound (`AnimProcessScriptCmds` 0x1c1b38 op 1 → `Sound_Play3D`
                // 0x1dd548 [M]) with the per-weapon remap (`remap_op1_sound`, sound.md §7) —
                // heard at the shooter's head (the port has no remote view model; [S] position).
                // Each op-1 falls due at its script frame (1 frame per tick [S]).
                let script = self.weapons.fire_script(shot.weapon as usize);
                let mut ops: Vec<(u32, u32)> = Vec::new();
                if let Some(sc) = self.scripts.get(&script) {
                    for c in sc.commands.iter().filter(|c| c.op == 1 && c.args.len() >= 2) {
                        let id = crate::scene::remap_op1_sound(c.args[1] as u32, shot.weapon);
                        // A looped op-1 (the held laser/stunner hum) needs the script's own stop —
                        // not ported for remote bodies [S, open item].
                        if self.sfx.as_ref().is_some_and(|t| t.get(id).looped) { continue; }
                        ops.push((c.args[0] as u32, id));
                    }
                }
                for (f, id) in ops {
                    if f <= 1 { self.mpfx_play3d(id, shot.from, shot.shooter); }
                    else { self.mp_fx.due.push((frame + f - 1, shot.shooter, id)); }
                }
                // mp-shot-effects: the shot's own effects now ride the SP bullet pipeline — the
                // same code the shooter's machine runs in `step_bond`'s 'shot block [S docs/
                // systems/mp-shot-effects.md]. The cosmetic rounds (`Drones::fire_remote_cosmetic`,
                // `Bullet_init` 0x12c5a0's per-pellet spawn) fly, trail, whiz and impact through
                // `update_player_bullets` — the trail per the same +0x6C gates (`Bullet_DoTrails`
                // 0x12d890), the contact through the SP world impact (`flush_world_impacts`:
                // `Effect_Bullet`'s decal / gas / debris / emitter, the `EffectInfo+0x14` impact
                // sound, the ricochet) — and the shell ejects through the same casings pool
                // (`Bullet_init_casing` 0x12fde0). The shared MP combat owns the damage; these
                // rounds draw and sound only. The synthetic hit-feedback events (weapon 0, from
                // `mp_hurt`) spawn nothing — their presentation is the victim's own arm below.
                if shot.weapon != 0 {
                    // Disjoint field borrows: the immutable Data fields while `drones` is borrowed
                    // mutably (the Sim::data() helper would hold the whole `self`).
                    let difficulty = self.drones.difficulty;
                    let d = nf_game::drone::Data {
                        tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons,
                        world: &self.world, level: self.level, sequences: &self.seqs, skins: &self.skins,
                        skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning,
                        hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models,
                        surfaces: self.surfaces.as_ref(), voices: self.voices.as_deref(), difficulty,
                        ai_points: &[],
                    };
                    // The remote body's gun, not its eye (a two-window render: the smoke
                    // and the casing started at the shooter's head while the flash was out at arm's length). The
                    // drawn muzzle datum lives on the render side, so the sim takes the same spot from the shot
                    // ray: forward along the aim, to the gun side and down to hand height [S, matched by eye to the
                    // render's flash]. The bullet's own trajectory stays the eye ray (the damage is the shared
                    // combat's); only these cosmetic effects move.
                    let muzzle = remote_muzzle(shot.from, shot.dir);
                    self.drones.fire_remote_cosmetic(&d, muzzle, shot.dir, shot.weapon, &mut rng);
                    // The muzzle smoke: `Player_MuzzleFlash` 0x1a4298's `Gas_Init(0.05, 0.1, …,
                    // 0x2000039, 7, 0x48403080, life 128)` with its `Rand_FRand(2π)` draw, under the
                    // same +0x68 & 0x2000 no-smoke gate as SP [M; the game's own MP mode skips the
                    // puff (MPSettings+0x180) — the port matches SP instead, a port choice [S]].
                    if rec_u32(&rec, 0xdc) != 0 && rec_u32(&rec, 0x48) != 0 && rec_u32(&rec, 0x68) & 0x2000 == 0 {
                        let _angle = rng.frand(6.2831854820251465);
                        // The muzzle smoke's NULL velocity takes the wind branch [M 0x1a4428; 0x1aacc8:108-116]
                        // — init_ex consumes spin, sign and the 3 wind draws from the shared stream.
                        self.drones.gas.init_ex(&mut rng, nf_game::gas::SMOKE_QUAD, muzzle, 0.0, [0.0; 3], 0.05, 0.1, 7, [0x48, 0x40, 0x30, 0x80], 1, 128.0);
                    }
                }
            }
            // The bots keep the bespoke trail slot (their shots are the game's drone path, not the
            // player pipeline); a remote HUMAN's trail now comes from its cosmetic bullet through
            // the SP trail pass (`update_player_bullets` / `Bullet_DoTrails` 0x12d890's +0x6C gates).
            if shot.bot && flags6c & 0x1010 != 0 {
                let speed = rec_f32(&rec, 0x20).max(0.01);
                let d = [shot.end[0] - shot.from[0], shot.end[1] - shot.from[1], shot.end[2] - shot.from[2]];
                let left = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if left > 0.0 { self.mp_fx.tracers.push(MpTracer { pos: shot.from, dir: shot.dir, speed, left }); }
                if self.mp_fx.tracers.len() > MP_TRACER_SLOTS { self.mp_fx.tracers.remove(0); }
            }
        }
        match shot.hit {
            MpShotHit::World { normal, material } if shot.bot => {
                // A bot's round: `Bullet_CollisionHandler` 0x12ce80 → `Effect_Create` /
                // `Effect_Bullet` at the flush — the impact sound `EffectInfo+0x14` only with the
                // effects bit (+0x70 & 2) and never for the laser kind (+0x6C & 0x100) [M
                // :186-192, Effect_Bullet 0x1ce8a8:86-90]; then the decal / gas / debris / emitter
                // stack of the surface (`Effects::bullet_impact`, the player's own world-hit
                // path). A remote HUMAN's world impact now comes from its cosmetic bullet through
                // the SP collision instead (mp-shot-effects) — one tick after the shot, like the
                // shooter's own [S]. The ricochet re-aim is not followed (no remote bullet object)
                // [S].
                let laser = flags6c & 0x100 != 0;
                if !laser && flags70 & 2 != 0 {
                    let id = self.surfaces.as_ref().and_then(|s| nf_formats::sfx::SurfaceSounds::get(&s.impact, material));
                    if let Some(id) = id { self.mpfx_play3d(id, shot.end, shot.shooter); }
                }
                if laser {
                    // The bot laser's impact puff: Effect_Laser 0x1cef38's shape (kind 5, life 8),
                    // the same real Gas_Init site the SP path rides — spin/sign from the stream.
                    self.drones.gas.init_ex(&mut rng, nf_game::gas::SMOKE_QUAD, shot.end, 0.0, [0.0; 3], 2.0, 3.0, 5, [0x7f, 0x7f, 0x64, 0xff], 0, 8.0);
                } else {
                    let decals = self.drones.effects.decals.len();
                    let _ = self.drones.effects.bullet_impact(&mut rng, self.surfaces.as_ref(), &mut self.drones.gas, &mut self.drones.fx,
                        shot.end, normal, material, shot.dir, flags70, 0, !shot.bot, 8);
                    if log_on() {
                        eprintln!("[mpfx] f{frame} slot {} weapon {:#x} world impact at ({:.2}, {:.2}, {:.2}) material {:#x} decals {} -> {}",
                            shot.shooter, shot.weapon, shot.end[0], shot.end[1], shot.end[2], material, decals, self.drones.effects.decals.len());
                    }
                }
            }
            MpShotHit::Bot(_) if shot.damage > 0.0 || matches!(shot.weapon, 0x32 | 0x33) => {
                // The bots are drones: `NDrone2_DealWithObjHit` 0x13fff0 case 5 puffs the flesh gas
                // at the hit — 0x20008bc (3.0, 6.0, life 0x3c) for every weapon outside {0x2b,
                // 0x34..0x37} with damage, 0x200026e (1.0, 16.0) for the Phoenix Samurai bolt
                // [M :235-256] — on every machine, the local player's own hits included (the
                // local pipeline's bullets do not touch the MP bots).
                let puff = match shot.weapon {
                    0x32 | 0x33 => Some((1.0, 16.0, 0x0200_026e)),
                    w if !matches!(w, 0x2b | 0x34..=0x37) => Some((3.0, 6.0, 0x0200_08bc)),
                    _ => None,
                };
                if let Some((s0, s1, mesh)) = puff {
                    // The bot flesh puff: NDrone2_DealWithObjHit 0x13fff0:245-253 — quat NULL, vel
                    // NULL, no caller draw; Gas_Init's spin/sign come from the shared stream.
                    self.drones.gas.init_ex(&mut self.drones.rng, mesh, shot.end, 0.0, [0.0; 3], s0, s1, 5, [0xff, 0xff, 0xff, 0xff], 0, 60.0);
                    if log_on() { eprintln!("[mpfx] f{frame} slot {} hit bot at ({:.2}, {:.2}, {:.2}): flesh puff {mesh:#x}", shot.shooter, shot.end[0], shot.end[1], shot.end[2]); }
                }
            }
            MpShotHit::Human(v) if v == local && !own => {
                // `Player_DealWithObjHit` 0x194458 on the local player [M]: `Player_HandlePain`
                // (the flash +0x968 += min(d·21.333, 128), capped 255 [M 0x1902c8:110-119]; its
                // 1-in-4 grunt 0x88), the direction bits +0x967 from the shooter's position in the
                // view frame [M :201-226], and `Sound_Play(100, 0x412)` when `+0x70 & 0x180 == 0`
                // [M :229-236]. A human victim shows no flesh puff: the body's hit carries material
                // 0 (`Collide_Jointy` 0x1ec180:125/173) and `Effect_Create` returns [M 0x1ce688:52].
                // The health itself is the hashed `mp_hp` (the combat already took it).
                let shooter_at = self.mpfx_slot_head(shot.shooter).unwrap_or(shot.from);
                let h = &mut self.drones.health;
                h.dir_bits = nf_game::bullet::PlayerHealth::hit_dir_bits(shooter_at, self.bond.head(), self.bond.yaw);
                h.flash = nf_game::bullet::PlayerHealth::pain_glow(h.flash, shot.damage);
                if flags70 & 0x180 == 0 {
                    self.drones.audio.play(0x412);
                    self.mp_fx.started.push((frame, 0x412, self.bond.head(), local));
                    if log_on() { eprintln!("[mpfx] f{frame} hit by slot {}: sound 0x412 (2D), dir bits {:#x}, flash {:.0}", shot.shooter, self.drones.health.dir_bits, self.drones.health.flash); }
                }
                if rng.rand(4) == 0 { let p = self.bond.pos; self.mpfx_play3d(0x88, p, local); }
            }
            _ => {}
        }
    }
}

/// A remote body's gun muzzle from its shot ray: `from` (the eye) + 0.55 forward along the aim, 0.30 to the gun side
/// and 0.35 down [S — the third-person gun's place in the render; the drawn muzzle datum is render-side only].
fn remote_muzzle(from: [f32; 3], dir: [f32; 3]) -> [f32; 3] {
    let l = (dir[0] * dir[0] + dir[2] * dir[2]).sqrt().max(1e-6);
    let (fx, fz) = (dir[0] / l, dir[2] / l);
    // The side vector on the floor plane: the aim turned a quarter right.
    let (sx, sz) = (-fz, fx);
    [from[0] + dir[0] * 0.55 + sx * 0.30, from[1] + dir[1] * 0.55 - 0.35, from[2] + dir[2] * 0.55 + sz * 0.30]
}
