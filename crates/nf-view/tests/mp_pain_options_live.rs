//! mp-human-pain: the Player Mods options and the bot's armour pool through the real match pain
//! path. Two headless sims (the mp-combat-weapons harness shape) tick the SAME input rows through
//! `nf_view::net_tick`; every tick's hashes must agree, so the damage lands identically on both
//! machines. Pins:
//! - a head round with Location Damage ON scales by the body part (× Plr_DMod_Head) [M
//!   `Player_HandlePain` 0x1902c8:54-78], and the SAME round with the option OFF does not
//!   ([M :54] — the whole block keys on `MPSettings._456_4_`);
//! - Professional Mode (+0x1BC) triples the hit [M :80-82];
//! - a shot on an armoured player spends the armour first [M :100-104];
//! - a shot on an armoured BOT spends the bot's armour pool (`BOT_vars+0x769`) first, the health
//!   untouched while the pool covers the hit [M `BOT_handlePain` 0x125ed8:80-92].
//! NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::combat::{self as cb};
use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;
use nf_view::scene::{Scene, Sim};

const N: usize = nf_net::MAX_AGENTS;

fn iso() -> Option<String> {
    std::env::var("NF_ISO").ok()
}

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.total_players = 2;
    s.seed = 0x0bed_0bad;
    s.time_limit = 10 * 60;
    s.frag_limit = 50;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
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
    fn new() -> Option<Pair> {
        let path = iso()?;
        let s = settings();
        Some(Pair { a: machine(&path, &s, 0), b: machine(&path, &s, 1), t: 0 })
    }
    fn sa(&self) -> &Sim { self.a.sim.as_ref().unwrap() }
    fn sims(&mut self) -> [(&mut Sim, usize); 2] {
        [(self.a.sim.as_mut().unwrap(), 0), (self.b.sim.as_mut().unwrap(), 1)]
    }
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
    fn hp(&self, slot: usize) -> f32 { self.sa().mp_hp[slot] }
    fn place(&mut self, slot: usize, pos: [f32; 3], yaw: f32) {
        for (sim, local) in self.sims() {
            let b = nf_game::player::Bond::stand_at(&sim.world, [pos[0], pos[1] + 0.5, pos[2]], yaw);
            let bp = b.pos;
            if slot == local { sim.bond = b; } else { sim.mp_bonds[slot] = Some(b); }
            sim.mp.as_mut().unwrap().set_slot(slot, bp);
        }
    }
    fn rec(&self, w: u8) -> [u8; nf_formats::weapon::RECORD] { self.sa().weapons.records[w as usize] }
}

fn f32_at(r: &[u8], o: usize) -> f32 { f32::from_le_bytes(r[o..o + 4].try_into().unwrap()) }

fn shot_row(shooter: usize, weapon: u8, dir: [f32; 3]) -> [nf_net::Input; N] {
    let mut r = [NEUTRAL; N];
    r[shooter].held = weapon;
    r[shooter].shot = true;
    r[shooter].shot_weapon = weapon;
    r[shooter].shot_aim = cb::aim_from_dir(dir);
    r
}

/// Two open spots 6 u apart with a clear line between the heads (the mp-combat-weapons scan).
fn duel_spots(p: &Pair) -> ([f32; 3], [f32; 3]) {
    let s = p.sa();
    let m = s.mp.as_ref().unwrap();
    for sp in m.spawns.points.iter().filter(|sp| sp.used != 0) {
        for k in 0..16 {
            let a = k as f32 * std::f32::consts::TAU / 16.0;
            let a0 = nf_game::player::Bond::stand_at(&s.world, [sp.pos[0], sp.pos[1], sp.pos[2]], 0.0);
            let far = [sp.pos[0] + 6.0 * a.sin(), sp.pos[1], sp.pos[2] + 6.0 * a.cos()];
            let b0 = nf_game::player::Bond::stand_at(&s.world, far, 0.0);
            if (b0.pos[1] - a0.pos[1]).abs() > 0.2 { continue; }
            let (ha, hb) = (a0.head(), b0.head());
            let feet = |b: &nf_game::player::Bond| [b.pos[0], b.pos[1] - 0.9, b.pos[2]];
            if s.world.segment(ha, hb).is_none() && s.world.segment(feet(&a0), feet(&b0)).is_none() {
                return (a0.pos, b0.pos);
            }
        }
    }
    panic!("no clear duel line on the arena");
}

fn setup(p: &mut Pair) -> ([f32; 3], [f32; 3]) {
    let (a, b) = duel_spots(p);
    let yaw_ab = (b[0] - a[0]).atan2(b[2] - a[2]);
    p.place(0, [a[0], a[1] - 1.1, a[2]], yaw_ab);
    p.place(1, [b[0], b[1] - 1.1, b[2]], yaw_ab + std::f32::consts::PI);
    p.idle(2);
    (p.body(0).pos, p.body(1).pos)
}

