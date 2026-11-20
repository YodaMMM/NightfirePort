//! walkreal-h: 07000008 (Mayhew D, the gondola) walked with the player's own movement — no
//! teleports after the spawn, NO channel or state pokes in the walk: the whole mission from one
//! fresh load, both objectives in player order, ending on the level exit firing
//! (level-08-every-objective). Route (docs/levels/07000008.md, the interactables census):
//!
//! 1. the mountain-top station #0 (139.5, 50.5, −76.4) [D placements kind 0x24] → the intro plate
//!    TouchOnce #237 (139.2, 49.5, −81.2) → **ch 0x49** (this completes 07000007's carried
//!    "Spy on secret meeting" objective [L ee_start_05 record 2]) + SP #294 (0x06000077, the
//!    station intro, 3880 ticks [M levels.md §5.1]) plays, skipped only with the game's own skip;
//! 2. down the station/walkway (the ch-0x2a zone plates wake the placed guards, sw_ch 42 [D]) to
//!    the fusebox #219 (FUSEBOX_GuardRoomDoorLock, group-0 channel 137 [D]) — Bond SHOOTS its
//!    wires (the gun contact survives the wire filter [M fusebox.rs Collide_FilterBullets]) →
//!    **ch 137** unlocks the Secure Room door #122/10 (lock ch 137 [D]) and starts SP #221
//!    (GuardRoomLights);
//! 3. inside, the LASER (0x4E, owned [D inventory]) burns the safe bolts #207/#208 (Lock_Create
//!    0x1ae1f8, out 134/135 [M Lock_Update 0x1ae3d8:194]) → mux #278 → **ch 136** opens the safe;
//! 4. the briefcase pickup #290 (kind 0xF0 type 4, switch channel 74 = 0x4a [D]) → the take writes
//!    **ch 0x4A** [M Pickup_Handler 0x18e650:330-333] → objective "Retrieve guidance chip" folds;
//! 5. back to the gondola station: the floor plate Touch #236 (out 4, mask 8 [D]) → ch 4; mux #235
//!    (out 1 = 0x4A AND 4) → **ch 1** starts SP #295 ScriptPlayer_CableCar (mode 64 loaded-armed
//!    [M SP_LoadScript 0x1dbc08]; the ride script 0x0600004C lives in the LEVEL BIN, 1420 ticks)
//!    — the carriage (112.0, 32.4, −126.3) → (112.0, 18.0, −179.4) [D stream 1] carries Bond
//!    (the game's riding [M Control_InheritVelocity 0x133e88]) — its stream 46 writes **ch 83
//!    (0x53)** at tick 1410;
//! 6. ch 83 starts SP #296 (0x06000078, the bottom-of-mountain cinematic, 342 ticks) whose
//!    stream 10 writes **ch 5**; the helicopter #45 (Copter_Create 0x17d340: gate ch 5, out
//!    ch 0x4B [M record+0x70/+0x72; Copter_Update 0x17dbb8 cases 4/5/6]) flies and writes
//!    **ch 0x4B** → objective "Escape with Zoe in gondola" folds → mux #276 → **ch 100** →
//!    Mission_Update ends the level into the DRIVING hand-off **0x0730008**
//!    [M 0x182c60:85-116 — LevelToEndTo; no LOAD_LEVEL placement exists on this level [D]].
//!
//! The level's mission data lists exactly TWO objectives for 07000008
//! [L ee_start_05.bin @ 0x2a3e88, 2 records; the states Mission_MonitorObjectives 0x1827d8 runs]:
//! 1. "Retrieve guidance chip from Safe Room" (ch 0x4A, enabled by ch 0x49) — completes at the
//!    briefcase take (leg 4);
//! 2. "Escape with Zoe in gondola" (ch 0x4B, enabled by ch 0x4A) — completes at the Hind's
//!    death tail (leg 6, the only ch-0x4B writer on this level [M Copter_Update's state-4 arm;
//!    D the route dump: no kind-2 arrival path]).
//!
//! The carried latches (05's 0x45/0x70, 06's 0x02, 07's 0x48) are the mission's ENTRY state — in
//! play they ride the RamSave [M Player_RamSave 0x195670 / Player_RamLoad 0x1957a8]; the cold
//! test load sets them once at spawn (the same set the cold stand-in walk_07000008_live pins),
//! never during the walk. The old separate ride test's ch-0x4A stand-in poke is gone: the real
//! ch 0x4A from the briefcase take feeds mux #235 (level-08-every-objective).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission objective states for 07000008 (LEVELS[3] [M mission.rs LEVELS table]).
fn obj08(w: &Walker) -> (u32, u32) {
    let m = &w.scene.sim.as_ref().unwrap().drones.mission;
    (m.states[3][0], m.states[3][1])
}

/// Face `target` from Bond's head (turn + look), then hold fire for `ticks` — the REAL fire input
/// drives the gun bullets (the fusebox wires) and the laser beam (the safe bolts' burn [M
/// Player_LaserPointer 0x1a65d8's hook; walker.rs tick_fire]).
fn aim_and_fire(w: &mut Walker, target: [f32; 3], weapon: u8, fire_ticks: u32) {
    aim_and_fire_until(w, target, weapon, fire_ticks, |_| false);
}

/// Hold the fire until `done` holds (a player lets go once the lock or wire goes) — the laser's
/// meter drains ~31/s while held [D docs/live/2026-10-14-weapon-measurements-pcsx2.md].
fn aim_and_fire_until(w: &mut Walker, target: [f32; 3], weapon: u8, fire_ticks: u32, done: impl Fn(&nf_view::scene::Sim) -> bool) {
    // The switch through the real weapon-change input.
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == weapon { break; }
        w.tick(Input { switch_to: Some(weapon), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, weapon, "the switch to {weapon:#x} never landed");
    for t in 0..fire_ticks {
        let sim = w.scene.sim.as_ref().unwrap();
        if done(sim) { break; }
        let head = sim.bond.head();
        let to = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let desired_yaw = to[0].atan2(to[2]);
        // `bond.pitch` is BL+0x8A8 in units of π/2 (player.rs) — the angle converts.
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
        let pitch_err = desired_pitch - sim.bond.pitch;
        let input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: pitch_err.clamp(-0.05, 0.05),
            ..Default::default()
        };
        w.tick_fire(input);
        if t % 120 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 aim t{t}] bond {:?} yaw {:.2} pitch {:.2} weapon {:#x}", sim.bond.pos, sim.bond.yaw, sim.bond.pitch, sim.gun.weapon);
        }
    }
}

/// The shoot-through check for a rocket launch: the round flies the straight corridor its
/// muzzle opens [M Bullet_update 0x12e7b0's per-tick world segment + Collide_Pick 0x1eb320],
/// so a launch whose line the bottom-station's own geometry blocks explodes at the contact —
/// and at 0.65–5.5 u from Bond that blast (radius 8, ≈100 [L ee_start_05.bin @0x2bf150 +
/// 0x2C·0x10C], the (int)-truncated linear falloff × the level's 0.6 pain factor [M
/// Explode_Propagate 0x17c518:69-71; Player_HandlePain's table]) takes Bond with it: the
/// measured death run's four "world" hits −52.5/−7.5/−22.5/−3.7 at frames 26477/26580/26639/
/// 26696 are Bond's OWN rockets 1/3/4 (docs/reports/level-08-long-range-hits.md). A player
/// pulls the trigger only on a clear line — the eye ray AND both gun stand-ins (the view
/// model's low-right/left offsets; the lane-fights' rule — at a wall edge the eye ray clears
/// by a hair while the gun ray clips the corner [L the level-13 rf-trace]).
fn launch_line_clear(sim: &nf_view::scene::Sim, to: [f32; 3]) -> bool {
    let b = sim.bond.pos;
    let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
    let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
    let right = [cy, 0.0, -fy];
    let spots = [eye,
        [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
        [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]];
    spots.iter().all(|&from| {
        sim.world.segment_bullet_scoped(sim.world.find_cel(from), from, to)
            .is_none_or(|t| t >= 0.999)
    })
}

/// A player does not board the gondola with a burst-rifle pair alive at 4 u behind him: close
/// guards (team 1, within 12 u horizontal / 4 u vertical of Bond) are answered with the GUN
/// (`switch_to_gun` → `stand_and_fire`, the shared fight machinery) before the walk goes on.
/// REAL-DAMAGE ONLY (gated on `tops_up()`): the tactic exists for the run that has to survive
/// its own damage; the topped-up sister's fight timeline stays exactly as pinned.
fn fight_close_guards(w: &mut Walker, label: &str, max_ticks: u32) {
    fight_close_guards_opts(w, label, max_ticks, false)
}

/// The VISIBLE-only sister (guard-glass-finish): the pick keeps only close guards a sight ray
/// reaches, and the loop breaks the moment the nearest close guard is buried — a stand on a
/// guard the world trace cannot see is pure exposure: the 07000008 stairwell-mouth stands ran
/// eleven 61-tick stands on the burst pair 3-10 u behind the mouth's solid wall band (aim None
/// every stand, NF_RF_TRACE 2026-11-06) while the pair walked 8.5 u to point-blank and the
/// stairwell-top pair (#23/#24) landed free hits into the `fighting=true` window — the d2 run
/// died at tick 25036 with the pair at d 0.6-0.7. The shared `stand_and_fire` cannot retarget
/// mid-stand, so a buried nearest guard must not open a stand at all: the legs' own return fire
/// answers these guards the moment they round the mouth's opening. The plain sister keeps every
/// pinned timeline (each existing call site passes false).
fn fight_close_guards_visible(w: &mut Walker, label: &str, max_ticks: u32) {
    fight_close_guards_opts(w, label, max_ticks, true)
}

fn fight_close_guards_opts(w: &mut Walker, label: &str, max_ticks: u32, visible_only: bool) {
    fight_guards_within(w, label, max_ticks, visible_only, 12.0)
}

/// real-health-00agent-08-bleed-3 (d3): the WIDE sister of `fight_guards_within` — the same
/// single-target stands with a larger radius and y band. The fusebox pit guard #16 (skin
/// 0x5000030, (148.1, 38.7, −110.5)) sniped the burn stands from the level below at 12.7-12.8
/// u and 4.4 u down — outside the close fights' 12 u ring and their ±4.0 y band [L the
/// 2026-11-14 run-lever4 log: fourteen −4.0 hits t3550-6554 drained the last 60.0 while Bond
/// held the laser burn]. Local to this file, every call d3-gated.
///
/// real-health-00agent-08-bleed-4 (d3): the pick is the guard who actually LANDED THE LAST HIT
/// inside the wide ring — the walker's hurt bookkeeping (`player_hits`, the (frame, owner,
/// damage) record the [dmg] trace reads) names him — not the nearest: the unseen nearest-first
/// variant opened on the buried doorway man #5, stalled 60 ticks and died t4320 [L the 2026-11-14
/// run-lever6 log]. Before the first burn no hit has landed yet, so the pick falls back to the
/// in-ring guard whose muzzle is live on Bond NOW (his own shot log + his aim point on Bond —
/// the same firing evidence `DamageLog::shooters` reads). #16 is NOT visible from the burn spot
/// (the run-lever5 rerun: the visible-only wide fight found nobody while his rounds reached
/// Bond), so a below-the-level pick gets the PIT EXCURSION: the route's own descent legs (the
/// path-2 link chain the walk itself takes after the secure room) carry Bond down toward the
/// guard with a sight check after every leg; the stand fires the moment a leg opens the line,
/// and the return walks the same legs back to the burn spot (the caller's laser needs the 4.0
/// range [L weapon_data +0x4E·0x10C+0x1C]). A guard at or above Bond's level never gets the
/// excursion (the descent is the pit's answer, not a hall guard's). [S] the excursion legs are
/// the walk's own player tactic; the legs themselves are the master flow's measured green
/// descent.
///
/// The y band is 8.0 at both call sites, not the ledger's ±6.0: the pit pair MOVES — the
/// lever run's #16 fired from (144.4, 37.2, −111.1) mid-descent and the poll at the burn spot
/// (y 44.3) read dy 7.1 — outside ±6 — so the whole fight stayed a silent no-op while a
/// SECOND pit shooter (#15, skin 0x500002e, (151.7, 39.2, −111.5)) opened up from dy 3.9-5.1
/// and the death came t5050 [L run-bleed4-2]. The pit floor spans y 37-39.5 under a y 43-44.3
/// Bond: the band that always holds the pit class is 8.0.
fn fight_guards_wide(w: &mut Walker, label: &str, max_ticks: u32, radius: f32, dy: f32) {
    if w.tops_up() { return; }
    let start = w.total_ticks;
    let mut excursions = 0u32;
    // retune-nb-08-2: a stand the creep cannot open (the buried doorway man's 60-tick unseen
    // abort) must not silence the whole response — on the 2026-11-19 lever run the failed #5
    // stand broke the fight, the pit guard #15 below kept firing and killed the armed walk at
    // the doorway (t6408, −23.0). Skip the failed pick and answer the next one (the
    // fight_guards_within idiom); every ring guard skipped ends the fight.
    let mut skipped: Vec<usize> = Vec::new();
    loop {
        if w.total_ticks - start >= max_ticks {
            eprintln!("[08 guards] {label}: fight cap at tick {} (health {:.1})", w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
            break;
        }
        let (di, below) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let now = sim.drones.frame;
            let ring: Vec<usize> = sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1)
                .filter(|(_, d)| ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt() < radius && (d.pos[1] - b[1]).abs() < dy)
                .map(|(i, _)| i).collect();
            let ring_now_count = ring.len();
            // 1. the guard who landed the LAST hit on Bond inside the ring — the hurt bookkeeping
            //    names him (player_hits is frame-ordered; the newest entry with an in-ring live owner).
            let hurt = sim.drones.player_hits.iter().rev()
                .find(|h| h.1 < sim.drones.list.len() && ring.contains(&h.1) && !skipped.contains(&h.1))
                .map(|h| h.1);
            // 2. before any round has landed: the in-ring guard firing at Bond NOW — his own shot
            //    log is live (≤ 90 frames) and his aim point sits on Bond (the shooters() law,
            //    [M DroneFiring opp_target reads — the same evidence the return-fire pick uses]).
            let firing = ring.iter().copied().find(|&i| !skipped.contains(&i) && {
                let d = &sim.drones.list[i];
                d.shots.last().is_some_and(|&f| now.saturating_sub(f) <= 90)
                    && ((d.opp_target[0] - b[0]).powi(2) + (d.opp_target[2] - b[2]).powi(2)).sqrt() < 1.5
            });
            match hurt.or(firing) {
                Some(i) => {
                    let d = &sim.drones.list[i];
                    let visible = walker::DamageLog::visible(sim, i);
                    (Some(i), !visible && d.pos[1] < b[1] - 2.0)
                }
                None => (None, false),
            }
        };
        let Some(di) = di else { break };
        if below && excursions < 3 {
            excursions += 1;
            // retune-nb-08-8 (d3): TRIED and DEAD on its own real run — the crouch press BEFORE
            // the excursion's descent legs: the crouched descent dropped Bond's eye INTO the pit
            // pair's line (the ramp west edge no longer blocks) and the walk died ON THE RAMP at
            // t6918 to #15's burst at (153.2, 40.1, −113.0) — was t22649 in the shipped tree [L
            // the 2026-11-20 lever-1b run]. The 28.0 the legs pay (#16 5 hits t5168-5583) needs a
            // fight-geometry answer, not a stance answer (the ledger's next-window levers).
            eprintln!("[08 guards] {label}: the pit pick #{di} ({:?}) is unseen from here — the pit excursion {excursions} at tick {}, health {:.1}",
                w.scene.sim.as_ref().unwrap().drones.list[di].pos, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
            // The route's own descent legs (the outbound list below, "the gondola: the descent to
            // the floor plate"): the path-2 link chain, walked on every green run. Sight check
            // after every leg — the stand fires the moment the line opens.
            let excursion: &[(&str, [f32; 3])] = &[
                ("the landing east", [151.2f32, 42.5, -120.5]),
                ("through the door", [153.0, 42.5, -120.7]),
                ("the ramp south", [153.1, 41.2, -116.0]),
                ("the ramp mid", [153.2, 39.7, -111.7]),
                ("the ramp turn", [148.3, 38.5, -110.9]),
            ];
            let mut opened_at: Option<usize> = None;
            for (k, (name, at)) in excursion.iter().enumerate() {
                // retune-nb-08-9 (d3): TRIED and DEAD on its own real run — the ledger lever (2)
                // "the descent with the gun ALREADY raised" (a 30-tick aim hold before each leg;
                // reverted with the lever): the raise is NOT the stands' latency — the hip cone
                // fires during the raise [M Player_SetWeaponAnimObj case 0xf; sniper_stands'
                // settle note], so the first visible tick already fired, and the 30-tick holds
                // per leg (5 legs) wobbled every downstream guard roll instead: #16 still cost
                // 24.0 and the walk died EARLIER, t18365 to #26's burst rifle point-blank (d 1.0)
                // at (106.9, 28.4, −120.3) with the armour drained — was t22649 [L the 2026-11-20
                // lever-2 run]. The bleed is the blind stands' exposure, not the gun's latency.
                w.walk_nav_to(*at, &format!("{label} {name}"), if k + 1 == excursion.len() { 0.9 } else { 1.2 });
                let vis = walker::DamageLog::visible(&w.scene.sim.as_ref().unwrap(), di);
                eprintln!("[08 guards] {label}: {name} at tick {} bond {:?} — pick #{di} visible {vis} (health {:.1})",
                    w.total_ticks, w.scene.sim.as_ref().unwrap().bond.pos, w.scene.sim.as_ref().unwrap().drones.health.health);
                if vis { opened_at = Some(k); break; }
            }
            let mut killed = false;
            if opened_at.is_some() {
                killed = w.switch_to_gun(label) && w.stand_and_fire(di, 900);
            }
            eprintln!("[08 guards] {label}: the pit pick #{di} {} (killed {killed}) at tick {}, health {:.1}",
                if opened_at.is_none() { "never opened" } else if killed { "down" } else { "held" },
                w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
            // The return: the walked legs in reverse, then the caller's own spot — the laser's
            // 4.0 range [L weapon_data +0x4E·0x10C+0x1C] needs Bond back on the fusebox walkway.
            if let Some(k) = opened_at {
                for (name, at) in excursion[..k].iter().rev() {
                    w.walk_nav_to(*at, &format!("{label} back {name}"), 1.2);
                }
            } else {
                for (name, at) in excursion.iter().rev() {
                    w.walk_nav_to(*at, &format!("{label} back {name}"), 1.2);
                }
            }
            w.walk_nav_to([154.35, 43.41, -119.67], &format!("{label} back to the burn spot"), 1.0);
            if !killed { break; } // one failed excursion — walk on (the caller's leg continues)
            continue;
        }
        eprintln!("[08 guards] {label}: the hurting pick #{di} ({:?}) at tick {}, health {:.1} — the gun first",
            w.scene.sim.as_ref().unwrap().drones.list[di].pos, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
        let killed = w.switch_to_gun(label) && w.stand_and_fire(di, 900);
        eprintln!("[08 guards] {label}: pick #{di} {} (killed {killed}) at tick {}, health {:.1}",
            if killed { "down" } else { "held (out of sight or cap)" }, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
        if !killed {
            skipped.push(di);
            continue; // the pick skips failed guards; no candidate left breaks the loop
        }
    }
}

fn fight_guards_within(w: &mut Walker, label: &str, max_ticks: u32, visible_only: bool, radius: f32) {
    if w.tops_up() { return; }
    let close = |sim: &nf_view::scene::Sim| -> Option<usize> {
        let b = sim.bond.pos;
        sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1)
            .filter(|(_, d)| ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt() < radius && (d.pos[1] - b[1]).abs() < 4.0)
            .filter(|(i, _)| !visible_only || walker::DamageLog::visible(sim, *i))
            .min_by(|x, y| {
                let dx = (x.1.pos[0] - b[0]).powi(2) + (x.1.pos[2] - b[2]).powi(2);
                let dy = (y.1.pos[0] - b[0]).powi(2) + (y.1.pos[2] - b[2]).powi(2);
                dx.total_cmp(&dy)
            })
            .map(|(i, _)| i)
    };
    let start = w.total_ticks;
    let mut skipped: Vec<usize> = Vec::new();
    loop {
        if w.total_ticks - start >= max_ticks {
            eprintln!("[08 guards] {label}: fight cap at tick {} (health {:.1})", w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
            break;
        }
        let Some(di) = close(&w.scene.sim.as_ref().unwrap()) else {
            if visible_only {
                eprintln!("[08 guards] {label}: no visible close guard at tick {} (health {:.1}) — walk on", w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
            }
            break;
        };
        if skipped.contains(&di) && skipped.len() >= w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1).count() {
            break; // every close guard has had its stand; walk on
        }
        eprintln!("[08 guards] {label}: the close guard #{di} ({:?}) at tick {}, health {:.1} — the gun first",
            w.scene.sim.as_ref().unwrap().drones.list[di].pos, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
        let killed = w.switch_to_gun(label) && w.stand_and_fire(di, 900);
        // Level the view again before walking on (a fight leaves the pitch on the guard).
        for _ in 0..90 {
            let p = w.scene.sim.as_ref().unwrap().bond.pitch;
            if p.abs() < 0.01 { break; }
            w.tick(Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
        }
        eprintln!("[08 guards] {label}: guard #{di} {} (killed {killed}) at tick {}, health {:.1}",
            if killed { "down" } else { "held (out of sight or cap)" }, w.total_ticks, w.scene.sim.as_ref().unwrap().drones.health.health);
        if !killed { skipped.push(di); }
    }
}

/// An owned FIREARM with rounds in the pool — the AT-420 launcher (0x2c) EXCLUDED: its rounds
/// are the Hind shoot-down's (the walk asserts the rockets on hand at the fight) and its blast
/// at a 9.8 u guard is Bond's own geometry (radius 8, ≈100 [L ee_start_05.bin + 0x2C·0x10C]).
/// The 25773 death's trap: `DamageLog::stand_and_fire` fires whatever is in hand, and the dry
/// pistol's `switch_to_armed` wheel (2..=0x47) counts the owned launcher armed — the stand drew
/// it instead of a gun [L the 2026-11-03 run log: cycling (0) 0xc 0/0, (1) 0x1 clip 1 → the next
/// owned stop is 0x2c].
fn firearm_armed(sim: &nf_view::scene::Sim) -> bool {
    let inv = &sim.drones.inventory;
    (2u8..=0x47).filter(|k| *k != 0x2c).any(|k| {
        inv.owned.get(k as usize).copied().unwrap_or(false) && {
            let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, k as usize);
            inv.clip.get(slot).copied().unwrap_or(0) > 0
                || inv.carried.get(sim.weapons.ammo_type(k as usize)).copied().unwrap_or(0) > 0
        }
    })
}

/// Put a FIREARM with rounds in hand through the real switch input (never the 0x2c launcher —
/// see `firearm_armed`). Returns whether a firearm is in hand with rounds.
fn switch_to_firearm(w: &mut Walker, label: &str) -> bool {
    for _ in 0..120 {
        let want = {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            if g.weapon != 0x2c && (2..=0x47).contains(&g.weapon) && g.clip + g.spare > 0 { return true; }
            let inv = &sim.drones.inventory;
            (2u8..=0x47).filter(|k| *k != 0x2c).find(|k| {
                inv.owned.get(*k as usize).copied().unwrap_or(false) && {
                    let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, *k as usize);
                    inv.clip.get(slot).copied().unwrap_or(0) > 0
                        || inv.carried.get(sim.weapons.ammo_type(*k as usize)).copied().unwrap_or(0) > 0
                }
            })
        };
        let Some(k) = want else { break };
        w.tick(Input { switch_to: Some(k), ..Default::default() }, false);
    }
    eprintln!("[08] {label}: no firearm with ammo to put in hand");
    let sim = w.scene.sim.as_ref().unwrap();
    let g = &sim.gun;
    g.weapon != 0x2c && (2..=0x47).contains(&g.weapon) && g.clip + g.spare > 0
}

/// The game's R between fights (the 02 walk's `reload_full`): one reload press, then the
/// reload's own wait.
fn reload_full(w: &mut Walker) {
    let spare = w.scene.sim.as_ref().unwrap().gun.spare;
    if spare == 0 { return; }
    w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, true, false);
    for _ in 0..90 { w.tick(Default::default(), false); }
}

/// The game's own weapon cycle to the carried firearm holding the most rounds (clip + spare),
/// launcher excluded — the 02 walk's `arm_fullest` verbatim (a player walking into a known
/// shoot-out with a near-empty gun switches to his fullest gun first).
/// retune-nb-08-11: `arm_fullest_at(.., direct_first = true)` reads the pick DIRECTLY from the
/// inventory first (the same candidate law `switch_to_firearm` uses — owned firearm with clip
/// or carried ammo > 0, fullest = max clip+carried). A player already holding his fullest gun
/// does not cycle the whole loadout: the shipped 12-cycle scan burned 762 ticks on the end
/// loadout's 6 weapons (f25309-26071 [L the 2026-11-20 baseline instrumentation run]) — most
/// of the ~880-tick pane shoot-out. TAIL-ONLY: the mid-game fights (the east deck, the walkway
/// pair) keep the shipped scan verbatim — the first lever run took the fast path there too and
/// the mid-game timeline wobbled (death t24113 at the trio, the boarding drained HEALTH [L the
/// 2026-11-20 first lever run]); those fights' guards fire during the shipped cycles and their
/// shapes are the shipped tree's own pace. Measured on the tail: 5 cycles (the pick 0xa is 5
/// presses out of the in-hand 0x2) — the arm phase 396 ticks with the reload, was ~853 [L the
/// 2026-11-20 lever run].
fn arm_fullest(w: &mut Walker, label: &str) {
    arm_fullest_at(w, label, false);
}

fn arm_fullest_at(w: &mut Walker, label: &str, direct_first: bool) {
    let read = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.gun.weapon, sim.gun.clip as i32 + sim.gun.spare as i32)
    };
    let cycle = |w: &mut Walker| {
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(Default::default(), false); }
    };
    let best_direct = if direct_first {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        (2u8..=0x47).filter(|k| *k != 0x2c)
            .filter(|k| inv.owned.get(*k as usize).copied().unwrap_or(false))
            .filter(|k| {
                let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, *k as usize);
                inv.clip.get(slot).copied().unwrap_or(0) > 0
                    || inv.carried.get(sim.weapons.ammo_type(*k as usize)).copied().unwrap_or(0) > 0
            })
            .map(|k| {
                let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, k as usize);
                (k, inv.clip.get(slot).copied().unwrap_or(0) as i32
                    + inv.carried.get(sim.weapons.ammo_type(k as usize)).copied().unwrap_or(0) as i32)
            })
            .max_by_key(|(_, n)| *n)
    } else { None };
    if let Some((best_id, best_n)) = best_direct {
        if read(w).0 != best_id {
            for _ in 0..12 {
                if read(w).0 == best_id { break; }
                cycle(w);
            }
        }
        let (id, n) = read(w);
        assert_eq!(id, best_id, "the fullest gun {best_id:#x} never landed in hand (in hand {id:#x})");
        eprintln!("[08] {label}: armed the fullest gun {id:#x} ({n} rounds; the direct pick {best_id:#x} {best_n} rounds)");
        reload_full(w);
        return;
    }
    let first = read(w);
    let mut seen = vec![first];
    for _ in 0..12 {
        cycle(w);
        let now = read(w);
        if now.0 == first.0 { if seen.len() > 1 { break; } continue; }
        if !seen.iter().any(|g| g.0 == now.0) { seen.push(now); }
    }
    let firearm = |id: u8| (2..=0x47).contains(&id) && id != 0x2c;
    let Some(best) = seen.iter().filter(|(id, n)| firearm(*id) && *n > 0).max_by_key(|(_, n)| *n).map(|b| b.0) else { return };
    for _ in 0..12 {
        if read(w).0 == best { break; }
        cycle(w);
    }
    for _ in 0..12 {
        if read(w).0 == best { break; }
        cycle(w);
    }
    let (id, n) = read(w);
    eprintln!("[08] {label}: armed the fullest gun {id:#x} ({n} rounds; carried {:?})", seen);
    reload_full(w);
}

/// Walk to the nearest live pickup at the feet (the guards' drops land at the fight floor) and
/// take it — the 02 walk's `scoop_drops` verbatim; `n` pickups, then cycle to a firearm.
fn scoop_drops(w: &mut Walker, label: &str, n: usize) {
    for k in 0..n {
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let target = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
            .filter(|p| !p.gone)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .filter(|q| walker::dxz(*q, bp) < 6.0 && (q[1] - bp[1]).abs() < 1.5)
            .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)));
        let Some(target) = target else { break };
        eprintln!("[08] {label}: scooping the drop ({:.1},{:.1},{:.1}) (d {:.1})", target[0], target[1], target[2], walker::dxz(target, bp));
        w.walk_nav_to(target, &format!("{label} scoop {k}"), 0.4);
        let _ = w.switch_to_armed(&format!("{label} scoop {k} arm"));
    }
    let _ = w.switch_to_armed(label);
}

/// The whole mission on one fresh load: the intro, the fusebox, the safe, the briefcase
/// (objective 1 → 4), the descent back down, the boarding, the ride, the cinematic, the Hind
/// shoot-down on real aim and trigger (objective 2 → 4), and the mission end into 0x0730008.
/// The station walkway pair still standing on the walkway at the dip (148.4, 43.0, −121.4).
fn walkway_pair_holds(w: &Walker) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
        && ((d.pos[0] - 148.4f32).powi(2) + (d.pos[2] - -121.2f32).powi(2)).sqrt() < 3.0 && (d.pos[1] - 43.0).abs() < 1.5)
}

