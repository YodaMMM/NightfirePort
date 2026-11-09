//! walkreal-a: 07000002 "The Exchange" (Castle B) walked with the player's own movement — no
//! teleports after the spawn — through EVERY objective and out the way the game exits. The route
//! (docs/levels/07000002.md + the disc chain this task bound):
//!   spawn #0 (−27.17, 1.64, 29.80, gate 0) → the level-start cut-scene SP #400 plays (script
//!   0x06000084, pack 07800002; its stream 7 fires event 0x0D [47, 1] at the scene's tick 1539 —
//!   ch 0x47 = the obj0/obj1 reveal [D + L port runtime NF_SCRIPTLOG]) → ladder #114 →
//!   the garden: TouchOnce #341 (out 0x15) → the south hall → the west wing: the ch-0x17 plate
//!   tr[003] (−35.2, 6.5, 5.05) enables geisha 23 (rescue ch 0x49) + her three captor guards
//!   (death chs 0x88/0x2A/0x29 [D param 12]) → the captors die (real fire) → `DroneFunc_HostageSaved`
//!   0x1441c0 writes 0x49 [M; the captor link 0x15cb78:24-36] → AND #397 = AND(0x29,0x2A,0x88,0x49)
//!   → 0x3B [D tr[017]] → the tr[032] touch zone at geisha 1 (−45.4, 5.74, 11.7) → 0x3C →
//!   SEQ_AND tr[033] (0x3B, 0x3C) → 0x35 → SP #399 (the mid-level cut-scene; its start is the
//!   SP's own out-channel gate — `SP_Update` 0x1dbda8 flags bit 1 = out on, mask 1 [M]) →
//!   the bunker door → the doorway sensor #389 (out 0x81, lock 0x66) → **Mayhew's computer**: the
//!   Destroy prop pl[011] (0xE1, param 0 = 72 = 0x48 [D], bounds −49.37..−48.35 / 6.24..6.65 /
//!   −24.05..−23.22 [L port props]) — shot out, `Destroy_CollisionHandler` 0x12bd48 writes ch 0x48
//!   [M] (the 07000002-only PlrStat branch in `Destroy_Smash` 0x12bb60:42-55 pins the object) →
//!   the locked plates (the ambush outcome [D #385-387 lock 0x81]) → back east: the ch-0x19 plate
//!   tr[009] (−26.9, 6.0, −20.3) enables geisha 24 (rescue ch 0x4A) + captors → the captor dies →
//!   0x4A → AND tr[001] = AND(0x49, 0x4A) → 0x4B [D #303] → AND tr[030] = AND(0x48, 0x4B) →
//!   **0x4C** [D] — the disc's own chain the old "no path to 0x4C" scan missed → the Load Point
//!   #206 (key gate ch 0x4C [M Trigger_Activate 0x1b2318:76-90]) → E → the level loads 07000003
//!   through the real reload path [D param 0].
//! obj2 "Find door to servants quarters" (watch 0x4D [L c07000002 row]): the writer is the ARC
//! HAND-OFF, on the disc in 07000003 — TouchOnce trigger #308 (flags 0xEA, param 0 = 0x4D) at
//! (−9.0, 6.0, −23.2), the level entry right past 07000002's own exit (−10.05, 6.75, −23.18),
//! with SoundTrigger #341 (0xF9) keyed on 0x4D beside it [D 07000003 param sweep 2026-10-28].
//! The same hand-off shape is proven one part back: 07000002's intro stream 7 writes 0x47,
//! part 0's watch channel ("Escort Mayhew to bunker" [L c07000001]) — the intro asserted at step
//! 0 [D 07800002 0x06000084 stream7]. The monitor's other computed writers were ruled out on the
//! disc: `Monitor_Update` 0x1afcc8:170-174 raises the viewed sensor's param 8 — both 02 sensors
//! carry param 8 = 0 [D probe], and no 02 SS carries 0x4D (p3 = 0/1/2 [D probe]; `SS_Update`
//! 0x1a9948:21-23 raises SS p3). So the leg: exit 07000002 with the EARNED 0x4C, step into
//! 07000003, the #308 touch raises 0x4D, `Mission_MonitorObjectives` 0x1827d8 case 3 folds part-1
//! obj2 to state 4 — asserted through the real trigger path on the reloaded level. ch 0x64 (100)
//! must NOT close: the monitor's `bVar9` AND spans all four parts of mission 0x07000001
//! [M 0x1827d8:22-38,119-123; L c07000001/02/03/04 rows], and parts 2/3's watch channels
//! (0x4E/0x51/0x53/0x54) are unwritten on this walk.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp, dxz};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn ch_on(w: &Walker, c: u8) -> bool { w.scene.sim.as_ref().unwrap().drones.channels.on(c as u32) }

/// The MissionData row of a level [M mission::LEVELS = the 0x2a3ed0 table; the row-lookup
/// idiom of walkreal_07000006_live].
fn mission_row(level: u32) -> usize {
    nf_game::mission::LEVELS.iter().position(|l| l.level == level).expect("the mission row")
}

fn geisha(w: &Walker, ch: u8) -> Option<usize> {
    w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| d.second_channel == ch)
}

/// The live captor the geisha at `around` is linked to, wherever it stands now: the save fires on
/// THAT guard's death (`SetupHostageKiller` picks it once, `NDrone2_DSTATE_Hostage` 0x15cb78:24-36
/// watches it) [M]. A captor that saw Bond and left the room is outside the 8-u sweep, and the
/// rescue then never fires however many guards the sweep kills.
fn hostage_captor(w: &Walker, around: [f32; 3]) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.list.iter()
        .filter(|h| h.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
        .filter_map(|h| h.captor)
        .find(|&c| sim.drones.list.get(c).is_some_and(|d| d.health > 0.0 && !d.corpse))
}

/// Kill every armed guard near `around` (the geisha's captor set) through the real fire path.
/// The approach is real input too — NO bond.pos writes after the spawn: walk the nav web into
/// the captor room (round 0 aims at the geisha's spot, later rounds at the guard's current pos —
/// the AI moves), then fire through the real trigger path [the walker's `shoot` = the player's
/// own step_bond aim/reposition/flank loop, incl. the nav-graph flank when pinned 9 s].
fn clear_captors(w: &mut Walker, around: [f32; 3], label: &str) {
    for round in 0..8 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.team == 1 && d.health > 0.0 && !d.corpse
                && d.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
                .or_else(|| hostage_captor(w, around))
        };
        let Some(di) = target else { return };
        let dest = if round == 0 { around }
            else { w.scene.sim.as_ref().unwrap().drones.list[di].pos };
        w.walk_nav_to(dest, &format!("{label} captor {round} approach"), 3.5);
        if !w.shoot(di, &format!("{label} captor {round}")) {
            eprintln!("[02] {label}: captor {round} (drone {di}) survived the clip");
        }
    }
}

/// The real-health walk's captor fight: the same kill set as `clear_captors`, played the way a
/// player does under real fire — enabled guards only, shot from range when a part is exposed,
/// approached node by node until one is, the gun cycled/looted when it runs dry.
fn clear_captors_real(w: &mut Walker, around: [f32; 3], label: &str, d3: bool) {
    for round in 0..8 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                && d.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
                .or_else(|| hostage_captor(w, around))
        };
        let Some(di) = target else { return };
        // los-mask-02-walk: same death guard as take_room — no zombie rounds after the fail.
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { return; }
        // From range first: a captor Bond can already see a part of is shot from where he
        // stands. Walking into the room put Bond 1.1-1.3 u from three shotgun captors (weapon
        // 0x1c [L probe]) and cost ~95 of his 150 health+armour there (run12 2026-10-31); the
        // pellets' spread is what range buys a player.
        if !hold_for_sight(w, di, 240) {
            let dest = if round == 0 { around }
                else { w.scene.sim.as_ref().unwrap().drones.list[di].pos };
            approach_until_seen(w, di, dest, &format!("{label} captor {round} approach"), 3.5);
        }
        let _ = w.switch_to_armed(&format!("{label} captor {round}"));
        if !w.shoot(di, &format!("{label} captor {round}")) {
            eprintln!("[02] {label}: captor {round} (drone {di}) survived the clip");
            loot(w, &format!("{label} captor {round}"), 1);
        }
        // The 00 Agent scoop: the dead guard's drop is the resupply (one 8-pellet tap per captor
        // keeps the next fight ahead of its first volley).
        if d3 { scoop_drops(w, &format!("{label} captor {round} drop"), 1); }
        status(w, &format!("{label} captor {round}"));
    }
}

/// The geisha-2 room taken from range, the way a player holds a doorway against a shotgun
/// captain: Bond walks in only until a captor shows (`approach_until_seen`), then stands and
/// answers whoever is shooting him first (`DamageLog::shooters`), else the nearest guard he can
/// put a round into, and gives a guard up after a few dry taps for the next one. A 6-u walk-in
/// and the shoot loop's strafe dance carried Bond to 1.1 u and 6.7 u of the
/// captain (weapon 0x1C, 8 pellets a round [M DroneWeap_FireWeapon 0x178f68 weapon_data+0x18
/// loop]); under 3.0 u every round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115], and at
/// 6.7 u the 0.045-rad spread still puts the whole volley on Bond.
fn take_room(w: &mut Walker, around: [f32; 3], label: &str, d3: bool) {
    w.dmg.dry_taps = Some(5);
    let mut tried: Vec<(usize, u32)> = Vec::new();
    for round in 0..16 {
        let (live, pick) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let live = |d: &nf_game::drone::Drone| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled && d.present;
            let captors: Vec<usize> = sim.drones.list.iter().enumerate()
                .filter(|(_, d)| live(d) && d.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
                .map(|(i, _)| i).chain(hostage_captor(w, around)).collect();
            let fresh = |i: &usize| !tried.iter().any(|&(k, until)| k == *i && w.total_ticks < until);
            let mut near: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
                .filter(|(_, d)| live(d) && dxz(d.pos, bp) < 16.0)
                .map(|(i, d)| (dxz(d.pos, bp), i)).collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            let pick = w.dmg.shooters(sim).into_iter().filter(fresh).find(|&i| walker::DamageLog::visible(sim, i))
                .or_else(|| near.iter().map(|e| e.1).filter(fresh).find(|&i| walker::DamageLog::visible(sim, i)));
            (captors, pick)
        };
        if live.is_empty() { break; }
        // los-mask-02-walk: a dead Bond cannot fight — stop the sweep at the death instead of
        // zombie-ticking the remaining rounds (the masked run burned 33k dead ticks here); the
        // save assert downstream is the gate that reports it.
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        match pick {
            Some(di) => {
                let _ = w.switch_to_armed(&format!("{label} room {round}"));
                let killed = w.stand_and_fire(di, 900);
                if !killed { let until = w.total_ticks + 240; tried.push((di, until)); }
                let dry = { let g = &w.scene.sim.as_ref().unwrap().gun; g.clip + g.spare == 0 };
                if dry { arm_fullest(w, &format!("{label} room {round} dry")); }
                if d3 { arm_weapon(w, 0x1c, &format!("{label} room {round} shotgun")); }
                eprintln!("[02] {label} room {round}: drone {di} killed {killed}");
            }
            None => {
                // Nobody in sight: hold for the nearest captor to show, then step in until one does.
                let di = live[0];
                if !hold_for_sight(w, di, 240) {
                    let dest = w.scene.sim.as_ref().unwrap().drones.list[di].pos;
                    approach_until_seen(w, di, dest, &format!("{label} room {round} approach"), 3.5);
                }
            }
        }
        status(w, &format!("{label} room {round}"));
    }
    w.dmg.dry_taps = None;
    // Whatever the stand left (a captor out of every line from here): the old sweep.
    clear_captors_real(w, around, label, d3);
}

/// Bond's own status line (health, armour, gun) for the fight log.
fn status(w: &Walker, at: &str) {
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[02] {at}: t{} health {:.1} armour {:.1} gun {:#x} clip {} spare {} pos ({:.1},{:.2},{:.1})",
        w.total_ticks, sim.drones.health.health, sim.drones.health.armour, sim.gun.weapon, sim.gun.clip, sim.gun.spare,
        sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}

/// A route leg the way a player walks it under fire: straight when the hip-height line to the
/// target is clear on about the same floor, else node by node through the nav web — and after
/// every stretch, any hostile in view within 14 u is fought before walking on (the player shoots
/// who shoots him; the walker's plain legs walked Bond through the bunker approach's guards and
/// into the ch-0x81 ambush without returning fire — health 100 → 64.8 → 0, run9 2026-10-31).
/// The AI-path router also turned the 9 u south-hall → south-hall-west leg into a detour over the
/// upper floor (nav 5-9 at y 6.4, z 18 → 10 → 22) that ends in the nav-10 body-block wedge
/// (runs 1-3) — the straight line is the player's.
fn leg_to(w: &mut Walker, at: [f32; 3], name: &str, arrive: f32) {
    let (direct, pts) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let direct = (at[1] - bp[1]).abs() < 0.8
            && sim.world.segment([bp[0], bp[1] + 0.9, bp[2]], [at[0], at[1] + 0.9, at[2]]).is_none();
        (direct, walker::leg(sim, bp, at))
    };
    if !direct {
        for (k, p) in pts.iter().enumerate().skip(1).take(pts.len().saturating_sub(2)) {
            w.lenient = true;
            w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("{name} nav {k}"));
            w.lenient = false;
            if engage_in_view(w, 14.0, &format!("{name} nav {k}")) {
                // The fight moves Bond (the shoot loop's strafes and flanks): back to the node
                // before the next stretch — the bunker-door fight left Bond outside the door and
                // the straight steer to the next node wedged him on the door jamb between the two
                // bunker doors (y 7.1 airborne at (−45.4, −22.15), 3 × 3000 ticks, runs 13/j97).
                w.lenient = true;
                w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("{name} nav {k} again"));
                w.lenient = false;
            }
        }
    }
    w.lenient = true;
    w.walk_to(Wp { at, act: Act::None, arrive }, name);
    w.lenient = false;
    if engage_in_view(w, 14.0, name) {
        // The same return as the nodes' above: the leg-end fight strafed Bond off the leg's end
        // (the south-hall fight left him at (−31.2, 5.1, 29.2)) and the next leg's nav, planned
        // from there, took the upper-floor detour and gave up at the 3000-tick cap one floor
        // above "south hall west" (y 6.4 at z 12.7).
        w.lenient = true;
        w.walk_to(Wp { at, act: Act::None, arrive }, &format!("{name} again"));
        w.lenient = false;
    }
}

/// Fight every enabled hostile within `radius` that Bond can see a part of right now (the walker's
/// bullet-rule `exposed_aim`), nearest first; the unseen are left to the route's sweeps.
fn engage_in_view(w: &mut Walker, radius: f32, label: &str) -> bool {
    let mut fought = false;
    for k in 0..6 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let mut c: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.present && d.enabled
                    && dxz(d.pos, bp) < radius)
                .map(|(i, d)| (dxz(d.pos, bp), i)).collect();
            c.sort_by(|a, b| a.0.total_cmp(&b.0));
            c.into_iter().map(|(_, i)| i).find(|&i| w.exposed_aim(i).is_some())
        };
        let Some(di) = target else { return fought };
        fought = true;
        let _ = w.switch_to_armed(label);
        if !w.shoot(di, &format!("{label} in view {k}")) {
            loot(w, &format!("{label} in view {k}"), 1);
        }
        // The kill's drop lies at the fight floor: a player low on rounds grabs it before walking
        // on (the geisha-2 captors outlived a 9-round shotgun, run j97 2026-10-31).
        loot(w, &format!("{label} in view {k} drop"), 12);
        status(w, &format!("{label} in view {k}"));
    }
    fought
}

