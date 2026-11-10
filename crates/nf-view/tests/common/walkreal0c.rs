//! walkreal-0c shared prologue: the main walk's legs 1-9c, MOVED VERBATIM from
//! walkreal_0700000c_live's `walk_the_whole_mission_from_the_start_to_the_0d_handoff`
//! (same load, same ticks, same order) so a second test can START on the gate band by
//! REPLAYING the game's own route instead of teleporting to it (the eastroute test's
//! former start write). The two documented stand-ins inside — the trench exit and the
//! mezzanine reach — carry their reason + sisters inline exactly as in the main test.

#[path = "walker_b2.rs"]
pub mod walker;

pub use walker::{Act, Walker, Wp, wrap_pi};

thread_local! {
    /// `with_yard_aimed_stands`'s switch: the yard's deck-sniper stands run on this test's
    /// thread only (libtest runs each test on its own thread, so the top-up walk, the
    /// eastroute replay and the upward-shot proofs on master's plain behaviour are untouched).
    static YARD_AIMED_STANDS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// `with_far_sniper_stands`'s switch (the real-health walks only): the return-fire fights
    /// answer the level's FAR snipers (#39 0x5000093 and #20 0x5000087, both weapon 0x1e,
    /// hp 10, at d 61-90 [L the death ledgers]) — the shooters window opens to their stand-off
    /// (95, 13) and the fights run the still-gate aimed stand (`dmg.sniper_stands`, the
    /// yard_stand_aimed shape). The flags ride the walk from the FIRST leg: the snipers open
    /// up on the band approach INSIDE the prologue (the measured 13461 death), so a flag
    /// block after `walk_spawn_to_the_band_at` returns never runs. Off = master behaviour
    /// (the plain 20 u window, the plain stands) for every other caller.
    static FAR_SNIPER_STANDS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run a walk with the far-sniper fights enabled (OPT-IN, the real-health walks — see the
/// thread-local's doc). The plain master replay and the probe tests keep master behaviour.
pub fn with_far_sniper_stands<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { FAR_SNIPER_STANDS.with(|c| c.set(false)); } }
    FAR_SNIPER_STANDS.with(|c| c.set(true));
    let _reset = Reset;
    f()
}

/// Run a walk with the yard's deck-sniper stands enabled (OPT-IN, level-0c-yard-climb-real-
/// health-2): the real-health run's killer chain is the deck snipers (weapon 0x1e, -16 a hit)
/// sitting 10.7-11 u ABOVE the yard floor — outside the shared shooters window's 7.5 vertical
/// reach (common/damage_log.rs), the plain fights never answer them, and #18 ((36.4, 12.1,
/// -208.8), hp 10, a STATIONARY sentry [L the real-health run's yard census]) dealt 51.8 of
/// 100 in 4 hits from d ~34 u before killing Bond ON the ladder at y 7.95 [L the predecessor
/// run]. The game's own climb has no pain gate to hide behind [M Player_Climb 0x196310 - the
/// stick advances the climb, the only up-block is the 0x20 head-hit flag;
/// Player_HandlePain 0x1902c8 writes no substate; Player_CollLadder 0x196118:17's sole health
/// read is the grab gate] — only death ends it [M Player_CheckForDeath 0x190810] — so a real
/// player kills the deck before climbing: the sniper 0x1e is in the boot loadout [D the boot
/// loadout], and the AIMED act's spread is zero [M Bullet_init 0x12c5a0:139-170 — mul = 0
/// only with the aim bit; the 0x1e record carries 0x400000, flags68 0x408050 [L
/// ee_start_05.bin @ 0x2c10b8]] where the HIP cone (spread base +0x24 = 100.0 → 0.14 rad =
/// 4.9 u of scatter at 35 u) misses the ~0.5 u chest box the way the game's does — the
/// player-upward-shot root cause, pinned by upward_shot_0700000c_live on master.
#[allow(dead_code)]
pub fn with_yard_aimed_stands<R>(f: impl FnOnce() -> R) -> R {
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { YARD_AIMED_STANDS.with(|c| c.set(false)); } }
    YARD_AIMED_STANDS.with(|c| c.set(true));
    let _reset = Reset;
    f()
}

/// Trace helpers (scratch): the gun's state for the no-bullet-spawned line.
fn s_gun_debug(w: &Walker) -> String { format!("{:?}", w.scene.sim.as_ref().unwrap().gun.weapon) }
fn s_gun_fire_state(w: &Walker) -> bool { w.scene.sim.as_ref().unwrap().gun.fire_state }

