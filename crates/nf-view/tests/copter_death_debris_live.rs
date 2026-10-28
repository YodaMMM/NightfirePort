//! copter-death-debris: destroying the copter creates the game's death blast and its metal
//! debris. The chain [M]:
//! - the death tail (`Copter_Update` 0x17dbb8:358-370) sends obj+0xF4 to 6 on
//!   GameState+0x12 ∈ {0x0700000b, 0x07000012} — the explode levels (the port's own death test
//!   already pins the transition); off them the death departs (state 4) and creates NOTHING;
//! - case 6 (0x17e0e0-0x17e178) writes the arrival channel, calls `Explode_Create` (0x17e120-
//!   0x17e168: radius 10.0 / damage 67.4 / script 0x0600052 / debris kind 6 / attacker NULL, at
//!   obj+0x30) and sets obj+0xFE bit 1 — the object loop then deletes the copter;
//! - `Explode_Update` 0x17c350:34-67 throws the debris: while the blast script (0x0600052,
//!   lifetime 165 [M 0x17cd18]) is young (`lifetime >> 2 >= age`) and
//!   `GameState_time % (Rand(5)+10) == 0`, kind 6 picks `MetalHash_132` and calls
//!   `Debris_CreateEx(2·radius, …, 180, 3, …)` — 3 particles per passing tick.
//!
//! The test drives the real sim: the real level (0700000b — the roof helicopter #328, gate
//! 0x2A [D the level census]), plain 60 Hz steps, and the damage delivered through the game's
//! own blast-propagate entry (`Explode_Propagate` case 0 adds the falloff straight into
//! rec+0x64 [M 0x17c518:86-105]) the way the port's frag grenade delivers it — the grenade
//! path's detonation lands in the same `pending_blasts` slot with the frag's record values
//! radius 8 / damage 100 [L weapon record 0x34 'Fragmentation Grenade', the port's weapon
//! census] and debris kind 0 (a grenade blast spawns no debris of its own [M the grenade
//! drain's `sw` of 0 — drone.rs:18702's 8th tuple slot]). The gate poke stands in for the
//! level's own ch-0x2A writer (the roof arrival script at 0x990 [D the census]; the poke is the
//! only non-plain input here).
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_game::explode::METAL_HASH_132;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn frag_at(sim: &mut nf_view::scene::Sim, pos: [f32; 3]) {
    // The frag grenade's detonation record [L record 0x34: radius 8, damage 100; the port's
    // grenade drain pushes (pos, 8.0, 100.0, owner, weapon 0x34, script 0x0600052, scale 8.0,
    // debris 0) — the same slot this test feeds].
    sim.drones.pending_blasts.push((pos, 8.0, 100.0, usize::MAX, 0, 0x0600_0052, 8.0, 0));
}