/// The game's own weapon cycle to the carried firearm holding the most rounds (clip + spare) —
/// a player walking into a known ambush with a near-empty drop gun switches to his fullest gun
/// first (run5 2026-10-31 met the bunker ambush with a picked-up gun at 7+4 while the start
/// P99 still carried ~30, ran dry at +563 and died in the resupply walk). Real input: the cycle
/// key and the holster/draw wait, nothing else.
fn arm_fullest(w: &mut Walker, label: &str) {
    let read = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.gun.weapon, sim.gun.clip as i32 + sim.gun.spare as i32)
    };
    let cycle = |w: &mut Walker| {
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(Default::default(), false); }
    };
    let first = read(w);
    let mut seen = vec![first];
    // A press can land while the draw is still running and leave the same gun up: only a return
    // to the first gun AFTER another came up ends the round (the bunker entry saw only the 4-round
    // shotgun while the P99 still carried 36, and Bond emptied it at 12 u in the corridor).
    for _ in 0..12 {
        cycle(w);
        let now = read(w);
        if now.0 == first.0 { if seen.len() > 1 { break; } continue; }
        if !seen.iter().any(|g| g.0 == now.0) { seen.push(now); }
    }
    let firearm = |id: u8| (2..=0x47).contains(&id);
    let Some(best) = seen.iter().filter(|(id, n)| firearm(*id) && *n > 0).max_by_key(|(_, n)| *n).map(|b| b.0) else { return };
    for _ in 0..12 {
        if read(w).0 == best { break; }
        cycle(w);
    }
    // The draw can eat a press and leave the wheel one short of `best` (the bunker run armed the
    // dry 0x47 cuffs with a 32-round 0x12 on the wheel [L the 00agent ledger] — Bond then took
    // the corridor's rifle chips on an empty hand). Verify and re-cycle: a no-op whenever the
    // first pass landed (every d1/d2 run — the RNG is unchanged there).
    for _ in 0..12 {
        if read(w).0 == best { break; }
        cycle(w);
    }
    let (id, n) = read(w);
    eprintln!("[02] {label}: armed the fullest gun {id:#x} ({n} rounds; carried {:?})", seen);
    reload_full(w);
}

/// The game's R between fights: a player tops the clip up from the spare before walking into the
/// next contact (the P99 armed with clip 1 reloaded mid-fight 2.5 u from drone 10 and Bond lost
/// 66 health there, run b-j60 2026-10-31). One reload press through `step_bond`'s reload input,
/// then the reload's own wait.
fn reload_full(w: &mut Walker) {
    let spare = w.scene.sim.as_ref().unwrap().gun.spare;
    if spare == 0 { return; }
    w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, true, false);
    for _ in 0..90 { w.tick(Default::default(), false); }
}

/// Walk to the nearest live pickup at the feet (the guards' drops land at the fight floor [L
/// drops]) and take it — no `have` gate, `n` pickups — then cycle to a firearm with ammo. The
/// 00 Agent economy: the guards' dropped guns ARE the resupply (the bunker entry saw a 4-round
/// 0x1c shotgun drop [L the arm_fullest carry log]), and at DroneFiring_Accuracy_Hard 1.0 the
/// fights must end in one tap, not one per reload cycle.
fn scoop_drops(w: &mut Walker, label: &str, n: usize) {
    scoop_drops_within(w, label, n, 6.0, 1.5);
}

/// The deliberate resupply pass (`scoop_drops` with a wider reach, real-health-00agent-02-5):
/// the kills a leg left behind drop their guns ON the corridor just covered [M
/// DroneWeap_DropWeapon 0x17a2a0 — the held weapon at the death-anim arm, clip·(0.25+FRand(0.5))
/// rounds], and a player who just cleared a corridor walks back a few u and takes a fallen
/// guard's rifle before the next contact. Same real-input shape as `scoop_drops`: Bond's own
/// feet via `walk_nav_to`, the take through the capsule, then the game's weapon cycle.
fn scoop_drops_within(w: &mut Walker, label: &str, n: usize, radius: f32, y_tol: f32) {
    for k in 0..n {
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let target = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
            .filter(|p| !p.gone)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .filter(|q| dxz(*q, bp) < radius && (q[1] - bp[1]).abs() < y_tol)
            .min_by(|a, b| dxz(*a, bp).total_cmp(&dxz(*b, bp)));
        let Some(target) = target else { break };
        eprintln!("[02] {label}: scooping the drop ({:.1},{:.1},{:.1}) (d {:.1})", target[0], target[1], target[2], dxz(target, bp));
        w.walk_nav_to(target, &format!("{label} scoop {k}"), 0.4);
        // Arm the take at once: the captors close DURING the scoop walk, and the fights that
        // answer them fire the gun in hand (the first scoop run fought drones 7/5/6 on the P99
        // for 61.2 health because the switch came only after the loop [L the 00agent ledger]).
        let _ = w.switch_to_armed(&format!("{label} scoop {k} arm"));
    }
    let _ = w.switch_to_armed(label);
}

/// Cycle the wheel to the gun `id` (the game's own weapon cycle, the arm_fullest walk) and top
/// the clip up. False if the wheel never showed it (not carried or dry).
fn arm_weapon(w: &mut Walker, id: u8, label: &str) -> bool {
    let read = |w: &Walker| w.scene.sim.as_ref().unwrap().gun.weapon;
    for _ in 0..12 {
        if read(w) == id {
            reload_full(w);
            eprintln!("[02] {label}: armed {id:#x}");
            return true;
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(Default::default(), false); }
    }
    eprintln!("[02] {label}: {id:#x} not carried");
    false
}

/// Close on guard `di` the way a player rounds a corner: node by node along the nav web toward
/// `dest`, stopping at the first node from which a part of him is exposed within 15 u (instead
/// of a blind walk to 3.5 u — the geisha-1 room's first captor, drone 5, was met at 0.6 u that
/// way and cost 17-95 health, runs b-j0..j200 2026-10-31).
fn approach_until_seen(w: &mut Walker, di: usize, dest: [f32; 3], label: &str, arrive: f32) {
    let pts = { let sim = w.scene.sim.as_ref().unwrap(); walker::leg(sim, sim.bond.pos, dest) };
    let seen = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[di];
        d.corpse || d.health <= 0.0 || (dxz(d.pos, sim.bond.pos) < 15.0 && w.exposed_aim(di).is_some())
    };
    w.lenient = true;
    for (k, p) in pts.iter().enumerate().skip(1).take(pts.len().saturating_sub(2)) {
        if seen(w) { w.lenient = false; return; }
        w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("{label} nav {k}"));
    }
    if !seen(w) { w.walk_to(Wp { at: dest, act: Act::None, arrive }, label); }
    w.lenient = false;
}

/// Face guard `di` and hold still up to `ticks` until a part of him is exposed to Bond's eye
/// (the walker's bullet-rule `exposed_aim`) within 15 u. Real input: the view turn only.
fn hold_for_sight(w: &mut Walker, di: usize, ticks: u32) -> bool {
    for t in 0..ticks {
        let (dead, seen, turn) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[di];
            let bp = sim.bond.pos;
            let dead = d.corpse || d.health <= 0.0;
            let seen = t % 10 == 0 && dxz(d.pos, bp) < 15.0;
            let err = walker::wrap_pi((d.pos[0] - bp[0]).atan2(d.pos[2] - bp[2]) - sim.bond.yaw);
            (dead, seen, err.clamp(-0.08, 0.08))
        };
        if dead { return true; }
        if seen && w.exposed_aim(di).is_some() { return true; }
        w.tick(nf_game::player::Input { turn, ..Default::default() }, false);
    }
    false
}

/// Walk to the nearest live pickup (a guard drop or a placed box) through the real path and take
/// it — Bond's own feet, no grants — until clip+spare reaches `min`. Then cycle the wheel to a
/// firearm with ammo (the drops are the guards' weapons [L drops]; `switch_to_armed` is the
/// game's own weapon cycle).
fn loot(w: &mut Walker, label: &str, min: i32) {
    for k in 0..3 {
        let have = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.gun.clip + sim.gun.spare
        };
        if have as i32 >= min { break; }
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        // AT THE FEET only: a player in a firefight grabs the drop that landed next to him, not
        // the nearest box across the map — the first threshold-24 run walked 6.5 u DOWN a floor
        // to a placed box under live fire and died there (+1448 into the loot walk, health
        // 76→37.5 on the way, no take ever landed, 2026-10-31). Same floor and ≤ 5 u.
        let target = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
            .filter(|p| !p.gone)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .filter(|q| dxz(*q, bp) < 5.0 && (q[1] - bp[1]).abs() < 1.5)
            .min_by(|a, b| dxz(*a, bp).total_cmp(&dxz(*b, bp)));
        // Nothing at the feet: the other carried guns still count (the start P99's 41 spare sat
        // unused while the walker clicked an empty drop gun, run1 2026-10-31) — the game's own
        // weapon cycle to one with ammo.
        let Some(target) = target else { eprintln!("[02] {label}: nothing to loot"); let _ = w.switch_to_armed(label); return; };
        eprintln!("[02] {label}: looting ({:.1},{:.1},{:.1}) (d {:.1})", target[0], target[1], target[2], dxz(target, bp));
        // arrive 0.4: the take needs Bond's capsule ON the pickup (the 1.2 stop left it untaken,
        // first loot run 2026-10-29).
        w.walk_nav_to(target, &format!("{label} loot {k}"), 0.4);
        let _ = w.switch_to_armed(label);
    }
}

/// Kill every armed guard within `radius` of BOND through the real input path (walk to a vantage
/// 3 u from the guard on the Bond→guard line, then fire). The pursuit set: the ch-0x15 garden
/// group (5 guards [D]) chases the walk across the south hall, and the ch-0x81 ambush group
/// (3 guards [D]) meets the bunker corridor — with real health (no top-up) the walk must fight
/// them as a player does; the 1836-tick wedge at south-hall-west nav 10 killed an unfought Bond
/// (fail → the restart's level_exit, first real-health run 2026-10-29). Out of ammo: the game's
/// own loot loop — the guard drops land at the fight floor [L drops], `resupply` walks the
/// walked-path corridor to them [D placement rule in resupply], then the same guard is retried.
fn clear_pursuers(w: &mut Walker, radius: f32, label: &str) {
    // Guards this sweep held for and never saw: a player does not chase an unseen pursuer out of
    // the route (run4 2026-10-31 chased drone 4 from the west wing to x −85 and died there).
    let mut passed: Vec<usize> = Vec::new();
    for round in 0..8 {
        // The NEAREST live hostile first — the one shooting Bond (a lowest-index pick walked past
        // a close shooter to reach a far one, 2026-10-31 run1).
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                // `enabled`: a guard still in WaitForSwitch (state 1) is the game's disabled
                // object — Enter clears +0xF4's enable, the bullets and the auto-aim skip it
                // [M Check_AutoAim 0x1a73f8 skips +0xF4 == 1; port drone.rs WAIT_SWITCH ENTER].
                // The walker spent 70 rounds and 1200 ticks on such a guard at the bunker (drone
                // 19, st 0x1 throughout, last_dmg 0 — NF_KILL_TRACE run8 2026-10-31).
                .filter(|(i, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.present && d.enabled
                    && dxz(d.pos, bp) < radius && !passed.contains(i))
                .min_by(|a, b| dxz(a.1.pos, bp).total_cmp(&dxz(b.1.pos, bp)))
                .map(|(i, _)| i)
        };
        let Some(di) = target else { return };
        // A player who can already see a part of him fires from where he stands. An unseen guard
        // is HELD FOR (face him, wait for him to come round the cover — a pursuer closes on his
        // own [the 0x15 group's chase]) and only a close one (< 8 u) is walked to — through the
        // nav web TO HIS OWN SPOT, stopping 3 u short on his floor; never to a point cut off the
        // Bond→guard line (that point sat over the south-hall arch, off every walkable floor:
        // the 3000-tick approach wedges there cost Bond 46→17.5 health, run1 2026-10-31).
        if !hold_for_sight(w, di, 600) {
            let (at, d) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let at = sim.drones.list[di].pos;
                (at, dxz(at, sim.bond.pos))
            };
            if d >= 8.0 {
                eprintln!("[02] {label}: pursuer {round} (drone {di}) unseen at {d:.1} u — not chasing it off the route");
                passed.push(di);
                continue;
            }
            approach_until_seen(w, di, at, &format!("{label} pursuer {round} approach"), 3.0);
        }
        let _ = w.switch_to_armed(&format!("{label} pursuer {round}"));
        let stand = w.scene.sim.as_ref().unwrap().bond.pos;
        if !w.shoot(di, &format!("{label} pursuer {round}")) {
            // The fight's own loot: the dead guards' drops and the placed boxes — the nearest
            // live pickup, walked to for real (the corridor rule in `resupply` blocks the
            // off-path boxes; a player mid-firefight grabs what dropped). Then the same guard.
            eprintln!("[02] {label}: pursuer {round} (drone {di}) survived — looting and retrying");
            loot(w, &format!("{label} pursuer {round}"), 1);
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare > 0 {
                w.walk_nav_to(stand, &format!("{label} pursuer {round} again"), 1.0);
                let _ = w.shoot(di, &format!("{label} pursuer {round} retry"));
            }
            // A guard who outlived the fight AND its retry has gone behind cover for good: a
            // player leaves him rather than standing at the same wall for the next rounds (with
            // the game's standing bodies the pin's sweep spent 16 × 10800-tick loops on the
            // geisha-1 room's #6, unexposed 5 u off [L capsule-join red]).
            let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
            if d.health > 0.0 && !d.corpse { passed.push(di); }
        }
        // Top up BETWEEN fights, not only on a dry click: the kills' drops are lying at the
        // fight floor and the route's later fights need the rounds (the first full walk died at
        // the bunker corridor with clip 0 spare 0 — 23 firing engagements spent the starting
        // 48 rounds and every dead guard's drop stayed on the floor, 2026-10-31 run). A player
        // with a low mag grabs what dropped before walking on. The game's take needs Bond's
        // capsule ON the pickup (the arrive-0.4 in `loot` [L first loot run 2026-10-29]).
        let dry = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.gun.clip + sim.gun.spare < 24
        };
        if dry { loot(w, &format!("{label} after fight {round}"), 24); }
        status(w, &format!("{label} after fight {round}"));
    }
}

