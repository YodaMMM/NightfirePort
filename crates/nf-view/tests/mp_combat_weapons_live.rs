//! mp-combat-weapons: online combat with every MP weapon class, two machines in lockstep.
//!
//! Two headless sims (slot 0's machine and slot 1's machine — the live match-start setup: the
//! per-machine private generator, the joiner's `swap_local_slot`) tick the SAME input rows
//! through `nf_view::net_tick` (the live loop's tick shape). A shooter fires a shot description
//! (`Input::shot` + `shot_weapon` + `shot_aim`) at the other human; every tick's hash must agree
//! and the victim's health must fall by the game's MP pain arithmetic (`Player_HandlePain`
//! 0x1902c8: × Plr_DMod_Multi, the Location Damage table, armour) — per weapon class: hitscan,
//! automatic, shotgun, rocket (contact blast), guided rocket (the ride), grenade (fuse blast),
//! remote mine + detonator, melee. NF_ISO-gated (the arena loads from the disc).

use nf_game::mp::combat::{self as cb, PainRules};
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
    fn hp(&self, slot: usize) -> f32 { self.sa().mp_hp[slot] }
    /// Both machines' copies agree on every slot's health (the hash covers it; this names it).
    fn hp_agree(&self) {
        assert_eq!(self.sa().mp_hp, self.b.sim.as_ref().unwrap().mp_hp);
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
    fn rules(&self) -> PainRules {
        let s = self.sa();
        let m = s.mp.as_ref().unwrap();
        PainRules::new(&s.tuning, m.settings.location_damage != 0, m.settings.capture_reward != 0)
    }
    fn rec(&self, w: u8) -> [u8; nf_formats::weapon::RECORD] { self.sa().weapons.records[w as usize] }
}

fn f32_at(r: &[u8], o: usize) -> f32 { f32::from_le_bytes(r[o..o + 4].try_into().unwrap()) }
fn u32_at(r: &[u8], o: usize) -> u32 { u32::from_le_bytes(r[o..o + 4].try_into().unwrap()) }

/// A row where `shooter` fires `weapon` along `dir` (the shot description the live app sends).
fn shot_row(shooter: usize, weapon: u8, dir: [f32; 3]) -> [nf_net::Input; N] {
    let mut r = [NEUTRAL; N];
    r[shooter].held = weapon;
    r[shooter].shot = true;
    r[shooter].shot_weapon = weapon;
    r[shooter].shot_aim = cb::aim_from_dir(dir);
    r
}

/// Two open spots `dist` apart with a clear line between the heads, on 07000023: scanned from the
/// slot spawns outward.
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
            // Clear at the heads AND the knees (grenades roll along the floor), and some room
            // behind the victim (a contact blast's wall).
            if s.world.segment(ha, hb).is_none() && s.world.segment(feet(&a0), feet(&b0)).is_none() {
                return (a0.pos, b0.pos);
            }
        }
    }
    panic!("no clear duel line {dist} on the arena");
}

fn setup(p: &mut Pair, dist: f32) -> ([f32; 3], [f32; 3]) {
    let (a, b) = duel_spots(p, dist);
    let yaw_ab = (b[0] - a[0]).atan2(b[2] - a[2]);
    p.place(0, [a[0], a[1] - 1.1, a[2]], yaw_ab);
    p.place(1, [b[0], b[1] - 1.1, b[2]], yaw_ab + std::f32::consts::PI);
    p.idle(2);
    (p.body(0).pos, p.body(1).pos)
}

