//! abseil-damage-repro: the widened search that found why 07000011's rope guards "cannot be killed".
//!
//! The existing `abseil_damage_live` pins real-round damage at the difficulty-1 rope guard
//! (p258, switch channel 0x13) from the (22.40, 24.01, −1.39) repro spot with the aim written
//! straight into `bond.yaw/pitch` — it passes, so it never carried the symptom. This file widens
//! the search along every axis below, real input only, and pins the two divergences it
//! found:
//!
//! 1. **The never-descending roper** (`the_roping_guard_descends_to_his_opponent_without_needing_sight`):
//! from most stairwell landings the guard cannot see Bond, and the port's old `ever_seen`
//! stand-in for the abseil Slide's `+0x170 == 0` arm sent him to Hang AT THE ROPE TOP — behind
//! the stairwell's flights, where every landing's line of fire ends on the structure above the
//! player (the widened probe: 7 rounds, 0 guard hits, all world hits on the flight face). The
//! game's SP opponent pointer is unconditional [M `NDrone2_FindOpponent` 0x1424e0:100-120 — run
//! every tick by ControlSTANDARD 0x1490f0:167, `SetOpponent(glb_players)` with no sight test],
//! so the guard always has an opponent and always descends to hang 3.0 above him [M
//! 0x169730:46-51 + 0x169940:42-47].
//! 2. **The auto-aim lock through the well pane** (`auto_aim_locks_the_hanging_guard_through_the_well_pane`):
//! with auto-aim on (the shipped SP default [M lib.rs:6115 + menu.rs:1332]) the port's lock
//! check was a plain segment test, and the east connector's 0x40-material well pane [D the
//! probe's material byte 0x40; stairwell_walls_07000011_live's pass-class pane] blocked it from
//! every landing behind the pane — the lock never engaged on a guard the player visibly faces.
//! The game's sensor LOS passes that pane [M `Check_AutoAim` 0x1a73f8:164-165 calls
//! `Collide_LineOfSight` with mask 0xa; `Collide_Intersect` 0x1ebaa8:305-309/317-319 drop
//! surface-0x40/0x80 hits under the mask's 0x8 bit].
//!
//! The aim everywhere is the mouse-look path — `Input.turn` (radians) / `Input.look` (π/2 units)
//! deltas through `Player::update`'s `yaw += turn; pitch = (pitch + look).clamp(-1, 1)` [M
//! player.rs:1569-1570] — never a direct yaw/pitch write after Bond is placed. Every round is a
//! real trigger round through `Sim::step_bond` (the fire script, the bullet flight, the pick, the
//! impact), and the log records where rounds end (`Drones::player_impacts`: the guard, another
//! drone, the world) against the guard's phase.

use nf_game::drone::state;
use nf_game::player::Input;

const LEVEL: &str = "07000011.bin";

/// The stairwell landings a player shoots from [D walkreal_07000011_live.rs:66-120 waypoints +
/// stairwell_guards_07000011_live.rs:108-112].
const WEST_LANDING: [f32; 3] = [14.80, 31.40, -2.60];
const EAST_CONN: [f32; 3] = [22.34, 29.37, 3.52];
const UNDER_ROPE: [f32; 3] = [18.90, 26.00, 1.40];

/// The mouse-look aim: returns the `Input` with the `turn`/`look` deltas toward `target` (the
/// real input path [M player.rs:1569-1570]); the per-tick slew is clamped to a mouse-plausible
/// step. Never writes yaw/pitch directly.
fn mouse_look_toward(sim: &mut nf_view::scene::Sim, target: [f32; 3]) -> Input {
    let head = sim.bond.head();
    let d = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
    let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let want_yaw = d[0].atan2(d[2]);
    let want_pitch = (d[1].atan2(horiz)) / (std::f32::consts::FRAC_PI_2);
    let mut dy = want_yaw - sim.bond.yaw;
    while dy > std::f32::consts::PI {
        dy -= std::f32::consts::TAU;
    }
    while dy < -std::f32::consts::PI {
        dy += std::f32::consts::TAU;
    }
    Input { turn: dy.clamp(-0.5, 0.5), look: (want_pitch - sim.bond.pitch).clamp(-0.3, 0.3), ..Input::default() }
}