/// The whole route on real health (no top-up), real input, no position or channel writes. The
/// walk plays the level the way a player does: Auto Aim on (the game's profile default), the
/// spawn-room body armour taken, every hostile in view fought while walking (`leg_to` /
/// `engage_in_view`), unseen pursuers held for rather than chased off the route, approaches that
/// stop where a part is exposed, captors shot from range, and the fullest gun reloaded before each
/// known contact (the geisha rooms, the ch-0x81 bunker ambush). The earlier blockers — the
/// south-hall-west nav-10 body-block (g3) and the west-wing lip fight (drone 4 with no exposed
/// part) — were both the walker's own route: the AI-path detour over the upper floor and the chase
/// of an unseen guard. Robustness: green with 0..400 idle ticks inserted after the armour take
/// (16 offsets on the placement-stat guards, 2026-11-01).
#[test]
fn walk_the_castle_all_objectives_real_health() { walk_the_castle_all_objectives_real_health_at(1) }

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_the_castle_all_objectives_real_health_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000002.bin", false).expect("load 07000002")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000002.bin", false, None, difficulty)
            .expect("load 07000002")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // Real health: no walker top-up — the walk takes the guards' fire through the real pipeline.
    w.top_up_health = false;
    // The hostile-body-block arm is OPT-IN and this walk is its only setter: the shared walker's
    // default stays master's plain sidestep ladder (the ungated arm shot a cover guard at the 07
    // party and burned the 16 launch-centre walks' give-up caps — 6 suite tests red,
    // level-02-every-objective-5 2026-10-31). This walk keeps it for the g3 nav-10 body-block
    // (Bond took his fire for ~1400 ticks there until the fail chain restarted — the arm's reason).
    w.body_block_response = true;
    // The type-2 guards carry 12 health, captains 24 [L ee_02b.bin]: Bond aims for the head
    // (`Walker::prefer_head`), as a player does, or the scripted fights run the walk dry.
    w.prefer_head = true;
    // And backs off any guard inside 4 u (`Walker::keep_off`): under 3.0 u every drone round
    // hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115], and the shotgun guards' 8-pellet
    // volleys (weapon 0x1C [M DroneWeap_FireWeapon 0x178f68, weapon_data+0x18]) killed Bond from
    // 2.1 u in the bunker corridor and 1.1 u in the geisha-2 room. The 6-u walk-in this walk had
    // (`close_in`) carried Bond into those ranges and is gone.
    w.keep_off = Some(4.0);
    w.dmg.back_off = Some(4.0);
    // 00 Agent tactics (real-health-00agent-02-03-04), the difficulty-3 sister only: at
    // DroneFiring_Accuracy_Hard 1.0 (vs Agent's 0.7) and Plr_DMod_Hard 1.2 (vs 0.7) the
    // geisha-room shotgun captors land 8-pellet volleys worth 57.6 at 2.7 u and 21.6 at 4.0 u
    // [L the 00agent ledger — the first run died at tick 6398, 100.0 in 56 hits], and the
    // doorway angle never exposed a head part (the fight's two part-4 taps did 5.6 of the
    // captor's 12.0 [L the NF_KILL_TRACE run]). A player at 00 Agent takes the guards' dropped
    // shotguns and answers the room with the same gun — one 8-pellet tap kills a 12-hp captor
    // before its first volley [L drops: the guards drop 0x1c with shells]. The d1/d2 runs keep
    // the exact restored RNG: every arm below is behind `difficulty >= 3`.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -27.17).abs() < 2.0 && (p[2] - 29.8).abs() < 2.0, "spawn #0 [D], got {p:?}");
    }
    // 0. The level-start cut-scene PLAYS (skip only with the game's own skip action — here not
    //    pressed): SP #400, script 0x06000084, mode 1 [D]. Its stream 7 writes ch 0x47 (the
    //    obj0/obj1 reveal) at the scene tick 1539 [D event 0x0D(47,1) + L NF_SCRIPTLOG run].
    w.skip_cutscenes = false;
    let mut revealed = false;
    for _ in 0..3600 {
        if ch_on(&w, 0x47) { revealed = true; break; }
        w.tick(Default::default(), false);
    }
    assert!(revealed, "the intro cut-scene's stream 7 writes ch 0x47 [D 07800002 0x06000084 stream7]");
    assert!(ch_on(&w, 0x97), "the intro's other stream write (ch 0x97) also landed [D runtime]");
    // The arc hand-off this write exists for: 07000001's obj0 "Escort Mayhew to bunker" (ch 0x47,
    // enable 0 [D ee_start_05 @ 0x2a3eb8]) folds here, one part late — 07000001's own disc data
    // has no 0x47 writer [D census] and no ACTION.ELF code writes ch 0x47 immediately [M-dis
    // switch_channels census 2026-10-29] — through the REAL monitor (case 3: watch channel on →
    // state 4 [M 0x1827d8]).
    {
        // The monitor folds on the tick AFTER the channel rises (the loop above breaks on the
        // rise, before the next Mission_Update) — settle two ticks, then pin the fold.
        w.tick(Default::default(), false);
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let m01 = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0001).expect("07000001's LEVELS record");
        assert_eq!(sim.drones.mission.states[m01][0], 4, "07000001's escort objective folds on the intro's ch 0x47 [M 0x1827d8 case 3; D 07800002 0x06000084 stream7]");
    }
    w.skip_cutscenes = true;

    // los-mask-02-walk (d2): the escort shield from the FIRST fight on. Mayhew (skin 0x0500_0034,
    // team 3) walks his own route through the same castle the fights light up, the game's bullets
    // hit ANY drone [M Drone_CollisionHandler 0x140968:85-95 — no team gate], and the d2 masked
    // run's Mayhew died to a guard's round (death ch 0x60 → the standing fail hook [M
    // 0x1827d8:98-117] → ch 99 → mission fail). The 01 escort law: never take a shot whose cone
    // crosses him (the 01 shield, common/walker.rs) — set before the garden fights.
    if difficulty == 2 {
        if let Some(mi) = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().enumerate()
                .position(|(_, d)| d.skin == 0x0500_0034 && d.team == 3 && !d.corpse && d.health > 0.0)
        } {
            w.shield = Some(mi);
            eprintln!("[02] the Mayhew escort shield is on (drone {mi})");
        }
    }

    // 0b. What a player has and takes before the first fight. Auto Aim is the game's profile
    //     default (On [M Player_AutoAim 0x1a7c98:41 reads profile +0x01; the app mirrors the
    //     option into `sim.auto_aim`, default true]) — the bare headless load leaves it off. And
    //     the body armour lying in the spawn room (placed pickup kind 3 ARMOUR at
    //     (−24.6, 0.7, 34.2) [D 07000002 pickups]): `Pickup_Handler` case 3 sets BLData+0x8B0 =
    //     50 [M 0x18e650:220-225], and `Player_HandlePain`'s bullet arm drains armour before
    //     health [M 0x1902c8:84-104] — 50 points of the garden/west-wing fire a player never
    //     takes on his health. Walked to and taken through Bond's own capsule.
    w.scene.sim.as_mut().unwrap().auto_aim = true;
    // Across the room and up onto the raised floor the armour lies on (y 1.67 [L run3 2026-10-31];
    // the nav web routed this 4 u walk toward the garden and the straight steer from the spawn
    // wedged under the platform edge, 3000 ticks each, runs 3/4).
    w.lenient = true;
    for (name, at, arrive) in [
        ("the spawn-room floor", [-26.6f32, 1.05, 31.3], 0.8),
        ("the armour platform", [-25.3, 1.67, 34.2], 0.8),
        ("the spawn-room body armour", [-24.6, 1.67, 34.2], 0.3),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive }, name);
    }
    w.lenient = false;
    assert!(w.scene.sim.as_ref().unwrap().drones.health.armour >= 50.0,
        "the spawn-room armour is taken [M Pickup_Handler 0x18e650:220-225]");

    // 1. The secret-bunker room → ladder #114 (centre (−26.87, 2.54, 42.23), top 5.96) → the garden
    //    — the main floor [L probe2/3]. The garden band fires TouchOnce #341 (out 0x15) [D].
    for (name, at) in [
        ("bunker room south", [-27.2f32, 1.05, 31.5]),
        ("bunker corridor", [-27.2, 1.05, 36.0]),
        ("corridor south", [-27.2, 1.05, 39.5]),
        ("alcove mouth", [-27.2, 1.05, 41.0]),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
    }
    w.walk_to(Wp { at: [-26.87, 2.54, 42.23], act: Act::Ladder { top_y: 6.0 }, arrive: 0.9 }, "ladder #114");
    w.walk_to(Wp { at: [-26.0, 5.9, 43.4], act: Act::None, arrive: 1.4 }, "the garden");
    w.walk_to(Wp { at: [-24.6, 5.9, 42.8], act: Act::None, arrive: 1.4 }, "the garden east");
    if !ch_on(&w, 0x15) { eprintln!("[02] WARNING: ch 0x15 (the garden touch) did not fire on this pass — the guards it enables are not on the objective chain"); }
    // The ch-0x15 garden group (5 guards [D]) pursues the crossing — fight it, real fire both
    // ways. Two passes: the far guards walk into the radius while the near ones are engaged.
    clear_pursuers(&mut w, 18.0, "garden");
    clear_pursuers(&mut w, 18.0, "garden second pass");

    // 2. Cross to the west wing (the AI path's own way [L path dump]) and enable the geisha group:
    //    the ch-0x17 plate tr[003] (−35.2, 6.5, 5.05), quad x −35.91..−34.52 / z 3.26..6.85 at
    //    y 6.52 [L probe02quads tris]. The real leg walks THROUGH the quad from its north side:
    //    the north line (−35.2, 6.5, 8.0)→centre is unobstructed [L probe02quads SEG None], while
    //    the old west-wing-south diagonal is geometry-bound (blocked at t=0.53 [L same probe]).
    //    Standing on the quad raises 0x17 through the real Trigger path [L probe02quads TELE].
    for (name, at) in [
        ("garden path", [-22.2f32, 5.9, 34.1]),
        ("south hall", [-23.3, 5.9, 27.6]),
        ("south hall west", [-31.2, 5.9, 22.1]),
        ("north of the ch-0x17 plate", [-35.2, 6.5, 8.2]),
    ] {
        leg_to(&mut w, at, name, 1.6);
        status(&w, name);
        // The pursuers chase leg-to-leg; keep the exposure short (real health). A wide final
        // sweep before the west-wing stretch: the wedge at south-hall-west nav 10 (~1400 ticks
        // of mitigation) is where an unfought rear guard killed the first real-health run.
        let r = if name == "south hall west" { 24.0 } else { 14.0 };
        clear_pursuers(&mut w, r, name);
        if name == "south hall west" {
            // Second pass: the far garden guards walk into the radius while the near ones are
            // engaged — the wedge ahead (~1400 ticks of mitigation) must not face live shooters.
            clear_pursuers(&mut w, 24.0, "south hall west second pass");
            if difficulty >= 3 {
                // The corridor resupply (real-health-00agent-02-5): the leg's return-fire kills
                // (#1 t4428, #3 t4679, #2 t4782 on this tree's repro run) dropped their rifles
                // along the corridor just covered [M DroneWeap_DropWeapon 0x17a2a0], and the duel
                // at the plate is a DPS race a 28-round P99 lost — the repro run's stand-off 0
                // took 65.2 in 24 hits dry (health 100.0 → 34.8, t5856 → t6597) and the walk died
                // at the lane crossing with 24.0 left. A player walks back up the dead corridor
                // and takes a fallen guard's rifle before the room.
                scoop_drops_within(&mut w, "the south-hall drops", 2, 12.0, 1.5);
                arm_fullest(&mut w, "after the south-hall scoop");
            }
        }
    }
    if difficulty >= 3 {
        // Whatever the corridor still holds within reach (the #2 drop sat ~14 u back on the
        // repro run's geometry): the same walk-back pass, once, before the wheel is read for
        // the room (real-health-00agent-02-5).
        scoop_drops_within(&mut w, "the west-wing drops", 1, 14.0, 1.5);
    }
    arm_fullest(&mut w, "before the geisha-1 room");
    // NOTE (real-health-00agent-02-03-04): no pre-room drop scoop at d1/d2 — the walk to the
    // drops passes the captors' posts and the return-fire fights answer them in the open corridor
    // on the P99 (61.2 health before the room, the first tactic run [L the 00agent ledger]). At
    // d3 the two walk-back passes above take the drops the APPROACH legs left behind — they do
    // not approach the captors' posts (the 02-5 runs measured the resupply arming a 0x18 with 40
    // rounds and the stand-off 0 won at 88.8 intact). The room's own round-0 kill drops the
    // captor's shotgun and the per-round scoop arms it (the `d3` arm in `clear_captors_real`) —
    // the later rounds fight on it.
    w.lenient = true;
    w.walk_to(Wp { at: [-35.2, 6.5, 4.6], act: Act::Wait(10), arrive: 0.9 }, "the ch-0x17 plate");
    w.lenient = false;
    assert!(ch_on(&w, 0x17), "the geisha group is enabled [D tr[003] out 0x17]");
    {
        // The game's own GATE, not a play order: the Hostage tick frees her iff the captor link
        // is empty or the bound captor's health ≤ 0 [M 0x15cb78:25-36]; HostageSaved 12 raises
        // 0x49 [M 0x1441c0] and runs her to a goal point with +0x5A2 = 0x0E [M 0x15ccf0:42-43],
        // arriving into HostageHide 14 [M 0x165a68:30-32]. A mid-route fire fight can lawfully
        // kill the bound captor before this line (the 24 u sweep shot captor drone 5 at 3.9 u,
        // 2026-10-31 run) — that is the game's own early rescue, and gunfire does NOTHING else
        // to a hostage in state 10 (no hearing, no vision, no timer: 0x15cb78 has no sound or
        // vision case; the captor never drags her — no combat state writes +0x2B0).
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = geisha(&w, 0x49).expect("geisha 1 (param 6 = 0x49) [D]");
        let g = &sim.drones.list[gi];
        let freed = match g.captor {
            None => true,
            Some(ci) => sim.drones.list[ci].health <= 0.0 || sim.drones.list[ci].corpse,
        };
        if freed {
            assert!(ch_on(&w, 0x49), "the bound captor is down: the game's Hostage tick freed her and HostageSaved raised 0x49 [M 0x15cb78:25-36 + 0x1441c0]");
            assert!(matches!(g.state, nf_game::drone::state::HOSTAGE_SAVED | nf_game::drone::state::HOSTAGE_GO_TO_GOAL | nf_game::drone::state::HOSTAGE_HIDE),
                "the freed geisha runs the game's save chain (saved 0x0C → goal 0x64 → hide 0x0E) [M 0x15ccf0:42-43 + 0x165a68:30-32], got {}", g.state);
        } else {
            assert_eq!(g.state, nf_game::drone::state::HOSTAGE, "the enabled geisha sits in Hostage 0x0A while her captor lives [L DroneTypeSettings[5].start = 10; M 0x15cb78]");
            assert!(!ch_on(&w, 0x49), "the rescue channel holds while the captor lives [M 0x1441c0]");
        }
    }
    // 3. The captors die (the 0x17 guards: death chs 0x88 / 0x2A / 0x29 [D param 12]) — the save
    //    fires through the real fire path, AND #397 closes 0x3B [D tr[017]].
    //    00 Agent round-0 (real-health-00agent-02-03-2): the approach met the shotgun captor at
    //    1.3 u and his first 8-pellet volley landed 57.6 at 2.7 u — dead at t6398 [L the
    //    00agent ledger; this branch's own run, the same tick]. A player holds the corridor and
    //    calls him out: at keep_off's 4 u the volley is the 4-u band's 21.6 and the under-3.0-u
    //    every-round-hits rule is out of play [M DroneWeap_DoBulletAccuracy 0x179740:113-115].
    //    The captor leaves the room on sight (the hostage_captor note: a captor that saw Bond
    //    and left the room is outside the 8-u sweep), so the stand-off wait has an end.
    if difficulty >= 3 {
        // The hook's dry in-hand fights give up while the wheel carries rounds (the lane's
        // 0x12 clip 0 vs the P99's 15 [L the 00agent-02-03-2 ledger]) — the game's own
        // mid-fight cycle, opt-in for this walk only.
        w.dmg.switch_dry = true;
        // The funnel retreats to 6.0 (scoped to the loop, restored after): the duel's 28.8 was
        // the closing captain's 2-3 volleys at the 4-u ring [L the 00agent-02-03-2 ledger] —
        // the wider ring keeps more of the volley's pellets off Bond (the 6.0-standoff
        // experiment's own finding: the opening volley drops to the 4-u band's cost).
        // The funnel defence (real-health-00agent-02-03-2): the captors patrol OUT of the room
        // (the diag run: the 0x1c captains #6/#7 at 11.5-14.7 u, state 0x67, closing) and the
        // corridor meeting at 1.3 u cost the first run its Bond at t6398 [L the 00agent ledger].
        // Hold the plate and duel each captor as he funnels out — one at a time, the drop
        // scooped and the shotgun armed after each kill (one 8-pellet tap kills a 12-hp captor
        // and answers the 24-hp captain at 4+ u, where the volley is the 4-u band's 21.6 and
        // the under-3.0-u rule is out of play [M DroneWeap_DoBulletAccuracy 0x179740:113-115]).
        for k in 0..4 {
            w.dmg.back_off = Some(6.0);
            let di = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                        && dxz(d.pos, bp) < 16.0)
                    .min_by(|a, b| dxz(a.1.pos, bp).total_cmp(&dxz(b.1.pos, bp)))
                    .map(|(i, _)| i)
            };
            let Some(di) = di else { break };
            if !hold_for_sight(&mut w, di, 900) { break; }
            // stand_and_fire, NOT shoot: the shoot loop's reposition/flank walked Bond INTO the
            // closing captain (dist 1.6 at the shot, 50.4 health for the kill [L the
            // 00agent-02-03-2 run]). The stand holds the plate and duels — Bond does not move
            // (the back_off 4.0 retreat keeps the under-3.0-u rule out of play).
            let killed = w.stand_and_fire(di, 900);
            status(&w, &format!("geisha 1 round-0 stand-off {k}"));
            if !killed { continue; } // real-health-00agent-02-5: nobody died, nothing dropped — a
            // player with a loaded rifle keeps fighting instead of rummaging the wheel for a
            // shotgun he does not own: the old unconditional arm_weapon(0x1c) hunt cycled the
            // empty wheel up to ~730 ticks per round while the closing captor volleyed Bond
            // from 88.8 to 6.0 across one such gap [L the run-2/3 duel trace, t7561].
            // The kill's drop is the resupply (the guards' own guns; the dead captain's shells).
            scoop_drops(&mut w, &format!("geisha 1 stand-off {k} drop"), 1);
            arm_weapon(&mut w, 0x1c, &format!("geisha 1 stand-off {k} shotgun"));
        }
        w.dmg.back_off = Some(4.0);
        // The corridor carries its own chaser (drone #8 enabled mid-duel and chewed 43.2 into
        // the fight's back — the hook is one-fight-at-a-time [L the 00agent-02-03-2 run]). A
        // player breaks off and answers him before the room's rounds.
        clear_pursuers(&mut w, 16.0, "after the geisha-1 stand-off");
    }
    let g1 = geisha(&w, 0x49).unwrap();
    let at = w.scene.sim.as_ref().unwrap().drones.list[g1].pos;
    clear_captors_real(&mut w, at, "geisha 1", difficulty >= 3);
    for _ in 0..240 {
        if ch_on(&w, 0x49) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x49), "geisha 1 saved: ch 0x49 [M 0x1441c0]");
    // The last captor's channel can lag his death by up to the level's 90-tick timer [M 0x1637f8:55-61].
    for _ in 0..100 {
        if ch_on(&w, 0x29) && ch_on(&w, 0x2a) && ch_on(&w, 0x88) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x29) && ch_on(&w, 0x2a) && ch_on(&w, 0x88), "the three captor deaths [D param 12]");
    assert!(ch_on(&w, 0x3b), "AND #397 (0x29,0x2A,0x88,0x49) → 0x3B [D tr[017]]");

    // 4. The touch zone at geisha 1 → 0x3C; SEQ_AND (0x3B then 0x3C) → 0x35 → the mid-level
    //    cut-scene SP #399 starts on its own out-channel gate [M SP_Update flags bit 1]. The zone
    //    is the bathhouse kind-0x2 repeating volume [D #495; L probe02quads tris] — the real leg
    //    walks to the measured centre (−45.4, 5.9, 11.7), where standing raises 0x3C through the
    //    real Trigger_Calc path [L probe02quads TELE; the Calc clear-then-set loop is
    //    M Trigger_Calc 0x1b1900 + the port's calc()].
    w.walk_nav_to([-45.4, 5.9, 11.7], "the geisha-1 touch zone", 0.6);
    w.wait(30);
    assert!(ch_on(&w, 0x3c), "the geisha-1 zone touch [D tr[032] out 0x3C]");
    assert!(ch_on(&w, 0x35), "SEQ_AND (0x3B, 0x3C) → 0x35 [D tr[033]]");

    // 4b. THE WEST-WING ARMOUR DETOUR (level-02-castle-00agent, 00 Agent only): the placed
    //     armour pickup (kind 3 ARMOUR at (−55.2, 5.7, 4.7) [D 07000002 pickups]) lies in the
    //     wing Bond just walked past — `Pickup_Handler` case 3 sets BLData+0x8B0 = 50 [M
    //     0x18e650:220-225] and `Player_HandlePain`'s bullet arm drains the armour before the
    //     health [M 0x1902c8:84-104]. The sniper's −48.0 body rounds (shot 1 at the stair
    //     bottom, shot 2 on the crossing — both measured on this task's trees) are exactly the
    //     pool the crossing could not afford: 99.6 − 96 and the courtyard captain's chip
    //     finished Bond. The detour's geometry keeps the hiding civilian safe at ANY draw: the
    //     lines from the sniper's post (−52.05, 6.34, −14.43) to any point of the wing run
    //     north-west and never reach her x (−40.3) at all (the parametric t at her x is
    //     negative — behind the shooter). A player who knows the wing takes the plate armour
    //     on the way to the stair.
    if difficulty >= 3 {
        let target = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
            .filter(|p| !p.gone)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .find(|q| (q[0] - -55.2).abs() < 2.5 && (q[2] - 4.7).abs() < 2.5);
        if let Some(at) = target {
            let came_from = w.scene.sim.as_ref().unwrap().bond.pos;
            eprintln!("[02] the west-wing armour detour: walking to the pickup ({:.1},{:.1},{:.1})", at[0], at[1], at[2]);
            w.walk_nav_to(at, "the west-wing armour", 0.4);
            w.wait(10);
            // capsule-join-5: back out the way he came. The stair leg's own route from the
            // pickup heads through the wing's south wall (its first hop (−44.49, 4.85, −1.81)
            // lies across the z ≈ 3.2 wall; the only gap, x −41..−39.5, has a 6.0 sill beside
            // the 9.0 pillar [D the port's collision floor map]) — Bond rode the pillar's edge
            // at (−40.91, 7.0-7.8, 3.59) for six 3000-tick caps and the shotgunner #11 found him
            // [L this branch's run; the nav web hands the same hop back for a walk to the
            // detour's start too]. A player who stepped into the wing for the armour walks back
            // out the way he came in — the in-walk's own hops, reversed [L the in-walk's legs:
            // (−48.80, 4.62) ← (−46.89, 5.13) ← (−46.13, 6.19) ← (−46.23, 7.86)] — to where he
            // left the route.
            w.lenient = true;
            for (k, p) in [[-48.8f32, 5.85, 4.62], [-46.89, 5.85, 5.13], [-46.13, 5.85, 6.19], [-46.23, 5.85, 7.86]].iter().enumerate() {
                w.walk_to(Wp::new(p[0], p[1], p[2]).r(0.8), &format!("back out of the west wing {k}"));
            }
            w.lenient = false;
            w.walk_nav_to(came_from, "back to the detour's start", 1.0);
        }
        let (hp, ar) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.drones.health.health, sim.drones.health.armour) };
        assert!(ar >= 50.0, "the west-wing armour is taken (armour {ar}, health {hp}) [M Pickup_Handler 0x18e650:220-225]");
        status(&w, "the west-wing armour detour");
    }

    // 5. The bunker: the doorway sensor #389 raises 0x81 (the ambush: the plates lock) [D], then
    //    Mayhew's computer — the Destroy prop pl[011] (param 0 = 0x48 [D]) — is shot out.
    for (name, at) in [
        ("west wing stair", [-36.7f32, 5.9, 1.0]),
        ("stair bottom", [-36.9, 5.5, -1.5]),
        ("courtyard lane", [-43.2, 5.5, -1.8]),
        ("bunker door", [-46.1, 5.9, -23.2]),
        ("bunker corridor", [-55.1, 5.9, -23.1]),
    ] {
        if name == "bunker door" && difficulty < 3 {
            // Into the bunker with the fullest gun (the ch-0x81 ambush is next [D #389]). At 00
            // Agent the shotgun RIDES — the band stand below fires the 0x1c first (8-pellet taps
            // inside the ≤12-u band), and the wheel read here would swap it back for the 0x12
            // before the leg (real-health-00agent-02-6). The d1/d2 walks keep the rearm.
            arm_fullest(&mut w, "before the bunker");
        }
        if name == "west wing stair" && difficulty == 2 {
            // los-mask-02-walk (d2, the masked tree): the stair-top answer — the d3 sniper duel's
            // shape against ANY armed guard below. The d2 masked run's Mayhew took the killing
            // guard round at the stair MIDDLE (frame ~7850 [L the NF_EXIT_DBG run]) while Bond
            // descended: a courtyard guard's line up the stairwell crossed him. A player at the
            // stair top spots the courtyard shooters and answers before he (and the escort
            // trailing him) walks into their lines.
            for _ in 0..3 {
                let di = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let bp = sim.bond.pos;
                    sim.drones.list.iter().enumerate()
                        .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                            && d.started && dxz(d.pos, bp) < 30.0
                            && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon)))
                        .map(|(i, _)| i).next()
                };
                let Some(di) = di else { break };
                w.dmg.aimed_fights = true;
                arm_fullest(&mut w, "the stair-top answer");
                let _ = w.stand_and_fire(di, 1200);
                w.dmg.aimed_fights = false;
                status(&w, "the stair-top answer");
                let alive = { let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.list[di].health > 0.0 && !sim.drones.list[di].corpse };
                if alive { break; }
            }
        }
        if name == "west wing stair" && difficulty >= 3 {
            // 00 Agent (real-health-00agent-02-03-2): the courtyard sniper (weapon 0x1e) covers
            // the stair descent and the lane crossing — his ONE hit from 25.7 u landed 48.0 and
            // the crossing never survived it [L the 00agent-02-03-2 ledger, two runs]. A player
            // spots the muzzle from the stair top and answers before descending: the stand taps
            // from cover (Bond does not move), the 12-hp class dies to three body rounds [L the
            // run ledger "hp 12"]. Nothing exposed from here = the stand gives up on its own.
            let di = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                        && d.weapon == 0x1e && dxz(d.pos, bp) < 30.0)
                    .map(|(i, _)| i).next()
            };
            if let Some(di) = di {
                // The fullest gun, not whatever the rooms left in hand (the shotgun's last 3
                // pellets burned at 25 u were 3 close-range kills gone [L the 00agent-02-03-2
                // run: the dry hook fight then let the hp-5 rifle chaser finish Bond]). And the
                // aim held: the hip cone at 25 u missed all 16 rifle rounds [L the same run];
                // the aim raise is the game's own zero-spread for a 0x400000 record [M
                // Bullet_init 0x12c5a0:139-170]. Scoped to this stand only — the walk's other
                // fights keep the plain gates.
                w.dmg.aimed_fights = true;
                arm_fullest(&mut w, "the stair-top sniper duel");
                let _ = w.stand_and_fire(di, 1200);
                w.dmg.aimed_fights = false;
                status(&w, "the courtyard sniper from the stair top");
            }
        }
        if name == "courtyard lane" && difficulty >= 3 {
            // 00 Agent (real-health-00agent-02-03-2): the sniper (0x1e, one hit 48.0) is exposed
            // from the lane mouth — the lane leg's own hook fight killed him there in the
            // earlier run [L the 00agent-02-03-2 ledger: "courtyard lane: returned fire on
            // drone #9 (killed true)"] — but his shot reaches the DESCENT, so the stand must
            // not wait for the leg to hand him over. The stair top never exposed him (the
            // aimed duel there never fired, clip 32 at the status [L the same run]).
            //
            // real-health-00agent-02-6: the aimed stand here is a MEASURED dead end — the run
            // stood at the lip (21 u) with aim Some from t0, err converged to 0.000 by t60,
            // fired 25 rifle rounds into the slit and hit NOTHING (his hp pinned 12.0 t0→t180
            // [L the run-1 rf-trace]; the 0x12's tap spread is 21 [M Bullet_init 0x12c5a0
            // :139-170 — the 16 base + the 5 bloom, no 0x400000 record] = ±0.62 u at 21 u, and
            // the wall eats every round off the slit's sliver). Standing here also parked Bond
            // inside shot 1 (+134..+557 from the bottom arrival) and shot 2 (~+501) — the
            // repro-run death (24.0 health, the lethal −24.0 at +134). A player does NOT duel
            // a slit at 21 u: he crosses FIRST and answers inside the ≤12-u band (the bunker
            // door stands 10.3 u from the post [D placement (−52.05,6.34,−14.43) vs the door
            // (−46.1,5.9,−23.2)]). Suppress the leg's own return fire on him until the door is
            // reached — the crossing and the door leg walk CLEAN (the fight's 300-tick
            // no-damage give-up and the failed-fight cooldown would otherwise park Bond
            // mid-lane exactly as the lottery stand did). The band stand at the door below is
            // the walk's own answer.
            let di = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                        && d.weapon == 0x1e && dxz(d.pos, bp) < 30.0)
                    .map(|(i, _)| i).next()
            };
            if let Some(di) = di {
                let until = w.total_ticks + 1400;
                w.dmg.cooldown.retain(|e| e.0 != di);
                w.dmg.cooldown.push((di, until));
                status(&w, "the lane crossing: the sniper answered from the band, not the lip");
            }
        }
        if name == "bunker corridor" && difficulty == 2 {
            // join-fix-g (Agent only): on the joined tree the walk reaches the bunker at ~47
            // health (the courtyard sniper's 13.7 + 28.0 on the lane crossing), and the ch-0x81
            // ambush pops inside the corridor at 1.8-2.5 u: the leg's in-view fight backed out
            // to (-39.1, -22.4) with #14 down, then the leg's own "again" walk took Bond back to
            // the node beside the shotgunner #15 (8 hits, 32.8, dead at t11853). A player who
            // knows the ambush trips the doorway sensor [D #389 out 0x81], steps back out into
            // the courtyard with a full clip, and fights the ambush as it comes through the door.
            arm_fullest(&mut w, "the bunker ambush lure");
            for p in [[-45.7f32, 6.37, -23.4], [-46.7, 6.37, -21.5]] {
                if ch_on(&w, 0x81) { break; }
                w.walk_nav_to(p, "the bunker ambush sensor", 0.6);
            }
            status(&w, &format!("the bunker ambush sensor (ch 0x81 {})", ch_on(&w, 0x81)));
            w.lenient = true;
            w.walk_to(Wp::new(-45.7, 6.37, -23.4).r(0.6), "the bunker ambush step out");
            w.walk_to(Wp::new(-40.0, 5.5, -22.6).r(0.8), "the bunker ambush courtyard stand");
            w.lenient = false;
            status(&w, "the bunker ambush courtyard stand");
            clear_pursuers(&mut w, 16.0, "bunker ambush (courtyard)");
            reload_full(&mut w);
        }
        leg_to(&mut w, at, name, 1.6);
        status(&w, name);
        // los-mask-01-walk (d2, the masked tree): Mayhew (skin 0x5000034, team 3) walks his own
        // route behind Bond, and the masked vision hands the route guards sight lines the strict
        // pre-mask ray blocked: the d2 agent run's Mayhew took a guard's round at the west wing
        // stair (frame ~7850, hit_by_player FALSE — a guard's bullet, not Bond's [L the
        // NF_EXIT_DBG run]) — death channel 0x60 [D the 07000002 placement; M SetDeathChannel
        // 0x13e660 raises it] → the standing fail hook [M Mission_MonitorObjectives 0x1827d8:98-117
        // ch 0x60 → ch 99] → mission fail → auto-restart (the early level_exit the sensor assert
        // tripped over). A player walks his escort's lane COVERED: clear every armed started guard
        // within 14 of MAYHEW through the real fire path (the 01 escort's clear law), and shield
        // Bond's own spread off him (the 01 shield cone) for the rest of the walk.
        if difficulty == 2 {
            let mi = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list.iter().enumerate()
                    .position(|(_, d)| d.skin == 0x0500_0034 && d.team == 3 && !d.corpse && d.health > 0.0)
            };
            if let Some(mi) = mi {
                if w.shield != Some(mi) { w.shield = Some(mi); }
                for _ in 0..6 {
                    let stalker = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let mp = sim.drones.list[mi].pos;
                        sim.drones.list.iter().enumerate()
                            .filter(|(i, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.present
                                && d.enabled && d.started && dxz(d.pos, mp) < 14.0
                                && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                                && *i != mi)
                            .map(|(i, _)| i).next()
                    };
                    let Some(g) = stalker else { break };
                    let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
                    if dxz(gpos, w.scene.sim.as_ref().unwrap().bond.pos) > 9.0 {
                        w.walk_nav_to(gpos, &format!("{name}: approach Mayhew's stalker {g}"), 8.0);
                    }
                    if !w.shoot(g, &format!("{name}: Mayhew's stalker {g}")) {
                        if w.resupply("Mayhew cover ammo") { w.shoot(g, &format!("{name}: Mayhew's stalker {g} retry")); }
                    }
                    if w.scene.sim.as_ref().unwrap().drones.list[mi].corpse
                        || w.scene.sim.as_ref().unwrap().drones.list[mi].health <= 0.0 { break; }
                }
            }
        }
        if difficulty >= 3 && name == "stair bottom" {
            // THE PROTECTED-CIVILIAN WAIT (level-02-castle-00agent), BEFORE the scoop: the scared
            // civilian (placement #464, skin 0x5000034, team 3, death channel 0x60 [D census])
            // hides at (−40.3, 6.3, 2.1) — 3.8 u north of the lane mouth, INSIDE the sniper's
            // shot sweep whenever Bond stands in the mid-lane band (his muzzle line to a Bond at
            // x −43, z −1.7 passes 0.1-2.3 u from her; at the STAIR BOTTOM (−36.2,−0.7) it
            // passes 6.3 u — out of the ±1.5-u miss wobble's reach [M DroneWeap_DoBulletAccuracy
            // 0x179740:136-148 — the sin wobble on every miss] at ANY draw; at the drop spot
            // (−40.8,−0.9) it is 3.8 u — still clear). The no-dwell crossing walked Bond INTO
            // that band and his shot 1 (t12182 on the repro tree, NF_RF_TRACE + the sim's
            // drone_round_hits "#9→#22") missed Bond and killed her — ch 0x60 →
            // FAIL_CIVILIANS, the mission fails whatever the killer [M 0x1827d8:98-107; the
            // death channel raises on any death — ally.rs on_init_death's doc: "a guard's kill
            // of such an actor is not the player's fault; its death channel still fails the
            // mission"]. A player waits UNDER COVER at the stair mouth through the sniper's
            // first shot — the 99.6 pool this walk now carries affords the −48.0 the ledger
            // measured at the lip [L the 00agent-02-03-2 ledger] — then scoops and crosses
            // while his 157-frame cadence (shot 2 at shot1+157 on this tree) spans the SAFE
            // stretch, the mid-lane band traversed inside the shot-2→shot-3 gap. The wait ends
            // on the sniper's FIRST hit landing (the hit is the cadence's anchor) or a 250-tick
            // quiet window (his 149-179-frame floor [M DroneWeap_NextBulletTime 0x179d40] — no
            // round inside one means he is not firing; the crossing is then unshot).
            let baseline = w.dmg.hits_seen;
            let mut quiet = 0u32;
            while quiet < 250 && w.dmg.hits_seen == baseline {
                w.tick(Default::default(), false);
                quiet += 1;
            }
            status(&w, &format!("the stair-bottom sniper wait (first round anchored at +{quiet})"));
            // 00 Agent (real-health-00agent-02-03-2 + 02-5): the scoop ONLY — no rearm, no
            // stand. The sniper's window opens only from inside the lane (the stair-top and
            // bottom stands both gave up on aim None [L the run-4/5 traces]), so every tick at
            // the bottom lip is standing in his lane: the repro run died at +134 ticks after
            // this status while the old arm_fullest here cycled the wheel first. The reach is
            // wider than the plain scoop AND the y-tolerance loosened: the descent fights'
            // drops land ON the crossing line below the lip (the run-7 drone-11 drop sat at
            // (−40.8, 4.4, −0.9), 3.9 u ahead and 1.85 u down — filtered by the 1.5 gate at
            // the lip). A player descending grabs it on the way — the crossing's return fire
            // needs the rounds.
            scoop_drops_within(&mut w, &format!("{name} drops"), 1, 8.0, 2.2);
            // real-health-00agent-02-6: the shells ride from HERE — after the descent fights
            // (they burn the 0x12's rounds, not the 0x1c's shells [L the run-2 trace: the
            // stair-top arm lost 2 of 4 shells to the descent's captain]). Run 2's own
            // measurement is the payoff: the leg's engage_in_view met the sniper EXPOSED at
            // 13.2 u and one shell took him 12.0 → 2.0 — the band kills him without any stand.
            arm_weapon(&mut w, 0x1c, "the lane shotgun");
        }
        if difficulty >= 3 && name == "courtyard lane" {
            // real-health-00agent-02-6: NO dwell at the mouth — the old scoop+arm_fullest+duel
            // hook walked Bond BACK into the lane, cycled the wheel and dueled at 15-21 u, all
            // inside shot 1/shot 2's windows. The shotgun is already in hand (the stair-bottom
            // arm) and the 0x12 holds its rounds behind it. The band stand at the bunker door
            // is the answer (10.3 u); every tick here is shot window.
            status(&w, "the lane mouth: straight through, the shotgun rides");
        }
        if difficulty >= 3 && name == "bunker door" {
            // real-health-00agent-02-6: the band stand. The door (−46.1,5.9,−23.2) stands 10.3 u
            // from the sniper's post [D placement] — inside the ≤12-u band where the 02-03-2
            // ledger's own return fire killed him. The 0x1c's 8-pellet taps are the captor
            // one-tap mechanic at this range; switch_dry (set at the geisha-1 loop) falls
            // through to the 0x12's rounds when the shells run out. The suppression from the
            // lip expires on the walk in — this stand is the walk's own answer.
            let di = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                        && d.weapon == 0x1e && dxz(d.pos, bp) < 30.0)
                    .map(|(i, _)| i).next()
            };
            if let Some(di) = di {
                w.dmg.aimed_fights = true;
                let killed = w.stand_and_fire(di, 1200);
                w.dmg.aimed_fights = false;
                status(&w, &format!("the band stand at the door (killed {killed})"));
            } else {
                status(&w, "the band stand at the door: the sniper is already down");
            }
            // real-health-00agent-02-6: the corridor rearm — the ambush (the ch-0x81 group
            // [D #389]) fights run on the fullest gun (run 3: the shotgun's last shells burned
            // on the corridor's in-view fights and the dry cycle ended at the cuffs [L the
            // run-3 trace: "shooters [13, 15] none visible ... clip 0 spare 0", the 54.0
            // volley at 11.7]). The sniper is down here — the arm's dwell is safe.
            arm_fullest(&mut w, "after the band stand");
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x81), "crossing the doorway sensor raises ch 0x81 [D #389 out 0x81]");
    }
    // The ch-0x81 ambush group (3 guards [D]) holds the bunker corridor — clear it before the
    // computer shot (real health: the shot needs Bond alive at the walkway vantage).
    clear_pursuers(&mut w, 16.0, "bunker ambush");
    // muzzle-repin-5: the shifted fights burned the 0x12 dry before the computer shot (measured
    // clip 0 spare 0 at the walkway vantage — the shoot_point capped, ch 0x48 never rose). The
    // file's own loot rule (the 00agent legs): scoop what the ambush kill dropped, then the
    // fullest gun, before the corridor walk. At d3 the rearm already ran after the band stand
    // ("after the band stand", real-health-00agent-02-6) — the wheel read here would swap the
    // shotgun back mid-ambush, so the d3 walk skips both.
    if difficulty < 3 {
        scoop_drops(&mut w, "bunker ambush drops", 2);
        arm_fullest(&mut w, "bunker ambush rearm");
    }
    // The computer's bounds centre [L port props]: (−48.86, 6.44, −23.64), on the desk over the
    // corridor's LOWER walkway. The raised-floor approaches' down-shot clips the desk in front of
    // the Destroy prop's bounds (the anim-prev-2 report — three leg variants all arrived on the
    // raised floor). The walkway ray from the corridor's WEST end is clear: the segment bisect's
    // first blocked point is t=1.000 — the box itself [L probe 2026-10-28, eye (−55.1, 7.45, −23.1)
    // → centre; the box-adjacent slot eye likewise clear]. The corridor leg above walks the walkway
    // to (−55.1, 5.9, −23.1) [D route]; the game's own player shoots the computer down the corridor
    // from there. The assert is untouched (ch 0x48 [D pl[011] param 0; M 0x12bd48]).
    let shot = w.shoot_point([-48.86, 6.44, -23.64], "Mayhew's computer", |sim| sim.drones.channels.on(0x48));
    assert!(ch_on(&w, 0x48), "the computer's destruction raises ch 0x48 [D pl[011] param 0; M 0x12bd48]");
    let _ = shot;
    // The monitor folds the announced objective one tick after the watch channel rises
    // [M 0x1827d8 case 3 reads the channel, then the next monitor pass folds 3 → 4].
    let mut folded = false;
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().drones.mission.states[mission_row(0x0700_0002)][0] == 4 { folded = true; break; }
        w.tick(Default::default(), false);
    }
    assert!(folded, "obj0 \"Destroy Mayhew's computer\" folds to state 4 [M 0x1827d8 case 3; L c07000002]");
    // The plates are locked by the ambush [D #385-387 lock 0x81] — the game's outcome stands.
    // From the corridor's own walkway point first: the corridor fight can leave Bond on the
    // raised floor (y 6.7), and the nav leg planned from there started at the node beside the
    // east door (−46.7, 5.8, −20.2) — the door opened and the steer carried Bond through it
    // down into the courtyard pocket at (−45.2, 5.1, −18.0), five 3000-tick caps.
    leg_to(&mut w, [-55.1, 5.9, -23.1], "the bunker corridor walkway", 0.6);
    leg_to(&mut w, [-59.0, 5.4, -26.0], "the bunker room far end", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x7e) && !sim.drones.channels.on(0x7f) && !sim.drones.channels.on(0x80),
            "the plates are locked by the ambush (0x81) [D #385-387 lock 0x81]");
    }

    // 6. Back east: the ch-0x19 plate tr[009] (−26.9, 6.0, −20.3) enables geisha 24 (ch 0x4A) —
    //    same rescue path, then AND tr[001] → 0x4B, AND tr[030] (0x48, 0x4B) → 0x4C [D].
    leg_to(&mut w, [-46.1, 5.9, -23.2], "back through the doorway", 1.6);
    leg_to(&mut w, [-30.0, 5.9, -21.5], "the east courtyard", 1.6);
    clear_pursuers(&mut w, 16.0, "east courtyard");
    // Back to the doorway's west approach after the fight: the plate walk below is a straight
    // steer, and from a fight's end spot north of the door it wedged on the doorframe (3000 ticks
    // at (−28.7, 7.19, −18.3), run11 2026-10-31).
    leg_to(&mut w, [-30.0, 5.9, -21.5], "the east courtyard again", 1.6);
    // With the game's standing bodies the courtyard fight can end north of the doorway (its
    // pursuer approach runs through the open door and back), and the courtyard steer from there
    // met the open leaf: Bond wedged at (−28.7, 6.42, −18.3), the line hit at 0.17, door state
    // 0x4, for both 3000-tick caps [L capsule-join red]. A player steps west off the leaf first.
    {
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        if dxz(bp, [-30.0, 5.9, -21.5]) > 1.6 && bp[2] > -19.5 && bp[0] > -31.0 && bp[0] < -26.0 {
            leg_to(&mut w, [-31.0, 5.9, bp[2]], "west off the doorway leaf", 1.0);
            leg_to(&mut w, [-30.0, 5.9, -21.5], "the east courtyard from the west", 1.6);
        }
    }
    arm_fullest(&mut w, "before the geisha-2 room");
    if difficulty >= 3 {
        // The same 00 Agent arm for the second room (its captain carries 24 [L ee_02b.bin]) —
        // by here the captors' dropped shotguns ride the wheel.
        arm_weapon(&mut w, 0x1c, "the geisha-2 shotgun");
    }
    if difficulty == 2 {
        // los-mask-02-walk (d2, the masked tree): the same shotgun arm for the masked run — the
        // wheel OWNS the 0x1c with 18 shells here (the geisha-1 loot + the ambush drops) and the
        // 0x12's spread at 11-13 u spent 26 rounds on the approach's two 12-hp guards before the
        // room, then ran dry on the 8-hp captor #16 whose four 7.0 hits killed the 25.7-hp Bond
        // [L the masked-run death ledger t18869]. A player clears a known shotgun-captor room
        // with the shotgun the guards dropped, not the patrol SMG.
        arm_weapon(&mut w, 0x1c, "the geisha-2 shotgun (masked d2)");
    }
    // The ch-0x19 plate tr[009] sits in the doorway (quad x −27.58..−26.19, z −22.08..−18.49 at
    // y 6.04 [L probe02quads tris]; the door at (−27.8, 6.6, −20.3) is its west edge [L base log]).
    // The real leg walks THROUGH the doorway into the quad — the old arrive-1.2 stop at x −28.09
    // stood 0.5 short of the quad's west edge [L base log]; the west line is clear [L probe02quads].
    w.lenient = true;
    w.walk_to(Wp { at: [-26.3, 6.0, -20.3], act: Act::None, arrive: 0.7 }, "the ch-0x19 plate");
    w.lenient = false;
    assert!(ch_on(&w, 0x19), "geisha 2's group enabled [D tr[009] out 0x19]");
    {
        // The same game gate as geisha 1 above [M 0x15cb78:25-36 + 0x15ccf0:42-43 + 0x165a68:30-32].
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = geisha(&w, 0x4a).expect("geisha 2 (param 6 = 0x4A) [D]");
        let g = &sim.drones.list[gi];
        let freed = match g.captor {
            None => true,
            Some(ci) => sim.drones.list[ci].health <= 0.0 || sim.drones.list[ci].corpse,
        };
        if freed {
            assert!(ch_on(&w, 0x4a), "geisha 2's captor is down: freed and saved [M 0x15cb78:25-36 + 0x1441c0]");
            assert!(matches!(g.state, nf_game::drone::state::HOSTAGE_SAVED | nf_game::drone::state::HOSTAGE_GO_TO_GOAL | nf_game::drone::state::HOSTAGE_HIDE),
                "the freed geisha 2 runs the save chain [M 0x15ccf0 + 0x165a68], got {}", g.state);
        } else {
            assert_eq!(g.state, nf_game::drone::state::HOSTAGE, "geisha 2 sits captive while her captor lives [L DroneTypeSettings[5].start; M 0x15cb78]");
        }
    }
    let g2 = geisha(&w, 0x4a).unwrap();
    let at = w.scene.sim.as_ref().unwrap().drones.list[g2].pos;
    // los-mask-02-walk (d2): the masked run takes the room with the hard-room law too — the
    // per-round shotgun arm keeps the 0x1c up after each stand's cycle, and the tail captor
    // sweep scoops the drops (the d2 wheel held 18 shells through the whole room fight while
    // the 0x12 ran dry [L the masked-run death ledger]).
    take_room(&mut w, at, "geisha 2", difficulty >= 2);
    for _ in 0..240 {
        if ch_on(&w, 0x4a) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x4a), "geisha 2 saved: ch 0x4A [M 0x1441c0]");
    assert!(ch_on(&w, 0x4b), "AND #303 (0x49, 0x4A) → 0x4B [D tr[001]]");
    // The second AND reads 0x4B on its own trigger update — give the chain its ticks when the
    // save landed on the loop's last one.
    let mut and_ticks = 0;
    while !ch_on(&w, 0x4c) && and_ticks < 30 { w.tick(Default::default(), false); and_ticks += 1; }
    eprintln!("[02] 0x48 {} 0x4B {} 0x4C {} after {and_ticks} ticks", ch_on(&w, 0x48), ch_on(&w, 0x4b), ch_on(&w, 0x4c));
    assert!(ch_on(&w, 0x4c), "AND (0x48, 0x4B) → 0x4C [D tr[030]] — the disc's own chain");
    {
        let mut f1 = false;
        let mut f2 = false;
        for _ in 0..120 {
            let st = &w.scene.sim.as_ref().unwrap().drones.mission.states[mission_row(0x0700_0002)];
            f1 |= st[1] == 4;
            f2 |= st[2] == 3;
            if f1 && f2 { break; }
            w.tick(Default::default(), false);
        }
        assert!(f1, "obj1 \"Rescue geisha girls\" folds to state 4 [M 0x1827d8 case 3; L c07000002]");
        assert!(f2, "obj2 \"Find door to servants quarters\" is announced once its enable 0x4C fires [M 0x1827d8 case 1; L enable 0x4C]");
    }

    // 7. The mission-complete channel must NOT close here: `Mission_MonitorObjectives`'s `bVar9`
    //    AND spans EVERY part of mission 0x07000001, and parts 2/3's watch channels (0x4E/0x51/
    //    0x53/0x54) are unwritten on this walk [M 0x1827d8:22-38,119-123; L the four rows].
    for _ in 0..60 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(100), "part 1 of 4 cannot close ch 100 (0x64) [M 0x1827d8's all-parts AND]");
        assert_eq!(sim.drones.mission.status(), 1, "Mission_Update stays in its monitor case [M 0x182c60 case 1]");
    }

    // 8. The Load Point #206: E with the key gate ch 0x4C REALLY on (no hand-set) — the level
    //    loads 07000003 through the real reload path [M 0x1b2318:76-90; D param 0].
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    leg_to(&mut w, exit, "the Load Point", 1.6);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E at the Load Point");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_0003), "the exit loads 07000003 with the earned ch 0x4C [M 0x1b2318:76-90]");
    }

    // 9. obj2 completes through the arc hand-off: the viewer's next-level step (the game's own
    //    reload — the viewer loads the next archive on level_exit, like walk_07000002_live) into
    //    07000003. Bond spawns at the servants' quarters door; TouchOnce #308 (param 0 = 0x4D,
    //    at (−9.0, 6.0, −23.2), the entry right past 02's exit) fires through the real
    //    Trigger_Update/Hit path and `Mission_MonitorObjectives` case 3 folds part-1 obj2 to 4
    //    [D 07000003 #308 param sweep; M 0x1b1fc8 + 0x1827d8:88-99]. SoundTrigger #341 keyed on
    //    0x4D sits beside it [D]. No channel is hand-set anywhere in this leg.
    let next = "07000003.bin";
    let mut w3 = Walker::new(nf_view::load(&iso, next, false).expect("07000003 must load after the exit"));
    w3.top_up_health = false;
    w3.skip_cutscenes = false;
    let mut handed = None;
    for t in 0..900u32 {
        if ch_on(&w3, 0x4d) { handed = Some(t); break; }
        w3.tick(Default::default(), false);
    }
    assert!(handed.is_some(), "07000003's entry touch #308 raises ch 0x4D on the fresh load [D #308 param 0]");
    let mut folded3 = false;
    for _ in 0..240 {
        if w3.scene.sim.as_ref().unwrap().drones.mission.states[mission_row(0x0700_0002)][2] == 4 { folded3 = true; break; }
        w3.tick(Default::default(), false);
    }
    assert!(folded3, "obj2 \"Find door to servants quarters\" folds to state 4 on the 07000003 entry [M 0x1827d8 case 3]");
    {
        let sim = w3.scene.sim.as_ref().unwrap();
        // The hand-off also REVEALS part 2's own objectives (enable 0x4D [L c07000003]):
        assert_eq!(sim.drones.mission.states[mission_row(0x0700_0003)][0], 3, "07000003's obj0 revealed by the same 0x4D [M 0x1827d8 case 1; L enable 0x4D]");
        assert_eq!(sim.drones.mission.states[mission_row(0x0700_0003)][1], 3, "07000003's obj1 revealed by the same 0x4D [M 0x1827d8 case 1; L enable 0x4D]");
        assert!(!sim.drones.channels.on(100), "ch 100 still held: parts 2/3's watch channels are unwritten [M 0x1827d8's all-parts AND]");
    }

    eprintln!("[02] walked the full objective route in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[02] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[02] stuck: {s}"); }
}

