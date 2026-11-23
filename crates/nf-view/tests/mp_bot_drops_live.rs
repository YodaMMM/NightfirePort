//! mp-bot-drops: a bot that dies in a multiplayer match drops its gun like a guard does
//! (`DroneWeap_DropWeapon` 0x17a2a0 [M]), from the bot death states: a round's death enters
//! `BotDeathAnim` 0xF2 (`NDrone2_DealWithObjHit` 0x13fff0 case 0x27's MP arm) whose UPDATE (msg 3)
//! drops [M 0x175508 case 3], so the gun appears on the tick after the kill; a blast death enters
//! `BotDeathByExplosion` 0xF3 (`NDrone2_ExplosiveImpact` 0x146480's MP arm) whose ENTER drops right
//! after `MP_PlayerKilled` [M 0x1755a8]. The drop: the held weapon's base with
//! `clip_size · (0.25 + FRand(0.5))` rounds (0.5 + … below a clip of 10), one `Rand_FRand` on the
//! shared stream; `Pickup_CreateSimple`'s MP arm: 1800 ticks, spinning, no respawn [M 0x18f348].
//! It takes a reserved draw slot (mp-drops-draw), falls, can be taken, and expires. Online both
//! machines create it on the same tick and stay in lockstep. NF_ISO-gated, arena 07000023.

use nf_formats::ps2float::{add, mul};
use nf_game::mp::combat::Hurt;
use nf_game::mp::MpSettings;
use nf_net::NEUTRAL;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn step(scene: &mut nf_view::scene::Scene, n: u32) {
    for _ in 0..n {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
}

/// A live drop: running lifetime, no respawn, not a drone's, visible.
fn live_drops(sim: &nf_view::scene::Sim) -> Vec<usize> {
    (0..sim.drones.pickups.len()).filter(|&k| {
        let p = &sim.drones.pickups[k];
        !p.gone && !p.hidden && p.from.is_none() && p.respawn == 0 && p.lifetime > 0
    }).collect()
}

fn slot_mesh(scene: &nf_view::scene::Scene, k: usize) -> usize {
    scene.char_verts.iter().filter(|c| c.pickup == Some(k as u16)).count()
}

fn assert_drawn(scene: &nf_view::scene::Scene, k: usize) {
    let sim = scene.sim.as_ref().unwrap();
    let p = &sim.drones.pickups[k];
    let n = slot_mesh(scene, k);
    assert!(n > 0, "the bot's drop (slot {k}, weapon {:#x}) has drawn vertices", p.weapon);
    assert!(sim.drones.mp_drop_slots.contains(&k), "the drop sits in a reserved draw slot");
    assert_eq!(sim.placed_pickup_pose(k), Some(p.m), "drawn at the pickup's matrix");
}

fn local_match() -> nf_view::scene::Scene {
    let iso = iso().expect("NF_ISO");
    let mut ms = MpSettings::deathmatch(0x0700_0023);
    ms.bot_count = 1;
    ms.total_players = 2;
    let mut scene = nf_view::load_mp_carry(&iso, "07000023.bin", false, None, ms).expect("the arena loads");
    step(&mut scene, 120);
    scene
}

fn bullet(victim: usize) -> Hurt {
    Hurt::Bullet { victim, attacker: 0, amount: 1000.0, part: nf_game::mp::combat::part::TORSO, weapon: 6, dir: [0.0, 0.0, 1.0], at: [0.0; 3] }
}

/// (slot, held weapon, eye) of the first bot.
fn bot0(sim: &nf_view::scene::Sim) -> (usize, u8, [f32; 3]) {
    let b = &sim.mp.as_ref().unwrap().bots[0];
    (b.slot, b.brain.vars.weapon_id, b.brain.pos)
}

/// The expected rounds for a held weapon from a copy of the stream at the drop.
fn expected_rounds(sim: &nf_view::scene::Sim, w: u8, rng: &mut nf_game::drone::Rng) -> i16 {
    let n = sim.weapons.clip_size(w as usize);
    let f = add(rng.frand(0.5), if n < 10 { 0.5 } else { 0.25 });
    mul(n as f32, f) as i32 as i16
}

/// Offline: a shot bot drops its held gun on the tick after the kill (the death anim's UPDATE):
/// the base id, the FRand rounds, 1800 ticks, spinning, in a reserved drawn slot, near the bot;
/// it falls and lands, and the respawned human takes it.
#[test]
fn a_shot_bot_drops_its_gun_the_tick_after_the_kill() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match();
    let (slot, held, eye) = bot0(scene.sim.as_ref().unwrap());
    let len0 = scene.sim.as_ref().unwrap().drones.pickups.len();
    let base = scene.sim.as_ref().unwrap().weapons.base(held as usize);
    eprintln!("bot slot {slot} holds {held:#x} (base {base:#x}) at {eye:?}");
    assert!(held >= 1 && scene.sim.as_ref().unwrap().weapons.model(held as usize) != 0, "the bot holds a gun with a world model");
    assert!(live_drops(scene.sim.as_ref().unwrap()).is_empty(), "no drop before the kill");
    scene.sim.as_mut().unwrap().mp_hurt(bullet(slot), 0);
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.mp.as_ref().unwrap().bots[0].dead, "the bot is dead");
        assert!(live_drops(sim).is_empty(), "the death anim's ENTER drops nothing");
    }
    step(&mut scene, 1);
    let k = {
        let sim = scene.sim.as_ref().unwrap();
        let d = live_drops(sim);
        assert_eq!(d.len(), 1, "one visible pickup [M 0x17a2a0]");
        assert_eq!(sim.drones.pickups.len(), len0 + (base == 0x1a) as usize, "the gun takes a reserved slot (only the AR's hidden rounds append)");
        let k = d[0];
        let p = &sim.drones.pickups[k];
        let n = sim.weapons.clip_size(held as usize);
        let lo = if n < 10 { n as f32 * 0.5 } else { n as f32 * 0.25 };
        eprintln!("drop slot {k}: weapon {:#x} rounds {} (clip {n}) at {:?}", p.weapon, p.rounds, p.m[3]);
        assert_eq!((p.kind, p.weapon), (nf_game::pickup::kind::WEAPON, base), "the held weapon's base");
        assert!(p.rounds as f32 >= lo.floor() && (p.rounds as f32) < lo + n as f32 * 0.5, "clip · (0.25|0.5 + FRand(0.5))");
        // FRAME_RATE_INT · 30 [M 0x18f440-0x18f47c], and the drop tick's own `Pickup_Update`
        // already ran (the drop is made at the bots' pass, before the drones' tick).
        assert_eq!(p.lifetime, 1799, "1800 less the drop tick's update");
        assert!(p.spin && p.state == 0 && p.respawn == 0, "spinning, falling, deleted when taken");
        let at = [p.m[3][0], p.m[3][1], p.m[3][2]];
        let d = ((at[0] - eye[0]).powi(2) + (at[1] - eye[1]).powi(2) + (at[2] - eye[2]).powi(2)).sqrt();
        assert!(d < 1.5, "created at the bot's gun, near its eye ({d:.2})");
        k
    };
    assert_drawn(&scene, k);
    let r0 = scene.sim.as_ref().unwrap().drones.pickups[k].m[0];
    step(&mut scene, 120);
    {
        let sim = scene.sim.as_ref().unwrap();
        let p = &sim.drones.pickups[k];
        assert_eq!(p.state, 1, "landed");
        assert!((p.m[0][0] - r0[0]).abs() + (p.m[0][2] - r0[2]).abs() > 1e-3, "spun");
        assert_eq!(p.lifetime, 1799 - 120);
        assert_eq!(live_drops(sim).len(), 1, "the bot drops once per death");
    }
    assert_drawn(&scene, k);
    // The human walks onto it and takes it.
    let at = { let p = &scene.sim.as_ref().unwrap().drones.pickups[k]; [p.m[3][0], p.m[3][1], p.m[3][2]] };
    {
        let sim = scene.sim.as_mut().unwrap();
        let t = sim.weapons.ammo_type(base as usize);
        sim.drones.inventory.carried[t] = 0;
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, at, 0.0);
    }
    step(&mut scene, 2);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.pickups[k].gone, "taken and deleted (respawn 0)");
    assert!(sim.placed_pickup_pose(k).is_none(), "a taken drop leaves the draw list");
}

