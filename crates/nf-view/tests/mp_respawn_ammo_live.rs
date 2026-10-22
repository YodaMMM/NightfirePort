//! mp-respawn-ammo: every `MP_ReSpawn` re-arms the game's spawn loadout.
//!
//! The game's respawn for a human is `MP_ReSpawn` 0x185368 → `MP_EquipPlayer` 0x1855f8 [M
//! 0x185368:74 — the full arm runs on the respawn of every human, dead or alive]: it WIPES the
//! inventory first (`Player_InitAmmoWeapons` 0x1a7ef8's arm [M nf-game::player::init_ammo_weapons])
//! and re-equips the weapon set's spawn weapon with `clip_size·2` rounds (the equip splits
//! `clip_size` into the clip, the rest carried [M nf-game::mp::equip]), the grapple when set, the
//! scenario bonus weapon — plus `Player_SetHealth(armor_bonus + 100.0)` [M 0x185368:65-73]. The
//! bots re-arm through `BOT_respawn` 0x124e38 → `BOTWEAP_InitWeapon` 0x129e48:44-48 (the set's
//! spawn weapon, `clip_size·2` [M nf-game::mp::equip::bot_start_weapon]).
//!
//! The user's report (an MP match from the menus): "you respawn with no ammo" — the offline
//! death→respawn path (`Sim::step_mp_death_respawn`) never re-equipped; the online path
//! (`step_net_combat`) did. These tests pin the respawned kit through the real paths: offline
//! (the death sequence → `mp_respawn`), and online (a real P2K kill, then the shared respawn —
//! every tick's hash must agree, so the re-armed `mp_combat` state agrees on every machine).
//! Weapon sets 0 (Normal, spawn weapon 6) and 1 (Pistols, spawn weapon 2) [D the PickupMatrix
//! bytes @0x2b9100; M nf-game::mp::equip::PICKUP_MATRIX]. Sister: a fresh match start is
//! unchanged (the load pins the same kit). NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::combat as cb;
use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;
use nf_view::scene::{Scene, Sim};

const N: usize = nf_net::MAX_AGENTS;
const ARENA: &str = "07000023.bin";

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings(weapon_set: u32) -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.bot_count = 0;
    s.seed = 0x0bed_0bad;
    s.time_limit = 60 * 60;
    s.frag_limit = -1;
    s.weapon_set = weapon_set;
    s
}

/// The spawn weapon of weapon set `set`: `PickupMatrix[set][0]` [D the ELF's 0x2b9100 bytes;
/// M nf-game::mp::equip::PICKUP_MATRIX].
fn spawn_weapon(set: u32) -> u8 {
    nf_game::mp::equip::PICKUP_MATRIX[set as usize][0] as u8
}

/// The kit `MP_EquipPlayer` grants for the set's spawn weapon: `clip_size` rounds in the clip and
/// `clip_size` more carried (the `clip_size·2` split) [M 0x1855f8:26-29 + the equip_weapon split;
/// D the weapon record's +0x92 clip byte through the same WeaponData].
fn kit(sim: &Sim, w: u8) -> (usize, i16, i16, i16) {
    let cs = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w as usize);
    let clip = sim.drones.inventory.clip.get(cs).copied().unwrap_or(-1);
    let carried = sim.drones.inventory.carried.get(sim.weapons.ammo_type(w as usize)).copied().unwrap_or(-1);
    let expect = sim.weapons.clip_size(w as usize) as i16;
    (cs, clip, carried, expect)
}

fn assert_kit(sim: &Sim, w: u8, what: &str) {
    let (cs, clip, carried, expect) = kit(sim, w);
    assert_eq!(sim.gun.weapon, w, "{what}: the spawn weapon is drawn");
    assert!(sim.drones.inventory.owned.get(w as usize).copied().unwrap_or(false), "{what}: the spawn weapon is owned");
    assert_eq!(clip, expect, "{what}: clip[slot {cs}] holds clip_size({w:#x}) — the equip's clip_size·2 split");
    assert_eq!(carried, expect, "{what}: the ammo type carries the other clip_size rounds");
}

