// Real-damage bookkeeping shared by both walkers (`include!`d by walker.rs and walker_b2.rs, so
// each module gets its own copy of the type).
//
// `NF_WALK_NO_TOPUP=1` turns the walker's health top-up off for every walk in the binary, the
// tests' own `top_up_health = true` included (`Walker::tops_up`). The log then reads the sim's
// own hit record (`Drones::player_hits` — (frame, owner drone, health lost), pushed by every
// Bond-damage site in nf-game/drone.rs) each tick, keeps the lowest health, and on the first
// tick at 0 health prints the death spot and the last hits (owner skin, weapon, distance).
// The walker's Drop prints the end summary. No game state is written.

thread_local! {
    /// `with_real_damage`'s switch: the top-up is off for every walker on this test's thread.
    static REAL_DAMAGE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_real_damage_no_return_fire`: the walkers made on this thread do not return fire.
    static NO_RETURN_FIRE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The top-up is off: `NF_WALK_NO_TOPUP` for the whole binary, or `with_real_damage` for this
/// test's thread (libtest runs each test on its own thread, so a sister test is untouched).
pub fn no_topup_env() -> bool {
    REAL_DAMAGE.with(|c| c.get()) || std::env::var_os("NF_WALK_NO_TOPUP").is_some()
}

/// Run a full walk on real damage: no health top-up for any walker the closure makes (the
/// tests' own `top_up_health = true` included), and Bond's death panics with the death line.
#[allow(dead_code)]
pub fn with_real_damage<R>(f: impl FnOnce() -> R) -> R {
    REAL_DAMAGE.with(|c| c.set(true));
    let r = f();
    REAL_DAMAGE.with(|c| c.set(false));
    r
}

/// `with_real_damage` for a route whose legs were laid for an unanswered walk (a return-fire
/// stop on a climb or ahead of a scripted rescue breaks the leg): real damage, no return fire.
#[allow(dead_code)]
pub fn with_real_damage_no_return_fire<R>(f: impl FnOnce() -> R) -> R {
    NO_RETURN_FIRE.with(|c| c.set(true));
    let r = with_real_damage(f);
    NO_RETURN_FIRE.with(|c| c.set(false));
    r
}

#[derive(Default)]
pub struct DamageLog {
    /// Entries of `player_hits` already read.
    pub hits_seen: usize,
    /// Lowest health seen.
    pub min_health: f32,
    /// Health lost to recorded hits, by owner (drone index, or usize::MAX for the world).
    pub by_owner: Vec<(usize, f32, u32)>,
    /// Health lost outside the hit record (falls, drowning, hurt volumes).
    pub other: f32,
    /// The death line, once.
    pub died: Option<String>,
    /// Panic on Bond's death (the walk cannot go on; the real-damage variants and the env run).
    pub die_panics: bool,
    /// The log is live (top-up off at construction or later).
    pub active: bool,
    /// Return fire on real damage (`shooters`); a test that scripts its own fight turns it off.
    pub return_fire: bool,
    /// Inside a return-fire fight (the fight's own walks must not start another).
    pub fighting: bool,
    /// Return-fire fights started, for the summary.
    pub fights: u32,
    /// (drone, tick) — a guard a fight did not drop is not re-fought before the tick.
    pub cooldown: Vec<(usize, u32)>,
    last_health: f32,
    last_hits: Vec<(u32, usize, f32)>,
}

impl DamageLog {
    pub fn new() -> DamageLog {
        DamageLog { min_health: 100.0, last_health: 100.0, die_panics: no_topup_env(), active: no_topup_env(), return_fire: !NO_RETURN_FIRE.with(|c| c.get()) && std::env::var_os("NF_WALK_NO_RETURN_FIRE").is_none(), ..Default::default() }
    }

    fn owner_desc(sim: &nf_view::scene::Sim, owner: usize) -> String {
        let b = sim.bond.pos;
        match sim.drones.list.get(owner) {
            Some(d) => format!("drone #{owner} skin {:#x} weapon {:#x} at ({:.1},{:.1},{:.1}) d {:.1} hp {:.0}",
                d.skin, d.weapon, d.pos[0], d.pos[1], d.pos[2],
                ((d.pos[0] - b[0]).powi(2) + (d.pos[1] - b[1]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt(), d.health),
            None => "the world (live wire / blast / hurt volume)".into(),
        }
    }

    /// Read this tick's damage (before the walker's top-up, when on).
    pub fn note(&mut self, sim: &nf_view::scene::Sim, tick: u32) {
        let h = sim.drones.health.health;
        let hits = &sim.drones.player_hits;
        let mut recorded = 0.0;
        if self.hits_seen > hits.len() { self.hits_seen = 0; }
        for &(f, o, dmg) in &hits[self.hits_seen..] {
            recorded += dmg;
            match self.by_owner.iter_mut().find(|e| e.0 == o) {
                Some(e) => { e.1 += dmg; e.2 += 1; }
                None => self.by_owner.push((o, dmg, 1)),
            }
            if std::env::var_os("NF_DMG_TRACE").is_some() && dmg != 0.0 {
                let p = sim.bond.pos;
                eprintln!("[dmg] tick {tick} frame {f}: -{dmg:.2} (health {:.1}) Bond ({:.1},{:.1},{:.1}) from {}",
                    sim.drones.health.health, p[0], p[1], p[2], Self::owner_desc(sim, o));
                if o >= sim.drones.list.len() {
                    for e in sim.drones.explosions.iter().rev().take(3) {
                        let d = ((e.0[0] - p[0]).powi(2) + (e.0[1] - p[1]).powi(2) + (e.0[2] - p[2]).powi(2)).sqrt();
                        eprintln!("[dmg]   explosion at ({:.1},{:.1},{:.1}) d {d:.1} radius {:.1} damage {:.1} owner {} kind {}",
                            e.0[0], e.0[1], e.0[2], e.1, e.2, e.3 as isize, e.4);
                    }
                }
            }
            self.last_hits.push((f, o, dmg));
            if self.last_hits.len() > 6 { self.last_hits.remove(0); }
        }
        self.hits_seen = hits.len();
        if !self.active { self.last_health = h; return; }
        let lost = self.last_health - h;
        if lost > recorded + 0.01 { self.other += lost - recorded; }
        if h < self.min_health { self.min_health = h; }
        self.last_health = h;
        if h <= 0.0 && self.died.is_none() {
            let p = sim.bond.pos;
            let mut s = format!("Bond died at tick {tick} (frame {}) at ({:.2},{:.2},{:.2}) armour {:.1}; last hits:",
                sim.drones.frame, p[0], p[1], p[2], sim.drones.health.armour);
            for &(f, o, dmg) in &self.last_hits {
                s.push_str(&format!("\n  frame {f}: -{dmg:.2} from {}", Self::owner_desc(sim, o)));
            }
            if lost > recorded + 0.01 { s.push_str(&format!("\n  -{:.2} this tick outside the hit record (fall / drown / hurt volume)", lost - recorded)); }
            eprintln!("[real-damage] {s}");
            self.died = Some(s);
        }
    }

    /// Return fire (real damage only): the hostiles shooting AT Bond right now, nearest first — a
    /// guard (team 1, alive, enabled) within 20 u that hit him in the last 240 frames, or fired
    /// in the last 90 with his aim point on Bond (`opp_target` within 1.5 u). A player answers
    /// whoever shoots him; a guard that has not opened fire (a party guest's guard, an unalerted
    /// patrol) is left alone, so stealth and no-kill legs are unchanged. The caller keeps the
    /// ones it can see a part of (`exposed_aim`).
    pub fn shooters(&self, sim: &nf_view::scene::Sim) -> Vec<usize> {
        let b = sim.bond.pos;
        let now = sim.drones.frame;
        let dxz = |p: [f32; 3]| ((p[0] - b[0]).powi(2) + (p[2] - b[2]).powi(2)).sqrt();
        let mut c: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
            .filter(|(i, d)| {
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { return false; }
                if dxz(d.pos) > 20.0 || (d.pos[1] - b[1]).abs() > 6.0 { return false; }
                let hit = sim.drones.player_hits.iter().rev()
                    .take_while(|h| now.saturating_sub(h.0) <= 240)
                    .any(|h| h.1 == *i);
                let aiming = d.shots.last().is_some_and(|&f| now.saturating_sub(f) <= 90)
                    && ((d.opp_target[0] - b[0]).powi(2) + (d.opp_target[2] - b[2]).powi(2)).sqrt() < 1.5;
                hit || aiming
            })
            .map(|(i, d)| (dxz(d.pos), i)).collect();
        c.sort_by(|a, b| a.0.total_cmp(&b.0));
        c.into_iter().map(|(_, i)| i).collect()
    }

    /// The exposed part of drone `i` Bond's bullet can reach: the hit box whose centre ray enters
    /// the box before the static world's bullet trace stops it (walker.rs `exposed_aim`'s rule —
    /// `Collide_Jointy` 0x1ec180's first box vs the flight's world stop [M 0x1ea398:216-229]),
    /// the one nearest the torso. None = nothing of him is reachable from here.
    pub fn exposed_part(sim: &nf_view::scene::Sim, i: usize) -> Option<[f32; 3]> {
        let d = &sim.drones.list[i];
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
            ai_points: &sim.ai_points,
        };
        let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
        let mut best: Option<(f32, [f32; 3])> = None;
        for b in d.hit_boxes(&data) {
            let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
            let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
            let Some(t_box) = nf_game::damage::ray_box(eye, ray, &b) else { continue };
            let tw = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, c);
            if tw.is_some_and(|w| w < t_box) { continue; }
            let dev = ((c[0] - torso[0]).powi(2) + (c[1] - torso[1]).powi(2) + (c[2] - torso[2]).powi(2)).sqrt();
            if best.map_or(true, |(bd, _)| dev < bd) { best = Some((dev, c)); }
        }
        best.map(|(_, c)| c)
    }

    /// Bond can put a round into drone `i` from where he stands (`exposed_part`).
    pub fn visible(sim: &nf_view::scene::Sim, i: usize) -> bool { Self::exposed_part(sim, i).is_some() }

    pub fn summary(&self, sim: &nf_view::scene::Sim) -> String {
        let mut top = self.by_owner.clone();
        top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut s = format!("end health {:.1} armour {:.1}, lowest {:.1}, {} hits for {:.1}, other {:.1}{}",
            sim.drones.health.health, sim.drones.health.armour, self.min_health,
            top.iter().map(|e| e.2).sum::<u32>(), top.iter().map(|e| e.1).sum::<f32>(), self.other,
            if self.died.is_some() { ", DIED" } else { "" });
        s.push_str(&format!(", {} return-fire fights", self.fights));
        for e in top.iter().take(5) {
            s.push_str(&format!("\n  {:.1} in {} hits: {}", e.1, e.2, Self::owner_desc(sim, e.0)));
        }
        s
    }
}

// The return-fire response, shared by both walkers (this file is `include!`d into each walker
// module, so `Walker` is that module's walker; both carry `tick`, `tick_fire`, `switch_to_armed`,
// `tops_up`, `scene`, `total_ticks` and `dmg`).
#[allow(dead_code)]
impl Walker {
    /// Real damage only: answer the guard shooting Bond (`DamageLog::shooters`, the nearest one
    /// Bond can see) from where he stands, then let the leg walk on. Returns true when a fight
    /// ran. Off while the top-up is on, so the top-up walks are unchanged.
    pub fn return_fire(&mut self, label: &str) -> bool {
        if self.tops_up() || !self.dmg.return_fire || self.dmg.fighting { return false; }
        let di = {
            let sim = self.scene.sim.as_ref().unwrap();
            if sim.disabled || !sim.bond.grounded { return false; }
            self.dmg.shooters(sim).into_iter().find(|&i| DamageLog::visible(sim, i))
        };
        let Some(di) = di else { return false };
        if self.dmg.cooldown.iter().any(|&(i, until)| i == di && self.total_ticks < until) { return false; }
        // Only with a gun in hand: a player without rounds walks on (the 16 arrival carries
        // the fists alone — the weapon cycle stood Bond 730 ticks at the ramp foot under fire).
        {
            let sim = self.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            let in_hand = (2..=0x47).contains(&g.weapon) && g.clip + g.spare > 0;
            let inv = &sim.drones.inventory;
            let carried = (2..=0x47usize).any(|w| inv.owned.get(w).copied().unwrap_or(false) && {
                let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w);
                inv.clip.get(slot).copied().unwrap_or(0) > 0
                    || inv.carried.get(sim.weapons.ammo_type(w)).copied().unwrap_or(0) > 0
            });
            if !in_hand && !carried { return false; }
        }
        self.dmg.fighting = true;
        self.dmg.fights += 1;
        let h0 = self.scene.sim.as_ref().unwrap().drones.health.health;
        let killed = self.switch_to_armed(label) && self.stand_and_fire(di, 900);
        // Level the view again before walking on (the walk's E presses and plate legs were
        // written for a level view; a fight leaves the pitch on the guard).
        for _ in 0..90 {
            let p = self.scene.sim.as_ref().unwrap().bond.pitch;
            if p.abs() < 0.01 { break; }
            self.tick(Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
        }
        self.dmg.fighting = false;
        // A guard the stand did not drop (out of sight, the cap) is left for 240 ticks: the
        // leg walks on instead of standing in the open re-fighting him.
        if !killed { let until = self.total_ticks + 240; self.dmg.cooldown.retain(|e| e.0 != di); self.dmg.cooldown.push((di, until)); }
        let sim = self.scene.sim.as_ref().unwrap();
        eprintln!("[real-damage] {label}: returned fire on drone #{di} (killed {killed}) at tick {}, health {h0:.1} -> {:.1}",
            self.total_ticks, sim.drones.health.health);
        true
    }

    /// Aim at drone `di` (his exposed part, `DamageLog::exposed_part`) with the view and tap the
    /// trigger when the view is on him; reload on an empty clip. Bond does not move: a player
    /// returning fire on a ledge or a stair does not sidestep off it (the walker's `shoot`
    /// flanks, which walked Bond off the 05 tower lane and the 14 stairwell). Stops when the
    /// guard dies (true), is out of sight for 60 ticks, takes no round for 300, the gun is dry,
    /// or after `cap` ticks. A
    /// tap whose line passes within 0.6 u of another live drone short of the target is held
    /// (the game's bullets hit any drone — the 03 servants stand among the guards).
    pub fn stand_and_fire(&mut self, di: usize, cap: u32) -> bool {
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
        let mut aim: Option<[f32; 3]> = None;
        let mut hp0 = self.scene.sim.as_ref().unwrap().drones.list[di].health;
        let mut last_hit = 0u32;
        for t in 0..cap {
            let (input, fire, reload) = {
                let sim = self.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                if d.corpse || d.health <= 0.0 { return true; }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 { return false; }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let torso = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                // Re-pick the exposed part every 10 ticks (the guard animates, cover buries a part).
                if t % 10 == 0 { aim = DamageLog::exposed_part(sim, di); }
                if aim.is_none() { unseen += 1; if unseen > 60 { return false; } } else { unseen = 0; }
                // No round landed for 300 ticks: walk on (a guard in cover the stand cannot reach;
                // the 13 lobby stand took 40 health that way).
                if d.health < hp0 { hp0 = d.health; last_hit = t; }
                if t - last_hit > 300 { return false; }
                let at = aim.unwrap_or(torso);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let err = { let mut a = dx.atan2(dz) - sim.bond.yaw;
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let perr = (dy / h).atan() / std::f32::consts::FRAC_PI_2 - sim.bond.pitch;
                // Another drone on the line, short of the target: hold the tap.
                let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                let u = [dx / len, dy / len, dz / len];
                let blocked = sim.drones.list.iter().enumerate().any(|(k, o)| {
                    if k == di || !o.present || o.corpse || o.health <= 0.0 { return false; }
                    let c = [o.pos[0] - eye[0], o.pos[1] + 0.6 - eye[1], o.pos[2] - eye[2]];
                    let s = c[0] * u[0] + c[1] * u[1] + c[2] * u[2];
                    if s <= 0.0 || s >= len - 0.3 { return false; }
                    let q = [c[0] - u[0] * s, c[1] - u[1] * s, c[2] - u[2] * s];
                    (q[0] * q[0] + q[2] * q[2]).sqrt() < 0.6 && q[1].abs() < 1.2
                });
                let reload = sim.gun.clip == 0;
                let fire = !reload && unseen == 0 && !blocked && err.abs() < 0.03 && perr.abs() < 0.04
                    && t.saturating_sub(last_tap) >= 8;
                (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, fire, reload)
            };
            if reload {
                self.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false);
                self.total_ticks += 1;
                self.after_step();
                for _ in 0..60 { self.tick(Input::default(), false); }
            } else if fire {
                last_tap = t;
                self.tick_fire(input);
            } else {
                self.tick(input, false);
            }
        }
        false
    }
}