/// The aim from `from` to a point `dy` above the victim's `pos`.
fn aim(from: [f32; 3], to: [f32; 3], dy: f32) -> [f32; 3] {
    let d = [to[0] - from[0], to[1] + dy - from[1], to[2] - from[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    [d[0] / l, d[1] / l, d[2] / l]
}

/// One P2K (0x02) round from slot 0 at `dy` above slot 1's pos; returns slot 1's health drop.
fn one_round(p: &mut Pair, dy: f32) -> f32 {
    let before = p.hp(1);
    let dir = aim(p.body(0).head(), p.body(1).pos, dy);
    p.tick(&shot_row(0, 0x02, dir));
    p.idle(4);
    assert_eq!(p.sa().mp_hp, p.b.sim.as_ref().unwrap().mp_hp, "both machines agree on the health");
    before - p.hp(1)
}

/// The same shot with Location Damage ON scales by the part (the head ×4) and with it OFF does
/// not [M 0x1902c8:54-78 — the block keys on `MPSettings._456_4_`].
#[test]
fn location_option_scales_the_head_and_off_does_not() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    let d = f32_at(&p.rec(0x02), 0x0c);
    setup(&mut p);
    // The boot default: Location Damage on [M bootup_bootup 0x1c7e78 @0x1c7ff8].
    assert_eq!(p.sa().mp.as_ref().unwrap().settings.location_damage, 1);
    let on = one_round(&mut p, 0.75); // the head (the top 0.3 of the body [S mp-combat.md])
    assert_eq!(on, d * 4.0 * 4.0, "the head round: × Plr_DMod_Multi × Plr_DMod_Head");
    // The option off: the same round, no part multiplier — the × Plr_DMod_Multi remains [M :51-53].
    for (sim, _) in p.sims() { sim.mp.as_mut().unwrap().settings.location_damage = 0; }
    let off = one_round(&mut p, 0.75);
    assert_eq!(off, d * 4.0, "the same head round without the option: no × Plr_DMod_Head");
    assert_ne!(on, off, "the option changes the same shot");
}

/// Professional Mode (+0x1BC, the port's `capture_reward`) triples the hit [M 0x1902c8:80-82].
#[test]
fn professional_mode_triples() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    let d = f32_at(&p.rec(0x02), 0x0c);
    setup(&mut p);
    for (sim, _) in p.sims() { sim.mp.as_mut().unwrap().settings.capture_reward = 1; }
    let drop = one_round(&mut p, 0.2); // the torso: no part multiplier
    assert_eq!(drop, d * 4.0 * 3.0, "the torso round with Professional Mode: × Multi × 3");
}

/// A shot on an armoured player spends the armour first [M 0x1902c8:100-104]: the pool pays what
/// it covers, the health the remainder.
#[test]
fn armour_spends_first() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    let d = f32_at(&p.rec(0x02), 0x0c);
    setup(&mut p);
    for (sim, _) in p.sims() { sim.mp_armour[1] = 10.0; }
    let drop = one_round(&mut p, 0.2);
    assert_eq!(drop, d * 4.0 - 10.0, "the pool paid its 10 first, the health the remainder");
    assert_eq!(p.sa().mp_armour[1], 0.0, "the pool drained");
}

/// A hit on an armoured BOT spends the bot's armour pool first, the health untouched while the
/// pool covers the hit [M `BOT_handlePain` 0x125ed8:80-92 — `min(d, BOT_vars+0x769)`], through
/// the live match's shared hurt path on both machines (the mp_death_effects shape).
#[test]
fn armoured_bot_spends_the_pool() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = settings();
    s.bot_count = 1;
    s.total_players = 3;
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    let mut p = Pair { a, b, t: 0 };
    let d = f32_at(&p.rec(0x02), 0x0c);
    setup(&mut p);
    p.idle(30); // the bots arm on their first ticks
    let slot = p.sa().mp.as_ref().unwrap().bots[0].slot;
    let dead = p.sa().mp.as_ref().unwrap().bots[0].dead;
    assert!(!dead, "the bot is live");
    // Park the bot's armour pool on both machines (the take's case-3 shape [M 0x18e650:119-133]).
    for sim in [&mut p.a, &mut p.b] {
        let s = sim.sim.as_mut().unwrap();
        if let Some(bt) = s.mp.as_mut().unwrap().bots.first_mut() {
            bt.brain.vars.arsenal.armour = 40;
        }
    }
    let hp0 = p.sa().mp.as_ref().unwrap().bots[0].health;
    // The round's hit through the shared hurt path — the bullet route (a located part-4 hit,
    // pain type 0), applied identically on both machines.
    let hurt = cb::Hurt::Bullet { victim: slot, attacker: 0, amount: d, part: cb::part::TORSO, weapon: 0x02, dir: [0.0, 0.0, 1.0], at: [0.0; 3] };
    p.a.sim.as_mut().unwrap().mp_hurt(hurt, 0);
    p.b.sim.as_mut().unwrap().mp_hurt(hurt, 1);
    let m = p.sa().mp.as_ref().unwrap();
    assert_eq!(m.bots[0].brain.vars.arsenal.armour, 40 - (d * 4.0) as u8, "the pool paid the scaled damage");
    assert_eq!(m.bots[0].health, hp0, "the health is untouched while the pool covers it");
    // The state agrees (the same pool on both machines, the next tick's hash asserts it too).
    assert_eq!(
        p.sa().mp.as_ref().unwrap().bots[0].brain.vars.arsenal.armour,
        p.b.sim.as_ref().unwrap().mp.as_ref().unwrap().bots[0].brain.vars.arsenal.armour,
    );
    p.idle(1);
}