/// mp-respawn-fists-draw: the equip leaves the fists 0x47 held with the spawn weapon pending
/// [M 0x1855f8:50-53] and the draw runs on the next ticks through the normal switch [M
/// 0x1a2cc0 + 0x1a2e38]. Steps the offline tick until `w` is held and the switch is idle again.
fn wait_drawn_offline(sim: &mut Sim, w: u8) {
    for _ in 0..180u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if sim.gun.weapon == w && sim.gun.switch_state == 0 { return; }
    }
    panic!("the spawn weapon {w:#04x} never finished drawing (offline)");
}

// ── the offline path: the death sequence → step_mp_death_respawn ────────────────────────────

fn offline_machine(weapon_set: u32) -> Scene {
    let path = iso().expect("NF_ISO");
    nf_view::load_mp_carry(&path, ARENA, false, None, settings(weapon_set)).expect("arena loads")
}

fn offline_kill_and_respawn(weapon_set: u32) -> Scene {
    let mut scene = offline_machine(weapon_set);
    let w = spawn_weapon(weapon_set);
    {
        let sim = scene.sim.as_mut().unwrap();
        // The load's equip leaves the fists 0x47 held with the spawn weapon pending [M
        // 0x1855f8:50-53 via Player_InitWeapon 0x1a7ef8:143-145]; the draw settles first.
        wait_drawn_offline(sim, w);
        // Sister baseline: a fresh match start holds the kit (unchanged by the fix).
        assert_kit(sim, w, "fresh match start");
    }
    // Spend the spawn kit through the real fire path (the trigger taps — the P-series is
    // semi-auto [M the fire-mode byte] — so the trigger gets a fresh edge every few ticks) until
    // the weapon is dry — the user died having fired their rounds. `step_bond` is the app's
    // offline tick (the death bracket + the respawn run inside it).
    let mut dry = false;
    for t in 0..900u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), t % 6 < 2, false, false);
        let sim = scene.sim.as_ref().unwrap();
        let cs = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w as usize);
        let clip = sim.drones.inventory.clip.get(cs).copied().unwrap_or(0);
        let carried = sim.drones.inventory.carried.get(sim.weapons.ammo_type(w as usize)).copied().unwrap_or(0);
        if clip <= 0 && carried <= 0 { dry = true; break; }
    }
    assert!(dry, "the spawn weapon ran dry through the real fire path");
    {
        let sim = scene.sim.as_mut().unwrap();
        // The kill: the health runs out — `Player_CheckForDeath` 0x190810:91's trigger — and the
        // death sequence starts; 5 s later `MP_ReSpawn` respawns [M 0x190bb0:41-54].
        sim.drones.health.health = 0.0;
    }
    let mut respawned = false;
    for t in 0..400u32 {
        scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if t > 40 && !sim.drones.death.active && sim.drones.health.health >= 100.0 {
            respawned = true;
            break;
        }
    }
    assert!(respawned, "the death sequence ran and the respawn came");
    {
        // The respawn tick holds the fists 0x47 with the spawn weapon pending [M 0x1855f8:50-53]
        // — the draw settles through the normal switch before the kit assert.
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.gun.weapon, 0x47, "respawn tick: the fists are held [M 0x1855f8:50-51]");
        assert_eq!(sim.gun.switch_request, w, "respawn tick: the spawn weapon is the switch request [M 0x1855f8:52]");
        wait_drawn_offline(sim, w);
    }
    scene
}

#[test]
fn offline_respawn_rearms_set_0_normal() {
    let scene = offline_kill_and_respawn(0);
    assert_kit(scene.sim.as_ref().unwrap(), spawn_weapon(0), "respawn (set 0 Normal)");
    assert_eq!(scene.sim.as_ref().unwrap().drones.health.health, 100.0, "armor_bonus 0 + 100 [M 0x185368:65-73]");
}

#[test]
fn offline_respawn_rearms_set_1_pistols() {
    let scene = offline_kill_and_respawn(1);
    assert_kit(scene.sim.as_ref().unwrap(), spawn_weapon(1), "respawn (set 1 Pistols)");
}

// ── the online path: a real kill, then the shared respawn on two machines ───────────────────