/// One aimed-0x1e stand on drone `di` with the STILL GATE. The aimed act's zero spread
/// [M Bullet_init 0x12c5a0:139-170] puts the round exactly along the view, but the round
/// FLIES ([M] Bullet_update 0x12e7b0 — the round steps its `speed` per tick), and a guard
/// that steps during the flight is not where the view was — no lead. Measured (this
/// session's probe, level-0c-yard-climb-real-health-3): converged aimed rounds (err 0.000,
/// perr 0.000, blocked false) at the deck sentry #18 never landed while it wiggle-stepped
/// ±0.1 u at (36.36, 12.06, -208.79); the same stand from the same spot LANDED the run the
/// sentry stood still (probe A round 2: hp 10.0 → -50.0). A player holds the shot until the
/// guard pauses. So: converge, tap only after the target has held still STILL_NEED ticks.
/// The direct number-key switch through the real input path [M Player_WeaponChange mode 0];
/// the view levelled after (the fights' own idiom). Never writes health, armour or ammo.
fn yard_stand_aimed(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    for _ in 0..120 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.gun.weapon == 0x1e { break; }
        sim.step_bond(nf_game::player::Input { switch_to: Some(0x1e), ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    // level-0c-yard-climb-real-health-4: the aim-RAISE loop. Measured (this session): the
    // stand's aim press at t0 fired its edge INTO the switch anim (switch_state 3) and the
    // raise never queued — the aim bit stayed false for the whole stand (360+ ticks, gun
    // idle [L the a04i run's mid-crossing trace]), so every round rode the hip cone. The
    // game queues the raise ONLY on the aim-PRESSED edge from the idle machine
    // [M 0x1a24c8:113-152 — `(uVar8 & 1) == (bVar5 & 1)` skips the arms, the raise arm runs
    // on the 0→1 edge; the port's queue gate scene.rs 4707 needs switch_state 0 + the idle
    // script too], so a player RELEASES the aim after the switch, lets the weapon settle,
    // and RE-PRESSES. Wait for switch_state 0 (no input), then hold aim until the bit sets
    // (the raise plays ~len/1.25 ticks; its finish sets the bit [M 0x1a2e38:554-561]).
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
    let killed = {
        let trace = std::env::var_os("NF_RF_TRACE").is_some();
        let mut di = di;
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
        let mut aim: Option<[f32; 3]> = None;
        let mut hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
        let mut last_hit = 0u32;
        let mut still = 0u32;
        let mut last_pos = [0.0f32; 3];
        let mut have_pos = false;
        let mut trace_impact_frame = 0u32;
        for t in 0..cap {
            let (input, fire, reload) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                if d.corpse || d.health <= 0.0 || !d.present { return true; }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 { return false; }
                // The still gate: root motion only (the box sway rides the anim; the root is
                // what carries the chest out of the round's path during the flight).
                let moved = have_pos && ((d.pos[0] - last_pos[0]).powi(2)
                    + (d.pos[1] - last_pos[1]).powi(2) + (d.pos[2] - last_pos[2]).powi(2)).sqrt() > 0.03;
                have_pos = true;
                last_pos = d.pos;
                still = if moved { 0 } else { still.saturating_add(1) };
                if d.health < hp0 { hp0 = d.health; last_hit = t; }
                if t - last_hit > 400 { return false; }
                // Re-pick the exposed part every 10 ticks with the STRICT gun-ray set (the
                // fights' lane idiom, `DamageLog::exposed_part_gun_rays`): the round leaves the
                // muzzle low-right of the eye, and the measured #15 case is the predictor gap —
                // the plain EYE ray read Some while every round died on the deck structure
                // (5 rounds, hp 10.0 never moved [L this session's pre-climb stand]); the
                // muzzle stand-in rays demand the flight's own lane clear.
                if t % 10 == 0 { aim = walker::DamageLog::exposed_part_gun_rays(&sim, di); }
                if aim.is_none() { unseen += 1; if unseen > 60 { return false; } } else { unseen = 0; }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let at = aim.unwrap_or([d.pos[0], d.pos[1] + 0.3, d.pos[2]]);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let desired_yaw = dx.atan2(dz);
                let desired_pitch = (dy / h).atan();
                // level-0c-yard-climb-real-health-4: the AIM-MODE convergence. With the aim bit on,
                // the fired round's direction is the VIEW PLUS the cursor offsets [M scene.rs
                // free-aim: the cursor follows the stick (`want = cursor − turn/0.76966`,
                // `+look·π/2/0.57724`, clamped ±0.296/±0.4), the offsets ease toward
                // (−cx·0.76966, +cy·0.57724) and the fired dir = the view + the eased offsets —
                // the same view+cursor round the 07000011 walk pinned (precise_fights' doc)] — so
                // converging the VIEW alone (the old err/perr) left the shot on the cursor: the
                // measured rounds steered −12° off (aoff −0.216) and died on the wall behind
                // #15 [L the a04k trace]. Drive the CURSOR to the position that puts the effective
                // aim on the target, and gate the tap on the EFFECTIVE error.
                let yaw_err = { let mut a = desired_yaw - sim.bond.yaw;
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let pitch_err = desired_pitch - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
                let cx_want_raw = -yaw_err / 0.76966;
                let cy_want_raw = pitch_err / 0.57724;
                let cx_want = cx_want_raw.clamp(-0.296, 0.296);
                let cy_want = cy_want_raw.clamp(-0.4, 0.4);
                let cur = sim.cursor;
                // The drive uses the UNCLAMPED target: at the window edge the overflow becomes
                // the free-aim spill (scene.rs 3066-3067 rewrites the input to the clamp
                // overflow) and turns the VIEW — the clamped drive fed zero there and the aim
                // stalled with the cursor pinned at (±0.296, ±0.4) [L the a04m trace: the err
                // frozen at −0.924 for 300+ ticks].
                let turn = ((cur[0] - cx_want_raw) * 0.76966).clamp(-0.08, 0.08);
                let look = ((cy_want_raw - cur[1]) * 0.57724 / std::f32::consts::FRAC_PI_2).clamp(-0.08, 0.08);
                // The effective aim (what the fired round obeys): the view + the eased offsets.
                let aoff = sim.drones.aim_offset;
                let err = { let mut a = yaw_err - aoff[0];
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let perr = pitch_err - aoff[1];
                // Another live hostile on the line, short of the target: the game's bullets hit
                // any drone [M Drone_CollisionHandler 0x140968] — shoot the blocker (the
                // fights' shoot_blockers idiom; the yard top carries several 0x1e guards).
                let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
                let u = [dx / len, dy / len, dz / len];
                let mut blocker: Option<usize> = None;
                let blocked = sim.drones.list.iter().enumerate().any(|(k, o)| {
                    if k == di || !o.present || o.corpse || o.health <= 0.0 { return false; }
                    let c = [o.pos[0] - eye[0], o.pos[1] + 0.6 - eye[1], o.pos[2] - eye[2]];
                    let s = c[0] * u[0] + c[1] * u[1] + c[2] * u[2];
                    if s <= 0.0 || s >= len - 0.3 { return false; }
                    let q = [c[0] - u[0] * s, c[1] - u[1] * s, c[2] - u[2] * s];
                    let on_line = (q[0] * q[0] + q[2] * q[2]).sqrt() < 0.6 && q[1].abs() < 1.2;
                    if on_line && blocker.is_none() { blocker = Some(k); }
                    on_line
                });
                if blocked && self_blocker_hostile(&sim, blocker) {
                    let k = blocker.unwrap();
                    if trace { eprintln!("[0c] yard stand ({label}) t{t}: line blocked by live hostile #{k} — retarget"); }
                    di = k;
                    hp0 = sim.drones.list[k].health;
                    last_hit = t;
                    aim = None;
                    unseen = 0;
                    continue;
                }
                let reload = sim.gun.clip == 0;
                // level-0c-yard-climb-real-health-4: the AIM-BIT gate. Measured (this session's
                // NF_RF_TRACE shot trace): the stand's taps fired with `self.aiming == false` —
                // the aim-raise anim had not finished [M the aim bit is set only after the
                // aim-raise anim finishes — Player_SetWeaponAnimObj case 0xf, scene.rs's auto-aim
                // comment] — so every round rode the HIP cone (the 0x1e's spread base 100.0 [L
                // ee_start_05.bin @ 0x2c10b8 +0x24]: 2-3° of scatter at d 16 — the measured
                // rounds died 1.2-1.3 u off, on the wall BEHIND #13 at (-4.20,12.58,-221.83) and
                // (-4.26,13.04,-221.78) [L the impact trace], while the probe's AIMED round from
                // the same class of pose HIT #13 part 4 [L walkreal_probe0c_lip]). The trigger
                // also kept the gun in its fire script (fire_state true t88-t344), so the raise
                // never ran. A player holds L1 WITHOUT tapping: the raise finishes, the bit sets,
                // and the tap is then a zero-spread round. Gate the tap on the bit.
                let fire = sim.aiming && !reload && unseen == 0 && !blocked && err.abs() < 0.01 && perr.abs() < 0.01
                    && still >= STILL_NEED && t.saturating_sub(last_tap) >= 8;
                if trace && t % 60 == 0 {
                    eprintln!("[0c] yard stand ({label}) t{t}: tgt #{di} hp {:.1} at ({:.2},{:.2}) still {still} aim {:?} err {err:.3} perr {perr:.3} clip {} spare {} hp {:.1} atgt {:?} aoff {:?} aimbit {} fire_state {} cursor ({:.3},{:.3})",
                        sim.drones.list[di].health, sim.drones.list[di].pos[0], sim.drones.list[di].pos[2],
                        aim.map(|a| [a[0], a[2]]), sim.gun.clip, sim.gun.spare, sim.drones.health.health,
                        sim.drones.aim_target, sim.drones.aim_offset, sim.aiming, sim.gun.fire_state, cur[0], cur[1]);
                }
                (nf_game::player::Input { turn, look, aiming: true, ..Default::default() }, fire, reload)
            };
            if reload {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, true, false);
                w.total_ticks += 1;
                w.after_step();
                for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
            } else if fire {
                last_tap = t;
                let pre = w.scene.sim.as_ref().unwrap().drones.player_bullets.len();
                let (vyaw, vpitch) = { let s = w.scene.sim.as_ref().unwrap(); (s.bond.yaw, s.bond.pitch) };
                let aiming_bit = w.scene.sim.as_ref().unwrap().aiming;
                w.tick_fire(input);
                if trace {
                    let s = w.scene.sim.as_ref().unwrap();
                    if let Some(b) = s.drones.player_bullets.get(pre).or_else(|| s.drones.player_bullets.last()) {
                        eprintln!("[0c] yard stand ({label}) t{t}: SHOT from ({:.2},{:.2},{:.2}) dir ({:.4},{:.4},{:.4}) | view yaw {vyaw:.4} pitch {vpitch:.4} view_dir ({:.4},{:.4},{:.4}) aiming_bit {aiming_bit} spread {:?}",
                            b.pos[0], b.pos[1], b.pos[2], b.dir[0], b.dir[1], b.dir[2],
                            vyaw.sin(), vpitch.sin(), vyaw.cos(), s.drones.player_spread);
                    } else {
                        eprintln!("[0c] yard stand ({label}) t{t}: SHOT but NO bullet spawned (gun state {:?} fire_state {})",
                            s_gun_debug(w), s_gun_fire_state(w));
                    }
                }
            } else {
                w.tick(input, false);
            }
            // level-0c-yard-climb-real-health-4 (trace): what did this tick's rounds do? The
            // probe (walkreal_probe0c_lip, scratch) fires the SAME shot from the SAME pose and
            // HITS #13 (part 4) — so the real stand's rounds dying means the live pose/differs;
            // the impact events name the death point.
            if trace && w.total_ticks != trace_impact_frame {
                trace_impact_frame = w.total_ticks;
                let imps: Vec<String> = w.scene.sim.as_ref().unwrap().drones.player_impacts.iter()
                    .map(|i| match i {
                        nf_game::drone::PlayerImpact::Drone { drone, part, at } =>
                            format!("DRONE #{drone} part {part} at ({:.2},{:.2},{:.2})", at[0], at[1], at[2]),
                        nf_game::drone::PlayerImpact::World(p) =>
                            format!("WORLD at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]),
                    }).collect();
                if !imps.is_empty() {
                    let s = &w.scene.sim.as_ref().unwrap();
                    eprintln!("[0c] yard stand ({label}) t{t}: impacts {} | bond ({:.2},{:.2},{:.2}) eye {:.2} crouch {:.2} yaw {:.3} pitch {:.3} weapon {:#x}",
                        imps.join(" | "), s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.bond.eye, s.bond.crouch_drop, s.bond.yaw, s.bond.pitch, s.gun.weapon);
                }
            }
        }
        w.scene.sim.as_ref().unwrap().drones.list[di].health <= 0.0
    };
    w.dmg.aimed_fights = prev;
    for _ in 0..90 {
        let p = w.scene.sim.as_ref().unwrap().bond.pitch;
        if p.abs() < 0.01 { break; }
        w.tick(nf_game::player::Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[0c] the yard sniper stand ({label}): drone #{di} killed {killed}, health {:.1}",
        sim.drones.health.health);
    killed
}

/// The blocker retarget's gate: only a live hostile (team 1) takes the round — the shared
/// `shoot_blockers` rule ([M] the fights' own gate, damage_log.rs stand_and_fire).
fn self_blocker_hostile(sim: &nf_view::scene::Sim, blocker: Option<usize>) -> bool {
    blocker.is_some_and(|k| {
        let o = &sim.drones.list[k];
        o.team == 1 && o.enabled && o.present && !o.corpse && o.health > 0.0
    })
}

/// The yard's deck-sniper stands (OPT-IN, `with_yard_aimed_stands` — a no-op for every other
/// caller). Labels:
/// - "mid-crossing" (called after the yard's first west legs, BEFORE the open-ground beat
///   costs health): the deck-emplacement stand — the deck sentry #18 at (36.36, 12.06,
///   -208.79) [L this session's dumps; the -2 census's twin #36 is present false, enabled
///   false on this tree — the PAIR reading does not hold and the round-vs-twin theory is
///   retired] — killed by the still-gate stand. Measured exposure [L this session]: aim None
///   from the secret door (d ~37.3, elevation 15.6° — the stand can never land there, the
///   old at-the-door sweep never fired a round), Some from the ladder base (d 33.3, 18.1°)
///   and from the mid-crossing legs (d ~24.6-32, 19-23°).
/// - "at-the-door" (the secret door, before the yard legs): the east-ledge sweep — the yard's
///   OTHER snipers (#20/#37, weapon 0x1e, (61.3/60.4, 9.7, -160/-157) [L this session's death
///   log — they dealt 2 × -16 during the door approach/first legs from d ~59, outside every
///   fight window]) get one capped still-gate stand each. A landing round kills a 10-hp
///   sniper [L the census].
/// - "pre-climb" (the ladder base): the emplacement stand again if #18 survived the crossing,
///   then the suppression sweep — every other alive weapon-0x1e within 48 u gets one capped
///   still-gate stand: the patroller #15 ((17.7, 12.2, -216.8) [L this session's dump]) and
///   the walkway-deck #1 ((13.0, 12.1, -196.0) [L the census]) acquire Bond late and never
///   pass the recent-hit pick, and #1's close beat at the mantle is what killed the 3.8 hp
///   Bond [L the predecessor's death at (9.25, 12.12, -188.53)]. Short caps: these are
///   suppression (a landing round kills a 10-hp sniper [L the census hp 10.0]).
/// - "ladder-top": the same sweep at deck level (Bond is LEVEL with the deck there; #1's
///   first beat lands ~110 ticks after the mantle — BEFORE any recent-hit window could pick
///   him [L the frame-10681 beat in this session's death log] — so the sweep does not wait
///   for a hit).
/// The recent-hitter pass (all labels): stands on whoever of the alive weapon-0x1e hostiles
/// within 48 u is actually hitting Bond (a hit in the last 600 frames — the snipers'
/// observed beat is 170-360 frames [L the real-health run's fight traces]) and whose STRICT
/// gun rays clear from here (`DamageLog::exposed_part_gun_rays`).
pub fn yard_sniper_stands(w: &mut Walker, label: &str, max: usize) {
    if !YARD_AIMED_STANDS.with(|c| c.get()) { return; }
    // The east-ledge / pre-climb / ladder-top sweep: every alive weapon-0x1e hostile within
    // `range` u gets one capped still-gate stand (the strict gun-ray aim keeps a covered
    // guard from eating the clip — the #15 measurement).
    let sweep = match label {
        "at-the-door" => Some((64.0f32, 900u32, 3usize)),
        "pre-climb" | "ladder-top" | "ring-corner" | "west-edge" | "west-ledge" => Some((48.0, 700, 3)),
        "lip-1" | "lip-2" | "lip-3" | "lip-4" => Some((48.0, 300, 3)),
        _ => None,
    };
    if let Some((range, cap, count)) = sweep {
        for _ in 0..count {
            let di_opt = {
                let sim = w.scene.sim.as_ref().unwrap();
                let b = sim.bond.pos;
                (0..sim.drones.list.len()).find(|&i| {
                    let d = &sim.drones.list[i];
                    if !(d.weapon == 0x1e && d.health > 0.0 && d.present && !d.corpse && d.team == 1 && d.enabled) { return false; }
                    let dxz = ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt();
                    if dxz > range { return false; }
                    // The emplacement is the mid-crossing/pre-climb pass's own target; from the
                    // door it is not exposed at all (aim None [L the probe]) — the gun rays
                    // keep this sweep honest either way.
                    walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
                })
            };
            let Some(di) = di_opt else { break };
            yard_stand_aimed(w, di, label, cap);
        }
    }
    // The deck emplacement for the yard-floor labels: the still-gate stand re-runs until the
    // sentry drops (each stand waits out the wiggle; the proof + this session's probe A both
    // landed within a few rounds once the sentry paused).
    if label == "pre-climb" || label == "mid-crossing" {
        for _ in 0..3 {
            let di_opt = {
                let sim = w.scene.sim.as_ref().unwrap();
                (0..sim.drones.list.len()).find(|&i| {
                    let d = &sim.drones.list[i];
                    d.weapon == 0x1e && d.health > 0.0 && d.present && !d.corpse
                        && (d.pos[0] - 36.4).abs() < 2.5 && (d.pos[2] + 208.8).abs() < 3.5 && d.pos[1] > 8.0
                })
            };
            let Some(di) = di_opt else { break };
            yard_stand_aimed(w, di, label, 2400);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let left: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.weapon == 0x1e && d.health > 0.0 && d.present && !d.corpse
                && (d.pos[0] - 36.4).abs() < 2.5 && (d.pos[2] + 208.8).abs() < 3.5 && d.pos[1] > 8.0
        }).collect();
        eprintln!("[0c] the deck emplacement after the aimed stands ({label}): {left:?} still alive");
    }
    for _ in 0..max {
        let di_opt = {
            let sim = w.scene.sim.as_ref().unwrap();
            let now = sim.drones.frame;
            let b = sim.bond.pos;
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { return false; }
                if d.weapon != 0x1e { return false; }
                let dxz = ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt();
                if dxz > 48.0 { return false; }
                let hitting = sim.drones.player_hits.iter().rev()
                    .take_while(|h| now.saturating_sub(h.0) <= 600)
                    .any(|h| h.1 == i);
                if !hitting { return false; }
                walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
            })
        };
        let Some(di) = di_opt else { break };
        if yard_stand_aimed(w, di, label, 1500) { continue; }
    }
}

/// The spawn asserts + legs 1-9c of the main walk, ending on the slot jump's band landing
/// (grounded, floor 10.2..11.7, x > 42.6). Deterministic: the caller continues from here.
/// player-upward-shot: legs 1-6 of `walk_spawn_to_the_band` — the REAL route from the level
/// start (the walkway start #1 [D]) through the laser-watch burn, the vent drop, the staircase
/// descent, the plates chain, the open secret door and the yard crossing — ending at the yard
/// ladder base (11.6, 1.2, -186.6). Same legs, same asserts, one cut point; the band walk calls
/// this and continues at leg 7. No behaviour change for any caller.
pub fn walk_spawn_to_the_yard_ladder_base(iso: &str) -> Walker {
    walk_spawn_to_the_yard_ladder_base_at(iso, 1)
}

/// The real-health walk's own way to the yard: the mezzanine, the annex vent, the corridor and
/// the secret door. Its fights and stands are tuned to this order, so it keeps one placement
/// onto the mezzanine deck (how a player climbs there is not pinned); the plain walk reaches the
/// yard on foot instead (Leg 4). Neither writes health, armour or ammo.
fn mezzanine_door_leg(w: &mut Walker) {
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [54.0, 5.8, -158.0];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (5.2..6.2).contains(&sim.bond.pos[1]),
            "the mezzanine-deck stand-in (5.44..6.03 [D reach-1 scans]) must hold Bond, got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
        assert!(sim.drones.locks.iter().any(|l| l.out as u32 == 0x54),
            "the ch-0x54 VentLatch #182 exists [D #182 p5]");
        assert!(!sim.drones.channels.on(0x54), "ch 0x54 is off before the burn [D #182]");
    }
    w.walk_to(Wp::new(54.0, 5.8, -160.8).r(0.8), "the deck south");
    // The deck's internal 0.56 step-down around z −161 holds the walk (the game's own lip-hold
    // [L 2026-10-31 walkway]); cross it with a grounded jump, real input.
    {
        let start = w.total_ticks;
        let mut jumped = false;
        for _ in 0..400u32 {
            let (p, grounded, yaw) = {
                let sim = w.scene.sim.as_ref().unwrap();
                (sim.bond.pos, sim.bond.grounded, sim.bond.yaw)
            };
            if p[2] < -164.5 && grounded { break; }
            let desired = (53.8f32 - p[0]).atan2(-165.5f32 - p[2]);
            let err = wrap_pi(desired - yaw);
            let jump = !jumped && grounded && p[2] < -160.4;
            if jump { jumped = true; }
            w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, jump, ..Default::default() }, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[2] < -163.0,
            "the deck's step-down crossing must reach the latch side (z < −163), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
        eprintln!("[0c] the deck step-down crossed at +{} ticks onto ({:.2},{:.2},{:.2})",
            w.total_ticks - start, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    }
    w.walk_to(Wp::new(53.8, 5.8, -164.8).r(0.6), "under the latch");
    // The Q-Laser burn of the VentLatch #182 through the held-fire path (the same shape as the
    // hinge burn above [M Lock_Update 0x1ae3d8:82-101]) — ch 0x54.
    // The switch press waits for the gun to settle, as a player's does: after a fight the sniper
    // can still be firing or reloading round by round, and the game refuses a change in those
    // view states [M Player_WeaponChange 0x1a6f80:33-47]. Press once it is idle (up to 900 ticks).
    for _ in 0..900 {
        let (held, busy) = { let g = &w.scene.sim.as_ref().unwrap().gun; (g.weapon, g.reload_state != 0 || g.fire_state || g.switch_state != 0) };
        if held == 0x4e { break; }
        let press = if busy { None } else { Some(0x4e) };
        w.tick(nf_game::player::Input { switch_to: press, ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the laser is held [M 0x1a7ef8 case 0x700000c]");
    {
        let latch = w.scene.sim.as_ref().unwrap().drones.locks.iter()
            .find(|l| l.out as u32 == 0x54).map(|l| l.pos)
            .expect("the ch-0x54 VentLatch #182 lock [D]");
        let mut burned = false;
        let start = w.total_ticks;
        while w.total_ticks - start <= 900 {
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(0x54) { burned = true; break; }
            }
            let fire_input = {
                let sim = w.scene.sim.as_ref().unwrap();
                let h = sim.bond.head();
                let d = [latch[0] - h[0], latch[1] - h[1], latch[2] - h[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let yaw_err = wrap_pi(d[0].atan2(d[2]) - sim.bond.yaw);
                let pitch_err = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI - sim.bond.pitch;
                nf_game::player::Input { turn: yaw_err.clamp(-0.08, 0.08), look: pitch_err.clamp(-0.08, 0.08), ..Default::default() }
            };
            w.tick_fire(fire_input);
        }
        assert!(burned, "the VentLatch #182 never burned under the held laser fire [M 0x1ae3d8:82-101]");
        eprintln!("[0c] the VentLatch #182 burned — ch 0x54 at +{} ticks", w.total_ticks - start);
    }
    // The vent SSs sweep (ch 0x54 [D #180/#184]) and the end pose clears the hole.
    {
        let mut swept = false;
        for _ in 0..300u32 {
            w.tick(nf_game::player::Input::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(ss) = sim.drones.sss.list.iter().find(|s| s.channel == 0x54) {
                if ss.state == 0 { swept = true; break; }
            }
        }
        assert!(swept, "the vent SS never swept open on ch 0x54 [D #180/#184; M SS_Update 0x1a9948]");
    }
    // Onto the vent hole and the drop into the corridor (the floor 0.90 [D secretreach]; the
    // 4.5 m fall lands inside the corridor SOUTH of the barrel row at z −150.23 [L]).
    w.walk_to(Wp::new(53.8, 5.8, -163.6).r(0.5), "onto the vent hole");
    let mut landed = false;
    for _ in 0..300u32 {
        w.tick(nf_game::player::Input { forward: 0.3, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.pos[1] < 3.0 {
            landed = true;
            break;
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(landed, "the vent drop never landed in the corridor, pos {:?}", sim.bond.pos);
        assert!(sim.bond.pos[2] < -162.0 && sim.bond.pos[2] > -165.5 && (51.5..56.0).contains(&sim.bond.pos[0]),
            "the vent drop must land inside the corridor past the barrel row (x 52..56, z −162..−165.5), got {:?}",
            sim.bond.pos);
        eprintln!("[0c] the vent drop landed in the corridor at ({:.2},{:.2},{:.2}) — past the barrel row [L z −150.23]",
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    }
    // The corridor south to the plates (the REAL walk the stand-in used to skip).
    w.walk_to(Wp::new(53.5, 1.0, -167.0).r(1.0), "the corridor south 1");
    w.walk_to(Wp::new(53.2, 1.0, -171.0).r(1.0), "the corridor south 2");
    w.walk_to(Wp::new(52.9, 1.0, -173.3).r(0.7), "the plates");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister: the corridor floor under the plates (0.90 [D this session's scan]) holds.
        let floor = sim.world.segment_hit([52.76, 3.4, -173.59], [52.76, -1.0, -173.59]).map(|(t, _)| 3.4 - 4.4 * t);
        assert!(floor.map_or(false, |y| (0.6..1.2).contains(&y)),
            "the secret-door corridor floor under the plates moved (expected 0.90 [D this session]), got {floor:?}");
        // Sister: the corridor's SOUTH end is the 2.07 step (z −176.0, x 52.0..55.6 [D this
        // session's scan]; its north face slopes 2.07 → ~1.71 over z −176.0..−175.9) —
        // pinning why the descent goes WEST through the door instead.
        let step = sim.world.segment_hit([53.8, 4.0, -176.0], [53.8, -1.0, -176.0]).map(|(t, _)| 4.0 - 5.0 * t);
        assert!(step.map_or(false, |y| (1.8..2.3).contains(&y)),
            "the corridor's south step moved (expected 2.07 [D this session]), got {step:?}");
    }
    eprintln!("[0c] the corridor walked for real through the open vent — on the plates at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 5 (REAL): the secret-door chain — the co-located plates [D #558/#559]: #559 (mask 8)
    // is the player BODY touch; #558 (mask 4) is the E-interactable arm (`Trigger_Activate`
    // 0x1b2318:23-33's type-2 `mask & 4` gate). The AND #560 [D: out 8, inputs 2, 3] raises
    // ch 8 and `Door_secret1` #544 (ch 8, 2 keys, rate 400, wrap 1 [D]) sweeps open over two
    // E pulses and holds.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(8), "the secret door's ch 8 is off before the chain runs [D #560]");
        assert!(sim.drones.channels.on(2), "the body-touch plate fires ch 2 under Bond [D #559]");
        assert!(!sim.drones.channels.on(3), "sister: the body touch alone never writes ch 3 [M 0x1b2318:23-33]");
    }
    let mut and_at = None;
    for _ in 0..120 {
        w.tick(nf_game::player::Input::default(), true);
        if and_at.is_none() && w.scene.sim.as_ref().unwrap().drones.channels.on(8) { and_at = Some(w.total_ticks); break; }
    }
    assert!(and_at.is_some(), "E on the plate never raised ch 8 (the AND of the body touch + the E arm [D #560; M 0x1b2318:23-33]) within 120 ticks");
    let mut open_at = None;
    for _ in 0..900 {
        w.tick(nf_game::player::Input::default(), true);
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.sss.list.iter().find(|s| s.channel == 8).expect("the ch-8 door SS [D #544]");
        if door.state == 0 { open_at = Some((w.total_ticks, door.pos())); break; }
    }
    assert!(open_at.is_some(), "the door never reached its spent (wrap-1 [D #544]) open pose — state {:?}",
        w.scene.sim.as_ref().unwrap().drones.sss.list.iter().find(|s| s.channel == 8).map(|s| (s.state, s.t)));
    eprintln!("[0c] the secret door swept spent open at +{:?}", open_at.map(|(t, _)| t));
    // Walk WEST through the spent door for real (the doorway is in the corridor's west wall —
    // the closed leaf sat at x ≈ 52.0 spanning z −174.3..−173.1 [D this session's x-rays]) and
    // step down onto the warehouse yard (0.69) — the REAL descent the master's stand-in
    // replaced. The opening's top clears the standing body (the frame's top surfaces ≥ 3.0
    // [D this session's scan]; the standing capsule tops at root + 0.825 ≈ 2.76 [M
    // Player_Collision 0x194df0 default arm]).
    w.walk_to(Wp::new(50.4, 1.7, -174.2).r(0.55), "west through the open secret door onto the yard");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let under = sim.world.segment_hit([sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]],
            [sim.bond.pos[0], sim.bond.pos[1] - 3.0, sim.bond.pos[2]]).map(|(t, _)| sim.bond.pos[1] - 3.0 * t);
        assert!(sim.bond.grounded && sim.bond.pos[0] < 51.5 && under.map_or(false, |y| (0.4..1.0).contains(&y)),
            "Bond must walk west through the open secret door onto the yard (0.69 [D this session's scan]), got pos {:?} under {under:?}",
            sim.bond.pos);
    }
    eprintln!("[0c] the yard reached for real through the open secret door — on the yard floor at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
}

/// The difficulty-parameterized prologue (real-health-agent-0c-0d): 1 is the byte-identical
/// master expression (every existing call site keeps its replay); 2+ loads the menu way —
/// the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty;
/// GameState+0x28 0x2a3790 → Drones::difficulty, asserted by the caller]. Agent spawns the
/// min-difficulty-2 placements [M Drone_Create 0x136be0:49] and swaps the captain flag
/// [M NDrone2_DefaultInit 0x14b300:504-535]; the route legs are untouched.
pub fn walk_spawn_to_the_yard_ladder_base_at(iso: &str, difficulty: u32) -> Walker {
    let iso = iso.to_string();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap()
    } else {
        nf_view::load_with_carry_difficulty(&iso, "0700000c.bin", false, None, difficulty).unwrap()
    });
    if difficulty >= 2 {
        // The Agent fights (opt-in by construction — no existing caller loads 2+). Measured
        // reason (the first d2 run, tick 4054 death): the plain nearest-visible pick spent
        // its 900-tick stands on shooters whose own boxes the hall slab eats — the
        // display-ground guards (#27 twice, ticks 2707/3637) shoot UP through the staircase
        // opening while the return down-rays die on its edge, and each failed stand cost
        // 26-40 health while the d 19.8 machine pistol (#30) chewed on. The lane fights'
        // strict gun-ray gate + the damage-weighted pick (the 07000013 crossing's own
        // passing law) fight whoever is actually hurting Bond from a line the gun can
        // answer [M DroneWeap_DoBulletAccuracy 0x179740:113-115 — the fights' own range law].
        w.dmg.lane_fights = true;
        // And the failed fights stop re-taking the same spot: the second d2 run still died
        // (tick 6716, the display ground) re-fighting the below-floor shooter #31 (y 3.2 —
        // the down-rays die on the floor edge) whose 240-tick cooldown put Bond back in the
        // open twice more. The escalation doubles the wait per failure; the leg's own walk
        // carries Bond out of the sight line.
        w.dmg.escalating_cooldowns = true;
    }
    // OPT-IN (`with_yard_aimed_stands`): the whole walk's fights hold the aim button —
    // `DamageLog::aimed_fights`, the zero-spread aimed act [M Bullet_init 0x12c5a0:139-170].
    // Measured reason (the first real-health run on this branch): the plain HIP fights pick the
    // deck sentry #18 themselves from the ladder base (d 34.4) and pull 5 hip rounds into the
    // 4.9 u cone — all miss (the pinned player-upward-shot symptom), the clip burns to 0, and
    // the aimed stand after it starts mid-reload standing in the lane at 3.8 hp — death. With
    // the fights aimed the same engagement lands (the upward-shot proof). The flag is OFF for
    // every other caller: the upward-shot hip pin (the dry-clip symptom test) walks this same
    // function and must keep its hip pulls.
    if YARD_AIMED_STANDS.with(|c| c.get()) { w.dmg.aimed_fights = true; }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 8.98).abs() < 2.0 && (p[1] - 15.07).abs() < 2.0 && (p[2] + 149.65).abs() < 2.0,
            "the walkway start #1 [D], got {p:?}");
        assert!(sim.drones.inventory.owned.get(0x54).copied().unwrap_or(false),
            "the camera gadget (0x54) is in the loadout [M 0x1a7ef8 case 0x700000c]");
        // The boot loadout's other members, for the record ([D loadout; no assert on the order]).
        eprintln!("[0c] the boot loadout: {:?}", (0..0x60).filter(|i| sim.drones.inventory.owned.get(*i).copied().unwrap_or(false)).collect::<Vec<_>>());
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);

    // Leg 1 (REAL, blocker 2 replaced): the walkway NORTH to the inner balcony, the LASER-WATCH
    // burn of the four hinge locks, and the vent drop through the trench — the game's own route
    // to objective 1 (the vent/pulley stand-in is gone). Evidence: the walkway floor 14.61
    // [D probe blk]; the balcony repro spot (7.61, 15.71, −113.39) [L 991]; the hinges #139-142
    // (out 14..17, (7.09/4.54, 15.70/16.87, −112.37) [D]); one contact per held tick, −3 off
    // both burn bytes from 0xff, complete when +0x105 < 4 → 84 contact ticks [M Lock_Update
    // 0x1ae3d8:82-101 + Player_WeaponFiring 0x1a5460:653-658]; the AND #505 (14,15,16,17 → 18
    // [D]) drives SS #138 ventmesh-front 16.4 → 14.76 and SS #179 VentBlockingPoly 16.3 → 20.0
    // [D #138/#179; M SS_Update 0x1a9948]; after both sweep, the y-16.3 ray crosses the vent
    // [M the SS end poses]. The trench (x 5..6.5, z −110.5..−111.5, floor 11.84 [D probe blk
    // floors]) carries plate #274 (kind 2, out 0x1b, centre (5.79, 14.09, −111.10) [D trig]) in
    // the fall path — the game fires it while the body's volume crosses [L hatch-2].
    w.walk_to(Wp::new(9.0, 15.7, -140.0).r(1.5), "the walkway north 1");
    // The east lane: the walkway carries a raised block x 5..10, z −122..−125 (top 16.49, west
    // face x 10.15 [D probe0c-lane floors+rays]) — the x 11..13 lane is the clear walk north.
    w.walk_to(Wp::new(12.0, 15.7, -118.0).r(1.5), "the walkway north 2 (east of the raised block)");
    w.walk_to(Wp::new(7.61, 15.71, -113.39).r(0.8), "the inner balcony repro spot [L 991]");
    // The laser out through the REAL switch input (the draw runs its anim; the fire gate holds
    // shots until it settles [M 0x1a5460's switching gate]).
    w.tick(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..60 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Laser equips [M 0x1a7ef8]");
    // Burn the hinges: the aim re-converges per hinge through turn/look (the player's own
    // re-aim) and the trigger HOLDS — 84 contacts each [M 0x1ae3d8:82-101]; the clip 500 [L
    // +0x92] covers 4 × 85 spends.
    for out in [14u32, 15, 16, 17] {
        let target = w.scene.sim.as_ref().unwrap().drones.locks.iter()
            .find(|l| l.out as u32 == out).map(|l| l.pos)
            .unwrap_or_else(|| panic!("the out-{out} hinge lock exists [D #139-142]"));
        let mut burned = false;
        let start = w.total_ticks;
        while w.total_ticks - start <= 500 {
            let fire_input = {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.drones.channels.on(out) { burned = true; break; }
                let h = sim.bond.head();
                let d = [target[0] - h[0], target[1] - h[1], target[2] - h[2]];
                let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let yaw_err = walker::wrap_pi(d[0].atan2(d[2]) - sim.bond.yaw);
                // the port's look convention: pitch ±1 → ±π/2 [M nf-view lib.rs --bond parser]
                let pitch_err = (d[1] / l).clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI - sim.bond.pitch;
                nf_game::player::Input {
                    turn: yaw_err.clamp(-0.08, 0.08),
                    look: pitch_err.clamp(-0.08, 0.08),
                    ..Default::default()
                }
            };
            w.tick_fire(fire_input);
        }
        assert!(burned, "hinge out {out} never burned under the held laser fire [M 0x1ae3d8:82-101]");
        eprintln!("[0c] hinge out {out} burned at +{} ticks", w.total_ticks - start);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(18), "the AND mux #505 raised ch 18 [D #505 list 14,15,16,17]");
    }
    // The SSs animate after ch 18 (rate 50 / rate 10 [D]) and each RESETS to t 0 / state 0
    // when its sweep completes (this run: t138 0.1 → 3.1 → 0 by +60) — so the completion pin
    // is the END POSE: the y-16.3 ray crossing the open vent [M the SS end poses; the same
    // ray the laser-watch test pins].
    let mut saw_sweep = false;
    let mut open = false;
    for _ in 0..300 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let ss138 = sim.drones.sss.of_placement(138).expect("ventmesh-front SS [D #138]");
            let ss179 = sim.drones.sss.of_placement(179).expect("VentBlockingPoly SS [D #179]");
            let (t138, t179) = (sim.drones.sss.list[ss138].t, sim.drones.sss.list[ss179].t);
            if t138 > 0.0 || t179 > 0.0 { saw_sweep = true; }
            if sim.world.segment_hit([5.78, 16.3, -111.7], [5.78, 16.3, -113.0]).is_none() { open = true; break; }
        }
        w.tick(nf_game::player::Input::default(), false);
    }
    eprintln!("[0c] the vent sweep ran ({saw_sweep}) and the end pose passed the ray ({open})");
    assert!(open, "the open vent never let the y-16.3 ray through [M the SS end poses]");
    eprintln!("[0c] all four hinges burned — the vent is open");
    // Through the open vent (the passage the y-16.3 ray proves) and off the lip into the
    // trench — the REAL fall crosses plate #274 mid-fall (ch 0x1b) and lands on the lower
    // walkway (11.84 [D probe blk floors]).
    w.walk_to(Wp::new(5.78, 15.7, -112.3).r(0.6), "through the open vent");
    let mut fired_at = None;
    let start = w.total_ticks;
    let landed = loop {
        let sim = w.scene.sim.as_ref().unwrap();
        let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
        if grounded && fired_at.is_some() { break true; }
        if w.total_ticks - start > 400 { break false; }
        if !grounded && fired_at.is_none() && sim.drones.channels.on(0x1b) {
            fired_at = Some(w.total_ticks - start);
        }
        let desired = (5.78f32 - p[0]).atan2(-110.8f32 - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: err.clamp(-0.08, 0.08),
            forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        w.tick(input, false);
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(landed && fired_at.is_some() && sim.bond.grounded && (p[1] - 12.93).abs() < 0.6,
            "the vent drop must fire plate #274 mid-fall (ch 0x1b [D #274; L hatch-2]) and land on the \
             lower walkway (11.84 [D probe blk floors]), got pos {p:?} fired {fired_at:?}");
        assert_eq!(sim.drones.mission.states[11][0], 4, "objective 1 completes on the #274 drop [M 0x1827d8:88-99]");
        assert_eq!(sim.drones.mission.states[11][1], 3, "ch 0x1b reveals objective 2 [L record 11 obj 1]");
    }
    eprintln!("[0c] the vent drop fired plate #274 mid-fall at +{:?} — objective 1 complete on the game's own route", fired_at);

    // Leg 2 (REAL): out of the case's trench south-west onto the 11.27 hall, the hall east
    // under the case, the upper east lane (11.27, x 13..14 [D probe]) south to the STAIRCASE
    // HEAD, the ramp descent to the display ground (6.27), the ledge north to the SUBJECT's
    // clear-line eye, and the photograph through the REAL camera switch and bullet path [M
    // CamSubject_Update 0x1bc668:14-24]. (Route floors from the crossings session's probe: the
    // direct south lane hits the 1.28
    // riser wall at z −114.09 [D probe]; the x 13..14 upper lane carries.)
    // The vent EXIT is a crawl. The shaft's south end is a pocket 1.5 high (floor 11.84, ceiling
    // 13.34 [D]) that opens into the hall: the portal between cels #212 and #285 fills it (x
    // 4.8..6.7, y 11.8..13.3, z -112.3 [D block 0x21]). The only face across it is
    // `ventblocker_01` #457 (z -112.363, surface byte 0x40), whose normal points south, so a
    // body coming from the north is behind it and passes [M Dist2Tri 0x109770: the behind-plane
    // arm returns code 99; Intersect_CylGeom 0x109da0 skips the face on 99]; the hall side stays
    // shut. Standing, the lintel at z -111.89 stops the body [L 2026-10-31 c8-stand], so Bond
    // crouches once [M Player_SSCrouch 0x1924c0:84-105], holds south and drops to the hall floor.
    // The real-health walk keeps its tuned timing here: the same 463 ticks in the shaft, then a
    // placement where the crawl lands (its fights are a pinned sequence; see mezzanine_door_leg).
    if YARD_AIMED_STANDS.with(|c| c.get()) {
        for _ in 0..463 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [5.0, 12.35, -112.9];
        sim.bond.vel = [0.0; 3];
        w.wait(10);
    } else {
        for t in 0..240 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.pos[2] < -113.0 && sim.bond.grounded { break; }
            let err = walker::wrap_pi(std::f32::consts::PI - sim.bond.yaw);
            let input = nf_game::player::Input {
                crouch: t < 2, // the toggle press; the 60-tick lockout eats the repeat [M 0x1924c0]
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
                ..Default::default()
            };
            w.tick(input, false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[2] < -112.6 && p[1] < 12.6, "the crawl must leave the shaft into the hall (floor 11.27 [D]), got {p:?}");
        for _ in 0..3 {
            w.tick(nf_game::player::Input { crouch: true, ..Default::default() }, false);
        }
        for _ in 0..40 {
            w.tick(nf_game::player::Input::default(), false);
        }
        assert!(!w.scene.sim.as_ref().unwrap().bond.crouched, "the stand-up toggle failed after the crawl");
    }
    eprintln!("[0c] the vent crawl walked — on the 11.27 hall at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w.walk_to(Wp::new(8.0, 12.4, -113.2).r(1.0), "the hall east (south of the x 6..7 pillars)");
    w.walk_to(Wp::new(13.5, 12.4, -115.0).r(1.2), "the upper east lane");
    // The STAIRCASE descent — the game's own route to the display ground, no class port
    // needed: the 0xE2 SS class is fully ported (SS_Create 0x1a9c58 / SS_Update 0x1a9948 /
    // SS_Operate 0x1a9df8 / SS_Activate 0x1a9ea8 = crates/nf-game/src/ss.rs create/tick/
    // activate), and the level's 0xE2 census [D] has NO travel lift: the obj_stor*_pully*
    // winches #351/#364/#377 carry four SAME-position keys (a ±0.026-rad wheel sway, mask 0x2
    // = the bullet contact class [M 0x1a9df8:35-37]) — dead props, not rides. The walkable
    // descent is the hall floor's own opening (the floor is MISSING at x 9..12.4 for
    // z −115.5..−131.5 [D probe rays]) crossed by the static `obj_stor2_staircase01` #359
    // (kind 0x19, bounds x 9.97..12.29, y 6.13..12.14, z −127.5..−117.1 [D placement]): the
    // landing at 11.12..11.29 (z −127..−128, level with the 11.27 hall floor) and the ramp
    // down to 6.29 at z −120 [D probe rays]. The lane walk stays EAST of x 12.5 (the shaft
    // is open to the 2.18 deep floor); the descent lands on the south ledge (6.27,
    // x 12.6..14.5 [D probe rays]) whose north end joins the display ground at z −113.5
    // (the 6.27 floor spans x 1..14.5 there [D probe rays]).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sisters pinning the route's real geometry: the shaft hole east of the lane, the
        // landing Bond is about to walk onto, and the ramp's mid surface.
        let shaft = sim.world.segment_hit([11.0, 12.0, -122.0], [11.0, 10.6, -122.0]);
        assert!(shaft.is_none(), "the stair shaft gained hall-height floor at (11, −122); the lane walk could cross it [D probe rays]");
        let landing = sim.world.segment_hit([11.2, 13.0, -128.0], [11.2, 10.6, -128.0]);
        assert!(landing.is_some(), "the stair landing (11.29 at z −128 [D probe rays]) vanished; the descent has no head");
        let ramp = sim.world.segment_hit([11.2, 12.0, -124.0], [11.2, 7.0, -124.0]);
        assert!(ramp.is_some(), "the ramp's mid surface (~8.52 at z −124 [D probe rays]) vanished; the descent would free-fall");
    }
    w.walk_to(Wp::new(13.2, 12.4, -124.0).r(1.0), "the lane south to the stair head");
    w.walk_to(Wp::new(13.0, 12.4, -128.2).r(0.8), "the stair approach");
    // West onto the landing (z −128: the hall floor 11.27 at x 12.5+, the landing 11.29..11.32
    // at x 10.5..12 — level [D probe rays]).
    w.walk_to(Wp::new(11.2, 12.35, -128.1).r(0.6), "west onto the stair landing");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 12.35).abs() < 0.4,
            "Bond stands on the stair landing (11.29 [D probe rays]), got {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
    }
    // Down the ramp (south, ~0.8 y per z [D probe rays]): targets every ~1.5 z keep the
    // steering on the surface; the body rides the slope with short airborne hops.
    for (x, z, label) in [
        (11.2f32, -126.5f32, "the ramp 1"),
        (11.2, -125.0, "the ramp 2"),
        (11.2, -123.5, "the ramp 3"),
        (11.3, -122.0, "the ramp 4"),
        (11.4, -120.8, "the ramp 5"),
    ] {
        w.walk_to(Wp::new(x, 7.0, z).r(0.9), label);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[1] < 9.0 && sim.bond.pos[2] > -122.0,
            "the ramp descent reaches the bottom landing (6.29 at z −120 [D probe rays]), got {:?}", sim.bond.pos);
    }
    // East onto the south ledge (the bottom 6.29..6.32 joins the ledge 6.27 at x 12.6 [D
    // probe rays]), then NORTH along it to the display pocket — the ledge's west edge is the
    // shaft (x 11.5..12.3 open to 2.18 at z −114..−117 [D probe rays]), so the west turn
    // happens at z −113.5 where the 6.27 floor spans the full width.
    w.walk_to(Wp::new(12.2, 7.4, -120.3).r(0.7), "the stair bottom east");
    w.walk_to(Wp::new(13.2, 7.4, -120.0).r(0.8), "onto the ledge");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 7.41).abs() < 0.5,
            "Bond stands on the south ledge (6.27 [D probe rays]), got {:?} grounded {}", sim.bond.pos, sim.bond.grounded);
        let hole = sim.world.segment_hit([11.8, 8.0, -115.5], [11.8, 1.4, -115.5]);
        let hole_y = hole.map(|(t, _)| 8.0 + t * (1.4 - 8.0));
        assert!(hole_y.map_or(false, |y| y < 3.0), "the ledge's west shaft (x 11.8, z −115.5 open to 2.18 [D probe rays]) gained floor: {hole_y:?}");
    }
    w.walk_to(Wp::new(13.2, 7.4, -116.0).r(0.9), "the ledge north 1");
    w.walk_to(Wp::new(13.2, 7.4, -113.4).r(0.8), "the ledge north 2");
    w.walk_to(Wp::new(10.5, 7.4, -113.4).r(0.9), "the display ground west");
    w.walk_to(Wp::new(5.4, 7.35, -113.0).r(0.7), "the display eye");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && (sim.bond.pos[1] - 7.41).abs() < 0.5,
            "Bond stands on the display ground at the eye spot (6.27 floor [D probe rays]), got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    eprintln!("[0c] the staircase descent walked for real — the eye on the display ground at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    let subject = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.props.subjects.iter().find(|s| s.channel == 0x1c)
            .map(|s| (s.lo, s.hi)).expect("the jetpack subjects [D #414/#454]")
    };
    eprintln!("[0c] the display eye (the staircase-routed spot) — the camera next");
    // The camera out through the REAL switch input (the old gun.switch_request write is gone).
    // The press is refused while the held gun's reload plays (Sim::step_bond's `reload_state == 0`
    // request gate); the return-fire legs can leave a reload running at the eye, so the press
    // repeats until the draw starts, the way a player presses again.
    for _ in 0..400 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.weapon == 0x54 { break; }
        let press = sim.gun.switch_request != 0x54;
        w.tick(nf_game::player::Input { switch_to: press.then_some(0x54), ..Default::default() }, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x54, "the camera is held [M 0x1a7ef8]");
    let sc = [(subject.0[0] + subject.1[0]) / 2.0, (subject.0[1] + subject.1[1]) / 2.0, (subject.0[2] + subject.1[2]) / 2.0];
    let photographed = shoot_at(&mut w, sc, 30.0, 900, "the jetpack photograph",
        &|sim| sim.drones.channels.on(0x1c));
    assert!(photographed, "the photograph never wrote ch 0x1c [M 0x1bc668:14-24]");
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for _ in 0..2 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        assert_eq!(sim.drones.mission.states[11][1], 4, "objective 2 completes [M 0x1827d8:88-99]");
    }
    eprintln!("[0c] objective 2 complete — the photograph (the staircase-routed eye spot)");

    if YARD_AIMED_STANDS.with(|c| c.get()) {
        mezzanine_door_leg(&mut w);
    } else {
    // Leg 4 (REAL): out of ST-1 on foot. The display ground's east ledge runs south to the head
    // of the 6.29 stair at z -143, which descends north to the 2.32 deck at z -137; under the
    // ledge the 2.37 deck runs north to the ramp at x 15.5..20, z -122.5, down to the outside
    // ground (y 0). West of the x 31.7 boundary wall the ground wraps round to z -122, then the
    // x 46 lane runs south past the warehouse to the yard. Walked here and checked at each end;
    // the secret door's chain is covered by interactables_0700000c_live.
    for (x, y, z, r, label) in [
        (10.5, 7.4, -113.4, 0.9, "the display ground east"),
        (13.2, 7.4, -113.4, 0.8, "back to the ledge"),
        (12.8, 7.4, -120.0, 0.8, "the ledge south 1"),
        (12.8, 7.4, -140.0, 0.8, "the ledge south 2"),
        (12.8, 7.4, -143.2, 0.45, "the ledge's south end"),
        (11.5, 7.4, -143.2, 0.45, "the stair head"),
        (11.5, 6.4, -141.0, 0.45, "down the stair"),
        (11.5, 4.0, -138.0, 0.45, "the stair's lower half"),
        (11.5, 3.4, -136.5, 0.45, "the stair foot"),
        (13.25, 3.4, -136.6, 0.45, "under the ledge"),
        (14.5, 3.4, -135.0, 0.45, "the east deck"),
        (14.5, 3.4, -129.0, 0.45, "the deck north"),
        (14.75, 3.4, -126.25, 0.45, "the deck north 2"),
        (15.25, 3.4, -123.75, 0.45, "the ramp head"),
        (17.0, 2.2, -122.5, 0.45, "down the ramp"),
        (20.0, 1.1, -122.5, 0.45, "the ramp foot"),
        (25.75, 1.1, -122.25, 0.45, "the ground east"),
        (27.5, 1.1, -121.0, 0.45, "the wall gap"),
        (27.75, 1.1, -112.25, 0.45, "north along the wall"),
        (29.5, 1.1, -108.0, 0.45, "round the wall's end"),
        (31.25, 1.1, -109.25, 0.45, "the far side"),
        (31.25, 1.1, -121.0, 0.45, "south along the wall"),
        (33.5, 1.1, -122.0, 0.45, "round the corner"),
        (45.5, 1.1, -122.0, 0.45, "the ground east to the lane"),
        (46.0, 1.1, -124.5, 0.45, "the lane head"),
        (46.0, 1.1, -136.5, 0.8, "the lane south 1"),
        (46.0, 1.4, -160.0, 0.8, "the lane south 2"),
        (46.0, 1.4, -174.2, 0.8, "the lane south 3"),
        (50.4, 1.7, -174.2, 0.55, "the yard at the secret door's west side"),
    ] {
        w.walk_to(Wp::new(x, y, z).r(r), label);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let under = sim.world.segment_hit([sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]],
            [sim.bond.pos[0], sim.bond.pos[1] - 3.0, sim.bond.pos[2]]).map(|(t, _)| sim.bond.pos[1] - 3.0 * t);
        assert!(sim.bond.grounded && under.map_or(false, |y| (0.3..1.0).contains(&y)),
            "Bond must walk out of ST-1 onto the yard (0.69 [D]), got pos {:?} under {under:?}", sim.bond.pos);
    }
    eprintln!("[0c] the yard reached on foot from the display ground — at {:?}",
        w.scene.sim.as_ref().unwrap().bond.pos);
    }

    // Leg 6a (OPT-IN, `with_yard_aimed_stands`): the east-ledge sweep FROM THE DOOR — the
    // yard's other snipers (#20/#37) dealt 2 × -16 during the door approach/first legs from
    // d ~59, outside every fight window [L this session's death log]; a player clears the
    // overlooks before walking the open ground. Then the deck-emplacement stand MID-CROSSING
    // — the old at-the-door emplacement sweep is retired with a measurement: from the secret
    // door (d ~37.3, elevation 15.6°) the emplacement is NOT exposed (aim None, the stand
    // never fired a round [L this session's probe, level-0c-yard-climb-real-health-3]). The
    // first crossing legs take Bond to spots where it IS exposed (d ~24.6-32, 19-23°), and
    // the still-gate stand lands there (the deck sentry's wiggle-step is what the gate waits
    // out). Killing #18 before the last two legs is the health budget: its crossing beat was
    // 2-3 × -16 [L the -2 runs] and 51.8 of the -1 run's 100 damage.
    yard_sniper_stands(&mut w, "at-the-door", 3);
    // Leg 6 (REAL): south-west across the yard to the ladder base.
    w.walk_to(Wp::new(48.0, 1.4, -179.0).r(1.5), "the yard south of the door");
    w.walk_to(Wp::new(40.0, 1.4, -184.5).r(2.0), "the yard west");
    yard_sniper_stands(&mut w, "mid-crossing", 2);
    w.walk_to(Wp::new(20.0, 1.4, -184.5).r(1.5), "the yard west 2");
    w.walk_to(Wp::new(11.6, 1.2, -186.6).r(1.2), "the ladder base");
    w
}