/// The aim from `from` to a point `dy` above the victim's `pos` (Bond::pos — the chest is ~+0.2).
fn aim(from: [f32; 3], to: [f32; 3], dy: f32) -> [f32; 3] {
    let d = [to[0] - from[0], to[1] + dy - from[1], to[2] - from[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    [d[0] / l, d[1] / l, d[2] / l]
}

#[test]
fn dump_records() {
    let Some(p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    for w in [0x01u8, 0x63, 0x64, 0x65, 0x66, 0x4b, 0x0e, 0x0a, 0x02, 0x12, 0x16, 0x1c, 0x1d, 0x1e, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x34, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x47, 0x4a] {
        let r = p.rec(w);
        eprintln!("w {w:#04x} base {:#04x} rad {} dmg {} pellets {} range {} speed {} spread {} f68 {:#x} f6c {:#x} f70 {:#x}",
            u16::from_le_bytes([r[2], r[3]]), f32_at(&r, 8), f32_at(&r, 0xc), u32_at(&r, 0x18), f32_at(&r, 0x1c), f32_at(&r, 0x20), f32_at(&r, 0x24), u32_at(&r, 0x68), u32_at(&r, 0x6c), u32_at(&r, 0x70));
    }
    let t = &p.sa().tuning;
    eprintln!("tuning multi {} head {} upper {} lower {}", t.Plr_DMod_Multi, t.Plr_DMod_Head, t.Plr_DMod_UpperLimb, t.Plr_DMod_LowerLimb);
}

/// Fire `n` shots of `weapon` from slot 0 at slot 1 (one per tick, aimed `dy` above the victim's
/// `Bond::pos`), then let the rounds land. Returns slot 1's health drop.
fn volley(p: &mut Pair, weapon: u8, n: u32, dy: f32, settle: u32) -> f32 {
    let before = p.hp(1);
    for _ in 0..n {
        let dir = aim(p.body(0).head(), p.body(1).pos, dy);
        p.tick(&shot_row(0, weapon, dir));
    }
    p.idle(settle);
    p.hp_agree();
    before - p.hp(1)
}

#[test]
fn hitscan_torso_head_armour_and_kill_credit() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    setup(&mut p, 6.0);
    let r = p.rules();
    let d = f32_at(&p.rec(0x02), 0x0c);
    // A torso round: × Plr_DMod_Multi [M 0x1902c8:55-57].
    let drop = volley(&mut p, 0x02, 1, 0.2, 4);
    assert_eq!(drop, cb::mp_damage(&r, d, cb::part::TORSO), "one P2K torso round");
    assert_eq!(drop, 14.0, "3.5 × 4 on the arena tuning");
    // Armour absorbs first [M :82-104].
    for (sim, _) in p.sims() { sim.mp_armour[1] = 20.0; }
    let drop = volley(&mut p, 0x02, 1, 0.2, 4);
    assert_eq!(drop, 0.0, "the armour took the round");
    assert_eq!(p.sa().mp_armour[1], 6.0);
    // A head round: × Multi × Head with Location Damage on (the boot default) [M :62-78].
    let drop = volley(&mut p, 0x02, 1, 0.75, 4);
    assert_eq!(drop, 56.0 - 6.0, "3.5 × 4 × 4, the last 6 armour first");
    // The kill: the next head round, credited to the shooter through player_killed.
    let kills = p.sa().mp.as_ref().unwrap().game.slots[0].kills;
    volley(&mut p, 0x02, 1, 0.75, 4);
    assert!(p.sa().mp_dead_ticks[1] > 0, "slot 1 is dead");
    let m = p.sa().mp.as_ref().unwrap();
    assert_eq!(m.game.slots[0].kills, kills + 1, "the kill is the shooter's");
    assert_eq!(m.game.slots[1].deaths, 1);
    // Sister: a miss (aimed a metre over the head) deals nothing.
    p.idle(320); // the respawn
    setup(&mut p, 6.0);
    assert_eq!(volley(&mut p, 0x02, 1, 2.0, 4), 0.0, "a miss");
}

#[test]
fn automatic_volley() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    setup(&mut p, 6.0);
    let r = p.rules();
    let per = cb::mp_damage(&r, f32_at(&p.rec(0x12), 0x0c), cb::part::TORSO);
    // mp-shot-gate: the shared gate enforces the game's fire rate ([M] Player_WeaponFiring
    // 0x1a5460: the +0x92c refire timer resets to max(weapon_data+0x40, 1.0) per shot), so the
    // volley sends its 10 releases at the record's interval (6 frames [D +0x40]) — the cadence
    // the sender's own pipeline produces, one shot description per release.
    let interval = u32::from_le_bytes(p.rec(0x12)[0x40..0x44].try_into().unwrap()) as u32;
    let before = p.hp(1);
    for k in 0..10u32 {
        for _ in 0..interval {
            let dir = aim(p.body(0).head(), p.body(1).pos, 0.2);
            p.tick(&shot_row(0, 0x12, dir));
        }
        let _ = k;
    }
    p.idle(4);
    p.hp_agree();
    let drop = before - p.hp(1);
    eprintln!("storm: 10 rounds at the interval, drop {drop} (per round {per})");
    assert!(drop > 0.0 && (drop / per).fract() == 0.0, "whole torso rounds: {drop} / {per}");
    assert!(drop >= per * 8.0, "most of the spread rounds land at 6 u");
}

#[test]
fn shotgun_pellets() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    setup(&mut p, 4.0);
    let r = p.rules();
    let rec = p.rec(0x1c);
    assert_eq!(u32_at(&rec, 0x18), 8, "the Frinesi fires 8 pellets [L +0x18]");
    let per = cb::mp_damage(&r, f32_at(&rec, 0x0c), cb::part::TORSO);
    let draws0 = p.sa().drones.rng.draws;
    let drop = volley(&mut p, 0x1c, 1, 0.2, 6);
    eprintln!("frinesi: drop {drop} (per pellet {per}), shared draws {}", p.sa().drones.rng.draws - draws0);
    assert!(drop > per && (drop / per).fract() == 0.0, "several whole pellets: {drop} / {per}");
}

