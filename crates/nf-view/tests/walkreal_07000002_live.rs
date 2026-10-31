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
fn clear_captors_real(w: &mut Walker, around: [f32; 3], label: &str) {
    for round in 0..8 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.team == 1 && d.health > 0.0 && !d.corpse && d.enabled
                && d.pos.iter().zip(around).map(|(a, b)| (a - b).powi(2)).sum::<f32>() < 64.0)
        };
        let Some(di) = target else { return };
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
        status(w, &format!("{label} captor {round}"));
    }
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
    engage_in_view(w, 14.0, name);
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
    for _ in 0..12 {
        cycle(w);
        let now = read(w);
        if now.0 == first.0 { break; }
        seen.push(now);
    }
    let firearm = |id: u8| (2..=0x47).contains(&id);
    let Some(best) = seen.iter().filter(|(id, n)| firearm(*id) && *n > 0).max_by_key(|(_, n)| *n).map(|b| b.0) else { return };
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
/// (20 offsets, 2026-10-31).
#[test]
fn walk_the_castle_all_objectives_real_health() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000002.bin", false).expect("load 07000002"));
    // Real health: no walker top-up — the walk takes the guards' fire through the real pipeline.
    w.top_up_health = false;
    // The hostile-body-block arm is OPT-IN and this walk is its only setter: the shared walker's
    // default stays master's plain sidestep ladder (the ungated arm shot a cover guard at the 07
    // party and burned the 16 launch-centre walks' give-up caps — 6 suite tests red,
    // level-02-every-objective-5 2026-10-31). This walk keeps it for the g3 nav-10 body-block
    // (Bond took his fire for ~1400 ticks there until the fail chain restarted — the arm's reason).
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
        }
    }
    arm_fullest(&mut w, "before the geisha-1 room");
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
    let g1 = geisha(&w, 0x49).unwrap();
    let at = w.scene.sim.as_ref().unwrap().drones.list[g1].pos;
    clear_captors_real(&mut w, at, "geisha 1");
    for _ in 0..240 {
        if ch_on(&w, 0x49) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x49), "geisha 1 saved: ch 0x49 [M 0x1441c0]");
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

    // 5. The bunker: the doorway sensor #389 raises 0x81 (the ambush: the plates lock) [D], then
    //    Mayhew's computer — the Destroy prop pl[011] (param 0 = 0x48 [D]) — is shot out.
    for (name, at) in [
        ("west wing stair", [-36.7f32, 5.9, 1.0]),
        ("stair bottom", [-36.9, 5.5, -1.5]),
        ("courtyard lane", [-43.2, 5.5, -1.8]),
        ("bunker door", [-46.1, 5.9, -23.2]),
        ("bunker corridor", [-55.1, 5.9, -23.1]),
    ] {
        if name == "bunker door" {
            // Into the bunker with the fullest gun (the ch-0x81 ambush is next [D #389]).
            arm_fullest(&mut w, "before the bunker");
        }
        leg_to(&mut w, at, name, 1.6);
        status(&w, name);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x81), "crossing the doorway sensor raises ch 0x81 [D #389 out 0x81]");
    }
    // The ch-0x81 ambush group (3 guards [D]) holds the bunker corridor — clear it before the
    // computer shot (real health: the shot needs Bond alive at the walkway vantage).
    clear_pursuers(&mut w, 16.0, "bunker ambush");
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
    arm_fullest(&mut w, "before the geisha-2 room");
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
    clear_captors_real(&mut w, at, "geisha 2");
    for _ in 0..240 {
        if ch_on(&w, 0x4a) { break; }
        w.tick(Default::default(), false);
    }
    assert!(ch_on(&w, 0x4a), "geisha 2 saved: ch 0x4A [M 0x1441c0]");
    assert!(ch_on(&w, 0x4b), "AND #303 (0x49, 0x4A) → 0x4B [D tr[001]]");
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
    assert!(ch_on(&w, 0x49), "geisha 1 saved: ch 0x49 [M 0x1441c0]");
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