pub fn walk_spawn_to_the_band(iso: &str) -> Walker {
    walk_spawn_to_the_band_at(iso, 1)
}

/// The difficulty-parameterized band prologue (real-health-agent-0c-0d): 1 is the
/// byte-identical master expression; the legs are shared verbatim with the Agent sister.
pub fn walk_spawn_to_the_band_at(iso: &str, difficulty: u32) -> Walker {
    // Legs 1-6 (unchanged — now shared with the yard-deck-sentry test).
    let mut w = walk_spawn_to_the_yard_ladder_base_at(iso, difficulty);
    // The far-sniper opt-in (with_far_sniper_stands, the real-health walks only): the flags
    // ride the walk from the first leg — the d 61-90 0x1e snipers open up on the band
    // approach INSIDE this prologue (the measured 13461 death), so a block after the return
    // never runs. See the thread-local's doc.
    if FAR_SNIPER_STANDS.with(|c| c.get()) {
        w.dmg.shooters_window = Some((95.0, 13.0));
        w.dmg.sniper_stands = true;
    }
    // Leg 6b (OPT-IN, the yard's deck snipers, level-0c-yard-climb-real-health-2): on real
    // health the deck snipers (weapon 0x1e, -16 a hit) are the walk's killer chain — outside
    // the plain fights' 7.5 u vertical reach, never answered — so the stand kills them AIMED
    // from the strip (the upward-shot proof) before the climb; on the top-up walk and the
    // eastroute replay this is a no-op (the flag is off — the choreography stays pinned).
    // Without a landing round this stand was SUPPRESSION only (the predecessor branch) and
    // the walk stayed ignored; the aimed act [M 0x12c5a0:139-170] lands.
    yard_sniper_stands(&mut w, "pre-climb", 4);
    // Leg 7 (REAL): the ladder walk-in grab + climb (the walk substate rides the DEFAULT arm
    // r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]).
    w.walk_to(Wp::new(10.3, 1.05, -186.8).r(0.7), "the ladder approach");
    let mut climbed = false;
    for _ in 0..900 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbed = true; break; }
        // yaw = atan2(dx, dz) — forward = (sin yaw, cos yaw) — [S] unverified (Bond::look is the port's look maths; the game's heading convention is not pinned here).
        let desired = (9.669741f32 - sim.bond.pos[0]).atan2(-187.3427f32 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: if err.abs() > 0.2 { err.clamp(-0.08, 0.08) } else { 0.0 },
            forward: if err.abs() > 0.2 { 0.3 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(input, false, false, false);
        w.total_ticks += 1;
    }
    assert!(climbed,
        "walking into the yard ladder #164 never grabbed it — the walk substate must ride the switch's DEFAULT arm r 0.55 [M 0x1953f4 via the jump table @0x2fc0d0]");
    // Climb for real: hold forward until the mantle fires (the dismount at pos.y > climb_top
    // 12.139 [M Player_Climb 0x196310:54-62 + Player_CollLadder 0x196118's +0.85 top]) and the
    // walk resumes, then keep the stick forward through the lunge (+0.6 [M 0x196310:56]).
    let mut mantled = false;
    for _ in 0..2400 {
        let sim = w.scene.sim.as_mut().unwrap();
        if sim.bond.substate != nf_game::player::SS_CLIMB && sim.bond.pos[1] > 2.0 { mantled = true; break; }
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    assert!(mantled, "the yard climb never mantled past climb_top 12.139 [M 0x196118/0x196310], y {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && sim.bond.substate == nf_game::player::SS_WALK && sim.bond.pos[1] > 10.5 { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded && sim.bond.pos[1] > 10.5,
            "Bond lands on the ladder-top walkway (11.086 [D probes]) after the mantle, got {:?} grounded {}",
            sim.bond.pos, sim.bond.grounded);
    }
    eprintln!("[0c] the yard ladder climbed for real to y {:.2}", w.scene.sim.as_ref().unwrap().bond.pos[1]);

    // The ladder-top stands (OPT-IN, same flag): on the walkway Bond is LEVEL with the deck
    // snipers and #1 ((13.0, 12.1, -196.0), d 7.6 from the top [L the yard census]) — too
    // steep to shoot from the yard floor (its gun rays were false there) — killed Bond within
    // ~110 ticks of the mantle at 3.8 hp [L the real-health run's death at (9.25, 12.12,
    // -188.53), tick 9741]. The same aimed-0x1e stands, now at deck level; a no-op on the
    // top-up walk and the eastroute replay.
    yard_sniper_stands(&mut w, "ladder-top", 3);

    // Leg 8 (REAL): the patrol-ring diagonal from the ladder top to the north-west corner
    // ((6-9,−188) → (9-11,−190) → (11-12,−192) → (12-13,−194) [D probes]) and west along the
    // north-west walkway to its west edge (0..8, −208 [D probes]).
    w.walk_to(Wp::new(10.5, 11.1, -190.5).r(1.2), "the ring diagonal");
    w.walk_to(Wp::new(12.0, 11.1, -196.0).r(1.2), "the ring diagonal 2");
    w.walk_to(Wp::new(12.0, 11.1, -203.0).r(1.2), "the ring north-west corner");
    // Leg 8 (OPT-IN, same flag): the sweep at the north-west corner. The walkway-deck #1 closes
    // to ~1 u here and holds AimStandFire at ~4.6 u through the west-edge leg; his single
    // 16-point round lands outside every recent-hit window, so the plain fights never pick him.
    // A player shoots the guard standing in front of him before walking on.
    yard_sniper_stands(&mut w, "ring-corner", 3);
    w.walk_to(Wp::new(4.0, 11.1, -207.5).r(1.2), "the ring west edge");
    // Leg 8a (OPT-IN, `with_yard_aimed_stands`): the west-side sweep AT THE WEST EDGE, before
    // the wire legs — the west sniper #13 ((-14.59, 12.24, -216.35) [L this session's dump])
    // survived the ring-west-edge fight (killed false [L the real-health run]); from the
    // ladder top it is not gun-ray exposed (the sweep found no candidate [L this session's
    // run]), from the edge it is LEVEL (y 12.15 vs 12.24) and the fights' own stand fought it
    // here — a player clears the far side before stepping off the edge. Same still-gate
    // stands; a no-op when the flag is off.
    yard_sniper_stands(&mut w, "west-edge", 3);
    // Leg 8b (REAL, blocker 4 replaced): the west-gap wire crossing. The disc's own mechanism
    // [D probe blk/blk2]: the type-3 grab icon #4 (−3.30, 12.36, −207.51) sits at the west
    // edge's south end; `wire_firstsilo2mainsilos` #284's grab segment A (−4.63, 13.67, −209.88)
    // B (−8.47, 13.67, −216.52) spans the gap, and BOTH ends have landing floor: the west edge
    // (11.1) at A's side and the west ledge (11.24-11.38, x −12..−8, z −216..−218) at B's side.
    // The grab: the jump grab with the icon's ctx 3 [M Player_HandleJump 0x1916b0:118-147] or
    // the body-contact grab [M Player_CollWire 0x197130]; the ride is the strafe tree [M
    // Player_Wire 0x197658], the drop the action press [M 0x197758].
    let grabbed = {
        let mut got = false;
        let trace = std::env::var_os("NF_RF_TRACE").is_some();
        for t in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_WIRE { got = true; break; }
            let (p, grounded) = (sim.bond.pos, sim.bond.grounded);
            if trace && t % 20 == 0 {
                eprintln!("[0c wire-grab] t{t} pos ({:.2},{:.2},{:.2}) grounded {grounded} substate {:#x} yaw {:.2} level_exit {:?} mission_end {:?} disabled {}",
                    p[0], p[1], p[2], sim.bond.substate, sim.bond.yaw, sim.drones.level_exit, sim.drones.mission_end, sim.disabled);
            }
            if !grounded && p[1] < 9.0 { break; }
            let desired = (-4.6f32 - p[0]).atan2(-209.9f32 - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = nf_game::player::Input {
                turn: err.clamp(-0.08, 0.08),
                forward: if err.abs() > 0.6 { 0.0 } else { 1.0 },
                jump: grounded && p[0] < 2.0 && t > 6,
                strafe: if sim.bond.substate == nf_game::player::SS_WIRE { 1.0 } else { 0.0 },
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        got
    };
    assert!(grabbed, "the west-edge jump never grabbed wire #284 (icon ctx 3 [D #4] or the body sphere [D probe blk])");
    // Ride to B (−8.47, −216.52): strafe +1 faces wire.yaw + 3π/2 = toward B [M Player_Wire's
    // mode-7 facing]; drop with the action press inside 0.9 of B.
    let dropped = {
        let mut dropped = false;
        let trace = std::env::var_os("NF_RF_TRACE").is_some();
        for t in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] + 8.47).powi(2) + (sim.bond.pos[2] + 216.52).powi(2)).sqrt();
            if trace && t % 120 == 0 {
                // The ride's own progress line: a stall (d not shrinking) names its blocker —
                // this session's real-health run stalled at d 1.6 (the natural stop is 1.05
                // PAST B [L blockers-2's hang]) with #13 alive on the west side [L the
                // ring-west-edge fight, killed false].
                let bb = &sim.bond;
                eprintln!("[0c] the #284 ride t{t}: pos {:?} d_B {d:.2} mode {:#x} anim {:#x} at {:.1} rate {:.2} yaw {:.3} auto {:.1} disabled {} sub {}",
                    bb.pos, bb.mode, bb.wire_anim, bb.wire_anim_time, bb.wire_anim_rate, bb.yaw, bb.auto_drop, sim.disabled, bb.substate);
            }
            // The drop is the jump press inside 0.9 of B (blockers-2's green drop). The
            // FIRE-action variant (real-crossings 29b43fa9, never run on that branch) rode the
            // full 2400 ticks to the natural stop d_B ≈ 1.05 past B WITHOUT releasing [this
            // run's hang at (−8.98, 12.65, −217.44) — the y 12.65 hang root = wire 13.67 − 1.02
            // [M hang 1.0207]], so the wire release does not answer the trigger here; the jump
            // press does (blockers-2's three green drops).
            let input = nf_game::player::Input { strafe: 1.0, jump: d < 0.9, ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        dropped
    };
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dropped && sim.bond.grounded && p[0] < -6.5 && p[0] > -13.0 && p[2] < -214.5 && p[2] > -219.5
            && (p[1] - 12.4).abs() < 0.8,
            "the #284 ride must drop onto the west ledge (11.24-11.38 [D probe blk2 west map]), got {p:?} dropped {dropped}");
    }
    eprintln!("[0c] the west gap crossed on wire #284 — landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // Leg 8c (OPT-IN, `with_yard_aimed_stands`): the west-ledge sweep, before the ledge
    // diagonals — the west sniper #13 patrols to (-3.0, 12.2, -221.1), d ~6 LEVEL across the
    // gap [L this session's death log: 2 × -16 + the fatal -1.6 graze at 14308], and the
    // walk's own fights land killed false against him (the plain eye-ray aim reads Some while
    // the rounds die on the ledge lip — the same predictor-vs-muzzle-flight gap the #15 case
    // measured). The still-gate stand aims with the strict gun rays; a no-op when the flag is
    // off.
    yard_sniper_stands(&mut w, "west-ledge", 3);
    // los-mask-repins-0c scoped map: who stands near the strip now (the sight era moves the
    // guards — the ledgers' fixed (-7.07,12.24,-218.60) is a stale-tree measurement), and
    // where the strip's floor actually is (the -4 scoped map rows −218.0..−219.0 held
    // x −10..−8.5..−5.0 [L the diag dump]; re-prove it on this tree before walking it).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present { continue; }
            let dd = ((d.pos[0] - p[0]).powi(2) + (d.pos[2] - p[2]).powi(2)).sqrt();
            if dd < 10.0 && d.team == 1 && d.health > 0.0 {
                eprintln!("[0c-diag] drone {0}: pos=({1:.2},{2:.2},{3:.2}) d={4:.2} health={5:.1} corpse={6} skin={7:#x} state={8:#x}",
                    i, d.pos[0], d.pos[1], d.pos[2], dd, d.health, d.corpse as u8, d.skin, d.state);
            }
        }
        // The round-3 note, measured in-sim: cast the port's world rays (the disc's collision
        // [D]; the game's bullet pick walks the same world [M]) from Bond's landing eye to the
        // STRIP guard (the live guard nearest (-7.07,-218.60)) — a clear ray means the miss is
        // a port-side spread/lead artifact and the shot is the route; a blocked ray is the
        // game-correct miss and the re-route must go around him.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let strip: [f32; 2] = [-7.07, -218.60];
            let mut best: Option<(usize, f32)> = None;
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !d.present || d.team != 1 || d.corpse || d.health <= 0.0 { continue; }
                let dd = ((d.pos[0] - strip[0]).powi(2) + (d.pos[2] - strip[1]).powi(2)).sqrt();
                if dd < 3.0 && best.map_or(true, |(_, bd)| dd < bd) { best = Some((i, dd)); }
            }
            if let Some((gi, gd)) = best {
                let gp = sim.drones.list[gi].pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                eprintln!("[0c-diag] strip guard {gi} at ({:.2},{:.2},{:.2}) state {:#x} (off-post {gd:.2}); bond eye ({:.2},{:.2},{:.2})",
                    gp[0], gp[1], gp[2], sim.drones.list[gi].state, eye[0], eye[1], eye[2]);
                for (label, dy) in [("feet", 0.3f32), ("torso", 0.9), ("head", 1.5)] {
                    let to = [gp[0], gp[1] + dy, gp[2]];
                    let hit = sim.world.segment_hit(eye, to).map(|(t, _)| {
                        format!("at ({:.2},{:.2},{:.2})", eye[0] - (eye[0] - to[0]) * t, eye[1] - (eye[1] - to[1]) * t, eye[2] - (eye[2] - to[2]) * t)
                    });
                    eprintln!("[0c-diag] eye->{label}: {}", hit.map(|h| format!("BLOCKED {h}")).unwrap_or_else(|| "clear".into()));
                }
                // The muzzle line sits lower than the eye — the -4 session's rounds died on the
                // lip while the eye ray read Some. Cast chest-height rays from forward steps
                // toward the guard: the first clear offset is the shooting spot.
                let dir = [(gp[0] - bp[0]), (gp[2] - bp[2])];
                let dl = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt().max(1e-3);
                for off in [0.0f32, 0.4, 0.8, 1.2, 1.6] {
                    let from = [bp[0] + dir[0] / dl * off, bp[1] + 0.7, bp[2] + dir[1] / dl * off];
                    let to = [gp[0], gp[1] + 0.9, gp[2]];
                    let hit = sim.world.segment_hit(from, to).map(|(t, _)| {
                        format!("at ({:.2},{:.2},{:.2})", from[0] - (from[0] - to[0]) * t, from[1] - (from[1] - to[1]) * t, from[2] - (from[2] - to[2]) * t)
                    });
                    eprintln!("[0c-diag] chest+{off:.1}u->torso: {}", hit.map(|h| format!("BLOCKED {h}")).unwrap_or_else(|| "clear".into()));
                }
            } else {
                eprintln!("[0c-diag] no live guard within 3.0 of the strip post at the landing");
            }
        }
        eprintln!("[0c-diag] strip floor map (x -10..-5, z -215.5..-219.5 step 0.5):");
        for z in [-215.5f32, -216.0, -216.5, -217.0, -217.5, -218.0, -218.5, -219.0, -219.5] {
            let mut row = String::new();
            for x in [-10.0f32, -9.5, -9.0, -8.5, -8.0, -7.5, -7.0, -6.5, -6.0, -5.5, -5.0] {
                let f = sim.world.segment_hit([x, 13.4, z], [x, 8.4, z]).map(|(t, _)| 13.4f32 - 5.0f32 * t);
                row.push_str(&match f { Some(y) => format!("{y:5.2}"), None => "  ...".to_string() });
            }
            eprintln!("[0c-diag] z {z:7.1}: {row}");
        }
        eprintln!("[0c-diag] deep map (8.4..13.4), x -10..2 step 1.0, cols x=-10,-9,-8,-7,-6,-5,-4,-3,-2,-1,0,1,2:");
        for z in [-215.5f32, -216.5, -217.5, -218.0, -218.5, -219.0, -219.5, -220.0, -220.5, -221.0, -221.5, -222.0, -222.5, -223.0, -223.5, -224.0] {
            let mut row = String::new();
            for c in 0..13i32 {
                let x = -10.0f32 + c as f32;
                let f = sim.world.segment_hit([x, 13.4, z], [x, 8.4, z]).map(|(t, _)| 13.4f32 - 5.0f32 * t);
                row.push_str(&match f { Some(y) => format!("{y:5.2}"), None => "  ...".to_string() });
            }
            eprintln!("[0c-diag] z {z:7.1}: {row}");
        }
    }
    // los-mask-repins-0c: the shot route. The world gives a CLEAN shot from the landing to
    // the strip guard — the eye rays to his feet/torso/head AND the chest-height rays from
    // every forward offset 0..1.6 u are all clear [L the 0c-diag dump on this tree; the
    // port's collision is the disc's, so the game's bullet reaches the same [M]] — the
    // 600-tick miss was the shooter artifact, not the world. The player's play: kill the
    // 10-hp guard from the landing, then walk the straight -3 diagonals unobstructed (the
    // lip-route and floor-route repairs tried and reverted this round are recorded in the
    // ignore messages: the -4 slide now falls to the yard, the 11.24 lattice is thin tops).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let strip: [f32; 2] = [-7.07, -218.60];
        let mut target: Option<usize> = None;
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || d.team != 1 || d.corpse || d.health <= 0.0 { continue; }
            let dd = ((d.pos[0] - strip[0]).powi(2) + (d.pos[2] - strip[1]).powi(2)).sqrt();
            if dd < 3.0 && target.map_or(true, |t| dd < ((sim.drones.list[t].pos[0] - strip[0]).powi(2) + (sim.drones.list[t].pos[2] - strip[1]).powi(2)).sqrt()) {
                target = Some(i);
            }
        }
        drop(sim);
        if let Some(g) = target {
            // The ammo lever (the round-2 telemetry: the gun arrived DRY — clip 1 spare 0 [L
            // the run]): fetch ammo through the walk's own pickup walk first, then re-anchor
            // on the landing (the fetch displaces Bond), then shoot.
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let (clip, spare) = (sim.gun.clip as i32, sim.gun.spare as i32);
                drop(sim);
                if clip + spare < 4 {
                    eprintln!("[0c] dry at the landing (clip {clip} spare {spare}) — the ammo fetch");
                    if w.resupply("the strip ammo run") {
                        // The take lands gun 0x0006 (50 rounds at (-21.53,11.33,-218.54)
                        // [D the pickup dump]) in the INVENTORY — the current clip does not
                        // move [M Pickup_Handler 0x18e650's equip path] — cycle the wheel to
                        // the armed gun, then re-anchor on the landing (the fetch displaced
                        // Bond 13 u west).
                        if w.switch_to_armed("the strip gun switch") {
                            let sim = w.scene.sim.as_ref().unwrap();
                            eprintln!("[0c] armed: weapon {:#x} clip {} spare {}", sim.gun.weapon, sim.gun.clip, sim.gun.spare);
                            drop(sim);
                            w.walk_to(Wp::new(-8.21, 12.82, -215.66).r(0.5), "re-anchor on the #284 landing");
                        }
                    }
                }
            }
            // The bounded landing-shot attempt (the repro of the ledger's 600-tick miss) plus
            // the ammo telemetry: this round's shot-telemetry run measured the gun DRY at the
            // landing — clip 1 fired off-target, then clip 0 spare 0 weapon 0x54, and every
            // on-target trigger pull after that was a dry click [L the run] — while the world
            // rays (eye and chest, every forward offset 0..1.6 u) are ALL clear [L the
            // 0c-diag dump]. The miss was never the world; it was the ammo.
            for attempt in 0..2 {
                let (alive, hp) = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[g];
                    (!d.corpse && d.health > 0.0, d.health)
                };
                if !alive { break; }
                let (clip, spare) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare) };
                eprintln!("[0c] the strip guard {g} from the landing (attempt {attempt}, hp {hp:.1}, clip {clip} spare {spare})");
                if w.shoot(g, "the strip guard from the landing") { break; }
            }
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[g];
            let (clip, spare) = (sim.gun.clip, sim.gun.spare);
            eprintln!("[0c] after the landing shots: guard {g} hp {:.1} corpse {} (clip {clip} spare {spare} — the on-target pulls were dry [L the telemetry])", d.health, d.corpse);
        }
    }
    // The diagonal ledges south-east to icon #6 [D probe blk2 west map].
    yard_sniper_stands(&mut w, "lip-1", 3);
    w.walk_to(Wp::new(-6.0, 12.4, -219.6).r(1.5), "the west ledge diagonal");
    yard_sniper_stands(&mut w, "lip-1b", 3);
    w.walk_to(Wp::new(-4.5, 12.4, -221.5).r(1.5), "the west ledge diagonal 2");
    yard_sniper_stands(&mut w, "lip-2", 3);
    w.walk_to(Wp::new(-2.5, 12.4, -222.8).r(1.4), "the west ledge diagonal 3");
    yard_sniper_stands(&mut w, "lip-3", 3);
    w.walk_to(Wp::new(-1.4, 12.4, -223.6).r(1.0), "icon #6 at wire #282's B end");
    yard_sniper_stands(&mut w, "lip-4", 3);
    // REAL: grab wire #282 (`wire_silosilo2`, A (5.85, 14.11, −223.15) B (−0.25, 14.11, −223.15)
    // [D probe blk]) — the standing body at (−1.4, −223.6) is inside the wire's world sphere
    // (centre (2.80, 14.11, −223.15) r 5.05 [D probe blk]), so the walk-east contact grab fires
    // [M Player_CollWire 0x197130]; the jump grab at icon #6 (ctx 3 [D]) is the fallback.
    let grabbed = {
        let mut got = false;
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_WIRE { got = true; break; }
            let grounded = sim.bond.grounded;
            let input = nf_game::player::Input {
                forward: 1.0,
                jump: grounded,
                strafe: if sim.bond.substate == nf_game::player::SS_WIRE { -1.0 } else { 0.0 },
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        got
    };
    assert!(grabbed, "walking east at wire #282's B end never grabbed it (sphere r 5.05 [D probe blk])");
    // Ride EAST to A (5.85, −223.15): strafe −1 (mode 8 faces wire.yaw + π/2 = +X toward A) [M
    // Player_Wire's mode-8 facing]; drop inside 0.9 of A.
    let dropped = {
        let mut dropped = false;
        for _ in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] - 5.85).powi(2) + (sim.bond.pos[2] + 223.15).powi(2)).sqrt();
            // The drop is the jump press inside 0.9 of B (see the #284 note).
            let input = nf_game::player::Input { strafe: -1.0, jump: d < 0.9, ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
        }
        dropped
    };
    for _ in 0..200 {
        if w.scene.sim.as_ref().unwrap().bond.grounded { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dropped && sim.bond.grounded && p[0] > 3.0 && p[0] < 9.0 && p[2] < -220.0 && p[2] > -227.0
            && (p[1] - 12.4).abs() < 0.8,
            "the #282 ride must drop onto the (4..8, −221..−226) ledge (11.24 [D probe blk2]), got {p:?} dropped {dropped}");
    }
    eprintln!("[0c] wire #282 ridden east — landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    // The SW arc: the ledge to the arc's west end and east along it [D probes].
    w.walk_to(Wp::new(8.0, 12.4, -222.0).r(1.0), "the ledge to the arc");
    w.walk_to(Wp::new(13.0, 12.4, -218.5).r(1.2), "the arc west end");
    w.walk_to(Wp::new(24.0, 12.4, -218.0).r(1.5), "the ring south-west arc");
    // The grab line: the 1.5 arrival can stop anywhere on the arc's bend (a guard fight on the arc
    // shifts it), and a grab walked in from the north-east took the wire facing its A end, where
    // the strafe has no headway. Step back west, then walk east onto the arc's line through the
    // sphere HIT at (28, 12.3, -217.5) [D probe blk], as the proven grab did (22.65, -217.36).
    w.walk_to(Wp::new(21.3, 12.4, -217.4).r(0.4), "the arc's grab line west");
    w.walk_to(Wp::new(22.65, 12.4, -217.36).r(0.3), "the arc's grab line");
    // REAL: grab wire #283 (`wire_silo2turbine`, A (29.58, 14.44, −216.96) B (35.89, 14.44,
    // −209.44) [D probe blk]) at the arc's east end — the body sphere-HIT is confirmed at
    // (28, 12.3, −217.5) [D probe blk]; icon #12 (28.42, 12.24, −218.43, ctx 3 [D]) marks it.
    let grabbed = {
        let mut got = false;
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate == nf_game::player::SS_WIRE { got = true; break; }
            let grounded = sim.bond.grounded;
            let input = nf_game::player::Input {
                forward: 1.0,
                jump: grounded,
                strafe: if sim.bond.substate == nf_game::player::SS_WIRE { 1.0 } else { 0.0 },
                ..Default::default()
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
            // The walker's damage allowance (`Walker::tick`'s top-up), which these direct
            // `step_bond` loops bypass: a guard round on the ride must not end the crossing.
            if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
        }
        got
    };
    assert!(grabbed, "walking east at the arc's end never grabbed wire #283 (the sphere HIT [D probe blk])");
    // Ride to B (35.89, −209.44): strafe +1 (mode 7 faces wire.yaw + 3π/2 = north-east toward B)
    // [M Player_Wire]. The ride must run to the wire's END before the jump: the corner floor's
    // west edge is x ≈ 35.8 (10.89 step at z −210..−209.8, the 11.06..11.09 corner east of it
    // [D scratch probe rays; the corner 11.09..11.17 at x ≤ 40.5, z −210..−208 D probe door2]),
    // so the old `jump at d < 0.9` dropped Bond at x ≈ 35.15 — 0.65 WEST of the edge, over the
    // void (no face y 15..8 under (35.15, −209.95) [D rays]) and he fell to the yard. The
    // strafe zero rule parks the idle at d_B ≈ 0.5 [M 0x1977c0's `d_b <= 0.5`], but the playing
    // mode-7 move anim finishes its root motion and overshoots to d_B ≈ 1.1 PAST B — x ≈ 36.7,
    // over the 11.06 corner floor. Jump there (the ride's natural stop, d no longer shrinking).
    let dropped = {
        let mut dropped = false;
        let mut best_d = f32::MAX;
        let mut still = 0u32;
        for _t in 0..2400u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.bond.substate != nf_game::player::SS_WIRE { dropped = true; break; }
            let d = ((sim.bond.pos[0] - 35.89).powi(2) + (sim.bond.pos[2] + 209.44).powi(2)).sqrt();
            // Hold the jump one tick past the stop so the exit reads the overshoot pose.
            let at_stop = still >= 90;
            if d < best_d - 1e-4 { best_d = d; still = 0; } else { still += 1; }
            let input = nf_game::player::Input { strafe: 1.0, jump: at_stop, ..Default::default() };
            w.scene.sim.as_mut().unwrap().step_bond(input, false, false, false);
            w.total_ticks += 1;
            // The walker's damage allowance (`Walker::tick`'s top-up), which these direct
            // `step_bond` loops bypass: a guard round on the ride must not end the crossing.
            if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
        }
        dropped
    };
    // Settle through the REAL fall: the wire ride leaves `grounded` stale-true on the exit tick
    // (wire_tick zeroes vel and returns; the flag clears the next tick [M wire_tick 0x197758's
    // exit]), so the old `if grounded break` snapped the assert onto the HANG pose — the merge
    // RED's [35.15, 13.55, −209.95] is exactly the hang root (wire y 14.44 − 1.02 [M hang
    // 1.0207]). Wait for air, then ground.
    let mut saw_air = false;
    for _ in 0..300 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.grounded && saw_air { break; }
        if !sim.bond.grounded { saw_air = true; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
            // The walker's damage allowance (`Walker::tick`'s top-up), which these direct
            // `step_bond` loops bypass: a guard round on the ride must not end the crossing.
            if w.top_up_health { w.scene.sim.as_mut().unwrap().drones.health.health = 100.0; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dropped && sim.bond.grounded && p[0] > 33.5 && p[0] < 40.5 && p[2] < -207.0 && p[2] > -211.5
            && (p[1] - 12.2).abs() < 0.8,
            "the #283 ride must drop onto the ring SE corner (11.09..11.17 [D probe door2]), got {p:?} dropped {dropped} health {} substate {}",
            sim.drones.health.health, sim.bond.substate);
    }
    eprintln!("[0c] the south gap crossed on wire #283 — landed {:?}", w.scene.sim.as_ref().unwrap().bond.pos);

    // Leg 9 (REAL from the SE corner through the door): the district's true geometry [D, probe
    // walkreal_probe0c_door2_live]: the corner (11.10..11.17, x ≤40.5, z −210..−208) is where
    // #283's B end drops Bond; the corridor (11.09, x 38.05..39.55, z −212.75..−212) runs to
    // door2turbine #281's mouth; the E stance (39.0, 11.09, −210.8) — the live sphere (40.01,
    // 12.57, −209.69) r 1.62 within the 2.62 pick [M Collide_PickObj 0x1eae80's 0x101 arm].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let wall = sim.world.segment_hit([40.2, 13.0, -213.51], [40.2, 11.9, -213.51]);
        assert!(wall.is_some(), "the slot end wall at z −213.51 (y 12.02..14.22 [D room_turbine01 tri 463]) vanished");
    }
    // The corner to the door stance (the port's kind-0xDB doors carry no leaf collision [M
    // build_LinkToRoom 0x1c9680:63-66, Collide_Pick 0x1eb320:369-375]).
    w.walk_to(Wp::new(38.9, 11.1, -210.8).r(0.7), "the corner to the door stance");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister: pin the door closed right before the E press — if a fix opens it early, this
        // leg must change.
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert_eq!(door.state, 0x2, "door2turbine opened before the E press");
    }
    // The REAL E press at the door mouth: face the leaf, pulse E every 6 ticks.
    // [M 0x135be0: info+0x68 (ch_lock) == 0 skips the channel gate; kind 0 (obj+0xf6) closed
    //  (obj+0xf4 & 0x12 == 0) → Door_SetState(obj, 8); M 0x135618: stores obj+0xf4 = 8 and
    //  writes switch_channels[info+0x66] = (8 & 0xc != 0) → the door's ch_open 5 turns on.]
    let mut opened = false;
    for t in 0..90u32 {
        let turn = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let desired = (40.0f32 - b.pos[0]).atan2(-210.4f32 - b.pos[2]);
            walker::wrap_pi(desired - b.yaw).clamp(-0.08, 0.08)
        };
        w.tick(nf_game::player::Input { turn, ..Default::default() }, t % 6 == 0);
        if t % 6 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            if let Some(d) = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5) {
                if d.state != 0x2 { opened = true; break; }
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert!(opened, "the E press at the corridor mouth never moved door2turbine (state {})", door.state);
    }
    eprintln!("[0c] door2turbine moved on the real E press (the swing-open arm)");
    // The channel write lands on the transitions where the OLD state's 0xc bits are set [M
    // 0x135618's switch_channels store]: the E press's OPENING (8) write happens with the old
    // CLOSED (2) → stays off; the channel turns ON exactly when the swing completes
    // (set_state(OPEN=4) with the old OPENING=8). Poll for that completion.
    let mut open_settled = false;
    for _ in 0..300 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(5) { open_settled = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let door = sim.drones.doors.list.iter().find(|d| (d.pos[0] - 40.0).abs() < 0.5 && (d.pos[2] + 210.4).abs() < 0.5)
            .expect("the door2turbine #281 [D]");
        assert!(open_settled, "the door never completed its swing with ch_open 5 on [M 0x135618 via the OPEN transition], state {:#x}", door.state);
        // The ch5-on observable PROVES the OPEN transition ran (the channel only turns on from
        // an old state with the 0xc bits [M 0x135618]); the door's auto-close [D audit: #281
        // auto-close] may already have moved it to CLOSING by the time the poll breaks.
        assert!(door.state == nf_game::door::OPEN || door.state == nf_game::door::CLOSING,
            "the door state after the swing [M 0x135618], got {:#x}", door.state);
        assert!(sim.drones.channels.on(5), "the door's OPEN wrote its ch_open 5 [M 0x135618's switch_channels store]");
    }
    eprintln!("[0c] door2turbine opened on the real E press (state OPEN, ch 5 on)");
    // REAL: through the doorway (the port's kind-0xDB doors carry no leaf collision — the game
    // picks them through the ForcedList [M build_LinkToRoom 0x1c9680:63-66, Collide_Pick
    // 0x1eb320:369-375]) onto the corner.
    w.walk_to(Wp::new(39.4, 11.1, -209.6).r(0.9), "through the doorway onto the corner");
    // The corner's east boundary is a wall (x 40.01..40.37) with a 1.52-wide doorway tunnel
    // (z −208.93..−210.45, floor 11.10, ceiling 13.96 [D room_turbine01 tris 478-485 — no
    // x-face tris: open along x]); the band's west lip exists only at z ≥ −209.0 [(43.4, 10.89)
    // / (44.2, 11.17) [D map]]. Run up west of the wall, THROUGH the tunnel, and jump east of it.
    w.walk_to(Wp::new(38.6, 11.1, -209.7).r(0.5), "the jump run-up start");
    let mut jumped = false;
    for t in 0..300u32 {
        let (p, grounded) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            (b.pos, b.grounded)
        };
        // Land (after a real airborne phase) → the assert judges where.
        if jumped && grounded && t > 20 { break; }
        // Still falling deep below the band's lip → the shaft; the assert reports it.
        if !grounded && jumped && p[1] < 9.0 { break; }
        let (turn, forward) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let desired = (44.5f32 - b.pos[0]).atan2(-208.8f32 - b.pos[2]);
            let err = walker::wrap_pi(desired - b.yaw);
            (err.clamp(-0.08, 0.08), if err.abs() > 0.6 { 0.0 } else { 1.0 })
        };
        // Fire the jump while grounded just EAST of the wall's east face (x 40.37 [D]) — full
        // run speed through the 1.8-unit run-up and the tunnel.
        let jump = !jumped && grounded && p[0] >= 40.42;
        if jump { jumped = true; }
        w.tick(nf_game::player::Input { turn, forward, jump, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let under = sim.world.segment_hit([p[0], p[1], p[2]], [p[0], p[1] - 3.0, p[2]])
            .map(|(t, _)| p[1] - 3.0 * t);
        assert!(sim.bond.grounded && p[0] > 42.6 && under.map_or(false, |y| (10.2..11.7).contains(&y)),
            "the slot jump must land on the band (10.89..11.17 floor at x 43.4+ [D]), got pos {p:?} under {under:?}");
    }
    eprintln!("[0c] the slot jump landed on the band at {:?}", w.scene.sim.as_ref().unwrap().bond.pos);
    w
}