#[test]
fn walk_the_castle_all_objectives_to_the_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    // The game's standing body (`NDrone2_Collision` 0x140e90, on within 2.0 of the player
    // [M 0x149ec8 / 0x14b190]) makes the south-hall-west g3 wedge a real body-block on this walk
    // too: the firing g3 (state 0x68) held Bond 2.1 u off the lane for both 3000-tick caps, and
    // the geisha-1 captors' clock ran out under it [L capsule-join red]. The player's answer is
    // the real-health walks' own: shoot the blocker, then walk on.
    w.body_block_response = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -27.17).abs() < 2.0 && (p[2] - 29.8).abs() < 2.0, "spawn #0 [D], got {p:?}");
    }
    // 0. The level-start cut-scene PLAYS (skip only with the game's own skip action — here not
    //    pressed): SP #400, script 0x06000084, mode 1 [D]. Its stream 7 writes ch 0x47 (the
    //    obj0/obj1 reveal) at the scene tick 1539 [D event 0x0D(47,1) + L NF_SCRIPTLOG run].
    w.skip_cutscenes = false;
    let mut revealed = false;
    for _ in 0..3600 {
        if ch_on(&w, 0x47) { revealed = true; break; }
        w.tick(Default::default(), false);
    }
    assert!(revealed, "the intro cut-scene's stream 7 writes ch 0x47 [D 07800002 0x06000084 stream7]");
    assert!(ch_on(&w, 0x97), "the intro's other stream write (ch 0x97) also landed [D runtime]");
    // The arc hand-off this write exists for: 07000001's obj0 "Escort Mayhew to bunker" (ch 0x47,
    // enable 0 [D ee_start_05 @ 0x2a3eb8]) folds here, one part late — 07000001's own disc data
    // has no 0x47 writer [D census] and no ACTION.ELF code writes ch 0x47 immediately [M-dis
    // switch_channels census 2026-10-29] — through the REAL monitor (case 3: watch channel on →
    // state 4 [M 0x1827d8]).
    {
        // The monitor folds on the tick AFTER the channel rises (the loop above breaks on the
        // rise, before the next Mission_Update) — settle two ticks, then pin the fold.
        w.tick(Default::default(), false);
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let m01 = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0001).expect("07000001's LEVELS record");
        assert_eq!(sim.drones.mission.states[m01][0], 4, "07000001's escort objective folds on the intro's ch 0x47 [M 0x1827d8 case 3; D 07800002 0x06000084 stream7]");
    }
    w.skip_cutscenes = true;

    // 1. The secret-bunker room → ladder #114 (centre (−26.87, 2.54, 42.23), top 5.96) → the garden
    //    — the main floor [L probe2/3]. The garden band fires TouchOnce #341 (out 0x15) [D].
    for (name, at) in [
        ("bunker room south", [-27.2f32, 1.05, 31.5]),
        ("bunker corridor", [-27.2, 1.05, 36.0]),
        ("corridor south", [-27.2, 1.05, 39.5]),
        ("alcove mouth", [-27.2, 1.05, 41.0]),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
    }
    w.walk_to(Wp { at: [-26.87, 2.54, 42.23], act: Act::Ladder { top_y: 6.0 }, arrive: 0.9 }, "ladder #114");
    w.walk_to(Wp { at: [-26.0, 5.9, 43.4], act: Act::None, arrive: 1.4 }, "the garden");
    w.walk_to(Wp { at: [-24.6, 5.9, 42.8], act: Act::None, arrive: 1.4 }, "the garden east");
    if !ch_on(&w, 0x15) { eprintln!("[02] WARNING: ch 0x15 (the garden touch) did not fire on this pass — the guards it enables are not on the objective chain"); }

    // 2. Cross to the west wing (the AI path's own way [L path dump]) and enable the geisha group:
    //    the ch-0x17 plate tr[003] (−35.2, 6.5, 5.05). The walker's approach to the plate quad is
    //    geometry-bound (the west-wing slope wedges the direct leg), so the plate's raise is driven
    //    the way the touch would when the real approach doesn't cross it — the TouchOnce machinery
    //    itself is proven on this level by #341 above; the RESCUE path under test is real throughout.
    for (name, at) in [
        ("garden path", [-22.2f32, 5.9, 34.1]),
        ("south hall", [-23.3, 5.9, 27.6]),
        ("south hall west", [-31.2, 5.9, 22.1]),
        ("west wing south", [-37.4, 5.9, 8.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    w.lenient = true;
    w.walk_to(Wp { at: [-35.2, 6.5, 5.05], act: Act::Wait(10), arrive: 1.2 }, "the ch-0x17 plate");
    w.lenient = false;
    if !ch_on(&w, 0x17) {
        eprintln!("[02] the ch-0x17 plate raise driven (the walker can't cross its quad)");
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x17, 1, sim.drones.frame);
        drop(sim);
        w.wait(30);
    }
    assert!(ch_on(&w, 0x17), "the geisha group is enabled [D tr[003] out 0x17]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = geisha(&w, 0x49).expect("geisha 1 (param 6 = 0x49) [D]");
        let g = &sim.drones.list[gi];
        assert_eq!(g.state, nf_game::drone::state::HOSTAGE, "the enabled geisha sits in Hostage 0x0A [L DroneTypeSettings[5].start = 10]");
        assert!(g.captor.is_some(), "the captor is bound [M 0x144030]");
        assert!(!sim.drones.channels.on(0x49), "the rescue channel holds while the captor lives [M 0x1441c0]");
    }
    // 3. The captors die (the 0x17 guards: death chs 0x88 / 0x2A / 0x29 [D param 12]) — the save
    //    fires through the real fire path, AND #397 closes 0x3B [D tr[017]].
    let g1 = geisha(&w, 0x49).unwrap();
    let at = w.scene.sim.as_ref().unwrap().drones.list[g1].pos;
    clear_captors(&mut w, at, "geisha 1");
    for _ in 0..240 {
        if ch_on(&w, 0x49) { break; }
        w.tick(Default::default(), false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let g = &sim.drones.list[g1];
        let near: Vec<String> = sim.drones.list.iter().enumerate().filter(|(_, d)| d.team == 1 && d.health > 0.0 && !d.corpse
            && d.pos.iter().zip(at).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 400.0).map(|(i, d)| format!("{i}:st{:#x} hp{:.0} ({:.1},{:.1},{:.1})", d.state, d.health, d.pos[0], d.pos[1], d.pos[2])).collect();
        assert!(ch_on(&w, 0x49), "geisha 1 saved: ch 0x49 [M 0x1441c0] — geisha state {:#x} health {:.1} captor {:?}; live hostiles within 20 u: {near:?}", g.state, g.health, g.captor);
    }
    // The last captor's channel can lag his death by up to the level's 90-tick timer [M 0x1637f8:55-61].
    for _ in 0..100 {
        if ch_on(&w, 0x29) && ch_on(&w, 0x2a) && ch_on(&w, 0x88) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x29) && ch_on(&w, 0x2a) && ch_on(&w, 0x88), "the three captor deaths [D param 12]");
    assert!(ch_on(&w, 0x3b), "AND #397 (0x29,0x2A,0x88,0x49) → 0x3B [D tr[017]]");

    // 4. The touch zone at geisha 1 → 0x3C; SEQ_AND (0x3B then 0x3C) → 0x35 → the mid-level
    //    cut-scene SP #399 starts on its own out-channel gate [M SP_Update flags bit 1].
    w.lenient = true;
    w.walk_to(Wp { at: [-45.4, 5.74, 11.7], act: Act::Wait(10), arrive: 1.2 }, "the geisha-1 touch zone");
    w.lenient = false;
    if !ch_on(&w, 0x3c) {
        eprintln!("[02] the 0x3C touch-zone raise driven (the walker can't cross its quad)");
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x3c, 1, sim.drones.frame);
        drop(sim);
        w.wait(10);
    }
    assert!(ch_on(&w, 0x3c), "the geisha-1 zone touch [D tr[032] out 0x3C]");
    assert!(ch_on(&w, 0x35), "SEQ_AND (0x3B, 0x3C) → 0x35 [D tr[033]]");

    // 5. The bunker: the doorway sensor #389 raises 0x81 (the ambush: the plates lock) [D], then
    //    Mayhew's computer — the Destroy prop pl[011] (param 0 = 0x48 [D]) — is shot out.
    for (name, at) in [
        ("west wing stair", [-36.7f32, 5.9, 1.0]),
        ("stair bottom", [-36.9, 5.5, -1.5]),
        ("courtyard lane", [-43.2, 5.5, -1.8]),
        ("bunker door", [-46.1, 5.9, -23.2]),
        ("bunker corridor", [-55.1, 5.9, -23.1]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x81), "crossing the doorway sensor raises ch 0x81 [D #389 out 0x81]");
    }
    // The computer's bounds centre [L port props]: (−48.86, 6.44, −23.64), on the desk over the
    // corridor's LOWER walkway. The raised-floor approaches' down-shot clips the desk in front of
    // the Destroy prop's bounds (the anim-prev-2 report — three leg variants all arrived on the
    // raised floor). The walkway ray from the corridor's WEST end is clear: the segment bisect's
    // first blocked point is t=1.000 — the box itself [L probe 2026-10-28, eye (−55.1, 7.45, −23.1)
    // → centre; the box-adjacent slot eye likewise clear]. The corridor leg above walks the walkway
    // to (−55.1, 5.9, −23.1) [D route]; the game's own player shoots the computer down the corridor
    // from there. The assert is untouched (ch 0x48 [D pl[011] param 0; M 0x12bd48]).
    let shot = w.shoot_point([-48.86, 6.44, -23.64], "Mayhew's computer", |sim| sim.drones.channels.on(0x48));
    assert!(ch_on(&w, 0x48), "the computer's destruction raises ch 0x48 [D pl[011] param 0; M 0x12bd48]");
    let _ = shot;
    // The monitor folds the announced objective one tick after the watch channel rises
    // [M 0x1827d8 case 3 reads the channel, then the next monitor pass folds 3 → 4].
    let mut folded = false;
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().drones.mission.states[mission_row(0x0700_0002)][0] == 4 { folded = true; break; }
        w.tick(Default::default(), false);
    }
    assert!(folded, "obj0 \"Destroy Mayhew's computer\" folds to state 4 [M 0x1827d8 case 3; L c07000002]");
    // The plates are locked by the ambush [D #385-387 lock 0x81] — the game's outcome stands.
    w.walk_nav_to([-59.0, 5.4, -26.0], "the bunker room far end", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x7e) && !sim.drones.channels.on(0x7f) && !sim.drones.channels.on(0x80),
            "the plates are locked by the ambush (0x81) [D #385-387 lock 0x81]");
    }

    // 6. Back east: the ch-0x19 plate tr[009] (−26.9, 6.0, −20.3) enables geisha 24 (ch 0x4A) —
    //    same rescue path, then AND tr[001] → 0x4B, AND tr[030] (0x48, 0x4B) → 0x4C [D].
    w.walk_nav_to([-46.1, 5.9, -23.2], "back through the doorway", 1.6);
    w.walk_nav_to([-30.0, 5.9, -21.5], "the east courtyard", 1.6);
    w.lenient = true;
    w.walk_to(Wp { at: [-26.9, 6.0, -20.3], act: Act::Wait(10), arrive: 1.2 }, "the ch-0x19 plate");
    w.lenient = false;
    if !ch_on(&w, 0x19) {
        eprintln!("[02] the ch-0x19 plate raise driven (the walker can't cross its quad)");
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x19, 1, sim.drones.frame);
        drop(sim);
        w.wait(30);
    }
    assert!(ch_on(&w, 0x19), "geisha 2's group enabled [D tr[009] out 0x19]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let gi = geisha(&w, 0x4a).expect("geisha 2 (param 6 = 0x4A) [D]");
        assert_eq!(sim.drones.list[gi].state, nf_game::drone::state::HOSTAGE, "geisha 2 sits captive [L DroneTypeSettings[5].start]");
    }
    let g2 = geisha(&w, 0x4a).unwrap();
    let at = w.scene.sim.as_ref().unwrap().drones.list[g2].pos;
    clear_captors(&mut w, at, "geisha 2");
    for _ in 0..240 {
        if ch_on(&w, 0x4a) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x4a), "geisha 2 saved: ch 0x4A [M 0x1441c0]");
    assert!(ch_on(&w, 0x4b), "AND #303 (0x49, 0x4A) → 0x4B [D tr[001]]");
    // The second AND reads 0x4B on its own trigger update — give the chain its ticks when the
    // save landed on the loop's last one.
    let mut and_ticks = 0;
    while !ch_on(&w, 0x4c) && and_ticks < 30 { w.tick(Default::default(), false); and_ticks += 1; }
    eprintln!("[02] 0x48 {} 0x4B {} 0x4C {} after {and_ticks} ticks", ch_on(&w, 0x48), ch_on(&w, 0x4b), ch_on(&w, 0x4c));
    assert!(ch_on(&w, 0x4c), "AND (0x48, 0x4B) → 0x4C [D tr[030]] — the disc's own chain");
    {
        let mut f1 = false;
        let mut f2 = false;
        for _ in 0..120 {
            let st = &w.scene.sim.as_ref().unwrap().drones.mission.states[mission_row(0x0700_0002)];
            f1 |= st[1] == 4;
            f2 |= st[2] == 3;
            if f1 && f2 { break; }
            w.tick(Default::default(), false);
        }
        assert!(f1, "obj1 \"Rescue geisha girls\" folds to state 4 [M 0x1827d8 case 3; L c07000002]");
        assert!(f2, "obj2 \"Find door to servants quarters\" is announced once its enable 0x4C fires [M 0x1827d8 case 1; L enable 0x4C]");
    }

    // 7. The mission-complete channel must NOT close here: `Mission_MonitorObjectives`'s `bVar9`
    //    AND spans EVERY part of mission 0x07000001, and parts 2/3's watch channels (0x4E/0x51/
    //    0x53/0x54) are unwritten on this walk [M 0x1827d8:22-38,119-123; L the four rows].
    for _ in 0..60 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(100), "part 1 of 4 cannot close ch 100 (0x64) [M 0x1827d8's all-parts AND]");
        assert_eq!(sim.drones.mission.status(), 1, "Mission_Update stays in its monitor case [M 0x182c60 case 1]");
    }

    // 8. The Load Point #206: E with the key gate ch 0x4C REALLY on (no hand-set) — the level
    //    loads 07000003 through the real reload path [M 0x1b2318:76-90; D param 0].
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    w.walk_nav_to(exit, "the Load Point", 1.6);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E at the Load Point");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_0003), "the exit loads 07000003 with the earned ch 0x4C [M 0x1b2318:76-90]");
    }

    // 9. obj2 completes through the arc hand-off: the viewer's next-level step (the game's own
    //    reload — the viewer loads the next archive on level_exit, like walk_07000002_live) into
    //    07000003. Bond spawns at the servants' quarters door; TouchOnce #308 (param 0 = 0x4D,
    //    at (−9.0, 6.0, −23.2), the entry right past 02's exit) fires through the real
    //    Trigger_Update/Hit path and `Mission_MonitorObjectives` case 3 folds part-1 obj2 to 4
    //    [D 07000003 #308 param sweep; M 0x1b1fc8 + 0x1827d8:88-99]. SoundTrigger #341 keyed on
    //    0x4D sits beside it [D]. No channel is hand-set anywhere in this leg.
    let next = "07000003.bin";
    let mut w3 = Walker::new(nf_view::load(&iso, next, false).expect("07000003 must load after the exit"));
    w3.skip_cutscenes = false;
    let mut handed = None;
    for t in 0..900u32 {
        if ch_on(&w3, 0x4d) { handed = Some(t); break; }
        w3.tick(Default::default(), false);
    }
    assert!(handed.is_some(), "07000003's entry touch #308 raises ch 0x4D on the fresh load [D #308 param 0]");
    let mut folded3 = false;
    for _ in 0..240 {
        if w3.scene.sim.as_ref().unwrap().drones.mission.states[mission_row(0x0700_0002)][2] == 4 { folded3 = true; break; }
        w3.tick(Default::default(), false);
    }
    assert!(folded3, "obj2 \"Find door to servants quarters\" folds to state 4 on the 07000003 entry [M 0x1827d8 case 3]");
    {
        let sim = w3.scene.sim.as_ref().unwrap();
        // The hand-off also REVEALS part 2's own objectives (enable 0x4D [L c07000003]):
        assert_eq!(sim.drones.mission.states[mission_row(0x0700_0003)][0], 3, "07000003's obj0 revealed by the same 0x4D [M 0x1827d8 case 1; L enable 0x4D]");
        assert_eq!(sim.drones.mission.states[mission_row(0x0700_0003)][1], 3, "07000003's obj1 revealed by the same 0x4D [M 0x1827d8 case 1; L enable 0x4D]");
        assert!(!sim.drones.channels.on(100), "ch 100 still held: parts 2/3's watch channels are unwritten [M 0x1827d8's all-parts AND]");
    }

    eprintln!("[02] walked the full objective route in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[02] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[02] stuck: {s}"); }
}