fn chest(sim: &nf_view::scene::Sim, gi: usize) -> [f32; 3] {
    let p = sim.drones.list[gi].pos;
    [p[0], p[1] + 1.0, p[2]]
}

/// One semi-auto volley (3 pressed ticks, 7 released — one shot per trigger edge [M the fire
/// action's mask-4 edge]), the mouse-look re-tracking `target` every tick, a reload pulse every
/// 2nd volley so the fire is sustained. Returns (shots fired, guard hits, hits on other drones,
/// world hits, the world-hit sample) accumulated over the volley from `Drones::player_impacts`.
fn volley(
    sim: &mut nf_view::scene::Sim,
    gi: usize,
    target: [f32; 3],
    volley_n: usize,
) -> (u32, u32, u32, u32, Option<[f32; 3]>) {
    let (mut fired, mut gh, mut oh, mut wh) = (0u32, 0u32, 0u32, 0u32);
    let mut sample = None;
    let mut scan = |sim: &nf_view::scene::Sim, fired: &mut u32, gh: &mut u32, oh: &mut u32, wh: &mut u32, sample: &mut Option<[f32; 3]>| {
        if sim.shot_tick {
            *fired += 1;
        }
        for im in &sim.drones.player_impacts {
            match im {
                nf_game::drone::PlayerImpact::Drone { drone, .. } => {
                    if *drone == gi { *gh += 1; } else { *oh += 1; }
                }
                nf_game::drone::PlayerImpact::World(p) => {
                    *wh += 1;
                    if sample.is_none() { *sample = Some(*p); }
                }
            }
        }
    };
    for _ in 0..3 {
        let inp = mouse_look_toward(sim, target);
        sim.step_bond(inp, true, false, false);
        scan(sim, &mut fired, &mut gh, &mut oh, &mut wh, &mut sample);
    }
    for rt in 0..7 {
        let inp = mouse_look_toward(sim, target);
        let reload = volley_n % 2 == 1 && rt == 0;
        sim.step_bond(inp, false, reload, false);
        scan(sim, &mut fired, &mut gh, &mut oh, &mut wh, &mut sample);
    }
    (fired, gh, oh, wh, sample)
}

/// The p258 rope guard: skin 0x05000008 at (18.895, 46.516, 1.434) on switch channel 0x13 [D
/// `nf-extract drones`; docs/systems/drone-abseil-death.md], woken here by the channel write the
/// flight plate #253 delivers. Returns his index.
fn wake_p258(sim: &mut nf_view::scene::Sim) -> usize {
    let gi = sim.drones.list.iter().position(|d| d.mode == 0x1c && d.present && d.switch_channel == 0x13)
        .expect("the mode-0x1c abseiler on channel 0x13 [D placements]");
    sim.drones.channels.set(0x13, 1, sim.drones.frame);
    gi
}

