//! walkreal: 07000003 "Alpine Escape" (Castle C) walked with the player's own movement — no
//! teleports after the spawn. The route (docs/levels/07000003.md + the disc dump):
//!   the 0x24 start (−8.85, 6.44, −23.19; the 0x4d objective-reveal touch fires under Bond) →
//!   the front gate (door[0] at (14.7, 6.6, −35.4) starts locked behind ch 0x31; the E-plate
//!   kind-1 mask-4 at (11.8, 6.0, −35.6) writes 0x31, which unlocks the door and plays the
//!   gate SS #502, ch 0x31, 2 keys — the "key-gated route") → the 0x21 guard-enable plates →
//!   the servants (skin 0x0500002b ×3, team 3, death channel 0x60 = protected) → the east lawn →
//!   the Load Point #451 (18.11, 6.65, 16.05; E, key gate ch 0x52).
//!
//! The objective channels: the disc's mux family writes the gate —
//!   AND(0x4f, 0x50) → 0x51 (obj1 "Rescue Mayhew's servants") and AND(0x4e, 0x51) → 0x52
//!   (obj2's reveal AND the exit's key gate) [D the kind-3 triggers, out 0x51 list 4f,50 and
//!   out 0x52 list 4e,51]. The LEAF writers are on the disc and ported:
//!   - 0x4e — the Dragon Safe key: E on the case (SP #110, script 0x06000563, mask 4 use-key
//!     [M SP_Activate 0x1dc338:82]) spawns the kind-2 pickup with +0x28 = 0x4e [D script stream 1;
//!     M Pickup_Handler 0x18e650:330-333 writes it on take];
//!   - 0x4f/0x50 — the servants' rescue channels, each placement's param 6 = the game's
//!     Drone+0x135 (#389 carries 0x4f, #388 0x50, #422 none [D probe; 07000002's servants carry
//!     0x49/0x4A the same way [D]]): the enable plate (p3) starts the HOSTAGE cower [D
//!     DroneTypeSettings row 5 start 0xa via dtype 0xe + prop 0x4e; M 0x14c938], the captor — the
//!     nearest armed alive guard at the entering tick [M 0x144030; the port's Drones tick] — dying
//!     moves the Hostage UPDATE to HostageSaved [M 0x15cb78:24-36], whose ENTER raises the channel
//!     [M 0x15ccf0 case 1 → DroneFunc_HostageSaved 0x1441c0:11-18].
//!   0x53 (obj2's watch) is written by 07000004's story script [D 07000003-audit] — in this level
//!   obj2 can only reach state 3 (announced), and the mission ENDS on the exit trigger, not on
//!   mission completion. The short route below still hands the leaves at the moments the level
//!   offers (documented stand-in, not the whole-mission walk); the full-route test drives every
//!   leaf through the real input.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The G-055 flush's kill-loot cycle (g055-4, test-local): the fights' drops rearm the walk —
/// the walk-over take [M Pickup_Handler 0x18e650] files the weapon + its clip, then the deep
/// firearm cycle holds a real gun with rounds (the merged-tree real-damage run spent its health
/// in the servants'-ground fights: 88 of 100 across the trio's crossfire before the exit lawn
/// finished Bond at tick 56836).
fn loot_the_drops_03(w: &mut Walker, label: &str) {
    for _ in 0..4 {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let t = sim.drones.pickups.iter().filter(|p| !p.gone)
            .filter(|p| (p.m[3][1] - bp[1]).abs() < 1.5)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)))
            .filter(|t| walker::dxz(*t, bp) < 8.0)
            // A drop Bond can see: with the guards' aim stop (Drone+0x38) #10 died in the
            // east-lane door at (10.1, -19.2) and his gun came to rest under the door leaf
            // (y 5.34); the nav to it wedged 2 x 3000 ticks on the leaf's far side and the
            // servant legs after it never recovered.
            .filter(|t| sim.world.segment([bp[0], bp[1] + 0.9, bp[2]], [t[0], t[1] + 0.3, t[2]]).is_none());
        let Some(t) = t else { break };
        drop(sim);
        w.walk_nav_to(t, &format!("{label} loot"), 0.6);
    }
    for _ in 0..12 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            if (2..=0x47).contains(&g.weapon) && g.weapon != 0x34 && g.weapon != 0x37
                && g.clip + g.spare >= 5 { return; }
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    }
    w.switch_to_armed(label);
}

/// The mission-table row and objective indices for this level [L ee_start_05 @ 0x2a3f18 via
/// mission::c07000003]: obj0 0x4e "Retrieve Dragon Safe contents", obj1 0x51 "Rescue Mayhew's
/// servants", obj2 0x53 "Find door leading to Mayhew" (enable 0x52).
/// Shoot every enabled, alerted hostile within `r` of Bond on his floor, nearest first, through the
/// real fire path. With the placement stats (accuracy / aggression / reaction [M DoModeSettingsNEW
/// 0x14c600:81-107]; bisected: green before that merge, red after) four live guards stood on
/// Bond's next step at (10.4, 6.5, −31.5) in state 0x5C, and the west-lane and east-lane legs
/// pressed into them for whole caps until Bond left the floor. A player clears the guards in his
/// way.
fn clear_near(w: &mut Walker, r: f32, label: &str) {
    for k in 0..8 {
        let near = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.team == 1 && d.enabled && d.present && !d.corpse && d.health > 0.0
                    && d.state != nf_game::drone::state::HOSTAGE && (d.pos[1] - b[1]).abs() < 2.0 && walker::dxz(d.pos, b) < r)
                .min_by(|a, c| walker::dxz(a.1.pos, b).total_cmp(&walker::dxz(c.1.pos, b)))
                .map(|(i, _)| i)
        };
        let Some(di) = near else { return };
        // The far+unexposed skip (g055-4's 16-walk lever): a guard whose boxes no eye ray
        // reaches (the door-8 cluster #18/#19/#2 behind the leaves [D the 00agent-02-03-2
        // note]) is not a fight — the measured d2 run's flank nav walked INTO the doorway's
        // sweep line and #18's point-blank volleys (38.7 in 16 hits, hp 12 — the taps never
        // landed through the door) finished Bond at 58345. The route's own legs engage the
        // cluster through the open doorway (the green runs fought from (36.2,5.7) THROUGH it).
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if walker::dxz(sim.drones.list[di].pos, sim.bond.pos) > 8.0 && w.exposed_aim(di).is_none() { return; }
        }
        let _ = w.switch_to_armed(label);
        if !w.shoot(di, &format!("{label} guard {k}")) { let _ = w.resupply(label); }
    }
}

/// The wedged lane ladder's roof-proof sidestep (real-health-00agent-03-captors, 00 Agent
/// only): the old ladder's `walk_nav_to` sidestep snapped its A* onto the building's upper row
/// (Bond lifted to y 10.12 at (11.19,-29.66)) and the east-wing sniper #6 (26.2,9.9,0.7) saw
/// him along the upper row's windows — one 40.0 round at d 22.8 ended the walk at tick 53672
/// [L the los-mask-repins-03 ledger]. A player in a ditch side-steps ALONG it, on the ground:
/// short DIRECT legs (no A* to ride), each y-guarded — a step that lifts Bond more than 0.9
/// above the sidestep's start y means he is climbing something, so the side is given up and
/// the anchor is skipped (the next anchor re-derives its own path — the route's own doctrine).
/// Returns true when Bond stands >= 1.0 clear of the wedge origin, still on the ground.
fn direct_sidestep_03(w: &mut Walker, side: f32, label: &str) -> bool {
    let start = w.scene.sim.as_ref().unwrap().bond.pos;
    let y0 = start[1];
    let mut cur = start;
    for k in 0..3 {
        let to = [cur[0] + side * 0.9, y0, cur[2]];
        let got = w.walk_to(Wp { at: to, act: Act::None, arrive: 0.5 }, &format!("{label} step {k}"));
        let now = w.scene.sim.as_ref().unwrap().bond.pos;
        if now[1] > y0 + 0.9 {
            eprintln!("[03] {label}: step {k} climbed to y {:.2} — bailing (the roof-proof guard)", now[1]);
            return false;
        }
        cur = now;
        if got == u32::MAX { return false; }
        if walker::dxz(cur, start) >= 1.0 { return true; }
    }
    walker::dxz(cur, start) >= 1.0
}

/// The wake drops' scoop (real-health-00agent-03-captors-6, the 0x23 trio wake block only):
/// the shared loot helper's nav arrives up to ~1.3 u short of a drop (run 35: its nav 2 ended
/// at (24.28,13.19), 1.29 u from drop #13 at (24.84,14.35)) and the walk-over take's radius
/// never reached it — the post-sweep wheel carried NO 0x1c/0x12 entries while both kind-0
/// drops sat unclaimed inside the loot band [L d3run35]. A player walks ONTO the drop: the
/// nav gets Bond through the doorway, then a short DIRECT leg lands on the spot so the
/// walk-over take [M Pickup_Handler 0x18e650] fires.
fn scoop_drops_03(w: &mut Walker, label: &str, r: f32) {
    let mut skip: Vec<[f32; 3]> = Vec::new();
    for _ in 0..4 {
        let t = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            sim.drones.pickups.iter().filter(|p| !p.gone)
                .filter(|p| (p.m[3][1] - bp[1]).abs() < 1.5)
                .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                .filter(|t| !skip.iter().any(|s| walker::dxz(*s, *t) < 0.6))
                .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)))
                .filter(|t| walker::dxz(*t, bp) < r)
        };
        let Some(t) = t else { break };
        w.lenient = true;
        w.walk_nav_to(t, &format!("{label} scoop"), 0.35);
        // The direct finisher only when the nav already brought Bond close: walk_to's own
        // 6000-tick cap panics unconditionally [walker.rs:1381], and an unreachable void-edge
        // drop (run 40/42's west ditch, grounded=false at d 4.6) must never walk it.
        let near = { let sim = w.scene.sim.as_ref().unwrap(); walker::dxz(sim.bond.pos, t) < 2.0 };
        if near { w.walk_to(Wp { at: t, act: Act::None, arrive: 0.3 }, &format!("{label} scoop (direct)")); }
        w.lenient = false;
        // The walk-over take [M Pickup_Handler 0x18e650] clears the pickup on success; a drop
        // the nav cannot reach (the west ditch's void edge — run 40's grounded=false wedge)
        // is skipped for the rest of this scoop so the loop cannot cycle it.
        let still_there = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.pickups.iter().any(|p| !p.gone
                && walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], t) < 0.5)
        };
        if still_there { skip.push(t); }
    }
}

fn mission_row() -> (usize, usize, usize, usize) {
    let li = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0003).expect("the 07000003 mission record");
    (li, 0, 1, 2)
}