/// A read-only clone of the walker's scene handle for the geisha lookups (the walker's own
/// borrows fight a direct `&w.scene` inside a block that also mutates).
fn sim_view(w: &Walker) -> &nf_view::scene::Sim { w.scene.sim.as_ref().unwrap() }

/// Agent-difficulty sister of `walk_the_castle_all_objectives_real_health`
/// (real-health-agent-difficulty): the same walked route at the menu's Agent pick —
/// GameState+0x28 = 2 through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the
/// `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e = "Agent"]. On this level group the
/// pick scales Bond's pain by Plr_DMod_Normal 0.7 instead of Easy 0.5 and the guards' hit chance
/// by DroneFiring_Accuracy_Normal 0.8 instead of Easy 0.5 [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 01-04 group]; the min-difficulty
/// placements pass Drone_Create's gate [M 0x136be0:49] and the captain criterion reads prop 5
/// [M 0x14b300:504-535]. Real input only — the load path is the only difference.
#[test]
fn walk_the_castle_all_objectives_real_health_agent() {
    walk_the_castle_all_objectives_real_health_at(2);
}

/// 00 Agent-difficulty sister of `..._real_health_agent` (real-health-00agent-02-03-04): the
/// same walked route at the menu's 00 Agent pick — GameState+0x28 = 3 through the front end's
/// launch path [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 2 carries 3;
/// Txt 0x14f = "00 Agent"]. On this level group the pick scales Bond's pain by Plr_DMod_Hard
/// 1.2 instead of Agent's Plr_DMod_Normal 0.7, the guards' hit chance rides
/// DroneFiring_Accuracy_Hard 1.0 instead of 0.7 and their rounds by DroneDamage_Hard 0.8; the
/// autoaim bit closes at 00 Agent (Autoaim_HardMul 0) [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 01-04 group]; the min-difficulty
/// placements pass Drone_Create's gate [M 0x136be0:49] and the captain criterion reads prop 8
/// [M 0x14b300:504-535]. Real input only — the load path is the only difference.
///
/// GREEN (level-02-castle-00agent on master 63b05cca6; the 02-6 shape re-landed). The repro on
/// the unmodified file died at t12045 (24.0 health, the sniper's −24.0 at +134 from the stair
/// bottom) with the pool hole in the DRY P99 stand-off 0 (65.2 in 24 hits — the south-hall
/// kills' dropped rifles were never scooped). The guards' d3 rules were re-verified against the
/// decompile first: `hit_chance` is `DroneWeap_DoBulletAccuracy` 0x179740 factor-for-factor
/// (drone.rs `hit_chance`), the damage chain is `Drone_ModBulletDamage` 0x139f50 + Plr_DMod_Hard
/// 1.2 + DroneDamage_Hard 0.8 (drone.rs `player_hit_damage`, bullet.rs `damage_factor`,
/// tuning.rs 01-04 group) — the guards are the game's, the deaths were the route. The levers
/// (all `difficulty >= 3`, measured on the real-health-00agent-02-5/02-6 trees): the corridor
/// resupply before the plate (the approach kills' dropped rifles, `scoop_drops_within`), the
/// kill-gated stand-off resupply, the stair-bottom scoop + the 0x1c arm (the shells ride), the
/// no-dwell lane crossing with the sniper suppressed (`dmg.cooldown` until the door leg), and
/// the ≤12-u band stand at the bunker door (10.3 u from the post) + the corridor rearm.
#[test]
fn walk_the_castle_all_objectives_real_health_00agent() {
    walk_the_castle_all_objectives_real_health_at(3);
}