/// FIX PIN (failed before the bond_alive fix): the rope guard descends to his opponent WITHOUT
/// ever needing line-of-sight. The game's SP opponent pointer is unconditional [M
/// NDrone2_FindOpponent 0x1424e0:100-120 — SetOpponent(glb_players) every tick, zeroed only by
/// PlrStat_OkToUpdate / the player's death], so the abseil Slide's `+0x170 == 0 → Hang` arm
/// [M 0x169730:46-51] fires only on the player's death; the guard tracks the rope down and hangs
/// 3.0 above his opponent [M 0x169730:52-58 + 0x169940:42-47]. The old `ever_seen` stand-in hung
/// him AT THE ROPE TOP (y ≈ 46.5) whenever the stairwell's flights blocked the guard's view of
/// Bond — the west landing here — where no landing shot can reach him.
#[test]
fn the_roping_guard_descends_to_his_opponent_without_needing_sight() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry_difficulty(&iso, LEVEL, false, None, 1).expect("load 07000011");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = WEST_LANDING;
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    let gi = wake_p258(sim);
    let top = sim.drones.list[gi].pos[1];
    assert!((top - 46.52).abs() < 0.5, "the guard starts at the rope anchor [D placements], got {top}");
    let mut slid = false;
    for _ in 0..700 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[gi].state == state::ABSEIL_SLIDE { slid = true; }
    }
    let g = &sim.drones.list[gi];
    assert!(slid, "the guard runs the Slide (the opponent exists without sight [M 0x1424e0:100-120])");
    assert!(
        g.pos[1] < sim.bond.pos[1] + 4.0 && g.pos[1] > sim.bond.pos[1],
        "the guard hangs just above his opponent (3.0 gate [M 0x169730:57-58]), got y {:.2} vs Bond {:.2} (state {:#x})",
        g.pos[1], sim.bond.pos[1], g.state
    );
    assert!(
        matches!(g.state, state::ABSEIL_SLIDE | state::ABSEIL_HANG),
        "the guard is still on the rope above Bond (state {:#x})", g.state
    );
    assert!(g.air, "the rope flag holds [M 0x169730 ENTER +0x23 = 1]");
}

/// FIX PIN (failed before the masked-LOS fix): with auto-aim on — the shipped SP default [M
/// lib.rs:6115 + menu.rs:1332] — the lock ENGAGES on the roping guard a player faces from the
/// east-connector landing, through the landing's 0x40-material well pane [D the pane's material
/// byte 0x40 at t 0.16 of the head→guard ray; stairwell_walls_07000011_live's pass-class pane].
/// The game's Check_AutoAim LOS passes it [M 0x1a73f8:164-165 — Collide_LineOfSight mask 0xa;
/// Collide_Intersect 0x1ebaa8:305-309 set HITDATA+0x51 bit 1 on surface 0x40/0x80 and :317-319
/// drop such hits under the mask's 0x8 bit]; the old plain segment test blocked the lock from
/// behind the pane, and the rounds the game would have aimed at the guard sprayed the pane
/// instead. The locked shot then kills him through the real pipeline: the pane's bullet-pierce
/// carries the round to the guard, the abseil DSTATE's case 8 passes HandleImpact(0,0) [M
/// 0x169610:48-50 / 0x169730:68-70 / 0x169940:50-52], and the kill routes to AbseilDeath 0x92 by
/// the mode byte [M 0x1461c8:44-52] — the rope releases [M 0x169b28:28].
#[test]
fn auto_aim_locks_the_hanging_guard_through_the_well_pane() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry_difficulty(&iso, LEVEL, false, None, 1).expect("load 07000011");
    let sim = scene.sim.as_mut().unwrap();
    sim.auto_aim = true;
    sim.bond.pos = EAST_CONN;
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    let gi = wake_p258(sim);
    let hp0 = sim.drones.list[gi].health;
    // Track him by mouse-look while he descends to the hang just above Bond.
    let mut hung = false;
    for _ in 0..700 {
        let inp = mouse_look_toward(sim, [sim.drones.list[gi].pos[0], sim.drones.list[gi].pos[1] + 0.4, sim.drones.list[gi].pos[2]]);
        sim.step_bond(inp, false, false, false);
        let g = &sim.drones.list[gi];
        if g.state == state::ABSEIL_HANG && g.pos[1] < sim.bond.pos[1] + 4.0 {
            hung = true;
            break;
        }
    }
    assert!(hung, "the guard hangs within reach above Bond (the descent needs no sight [M 0x1424e0:100-120])");
    // The lock: Check_AutoAim's filters — range, the angle windows, then the mask-0xa LOS —
    // take the guard the crosshair faces.
    let mut locked = 0;
    for _ in 0..40 {
        let inp = mouse_look_toward(sim, [sim.drones.list[gi].pos[0], sim.drones.list[gi].pos[1] + 0.4, sim.drones.list[gi].pos[2]]);
        sim.step_bond(inp, false, false, false);
        if sim.drones.aim_target == Some(gi) { locked += 1; }
    }
    assert!(locked > 0, "auto-aim locks the roping guard through the well pane [M 0x1a73f8:164-165 + 0x1ebaa8:305-319], locked ticks {locked}");
    // The locked rounds kill him through the real trigger path.
    let mut fired = 0;
    for v in 0..30 {
        if sim.drones.list[gi].health <= 0.0 { break; }
        let (f, gh, _oh, wh, sample) = volley(sim, gi, [sim.drones.list[gi].pos[0], sim.drones.list[gi].pos[1] + 0.4, sim.drones.list[gi].pos[2]], v);
        fired += f;
        eprintln!("volley {v}: fired {f} guard-hits {gh} world {wh} at {sample:?} hp {}", sim.drones.list[gi].health);
    }
    assert!(fired > 0, "the trigger produced shots");
    let g = &sim.drones.list[gi];
    assert!(g.health < hp0, "the locked rounds damage the roping guard (case 8 → HandleImpact(0,0) [M 0x169730:68-70])");
    let mut died = false;
    for _ in 0..240 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[gi].state == state::ABSEIL_DEATH { died = true; break; }
    }
    assert!(died, "the kill routes to AbseilDeath 0x92 [M 0x1461c8:44-52], state {:#x}", sim.drones.list[gi].state);
    assert!(!sim.drones.list[gi].air, "the rope releases at the ENTER [M 0x169b28:28]");
}