/// One STILL-gate aimed stand on drone `di` — the 07000006 file's own idiom, level-local here
/// per the merge rules (shared helpers stay opt-in; this copy runs only inside this file's
/// d2+ shooter sweeps). The parts that make far stands land, each measured on the 0c/06
/// branches:
/// - the aim-RAISE loop: release, let the switch anim finish (switch_state 0), then hold aim
///   until the bit sets — the raise queues only on the aim-PRESSED edge from the idle machine
///   [M Player_Weapon 0x1a24c8:113-152; the finish sets the bit 0x1a2e38:554-561];
/// - the AIM-BIT tap gate: with the bit unset every round rides the HIP cone (spread base 100.0
///   [L ee_start_05.bin @ 0x2c10b8 +0x24] — 4.9 u of scatter at 35 u), with it set the aimed act
///   is zero-spread [M Bullet_init 0x12c5a0:139-170];
/// - the CURSOR-aware convergence: the fired round obeys the view PLUS the eased cursor offsets
///   [M scene.rs free-aim; Player_AutoAim 0x1a7c98:50-59 eases the offsets every aim tick], so
///   the drive targets the cursor position that puts the effective aim on the part;
/// - the STILL gate: tap only after the guard holds still 18 ticks — a stepping guard is not
///   where the view was when the round lands (the round FLIES [M Bullet_update 0x12e7b0]);
/// - the STRICT exposure re-pick every 10 ticks (`DamageLog::exposed_part` — the eye-ray set, THE
///   shared rule this walk's own plain fights fight with) with the 200-tick unseen bail (the 06
///   roof guard's lean-in flicker: a window that lasts ~100 ticks still ends in a tap within 200).
/// Never writes health, armour or ammo; the only inputs are the view drives + the trigger taps.
fn aimed_stand_03(w: &mut Walker, di: usize, label: &str, cap: u32, keep_aim: bool) -> bool {
    // Settle: no input until the switch anim is done, then the raise loop (the 0c idiom).
    for _ in 0..90 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.switch_state == 0 && !sim.gun.fire_state { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    for _ in 0..240 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.aiming { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    const STILL_NEED: u32 = 18;
    let prev = w.dmg.aimed_fights;
    w.dmg.aimed_fights = true;
    eprintln!("[03] the aimed stand ({label}): engaging drone #{di}");
    let killed = {
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
        let mut aim: Option<[f32; 3]> = None;
        let mut hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
        let mut last_hit = 0u32;
        let mut still = 0u32;
        let mut last_pos = [0.0f32; 3];
        let mut have_pos = false;
        for t in 0..cap {
            let (input, fire, reload) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                if d.corpse || d.health <= 0.0 || !d.present { return true; }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 {
                    eprintln!("[03] the aimed stand ({label}): drone #{di} bailed DRY at +{t}");
                    return false;
                }
                // The still gate: root motion only (the 0c measurement — the box sway rides the
                // anim; the root is what carries the chest out of the round's path).
                let moved = have_pos && ((d.pos[0] - last_pos[0]).powi(2)
                    + (d.pos[1] - last_pos[1]).powi(2) + (d.pos[2] - last_pos[2]).powi(2)).sqrt() > 0.03;
                have_pos = true;
                last_pos = d.pos;
                still = if moved { 0 } else { still.saturating_add(1) };
                if d.health < hp0 { hp0 = d.health; last_hit = t; }
                // 1500: the far posts (d 30-40) expose in firing windows — the 500 default
                // aborted before a first hit ever landed on the 06 mouth runs. A landing round
                // kills a guard, so the wait is the tactic, not a stall (the cap still bounds it).
                if t - last_hit > 1500 {
                    eprintln!("[03] the aimed stand ({label}): drone #{di} bailed NO-HIT at +{t} (still {still})");
                    return false;
                }
                if t % 10 == 0 { aim = walker::DamageLog::exposed_part(&sim, di); }
                if aim.is_none() { unseen += 1; if unseen > 200 { eprintln!("[03] the aimed stand ({label}): drone #{di} bailed UNSEEN at +{t}"); return false; } } else { unseen = 0; }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let at = aim.unwrap_or([d.pos[0], d.pos[1] + 0.3, d.pos[2]]);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let desired_yaw = dx.atan2(dz);
                let desired_pitch = (dy / h).atan();
                // The cursor drive + the effective-aim error (the round obeys the view + the
                // eased offsets, not the view alone).
                let yaw_err = { let mut a = desired_yaw - sim.bond.yaw;
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let pitch_err = desired_pitch - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
                let cx_want_raw = -yaw_err / 0.76966;
                let cy_want_raw = pitch_err / 0.57724;
                let cur = sim.cursor;
                let turn = ((cur[0] - cx_want_raw) * 0.76966).clamp(-0.08, 0.08);
                let look = ((cy_want_raw - cur[1]) * 0.57724 / std::f32::consts::FRAC_PI_2).clamp(-0.08, 0.08);
                let aoff = sim.drones.aim_offset;
                let err = { let mut a = yaw_err - aoff[0];
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let perr = pitch_err - aoff[1];
                let reload = sim.gun.clip == 0;
                // The tap: the aim bit ON, the effective aim converged, the guard still.
                let fire = sim.aiming && !reload && unseen == 0 && err.abs() < 0.01 && perr.abs() < 0.01
                    && still >= STILL_NEED && t.saturating_sub(last_tap) >= 8;
                (nf_game::player::Input { turn, look, aiming: true, ..Default::default() }, fire, reload)
            };
            if reload {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, true, false);
                w.total_ticks += 1;
                w.after_step();
                for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
            } else if fire {
                last_tap = t;
                w.tick_fire(input);
            } else {
                w.tick(input, false);
            }
        }
        w.scene.sim.as_ref().unwrap().drones.list[di].health <= 0.0
    };
    w.dmg.aimed_fights = prev;
    // Level the view back (the fights' own idiom) so the next nav leg steers from level pitch.
    // A serial clear keeps the aim up instead (the settle/raise of the next stand break
    // instantly — the window between stands is where the unanswered shooters land).
    if !keep_aim {
        for _ in 0..90 {
            let p = w.scene.sim.as_ref().unwrap().bond.pitch;
            if p.abs() < 0.01 { break; }
            w.tick(nf_game::player::Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[03] the aimed stand ({label}): drone #{di} killed {killed}, health {:.1}", sim.drones.health.health);
    killed
}

/// The 07000006 dock-guard law, level-local here (guard-repin-master-2 item 1): answer a
/// hostile who has hit Bond in the last 600 frames — ANY range to `radius`, the fights'
/// plain 20 u shooters window widened to the guards' real reach — with one aimed stand, up
/// to 8 of them, each target the strict gun rays expose from where Bond stands. The measured
/// red (the agent death at tick 51805 at (23.09, 6.37, 10.54)): the far shooters #11 (13.6,
/// 6.3, −28.5, d 40.2) and #10 (19.4, 6.3, −19.6, d 30.3) plinked the servant/exit legs
/// through glass from OUTSIDE the fights' plain 20 u window — the return fire never answered
/// them and the drain finished Bond (29 hits for 100.0, 12 fights). A player answers whoever
/// is shooting him. [M the glass pass 0x12f048-0x12f058 — their rounds reach through glass;
/// the recent-hitter pick is the 06 `stand_shooters_down` law]
/// Up the lane to the lane doors' south side: the moat node (8.7, −34.1) by the nav, then the
/// lane's own nodes walked direct (the nav's A* from the courtyard picks a line up the west
/// wall instead and wedges at x 6.35 [L the walk logs]).
fn lane_doors_south(w: &mut Walker, label: &str, moat_clear: bool) {
    w.walk_nav_to([8.7, 5.9, -34.1], &format!("{label}: the moat node"), 1.0);
    // 00 Agent (runs 6-8): the machine gun #11 (skin 0x5000038 weapon 0xe, 12 a hit, hp 24)
    // holds (14.5,-29.0) east of the lane and CANNOT be picked from the doorway's south end
    // (his box never passed the eye-ray set there) while his rounds reach Bond in the same
    // spot — the pre-north clear with the doors shut stood Bond in that line and he dealt
    // 76.6 in 7 hits [L d3run7]. A moat-node clear only ENGAGED #17 (nobody else is exposed
    // from the flank seat) and its shots alerted the lane cluster early — the legs ran HOT
    // (#11 5 hits 58.6 + #10 19 hits 34.2 on the legs) and the run died on lane 3 [L d3run8].
    // The legs stay COLD (no shots before the crossing): run 6's cold legs took 1 hit. The
    // moat_clear opt-in stays for a later round's measurement.
    if moat_clear {
        clear_exposed_nearest_03(w, &format!("{label} (the moat clear)"), 40.0, 8, 0, false);
    }
    for (k, at) in [[10.03f32, 5.9, -31.87], [10.26, 5.9, -30.02], [10.59, 5.9, -22.92], [10.62, 5.9, -20.99]].into_iter().enumerate() {
        w.walk_to(Wp { at, act: Act::None, arrive: 0.6 }, &format!("{label}: lane {k}"));
    }
}

/// Cross the lane double doors (leaves at hinges (11.86/9.99, −20.26) [D the door dump]) the
/// way a player does: square up on the doorway's centre line x 10.93 on the near side, E the
/// doors if they are shut, let the swing finish, walk the centre line through. The doorway
/// clears 0.55 on each side of the line only between x 10.6 and 11.3 once both leaves stand
/// open; the nav's diagonal through the doorway runs at the hinge and wedges on the open leaf
/// (Bond held at (11.40, −19.71) for whole legs, and the old stuck ladder rode the door top).
fn cross_lane_doors(w: &mut Walker, south_to_north: bool, label: &str) {
    let (near_z, far_z) = if south_to_north { (-21.25f32, -19.0f32) } else { (-19.0, -21.25) };
    let doors: Vec<usize> = w.scene.sim.as_ref().unwrap().drones.doors.list.iter().enumerate()
        .filter(|(_, d)| walker::dxz(d.pos, [10.93, 6.45, -20.24]) < 1.2).map(|(i, _)| i).collect();
    assert_eq!(doors.len(), 2, "the lane double door");
    w.lenient = true;
    w.walk_to(Wp { at: [10.93, 5.35, near_z], act: Act::None, arrive: 0.35 }, &format!("{label}: the doorway line"));
    w.lenient = false;
    // Wait out a swing already running (an auto-close or a guard's own open).
    for _ in 0..120 {
        let busy = doors.iter().any(|&i| w.scene.sim.as_ref().unwrap().drones.doors.list[i].state & 0x18 != 0);
        if !busy { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    let shut = doors.iter().any(|&i| w.scene.sim.as_ref().unwrap().drones.doors.list[i].state == nf_game::door::CLOSED);
    if shut { w.use_door(doors[0], &format!("{label}: the lane doors")); }
    w.lenient = true;
    w.walk_to(Wp { at: [10.93, 5.35, far_z], act: Act::None, arrive: 0.35 }, &format!("{label}: through the doorway"));
    w.lenient = false;
}

/// Pull the lane double doors shut from the north side (the retune's d3-only lever, called
/// right after the mid-courtyard crossing). `cross_lane_doors` opens them and walks through;
/// the open leaves leave the lane cluster's fire line into the mid courtyard unblocked. The
/// E press toggles the door [M Door_Activate 0x135be0:71-74 — the walker's re-press arm and
/// the CLOSING reversal arm both ride the same toggle], and a press while a leaf swings
/// REVERSES it (use_door's CLOSING arm), so each press is a single edge followed by a
/// wait-out. Bond stays NORTH of the door line the whole time — the first cut of this lever
/// stepped back out of the sweep backward and walked him through the open doorway to the lane
/// side, into the cluster's point-blank band [L the 8575 death on this branch].
fn shut_lane_doors_03(w: &mut Walker, label: &str) {
    let doors: Vec<usize> = w.scene.sim.as_ref().unwrap().drones.doors.list.iter().enumerate()
        .filter(|(_, d)| walker::dxz(d.pos, [10.93, 6.45, -20.24]) < 1.2).map(|(i, _)| i).collect();
    assert_eq!(doors.len(), 2, "the lane double door");
    let mut presses = 0;
    for _cycle in 0..6 {
        // Wait out any swing, then look for an open leaf (state 4 OPEN; 2 CLOSED, 8 OPENING,
        // 0x10 CLOSING [M nf_game::door]). Bond presses from where the crossing left him
        // (1.24 off the door centre — inside the pick's 1.0 refine reach): the press must beat
        // the spacing scan's follower through the open doorway (#10 crossed north in the
        // 9388-run's press window and the fights paid 45.0 for the trap), so no approach walk.
        for _ in 0..150u32 {
            let swinging = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
                .enumerate().any(|(i, d)| doors.contains(&i) && d.state & 0x18 != 0);
            if !swinging { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
        let open = {
            let sim = w.scene.sim.as_ref().unwrap();
            doors.iter().copied().filter(|&i| sim.drones.doors.list[i].state == nf_game::door::OPEN).collect::<Vec<_>>()
        };
        if open.is_empty() { break; }
        // Walk to ~1.9 of the nearest open leaf's live sphere centre, staying north of the
        // door line, aim the use point at it (use_door's aim shape), press ONE edge.
        let di = *open.iter()
            .min_by(|&&a, &&b| {
                let sim = w.scene.sim.as_ref().unwrap();
                let ca = sim.drones.doors.list[a].live_sph().0;
                let cb = sim.drones.doors.list[b].live_sph().0;
                let bp = sim.bond.pos;
                walker::dxz(ca, bp).total_cmp(&walker::dxz(cb, bp))
            }).unwrap();
        for k in 0..120u32 {
            let (c, far, north) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let c = sim.drones.doors.list[di].live_sph().0;
                (c, walker::dxz(c, sim.bond.pos) > 1.85, sim.bond.pos[2] > -20.0)
            };
            if !far && north {
                let sim = w.scene.sim.as_ref().unwrap();
                let to = [c[0] - sim.bond.pos[0], c[1] - (sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop), c[2] - sim.bond.pos[2]];
                let want_pitch = (to[1] / to[0].hypot(to[2]).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                let desired = (c[0] - sim.bond.pos[0]).atan2(c[2] - sim.bond.pos[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                w.tick(nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08), look: (want_pitch - sim.bond.pitch).clamp(-0.06, 0.06),
                    forward: 0.0, ..Default::default()
                }, true);
                presses += 1;
                break;
            }
            let sim = w.scene.sim.as_ref().unwrap();
            let desired = (c[0] - sim.bond.pos[0]).atan2(c[2] - sim.bond.pos[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            // Never walk SOUTH of the door line chasing the open leaf's centre: the first
            // cut's backward step crossed the doorway and met #9 point-blank [L the 8575 run].
            let can_advance = north && sim.bond.pos[2] > -19.8;
            w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if can_advance { 0.5 } else { 0.0 }, ..Default::default() }, false);
        }
    }
    // Let the last swing finish before the next leg.
    for _ in 0..120u32 {
        let swinging = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
            .enumerate().any(|(i, d)| doors.contains(&i) && d.state & 0x18 != 0);
        if !swinging { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let st: Vec<u16> = doors.iter().map(|&i| sim.drones.doors.list[i].state).collect();
        eprintln!("[03] {label}: {presses} presses, leaf states {st:?}");
    }
}

fn stand_shooters_down_03(w: &mut Walker, label: &str, radius: f32) {
    w.switch_to_armed(label);
    for _ in 0..8 {
        let di_opt = {
            let sim = w.scene.sim.as_ref().unwrap();
            let now = sim.drones.frame;
            let b = sim.bond.pos;
            let mut best: Option<usize> = None;
            for i in 0..sim.drones.list.len() {
                let d = &sim.drones.list[i];
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { continue; }
                let d_v = ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt();
                if d_v > radius { continue; }
                let hitting = sim.drones.player_hits.iter().rev()
                    .take_while(|h| now.saturating_sub(h.0) <= 600)
                    .any(|h| h.1 == i);
                if !hitting { continue; }
                if walker::DamageLog::exposed_part(&sim, i).is_none() { continue; }
                best = Some(i);
                break;
            }
            best
        };
        let Some(di) = di_opt else { break };
        aimed_stand_03(w, di, label, 2400, false);
    }
}

/// The d3 lane ledger's lever 1 (runs 4-6): the REACTIVE law (recent-hit gate) misses the worst
/// shooter — the upper-east sniper #6 (skin 0x5000036 weapon 0x1e, 40.0 a hit, hp 12) fires on
/// the courtyard mouth and MISSES until one round lands for the whole remaining health (run 6:
/// the mouth stand at (10.91,-19.34) had no `hitting` candidate — #6's rounds had not landed —
/// the sweep broke instantly, and one 40.0 at d 22.3 from still Bond at 40.0 ended the run at
/// 9585 [L d3run6]; the mid-courtyard plain trades before it had eaten 60.0: #9 22.8 in the
/// north crossing, #8 28.2 [L d3run6]). A player shoots the guard he can SEE before stepping
/// into his field: the pick is the NEAREST EXPOSED enabled guard — no recent-hit gate — and a
/// guard whose stand bailed (NO-HIT/UNSEEN/DRY) is skipped for the rest of this clear so the
/// window cannot cycle him. `wait_budget` ticks are spent WAITING for a first candidate with
/// the aim already held up (the exposure windows flicker — the pick's single snapshot missed
/// #6's lean entirely in run 6); 0 keeps the old one-shot behaviour. `sniper_first` prefers
/// the weapon-0x1e guards in the pick (the 40.0-a-hit shots decide the run: run 11 spent the
/// clip on #10's stand and #6's stand bailed DRY at +418, leaving the sniper alive to fire
/// one 41.2 round at the north courtyard at 15555 [L d3run11]); a pre-stand dry check breaks
/// the clear when no round is left at all. Only this file's d>=3 lane sites call it; every
/// other call site keeps `stand_shooters_down_03` (the pick-order change must not move the
/// green d<3 streams or the other difficulties' sites).
fn clear_exposed_nearest_03(w: &mut Walker, label: &str, radius: f32, max: usize, wait_budget: u32, sniper_first: bool) {
    w.switch_to_armed(label);
    // Raise the aim BEFORE the first pick (the settle/raise inside each stand then break
    // instantly — a tap can land within ~50 ticks of a window opening).
    for _ in 0..240 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.aiming { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    let mut skip: Vec<usize> = Vec::new();
    let mut waited = 0u32;
    for _ in 0..max {
        // DRY: break (run 13's mid-clear resupply walked Bond OFF the stand and cycled his
        // weapon — `cycling weapons (0) (weapon 0x1c clip 0 spare 0)` — and the whole lane
        // fought on the wrong gun; the run died at the moat at 29227 vs run 12's 88424
        // [L d3run13]). The legs' plain clears and the walker's own resupply arm carry ammo
        // instead; the stands just stop when the rounds are gone.
        let di_opt = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            let mut best: Option<(usize, f32)> = None;
            let mut sniper: Option<(usize, f32)> = None;
            for i in 0..sim.drones.list.len() {
                let d = &sim.drones.list[i];
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { continue; }
                if skip.contains(&i) { continue; }
                let dist = walker::dxz(d.pos, b);
                if dist > radius { continue; }
                if walker::DamageLog::exposed_part(&sim, i).is_none() { continue; }
                let nearer = |cur: &Option<(usize, f32)>| match cur { None => true, Some((_, bd)) => dist < *bd };
                if sniper_first && d.weapon == 0x1e {
                    if nearer(&sniper) { sniper = Some((i, dist)); }
                } else if nearer(&best) { best = Some((i, dist)); }
            }
            sniper.or(best).map(|(i, _)| i)
        };
        let Some(di) = di_opt else {
            if waited + 10 <= wait_budget {
                waited += 10;
                for _ in 0..10 { w.tick(nf_game::player::Input { aiming: true, ..Default::default() }, false); }
                continue;
            }
            break;
        };
        if !aimed_stand_03(w, di, label, 2400, true) { skip.push(di); }
    }
    // Level the view back for the next nav leg (the serial clear held the aim throughout).
    for _ in 0..90 {
        let p = w.scene.sim.as_ref().unwrap().bond.pitch;
        if p.abs() < 0.01 { break; }
        w.tick(nf_game::player::Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
    }
}

/// The guards the courtyard plates wake stand in the route with the game's standing bodies
/// (`NDrone2_Collision` 0x140e90's capsule, on within 2.0 of the player [M 0x149ec8 / 0x14b190]):
/// the AimStandFire 0x68 rifleman #9 at (11.0, 6.3, −18.0) held Bond 1.9 u off the mid-courtyard
/// door line for the whole exit leg's 6000-tick cap [L capsule-join red]. A player shoots who
/// stands in his way: every live guard within 5 u is answered before the next leg.
fn clear_blockers_03(w: &mut Walker, label: &str) {
    for _ in 0..6 {
        let g = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                d.team == 1 && d.present && d.enabled && !d.corpse && d.health > 0.0
                    && walker::dxz(d.pos, bp) < 5.0 && (d.pos[1] - bp[1]).abs() < 2.5
            })
        };
        let Some(g) = g else { break };
        if !w.shoot(g, &format!("{label} blocker {g}")) { break; }
    }
}

#[test]
fn walk_the_alpine_route_to_the_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    // capsule-join-5: the woken courtyard guards stand IN the route's doorways (the mid-courtyard
    // door: AimStandFire 0x68 #9 at 1.0 u; the exit-lawn door (22.3, 5.3, 10.9): #3/#4/#5 at
    // ~1 u [L this branch's wedge dumps]) and their standing bodies push Bond off the line
    // [M NDrone2_Collision 0x140e90:91-121 via 0x149ec8/0x14b190]. The walker's opt-in answer to
    // a body wedge is the fight (Walker::body_block_response — the 02/08 lever).
    w.body_block_response = true;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -8.85).abs() < 2.0 && (p[2] - -23.19).abs() < 2.0, "the 0x24 spawn [D], got {p:?}");
        assert!(!sim.disabled, "no intro blocks this level's start");
    }
    w.wait(60);
    {
        // The spawn's objective-reveal touch (out 0x4d at (−9.0,−23.2) [D dump]) — both first
        // objectives' enable channel, so the monitor announces them [M 0x1827d8:55-67].
        let sim = w.scene.sim.as_ref().unwrap();
        let (li, o0, o1, _) = mission_row();
        assert!(sim.drones.channels.on(0x4d), "the spawn reveal touch fires under Bond [D]");
        assert_eq!(sim.drones.mission.states[li][o0], 3, "obj0 announced through the real monitor");
        assert_eq!(sim.drones.mission.states[li][o1], 3, "obj1 announced through the real monitor");
    }

    // The route: north-east across the courtyard to the Load Point. The nav graph fills between
    // the anchors; the guard-enable plates (0x21/0x23/0x24/0x25) fire under Bond's feet.
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("mid courtyard", [13.0, 5.9, -18.0]),
        ("the east lawn", [22.0, 5.9, 5.0]),
        ("the exit lawn", [18.5, 5.9, 13.0]),
    ] {
        clear_blockers_03(&mut w, name);
        w.walk_nav_to(at, name, 1.6);
    }
    clear_blockers_03(&mut w, "the exit approach");

    // 6. The exit: E on the Load Point #451 without ch 0x52 refuses [M Trigger_Activate 0x1b2318:76-90].
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E without the key");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x52), "neither objective fired, so the AND(0x4e,0x51) has not written 0x52 [D]");
        assert_eq!(sim.drones.level_exit, None, "the key gate refuses the exit without ch 0x52 [M 0x1b2318:76-90]");
    }
    // The short route's stand-in: hand the LEAF channels at the moments the level offers — the
    // real writers (the servants' p6 rescue channels and the safe key's take) are driven by the
    // full-route test below; 0x51 and 0x52 are the mux family's OUTPUTS (the AND triggers drive
    // them every tick, so a direct hand is overwritten [M 0x1b1a78's unconditional AND write; the
    // recon]). Everything downstream is the disc's own mux family: AND(0x4f,0x50) writes 0x51
    // [D kind-3 out 0x51 list 4f,50], then AND(0x4e,0x51) writes 0x52 [D out 0x52 list 4e,51],
    // which announces obj2 (enable 0x52) and unlocks the exit.
    let sim = w.scene.sim.as_mut().unwrap();
    for &c in &[0x4eu32, 0x4f, 0x50] { sim.drones.channels.set(c, 1, sim.drones.frame); }
    drop(sim);
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (li, _, o1, o2) = mission_row();
        assert!(sim.drones.channels.on(0x51), "the disc's AND(0x4f,0x50) writes obj1's channel [D]");
        assert!(sim.drones.channels.on(0x52), "the disc's AND(0x4e,0x51) writes the key gate [D]");
        assert_eq!(sim.drones.mission.states[li][o1], 4, "obj1 COMPLETE through the real monitor [M 0x1827d8:78-85]");
        assert_eq!(sim.drones.mission.states[li][o2], 3, "obj2 announced when its enable channel 0x52 fires [M 0x1827d8:55-67]");
    }
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E with the key");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0004), "the exit loads 07000004 through the real reload path");
    eprintln!("[03] walked to the exit in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[03] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[03] stuck: {s}"); }
}