/// Fire the held weapon at a static point through the real trigger path (the walker's aim loop;
/// kept local to keep walker.rs additive).
fn shoot_at(w: &mut Walker, target: [f32; 3], range: f32, max_ticks: u32, _label: &str, done: &dyn Fn(&nf_view::scene::Sim) -> bool) -> bool {
    let start = w.total_ticks;
    let mut last_tap = 0u32;
    loop {
        if done(&w.scene.sim.as_ref().unwrap()) { return true; }
        if w.total_ticks - start >= max_ticks { return false; }
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            let eye = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = target[0] - bp[0];
            let dz = target[2] - bp[2];
            let dy = target[1] - eye;
            if (dx * dx + dy * dy + dz * dz).sqrt() > range { return false; }
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, target).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on_target = err.abs() < 0.008 && perr.abs() < 0.012 && w.total_ticks - last_tap > 15;
            let trigger = on_target;
            if trigger { last_tap = w.total_ticks; }
            let input = if trigger {
                nf_game::player::Input { aiming: true, ..Default::default() }
            } else {
                nf_game::player::Input {
                    turn: err.clamp(-0.08, 0.08),
                    look: perr.clamp(-0.08, 0.08),
                    aiming: true,
                    ..Default::default()
                }
            };
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        }
        w.total_ticks += 1;
        w.after_step();
    }
}

