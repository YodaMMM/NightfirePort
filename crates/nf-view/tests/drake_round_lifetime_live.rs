//! drake-round-lifetime: Drake's weapon-0x30 rounds on 0700001b — the Torpedo Launcher's slow
//! glowing ball. What the game does with a round whose shooter died mid-flight, and the exact
//! flight numbers, pinned through the real spawn + flight paths:
//!
//! - The round's flight data is the record's: range `weapon_data[0x30]+0x1C` = 500.0, speed
//!   `+0x20` = 0.11666697 u/tick quartered by the `+0x6C & 0x200` bit to 0.02916674 at spawn,
//!   damage `+0x0C` = 20.0 [M `Bullet_init` 0x12c5a0:188-191 `pfVar3[0x3e] = +0x20; if
//!   (+0x6C & 0x200) *= 0.25` and :166 the damage carry; L ee_start_05.bin @0x2bf150 +
//!   0x30·0x10C — asserted below from the port's ISO-extracted table].
//! - `Bullet_DoTrails` 0x12d890 ramps a `+0x6C & 0x200` round by +`0x20`·0.02 per tick while
//!   `BU+0xF8 < +0x20` [M disasm 0x12d924-0x12d964 — the `< +0x20` check is BEFORE the add, so
//!   the 38th step overshoots to 0.11783366 and stops], and the ramp starts on the SPAWN tick:
//!   `Bullet_update` 0x12e7b0's state-0 path falls into the same move-branch tail whose tail
//!   calls `Bullet_DoTrails` [M disasm 0x12efd8-0x12f000, the jal 0x12d890 after the move].
//!   The port skipped exactly that tick (`born != now` filter) — fixed.
//! - The spawn tick's own STEP is (0.5 + FRand(0.5)) · quarter and is NOT stored [M disasm
//!   0x12e854-0x12e888 computes the step; the tail's `swc1 0xf4($s4)` writes BU+0xF4 only] —
//!   `Rand_FRand` 0x1e3780 returns [0, max), so the first step is 0.5..1.0 × the stored speed.
//! - A flying round's death is the RANGE or a hit — never its shooter: state 1 kills when
//!   `weapon+0x1C < BU+0xF4` [M 0x12e95c-0x12e984], the step clamps to the boundary and
//!   `BU+0xF4` keeps the overshoot so the next tick kills [M 0x12f00c-0x12f02c]. No arm of
//!   `Bullet_update` reads the shooter's alive state, so a dead shooter's round keeps flying
//!   (the port's flight records carry no owner at all — `PlayerBullet` — and nothing reaps
//!   them on a drone's death).
//!
//! Test setup stand-ins ([S], the fix_npcs_live convention): the round is fired through the
//! port's real `Bullet_init` implementation (`Drones::fire_player` — the spawn path
//! `Player_WeaponInitBullet` 0x1a4bf0 / `Bullet_init` 0x12c5a0 share) with Drake's weapon id,
//! because the fight AI's own fire gate (`astronaut_move_pick`'s `firing = los`, 0x154618:78)
//! never opens on an idle Bond in the port (see the report). The FLIGHT under test — the
//! quarter speed, the spawn-tick ramp, the range death, the shooter-independence — is the
//! real `update_player_bullets` walk. Opt-in via NF_ISO (the slim ISO is enough).

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn free(sim: &mut nf_view::scene::Sim, ticks: u32) -> bool {
    for _ in 0..ticks { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { return true; } }
    !sim.disabled
}

/// Drake's weapon-0x30 round, fired from his own position toward the fight, through the port's
/// real `Bullet_init` spawn path (fix_npcs_live's stand-in shape): the Data view is assembled
/// the way `Sim::fire` does it; the muzzle is Drake's head. Returns the spawned round.
fn fire_drakes_round(scene: &mut nf_view::scene::Scene, at: [f32; 3]) -> (f32, f32, [f32; 3]) {
    let sim = scene.sim.as_mut().unwrap();
    let drake = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba)
        .expect("Drake present (spawner #187 [D])");
    let d = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: None, models: &sim.models,
        surfaces: None, voices: None, difficulty: sim.drones.difficulty, ai_points: &sim.ai_points,
    };
    let from = {
        let p = sim.drones.list[drake].pos;
        [p[0], p[1] + 0.6, p[2]]
    };
    let dir = {
        let dx = at[0] - from[0]; let dy = at[1] - from[1]; let dz = at[2] - from[2];
        let l = (dx * dx + dy * dy + dz * dz).sqrt();
        [dx / l, dy / l, dz / l]
    };
    // Drake's weapon is 48 = 0x30 [D the placement's weapon param 9 — the sim reads it:
    // assert below pins the record shape against it].
    assert_eq!(sim.drones.list[drake].weapon, 0x30, "Drake's weapon is 0x30 [D placement param 9]");
    let snd = nf_formats::sfx::SfxHearing { id: 0x403, inner: 0.0, outer: 0.0, weight: 0.0, looped: false, cull: false };
    sim.drones.fire_player(&d, from, dir, 0x30, snd);
    let b = sim.drones.player_bullets.last().expect("the round spawned").clone();
    (b.speed, b.range, b.pos)
}