/// COVERAGE PIN (held before and after the fixes): real trigger rounds with the mouse-look aim
/// (auto-aim off) damage and kill p258 from the east-connector landing — the widened version of
/// `abseil_damage_live`'s repro shot: the aim rides `Input.turn`/`Input.look` deltas, not yaw
/// writes. Damage applies in the Slide and Hang phases [M 0x169730:68-70 / 0x169940:50-52 — the
/// case 8/9 arms pass HandleImpact(0,0)], the kill routes to AbseilDeath 0x92 [M 0x1461c8:44-52]
/// and the rope releases [M 0x169b28:28].
#[test]
fn mouse_look_rounds_kill_p258_from_the_east_connector() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry_difficulty(&iso, LEVEL, false, None, 1).expect("load 07000011");
    let sim = scene.sim.as_mut().unwrap();
    sim.auto_aim = false;
    sim.bond.pos = EAST_CONN;
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    let gi = wake_p258(sim);
    let hp0 = sim.drones.list[gi].health;
    let (mut fired, mut gh, mut dmg_in_slide, mut dmg_in_hang) = (0u32, 0u32, false, false);
    for v in 0..40 {
        if sim.drones.list[gi].health <= 0.0 { break; }
        let phase_before = sim.drones.list[gi].state;
        let (f, g_h, _oh, _wh, _s) = volley(sim, gi, chest(sim, gi), v);
        fired += f;
        gh += g_h;
        if g_h > 0 {
            if phase_before == state::ABSEIL_SLIDE { dmg_in_slide = true; }
            if phase_before == state::ABSEIL_HANG { dmg_in_hang = true; }
            // AbseilInit 0x8e takes hits through the same HandleImpact arm (case 8, resume 0, allow 0
            // [M 0x169610:48-50]); a round that reaches him at the top of the rope (room-scoped bullet
            // pick [M Collide_Pick 0x1eb320:125]) is still a rope hit.
            if phase_before == state::ABSEIL_INIT { dmg_in_slide = true; }
        }
        eprintln!("volley {v}: phase {:#x} fired {f} guard-hits {g_h} hp {}", phase_before, sim.drones.list[gi].health);
    }
    assert!(fired > 0, "the trigger produced shots");
    assert!(gh > 0, "mouse-look-aimed rounds reach the guard (impacts logged per volley)");
    assert!(dmg_in_slide || dmg_in_hang, "the damage lands on the rope (the Init/Slide/Hang case-8 arms [M 0x169610:48-50 / 0x169730:68-70 / 0x169940:50-52])");
    assert!(sim.drones.list[gi].health < hp0, "the guard's health dropped");
    let mut died = false;
    for _ in 0..240 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[gi].state == state::ABSEIL_DEATH { died = true; break; }
    }
    assert!(died, "the kill routes to AbseilDeath 0x92 [M 0x1461c8:44-52], state {:#x}", sim.drones.list[gi].state);
    assert!(!sim.drones.list[gi].air, "the rope releases [M 0x169b28:28]");
}