/// Offline: the drop's rounds are exactly the one FRand draw on the shared stream at the drop.
#[test]
fn the_bot_drop_draws_one_frand_for_its_rounds() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match();
    let (slot, held, _) = bot0(scene.sim.as_ref().unwrap());
    scene.sim.as_mut().unwrap().mp_hurt(bullet(slot), 0);
    // The drop is the next tick's first shared draw (the match update and the scenario sync draw
    // nothing on a plain tick).
    let mut rng = scene.sim.as_ref().unwrap().drones.rng.clone();
    let want = expected_rounds(scene.sim.as_ref().unwrap(), held, &mut rng);
    step(&mut scene, 1);
    let sim = scene.sim.as_ref().unwrap();
    let k = live_drops(sim)[0];
    assert_eq!(sim.drones.pickups[k].rounds, want, "rounds = (short)(clip · f)");
}

/// Offline: a blast death (`BotDeathByExplosion`'s ENTER) drops at once, on the kill itself.
#[test]
fn a_blasted_bot_drops_at_once() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match();
    let (slot, held, _) = bot0(scene.sim.as_ref().unwrap());
    let base = scene.sim.as_ref().unwrap().weapons.base(held as usize);
    scene.sim.as_mut().unwrap().mp_hurt(Hurt::Blast { victim: slot, attacker: 0, amount: 1000.0, weapon: 0x34, at: [0.0; 3] }, 0);
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.mp.as_ref().unwrap().bots[0].dead);
    let d = live_drops(sim);
    assert_eq!(d.len(), 1, "dropped on the kill [M 0x1755a8 ENTER]");
    assert_eq!(sim.drones.pickups[d[0]].weapon, base);
    assert_drawn(&scene, d[0]);
}

