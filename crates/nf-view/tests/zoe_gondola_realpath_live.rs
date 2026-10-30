//! 994-zoe-gondola-real-path: Zoe on the 07000008 gondola (real path, input-driven Bond).
//!
//! The ally (dtype 0xC, voice 0xC → dtypenew 0x11, `NDrone2_ControlDTYPE_Zoe` 0x14b058) is never
//! given the player as her opponent: `NDrone2_FindOpponent` 0x1424e0:101 takes the player only for
//! `Drone+0x44 != 2`, so a team-2 ally's opponent comes from the enemy-object walk or none [M].
//! Her control (`ControlDTYPE_Zoe`) also fails the mission (reason 0xf) when Bond drifts > 10
//! units from her after the ride channel ch 1 [M 0x14b058] — she is WITH Bond on the ride.
//!
//! Pinned here (real route, no teleports after the spawn):
//! (a) no round Zoe fires ever targets Bond or hits him [M 0x1424e0:101 — the opponent is never
//!     the player for team 2];
//! (b) she rides the carriage: her position stays inside the carriage's volume for the whole ride
//!     [D the ride script 0x0600004C stream 1: OBJECT_Carriage (112.0, 32.4, −126.3) →
//!     (112.0, 18.0, −179.4); M Control_InheritVelocity 0x133e88 — `Drone_FeetOnPoint` 0x140660
//!     calls it for every standing drone whose ground object is not a static type
//!     (obj+0xff ∉ {2, 3, 5, 0x11, 0x12, 0x1a}), so a drone on the moving carriage moves with it].
//!
//! (b) is GREEN on this branch (the riding port, drone.rs collision_handler). (a) is the
//! EXPECTED red this branch leaves for the team-2 opponent model — the failure message
//! names it; the ride trace above the failure shows (b) holding.
//!
//! Sisters (kept on purpose): the placed guards (team 1) DO engage Bond on the walked descent —
//! their combat is the positive control that the observation harness sees guard fire; and the
//! walk-level ally rule (`walkreal_07000008_live::the_ally_never_shoots_bond_on_the_walk`)
//! already pins the descent itself.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission objective states for 07000008 (LEVELS[3] [M mission.rs LEVELS table]).
fn obj08(w: &Walker) -> (u32, u32) {
    let m = &w.scene.sim.as_ref().unwrap().drones.mission;
    (m.states[3][0], m.states[3][1])
}

/// The carriage script entity's matrix row 3 (its origin) [D stream 1 keys].
fn carriage(sim: &nf_view::scene::Sim) -> Option<[f32; 3]> {
    sim.drones.scripts.objects.list.iter().flatten().find_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: 0x0200_038a, matrix, .. } =>
            Some([matrix[3][0], matrix[3][1], matrix[3][2]]),
        _ => None,
    })
}