/// The full play: every objective, every leaf written by the disc's own writers under real input
/// (walking, turning, E, real fire) — the servants rescued through their enable plates + captor
/// kills, the Dragon Safe opened and its key taken, the exit fired. No channel or state pokes.
#[test]
fn walk_the_full_route_and_complete_every_objective() { walk_the_full_route_and_complete_every_objective_at(1) }

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_the_full_route_and_complete_every_objective_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000003.bin", false).expect("load 07000003")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000003.bin", false, None, difficulty)
            .expect("load 07000003")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // The type-2 guards carry 12 health, captains 24 [L ee_02b.bin]: Bond aims for the head
    // (`Walker::prefer_head`), as a player does, or the scripted fights run the walk dry.
    w.prefer_head = true;
    // Agent tactics (real-health-agent-difficulty), opt-in for the difficulty-2 sister: at
    // DroneFiring_Accuracy_Normal 0.8 the shotgun (weapon 0x1C) guards close in — under 3.0 u
    // every drone round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] and their 8-pellet
    // volleys cost ~3.3 health a pellet at Plr_DMod_Normal 0.7 (the sister's first run died at
    // tick 40108: 52.1 in 16 hits from a guard at 2.8 u). The 02 walk's keep_off/back_off 4.0
    // (the same `Walker` arms its real-health walk pins green).
    // Difficulty 1 too [master 2ea70f9d2]: the door-8 shotgun closed to 1.9 u and took 48.5 of
    // its real-health 100. fx-muzzle-repin-2 measured: 6.5 changed nothing on the corrected
    // stream (the same t62511 death, the same 60.4 ledger — the fights' stand movement does not
    // read keep_off), so the pinned 4.0 stays and the fight-back lever is the fights' pick, not
    // the stand-off.
    if difficulty >= 1 {
        w.keep_off = Some(4.0);
        w.dmg.back_off = Some(4.0);
    }
    if difficulty >= 3 {
        // 00 Agent (real-health-00agent-02-03-2): the west-lawn fights on drone #8 (weapon 0x12)
        // burned five 900-tick caps at "killed false" while his −1.8 chips bled Bond out at
        // t25850 [L the 00agent ledger; the NF_RF_TRACE run: aim Some, blocked true every
        // sample] — the gun line to #8 crosses a NOT-YET-ENABLED guard (the 0x23/0x24/0x25
        // plates fire later on the route) and the stand never shoots through a drone (the
        // blocker hold). The game's own collide drops Enable(0) objects before the hit test
        // [M Collide_PickObj 0x1eae80; Check_AutoAim 0x1a73f8 skips +0xF4 == 1] — a disabled
        // drone never stops a round — so the hold is not game-shaped there. The opt-in
        // fire_through_disabled drops disabled drones from the line; live hostiles on it are
        // still answered by shoot_blockers (the retarget reads team == 1, so the servants
        // (team 3, death ch 0x60) keep their hold and are never answered to).
        w.dmg.shoot_blockers = true;
        w.dmg.fire_through_disabled = true;
        // The lane's dry in-hand fights (the 0x12 clip 0 vs the P99's 15 in reserve) — the
        // game's own mid-fight cycle [L the 00agent-02-03-2 ledger].
        w.dmg.switch_dry = true;
        // nb-scan-all retune (this branch, the 13566 ledger): the spacing scan walks the ditch
        // machine gun #12 west along the moat while the loot nav stands Bond in his line, and
        // every failed stand on him (the ditch lip blocks the eye ray from the lane floor [L
        // d3run38]) re-fights 240 ticks later — three stands, 43.0 landed, dead at 13566. The
        // escalation (opt-in, the field doc) doubles the wait per failure on the same drone so
        // the response walks on instead of re-standing in the blocked line.
        w.dmg.escalating_cooldowns = true;
    }
    if difficulty >= 2 {
        // guard-repin-master-2, measured on the sweep run's ledger: the fights answered the
        // east-lawn post #4's 26-chip volley (36.7 across the servant-0 navs, ticks 40215-40329)
        // only after fighting the nearer #3/#5 first — the nearest-first pick ignores who is
        // actually draining. The damage-weighted pick (the 08 walks' hurting_first arm, opt-in
        // [M the DamageLog::return_fire pick doc]) answers the biggest current damager first;
        // the visibility gate and the windows are untouched.
        w.dmg.hurting_first = true;
    }
    let (li, o0, o1, o2) = mission_row();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -8.85).abs() < 2.0 && (p[2] - -23.19).abs() < 2.0, "the 0x24 spawn [D], got {p:?}");
    }
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4d), "the spawn reveal [D]");
        assert_eq!(sim.drones.mission.states[li][o0], 3, "obj0 announced [M 0x1827d8:55-67]");
        assert_eq!(sim.drones.mission.states[li][o1], 3, "obj1 announced [M 0x1827d8:55-67]");
    }

    // The servants' identity snapshot, BEFORE any fight: the p6 rescue bytes are the placements'
    // own words [D p6], static until PreDroneControl consumes one [M 0x148db0:51-90] — and the
    // real-damage run's courtyard fight CAN consume 0x4f early: the approach crosses the east
    // 0x21 twin (measured: ch 0x21 at tick 7165), #389 binds a courtyard captor [M 0x144030], and
    // the fight's kills reach that captor (Hostage case 3's captor-death check [M 0x15cb78:24-36]
    // → HostageSaved → switch_channels[+0x135] [M 0x1441c0:11-18]) — the game's own designed save,
    // just earlier than the scripted leg. Sampling after the fights read a consumed byte (0x0)
    // and the old "the servant carrying 0x4f [D p6]" lookup panicked; the rescue loop's live
    // `channels.on(rescue_ch)` check already skips the captor shot for an early rescue.
    let servants: Vec<(usize, [f32; 3], u8)> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s: Vec<(usize, [f32; 3], u8)> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.skin & 0xffff == 0x2b && d.team == 3)
            .map(|(i, d)| (i, d.pos, d.second_channel)).collect();
        for (i, p, ch) in &s {
            let d = &sim.drones.list[*i];
            eprintln!("[03] servant #{i} at ({:.1},{:.1},{:.1}) ch {ch:#x} state {:#x} switch {:?} on {}",
                p[0], p[1], p[2], d.state, d.switch_channel, d.channel_on);
        }
        s
    };
    assert_eq!(servants.len(), 3, "three protected servants [D the dump]");
    // The servants stand IN the east-lane fights' field (servant 1 at (15.4,6.3,−27.0); the
    // lane guards #9/#10 at (18.7,−24.0)/(19.8,−26.1) miss Bond into him) — a crossfire kill
    // raises ch 0x60, the standing fail hook fails the mission and the auto-restart sets
    // level_exit, which latches the walker's stop_on_exit and dead-walks the rest of the route
    // (the fix4 run: "level_exit reached at +0 ticks" from the east-lane resupply on). The
    // walk keeps them alive the way the escort walks keep their allies alive (the walker's own
    // keep_alive arm); every rescue assert below still reads the REAL save path.
    w.keep_alive = servants.iter().map(|(i, _, _)| *i).collect();

    // The route legs up to the servants' ground (the guard-enable plates 0x21/0x23/0x24/0x25 fire
    // under Bond's feet as he crosses them [D the TouchOnce census]).
    w.walk_nav_to([0.0f32, 5.9, -20.0], "courtyard east", 1.6);
    // Into the mid courtyard through the lane doors on their centre line (`cross_lane_doors`).
    lane_doors_south(&mut w, "the lane doors south", false);
    // 00 Agent (runs 6-9): the plain fights stop Bond IN the doorway kill zone — the north
    // crossing's two fights ate 60.0 (#9 22.8, #8 28.2 [L d3run6/9]) and left the whole lane
    // phase a 4.0-hp race. A player sprints the gap and answers from the far side, where the
    // shut doors re-block the cluster's lines (the mouth window's only hits came through
    // glass [L d3run9]); the walker's return-fire switch is the fights' own opt-in, off for
    // the crossing legs only.
    if difficulty >= 3 { w.dmg.return_fire = false; }
    cross_lane_doors(&mut w, true, "mid courtyard");
    // 00 Agent (nb-scan-all retune, measured on this branch): the crossing leaves the lane
    // double doors OPEN behind Bond and the spacing scan walks the lane cluster up the lane to
    // the doorway's south side — the 2026-11-18 run's #9 (skin 0x5000036, 0x1c) fired his
    // point-blank burst THROUGH the open doorway from (12.1,−21.7) at the crossing's far line
    // and the run died at tick 8396 (96.4 in 16 pellet-hits at 2.9 u, 0 return-fire fights —
    // the plain pick answers only a guard who has LANDED hits, and #9's whole approach was
    // misses). Run 9's own doctrine says the shut doors re-block the cluster's lines; nothing
    // shut them. A player pulls the doors shut behind him: the E press toggles the door
    // [M Door_Activate 0x135be0:71-74 reads the door−activator pair; the walker's own
    // re-press arm relies on the same toggle], the leaves cover the doorway span and #9's
    // line from the lane's south side is blocked.
    if difficulty >= 3 { shut_lane_doors_03(&mut w, "the lane doors shut behind"); }
    if difficulty >= 3 { w.dmg.return_fire = true; }
    w.walk_nav_to([13.0, 5.9, -18.0], "mid courtyard", 1.6);
    // guard-repin-master-2 item 1 (the 06 stand_shooters_down law, d-gated), measured on the
    // fatal NF_DMG_TRACE ledger: the courtyard cluster opens on the mid-courtyard leg (#11
    // skin 0x5000038 hp 24 fires at tick 7523 from (14.5,−29.0) d 9.1) and the plain fights
    // need three windows (7931/9419/9952) to take him — during the west-lawn legs he CHARGED
    // to d 2.9 and dealt 56.0, and the walk reached the servant legs at 34.6 where the door
    // ambush (#5 + #3, exactly 34.6 dealt at the (22.3,10.9) door) finished Bond at 51805.
    // From the leg-end anchor (13,−18) neither #10 nor #11 has a reachable box (the sweep's
    // own cand dump: both "hitting true exposed false"), but the fights fought them from the
    // nav's mid-leg stops (10.4..11.5, −20.8) — the ray line needs the courtyard's east lane.
    // A player steps back to the angle he fought from and drops the posts: the walk-back leg
    // re-takes that line, the aimed stands (the aim-bit zero-spread act [M Bullet_init
    // 0x12c5a0:139-170]) take the 5-hp #10 and the 24-hp #11 before the west-lawn legs feed
    // the charge.
    if difficulty == 2 {
        // d3 skips the angle trip (runs 6-9): its fight-line sweep found no candidate at d3
        // and the south crossing re-opened the door cluster's lines for another 22.8 fight;
        // the mouth clear below answers the shooters instead. d2 keeps its measured law.
        cross_lane_doors(&mut w, false, "the mid-courtyard angle");
        w.walk_to(Wp { at: [11.0, 6.5, -21.5], act: Act::None, arrive: 1.0 }, "the mid-courtyard angle (the shooters' fight line)");
        stand_shooters_down_03(&mut w, "the mid-courtyard shooters", 40.0);
    }

    // 1. "Rescue Mayhew's servants" — the leaves 0x4f (servant #389) and 0x50 (#388) are the
    //    placements' param 6 [D probe]. Each servant's enable plate (p3) enables the servant AND
    //    the guards sharing the channel [D], the servant starts the HOSTAGE cower (DroneTypeSettings
    //    row 5 start 0xa) and the captor binds. Killing the captor completes the save: Hostage
    //    UPDATE → HostageSaved ENTER → DroneFunc_HostageSaved writes switch_channels[+0x135 = p6]
    //    [M 0x15cb78:24-36, 0x15ccf0 case 1, 0x1441c0:11-18].
    //    The enable channel 0x21 hangs off TWO kind-1 plates [D two out-0x21 placements]. The east
    //    twin (centre 14.3, −33.3; footprint x 13.1..15.4, z −35.2..−31.5 [D the trigger lo/hi])
    //    is cut off from every standable lane by the garden moat ditch: the straight walk from the
    //    gate-plate lawn burns the 6000-tick cap and the nav route parks on the ledge above (y
    //    9.37 — the prior session's stuck dump). The WEST twin (centre 8.98, −34.35; footprint x
    //    7.27..10.69, z −35.05..−33.65 [D the trigger lo/hi]) IS crossed on real input: the west
    //    lane's approach to the gate-plate lawn stands inside it at (9.8, 6.37, −34.67) and ch 0x21
    //    fires (a plate probe run on the same legs as below). So the plate is fired
    //    BEFORE the servant walk — #389 is already HOSTAGE with its captor bound when Bond arrives
    //    [M 0x144030's bind at the HOSTAGE entering tick].
    let sim = w.scene.sim.as_ref().unwrap();
    let plate0x25 = sim.drones.triggers.iter()
        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x25).map(|t| t.centre())
        .expect("servant #388's enable plate");
    drop(sim);
    // The 0x21 enable: out of the courtyard the way a player must — the LANE (through the
    // double lane doors (11.83/10.02, −20.23) [D the door dump]) south to the moat band's east
    // end, then west along the band. NOT the one-way door[1] (13.96, 6.63, −30.27, approach 1
    // [L the door dump]): its E refuses the courtyard side since the door-direction fix
    // (Door_Activate 0x135be0:71-74 reads door−activator [M]), and the pre-fix route rode that
    // mirrored read — the walk's A* snapped onto the building's upper floor (nav path 3's
    // y-9.49/9.64 row), dropped Bond at the shut door, the wedge ladder stood him on the door
    // top and the diagonal steer across the ditch dropped him through the moat into the void
    // (the red run: Bond at y −62818 by the servant walk, clip dry, captor #11 alive).
    // The courtyard→lane hop is walked DIRECT (the doors are 1.6 u ahead; the stuck handler
    // E-opens them) and lands ON the lane node (10.65,5.81,−21.62): a walk_nav_to from the
    // courtyard start snaps its A* onto the upper row (node (13.52,9.49,−19.06) is dxz 0.9 —
    // nearer than the ground n8) and the whole leg rides the roof. The band legs are safe as
    // nav hops — path 0 spans only the moat band, no upper row to snap to [D the nav dump].
    // The courtyard→lane hop: E on the lane door DIRECTLY (use_door's own aim/press cycle —
    // the doors are approach 0, no refusal [D the door dump]). A plain walk_to from the
    // courtyard start wedges on the closed leaf and its rail/jump ladder throws Bond onto
    // the door top (the fix2 roof detour: "arrived" at y 10.22 while the anchor sat at 5.81 —
    // walk_to's arrive is horizontal), and from the upper row the A* rides the roof to the
    // garden. After the press, use_door walks Bond through the opening (its open-state arm).
    let lane_door = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| walker::dxz(d.pos, [11.83, 6.45, -20.23]) < 0.5)
        .expect("the lane door at (11.83,6.45,-20.23)");
    // 00 Agent (real-health-00agent-03-captors-2, the merged tree): the lane crossing's
    // reactive trade is lethal — #11 (machine gun, 12/a-hit) was killed FROM the doorway
    // line but the fight cost 19.2 (40.0 -> 20.8 at tick 9388) and #17's 2 hits finished
    // Bond at 9872 [L the d3run3 ledger]. The guards are exposed from the doorway line
    // (the lane runs straight south from it) — the aimed stands (zero-spread [M Bullet_init
    // 0x12c5a0:139-170]) kill a 24-hp guard in 1-2 taps where the plain trade ate 19.2,
    // so a player squares up on the mouth line and answers the shooters BEFORE stepping
    // through. d>=3 only: the d<3 streams keep their crossing byte-identical.
    if difficulty >= 3 {
        // real-health-00agent-03-captors-6, run 31 MEASURED (the -5 ledger item 2's "colder"
        // answer, tried and REFUTED): with the pre-crossing stands dropped the mouth runs cold
        // and the crossing itself is lethal — Bond reached the mouth at 65.2 (the
        // mid-courtyard fights' own price) and died at tick 10009 with 100.0 across 9 hits,
        // 5 fights: the fights are OFF across the crossing legs by the run-9 doctrine, so the
        // lane cluster's (#11/#17/#9/#10) fire through the glass ran unanswered [L d3run31].
        // The pre-crossing stands STAY: 48.0 (36.0 #11 in 3 hits, 12.0 #17 in 1) is the
        // measured price of the two kills and a crossing that lands on the lane at ~41 with
        // the lane-north stands still ahead [L d3run29/30 vs d3run31].
        w.walk_to(Wp { at: [10.93, 5.35, -19.0], act: Act::None, arrive: 0.35 },
            "the lane mouth line (pre-crossing)");
        clear_exposed_nearest_03(&mut w, "the lane mouth shooters (pre-crossing)", 40.0, 8, 600, false);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[03-ammo] after the pre-crossing stands: gun {:#x} clip {} spare {} hp {:.1}",
                sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
        }
    }
    let mut on_lane = false;
    // 00 Agent (run 9): the through-crossing's plain fight vs #11 stopped Bond on the lane for
    // the trade — the fights stay off across the crossing like the north one (the door
    // cluster's lines are at Bond's back once he is south of the doors; the lane-north clear
    // below answers what the open gap exposes).
    if difficulty >= 3 { w.dmg.return_fire = false; }
    for attempt in 0..3 {
        w.use_door(lane_door, "the lane doors");
        if attempt > 0 {
            // The door noise draws an awake guard into the doorway (the fix3 run: drone #9,
            // state 0x68, stood at (10.7,6.3,-20.1) and held the open doors the whole 6000-tick
            // leg). A player shoots the blocker — the same arm the 0x25 doorway uses below.
            let blockers: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && d.team != 3
                        && walker::dxz(d.pos, [10.9, 6.4, -20.2]) < 3.0)
                    .map(|(i, _)| i).collect()
            };
            for gi in blockers { w.shoot(gi, "the guard in the lane doorway"); }
        }
        cross_lane_doors(&mut w, false, "the lane");
        w.lenient = true;
        let got = w.walk_to(Wp { at: [10.65, 5.81, -21.62], act: Act::None, arrive: 1.2 }, "the lane north");
        w.lenient = false;
        if got != u32::MAX { on_lane = true; break; }
    }
    if difficulty >= 3 { w.dmg.return_fire = true; }
    assert!(on_lane, "the lane north never reached (the doorway held)");
    // 00 Agent (real-health-00agent-03-captors-2, on the merged tree): the AimStandFire root
    // stop is live on 03 [M drone.rs ZERO_ROOT 0x03] and the guards react to gunfire the
    // game's way, so the crossing's shooters (#11 skin 0x5000038 at (14.5,-29.3) d 8.2
    // machine-gun 12/a-hit, #17 weapon 0xe at (9.5,-34.2) d 13.2, #10 at (20.6,-20.7)) hold
    // their fire lines instead of sliding — the measured run's lane trade cost 40.0 -> 19.0
    // and the walk ended at 9872 (the prior branch reached the mouth at 40.8 [L both ledgers]).
    // A player steps onto the lane's own node and answers the shooters down the lane's length
    // with the aimed stands (the aim-bit zero-spread act [M Bullet_init 0x12c5a0:139-170])
    // BEFORE walking the lane legs.
    if difficulty >= 3 {
        // 00 Agent (real-health-00agent-03-captors-3, run 14's ledger): sniper-first at THIS
        // site starves the lane — after #6's DRY bail the loop's next stand (#10, exposed)
        // burned the last round at +0 and the whole lane fought on clip 3 spare 13 (the walk
        // died at 16392 to #6's 39.4 at the door-8 mouth nav [L d3run14]; run 12's
        // nearest-first north stand broke after #6 alone and reached the servants' rescue with
        // clip 32 [L d3run12]). The #6 kill is the servants'-ground sweep's job instead (below,
        // sniper-first THERE, where the 0x50-plate resupply has topped the clip).
        // Runs 14-16 (the scoop, added after run 16): the stand's DRY bail is the design hole —
        // #6 was exposed at the lane node in 4 of 4 runs (engaged every time) and the clip was
        // empty every time, and his one lean-window round at the si-1 door-8 nav is a 40-class
        // coin flip (run 12 walked that nav at 22.0 hp and LOST nothing to him — the wedge at
        // (11.88,-20.9) held Bond behind the leaves — while runs 14/15/16 walked it smooth and
        // ate 39.4/41.2/41.2 [L d3run12/14/15/16]). The mid-courtyard fights' own kills lie
        // within the loot band of the lane node — #9 died 1.35 u from it (10.8,-22.3), #10
        // 8.8 u (19.7,-21.4) [L the fights lists] — and a player scoops their guns BEFORE
        // answering the lane: the loot's armed-gun cycle returns a clip+spare >= 5 gun, the
        // stands kill #6's 12 hp in 2-4 taps and the lane legs walk on armed.
        w.lenient = true;
        loot_the_drops_03(&mut w, "the lane north drops");
        w.lenient = false;
        // real-health-00agent-03-captors-4, lever 1 (the -3 ledger): #6 was exposed at the lane
        // node in 4 of 4 runs and the clip was empty every time — the stands DRY-bailed on him
        // (run 14) and his one lean-window round at the si-1 door-8 nav is the 40-class coin
        // flip (runs 15/16/17 died at 15555-15776 to it [L d3run15/16/17]). The pickups the
        // resupply arm reaches from the lane node are south of the doors ((14.2,-28.0) d 7.8
        // [L d3run13]) and that corridor is the covered path (run 13's resupply walk survived
        // it — the death there was the CYCLED GUN, not the walk). A player rearms from the
        // covered spot and comes BACK to the angle he fought from: resupply on the walked-path
        // corridor, restore the armed rifle the stands assume (run 13's variant skipped the
        // restore and fought the whole lane on the cycled gun [L d3run13]), re-take the lane
        // node, re-stand. MEASURED this round (runs 18/19): the resupply sprint is the wrong
        // shape — the corridor's only pickup (14.2,-28.0) is an EMPTY gun (no ammo gain; run
        // 13's `cycling weapons (0) (weapon 0x1c clip 0 spare 0)` is the same box [L d3run19:
        // attempt 1's second target (14.5,-29.0) sits behind the (14.0,-30.3) door and the cold
        // walk to it ate 25 hits]), and the sprint itself walked Bond INTO the uncleared lane
        // cluster (run 18: #10's 18-hit point-blank drain, died 10463 [L d3run18]). The kill-
        // loot cycle the file already runs on the servants' ground is the player shape instead:
        // the stands kill the lane's nearest guards with the in-hand clip, their drops land at
        // their feet (#9 died at (10.8,-22.3) 1.35 u from the node, #10 at (19.7,-21.4) [L the
        // fights lists]), the scoop re-arms Bond (each drop carries the guard's own weapon +
        // clip), and the re-stand answers #6 with rounds left.
        clear_exposed_nearest_03(&mut w, "the lane north shooters", 40.0, 8, 0, false);
        w.lenient = true;
        loot_the_drops_03(&mut w, "the lane north drops (after the stands)");
        w.lenient = false;
        clear_exposed_nearest_03(&mut w, "the lane north shooters (re-armed)", 40.0, 8, 0, false);
    }
    for (name, at) in [
        ("the lane south", [10.41f32, 6.08, -28.74]),
        ("the moat east", [9.95, 5.84, -34.41]),
        ("along the moat west", [6.87, 5.84, -33.47]),
        ("the moat west end", [2.08, 5.30, -33.39]),
        ("west lawn", [0.0, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("gate plate lawn", [11.0, 5.9, -35.0]),
    ] {
        if difficulty >= 3 {
            // Run 10: the plain clear_near's hip-cone taps take 3-5 rounds a d3 guard and the
            // moat-ditch machine gun #12 (skin 0x500003a, weapon 0xe, (4.2,-26.7)) survived
            // them at hp 1 and ambushed the moat-west-end resupply leg for 41.2 (4 hits, the
            // run died at 12475) [L d3run10]. The aimed stands (the aim-bit zero-spread act
            // [M Bullet_init 0x12c5a0:139-170]) kill a 12-hp guard in 1-2 taps, and the pick
            // is PROACTIVE (no recent-hit gate — #12's first round landed only after his
            // leg's reactive window had passed); the plain clear stays as the backstop.
            clear_exposed_nearest_03(&mut w, name, 12.0, 4, 0, false);
            clear_near(&mut w, 9.0, name);
            // real-health-00agent-03-captors-5 (the -4 ledger item 1, arming the walk to the
            // saddle): the clears' kills leave their guns at the guards' feet — run 21 reached
            // the saddle DRY (clip 0 spare 0, the wheel cycled to the fists) and #7's hip taps
            // finished Bond there [L d3run21]; the si-1 saddle walk's own pickup scan found
            // every weapon/ammo pickup within 45 u of the saddle already gone, so the rounds
            // for the endgame must come from the route's own drops [L the captors-5 pickup
            // scan, queue 11-14]. The servants'-ground loop already runs the same cycle.
            w.lenient = true;
            loot_the_drops_03(&mut w, &format!("{name} drops"));
            w.lenient = false;
            // real-health-00agent-03-captors-6 (run 38's loot diag): the lane kills' guns sit
            // in the moat ditch BELOW the lane's east edge — #11's at (14.50,5.35,-29.03)
            // dxz 4.2, (14.19,5.34,-28.02) dxz 3.6, (9.66,5.52,-33.13) dxz 4.5 from the leg
            // spots — every one in the loot's 8-u/±1.5-y band but DECLINED by the helper's
            // sight filter (the ditch lip blocks the eye ray from the lane floor). The loot
            // declined on EVERY leg [L d3run38's diag], so the route's own machine-gun clips
            // (~30 a 0xe) were never claimed and the walk fought the whole mid-game on the
            // starting 23 rounds. The ditch scoop drops the sight filter (the nav finds its
            // own path down; the walk-over take needs no sight) and walks onto each drop.
            scoop_drops_03(&mut w, &format!("{name} ditch scoop"), 8.0);
            // nb-scan-all retune (this branch, the t30603 ledger): the spacing scan walks the
            // ditch machine gun #12 west along the moat (the green -9 route knew him at his
            // (4.2,-26.7) post) and every stand on him from the LANE floor failed killed-false
            // — the ditch lip blocks the eye ray [L d3run38]. From the ditch floor — where the
            // scoop just put Bond — the line is open: an aimed stand right after each scoop
            // answers him (and anything else the ditch exposes) before the next leg walks the
            // moat west end under his muzzle.
            clear_exposed_nearest_03(&mut w, &format!("{name} ditch stand"), 40.0, 4, 0, false);
        }
        w.walk_nav_to(at, name, 1.6);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x21),
            "the west-lane approach stands inside the west 0x21 plate's footprint [D lo/hi x 7.3..10.7 z −35.0..−33.7]");
    }
    // guard-repin-master-2 item 1 (the 06 stand_shooters_down law, d-gated), measured on the
    // first run: #11 is already dead (the plain fights killed him at 9952 during the west-lawn
    // legs) by this leg end — the sweep here is the law's placement for the legs AHEAD (any
    // shooter who opened on the moat legs).
    if difficulty >= 2 {
        stand_shooters_down_03(&mut w, "the gate-plate-lawn shooters", 40.0);
    }
    // Back to the servants' ground by the east lane the "mid courtyard" leg itself walked
    // (x 10.0-10.6 from the moat to the door at (11.8, 6.5, -20.2): the nav stops Bond reached
    // on that leg). The nav's own way "back across the courtyard" to (13, -18) runs through the
    // obstacle at (6.35, -15.7..-17.7) (Bond climbs to y 7.3, airborne) and wedges at every cap, the
    // servant approach and the captor flanks with it — the captor (mode 4, dodging at
    // x 10-15, z -21..-25) stays out of sight across the wall.
    w.lenient = true;
    for (name, at) in [
        ("the east lane south", [10.05f32, 6.56, -31.87]),
        ("the east lane", [10.27, 6.65, -30.01]),
        ("the east lane north", [10.59, 6.47, -22.91]),
    ] {
        clear_near(&mut w, 8.0, name);
        // audit-voice-triggers-3: lenient is re-armed per leg — the clear_near/shoot internals
        // reset it on their way out, and the 2026-11-05 run's wedged leg panicked at the
        // non-lenient cap instead of returning to the ladder below.
        w.lenient = true;
        let mut got = w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
        // The wedge ladder: the lane climb wedges from the moat's south edge (the 2026-11-05
        // run: the mid anchor's nav slid Bond DOWN the ditch to (10.28, 6.34, -36.05), the
        // sidesteps never finding the ramp; segment_to_target hit at t 0.10). A player does not
        // stand in a ditch: sidestep ALONG the moat (the moat legs above walked it fine), then
        // take the anchor again from the stepped-aside spot. Both sides fail: skip the anchor —
        // the anchors steer the nav, the NEXT anchor re-derives its own path and the route
        // continues (the same doctrine as the 01 walk's wedged-lane ladders).
        if got == u32::MAX {
            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
            for (side, sname) in [(1.0f32, "east"), (-1.0f32, "west")] {
                if difficulty >= 3 {
                    // 00 Agent (real-health-00agent-03-captors): the nav sidestep rode the A*
                    // onto the building's upper row and the east-wing sniper ended the walk
                    // there [L the los-mask-repins-03 ledger, tick 53672]. The roof-proof
                    // direct sidestep (short ground legs, y-guarded) replaces it; the re-take
                    // below stays the same direct anchor walk.
                    if !direct_sidestep_03(&mut w, side, &format!("{name}: the moat sidestep {sname}")) { continue; }
                } else {
                    let to = [bp[0] + side * 2.5, bp[1], bp[2] + side * 1.5];
                    w.walk_nav_to(to, &format!("{name}: the moat sidestep {sname} (the wedged nav's ladder)"), 1.2);
                }
                w.lenient = true;
                got = w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, &format!("{name} (after the {sname} sidestep)"));
                if got != u32::MAX { break; }
            }
            if got == u32::MAX {
                eprintln!("[03] {name}: both sidesteps wedged — skipping the anchor, the next leg re-derives the path");
            }
        }
    }
    w.lenient = false;
    if difficulty >= 2 {
        // 00 Agent (real-health-00agent-02-03-2): the servants' ground is a running fight (the
        // 0x24/0x25 guards and their captains) and the real-damage hook's resupply is the lane
        // walks' arm, not this walk's. A player scavenges the dead guards' drops between
        // contacts: the dry P99 (clip 1 spare 0) met shotgun captain #5 at 1.6 u and died to
        // his volley at t42315 [L the 00agent-02-03-2 ledger]. resupply walks the covered path
        // only (the 9-u corridor rule), so this stays on the route.
        // door-walk-03b: AGENT runs the same failure one difficulty down — the d2 ledger died
        // at tick 52087 with weapon 0x12 clip 0 spare 0 (nav 6 was "cycling weapons (0)") —
        // so the arm extends to the Agent pick (difficulty 2 keeps difficulty 1's two green
        // walks byte-identical).
        // real-health-00agent-03-captors-5 (the -4 ledger item 1, the d3 half): the resupply's
        // attempt-0 target (-10.9,-38.0) walked Bond 25.7 u WEST through the moat for nothing
        // — `resupply done (false) clip 3 spare 13` [L d3run20] — and its return puts the si-1
        // saddle nav's A* on the wedging lane-doors route (run 22's 22 stuck stops at
        // (11.88,6.46,-20.90) all start from the resupply's end). d3 drops this walk (its own
        // loot cycle above is the arm); d2 keeps it byte-identically under == 2.
        if difficulty == 2 {
            let _ = w.resupply("before the servants' rescue (agent)");
        }
    }
    for (plate, want_ch) in [(None, 0x4fu8), (Some(plate0x25), 0x50u8)] {
        let (si, at, rescue_ch) = *servants.iter().find(|(_, _, ch)| *ch == want_ch)
            .unwrap_or_else(|| panic!("the servant carrying {want_ch:#x} [D p6]"));
        // #389's 0x21 plate already fired on the lane approach; #388's 0x25 plate is walked here.
        if let Some(plate) = plate {
            if difficulty >= 2 {
                // 00 Agent: rounds before the plate walk (the running fight on the servants'
                // ground; the dry-gun death above) [L the 00agent-02-03-2 ledger].
                // door-walk-03b: extended to Agent (the same dry-gun death mode at d2).
                let _ = w.resupply(&format!("before the {want_ch:#x} plate (agent)"));
            }
            w.walk_nav_to(plate, &format!("onto the {want_ch:#x}-servant's enable plate"), 1.8);
            if !w.scene.sim.as_ref().unwrap().drones.channels.on(0x25) {
                // An awake guard standing in the double doors' sweep on this leg (doors 4/5 at
                // (19.9/21.8, −20.25)) turns the leaves back each tick — they shuttle between
                // pos01 0.71 and 0.79 and the doorway never clears [L NF_DOORTRACE]. A player
                // shoots him: drop every live armed guard within 3 of the doorway, then re-walk.
                let blockers: Vec<usize> = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.list.iter().enumerate()
                        .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && d.team != 3
                            && ((d.pos[0] - 20.9).powi(2) + (d.pos[2] + 20.25).powi(2)).sqrt() < 3.0)
                        .map(|(i, _)| i).collect()
                };
                for gi in blockers {
                    w.shoot(gi, "the guard in the 0x25 doorway");
                }
                w.walk_nav_to(plate, &format!("onto the {want_ch:#x}-servant's enable plate (again)"), 1.8);
            }
        }
        w.wait(10);
        if difficulty >= 2 {
            // 00 Agent: rounds before the servant walk (the same running-fight rule) [L the
            // 00agent-02-03-2 ledger].
            // door-walk-03b: extended to Agent (the same dry-gun death mode at d2).
            let _ = w.resupply(&format!("before servant {si} (agent)"));
        }
        // The G-055 flush's second-block guards hold the servants' ground posts and cross-fire
        // the approach (the merged-tree real-damage run: the trio #14/#2/#18 each cost 24-34
        // health during the servant navs, 88 total, and the exit lawn finished the last 6.8).
        // A player clears the ground one guard at a time from where he stands and scoops the
        // drops (the kill-loot cycle) — the fights shorten with every kill.
        clear_near(&mut w, 9.0, "the servants' ground");
        loot_the_drops_03(&mut w, "the servants' ground");
        // 00 Agent (real-health-00agent-03-captors-3, runs 12-15): the east-wing upper sniper
        // #6 (skin 0x5000036 weapon 0x1e, hp 12, never left (26.2,9.9,0.8)) reaches over BOTH
        // servant phases — one 39.4-41.2 round at the door-8 mouth nav or the saddle nav ends
        // the run outright (run 15 died at 15555 at (16.98,-11.14) mid door-8 nav, run 12
        // survived the same nav at 22.0 hp only because #6's lean windows missed until 86297
        // [L d3run12/15]). The walkway the servants' ground stands on is #6's own height
        // (y ~9.1 vs 9.9, d ~31) — a player answers the east-wing window BEFORE walking the
        // mouth: sniper-first takes #6's 12 hp in 1-2 taps with the clip-32 resupply the
        // servant leg just ran; unseen/DRY stands skip (the sweep's own gates). d>=3 only —
        // the d<3 streams and the other difficulties never enter.
        if difficulty >= 3 {
            clear_exposed_nearest_03(&mut w, "the servants' ground elevated (agent)", 40.0, 4, 600, true);
        }
        // real-health-00agent-03-captors (00 Agent): the mouth nav crosses the east-lawn door
        // (22.3,5.3,10.9) and the east-lawn trio's shotgun #5 stands ON that doorway — the
        // measured run died there at d 2.4 with his hp 12 still on him. The saddle clear is the
        // player's answer in shape, but the measured run shows the trio is DISABLED until the
        // 0x23 plate fires — and the plate sits past this leg, under the mouth nav's door
        // crossing — so this clear passes the sleeping #5 by (clear_near's d.enabled filter)
        // and the wake ambush happens on the mouth nav (the ledger below). Kept for the loot
        // and the lawn posture; the wake ambush itself is the ledger's open item.
        if difficulty >= 3 && si == 0 {
            // 00 Agent (real-health-00agent-03-captors-3, run 12's ledger): the lane's elevated
            // survivors reach over the servants' ground and finish the worn-down walk ON the
            // saddle nav — the upper-east sniper #6 (skin 0x5000036 weapon 0x1e, hp 12, never
            // left (26.2,9.9,0.8), d ~8.0 from the ground) hit for 11.4 and the lane's #7
            // (skin 0x5000039 weapon 0x12, hp 2, (11.7,10.1,-16.4)) plinked 1.8 a round while
            // Bond walked; he died at 88424 at (21.64,5.45,-0.99) mid saddle nav with 22.0
            // entering the leg [L d3run12]. A player looks UP from the ground and answers the
            // rail/upper-floor shooters BEFORE stepping onto the lawn: sniper-first takes #6's
            // 12 hp in 1-2 taps (the 40.0-a-hit rounds decide every endgame run), and the plate
            // under the door crossing stays unfired — the trio's wake is untouched. Unseen/DRY
            // stands skip (the sweep's own gates); the d<3 streams and the si-1 leg never
            // enter this block.
            clear_exposed_nearest_03(&mut w, "the east lawn elevated (00agent)", 40.0, 4, 600, true);
            w.walk_nav_to([23.5, 6.3, 8.4], "the east lawn saddle (00agent, before the mouth)", 1.5);
            w.lenient = true;
            clear_near(&mut w, 9.0, "the east lawn (00agent, before the mouth)");
            w.lenient = true;
            loot_the_drops_03(&mut w, "the east lawn (00agent)");
            w.lenient = false;
        }
        if difficulty >= 3 && si != 0 {
            // real-health-00agent-03-captors-4, lever 2 (the -3 ledger): the si-1 door-8 mouth
            // nav vs #6 is the 40-class coin flip — runs 15/16/17 all died at 15555-15776 at
            // (16.9,-11.2) mid that nav to ONE #6 round (39.4/41.2/41.2), while run 12 survived
            // it only behind the (11.88,-20.9) wedge [L d3run12/15/16/17]. The player's shape:
            // walk the east-lawn saddle (23.5,6.3,8.4) BEFORE the cluster block — #6 is exposed
            // from the saddle (run 12's 86895 hit), the aimed sweep takes him at d 8.1 (12 hp =
            // 1-2 taps), and a 0x23-plate trio wake on the approach is then answered AT the
            // saddle with the stands already up (the -2 ledger's lever-2 shape — outside the
            // first volley's arc) instead of mid-mouth-crossing. The si-0 blocks and the trio
            // gates then see the trio already enabled: re-checked — the 0x25 check below is a
            // conditional (no assert), the only other plate assert (ch 0x21, line above) runs
            // BEFORE the loop, and the cluster band ((31.9,-9.4) r 8) excludes the trio's spots
            // ((22.8,9.8)/(24.3,13.5)/(24.5,14.8)) [L the gas-rng-draws-6 probe]. Unseen/DRY
            // stands skip (the sweep's own gates); d>=3 + this phase only.
            // MEASURED (run 20): the A* from the servants' ground to the saddle detours WEST
            // through the whole moat band and wedges 7x3000 on the lane doors — the mouth nav's
            // own route is the way across instead: its first legs land on the lawn anchor
            // (22.13,5.47,-2.03) and from THERE the saddle climb is the si-0-proven hop; the
            // mouth nav's own crossing then rode the saddle zone at (25.30,7.37,8.79) with the
            // 0x23 plate firing mid-climb — #5's charge reached d 2.5 and the 8-pellet volleys
            // finished Bond at 80288 [L d3run20]. Stand at the saddle BEFORE the mouth nav.
            // real-health-00agent-03-captors-5, run 22 MEASURED (ledger item 2, the anchorless
            // saddle route): dropping the anchor nav REGRESSED the phase back to run 20's shape
            // byte-identically — the saddle nav from the si-1 resupply end wedged on the lane
            // doors (22 stuck stops at (11.88,6.46,-20.90)), the climb never engaged, and the
            // walk died at 80288 at (25.30,7.37,8.79) mid-mouth-crossing, run 20's exact tick
            // and spot [L d3run22]. The anchor nav's own wedged-bridge ladder is what moves
            // Bond onto the route the saddle nav can climb from (run 21: the anchor capped
            // 20x3000, the saddle nav then worked and arrived (23.18,5.50,7.20)). The anchor
            // stays; the DRY arrival (run 21's real killer) is the arming lever below.
            // real-health-00agent-03-captors-5, run 23 MEASURED: the armed-gate stand from
            // (20,3) landed NOTHING — the diagnostic trace shows the aim CONVERGED (err
            // 0.0000, perr 0.0000, aim on #7's boxes, still 1502, aiming true) but the gun was
            // the FISTS (clip 1 spare 0) from tap-gate t0: the return-fire fights on the
            // (20,3) approach walk drained the 16 rounds before the stand [L d3run23/run24
            // trace]. Lever: the si-1 approach runs COLD (the crossing-legs doctrine — run 13
            // kept 3+13 cold) so the rounds reach the stand, and the ammo is re-checked at the
            // stand itself (a gate checked before a walk is not the gun at the stand's t0).
            w.dmg.return_fire = false;
            w.walk_nav_to([22.1, 5.5, -2.0], "the mouth path east anchor (si-1)", 1.2);
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let inv = &sim.drones.inventory;
                let total: i16 = inv.carried.iter().sum();
                eprintln!("[03-ammo] si-1 anchor: gun {:#x} clip {} spare {} carried-total {} hp {:.1}",
                    sim.gun.weapon, sim.gun.clip, sim.gun.spare, total, sim.drones.health.health);
            }
            // real-health-00agent-03-captors-5, lever 2 (the -4 ledger item 3): #7 is the
            // endgame drain — 43-51 hip hits for 27.4-41.2 across runs 20/21, alive at hp 4-10
            // at both deaths [L d3run20/21]. A player with rounds takes the aimed stand from
            // the lawn band the saddle approach crosses BEFORE the climb: #7's boxes are clear
            // from (20,2..4) and (18..24,8) while the saddle itself (23.2,7.2) is not
            // [L the captors-5 bullet-line scan, queue 11-14]. Gated on the rounds at the
            // stand (>= 5: a 12-hp guard at the P2K's 3.5 a hit needs 4) and the walker's own
            // eye-ray exposure (the shared rule) — an unseen or dry walk skips the stand and
            // climbs as before.
            {
                let seven = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.list.iter().enumerate().find(|(_, d)| {
                        d.team == 1 && d.skin & 0xffff == 0x39
                            && !d.corpse && d.health > 0.0 && d.present
                    }).map(|(i, _)| i)
                };
                let armed = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let g = &sim.gun;
                    (2..=0x47).contains(&g.weapon) && g.weapon != 0x34 && g.weapon != 0x37
                        && g.clip + g.spare >= 5
                };
                if let (Some(di), true) = (seven, armed) {
                    w.walk_nav_to([20.0, 5.9, 3.0], "the #7 line spot (si-1 saddle approach)", 1.5);
                    let armed = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let g = &sim.gun;
                        (2..=0x47).contains(&g.weapon) && g.weapon != 0x34 && g.weapon != 0x37
                            && g.clip + g.spare >= 5
                    };
                    if armed && w.exposed_aim(di).is_some() {
                        aimed_stand_03(&mut w, di, "the #7 aimed stand (si-1 saddle approach)", 1800, false);
                        let sim = w.scene.sim.as_ref().unwrap();
                        eprintln!("[03-ammo] after the #7 stand: gun {:#x} clip {} spare {} hp {:.1}",
                            sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
                    }
                }
            }
            w.dmg.return_fire = true;
            w.walk_nav_to([23.5, 6.3, 8.4], "the east lawn saddle (00agent, si-1 before the mouth)", 1.5);
            clear_exposed_nearest_03(&mut w, "the east lawn saddle wake (00agent, si-1)", 40.0, 4, 600, true);
            w.lenient = true;
            loot_the_drops_03(&mut w, "the east lawn saddle (00agent, si-1)");
            w.lenient = false;
            // real-health-00agent-03-captors-5, run 25 MEASURED (the -4 ledger item 4): with
            // #7 answered the run reached PAST the saddle and died at (23.61,6.37,10.57) — the
            // mouth nav's own door crossing fired the 0x23 plate mid-leg and the waking trio
            // point-blanked a DRY Bond before any stand could react (#5 charged to d 3.3 for
            // 16.8, #4 12.6 in 7, #3 11.8 in 2 — the wake sweep at the saddle had found nobody:
            // the trio is DISABLED until the plate fires [L d3run25, the no-op sweep]). The
            // player's shape is the door-8 cluster block's own: step ONTO the plate, let the
            // trio pop, and fight the wake from the saddle side with the kill-loot cycle
            // instead of walking into it mid-nav — #5's drop (his own 0x1c) re-arms the endgame
            // the pickup scan says is otherwise dry [L the captors-5 pickup scan, queue 11-14].
            {
                let plate0x23 = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.triggers.iter()
                        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x23)
                        .map(|t| t.centre())
                };
                if let Some(pc) = plate0x23 {
                    // 12.0: the trio's LOAD spots sit 8-10.5 u from the plate centre ((29.36,16.36)
                    // #5, (28.8,16.3) #3, (26.3,16.4) #4 [L d3run27's census]) — they only WALK to
                    // the plate's doorway once it fires.
                    let trio_alive = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        sim.drones.list.iter().any(|d| d.team == 1 && !d.corpse && d.health > 0.0
                            && walker::dxz(d.pos, pc) < 12.0)
                    };
                    if trio_alive {
                        {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let inv = &sim.drones.inventory;
                            eprintln!("[03-ammo] at the plate: gun {:#x} clip {} spare {} carried-total {} hp {:.1}",
                                sim.gun.weapon, sim.gun.clip, sim.gun.spare, inv.carried.iter().sum::<i16>(), sim.drones.health.health);
                            for (i, d) in sim.drones.list.iter().enumerate() {
                                if d.team == 1 && !d.corpse && d.health > 0.0 && walker::dxz(d.pos, pc) < 12.0 {
                                    eprintln!("[03-ammo] trio #{i} skin {:#x} weapon {:#x} hp {:.1} at ({:.1},{:.1},{:.1}) enabled {}",
                                        d.skin, d.weapon, d.health, d.pos[0], d.pos[1], d.pos[2], d.enabled);
                                }
                            }
                        }
                        w.walk_nav_to([pc[0], pc[1], pc[2]], "onto the 0x23 plate (si-1, the trio wake)", 1.6);
                        w.wait(30);
                        // real-health-00agent-03-captors-6, lever 1 (the -5 ledger item 1, the
                        // trio wake race, MEASURED on runs 29/30): the wake sweep's 600-tick
                        // budget found NOBODY — the trio's scripted wake walk ends at the
                        // east-lawn door (22.3,5.3,10.9) and that door stands CLOSED through
                        // the whole block (the si-0 saddle walk went through it, but the leaf
                        // shut again before si-1), so the trio bunches BEHIND the leaf and the
                        // single eye ray never reaches their boxes; they crossed only when the
                        // servant nav's own wedge E-opened the door mid-walk, and the doorway
                        // point-blank finished Bond at 73318 with the trio at FULL hp — the
                        // wake sweep never fired a shot (the gun state at the post-wake print
                        // is unchanged: clip 5 spare 6) [L d3run29/30]. The player's shape is
                        // the door funnel: open the door he is about to need HIMSELF, step
                        // back south off the doorway line, and fight what comes through — one
                        // guard crosses the door plane at a time, exposed from the moment his
                        // boxes cross, and his drop re-arms the next fight (the kill-loot
                        // cycle the servants' ground runs).
                        {
                            let east_door = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                sim.drones.doors.list.iter().enumerate()
                                    .find(|(_, d)| walker::dxz(d.pos, [22.3f32, 5.3, 10.9]) < 1.0)
                                    .map(|(i, d)| (i, d.state, d.pos))
                            };
                            if let Some((di, state, dpos)) = east_door {
                                if state == nf_game::door::CLOSED {
                                    eprintln!("[03] the east-lawn door #{di} sits closed — opening it for the trio funnel");
                                    w.walk_to(Wp { at: [dpos[0], 5.9, dpos[2] - 1.1], act: Act::None, arrive: 0.7 },
                                        "the east-lawn door threshold (the trio wake)");
                                    w.use_door(di, "the east-lawn door (the trio wake funnel)");
                                }
                            }
                        }
                        // The duel line: run 32 MEASURED the trio does NOT pour through — they
                        // halt NORTH of the open door and duel through the opening (#4 held
                        // (26.5,16.7) d 8.4 for 13 hits, #5 (24.7,14.9) d 6.1 for 16, both
                        // alive at the death): from the (22.6,8.0) standoff Bond's eye ray to
                        // their boxes crosses the wall plane EAST of the doorway band (the ray
                        // to (26.5,16.7) exits at x 23.47) — the plain fight on #4 ran 436
                        // ticks killed-false with every tap unseen-gated off [L d3run32]. The
                        // player steps ONTO the doorway line (the door-8 mouth fights' own
                        // shape — fought from (36.2,5.7) THROUGH the open doorway), where the
                        // ray passes the opening; the trio's halt keeps them out of the
                        // under-3.0 point-blank band [M DroneWeap_DoBulletAccuracy
                        // 0x179740:113-115] at d 4.7-5.9.
                        w.walk_to(Wp { at: [22.3, 5.9, 9.6], act: Act::None, arrive: 0.6 }, "the trio wake duel line (the doorway sight line)");
                        // THE KILL-LOOT CYCLE (runs 33-39): with the ditch rounds in the wheel
                        // (the leg scoops above) the serial sweep kills ALL THREE through the
                        // stands — the single-sweep shape runs 37/38 measured (engage #5 → #4
                        // → the captain back-to-back, keep_aim serial, ZERO damage taken: the
                        // interleave's post-stand gap in run 39 instead let #4/#3 pour 41.2
                        // through the doorway while Bond levelled the aim). The loot + the
                        // scoop after the sweep re-arm from the trio's own drops.
                        clear_exposed_nearest_03(&mut w, "the 0x23 trio wake (si-1)", 40.0, 4, 900, true);
                        w.lenient = true;
                        loot_the_drops_03(&mut w, "the 0x23 trio wake (si-1)");
                        scoop_drops_03(&mut w, "the 0x23 trio wake drops", 8.0);
                        loot_the_drops_03(&mut w, "the 0x23 trio wake (armed cycle)");
                        w.lenient = false;
                        // The funnel's second round: re-take the duel line, re-sweep, re-loot
                        // (the servants' ground cycle's shape).
                        w.walk_to(Wp { at: [22.3, 5.9, 9.6], act: Act::None, arrive: 0.6 }, "the trio wake duel line (again)");
                        clear_exposed_nearest_03(&mut w, "the 0x23 trio wake (si-1, again)", 40.0, 4, 900, true);
                        w.lenient = true;
                        loot_the_drops_03(&mut w, "the 0x23 trio wake (si-1, again)");
                        w.lenient = false;
                        {
                            let sim = w.scene.sim.as_ref().unwrap();
                            eprintln!("[03-ammo] after the wake: gun {:#x} clip {} spare {} hp {:.1}",
                                sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
                            let inv = &sim.drones.inventory;
                            let clips: Vec<String> = inv.clip.iter().enumerate()
                                .filter(|(_, c)| **c != 0)
                                .map(|(w, c)| format!("{w:#x}:{c}")).collect();
                            let carried: Vec<String> = inv.carried.iter().enumerate()
                                .filter(|(_, c)| **c != 0)
                                .map(|(t, c)| format!("{t}:{c}")).collect();
                            eprintln!("[03-ammo] wheel clips [{}] carried [{}]", clips.join(" "), carried.join(" "));
                            for (i, p) in sim.drones.pickups.iter().enumerate() {
                                if !p.gone && walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], sim.bond.pos) < 20.0 {
                                    eprintln!("[03-ammo] pickup #{i} kind {} at ({:.2},{:.2},{:.2}) ch {:#x}",
                                        p.kind, p.m[3][0], p.m[3][1], p.m[3][2], p.switch_channel);
                                }
                            }
                            for (i, d) in sim.drones.list.iter().enumerate() {
                                if d.team == 1 && walker::dxz(d.pos, pc) < 14.0 && d.skin & 0xffff != 0x2b {
                                    eprintln!("[03-ammo] post-wake #{i} skin {:#x} hp {:.1} corpse {} at ({:.1},{:.1},{:.1})",
                                        d.skin, d.health, d.corpse, d.pos[0], d.pos[1], d.pos[2]);
                                }
                            }
                        }
                    }
                }
            }
        }

        // gas-rng-draws: the door-8 cluster (#18/#19/#2 behind the leaves at (38.7, 0.2)) can
        // outlive the ground clear — clear_near's far+unexposed skip passes them by while the
        // leaves hide their boxes, and the servant nav then walks THROUGH the opening door
        // into #18's point-blank volley (the fresh d2 ledger: killed at 56606 at (40.30, 0.29),
        // 1.6 u, health 6.2 entering the leg). A player at a door that just hissed open does
        // not step through blind: he fights what shows through the mouth from the hinge side.
        // Self-gating on the cluster being alive keeps every stream that already cleared it
        // byte-identical (the green runs fought the cluster through this doorway earlier).
        {
            let cluster_alive = {
                let sim = w.scene.sim.as_ref().unwrap();
                // The anchor is the cluster's LOAD band [L the gas-rng-draws-6 probe:
                // #18 (27.92, 6.25, −5.57) hp 12 weapon 0x1c, #2 (30.69, 6.25, −7.55) hp 12
                // weapon 0xe, #19 (37.01, 6.25, −15.04) hp 12 weapon 0xe, all disabled state
                // 0x1, and door #8 (35.62, 6.51, 2.71) closed state 0x2 pos01 0.50; no other
                // hostile stands within 8 of the band centre]. The first anchor — the
                // POST-open pocket (38.7, 0.2) r 6.0 they walk to AFTER the door opens —
                // never matched the load spots, so the block never fired on the failing
                // stream. No `enabled` in the filter: the door-8 members load DISABLED behind
                // the leaves (the 01 ch-0xF pocket law — disabled objects are dropped from
                // bullet collision [M Collide_PickObj 0x1eae80:82]) and pop when the door
                // opens — which is exactly why the mouth walk must run first. Only a corpse
                // or a finished fight keeps the block a no-op (the green streams cleared
                // them).
                sim.drones.list.iter().any(|d| {
                    d.team == 1 && !d.corpse && d.health > 0.0
                        && walker::dxz(d.pos, [31.9f32, 6.25f32, -9.4f32]) < 8.0
                })
            };
            if cluster_alive {
                // real-health-00agent-03-captors-6, runs 43-48: the mouth walk's A* route
                // detours NORTH-EAST through (38.69,10.96)/(39.73,14.82) — over the door-8
                // area's own wake (the (39.0,12.9) door's open channel and/or the (38.9,11.1)
                // 0x23 plate [L d3run43/45]) — and that wake pops #14 (skin 0x5000039, weapon
                // 0xe, load disabled (30.7,6.2,4.1) [L d3run35's census], enabled false
                // through the whole trio fight): he closed to d 2.5 and his machine-gun
                // bursts (41.2 in 3 hits, one 24.0 a round) finished Bond mid-walk at
                // (35.96,4.27) with the plain return fire killed-false on him [L d3run43].
                // The tried player shapes (the si==0 south legs z 5.7-6.8 [runs 45-47: the
                // garden wall corner wedges the direct legs and the A* rides the north lawn
                // into the plate anyway] and the leg-0 aimed sweep [run 46: #14 unexposed
                // behind the building's west face]) did not beat it — the d3 sweep below
                // (after the settle loop) is this round's answer and the approach problem is
                // the ledger's open item.
                // real-health-00agent-03-captors-7, lever 1 (the -6 ledger item 1, the mouth
                // approach): the si-1 mouth nav's A* starts from the duel line (22.3,9.6) and
                // runs 43/49's route detoured NORTH-EAST over the door-8 wake zone
                // ((38.69,10.96)/(39.73,14.82) [L d3run43]) — popping #14 mid-walk, death 67488.
                // The si-0 mouth nav STARTS from the saddle (the si==0 block walks Bond onto
                // (23.5,6.3,8.4) before this block) and its route left #14 asleep through the
                // whole trio fight [L d3run43: #14 enabled false until the si-1 mouth walk] — a
                // player walks the mouth approach from the saddle cell the si==0 way, not from
                // the duel line's cell. The hop is a no-op when Bond already stands on the
                // saddle (the si-0 iteration; the gate reads his live spot).
                if difficulty >= 3 {
                    let off_saddle = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        walker::dxz(sim.bond.pos, [23.5f32, 6.3, 8.4]) > 2.0
                    };
                    if off_saddle {
                        w.lenient = true;
                        w.walk_nav_to([23.5, 6.3, 8.4], "the saddle re-take (the si-0 way to the door-8 mouth)", 1.0);
                        w.lenient = false;
                    }
                    // real-health-00agent-03-captors-7, lever 2 (the #14 funnel): the route
                    // probe [L d3run51] shows EVERY nav route to the mouth crosses the
                    // (39.0,12.9) doorway band (the only descent is nodes (39.93,13.55) ->
                    // (39.90,12.23) through the leaf) — the mouth nav's stuck handler E-opens
                    // that door MID-NAV and #14 pops on the open channel unprepared (run 50:
                    // the hits began during nav 9, 41.2 in 4 at d 1.4, death 67870). The
                    // player's shape is the trio wake funnel's own: take the last pre-door
                    // node, hold the aim up, E-open the door HIMSELF, and fight what pops
                    // through the pass with the aimed stands — one charger crosses the
                    // doorway plane at a time, exposed the whole run.
                    w.walk_nav_to([39.9, 6.3, 14.4], "the north-door standoff (the #14 funnel)", 1.0);
                    {
                        // The -6 ledger item 2 [S unverified]: cycle the wheel to the 0x2
                        // heavy gun before the fight (the 0x12's 7 light rounds outrank it in
                        // the default cycle; the fight needs rounds that kill a 12-hp
                        // machine gunner fast). Fallback: after 8 cycles the walk fights on
                        // whatever armed gun the wheel holds.
                        for _ in 0..8 {
                            let wnow = { let sim = w.scene.sim.as_ref().unwrap(); sim.gun.weapon };
                            if wnow == 0x2 { break; }
                            w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
                            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
                        }
                        {
                            let sim = w.scene.sim.as_ref().unwrap();
                            eprintln!("[03-ammo] at the north-door standoff: gun {:#x} clip {} spare {} hp {:.1}",
                                sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
                        }
                        // Pre-raise the aim so the sweep's first stand converges while #14 is
                        // still crossing the pass.
                        for _ in 0..240 {
                            let sim = w.scene.sim.as_ref().unwrap();
                            if sim.aiming { break; }
                            w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
                            w.total_ticks += 1;
                        }
                        let north_door = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            sim.drones.doors.list.iter().enumerate()
                                .find(|(_, d)| walker::dxz(d.pos, [39.0f32, 5.4, 12.9]) < 1.0)
                                .map(|(i, d)| (i, d.state))
                        };
                        if let Some((di, state)) = north_door {
                            if state == nf_game::door::CLOSED {
                                eprintln!("[03] the (39.0,12.9) door #{di} sits closed — E-opening it for the #14 funnel");
                                w.use_door(di, "the (39.0,12.9) door (the #14 funnel)");
                            }
                        }
                        // The funnel's answer, captor-chase shape (the blockers'/chase's own
                        // w.shoot law — no STILL gate: #14 is a CHARGER, he never holds still
                        // until point-blank, and the aimed stand's tap gate loses that race
                        // (run 52: one 8-dmg tap landed, his 41.2 point-blank burst won)). The
                        // real-fire shoot path taps him THROUGH the charge: 2-3 heavy taps
                        // land while he runs the ~11 u from his load spot to Bond's position.
                        for round in 0..6 {
                            let di = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                sim.drones.list.iter().enumerate().find(|(_, d)| {
                                    d.team == 1 && d.skin & 0xffff == 0x39 && d.weapon == 0xe
                                        && !d.corpse && d.health > 0.0 && d.present && d.enabled
                                }).map(|(i, _)| i)
                            };
                            let Some(di) = di else { break };
                            let (hp, at) = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                let d = &sim.drones.list[di];
                                (d.health, d.pos)
                            };
                            eprintln!("[03] the #14 funnel round {round}: drone #{di} hp {hp:.1} at ({:.1},{:.1},{:.1}) bond ({:.1},{:.1},{:.1}) hp {:.1}",
                                at[0], at[1], at[2],
                                { let sim = w.scene.sim.as_ref().unwrap(); let b = &sim.bond; b.pos[0] }, { let sim = w.scene.sim.as_ref().unwrap(); let b = &sim.bond; b.pos[1] }, { let sim = w.scene.sim.as_ref().unwrap(); let b = &sim.bond; b.pos[2] },
                                { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.health.health });
                            if !w.shoot(di, "the #14 funnel (the charger)") { break; }
                            for _ in 0..30 {
                                let sim = w.scene.sim.as_ref().unwrap();
                                if sim.drones.list[di].corpse || sim.drones.list[di].health <= 0.0 { break; }
                                w.tick(nf_game::player::Input::default(), false);
                            }
                        }
                        // The aimed sweep stays as the backstop (a survivor at range).
                        clear_exposed_nearest_03(&mut w, "the north-door funnel (#14)", 40.0, 4, 900, false);
                        w.lenient = true;
                        loot_the_drops_03(&mut w, "the north-door funnel drops");
                        scoop_drops_03(&mut w, "the north-door funnel scoop", 8.0);
                        w.lenient = false;
                        {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let fourteen = sim.drones.list.iter().enumerate().find(|(_, d)| d.skin & 0xffff == 0x39 && d.weapon == 0xe);
                            if let Some((i, d)) = fourteen {
                                eprintln!("[03-ammo] #14 after the funnel: #{i} hp {:.1} corpse {} enabled {} at ({:.1},{:.1},{:.1})",
                                    d.health, d.corpse, d.enabled, d.pos[0], d.pos[1], d.pos[2]);
                            }
                            eprintln!("[03-ammo] after the funnel: gun {:#x} clip {} spare {} hp {:.1}",
                                sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
                        }
                    }
                }
                w.walk_nav_to([35.6f32, 6.4f32, 3.4f32], "the door-8 mouth (the cluster holds)", 1.0);
                // real-health-00agent-03-captors-7, lever 3 (the door-8 funnel): the mouth nav
                // ARRIVES without touching door 8 (run 53: the arrival (35.91,4.34), the door
                // still closed) — the settle loop's at_pocket reads 0 (the sleeping cluster
                // never walked to the pocket) and breaks, the d3 sweep finds nobody exposed,
                // and the servant nav's own door-opening then wakes the cluster MID-WALK —
                // their pocket walk ambushed the rescue chase point-blank (death 106888: #2
                // 12.0 + #19 11.4 at d 5.3/5.7). The player's shape is the #14 funnel's own:
                // E-open door 8 HIMSELF at the mouth, then shoot down what walks out (they
                // cross the doorway exposed on their pocket route to (38.7,0.2)).
                if difficulty >= 3 {
                    let door8 = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        sim.drones.doors.list.iter().enumerate()
                            .find(|(_, d)| walker::dxz(d.pos, [35.62f32, 6.51, 2.71]) < 1.0)
                            .map(|(i, d)| (i, d.state, d.pos))
                    };
                    if let Some((di, state, dpos)) = door8 {
                        if state == nf_game::door::CLOSED {
                            eprintln!("[03] door 8 #{di} sits closed — E-opening it for the cluster funnel");
                            w.lenient = true;
                            w.walk_to(Wp { at: [dpos[0] + 0.6, 5.9, dpos[2] + 1.2], act: Act::None, arrive: 0.7 },
                                "the door-8 threshold (the cluster funnel)");
                            w.lenient = false;
                            w.use_door(di, "door 8 (the cluster funnel)");
                            // The pops ride the pocket route, not the door channel alone
                            // (run 55: the funnel's open + 600-tick poll found nobody — the
                            // cluster woke only under the servant nav's interior legs). The
                            // player steps INTO the pocket the way the rescue walk does and
                            // fights the poppers there, where their own pocket route ends.
                            w.lenient = true;
                            w.walk_nav_to([38.7f32, 6.2, 0.2], "into the door-8 pocket (the cluster funnel)", 1.2);
                            w.lenient = false;
                        }
                    }
                    // real-health-00agent-03-captors-8, lever 1 (the -7 ledger item 1): 3
                    // guards = 48 hp (#2 24 + #18 12 + #19 12 [L the gas-rng-draws-6 probe])
                    // against the ~11 spare rounds the walk carries — the fights' OWN drops
                    // re-arm the loop (the kill-loot cycle the servants' ground runs:
                    // loot_the_drops_03 + scoop_drops_03 between rounds; each drop carries the
                    // guard's own weapon + its clip [M Pickup_Handler 0x18e650]). The dry
                    // give-up no longer breaks the loop — run 56's break on !w.shoot lost #2 at
                    // hp 4 and the survivors' point-blank fire finished the post-rescue chase at
                    // 151214 (bond hp 9.0); the loop keeps shooting while ANY cluster member
                    // lives. 16 rounds: 3 fights at ~2-5 rounds a fight with the loot legs
                    // between.
                    for round in 0..16 {
                        // The pocket walk takes ~2-3 s to bring the poppers into range (run 54:
                        // #18's load spot sits 12.6 u from the mouth — outside the 12 band —
                        // and the one-shot poll broke at round 0, leaving the awake cluster to
                        // ambush the servant nav). Poll up to 600 ticks between rounds (the
                        // round-0 popper is instant; the far members walk their pocket route
                        // from the load spots — 12.6-15.3 u — and the 20-tick poll expired
                        // between their kills, leaving the walker alive behind the funnel).
                        let di = {
                            let mut found = None;
                            for _ in 0..600 {
                                let sim = w.scene.sim.as_ref().unwrap();
                                let b = sim.bond.pos;
                                found = sim.drones.list.iter().enumerate().find(|(_, d)| {
                                    d.team == 1 && !d.corpse && d.health > 0.0 && d.present && d.enabled
                                        && walker::dxz(d.pos, b) < 16.0 && (d.pos[1] - b[1]).abs() < 2.5
                                }).map(|(i, _)| i);
                                if found.is_some() { break; }
                                drop(sim);
                                w.tick(nf_game::player::Input::default(), false);
                            }
                            found
                        };
                        let Some(di) = di else { break };
                        // The pre-fight top-up (the -7 ledger item 1's volume half): a fight
                        // that starts thin burns its clip into an unexposed walker while HE
                        // closes (run 60 round 3: clip 5 spare 0 at an unexposed #19 at
                        // 14.6 u — the shoot's flank push met his closing line and his two
                        // 0xe point-blank rounds finished Bond at 23.4). A player tops up
                        // from the fresh drops before engaging; the 12-u scoop walks the
                        // 10-11 u death spots the loot's own 8-band misses. Bounded: a pass
                        // that takes no new drop means the pocket is picked clean — fight on
                        // whatever is held.
                        for _ in 0..3 {
                            let armed = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                sim.gun.clip + sim.gun.spare >= 6
                            };
                            if armed { break; }
                            let before = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                sim.drones.pickups.iter().filter(|p| !p.gone).count()
                            };
                            w.lenient = true;
                            loot_the_drops_03(&mut w, "the cluster funnel top-up");
                            scoop_drops_03(&mut w, "the cluster funnel top-up scoop", 12.0);
                            w.lenient = false;
                            let after = {
                                let sim = w.scene.sim.as_ref().unwrap();
                                sim.drones.pickups.iter().filter(|p| !p.gone).count()
                            };
                            if after >= before { break; }
                        }
                        let (hp, at) = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let d = &sim.drones.list[di];
                            (d.health, d.pos)
                        };
                        eprintln!("[03] the cluster funnel round {round}: drone #{di} hp {hp:.1} at ({:.1},{:.1},{:.1}) bond hp {:.1}",
                            at[0], at[1], at[2],
                            { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.health.health });
                        let killed = w.shoot(di, "the cluster funnel (the pocket walk)");
                        for _ in 0..30 {
                            let sim = w.scene.sim.as_ref().unwrap();
                            if sim.drones.list[di].corpse || sim.drones.list[di].health <= 0.0 { break; }
                            w.tick(nf_game::player::Input::default(), false);
                        }
                        // The kill-loot cycle BETWEEN rounds (the -7 ledger item 1): each kill
                        // drops its own weapon + clip (#18 0x1c, #2/#19 0xe [L the
                        // gas-rng-draws-6 probe]) and the walk-over take files it [M
                        // Pickup_Handler 0x18e650]. The 12-u scoop reaches the 10-11 u death
                        // spots. Runs on the dry give-up too — the loop re-arms and keeps the
                        // fight while any member lives.
                        w.lenient = true;
                        loot_the_drops_03(&mut w, "the cluster funnel loot");
                        scoop_drops_03(&mut w, "the cluster funnel scoop", 12.0);
                        w.lenient = false;
                        {
                            let sim = w.scene.sim.as_ref().unwrap();
                            eprintln!("[03] the cluster funnel round {round}: killed {killed} — gun {:#x} clip {} spare {} hp {:.1}",
                                sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
                        }
                    }
                }
                // The mouth stand opens door #8 the way the mid-nav approach did [L the d2
                // ledger — the door opened under the servant nav]; the popped members then
                // walk their pocket route (to (38.7, 0.2) [L the d2 ledger]) — ~6.9 u from
                // the mouth, inside clear_near's 9 band once they arrive. The settle loop
                // gives them that walk and fights them from the hinge side (the green runs'
                // own fight shape from (36.2, 5.7) THROUGH the doorway), instead of stepping
                // off into the servant nav with the fight unanswered.
                for round in 0..5 {
                    w.wait(90);
                    let at_pocket = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        sim.drones.list.iter().filter(|d| d.team == 1 && !d.corpse && d.health > 0.0
                            && walker::dxz(d.pos, [38.7f32, 6.2f32, 0.2f32]) < 7.0).count()
                    };
                    clear_near(&mut w, 9.0, "the door-8 mouth");
                    if at_pocket == 0 && round >= 1 { break; }
                }
                // real-health-00agent-03-captors-6 (run 43): the mouth walk's own route crosses
                // the door-8 area's second 0x23 plate (census (38.9,11.1) [L d3run27]) and that
                // wake pops #14 (skin 0x5000039, weapon 0xe, load disabled (30.7,6.2,4.1) [L
                // d3run35's census]) — he closed to d 2.5 and his machine-gun bursts (41.2 in
                // 3 hits, one 24.0) finished Bond at (35.96,4.27) with the walk's plain return
                // fire killed-false on him (the 0x12's 7 light rounds) [L d3run43]. A player at
                // a door that just hissed open squares up and AIMED-stands what popped: the
                // aim-bit zero-spread taps kill his 12 hp in 2-4 taps where the plain exchange
                // ate the whole health. d>=3 only — the d<3 streams keep their settle loop.
                if difficulty >= 3 {
                    clear_exposed_nearest_03(&mut w, "the door-8 mouth (d3 wake sweep)", 40.0, 4, 900, true);
                    w.lenient = true;
                    loot_the_drops_03(&mut w, "the door-8 mouth (d3 wake drops)");
                    scoop_drops_03(&mut w, "the door-8 mouth (d3 wake scoop)", 8.0);
                    w.lenient = false;
                }
                // los-mask-repins-03 (Agent): the cluster's guns (#18 0x1c, #2/#19 0xe [L the
                // gas-rng-draws-6 probe]) are the route's own ammo from here on — the exit legs
                // met the lane captains with clip 0 spare 0 and no armed weapon anywhere on the
                // corridor [L the 164663 run: the fights' own cycle found nothing]. A player
                // scavenges the guns he just fought for before walking on.
                if difficulty >= 2 {
                    loot_the_drops_03(&mut w, "the door-8 mouth drops");
                }
            }
        }

        // A player already standing in the servant's own room does not walk back out to the
        // saddle: when the door-8 fight and its resupply left Bond inside the cluster pocket
        // (the resupply walk ended at (38.26, 0.55), 9.7 u from the servant, through the open
        // door 8), the saddle route runs back across that room and climbs its furniture
        // (measured: Bond lifted to y 7.29 at (27.48, −3.09), then wedged at (33.10, −0.85)
        // with no door, trigger or guard within 3 u). The saddle detour exists for the east-wing
        // approach from the west, so it is skipped once Bond is within 10 u of the servant.
        // Self-gating: a run that reaches this point from the door-8 mouth stand (11.3 u) still
        // walks the saddle unchanged.
        let in_servant_room = {
            let sim = w.scene.sim.as_ref().unwrap();
            walker::dxz(sim.bond.pos, at) < 10.0
        };
        if difficulty >= 2 && si == 0 && !in_servant_room {
            // door-walk-03b (the Agent ledger, tick 52087): the si-0 (0x50) servant at
            // (30.8, 6.2, −5.7) is only reachable around the east wing — the A* hops z −1.8 →
            // +1.0 → +8.5 through the door (22.3, 5.3, 10.9) / the 0x23 plate, and the east-lawn
            // trio (#5 shotgun (22.8, 9.8), #3 captain weapon 0xe (24.3, 13.5), #4 (24.5, 14.8))
            // met the nav mid-walk: the return fire killed #4 but the crossfire (59.4 in 26 hits
            // from #5 at 2.1 u + 21.0 in 2 from #3 at 1.8 u) finished a DRY Bond. A player walks
            // the saddle first, clears the ground from it (the walker's own keep_off holds the
            // shotgun at 4 u here, unlike the mid-nav stand) and scoops the drops — the kill-loot
            // cycle the servants' ground already runs. The saddle itself is nav-reachable (the
            // failed run's nav 3 arrived there at +298 ticks [L this branch's run]).
            w.walk_nav_to([23.5, 6.3, 8.4], "the east lawn saddle", 1.5);
            w.lenient = true;
            clear_near(&mut w, 9.0, "the east lawn");
            w.lenient = true;
            loot_the_drops_03(&mut w, "the east lawn");
            w.lenient = false;
        }
        w.walk_nav_to([at[0], at[1] - 0.3, at[2]], &format!("servant {si}"), 1.7);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[si];
        assert!(!d.corpse, "servant {si} alive when Bond arrives");
        assert_eq!(d.death_channel, 0x60, "servant {si} is protected [D death_ch]");
        let captor = d.captor;
        let ch_on = sim.drones.channels.on(rescue_ch as u32);
        drop(sim);
        if !ch_on {
            // The captor (nearest armed alive guard at the HOSTAGE entering tick [M 0x144030; the
            // port's Drones tick]) — kill it with real fire; its death is the save's trigger.
            if let Some(ci) = captor {
                {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let c = &sim.drones.list[ci];
                    eprintln!("[03] servant {si}'s captor #{ci} at ({:.1},{:.1},{:.1}) state {:#x} hp {:.1} corpse {}",
                        c.pos[0], c.pos[1], c.pos[2], c.state, c.health, c.corpse);
                }
                // 00 Agent (real-health-00agent-02-03-3): the east captor's room guards (the
                // door-8 cluster at (35.5-37.5, -0.6) — #19 hp 12, #18 hp 12, #2 hp 24 shotgun)
                // shot Bond out of the captor flank's REACTIVE fights: under the guards' door
                // routes the flipped kill order (#2 first) left #18 live into a second fight and
                // his 28.8 volley (40.8 -> 12.0) was the whole green margin — the
                // pre-door-routes run answered the same cluster from (36.2,5.7) for 12.0 total
                // and finished at 28.8 [L the two NF_RF_TRACE ledgers, identical to t48500]. A
                // player takes the room's fight spot FIRST (the flank-1 nav-3 ground the green
                // run fought from — parts expose from there; from the servant ground they do
                // not, and the flank walk into the shotgun's 3.0 band died at 26.4 [L this
                // branch's first pre-clear run]) and clears the cluster hp-ascending with real
                // fire before the duel. si==0 only — the west (0x4f) leg's reactive sequence is
                // green as-is and is not touched.
                if difficulty >= 3 && si == 0 {
                    // 00 Agent (real-health-00agent-03-captors): the rally's own nav crossed
                    // the door-8 doorway with the cluster's survivors (#18/#19/#2) live on its
                    // south side and their gun line cost 25.2 health across the crossing
                    // (40.8 -> 15.6-19.2 [L the los-mask-repins-03 ledger]) — the exit lawn's
                    // ~29 never fits after it. The player's approach stays NORTH of door 8:
                    // back out to the east lawn the saddle's own way (the A* hops z -1.8 ->
                    // +1.0 -> +8.5 through the (22.3,10.9) door), then east along the lawn on
                    // direct legs (z 5.7-6.8 — north of the door's z 2.71 band the whole way —
                    // no A* to ride onto the doorway line), and fight the survivors THROUGH
                    // the open doorway from (36.2, 5.7) — the green runs' own fight spot.
                    w.walk_nav_to([23.5, 6.3, 8.4], "the east lawn (back out north of door 8)", 1.5);
                    for (k, at) in [[28.5f32, 5.9, 6.8], [33.0, 5.9, 6.2], [36.2, 5.9, 5.7]]
                        .into_iter().enumerate() {
                        w.walk_to(Wp { at, act: Act::None, arrive: 1.2 },
                            &format!("north of door 8, leg {k}"));
                    }
                    let escorts: Vec<usize> = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let mut es: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
                            .filter(|(i, d)| Some(*i) != captor && d.present && !d.corpse && d.health > 0.0
                                && d.team == 1
                                && walker::dxz(d.pos, [36.5, 5.9, 5.7]) <= 9.0)
                            .map(|(i, d)| (d.health, i)).collect();
                        es.sort_by(|a, b| a.0.total_cmp(&b.0));
                        es.into_iter().map(|(_, i)| i).collect()
                    };
                    for gi in escorts {
                        let (alive, gp, hp) = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let g = &sim.drones.list[gi];
                            (!g.corpse && g.health > 0.0, g.pos, g.health)
                        };
                        if !alive { continue; }
                        eprintln!("[03] captor room guard #{gi} at ({:.1},{:.1},{:.1}) hp {hp:.1} — pre-clear", gp[0], gp[1], gp[2]);
                        if !w.shoot(gi, &format!("servant {si} captor room guard {gi}")) {
                            if w.resupply("captor room ammo") {
                                w.shoot(gi, &format!("servant {si} captor room guard {gi} (retry)"));
                            }
                        }
                    }
                }
                let mut killed = w.shoot(ci, &format!("servant {si}'s captor"));
                // The mode-4 dodging captor pattern (the east-lane legs' own note: a captor
                // "stays out of sight across the wall"): a player chases him — walk to his LIVE
                // position, top the clip back up (clear_near's own resupply-on-fail: the earlier
                // fights drain it) and reshoot, until the line clears.
                for round in 0..4 {
                    if killed { break; }
                    let _ = w.resupply(&format!("servant {si}'s captor chase {round}"));
                    let (cpos, gone) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let c = &sim.drones.list[ci];
                        (c.pos, c.corpse || c.health <= 0.0)
                    };
                    if gone { killed = true; break; }
                    eprintln!("[03] captor {ci} chase round {round} at ({:.1},{:.1},{:.1})", cpos[0], cpos[1], cpos[2]);
                    w.walk_nav_to(cpos, &format!("onto servant {si}'s captor (chase {round})"), 3.5);
                    killed = w.shoot(ci, &format!("servant {si}'s captor (chase {round})"));
                }
                assert!(killed, "servant {si}'s captor #{ci} died to the chase");
            } else {
                eprintln!("[03] servant {si}: no captor bound (ch {rescue_ch:#x} off) — the save rule reads an empty link as saved [M 0x15cb78:24-36]");
            }
            w.wait(30);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[si];
        assert!(!d.corpse, "servant {si} survived the rescue");
        assert!(d.state != 0x01 && d.state != 0x0a,
            "servant {si} left WAIT_SWITCH/HOSTAGE [M the save path], got {:#x}", d.state);
        eprintln!("[03] servant {si} post-save state {:#x}", d.state);
        assert!(sim.drones.channels.on(rescue_ch as u32),
            "the save raised servant {si}'s rescue channel {rescue_ch:#x} [D p6; M 0x1441c0:11-18]");
        if difficulty >= 3 && si == 0 {
            // 00 Agent (real-health-00agent-02-03-3): the green run's post-rescue west-lawn leg
            // started from the chase's end (38.34,0.72) and its nav threads the building's east
            // side through the (39.0,12.9) door; from the rally the nav detours through the
            // (35.6,8.7) door instead and that E stalls the leg 29x3000 ticks [L the first rally
            // run's ledger]. The player walks the chase's last leg: restore the green
            // continuation point (the flank's own nav link, walked there in the green run).
            w.walk_nav_to([38.4, 5.9, 0.8], "the east flank end", 1.6);
            // real-health-00agent-03-captors: the west-lawn legs' first nav (the courtyard mid)
            // detours through the (35.6,8.7) door and its E stalls the leg 29x3000 ticks
            // [L the los-mask-repins-03 ledger]. The player opens the door he is about to need
            // before the legs: find it, walk to its threshold, E it once. No-op when no door
            // sits there (the band is 2.0) or when it already stands open.
            let stall_door = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.doors.list.iter().enumerate()
                    .find(|(_, d)| walker::dxz(d.pos, [35.6f32, 6.5f32, 8.7f32]) < 2.0)
                    .map(|(i, d)| (i, d.state, d.pos))
            };
            if let Some((di, state, dpos)) = stall_door {
                if state == nf_game::door::CLOSED {
                    eprintln!("[03] the (35.6,8.7) door #{di} sits closed — opening it before the west-lawn legs");
                    w.walk_to(Wp { at: [dpos[0], 5.9, dpos[2] - 1.1], act: Act::None, arrive: 1.0 },
                        "to the (35.6,8.7) door");
                    w.use_door(di, "the (35.6,8.7) door (the west-lawn leg's stall)");
                    w.wait(30);
                }
            }
        }
        // guard-repin-master-2 item 1 (the 06 stand_shooters_down law, d-gated): at the servant
        // leg end answer whoever hit Bond across the leg — the far shooters outside the fights'
        // plain 20 u window (#11 at d 40.2, #10 at d 30.3 [L the agent death ledger, tick
        // 51805]) plinked the legs through glass and the return fire never answered them.
        if difficulty >= 2 {
            stand_shooters_down_03(&mut w, "the servant ground shooters", 40.0);
        }
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x51), "the disc's AND(0x4f,0x50) writes obj1's channel [D]");
        assert_eq!(sim.drones.mission.states[li][o1], 4, "obj1 COMPLETE through the real monitor [M 0x1827d8:78-85]");
    }

    // 2. The front gate. The plate (kind 1, mask 4 = E-usable [M Trigger_Activate 0x1b2318:38-49])
    //    writes 0x31; the door starts locked behind it and the gate SS #502 listens on 0x31. The
    //    approach is the west lane (z≈−33..−36): the straight courtyard line crosses the garden
    //    moat ditch ((11.8,−34.5) and (16,−31) drop into the water [D the dump's 5 water cels]).
    //    Rerouted the same way as the first visit (the fix6 run: the nav from the servants'
    //    east wing to west lawn rode the upper row again and Bond fell through the moat into
    //    the void, y −105962): courtyard ground → the lane node → the band's east end, then the
    //    original anchors. UNVERIFIED on this branch (the round's window ended) — next message
    //    runs it first.
    for (name, at) in [
        ("the courtyard mid", [13.02f32, 5.81, -22.00]),
        ("the lane south", [10.41, 6.08, -28.74]),
        ("the moat east", [9.95, 5.84, -34.41]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    // los-mask-repins-03 (Agent, the measured ledger: death at tick 164663 at (3.0, 6.0, −33.3)
    // to the shotgun captain #16 skin 0x5000038 weapon 0x1c at 1.2 u, the hand gun dry clip 0
    // spare 0, at 41.2): the lane legs run the return-fire fights (#9 14.7 in 7, #11 28.0 in 4,
    // #17 14.0 in 2 — all killed) and the fights give up on the dry gun [L the run; the fights'
    // own end law], so the lane captain's first point-blank volley (under 3.0 u every round
    // hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115]) is unanswered. A player crossing the
    // moat band scavenges the fights' own drops and takes the west lane loaded (the servants'
    // ground's resupply arm; the corridor keeps him on the walked path).
    if difficulty >= 2 {
        let _ = w.resupply("the moat band (agent)");
        stand_shooters_down_03(&mut w, "the moat band shooters", 40.0);
    }
    for (name, at) in [
        ("west lawn", [0.0, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("along the moat", [8.0, 5.9, -36.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    // los-mask-repins-03 (Agent): the same rearm before the gate-plate leg — the exit-leg
    // ledger's second drain was the dry gun at the door, and the gate plate leg is the route's
    // next fight-free walk.
    if difficulty >= 2 {
        let _ = w.resupply("the west lane (agent)");
        stand_shooters_down_03(&mut w, "the west lane shooters", 40.0);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let gate_plate = sim.drones.triggers.iter()
        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x31).map(|t| t.centre())
        .expect("the gate plate");
    let gate_door = sim.drones.doors.list.iter()
        .position(|d| d.ch_lock == 0x31).expect("the locked gate door");
    let gate_ss = sim.drones.sss.list.iter()
        .position(|s| s.channel == 0x31).expect("the gate SS");
    drop(sim);
    w.walk_to(Wp { at: gate_plate, act: Act::None, arrive: 1.4 }, "to the gate plate");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x31), "the gate is shut before the plate [sister]");
        assert_eq!(sim.drones.doors.list[gate_door].state, 2, "the gate door sits closed at create [M Door_Create 0x136100 +0xf4 = 2]");
    }
    // Sister first: E on the locked door refuses (the ch_lock refusal [M Door_Activate 0x135be0 —
    // the ch_lock gate + the hit feedback]).
    let sim = w.scene.sim.as_ref().unwrap();
    let gate_at = sim.drones.doors.list[gate_door].pos;
    drop(sim);
    w.walk_to(Wp { at: gate_at, act: Act::Use { use_ticks: 90 }, arrive: 1.3 }, "E the locked gate door");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[gate_door];
        assert!(d.pos01 < 0.5 && d.state != 4, "the locked door refuses E while ch 0x31 is off [M 0x135be0] — state {:#x} pos01 {}", d.state, d.pos01);
    }
    // The plate: E writes 0x31, the door's lock channel turns on, the gate SS plays its keys —
    // a 2-key one-shot (wrap flag non-zero → the wrap disarms to state 0 [M 0x1a9948:119-128]) whose
    // pose HOLDS the end key, so pin the posed matrix leaving its rest pose.
    let ss_rest = w.scene.sim.as_ref().unwrap().drones.sss.list[gate_ss].matrix;
    w.walk_to(Wp { at: gate_plate, act: Act::Use { use_ticks: 90 }, arrive: 1.4 }, "E on the gate plate");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x31), "the plate writes ch 0x31 [D out 0x31 mask 4]");
        let ss = &sim.drones.sss.list[gate_ss];
        assert!(ss.matrix != ss_rest || ss.t > 0.0 || ss.state == 1,
            "the gate SS plays its keys on 0x31 [D ss #502 ch 0x31], got state {:#x} t {}", ss.state, ss.t);
    }
    w.wait(90);
    // Now the door opens to E and Bond walks through to the gate's north side.
    w.walk_to(Wp { at: gate_at, act: Act::Use { use_ticks: 120 }, arrive: 1.3 }, "E the unlocked gate door");
    w.walk_nav_to([14.7, 5.9, -36.8], "through the gate", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] < -36.0, "Bond is behind the front gate [route], got {:?}", sim.bond.pos);
    }

    // 3. obj0 "Retrieve Dragon Safe contents" through the REAL chain: behind the gate, E on the
    //    Dragon Safe (SP #110, script 0x06000563 [D]) spawns the kind-2 key pickup with +0x28 =
    //    0x4e [M Pickup_Create 0x18dda0:122 + the script's Entity params], the script's stream 0/1
    //    lifts it above the counter [D the keys; M Script_Interp 0x1d9a08:94-110], and stepping
    //    back clears the take's line of sight — the take writes ch 0x4e [M 0x18e650:330-333; L
    //    PCSX2 2026-10-22: the first back-step took it].
    //    The items test's repro spot faces the display case; walking there through the real path.
    w.walk_nav_to([5.90, 5.9, -42.41], "to the dragon safe", 0.35);
    {
        // Aim at the case (SP #110's body at (6.7, 6.1, −42.6) [D the SP dump]) with the real view
        // (turn/look inputs). The use is EDGE-triggered — the world-use block fires on the action
        // bits' press edge, `Input_Action(p, 0xe, 4)`, and a held E activates the world ONCE [M
        // 0x1a5460:395-403; the action machine 0x17ee88 latches per press] — so the view converges
        // FIRST with no use (the earlier hold-E loop spent its one edge while still facing away:
        // no spawn), then ONE press tick at the converged view: the proven doublecross_items_live
        // shape [L PCSX2 2026-10-22: (5.90,5.97,−42.41) yaw 1.710 pitch −0.458, one E press → the
        // card].
        let case = [6.7f32, 6.1, -42.6];
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { break; }
            let bp = sim.bond.pos;
            let dx = case[0] - bp[0];
            let dz = case[2] - bp[2];
            let dy = case[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, case).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let settled = err.abs() < 0.02 && perr.abs() < 0.02;
            let input = nf_game::player::Input {
                turn: if settled { 0.0 } else { err.clamp(-0.08, 0.08) },
                look: if settled { 0.0 } else { perr.clamp(-0.08, 0.08) },
                ..Default::default()
            };
            drop(sim);
            w.tick(input, false); // converge the view only — the edge is NOT spent yet
            if settled { break; }
        }
        // The single press at the converged view (one tick = one edge, the repro's shape).
        w.tick(Default::default(), true);
        // The stream reaches the Entity command within a couple of seconds [M 0x1d8968's 0xF0 arm].
        let mut live = false;
        for _ in 0..120 {
            w.tick(Default::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { live = true; break; }
        }
        assert!(live, "the converged single E press spawned the key pickup [D script 0x06000563 s1]");
    }
    for _ in 0..130 { w.tick(Default::default(), false); } // the reveal plays; the card lifts
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = sim.drones.pickups.iter().find(|p| p.kind == 2 && !p.gone).expect("the keycard is live");
        assert!(k.m[3][1] > 6.04, "the stream lifted the keycard to its last key (y 6.05) [D 0x06000563 s0/s1], got y {:.3}", k.m[3][1]);
        assert_eq!(k.switch_channel, 0x4e, "the keycard's take channel is obj0's leaf [D params]");
    }
    // Standing at the counter the take's LOS is blocked (as in the game [L]); step back (−x, the
    // case faces +x) and the take writes ch 0x4e [M 0x18e650:330-333].
    {
        let back = nf_game::player::Input { forward: -1.0, ..Default::default() };
        let mut taken = false;
        for _ in 0..120 {
            w.tick(back, false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e) { taken = true; break; }
        }
        assert!(taken, "backing off the counter, Bond takes the keycard and ch 0x4e goes on [L + M]");
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[li][o0], 4, "obj0 COMPLETE through the real take [M 0x1827d8:78-85]");
        assert!(sim.drones.channels.on(0x52), "the disc's AND(0x4e,0x51) writes the key gate [D]");
        assert_eq!(sim.drones.mission.states[li][o2], 3, "obj2 announced by its enable channel [M 0x1827d8:55-67]");
        assert!(!sim.drones.channels.on(0x60) && !sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x63),
            "no fail hook fired on the route [M 0x1827d8:98-117]");
        // The safe's alarm responders: guards #456/#462's enable channel IS 0x4e [D p3].
        for di in sim.drones.list.iter().filter(|d| d.team == 1 && d.switch_channel == 0x4e) {
            assert!(di.enabled, "guard with enable 0x4e responded to the key [D p3]");
        }
    }
    // los-mask-repins-03 (Agent): the safe→exit legs are fight-bearing under the game's sight
    // (the exit lane walks back into the lane's sight lines) and there is no pickup on the
    // corridor between the safe and the exit lawn [L the 164663 run: both resupply calls
    // returned false at clip 1 spare 0]. A player rearms at the safe before walking back.
    if difficulty >= 2 {
        let _ = w.resupply("the safe (agent)");
        loot_the_drops_03(&mut w, "the safe drops");
        stand_shooters_down_03(&mut w, "the safe shooters", 40.0);
    }
    // real-health-00agent-03-captors-8: the safe->exit legs run into the exit captains
    // #15/#16 and their chase cost 21.6 health across the back-through-gate nav (run 64:
    // the return fights engaged 3x and missed all — killed false, #15 still hp 12). The
    // hitters-only stand above picks nobody (neither captain had hit Bond yet), so the
    // player's shape is the lane sweep's own: shoot the guard he can SEE before stepping
    // into his field (the nearest-exposed pick, no recent-hit gate [L d3run6's law]).
    // d>=3 only — the d2 streams' exit legs stay byte-identical.
    if difficulty >= 3 {
        for (i, d) in w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate() {
            if d.team == 1 && !d.corpse && d.health > 0.0 && walker::dxz(d.pos, w.scene.sim.as_ref().unwrap().bond.pos) < 40.0 {
                eprintln!("[03] safe exit census #{i} skin {:#x} weapon {:#x} hp {:.1} at ({:.1},{:.1},{:.1})",
                    d.skin, d.weapon, d.health, d.pos[0], d.pos[1], d.pos[2]);
            }
        }
        clear_exposed_nearest_03(&mut w, "the safe exit sweep", 40.0, 4, 300, false);
        w.lenient = true;
        loot_the_drops_03(&mut w, "the safe exit sweep drops");
        scoop_drops_03(&mut w, "the safe exit sweep scoop", 12.0);
        w.lenient = false;
    }

    // 4. The exit: E on the Load Point loads 07000004 through the real reload path (the exit
    //    lawn first — the direct line rides the slope without settling).
    // real-health-00agent-03-captors-8: the gate door auto-closed behind the safe visit —
    // the exit's A* could not route back through it and detoured WEST around the courtyard
    // (run 62's exit moat-node nav walked to (-1.83,-35.51) where the first visit's same nav
    // reached (7.48,-33.71) direct), straight past the exit captain #15's stand at
    // (-0.5,5.8,-32.8): his point-blank pistol (23.4 in 7 hits at d 2.6) finished Bond at
    // 171891 while the return fight was still converging (err 0.918 at t0 — the nav left Bond
    // facing the walk direction, and at 00 Agent the autoaim bit is closed [M Autoaim_HardMul
    // 0], so the raw-view convergence lost the race). A player re-opens the door he came in
    // through before walking back out: the exit nav then routes the direct gate line, 9+ u
    // clear of #15's stand. d>=3 only — the d2 streams' exit legs stay byte-identical.
    if difficulty >= 3 {
        let gstate = { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.doors.list[gate_door].state };
        eprintln!("[03] the gate door state {gstate:#x} before the exit legs");
        if gstate == nf_game::door::CLOSED {
            eprintln!("[03] the gate door sits closed — re-opening it for the exit (the A* west detour past #15)");
            w.use_door(gate_door, "the gate door (the exit re-open)");
            w.wait(30);
        }
        // The measured A* (run 63): even with the gate door open, the exit moat-node nav
        // from the safe heads NORTH-WEST (Bond died at (1.51,-37.30) mid nav 1) — the graph
        // prefers the west line out of the safe room and #15 intercepts it point-blank.
        // A player walks out the way he came: back through the gate door to the through-gate
        // spot the first visit walked, EAST of #15's west choke — the exit's moat-node route
        // then starts from there (6.2 u, clear).
        w.walk_nav_to([14.7f32, 5.9, -36.8], "back through the gate (the exit)", 1.6);
        // real-health-00agent-03-captors-9 (the -8 ledger lever 2, the exit fight geometry):
        // the fights' plain taps at the gate spot never landed — run 63's NF_RF_TRACE: the
        // fight on #16 CONVERGED (err 0.000 at t60, 12.5 u) and the in-hand shell MISSED (hp
        // 24.0 untouched), because the 0x1c's own pellet scatter (0.0448 rad [L ee_start_05
        // weapon_data] = +-0.54 u at 12.5 u) is a lottery against a ~0.4 u part box even
        // converged. A player picks the gun whose aimed round has no such cone: the 0x2 heavy
        // (the -6 ledger's own stand-off pick; ~8 a tap [L d3run52]) with the aim-bit
        // zero-spread act [M Bullet_init 0x12c5a0:139-170], and squares up on the gate band
        // where #16 stood still (his fire-stop root [M drone.rs ZERO_ROOT 0x03] gives the
        // stand its still window). The stand's own gates keep it safe: no exposed part (UNSEEN)
        // or a still-less charger bails inside the cap and the plain clear_near below still
        // runs. d>=3 only — the d<3 streams stay byte-identical.
        for _ in 0..8 {
            let wnow = { let sim = w.scene.sim.as_ref().unwrap(); sim.gun.weapon };
            if wnow == 0x2 { break; }
            w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            eprintln!("[03-ammo] at the through-gate stand: gun {:#x} clip {} spare {} hp {:.1}",
                sim.gun.weapon, sim.gun.clip, sim.gun.spare, sim.drones.health.health);
        }
        for k in 0..4 {
            let near = {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.team == 1 && d.enabled && d.present && !d.corpse && d.health > 0.0
                        && (d.pos[1] - b[1]).abs() < 2.5 && walker::dxz(d.pos, b) < 18.0)
                    .min_by(|a, c| walker::dxz(a.1.pos, b).total_cmp(&walker::dxz(c.1.pos, b)))
                    .map(|(i, _)| i)
            };
            let Some(di) = near else { break };
            let (hp, pos) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                (d.health, d.pos)
            };
            eprintln!("[03] the through-gate aimed sweep round {k}: drone #{di} hp {hp:.1} at ({:.1},{:.1},{:.1})",
                pos[0], pos[1], pos[2]);
            if !aimed_stand_03(&mut w, di, "the through-gate aimed sweep", 900, false) { break; }
        }
        // The chasers (#15/#16) cross the safe->gate line behind him (run 64: 21.6 health
        // across the pre-leg nav, the fights engaged 3x and missed all). From the CONVERGED
        // gate spot a player squares up and fights what followed: clear_near's full fights
        // (the plain w.shoot shape — the same shape that killed the charger #14 and the
        // pocket cluster) — not the nav-leg reactive taps that never converged.
        clear_near(&mut w, 16.0, "the through-gate chasers");
        w.lenient = true;
        loot_the_drops_03(&mut w, "the through-gate drops");
        scoop_drops_03(&mut w, "the through-gate scoop", 12.0);
        w.lenient = false;
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    lane_doors_south(&mut w, "the lane doors south (exit)", false);
    cross_lane_doors(&mut w, true, "the exit");
    // nb-scan-all retune, Operative (d1) lever (this branch, the 2026-11-18 ledger): the
    // spacing scan walks the exit-lawn cluster INTO Bond during the last legs — #5 (skin
    // 0x500003a) was fought only AFTER his own 19-hit volley (40.5) because the plain pick
    // answers only a guard who has LANDED hits, and #3/#4 closed to 2.1-2.7 u to finish the
    // run at tick 63923. A player clears the door cluster from across the lawn BEFORE walking
    // into its point-blank band: the aimed stands (the aim-bit zero-spread act [M Bullet_init
    // 0x12c5a0:139-170]) at 7-14 u where d1's pain scale 0.5 halves the trade. d1-only: the
    // d2/d3 streams keep their pinned exit legs.
    if difficulty == 1 {
        clear_exposed_nearest_03(&mut w, "the exit lawn east guard", 40.0, 4, 300, true);
    }
    // The exit lawn on the nav's own nodes from the doorway, walked direct (from the doorway
    // line the A* cuts the corner at (22.3, 10.9) and wedges on the garden wall [L the walk log]).
    let exit_lawn_legs: [[f32; 3]; 12] = [
        [15.58f32, 5.81, -18.43], [15.65, 5.69, -15.69], [15.70, 4.92, -14.13], [15.70, 4.85, -12.07],
        [20.02, 4.84, -6.42], [23.00, 4.84, -1.09], [23.27, 4.74, 0.97], [23.20, 5.75, 7.11],
        [22.99, 5.74, 9.94], [23.24, 5.73, 11.53], [24.23, 5.73, 15.36], [20.86, 5.73, 15.90],
    ];
    for (k, at) in exit_lawn_legs.iter().take(5).enumerate() {
        w.walk_to(Wp { at: *at, act: Act::None, arrive: 0.8 }, &format!("the exit lawn {k}"));
    }
    // The east lawn's sniper class (skin 0x5000036, the 40.0-a-hit rounds): #6's single round
    // cost 20.0 at d 13.7 on the same ledger — answered BEFORE the east-lawn legs walk his
    // sight line.
    if difficulty == 1 {
        clear_exposed_nearest_03(&mut w, "the east lawn sniper", 40.0, 4, 300, true);
    }
    for (k, at) in exit_lawn_legs.iter().skip(5).take(3).enumerate() {
        w.walk_to(Wp { at: *at, act: Act::None, arrive: 0.8 }, &format!("the exit lawn {}", k + 5));
    }
    // The door cluster (#3/#4/#5 at (25.7,14.1)/(22.8,14.5)/(20.8,10.3) [L the ledger's death
    // positions]): cleared from (23.20,7.11) at 3-10 u before legs 9-11 walk past them at
    // 1-3 u, where the d1 fights' 19-hit trades live.
    if difficulty == 1 {
        clear_exposed_nearest_03(&mut w, "the exit lawn door cluster", 14.0, 6, 0, false);
    }
    for (k, at) in exit_lawn_legs.iter().skip(8).enumerate() {
        w.walk_to(Wp { at: *at, act: Act::None, arrive: 0.8 }, &format!("the exit lawn {}", k + 8));
    }
    w.walk_to(Wp { at: [18.5, 5.9, 13.0], act: Act::None, arrive: 1.6 }, "the exit lawn");
    // guard-repin-master-2 item 1 (the 06 stand_shooters_down law, d-gated): the exit legs are
    // the second measured drain — the death spot (23.09, 6.37, 10.54) sits on the east-lawn →
    // exit-lawn line, with #11 (d 40.2) and #10 (d 30.3) still alive and shooting from the
    // courtyard through glass [L the agent death ledger]. Answer them from the exit lawn
    // before the E.
    if difficulty >= 2 {
        stand_shooters_down_03(&mut w, "the exit lawn shooters", 40.0);
    }
    // level-03-loadpoint-push: the plate-rest pin, walked on the real path (the lawn leg above,
    // then the plate's own spot — no teleport). The disc's invisible "endleveldoor" blocker
    // (placement #505, kind 0x3b, target obj map 50 #344: a 50-tri wall plane at x 17.979
    // spanning the plate's z band, surface byte 0x04, plus the small doorframe mesh [D
    // coll-obj]) rides the port's dynamic set under the game's own admission law [M
    // Collide_PickObj 0x1eae80 — the object's obj+0xF0 = glist+0x44 = 0x80000 word carries no
    // 0x20 skip bit; build_LinkToRoom 0x1c9680 links it through Control_BuildWorldSph's sphere
    // centre (18.06, 6.71, 16.04) into the lawn cel; Collide_Pick 0x1eb320:314-329 walks the
    // straddled cels' object chains]. The game's walk radius 0.55 [M 0x194df0's 0x3f0ccccd]
    // holds Bond's spine 0.55 east of the wall plane — x 18.529, 0.42 off the plate marker's
    // centre — which is exactly the port's rest. The pre-ride master had no DynObject collision
    // and let him stand INSIDE the marker slab; the game's collision says the wall holds.
    w.walk_to(Wp { at: [exit[0] + 0.422, exit[1] - 0.7, exit[2]], act: Act::None, arrive: 0.15 },
        "onto the Load Point plate (the wall-held spot)");
    w.wait(90);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = &sim.bond;
        let dxz = walker::dxz(b.pos, exit);
        assert!((b.pos[0] - (17.979f32 + 0.55)).abs() < 0.06,
            "the endleveldoor wall holds the spine at 0.55 east of its plane [M 0x194df0 radius; D coll-obj 50/344], got x {:.3}", b.pos[0]);
        assert!((b.pos[2] - exit[2]).abs() < 0.2, "Bond rests at the plate's z band, got z {:.3}", b.pos[2]);
        assert!((dxz - 0.422).abs() < 0.08,
            "the rest sits 0.42 off the plate centre — the game's own hold, not a port drift, got {dxz:.3}");
    }
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E the exit");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0004), "the exit loads 07000004 [D param 0]");
    eprintln!("[03] full walk: {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[03] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[03] stuck: {s}"); }
}