#[test]
fn melee_fists() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    setup(&mut p, 1.2);
    let r = p.rules();
    let drop = volley(&mut p, 0x01, 1, 0.2, 4);
    assert_eq!(drop, cb::mp_damage(&r, 5.0, cb::part::TORSO), "a punch: the fists' 5.0 × Multi");
    // Sister: out of reach (the record's 1.75 range) the punch lands nowhere.
    setup(&mut p, 4.0);
    assert_eq!(volley(&mut p, 0x01, 1, 0.2, 4), 0.0, "out of reach");
}

/// Ticks until a blast happens (bounded); returns it with both bodies' positions before that
/// tick and both slots' health drops on it.
fn until_blast(p: &mut Pair, row: &[nf_net::Input; N], max: u32) -> (cb::Blast, nf_game::player::Bond, nf_game::player::Bond, f32, f32) {
    for _ in 0..max {
        let (h0, h1) = (p.hp(0), p.hp(1));
        let (b0, b1) = (p.body(0), p.body(1));
        p.tick(row);
        if let Some(bl) = p.sa().mp_combat.blasts.first().copied() {
            assert_eq!(p.b.sim.as_ref().unwrap().mp_combat.blasts, p.sa().mp_combat.blasts, "both machines blast alike");
            return (bl, b0, b1, h0 - p.hp(0), h1 - p.hp(1));
        }
    }
    panic!("no blast within {max} ticks");
}

/// The blast falloff the game applies to a body (`Explode_Propagate` 0x17c518 → `Player_Hurt`:
/// the body's world sphere, no part multiplier), capped by the health it had.
fn blast_amount(p: &Pair, bl: &cb::Blast, slot: usize, body: &nf_game::player::Bond, hp: f32) -> f32 {
    let b = cb::Body { slot, pos: body.pos, head_y: body.head()[1], yaw: body.yaw, alive: true };
    let a = cb::mp_damage(&p.rules(), b.blast_amount(bl), cb::part::NONE);
    if hp - a < 1.0 { hp } else { a }
}

