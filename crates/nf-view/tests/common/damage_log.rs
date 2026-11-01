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

    /// Bond's eye sees drone `i`'s torso or head (the static world's segment is clear).
    pub fn visible(sim: &nf_view::scene::Sim, i: usize) -> bool {
        let b = sim.bond.pos;
        let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
        let d = sim.drones.list[i].pos;
        [0.3f32, 1.5].iter().any(|h| sim.world.segment(eye, [d[0], d[1] + h, d[2]]).is_none())
    }

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
        let sim = self.scene.sim.as_ref().unwrap();
        eprintln!("[real-damage] {label}: returned fire on drone #{di} (killed {killed}) at tick {}, health {h0:.1} -> {:.1}",
            self.total_ticks, sim.drones.health.health);
        true
    }

    /// Aim at drone `di` (torso, or the head when the torso is hidden) with the view and tap the
    /// trigger when the view is on him; reload on an empty clip. Bond does not move: a player
    /// returning fire on a ledge or a stair does not sidestep off it (the walker's `shoot`
    /// flanks, which walked Bond off the 05 tower lane and the 14 stairwell). Stops when the
    /// guard dies (true), is out of sight for 60 ticks, the gun is dry, or after `cap` ticks. A
    /// tap whose line passes within 0.6 u of another live drone short of the target is held
    /// (the game's bullets hit any drone — the 03 servants stand among the guards).
    pub fn stand_and_fire(&mut self, di: usize, cap: u32) -> bool {
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
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
                let head = [d.pos[0], d.pos[1] + 1.5, d.pos[2]];
                let at = if sim.world.segment(eye, torso).is_none() { Some(torso) }
                    else if sim.world.segment(eye, head).is_none() { Some(head) } else { None };
                if at.is_none() { unseen += 1; if unseen > 60 { return false; } } else { unseen = 0; }
                let at = at.unwrap_or(torso);
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