#[test]
fn destroying_the_0b_roof_copter_creates_the_kind6_death_blast_and_the_metal_debris() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "0700000b.bin", false).expect("load 0700000b");
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.level, 0x0700_000b, "the explode level [M Copter_Update 0x17dbb8:360-363]");
    // The roof helicopter: gate 0x2A, out 0x66, flight script 060008BE [D the 0700000b census].
    let ci = sim.drones.copters.iter().position(|c| c.gate == 0x2a).expect("the gate-0x2A copter parses");
    let placement = sim.drones.copters[ci].placement;
    let hp = sim.drones.copters[ci].hp;
    eprintln!("[0b-death] copter {ci} placement {placement} hp {hp} gate 0x2a");
    // The level's own gate write (the poke stands in for the 0x990 roof-arrival writer [D]).
    sim.drones.channels.set(0x2a, 1, sim.drones.frame);
    for _ in 0..20 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.copters.is_empty(), "the gate-on copter updates");
    let (targets, present) = sim.drones.scripts.copter_part_targets(placement, &sim.data());
    eprintln!("[0b-death] parts present {present:04x}: {:?}", targets.iter().flatten().collect::<Vec<_>>());
    assert!(present != 0, "the copter's script entities pose — the damage sweep's targets [M Explode_Propagate case 0 walks the SP part spheres]");

    // Destroy it: frag detonations at the body until the death tail takes over. The falloff
    // lands in rec+0x64 [M 0x17c518:86-105], the tail reads it against hp [M 0x17dbb8:358-370].
    let mut death_pos = None;
    for i in 0..14 {
        let Some(c) = sim.drones.copters.get(ci) else { break };
        let pos = c.pos();
        if hp < c.damage {
            // The damage landed after this tick's copter update (the blast drain runs behind the
            // walk) — the death tail reads it on the NEXT update [M the walk order: the copter
            // update, then Collide/propagate]. One more step takes the state to 6.
            sim.step_bond(Default::default(), false, false, false);
            let c = &sim.drones.copters[ci];
            assert_eq!(c.state, 6, "the death on 0x0700000b goes to 6 [M 0x17e3b0-0x17e3b8]");
            death_pos = Some(c.pos());
            break;
        }
        frag_at(sim, pos);
        sim.step_bond(Default::default(), false, false, false);
        if i == 13 { panic!("the copter survived {} frag blasts (hp {hp})", i + 1); }
    }
    let death_pos = death_pos.expect("the death tail ran");
    // The transition tick creates nothing yet — the case-6 dispatch is the NEXT update.
    assert!(sim.drones.blasts.iter().all(|b| b.debris != 6), "no death blast before the case-6 dispatch [M the tail's stores, then return]");

    // The next tick: case 6 → the Explode_Create. The copter record leaves the list
    // (obj+0xFE bit 1 → the object loop's delete [M 0x17e16c-0x17e178]) and the blast object
    // appears with the call-site's exact values [M 0x17e120-0x17e168].
    sim.step_bond(Default::default(), false, false, false);
    assert!(sim.drones.copters.iter().all(|c| c.placement != placement), "the record is deleted [M the object loop]");
    let b = sim.drones.blasts.iter().find(|b| b.debris == 6).expect("the death blast exists");
    assert_eq!(b.script_hash, 0x0600_0052, "script 0x0600052 [M 0x17e130/0x17e154]");
    assert_eq!(b.radius, 10.0, "f13 = 0x41200000 [M 0x17e120-0x17e124, 0x17e150]");
    assert_eq!(b.damage, 67.4, "f14 = 0x4286cccd [M 0x17e134-0x17e13c]");
    assert_eq!(b.attacker, usize::MAX - 1, "attacker NULL at the call site, the port's not-Bond mark [M 0x17e168 move $t3, $zero]");
    assert_eq!(b.pos, death_pos, "the blast sits at obj+0x30 [M 0x17e14c; the walk's sync 0x1336f8:386-387]");
    assert_eq!(b.lifetime, 165, "the blast script's +0xAB0 [L/M 0x17cd18]");
    eprintln!("[0b-death] blast at {:?} radius {} damage {}", b.pos, b.radius, b.damage);

    // The debris cadence [M 0x17c350:34-67]: inside the first lifetime quarter only
    // (165 >> 2 = 41), one `Rand_Rand(5)` draw per tick with the `time % (r + 10)` gate, and
    // every passing tick hands `Debris_CreateEx` 3 MetalHash_132 meshes.
    let mut passing = 0usize;
    let mut spawned = 0usize;
    for _ in 0..41 {
        let before = sim.drones.blast_debris.len();
        sim.step_bond(Default::default(), false, false, false);
        let grew = sim.drones.blast_debris.len() - before;
        if grew > 0 {
            assert_eq!(grew, 3, "3 particles per passing tick [M 0x17c350:63-67 — Debris_CreateEx's count 3]");
            passing += 1;
            spawned += grew;
        }
    }
    eprintln!("[0b-death] {passing} passing ticks, {spawned} debris in the 41-tick window");
    assert!(passing >= 1, "the cadence spawns inside the window (deterministic seed)");
    for d in &sim.drones.blast_debris {
        assert!(METAL_HASH_132.contains(&d.model), "kind 6 throws MetalHash_132 [M 0x17c350:41-43], got {:08x}", d.model);
    }
    // Past the script's 165-tick lifetime the blast dies and throws no more [M 0x17c394-0x17c3a0]
    // — no NEW debris; the already-thrown particles age out of the list on their own lives
    // (Rand(90)+90 ticks [M 0x1ac808:29-31]), so the length may shrink, never grow.
    let mut peak = sim.drones.blast_debris.len();
    let mut blast_gone_tick = None;
    for t in 0..165 {
        sim.step_bond(Default::default(), false, false, false);
        assert!(sim.drones.blast_debris.len() <= peak, "no NEW debris past the window ({} > {})", sim.drones.blast_debris.len(), peak);
        peak = peak.max(sim.drones.blast_debris.len());
        if blast_gone_tick.is_none() && !sim.drones.blasts.iter().any(|b| b.debris == 6) {
            blast_gone_tick = Some(t);
        }
    }
    assert!(blast_gone_tick.is_some(), "the death blast's script ended → the object dies [M 0x17c394-0x17c3a0]");
    eprintln!("[0b-death] the blast died at +{:?} ticks past the window", blast_gone_tick);
}

// The sister: off the explode levels the death departs (state 4) and creates no blast [M
// 0x17e3a4-0x17e3b8 — the 0x7000012/0x700000b test gates the 6; the case-6 Explode_Create never
// dispatches]. 07000005's gate-0 courtyard copter #241 flies from the load [D the census].
#[test]
fn the_07000005_death_departs_and_creates_no_blast() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.level, 0x0700_0005);
    let ci = sim.drones.copters.iter().position(|c| c.gate == 0).expect("the gate-0 fly-by copter parses");
    let hp = sim.drones.copters[ci].hp;
    for i in 0..14 {
        let Some(c) = sim.drones.copters.get(ci) else { panic!("the 05 copter record never departs the list [M state 4 flies on]") };
        let pos = c.pos();
        if hp < c.damage {
            // The damage landed behind this tick's update — the tail reads it next tick [M the
            // walk order]. One more step takes the state to 4.
            sim.step_bond(Default::default(), false, false, false);
            let c = &sim.drones.copters[ci];
            assert_eq!(c.state, 4, "off 0b/12 the death departs [M 0x17e3a4]");
            assert_eq!(c.landed, false, "state 4 sets no delete flag [M Copter_Move's latch is the +0x74 == 100 route end]");
            break;
        }
        frag_at(sim, pos);
        sim.step_bond(Default::default(), false, false, false);
        if i == 13 { panic!("the copter survived {} frag blasts (hp {hp})", i + 1); }
    }
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.blasts.iter().all(|b| b.debris != 6), "no death blast off the explode levels [M the case-6 gate]");
    assert!(sim.drones.copters.iter().any(|c| c.gate == 0), "the copter flies on (the departure, not the deletion)");
}