/// THE FULL ROUTE ON REAL HEALTH (`damage_log.rs::with_real_damage`): the same walked route,
/// the top-up off — no health, armour or ammo writes anywhere — and Bond's death panics with
/// the damage log (who, weapon, range). Auto-aim is the app default.
/// nb-scan-all retune (this branch): the fights ride the glass-fights laws (the d2 sister's
/// own wrapper — head-hunting, the glass sight, hurting-first). The spacing scan walks the
/// exit-lawn cluster (#3/#4/#5) INTO Bond's last legs, and under the plain fights the run
/// died at tick 63923 (#5 answered only after his own 19-hit volley, 40.5; #4's 0x12 rounds
/// at 2.1 u finished it [L the 2026-11-18 run]) because the plain sight treats the door
/// cluster's intact glass as opaque — the proactive exit-lawn clears found no exposed boxes
/// and the fights engaged only inside the point-blank band. The game's own rounds fly through
/// intact glass [D the mat-0x10 flight; damage_log.rs exposed_impl], so the sight rule and
/// the fight order are the player's: answer the shooter who is actually hurting him, take the
/// clear head box first, and clear the cluster from across the lawn (the d1-gated clears
/// below). No damage, health or ammo writes anywhere.
#[test]
fn walk_the_full_route_and_complete_every_objective_real_damage() {
    walker::with_real_damage_glass_fights(walk_the_full_route_and_complete_every_objective);
}