#[test]
fn walk_the_whole_mission_on_real_input() { walk_the_whole_mission_on_real_input_at(1) }

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_the_whole_mission_on_real_input_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000008.bin", false).expect("load 07000008")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000008.bin", false, None, difficulty)
            .expect("load 07000008")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // guard-door-park: per-guard ticks spent in GoToGoalPosition 0x63 inside the guard-room
    // door arc (154.3, 43.0, −119.0), sampled every tick until the flight record takes the hook
    // over. The cel-blind goal-node search handed the guard-room pair a near-side fallback node
    // and a route that ends at the locked leaf: on the d1 walk they stood at x 153.97 for ~14k
    // ticks (frames 14451-21574+, measured with the old search). The game's
    // `AINetwork_NodesForPosition` 0x1586a8 searches the goal's own cel, so a goal whose cel
    // holds no node of the path gives `AINetwork_DoAStarPath` 0x1571b0 code 4 and the guard
    // leaves 0x63 instead of pressing the door (`NDrone2_MoveToGoalPosition` 0x1517d8).
    let park: std::rc::Rc<std::cell::RefCell<[u32; 24]>> = Default::default();
    {
        let park = park.clone();
        w.on_step = Some(Box::new(move |scene: &nf_view::scene::Scene, _| {
            let sim = scene.sim.as_ref().unwrap();
            for (i, g) in sim.drones.list.iter().enumerate() {
                if g.present && g.enabled && !g.corpse && g.team == 1 && g.state == 0x63
                    && ((g.pos[0] - 154.3).powi(2) + (g.pos[2] + 119.0).powi(2)).sqrt() < 3.5
                    && (g.pos[1] - 43.0).abs() < 4.0
                {
                    park.borrow_mut()[i.min(23)] += 1;
                }
            }
        }));
    }
    // 00 Agent (difficulty 3): the fights keep the 05/07 sisters' 4.0 back-off — the boarding
    // platform's burst-rifle pair closes under 2 u during the exchange and its interval-4
    // bursts land point-blank (the measured d3 ledger: #20's fight alone cost 54.0 from
    // 1.2-2 u). A player steps back while firing. D2 JOINS IT under the armed pass
    // (glass-pierce-2, the block above): the pair's bursts landed point-blank in the d2 death
    // run too (d 0.6-0.7).
    // The real-damage Operative walk takes the same fight package as the Agent sisters (the
    // back-off, the stairwell and platform-mouth stands, the east-ledge boarding): the guards'
    // stand-off moves (step, strafe, dodge with no world gate [M DroneAnim_PreTransCallback
    // 0x13b490]) bring the burst pair to 3.2 u on its old west-ledge timeline (died t26394,
    // #20 32.4 in 9 hits). The topped-up Operative walk keeps the master timeline.
    let fight_pkg = difficulty >= 2 || !w.tops_up();
    if fight_pkg {
        w.dmg.back_off = Some(4.0);
    }
    // capsule-join-3: the solid guard bodies wedge the fusebox leg — the walk_to by the fusebox
    // hit the 6000-tick cap against a guard standing in the nook (this tree's measured red). The
    // player's response to a body blocker is to shoot him; the opt-in lever turns the wedge into
    // the fight [M walker.rs body_block_response — the 02 real-health walk's same lever].
    w.body_block_response = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 139.5).abs() < 2.0 && (p[2] + 76.4).abs() < 2.0, "the station spawn #0 [D], got {p:?}");
        // The carried latches (in play they ride the RamSave [M 0x195670]); the cold stand-in
        // walk_07000008_live pins the same set.
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    w.wait(30);

    // ── the intro plate: ch 0x49 + the station-intro scene ──────────────────────────────────────
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x49), "the intro plate #237 fired ch 0x49 [D placements]");
    }
    w.wait_enabled(4500, "the station-intro scene");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0077) && p.state >= 3),
            "the station-intro scene (0x06000077) ran through the real trigger [M levels.md §5.1]");
        assert_eq!(sim.drones.mission.states[2][1], 4, "07000007's carried objective 2 completes on ch 0x49 [L record 2]");
        assert_eq!(obj08(&w).0, 3, "objective 1 announced by ch 0x49 [L record 3]");
    }

    // ── down the station and the walkway toward the fusebox ─────────────────────────────────────
    // The ch-0x2a plate sits inside the station (139.1, 50.2, −93.6 [D]) — the placed guards
    // (sw_ch 42) wake as Bond passes. The walkway descends south (door 11 free at (145,45,−108.6)).
    //
    // THE SCOPED-FLIGHT WALK HARDENING (level-4a-every-objective-4): under the game's cel-scoped
    // drone bullet flight [M Bullet_update 0x12e7b0:343 + Collide_Pick 0x1eb320:125] the station's
    // guards' rounds reach Bond on this approach (228 PlayerImpact hits by the lower-landing wedge
    // in the measured run), and each hit's `Player_HandlePain` grunt roll draws the shared
    // bit-exact RNG [M 0x1902c8:111 — the port's bullet.rs apply()] that the guards' AI also
    // draws — so the guards' positions and the doors' approach states wobble run-to-run, and the
    // walk's stuck-mitigation sidesteps land on different spots. The dip crossing at the stair
    // down to the fusebox walkway (the nav node (147.30, 42.41, −120.40) [D nav]) is the fragile
    // step: a sidestep at its edge drops Bond into the station's pit (measured: y 44.85 → 37.13,
    // the walk then fires the laser from the pit and the 4.0-range segment [L weapon_data
    // +0x4E·0x10C+0x1C] tops 1.4 below the wire box at y 43.41 — ch 137 never rises; the -3
    // report's mechanism, reproduced). The game's own player walks this approach live (the -3
    // live check stands) — the walk therefore RETRIES the approach like a player re-climbing:
    // the stance is checked before the shot (on the fusebox level, beside the box), and a wrong
    // stance re-walks the approach from the lower landing (bounded). [S] unverified as game
    // behaviour: the retry is the walk's own robustness, not a game claim.
    let fusebox = [154.35f32, 43.41, -119.67];
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        // real-health-00agent-08-bleed-3 (d3): the east-deck head — the walkway-head nav's own
        // measured first arrival (152.16, 46.08, −93.92) [L the 2026-11-14 baseline run: nav 1
        // lands IN the east placements' fire, the t684 hit −4.0 from #14 at d 4.1, and the
        // nav-2 return-fire stand on #11 cost 32.0 (96.0 → 64.0)] — walked as its own leg so
        // the visible-only stand fights the east placements (152.5-161.5, 46.0,
        // −87.7..−101.5) FROM the deck, before the westward nav crosses their lanes. The
        // 11-13 mouth stand fights nobody from the hall mouth (no visible guard at t373 [L the
        // same run]); from the deck head every shooter is at 4-12 u. D3-GATED: the sisters'
        // timelines stay byte-exact.
        ("the east deck head", [152.2f32, 46.0, -93.9]),
        // No "station doors" stop at (139.5, 49.5, −97.5) on this walk: its nav route runs east
        // through door (160.6, 45.0, −100.1) and back west along the lower deck (y 46.1), and the
        // last leg aims at a point 3.4 above that deck — the leg wedged there, and the stuck
        // ladder's sidesteps at the deck edge dropped Bond into the station pit and out of the
        // map (y 46.1 → 40.8 → −508). The walkway head's own route walks the same deck.
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
    ] {
        if name == "the east deck head" && difficulty < 3 { continue; }
        // real-health-00agent-08-bleed-3 (d3): the landing nav from the deck-line approach
        // wedges ON the stair lip (146.34, 44.99, −115.69) grounded=false [L the 2026-11-14
        // run-lever3 log] — take the approach chain's own stair-head waypoint instead (the
        // chain below re-walks it; its arrival is the measured green crossing). D3-GATED.
        if difficulty >= 3 && name == "the lower landing" {
            w.walk_to(Wp::new(146.6, 44.4, -117.6).r(2.2), "the stair head from the walkway");
            continue;
        }
        if name == "the lower landing" && w.tops_up() && walkway_pair_holds(&w) { continue; }
        // 00 Agent real damage: cross the station hall crouched (the walkway-head leg cost 40.0 to
        // the hall's four 0xa guards standing; measured: the walk then reaches the Hind with more
        // health and dies at t29598, was t26671).
        if difficulty >= 3 && !w.tops_up() && name == "the walkway head" && !w.scene.sim.as_ref().unwrap().bond.crouched {
            w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
        }
        // retune-nb-08-8 (d3): TRIED and DEAD on its own real run — the crouch press BEFORE the
        // east-deck-head nav: the crouched crossing diverged the approach geometry and the walk
        // never reached the fusebox — the deck-line west walk_to wedged at the closed hall door
        // (143.45, 49.55, −97.30) head_bump grounded=false for the 6000-tick cap, crouched=false
        // by then, health 100.0, ZERO damage progress [L the 2026-11-20 lever-1 run]. The crouch
        // sweep stays on the shipped legs only (the hall crossing, the briefcase stretch).
        // The east deck head takes the tight 0.7 arrival: the 1.5 radius counted an arrival
        // 1.3 u west of the measured node and the next nav re-routed through the east door
        // (160.6, −100.1) into its frame wedge [L the 2026-11-14 run-lever1 log].
        let nav_r = if name == "the east deck head" { 0.7 } else { 1.5 };
        // real-health-00agent-08-bleed-3 (d3): after the deck-head fight the walkway-head nav
        // re-routes through the east door (160.6, −100.1) and wedges in its frame from the deck
        // above (head_bump at (160.27, 46.60, −99.37), the door already open pos01 1.00; every
        // later leg wedged there) [L the 2026-11-14 run-lever1 log]. Walk the deck line west
        // explicitly first (the baseline bleed's own line: (152.4, −93.9) → (150.0, −96.4)),
        // then the short walkway-head hop navs direct. D3-GATED: the sisters' timelines stay
        // byte-exact.
        if difficulty >= 3 && name == "the walkway head" {
            w.walk_to(Wp::new(148.5, 46.1, -98.5).r(1.5), "the deck line west");
        }
        // retune-nb-08-9 (d3): TRIED and DEAD on its own real run — the ledger lever (1) "the
        // deck pin 36.0", the stand pull west 2.5 u before every east-deck-head return-fire
        // stand (an opt-in stand_pull walk in damage_log.rs, reverted with the lever). The
        // pulled stands STILL took the −4.0 rows at the pulled spots — the trio's lanes cover
        // the WHOLE deck line, not just the pin (#2 hit at (148.4,−93.8) d 13.7, #11 at
        // (145.1,−92.4) d 10.4, #14 at (147.5,−92.3) d 6.2 [L the 2026-11-20 lever run]) — and
        // the fights ran LONGER from the lip (the dry pistol's 1-round cycle walked back east
        // between taps): the deck-head fight armed 12.0 health (baseline ~52.0) and the walk
        // died t7313 on the pit ramp to #16 — 15336 ticks WORSE than the shipped t22649. The
        // pin's bleed is not a stand-spot problem; there is no out-of-lane spot on the deck.
        w.walk_nav_to(at, name, nav_r);
        // los-mask-repins-08-agent (d2): the east-hall guards (#11/#14/#2/#9/#10, placements
        // (152.5-161.5, 46.0, −87.7..−101.5) [L the run ledgers]) wake on the ch-0x2a plate and
        // hold the deck the walkway-head nav crosses — on the Agent accuracy (1.1
        // [M DroneWeap_DoBulletAccuracy 0x179740:74-88]) their bursts cost 44.8 across the
        // walkway legs (the d2 ledger: #11 19.6 at tick 987, #2 14.0, #9 8.4 [L the
        // NF_DMG_TRACE run on this branch]). A player stops at the station hall's mouth (the
        // plate at (139.1, 50.2, −93.6) [D]) and fights the men he can see ACROSS the hall
        // before walking their placements: the same visible-only stand the walkway legs use,
        // out to 25 u. D2-GATED: the Operative sisters' timelines stay byte-exact.
        if difficulty >= 2 && fight_pkg && name == "the station hall" {
            fight_guards_within(&mut w, "the station hall mouth", 4200, true, 25.0);
        }
        // real-health-00agent-08-bleed-3 (d3): the deck-head stand — fight the east placements
        // from the east deck head before the westward leg crosses their lanes (the leg's own
        // comment above; the 11-13 ledger's "fight the hall guards from cover as they come
        // round" — they never come round, but from the deck head they are all visible at
        // 4-12 u [L the 2026-11-14 baseline run]). D3-GATED.
        if difficulty >= 3 && fight_pkg && name == "the east deck head" {
            // retune-nb-08-2 (d3): THE ARM-UP, post-nav (the pre-nav call was TRIED and DEAD on
            // its own run: at leg start nothing is dead yet, the pick/scoop are silent no-ops
            // and the arm cycle's ticks alone shifted the transit into the south lane [L the
            // 2026-11-19 lever run]). (1) The wide stand on any surviving buried hurter — the
            // hurt bookkeeping names him, the unseen-cover creep opens the line, back-off
            // suspended (its 4.0 ring reads the 3.4-u muzzle as a crowd). (2) Scoop the deck
            // drops — the guards' own 0xa machine guns — and arm the fullest: the transit
            // fought on the dry pistol (0x1 clip 1 spare 0, the resupply found nothing on the
            // corridor [L the NF_DMG_TRACE run]) and its 60-tick-per-tap reload cycles are why
            // the crossfire ate 36.0. D3-GATED: the sisters' timelines stay byte-exact.
            let (saved_back_off, saved_unseen) = (w.dmg.back_off, w.dmg.unseen_fight);
            w.dmg.back_off = None;
            w.dmg.unseen_fight = true;
            fight_guards_wide(&mut w, "the east deck head", 1200, 25.0, 8.0);
            w.dmg.back_off = saved_back_off;
            w.dmg.unseen_fight = saved_unseen;
            scoop_drops(&mut w, "the east deck", 3);
            arm_fullest(&mut w, "the east deck");
        }
    }
    // With the game's standing bodies the live walkway pair (#7/#8, AimStandFire 0x68 / StepAim
    // 0x74 at (148.4, 43.0, −121.4)) stand in the landing leg's lane: the nav leg runs down to the
    // walkway and back east past them, their bodies held Bond 1.1-2 u off for 226 ticks and left
    // him on the landing's pit lip, and the stair-head leg dropped him to y 37.0 (the laser then
    // cannot reach the box: ch 137 never rises) [L capsule-join trace]. While the pair holds the
    // walkway, Bond takes the nav web straight down to the stair foot (the landing leg's own
    // first nodes) and walks the walkway east from there.
    // Top-up walk only: the real-damage sisters' own fights (`fight_guards_within`, gated on
    // `!tops_up()`) keep their measured route green, and their timelines stay byte-exact.
    let walkway_first = w.tops_up() && walkway_pair_holds(&w);
    if walkway_first {
        w.walk_nav_to([146.15, 43.26, -119.85], "the stair foot from the walkway mid", 0.7);
    }
    // THE SCOPED-FLIGHT APPROACH FIX (level-4a-every-objective-4). The nav route into the fusebox
    // walkway steers at the dip node (147.30, 42.41, −120.40 [D nav]) from the lower-landing
    // wedge spot — and the wedge spot WOBBLES with the guards' fire: under the game's cel-scoped
    // drone bullet flight [M Bullet_update 0x12e7b0:343 + Collide_Pick 0x1eb320:125] the station
    // guards' rounds reach Bond here (228 PlayerImpact hits by the lower-landing wedge in the
    // measured run), and every hit's `Player_HandlePain` grunt roll draws the shared bit-exact
    // RNG [M 0x1902c8:111; the port's bullet.rs apply()] the guards' AI also draws — the guards'
    // positions and the doors' approach states wobble, the walk's stuck-mitigation sidesteps
    // land on different spots, and a sidestep at the stair edge drops Bond into the station's
    // pit (measured y 44.85 → 37.13; the 4.0-range laser [L weapon_data +0x4E·0x10C+0x1C] then
    // tops 1.4 below the wire box at y 43.41 and ch 137 never rises — the -3 report's mechanism,
    // reproduced). The green run's crossing is the proof the stair is walked head-on: its nav-1
    // arrival (146.15, 43.26, −119.85) sits on the walkway's west end after a straight south
    // descent. So the approach squares UP first — explicit waypoints down the stair's measured
    // line, each with an arrive radius the wobble cannot outrun (a leg that arrives within its
    // first stuck sample never enters the mitigation ladder). The last leg walks the walkway
    // east to the firing spot. [S] unverified as game behaviour: the waypoints are the walk's
    // own robustness, the descent line is the measured green crossing.
    for (name, at, r) in [
        ("the stair head", [146.6f32, 44.4, -117.6], 2.2),
        ("the walkway west", [146.3, 43.3, -119.8], 2.0),
        // The stair foot: the green crossing's own arrival (146.15, 43.26, −119.85). The 2.0
        // radius above can count Bond arrived still ON the stair (measured (146.33, 44.15,
        // −117.85)); the diagonal leg east from there pressed the stair's side, and the stuck
        // ladder walked him back up to the walkway door (146.3, 46.1, −109.4) for the cap.
        ("the stair foot", [146.15, 43.26, -119.85], 0.7),
        ("the walkway line", [149.5, 43.2, -120.3], 2.2),
        ("by the fusebox", [153.0, 43.4, -118.5], 1.5),
    ] {
        if walkway_first && matches!(name, "the stair head" | "the walkway west") { continue; }
        w.walk_to(Wp::new(at[0], at[1], at[2]).r(r), name);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[08] at the fusebox tick {} bond {:?} ch42 {} ch43 {} ch44 {}", w.total_ticks, sim.bond.pos,
            sim.drones.channels.on(0x2a) as u8, sim.drones.channels.on(0x2b) as u8, sim.drones.channels.on(0x2c) as u8);
    }

    // 00 Agent (difficulty 3), real-health-00agent-08-bleed: the burn stand is fought first. With
    // the game's solid guard bodies [M master 0a9d39435] the walkway door's burst-rifle man stands
    // in his arc 2.9-4.2 u from the burn spot (the measured death: drone #5, weapon 0x15, at
    // (156.6, 43.0, −120.6) — eight interval-4 bursts of 6.0 ate health 42 → 0 in 58 ticks at
    // (153.7, 43.1, −120.5), tick 8995 [L the NF_DMG_TRACE run on this branch]) while Bond stood
    // the laser burn and then the E-press with no fight before it. A player does not stand a
    // 4-u laser on a fusebox with a rifleman at arm's reach behind the half-open leaf: he clears
    // the doorway first — the platform-mouth lip's pattern (the unseen-cover creep through the
    // gap the half-open leaf leaves, the back-off suspended: its 4.0 ring reads the doorway man
    // as a crowd and backs away from the only winning spot). D3-GATED: the Operative/Agent
    // sisters' timelines stay byte-exact.
    if difficulty >= 3 {
        let d3_door_man = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                && ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)).sqrt() < 6.0 && (d.pos[1] - bp[1]).abs() < 2.0)
        };
        if d3_door_man {
            let (saved_back_off, saved_unseen) = (w.dmg.back_off, w.dmg.unseen_fight);
            w.dmg.back_off = None;
            w.dmg.unseen_fight = true;
            fight_close_guards(&mut w, "the fusebox doorway", 4200);
            w.dmg.back_off = saved_back_off;
            w.dmg.unseen_fight = saved_unseen;
        }
    }

    // ── cut the fusebox wires with the LASER → ch 137 (the Secure Room door's lock channel) ─────
    // The wire filter DROPS guns (kind 0x0008) and keeps the Laser (0x0002) [M fusebox.rs's
    // Collide_FilterBullets 0x1ece30:24-31 with the disc category words] — gunfire never cuts a
    // wire; the player's laser does.
    // real-health-00agent-08-bleed-3 (d3): the pit fight before the burn — the level-below
    // snipers (#16-class) sit outside the doorway fight's 6 u and the close fights' 12 u/±4
    // rings [L the run-lever4 log]. D3-GATED: the sisters' timelines stay byte-exact.
    // real-health-00agent-08-bleed-4 (d3): the pick is the HURTING one — the guard who landed
    // the last hit inside the wide 25-u/±6-y ring (the hurt bookkeeping names him — #16 —
    // [L the run-lever4 log: fourteen −4.0 hits t3550-6554]), falling back to the in-ring
    // guard whose muzzle is live on Bond before his first round lands; the unseen below-level
    // pick takes the pit excursion (the function doc). The UNSEEN nearest-first variant
    // (run-lever6) REGRESSED — its pick opened on the buried doorway man #5, stalled 60 ticks,
    // death t4320 — the hurting pick never stands a guard the bookkeeping has not named.
    if difficulty >= 3 && !w.tops_up() {
        fight_guards_wide(&mut w, "the fusebox pit", 4200, 25.0, 8.0);
    }
    aim_and_fire_until(&mut w, fusebox, 0x4e, 420, |s| s.drones.channels.on(137));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(137), "shooting the fusebox wires raises ch 137 [D #219 group-0; M FuseBox_Update 0x1adfe8]");
    }
    if walkway_first {
        // The same pair then walks into the half-open walkway door (154.5, 42.0, −121.3)
        // (GoToGoal 0x63 at (154.0, 43.0, −120.5..−120.8)): the secure-room and cabin legs wedged
        // on their bodies for every cap [L capsule-join red]. A player answers the two guards
        // standing in his doorway with his fullest gun before walking on.
        arm_fullest(&mut w, "the walkway pair");
        for _ in 0..4 {
            let g = {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                sim.drones.list.iter().position(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, b) < 8.0 && (d.pos[1] - b[1]).abs() < 2.0)
            };
            let Some(g) = g else { break };
            if !w.shoot(g, &format!("the walkway guard {g}")) { break; }
        }
    }

    // real-health-00agent-08-bleed-4 (d3): the SAME wide stand before the bolts/safe burns —
    // the ledger's lever (the ignore message): the pit guard's rounds reached the door
    // approach too (the last ledgered hit lands t6554 at (154.51, 43.10, −120.72) — the door
    // leg itself [L the run-lever4 log]), so a pit guard the first stand missed is answered
    // here, before the door E and the burns. The pick re-reads the hurt bookkeeping; a pit
    // guard already dead makes this a no-op. The stand sits at the door (not inside the room,
    // where the excursion out would wedge on the closed leaf — the arc-occupied law above).
    // D3-GATED: the sisters' timelines stay byte-exact.
    if difficulty >= 3 && !w.tops_up() {
        // THE OPENING-VOLLEY POLL (real-health-00agent-08-bleed-4, measured on the lever run):
        // both pre-burn stands ran BEFORE the pit guard ever fired — his first volley lands
        // ~56 ticks after the burn ends (t3550 in the lever run, the door leg's own start) and
        // the bookkeeping is empty at both call sites, so both picks were silent no-ops and
        // the timeline matched the ledger death t6554 exactly [L run-bleed4-1]. A player at
        // the door hears the rounds from below before he walks the arc: hold the burn spot for
        // up to 420 ticks while the walk's own return fire answers the visible doorway men,
        // until ANY in-ring guard's round has landed (the hurt bookkeeping names him) — then
        // the hurting pick fights him, the pit excursion included. The poll's own exposure is
        // the door arc's standing exposure relocated (the door leg stood there anyway).
        for t in 0..420 {
            let named = {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                sim.drones.player_hits.iter().rev().any(|h| {
                    h.1 < sim.drones.list.len() && {
                        let d = &sim.drones.list[h.1];
                        d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                            && ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt() < 25.0
                            && (d.pos[1] - b[1]).abs() < 8.0
                    }
                })
            };
            if named { eprintln!("[08] the pit poll: a round landed at tick {} — the hurting fight runs", w.total_ticks); break; }
            w.tick(Input::default(), false);
        }
        // retune-nb-08-9 (d3): the ledger item (2)'s substance lever — the pit fight's legs run
        // under level_fights (the existing opt-in pick filter, ±2.0 y): the return-fire stands
        // the descent legs kept opening on the below-level pit pair (dy 3.1-6.1) were ALL blind
        // and all failed (killed false t5044/t5384 [L the 2026-11-20 baseline run]) — each one
        // stood Bond in #16's lane for up to 300 unseen ticks. The excursion's own machinery
        // owns the pair (the ramp-mid stand kills #15 at the opener); the legs' own return fire
        // answers only the at-level ring. Judged on its own real run. D3-GATED: the sisters'
        // timelines stay byte-exact.
        let saved_level = w.dmg.level_fights;
        w.dmg.level_fights = true;
        fight_guards_wide(&mut w, "the pit before the burns", 4200, 25.0, 8.0);
        w.dmg.level_fights = saved_level;
    }

    // ── the secure room: the laser burns the safe bolts → ch 136 ────────────────────────────────
    // Door #10/122 (the GuardRoomDoor, lock ch 137 [D]) sits at (154.5, 42, −121.3) — E it ON its
    // position, then step through into the room (the safe wall's face is east).
    w.walk_to(Wp { at: [154.6, 42.5, -121.0], act: Act::Use { use_ticks: 240 }, arrive: 1.1 }, "E the secure room door");
    w.walk_nav_to([158.0, 43.0, -119.0], "into the secure room", 1.4);
    // The bolts sit at (162.24, 43.88/42.70, −117.14) [D #207/#208] — inside, on the safe face.
    w.walk_nav_to([160.0, 43.4, -117.0], "before the safe", 1.2);
    let bolts = [[162.24f32, 43.88, -117.14], [162.24, 42.70, -117.14]];
    for (k, bolt) in bolts.iter().enumerate() {
        let ch = [134u32, 135][k];
        aim_and_fire_until(&mut w, *bolt, 0x4e, 240, |s| s.drones.channels.on(ch));
        eprintln!("[08] bolt {k} laser_charge {:.1}", w.scene.sim.as_ref().unwrap().laser_charge);
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(ch), "the laser burn wrote the bolt's out channel {ch} [M Lock_Update 0x1ae3d8:194]");
    }
    // The mux folds 134|135 → 136; the safe door opens and SP #286 (SafePickUps) starts.
    w.wait(10); // the fire stops on the burn-through tick; the mux and the SP start on the following ticks
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(136), "mux #278 folded the bolt channels into ch 136 [D]");
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_09e2) && p.state >= 2), // 2 = playing, 3 = done (nis.rs SP states)
            "SP #286 SafePickUps started on ch 136 [D mask 1]");
    }

    // ── the briefcase: the take writes ch 0x4A (objective 1) ────────────────────────────────────
    if difficulty >= 3 && !w.tops_up() {
        // 00 Agent real health: the bolt stand leaves Bond at z -117.6 (the plain walk at
        // -116.5), and from there the nav leg to the safe runs him into the safe-face corner at
        // (162.59, -118.17) for the whole cap (the briefcase never taken). Start the leg from the
        // plain walk's spot.
        // 00 Agent: crouched for the briefcase stretch (#4, 0xa, landed 5 x 4.0 on the standing
        // walk at (159.0, -116.4)); measured: death t29598 -> t34720 in the Hind fight.
        if !w.scene.sim.as_ref().unwrap().bond.crouched { w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false); }
        let _ = w.try_walk_to(Wp::new(158.93, 43.13, -116.52).r(0.3), "the briefcase leg's start");
    }
    w.walk_nav_to([172.7, 43.5, -116.2], "to the briefcase", 1.0);
    let mut got_case = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a) { got_case = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(got_case, "taking the briefcase writes ch 0x4a [M Pickup_Handler 0x18e650:330-333]");
    for _ in 0..120 {
        if obj08(&w).0 == 4 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(obj08(&w).0, 4, "objective 'Retrieve guidance chip' completes on ch 0x4a [L record 3]");

    eprintln!("[08] the briefcase stretch done in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
    // join-reds2-0 (Agent, real damage): the secure room's BodyArmour (kind 3, 50, at (159.4,
    // 42.6, -113.6) [D the placement census]) lies 3.6 u from the briefcase stand and the walk
    // left it on the floor (armour 0, health 70.6 here [L the run]). On the joined tree the
    // platform riflemen #17/#20/#26 then took Bond from 70.6 to 7.6 before the ride, and the
    // Hind fight had nothing left to absorb its own-rocket and machine-gun hits. A player takes
    // the armour in the room he is standing in. D2-GATED, real damage only: the other sisters'
    // timelines stay byte-exact.
    // los-mask-join: the Operative walk takes it too — on the masked tree it reached the Hind
    // at 62.2 with no armour and the gunship's fire outlasted its four rockets [L the run].
    // real-health-00agent-08-bleed-4 (d3): the 00 Agent walk takes it too. The pit-lever run
    // reached the boarding legs at health 24.0 (the fusebox pit pair + the doorway men cost 76
    // upstream) and the burst rifle #20 (skin 0x500002f) closed to 3.3 u and burst three times
    // for exactly those 24.0 — death t15465 at (108.70, 28.37, −118.08) [L run-bleed4-3]. The
    // room's armour lies on the walk's own path; a player at 24 health puts it on. The gate
    // simplifies to !tops_up(): d1/d2 real-damage take it already, the topped-up sisters keep
    // their skip, so only the ignored d3 walk's timeline changes.
    if !w.tops_up() {
        w.walk_nav_to([159.4, 43.0, -113.6], "the secure room armour", 0.6);
        eprintln!("[08] the secure room armour: armour {}", w.scene.sim.as_ref().unwrap().drones.health.armour);
    }

    // ── back out of the secure room: the same waypoints in reverse, real walking ────────────
    for (name, at) in [
        ("before the safe", [160.0f32, 43.4, -117.0]),
        ("the secure room door", [158.0, 43.0, -119.0]),
        ("at the door west side", [154.6, 42.5, -121.0]),
        ("by the fusebox", [153.0, 43.4, -118.5]),
        ("the lower landing", [150.0, 43.5, -116.0]),
    ] {
        if name == "at the door west side" {
            // The way out is the door Bond came in by (#10/122 at (154.5, 42, −121.3)): walk INTO
            // its gap and press E, as the way in did. The old 1.5 nav arrival counted (155.19,
            // −119.93) — still on the room side of the wall line x 155.19, beside the kind-2 door
            // at (154.3, −119.6) — as arrived; the straight leg west to the fusebox then pressed
            // that wall and slid into the corner at (155.19, −121.78), wedged there for every
            // later leg while two guards stood in GoToGoalPosition 0x63 just outside at (154.0,
            // 43.0, −120.1) (measured on the seven-bone sight build; the arrival spot depends on
            // the run's earlier fights).
            //
            let door_i = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
                .position(|d| (d.pos[0] - 154.5).abs() < 1.5 && (d.pos[2] + 121.3).abs() < 1.5);
            // THE SWING-ARC GUARDS (latch-walks-3, measured with the temporary NF_DOORPROBE):
            // with the contact-turn rule's guards parked in the doorway, every E press opens the
            // leaf for ~2 ticks before Door_ReactToHit's anti-crush ([M 0x135980]) reverses it
            // shut on the guard capsule 0.55 u west — probe: press → 0x2→0x8 → contact
            // (153.97, 43.00, −121.05) lz +0.55 f +0.55 > eps while OPENING → 0x8→0x10 → closed,
            // 1065 cycles without one full swing. The door code is the game's ([M 0x135be0 /
            // 0x135980]); what a player does first is kill the guard shooting from inside the
            // room with him (measured #5 at (157.95, 43.00, −117.14) AIM_STAND_FIRE 1.9 u from
            // the door stop — down in one stand, health 100.0 after). The gate is the arc itself
            // on the difficulty-1 walk only: a living team-1 guard within 2.4 u of the door (the
            // measured leaf-tip + capsule reach — the reverser MUST be inside it, a bystander
            // at 2.4-3.0 does not hold the door), and the d1 difficulty keeps the d2/d3 sisters'
            // timelines byte-exact (not one extra tick on their code path). The d1 real-damage
            // sister shares _at(1) and is green — its arc is clear, so its path is unchanged
            // too. The doorway pair (#7/#8, 0x73/0x74 strafes at d 2.33/2.68) held the arc
            // through every player move tried this session: re-press, a 900-tick back-off
            // settle (their no-sight combat does not decay), the fusebox-nook slide #25 (opens
            // on E but its passage does not take Bond out), and shooting through the closed
            // leaf (no sight — stand_and_fire held). batch-1109a-fix settled it: the game has no
            // drone push out of a swinging leaf (the bounce is the game's too [L 2026-11-09]),
            // and the root-point contact was the port's stand-in — the guards' collide capsule
            // now meets the leaf itself [M 0x140e90 / 0x1ebaa8], so the leaf swings far enough
            // to open a gap on the guard behind it, and the walk shoots him through it (below).
            let arc_occupied = difficulty == 1 && door_i.is_some_and(|di| {
                let sim = w.scene.sim.as_ref().unwrap();
                let dp = sim.drones.doors.list[di].pos;
                sim.drones.list.iter().any(|g| {
                    g.present && g.enabled && !g.corpse && g.health > 0.0 && g.team == 1
                        && ((g.pos[0] - dp[0]).powi(2) + (g.pos[2] - dp[2]).powi(2)).sqrt() < 2.4
                        && (g.pos[1] - dp[1]).abs() < 4.0
                })
            });
            if arc_occupied {
                let mut fought = 0;
                loop {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let b = sim.bond.pos;
                    let target = (0..sim.drones.list.len())
                        .filter(|&i| {
                            let g = &sim.drones.list[i];
                            g.present && g.enabled && !g.corpse && g.health > 0.0 && g.team == 1
                                && ((g.pos[0] - b[0]).powi(2) + (g.pos[2] - b[2]).powi(2)).sqrt() < 3.0
                                && (g.pos[1] - b[1]).abs() < 4.0
                        })
                        .min_by(|x, y| {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let b = sim.bond.pos;
                            let (px, py) = (sim.drones.list[*x].pos, sim.drones.list[*y].pos);
                            let dx = (px[0] - b[0]).powi(2) + (px[2] - b[2]).powi(2);
                            let dy = (py[0] - b[0]).powi(2) + (py[2] - b[2]).powi(2);
                            dx.total_cmp(&dy)
                        });
                    let Some(gi) = target else { break };
                    eprintln!("[08] {name}: the arc guard #{gi} at tick {} — the door first", w.total_ticks);
                    let killed = w.switch_to_gun(name) && w.stand_and_fire(gi, 900);
                    for _ in 0..90 {
                        let p = w.scene.sim.as_ref().unwrap().bond.pitch;
                        if p.abs() < 0.01 { break; }
                        w.tick(Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
                    }
                    eprintln!("[08] {name}: arc guard #{gi} {} (killed {killed}) at tick {}, health {:.1}",
                        if killed { "down" } else { "held" }, w.total_ticks,
                        w.scene.sim.as_ref().unwrap().drones.health.health);
                    fought += 1;
                    if fought >= 4 { break; }
                }
            }
            w.walk_to(Wp { at, act: Act::Use { use_ticks: 240 }, arrive: 0.5 }, name);
            if arc_occupied {
                if let Some(di) = door_i {
                    for _ in 0..90 {
                        let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
                        if st & 0x4 != 0 { break; }
                        w.tick(Input::default(), false);
                    }
                    let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
                    eprintln!("[08] {name}: the door after the press: state {st:#x} at tick {}", w.total_ticks);
                    // batch-1109a-fix: the leaf now swings until it meets a guard's collide
                    // capsule [M 0x140e90 mask 0x800 → Collide_Intersect 0x1ebaa8's CylGeom
                    // refine] and the game bounces it on a guard who stays there [L 2026-11-09:
                    // opened to 0.4138, reversed, never OPEN]. The game does not push him out of
                    // the sweep [M NDrone2_DealWithObjHit 0x13fff0: a door hit lands in the default
                    // arm, inert for every drone type but 0x1d]. A player at a door that keeps
                    // bouncing shoots the guard behind it through the gap the swing opens, then
                    // presses again.
                    for round in 0..4 {
                        let (open, target) = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let dp = sim.drones.doors.list[di].pos;
                            let open = sim.drones.doors.list[di].state & 0x4 != 0;
                            let t = (0..sim.drones.list.len()).find(|&i| {
                                let g = &sim.drones.list[i];
                                g.present && g.enabled && !g.corpse && g.health > 0.0 && g.team == 1
                                    && ((g.pos[0] - dp[0]).powi(2) + (g.pos[2] - dp[2]).powi(2)).sqrt() < 2.4
                                    && (g.pos[1] - dp[1]).abs() < 4.0
                            });
                            (open, t)
                        };
                        if open { break; }
                        let Some(gi) = target else { break };
                        let killed = w.switch_to_gun(name) && w.stand_and_fire(gi, 900);
                        for _ in 0..90 {
                            let p = w.scene.sim.as_ref().unwrap().bond.pitch;
                            if p.abs() < 0.01 { break; }
                            w.tick(Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
                        }
                        eprintln!("[08] {name}: the guard behind the leaf #{gi} {} (round {round}) at tick {}, health {:.1}",
                            if killed { "down" } else { "held" }, w.total_ticks,
                            w.scene.sim.as_ref().unwrap().drones.health.health);
                        w.walk_to(Wp { at, act: Act::Use { use_ticks: 240 }, arrive: 0.5 }, name);
                        for _ in 0..90 {
                            if w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0x4 != 0 { break; }
                            w.tick(Input::default(), false);
                        }
                        let st = w.scene.sim.as_ref().unwrap().drones.doors.list[di].state;
                        eprintln!("[08] {name}: the door after the re-press: state {st:#x} at tick {}", w.total_ticks);
                    }
                }
            }
            continue;
        }
        w.walk_nav_to(at, name, 1.5);
    }

    // ── the gondola: the descent to the floor plate ─────────────────────────────────────────
    // The outbound descent follows path 2's own link chain ([D the path-2 dump]): the landing
    // east (147.3/149.3/151.2/153, y 42.4, z −120.5) → the north ramp (153.1, 41.15, −115.95) →
    // (153.16, 39.65, −111.63) → (148.29, 38.4, −110.84) → the mid level (143.32, 36.4, −111.43)
    // → west past the mid doors (135.1, −111/−113.5) → (132.1, −112.27) → the long link to
    // (120.5, 34.04, −112.51) → the y-33.9 hall.
    for (name, at) in [
        ("the landing east", [151.2f32, 42.5, -120.5]),
        ("through the door", [153.0f32, 42.5, -120.7]),
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
        // 00 Agent (difficulty 3): the rifle pair (#15/#16, weapon 0x0, at (122.2/122.6,
        // 34.6/35.3, -112.8/-114.7), ~10 u east of the mid west stop [L the d3 ledger]) owns the
        // open long link from its east end — on the d3 run both answered fights ("the long link
        // nav 2" #16, "the hall y33 nav 1" #15) cost 40.0 of pool while Bond stood on the open
        // link. A player fights them from the mid-doors doorway (cover at the link's west end)
        // BEFORE stepping out: the pair is 10.2 u east, dy −1..−2 — inside the close window.
        // D3-GATED: the Operative/Agent sisters' timelines stay byte-exact.
        if difficulty >= 3 && name == "the mid west" {
            fight_close_guards(&mut w, "the mid west", 4200);
        }
        // 00 Agent (difficulty 3): #16/#17 are ambush spawners (their plates fire mid-leg) —
        // the walk-out stand catches whatever the link walk enabled before the hall legs.
        // D3-GATED.
        if difficulty >= 3 && name == "the long link" {
            fight_close_guards(&mut w, "the long link", 4200);
            // 00 Agent (difficulty 3): the mid-west and long-link stands left #15/#16/#17's guns
            // ON the walked path (#15 fell 1.6 u off this waypoint, (121.5, 34.5, −113.5) [L the
            // 2026-11-03 25773-death run]) — a player scoops the free rifle before the stairwell:
            // these fights drain the pistol pool dry (the death run's platform-mouth stand opened
            // on weapon 0xc clip 0 spare 0) and the post-climb reload_full has no spare to fill
            // from. REAL INPUT: the walk-over take [L the level-11 kill-drop pass]. D3-GATED: the
            // Operative/Agent sisters' timelines stay byte-exact.
            w.resupply("the long link");
        }
    }
    // Sister: the descent was WALKED — the walkway's zone plates (TouchOnces out 42/43/44, the
    // ambush spawners' start channels [D placements; M Trigger_Hit 0x1b1fc8 walk-in mask 8])
    // fired on the way down; a teleport to the landing would leave them off.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x2a) || sim.drones.channels.on(0x2b) || sim.drones.channels.on(0x2c),
            "the walked descent crossed the walkway ambush plates (42/43/44) [D]");
    }
    // guard-glass-finish RE-PIN (Agent + 00 Agent, merged-master tree): the stairwell-top pair
    // (#23/#24, weapon 0x0, ~1-2 u from the top stop, VISIBLE — the same guards the return fire
    // fought from the foot at 11.6 u [L the 2026-11-06 NF_RF_TRACE run]) shoot down the shaft
    // while Bond descends — climb_stairs has NO return-fire hook, so the descent ate their
    // volleys unanswered. A player clears the two men at the stairwell's head BEFORE stepping
    // down. VISIBLE-ONLY stand: a buried guard must not open a stand (the d2 death's mechanism).
    // D2/D3-GATED: the Operative/Agent sisters' timelines stay byte-exact.
    if fight_pkg {
        fight_close_guards_visible(&mut w, "the stairwell top", 4200);
    }
    w.climb_stairs([107.25, 27.7, -119.5], "the stairwell down", 1.6, 4200);
    // Agent/00 Agent: the descent's return-fire stands drained the clip — the 05 sisters'
    // rule: the fights must not open on a drained clip (a mid-fight reload is free burst time
    // for the platform pair). los-mask-repins-08-agent: a D2 reload here was TRIED and RETIRED
    // on the measured run — the fights got cheaper (76.9 total hits vs 100.0, the boarding
    // chunk 74.8 → 32.1 [L the run on this branch]) but the reload's R presses re-roll the
    // shared RNG downstream (the walk's own wobble note: every hit's grunt roll draws the
    // shared stream [M Player_HandlePain 0x1902c8:111]) and the re-rolled boarding drove the
    // stuck mitigation into a flank that walked Bond off the platform's east edge (the fall:
    // 23.1 "other", death at (118.71, 22.70, −138.15) tick 30399 [L the run]). The fights keep
    // their measured timelines; the Agent chunk is answered at the fight itself (the doorway
    // slit, the fight block below).
    if difficulty >= 3 {
        w.reload_full();
    }
    // guard-glass-finish RE-PIN (Agent + 00 Agent, merged-master tree): the stairwell foot's
    // sight into the platform is blocked by the mouth's SOLID wall band (the eye ray stops at
    // x≈105.25 inside the owner-22 runs — the 2026-11-06 NF_RF_GEOM dump; NOT glass, the
    // glass-pass sight trace clears nothing here), so the guard-mode-c4-glass foot stands ran
    // eleven 61-tick stands on the burst pair (#19/#20) while it walked 8.5 u to point-blank —
    // the d2 run died at tick 25036 at the foot with the pair at d 0.6-0.7. A player does NOT
    // stand at the foot firing at a wall: he fights whatever he can SEE (this stand), walks the
    // short legs out, and takes the pair from the platform (the "platform mouth" stand below;
    // the "platform west" stands of the d3 runs killed #19 there [L the run logs]). VISIBLE-ONLY:
    // the legs' return fire answers the pair the moment it rounds the mouth's opening. The
    // Operative sister stays topped-up (the stand no-ops under tops_up) and its timeline is
    // untouched.
    if fight_pkg {
        fight_close_guards_visible(&mut w, "the stairwell mouth", 4200);
    }
    // los-mask-repins-08-agent: a pre-leg unseen lip stand at the descent's end was TRIED and
    // RETIRED on the measured run — the half-open leaves give Bond no shot line from the
    // stairwell mouth (every 61-tick episode printed "held (out of sight or cap)", no round
    // fired in 430 ticks [L the run on this branch]) while the pair circled into flanking
    // spots north of the mouth and burst the legs from the open. The pair is answered the way
    // the d3 ledger answers it: from each boarding leg's arrival spot (the per-leg lip stand
    // below, opened to d2), where "the platform west" stands of the d3 runs killed #19 [L the
    // run logs].
    // The boarding ([D the floor probe]): the platform (y 27.25, z −121..−122.5) ends at the car
    // shaft — the middle (x 112) is open below (the cabin hangs there; its floor is the plate at
    // y 28, z −127.4), the side ledges (x 110/114) run south at y 26.75 past the parked cabin's
    // mouth. Walk the west ledge south; the plate's volume (12 tris = its box) touches Bond's body.
    //
    // glass-pierce-2 (the real-damage Agent sisters): with the guard-side pass armed, the west
    // ledge IS the pair's lane — the burst-rifle pair (#19/#20) holds the platform's west end
    // (x 102-105) and both Agent sisters died in these legs with the pass armed (d3: death
    // 25116 mid-leg, #20's bursts 59.0 in 7 hits; d2: death 25209 at the mouth; the fights won
    // on the Agent timeline still cost 37-59 health at the 1.3-accuracy bursts). A player with
    // two riflemen holding the west end takes the EAST ledge (x 114 — the same [D the floor
    // probe] ledges, the same south run past the cabin's mouth) and boards from its side: the
    // plate at (112, 28, −127) is 2 u west of the east ledge's south end. The topped-up d1 walk
    // keeps the master west timeline.
    let boarding_legs: &[(&str, [f32; 3])] = if fight_pkg {
        &[
            // The first leg swings EAST of the pair's lane (their arc covers the x 107-110
            // descent line — both Agent sisters died in it with the pass armed), then the route
            // cuts back west at the cabin's north face and boards through the west doorway as
            // the master route does (the plate is reached through the west side; the east ledge
            // run stops at the cabin's east wall).
            ("the lower level east", [114.5f32, 27.7, -121.5]),
            ("the gondola mouth", [112.0, 27.4, -122.0]),
            ("the platform west", [110.3, 27.3, -122.2]),
            ("the west ledge south", [110.0, 27.0, -124.5]),
            ("the west ledge at the cabin", [110.0, 27.0, -126.3]),
        ]
    } else {
        &[
            ("the lower level east", [110.7f32, 27.7, -121.5]),
            ("the gondola mouth", [112.0, 27.4, -122.0]),
            ("the platform west", [110.3, 27.3, -122.2]),
            ("the west ledge south", [110.0, 27.0, -124.5]),
            ("the west ledge at the cabin", [110.0, 27.0, -126.3]),
        ]
    };
    for &(name, at) in boarding_legs {
        w.walk_nav_to(at, name, 1.4);
        // 00 Agent (difficulty 3) + guard-mode-c4-glass Agent re-pin: the burst-rifle pair
        // (#19/#20, weapon 0x15, skin 0x500002f, at (101.9-104.5, 28.2, -122.3/-122.6), 7-10 u
        // west [L the d3 ledgers]) fires into the boarding legs from the platform's west — the
        // d3 runs died in the legs (ticks 25164/25382/25426) before the `before the boarding`
        // stand below ran, and the Agent run died at tick 25209 once the guard rounds passed
        // the panes [M 0x12e7b0 disasm]. A player fights the pair the moment the platform mouth
        // opens (the "platform west" stand of the same runs killed #19 from there) instead of
        // walking the burst lane first. D2/D3-GATED: the Operative sister's timeline stays
        // byte-exact.
        if fight_pkg && name == "the lower level east" {
            // 00 Agent (difficulty 3): the descent's fights can leave the FIREARMS dry while the
            // AT-420 sits owned with clip 4 — the 25773 death's stand cycled the dry pistol onto
            // the LAUNCHER (see firearm_armed) and stood into #20's interval-4 bursts (13 × 6.0,
            // health 78 → 0, ticks 25375–25773 [L the run log]). A player does not fight a 9.8 u
            // burst lane rocket-handed: grab the freshest drops when no firearm has ammo (#23 and
            // #19 fell ON this floor seconds before [L the run log]), put a firearm in hand,
            // THEN take the stand — kill #20 first, from the stand's own back-off cover, and only
            // then approach the car. D3-GATED: the Operative/Agent sisters' timelines stay
            // byte-exact.
            if !firearm_armed(&w.scene.sim.as_ref().unwrap()) { w.resupply("the platform mouth"); }
            switch_to_firearm(&mut w, "the platform mouth");
            // level-08-rocket-self-blast (d3): the room's SECOND burst pair (#26/#27, skin
            // 0x500002f) closes to point-blank BEHIND the half-open door leaves (state 0x2
            // pos01 0.50 [L the 2026-11-08 run's STUCK dumps]) — no eye ray reaches them, the
            // visible-only stand picked "no visible close guard" and walked on, and the lane
            // crossing ate −12/−3 point blank at 15 health (death 29022, the run's ledger:
            // both bursts the instant the door line opened). A player at the door lip with
            // riflemen the other side pushes to the lip and shoots through the gap: when a
            // live hostile stands within 5 u, take the PLAIN stand with the shared
            // unseen-cover creep armed (dmg.unseen_fight — the flag the shared stand already
            // checks) and the back-off suspended (its 4.0 ring reads the hidden pair as a
            // crowd and backs away from the only winning spot, the lip). D3-GATED: the
            // Operative/Agent sisters' timelines stay byte-exact.
            // real-health-00agent-08-bleed: on the solid-body tree the pair NO LONGER CLOSES —
            // the measured run's pair stayed at its placements (103.0-106.4, 28.2, −121.3/
            // −122.3), 8-11.5 u from the mouth, behind the leaves, and its bursts took 24.0
            // during the shoot-out and board legs (frames 26851-27072 [L the NF_DMG_TRACE run
            // on this branch]) while every gate here read "no hostile under 5 u". A player
            // pushes to the lip for a rifleman he is taking fire from at ANY stand-off inside
            // the fight's 12 u ring, visible or not. los-mask-repins-08-agent: D2 takes it too
            // — the Agent pair bursts the legs from behind the leaves the same way (the d2
            // ledger's 33.6 from #19 landed during the leg-1 nav stall [L the NF_DMG_TRACE run
            // on this branch]), and the ring reads 15 u: at leg 1's end the surviving rifleman
            // stands 12-14 u west behind the leaves (the retreat the RF fights drive), inside
            // the fight's own approach but outside the 12 u scan.
            let close_hidden = difficulty >= 2 && {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)).sqrt() < 15.0)
            };
            if close_hidden {
                let (saved_back_off, saved_unseen) = (w.dmg.back_off, w.dmg.unseen_fight);
                w.dmg.back_off = None;
                w.dmg.unseen_fight = true;
                fight_close_guards(&mut w, "the platform mouth lip", 4200);
                w.dmg.back_off = saved_back_off;
                w.dmg.unseen_fight = saved_unseen;
            } else {
                fight_close_guards_visible(&mut w, "the platform mouth", 4200);
            }
        }
        // join-fix-g (Agent only): on the joined tree the west burst-rifle pair #26/#27 (weapon
        // 0x15) walks in along the platform from x 97-101, 12-16 u out and in view, while the
        // walk boards; #26 took the last 34.9 in 9 hits at 10.5 u with no stand on him. A player
        // on the platform mouth answers the riflemen he can see before he walks into their lane:
        // the same visible-only stand, out to 18 u.
        if difficulty == 2 && fight_pkg && name == "the lower level east" {
            fight_guards_within(&mut w, "the platform mouth west", 4200, true, 18.0);
        }
        // real-health-00agent-08-bleed: the second pair (#26/#27) wakes MID-BOARDING on the
        // solid-body tree — the measured run's first pair bursts landed 33 ticks into "the west
        // ledge south nav 2" (frames 26851/26856 at 4.5 u) while the legs' return fire needed its
        // own pick cadence to answer, and the pair cost 24.0 before both stands killed it (the
        // death run's ledger [L NF_DMG_TRACE on this branch]). A player crossing a platform with
        // a fresh ambush pair at 4-8 u clears it from each leg's arrival spot instead of walking
        // into the next burst: the same unseen lip stand after EVERY boarding leg, not just the
        // first. los-mask-repins-08-agent: D2 takes it too — the Agent run's burst pair bursts
        // the legs the same way and the leg-arrival stands are where the sight lines open (the
        // d2 ledger's 33.6 from #19 landed across the legs' walk [L the NF_DMG_TRACE run on
        // this branch]).
        if difficulty >= 2 && fight_pkg && name != "the lower level east" {
            let pair_awake = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter().any(|d| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)).sqrt() < 12.0 && (d.pos[1] - bp[1]).abs() < 4.0)
            };
            if pair_awake {
                let (saved_back_off, saved_unseen) = (w.dmg.back_off, w.dmg.unseen_fight);
                w.dmg.back_off = None;
                w.dmg.unseen_fight = true;
                fight_close_guards(&mut w, &format!("the {name} lip"), 4200);
                w.dmg.back_off = saved_back_off;
                w.dmg.unseen_fight = saved_unseen;
            }
        }
    }
    // 00 Agent (difficulty 3): the ledge's armour box #282 (kind 3, (109.5, 27.8, −129.4) [D the
    // placement census]) lies 3 u further down the west ledge Bond is standing on. A player at
    // 00 Agent takes it before the ride (armour 50 [M Pickup_Handler 0x18e650 case 3]): the Hind
    // fight's own-rocket blasts (−54/−77 at 2-5 u in the d3 runs) left the walk no margin once
    // the weapon-10 drops stopped handing Bond the stronger record 0xC [M Upgrade_Weapon
    // 0x1a8f30:18-22]. D3-GATED: the Operative/Agent sisters' timelines stay byte-exact.
    // join-fix-g: the Agent walk takes it too. On the joined tree the platform's west riflemen
    // leave Bond at 7.6 before the ride, and the ride and the Hind fight still land hits.
    // agent-copter-rate: every REAL-DAMAGE run takes it too. The Hind's route step is count/scale
    // [L ee_start_05.bin; M 0x17d340:33-41] — twice the old half-rate build — and the d1 run
    // reaches the ride on 19.6 health: the faster machine's passes put an own-rocket blast at
    // 1.4 u (kind 44, owner -1 [L the NF_DMG_TRACE run]) with no margin left. A player on
    // a fifth of his health picks up the armour lying 3 u away before the fight.
    if difficulty >= 2 || !w.tops_up() {
        w.walk_nav_to([109.6, 27.0, -129.2], "the ledge armour", 0.6);
        eprintln!("[08] the ledge armour: armour {}", w.scene.sim.as_ref().unwrap().drones.health.armour);
        // Back to the ledge spot the boarding leg starts from (the route's own arrival at
        // "the west ledge at the cabin", (109.62, 27.88, −125.14) [L the run log]): from 2 u
        // further south the east walk presses the cabin's west wall and the plate never fires.
        w.walk_nav_to([109.62, 27.88, -125.14], "back at the cabin", 0.4);
    }
    // 00 Agent (difficulty 3): the same firearm hygiene before the last pre-boarding stand —
    // the legs' return-fire stands may have spent the clip on the walk over. D3-GATED.
    if difficulty >= 3 {
        if !firearm_armed(&w.scene.sim.as_ref().unwrap()) { w.resupply("before the boarding"); }
        switch_to_firearm(&mut w, "before the boarding");
    }
    // THE CLOSE-GUARD FIGHT (level-08-real-health-rockets): the descent wave's burst-rifle
    // pair (#263/#264, weapon 0x15, skin 0x0500002f [D the audit's param dump]) closes to
    // 4.0–6.5 u on this platform and its interval-4 bursts land (the measured death run:
    // −1.80/−3.60/−3.60 at frames 24647/24663/24667, drone #19/#20 —
    // docs/reports/level-08-long-range-hits.md). A player switches to the gun for guards this
    // close before stepping into the car. REAL-DAMAGE ONLY (the fn gates on `tops_up()`).
    fight_close_guards(&mut w, "before the boarding", 4200);
    {
        // East into the open west doorway, polling for the plate.
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
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, use_key);
            if t % 300 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[08 board t{t}] bond {:?} ch4 {}", sim.bond.pos, sim.drones.channels.on(4) as u8);
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[08 gondola] bond {:?} ch4 {} falls {:?} stuck {}", sim.bond.pos,
            sim.drones.channels.on(4) as u8, w.report.falls, w.report.stuck_events.len());
        assert!(sim.drones.channels.on(4), "the cable-car floor plate fired ch 4 [D #236 mask 8]");
    }
    // mux #235 folds the REAL ch 0x4A (the briefcase take above) AND 4 → ch 1 → SP #295 (the
    // ride, 1420 ticks) starts. Watch the ride: the carriage carries Bond [M
    // Control_InheritVelocity 0x133e88] and writes ch 83 at tick 1410.
    let mut ride_started = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(1) { ride_started = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(ride_started, "mux #235 (0x4a AND 4) fired ch 1 [D]");
    let start_y = w.scene.sim.as_ref().unwrap().bond.pos[1];
    let rockets_at = |w: &Walker| { let inv = &w.scene.sim.as_ref().unwrap().drones.inventory;
        (inv.owned.get(0x2c).copied().unwrap_or(false), inv.clip.get(0x2c).copied().unwrap_or(0), inv.carried.get(0xe).copied().unwrap_or(0)) };
    eprintln!("[08] rockets boarding (owned, clip, carry): {:?}", rockets_at(&w));
    let mut ch83 = false;
    for t in 0..3600u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x53) { ch83 = true; break; }
        if difficulty == 3 {
            // retune-nb-08-6 lever 1: collect the cable car's three rocket pickups DURING the
            // ride (the ch1→ch53 loop). The ride script 0x0600004C parents them to the carriage
            // [D streams 37-39] and they ride WITH Bond — the measured car-space offsets hold
            // constant x/z through the whole descent [L this branch's t100-t1300 dump: the two
            // ammo boxes at +0.52/+1.46 and −0.42/+2.21 planar, the launcher at +0.74/−1.53],
            // each parked OUT past the take radius (root distance < bounds.radius + 1.1 [M
            // Pickup::touches 0x18e148:210-214; Pickup_Handler 0x18e650] — the t1300 ammo read
            // 1.106 u, just over), so the game's player walks to each one standing in the car
            // (the launcher is handed over mid-ride [D stream 37]). The steer reads the LIVE
            // centre every tick (the relative direction is car-space stable — no world-space
            // aim), stops inside 0.85 planar u, and the take is the game's own sphere + LOS
            // check — the owned launcher converts to carried rounds [M Pickup_Handler case 0
            // else-branch → Player_EquipAmmo]. D3-GATED: the d1/d2 sisters' ride stays
            // Input::default() byte-exact. No exposure: the machine engages only at the ride's
            // end and the car body is cover.
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let mut best: Option<([f32; 3], f32)> = None;
            for p in sim.drones.pickups.iter() {
                if p.gone || p.weapon != 0x2c || (p.kind != 0 && p.kind != 1) { continue; }
                let c = p.centre();
                let planar = ((c[0] - b[0]) * (c[0] - b[0]) + (c[2] - b[2]) * (c[2] - b[2])).sqrt();
                if planar > 4.0 { continue; }
                if best.map_or(true, |(_, d)| planar < d) { best = Some((c, planar)); }
            }
            let input = best.map(|(c, d)| {
                let desired = (c[0] - b[0]).atan2(c[2] - b[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 || d < 0.85 { 0.0 } else { 1.0 }, ..Default::default() }
            }).unwrap_or_default();
            w.tick(input, false);
        } else {
            w.tick(Input::default(), false);
        }
        if t % 300 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 ride t{t}] bond {:?} ch1 {} ch83 {} disabled {}", sim.bond.pos,
                sim.drones.channels.on(1) as u8, sim.drones.channels.on(0x53) as u8, sim.disabled);
        }
        // retune-nb-08-6: the ride-time pickup geometry the lever needs — the three cable-car
        // rocket pickups' live world centres (the ride script 0x0600004C parents them to the
        // carriage [D streams 37-39]) and their distances to Bond. Diagnostic only: eprintln,
        // no sim write, the timeline stays byte-exact with the shipped walk.
        if t % 200 == 100 {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let rocket_picks: Vec<(f32, u16, i16, [f32; 3], bool)> = sim.drones.pickups.iter()
                .filter(|p| p.weapon == 0x2c)
                .map(|p| { let c = p.centre();
                    let d = ((c[0]-b[0])*(c[0]-b[0]) + (c[1]-b[1])*(c[1]-b[1]) + (c[2]-b[2])*(c[2]-b[2])).sqrt();
                    (d, p.kind, p.rounds, c, p.gone) })
                .collect();
            eprintln!("[08 ride t{t}] rocket pickups (dist, kind, rounds, centre, gone): {:?}", rocket_picks);
        }
    }
    assert!(ch83, "the ride script wrote ch 83 (0x53) at its tick 1410 [D stream 46]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[08] ride done: bond {:?} (started y {start_y:.1}); rockets (owned, clip, carry) {:?}", sim.bond.pos, rockets_at(&w));
        assert!(!sim.drones.channels.on(0x7c), "Bond rode the carriage — the plunge (out 0x7C) never fired [D §6]");
    }
    w.wait_enabled(1500, "the bottom-of-mountain cinematic");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0078) && p.state >= 3),
            "SP #296 (0x06000078) ran on ch 83 [D]");
        assert!(sim.drones.channels.on(5), "the cinematic wrote ch 5 [D stream 10]");
    }
    // The helicopter: gate ch 5 opens the route flight (state 1) [M Copter_Update 0x17dbb8 head].
    // The faithful machine writes the out channel (rec+0x72 = param 7 = ch 0x4B [D #45 p7; M
    // 0x17d4ac]) ONLY from the death tail — hp < damage → state 4, disasm 0x17e390-0x17e3c8 — or
    // cases 4/5/6, which only the death (or a kind-2 arrival path; none exists on 08 [D the route
    // dump]) reaches. The walk shoots the Hind down with REAL aim and trigger pulls: Bond stands
    // where the cinematic left him, the aim tracks the machine's live position every tick (the
    // real look input), the damage accumulates from the weapon contacts on the part spheres
    // [M 0x17dbb8:277-353] — no position or damage pokes.
    let mut flying = false;
    for _ in 0..600 {
        if w.scene.sim.as_ref().unwrap().drones.copters.iter().any(|c| c.out == 0x4b && (1..=2).contains(&c.state)) { flying = true; break; }
        w.tick(Input::default(), false);
        // hind-flight-pin: the arrival log — every 30 ticks from the gate opening [M Copter_Update
        // 0x17dbb8 head], the copter's live flight record.
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.frame % 30 == 0 {
            for c in sim.drones.copters.iter().filter(|c| c.out == 0x4b) {
                eprintln!("[08 flight] f{} pos ({:.2},{:.2},{:.2}) state {} path {:?} t {:.2} dmg {:.1}",
                    sim.drones.frame, c.pos()[0], c.pos()[1], c.pos()[2], c.state, c.path, c.t, c.damage);
            }
        }
    }
    assert!(flying, "the gate ch 5 opened the route flight (state 1/2) [M 0x17dbb8 head]");
    // hind-flight-pin: the flight record starts HERE, at the gate opening — the same every-30-
    // frames (pos, state, path, t) samples the fight loop takes, through whatever the walk does
    // before the fight (the 00 Agent pane shoot-out, the rocket switch). The pin below reads the
    // whole observed flight; a fight that wins in three straight hits (466 ticks, 87.3 u from the
    // fight's start alone — measured on batch-1109a-fix) still has the gate-to-death flight.
    {
        let park = park.borrow();
        let worst = park.iter().copied().enumerate().max_by_key(|(_, c)| *c).unwrap();
        eprintln!("[08] guard-room door arc: worst 0x63 park guard #{} {} ticks", worst.0, worst.1);
        assert!(worst.1 < 300,
            "guard #{} held 0x63 in the guard-room door arc for {} ticks — the truncated-route park",
            worst.0, worst.1);
    }
    let pre_fight: std::rc::Rc<std::cell::RefCell<Vec<([f32; 3], u16, Option<usize>, f32)>>> = Default::default();
    let pre_fight_frame: std::rc::Rc<std::cell::Cell<u32>> = Default::default();
    {
        let rec = pre_fight.clone();
        let last = pre_fight_frame.clone();
        w.on_step = Some(Box::new(move |scene: &nf_view::scene::Scene, _| {
            let sim = scene.sim.as_ref().unwrap();
            if sim.drones.frame % 30 == 0 && sim.drones.frame != last.get() {
                last.set(sim.drones.frame);
                if let Some(c) = sim.drones.copters.iter().find(|c| c.out == 0x4b) {
                    rec.borrow_mut().push((c.pos(), c.state, c.path, c.t));
                }
            }
        }));
    }
    // retune-nb-08-10: this whole drop sequence runs RIGHT AFTER the bottom-of-mountain
    // cinematic (was: after the pane shoot-out). The terminal's hurt volume arms with the
    // machine's approach — the 2026-11-20 baseline's first zap landed t25666, ~106 ticks
    // after wait_enabled returned, while the shoot-out still stood Bond at the ride-end spot
    // (111.7, 13.5, -179.2) — and the walk-down took 12 x -2.00 (health 44.0 -> 20.0 at fight
    // entry [L the baseline run]). The drop takes ~65 ticks from the ride-end spot (the walk
    // 40 [L the baseline: the anchor arrived at +40], the press and the fall ~25), so running
    // it FIRST lands the zap-free plate (feet 13.086 [L run 1's drop]) BEFORE the arming, and
    // the whole pre-fight tail (the lockout, the stand-off walk, the stand press, the pane
    // shoot-out) then runs from the plate/rim — the rim (feet 13.47) reads nearly zap-free [L
    // the round-6 ledger: ONE -1.73 row in ~800 fight ticks]. The fight-entry health returns
    // to the earned 44.0. D3-GATED: the sisters never run the block, their timelines stay
    // byte-exact.
    // retune-nb-08-4 (d3): walk to the fight anchor BEFORE the loop. The loop's walk-back
    // overwrites the aim's turn while it runs (its input.turn points at the anchor, its
    // input.look = 0.0), so a fight that OPENS off the anchor never tracks the machine: the
    // measured starvation [L the 2026-11-19 NF_DMG_TRACE run] — bond parked at the ride-end
    // spot (111.096, −179.73), 1.41 u off the anchor (112.5, −179.4), aim_yaw_err
    // 2.404→3.035 for 500 fight ticks (cross_still 101→501 — the cursor frozen) while the
    // machine's LOW passes (y 12.8-14.1, the eye/gun sight stops t0.894-0.896 ≥ 0.8) crossed
    // the launchable bearings — ZERO launches, the window starved; the wedge that parked him
    // broke only when a missile blast's dodge strafe moved him (t500-600). The walker's
    // walk_to breaks wedges with its own sidestep mitigation and puts Bond INSIDE the
    // walk-back's 1.2 u hold before the loop starts, so the aim tracks from tick 0 and the
    // first low pass is a live launch window. D3-GATED: the sisters' timelines stay
    // byte-exact (the d1 real-damage sister runs neither walk; the d2 Agent sister keeps the
    // slit walk above).
    // retune-nb-08-4 run 2: the r(0.6) arrival parked Bond on the cabin's UPPER floor lip
    // (111.99, 13.52, −179.52) — the hurt volume's band (its zaps hit feet y ≈ 13.3-13.5, the
    // lower floor 13.086 is clear [L the run: 8 −2.00 zaps at (112.0, 13.5, −179.5), fight
    // t0-t285, death t20993 while the standing fight's rocket 1 LANDED 101.0 at t197]) — the
    // target walks him PAST the lip onto the lower floor: the zap-free plate the 13.086-floor
    // traces cover ((111.99..112.25, −179.16..−179.52) [L the same run's traces]).
    // retune-nb-08-4 run 3: the (112.4, −179.3) r(0.45) target still "arrived" IN x/z while
    // Bond stood on the upper lip (the walk's arrival is the x/z radius — (112.02, −179.45) is
    // 0.42 from the target without the 0.44-y step) — the zaps resumed (8 more, death t21077,
    // rockets 1 landed / 2 launched). The target moves past the lip to the lower plate's east
    // edge (113.0, −179.2 — the traces' 13.086 floor runs to the cabin's east band x ≈ 113.6
    // [D the eye-ray stops]): the walk presses THROUGH the lip, the 0.44 drop is gravity's own
    // step-down (the raw walk-back's drop is the measured precedent [L run 1: bond y 13.52 →
    // 13.0856 in the first 100 fight ticks]), and the arrival lands Bond on the zap-free plate
    // still inside the fight anchor's 1.2 u hold ((113.0, −179.2) is 0.58 u from (112.5, −179.4)).
    // retune-nb-08-4 run 4: the (113.0, −179.2) target "arrived" at (112.55, 13.52, −179.36) —
    // the UPPER floor extends east past x 112.3 at z −179.36 [L the run: the launch from
    // (112.55, 14.22) — feet 13.52 — and the zaps drained the fight again, rockets 1 landed /
    // death t20994]. The measured LOWER-floor plate (feet 13.086) is the hole Bond dropped
    // into in run 1: (111.99, −179.52) [L run 1: the in-place drop y 13.52 → 13.0856 with x
    // parked] through (112.14..112.25, −179.16..−179.18) [L run 1: the post-dodge position].
    // The target aims AT the hole with a 0.3 radius: the walk presses onto the hole's edge and
    // gravity does the drop, the arrival checks the full plate, and Bond fights from feet
    // 13.086 — the zap-free floor.
    if difficulty >= 3 {
        eprintln!("[08 tail] the drop block starts f{}", w.scene.sim.as_ref().unwrap().drones.frame);
        w.walk_to(Wp::new(112.0, 13.08, -179.5).r(0.1), "the d3 fight anchor");
    }
    // retune-nb-08-4 run 8: the raw press that finishes the drop. The walk_to's x/z arrival
    // cannot step Bond down (runs 3-7: every target 'arrived' on the UPPER floor feet 13.52
    // short of the hole edge x ≈ 111.99 — the hurt volume's band zaps the upper floor, the
    // lower floor feet 13.086 is zap-free [L run 1's in-place drop y 13.52 → 13.0856 at
    // (111.99, −179.52)]). The measured drop mechanism is the CONTINUOUS raw press: run 1's
    // walk-back walked Bond ONTO the hole and gravity did the 0.44 step-down in place. The
    // press repeats the walk-back's own input shape (turn toward the anchor, forward when
    // aligned) until Bond's feet reach the lower floor (y ≤ 13.2), bounded at 80 ticks; the
    // aim holds the anchor bearing for the press's ~20-40 ticks, and the machine's low passes
    // recur (~700 ticks per circuit, the 100-200-tick windows [L the flight logs]) so the
    // converged aim still catches them. D3-GATED: the sisters' timelines stay byte-exact.
    if difficulty >= 3 {
        for _ in 0..80 {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            if b[1] <= 13.2 { break; }
            // run 8: pressing toward the ANCHOR slid Bond around the hole (he ended at
            // (112.25, 13.52, −179.63) — the hole is a small patch at the run-1 drop spot
            // (111.99, −179.52)); the press aims AT the hole itself, the drop breaks the loop.
            let dx = 111.99 - b[0];
            let dz = -179.52 - b[2];
            let want_yaw = dx.atan2(dz);
            let yaw_err = walker::wrap_pi(want_yaw - sim.bond.yaw);
            w.tick(Input { turn: yaw_err.clamp(-0.08, 0.08), forward: if yaw_err.abs() < 0.3 { 1.0 } else { 0.0 }, ..Default::default() }, false);
        }
        eprintln!("[08 anchor] bond {:?} after the drop press", w.scene.sim.as_ref().unwrap().bond.pos);
        // run 9: Bond reached the exact run-1 drop spot (111.998, 13.52, −179.514) and the
        // standing capsule BRIDGED the hole — run 1's drop happened under the CROUCH (the
        // smaller capsule falls through). The crouch-drop-uncrouch: one crouch press (the
        // toggle), the fall (bounded), one uncrouch press — the fight then runs STANDING on
        // the lower floor (feet 13.086, eye 13.77 — above the ledge 13.5, under the lintel
        // 14.31), zap-free.
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.pos[1] > 13.2 {
            w.tick(Input { crouch: true, ..Default::default() }, false);
            for _ in 0..60 {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.bond.pos[1] <= 13.2 { break; }
                w.tick(Input::default(), false);
            }
            w.tick(Input { crouch: true, ..Default::default() }, false);
            w.tick(Input::default(), false);
            eprintln!("[08 anchor] bond {:?} after the crouch drop", w.scene.sim.as_ref().unwrap().bond.pos);
            // run 11 TRIED and REVERTED: forcing the stance standing after the drop (the
            // default-tick release, then a crouch toggle, then the assert) did NOT uncrouch
            // Bond (the log: crouched true after 4 default ticks + a crouch press — the
            // port's crouch latch resists both semantics from the walk's side) and the extra
            // ~6 upper-floor ticks caught a missile blast (death t20760, was t21070) — the
            // block is reverted to the run-10 state. The stance question needs the main
            // agent's live check (the report lists it): what input state stands Bond up from
            // the port's crouch latch.
        }
        // retune-nb-08-6 stand-off lever: the fight must run STANDING on the lower floor
        // (eye 13.77 — above the exterior ledge 13.3-13.5, under the lintel 14.31 — the
        // proven flat-ray launch windows; the crouched eye 13.3 is blindfolded and its only
        // rising launches self-blast in the west band — the shipped run-10 death t21070 and
        // this branch's re-measure). The round-5 stand press WORKS mechanically [L: crouched
        // false once the lockout is out] but the tried spots (the hole footprint, the plate's
        // (112.2, −179.2) edge) put the standing capsule ON the hole's rim — it re-bridges
        // (feet 13.47, eye 14.18, the launch self-blasts −14.00, t20944/t20942). UNTRIED: a
        // stand spot a full planar unit OFF the hole centre (111.99, −179.51) — the lower
        // plate runs north-east of the hole (the run-1 post-dodge positions (112.14..112.25,
        // −179.16..−179.18) held feet 13.086) toward the cabin's east band (the 13.086 floor
        // runs to x ≈ 113.6 [D the eye-ray stops]). The walk stays CROUCHED (the crouch
        // capsule is inside the pit) and presses north-east ONLY while the feet stay on the
        // 13.086 plate (feet ≤ 13.15 — the upper plateau's 13.52 step is the hurt volume's
        // zap band, never climbed); then the stand press holds crouch until crouched false
        // (the lockout armed by the drop's crouch press expires on the zap-free pit floor —
        // [M Player_ChangeSubState 0x190630:48 arms BL+0x960 = 60 on every substate change;
        // the round-5 press measured the stand through the same latch]). D3-GATED: the
        // sisters' timelines stay byte-exact.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 standoff] pit entry bond {:?} crouched {}", sim.bond.pos, sim.bond.crouched);
            // 1) the lockout out (70 default ticks on the zap-free pit floor).
            for _ in 0..70 { w.tick(Input::default(), false); }
            // 2) the crouched east walk to the off-hole stand spot (113.3, −179.25) — the
            // first try's (112.8, −179.0) stand popped to the rim 13.47 [L the lever run:
            // the crouch walk held feet 13.0856 to (112.74, −179.06) but the stand press
            // put the standing capsule back on the step edge (feet 13.470, the launch
            // t377 missed, the fight bled at the rim)]; the 13.086 floor runs east to the
            // cabin's east band x ≈ 113.6 [D the eye-ray stops], so the stand spot pushes
            // further east where the plate is wider than the standing capsule.
            let stand = [113.3f32, -179.25f32];
            // retune-nb-08-11: the shipped walk WEDGED at the pit's northeast corner — the
            // k-stamps parked at (113.08286, −179.23654) from k80 through k220, the (113.3,
            // −179.25) target 0.22 u short of reachable [L the 2026-11-20 baseline
            // instrumentation run] — ~140 of the drop block's 437 ticks pressing a wall. A
            // player whose press stops moving him stops pressing: the walk breaks 30 ticks
            // after its last progress (≥ 0.01 u), from the SAME spot the shipped run's k220
            // exit parked at (the stamps: x/z frozen to 5 decimals k80-k220 [L the same run]).
            let mut stuck_at: Option<[f32; 2]> = None;
            let mut stuck_for = 0u32;
            for k in 0..220u32 {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                if b[1] > 13.15 { break; }
                let dx = stand[0] - b[0];
                let dz = stand[1] - b[2];
                if dx * dx + dz * dz < 0.12 * 0.12 { break; }
                match stuck_at {
                    Some(p) if (b[0] - p[0]).powi(2) + (b[2] - p[1]).powi(2) < 0.01 * 0.01 => { stuck_for += 1; }
                    _ => { stuck_at = Some([b[0], b[2]]); stuck_for = 0; }
                }
                if stuck_for >= 30 { eprintln!("[08 standoff] wedged k{k} at ({:.5}, {:.5}) — the press stopped", b[0], b[2]); break; }
                let want_yaw = dx.atan2(dz);
                let yaw_err = walker::wrap_pi(want_yaw - sim.bond.yaw);
                w.tick(Input { turn: yaw_err.clamp(-0.08, 0.08), forward: if yaw_err.abs() < 0.3 { 1.0 } else { 0.0 }, ..Default::default() }, false);
                if k % 40 == 0 { let sim = w.scene.sim.as_ref().unwrap();
                    eprintln!("[08 standoff] walk k{k} bond {:?} crouched {}", sim.bond.pos, sim.bond.crouched); }
            }
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 standoff] walked bond {:?} crouched {}", sim.bond.pos, sim.bond.crouched);
            // 3) the stand press: hold crouch until crouched false (bounded 80 ticks).
            for k in 0..80u32 {
                let sim = w.scene.sim.as_ref().unwrap();
                if !sim.bond.crouched { break; }
                w.tick(Input { crouch: true, ..Default::default() }, false);
                if k % 20 == 0 { let sim = w.scene.sim.as_ref().unwrap();
                    eprintln!("[08 standoff] stand k{k} bond {:?} crouched {}", sim.bond.pos, sim.bond.crouched); }
            }
            w.tick(Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 standoff] stood bond {:?} crouched {}", sim.bond.pos, sim.bond.crouched);
            eprintln!("[08 tail] the drop block ends f{}", sim.drones.frame);
        }
    }

    // 00 Agent (difficulty 3): SHOOT OUT the carriage's north and south panes before the fight.
    // The cabin box stops the eye rays 0.4-3.6 u out in every direction [L the run sweeps] and
    // there is NO floor outside to walk to (the probe: floor_below is None at every candidate
    // 2-8 u out — the terminal hangs over the valley, the kill floor y −24 [D #42/#43/#44]) —
    // the game's player fights from INSIDE, through the panes he has shot out. The measured
    // proof: the fix1 run's first rocket (t191) broke the north pane and the north ray then read
    // CLEAR 30 u for the rest of the fight (the t600-t2400 sweeps az0.00:1.00) while the intact
    // south stayed blocked (az3.14:0.08) [L the run logs]. With a pane broken the launch
    // corridor is open air — a round cannot detonate on the carriage's own wall band next to
    // Bond (the two measured d3 deaths: explosions at d 5.8 and d 2.2 [L NF_DMG_TRACE]). Real
    // aim and trigger, the in-hand firearm; the hold stops when the ray flies 30 u.
    //
    // glass-pierce-2: the shoot-out is no longer D3-ONLY. With the guard-side pass armed the d1
    // real-damage sister's fight timeline shifted and its rockets detonated on the carriage's
    // own wall band next to Bond — the death log's world hits −37.50 (f37643) and −13.90
    // (f37894) at (111.1, 13.5, −179.7) are the same own-rocket signature the d3 runs measured
    // (docs/reports/level-08-long-range-hits.md: four "world" hits are Bond's OWN rockets). The
    // game's player fights from INSIDE through panes he has shot out on EVERY difficulty — the
    // prep is the player tactic, not a difficulty rule. The topped-up sisters keep their
    // doorway-slot timelines (their top-up absorbs a detonation; the prep stays REAL-DAMAGE
    // ONLY, gated on tops_up() like every fight here).
    if difficulty >= 3 || !w.tops_up() {
        // The cabin's glass: the live panes' boxes name the shoot-out targets (the gondola's
        // windows are script panes, kind 2, health 1.0 — one bullet contact kills [M
        // breakable.rs bullet_hit; the pane test]). The ends' panes (|dz| > 1.5) are the fight's
        // launch corridors: the machine circuits north and south of the terminal, and a broken
        // pane leaves 30 u of open air on its bearing [L the fix1 run — the first rocket broke
        // the north pane and the north ray read CLEAR 30 u for the rest of the fight].
        let bond_at = w.scene.sim.as_ref().unwrap().bond.pos;
        let panes: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().drones.breaks.list.iter()
            .filter(|b| b.kind == 2 && !b.dead)
            .map(|b| [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0])
            .filter(|c| (c[0] - bond_at[0]).abs() < 3.0 && (c[1] - bond_at[1]).abs() < 2.5 && (c[2] - bond_at[2]).abs() > 1.5)
            .collect();
        eprintln!("[08 panes] live end panes near the cabin: {panes:?}");
        // The 00agent economy (the 02 walk's loot rule, muzzle-repin-6): the fights' rounds moved
        // with the muzzle-draw stream — the measured run reached the shoot-out with clip 3
        // spare 0, killed pane 0 with the last rounds and walked the rest dry. A player scoops
        // the end-fight drops at his feet and arms the fullest gun before shooting glass. The
        // d1/d2 runs' green streams stay byte-identical.
        if difficulty >= 3 {
            eprintln!("[08 tail] scoop starts f{}", w.scene.sim.as_ref().unwrap().drones.frame);
            scoop_drops(&mut w, "the panes", 3);
            eprintln!("[08 tail] scoop ends f{}", w.scene.sim.as_ref().unwrap().drones.frame);
            arm_fullest_at(&mut w, "the panes", true);
            eprintln!("[08 tail] arm ends f{}", w.scene.sim.as_ref().unwrap().drones.frame);
        }
        switch_to_firearm(&mut w, "the panes");
        eprintln!("[08 tail] switch ends f{}", w.scene.sim.as_ref().unwrap().drones.frame);
        // The shoot-out's rounds must be LIVE rounds (guard-repin-master): the cabin fights'
        // ammo drift decides the clip state here (the fights' shape moved with the guards'
        // per-goal arrival radii), and the measured red ran pane 1 dry (clip 0 spare 33) so
        // pane 2's five pulls were dry-fire clicks until the walker's own reload landed
        // mid-loop — "STILL GLAZED after 5 rounds". A player reloads before a planned glass
        // shoot: the game's R held to a full clip [M Player_WeaponFiring's reload branch],
        // real input, no ammo writes (reload_full is the walker's additive R law).
        w.reload_full();
        eprintln!("[08 tail] reload ends f{} clip {} spare {}", w.scene.sim.as_ref().unwrap().drones.frame, w.scene.sim.as_ref().unwrap().gun.clip, w.scene.sim.as_ref().unwrap().gun.spare);
        // A player shoots the glass he can see — the aim-ray-to-bounds-centre gate CANNOT work
        // here: the captured centre is the pane bounds' midpoint, which sits a hair short of the
        // sheet plane (the north pane's centre ray eye→centre reads OPEN while a perpendicular
        // ray through the sheet hits at t 0.52 [L the probe raytest 2026-11-04]), so every ray
        // reads OPEN before a single round and the old cleared-gated tap fired nothing (the
        // orphaned-pane symptom of the hind session: 9 rounds were the earlier fights, dead rows
        // []). The pane damage law needs no new sim code — the port's contact already drains it:
        // the game's collision pass registers the round's contact on the pane's obj+0xD0 chain
        // whether or not the round stops (the mirrored contact) and `Break_Update` 0x12bfd8:36-37
        // drains health from that chain; the port's seg_box_t contact over the pane bounds does
        // the same (drone.rs's player-bullet pass). The kill drops the row and the script object
        // (Break_Kill's obj+0xFE delete → retain_script_panes → the dynamic quads leave the set),
        // which is what "breaks and clears" means for the launch corridors. So: aim at the pane
        // centre with the real look input, pull the trigger on a CONVERGED aim (the fire edge
        // every 12 ticks — a held trigger is one pull [L the pane hold]), until the pane's row
        // dies (the kill drops it from breaks.list).
        let pane_gone = |sim: &nf_view::scene::Sim, c: [f32; 3]| {
            sim.drones.breaks.list.iter().filter(|b| b.kind == 2 && !b.dead).all(|b| {
                let m = [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0];
                (m[0] - c[0]).powi(2) + (m[1] - c[1]).powi(2) + (m[2] - c[2]).powi(2) > 0.7 * 0.7
            })
        };
        // glass-pierce-2: the sweep REPEATS over the still-glazed panes — a player whose rounds
        // spread off the thin sheet keeps shooting until the glass breaks. Pass 1 is the exact
        // pre-existing sweep (same aims, same draws); a later pass only runs when a pane
        // survived its 5 converged rounds (the d2 run left one pane glazed: the spread draws
        // missed the ~0.1 u sheet within the cap).
        for pass in 0..3u32 {
            if panes.iter().all(|c| pane_gone(&w.scene.sim.as_ref().unwrap(), *c)) { break; }
            if pass > 0 { eprintln!("[08 panes] pass {pass}: a pane is still glazed — a player keeps shooting"); }
        for (k, c) in panes.iter().enumerate() {
            if pane_gone(&w.scene.sim.as_ref().unwrap(), *c) { continue; }
            eprintln!("[08 tail] pane {k} window starts f{} at {:?}", w.scene.sim.as_ref().unwrap().drones.frame, c);
            let mut fired = 0u32;
            // The bracket (the #447 shoot_mine lesson, muzzle-repin-6): the round flies from the
            // camera along the view, its gravity arc and the camera-vs-head offset land it a
            // hair off a ~0.1-thin sheet — track each live round's closest approach to the pane
            // centre and walk the aim by the measured miss vector when a round dies without the
            // kill (the P2K's spread is 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24], so one
            // correction lands the next round). A round dying FAR out (≥ 1 u) is a BLOCKED line
            // (a mullion between), not a near miss — corrections would chase the blocker, so
            // the window ends and a sidestep re-takes the sight line instead.
            let mut aim = *c;
            let mut best: Option<(f32, [f32; 3])> = None;
            let mut prev_b = 0usize;
            let mut fixes = 0u32;
            let mut blocked = false;
            'window: for attempt in 0..3u32 {
                if attempt > 0 {
                    if !blocked { break 'window; }
                    // The sidestep: face the pane and shuffle aside (real input only), then a
                    // fresh bracket window from the plain centre aim.
                    for s in 0..120u32 {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let head = sim.bond.head();
                        let to = [c[0] - head[0], c[1] - head[1], c[2] - head[2]];
                        let yaw_err = walker::wrap_pi(to[0].atan2(to[2]) - sim.bond.yaw);
                        w.tick(Input { turn: yaw_err.clamp(-0.08, 0.08), strafe: if (s / 40) % 2 == 0 { 1.0 } else { -1.0 }, ..Default::default() }, false);
                    }
                    eprintln!("[08 tail] pane {k}: blocked after {fired} — sidestep f{}-", w.scene.sim.as_ref().unwrap().drones.frame);
                    eprintln!("[08 panes] pane {k}: the line was blocked — sidestepped, fresh window");
                    aim = *c; best = None; prev_b = 0; fixes = 0; fired = 0; blocked = false;
                }
                for t in 0..260u32 {
                    if pane_gone(&w.scene.sim.as_ref().unwrap(), *c) { break 'window; }
                    let sim = w.scene.sim.as_ref().unwrap();
                    let head = sim.bond.head();
                    let to = [aim[0] - head[0], aim[1] - head[1], aim[2] - head[2]];
                    let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt().max(0.01);
                    let desired_yaw = to[0].atan2(to[2]);
                    let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
                    let yaw_err = walker::wrap_pi(desired_yaw - sim.bond.yaw);
                    let pitch_err = desired_pitch - sim.bond.pitch;
                    let input = Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.05, 0.05), ..Default::default() };
                    // Fire on the edge, only on a converged aim — the pane's bounds box is thin
                    // (0.9-1.2 u across, ~0.1 thick [L the rows' bounds]) and an off-aim round
                    // misses it (the health 1.0 dies to one contact [D params [2,1,0]]).
                    let fire = t % 12 == 0 && yaw_err.abs() < 0.01 && pitch_err.abs() < 0.02 && fired < 12;
                    if fire { fired += 1; eprintln!("[08 tail] pane {k} fire {} f{} yaw_err {yaw_err:.4} pitch_err {pitch_err:.4}", fired, w.scene.sim.as_ref().unwrap().drones.frame); w.tick_fire(input); } else { w.tick(input, false); }
                    {
                        let sim = w.scene.sim.as_ref().unwrap();
                        for b in &sim.drones.player_bullets {
                            let d = (b.pos[0] - c[0]).powi(2) + (b.pos[1] - c[1]).powi(2) + (b.pos[2] - c[2]).powi(2);
                            let bd = best.map_or(f32::INFINITY, |(x, _)| x);
                            if d < bd { best = Some((d.sqrt(), b.pos)); }
                        }
                        if sim.drones.player_bullets.len() < prev_b && !pane_gone(sim, *c) {
                            if let Some((d, p)) = best.take() {
                                if d >= 1.0 {
                                    blocked = true;
                                    break;
                                } else if d > 0.1 && fixes < 3 {
                                    aim = [aim[0] + c[0] - p[0], aim[1] + c[1] - p[1], aim[2] + c[2] - p[2]];
                                    fixes += 1;
                                    eprintln!("[08 panes] pane {k}: the round missed by {d:.2} u — the aim walks to {aim:?}");
                                }
                            }
                        }
                        prev_b = sim.drones.player_bullets.len();
                    }
                }
            }
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[08 tail] pane {k} window ends f{} (fired {fired})", sim.drones.frame);
            eprintln!("[08 panes] pane {k} at {c:?}: {} after {fired} rounds, clip {} spare {}",
                if pane_gone(sim, *c) { "SHOT OUT" } else { "STILL GLAZED" }, sim.gun.clip, sim.gun.spare);
        }
        }
        {
            // The proof the shoot-out happened: every end pane's row is gone (killed rows drop),
            // and the two end bearings read open air 30 u out — a shot-out window breaks AND
            // clears for the player [L the fix1 run — the broken pane's bearing read CLEAR 30 u].
            let sim = w.scene.sim.as_ref().unwrap();
            let alive: Vec<[f32; 3]> = sim.drones.breaks.list.iter().filter(|b| b.kind == 2 && !b.dead)
                .map(|b| [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0]).collect();
            let unbroken = panes.iter().filter(|c| !pane_gone(sim, **c)).count();
            eprintln!("[08 panes] after the shoot-out: {unbroken} end panes still glazed, {} live kind-2 rows total", alive.len());
            let b = sim.bond.pos;
            let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
            for (name, c) in [("north", panes.iter().find(|c| c[2] < b[2] - 1.5)), ("south", panes.iter().find(|c| c[2] > b[2] + 1.5))] {
                let Some(c) = c else { continue };
                let dir = [(c[0] - eye[0]) / 3.0, (c[1] - eye[1]) / 3.0, (c[2] - eye[2]) / 3.0];
                let far = [eye[0] + dir[0] * 30.0, eye[1] + dir[1] * 30.0, eye[2] + dir[2] * 30.0];
                let t = sim.world.segment_bullet(eye, far);
                eprintln!("[08 panes] the {name} bearing 30 u out: {}", if t.is_none() || t.is_some_and(|t| t >= 0.999) { "CLEAR" } else { "blocked" });
            }
            assert_eq!(unbroken, 0, "the shoot-out must kill every end pane (the rows' mirrored contact drains [M Break_Update 0x12bfd8:36-37]) — {unbroken} still glazed");
            eprintln!("[08 tail] the shoot-out ends f{}", sim.drones.frame);
        }
    }
    assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b), "no arrival while the route flies");
    // los-mask-repins-08-agent (d2): the fight from the WEST DOORWAY SLIT. The end panes are all
    // shot out (the prep above) and the machine's rounds fly the same glass-pass holes the
    // rockets fly — the standing spot between the holes eats every strafing burst's falloff
    // (the d2 ledger: 14.9 in 34 world hits then the −3.57 killing blast, tick 37022 [L the
    // NF_DMG_TRACE run on this branch]). The cabin's wall band is the only cover: its west
    // doorway (the opening Bond boarded through at the top; the band's z gap −178.25..−180.6,
    // lintel y ≈ 14.31 [D the tri dump]) is the firing slit the walk's doorway-fight comment
    // names — a player steps into the slit and fights from the band's edge, the band shielding
    // the machine's bursts while the low passes still give fire windows. Real walk input only
    // (the pane block's own sidestep pattern, bounded); the fight loop's aim and launch gates
    // read the live spot, so the rockets now start through the slit. The Operative sister's
    // green timeline stays byte-exact (d2-gated).
    // retune-nb-08-4 (d3): the slit walk is the D2 fight's own lever (the d2 anchor below IS the
    // slit (110.98, −179.31)). The d3 fight anchors DEEPER at its own measured clear spot
    // (112.5, −179.4) — on the lever-1 phase the slit placement wedges the fight: the walk-back
    // fights the slit spot and Bond wedges on the cabin's step edge at (110.98, 13.09, −179.32)
    // [L the 2026-11-19 NF_DMG_TRACE run: bond x parked 110.982→110.985 over 700 fight ticks,
    // y 13.52 → 13.09 inside the first 100], stands IN the terminal's hurt volume there (10
    // −2.00 zaps, frames 20731-21778), and the crouched eye's sight rays stop on the west band
    // 0.6 u out (t0.017-t0.032 every sample) — ZERO launches, clip 4 intact, the machine LOW the
    // whole window (y 16.5-20.5 every 30-tick sample [L the same run's flight log]) — death
    // t21777 with health exactly 18.0 at fight entry. D3 now skips the walk and fights from the
    // ride-end spot its walk-back was built to hold. The d1 real-damage sister never took this
    // walk (difficulty < 2) and the d2 Agent sister keeps it byte-exact (== 2).
    if difficulty == 2 {
        let slit = [110.5f32, -179.4f32];
        let sim = w.scene.sim.as_ref().unwrap();
        let start = sim.bond.pos;
        for _ in 0..300 {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let dx = slit[0] - b[0];
            let dz = slit[1] - b[2];
            if dx * dx + dz * dz < 0.35 * 0.35 { break; }
            let want_yaw = dx.atan2(dz);
            let yaw_err = walker::wrap_pi(want_yaw - sim.bond.yaw);
            w.tick(Input { turn: yaw_err.clamp(-0.08, 0.08), forward: if yaw_err.abs() < 0.3 { 1.0 } else { 0.0 }, ..Default::default() }, false);
        }
        let arrived = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[08 slit] bond {:?} -> {:?} (target x {:.2} z {:.2})", start, arrived, slit[0], slit[1]);
    }
    // retune-nb-08-10: the d3 anchor walk + the drop press + the crouch drop + the stand-off
    // walk + the stand press moved ABOVE the pane shoot-out (right after the bottom-of-mountain
    // cinematic) — the drop must land the zap-free plate before the hurt volume arms with the
    // machine's approach (the note at the block). The shoot-out now runs from the rim.

    let ci = w.scene.sim.as_ref().unwrap().drones.copters.iter().position(|c| c.out == 0x4b).unwrap();
    let hp = w.scene.sim.as_ref().unwrap().drones.copters[ci].hp;
    // The game's way down: the AT-420 Sentinel. The level hands it out twice — the safe's launcher
    // #291 (kind 0, id 0x2C) beside the briefcase, its rounds forced to 4 on this level only
    // [M Pickup_Create 0x18dda0:116-118 `GameState+0xc == 0x7000008 && id == 0x2c`], and the cable
    // car's three pickups (a launcher and two rocket-ammo boxes, 4 each, parented to the carriage
    // [D 0x0600004C streams 37-39]) that Bond collects standing in the car on the ride. The copter's
    // hp is DiffMod·299 [M Copter_Create 0x17d340: 1.0 below difficulty 2]; the body (slot 2) is
    // the only part a pick reaches [M Copter_Register 0x17e530:7; Collide_PickObj 0x1eae80's
    // always-on 0x20 gate], so the P2K needs 86 rounds (3.5 [L weapon record 2]) — far past the
    // 48 the carry starts 07 with. A rocket on the body blasts at the contact [M
    // Bullet_CollisionHandler 0x12ce80's 0x4 arm] and `Explode_Propagate` 0x17c518 case 0 adds
    // ≈100 (radius 8, damage 100 [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C]) plus the round's
    // 1.0 contact: three rockets (303 > 299), one spare. The walk fires the 'Guided' mode the
    // launcher is picked up in and flies each rocket into the body with the real look input
    // (`Bullet_update` 0x12e7b0:177-244's steering) — no ammo or damage pokes.
    const ROCKET: u8 = 0x2c;
    {
        // retune-nb-08-5: the pickup scan the task orders — every un-gone pickup within 60 u
        // of Bond at fight entry (kind: 0 weapon, 1 ammo, 2 key, 3 ARMOUR [M pickup.rs
        // kind::ARMOUR; take() hard-sets armour 50.0 when < 50.0, 0x18e650:220-225], 4 label,
        // 5 message, 6 bonus) plus the walk's armour/health. Diagnostic only: no sim tick
        // runs here, the timeline stays byte-exact with master's shipped walk.
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let mut near: Vec<(f32, u16, u16, i16, [f32; 3])> = sim.drones.pickups.iter()
            .filter(|p| !p.gone)
            .map(|p| { let c = p.centre();
                let d = ((c[0]-b[0])*(c[0]-b[0]) + (c[1]-b[1])*(c[1]-b[1]) + (c[2]-b[2])*(c[2]-b[2])).sqrt();
                (d, p.kind, p.weapon, p.rounds, c) })
            .filter(|e| e.0 < 60.0)
            .collect();
        near.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        eprintln!("[08 pickups] bond armour {:.1} health {:.1} at fight entry; un-gone pickups within 60 u: {:?}", sim.drones.health.armour, sim.drones.health.health, near);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        let owned: Vec<u8> = (0x02u8..0x4e).filter(|&wid| inv.owned.get(wid as usize).copied().unwrap_or(false)).collect();
        eprintln!("[08] owned weapons at the fight: {owned:02x?} (current {:#x}); rocket clip {} carried {}", sim.gun.weapon,
            inv.clip.get(ROCKET as usize).copied().unwrap_or(0), inv.carried.get(ROCKET as usize).copied().unwrap_or(0));
        assert!(inv.owned.get(ROCKET as usize).copied().unwrap_or(false), "the walk took the safe's rocket launcher #291");
        let at = sim.weapons.ammo_type(ROCKET as usize);
        eprintln!("[08] rocket ammo type {at:#x}: carried {} gun.spare {}; rocket pickups {:?}", inv.carried.get(at).copied().unwrap_or(0), sim.gun.spare,
            sim.drones.pickups.iter().filter(|p| p.weapon == ROCKET as u16).map(|p| (p.kind, p.rounds, p.gone, p.from, p.m[3])).collect::<Vec<_>>());
    }
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == ROCKET { break; }
        w.tick(Input { switch_to: Some(ROCKET), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, ROCKET, "the switch to the rocket launcher never landed");
    let mut rockets = 0u32;
    let mut ch4b = false;
    let mut riding = false;
    let mut launch_damage = 0.0f32;
    let mut best_rocket = 0.0f32;
    // guard-glass-finish: the free-aim cursor's stillness tracker (d3 only). RETIRED: the two
    // launch-gate variants built on it (a |cross| < 0.05 magnitude gate and a 20-tick stillness
    // gate) both failed the fight — the magnitude gate starved it (0 launches, the Hind's own
    // rocket blasts chipped Bond down, death 29996), and the stillness gate opened t43 with the
    // cursor frozen yet the round STILL flew ~0.13 rad off the gated corridor and died on the
    // pane-8 frame band (death 27559) — the spawn-tick bearing is not recoverable from the
    // gate-tick cursor state. The d3 launch self-blast stays OPEN (see the report).
    let mut cross_prev = [0.0f32; 2];
    let mut cross_still = 0u32;
    // hind-flight-pin: the every-30-ticks flight record (pos, state, path, t) [M Copter_Move
    // 0x17d628 / Copter_Update 0x17dbb8].
    w.on_step = None;
    let mut flight_samples: Vec<([f32; 3], u16, Option<usize>, f32)> = pre_fight.borrow().clone();
    // One sample per frame: the gate record's last frame is not sampled twice.
    let mut last_sample_frame = if flight_samples.is_empty() { u32::MAX } else { pre_fight_frame.get() };
    // retune-nb-08-10: the APPROACH flight dump — the pre-fight record (the machine's spawn →
    // approach, the frames before the fight loop opens) the next window needs to find the LOW
    // dive's true frame window (the 2000 observed fight ticks never saw the machine below
    // y 18.4, and the round-4 launchable band y 12.8-13.7 must live in the approach). Diagnostic
    // only: eprintln, no sim tick, the timeline stays byte-exact. D3-GATED.
    if difficulty >= 3 {
        let n = flight_samples.len();
        let last_f = pre_fight_frame.get();
        for (i, (p, st, pa, pt)) in flight_samples.iter().enumerate() {
            eprintln!("[08 approach {i}] f{} pos ({:.2},{:.2},{:.2}) state {} path {:?} t {:.2}",
                last_f.saturating_sub(((n - 1 - i) as u32) * 30), p[0], p[1], p[2], st, pa, pt);
        }
    }
    // The reposition state: Bond arrives INSIDE the bottom terminal (the ride's end (112, 13.5,
    // −179.7) sits in a walled box — the eye ray stops at x ≈ 110.4/113.6 and z ≈ −176.6/−182.2
    // in every direction, eye height [D the measured stops]). The machine's circuit is outside,
    // and the game's player walks out of the terminal to fight it — the death run's rocket 2
    // reached the machine only because the GUIDED round's per-tick steering threaded the
    // opening [M Bullet_update 0x12e7b0:177-244] while rockets 1/3/4 exploded on the walls
    // 0.65–5.5 u from Bond. So when the corridor to the machine is blocked, Bond LOOKS for the
    // opening (a 16-ray azimuth sweep at eye height — the farthest clear ray, preferring the
    // machine's bearing) and WALKS out (real forward input, grounded only), then fights from
    // the open pad.
    let mut exit_ticks = 0u32;
    // level-08-rocket-self-blast: the machine's per-tick position tracker (the forecast gate's
    // velocity source — the loop runs one sim tick per iteration).
    let mut prev_cp = [0.0f32; 3];
    let mut have_cp = false;
    // real-health-00agent-08-bleed (d3): the missile-dodge state — the side step off each
    // inbound round's line (see the dodge block inside the loop) and which side the current
    // episode steps to.
    let mut dodge_sign = 1.0f32;
    let mut dodge_hold = 0u32;
    // retune-nb-08-7 (d3): the salvo's LATCHED dodge side — the 2026-11-20 lever run measured the
    // per-episode side re-pick as a ZIG-ZAG: the anchor-ward bias recomputes every episode start
    // and flips as Bond crosses the anchor line, so the episodes walked him back over the old
    // ground and the second salvo's line (aimed at his spawn-tick spot) caught him at contact
    // (−15.00 at d 0.3, fight t745) after the first salvo was dodged for −2.70 [L the run]. The
    // side latches at the first episode of a salvo and holds until the salvo dies.
    let mut dodge_side = 0.0f32;
    // real-health-00agent-08-bleed-4 (d3): the ZAP hold — a world hit under Bond's feet (owner
    // past the drone list — the volume law, no blast row) means he is standing IN the terminal's
    // hurt volume; the walk-out holds for 40 ticks against the dodge (run-bleed4-4/6).
    let mut zap_hold = 0u32;
    // nb-scan-all-08: the fight stands CROUCHED at the DEEPER anchor (112.5, −179.4) on the
    // nb-scan-all phase, both real-damage sisters. The 11-14 d3 lever's measured answer to the
    // doorway lane: the Hind's gun rounds come down through the west doorway (x 103 → 111, y 16
    // → 14 [L the round log]) and stopped landing on the crouched deeper spot; measured with the
    // crouch there: death t20428 → t26671, rocket 1 launched and landed 100.7. The nb-scan-all
    // fight starts ~6500 ticks earlier in the machine's circuit [L the 2026-11-18 runs — rockets
    // 1/2 identical t49/t229/t275/t428, the whole upstream 37505 → ~28900] and its passes line up
    // WITH the doorway lane: at the slit the rounds fly the gap itself (the d2 lever run bled
    // −1.40 × 44 at the slit spot (111.10, −179.67) and the killing −49.20 blast landed there —
    // the dodge strafes perpendicular to a westward view, i.e. inside the gap plane, no escape
    // [L run 2]) while the deeper crouched spot is OUT of the lane's landing band. D3 MEASURED
    // on the old phase, reused as the player's stance on the new one; the d1 real-damage sister
    // (difficulty < 2) never crouches, its timeline byte-exact.
    // nb-scan-all-08 (d2): the fight OPENS crouched at the slit anchor — the movement inputs
    // (the dodge episodes and the drift walk) stand Bond back up into the lintel-window launch
    // geometry (the green run's launches all leave from the standing eye y 14.22 [L the run]);
    // the opening crouch blunts the first doorway-lane bursts. d3 keeps its original crouch at
    // (112.5, −179.4) (its fight code is unchanged — the ledger lives in its ignore message).
    // retune-nb-08-4 (d3): the d3 fight STANDS. The crouched eye (13.3) is BLINDFOLDED at the
    // anchor: the terminal's exterior ledge (its top y ≈ 13.3-13.5, 1.5-3 u out) stops EVERY
    // bearing's ray at t0.04-0.15 [L the 2026-11-19 NF_DMG_TRACE run: the crouched 16-ray sweep
    // az0.00:0.10 … az5.89:0.09, the UP ray t0.15, the eye/gun stops t0.042 at (110.4, 13.3,
    // −180.8)] while the same bearings from the STANDING eye (14.2) read CLEAR 30 u [L the same
    // run's t0 sweep az0.00:1.00 … az5.89:1.00]. The only rays the crouch lets out RISE steeply
    // (the machine's high close passes) — and those launches climbed into the west band above
    // the doorway slot and detonated 5.5 u from Bond (the −20.00 blast at t333, the round dead,
    // clip 3 left [L the same run]). The d2 fight's winning geometry is the standing eye + the
    // machine's LOW passes (the five standing-slit launches [L the 2026-11-18 run]); the d3
    // fight takes the same stance: flat rays through the doorway slot, the ledge cleared, the
    // rounds dying on the far terrain if they die (k > 8 — no self-damage). The machine fired
    // NOTHING for the first 500+ fight ticks on the lever-1 phase (bullets [] at every
    // 30-tick sample [L the same run]) — the doorway-lane bursts the crouch blunted are not
    // this phase's bleed; the volume zaps and the self-blasts were. D3-GATED: the d2 Agent
    // sister keeps its crouch-open fight byte-exact; the d1 real-damage sister never crouches.
    if difficulty == 2 { w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false); }
    for t in 0..20000u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4b) { ch4b = true; break; }
        let sim = w.scene.sim.as_ref().unwrap();
        // The cursor's stillness tick — tracked for the report's diagnostics, not gated on (see
        // the tracker's declaration above).
        {
            let cn = sim.cross;
            if (cn[0] - cross_prev[0]).abs() < 0.003 && (cn[1] - cross_prev[1]).abs() < 0.003 { cross_still += 1 } else { cross_still = 0; }
            cross_prev = cn;
        }
        // hind-flight-pin: the fight log — every 30 ticks to the death [M the death tail writes
        // ch 0x4B at 0x17e3bc-0x17e3c8].
        if sim.drones.frame % 30 == 0 && sim.drones.frame != last_sample_frame {
            last_sample_frame = sim.drones.frame;
            let c = &sim.drones.copters[ci];
            flight_samples.push((c.pos(), c.state, c.path, c.t));
            eprintln!("[08 flight] f{} pos ({:.2},{:.2},{:.2}) state {} path {:?} t {:.2} dmg {:.1}/{:.0} bullets {}",
                sim.drones.frame, c.pos()[0], c.pos()[1], c.pos()[2], c.state, c.path, c.t, c.damage, hp,
                sim.drones.bullets.iter().filter(|b| b.state != 2).count());
        }
        let cp = sim.drones.copters[ci].pos();
        let rocket = sim.drones.player_bullets.iter().find(|b| b.guided_cam && !b.dead).map(|b| (b.pos, b.dir));
        let (from, yaw, pitch) = match rocket {
            // Riding the rocket: the look input steers it [M 0x12e7b0:177-244; scene.rs's ride].
            Some((p, d)) => (p, d[0].atan2(d[2]), d[1].clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2),
            None => (sim.bond.head(), sim.bond.yaw, sim.bond.pitch),
        };
        let to = [cp[0] - from[0], cp[1] - from[1], cp[2] - from[2]];
        let l = (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt();
        let desired_yaw = to[0].atan2(to[2]);
        let desired_pitch = (to[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        // 00 Agent: the shot leaves through the CROSSHAIR, not the view centre — the aim law is
        // look + 0.76966·cx·right + 0.57724·cy·up, normalized [M the crosshair pass: the shot's
        // direction through the crosshair]. The walk's own tracker saturates the free-aim cursor
        // at its clamp while it converges (the fire dump at every launch: cross (−0.296, ±0.394),
        // the cursor pinned), which put every measured launch up to 0.23 rad off the tracker's
        // converged view — rocket 4's round crossed the south-east corner post 3 u out and the
        // blast took −62.5 at 2.2 u [L the diag run 2026-11-04 — the world hit at frame 28470].
        // The player aims the CROSSHAIR: the tracker's error (and the slit/corridor lanes below)
        // read the crosshair's own bearing, so a clamped cursor recenters before the shot. The
        // ride keeps the rocket-dir steering (the cursor is frozen mid-flight) [M the missile-cam
        // block zeroes the input the cursor arm would see]. D3-GATED: the sisters' timelines stay
        // byte-exact.
        let (aim_ref_yaw, aim_ref_pitch, aim_dir) = if difficulty >= 3 && rocket.is_none() {
            let (syy, cyy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            // `bond.pitch` is BL+0x8A8 in units of π/2, positive up (the walk's own aim code).
            let p = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
            let right = [cyy, 0.0, -syy];
            let up = [-p.sin() * syy, p.cos(), -p.sin() * cyy];
            let tx = 0.76966 * sim.cross[0];
            let ty = 0.57724 * sim.cross[1];
            let dd = [syy * p.cos() + tx * right[0] + ty * up[0],
                p.sin() + tx * right[1] + ty * up[1],
                cyy * p.cos() + tx * right[2] + ty * up[2]];
            let ll = (dd[0] * dd[0] + dd[1] * dd[1] + dd[2] * dd[2]).sqrt().max(1e-6);
            let ad = [dd[0] / ll, dd[1] / ll, dd[2] / ll];
            (ad[0].atan2(ad[2]), ad[1].clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2, ad)
        } else {
            let (sy, syp) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let pa = sim.bond.pitch * std::f32::consts::FRAC_PI_2;
            let (sp, spp) = (pa.sin(), pa.cos());
            (yaw, pitch, [sy * spp, sp, syp * spp])
        };
        let yaw_err = walker::wrap_pi(desired_yaw - aim_ref_yaw);
        let pitch_err = desired_pitch - aim_ref_pitch;
        // REAL-HEALTH TACTIC (level-08-real-health-rockets) — pick weapon and range the way a
        // player would:
        // 1. NO ROCKET AT A TARGET INSIDE THE BLAST RADIUS: the AT-420's blast is radius 8,
        //    ≈100 [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C; M Explode_Propagate 0x17c518 case
        //    0] with the (int)-truncated linear falloff × the 0.6 level factor [M :69-71;
        //    Player_HandlePain's table] — the fire gate holds at 10 u, and Bond steps back
        //    (aim held) while the machine is inside that reach (its circuit closes; the flight
        //    time can close 2-3 u more).
        // 2. NO ROCKET DOWN A BLOCKED LINE: the round flies the straight corridor its muzzle
        //    opens [M Bullet_update 0x12e7b0], and the death run's four "world" hits
        //    −52.5/−7.5/−22.5/−3.7 (frames 26477/26580/26639/26696) are Bond's OWN rockets
        //    1/3/4 exploding on the bottom-station geometry 0.65–5.5 u from him — only rocket
        //    2 reached the machine (docs/reports/level-08-long-range-hits.md). The launch
        //    needs the clear corridor (`launch_line_clear`, eye + both gun stand-ins).
        // The slit gate: what kills Bond is a round whose FIRST flight segments run into the
        // cabin walls (the death run's 0.65–5.5 u detonations). The player-model is not a
        // straight sniper line — the round is GUIDED and steers toward the machine's live
        // position every tick [M Bullet_update 0x12e7b0:177-244] — it is "fire through the
        // doorway, then steer": the muzzle ray's first 3 u must leave the cabin clear, and the
        // machine must be beyond the blast's 10 u reach. A round that dies mid-flight after
        // that detonates on the far terrain (k > 8 of the falloff — no self-damage).
        // (`aim_dir` above: the crosshair's own bearing on 00 Agent, the view's on the sisters.)
        let b = sim.bond.pos;
        let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
        // The slit gate reads the FLIGHT query on 00 Agent (the round's own world trace — the
        // glass pass [M Collide_Update 0x1ea398:216-229; Bullet_update disasm 0x12f048-0x12f058])
        // so an aim through the live panes reads as the game's flight reads it: the mat-0x10
        // pane quads pass, the mat-0x06 frame walls stop. The Operative/Agent sisters keep the
        // plain segment_bullet — their timelines stay byte-exact.
        let slit_clear = if difficulty >= 3 {
            sim.world.segment_flight(eye, [eye[0] + aim_dir[0] * 3.0, eye[1] + aim_dir[1] * 3.0, eye[2] + aim_dir[2] * 3.0], true).is_none()
        } else {
            sim.world.segment_bullet(eye, [eye[0] + aim_dir[0] * 3.0, eye[1] + aim_dir[1] * 3.0, eye[2] + aim_dir[2] * 3.0]).is_none()
        };
        // 00 Agent (difficulty 3): the launch CORRIDOR — the first 12 u of the round's flight
        // must be clear air from the eye AND both gun stand-ins (the round spawns at the muzzle
        // [M the fire block]; the AT-420's blast is radius 8, ≈100 [L ee_start_05.bin
        // + 0x2C·0x10C]). The measured d3 deaths are rounds that cleared the 3 u slit but chased
        // the machine behind the terminal and died on its own wall band next to Bond — explosion
        // at (112.4, 15.0, −174.3) d 5.8 → −37.5 (frame 27680) and at (110.5, 14.7, −181.4)
        // d 2.2 → −40.5 (frame 30000) [L the NF_DMG_TRACE run — the wall stops the eye/gunR/gunL
        // rays at t0.17 ≈ 5 u on the first, ~2.4 u on the second]. A player fires only on a
        // corridor that cannot detonate on him. D3-GATED: the Operative/Agent sisters' timelines
        // stay byte-exact.
        let corridor_clear = difficulty < 3 || {
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            // guard-glass-finish: the round's bearing is set at the SPAWN tick — the launcher's
            // release frame, ticks after this gate — and the tracker's free-aim cursor has moved
            // by then (the measured launches sat up to 0.23 rad off the gated bearing [L the diag
            // runs]; the 2026-11-06 runs: a clear 12-u corridor gated, and the round still
            // detonated at (113.6, 14.4, −177.6), 0.13 rad east of the copter bearing, on the
            // pane-8 frame band — the −26 world hit at tick 27552). The CONE form of this gate
            // (clearing yaw ±0.30 × pitch ±0.15) starves the fight instead — from inside the
            // cabin every bearing is within ~3 u of a frame band and only the shot-out pane
            // holes thread at all — so the spawn error is killed at its SOURCE instead: the
            // `cursor_settled` launch condition below holds the trigger until the free-aim
            // cursor has decayed to centre (the tracker's view-convergence drains it — scene.rs
            // `want = cursor − turn/0.76966`), at which point the spawn-tick bearing IS the
            // gated bearing. The corridor stays the single-ray flight query (the live panes'
            // mat-0x10 quads pass, the mat-0x06 frames stop [M 0x1ea398:216-229]).
            let (dyaw, dpitch) = (aim_dir[0].atan2(aim_dir[2]), (aim_dir[1].clamp(-1.0, 1.0)).asin());
            let d = [dyaw.sin() * dpitch.cos(), dpitch.sin(), dyaw.cos() * dpitch.cos()];
            [eye,
             [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
             [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]]
                .iter()
                .all(|from| { let from = *from; sim.world
                    .segment_flight(from, [from[0] + d[0] * 12.0, from[1] + d[1] * 12.0, from[2] + d[2] * 12.0], true)
                    .is_none() })
        };
        // level-08-hind-selfblast-2 (d3): the round's FIRST ticks fly along the EFFECTIVE aim —
        // the view plus the eased cursor offsets (the yard-stand law; on d3 the crosshair law
        // saturates the tracker and the measured launches sat up to 0.23 rad off the view [L
        // the diag runs, the file's own sister note]) — so the aim-direction corridor above is
        // NOT the corridor the round flies. The measured 28178 death: every gate cleared on
        // the aim direction and the round detonated on the east wall band 2.7 u from Bond (the
        // [dmg] explosion at (113.6, 14.3, −180.3), −75.0, kind 44, owner −1) — 0.23 rad at
        // 2.7 u is 0.6 u of lateral error, the jamb's width. The same three-source 12 u cast
        // along the effective aim direction (view + aim_offset, the yard-stand effective-aim
        // convention): a launch whose own first 12 u are clear air cannot detonate on the
        // cabin next to Bond. D3-GATED: the sisters' timelines stay byte-exact.
        let effective_corridor_clear = difficulty < 3 || {
            let ey = sim.bond.yaw + sim.drones.aim_offset[0];
            let ep = sim.bond.pitch * std::f32::consts::FRAC_PI_2 + sim.drones.aim_offset[1];
            let eff = [ey.sin() * ep.cos(), ep.sin(), ey.cos() * ep.cos()];
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            [eye,
             [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
             [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]]
                .iter()
                .all(|from| { let from = *from; sim.world
                    .segment_flight(from, [from[0] + eff[0] * 12.0, from[1] + eff[1] * 12.0, from[2] + eff[2] * 12.0], true)
                    .is_none() })
        };
        let line_ok = slit_clear && launch_line_clear(sim, cp);
        // The doorway fight: Bond rode down INSIDE the carriage (the cabin floor is the origin's
        // −4.5 [D stream 1; the ride logs]) and the game's fight is fought from in here — the
        // cabin's west doorway (the opening he boarded through at the top; the wall band's
        // z gap −178.25..−180.6 in the measured tri dump) is the firing slit, its top edge
        // (the lintel at y ≈ 14.31 [D the tri dump]) just above the aiming eye (14.24). The
        // machine's circuit dips below the lintel's sight line (y ≈ 14.6 at 40 u) on its low
        // passes — THOSE are the fire windows. The launch gates: the corridor must be clear at
        // the eye AND both gun stand-ins (the round spawns at the muzzle [M the fire block] and
        // the death run's rockets exploded at 0.65–5.5 u on the walls a blocked launch
        // produces); the aim tolerance is loose (±0.10 rad) because the round is GUIDED — the
        // ride steers it toward the machine's live position every tick [M Bullet_update
        // 0x12e7b0:177-244] — and the launch only has to START through the slit.
        let mut input = Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            look: pitch_err.clamp(-0.05, 0.05),
            forward: if rocket.is_none() && l < 10.0 { -1.0 } else { 0.0 },
            aiming: rocket.is_none(),
            ..Default::default()
        };
        // real-health-00agent-08-bleed (d3): the MISSILE DODGE (the laser-hinges-2 law — the
        // Hind's missiles fly straight at Bond's launch-time position, and a 2-3 u side step
        // off each missile's line is the dodge; the hind-fight-2 scan ran it against a live
        // missile's real Bullet fields). The walk stands the strict launch gates still, so a
        // round aimed at the standing spot lands there — the death run's world hits are ten
        // machine blasts for 15.0 ending in the −15.00 k=6 blast at 6 u (frames 30447-30508
        // [L the NF_DMG_TRACE run on this branch]), health 6.0 before it. A player watches the
        // rounds he can see coming and steps off their line: any live drone round 2-30 u out
        // whose flight line passes within 2 u of Bond's chest is dodged — strafe perpendicular
        // to the view for the episode, hold the side step ~24 ticks past the last threat, and
        // hold fire while moving (the launch gates re-read on the settled spot). los-mask-repins-08:
        // the real-damage sisters take it too — under the game's guard sight the Hind's strafing
        // rockets detonate on the carriage's own wall band next to the standing spot and the
        // standing walk bleeds out (the d1 death ledger: −1.20 × 15 world hits then the −23.00
        // blast at frames 29154-29332, (111.1, 13.5, −179.7) [L the NF_DMG_TRACE run on this
        // branch]; the same own-rocket signature the d3 runs measured). Same gate shape as the
        // sight gates above: difficulty ≥ 2 or the real-damage walk (`!tops_up()`), so the
        // topped-up Operative walk keeps its byte-exact timeline.
        let dodge_now = if (difficulty >= 2 || !w.tops_up()) && rocket.is_none() {
            let b = sim.bond.pos;
            let chest = [b[0], b[1] + 0.6, b[2]];
            sim.drones.bullets.iter().any(|bu| {
                if bu.state == 2 { return false; }
                // 00 Agent: the Hind CANNON's rounds (0x61) do not start a dodge - they pass within
                // 2 u nearly every tick and held dodge_hold > 0 for 5519 of ~7000 fight ticks (the
                // per-gate tally), blocking every launch; the missiles still dodge.
                // 11-15: from rocket 3 on the cannon rounds DO start a dodge again - the launch
                // windows left are the machine's SW/S passes and the cannon chips (-2.0 each) were
                // the fight's main bleed after the missiles [L the NF_DMG_TRACE runs].
                if difficulty >= 3 && bu.weapon == 0x61 && rockets < 3 { return false; }
                let rel = [chest[0] - bu.pos[0], chest[1] - bu.pos[1], chest[2] - bu.pos[2]];
                let d = (rel[0] * rel[0] + rel[1] * rel[1] + rel[2] * rel[2]).sqrt();
                if !(2.0..30.0).contains(&d) { return false; }
                let vm = (bu.dir[0] * bu.dir[0] + bu.dir[1] * bu.dir[1] + bu.dir[2] * bu.dir[2]).sqrt().max(1e-6);
                let along = (rel[0] * bu.dir[0] + rel[1] * bu.dir[1] + rel[2] * bu.dir[2]) / (d * vm);
                if along <= 0.0 { return false; }
                let miss = d * (1.0 - along * along).max(0.0).sqrt();
                // retune-nb-08-11 (d3): the band widens to 4.0. The compressed fight's first
                // salvo passed at miss ~3.4 — outside the shipped 2.0 band — and its detonation
                // at the launch chest's west-band corner (110.4, 14.1, −181.0) d 3.4 cost the
                // full −26.00 falloff on a stationary Bond, killing the walk with 24.0 left
                // [L the 2026-11-20 lever run: rocket 1 launched t36 dist 49.9, the 10 dense
                // zaps t25702-25807, the −26.00 at t25822]. A 2.0-4.0 pass detonates ~5.5 u
                // off a strafed Bond (−7-class) instead of 3.4 off a parked one (−26.0).
                // D3-GATED: the d2 sister's band stays byte-exact.
                miss < if difficulty >= 3 { 4.0 } else { 2.0 }
            })
        } else { false };
        // real-health-00agent-08-bleed-4 (d3): the fight anchor — the ride-end spot every green
        // run fights from (111.09, −179.72). The volume law's −2.0/6-tick rows cover (113.0,
        // −180.6) and (111.14, −180.95) [L run-bleed4-4/6]; the clear spot is the anchor the
        // dodge bias and the drift walk both keep Bond on.
        // 00 Agent (11-14): the anchor sits DEEPER in the carriage (112.5, -179.4) and Bond fights
        // crouched - the Hind's gun rounds come down through the west doorway (x 103 -> 111,
        // y 16 -> 14 [L the round log]) and stopped landing there; measured with the crouch:
        // death t20428 -> t26671, rocket 1 launched and landed 100.7. (Uncrouched at the same
        // anchor: t20403; (112.0, -179.0) crouched: t23916.)
        // real-health-00agent-08-bleed-5 (d3): the north-strip variant ([112.3, −177.6], off the
        // doorway's cannon lane) was TAKEN and measured WORSE: the cannon stopped (no −2.0 rows)
        // but the machine's missiles chase the strip — −5.94/−13.16/−15.00/−15.00 blast rows,
        // death t31593, EARLIER than the anchor's t34720 [L run-bleed5-3]. NOT shipped: the
        // anchor stays (112.5, −179.4).
        // nb-scan-all-08: the anchor is the DEEPER crouched spot (112.5, −179.4) for both
        // real-damage sisters (difficulty >= 2) — the doorway lane's rounds land x 103-111 and
        // stopped hitting the crouched deeper spot on the 11-14 d3 runs [L the round log]; the
        // nb-scan-all phase lines the machine up with the doorway lane and the slit spot bleeds
        // (the d2 lever run: −1.40 × 44 + the killing −49.20 blast at the slit, death t30395
        // [L run 2]). The bias and the drift walk keep Bond here; the ride-end slit spot
        // (111.09, −179.72) stays the d<2 const (unused below difficulty 2 — the bias and the
        // walk-back blocks are difficulty >= 2).
        // nb-scan-all-08: the D2 anchor moves to the doorway slit (110.98, −179.31) — the spot the
        // nb-scan-all fight starts at and the spot whose standing eye wins (the d2 lever run:
        // five rockets, 502.5 of 448.5 hp, end health 79.0 armour 24.1 [L the 2026-11-18 run]).
        // retune-nb-08-10: the d3 anchor moves to the shipped stand spot (113.08, −179.02) —
        // the spot the round-6 stand-off walk + stand press land on. The lever-1 run (the drop
        // shipped before the cinematic's arming, entry 41.5) measured the OLD anchor (112.5-112.6,
        // −179.39) at rim height ZAPPING the fight away: 13 −2.00 rows in the last ~830 fight
        // ticks (t26811-27636, the dense burst every 6 ticks t27409-27636) closed a walk that had
        // survived every missile salvo [L the 2026-11-20 lever-1 run]. The stand spot measured
        // ONE −2.53 row in ~430 pre-fight + fight ticks (the same lever-1 run's t26175) and the
        // round-6 rim fight read ONE −1.73 row in ~800 fight ticks — an order quieter — and the
        // round-6 launches FIRED from it (t377 at the machine's y 18.4 pass, t585 at y 20.8 [L
        // the round-6 ledger]). The d3 fight then holds the zap-free spot while the machine's
        // circuit brings the low launch band around (the round-7 finding: the low band arrives
        // at fight t~1400-2100). The north strip and the corners stay rejected (the round-7
        // ledger). The d2 anchor stays byte-exact. difficulty < 2 never reads the anchor.
        let ANCHOR: [f32; 2] = if difficulty >= 3 { [113.08, -179.02] }
            else if difficulty == 2 { [110.98, -179.31] }
            else { [111.09, -179.72] };
        // The zap detector: the newest player_hit is a world hit (owner past the drone list)
        // whose frame is fresh — the volume law's −2.0 rows (run-bleed4-4: frames 20344-20441,
        // (113.0, 13.5, −180.6), no explosion lines).
        {
            let zap_now = sim.drones.player_hits.last().is_some_and(|h| {
                h.1 >= sim.drones.list.len() && sim.drones.frame.saturating_sub(h.0) <= 3
            });
            if zap_now { zap_hold = 40; } else if zap_hold > 0 { zap_hold -= 1; }
        }
        if dodge_now {
            if dodge_hold == 0 { dodge_sign = -dodge_sign; }
            dodge_hold = 24;
        } else if dodge_hold > 0 {
            dodge_hold -= 1;
        }
        // retune-nb-08-7 (d3): a live missile salvo pins the walk-back AND drives the dodge. The
        // salvo's aim point is the chest the Hind launched at [L the diagnostic run: the killer's
        // dir fixed from spawn, straight through the parked chest]; walking back to the anchor
        // while a w62 round still flies at that point walks INTO its blast, and stopping between
        // the dodge's 24-tick episodes sits Bond ON the line while the round closes. Two measured
        // bands set the drive's window [L the 2026-11-20 runs]: a pass at miss >= ~1.6 detonates
        // on the west wall band for 0.0 to −2.7 (the lever-2 run's t350 salvo: miss 1.61 → the
        // −0.00 rows; the lever-1 run's crawl: miss 1.74 → −2.70 at f21779) while a pass inside
        // the contact band detonates AT Bond (−15.00 + the falloff row at d 0.3). And the drive
        // must STOP once every live round's miss clears 1.6: the unlimited drive walked Bond 3 u
        // into the box's NW hurt-volume corner and the volume's −2.00/6-12-tick rows closed the
        // walk (the lever-3 run's death f21765-21807 at (111.99, −180.55)) — the safe NE zone is
        // ~0.5-0.8 u across and the dodge spends ~15-20 ticks of band-edge exposure, not the
        // whole flight. D3-GATED: the d2 sister's walk-back byte-exact.
        let (salvo_live, salvo_close) = if difficulty >= 3 {
            let mut live = false;
            let mut close = false;
            for bu in sim.drones.bullets.iter() {
                if bu.state == 2 || bu.weapon != 0x62 { continue; }
                let rel = [b[0] - bu.pos[0], b[1] + 0.6 - bu.pos[1], b[2] - bu.pos[2]];
                let d = (rel[0] * rel[0] + rel[1] * rel[1] + rel[2] * rel[2]).sqrt();
                if d > 50.0 { continue; }
                let vm = (bu.dir[0] * bu.dir[0] + bu.dir[1] * bu.dir[1] + bu.dir[2] * bu.dir[2]).sqrt().max(1e-6);
                let along = (rel[0] * bu.dir[0] + rel[1] * bu.dir[1] + rel[2] * bu.dir[2]) / (d * vm);
                if along <= 0.0 { continue; }
                let miss = d * (1.0 - along * along).max(0.0).sqrt();
                if miss < 3.0 { live = true; }
                // retune-nb-08-11 (d3): the drive's close band widens with the dodge gate above
                // (the same 3.4-miss round: salvo_close false at 2.0 → the drive never strafed
                // even with dodge_hold latched [L the same run]). d3-only by construction (the
                // whole salvo block is difficulty >= 3).
                if miss < 4.0 { close = true; }
            }
            (live, close)
        } else { (false, false) };
        let dodge_drive = if difficulty >= 3 { salvo_close && (dodge_hold > 0 || salvo_live) } else { dodge_hold > 0 };
        if dodge_drive {
            if difficulty >= 2 {
            // real-health-00agent-08-bleed-4 (d3): the dodge SIDE is the one that also walks the
            // anchor-ward component. The missile's miss band is symmetric (the dodge gate's own
            // miss < 2 u test — either side steps off the line), so the side choice is free, and
            // the alternating `dodge_sign` was the measured drift source: the lever run's
            // episodes walked Bond 2 u east of the ride-end spot into the terminal's east band
            // and a hurt volume drained −2.0 every 6 ticks for the last 12.0 (frames 20344-20441
            // at (113.0, 13.5, −180.6) [L run-bleed4-4]); the zap-anchor variant that walked him
            // back AFTER the drift pinned him at (111.14, −180.95) — the volume reaches the
            // south band too — and died earlier [L run-bleed4-6]. A player side-steps toward
            // the spot he means to keep firing from. The strafe stays perpendicular to the view
            // (the dodge's own law); the component toward the ride-end anchor (111.09, −179.72)
            // picks its sign.
            // nb-scan-all-08: the gate opens to difficulty >= 2. The nb-scan-all Agent fight
            // dies of the alternating form's drift: the episodes walked Bond 2.3 u east of the
            // slit to (112.9-113.0, −177.5), OUT of the band gap (z −178.25..−180.6 [D the tri
            // dump]) into the pane-hole shooting gallery, and 59 machine drips (−1.20/−1.40) plus
            // the killing blast drained 79.0 while the launches stalled — 4 rockets landed
            // (401.8 of 448.5) and Bond died at the 5th launch, tick 31483 [L the 2026-11-18
            // run]. The d1 real-damage sister keeps the alternating-sign timeline byte-exact
            // (the regressed ungated form hit it at difficulty 1; the else branch below is its
            // old code path unchanged).
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            let (ax, az) = (ANCHOR[0] - b[0], ANCHOR[1] - b[2]);
            let dot = right[0] * ax + right[2] * az;
            input.strafe = if dot > 0.05 { 1.0 } else if dot < -0.05 { -1.0 } else { dodge_sign };
            input.forward = 0.0;
            // retune-nb-08-7 (d3): the dodge releases the AIM. Two measured facts drive it:
            // (1) the port's Player_Move zeroes the forward axis while aiming — `let (fwd, _) =
            // if input.aiming { (0.0, input.turn) } else { … }` [M player.rs Player_Move; the
            // game's own aim branch 0x191dd0] — so the aimed dodge can only strafe (target 0.075
            // [M]); (2) the aimed strafe CRAWLS at the rim: the 2026-11-20 lever run's episode
            // walked 0.17 u in 40 ticks (the capsule wedges on the trench/terminal lip) and the
            // baseline run's episode burst 0.58 u then crawled 0.01/tick [L both runs]. A player
            // sidestepping off a rocket's line RELEASES the aim: the forward axis lives
            // (target 0.105 full-press, 0.03 at the half-press [M Player_Move's square curve])
            // and the dodge walks the strafe + half-forward diagonal — the bearing az ~3.6 (NNW)
            // reads clear 30 u [L the rim sweep az2.36-4.32: 1.00] and sits ~89-125° off the
            // killer's WSW travel (the perpendicular component ~0.072-0.09/tick after the axis
            // ease ≈ 2.5-3 u inside the killer's ~50-tick flight — the miss grows past the
            // contact-fuse band and the blast lands on the west wall band ≤ −1.73-class instead
            // of the contact −15.00 [L the diagnostic run's detonation rows]). The 6-u eye ray
            // picks the open side, keeping the anchor-ward pick when both read clear; both
            // blocked → the measured pure-strafe fallback. D3-GATED: the d2 sister's dodge
            // byte-exact.
            if difficulty >= 3 {
                let fwd = [fy, 0.0, cy];
                // The side LATCHES at the salvo's first episode (dodge_side unset) and holds while
                // the salvo lives; the diagonal bearing (strafe + half-forward) is re-derived only
                // for the pick. The 6-u eye ray picks the open side, keeping the anchor-ward pick
                // when both read clear; both blocked → the measured pure-strafe fallback.
                if dodge_side == 0.0 {
                    let diag = |s: f32| [right[0] * s + fwd[0] * 0.5, right[2] * s + fwd[2] * 0.5];
                    let open = |dx: f32, dz: f32| sim.world
                        .segment_flight(eye, [eye[0] + dx * 6.0, eye[1], eye[2] + dz * 6.0], true).is_none();
                    let (dpx, dpz) = (diag(1.0)[0], diag(1.0)[1]);
                    let (dmx, dmz) = (diag(-1.0)[0], diag(-1.0)[1]);
                    let (op, om) = (open(dpx, dpz), open(dmx, dmz));
                    dodge_side = if op && (!om || input.strafe > 0.0) { 1.0 }
                        else if om { -1.0 }
                        else { input.strafe };
                }
                input.aiming = false;
                input.strafe = dodge_side;
                input.forward = 0.5;
            }
            } else {
                input.strafe = dodge_sign;
                input.forward = 0.0;
            }
        }
        // The latch releases when the salvo dies (the next salvo re-picks from its own spot).
        if difficulty >= 3 && dodge_side != 0.0 && !salvo_live && dodge_hold == 0 { dodge_side = 0.0; }
        // retune-nb-08-7: the SALVO DIAGNOSTIC — every live machine bullet each 10 ticks during
        // the rim fight (d3-gated, eprintln only, no sim state touched: the walk's timeline stays
        // byte-exact). Fields: the dodge gate's own d / miss for Bond's chest, the bullet's speed
        // and heading, and the dodge_hold + bond position at the same tick — this is what settles
        // why the −15.00/−3.27 kind-98 blast landed at d 0.3 (t21773) while the rim's dodge_hold
        // read 0 at both 100-tick samples around it.
        if difficulty >= 3 && t % 10 == 0 {
            let bpos = sim.bond.pos;
            let chest = [bpos[0], bpos[1] + 0.6, bpos[2]];
            for bu in sim.drones.bullets.iter().filter(|b| b.state != 2) {
                let rel = [chest[0] - bu.pos[0], chest[1] - bu.pos[1], chest[2] - bu.pos[2]];
                let d = (rel[0] * rel[0] + rel[1] * rel[1] + rel[2] * rel[2]).sqrt();
                let vm = (bu.dir[0] * bu.dir[0] + bu.dir[1] * bu.dir[1] + bu.dir[2] * bu.dir[2]).sqrt().max(1e-6);
                let along = (rel[0] * bu.dir[0] + rel[1] * bu.dir[1] + rel[2] * bu.dir[2]) / (d * vm);
                let miss = if along > 0.0 { d * (1.0 - along * along).max(0.0).sqrt() } else { -1.0 };
                eprintln!("[08 salvo] t{t} f{} w{:x} pos ({:.2},{:.2},{:.2}) dir ({:.3},{:.3},{:.3}) st {} d {d:.2} miss {miss:.2} spd {vm:.2} dh {dodge_hold} bond ({:.2},{:.2},{:.2})",
                    sim.drones.frame, bu.weapon, bu.pos[0], bu.pos[1], bu.pos[2], bu.dir[0], bu.dir[1], bu.dir[2], bu.state, bpos[0], bpos[1], bpos[2]);
            }
        }
        let _ = exit_ticks;
        // real-health-00agent-08-bleed-4 (d3): the ZAP walk-out + the drift walk. The dodge bias
        // above keeps the episodes anchor-ward; when a zap lands anyway (or plain drift over
        // 1.2 u between episodes) Bond walks back to the ride-end anchor — a player being zapped
        // steps out of the volume, and the aim re-converges on the spot (the launch gates read
        // the live spot every tick, so nothing fires mid-walk). The walk never runs while a
        // rocket rides. nb-scan-all-08: the gate opens to difficulty >= 2 — the Agent fight's
        // drift (above) is the death mechanism the walk-back answers; the d1 real-damage sister
        // (difficulty < 2) never enters this branch, its timeline byte-exact.
        // retune-nb-08-7 (d3): the walk-back waits for the salvo computed above (`salvo_live`).
        if difficulty >= 2 && rocket.is_none() && (zap_hold > 0 || dodge_hold == 0) && !salvo_live {
            let (dx, dz) = (ANCHOR[0] - b[0], ANCHOR[1] - b[2]);
            // retune-nb-08-7 (d3): the walk-back walks ALL THE WAY to the anchor (0.35 u — the
            // walker's own arrival class). The shared 1.2-u drift threshold measured as a
            // STRANDING: the 2026-11-20 lever-5 run's walk-back stopped 0.76 u short
            // ((111.74, −179.74), dist 0.85 < 1.2) INSIDE the volume's band and the cannon's
            // doorway lane, and the −2.00 world rows (f23145-23247) closed the walk at fight
            // t2110 — while the anchor itself stayed nearly zap-free through the baseline's
            // ~800-tick fight at 0.2 u off it (ONE −1.73 row) [L both runs]. The d2 threshold
            // stays byte-exact.
            let walk_back_r = if difficulty >= 3 { 0.35 } else { 1.2 };
            if dx * dx + dz * dz > walk_back_r * walk_back_r {
                let want_yaw = dx.atan2(dz);
                let yaw_err_a = walker::wrap_pi(want_yaw - sim.bond.yaw);
                input.turn = yaw_err_a.clamp(-0.08, 0.08);
                input.look = 0.0;
                input.forward = if yaw_err_a.abs() < 0.3 { 1.0 } else { 0.0 };
                input.strafe = 0.0;
                // retune-nb-08-7 (d3): the walk-back releases the aim — the same Player_Move law
                // that zeroes the dodge's forward while aiming zeroes the walk-back's [M player.rs
                // Player_Move's aim branch]. Measured: the aimed walk-back NEVER moved Bond — the
                // 2026-11-20 lever-4 run parked him in the NW band corner from fight t390 to the
                // zap death t676 with the aimed walk-back's forward-1.0 a no-op every tick [L the
                // run]. A player walking back to his firing spot lowers the weapon; the aim
                // re-converges on arrival (the gates read the live spot every tick). D3-GATED:
                // the d2 sister's walk-back byte-exact.
                if difficulty >= 3 {
                    input.aiming = false;
                }
            }
        }
        if rocket.is_some() != riding {
            riding = rocket.is_some();
            let now = sim.drones.copters[ci].damage;
            if riding { rockets += 1; launch_damage = now; } else { best_rocket = best_rocket.max(now - launch_damage); }
            eprintln!("[08 rocket] t{t} {} — damage {:.1} hp {hp} dist {l:.1} from {from:?} copter {cp:?} clip {} bullets {}",
                if riding { "launched" } else { "gone" }, sim.drones.copters[ci].damage,
                sim.drones.inventory.clip.get(ROCKET as usize).copied().unwrap_or(0),
                sim.drones.player_bullets.iter().filter(|b| !b.dead).count());
        }
        // Fire when the corridor to the machine is open and it is beyond the blast's reach —
        // the aim tolerance is the guided round's, not a settled sniper's (see above): the
        // machine's low passes through the slit are a few ticks wide and the tracker's
        // ±0.08/±0.05 per-tick clamps cannot settle inside a sniper gate there.
        // 00 Agent: the player fires at what he can SEE. The eye's flight ray to the machine's
        // live position must reach (essentially) the machine — the stop, if any, is the cable
        // sag the guided round threads (the measured launches' stops t0.85-0.95 [L the diag
        // run 2026-11-04]) — NOT the cabin's own wall band: the t347 launch's eye ray stopped
        // at t0.048 (the machine hidden behind the band between the windows), the chase then
        // steered the round onto the window frame and the blast took −75 at 2.4 u [L the diag
        // run — the world hit at frame 28066]. And the CHASE's lane — the machine-bearing
        // corridor from the eye and both gun stand-ins — must reach the machine the same way
        // (the t747 launch's round clipped the south-east corner post 2.2 u from Bond: −62.5
        // [L the diag run — frame 28470]). D3-GATED: the sisters' timelines stay byte-exact.
        // join-fix-08h: Agent (difficulty 2) takes the same two sight gates. With the Hind at its
        // full route speed (agent-copter-rate) the Agent run's rocket 3 left at t809 with the
        // machine 22.3 u out behind the cabin's west wall band (the eye ray stopped at t0.035,
        // 0.8 u out); the round died on that band and its blast (1.5 u, kind 44, owner -1 [L the
        // NF_DMG_TRACE run]) took the ledge armour's 50 and the last 7.6 health. A player holds
        // the rocket until he can see the machine through a shot-out pane. D2/D3-GATED: the
        // Operative runs' timelines stay byte-exact.
        // los-mask-join: the Operative real-damage walk takes them too — on the masked tree its
        // rocket 3 died 2 ticks after launch and the last one's blast finished Bond
        // [L the run]. The topped-up Operative walk keeps its timeline.
        let sight_gates = difficulty >= 2 || !w.tops_up();
        let eye_sees = !sight_gates || sim.world
            .segment_flight_scoped(sim.world.find_cel(eye), eye, cp, true)
            .is_none_or(|t| t >= 0.8);
        let machine_lane_clear = !sight_gates || {
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            [eye,
             [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
             [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]]
                .iter()
                .all(|from| { let from = *from; sim.world
                    .segment_flight_scoped(sim.world.find_cel(from), from, cp, true)
                    .is_none_or(|t| t >= 0.8) })
        };
        // level-08-rocket-self-blast (d3): the FORECAST gate — the aim-route-6 §3 next lever.
        // The chase steers the round toward the machine's LIVE position every tick [M
        // Bullet_update 0x12e7b0:177-244], so the round's lane is the machine's FLIGHT-TIME
        // lane: the 27975 launch cleared every gate at the tick's own positions and the
        // machine's flight-time position crossed the corridor the first 12 u left clear — the
        // round's curve followed it into the wall band 0.8 u from Bond. Extrapolate the
        // machine's position over the round's cruise flight time (T = l / 0.2525 [L the
        // measured 0x2c cruise 0.2525 u/tick]) along its measured per-tick velocity and
        // require the same three-source lane cast against BOTH positions. The game gives the
        // round no lock-on (Bullet_init 0x12c5a0 spawns it along the crosshair aim point; the
        // homing steer is the 0x20000 bit's [M 0x12f358] and 0x2C's +0x6C = 0x12204 [L
        // ee_start_05.bin + 0x2C·0x10C] lacks it) and no arming grace (the cam probe excludes
        // the shooter only for the first 1.0 u [M 0x12e7b0:218-234]; the BU+0x34 absorb is a
        // SILENT death [M 0x12ce80:127,179]; Explode_Propagate 0x17c518 case 3 has no shooter
        // exemption), so the player-model simply holds the rocket while the machine's
        // flight-time lane runs the wall band. D3-GATED: the sisters' timelines stay byte-exact.
        let machine_forecast = if difficulty >= 3 && have_cp {
            let v = [cp[0] - prev_cp[0], cp[1] - prev_cp[1], cp[2] - prev_cp[2]];
            let vm = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if vm > 1e-4 && l > 1.0 {
                let adv = (vm * (l / 0.2525)).min(l * 0.9);
                [cp[0] + v[0] / vm * adv, cp[1] + v[1] / vm * adv, cp[2] + v[2] / vm * adv]
            } else { cp }
        } else { cp };
        let machine_lane_fore_clear = difficulty < 3 || {
            let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
            let right = [cy, 0.0, -fy];
            [eye,
             [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
             [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]]
                .iter()
                .all(|from| { let from = *from; sim.world
                    .segment_flight_scoped(sim.world.find_cel(from), from, machine_forecast, true)
                    .is_none_or(|t| t >= 0.8) })
        };
        // real-health-00agent-08-bleed-5 (d3): the DISTANCE gate relaxes to the sisters' 10 u — the
        // ledger's lever (2) "one gate's relaxation". The measured d3 roll launches once per ~6500
        // ticks at l >= 16 (death t34720, one rocket landed 100.7 [L update-3]; the machine's
        // circuit dist 17.6-28.9 with its lintel-clear low passes rarely beyond 16 u), and the
        // pool cannot grow to buy more rounds: ammo_data type 0xe caps carried at 4 [D
        // ammo_data +0x02; the probe run run-bleed5-1] and the roll took the mouth box the moment
        // its carry had room (tick 24599, Some(Ammo 4) [L run-bleed5-1]) — the ride's remaining
        // boxes refuse at the cap every tick after ([take] → None from tick 24600 [L
        // run-bleed5-1]). So the fight must be won on the 8 rockets the cap allows, and the
        // launch line must open on the passes it actually has. 10 u is the game's own blast rule
        // (radius 8 ≈100 [L ee_start_05.bin @0x2bf150 + 0x2C·0x10C], the falloff past k=8 —
        // the sisters fire on l >= 10 [M the same line at difficulty < 3, byte-exact above]) and
        // the round is GUIDED past the muzzle — it threads the openings the view sees [M
        // Bullet_update 0x12e7b0:177-244; the d1/d2 gates fire from inside the cabin with no
        // corridor gates]. Every d3 self-blast gate STAYS: the 3 u slit, both 12 u corridors
        // (aim + effective aim), the eye-sight, the machine lane and the flight-time forecast.
        // D3-ONLY: the sisters' `l >= 10.0` is unchanged (the old expression returned 10.0 below
        // difficulty 3), their timelines byte-exact.
        // 00 Agent: the sisters' gate shape (the corridor/effective-corridor/forecast gates off;
        // aim, range, the 3-u slit, sight and the machine lane kept - without the slit a launch
        // blew up on the car's own ceiling 4.7 u above Bond, -25.06 [L the blast trace, owner -1
        // kind 0x2c]; with it the same three rockets land and the death moves t30165 -> t32039). With the cannon-free dodge: three
        // rockets land, the Hind 100.7 -> 287.7 of 598 (the fight ends sooner - death t34720 ->
        // t30165 to the Hind missile blasts). Loose gates alone (with the cannon dodge): t28440.
        let loose08 = difficulty >= 3;
        let ready = rocket.is_none() && sim.missile_cam.is_none() && dodge_hold == 0
            && yaw_err.abs() < if difficulty >= 3 { 0.10 } else { 0.25 }
            && pitch_err.abs() < if difficulty >= 3 { 0.10 } else { 0.25 }
            && l >= 10.0 && slit_clear && (loose08 || (corridor_clear
            && effective_corridor_clear)) && eye_sees && machine_lane_clear
            && (loose08 || machine_lane_fore_clear)
            // 00 Agent: no launch while the Hind is SOUTH-WEST of the car (x < 105, z < -165).
            // A held-health map of the whole fight [L the analysis run: ten launches] showed
            // every launch at the machine there - (83.4, -190.0), (91.2, -210.2), (85.2,
            // -188.9) - gone in 10-20 ticks on the car's west wall beside Bond (the -60 / -8.95
            // self-blasts, owner -1 kind 0x2c); launches at x >= 111 and the opening
            // (87.7, -160.5) all landed. Measured: five rockets land, the Hind 287.7 -> 489.6 of
            // 598, death t32039 -> t33393.
            // real-health-00agent-08 (11-15): the south-west ban above is replaced on 00 Agent by
            // the lowered muzzle cast - the self-blasts it guarded against were rounds whose first
            // 4 u ran into the cabin frame BELOW the eye ray (the t5537 blast at (113.6, 13.7,
            // -181.1), 1.2 u under the eye [L the NF_DMG_TRACE runs]): the round leaves from the
            // gun, so the 4-u flight cast runs from the eye line and both gun stand-ins 0.22 u
            // lower. With it the SW launches land (rocket 3 at (84.7, 22.3, -196.3)) and no launch
            // self-blasts. D3-GATED.
            && (difficulty < 3 || { let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
                [[eye[0] + cy * 0.25, eye[1] - 0.22, eye[2] - fy * 0.25], [eye[0], eye[1] - 0.22, eye[2]], [eye[0] - cy * 0.25, eye[1] - 0.22, eye[2] + fy * 0.25]]
                    .iter().all(|f| sim.world.segment_flight(*f, [f[0] + aim_dir[0] * 4.0, f[1] + aim_dir[1] * 4.0, f[2] + aim_dir[2] * 4.0], true).is_none()) })
        // retune-nb-08-4: the LOW-PASS OUTLAST gate (the ledger's lever (3)) — the flight logs
        // pin the machine's route bands: LOW passes y 13-24 (the d2 phase's five standing-slit
        // launches in 2100 ticks [L the 2026-11-18 run]; the lever-1 fight window flew y
        // 16.5-20.5 at every 30-tick sample [L the 2026-11-19 NF_DMG_TRACE run]) and HIGH
        // overhead passes y 25-29 (the nb-scan-all phase, from inside the cabin EVERY stance's
        // sight ray stops on the lintel/band ~1 u out — line_ok FALSE at every sample [L the
        // 2026-11-18 ray dumps]). The launch windows live in the LOW band only: on d3 the fight
        // holds through a HIGH phase and fires into the low passes. No new behaviour when the
        // machine is already low (the eye_sees/machine-lane gates above dominate — a HIGH pass
        // never passes them); the gate makes the outlast explicit and keeps a shifted route
        // phase from burning Bond's scarce rounds on a window the corridor cannot open.
        // retune-nb-08-6 TRIED and REVERTED: the band tightened to the proven landing band
        // 14.5 held fire through the whole rim-fight window (ZERO launches — the machine
        // never dropped below y 14.5 within a launchable bearing in ~800 fight ticks) and the
        // death was the SAME −15.00/−3.27 world blast at the SAME frame (21775 vs the 25.0
        // gate's 21774) [L the two lever runs] — the closer is the machine's MISSILE salvo,
        // not Bond's own round (the 25.0 gate's t585 launch at the y 20.8 pass missed with
        // damage 0.0 and the blast arrived on the salvo's schedule regardless). The gate
        // returned to the round-4 measured 25.0; the missile answer is the next window's lever.
        // retune-nb-08-7 RE-TRIED WITH THE MISSILE ANSWER: the 2026-11-20 diagnostic run split
        // the round-6 death in two — (1) the dodge gate never saw the killer: Bond was RIDING
        // his own t585 rocket when the salvo spawned (the dodge is gated on rocket.is_none())
        // and the port's missile cam zeroes the whole input while the cam is up — the substate-10
        // player runs no movement arms [M scene.rs ride block; Player_Update 0x193600 case 10];
        // (2) the one episode that DID fire (t350) crawled 0.7 u — the rim's strafe corridors
        // are blocked (above). The 25.0 gate's mid-pass launches are 0.0-miss rides that freeze
        // the dodge exactly over the salvo's schedule — the machine is FASTER than the round
        // (cruise 0.2525 [L] vs the machine's ~0.5/tick passes [L the flight samples]) so a
        // mid-band launch cannot land, only burn a clip round and freeze Bond. With the diagonal
        // dodge + the walk-back pin live (above), the band returns to the proven landing 14.5:
        // no rides in the mid-band window, the salvos dodged on the ground, and the launches
        // wait for the low band the landing geometry was proven in (the round-4 runs: y 12.8-13.7
        // passes, 101.0 landings [L]) — the circuit recurs it on the ~700-tick loop [L the
        // ledger's flight logs]. D3-GATED: the sisters keep l >= 10.0 with no cp[1] gate.
        && (difficulty < 3 || cp[1] <= 13.7);
        // 11-15 (00 Agent): the last rocket. Six rockets leave the machine at 590.9 of 598 and
        // its next pass kills Bond on 7.8 health before any gated window opens; a player with the
        // machine under the crosshair (0.2) fires the finisher. D3-GATED.
        let ready = ready || (difficulty >= 3 && rockets >= 6 && rocket.is_none() && sim.missile_cam.is_none()
            && sim.gun.clip > 0 && l >= 10.0 && yaw_err.abs() < 0.2 && pitch_err.abs() < 0.2);
        if ready { w.tick_fire(input); } else { w.tick(input, false); }
        if t % 100 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            // retune-nb-08-4: the aim-gate diagnostic — the launch window's convergence state
            // (yaw_err/pitch_err are the `ready` gate's own values, the dodge/corridor/sight
            // gates' booleans name which gate held the launch back).
            eprintln!("[08 hind t{t} f{}] dist {l:.1} line_ok {line_ok} aim_yaw_err {yaw_err:.3} aim_pitch_err {pitch_err:.3} dodge_hold {dodge_hold} exit {exit_ticks} cross_still {cross_still} damage {:.1} hp {hp} weapon {:#x} clip {} bond {:?} copter {:?} st {} bullets {:?}",
                w.total_ticks,
                sim.drones.copters[ci].damage, sim.gun.weapon, sim.gun.clip, sim.bond.pos, cp, sim.drones.copters[ci].state,
                sim.drones.player_bullets.iter().filter(|b| !b.dead).map(|b| [(b.pos[0]*10.0).round()/10.0, (b.pos[1]*10.0).round()/10.0, (b.pos[2]*10.0).round()/10.0]).collect::<Vec<_>>());
            if !line_ok {
                // Map the box: 16 azimuth rays + straight up — where does the eye see 30 u?
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                if exit_ticks == 1 {
                    // One-shot dump: the triangles around Bond name the box and its opening.
                    let lo = [b[0] - 5.0, b[1] - 3.0, b[2] - 6.5];
                    let hi = [b[0] + 5.0, b[1] + 4.0, b[2] + 6.5];
                    for (name, tris) in [("dyn", sim.world.dynamic_tris_near(lo, hi)), ("misl", sim.world.missile_tris_near(lo, hi))] {
                        eprintln!("[08 cabin] {name}: {} tris near bond", tris.len());
                        for (k, t) in tris.iter().enumerate().take(120) {
                            let c = [(t[0][0] + t[1][0] + t[2][0]) / 3.0, (t[0][1] + t[1][1] + t[2][1]) / 3.0, (t[0][2] + t[1][2] + t[2][2]) / 3.0];
                            eprintln!("[08 cabin] {name} t{k} c ({:.2},{:.2},{:.2})", c[0], c[1], c[2]);
                        }
                    }
                }
                let mut sweep = String::new();
                for k in 0..16 {
                    let az = k as f32 * std::f32::consts::TAU / 16.0;
                    let far = [eye[0] + az.sin() * 30.0, eye[1], eye[2] + az.cos() * 30.0];
                    let t = sim.world.segment_bullet(eye, far).unwrap_or(1.0);
                    sweep.push_str(&format!("az{az:.2}:{t:.2} "));
                }
                let tup = sim.world.segment_bullet(eye, [eye[0], eye[1] + 30.0, eye[2]]).unwrap_or(1.0);
                eprintln!("[08 hind t{t}] eye ({:.1},{:.1},{:.1}) UP t{tup:.2} sweep {sweep}", eye[0], eye[1], eye[2]);
                // Which ray stops, and where — the stop point names the blocker.
                let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
                let right = [cy, 0.0, -fy];
                for (name, from) in [("eye", eye), ("gunR", [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25]), ("gunL", [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25])] {
                    let tscope = sim.world.segment_bullet_scoped(sim.world.find_cel(from), from, cp);
                    let tflat = sim.world.segment_bullet(from, cp);
                    let desc = |t: Option<f32>| match t {
                        None => "CLEAR".to_string(),
                        Some(t) => {
                            let s = [from[0] + (cp[0] - from[0]) * t, from[1] + (cp[1] - from[1]) * t, from[2] + (cp[2] - from[2]) * t];
                            format!("stop t{t:.3} at ({:.1},{:.1},{:.1})", s[0], s[1], s[2])
                        }
                    };
                    eprintln!("[08 hind t{t}]   {name}: scoped {} flat {}", desc(tscope), desc(tflat));
                }
            }
        }
        // The forecast gate's velocity source: the machine's position this tick.
        prev_cp = cp;
        have_cp = true;
    }
    let damage = w.scene.sim.as_ref().unwrap().drones.copters[ci].damage;
    eprintln!("[08] the shoot-down: {rockets} rockets; damage {damage} vs hp {hp}; best single rocket {best_rocket}");
    // hind-flight-pin: the measured flight. The machine sweeps the valley the whole window —
    // the log's samples run (70.95, 22.16, −165.04) f25860 → (171.16, 16.32, −192.44) f26640 at
    // the path's 0.2533/t step (the old half-rate build; 0.5062 since agent-copter-rate [M 0x17d340, L ee_start_05.bin]), ≈120-170 u of travel per run, with NOT ONE 30-tick stretch below
    // 2 u — the task's pre-fix "holds one spot" is gone (Copter_Move's spline pose,
    // copter-path-flight). The case order is the game's [M Copter_Update 0x17dbb8]: 0 → 1 (fly)
    // with case 2 whenever the spot check lands (the 00 Agent sister's run samples a 2 — the
    // 45° cone off the path keys' facings points at Bond on her timeline) → the rockets cross hp
    // → state 4 (no kind-2 arrival path on 08 [D the route dump], rec+0x58 stays None) → 5, whose
    // ticks re-write ch 0x4B [M :255-275]. The death's 4/5 ticks fall between the 30-tick
    // samples, so the terminal state is pinned from the sim, not the samples.
    assert!(flight_samples.len() >= 10, "the flight was sampled every 30 ticks: {}", flight_samples.len());
    let mut travel = 0.0f32;
    for w2 in flight_samples.windows(2) {
        let d = ((w2[1].0[0] - w2[0].0[0]).powi(2) + (w2[1].0[1] - w2[0].0[1]).powi(2) + (w2[1].0[2] - w2[0].0[2]).powi(2)).sqrt();
        if w2[0].1 <= 2 && w2[1].1 <= 2 {
            // The route runs: the path time strictly advances between flying samples — a wedged
            // copter freezes it (the hind-flight-pin bug class). The WORLD distance per window
            // is not a law: the spline's speed near a path key dips arbitrarily low — the
            // measured route windows moved 1.88 (the d2 run, state 2→1 at t 707→714.7) and 0.98
            // (the topped-up run, state 1→1 at t 819.3→826.9) u per 30 ticks [L the
            // glass-pierce-2 run logs] — the old ≥2.0 and a 1.0 hold line both fired on them.
            // A held copter still advances t (the samples' t step is the same ~7.6 in states 1
            // and 2 [L the same logs]).
            assert!(w2[1].3 > w2[0].3, "the flight's path time froze: {:?} → {:?}", w2[0], w2[1]);
        }
        travel += d;
    }
    eprintln!("[08 flight-pin] samples {} travel {travel:.1} u; states {:?}", flight_samples.len(),
        flight_samples.iter().map(|s| s.1).collect::<Vec<_>>());
    assert!(travel >= 100.0, "the flight travels its circuit, measured {travel:.1} u");
    // The case order: the flying samples are case 1, with case 2 wherever the spot check landed
    // [M :188-243]; the death (4) is pinned from the sim at the ch-0x4B break [M :358-370].
    let flying_states: Vec<u16> = flight_samples.iter().map(|s| s.1).collect();
    assert!(flying_states.iter().all(|&st| st == 1 || st == 2), "the flight is cases 1/2 [M :138-243]: {flying_states:?}");
    assert!(flying_states.contains(&1), "the flight flew [M :138]");
    let end_state = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.copters[ci].state
    };
    assert_eq!(end_state, 4, "the death state at the ch-0x4B break [M :358-370 — 08 is not an explode level]");
    assert!(ch4b, "the rockets pushed damage {damage} past hp {hp} → state 4 → rec+0x72 [M :358-370]");
    // The supply: a full clip (4 [L record 0x2C +0x92]) and the type-0xE carry cap (4 [L ammo_data
    // 0x2C69B8 + 0xE·12, +0x02]) — 8 on hand, the ride's three rocket pickups [D 0x0600004C streams
    // 37-39: two type-1 rocket ammo + one launcher, 4 rounds each, parented to the carriage] topping
    // the carry up. Aimed rockets miss now and then (the moving machine, the station's geometry).
    assert!(rockets <= 8, "the rockets on hand (clip 4 + carry cap 4) are all the walk had: {rockets} launched");
    // A rocket that lands on the body blasts there [M 0x12ce80's 0x4 arm → Explode_Propagate case 0]:
    // ≈100 from the blast + 1.0 contact. Sister: without the blast a body hit adds only 1.0.
    assert!(best_rocket >= 100.0, "a body hit carries the blast's ≈100, not just the 1.0 contact: best {best_rocket}");
    assert!(damage > hp, "the walk's accumulation, not a poke, crossed hp");
    for _ in 0..120 {
        if obj08(&w).1 == 4 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(obj08(&w).1, 4, "objective 'Escape with Zoe in gondola' completes on ch 0x4b [L record 3]");
    // ch 0x4B → mux #276 → ch 100 → the mission ends the level. The DRIVING hand-off is logged
    // (state 6 + LevelToEndTo 0x0730008), not a level_exit — the port skips driving codes
    // [M 0x182c60:85-116; mission.rs to_end match; docs/systems/mission.md §4].
    let mut end = (0u32, 0u32);
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        end = (sim.drones.mission.status(), sim.drones.mission.to_end);
        if end.0 == 6 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(end, (6, 0x0730_0008), "the mission completes into the driving hand-off 0x0730008 [M 0x182c60:85-116]");
    eprintln!("[08] the whole mission walked on real input in {} ticks; falls {:?}", w.total_ticks, w.report.falls);
}

/// The ally rule (Zoe once shot at Bond at the gondola): the dtype-0xC ally
/// (KikoMission [M 0x14c938:54-59]) never targets or fires at Bond while Bond walks the whole
/// route. Sister: the placed guards DO engage (their combat states appear once the zone plates
/// fire) — the ally is not one of them.
#[test]
fn the_ally_never_shoots_bond_on_the_walk() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
    }
    w.wait_enabled(3000, "the station start");
    // Walk the same first legs (the station, the walkway) — where the guards wake and fight.
    for (name, at) in [
        ("the intro plate", [139.2f32, 49.5, -81.2]),
        ("the station hall", [139.0, 49.5, -93.0]),
        ("the station doors", [139.5, 49.5, -97.5]),
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.wait(600);
    let sim = w.scene.sim.as_ref().unwrap();
    let ally = sim.drones.list.iter().enumerate()
        .find(|(_, d)| d.present && d.dtype == 0xc)
        .map(|(i, d)| (i, d.pos, d.state, d.team))
        .expect("the ally (dtype 0xC, Zoe) is placed");
    eprintln!("[08 ally] after the walk: {ally:?}");
    // The guards' ambush is up (the zone plates fired); the ally is NOT in a combat state against
    // Bond — KikoMission's target is never the player [M 0x14c938 dtype 0xC voice 0xC → 0x11].
    assert!(sim.drones.channels.on(0x2a) || sim.drones.channels.on(0x2b) || sim.drones.channels.on(0x2c),
        "the zone plates fired on the walk (the ambush is up)");
    let (ai, _, astate, _) = ally;
    assert!(!matches!(astate, 0x56..=0x7e | 0x85..=0x99),
        "the ally never enters a combat state on the walk [M KikoMission], state {astate:#x}");
    let _ = ai;
    assert!(!sim.drones.list.iter().any(|d| d.present && d.dtype == 0xc && d.hit_by_player),
        "the ally was never shot by Bond");
}

/// The heard-noise path (Zoe must never fire at Bond at the gondola): the dtype-0xC
/// ally rides the SAME walked route down to the gondola mouth and through the ride while the guards
/// fight Bond. The game's rule set (docs/systems/ally-heard-noise.md): FindOpponent never gives a
/// team-2 drone the player [M 0x1424e0:101-121], the heard-noise chain DOES walk her into Attack
/// 0x56 (Suspect's sound case 0x14 and Alert's timer case 0xc are unconditional SetState(0x56)
/// [M 0x16b0e0:118-124, 0x16b540:78-82]) but `DroneFunc_CombatState`'s opening gate sends an
/// opponentless drone to NoOpponent 0x60 [M 0x148338:20-31], CombatNoMove 0x59 has no 0x5b/0x5c
/// exit [M 0x1643e8:49-101] (the old port arm thrashed 0x59 ↔ 0x5c every tick), and every shot's
/// aim is the OPPONENT's position (`SetOpponentAimPos` per shot [M 0x179d40:130, 0x1422d0:70-73]) —
/// never Bond's chest. Sister: the team-1 guards on the same route still fight Bond.
#[test]
fn the_ally_fights_only_the_guards_through_the_noise_path() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000008.bin", false).expect("load 07000008"));
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for ch in [0x45u32, 0x70, 0x02, 0x48] { sim.drones.channels.set(ch, 1, sim.drones.frame); }
        // The briefcase channel stand-in (mux #235 ANDs it with the floor plate), as the ride walk does.
        sim.drones.channels.set(0x4a, 1, sim.drones.frame);
    }
    w.wait_enabled(3000, "the station start");
    w.wait(30);
    // The walked descent (the repro's route; the walkway plates wake the ambush).
    w.walk_nav_to([139.2, 49.5, -81.2], "to the intro plate", 1.3);
    w.wait_enabled(4500, "the station-intro scene");
    w.wait(60);
    for (name, at) in [
        ("the station hall", [139.0f32, 49.5, -93.0]),
        // No "station doors" stop (the whole-mission walk's note): its nav route runs east
        // through door (160.6, 45.0, −100.1) and back west along the lower deck, and the leg
        // aiming 3.4 above that deck wedged and dropped Bond into the station pit — out of the
        // map here too once the E reach changed the door timing (use-pick).
        ("the walkway head", [144.0, 46.0, -105.0]),
        ("the walkway mid", [145.5, 45.0, -108.6]),
        ("the lower landing", [150.0, 43.5, -116.0]),
        ("the landing east", [151.2, 42.5, -120.5]),
        ("the ramp south", [153.1, 41.2, -116.0]),
        ("the ramp mid", [153.2, 39.7, -111.7]),
        ("the ramp turn", [148.3, 38.5, -110.9]),
        ("the mid level", [143.3, 36.5, -111.5]),
        ("the mid doors", [135.5, 36.5, -112.2]),
        ("the mid west", [132.1, 36.2, -112.3]),
        ("the long link", [120.5, 34.1, -112.5]),
        ("the hall y33", [118.2, 33.9, -111.0]),
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
    // The briefcase channel stand-in again: the walked descent below runs ~16k ticks and some
    // writer clears ch 0x4a on the way (the ride walk's shorter leg set keeps it; the mux #235
    // needs it ON at the board). The real chain is walk_the_gondola_to_the_briefcase_and_back.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4a, 1, sim.drones.frame);
    }
    let (zi, zpos) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let hit = sim.drones.list.iter().enumerate()
            .find(|(_, d)| d.present && d.dtype == 0xc)
            .map(|(i, d)| (i, d.pos))
            .expect("the ally (dtype 0xC, Zoe) is placed");
        hit
    };
    eprintln!("[08 noise] zoe #{zi} at {zpos:?} after the walk (tick {}) ch4a {} ch1 {}", w.total_ticks,
        w.scene.sim.as_ref().unwrap().drones.channels.on(0x4a) as u8, w.scene.sim.as_ref().unwrap().drones.channels.on(1) as u8);
    // Board the carriage (the real floor plate), then ride while collecting.
    {
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
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, use_key);
            if t % 300 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[08 noise board t{t}] bond {:?} ch4 {}", sim.bond.pos, sim.drones.channels.on(4) as u8);
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(4), "the floor plate fired ch 4 [D #236 mask 8]");
    }
    let mut ch83 = false;
    for t in 0..3600u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x53) { ch83 = true; break; }
        w.tick(Input::default(), false);
        if t % 600 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let z = &sim.drones.list[zi];
            eprintln!("[08 noise ride t{t}] zoe state {:#x} pos {:?} opp {:?} shots {} | bond {:?} ch1 {} ch4 {} ch4a {}", z.state, z.pos, z.opp_drone, z.shots.len(), sim.bond.pos, sim.drones.channels.on(1) as u8, sim.drones.channels.on(4) as u8, sim.drones.channels.on(0x4a) as u8);
        }
    }
    assert!(ch83, "the ride ran to its ch-83 writer (the window covers the repro's fight) [D stream 46]");

    // ── the assertions ──────────────────────────────────────────────────────────────────────────
    let sim = w.scene.sim.as_ref().unwrap();
    let zoe = &sim.drones.list[zi];
    // Sister (can fail): a team-1 guard on the same route entered the combat family — the ambush
    // is alive and guard combat is not what this task broke.
    let guard_fought = sim.drones.list.iter()
        .any(|d| d.present && d.team == 1 && d.log.iter().any(|(_, s)| matches!(s, 0x56..=0x72 | 0x7f | 0x8c)));
    assert!(guard_fought, "sister: a team-1 guard fought on the same route [M the 0x228&4 combat arms]");
    // The ally walked the noise chain (the repro's 0xa2 → 0xa5 → 0x56 flow): the heard-noise
    // states or the Attack/Combat/NoOpponent family appear in her state log.
    let chain = zoe.log.iter().any(|(_, s)| matches!(s, 0xa2..=0xa5 | 0x56 | 0x58 | 0x60));
    assert!(chain, "the ally's noise chain ran (0xa2..0xa5 or 0x56/0x58/0x60 in her log), log {:?}",
        &zoe.log[..zoe.log.len().min(24)]);
    // No per-tick 0x59 ↔ 0x5c thrash: no (x, y, x) subsequence with x/y ∈ {0x59, 0x5c} spanning
    // ≤ 4 ticks.
    for win in zoe.log.windows(3) {
        let (fa, sa) = (win[0].0, win[0].1);
        let (_, sb) = (win[1].0, win[1].1);
        let (fc, sc) = (win[2].0, win[2].1);
        let thrash = matches!(sa, 0x59 | 0x5c) && matches!(sb, 0x59 | 0x5c) && sa == sc && sb != sa
            && fc.saturating_sub(fa) <= 4;
        assert!(!thrash, "the 0x59 ↔ 0x5c per-tick thrash is gone [M 0x1643e8 has no 0x5b/0x5c exit], window {win:?}");
    }
    // Every shot the ally fired aimed at her drone opponent — the aim ray misses Bond's own hit
    // cylinder (the port's guard-bullet volume, r 0.25) with the spread margin; a shot that
    // never intersects his cylinder cannot damage him [M 0x179d40:130 re-aims every shot].
    let bond_feet = [sim.bond.pos[0], sim.bond.pos[1] - 0.4, sim.bond.pos[2]];
    let vol = nf_game::bullet::PlayerVolume { feet: bond_feet, height: 1.8, radius: 0.25 };
    let mut bond_shots = 0usize;
    for (f, m, aim) in &zoe.shot_aims {
        let slack = [aim[0] - bond_feet[0], aim[1] - bond_feet[1], aim[2] - bond_feet[2]];
        let miss = (slack[0] * slack[0] + slack[1] * slack[1] + slack[2] * slack[2]).sqrt() > 1.0
            && nf_game::bullet::segment_cylinder(*m, *aim, &vol).is_none();
        if !miss { bond_shots += 1; }
        let _ = f;
    }
    assert_eq!(bond_shots, 0,
        "the ally never targets or fires at Bond on the noise path [M 0x1424e0:101 + 0x148338:20-31], aims {:?}",
        &zoe.shot_aims[..zoe.shot_aims.len().min(8)]);
    eprintln!("[08 noise] zoe states {:?} shots {} aims {}", &zoe.log[..zoe.log.len().min(20)], zoe.shots.len(), zoe.shot_aims.len());
    let _ = zpos;
}