/// The main pin: the round flies the record's numbers (quarter speed, the spawn tick's ramp
/// already applied, range 500), gains exactly one ramp step per tick until the 38-tick
/// overshoot cap, KEEPS FLYING after every drone is removed (the game's flight path never
/// reads the shooter), and dies of the RANGE at +0x1C — with the boundary clamp and the
/// one-tick overshoot kill [M 0x12e95c-0x12e984 + 0x12f00c-0x12f02c].
#[test]
fn drakes_round_flies_the_record_and_dies_of_the_range_not_its_shooter() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "0700001b.bin", false).expect("load 0700001b");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(free(sim, 3000), "Bond is free after the shuttle arrival");
        // The fight gate (drake_fight_live's drive — the chain head runs behind the missiles).
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
    }
    // Drake spawns and settles past the spawn tick.
    let mut found = None;
    for _ in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
            found = Some(i);
            break;
        }
    }
    found.expect("the gate spawns Drake (spawner #187, trigger 10 [D])");
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }

    // The record, from the port's ISO-extracted table ([D] disc data; the [L] EE snapshot
    // cross-check is cited in the module doc).
    {
        let sim = scene.sim.as_ref().unwrap();
        let w = &sim.weapons.records[0x30];
        let f = |o: usize| f32::from_le_bytes(w[o..o + 4].try_into().unwrap());
        assert_eq!(f(0x1c), 500.0, "weapon 0x30 range +0x1C [L ee_start_05.bin @0x2bf150 + 0x30·0x10C]");
        assert!((f(0x20) - 0.11666697).abs() < 1e-4, "weapon 0x30 speed +0x20 [L], got {}", f(0x20));
        assert_eq!(f(0x0c), 20.0, "weapon 0x30 damage +0x0C [L]");
        assert_eq!(u32::from_le_bytes(w[0x6c..0x70].try_into().unwrap()) & 0x200, 0x200, "the quarter-speed bit [L +0x6C = 0x20200]");
    }
    // Fire Drake's round at the deck ahead and read it on its spawn tick.
    let (spawn_speed, spawn_range, pos0) = {
        let at = {
            let sim = scene.sim.as_ref().unwrap();
            let p = sim.drones.list[sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba).unwrap()].pos;
            [p[0], p[1], p[2] + 10.0]
        };
        fire_drakes_round(&mut scene, at)
    };
    let quarter = 0.11666697f32 * 0.25;
    let ramp = 0.11666697f32 * 0.02;
    assert_eq!(spawn_range, 500.0, "the round flies the record's +0x1C [M Bullet_init 0x12c5a0 → Bullet_update 0x12e95c]");
    assert!((spawn_speed - quarter).abs() < 1e-6,
        "the round spawns at the record's quarter speed — Bullet_init 0x12c5a0:188-191 writes BU+0xF8 = +0x20 × 0.25 before any tick runs [M], got {spawn_speed}");
    // The shooter dies (every drone goes absent — the strongest form of the dead-shooter case):
    // the round must keep flying and keep RAMPING exactly as the game's DoTrails does.
    for i in 0..scene.sim.as_ref().unwrap().drones.list.len() {
        scene.sim.as_mut().unwrap().drones.list[i].present = false;
    }
    // Sister (the fix's own pin): the SPAWN tick (the first Bullet_update, state 0) both moves
    // the round and runs its DoTrails — the stored speed leaves the muzzle already ramped by
    // exactly one step [M Bullet_update 0x12e7b0 state 0 → the move-branch tail 0x12efd8-0x12f000,
    // the jal 0x12d890; 0x12d924-0x12d964] — and the round is STILL in flight with all drones gone.
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    {
        let sim = scene.sim.as_ref().unwrap();
        let b = sim.drones.player_bullets.iter()
            .find(|b| b.weapon == 0x30 && (b.pos[0] - pos0[0]).abs() + (b.pos[2] - pos0[2]).abs() < 4.0)
            .expect("the round still flies with its shooter gone — no arm of Bullet_update 0x12e7b0 reads the shooter [M]");
        assert!((b.speed - spawn_speed - ramp).abs() < 1e-6,
            "the spawn tick's DoTrails ramps the stored speed once [M 0x12d924-0x12d964], got {} → {}", spawn_speed, b.speed);
    }
    // 600+ ticks later (the observed flavour) the round is STILL live: 600 ticks × the cruise
    // ≈ 60 units, far under the 500 range — the round lives because the RANGE says so.
    let mut mid = None;
    for t in 0..600u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if t == 598 {
            let sim = scene.sim.as_ref().unwrap();
            mid = sim.drones.player_bullets.iter().find(|b| b.weapon == 0x30).map(|b| (b.travelled, b.speed, b.dead));
        }
    }
    let (travelled, speed, dead) = mid.expect("the round is still in flight 600 ticks in");
    assert!(!dead, "not reaped by the shooter's absence [M 0x12e7b0]");
    assert!((speed - 0.11783366).abs() < 2e-3, "the stored speed reached the 38-tick overshoot cap [M 0x12d924-0x12d964], got {speed}");
    assert!(travelled > 50.0 && travelled < 500.0, "600 ticks of flight at the ramped cruise, still inside the range [M], travelled {travelled}");
    // Run it out: the round dies of the RANGE — its last seen travelled sits inside one step
    // of +0x1C = 500 [M 0x12e95c-0x12e984: the head check kills when the OVERSHOT BU+0xF4
    // exceeds the range; the boundary clamp keeps the previous step exactly at 500].
    let mut last = (0.0f32, false);
    for _ in 0..6000u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        match sim.drones.player_bullets.iter().find(|b| b.weapon == 0x30) {
            Some(b) => last = (b.travelled, b.dead),
            None => break,
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.player_bullets.iter().any(|b| b.weapon == 0x30), "the round is dead");
    assert!(last.0 >= 499.8, "the death is the range: the last seen travelled {} sits at +0x1C [M 0x12e95c-0x12e984]", last.0);
    assert!(last.0 <= 500.2, "the clamp kept it at the boundary [M 0x12f014-0x12f02c], got {}", last.0);
}