/// Sister of the full walk's key gate: one objective's channel alone never opens the gate —
/// the AND needs both inputs [D the kind-3 out-0x52 list 4e,51].
#[test]
fn the_key_gate_stays_shut_without_both_objectives() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    // obj0's channel only: the AND(0x4e,0x51) holds 0x52 off [D].
    sim.drones.channels.set(0x4e, 1, sim.drones.frame);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x52), "0x4e alone never opens the gate [D AND list 4e,51]");
    assert!(!sim.drones.channels.on(0x51), "0x51's own AND(0x4f,0x50) is still missing an input [D]");
    // obj1's sub-channels alone: same.
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("reload");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.channels.set(0x4f, 1, sim.drones.frame);
    sim.drones.channels.set(0x50, 1, sim.drones.frame);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.channels.on(0x51), "AND(0x4f,0x50) completes obj1's channel [D]");
    assert!(!sim.drones.channels.on(0x52), "0x51 alone never opens the gate [D AND list 4e,51]");
}

/// The servants are protected: killing one raises ch 0x60 and the standing fail hook fails the
/// mission with "Killed Civilians" [M Mission_MonitorObjectives 0x1827d8:98-104; the death channel
/// is the placement's own [D the dump]].
#[test]
fn killing_a_servant_fails_the_mission() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    let si = sim.drones.list.iter().position(|d| d.skin & 0xffff == 0x2b && d.team == 3).expect("a servant");
    let at = sim.drones.list[si].pos;
    drop(sim);
    w.walk_nav_to([at[0], at[1] - 0.3, at[2]], "to the servant", 2.0);
    w.shoot(si, "the servant");
    for _ in 0..120 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.list[si].corpse, "the servant died to real fire");
    assert!(sim.drones.channels.on(0x60), "the placement's death channel 0x60 rose [D]");
    // The mission layer's own records (the events vec is drained per step by the view path — the
    // persistent globals are the evidence): the standing hook's label, the fail state, ch 99.
    assert_eq!(sim.drones.mission.fail_label, 0x0400_0034, "the Killed Civilians fail label [M 0x1827d8:98-104]");
    assert_eq!(sim.drones.mission.state, 2, "MissionState 2 (the fail arm) [M 0x182c60:45-57]");
    assert!(sim.drones.channels.on(99), "the mission-fail channel rose [M mission.rs ch.set(99)]");
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty): the same walked
/// route at the menu's Agent pick — GameState+0x28 = 2 through the front-end's launch path [M
/// C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e =
/// "Agent"]. On this level group the pick scales Bond's pain by Plr_DMod_Normal 0.7 instead of
/// Easy 0.5 and the guards' hit chance by DroneFiring_Accuracy_Normal 0.8 instead of Easy 0.5
/// [M Player_HandlePain 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 01-04
/// group]; the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input
/// only — the load path is the only difference.
#[test]
fn walk_the_full_route_and_complete_every_objective_real_damage_agent() {
    // fx-muzzle-repin-2: the fights' pick is the fight-back lever. The corrected-stream Agent run
    // died at t62511 with #5 (0x500003a, 0x1c) at 60.4 across 16 hits while the torso-nearest
    // return-fire taps took him 12 -> 7 across 12 fights — the attrition law the head-hunting
    // opt-in documents (the torso-nearest taps take 3-5 rounds a 12-health guard while his own
    // volley lands): one head hit drops him [M NDrone2_HitDamage 0x145558]. The 4.0 stand-off and
    // the plain walks' pinned timelines keep their pick.
    // los-mask-join: the glass-fight rule (head-hunting + the glass sight + hurting-first). Under
    // the game's sight masks [M 0x176f18 mask 0xa27] the courtyard guards engage from the lane
    // doors on (guard #10 at the lane-3 leg), the plain sight rule left the fights unanswered,
    // and the walk ran dry before the servants and wedged at the door-8 mouth [L the masked run].
    // los-mask-repins-03 (the re-route, measured on this branch): the game's sight moved the
    // lane fights — the ledgered death was tick 164663 at (3.0, 6.0, −33.3) to the shotgun
    // captain #16 skin 0x5000038 weapon 0x1c at 1.2 u, the hand gun dry (clip 0 spare 0) at
    // 41.2, on the exit moat-node leg; the fights' own dry cycle found no armed weapon on the
    // walked-path corridor and the fights gave up [L the run]. The route now scavenges the way
    // a player does, all d≥2-gated level-local: a rearm + shooters stand on the moat band and
    // the west lane (the gate approach), the door-8 cluster's guns looted right after the
    // mouth fight (#18 0x1c, #2/#19 0xe [L the gas-rng-draws-6 probe]), and a rearm + loot +
    // shooters stand at the safe before the exit legs. The re-routed run kills both exit
    // lane captains (#15/#16, killed true at 151277/151381) and finishes with the exit fired
    // [L this run]. No damage, health or ammo writes anywhere; the resupply corridor is the
    // walker's walked-path rule.
    walker::with_real_damage_glass_fights(|| walk_the_full_route_and_complete_every_objective_at(2))
}