fn machine(path: &str, s: &MpSettings, slot: usize) -> Scene {
    let mut scene = nf_view::load_mp_carry(path, ARENA, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

struct Pair {
    a: Scene,
    b: Scene,
    t: u32,
}

impl Pair {
    fn new(weapon_set: u32) -> Pair {
        let path = iso().expect("NF_ISO");
        let s = settings(weapon_set);
        Pair { a: machine(&path, &s, 0), b: machine(&path, &s, 1), t: 0 }
    }
    fn sa(&self) -> &Sim { self.a.sim.as_ref().unwrap() }
    fn sims(&mut self) -> [(&mut Sim, usize); 2] {
        [(self.a.sim.as_mut().unwrap(), 0), (self.b.sim.as_mut().unwrap(), 1)]
    }
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
    /// mp-respawn-fists-draw: the equip leaves the fists 0x47 held with the spawn weapon pending
    /// [M 0x1855f8:50-53]; ticks both machines until each local player holds `w` idle again [M
    /// 0x1a2cc0 — the draw through the normal switch].
    fn wait_drawn(&mut self, w: u8) {
        for _ in 0..180u32 {
            // The law row: every human's byte 21 is its own gun (lib.rs:4643) — the gate's held
            // weapon follows this byte, so the draw ticks must ride the real held ids.
            let mut row: [nf_net::Input; N] = [NEUTRAL; N];
            row[0].held = self.a.sim.as_ref().unwrap().gun.weapon;
            row[1].held = self.b.sim.as_ref().unwrap().gun.weapon;
            self.tick(&row);
            let (a, b) = (self.a.sim.as_ref().unwrap(), self.b.sim.as_ref().unwrap());
            if a.gun.weapon == w && a.gun.switch_state == 0
                && b.gun.weapon == w && b.gun.switch_state == 0 { return; }
        }
        panic!("the spawn weapon {w:#04x} never finished drawing on both machines");
    }
    fn body(&self, slot: usize) -> nf_game::player::Bond {
        let s = self.sa();
        if slot == s.mp_local_slot { s.bond.clone() } else { s.mp_bonds[slot].clone().unwrap() }
    }
    /// Teleport a slot's body (both machines, identically), stood on the floor.
    fn place(&mut self, slot: usize, pos: [f32; 3], yaw: f32) {
        for (sim, local) in self.sims() {
            let b = nf_game::player::Bond::stand_at(&sim.world, [pos[0], pos[1] + 0.5, pos[2]], yaw);
            let bp = b.pos;
            if slot == local { sim.bond = b; } else { sim.mp_bonds[slot] = Some(b); }
            sim.mp.as_mut().unwrap().set_slot(slot, bp);
        }
    }
    /// The gate state one machine holds for `slot` (the shared, hashed combat state).
    fn gate(&self, slot: usize) -> (i16, i16) {
        let sim = self.sa();
        let g = &sim.mp_combat.gates[slot];
        let w = g.weapon as usize;
        let cs = nf_game::pickup::Inventory::clip_slot(&sim.weapons, w);
        (g.inv.clip.get(cs).copied().unwrap_or(-1), g.inv.carried.get(sim.weapons.ammo_type(w)).copied().unwrap_or(-1))
    }
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

/// Two open spots `dist` apart with a clear line between the heads, on 07000023.
fn duel_spots(p: &Pair, dist: f32) -> ([f32; 3], [f32; 3]) {
    let s = p.sa();
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

fn setup(p: &mut Pair, dist: f32) {
    let (a, b) = duel_spots(p, dist);
    let yaw_ab = (b[0] - a[0]).atan2(b[2] - a[2]);
    p.place(0, [a[0], a[1] - 1.1, a[2]], yaw_ab);
    p.place(1, [b[0], b[1] - 1.1, b[2]], yaw_ab + std::f32::consts::PI);
    p.idle(2);
}

#[test]
fn online_respawn_rearms_and_the_machines_agree() {
    let mut p = Pair::new(0);
    let w = spawn_weapon(0);
    // The load's equip leaves the fists 0x47 held with the spawn weapon pending [M
    // 0x1855f8:50-53 via Player_InitWeapon 0x1a7ef8:143-145]; the draw settles first.
    assert_eq!(p.a.sim.as_ref().unwrap().gun.weapon, 0x47, "fresh load (A): the fists are held [M 0x1855f8:51]");
    assert_eq!(p.a.sim.as_ref().unwrap().gun.switch_request, w, "fresh load (A): the spawn weapon is the switch request [M 0x1855f8:52]");
    assert_eq!(p.b.sim.as_ref().unwrap().gun.weapon, 0x47, "fresh load (B): the fists are held [M 0x1855f8:51]");
    p.wait_drawn(w);
    // Sister baseline: a fresh match start — both machines' local kits and the shared gates hold
    // the set's kit, and the gate states agree.
    assert_kit(p.a.sim.as_ref().unwrap(), w, "fresh start (machine A)");
    assert_kit(p.b.sim.as_ref().unwrap(), w, "fresh start (machine B)");
    assert_eq!(p.gate(0), (p.sa().weapons.clip_size(w as usize) as i16, p.sa().weapons.clip_size(w as usize) as i16), "the shared gate holds the same split");
    setup(&mut p, 6.0);
    // The kill: real P2K head rounds from slot 0 until slot 1 is dead (the shot pipeline rides
    // the rows; every tick's hash agrees).
    let mut dead = false;
    for _ in 0..12 {
        let dir = aim(p.body(0).head(), p.body(1).pos, 0.75);
        p.tick(&shot_row(0, 0x02, dir));
        p.idle(4);
        if p.sa().mp_dead_ticks[1] > 0 { dead = true; break; }
    }
    assert!(dead, "slot 1 is dead from the real shots");
    assert_eq!(p.a.sim.as_ref().unwrap().mp_hp, p.b.sim.as_ref().unwrap().mp_hp, "both machines agree on the health");
    // The 5 s death delay, then the shared respawn [M Player_HandleDeath 0x190bb0:41-54].
    let mut respawned = false;
    for _ in 0..400u32 {
        p.idle(1);
        if p.sa().mp_dead_ticks[1] == 0 && p.t > 40 { respawned = true; break; }
    }
    assert!(respawned, "slot 1 respawned");
    assert_eq!(p.sa().mp_hp[1], 100.0, "armor_bonus 0 + 100 [M 0x185368:65-73]");
    assert_eq!(p.a.sim.as_ref().unwrap().mp_hp, p.b.sim.as_ref().unwrap().mp_hp, "the machines agree on the respawned health");
    // Machine B re-armed its own player through the respawn (`MP_EquipPlayer` re-runs [M
    // 0x185368:74]). The respawn tick holds the fists 0x47 with the spawn weapon pending [M
    // 0x1855f8:50-53]; the draw settles through the normal switch before the kit assert.
    assert_eq!(p.b.sim.as_ref().unwrap().gun.weapon, 0x47, "respawn tick: the fists are held [M 0x1855f8:50-51]");
    assert_eq!(p.b.sim.as_ref().unwrap().gun.switch_request, w, "respawn tick: the spawn weapon is the switch request [M 0x1855f8:52]");
    p.wait_drawn(w);
    assert_kit(p.b.sim.as_ref().unwrap(), w, "respawn (machine B's local player, the draw settled)");
    // The shared state agrees on every machine: slot 1's gate holds the fresh kit on both, and
    // the two machines' copies are identical.
    let ga = p.a.sim.as_ref().unwrap().mp_combat.gates[1].clone();
    let gb = p.b.sim.as_ref().unwrap().mp_combat.gates[1].clone();
    assert_eq!(ga.weapon, w, "the gate's held weapon is the spawn weapon");
    assert_eq!(ga.inv.clip, gb.inv.clip, "the machines' gate clips agree");
    assert_eq!(ga.inv.carried, gb.inv.carried, "the machines' gate carried rounds agree");
    let expect = p.sa().weapons.clip_size(w as usize) as i16;
    let cs = nf_game::pickup::Inventory::clip_slot(&p.sa().weapons, w as usize);
    assert_eq!(ga.inv.clip.get(cs).copied().unwrap_or(-1), expect, "the re-seeded gate clip holds clip_size");
    assert_eq!(ga.inv.carried.get(p.sa().weapons.ammo_type(w as usize)).copied().unwrap_or(-1), expect, "the re-seeded gate carried holds clip_size");
}
