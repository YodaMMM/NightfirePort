//! mp-death-hands-online: the online death's hands switch and the respawn re-equip.
//!
//! The game's law: `Player_CheckForDeath` 0x190810 runs `Player_WeaponNone` on the death tick
//! [M 0x190810:61], which sets the held id to the fists 0x47 at once, no holster cycle [M
//! 0x1a9858:12-14]. Online, the dead machine's wire byte (nf-net input byte 21, `held`) carries
//! 0x47 for the whole dead window — the app builds it from its own gun (`ni.held =
//! sim.gun.weapon`, lib.rs:4643) and `step_net_combat` copies it into the per-slot view state on
//! every machine (scene.rs:10630) — so every machine's remote body shows the fists (the draw
//! getters `mp_body_held` / `mp_body_death`, scene.rs:10497 / mpfx.rs:118; class 0, no weapon
//! mesh). The shared respawn after the game's 5-second gate [M 0x190bb0:41 — FRAME_RATE_INT·5
//! frames, 300 at 60 Hz; the call :53] re-equips inside `MP_ReSpawn`'s dead arm [M
//! 0x185368:78 → MP_EquipPlayer 0x1855f8:14-53]: the wipe, the fists, the weapon set's spawn
//! weapon with `clip_size·2` rounds, and the machines' shot gates re-seed the same way
//! (scene.rs:10713 local `mp_equip_local`, :10720 `gate_equip` on every machine).
//!
//! Tests (NF_ISO-gated — the arenas load from the disc; NF_SILENT, numbers only):
//! - a 2-machine kill: the victim's gun is 0x47 while dead, the remote view holds the fists, the
//!   kit is back with clip_size·2 after the 300-tick gate and the gates agree; sister: the
//!   killer's gun is untouched (never 0x47, no re-grant).
//! - a 3-machine run with two deaths: `net_state_hash` equal on all three machines every tick.
//!
//! The one-sample lag of the byte behind the kill tick is the app's input-sampling order (the
//! input for tick T is sampled before tick T's sim runs) [S port design — the PS2 had no online
//! play]: the tests assert the byte while the victim's death sequence is active, not on the kill
//! tick itself.

use nf_game::mp::combat as cb;
use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;
use nf_view::scene::{Scene, Sim};

const N: usize = nf_net::MAX_AGENTS;
const FISTS: u8 = 0x47;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings(level: u32, humans: usize) -> MpSettings {
    let mut s = MpSettings::deathmatch(level);
    s.human_count = humans as u32;
    s.player_count = humans as u16;
    s.total_players = humans as u32;
    s.bot_count = 0;
    s.seed = 0x0bed_0bad;
    s.time_limit = 60 * 60;
    s.frag_limit = -1;
    s.weapon_set = 0;
    s
}

/// The spawn weapon of weapon set 0 (Normal): `PickupMatrix[0][0]` [D the ELF's 0x2b9100 bytes;
/// M nf-game::mp::equip::PICKUP_MATRIX].
fn spawn_weapon() -> u8 {
    nf_game::mp::equip::PICKUP_MATRIX[0][0] as u8
}