/// 00 Agent-difficulty sister of `..._real_damage_agent` (real-health-00agent-02-03-04): the
/// same walked route at the menu's 00 Agent pick — GameState+0x28 = 3 through the front end's
/// launch path [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 2 carries 3;
/// Txt 0x14f = "00 Agent"]. On this level group the pick scales Bond's pain by Plr_DMod_Hard
/// 1.2 instead of Agent's Plr_DMod_Normal 0.7, the guards' hit chance rides
/// DroneFiring_Accuracy_Hard 1.0 instead of 0.7 and their rounds by DroneDamage_Hard 0.8; the
/// autoaim bit closes at 00 Agent (Autoaim_HardMul 0) [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 01-04 group]; the min-difficulty
/// placements pass Drone_Create's gate [M 0x136be0:49]. Real input only — the load path is the
/// only difference.
///
/// The route ledger (real-health-00agent-03-captors, -2/-3/-4/-5/-6/-7/-8/-9): the frontier
/// moved 53672 (the wedge-ladder sniper) -> 71192 (the dry chase) -> 106888 (the pocket ambush)
/// -> 151214 (the post-rescue chase) -> 172900/172244 (the exit legs) -> GREEN. The keepers, in
/// the order they landed: the direct_sidestep_03 ground sidestep at the lane wedge (the A* rode
/// the building's upper row into sniper #6's windows); the pre-crossing + lane-north aimed
/// stands (the lane cluster's fire lines answered from the mouth line, the kill-loot cycle
/// re-arming between stands); the saddle-cell mouth approach (the si-0 way keeps #14 asleep
/// through the trio fight); the north-door standoff + funnel (E-open the door himself, fight
/// the charger at the pass); the cluster funnel v3 (the pre-fight top-up + the 12-u scoops —
/// the cluster dies at the pocket, both servants save, hp untouched through the rescue); the
/// gate re-open + the back-through-the-gate pre-leg (the safe's exit A* otherwise detours west
/// past #15); the safe exit sweep + census. The -8 round proved the exit race unwinnable from
/// the gate band with the in-hand 0x1c (run 63: the fight on #16 converged err 0.000 at 12.5 u
/// and the shell MISSED — the 0x1c's own pellet scatter 0.0448 rad [L ee_start_05 weapon_data]
/// is a lottery against a ~0.4 u part box even converged), and the -9 round is the fix: the
/// through-gate stand cycles to the 0x2 heavy and runs the aimed stands (the aim-bit
/// zero-spread act [M Bullet_init 0x12c5a0:139-170]) on what stands within 18 u before the
/// plain clear_near. Measured (runs 64-66): the walk runs 172244 -> 175712 and completes every
/// objective — the Load Point loads 07000004 — with bond 16.2 at the end (18 hits for 83.8
/// whole-route; the upstream floor is 23.4 at the exit, exactly one #16 volley cluster, the
/// 41.2 -> 23.4 gap's owner being #14's single 17.8 charger round at the north-door funnel —
/// the -8 ledger's live check (a) answered from the trace). The d<3 streams stay byte-identical
/// (every -9 arm is d>=3-gated).
#[test]
#[ignore = "nb-scan-all retune, frontier t8396 -> t30603 -> t71946 (this branch): the d3 door-shut (1 press, both lane leaves CLOSED) got the mid-courtyard crossing through at 98.2, escalating_cooldowns stopped the ditch re-fights, and the ditch-floor aimed stands after each scoop killed the advanced ditch machine gun #12 (his 24.0 landed before he fell); Bond now dies t71946 at (37.85,6.38,-0.24) on the door-8 mouth approach: drone #18 (skin 0x5000036, 0x1c) landed 13.6 across 19 chip hits at 9.4 u and stood at hp 12 — the fights' attrition vs the door-8 pocket cluster (the green -9 ledger's si-0 saddle approach kept #14 asleep through the trio fight; #18's pocket is the same shape). Next lever: the aimed stands from the saddle mouth line before the pocket nav (the -9 'north-door standoff + funnel' shape), and the mouth door's own shut arm while the stands run. [L the 2026-11-18 retune-nb-03 runs]"]
fn walk_the_full_route_and_complete_every_objective_real_damage_00agent() {
    walker::with_real_damage(|| walk_the_full_route_and_complete_every_objective_at(3));
}