#[test]
fn rocket_contact_blast() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    let (a, b) = setup(&mut p, 7.0);
    // The Sentinel Unguided 0x2D at the floor 2 u short of the victim: the world contact
    // blasts (+0x70 & 4 [M 0x12ce80:194-206]) with both players inside the 8 u radius.
    let d = [b[0] - a[0], b[2] - a[2]];
    let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
    let spot = [b[0] - d[0] / l * 2.0, b[1] - 1.1, b[2] - d[1] / l * 2.0];
    let head = p.body(0).head();
    let dir = aim(head, spot, 0.0);
    p.tick(&shot_row(0, 0x2d, dir));
    assert_eq!(p.sa().mp_combat.rounds.len(), 1, "one rocket in flight");
    let (h0, h1) = (p.hp(0), p.hp(1));
    let (bl, b0, b1, d0, d1) = until_blast(&mut p, &[NEUTRAL; N], 600);
    eprintln!("rocket blast at {:?} r {} dmg {}: shooter -{d0}, victim -{d1}", bl.pos, bl.radius, bl.damage);
    assert_eq!(bl.weapon, 0x2d);
    assert_eq!(d1, blast_amount(&p, &bl, 1, &b1, h1), "the victim's falloff damage (no part multiplier)");
    assert_eq!(d0, blast_amount(&p, &bl, 0, &b0, h0), "the shooter's own blast hurts him too");
    assert!(d1 > 0.0 && d0 > 0.0);
    p.hp_agree();
    // A direct hit: the rocket's own round (100 × Multi) kills outright.
    p.idle(320);
    setup(&mut p, 7.0);
    let deaths = p.sa().mp.as_ref().unwrap().game.slots[1].deaths;
    let dir = aim(p.body(0).head(), p.body(1).pos, 0.2);
    p.tick(&shot_row(0, 0x2d, dir));
    p.idle(200);
    assert_eq!(p.sa().mp.as_ref().unwrap().game.slots[1].deaths, deaths + 1, "a direct rocket kills");
}

#[test]
fn guided_rocket_ride_and_homing() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    setup(&mut p, 7.0);
    // The Sentinel Guided 0x2C: the owner rides it — every machine zeroes slot 0's movement
    // while the ride (and its 15-tick restore) lasts [M Bullet_init 0x12c5a0:234-254].
    let dir = aim(p.body(0).head(), p.body(1).pos, 3.0); // up and over: a longer flight
    p.tick(&shot_row(0, 0x2c, dir));
    assert!(p.sa().mp_combat.riding(0) && p.b.sim.as_ref().unwrap().mp_combat.riding(0));
    assert!(p.sa().missile_cam.is_some(), "the shooter's machine rides the shared rocket");
    assert!(p.b.sim.as_ref().unwrap().missile_cam.is_none(), "the other machine does not");
    let start = p.body(0).pos;
    let mut row = [NEUTRAL; N];
    row[0].forward = 1.0; // the rider's stick steers nothing but the rocket
    row[0].turn = 0.01;
    let mut ticks = 0;
    while p.sa().mp_combat.riding(0) && ticks < 1200 { p.tick(&row); ticks += 1; }
    let moved = p.body(0).pos;
    assert!(ticks > 15, "the ride lasted {ticks} ticks");
    assert_eq!(start, moved, "the riding body held still on both machines (the hashes agreed)");
    for _ in 0..5 { p.tick(&row); }
    assert_ne!(p.body(0).pos, moved, "after the restore the body walks again");
    // The Scorpion Heatseeker 0x2E fired 20° off the victim: `Bullet_homing` 0x12f358 locks it
    // and steers it in [M]. A straight round on that line passes ~3 u wide at 9 u.
    p.idle(320);
    setup(&mut p, 9.0);
    let head = p.body(0).head();
    let vpos = p.body(1).pos;
    let to = aim(head, vpos, 0.0);
    let (s, c) = (0.349f32.sin(), 0.349f32.cos());
    let off = [to[0] * c - to[2] * s, to[1], to[0] * s + to[2] * c];
    let straight = {
        let w = [vpos[0] - head[0], vpos[1] - head[1], vpos[2] - head[2]];
        let t = w[0] * off[0] + w[1] * off[1] + w[2] * off[2];
        let q = [w[0] - off[0] * t, w[1] - off[1] * t, w[2] - off[2] * t];
        (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt()
    };
    p.tick(&shot_row(0, 0x2e, off));
    let mut target = None;
    let mut closest = f32::MAX;
    for _ in 0..600 {
        p.tick(&[NEUTRAL; N]);
        let Some(r) = p.sa().mp_combat.rounds.first() else { break };
        target = target.or(r.target);
        let d = ((r.pos[0] - vpos[0]).powi(2) + (r.pos[1] - vpos[1]).powi(2) + (r.pos[2] - vpos[2]).powi(2)).sqrt();
        closest = closest.min(d);
    }
    eprintln!("heatseeker: target {target:?}, closest {closest:.2} (straight line {straight:.2}), victim hp {}", p.hp(1));
    assert_eq!(target, Some(1), "the heatseeker locked the victim");
    assert!(closest < 1.0 && closest < straight * 0.5, "it steered in: {closest} vs the straight {straight}");
    // A heatseeker 5° off at 9 u: steered into the body.
    p.idle(320);
    setup(&mut p, 9.0);
    let to = aim(p.body(0).head(), p.body(1).pos, 0.2);
    let (s, c) = (0.087f32.sin(), 0.087f32.cos());
    let off = [to[0] * c - to[2] * s, to[1], to[0] * s + to[2] * c];
    let deaths = p.sa().mp.as_ref().unwrap().game.slots[1].deaths;
    p.tick(&shot_row(0, 0x2e, off));
    p.idle(300);
    assert_eq!(p.sa().mp.as_ref().unwrap().game.slots[1].deaths, deaths + 1, "the heatseeker's direct hit kills");
}