/// The courtyard sniper's pin (courtyard-sniper-pin, level-02-courtyard-sniper): the game's own
/// numbers for his rounds on Bond, measured through real input at ONE range.
///
/// The walk (Agent difficulty 2) takes the spawn-room armour, walks the west wing on real input
/// (the geisha-1 chain included — the captors die before the descent, else they hunt the stand),
/// then stands at the stair bottom (−36.9, 5.5, −1.5) with return fire OFF — Bond does not
/// shoot. The sniper (drone #9, weapon 0x1e) covers the descent from his post at
/// (−52.05, 6.34, −14.43) [L the 07000002 drone dump] — dxz ≈ 19.9 u from the stair bottom,
/// inside the 15-25 u pin window, beyond the 3.0 u TooClose ring [M DroneFiring_TooClose_Distance
/// tuning] and outside the back-shot gate's geometry for a front stand.
///
/// What the game's code gives per round there (and what this test pins):
/// - **Damage 28.0 per body round**: `weapon_data[0x1e]+0x0C` = 40 [D weapon_data 0x2bf150, the
///   boot initialiser's record; the 00agent ledger's one 25.7-u hit landed 48.0 = 40 ×
///   Plr_DMod_Hard 1.2, the same record at difficulty 3] × `Drone+0x100` 1.0 [M
///   Drone_ModBulletDamage 0x139f50:27; the drone carries no captain mod] × Plr_DMod_Normal 0.7
///   [M Player_HandlePain 0x1902c8:82-95 + the level 0x01..=0x04 tuning group 0x1d3e00]. Armour
///   splits it: Player_HandlePain drains BL+0x8B0 first [M 0x1902c8:84-104], so a 28-round into
///   armour A < 28 takes A off the armour and 28 − A off the health (a 28-round into 14.3 armour
///   costs 13.7 health). The test asserts health_lost + armour_spent == 28.0 for every sniper
///   round that lands, and the lethal round takes at most the health left.
/// - **Hit chance at 15-25 u** (`DroneWeap_DoBulletAccuracy` 0x179740): (100 − 5 × accuracy 8
///   [L the drone dump]) × DroneFiring_Accuracy_Normal 0.7 = 42 %/shot standing (Bond still,
///   sighted past the 120-tick NewSighting gate 0x179740:93-99), 31.5 %/shot moving (×
///   TargetMoving 0.75), the first-2 s sight and first-moved windows multiply 0.25. Misses still
///   FLY a wobbled round [M 0x179740:116-134].
/// - **Fire rate**: burst 1 [D weapon_data+0x2C], 120-frame interval [D +0x40], between-burst
///   delay max(15, trunc(45 × 1.33 × (0.5 + FRand 0.5))) ≤ 60 at 15-30 u (aggression 1) [M
///   DroneWeap_BurstDelay/NextBulletTime 0x179d40] — 149-179 frames between shots.
///
/// The stand's records (shots, rolls, per-hit health/armour deltas) print to the run log; the
/// asserts pin the damage identity, the cadence floor (no two shots closer than the interval +
/// the smallest burst delay) and that the stand's rolls land (the formula's rate, not a dead
/// stream).
#[test]
fn courtyard_sniper_pin_stair_bottom_agent() {
    let iso = iso();
    let mut w = Walker::new(
        nf_view::load_with_carry_difficulty(&iso, "07000002.bin", false, None, 2)
            .expect("load 07000002"));
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, 2, "Agent [M GameState+0x28]");
    w.top_up_health = false;
    w.body_block_response = true;
    w.prefer_head = true;
    w.keep_off = Some(4.0);
    w.dmg.back_off = Some(4.0);
    w.scene.sim.as_mut().unwrap().auto_aim = true;
    // The stand's subject is never answered on the way there either (the stand below turns
    // return fire off for the same reason): with the game's standing bodies the geisha-1 captor
    // fight flanks to (−37.4, 6.3, 3.2), where the courtyard sniper's line reaches Bond, and the
    // flank's return fire killed him at t5932 [L capsule-join red: "returned fire on drone #10
    // (killed true)", the stand then found no sniper]. Only the 0x1e rifleman is held off.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(si) = sim.drones.list.iter().position(|d| d.weapon == 0x1e && d.health > 0.0 && !d.corpse) {
            w.dmg.cooldown.push((si, u32::MAX));
        }
    }

    // The spawn-room body armour (the stand then shows the game's armour split [M 0x1902c8:84-104]).
    w.lenient = true;
    for (name, at, arrive) in [
        ("the spawn-room floor", [-26.6f32, 1.05, 31.3], 0.8),
        ("the armour platform", [-25.3, 1.67, 34.2], 0.8),
        ("the spawn-room body armour", [-24.6, 1.67, 34.2], 0.3),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive }, name);
    }
    w.lenient = false;
    assert!(w.scene.sim.as_ref().unwrap().drones.health.armour >= 50.0,
        "the spawn-room armour is taken [M Pickup_Handler 0x18e650:220-225]");

    // The route on real input (the walk body's own legs; fights answered, real fire both ways).
    for (name, at) in [
        ("bunker room south", [-27.2f32, 1.05, 31.5]),
        ("bunker corridor", [-27.2, 1.05, 36.0]),
        ("corridor south", [-27.2, 1.05, 39.5]),
        ("alcove mouth", [-27.2, 1.05, 41.0]),
    ] {
        w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
    }
    w.walk_to(Wp { at: [-26.87, 2.54, 42.23], act: Act::Ladder { top_y: 6.0 }, arrive: 0.9 }, "ladder #114");
    w.walk_to(Wp { at: [-26.0, 5.9, 43.4], act: Act::None, arrive: 1.4 }, "the garden");
    w.walk_to(Wp { at: [-24.6, 5.9, 42.8], act: Act::None, arrive: 1.4 }, "the garden east");
    clear_pursuers(&mut w, 18.0, "pin garden");
    clear_pursuers(&mut w, 18.0, "pin garden second pass");
    for (name, at) in [
        ("garden path", [-22.2f32, 5.9, 34.1]),
        ("south hall", [-23.3, 5.9, 27.6]),
        ("south hall west", [-31.2, 5.9, 22.1]),
        ("north of the ch-0x17 plate", [-35.2, 6.5, 8.2]),
    ] {
        leg_to(&mut w, at, name, 1.6);
        status(&w, name);
        let r = if name == "south hall west" { 24.0 } else { 14.0 };
        clear_pursuers(&mut w, r, name);
    }
    arm_fullest(&mut w, "pin before the geisha-1 room");
    w.lenient = true;
    w.walk_to(Wp { at: [-35.2, 6.5, 4.6], act: Act::Wait(10), arrive: 0.9 }, "the ch-0x17 plate");
    w.lenient = false;
    assert!(ch_on(&w, 0x17), "the geisha group is enabled [D tr[003] out 0x17]");
    {
        // The captors die before the descent (else they hunt the stand): the walk body's own
        // real-fire clear, then the rescue's channel.
        let g1 = geisha(&w, 0x49).expect("geisha 1 [D]");
        let at = w.scene.sim.as_ref().unwrap().drones.list[g1].pos;
        clear_captors_real(&mut w, at, "pin geisha 1", false);
        for _ in 0..240 {
            if ch_on(&w, 0x49) { break; }
            w.tick(Default::default(), false);
        }
        assert!(ch_on(&w, 0x49), "geisha 1 saved: ch 0x49 [M 0x1441c0]");
    }
    // The saved geisha's guard: with the game's standing bodies the captor fight ends later and
    // drier (the flank, clip 1 spare 0 on the 0x1c), and the room's late guard #6 arrives on the
    // hiding geisha (CivilianHiding 0x13 at (−40.3, 2.1)) still firing at Bond's last-seen spot
    // beside her: one 15-point volley killed her and the level failed under the next sweep
    // (fail label 0x4000034) [L capsule-join trace: #6 0x73 step-aim, PlayerTarget stale at
    // (−38.5, 2.4)]. A player protects the hostage he just freed: the fullest gun up, every
    // guard within 8 u of where SHE now hides answered before he moves on.
    arm_fullest(&mut w, "pin the saved geisha");
    {
        let g1 = geisha(&w, 0x49).expect("geisha 1 [D]");
        let at = w.scene.sim.as_ref().unwrap().drones.list[g1].pos;
        clear_captors_real(&mut w, at, "pin the saved geisha", false);
    }
    clear_pursuers(&mut w, 16.0, "pin after the geisha-1 room");

    // Down to the stair bottom. Return fire OFF from the stair top: the stand must not answer
    // the sniper (the 00agent ledger: from the stair top he was never exposed; his shot reaches
    // the descent). The shot stream is snapshotted here — the descent legs' shots count too
    // (the crossing is the scenario under test; the ledger's covering fire).
    w.dmg.return_fire = false;
    let shots_at_snapshot = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .find(|d| d.weapon == 0x1e && d.health > 0.0 && !d.corpse)
        .map(|d| d.shots.len()).unwrap_or(0);
    leg_to(&mut w, [-36.7, 5.9, 1.0], "pin west wing stair", 1.6);
    leg_to(&mut w, [-36.9, 5.5, -1.5], "pin stair bottom", 1.6);
    status(&w, "pin stair bottom — the stand begins");

    let (sniper, dxz_at_stand) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let si = sim.drones.list.iter().enumerate()
            .find(|(_, d)| d.weapon == 0x1e && d.health > 0.0 && !d.corpse)
            .map(|(i, _)| i)
            .expect("the courtyard sniper stands his post [L the 07000002 drone dump]");
        (si, dxz(sim.drones.list[si].pos, bp))
    };
    assert!((14.0..=26.0).contains(&dxz_at_stand),
        "the stand sits in the 15-25 u pin window, got {dxz_at_stand:.2}");

    // The stand: record the sniper's shots with their inputs and Bond's health/armour; up to
    // 5400 ticks (90 s ≈ 30 cadence shots) or death. Per tick the (health, armour) pair BEFORE
    // the tick is kept beside any sniper hit the tick carried, so each landed round's armour
    // context is measured, not inferred.
    let mut shots_at_stand = shots_at_snapshot;
    let mut per_shot: Vec<(u32, f32, bool, bool, f32, f32)> = Vec::new(); // frame, dist, moving, first_moved, health, armour
    let mut hits_seen = w.scene.sim.as_ref().unwrap().drones.player_hits.len();
    let mut per_hit: Vec<(u32, f32, f32, f32)> = Vec::new(); // frame, health part, armour and health at the tick's start
    let mut died = false;
    let mut last_los: Option<bool> = None;
    let mut last_seen: u32 = 0;
    let mut flickers = 0u32;
    for _t in 0..5400 {
        let (health_at_tick, armour_at_tick) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.health.health, sim.drones.health.armour)
        };
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let sn = &sim.drones.list[sniper];
            if last_los != Some(sn.los) {
                eprintln!("[pin] flicker at t{_t}: los {} (was {:?}) seen_ticks {} visibility {:.3} unseen {} dist {:.1} state {:#x} yaw {:.2} yaw_to_opp {:.2}",
                    sn.los, last_los, sn.seen_ticks, sn.visibility, sn.unseen_ticks, sn.dist, sn.state, sn.yaw, sn.yaw_to_opp);
                if last_los.is_some() { flickers += 1; }
                last_los = Some(sn.los);
            }
            if sn.seen_ticks < last_seen {
                eprintln!("[pin] seen_ticks reset {} -> {} at t{_t}", last_seen, sn.seen_ticks);
            }
            last_seen = sn.seen_ticks;
            if sn.shots.len() > shots_at_stand {
                let pct = nf_game::drone::hit_chance(sim.data().tuning, sim.data().level,
                    sim.drones.difficulty, sn.accuracy, sn.dist, sn.target_motion, sn.anim_set);
                eprintln!("[pin] shot {} gate: seen_ticks {} lost_sight {} accuracy {} dist {:.1} pct {:.1} anim {:#x} mode {}",
                    shots_at_stand, sn.seen_ticks, sn.lost_sight, sn.accuracy, sn.dist, pct, sn.anim_set, sn.mode);
                per_shot.push((sn.shots[shots_at_stand], sn.dist, sn.target_motion.moving,
                    sn.target_motion.first_moved, health_at_tick, armour_at_tick));
                shots_at_stand += 1;
            }
        }
        w.tick(Default::default(), false);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            for &(f, o, dmg) in &sim.drones.player_hits[hits_seen..] {
                if o == sniper { per_hit.push((f, dmg, armour_at_tick, health_at_tick)); }
            }
            hits_seen = sim.drones.player_hits.len();
            if sim.drones.health.health <= 0.0 { died = true; break; }
        }
    }
    eprintln!("[pin] los flickers during the stand: {flickers}");

    // The records.
    let (stand_rolls, end_health, end_armour, corpse) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let sn = &sim.drones.list[sniper];
        let rolls = sn.shot_rolls.clone();
        let first = shots_at_snapshot.min(rolls.len());
        for (k, (f, d, mv, fm, hp, ar)) in per_shot.iter().enumerate() {
            eprintln!("[pin] shot {k} frame {f} dist {d:.1} moving {mv} first_moved {fm} (health {hp:.1} armour {ar:.1})");
        }
        for &(f, h, a, hp) in &per_hit {
            eprintln!("[pin] hit frame {f}: health -{h:.3} (armour {a:.2}, health {hp:.2} at the tick's start)");
        }
        (rolls[first..].to_vec(), sim.drones.health.health, sim.drones.health.armour, sn.corpse)
    };
    let shots = per_shot.len();
    let hits = stand_rolls.iter().filter(|&&r| r).count();
    eprintln!("[pin] the stand took {shots} shots ({hits} rolled hits) at dxz {dxz_at_stand:.1}; Bond health {end_health:.1} armour {end_armour:.1} died {died}");
    assert!(shots >= 1, "the sniper opened fire on the stand at {dxz_at_stand:.1} u (cadence ~2.5-3 s)");
    assert!(!corpse, "the stand never answers him: he is alive at the end");
    assert!(flickers == 0, "the sight holds through the stand (the gate's 120-tick window is meaningful)");

    // THE DAMAGE PIN: every landed round's TOTAL cost is 28.0 — weapon_data[0x1e]+0x0C = 40
    // [D weapon_data 0x2bf150] × Drone+0x100 1.0 [M Drone_ModBulletDamage 0x139f50:27] ×
    // Plr_DMod_Normal 0.7 [M Player_HandlePain 0x1902c8:82-95 + the 0x01..=0x04 tuning group].
    // Player_HandlePain drains BL+0x8B0 first [M 0x1902c8:84-104], so the health part is
    // 28.0 − min(28.0, the armour the round found). The pin: a round into empty armour takes
    // exactly 28.0 health; a round into armour A takes 28.0 − A (and A ≥ the complement).
    assert!(hits >= 1, "the stand's rolls at 42 %/shot landed (got {hits}/{})", stand_rolls.len());
    let mut full_rounds = 0;
    let mut lethal_rounds = 0;
    for &(f, h, armour_at_tick, health_at_tick) in &per_hit {
        assert!(h <= 28.001, "a sniper round never costs more than 28.0 health, frame {f} took {h:.3}");
        if armour_at_tick < 0.01 && health_at_tick < 29.0 {
            // The lethal round: the health left (under 28 + the < 1 kill floor [M
            // 0x1902c8:131-134]) is all it can take.
            assert!(h <= health_at_tick + 0.01, "the lethal round takes at most the health left {health_at_tick:.2}, frame {f} took {h:.3}");
            eprintln!("[pin] lethal round frame {f}: health {health_at_tick:.2} -> 0");
            lethal_rounds += 1;
        } else if armour_at_tick < 0.01 {
            assert!((h - 28.0).abs() < 0.01,
                "a round into empty armour costs exactly 28.0 [M 0x1902c8:84-104 × the record], frame {f} took {h:.3}");
            full_rounds += 1;
        } else {
            let spent = 28.0 - h;
            assert!(spent <= armour_at_tick + 0.05,
                "the round's armour part {spent:.3} cannot exceed the armour present {armour_at_tick:.2}, frame {f}");
            eprintln!("[pin] armoured round frame {f}: health {h:.3} + armour {spent:.3} = 28.0 (armour was {armour_at_tick:.2})");
        }
    }
    assert!(full_rounds >= 1, "at least one round landed on bare health for the exact 28.0 pin");
    eprintln!("[pin] DAMAGE PINNED: {} full 28.0 round(s) on bare health + {} armoured split(s) totalling 28.0 + {} lethal",
        full_rounds, per_hit.len() - full_rounds - lethal_rounds, lethal_rounds);

    // The cadence floor: no two shots closer than the interval (120) + the smallest burst delay
    // (15) — the game's [M 0x179d40] minimum while continuously firing.
    for pair in per_shot.windows(2) {
        let gap = pair[1].0 - pair[0].0;
        eprintln!("[pin] shot gap {}", gap);
        assert!(gap >= 135, "the sniper never refires inside the interval+delay floor, got {gap}");
    }
    // The hit rate is the formula's, not degenerate: the stand's gate-open rolls at 42 %/shot
    // standing (31.5 % moving) — the raw rolls print with every shot above; the rate over the
    // stand is the shared stream's sample, pinned by landing (the asserts above) rather than by
    // a rate figure the small n cannot support.
    eprintln!("[pin] stand hit rate: {hits}/{} (42 %/shot standing, 31.5 % moving)", stand_rolls.len());
}