/// Offline: an untaken bot drop is deleted after exactly 1800 ticks.
#[test]
fn an_untaken_bot_drop_expires_after_30_seconds() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = local_match();
    let (slot, _, _) = bot0(scene.sim.as_ref().unwrap());
    scene.sim.as_mut().unwrap().mp_hurt(bullet(slot), 0);
    step(&mut scene, 1);
    let k = live_drops(scene.sim.as_ref().unwrap())[0];
    // Keep the human and the respawned bot away from it.
    let far = { let p = scene.sim.as_ref().unwrap().drones.pickups[k].m[3]; [p[0], p[1], p[2] + 40.0] };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, far, 0.0);
    }
    let mut life = 1u32;
    for _ in 0..2000u32 {
        step(&mut scene, 1);
        life += 1;
        let p = &scene.sim.as_ref().unwrap().drones.pickups[k];
        if p.gone { break; }
    }
    let p = &scene.sim.as_ref().unwrap().drones.pickups[k];
    assert!(p.gone);
    assert_eq!(p.lifetime, 0, "expired (not taken)");
    assert_eq!(life, 1800, "deleted on its 1800th update, the drop tick's included [M 0x18e148:258-262]");
    assert!(scene.sim.as_ref().unwrap().placed_pickup_pose(k).is_none(), "an expired drop leaves the draw list");
}

fn settings() -> MpSettings {
    let mut s = MpSettings::deathmatch(0x0700_0023);
    s.human_count = 2;
    s.player_count = 2;
    s.bot_count = 1;
    s.total_players = 3;
    s.seed = 0x0bed_0bad;
    s.frag_limit = 120;
    s.time_limit = 10 * 60;
    s
}

fn machine(path: &str, s: &MpSettings, slot: usize) -> nf_view::scene::Scene {
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(path, &arena, false, None, s.clone()).expect("arena loads");
    let sim = scene.sim.as_mut().expect("sim");
    sim.net_rng = Some(nf_game::drone::Rng::new(0x5eed_0b0b ^ (slot as u32).wrapping_mul(0x9e37_79b9)));
    sim.swap_local_slot(slot);
    scene
}

fn tick2(a: &mut nf_view::scene::Scene, b: &mut nf_view::scene::Scene, t: u32) {
    let mut i = NEUTRAL;
    i.held = 6;
    let row: [nf_net::Input; nf_net::MAX_AGENTS] = [i; nf_net::MAX_AGENTS];
    let ha = nf_view::net_tick(a.sim.as_mut().unwrap(), &row, 0);
    let hb = nf_view::net_tick(b.sim.as_mut().unwrap(), &row, 1);
    assert_eq!(ha, hb, "tick {t}: the machines agree");
}

/// Online (two sessions, loopback lockstep): the bot dies on both machines; both create the same
/// drop in the same reserved slot on the same tick and agree tick for tick while it falls, lands
/// and spins.
#[test]
fn an_online_bot_drop_is_the_same_on_both_machines() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let s = settings();
    let mut a = machine(&path, &s, 0);
    let mut b = machine(&path, &s, 1);
    for t in 0..120 { tick2(&mut a, &mut b, t); }
    let (slot, held, _) = bot0(a.sim.as_ref().unwrap());
    assert_eq!(bot0(b.sim.as_ref().unwrap()).1, held);
    for sc in [&mut a, &mut b] {
        let sim = sc.sim.as_mut().unwrap();
        sim.mp_hurt(bullet(slot), sim.mp_local_slot);
    }
    tick2(&mut a, &mut b, 120);
    let da = live_drops(a.sim.as_ref().unwrap());
    let db = live_drops(b.sim.as_ref().unwrap());
    assert_eq!(da.len(), 1, "machine a holds the bot's drop");
    assert_eq!(da, db, "machine b holds it in the same slot");
    let k = da[0];
    let (pa, pb) = (a.sim.as_ref().unwrap().drones.pickups[k].clone(), b.sim.as_ref().unwrap().drones.pickups[k].clone());
    assert_eq!((pa.weapon, pa.rounds, pa.m), (pb.weapon, pb.rounds, pb.m), "the same pickup on both machines");
    assert_eq!(pa.weapon, a.sim.as_ref().unwrap().weapons.base(held as usize));
    assert_drawn(&a, k);
    assert_drawn(&b, k);
    for t in 121..400 { tick2(&mut a, &mut b, t); }
    let (pa, pb) = (&a.sim.as_ref().unwrap().drones.pickups[k], &b.sim.as_ref().unwrap().drones.pickups[k]);
    assert_eq!((pa.state, pa.lifetime, pa.gone, pa.m), (pb.state, pb.lifetime, pb.gone, pb.m));
    assert!(pa.gone || pa.state == 1, "landed (or taken by someone on both machines)");
}