/// COVERAGE PIN: the second disc abseiler — p280 (19.745, 46.580, 0.363) on channel 0x14, gated
/// to difficulty ≥ 2 by its param 2 [D `nf-extract drones`; M `Drone_Create` 0x136be0:49] —
/// never appeared in any earlier damage test. At difficulty 1 he does not exist; at difficulty 2
/// the same mouse-look trigger rounds kill him on his own channel.
#[test]
fn the_difficulty_2_abseiler_takes_the_rounds_the_same_way() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    // The create gate at difficulty 1: no mode-0x1c abseiler on channel 0x14 [M 0x136be0:49].
    let scene = nf_view::load_with_carry_difficulty(&iso, LEVEL, false, None, 1).expect("load d1");
    let sim = scene.sim.as_ref().unwrap().drones.list.iter().find(|d| d.mode == 0x1c && d.switch_channel == 0x14);
    assert!(sim.is_none(), "p280 is gated to difficulty >= 2 [M 0x136be0:49]");
    // Difficulty 2: he exists, wakes on 0x14, and dies to the same real-round flow.
    let mut scene = nf_view::load_with_carry_difficulty(&iso, LEVEL, false, None, 2).expect("load d2");
    let sim = scene.sim.as_mut().unwrap();
    sim.auto_aim = false;
    sim.bond.pos = UNDER_ROPE;
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    let gi = sim.drones.list.iter().position(|d| d.mode == 0x1c && d.present && d.switch_channel == 0x14)
        .expect("p280 exists at difficulty 2 [D placements + M 0x136be0:49]");
    sim.drones.channels.set(0x14, 1, sim.drones.frame);
    let hp0 = sim.drones.list[gi].health;
    let mut fired = 0;
    for v in 0..40 {
        if sim.drones.list[gi].health <= 0.0 { break; }
        let (f, _gh, _oh, _wh, _s) = volley(sim, gi, chest(sim, gi), v);
        fired += f;
        eprintln!("volley {v}: fired {f} hp {}", sim.drones.list[gi].health);
    }
    assert!(fired > 0 && sim.drones.list[gi].health < hp0, "p280 takes the real rounds on his channel");
    let mut died = false;
    for _ in 0..240 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.list[gi].state == state::ABSEIL_DEATH { died = true; break; }
    }
    assert!(died, "p280's kill routes to AbseilDeath 0x92 [M 0x1461c8:44-52], state {:#x}", sim.drones.list[gi].state);
}

// ---- The widened-search probes (kept for the next session; not CI) --------------------------------