/// Real damage: the whole-mission walk with the health top-up off for every walker it makes and
/// the walker's return fire on — Bond's death panics with the death spot and the last hits
/// (common/damage_log.rs), so a pass pins that the walk completes on the port's own damage.
/// No health/armour/ammo writes anywhere: the carried latches are the mission's ENTRY state
/// (set once at the cold spawn, header §The carried latches — in play they ride the RamSave
/// [M Player_RamSave 0x195670]), the fusebox wires, the safe bolts and the briefcase take are
/// the level's real interaction paths, and the gondola ride is the game's own script.
///
/// The fights pick weapon and range the way a player would (level-08-real-health-rockets): the
/// boarding platform's burst-rifle pair is answered with the GUN before the car
/// (`fight_close_guards`, real damage only), and the Hind fight fires the AT-420 only beyond
/// the blast's 10 u reach on a CLEAR corridor (`launch_line_clear`) — the old death at tick
/// 26696 was Bond's own rockets exploding on the bottom-station geometry 0.65–5.5 u from him
/// (−52.5/−7.5/−22.5/−3.7, frames 26477/26580/26639/26696) plus the pair's interval-4 bursts
/// (−9.0); docs/reports/level-08-long-range-hits.md.
#[test]
// los-mask-repins-08 (2026-11-10): green again on the exact-sight tree. The ledgered death
// (tick 29332 at (111.1, 13.5, −179.7), the Hind's strafing rockets detonating on the
// carriage's own wall band next to the standing spot: −1.20 × 15 world hits then the −23.00
// blast, frames 29154-29332 [L the NF_DMG_TRACE run]) is answered with the missile dodge
// opened to the real-damage walk (the dodge block's los-mask-repins-08 note) — real strafe
// input only, no damage/health writes. The Operative walk ends at 1.2 health, 116 hits for
// 98.8, the Hind down (f31140); the whole mission walked in 31146 ticks.
fn walk_the_whole_mission_on_real_input_real_damage() {
    // los-mask-join: under the game's sight masks [M 0x176f18 mask 0xa27] the secure-room guard
    // #4 hit Bond 28 times (68.2) while he held at the closed door, and the plain sight rule
    // never answered him [L the masked run]. The Agent sisters' glass-fight rule (Bond's sight
    // takes his own rounds' glass pass) answers him.
    walker::with_real_damage_glass_fights(walk_the_whole_mission_on_real_input);
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-08-09-0a-0b): the
/// same walked station → fusebox → safe → briefcase → gondola → Hind route at the menu's Agent
/// pick — GameState+0x28 = 2 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e = "Agent"]. On this level group
/// (05-08) the pick scales Bond's pain by Plr_DMod_Normal 0.7 instead of Easy 0.6 and the
/// guards' hit chance by DroneFiring_Accuracy_Normal 1.1 instead of Easy 0.8 [M Player_HandlePain
/// 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 05-08 group ← ReadTuningVars
/// 0x1d3a80]; the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input
/// only — the load path is the only difference.
#[test]
// los-mask-repins-08-agent (2026-11-10): green again on the exact-sight tree. The ledgered
// Hind-fight bleed (tick 37022, 69 hits for 100.0: the −1.40 × 34 machine drips through the
// shot-out panes over the pre-fight 33.6 from #19 [L the NF_DMG_TRACE runs]) is answered with
// the WEST DOORWAY SLIT (the fight block's los-mask-repins-08-agent note): after the pane
// shoot-out the fight walks Bond into the cabin's west doorway (the band's z gap
// −178.25..−180.6, lintel y ≈ 14.31 [D the tri dump]) — real walk input, the pane block's own
// sidestep pattern — and the fight's aim and launch gates read the live spot, so the band
// shields the machine's strafing drips while the rockets still start through the slit (the
// measured fight: five rockets landed 98.8 → 500.5, the machine down at t1619 [L the run on
// this branch]). End health 14.9 armour 3.8, 61 hits for 85.1, the whole mission walked in
// 37505 ticks. No damage or health writes, no top-ups; every helper here is gated to
// difficulty ≥ 2 (the Operative sister's green timeline stays byte-exact — its run re-checked
// on this tree). The boarding ledger's #19 33.6 and the walkway 36.4 stand as measured; the
// levers tried and retired this window (the pre-leg lip stand, the d2 reload, the hall-mouth
// stand) are documented in their blocks and docs/reports/los-mask-repins-08-agent.md.
// nb-scan-all-08 (2026-11-18): green again on the crouched-slit lever. The ledgered death
// (tick 31483 at (112.72,13.49,-177.24): 4 rockets land (401.8 of 448.5), Bond dies at the 5th
// launch [L the run]) came from the alternating dodge's drift walking Bond 2.3 u east of the
// slit into the pane-hole gallery (59 machine drips, −1.20/−1.40, for 72.6) and from the fight
// starting ~6500 ticks earlier in the machine's circuit than the 11-10 green run (the whole
// upstream 37505 → ~28900 [L the runs]) — the nb-scan-all phase lines the machine up with the
// doorway lane. The fight now crouches at the slit anchor (110.98, −179.31), the dodge bias and
// the drift walk-back keep Bond there, and the launches run the standing-eye windows the
// movement uncrouches into (the launch log's from-y 14.22 [L the green run]): five rockets,
// 502.5 damage vs hp 448.5, end health 79.0 armour 24.1, 14 hits for 21.0, the whole mission
// walked in 31070 ticks. No damage or health writes, no top-ups; every lever below is gated to
// difficulty == 2 or difficulty >= 2 with the d<2 else branches unchanged (the d1 real-damage
// sister's timeline byte-exact by construction; the whole binary re-run green on this tree).
#[test]
fn walk_the_whole_mission_on_real_input_real_damage_agent() {
    // HEAD-HUNT (the 05/07 00 Agent sisters' tactic on this level group): at Agent the guards'
    // hit chance is DroneFiring_Accuracy_Normal 1.1 (Operative 0.8) and Bond's pain is
    // Plr_DMod_Normal 0.7 (0.6) [M tuning.rs 05-08 group] — every tick a fight stands is a
    // burst tick, so the fights take the clear head box first (one head hit drops a 12-health
    // guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head]) instead of grinding bodies —
    // the plain pick's nearest-body order let the walkway shotgun guard #20 land 10 × −4.2.
    // GLASS-SIGHT (guard-glass-finish): the mouth's fight is THROUGH the intact panes — the
    // guards' rounds already fly the game's glass pass [M Bullet_update 0x12f048-0x12f058;
    // L weapon_data every gun bit 0x800 clear] and Bond's own rounds fly the same pass, so
    // the sight rule must too (the plain rule held fire through eleven 61-tick stands on the
    // pair 3-10 u behind the mouth's panes while it walked point-blank and killed Bond).
    walker::with_real_damage_glass_fights(|| walk_the_whole_mission_on_real_input_at(2))
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-08-09-0a-0b): the same
/// walked station → fusebox → safe → briefcase → gondola → Hind route at the menu's 00 Agent
/// pick — GameState+0x28 = 3 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 2 carries 3 [D]; Txt 0x14f = "00 Agent"]. On this level
/// group (05-08) the pick scales Bond's pain by Plr_DMod_Hard 1.0 (Easy 0.6, Agent 0.7) and the
/// guards' hit chance by DroneFiring_Accuracy_Hard 1.3 (Easy 0.8, Agent 1.1); the guards' base
/// bullet damage rides DroneDamage_Hard 0.8 (Agent 1.0) [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80 cases 0x7000005-8;
/// tuning.rs 05-08 group]; the min-difficulty placements pass Drone_Create's gate at d2 AND d3
/// [M 0x136be0:49]. Real input only — the load path is the only difference (plus the d3-gated
/// tactics the deaths force, inside this file). The Hind fight's shoot-out fires on the
/// CONVERGED aim (the ray-gate fired nothing: the bounds-centre ray reads OPEN before a round
/// — 10/10 end panes die to one round each) and the launch gates read the CROSSHAIR the shot
/// itself flies through [M the crosshair aim law]: the tracker's convergence saturates the
/// free-aim cursor at its clamp and every launch sat up to 0.23 rad off the view — the chase
/// then clipped the window frames (the measured −75/−62.5 self-blasts [L the diag runs]).
/// IGNORED with the measured reason (carried from the aim-route-6 report §3 [L the run on
/// aim-on-master and again on this branch 2026-11-06: the same death, tick 27975,
/// (111.06, 13.53, −179.71), 6 hits for 100.0, armour 0.0]): the chase's OWN mid-flight curve
/// — the round's first ticks fly along the effective aim (the view + the eased cursor
/// offsets; on d3 the crosshair law saturates the tracker and launches sit up to 0.23 rad off
/// the view), and the machine's flight-time position crosses the corridor the first 12 u of
/// the flight left clear — the −62.5/−15.5 self-blast pair at 27975. The d3-gated
/// effective-aim corridor gate (the same three-source 12 u cast along the effective
/// direction) prevented the 28178-class detonation but this death remains. NEXT LEVER: the
/// forecast gate — extrapolate the machine's flight-time position (`cp' = cp + v̂·(l/0.5)`,
/// the closing speed) and require the machine_lane_clear checks against BOTH positions —
/// plus the pin of the round's actual spawn direction (lock-on vs the effective aim) in
/// PCSX2 (the report's live check).
#[test]
#[ignore = "nb-scan-all (the spacing scan everywhere + CanAltAttack in the fire states): the fight starts ~6500 ticks earlier in the machine's circuit (upstream 37505 -> ~25700 [L]) and the machine flies HIGH overhead passes (y 25-29) — from inside the cabin every stance's sight ray stops on the lintel/band 1.1 u out, line_ok FALSE every sample, ZERO-ONE launches in ~2600 fight ticks: Bond bleeds out of the gun chips (the d2 -1.40 drips at d3 pain 1.0 = -2.00 x5, frames 27474-27638) plus the -7.78/-8.22 missile blasts and dies t28298 at (112.25,13.09,-179.16) with ONE rocket away [L the 2026-11-18 runs; reproduced exactly on this branch 2026-11-19, the whole-binary run 6/0/1]. Levers TRIED and DEAD on their own real runs: the crouched slit anchor (t29087, zero launches, clip 4 intact), the standing slit + 0.25 aim gates (t27383), the 2026-11-19 lane-mouth pre-empt (the mouth target's own nav chain wedges at (141.0,50.4,-93.7) grounded=false — the deck-head route is the only clean approach [L the lever run]). SHIPPED 2026-11-19 (still red, the walk reaches 2.5x deeper): the deck-head arm-up (scoop the guards' own 0xa drops + arm fullest) + the wide fight's skip-continue — the walkway pair now dies at ZERO cost (was -12.0), the pit class is answered (the #15 excursion kill t5449; the 2026-11-19 pre-fix lever run died to #15 at the doorway t6408 with -23.0), and the walk now dies at the BOARDING LEDGE t17664 at (110.07,28.31,-121.26): #27 (skin 0x500002f, (100.8,28.3,-122.2), 9.3 u) burst 21.0 in 7 hits with hp 12 INTACT — the return fire never landed a round on him (35 hits for 100.0 total: #14 16.0/4 + #5 15.0/5 + #11 12.0/3 + #16 12.0/3 + #27 21.0/7). SHIPPED 2026-11-19 round 3 (still red, the walk reaches the machine fight): the hurting-first pick's unseen-cover fallback — the same fallback the plain and lane arms run, opt-in via the UNSEEN_FIGHTS thread-local (damage_log.rs return_fire; the d1/d2 sisters set no UNSEEN_FIGHTS and stay byte-exact) — the BOARDING LEDGE IS PASSED with ZERO #27 damage (he burst 21.0 in 7 hits in the round-2 run) and the walk dies in the CABIN fight t21777 at (111.94,13.09,-179.18), the route's deepest point, with 61 hits for 100.0: the upstream bleed dominates — #16 28.0/5, the y-46 trio 36.0 (#14 16.0/4 + #11 12.0/3 + #2 8.0/2), world 18.0/34 with the last -2.00 drips at frames 20809-21778 closing a zero-health walk [L the NF_DMG_TRACE lever run]. LEVER (2) TRIED and DEAD on its own real runs (2026-11-19): the mid-fight scoop — the stand hook that resupplies on a small clip mid-exchange. The gate is the CLIP (the fights run clip 3-7 spare 27-41 — the spares never run out; the 61-tick burn between taps is the exposure; the fights took ~230 ticks per 10-hp guard), and the armed fights kill the same guards in single-digit ticks at ZERO cost (the walkway pair: two fights, 18 ticks). Measured: the clip+spare < 10 gate NEVER FIRED (the fights are not low; death t21777 unchanged); the clip < 8 gate with a 14 u widened corridor walked Bond to a PLACED box 9.9 u off the line ((156.1,45.9,-85.7), 190 ticks of open-deck walking) into the trio's own lane — death t1325; the clip < 8 gate at the master 9 u corridor walked to the (150.7,46.3,-96.1) drop mid-fight — death t1759. Every walk-under-fire variant costs more than the armed tail saves; the trio's own drops sit 9-11 u off the walked line, outside the master corridor, and the arm-up that would matter lands only at the deck head, after the transit is paid. SHIPPED 2026-11-19 round 4 (retune-nb-08-4, still red, the FIGHT IS NOW WINNABLE — the launches fire and land): lever (3) the Hind outlast, executed as the launch-geometry fix. The flight-log pin [L the 2026-11-19 NF_DMG_TRACE runs]: on the lever-1 phase the machine NEVER flies high in the fight window — y 12.8-20.5 at every 30-tick sample — the LOW band is the whole window, so the ledger's 'outlast the high phase' is answered by the phase itself; what starved the launches was the STANCE: (a) the d2 slit walk (difficulty >= 2) parked Bond on the cabin's step edge at (110.98, 13.09, -179.32) (wedged 700 fight ticks, x parked 110.982->110.985), IN the terminal's hurt volume (10 -2.00 zaps, frames 20731-21778), crouched, ZERO launches, death t21777; (b) the walk-back to the d3 anchor OVERWRITES the aim's turn while it runs (input.look = 0.0) — aim_yaw_err 2.404->3.035 for 500 fight ticks (cross_still 101->501, the cursor frozen) while the machine's low passes crossed launchable bearings (the t0 t0.895-class sight stops); (c) the CROUCHED eye (13.3) is blindfolded at the anchor: the terminal's exterior ledge (top y ~13.3-13.5, 1.5-3 u out) stops EVERY bearing at t0.04-0.15 [L the crouched 16-ray sweep] while the STANDING eye (14.2) reads the same bearings CLEAR 30 u [L the t0 sweep]; the only rays the crouch lets out RISE steeply (the machine's high close passes) and those launches climbed into the west band above the doorway slot, detonated 5.5 u from Bond (-20.00 at t333, run 2). The shipped d3 package (all d3-gated, the d1/d2 sisters byte-exact): the slit walk is now difficulty == 2 only; d3 walks to the fight anchor BEFORE the loop (walk_to, the walker's wedge mitigation); the fight STANDS (the crouch-open is difficulty == 2 only); the low-pass outlast gate cp[1] <= 25.0 joins ready. MEASURED: the aim tracks from tick 0 (yaw_err 2.2 -> 0.004 by t100), rocket 1 launches at t36 from the standing eye at the machine's LOW pass (y 12.8-13.7, dist 45-48) and LANDS 101.0 at t192-198 with ZERO self-damage, five runs in a row — the launch geometry is proven. REMAINING DEATH (runs 3-7, t20993-t21077): the walk_to cannot STEP Bond down — its arrival is the x/z radius, and every target (r 0.6 at (112.5,-179.4), r 0.45 at (112.4,-179.3), r 0.5 at (113.0,-179.2), r 0.3/r 0.1 at (112.0,-179.5)) 'arrives' on the UPPER floor (feet 13.52) short of the hole edge at x ~111.99 — the hurt volume's band zaps the upper floor (-2.00 every 6-24 ticks, feet y 13.3-13.5; the lower floor feet 13.086 is zap-free [L run 1's in-place drop y 13.52->13.0856 at (111.99,-179.52)]) — 8-10 zaps drain the arriving 18.0 health mid-fight (rockets 1 landed, 2 launched at the high close pass, death t20993-t21077). RUNS 8-11 (2026-11-19, the drop CHASED to its root): (8) the raw press toward the anchor slid Bond AROUND the hole ((112.25, 13.52, -179.63), feet 13.52, the zaps resumed, death t20879); (9) the press aimed AT the hole itself reached the EXACT run-1 spot ((111.998, 13.52, -179.514)) and the STANDING capsule BRIDGED it — run 1's drop happened under the CROUCH (the smaller capsule falls through); (10) the crouch-drop-uncrouch WORKS — walk_to the hole, one crouch press, the fall (bounded 60 ticks, break at y <= 13.2), feet land 13.086-13.106 ON THE LOWER FLOOR, the zaps STOP (3 zaps total during the upper-floor walk phases, ZERO in the fight), the best ledger of the window: death t21070, 55 hits (the fewest), rocket 1 launched t236... but the fight ran CROUCHED on the lower floor (the eye 13.3 — the second crouch press re-crouched him) and the only rays out are the steep rising ones — the t236 launch was the machine's HIGH close pass (y 20.4, dist 26.9) and the round self-blasted (-14.00 at t21070, the same west-band-above-the-doorway class); (11) forcing the stance standing after the drop (4 default ticks, a crouch toggle, an assert) FAILED — the port's crouch latch resists both semantics from the walk's side (the log: crouched true after the release ticks AND the press) — and the extra ~6 upper-floor ticks caught a missile blast (death t20760, WORSE) — reverted to the run-10 state. SHIPPED (the final tree): runs 1-7's d3 package + the crouch-drop (the hole walk_to (112.0, 13.08, -179.5) r(0.1) + the hole press + the crouch drop). REMAINING: ONE BUG — the fight must run STANDING on the lower floor (eye 13.77 — above the ledge 13.5, under the lintel 14.31 — the proven flat-ray launch windows; the run-3/5 standing launches landed 101.0 from the upper floor) instead of crouched (the blindfold + the rising self-blasts). The stance latch needs the main agent's live check: what input state stands Bond up from the port's crouch latch (the run-11 measurements: 4 default ticks no-op, a crouch:true press no-op — is the latch edge-triggered with the release already consumed by the drop loop's ticks, or animation-locked?). The d2 fight's ledger ('the movement inputs stand Bond back up') suggests the uncrouch rides the DODGE EPISODES' inputs — the next window can try letting the first dodge episode run before the launch gates, or reading the port's crouch latch (Player_Crouch 0x1908xx-class) for the true semantics. SHIPPED 2026-11-19 round 5 (retune-nb-08-5, still red — the walk KEEPS master's shipped run-10 state; every round-5 lever measured WORSE on its own real run and was reverted): (a) THE LOCKOUT ANSWER [M]: run 11's 'port's crouch latch' is the stand/crouch LOCKOUT — Player_ChangeSubState 0x190630:48 writes BL+0x960 = FRAME_RATE (60) on EVERY substate change; SSCrouch 0x1924c0:45-53 gates the stand press on +0x960 == 0 and :84-105 reads the action; the port (player.rs crouch-on sets lockout = 60, the stand branch requires lockout == 0 + the headroom LOS pos→pos+0.95). Run 11's 4-tick release never outlived the 60. The round-5 stand press (wait the lockout out on the zap-free lower floor → hold crouch) WORKS: the log reads crouched false lockout 59. (b) TRIED and DEAD on its own real runs: the stand-up after the crouch drop — the stand FIRES but Bond stands INSIDE the hole's footprint and the expanding standing capsule RE-BRIDGES the shaft (feet 13.47, eye 14.18; master's fall lands 13.106 — the prefix verified byte-identical on a master control run: after the drop press (111.997635, 13.522537, −179.51396) both trees, then master 'after the crouch drop' (112.018974, 13.106038, −179.51132)); the fight's first launch (t67, the machine's pass y 17.9, dist 43.8) self-blasts −14.00, death t20944 (was t21070). The plate variant (crouch-walk toward (112.2,−179.2), then stand) measured the same pop (feet 13.47 at (112.17,−179.46), death t20942). (c) THE PICKUP SCAN [L the run's fight-entry dump]: armour 0.0 health 14.0 rocket clip 4 carried 0 — four rockets = 404 damage < the machine's 598 hp (DiffMod·299 [M Copter_Create 0x17d340; L the dmg 0.0/598 dump]) — the fight is UNWINNABLE on the carried ammo; and the cable car's three rocket pickups are UN-GONE 1.2-2.4 u away (kind 1 weapon 0x2c rounds 4 (111.62,13.24,−178.47); kind 0 weapon 0x2c rounds 4 (111.84,13.37,−181.45); kind 1 weapon 0x2c rounds 4 (110.67,13.24,−177.72) [D 0x0600004C streams 37-39: a launcher and two rocket-ammo boxes parented to the carriage]) — with them 12 × 101 = 1212 wins. The take is the game's own sphere check [M Pickup::touches: root distance < bounds.radius + 1.1 then the root→centre LOS ray, 0x18e148:210-214; Pickup_Handler 0x18e650]. (d) TRIED and DEAD on its own real runs: the collection walks — BOTH routes (post-drop crouched legs; pre-anchor standing legs) climb the parked carriage's EXTERIOR (feet 13.71-14.14 [L the walk logs: 'ammo box 1 arrived +160 ticks pos (111.75,14.14,−178.80)']) into the terminal's hurt volume and the −2.00 zaps close the 14.0-hp walk earlier (t21286, t21016 — 6 extra world hits). The car's interior is NOT reachable by walk_to's straight steer from the landed spot or the lip; the way in is the ride's own exit path. NEXT WINDOW'S LEVER (in priority order): (1) collect the three pickups DURING THE RIDE (the ch1→ch53 loop) — they are parented to the carriage [D streams 37-39] and ride WITH Bond, so small real walks inside the car while it descends (the relative offset is constant; no world-space aim needed) put his root inside bounds.radius + 1.1 of each; NO exposure (the machine engages at the ride's end and the car body is cover). Then the fight has 12 rockets in master's shipped run-10 state (or with the stand press, once the stand spot is OFF the hole — e.g. the plate's east edge (112.3,−179.15)); (2) sweep the idle wait before the y-46 trio / #16 fights (each wait shifts the guards' roll) — the upstream bleed (84.0 before the fight: #16 28.0/5, the trio 36.0 (#14 16.0/4 + #11 12.0/3 + #2 8.0/2), world 20.0/29+) leaving 14.0 hp at fight entry remains the wall even with the ammo. The fight-entry pickup scan (retune-nb-08-5, eprintln only, timeline byte-exact) now dumps armour/health + every un-gone pickup within 60 u. Full ledger: the comment block above the test. SHIPPED 2026-11-19 round 6 (retune-nb-08-6, still red — the walk now dies t21773 at (112.69,13.47,-179.52), 700 ticks DEEPER than master's run-10 t21070, in the RIM fight): (a) THE RE-MEASURE on the merged tree (master's SniperAim ENTER c0e23ff23 landed since round 5): the death reproduces the run-10 ledger EXACTLY (t21070, (112.02,13.09,-179.51), the same 55-hit shape) [L this branch's re-measure run] — the guard change did not move this walk. (b) LEDGER CORRECTIONS — the round-5 scan's ammo reads were WRONG-INDEX: 'rocket clip 4 carried 0' read inv.carried.get(0x2c) (the weapon-id index) but the rocket pool is ammo TYPE 0xe [L the run's dump: carried 4] — the fight ALREADY has 8 rockets (clip 4 + carried 4, 808 > the machine's 598, winnable); and ONE cable-car ammo box ALREADY auto-takes at ride start ([take] tick 18432 Some(Ammo 4) [L the NF_PICKUPLOG run]: the ride script spawns the pickups with Bond's boarding spot inside the take radius, dist ~1.19 < bounds.radius + 1.1 = 1.53 [L the take-test rows; M Pickup::touches 0x18e148:210-214]), carried 0 -> 4 = the CAP [D ammo_data type 0xe cap 4; L the take stream: Some(Ammo 4) then -> None from tick 18433 onward] — the other two boxes refuse at the cap EVERY tick forever. So round-5's lever (1) 'collect during the ride' is MEASURED DEAD as an ammo lever; it was implemented anyway (the d3 ride loop steers to the nearest un-taken 0x2c pickup, stops inside 0.85 planar u — the walk holds dist 0.82-0.83 through the descent [L the ride dumps] and NO take ever fires; the fight's rockets unchanged at 8). (c) Lever (2) the lower-floor stand-off: TRIED and DEAD. Implemented (after the crouch drop: the 60-tick lockout out on the zap-free pit floor, the crouched walk to off-hole spots, the stand press): the crouched walk HOLDS feet 13.0856 across the plate — to (112.74,-179.06) and (113.08,-179.24), the plate runs east past x 113 [L the walk logs] — but the STAND press pops the standing capsule to feet 13.470367 at EVERY spot ((112.8,-179.15), (113.08,-179.02), plus round-5's hole footprint and plate edge) — the trench is NARROWER than the standing capsule; no standing spot exists on the lower floor. (d) SHIPPED tree: the ride walk + the stand-off walk + the stand — the fight now runs from the RIM (feet 13.47, eye 14.18 — the standing capsule's contact height on the trench edge): the rim is NEARLY ZAP-FREE (ONE -1.73 world row in ~800 fight ticks [L the lever run]) and the launches FIRE from it (t377 at the machine's y 18.4 pass dist 30.8, t585 at y 20.8 dist 40.1 — both damage 0.0 MISS). (e) THE CLOSER IS THE MACHINE'S MISSILE SALVO [L the decisive pair]: with the launch gate tightened to the proven landing band cp[1] <= 14.5 (TRIED: ZERO launches — the machine never dropped to the band within a launchable bearing in ~800 fight ticks) the walk died to the SAME -15.00/-3.27 world blast at the SAME frame (21775 vs 21774) as the 25.0-gate run WITH its launch — the blast is the Hind's missile on its own schedule, not Bond's own round; the gate reverted to the round-4 measured 25.0. NEXT WINDOW'S LEVERS (priority): (1) answer the missile salvo at the rim — verify the dodge gate's missile arm engages from the rim (the dodge_hold tally) and break the salvo's line (extend the hold past the missile's curve, or shift the stand spot so the carriage body covers the missile's line); (2) the idle-wait sweep before the y-46 trio / #16 fights (the standing ledger item — the rim fight only needs to SURVIVE the first salvo: the salvo takes 18.3 and this tree entered at 20.0 health vs master's 14.0 and died to it at 0.0; +10 fight-entry hp decides it); (3) the rim launches' misses — both fired at the mid passes y 18.4/20.8; the proven landing band y 12.8-13.7 never came up in the window — if the missile answer buys the time, the low band recurs on the ~700-tick circuit. Whole binary 6 passed / 0 failed / 1 ignored, 151.65 s [L this branch]. SHIPPED 2026-11-20 round 7 (retune-nb-08-7, still red — the walk survives the first TWO salvos and reaches fight t~1500-2110, tick 22649-23246, ~1500 ticks DEEPER): (a) THE RE-MEASURE on the merged tree (master's pose-pool/HUD/menu commits since round 6): the death reproduces t21773 EXACTLY [L this branch's re-measure run]. (b) THE DIAGNOSTIC (a per-10-tick machine-bullet dump, d3-gated, eprintln only) split the round-6 death in two [L the run]: the killer missile is weapon 0x62, kind-98 blast radius 4.0, STRAIGHT flight at 1.0 u/tick aimed at Bond's spawn-time chest — and the dodge gate never saw it because Bond was RIDING his own t585 rocket (a 0.0-damage miss at the y 20.8 pass): the dodge is gated on rocket.is_none() and the port's missile cam ZEROES the whole input while the cam is up — the substate-10 player runs no movement arms [M scene.rs ride block; Player_Update 0x193600 case 10] — the missile closed 30 → 0.3 u with dodge_hold 0 the whole way. The ONE episode that did fire (t350, miss 0.53) crawled 0.7 u — the aimed strafe WEDGES at the rim (the diagonal's forward half is a NO-OP: Player_Move zeroes the forward axis while aiming [M player.rs Player_Move's aim branch; the game's 0x191dd0]) — and the SAME law made the walk-back's forward-1.0 a no-op every tick it ever ran. (c) THE SHIPPED PACKAGE (all d3-gated; the sisters byte-exact — whole binary 6/0/1): the dodge RELEASES THE AIM (strafe ±1 + forward 0.5 — the bearing ~89-125° off the killer's travel), the side LATCHES per salvo (the per-episode anchor-bias re-pick measured as a ZIG-ZAG that walked Bond back INTO the second salvo's line, contact −15.00), the drive caps at the live rounds' miss < 2.0 (a pass at miss >= ~1.6 detonates on the wall band for 0.0 to −4.8 instead of the contact −15.00+row — the lever-2 run's t350 salvo: miss 1.61 → the −0.00 rows; the unlimited drive walked Bond 3 u into the volume's NW corner and the −2.00/6-12-tick rows closed the walk), the walk-back PINS while any w62 round is live (miss < 3.0, d < 50), and the walk-back now WALKS (aim released) ALL THE WAY to the anchor (0.35 u — the shared 1.2 threshold measured as a STRANDING 0.76 u short inside the band, the lever-5 run's zaps f23145-23247). The launch band returns to the proven landing top cp[1] <= 13.7 (the 14.5 gate's t2041-class launch fired at the machine's state-2 y 14.4 pass and MISSED 0.0 — the machine outruns the 0.2525-cruise round from a mid-band window; the proven landings were the state-1 y 12.8-13.7 dips). (d) MEASURED [L the eight lever runs, each on its own real run]: the package survives the first two salvos (the t350 salvo dodged for 0.0 at miss 1.61; the killer class for −2.7 to −4.8 wall rows) and the walk reaches fight t~1500-2110 (was t638) — the best run's walk-back returned him to the pocket between salvos and the FIRST LOW-BAND LAUNCH FIRED at t2041. The wall: the terminal's hurt volume covers the WHOLE interior at standing height (the north-anchor experiment wedged at z −177.21 inside −2.00/30-tick rows; the SW corner (110.98,−180.95) and the NW corner (111.99,−180.55) zap the same) — only the pocket (x 112.6-113.1, z −179.0..−179.6) reads nearly zap-free and it cannot hold a 3.5-u clearance from the missile lines' wall exits (−4.8 to −9.4 per salvo at the linear 1−d/4 falloff). The entry health is the wall: the dance costs ~4-9 + 1-2 zaps per salvo, the salvos come every ~150-200 ticks — 20.0 hp buys ~3 salvos; the low band arrives at fight t~1400-2100. NEXT WINDOW'S LEVERS (priority): (1) THE IDLE-WAIT SWEEP before the y-46 trio / #16 fights (the standing ledger item — each wait shifts the guards' roll; +10-20 fight-entry hp makes the dance survivable to the low band's launch windows and the 6 × 101 kills); (2) the dodge side pick aware of the line's WALL EXIT (walk the line forward from the bullet to its box exit and pick the side away from it — the −9.4 blast landed 0.26 u along-line from the closest approach); (3) the launch window: gate on the machine's SLOW low pass (the t2041 miss was a state-2 fast transit; the proven landings were state-1 dips). Whole binary 6 passed / 0 failed / 1 ignored, 107.63 s [L this branch]. SHIPPED 2026-11-20 round 8 (retune-nb-08-8, still red — the shipped timeline reproduces EXACTLY and the window's two sweep levers measured DEAD on their own real runs): (a) THE RE-MEASURE on the merged tree (master + 08-7; no src changes this window): the walk dies t22649 at (111.0,13.5,-180.9) to the -6.06 world row — round 7's ledger tick — with the fight-entry dump armour 5.0 health 20.0 [L the 2026-11-20 baseline NF_DMG_TRACE run]. The upstream bleed is now FULLY attributed: #16 28.0 in 5 (t5168-5583, the pit-excursion descent legs), the y-46 trio 36.0 (#2 8.0/2 + #14 16.0/4 + #11 12.0/3 — ALL at the wedged nav pin (149.5-149.6, 46.1, −93.1) t738-1009, while the DRY-PISTOL return-fire stands ground on #3/#2/#11 (0x1 clip 1 spare 0) and #14 was NEVER picked by any stand), #9/#10/#4 16.0 (t1549-6331), world 20.0 in 3 (the Hind fight t21527-22649); the secure-room armour 50 soaks EVERY hit t6331-21527 (health flat 20.0) and enters the fight at 5.0. (b) LEVER (1) THE IDLE-WAIT SWEEP, both halves TRIED and DEAD on their own real runs (the shipped crouch-sweep precedent — the hall crossing t26671→t29598, the briefcase stretch t29598→t34720 — does NOT extend): (1a) the crouch press BEFORE the east-deck-head nav — the crouched crossing diverged the approach (the fight armed 48 rounds vs the baseline 34) and the deck-line west walk_to wedged at the closed hall door (143.45, 49.55, −97.30) head_bump grounded=false for the 6000-tick cap, ZERO progress [L the lever-1a run]; (1b) the crouch press BEFORE the pit-excursion descent legs — the crouched eye dropped INTO the pit pair's line (the ramp west edge no longer blocks) and the walk died ON THE RAMP t6918 to #15's burst at (153.2, 40.1, −113.0) [L the lever-1b run]. Both reverted; the bleed needs FIGHT-GEOMETRY answers, not stance answers. NEXT WINDOW'S LEVERS (priority): (1) the deck pin 36.0 — the wedge + the stands run INSIDE the trio's crossfire at (149.6, −93.1); a player would fight from OUTSIDE it: pull the return-fire stand spots 2-3 u west (the deck's west lip), or the main agent's live check for a cover spot that blocks the trio's lanes (the placements (152.5-161.5, 46.0, −87.7..−101.5) all fire down the deck line the nav wedged on); (2) #16's 28.0 on the excursion legs — the pit pair's stands fail twice each BEFORE the ramp mid opens the line (killed false t5044/t5384): try the descent with the gun ALREADY raised (the aim press before the legs so the first visible tick fires [S] unverified), or the live check for which leg first opens #15/#16 and whether an earlier leg can; (3) the fight-entry armour 5.0 — the ledge armour box (109.5, 27.8, −129.4) is taken AFTER the platform-mouth fights; check whether the box can be taken BEFORE them (the order swap needs its own real run; the soak that costs 45.0 is the platform/boarding chunk #19/#20/#23/#24/#26). SHIPPED 2026-11-20 round 9 (retune-nb-08-9, still red — the walk dies t26803 at (113.1,13.5,-178.9), 4154 ticks DEEPER than the shipped t22649, and the pit bleed is structurally answered): (a) THE RE-MEASURE on the merged tree (origin/master + 08-8; no src changes): the baseline reproduces EXACTLY (death t22649, fight-entry armour 5.0 health 20.0, the trio 36.0 t738-1009, #16 28.0 t5168-5583) [L the 2026-11-20 baseline NF_DMG_TRACE run]. (b) LEVER (1) the deck pin — the stand pull west 2.5 u TRIED and DEAD on its own real run (an opt-in stand_pull walk in damage_log.rs, REVERTED with the lever): the pulled stands STILL took the −4.0 rows at the pulled spots — the trio's lanes cover the WHOLE deck line, not just the pin (#2 hit at (148.4,−93.8) d 13.7, #11 at (145.1,−92.4) d 10.4, #14 at (147.5,−92.3) d 6.2) — and the fights ran LONGER from the lip (the dry pistol's 1-round cycle walked back east between taps): the deck-head fight armed 12.0 (baseline ~52.0), death t7313 on the pit ramp to #16 — 15336 ticks WORSE [L the 2026-11-20 lever-1 run]. There is NO out-of-lane spot on the deck. (c) LEVER (2) the aim press TRIED and DEAD on its own real run (30-tick aim holds before each excursion descent leg; reverted): the raise is NOT the stands' latency — the hip cone fires during the raise [M Player_SetWeaponAnimObj case 0xf; the sniper_stands settle note], so the first visible tick already fired, and the holds (5 legs × 30 ticks) wobbled every downstream guard roll: #16 still cost 24.0 and the walk died t18365 to #26's burst rifle point-blank (d 1.0) at (106.9,28.4,−120.3) with the armour drained — 4284 ticks WORSE [L the 2026-11-20 lever-2 run]. (d) SHIPPED: the pit fight's legs run under level_fights (the existing ±2.0-y opt-in pick filter, this one call only, d3-gated — the d1/d2 sisters and the top-up walks byte-exact): the descent legs' return-fire stands on the below-level pit pair (dy 3.1-6.1) were ALL blind and ALL failed (killed false t5044/t5384 [L the baseline run]) and each stood Bond in #16's lane for up to 300 unseen ticks; with the filter they never open, the excursion's own machinery owns the pair (the ramp-mid stand killed #15 at t5250 — 290 ticks faster than the baseline's t5540), #16's bleed drops 28.0 in 5 → 8.0 in 1 (the one walk-into hit at (153.6,−113.4) t5227), and the walk crosses the mid-game at health 44.0 (the shipped tree crossed at 20.0, flat from t6331). The boarding chunk then drains the ARMOUR (100 → 0.0, health intact through t22616), and the terminal's upper-floor hurt-volume band ZAPS THE PRE-FIGHT WALK-DOWN: −2.00 × 12 at (111.7, 13.5, −179.2) t25666-25793 (feet y 13.5 — the round-4 ledger's upper-floor band; the lower floor 13.086 is zap-free [L this run's zap rows]) — the +24.0 mid-game gain is spent standing on the band before the crouch drop, and the walk enters the fight (fight t0 = frame 26206 [L the fight log's own t0 line]) at armour 0.0 health 20.0 — the SAME entry health as the shipped tree. The fight runs ~600 ticks and dies to the −2.33/−13.22/−4.45 world rows at (110.9-113.1, 13.5, −178.8..−180.9) — death t26803 (was t22649); the route's pre-fight phase is ~4700 ticks longer this run (the wobbled transits), so part of the death-tick gain is pace — the ATTRIBUTABLE lever effect is the #16 saving (28.0 → 8.0) and the mid-game health 44.0. Whole binary 6 passed / 0 failed / 1 ignored [L the final-tree run]. NEXT WINDOW'S LEVERS (priority): (1) THE PRE-FIGHT WALK-DOWN ZAPS — 12 × −2.00 on the upper-floor band at (111.7, −179.2) t25666-25793, all BEFORE the crouch drop: shorten the standoff→drop path (or drop from a spot out of the band) and the fight entry returns to the earned 44.0 — with a 44.0 entry the round-7 dance has the rounds to reach the low-band launch windows (the round-7 finding: the low band arrives at fight t~1400-2100); (2) the fight's own world rows (−2.33/−13.22/−4.45 in ~600 ticks at (110.9-113.1, −178.8..−180.9)): the round-7 rim answer — judge the dodge/walk-back package from the pocket with a fuller entry; (3) the trio 36.0 — every deck-leg lever has now measured dead (the crouch sweep, the pull west); before spending another one, a live check for the STATION HALL MOUTH's (139.0, 49.5, −93.0) sight lines into the trio's placements (the mouth is ABOVE the deck, dy 3.4 — a player fights the deck from the hall, not from the lanes). SHIPPED 2026-11-20 round 10 (retune-nb-08-10, still red — the walk dies t28256 at (113.08,13.47,-178.96), 1453 ticks DEEPER than the shipped t26803, and the pre-fight walk-down zaps are structurally answered): (a) THE RE-MEASURE on the merged tree (origin/master + 08-9, no src changes): the baseline reproduces EXACTLY (death t26803, fight-entry armour 0.0 health 20.0, the 12 walk-down zaps t25666-25793) [L the 2026-11-20 baseline NF_DMG_TRACE run]. The zaps' anatomy [L the same run]: the ride ends inside the parked car at (111.66,13.54,-179.17), the ~1500-tick bottom-of-mountain cinematic runs with Bond standing there ZAP-FREE (the volume not armed), then 8 zaps land DURING the pane shoot-out (the first t25666 — the volume ARMS with the machine's approach ~106 ticks after wait_enabled returns) and 4 more during the anchor walk + drop press t25775-25793. (b) SHIPPED lever (1) the ledger's priority item: the whole d3 drop sequence (the anchor walk_to + the drop press + the crouch drop + the lockout + the stand-off walk + the stand press) moves ABOVE the pane shoot-out, running RIGHT AFTER the cinematic ends — the drop takes ~65 ticks from the ride-end spot and lands the zap-free plate (feet 13.086) BEFORE the arming; the shoot-out then runs from the rim. MEASURED [L the lever-1 run]: the pre-fight zaps drop 12 → 1 (ONE −2.53 row at t26175 at the stand spot), the fight entry returns to health 41.5 (was 20.0), fight t0 26248 (the tick-neutral reorder — the machine's route phase at fight t0 unchanged), death t27636 at (113.08,13.47,-178.93) — 833 ticks deeper. (c) SHIPPED lever (2): the d3 fight anchor moves to the shipped stand spot (113.08, −179.02) — the lever-1 run measured the OLD anchor (112.5-112.6, −179.39) at rim height ZAPPING the fight away: 13 −2.00 rows in the last ~830 fight ticks (the dense burst every 6 ticks t27409-27636) closed a walk that had survived every missile salvo; the stand spot measured ONE −2.53 row in ~430 ticks and the round-6 launches FIRED from it (t377/t585). MEASURED [L the lever-2 run]: the anchor zaps are GONE (Bond held 13.6 health for 981 fight ticks at the anchor), the fight reaches ~2000 ticks (the round-7 low-band window), death t28256 — 1453 ticks deeper than the shipped tree. (d) THE REMAINING WALL — ZERO launches in BOTH lever runs (line_ok FALSE at all 105 fight samples, the machine's hp 598 untouched): the machine's route phase at fight t0 (path t ~630, the approach nearly done) flew the whole window on the southeast far legs (dist 36-61, y 16-22) and the HIGH overhead passes (y 24-29 at dist 15-27) — from the stand spot the southeast bearings exit through the south-east corner post (the ray to (135.8,16.5,-209.5) crosses z −182.18 at x ~115.4, ~1.8 u east of the shot-out south pane's east edge 113.62) and the high passes stop on the lintel; THE DIVE IS NOW MEASURED [L the 2026-11-20 approach dump — the pre_fight buffer printed at the fight loop's start, an eprintln-only d3-gated diagnostic, the timeline byte-exact]: the machine spawns f~24873 (path t 0, ch5's gate) and its approach dips INTO the launchable band at frames ~25710-25800 (path t 424-470) — y 13.61 (f25710) → 13.08 (f25740) → 12.84 (f25770) → 13.07 (f25800) → 13.93 (f25830) — at x 69-93, z −214..−219, dist ~49 southwest of the stand spot (the round-4 launch distance class 45-48) — and the fight loop opens f26248, ~450 ticks AFTER the dive ends. The 2000 observed fight ticks never saw the machine below y 18.4 (the post-dive circuit is the high/far class), so the low band does NOT recur inside the fight window — the round-7 'low band arrives at fight t~1400-2100' was that tree's fight window. THE TAIL IS NOW MEASURED [L the 2026-11-20 tail stamps, eprintln-only, the timeline byte-exact]: the drop block starts f24872 (the cinematic's end) and ends f25309 — 437 ticks, of which ~140 are the stand-off east walk WEDGING at k80 (the log: k80-k200 parked at (113.08286, −179.23654), the (113.3, −179.25) target 0.22 u short of reachable — a shorter target frees them); the shoot-out runs f25309-26196 — 887 ticks for ONE pane round (pane 2 killed after 1 rounds [L the lever-1 run]) — the shoot-out's internal split (the convergence, the %12 fire boundary, the sidestep retries at 120 ticks each) is the next instrument; the fight loop then opens ~26248. Compression budget: the dive ends f25800 — the fight loop must open ≤f25710, so the tail must shed ~540 ticks (the east walk's 140 + the shoot-out's excess ~400). NEXT WINDOW'S LEVERS (priority): (1) instrument the shoot-out's internal stages (tick stamps per pane window: the convergence ticks, the fire ticks, the sidestep retries), compress it + shorten the east-walk target, so the fight loop opens ≤f25710 — INSIDE the dive — with the panes shot and Bond at the stand spot; (2) the east-pane bearing: a stand spot a few tenths south (z −179.3..−179.5, still rim-height zap-free per the lever-1/2 zap map) swings the southeast circuit bearings into the east pane pair's windows (113.37-113.62, −180.7..−181.6); (3) the trio 36.0 live check (unchanged). Whole binary 6 passed / 0 failed / 1 ignored, 111.74 s [L the final-tree run]."]
// nb-scan-all-08 ledger (2026-11-18, still red; the original fight code measures t28298 and
// every lever this window tried measured WORSE — each on its own real run):
// 1. the original anchor+crouch fight (112.5, −179.4): death t28298 at (112.25,13.09,−179.16),
//    59 hits for 100.0 — 32.2 in 38 world hits (the gun chips −2.00 × 5 at frames 27474-27638,
//    the −7.78/−8.22 missile blasts) + the upstream 48.0, ONE rocket launched (t1269, in flight
//    at death) [L the run]. The nb-scan-all upstream is ~6500 ticks faster (the old green runs'
//    fight started ~t35500; this one ~t25700), so the machine's route phase differs and its
//    nb-scan-all passes fly HIGH OVERHEAD (y 25-29 — copter (113.9,28.8,−185.3), (113.0,29.1,
//    −182.7) [L the run logs]) — from inside the cabin EVERY stance's sight ray to them stops on
//    the lintel/west band 1.1 u out (crouched slit eye stop t0.106 at (111.1,15.0,−179.4), gunR
//    t0.124, gunL t0.115 [L the ray dump]) and line_ok reads FALSE at every 100-tick sample.
// 2. the crouched slit anchor (110.98, −179.31), the bias/walk-back there: death t29087, ZERO
//    launches (the crouched eye's sight stops on the lintel; clip 4 intact), the gun chips
//    (−2.00 × 5 at frames 28046-28207) drained armour 38 → 0 and the −14.00 blast closed [L].
// 3. the standing slit + the aim gates relaxed to the sisters' 0.25: death t27383 — the pain-1.0
//    drips (the d2 −1.40 drips at −2.00) with still no launch windows [L].
// The upstream cost the next window should attack first: the station-hall y-46 trio snipes the
// whole hall under the spacing scan — #14 (151.2,46.0,−97.8) 16.0/4, #11 (153.7,46.0,−85.9)
// 12.0/3, #2 (156.4,46.0,−91.0) 8.0/2, plus the secure-room #5 12.0/4 — 48.0 of the 78 budget
// before the fight [L the ledger]. NEXT LEVER: (1) a hall route/stance/fight-order change that
// answers the mezzanine snipers early (a hall-mouth stand on the live snipers, the fight-order
// lever family); (2) the fight outlasts the machine's high phase only with that budget back —
// the d2 phase's low passes (y 13-24) gave five standing-slit launches in 2100 ticks, so pin
// the machine's route period from the flight log before re-tuning the stance.
// retune-nb-08-2 ledger (2026-11-19, still red — the walk reaches 2.5x deeper, t17664 vs
// t6408-class; each lever judged on its own real run):
// 1. BASELINE reproduced exactly: death t28298 at (112.25,13.09,-179.16), 59 hits for 100.0
//    [L the NF_DMG_TRACE run on this branch]. The trio's geometry: all 9 hits land at the
//    lane mouth (149.5-149.6, 46.1, -93.1) where the transit nav WEDGES for ~860 ticks —
//    ZERO hits west of x 149.5; #14 4.2 u, #11 7.3, #2 12.9, all weapon 0xa; the mid-nav
//    return fire kills #3 t741 / #2 t1013 / #11 t1155 but NEVER #14 (the hurting-first pick
//    filters VISIBLE only — #14 is buried at 3.4 u from the deck-head arrival; #14's 16.0
//    lands during the #2/#11 stands). The fights run on the DRY PISTOL (0x1 clip 1 spare 0,
//    the resupply found nothing on the corridor) — 60-tick reloads per tap are the exposure
//    engine.
// 2. the lane-mouth pre-empt TRIED and DEAD: the mouth target's own nav chain wedges at
//    (141.0, 50.4, -93.7) grounded=false (walk_to AND walk_nav_to — the deck-head route is
//    the only clean approach) [L the lever runs].
// 3. the arm-up + skip-continue SHIPPED: the deck-head wide fight (hurt pick, back-off
//    suspended, unseen creep) + scoop_drops(3) + arm_fullest post-nav, and the wide fight
//    skips a failed pick instead of breaking (the pre-fix run: the failed #5 stand broke the
//    fight, the pit guard #15 kept firing and killed the walk at the doorway t6408, -23.0
//    [L the lever run]). Measured: the walkway pair dies at ZERO cost (was -12.0), #15 dies
//    in the pit excursion t5449, the whole ledger 35 hits for 100.0 (was 59), and the death
//    moves to the BOARDING LEDGE t17664 at (110.07,28.31,-121.26): #27 (skin 0x500002f,
//    (100.8,28.3,-122.2), 9.3 u) burst 21.0 in 7 hits with hp 12 INTACT — the return fire
//    never landed on him. Whole binary 6/0/1 (the sisters byte-exact) 150.25 s [L].
// retune-nb-08-3 (2026-11-19, still red — the walk reaches the machine fight): the
// hurting-first pick's unseen-cover fallback SHIPPED (the same fallback the plain and lane
// arms run, opt-in via the UNSEEN_FIGHTS thread-local; d1/d2 set no UNSEEN_FIGHTS and stay
// byte-exact): the boarding ledge is PASSED with ZERO #27 damage — #27 (skin 0x500002f)
// never lands a round (he burst 21.0/7 in the round-2 run) — and the walk reaches the CABIN
// fight, the route's deepest point: death t21777 at (111.94,13.09,-179.18), 61 hits for
// 100.0 (17 return-fire fights) — #16 28.0/5, the trio 36.0 (#14 16.0/4 + #11 12.0/3 +
// #2 8.0/2), world 18.0/34 (the last -2.00 drips at frames 20809-21778 close it) [L the
// NF_DMG_TRACE lever run]. LEVER (2) the mid-fight scoop TRIED and DEAD on its own runs
// (the clip-gated stand hook; three variants measured: no-fire at the fights' clip 3-7
// spare 27-41, the 14 u corridor walk death t1325, the 9 u corridor walk death t1759 —
// every walk-under-fire variant costs more than the armed tail saves). NEXT LEVER: (3) the
// Hind outlast (the route period pin, the live check in the report).
fn walk_the_whole_mission_on_real_input_real_damage_00agent() {
    // GLASS-SIGHT (guard-glass-finish): the Agent sister's sight rule — the d2 test's comment.
    // UNSEEN FIGHTS (level-08-rocket-self-blast): the lower level east room's second burst pair
    // (#26/#27, skin 0x500002f) sits behind the half-open door leaves (state 0x2 pos01 0.50 [L
    // the 2026-11-08 run's STUCK dumps]) — no eye ray reaches them from inside the room, the
    // platform-mouth stand picked "no visible close guard", and the walk crossed their lane at
    // 15 health and died at tick 29022 (−12/−3 point blank, both bursts the moment the door
    // line opened). The unseen-cover fights are the game's own push-to-the-door-lip-and-shoot:
    // the hurting-first fallback pick opens the stand on the shooter actually hurting Bond and
    // the stand creeps until the line opens. Opt-in per test: the sisters' timelines stay
    // byte-exact (none of them sets it).
    walker::with_real_damage_glass_fights(|| walker::with_real_damage_unseen_fights(|| walk_the_whole_mission_on_real_input_at(3)))
}