/// The kit `MP_EquipPlayer` grants: the clip holds `clip_size` and the ammo type carries
/// `clip_size` more (the `clip_size·2` split) [M 0x1855f8:22-26 + the equip split].
fn assert_kit(sim: &Sim, w: u8, what: &str) {
    let cs = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w as usize);
    let clip = sim.drones.inventory.clip.get(cs).copied().unwrap_or(-1);
    let carried = sim.drones.inventory.carried.get(sim.weapons.ammo_type(w as usize)).copied().unwrap_or(-1);
    let expect = sim.weapons.clip_size(w as usize) as i16;
    assert_eq!(sim.gun.weapon, w, "{what}: the spawn weapon is drawn");
    assert!(sim.drones.inventory.owned.get(w as usize).copied().unwrap_or(false), "{what}: the spawn weapon is owned");
    assert_eq!(clip, expect, "{what}: clip[slot {cs}] holds clip_size({w:#x}) — the equip's clip_size·2 split");
    assert_eq!(carried, expect, "{what}: the ammo type carries the other clip_size rounds");
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> Scene {
    let mut scene = nf_view::load_mp_carry(path, &format!("{:08x}.bin", s.level_code), false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

fn aim(from: [f32; 3], to: [f32; 3], dy: f32) -> [f32; 3] {
    let d = [to[0] - from[0], to[1] + dy - from[1], to[2] - from[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    [d[0] / l, d[1] / l, d[2] / l]
}

fn shot_row(shooter: usize, weapon: u8, dir: [f32; 3]) -> [nf_net::Input; N] {
    let mut r = [NEUTRAL; N];
    r[shooter].held = weapon;
    r[shooter].shot = true;
    r[shooter].shot_weapon = weapon;
    r[shooter].shot_aim = cb::aim_from_dir(dir);
    r
}

/// Two open spots `dist` apart with a clear line between the heads.
fn duel_spots(s: &Sim, dist: f32) -> ([f32; 3], [f32; 3]) {
    let m = s.mp.as_ref().unwrap();
    for sp in m.spawns.points.iter().filter(|sp| sp.used != 0) {
        for k in 0..16 {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let a0 = nf_game::player::Bond::stand_at(&s.world, [sp.pos[0], sp.pos[1], sp.pos[2]], 0.0);
            let far = [sp.pos[0] + dist * a.sin(), sp.pos[1], sp.pos[2] + dist * a.cos()];
            let b0 = nf_game::player::Bond::stand_at(&s.world, far, 0.0);
            if (b0.pos[1] - a0.pos[1]).abs() > 0.2 { continue; }
            let (ha, hb) = (a0.head(), b0.head());
            let feet = |b: &nf_game::player::Bond| [b.pos[0], b.pos[1] - 0.9, b.pos[2]];
            if s.world.segment(ha, hb).is_none() && s.world.segment(feet(&a0), feet(&b0)).is_none() {
                return (a0.pos, b0.pos);
            }
        }
    }
    panic!("no clear duel line {dist} on the arena");
}

// ── the 2-machine kill: the fists byte, the remote view, the kit back ───────────────────────

struct Pair {
    a: Scene,
    b: Scene,
    t: u32,
}

impl Pair {
    fn new(level: u32) -> Pair {
        let path = iso().expect("NF_ISO");
        let s = settings(level, 2);
        Pair { a: machine(&path, &s, 0), b: machine(&path, &s, 1), t: 0 }
    }
    fn sa(&self) -> &Sim { self.a.sim.as_ref().unwrap() }
    fn sb(&self) -> &Sim { self.b.sim.as_ref().unwrap() }
    /// One lockstep tick on both machines; the hashes must agree.
    fn tick(&mut self, row: &[nf_net::Input; N]) {
        let t = self.t;
        let ha = nf_view::net_tick(self.a.sim.as_mut().unwrap(), row, 0);
        let hb = nf_view::net_tick(self.b.sim.as_mut().unwrap(), row, 1);
        assert_eq!(ha, hb, "tick {t}: the two machines' hashes agree");
        self.t += 1;
    }
    fn idle(&mut self, n: u32) {
        for _ in 0..n { self.tick(&[NEUTRAL; N]); }
    }
    fn body(&self, slot: usize) -> nf_game::player::Bond {
        let s = self.sa();
        if slot == s.mp_local_slot { s.bond.clone() } else { s.mp_bonds[slot].clone().unwrap() }
    }
    /// Teleport a slot's body (both machines, identically), stood on the floor.
    fn place(&mut self, slot: usize, pos: [f32; 3], yaw: f32) {
        for (sim, local) in [(self.a.sim.as_mut().unwrap(), 0usize), (self.b.sim.as_mut().unwrap(), 1usize)] {
            let b = nf_game::player::Bond::stand_at(&sim.world, [pos[0], pos[1] + 0.5, pos[2]], yaw);
            let bp = b.pos;
            if slot == local { sim.bond = b; } else { sim.mp_bonds[slot] = Some(b); }
            sim.mp.as_mut().unwrap().set_slot(slot, bp);
        }
    }
    /// The row the app builds: every human's byte 21 is its own gun (lib.rs:4643).
    fn row(&self) -> [nf_net::Input; N] {
        let mut r = [NEUTRAL; N];
        r[0].held = self.sa().gun.weapon;
        r[1].held = self.sb().gun.weapon;
        r
    }
    /// The clip + carried of `w` in `sim`'s local inventory.
    fn rounds(sim: &Sim, w: u8) -> (i16, i16) {
        let cs = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w as usize);
        (sim.drones.inventory.clip.get(cs).copied().unwrap_or(-1),
         sim.drones.inventory.carried.get(sim.weapons.ammo_type(w as usize)).copied().unwrap_or(-1))
    }
}

#[test]
fn online_kill_victim_holds_the_fists_wire_byte_and_the_kit_returns() {
    let mut p = Pair::new(0x0700_0023);
    let w = spawn_weapon();
    // Sister baseline: the killer holds the set's spawn weapon, the kit fresh.
    assert_kit(p.sa(), w, "fresh start (the killer, machine A)");
    assert_kit(p.sb(), w, "fresh start (the victim, machine B)");
    let (a_pos, b_pos) = duel_spots(p.sa(), 6.0);
    let yaw_ab = (b_pos[0] - a_pos[0]).atan2(b_pos[2] - a_pos[2]);
    p.place(0, [a_pos[0], a_pos[1] - 1.1, a_pos[2]], yaw_ab);
    p.place(1, [b_pos[0], b_pos[1] - 1.1, b_pos[2]], yaw_ab + std::f32::consts::PI);
    p.idle(2);
    // The kill: real P2K head rounds from slot 0 until slot 1 is dead (the shot pipeline rides
    // the rows; the hash agrees every tick).
    let mut dead = false;
    for _ in 0..12 {
        let dir = aim(p.body(0).head(), p.body(1).pos, 0.75);
        p.tick(&shot_row(0, w, dir));
        p.idle(4);
        if p.sa().mp_dead_ticks[1] > 0 { dead = true; break; }
    }
    assert!(dead, "slot 1 is dead from the real shots");
    let kill_tick = p.t;
    // The killer's gun is untouched by the death: still the spawn weapon, his rounds exactly
    // what his firing left (the snapshot the sister compares against at the end).
    assert_eq!(p.sa().gun.weapon, w, "the killer's gun after the kill");
    let killer_rounds = Pair::rounds(p.sa(), w);

    // The dead window, one tick at a time, the app's held-byte law in force.
    let mut death_active_samples = 0u32;
    let mut fists_samples = 0u32;
    let mut was_dead = false;
    let mut respawned = false;
    for _ in 0..500u32 {
        let held_b = p.sb().gun.weapon;                 // byte 21 machine B submits (lib.rs:4643)
        let active_b = p.sb().drones.death.active;      // B's death sequence is on
        let row = p.row();
        p.tick(&row);
        if held_b == FISTS { fists_samples += 1; }
        if active_b {
            death_active_samples += 1;
            // [M 0x190810:61 → 0x1a9858:12-14] the victim's gun is on the fists for the whole
            // dead window and the byte it submits carries 0x47.
            assert_eq!(held_b, FISTS, "tick {}: the dead machine's held byte is 0x47", p.t - 1);
            if p.sb().drones.death.active {
                // Still dead after this tick (the respawn, when it comes, lands mid-tick and
                // clears the sequence + re-equips inside `step_net_combat`).
                assert_eq!(p.sb().gun.weapon, FISTS, "tick {}: the victim's own gun stays on the fists", p.t - 1);
            }
        }
        let dead_now = p.sa().mp_dead_ticks[1] > 0;
        if dead_now {
            was_dead = true;
            // The remote view on the killer's machine: the fists, the death pose up.
            assert_eq!(p.sa().mp_body_held(1), row[1].held, "tick {}: the remote body holds the wire's held id", p.t - 1);
            assert!(p.sa().mp_body_death(1).is_some(), "tick {}: the remote death pose is up", p.t - 1);
            if active_b {
                assert_eq!(p.sa().mp_body_held(1), FISTS, "tick {}: the remote body shows the fists", p.t - 1);
            }
        }
        // Sister, every tick of the window: the killer's gun never goes to the hands.
        assert_eq!(p.sa().gun.weapon, w, "tick {}: the killer's gun is untouched", p.t - 1);
        assert_eq!(nf_formats::mpanim::weapon_anim_class(&p.sa().weapons, FISTS), 0,
            "0x47 is the no-weapon/fists class [M mpanim weapon_anim_class]");
        if was_dead && !dead_now && p.t > kill_tick + 10 { respawned = true; break; }
    }
    assert!(respawned, "the victim respawned after the 5 s gate [M 0x190bb0:41-53]");
    assert!(was_dead, "the window was observed dead");
    assert!(death_active_samples >= 250, "the fists window is the game's 5 s (300 ticks [M 0x190bb0:41]); got {death_active_samples}");
    assert_eq!(death_active_samples, fists_samples, "every death-active sample carried 0x47 — no other byte in the window");

    // The respawn re-equips on the victim's machine [M 0x185368:78 → 0x1855f8:14-53].
    assert!(!p.sb().drones.death.active, "the death sequence is cleared at the respawn");
    assert_kit(p.sb(), w, "respawn (machine B's local player)");
    assert_eq!(p.sa().mp_hp[1], 100.0, "armor_bonus 0 + 100 [M 0x185368:71-73]");
    assert_eq!(p.sa().mp_armour[1], 0.0, "armour zeroed [M 0x185368:69]");
    assert_eq!(p.sa().mp_hp, p.sb().mp_hp, "the machines agree on the healths");
    assert_eq!(p.sa().mp_armour, p.sb().mp_armour, "the machines agree on the armours");
    // The next wire byte is the spawn weapon again (the kit is visible to everyone).
    assert_eq!(p.sb().gun.weapon, w, "the victim's gun after the respawn");
    // The shot gates agree on every machine: the re-seed holds the fresh kit on both.
    let ga = p.sa().mp_combat.gates[1].clone();
    let gb = p.sb().mp_combat.gates[1].clone();
    assert_eq!(ga.weapon, w, "the gate's held weapon is the spawn weapon");
    assert_eq!(ga.inv.clip, gb.inv.clip, "the machines' gate clips agree");
    assert_eq!(ga.inv.carried, gb.inv.carried, "the machines' gate carried rounds agree");
    let expect = p.sa().weapons.clip_size(w as usize) as i16;
    let cs = nf_game::pickup::Inventory::clip_slot(&p.sa().weapons, w as usize);
    assert_eq!(ga.inv.clip.get(cs).copied().unwrap_or(-1), expect, "the re-seeded gate clip holds clip_size");
    assert_eq!(ga.inv.carried.get(p.sa().weapons.ammo_type(w as usize)).copied().unwrap_or(-1), expect, "the re-seeded gate carried holds clip_size");
    // Sister, at the end: the killer's gun is still his own — the spawn weapon, his rounds
    // exactly what his firing left (no hands switch, no respawn re-grant).
    assert_eq!(p.sa().gun.weapon, w, "the killer's gun after the victim's respawn");
    assert_eq!(Pair::rounds(p.sa(), w), killer_rounds, "the killer's rounds are untouched by the victim's death and respawn");
}

// ── the 3-machine run: deaths keep the state hash equal ─────────────────────────────────────

struct Trio {
    m: [Scene; 3],
    t: u32,
}

impl Trio {
    fn new(level: u32) -> Trio {
        let path = iso().expect("NF_ISO");
        let s = settings(level, 3);
        Trio { m: [machine(&path, &s, 0), machine(&path, &s, 1), machine(&path, &s, 2)], t: 0 }
    }
    fn s(&self, i: usize) -> &Sim { self.m[i].sim.as_ref().unwrap() }
    /// One lockstep tick on all three machines; every pair's hashes must agree.
    fn tick(&mut self, row: &[nf_net::Input; N]) {
        let t = self.t;
        let ha = nf_view::net_tick(self.m[0].sim.as_mut().unwrap(), row, 0);
        let hb = nf_view::net_tick(self.m[1].sim.as_mut().unwrap(), row, 1);
        let hc = nf_view::net_tick(self.m[2].sim.as_mut().unwrap(), row, 2);
        assert_eq!(ha, hb, "tick {t}: machines 0 and 1 agree");
        assert_eq!(hb, hc, "tick {t}: machines 1 and 2 agree");
        self.t += 1;
    }
    fn idle(&mut self, n: u32) {
        for _ in 0..n { self.tick(&[NEUTRAL; N]); }
    }
    fn body(&self, slot: usize) -> nf_game::player::Bond {
        let s = self.s(slot);
        if slot == s.mp_local_slot { s.bond.clone() } else { s.mp_bonds[slot].clone().unwrap() }
    }
    /// Teleport a slot's body (all three machines, identically), stood on the floor.
    fn place(&mut self, slot: usize, pos: [f32; 3], yaw: f32) {
        for i in 0..3 {
            let sim = self.m[i].sim.as_mut().unwrap();
            let b = nf_game::player::Bond::stand_at(&sim.world, [pos[0], pos[1] + 0.5, pos[2]], yaw);
            let bp = b.pos;
            if slot == i { sim.bond = b; } else { sim.mp_bonds[slot] = Some(b); }
            sim.mp.as_mut().unwrap().set_slot(slot, bp);
        }
    }
    /// The row the app builds: every human's byte 21 is its own gun (lib.rs:4643).
    fn row(&self) -> [nf_net::Input; N] {
        let mut r = [NEUTRAL; N];
        for i in 0..3 { r[i].held = self.s(i).gun.weapon; }
        r
    }
    /// Real shots from `shooter` at `victim` until the victim is dead; the hashes agree.
    fn kill(&mut self, shooter: usize, victim: usize) -> u32 {
        let mut dead = false;
        for _ in 0..12 {
            let dir = aim(self.body(shooter).head(), self.body(victim).pos, 0.75);
            let mut r = shot_row(shooter, spawn_weapon(), dir);
            let law = self.row();
            r[shooter].held = law[shooter].held;
            self.tick(&r);
            self.idle(4);
            if self.s(shooter).mp_dead_ticks[victim] > 0 { dead = true; break; }
        }
        assert!(dead, "slot {victim} is dead from slot {shooter}'s real shots");
        self.t
    }
}

#[test]
fn three_machines_deaths_keep_the_state_hash_equal() {
    let mut t = Trio::new(0x0700_0024);
    // Death 1: slot 0 kills slot 1; the whole 300-tick window and the respawn run on the
    // shared stream, the hashes equal on every machine every tick.
    let kill1 = t.kill(0, 1);
    let mut settled = false;
    for _ in 0..500u32 {
        let row = t.row();
        t.tick(&row);
        if t.s(0).mp_dead_ticks[1] == 0 && t.t > kill1 + 10 { settled = true; break; }
    }
    assert!(settled, "slot 1 respawned [M 0x190bb0:41-53]");
    assert_eq!(t.s(0).mp_hp, t.s(1).mp_hp, "the machines agree on the healths after the respawn");
    assert_eq!(t.s(0).mp_hp, t.s(2).mp_hp, "the machines agree on the healths (2 vs 0)");
    // Idle combat-free ticks between the deaths (the gates tick, the hash stays equal).
    t.idle(30);
    // Death 2: slot 1 (respawned) kills slot 2. Place them with a clear line first.
    let (a_pos, b_pos) = duel_spots(t.s(0), 6.0);
    let yaw_ab = (b_pos[0] - a_pos[0]).atan2(b_pos[2] - a_pos[2]);
    t.place(1, [a_pos[0], a_pos[1] - 1.1, a_pos[2]], yaw_ab);
    t.place(2, [b_pos[0], b_pos[1] - 1.1, b_pos[2]], yaw_ab + std::f32::consts::PI);
    t.idle(2);
    let kill2 = t.kill(1, 2);
    let mut settled2 = false;
    for _ in 0..500u32 {
        let row = t.row();
        t.tick(&row);
        if t.s(0).mp_dead_ticks[2] == 0 && t.t > kill2 + 10 { settled2 = true; break; }
    }
    assert!(settled2, "slot 2 respawned [M 0x190bb0:41-53]");
    t.idle(30);
    // The full run stayed in lockstep: the final states agree beyond the hash.
    assert_eq!(t.s(0).mp_hp, t.s(1).mp_hp, "final healths agree");
    assert_eq!(t.s(0).mp_hp, t.s(2).mp_hp, "final healths agree (2 vs 0)");
    assert_eq!(t.s(0).mp_dead_ticks, t.s(1).mp_dead_ticks, "final dead ticks agree");
    assert_eq!(t.s(0).mp_dead_ticks, t.s(2).mp_dead_ticks, "final dead ticks agree (2 vs 0)");
    assert_eq!(t.s(0).mp_combat.gates[2].inv.clip, t.s(1).mp_combat.gates[2].inv.clip, "the respawned slot's gate clips agree");
    // The named end state: the three machines' hashes agree right now.
    let (h0, h1, h2) = (t.s(0).net_state_hash(0), t.s(1).net_state_hash(1), t.s(2).net_state_hash(2));
    assert_eq!(h0, h1, "the final hash: machines 0 and 1");
    assert_eq!(h1, h2, "the final hash: machines 1 and 2");
}