/// The full widened matrix: every abseil placement × difficulty 1/2/3 × the
/// stairwell landings × auto-aim on/off, logging per config where the rounds end and whether the
/// guard's health drops. Run with
/// `NF_ISO=… cargo test -p nf-view --test abseil_damage_matrix -- --ignored --nocapture probe_all`
#[test]
#[ignore = "the manual widened-search probe (90 sim configs, minutes)"]
fn probe_all() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("skip: NF_ISO not set"); return };
    let spots: &[(&str, [f32; 3])] = &[
        ("ne_landing_436", [22.30, 43.60, -3.20]),
        ("east_conn_294", EAST_CONN),
        ("east_mid_232", [22.40, 23.20, -3.00]),
        ("repro_240", [22.40, 24.01, -1.39]),
        ("west_314", WEST_LANDING),
        ("under_rope", UNDER_ROPE),
    ];
    for &diff in &[1u32, 2, 3] {
        for &aaim in &[false, true] {
            for &(name, spot) in spots {
                for (tag, channel) in [("p258", 0x13u8), ("p280", 0x14u8)] {
                    if diff < 2 && channel == 0x14 { continue; }
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        run_probe_config(&iso, diff, channel, spot, aaim)
                    }));
                    match r {
                        Ok(o) => eprintln!(
                            "{tag} d{diff} aaim={aaim} @ {name}: fired {} guardhits {} hp {:.1}->{:.1} world {} other {} lock {} seen[i{} s{} h{}] died {} endstate {:#x}",
                            o.0, o.1, o.2, o.3, o.4, o.5, o.6, o.7, o.8, o.9, o.10, o.11
                        ),
                        Err(_) => eprintln!("{tag} d{diff} aaim={aaim} @ {name}: no placement on this channel"),
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn run_probe_config(iso: &str, diff: u32, channel: u8, spot: [f32; 3], aaim: bool) -> (u32, u32, f32, f32, u32, u32, u32, bool, bool, bool, bool, u8) {
    let mut scene = nf_view::load_with_carry_difficulty(iso, LEVEL, false, None, diff).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    sim.auto_aim = aaim;
    let gi = sim.drones.list.iter().position(|d| d.mode == 0x1c && d.present && d.switch_channel == channel)
        .unwrap_or_else(|| panic!("no abseiler on {channel:#x}"));
    sim.bond.pos = spot;
    for _ in 0..3 { sim.step_bond(Input::default(), false, false, false); }
    sim.drones.channels.set(channel as u32, 1, sim.drones.frame);
    let hp0 = sim.drones.list[gi].health;
    let (mut fired, mut gh, mut oh, mut wh) = (0u32, 0u32, 0u32, 0u32);
    let (mut lock, mut si, mut ss, mut sh) = (0u32, false, false, false);
    for v in 0..90 {
        if sim.drones.list[gi].health <= 0.0 { break; }
        for _ in 0..3 {
            let inp = mouse_look_toward(sim, chest(sim, gi));
            if sim.drones.aim_target == Some(gi) { lock += 1; }
            match sim.drones.list[gi].state {
                state::ABSEIL_INIT => si = true,
                state::ABSEIL_SLIDE => ss = true,
                state::ABSEIL_HANG => sh = true,
                _ => {}
            }
            sim.step_bond(inp, true, false, false);
            if sim.shot_tick { fired += 1; }
            for im in &sim.drones.player_impacts {
                match im {
                    nf_game::drone::PlayerImpact::Drone { drone, .. } => {
                        if *drone == gi { gh += 1; } else { oh += 1; }
                    }
                    nf_game::drone::PlayerImpact::World(_) => wh += 1,
                }
            }
        }
        for rt in 0..7 {
            let inp = mouse_look_toward(sim, chest(sim, gi));
            sim.step_bond(inp, false, v % 2 == 1 && rt == 0, false);
            for im in &sim.drones.player_impacts {
                match im {
                    nf_game::drone::PlayerImpact::Drone { drone, .. } => {
                        if *drone == gi { gh += 1; } else { oh += 1; }
                    }
                    nf_game::drone::PlayerImpact::World(_) => wh += 1,
                }
            }
        }
    }
    let g = &sim.drones.list[gi];
    (fired, gh, hp0, g.health, wh, oh, lock, si, ss, sh, g.health <= 0.0, g.state)
}