#[test]
fn zoe_rides_the_carriage_and_never_targets_bond() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    let ally0 = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
        .find(|(_, d)| d.present && d.dtype == 0xc)
        .map(|(i, d)| (i, d.pos, d.state, d.team))
        .expect("the ally (dtype 0xC, Zoe) is placed");
    eprintln!("[zoe] at the spawn: index {} pos {:?} state {:#x} team {}", ally0.0, ally0.1, ally0.2, ally0.3);
    w.wait(30);

    // ── the real descent (the walked route of walk_the_gondola_ride_to_the_end, no teleports) ───
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x49), "the intro plate #237 fired ch 0x49 [D placements]");
    w.wait_enabled(4500, "the station-intro scene");
    w.wait(60);
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        ("the station doors", [139.5, 49.5, -97.5]),
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
        ("the landing east", [151.2, 42.5, -120.5]),
        ("through the door", [153.0, 42.5, -120.7]),
        ("the landing east", [151.2, 42.5, -120.5]),
        ("the ramp south", [153.1, 41.2, -116.0]),
        ("the ramp mid", [153.2, 39.7, -111.7]),
        ("the ramp turn", [148.3, 38.5, -110.9]),
        ("the mid level", [143.3, 36.5, -111.5]),
        ("the mid doors", [135.5, 36.5, -112.2]),
        ("the mid west", [132.1, 36.2, -112.3]),
        ("the long link", [120.5, 34.1, -112.5]),
        ("the hall y33", [118.2, 33.9, -111.0]),
        ("the hall y33 west", [118.2, 33.9, -107.9]),
        ("the hall y33 south", [113.2, 33.7, -106.4]),
        ("the stairwell top", [108.2, 31.5, -106.3]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.climb_stairs([107.25, 27.7, -119.5], "the stairwell down", 1.6, 4200);
    for (name, at) in [
        ("the lower level east", [110.7f32, 27.7, -121.5]),
        ("the gondola mouth", [112.0, 27.4, -122.0]),
        ("the platform west", [110.3, 27.3, -122.2]),
        ("the west ledge south", [110.0, 27.0, -124.5]),
        ("the west ledge at the cabin", [110.0, 27.0, -126.3]),
    ] {
        w.walk_nav_to(at, name, 1.4);
    }
    {
        // East into the open west doorway, polling for the floor plate (the boarding).
        let mut last = [0.0f32, 0.0];
        let mut still = 0u32;
        for t in 0..1200u32 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(4) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let target = [112.0f32, 28.0, -127.0];
            let use_key = if (p[0] - last[0]).abs() < 0.01 && (p[2] - last[1]).abs() < 0.01 {
                still += 1; still % 30 == 0
            } else { still = 0; last = [p[0], p[2]]; false };
            let input = {
                let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() }
            };
            w.tick(input, use_key);
            if t % 300 == 0 {
                let s = w.scene.sim.as_ref().unwrap();
                eprintln!("[zoe board t{t}] bond {:?} ch4 {}", s.bond.pos, s.drones.channels.on(4) as u8);
            }
        }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(4), "the cable-car floor plate fired ch 4 [D #236 mask 8]");

    // The briefcase chain is prerequisite for the ride (mux #235 ANDs ch 0x4A with ch 4); the
    // channel stands in for the walked chain exactly like the existing ride walk (its walked
    // stretch pins the same fold through the real pickups).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let f = sim.drones.frame;
        sim.drones.channels.set(0x4a, 1, f);
    }
    let mut ride_started = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(1) { ride_started = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(ride_started, "mux #235 (0x4a AND 4) fired ch 1 [D]");

    // ── the ride: record the ally, the carriage and every ally-fired round ─────────────────────
    let ai = ally0.0;
    let carriage0 = carriage(&w.scene.sim.as_ref().unwrap()).expect("OBJECT_Carriage exists once the ride starts");
    eprintln!("[zoe] ride start: carriage {:?} ally {:?}", carriage0, w.scene.sim.as_ref().unwrap().drones.list[ai].pos);

    // Per-ally-round record: the fire tick, the muzzle, the aim ray's closest approach to Bond's
    // head, and whether it hit (the sim's player_hits carry the shooter index).
    struct Round { tick: u32, from: [f32; 3], to_bond: f32, hit: bool, opp: Option<usize> }
    let mut rounds: Vec<Round> = Vec::new();
    // Per-tick riding trace: (tick, carriage origin, ally pos, ally state, in-volume?).
    let mut left_volume_at: Option<(u32, [f32; 3], [f32; 3], u8)> = None;
    let mut trace: Vec<String> = Vec::new();
    let mut ch83 = false;
    let mut guard_hits = 0usize; // the positive control: team-1 rounds that hit Bond
    let mut ally_hits = 0usize;  // the failure: ally rounds that hit Bond

    'ride: for t in 0..3600u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x53) { ch83 = true; break 'ride; }
        // The ally's fire this tick: every bullet whose owner is the ally, from its muzzle line to
        // Bond's head (the game's aim is the opponent's SetOpponentAimPos point [M 0x179d40:130];
        // a round aimed at Bond flies through his body — the miss wobble is ±1.5, so 2.0 keeps
        // aimed rounds and drops stray ones).
        let bond_head = w.scene.sim.as_ref().unwrap().bond.head();
        let zoe_shots0 = w.scene.sim.as_ref().unwrap().drones.list[ai].shots.len();
        w.tick(Input::default(), false);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            // player_hits holds THIS tick's hits (the sim clears it per tick — read it whole).
            for &(_, shooter, dmg) in &sim.drones.player_hits {
                let team = sim.drones.list[shooter].team;
                if shooter == ai || team == 2 { ally_hits += 1; }
                else { guard_hits += 1; }
                eprintln!("[zoe hit t{t}] shooter {shooter} team {team} dmg {dmg}");
            }
            // Every ally-owned bullet in flight: its line's closest approach to Bond's head. The
            // port's bullets fly straight (fixed dir), so the perp is constant along the flight —
            // re-sighting the same round repeats the same record.
            for b in &sim.drones.bullets {
                if b.owner != ai { continue; }
                let d = [bond_head[0] - b.pos[0], bond_head[1] - b.pos[1], bond_head[2] - b.pos[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let cross = [
                    b.dir[1] * d[2] - b.dir[2] * d[1],
                    b.dir[2] * d[0] - b.dir[0] * d[2],
                    b.dir[0] * d[1] - b.dir[1] * d[0],
                ];
                let perp = if l > 1e-6 { (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() / l } else { 0.0 };
                let ot = sim.drones.list[ai].opp_target;
                let opd = sim.drones.list[ai].opp_drone;
                eprintln!("[zoe shot t{t}] muzzle {:?} dir {:?} perp_to_bond {perp:.2} opp_target {ot:?} opp_drone {opd:?} bond_head {:?}", b.pos, b.dir, sim.bond.head());
                rounds.push(Round { tick: t, from: b.pos, to_bond: perp, hit: false, opp: sim.drones.list[ai].opp_drone });
            }
            let zoe_now = sim.drones.list[ai].shots.len();
            if zoe_now > zoe_shots0 { eprintln!("[zoe fired t{t}] +{} round(s)", zoe_now - zoe_shots0); }
        }
        // The riding trace.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[ai];
            let Some(car) = carriage(sim) else { continue };
            // The cabin's riding band around the carriage origin [D]: the parked origin
            // (112.0, 32.4, −126.3) hangs 4.4 above the floor plate (#236's bbox y 26.77..29.21,
            // top 28.0 [D]); the cabin box is x 110.6..113.4 (the doorway face + the hull) and the
            // origin's z ±2.5 covers the floor plate's z −128.10..−126.75. [S] unverified band —
            // measured on this harness; the RIDE claim (she moves with the carriage) is the [M]
            // inherit above.
            let in_v = (d.pos[0] - car[0]).abs() <= 2.6
                && (d.pos[2] - car[2]).abs() <= 3.4
                && d.pos[1] >= car[1] - 5.4 && d.pos[1] <= car[1] - 0.9;
            if !in_v && left_volume_at.is_none() {
                left_volume_at = Some((t, car, d.pos, d.state));
                eprintln!("[zoe LEFT the carriage band t{t}] carriage {car:?} ally {:?} state {:#x}", d.pos, d.state);
            }
            if t % 120 == 0 {
                let line = format!("[zoe ride t{t}] carriage {car:?} ally {:?} state {:#x} team {} present {}", d.pos, d.state, d.team, d.present);
                eprintln!("{line}");
                trace.push(line);
            }
        }
    }
    assert!(ch83, "the ride script wrote ch 83 (0x53) [D stream 46]");
    let end = w.scene.sim.as_ref().unwrap();
    eprintln!("[zoe] ride end: carriage {:?} ally {:?} state {:#x} present {}", carriage(end).unwrap_or([0.0; 3]),
        end.drones.list[ai].pos, end.drones.list[ai].state, end.drones.list[ai].present);
    for line in trace.iter().take(40) { eprintln!("{line}"); }
    eprintln!("[zoe] ally-fired rounds: {}  guard hits on Bond: {}  ally hits on Bond: {}",
        rounds.len(), guard_hits, ally_hits);

    // ── the assertions ──────────────────────────────────────────────────────────────────────────
    // The positive control first: the sisters (team 1) DID engage Bond on the walked descent — the
    // harness sees guard fire (their ambush is the level's own combat [D the 5 spawners]).
    assert!(guard_hits > 0, "no guard (team 1) round hit Bond on the walked descent — the observation harness saw no guard combat");
    // (b) she rides the carriage: inside the band for the whole ride.
    if let Some((t, car, pos, state)) = left_volume_at {
        panic!("Zoe left the carriage band at ride t{} (carriage {car:?} ally {pos:?} state {state:#x}) — the game's standing drone moves with its ground object [M 0x140660 → 0x133e88]", t);
    }
    // (a) the ally never targets or hits Bond [M 0x1424e0:101 — the opponent is never the player
    //     for team 2; 0x179d40:130 re-runs SetOpponentAimPos on the OPPONENT before every shot].
    //     EXPECTED RED until the team-2 opponent model lands (docs/systems/
    //     ally-heard-noise.md): the port's fire path still aims `opp_target` (= p.target) at Bond
    //     for the noise-path combat her behaviour props put her in. This branch ports the riding
    //     half only; see the stream evidence that
    //     keeps the model out of this batch.
    assert!(ally_hits == 0, "the ally's round hit Bond {ally_hits} times — team 2 never takes the player [M 0x1424e0:101; the fix is the team-2 opponent model]");
    // The game's pin is the OPPONENT identity, not the round's miss distance: team 2's opponent
    // comes from the drone-list walk and is never the player [M 0x1424e0:101-121], and every shot
    // re-aims at that opponent [M 0x179d40:130]. The old perp > 2.0 proxy contradicted the game's
    // own geometry: the descent's guards ambush AT Bond, so a round correctly aimed at guard
    // 19/20/24 passes within 2.0 of his head (measured opp_targets on this harness: guards, e.g.
    // t82 opp (100.87, 28.25, -122.15) vs his head (111.02, 28.33, -127.46) — 11 apart — while the
    // line's perp is 0.99).
    for r in &rounds {
        // A None pick is the game's own no-opponent arm (SetOpponent(0) — the aim degenerates to
        // the drone's own position [M 0x1422d0:25-29]); what team 2 never has is a PLAYER opponent.
        let ok = r.opp.map_or(true, |i| end.drones.list[i].team == 1);
        assert!(ok, "an ally-fired round at t{} had a non-team-1 opponent (opp {:?}) — team 2 never takes the player [M 0x1424e0:101-121]",
            r.tick, r.opp);
    }
    let _ = obj08(&w);
}