#[test]
fn frag_grenade_fuse_blast() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    let (_, b) = setup(&mut p, 6.0);
    // The frag 0x34 lobbed at the victim's feet: it bounces, sits and blasts on its 210-tick fuse
    // [M Bullet_init 0x12c5a0:103; Bullet_DoTrails 0x12d890:262-281].
    let dir = aim(p.body(0).head(), b, 0.5);
    p.tick(&shot_row(0, 0x34, dir));
    assert_eq!(p.sa().mp_combat.thrown.len(), 1);
    let t0 = p.t;
    let (h0, h1) = (p.hp(0), p.hp(1));
    let (bl, b0, b1, d0, d1) = until_blast(&mut p, &[NEUTRAL; N], 400);
    eprintln!("frag blast after {} ticks at {:?}: shooter -{d0}, victim -{d1}", p.t - t0, bl.pos);
    assert!((205..=215).contains(&(p.t - t0)), "the 210-tick fuse ({})", p.t - t0);
    assert_eq!(d1, blast_amount(&p, &bl, 1, &b1, h1));
    assert_eq!(d0, blast_amount(&p, &bl, 0, &b0, h0));
    assert!(d1 > 0.0);
    // The MGL Grenade 0x2A blasts on contact [M 0x12ce80:194-206].
    p.idle(320);
    setup(&mut p, 6.0);
    let dir = aim(p.body(0).head(), p.body(1).pos, 0.6);
    p.tick(&shot_row(0, 0x2a, dir));
    let t0 = p.t;
    let (bl, _, _, _, d1) = until_blast(&mut p, &[NEUTRAL; N], 300);
    eprintln!("mgl contact blast after {} ticks at {:?}: victim -{d1}", p.t - t0, bl.pos);
    assert!(p.t - t0 < 100, "contact, not a fuse");
    assert!(d1 > 0.0, "the victim is caught");
}