/// Cheap route probe (no walking): does ANY nav route reach the door-8 mouth / the south
/// pocket from the saddle WITHOUT crossing the (39.0,12.9) doorway band (the door whose open
/// channel wakes #14 [L d3run50: the hits began during the mouth nav's own door-open leg])?
/// Dumps the A* node chains for the approach starts and every nav node near the door band.
#[test]
#[ignore = "probe: the mouth approach's nav routes (no walking)"]
fn probe_mouth_approach_routes() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    let saddle = [23.5f32, 6.3, 8.4];
    let mouth = [35.6f32, 6.4, 3.4];
    let door_south = [38.69f32, 6.37, 10.96];
    let north_lawn = [39.81f32, 6.37, 13.52];
    for (name, from, to) in [
        ("saddle->mouth", saddle, mouth),
        ("door-south->mouth", door_south, mouth),
        ("north-lawn->door-south", north_lawn, door_south),
        ("saddle->door-south", saddle, door_south),
        ("duel-line->mouth", [22.3f32, 5.9, 9.6], mouth),
    ] {
        let pts = walker::nav_route(&sim, from, to);
        eprintln!("[03-probe] {name}: {}", match &pts {
            Some(p) => p.iter().map(|q| format!("({:.1},{:.1},{:.1})", q[0], q[1], q[2])).collect::<Vec<_>>().join(" -> "),
            None => "NO ROUTE".to_string(),
        });
    }
    eprintln!("[03-probe] nodes near the (39.0,12.9) door band (dxz < 8):");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        if p.kind & 0xd != 0 { continue; }
        for (ni, n) in p.nodes.iter().enumerate() {
            if walker::dxz(n.pos, [39.0f32, 6.4, 12.9]) < 8.0 {
                eprintln!("[03-probe] path {pi} node {ni} at ({:.2},{:.2},{:.2}) kind {:#x}", n.pos[0], n.pos[1], n.pos[2], p.kind);
            }
        }
    }
    eprintln!("[03-probe] nodes near (40.8,11.0) east of the door band (dxz < 6):");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        if p.kind & 0xd != 0 { continue; }
        for (ni, n) in p.nodes.iter().enumerate() {
            if walker::dxz(n.pos, [40.8f32, 6.4, 11.0]) < 6.0 {
                eprintln!("[03-probe] path {pi} node {ni} at ({:.2},{:.2},{:.2})", n.pos[0], n.pos[1], n.pos[2]);
            }
        }
    }
    // The east-strip descent: move_test the legs from the last north node (39.93,13.55) down
    // x ~40.0-40.2 (east of the closed leaf plane) to the south node (40.16,9.78), then the
    // mouth. If every leg passes, Bond descends WITHOUT the doorway (and #14 never wakes).
    let strip: [([f32; 3], [f32; 3]); 6] = [
        ([39.93, 6.37, 13.55], [40.05, 6.37, 13.0]),
        ([40.05, 6.37, 13.0], [40.10, 6.37, 12.3]),
        ([40.10, 6.37, 12.3], [40.16, 6.37, 11.0]),
        ([40.16, 6.37, 11.0], [40.16, 6.37, 9.78]),
        ([40.16, 6.37, 9.78], [38.12, 6.37, 9.82]),
        ([38.12, 6.37, 9.82], [35.6, 6.4, 3.4]),
    ];
    for (a, b) in strip {
        let (res, _) = sim.nav.move_test(&sim.world, a, b, false, false);
        eprintln!("[03-probe] move_test ({:.1},{:.1})->({:.1},{:.1}): {}", a[0], a[2], b[0], b[2], res);
    }
    // Wider crossings of the doorway band z 13.55->12.23 at x 40.6/41.0 (east of the leaf's
    // east edge?) and x 38.4/38.0 (west of the leaf's west edge?).
    let cross: [([f32; 3], [f32; 3]); 8] = [
        ([39.93, 6.37, 13.55], [40.6, 6.37, 13.3]),
        ([40.6, 6.37, 13.3], [40.6, 6.37, 12.3]),
        ([40.6, 6.37, 12.3], [40.6, 6.37, 11.0]),
        ([40.6, 6.37, 11.0], [40.16, 6.37, 9.78]),
        ([39.93, 6.37, 13.55], [41.0, 6.37, 12.9]),
        ([41.0, 6.37, 12.9], [41.0, 6.37, 10.5]),
        ([41.0, 6.37, 10.5], [40.16, 6.37, 9.78]),
        ([39.93, 6.37, 13.55], [38.4, 6.37, 12.5]),
    ];
    for (a, b) in cross {
        let (res, _) = sim.nav.move_test(&sim.world, a, b, false, false);
        eprintln!("[03-probe] move_test cross ({:.1},{:.1})->({:.1},{:.1}): {}", a[0], a[2], b[0], b[2], res);
    }
    // The direct south-lawn chain (west of the wall corner) to the mouth.
    let south: [([f32; 3], [f32; 3]); 6] = [
        ([23.5, 6.37, 8.4], [25.5, 6.37, 6.5]),
        ([25.5, 6.37, 6.5], [27.5, 6.37, 5.5]),
        ([27.5, 6.37, 5.5], [30.0, 6.37, 4.8]),
        ([30.0, 6.37, 4.8], [33.0, 6.37, 4.5]),
        ([33.0, 6.37, 4.5], [34.5, 6.37, 4.0]),
        ([34.5, 6.37, 4.0], [35.6, 6.4, 3.4]),
    ];
    for (a, b) in south {
        let (res, _) = sim.nav.move_test(&sim.world, a, b, false, false);
        eprintln!("[03-probe] move_test south ({:.1},{:.1})->({:.1},{:.1}): {}", a[0], a[2], b[0], b[2], res);
    }
    // The leaf's span: the north door's own fields.
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        if walker::dxz(d.pos, [39.0f32, 5.4, 12.9]) < 3.0 {
            eprintln!("[03-probe] door #{i} at ({:.2},{:.2},{:.2}) state {:#x} pos01 {:.2} ch_lock {:#x}",
                d.pos[0], d.pos[1], d.pos[2], d.state, d.pos01, d.ch_lock);
        }
    }
}

/// The door-8 cluster's LOAD-state probe (gas-rng-draws-6; run with `--ignored --nocapture`):
/// report-5's named first step — the staged pre-servant-nav mouth-clear's anchor
/// (`dxz < 6.0` of (38.7, 6.2, 0.2)) never fired because the cluster (#18/#19/#2) loads
/// DISABLED behind the leaves at its OWN spots and only walks to (38.7, 0.2) after the door
/// opens. This probe dumps every hostile's load position/state at tick 100 (before any leg)
/// so the filter can be re-anchored to the real spots. Cheap (no walking).
#[test]
#[ignore = "probe: dump the door-8 cluster's load state at tick 100"]
fn probe_door8_cluster_load_state() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(100);
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.team != 1 { continue; }
        eprintln!("[03-probe] #{i} skin {:#x} at ({:.2},{:.2},{:.2}) enabled {} present {} corpse {} state {:#x} hp {:.1} weapon {:#x}",
            d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.present, d.corpse, d.state, d.health, d.weapon);
    }
    let door8 = sim.drones.doors.list.iter().enumerate()
        .find(|(_, dr)| walker::dxz(dr.pos, [35.6f32, 2.1, 2.7]) < 4.0);
    if let Some((di, dr)) = door8 {
        eprintln!("[03-probe] door #{di} at ({:.2},{:.2},{:.2}) state {:#x} pos01 {:.2}",
            dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.pos01);
    } else {
        eprintln!("[03-probe] no door within 4 of (35.6, 2.1, 2.7) — dump all doors:");
        for (di, dr) in sim.drones.doors.list.iter().enumerate() {
            eprintln!("[03-probe] door #{di} at ({:.2},{:.2},{:.2}) state {:#x} pos01 {:.2}",
                dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.pos01);
        }
    }
}

/// Cheap probe (no walking): the saddle-area armour pickup (kind 3 [M pickup::kind::ARMOUR],
/// the -8 ledger's "one more upstream save" candidate — armour absorbs the #14 charger's
/// burst before the health ledger). Dumps the pickup's exact spot, the nav routes to it from
/// the saddle / the duel line / the wake spot, and the nav nodes within 6 u of it.
#[test]
#[ignore = "probe: the saddle armour pickup (kind 3) reachability"]
fn probe_saddle_armour_pickup() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(100);
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, p) in sim.drones.pickups.iter().enumerate() {
        if p.gone { continue; }
        if walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], [21.5f32, 9.2, 11.6]) < 10.0 {
            eprintln!("[03-probe] pickup #{i} kind {} weapon {:#x} rounds {} at ({:.3},{:.3},{:.3}) ch {:#x}",
                p.kind, p.weapon, p.rounds, p.m[3][0], p.m[3][1], p.m[3][2], p.switch_channel);
        }
    }
    eprintln!("[03-probe] all pickups within 25 u of the saddle (23.5,6.3,8.4):");
    for (i, p) in sim.drones.pickups.iter().enumerate() {
        if p.gone { continue; }
        if walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], [23.5f32, 6.3, 8.4]) < 25.0 {
            eprintln!("[03-probe] pickup #{i} kind {} weapon {:#x} rounds {} at ({:.3},{:.3},{:.3}) ch {:#x}",
                p.kind, p.weapon, p.rounds, p.m[3][0], p.m[3][1], p.m[3][2], p.switch_channel);
        }
    }
    let saddle = [23.5f32, 6.3, 8.4];
    let duel = [22.3f32, 5.3, 10.9];
    let wake = [26.9f32, 6.4, 16.1];
    let armour = [21.47f32, 9.17, 11.59];
    for (name, from) in [("saddle->armour", saddle), ("duel-line->armour", duel), ("wake->armour", wake)] {
        let pts = walker::nav_route(&sim, from, armour);
        eprintln!("[03-probe] {name}: {}", match &pts {
            Some(p) => p.iter().map(|q| format!("({:.1},{:.1},{:.1})", q[0], q[1], q[2])).collect::<Vec<_>>().join(" -> "),
            None => "NO ROUTE".to_string(),
        });
    }
    eprintln!("[03-probe] nav nodes within 6 of the armour spot:");
    for (pi, p) in sim.nav.paths.iter().enumerate() {
        if p.kind & 0xd != 0 { continue; }
        for (ni, n) in p.nodes.iter().enumerate() {
            if walker::dxz(n.pos, armour) < 6.0 {
                eprintln!("[03-probe] path {pi} node {ni} at ({:.2},{:.2},{:.2}) kind {:#x}", n.pos[0], n.pos[1], n.pos[2], p.kind);
            }
        }
    }
}