#[test]
fn remote_mine_and_detonator() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    let (_, b) = setup(&mut p, 4.0);
    // The remote mine 0x37 thrown at the floor by the victim: it sticks and waits (the player's
    // 7777.0 fuse [M 0x12c5a0:214-223]).
    let dir = aim(p.body(0).head(), b, -1.0);
    p.tick(&shot_row(0, 0x37, dir));
    p.idle(400);
    let g = p.sa().mp_combat.thrown.first().expect("the mine is placed").clone();
    assert!(g.stuck, "the mine stuck");
    assert!(p.sa().mp_combat.blasts.is_empty() && p.hp(1) == 100.0, "no blast without the detonator");
    // Somebody else's detonator does nothing to it.
    p.tick(&shot_row(1, 0x38, [0.0, 0.0, 1.0]));
    p.idle(3);
    assert_eq!(p.sa().mp_combat.thrown.len(), 1, "slot 1's detonator leaves slot 0's mine");
    // The owner's detonator (0x38, +0x68 & 0x20000): BU+0xFC = −10.0 → the blast next tick
    // [M Player_WeaponInitBullet 0x1a4bf0 disasm 0x1a4c88].
    let row = shot_row(0, 0x38, [0.0, 0.0, 1.0]);
    let t0 = p.t;
    let h1 = p.hp(1);
    let (bl, _, b1, _, d1) = until_blast(&mut p, &row, 5);
    eprintln!("mine blast after {} ticks: victim -{d1}", p.t - t0);
    assert_eq!(bl.weapon, 0x37);
    assert_eq!(d1, blast_amount(&p, &bl, 1, &b1, h1));
    assert!(d1 > 0.0 && p.t - t0 <= 2);
}

#[test]
fn friendly_fire_gate() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = settings();
    s.scenario = nf_game::mp::scenario::TEAM_DEATHMATCH;
    s.gametype_mask = s.scenario;
    s.team_game = 1;
    s.players[0].team = 0;
    s.players[1].team = 0;
    s.friendly_fire = 0;
    let mut p = Pair { a: machine(&path, &s, 0), b: machine(&path, &s, 1), t: 0 };
    setup(&mut p, 6.0);
    assert_eq!(volley(&mut p, 0x02, 1, 0.2, 4), 0.0, "friendly fire off: the team-mate's round is blocked [M 0x1857c0:43-45]");
    // Sister: friendly fire on, the same round hurts.
    for (sim, _) in p.sims() { sim.mp.as_mut().unwrap().settings.friendly_fire = 1; }
    assert_eq!(volley(&mut p, 0x02, 1, 0.2, 4), 14.0);
}

/// The live pipeline end to end: slot 0's machine holds the trigger with the start P2K; each
/// tick's `Sim::net_shot` rides the NEXT row (the app's one-tick lag) to both machines.
#[test]
fn live_pipeline_p2k() {
    let Some(mut p) = Pair::new() else { eprintln!("skip: NF_ISO not set"); return };
    setup(&mut p, 6.0);
    // Face the victim exactly (the P2K's aim = the view when not free-aiming).
    let (h0, b1) = (p.body(0).head(), p.body(1).pos);
    let dir = aim(h0, b1, 0.2);
    let yaw = dir[0].atan2(dir[2]);
    let pitch = dir[1].asin();
    for (sim, local) in p.sims() {
        let b = if local == 0 { &mut sim.bond } else { sim.mp_bonds[0].as_mut().unwrap() };
        b.yaw = yaw;
        b.pitch = pitch;
    }
    let mut pending: Option<nf_view::scene::NetShot> = None;
    let mut shots = 0;
    for t in 0..240 {
        let mut row = [NEUTRAL; N];
        row[0].trigger = t % 20 < 2; // taps (the P2K is semi-auto)
        row[0].held = p.sa().gun.weapon;
        if let Some(sh) = pending.take() {
            row[0].shot = true;
            row[0].shot_weapon = sh.weapon;
            row[0].shot_aim = cb::aim_from_dir(sh.dir);
            shots += 1;
        }
        p.tick(&row);
        pending = p.a.sim.as_mut().unwrap().net_shot.take();
        assert!(p.b.sim.as_mut().unwrap().net_shot.take().is_none(), "machine B's local slot never fired");
        if p.sa().mp_dead_ticks[1] > 0 { break; }
    }
    eprintln!("pipeline: {shots} shots, victim hp {} dead {}", p.hp(1), p.sa().mp_dead_ticks[1]);
    assert!(shots >= 3, "the local pipeline released rounds ({shots})");
    assert!(p.hp(1) < 100.0 || p.sa().mp_dead_ticks[1] > 0, "and they hurt the other machine's player");
}
