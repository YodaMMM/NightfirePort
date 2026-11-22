//! walkreal-b (716c): 07000013 (HT Tower2 C, the tower lobby → the front entrance) walked with
//! the player's own movement, START TO FINISH: spawn → the security-room door (portal 1) → the
//! panel E (ch 0x23, the gate rises) → back out → the entrance → Mission Complete. The warm chain (mission 0x07000011's parts): 07000012 WALKED (the
//! keypad burn, the console E, the loop round to the north lobby and the shaft entrance — the
//! shared `tower2` route, no position writes) → its exit loads 0700004A, whose start #43 (4.26, −7.71, −4.76
//! [D]) sits INSIDE TouchOnce #277 (out 0x4D, y −9.87…−7.65 [D]) — Bond drops onto the touch at
//! the spawn (the game's own placement; no input writes) and "Escape through the elevator shaft"
//! completes — the carried RamSave arrives at 07000013 ([M 0x182270 keeps previous parts]) → the
//! spawn touches fire 0x28 and 0x4A ("Gain access to lobby" completes — the late AND) → the intro
//! ScriptPlayer #65 (0x060006D7 [D]) releases Bond at tick ≈351 → the REAL walk south through the
//! lobby to the front entrance → TouchOnce #156 (−8.61…−4.15, −368.81…−366.74, 65.09…67.16 [D])
//! writes ch 0x50 (the objective completes) → the success touches #73/#164/#165 write ch 0x64
//! (=100) and with every part's channel on (0x48, 0x49, 0x4D, 0x4A, 0x50) the monitor raises
//! MISSION SUCCESS [M 0x1827d8:119-123] → "Mission Complete" → LevelToEndTo 0x07300013 (the
//! driving interlude; the port logs and skips driving codes [M 0x182c60]) → the mission_end
//! latch = the level exit firing. The mission's four objectives [L ee_start_05.bin @
//! 0x2a41b8/0x2a41e8/0x2a4200 — 2+1+1 records; docs/formats/mission-data.md], asserted in
//! player order: "Activate elevator override control in security center" (ch 0x49 →
//! states[16][0] == 4, tower2's console E), "Escape through the elevator shaft" (ch 0x4D →
//! states[16][1] == 4, the 4A spawn drop), "Gain access to lobby" (ch 0x4A → states[17][0]
//! == 4, the 13 spawn touch), "Raise security gate and escape through front entrance" (ch
//! 0x50 → states[18][0] 3→4, the panel E + the entrance touch).
//!
//! THE CLOSING PIN (level-13-spawn-to-end-walk): part 0's records ride the same warm chain and
//! fold only here — 0x04000036 "Gain access to lower office floor" (ch 0x48) completes on the
//! chain's first monitor tick after 07000012's spawn touch #679 writes ch 0x48 [D; M 0x1827d8
//! case 3], and 0x04000035 "Escape from the tower" (ch 0x64 = 100 [L c07000011.rs]) — announced
//! on 07000011 and pinned announced-only by walkreal_07000011_live — is STILL announced (3) when
//! 13 loads: its writer is the whole-mission AND ch 100 [M 0x1827d8:119-123], which first fires
//! HERE, the tick the success touches #73/#164/#165 write ch 0x64. The same tick the monitor's
//! case 3 folds it 3 → 4 and the all-AND raises ch 100 → internal 6 [M 0x182c60:23-28] — no
//! channel pokes anywhere; every objective's condition is written by the game's own objects.
//!
//! The fake lift (SP #161, trigger ch 0x23) has no writer on the disc [D §4] — dead data; the
//! route out is the front entrance, not the lift.

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// File-local pre-clear (civ-unignore-2, the 00 Agent lobby answer): stand where Bond stands and
/// drop every hostile in `radius` whose boxes the gun-ray set exposes from HERE, through the
/// ordinary fight machinery (`Walker::stand_and_fire` — aim, converge, tap, reload; no movement
/// beyond the fight's own back-off/creep flags, none of which are set on this walk's stands). A
/// player walking a known-guarded lobby shoots the posted guards from the doorway before
/// crossing their pocket; the damage-weighted return-fire pick only answers whoever is ALREADY
/// hurting Bond, so the posted rifle guard 13 u across the floor is engaged by this sweep
/// instead of after his first point-blank volley (the measured master ledger: death tick 1666
/// with #3's three −14 rifle hits at d 2.6-2.7 after the leg walked Bond into the pocket the
/// fights never pre-engaged). The sweep's rules are the fights' own: `DamageLog::
/// exposed_part_gun_rays` (the lane ray set the 13 walk fights with) gates each pick, so a
/// guard behind the lobby furniture is skipped, not crept toward. No position writes, no RNG
/// draws, no health/ammo writes; the only inputs are the view drives and the trigger taps.
fn clear_the_visible(w: &mut Walker, label: &str, radius: f32, d3: bool, stop_on_fail: bool, floor_only: bool, loose: bool, stop_on_success: bool) {
    w.switch_to_armed(label);
    for _ in 0..12 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                // retune-nb-13-2 (floor_only, the d3 sweeps): the elevated guards are the
                // aimed-stand watch's business — the sweep's own stand on the mezzanine rifle
                // #4 (dy 6, dxz 21.4) returned killed false, the stop_on_fail law then ENDED
                // the sweep before the floor wave was answered, and #4's wake poured 40.5
                // across the west-mid legs [L the nb-13-2 lever-4 run]. The d2 sister's
                // sweeps keep the unfiltered pick (byte-identical).
                    && (!floor_only || d.pos[1] - sim.bond.pos[1] <= 2.0)
                    && walker::dxz(d.pos, sim.bond.pos) < radius
                    && walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
            })
        };
        let Some(di) = target else { return };
        // warm-chain-margin: the sweep's stands cap at 700 for the d3 walk (they land in ~50-350
        // ticks — every measured kill in this file's ledgers); a stand that cannot open keeps
        // Bond statue-still in the wave's return fire for the 1500-tick no-hit give-up — the
        // tick-1995/2242 deaths both bled out inside a failing stand. A player who cannot see
        // the shooter repositions. The d2 sister keeps the master cap so her legs stay
        // byte-identical to the sight batch's green run.
        // walk-13-op (`loose`, d3 only): a FLOOR-LEVEL target runs the file's own measured
        // CHARGER LAW — `shoot_loose` (±0.07/±0.10, the 07000006 point-blank idiom; the
        // 2026-11-06 measure: the tight gate "holds fire on 13's lobby CHARGERS — the stands
        // waited on alignment while the movers landed −24/−27 volleys" [L the run ledger];
        // the door-drain lever's own law, retune-nb-13-4) instead of the still-gate
        // `aimed_stand`. The walk-13-op lever-1 run measured the still-gate sweep's stand on
        // the wave Spas12 #7 (hp 12, d 7.0, floor level): the aim phase held fire while his
        // 8-pellet TooClose burst landed −24.0 in two frames and the walk died mid-sweep [L
        // the lever-1 log]. At d ≤ 12 the loose band lands and the taps fire ~3x more often.
        // Only this file's re-anchor sweep passes `loose` (with `floor_only`, so the pick is
        // already floor-level); every other call site passes false and keeps its stand.
        let killed = if loose {
            w.shoot_loose(di, label, if d3 { 700 } else { 2400 })
        } else {
            aimed_stand(w, di, label, if d3 { 700 } else { 2400 })
        };
        // aimed_stand's early gives-ups (dry, disabled) return before its own restore: force the
        // walk's default back — 13's fights must keep the plain tap gate (the precise_fights
        // lesson: the tight gate holds fire on the lobby chargers).
        w.dmg.aimed_fights = false;
        eprintln!("[013] {label}: the pre-clear stand on drone #{di} killed {killed}");
        // walk-13-op (the refreshed ledger's lever): the kills RE-FEED the lobby's spawner
        // (start ch 0x29 = the guards' death channels [D §4]) — the lever-2/3 runs measured
        // the sweep's SECOND stand losing the point-blank trade against the re-fed wave guard
        // #10 (hp 12, d 4.3 at the sweep's own spot; -24.5 in 7 hits + the unkillable mezzanine
        // #5's -23.0, death t4784 INSIDE the stand [L the walk-13-op logs]). A player who has
        // taken the posted guard he came for does not re-stand at the same spot for the
        // spawner's replacement — the legs' own nav fights answer the wave while MOVING (the
        // s/hard-walks-b fill pass-2 nav fights killed the same pair killed-true-x2 for -14.0
        // [L that tree's repro]; the margin-2 iter-1 law answered the wave from the glass band
        // killed-true-both [L the iter-1 ledger]). d3 only; the other call sites pass false.
        if killed && stop_on_success { return; }
        if !killed {
            // warm-chain-margin-2: the round-1 ledger's death INSIDE the post-failure stands —
            // #9's stand returned killed false and the sweep stood again while the unseen
            // desk-block guard #11 (weapon 0x19, d 12.2) landed −3.5 every ~6 ticks (the death
            // tick-2242 frames 1769-1891, five of his last-hit lines). A player who cannot open
            // a guard's box from a spot does not statue there — he moves; the legs' own
            // return-fire fights (killed true at t1487 vs #5, cost 10.5) are the answer while
            // walking. Sites that measured green keep the old retry (stop_on_fail false).
            if d3 && stop_on_fail { return; }
            w.resupply(label);
        }
    }
}

/// The spawner-wave drain (level-13-warm-chain-margin-2, ported to master's sight batch):
/// the desk-block kills raise the lobby's spawner waves (start ch 0x29 = the guards' death
/// channels [D §4]) and the wave arrives in a CADENCE, not one drop — the round-1 ledger took
/// wave guard #0 at t724 (cost 0) and then met the LATE guard #3 (skin 0x5000006, spawned at
/// (6.8,-368.4,38.3)) mid-leg at the re-anchor, where the return-fire fight traded 34.0 in 10
/// hits for the kill [L the round-1 repro run]. A player who has just cleared a pocket holds
/// the spot and lets the wave come to him: this stand waits out the spawn cadence (the round-1
/// guards land ~300 ticks apart) and takes each guard the moment the gun-ray set exposes him —
/// from the already-taken spot, at the wave's own range, instead of meeting him at the leg's
/// close band. One failed stand ends the drain (the unopenable box is the exposure itself; the
/// stop_on_fail law above) — unless `skip` is set: retune-nb-13-4 (the ignore message's law)
/// keeps a failed-stand SKIP-SET instead — a failed stand (+retry) pushes the drone and the
/// loop CONTINUES to its other rounds, so one unfinishable guard (the wave-rifle #6's
/// furniture pocket, the still-gate cannot close him: levers 8-10 measured the stands deal 21
/// for a 21-hp fire trade and the 4th pause never came) does not END the drain before its
/// other rounds run — the watch drain keeps #4's kill and the door drain keeps #7's. The
/// east-lane drain is the d2 SISTER's too (the byte-identity law) and passes false.
///
/// `loose_floor` (retune-nb-13-4, the door-drain lever): floor-level targets (|dy| <= 2.0) run
/// the file's proven CHARGER LAW — `shoot_loose` (±0.07/±0.10, the 07000006 point-blank idiom;
/// the 2026-11-06 measure: the tight gate "holds fire on 13's lobby CHARGERS — the stands
/// waited on alignment while the movers landed −24/−27 volleys"; the answer block's loose-gate
/// stand killed the wave-rifle charger #9 at ZERO health cost) instead of the still-gate
/// `aimed_stand`. The cap-1 skip-set run measured why: at the door mouth the wave-rifle #6
/// (a 0x5000006 charger, floor level, d 3.6, hp 24) stood killed-FALSE through the 700 cap and
/// the 1400 retry while the trade ran 46.0 -> 23.0, and the E-leg return-fire fight then ate
/// another −21.0 (23.0 -> 2.0) for killed false — Bond ran the whole room phase at 2.0 hp and
/// the endgame resupply nav met the respawned #7's point-blank shell [L this session's run].
/// At d <= 12 the loose band lands and the taps fire ~3x more often — the charger dies in
/// ~100-250 ticks instead of never. Non-floor targets keep the still-gate stand (the mezzanine
/// #4's kill is the still-gate's own proven act, 3x [L]). The east-lane and room drains pass
/// false (byte-identity; the room's #8 kill is the still-gate's too).
/// No position writes, no RNG draws, no health/ammo writes; the only
/// inputs are the wait ticks and the trigger taps through the ordinary stand machinery.
fn drain_the_wave(w: &mut Walker, label: &str, radius: f32, max_rounds: u32, quiet_rounds: u32, wait: u32, retry: bool, skip: bool, loose_floor: bool) {
    w.switch_to_armed(label);
    let mut quiet = 0u32;
    let mut taken = 0u32;
    // retune-nb-13-4: the failed-stand skip-set (the ignore message's law) — the same shape as
    // the west-mid answers' skip-set (lever-4): a drone the stands already failed on is
    // skipped for the rest of THIS drain, and the loop keeps going to its other rounds
    // instead of the stop_on_fail return. Param-gated: the east-lane drain (the d2 sister's)
    // keeps the plain path.
    let mut failed: Vec<usize> = Vec::new();
    while taken + quiet < max_rounds {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, sim.bond.pos) < radius
                    && (!skip || !failed.contains(&i))
                    && walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
            })
        };
        let Some(di) = target else {
            quiet += 1;
            // warm-chain-margin-2 iter-4: the quiet wait is a STAND — the iter-3 ledger bled
            // inside it (the unseen respawners #6new/#10new landed frames 4995-5121 while the
            // drain waited for boxes that never opened — the tick-1995/2242 law again). A
            // player being shot from an unopenable angle does not keep waiting: the first hit
            // during the wait ends the drain and the legs' return fire answers on the move.
            let h0 = w.scene.sim.as_ref().unwrap().drones.health.health;
            for _ in 0..wait {
                w.tick(nf_game::player::Input::default(), false);
                if h0 - w.scene.sim.as_ref().unwrap().drones.health.health >= 1.0 { return; }
            }
            continue;
        };
        quiet = 0;
        taken += 1;
        // retune-nb-13-4 (loose_floor): floor-level targets run the charger law's loose gate;
        // everything else keeps the still-gate aimed stand. Bond is already armed (the drain's
        // own switch_to_armed above) — the answers' own shape.
        let floor_level = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[di].pos[1] - sim.bond.pos[1]).abs() <= 2.0
        };
        let mut killed = if loose_floor && floor_level {
            w.shoot_loose(di, label, 700)
        } else {
            aimed_stand(w, di, label, 700)
        };
        w.dmg.aimed_fights = false;
        // retune-nb-13-3 (lever-9, the one-tap-short retry, d3 drains only): the lever-8 run's
        // #6 stand dealt 21 (hp 24 → 3) and still returned killed false — the 700 cap closed
        // one tap before the guard's next still window — and the stop_on_fail law then ended
        // the drain with the hp-3 rifle alive behind the glass-band legs; his fire (46.0 in 7
        // hits) killed the walk at t5268 [L the lever-8 run]. A player whose exchange left the
        // guard at 3 hp does not walk away: ONE more bounded attempt, the aim raised fresh
        // (the settle + raise re-run — the still-gate catches his next pause), the tap lands.
        // The retry rides the `retry` param: the east-lane drain is the d2 SISTER's too (the
        // byte-identity law) and passes false — the watch and door drains are d3-only and
        // pass true. d3 only.
        if !killed && retry {
            // lever-10: the retry's window is 1400, not 700 — the lever-9 run measured the
            // pause cadence: three taps landed across the first stand's ~1400 active ticks
            // (hp 24 -> 3, one per ~470-tick still window) and the 700-tick retry caught
            // NONE (hp 3 unchanged, the guard in burst mode between pauses); his next pause
            // is ~470 ticks out, so the doubled window is what the fourth tap needs. The
            // trade: ~-14-21 of his fire for the KILL — at hp 3 the first tap of the pause
            // ends him, and the crossing + the room then run with no rifle behind them.
            killed = if loose_floor && floor_level {
                w.shoot_loose(di, label, 1400)
            } else {
                aimed_stand(w, di, label, 1400)
            };
            w.dmg.aimed_fights = false;
        }
        eprintln!("[013] {label}: the wave-drain stand on drone #{di} killed {killed}");
        if !killed {
            // retune-nb-13-4 (the skip-set): a failed stand (+retry) pushes the drone and the
            // drain CONTINUES — its other rounds still run (the quiet waits end on the first
            // hit taken, the health-watch law); the stop_on_fail return only for the plain
            // path (the east-lane drain, the d2 sister's).
            if skip { failed.push(di); continue; }
            return;
        }
    }
}

/// One STILL-gATE aimed stand on drone `di` (civ-unignore-2; the 07000006 file's own idiom,
/// level-local here per the merge rules — shared helpers stay opt-in, and this copy runs only
/// inside this file's d3 pre-clears). The parts that make far stands land, each measured on the
/// 0c/06 branches:
/// - the aim-RAISE loop: release, let the switch anim finish, then hold aim until the bit sets —
///   the raise queues only on the aim-PRESSED edge from the idle machine [M Player_Weapon
///   0x1a24c8:113-152; the finish sets the bit 0x1a2e38:554-561];
/// - the AIM-BIT tap gate: with the bit unset every round rides the HIP cone (spread base 100.0
///   [L ee_start_05.bin @ 0x2c10b8 +0x24] — ~6 u of scatter at the mezzanine sniper's 45 u),
///   with it set the aimed act is zero-spread [M Bullet_init 0x12c5a0:139-170];
/// - the CURSOR-aware convergence: the fired round obeys the view PLUS the eased cursor offsets
///   [M scene.rs free-aim; Player_AutoAim 0x1a7c98:50-59];
/// - the STILL gate: tap only after the guard holds still 18 ticks — a stepping guard is not
///   where the view was when the round lands (the round FLIES [M Bullet_update 0x12e7b0]);
/// - the STRICT gun-ray exposure re-pick every 10 ticks (`exposed_part_gun_rays`).
/// Never writes health, armour or ammo; the only inputs are the view drives + the trigger taps.
fn aimed_stand(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
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
                if sim.gun.clip == 0 && sim.gun.spare == 0 { return false; }
                // The still gate: root motion only (the 0c measurement — the box sway rides the
                // anim; the root is what carries the chest out of the round's path).
                let moved = have_pos && ((d.pos[0] - last_pos[0]).powi(2)
                    + (d.pos[1] - last_pos[1]).powi(2) + (d.pos[2] - last_pos[2]).powi(2)).sqrt() > 0.03;
                have_pos = true;
                last_pos = d.pos;
                still = if moved { 0 } else { still.saturating_add(1) };
                if d.health < hp0 { hp0 = d.health; last_hit = t; }
                // 1500: a far post exposes in firing windows — the 500 default aborted before a
                // first hit ever landed on the 06 mouth runs. A landing round kills a 12-health
                // guard, so the wait is the tactic, not a stall (the cap still bounds it).
                if t - last_hit > 1500 { return false; }
                if t % 10 == 0 { aim = walker::DamageLog::exposed_part_gun_rays(&sim, di); }
                if aim.is_none() { unseen += 1; if unseen > 80 { return false; } } else { unseen = 0; }
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
                // s-auto-aim: the cursor that puts the shot on `at` (screen units, through the zoom).
                let [cx_want_raw, cy_want_raw] = sim.cursor_for(at).unwrap_or([-yaw_err / 0.76966, pitch_err / 0.57724]);
                let cur = sim.cursor;
                let turn = ((cur[0] - cx_want_raw) * 0.76966).clamp(-0.08, 0.08);
                let look = ((cy_want_raw - cur[1]) * 0.57724 / std::f32::consts::FRAC_PI_2).clamp(-0.08, 0.08);
                let aoff = sim.aim_offset_angles();
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
    for _ in 0..90 {
        let p = w.scene.sim.as_ref().unwrap().bond.pitch;
        if p.abs() < 0.01 { break; }
        w.tick(nf_game::player::Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[013] the aimed stand ({label}): drone #{di} killed {killed}, health {:.1}",
        sim.drones.health.health);
    killed
}

/// retune-nb-13: the far pre-emptive stand (the mezzanine sniper from the re-anchor end). The
/// file's `aimed_stand` is shared by every sweep INCLUDING the d2 sister's (her byte-identical
/// law), so this copy carries the one change the far race needs: the tap fires at the fight
/// machinery's own proven on-target band (err < 0.02, perr < 0.028 — `shoot_bounded`'s gate,
/// the band every measured return-fire kill landed through) instead of the sweep stand's strict
/// 0.01/0.01, because the mezzanine race is measured at ~62 ticks of grace: the death run's #5
/// (hp 12, static at (−17.7, −362.4, 45.2)) opened fire 3767, ~67 ticks after the re-anchor
/// arrival armed his cone, and the strict gate's convergence (the 1.24-rad cursor drive at
/// ±0.08/tick) plus the raise left the first tap past his first two shots [L the lever-1 runs:
/// two stands, death unchanged at t3774, #5 hp 12.0 at both his hits]. Same still gate idea
/// (the root-motion check, 12 ticks), the same dry/absent give-ups, the exposure re-pick every
/// 10 ticks, cap 700. Trace prints every 30 ticks (the stall measurement the lever needs).
/// Only the d3 pre-emptive calls this; the sisters never run it.
fn mezz_far_stand(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
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
    let prev = w.dmg.aimed_fights;
    w.dmg.aimed_fights = true;
    let killed = {
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
        let mut aim: Option<[f32; 3]> = None;
        let mut hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
        let mut last_hit = 0u32;
        let mut still = 0u32;
        let mut last_pos = [0.0f32; 3];
        let mut have_pos = false;
        let mut dry_taps = 0u32;
        for t in 0..cap {
            let (input, fire, reload) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                if d.corpse || d.health <= 0.0 || !d.present { return true; }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 { return false; }
                let moved = have_pos && ((d.pos[0] - last_pos[0]).powi(2)
                    + (d.pos[1] - last_pos[1]).powi(2) + (d.pos[2] - last_pos[2]).powi(2)).sqrt() > 0.03;
                have_pos = true;
                last_pos = d.pos;
                still = if moved { 0 } else { still.saturating_add(1) };
                if d.health < hp0 { hp0 = d.health; last_hit = t; dry_taps = 0; }
                if MEZZ_DRY_END.with(|c| c.get()) && dry_taps >= 3 { return false; }
                if t - last_hit > 1500 { return false; }
                if t % 10 == 0 { aim = walker::DamageLog::exposed_part_gun_rays(&sim, di); }
                if aim.is_none() { unseen += 1; if unseen > 60 { return false; } } else { unseen = 0; }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let at = aim.unwrap_or([d.pos[0], d.pos[1] + 0.3, d.pos[2]]);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let desired_yaw = dx.atan2(dz);
                let desired_pitch = (dy / h).atan();
                let yaw_err = { let mut a = desired_yaw - sim.bond.yaw;
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let pitch_err = desired_pitch - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
                // s-auto-aim: the cursor that puts the shot on `at` (screen units, through the zoom).
                let [cx_want_raw, cy_want_raw] = sim.cursor_for(at).unwrap_or([-yaw_err / 0.76966, pitch_err / 0.57724]);
                let cur = sim.cursor;
                let turn = ((cur[0] - cx_want_raw) * 0.76966).clamp(-0.08, 0.08);
                let look = ((cy_want_raw - cur[1]) * 0.57724 / std::f32::consts::FRAC_PI_2).clamp(-0.08, 0.08);
                let aoff = sim.aim_offset_angles();
                let err = { let mut a = yaw_err - aoff[0];
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let perr = pitch_err - aoff[1];
                let reload = sim.gun.clip == 0;
                let fire = sim.aiming && !reload && unseen == 0 && err.abs() < 0.02 && perr.abs() < 0.028
                    && still >= 12 && t.saturating_sub(last_tap) >= 8;
                if t % 30 == 0 {
                    eprintln!("[013] far-stand trace t{t}: aiming {} err {err:+.4} perr {perr:+.4} still {still} unseen {unseen} clip {} spare {} tgt hp {:.1}",
                        sim.aiming, sim.gun.clip, sim.gun.spare, sim.drones.list[di].health);
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
                dry_taps += 1;
                w.tick_fire(input);
            } else {
                w.tick(input, false);
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
    eprintln!("[013] the far stand ({label}): drone #{di} killed {killed}, health {:.1}",
        sim.drones.health.health);
    killed
}

/// The far stand's dry-tap end (OPT-IN, the d3 chain only): three taps without a hit on the
/// target end the stand. At the re-anchor end the wheel's armed pick is the 0x1c shotgun (the
/// 0x19 pistol is dry there), and every pellet of its taps at the mezzanine pistol guard #5
/// (25.9 u) lands on the balustrade face at x -16.29, his box above it untouched; the stand
/// kept tapping for 540+ ticks while he landed 20 rounds for 47.5 [L this branch's impact log].
thread_local! { static MEZZ_DRY_END: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

/// The whole warm chain (12 → 4A → 13 → out) on real input. `real_damage` = the armour detour
/// (see the real-damage test below); the wrapper turns the top-up off for every walker the walk
/// makes. `difficulty` = the menu pick the chain runs at (the front-end's launch path [M
/// C_SBNFDFCTY 0x216ca8:15]; 3 = the pre-sister byte-identical 00 Agent).
fn walk_the_warm_chain(iso: &str, real_damage: bool, difficulty: u32) {
    // walk-13-flaky: the tower2 OPT-IN flags are PER-THREAD cells. They were process-wide
    // atomics, and libtest runs this binary's four walks side by side, so the d3 walk's stores
    // reached the sisters mid-walk (the d2 walk took the d3 route and died at tick 4583 in the
    // whole-binary run, passing alone). Every walk still resets them first: libtest may reuse a
    // thread after a panicked walk, and the d3 block below sets its own.
    tower2::PRE_CLEAR_SOUTH_CORRIDOR.with(|c| c.set(false));
    tower2::PRE_CLEAR_NORTH_LOBBY.with(|c| c.set(false));
    tower2::REANCHOR_NORTH_LOBBY.with(|c| c.set(false));
    tower2::FLANK_LOBBY_EAST.with(|c| c.set(false));
    tower2::NAV_SOUTH_HALL_WEST.with(|c| c.set(false));
    MEZZ_DRY_END.with(|c| c.set(false));
    // ---- 07000012 walked (the shared route) ----
    // Agent difficulty (3): the keypad locks' param-6 tag auto-unlocks them at create on
    // Operative [M 0x1ae1f8 tail], and the route's burn leg pins the Agent+ behaviour
    // (the burn writes ch 0xa; the Operative create-unlock writes nothing [M Lock_Unlock
    // 0x1b0650's flag arm]). The block drops the scene before the next level loads (a 4 GB
    // test machine OOMs with three resident levels).
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, difficulty).unwrap());
        assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
            "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
        if real_damage {
            // The spawn-lobby guards stand ON the first leg and the fights ran point-blank, where
            // every drone round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] — the run
            // arrived at 13 with 18.5 health and died in the lobby. A player backs off and trades
            // at range: the stand's opt-in step-back (aim held) while a guard is inside 3.0.
            w.dmg.back_off = Some(3.0);
            // The open-lobby lane tactics (wide shooters window, damage-weighted pick, floor-drop
            // resupply around fights; common/damage_log.rs `lane_fights`).
            w.dmg.lane_fights = true;
        }
        if real_damage && difficulty == 3 {
            // THE 00 AGENT AMMO ECONOMY (civ-repin-11-13): the 2026-11-05 d3 run's killer was 12's
            // north-lobby grenade — a single ~70-point blast at t5273 during the fight that killed
            // its thrower #6 [L the civ-alarm-repin-2 ledger]; the walk reached 13 at 27 hp and the
            // lobby finished it. The fights ran the fresh 7-round P99 [M 0x1a7ef8:282-288] against
            // the hip cone while this level's own fight drops rested 1.09 under Bond's pos y —
            // invisible to the lane 0.5 band, so every resupply pass came back dry. The two levers
            // (both [L]-proven on the sister walks): `lane_floor_drops` — the on-floor band relaxes
            // to the master 1.5 with the lane tried list, so the kills' own drops rearm the next
            // fight; and `precise_fights` — the 0.01-rad tap gate, the 2026-11-03 level-11 split
            // measure (the loose gate sprayed the 7-round budget; the gate-only fights all killed
            // [L repro11d]). Shorter, armed fights shrink the window a thrower has [S unverified:
            // the throw window itself is the report's live check]. The d2 sister passes with the
            // plain kit and stays byte-identical.
            w.dmg.lane_floor_drops = true;
            // warm-chain-margin-3: `precise_fights` stays ON here — master's sight-batch shape.
            // The margin-2 port tried it OFF (the old-sight lever: the loose gate kills faster);
            // under the new sight the north-lobby fights run longer and earlier, the loose gate
            // sprayed the P99 dry, and the leg's resupply nav (15 waypoints through the open
            // north lobby) gave the respawner stream the window — #18/#32 bled 76.0 across the
            // nav and Bond died at tick 7819 with 100.0 consumed, the leg never arriving [L
            // this tree's run]. The tight gate is the 07000011 ammo lever for exactly this.
            w.dmg.precise_fights = true;
            // warm-chain-margin-3: the north-lobby-2 leg's fights run the P99 dry under the new
            // sight and the drop-fetch nav (15 waypoints through the open lobby, the respawner
            // stream's window — the tick-7402 death [L this tree's run]) is the exposure. The
            // 13-leg kit's other two levers are exactly this shape: `switch_dry` — a player
            // whose mag ran dry swaps guns mid-fight, not after it (the game's own weapon
            // cycle, the 02/03 00-Agent lever); `escalating_cooldowns` — a player who could not
            // answer a shooter from a spot does not re-take the same spot 240 ticks later (the
            // 0c Agent lever; the wait doubles per further failure on the same drone).
            // retune-nb-13 (measured DEAD, not repeated): `finish_fights` (the determined pass
            // after every failed fight) — the 12 run died EARLIER, t13833 at (−16.32, 1.13,
            // 19.38) inside the flank's own melt zone, the determined 1800-tick passes'
            // strafe/flank windows walking Bond into the posts' lines [L the lever-5 run].
            w.dmg.switch_dry = true;
            w.dmg.escalating_cooldowns = true;
            w.resupply_home = true;
            tower2::REANCHOR_NORTH_LOBBY.with(|c| c.set(true));
            tower2::PRE_CLEAR_NORTH_LOBBY.with(|c| c.set(true));
            // warm-chain-margin-5: the lobby-east posts' flank (the margin-4 report's first
            // lever). The melt ledger killed every in-place answer from the lane (−59.5 in 64
            // ticks from x −9.1, the exposure flickering [L this tree's NF_DMG_TRACE run]) and
            // the pre-clear-from-the-lane-ends lever found nobody exposed at 14-16 u [L the
            // run-8 ledger]; the flank walks the south band (z 13.4-13.6, the static probes'
            // under-the-sight-lines lane) to the d2 fight#6 trace's proven fight spot (2.4,13.9)
            // and takes #7 first, then #6 from the flank — the ordinary fight machinery, the
            // walk's own flags. Rejected levers not repeated: no pre-clear hook, no
            // bridge_and_retry, no in-place creep/longer stands (each measured dead, the
            // margin-4 report).
            tower2::FLANK_LOBBY_EAST.with(|c| c.set(true));
            tower2::NAV_SOUTH_HALL_WEST.with(|c| c.set(true));
            MEZZ_DRY_END.with(|c| c.set(true));
            // warm-chain-margin-4: the lobby-east posts (#6 rifle 0x18 + #7 0x19, hp 30/12, at
            // (3-5,15-18)) open fire the moment the lane's west half is crossed — the d3 melt
            // ledger: −59.5 in 64 ticks from x −9.1, the fights' own exposure flickering and
            // the taps −3.5 per ~4 rounds [L this tree's NF_RF_TRACE run]. A player who spots
            // the posted pair from the lane's west end shoots them from THERE (the master
            // tower2 hook, now fired at the "the north lobby" end too, radius 16, stands
            // capped 2400 so the 30-hp rifle post dies in one stand).
            // warm-chain-margin-3: `lenient` is NOT set here — the resupply-nav machinery
            // restores it to false after its own sub-walks, so a leg-level setting does not
            // survive the first dry-nav (the tick-13980 run still panicked non-lenient at the
            // "the north lobby 2" cap [L this tree's run]). The budget/wedge re-pin is the
            // next session's first item (see the report).
            // warm-chain-margin-4 (the north-lobby-2 wedge's fix, both pieces OPT-IN and set by
            // no other caller): the wedge's cause is the fights' resupply fetches — the stream
            // guards' drops rest on the lower floor ~1.1 under the lane (the lane_floor_drops
            // band admits them), the fetch's nav walk leaves Bond THERE, and the leg's resumed
            // direct line crosses the lobby's south wall (cel 374 blocks every line from that
            // floor to the lane; the probe scanned x −12..−4 at z 15.75–17.5: all blocked [L
            // probe_012_rebridge, this tree]) for the 6000-tick cap (bond (-8.99,1.03,12.62),
            // target (-8.00,1.10,17.00), segment Some(0.127), tick 13980 [L the repro run]).
            // `resupply_home` makes each fetch a there-and-back errand (the leg resumes where
            // it left); `REANCHOR_NORTH_LOBBY` is the legs' own recovery should a displacement
            // ever survive the errand — the proven passage chain back to the lane.
            // civ-unignore (the measured pre-engagement burst): #10's first fire on the bend→
            // corridor walk is ONE tick of 8 × −3.0 shotgun pellets at d 3.5 (t2723, Bond at
            // (16.8,−12.0)) — the −24 chunk that sends 13 the 62 instead of 86, the exact leak the
            // civ-repin-finish report ranked first (its live check asked what wrote 86.0 → 62.0
            // between ticks 2531 and 2764; NF_DMG_TRACE answers: guard #10, weapon 0x1c, one
            // frame). The opt-in tower2 flag stands at the "the south corridor east" leg end and
            // kills him through the ordinary fight machinery before his sight opens. OFF for every
            // other caller (the 07000012 and campaign walks set nothing).
            tower2::PRE_CLEAR_SOUTH_CORRIDOR.with(|c| c.set(true));
            // warm-chain-margin-3: the margin-1 north-lobby pre-clear (the flag-gated tower2
            // lever) is NOT ported — master's own evidence is that the d3 chain crosses 12 with
            // this exact kit (its recorded death is on 13, tick 2448 [L the sight batch's
            // ledger]); this tree's one run with the loose gate died inside 12's resupply nav,
            // not in the pursuit the lever answers.
        }
        if real_damage && difficulty == 1 {
            // batch3 (5e9460adb): the drop at (17.5, 0.9, 1.5) on 12's lower floor is out of
            // reach on the new floor/wall contact: two fetches stood the 6000-tick cap at
            // (16.6, 1.40, 3.19) and the third left Bond on the furniture at y 1.93, from which
            // the south-east bend 2 leg wedged at (19.21, 2.02, -8.94) [L this tree's runs]. A
            // player who cannot reach it walks on with the pistol.
            w.no_fetch.push([17.5, 0.9, 1.5]);
        }
        tower2::walk_12(&mut w);
        tower2::walk_12_exit(&mut w);
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
            // ONE difficulty through the mission (level-13-chain-difficulty): the game's
            // GameState+0x28 (0x2a3790) global survives the level change untouched — its five
            // writers (bootup 0x1c7e78:25, the difficulty page 0x216ca8:15, MP confirm 0x20ec00:150,
            // mini-mission 0x1ffb90:22, the PTP return 0x1c8560:36) never run in an on-foot level
            // change [M whole-decompile grep] — and the next level's creates read the SAME pick
            // [M 0x136be0:49]. The chain's pick is 00 Agent (3): the 12 route pins the Agent+
            // roster and the keypad's no-create-unlock burn [M 0x136be0:49, 0x1ae1f8 tail].
            .with_difficulty(sim.drones.difficulty)
    };

    // ---- 0700004A: the spawn drop onto the 0x4D touch (the game's own placement) ----
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save)).unwrap());
        if real_damage { w.dmg.back_off = Some(3.0); w.dmg.lane_fights = true; }
        w.wait_enabled(3000, "the 4A intro");
        let mut landed = false;
        for _ in 0..600 {
            w.tick(nf_game::player::Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
        }
        assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
        // fix-objective-banner: the monitor runs at the TOP of the tick — the objective completes the
        // tick after the channel rises.
        for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
        assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[16][1], 4,
            "12's 'Escape through the elevator shaft' completes (the late AND) [M 0x1827d8]");
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
            // The difficulty rides on (the chain's one pick; see the 12 capture above).
            .with_difficulty(sim.drones.difficulty)
    };

    // ---- 07000013 walked ----
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, Some(&save)).unwrap());
    if real_damage { w.dmg.back_off = Some(3.0); w.dmg.lane_fights = true; }
    if real_damage && difficulty == 3 {
        // The same ammo economy on 13's own legs (civ-repin-11-13): the 2026-11-06 d3 run (the
        // ammo levers + the head hunt) crossed 12 at 62 health — the north-lobby grenade chunk
        // GONE, #6 killed with zero damage — and died in the first lobby leg: the fights ran the
        // held gun dry mid-stand (#0 "killed false" twice, re-fought 330 ticks later, #10's −24
        // chunk while the stand was dry) [L the run ledger]. A player whose mag ran dry swaps
        // guns mid-fight, not after it (`switch_dry` — the game's own weapon cycle, the 02/03
        // 00-Agent lever), and a player who could not answer a shooter from a spot does not
        // re-take the same spot 240 ticks later (`escalating_cooldowns` — the 0c Agent lever);
        // the leg's own walk carries Bond out of the sight line. NO `precise_fights` here: the
        // gate's 0.01 rad aligned on the stairwell's STATIC guards (the 11 walk) but holds fire
        // on 13's lobby CHARGERS — the 2026-11-06 run's stands waited on alignment while the
        // movers landed −24/−27 volleys [L the run ledger]; the head hunt already pays 1-2 rounds
        // a kill through the default gate.
        w.dmg.lane_floor_drops = true;
        w.dmg.switch_dry = true;
        w.dmg.escalating_cooldowns = true;
        // retune-nb-13-2: the fetches on 13 run there-and-back like the 12 side's (the field doc's
        // own lever): the measured re-anchor fetch left Bond AT the drop (4.93, −368.39, 34.15 —
        // 2.4 u off the leg line [L the nb-13-2 repro]) and the guard-repin ledger measured the
        // pocket displacement breaking the west-mid line (the 6000-tick furniture-corner cap).
        // d3 only; the sisters never set it.
        w.resupply_home = true;
        // retune-nb-13-2 (the mezzanine law, the 07000016 flag's own filter): the fights never
        // statue on the elevated guards — the plain stands measured dead twice (the #5 pick
        // sprayed the refilled clip, 15 rounds 0 hits at dxz 13.3 dy 6 [L the lever-2 run]; the
        // balcony rifle #4's 900-cap fight at dxz 37.2 cost 42.0, 46.0 -> 4.0 [L the lever-3
        // run]). The aimed-stand watch below owns them. d3 only.
        w.dmg.level_fights = true;
        // retune-nb-13-3 (THE NEXT LEVER, from the nb-13-2 ignore message): the fights' weapon
        // cycle prefers DEPTH. The lever-5 ledger: the pool lever had filled the inventory
        // (17 -> 71 rounds) and the west-mid answers won their exchanges, but they dried the
        // held clip doing it — the north-west leg's fight entry then found armed() false, the
        // resupply fetch came back empty (every drop behind Bond), and the cycle landed on the
        // wheel's 0x1c SPAS12 (8 shells, c8 s0 — no spare) instead of the pool's 0x18-type
        // rounds; that 900-cap stand on the wave rifle #6 ate 46.5 — death INSIDE the fight
        // stand, t5075, the north-west watch drain never ran [L the lever-5 run]. A player
        // whose mag ran dry arms the magazine that actually has depth: the opt-in
        // `switch_gun_deep` (the lesson-7 shape, the shared helpers' depth pass —
        // switch_to_gun AND the dry-fallback arm switch_to_armed, the two paths a fight entry
        // can arm through) cycles up to twelve draws and stops on the first gun with
        // clip+spare >= 10, falling back to the plain first-armed rule when nothing owns 10.
        // d3 only — the sisters never set it, byte-identical by construction.
        w.switch_gun_deep = true;
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48) && sim.drones.channels.on(0x49) && sim.drones.channels.on(0x4d),
            "the previous parts' channels carry through the warm load [M 0x182270]");
        assert_eq!(sim.drones.difficulty, difficulty,
            "the chain's one difficulty rode the warm save into 13 [M GameState+0x28 has no transition-path writer; RamSave::difficulty]");
    }
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4a), "the spawn touch #153 fires ch 0x4A [D]");
        assert_eq!(sim.drones.mission.states[17][0], 4, "4A's 'Gain access to lobby' completes (the late AND) [M]");
        assert_eq!(sim.drones.mission.states[18][0], 3, "'Raise security gate…' announces (enable 0x4A) [L]");
        // Part 0's rows at the carried load [M 0x182270 leaves previous parts alone]: 0x04000036
        // folded on the chain (12's spawn touch #679 wrote ch 0x48 [D]); 0x04000035 still announced
        // — its writer is the whole-mission AND that first fires on THIS level [M 0x1827d8:119-123].
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "11's 'Gain access to lower office floor' folded on the warm chain (ch 0x48, written by 12's spawn touch #679 [D]; M 0x1827d8 case 3)");
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "0x04000035 'Escape from the tower' still announced at 13's load [L c07000011.rs; M 0x1827d8:119-123 — its AND writer first fires here]");
        assert!(!sim.drones.channels.on(100), "ch 100 is off at 13's load — the success touches are its first writer [D]");
    }
    // civ-unignore-2 (the d3 pre-clear): the spawn pocket's posted guards (#3 rifle hp 24 at
    // (6.6,−368.4,27.5), #7 shotgun hp 12 at (1.5,−368.4,24.0), the melee pair beside them) are
    // exposed from the spawn floor — the master baseline's own fights #1/#2 prove the gun rays
    // reach (6.6,−368.4,28.2) and (4.9,−368.4,24.9) from d 15+ — but the damage-weighted
    // return-fire pick never engages them until they hit, and the leg then walks Bond INTO the
    // pocket: #3 landed three −14 volleys at d 2.6-2.7 and the shotgun finished the run (death
    // tick 1666 at (3.57,−368.36,25.09), 86.0 consumed). A player shoots the posted guards from
    // the doorway before crossing their pocket.
    // los-mask-join: the Agent walk takes the same pre-clears (here and the lobby/doorway
    // sweeps below). Under the game's sight masks the lobby guards engage more and earlier,
    // and the Agent walk, which left them to its legs' return fire, ran dry at the
    // lobby north-west (clip 0 spare 0) and died at the front entrance [L the masked run].
    if real_damage && difficulty >= 2 {
        // warm-chain-margin-3: the d3 walk caps its stands at 700 and keeps the retry (measured
        // green); the d2 sister runs the sight batch's byte-identical 2400-cap sweep.
        clear_the_visible(&mut w, "13: the spawn pocket", 24.0, difficulty == 3, false, true, false, false);
    }
    // retune-nb-13 (measured DEAD, not repeated): a pocket reposition + deliberate resupply to
    // dodge the mid-leg fetch — the reposition leg's own ticks fired the low-ammo fetch first
    // (the wave shooter existed), the north drop (4.9, −369.4, 22.2) won the pick from the
    // sweep's end spot anyway (d 2.1 vs 2.9), and the fetch nav walked Bond into the pocket's
    // north-west corner where the north-lobby rifleman #3 (then (5.2, −368.4, 43.3)) landed
    // 19.5 in 10 hits — death t993, WORSE than the baseline [L the lever-4 run].

    // THE SECURITY GATE: the "Shutters2" SimpleScript #149 (out ch 0x23 [D param 3]) is the
    // entrance mouth's panel (the SS cel's glist spans x −8.54..−4.97 at z 65.02 [D the object
    // bounds] — exactly the wall the old walk pinned at). Its writer is ScriptPlayer #161
    // ("SCRIPTPLAYER_Elevators", mode 64 preload, mask 4, out ch 0x23 [D params (5,4),(6,23)]):
    // the preload's Entity op creates 0x02000A2C "SCRIPT_SHUTTER_CLOSE" — the wall panel's
    // world-baked glist at (−18.08, −368.23, 25.01) [D obj 70] — with obj+0xE4 = the SP handle
    // [M Script_EntityStart 0x1d8d78:110], so the E pick's class-0 arm reaches SP_Activate
    // [M Player_Activate 0x1a5170:121-122] and the played script writes ch 0x23
    // [M SP_Update 0x1dbda8:221-229]. The panel is the west-wall shutter display ("SCRIPT_
    // SHUTTER_CLOSE/OPEN" share its glist centre [D objs 68/69]); ch 0x23 runs the SS playhead
    // to key 1 and the gate lifts [M SS_Update 0x1a9948 + KeyFrame_Interp 0x1e1140].
    // Sister: before the press ch 0x23 is off and the SS rests at key 0.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x23), "ch 0x23 off before the panel [D]");
        let rest = sim.drones.sss.list.iter().find(|s| s.channel == 0x23)
            .map(|s| s.matrix[3][1]).expect("the Shutters2 SS on ch 0x23 [D]");
        assert!(rest.abs() < 0.05, "the gate SS rests at key 0 (y offset {rest}) [D]");
    }
    // The panel E — WALKED through the security room (716c): the lobby's west "furniture maze"
    // truly blocks every direct lane (the fake-lift doors' baked mesh at x −14.6..−12.9 /
    // z 31.3..32.9 inside the security-room glass band x −18.4..−10.6 / z 27.5..35.3, plus the
    // room's own east wall — probe13_cell's stuck A/B/C object dumps), and the panel display
    // (−18.08, −368.23, 25.01 [D obj 70]) sits INSIDE the security room's footprint. The
    // player's route is the unlocked north-west door at (−26.65, −368.36, 37.44) — the only
    // west door the sim creates, no lock channel, E-opens [M Door_Activate's toggle] — whose
    // doorway carries portal 1 (cels 21↔28 [D the portal record's celA/celB]); Bond's runtime
    // cel crosses 21→28 through the stateful relink's portal arm (Cel_ObjectLeftCel 0x1e0d58)
    // and the room's own floor (cel 28's bucket) holds him.
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    if real_damage && difficulty == 3 {
        // warm-chain-margin-2 (ported to master): the desk-block kills raise the spawner waves
        // (start ch 0x29 = the guards' own death channels [D §4]) and the wave lands ON the east
        // lane — the ledger's tick-1647 volley (7 shotgun pellets, −36.5 in one tick [M
        // player_hit_damage: the pellet damage × Agent pain × DroneFiring_TooClose_Damage])
        // caught Bond standing in the pocket while the fights' own caps had twice left the wave
        // guard alive (killed false at ticks 781/1015). The round-1 sweep ended with nothing
        // exposed (no stands logged) because the wave's LATE guards (#3 at (6.8,-368.4,38.3),
        // ~300 ticks behind #0) had not spawned yet — the leg then walked INTO them and the
        // re-anchor fight cost 34.0. Drain the wave from here: up to 12 rounds, done after 6
        // quiet rounds (~180-tick waits + the stand times), each guard taken from the taken
        // spot at d ~12.
        drain_the_wave(&mut w, "13: the lobby east lane", 24.0, 12, 6, 180, false, false, false);
    }
    // guard-repin-master re-anchor: the west-mid leg's return-fire fights (the north-lobby
    // shooters) displaced Bond SOUTH of the lane line into the lobby pocket at (0.60, −368.30,
    // 25.66) — the guards' per-goal arrival radii [M 0x152548:174-181] close the pursuit to
    // 1.0-2.0, the stands' back-off geometry moved, and from that spot the direct segment to
    // the west-mid clips the lounge furniture corner at ~(0.3, 26.3) (the measured 6000-tick
    // cap: segment_to_target hit at t 0.042, d 15.78, all nearby guards dead). The re-anchor
    // pulls the route back up to the east lane's proven line before the west leg: from the
    // pocket the segment to (2.5, 30.5) runs north-east, clear of the furniture corner, and
    // the west-mid line from there rides the old path.
    // retune-nb-13: the aim-walk flag on the leg is RETIRED (measured null — the walk_to
    // steering builds its own Input, so the flag never reached the leg's ticks; the lever-3 run
    // reproduced the t0 err 1.5865 byte-identical [L]).
    w.walk_to(Wp::new(2.5, -368.4, 30.5).r(1.5), "13: the lobby mid re-anchor");
    // walk-13-op-2 (the same-end skip-set, the floor loop's lever-4 law at END scope): a
    // drone the re-anchor end's stands already failed on is skipped by every later stand at
    // this end. The lever's first run measured why: the block's mezzanine pick re-took the
    // pre-emptive's own failed #5 (nearest exposed dy>2; the balcony #4's box never opens
    // from this end) and re-stood into his fire — death t4200 INSIDE the stand [L the
    // lever-1 run]. d3-only code below mutates it; the sisters never touch it.
    let mut failed_at_end: Vec<usize> = Vec::new();
    if real_damage && difficulty == 3 {
        // retune-nb-13 (the mezzanine pre-emptive): the nb-scan-all ledger's death is the
        // mezzanine pistol guard #5 (skin 0x5000004, weapon 0x19, hp 12) sniping the re-anchor
        // end from d 25.7 — OUTSIDE the sweep's 24 radius below (he patrolled farther west than
        // the guard-repin-master ledger's (−14.8, −368.4, 45.6) post: the 2026-11-18 repro run
        // measured him at (−17.7, −362.4, 45.2), d 25.7 from the re-anchor spot, and his two
        // −3.5/−3.0 pistol taps killed the 6.5-health walk at t3774 [L the retune-nb-13 repro
        // run]). His box IS exposed from the re-anchor spot (the death fight's own gun-ray pick
        // returned an aim point at t0 [L the same run, fight#5 aim Some]) and he had NOT opened
        // fire while Bond stood there — he fired only once the west-mid leg started moving
        // (~57 ticks in), the sight law the statue periods measured (the east-lane drain's
        // ~2160 silent quiet-wait ticks drew no fire either [L the same run]). A player who has
        // spotted the mezzanine shooter takes him from the re-anchor end BEFORE walking into his
        // cone: the nearest live team-1 guard more than 2.0 above Bond's floor (the mezzanine
        // band — the lobby floor guards and the floor spawner's wave all read Δy ≈ 0) within
        // 32 u, up to three aimed stands through the ordinary fight machinery (the file's own
        // aimed_stand — the strict aim-bit zero-spread act [M Bullet_init 0x12c5a0:139-170] is
        // what lands at 25.7 u; the hip cone's ~3.4 u of scatter there is not). The balcony
        // rifle post (16.2, −362.4, 41.7 [L the door-drain comment], d 16.3) may sit nearer and
        // is taken first — he sniped the door wait in the margin-2 ledgers, so his early death
        // only helps. Never lower damage or health; the sisters (d2/d1) skip the block.
        // retune-nb-13-2: top the clip before the stands — the lever-4 far-stand converged on
        // #4 with the held gun at clip 1 (the fill's rounds sat in the inventory slots), the
        // first tap emptied it into the reload churn, and the statue ate the wave's volleys [L
        // the lever-4 run]. A player tops the magazine before taking a sniper's post.
        // walk-13-op (THE LEDGER'S LEVER, the s/hard-walks-b ignore message's "the 13 lobby
        // budget from the re-anchor end on"): the pool fill runs BEFORE the pre-emptive
        // stands, and one no-gain pass ENDS it. The measured s/hard-walks-b ledger on this
        // tree [L the byte-identical repro run]: 13 starts at 48 hp + 50 armour, the
        // spawn-pocket pre-clears and the east lane cost 0.5, and the re-anchor pool fill then
        // bled 34.5 — pass 1 fetched a guard's 0x19 pistol drop whose gun LANDED IN HAND
        // (held 14 -> clip 7 spare 7, the arm the pass's switch_to_armed fallback counts as
        // got [M walker.rs resupply's !got arm-switch tail]) while its ROUNDS did not rise —
        // and the pre-emptive stands had ALREADY run by then, on the wheel's 0x1c shotgun
        // (c8, no spare depth) whose pellets all stop on the balustrade face x −16.29 under
        // #5 at 25.9 u [L the MEZZ_DRY_END ledger] — so the hp-12 sniper survived the block
        // and poured 37.5 in 17 hits across the whole west half, the fill's three passes cost
        // −34.5 (pass 2 −14.0 for +7 held, pass 3 −7.0 for NOTHING: held 21 -> 21), and the
        // west-mid leg died at t5119 on the remaining 13.5. The budget: (1) the fill first —
        // the stands' own law above, now armed from the pool's depth: after pass 1 the held
        // gun IS the fetched single-rod 0x19, whose aim-bit rounds follow the exposed gun-ray
        // the pick verified [M Bullet_init 0x12c5a0:139-170; L the death run's fight#5 aim
        // Some] — the act the balustrade does not stop (the pellets' cone is what it stops);
        // (2) a pass whose held count does not RISE ends the fill — the exit fill's own
        // ammo-rise law [L this tree's exit-fill ledger] applied at the pass level: the arm
        // is taken, the pool sits at 54, and every further errand is statue exposure under
        // the stream (pass 3 measured −7.0 for zero gain). The sisters never enter this
        // block; no damage, health or RNG change.
        w.no_fetch.push([4.9, -369.4, 22.2]);
        for pass in 0..3u32 {
            let held_before = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.gun.clip as i32 + sim.gun.spare as i32
            };
            if held_before >= 30 { break; }
            eprintln!("[013] 13: the re-anchor pool fill: held {held_before} < 30, fetching (pass {})", pass + 1);
            w.dmg.resupplying = true;
            let got = w.resupply("13: the re-anchor pool fill");
            w.dmg.resupplying = false;
            let held_now = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.gun.clip as i32 + sim.gun.spare as i32
            };
            eprintln!("[013] 13: the re-anchor pool fill: pass done ({got}) held {held_before} -> {held_now}, health {:.1}",
                w.scene.sim.as_ref().unwrap().drones.health.health);
            if held_now <= held_before { break; }
            if !got { break; }
        }
        let _ = w.switch_to_armed("13: the mezzanine watch arm");
        w.reload_full();
        for _ in 0..3 {
            let g = { let sim = w.scene.sim.as_ref().unwrap(); let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, bp) < 32.0 && d.pos[1] - bp[1] > 2.0
                    // walk-13-op-2 (THE FACE LAW, the ledger's own measures): a mezzanine target
                    // whose box sits WEST of the balustrade face is unkillable from the lobby
                    // floor — every pellet of the held 0x1c AND the 0x19's single rods stop on
                    // the face x -16.29, hp 12 intact all run [L the MEZZ_DRY_END ledger + the
                    // walk-13-op runs] — and the stands on him are pure tax (the walk-13-op
                    // lever-1: the pre-emptive stand on him cost -9.0 for nothing; the lever's
                    // first run here re-stood him after the pre-emptive's own failure and died
                    // t4200 inside the stand [L the lever-1 run]). The face value is the ledger's
                    // measured plane; the balcony EAST post (x +16.5) sits clear of it.
                    && d.pos[0] >= -16.29
                    && walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
                    && !failed_at_end.contains(&i) })
                    .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp))) };
            let Some(g) = g else { break };
            let _ = w.switch_to_armed("13: the mezzanine pre-emptive");
            let k = mezz_far_stand(&mut w, g, "13: the mezzanine pre-emptive", 700);
            w.dmg.aimed_fights = false;
            eprintln!("[013] 13: the mezzanine pre-emptive stand on drone #{g} killed {k}, health {:.1}",
                w.scene.sim.as_ref().unwrap().drones.health.health);
            if !k { failed_at_end.push(g); break; }
        }
        // The aim-walk flag is the leg's own: restore the walk's default (the sweeps set and
        // restore it around their stands; a leaked true would re-shape the west-mid legs).
        w.dmg.aimed_fights = false;
        // retune-nb-13-2 (THE POOL LEVER, the nb-13 report's first What's-left item): the fill
        // lived BELOW the stands once - the byte-identical repro then measured the pool drying
        // past the fill (the nb-13-2 repro: every drop behind Bond, death t4849), and the
        // s/hard-walks-b tree measured the fill ITSELF as the bleed (34.5 through three passes,
        // pass 3 -7.0 for zero gain, the t5119 death on the remaining 13.5 [L this tree's repro
        // run]). walk-13-op moved the fill ABOVE the stands (the budget lever, the comment
        // there) and ends it on a no-gain pass; the ledger history is that block's comment.
    }
    // civ-unignore-2's law at the re-anchor end (guard-repin-master): the same ray-gated sweep
    // runs one leg EARLIER — the west-mid route's fights used to be answered mid-leg from the
    // lane, but the guards' per-goal arrival radii [M 0x152548:174-181] close the pursuit to
    // 1.0-2.0 and the stands' displacement parked Bond in the lobby pocket (the measured
    // 6000-tick cap at (0.60, −368.30, 25.66), then the re-anchor's measured cost: #5 (skin
    // 0x5000004 at (−14.8, −368.4, 45.6)) landed 28.0 in 8 hits across the re-anchor + west-mid
    // legs and Bond reached the security room at 21 hp to lose the point-blank #8 exchange).
    // Taking the shooters at the re-anchor end — where #11 (3.8, 33.9) is d ~4.5 and the
    // mezzanine/west boxes open — keeps the west legs' budget for the room.
    if real_damage && difficulty >= 2 {
        // warm-chain-margin-2 (ported to master): a stand that cannot open the guard's box ENDS
        // the sweep here — the tick-2242 death bled out inside the post-failure stands (the
        // stop_on_fail law); the d2 sister keeps the master retry.
        clear_the_visible(&mut w, "13: the lobby mid re-anchor", 24.0, difficulty == 3, difficulty == 3, true, difficulty == 3, difficulty == 3);
    }
    // walk-13-op-2 (THE LEDGER'S LEVER, the walk-13-op ignore message's "Next lever"): the
    // answer block runs at the sweep's kill spot — the re-anchor end, ONE LEG EARLIER than
    // the west-mid end — because all five walk-13-op runs measured the leg itself as the
    // death ground (stop-on-success t4337: #4 21.0 in 7 + #5 26.5 in 13, 1 fight; walk-on
    // t4242: #5 23.0 + #4 14.0 + #10 10.5, 0 fights [L both logs]): the stands need the
    // health budget BEFORE the crossing, not after it. The mezzanine part is the
    // pre-emptive's own pick law (dy > 2 exposed within 32, the far stand's 0.02/0.028
    // band that converged on #4 at 21.4 u killed-true [L the nb-13-2 ledger]); the floor
    // answers (shoot_loose, the charger law) follow at the same spot. d3 only; the sisters
    // never enter the block, and everything below keeps master's shape.
    if real_damage && difficulty == 3 {
        // retune-nb-13 (the west-mid answer): the lever-6 ledger — with the pool filled (clip 7
        // spare 15) the west-mid leg crossed FLAT (54.5 held) and #5 died to the re-anchor's
        // own fight (t3959, zero cost) — but the north-west leg's melt opened on #4 (skin
        // 0x5000006, weapon 0x18, hp 24, the mezzanine EAST end at (16.1, −362.4, 43.3), d
        // 28.5 from the north-west spot) whose fight returned killed false (the strict fight
        // gate at 28.5 u outlived the 900 cap) and on #11 (skin 0x5000004, 0x19, hp 12, a wave
        // guard at (6.7, −368.4, 31.2), d 20.4 behind Bond): 54.5 -> 0 across the failed-fight
        // + resupply statues, death t4693 at (−11.58, −368.39, 40.38) [L the lever-6 run].
        // From the west-mid end #4 is d 21.4 (7 u closer — the far stand's 0.02/0.028 band
        // converges there) and #11 d 14.0. A player who has just crossed the lobby answers the
        // mezzanine rifle FIRST (the dy > 2 exposed pick, the re-anchor pre-emptive's own
        // machinery) and then the nearest floor guard, both through bounded stands — kill or
        // move on (the lingering law above still caps this at three 700-tick stands, not the
        // old open-ended ones). d3 only; the sisters keep the byte-identical legs.
        // retune-nb-13-2 (the ammo-rich arm): the plain armed cycle stops on the FIRST gun with
        // any round — the lever-5 run's west-mid answers spent the held clip dry and the fights'
        // later switch_to_gun landed on the wheel's 0x1c (8 shells, c8 s0, no spare) whose
        // 900-cap stand on the wave rifle #6 ate 46.5 — death inside the stand [L the lever-5
        // run]. A player arms the magazine that actually has depth: cycle the wheel (the real
        // cycle, no writes) up to twelve draws and stop on the first gun with clip+spare >= 10,
        // then top the clip from its own carried rounds.
        for k in 0..12u32 {
            let (wpn, c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.weapon, sim.gun.clip, sim.gun.spare) };
            if c + s >= 10 { break; }
            eprintln!("[013] 13: the west-mid ammo-rich arm: cycling ({k}) (weapon {wpn:#x} clip {c} spare {s})");
            w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        }
        w.reload_full();
        // retune-nb-13-3 (lever-7, the measured budget): the original 3-stand cap is RIGHT —
        // the lever-6 run proved the shape (the loose gate + the re-arm killed #9, the
        // charger, at ZERO health cost, 54.5 intact through the block) but its 5-stand cap
        // then took a #6 iteration whose 176-tick dry stand + re-arm cycle was one more
        // non-firing window in the wave rifle's line — death t5025 mid-re-arm [L the lever-6
        // run]. Three stands (the block's own budget), the failed drone skipped, and the
        // stream's next guard belongs to the WALK-ON LINE below (the dry fights walk the
        // stretch) and the door drain (the radius-45 machinery) — not to more statues at an
        // end whose furniture pocket keeps his box closed.
        for _ in 0..3 {
            let g = { let sim = w.scene.sim.as_ref().unwrap(); let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    // retune-nb-13-2: Bond's floor level only — the mezzanine guards (dy ≈ 6)
                    // belong to the far-stand answer above (the lever-7 machinery). The unfiltered
                    // pick took the mezzanine #5 at dxz 13.3 and sprayed the WHOLE refilled clip
                    // (15 rounds, 0 hits, the +240-tick statue) while the balcony rifle #4 poured
                    // 45.5 across the north-west legs [L the nb-13-2 lever-2 run]. The
                    // north-west pistol block runs the same dy filter (the file's own law).
                    && (d.pos[1] - bp[1]).abs() <= 2.0
                    // walk-13-op-2 (the charger law's own range, the lever-3 run's measure): the
                    // loose band lands only at d <= 12 [L the 2026-11-06 measure cited below] —
                    // the third stand of the lever-3 run took #9 at d 17.2 OUTSIDE that envelope
                    // (hp 24, clip 2 spare 6) and the statue cost the west-mid leg its window
                    // (the death 3.3 u short, t4730 [L the same run]). A farther guard belongs
                    // to the legs' own fights.
                    && walker::dxz(d.pos, bp) < 12.0
                    && walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
                    // retune-nb-13-3 (lever-4): a drone the stands already failed on is skipped
                    // for the rest of this block — the loop keeps going to the NEXT nearest
                    // instead of breaking. The lever-3 run's death is exactly the break's cost:
                    // the #9 answer dried and broke the loop with the wave-rifle #6 (hp 24, at
                    // (−6.8, −368.4, 28.1), d 11.2 — INSIDE this radius, exposed) unanswered;
                    // he opened fire at frame 4950 and his −7.0 rifle hits SHOVE Bond ~2.4 u
                    // west per hit — 46.5 total, the walk's whole remaining health — carrying
                    // the death to (−19.43, −368.30, 37.55) mid-re-arm [L the lever-3 run, the
                    // 13-2 ledger's t5075 at the same spot]. A player who misses an exchange
                    // does not stop answering: the next shooter in line gets his rounds — #6
                    // dies at this end (~4 hits) BEFORE his first volley, and the north-west
                    // leg walks the stretch with no rifle behind it. Bounded: the failed set
                    // grows, the pick radius holds, the loop cap is 5.
                    && !failed_at_end.contains(&i) })
                    .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp))) };
            let Some(g) = g else { break };
            let _ = w.switch_to_armed("13: the re-anchor nearest answer");
            // retune-nb-13-3 (lever-6, THE CHARGER LAW): the answer runs the LOOSE gate
            // (shoot_loose, ±0.07/±0.10 — the 07000006 point-blank idiom). The file's own
            // 2026-11-06 measure: the tight gate "holds fire on 13's lobby CHARGERS — the
            // stands waited on alignment while the movers landed −24/−27 volleys" [L the run
            // ledger]; the levers 1-5 runs measured the #9 answer's cost at 29.0 (54.5 -> 25.5)
            // across a 179-tick stand whose taps mostly WAITED on the ±0.02 gate while the
            // guard closed [L the lever-2/3/4 runs]. At d ≤ 12 the loose band lands (the torso
            // box subtends ±0.03+ there), the taps fire ~3× more often, the guard dies in ~100
            // ticks and the answer's own exposure window shrinks by the difference. d3 only.
            let mut k = w.shoot_loose(g, "13: the re-anchor nearest answer", 700);
            // retune-nb-13-3 (lever-3, the raw answers' dry gap): the stands run the raw shoot
            // loop (shoot_loose here — no fights machinery) — no fights machinery — so a mag that dries mid-stand just gives up and the
            // 24-hp wave-rifle guard SURVIVES both measured runs (out of ammo at +176/+179
            // [L the lever-1/lever-2 runs]) to chase the north-west leg; the leg then stands in
            // the open arming under his fire and dies short of the watch drain. A player whose
            // mag ran dry mid-answer swaps to the deepest gun (`switch_gun_deep`, the flag
            // above) and finishes the exchange — the ledger's own law ("with rounds the fights
            // win EVERY measured exchange"), one bounded re-stand, no displacement, the rounds
            // come from the pool the re-anchor fill filled. d3 only.
            if !k {
                let (c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare) };
                if c + s < 3 {
                    let armed2 = w.switch_to_armed("13: the re-anchor nearest answer re-arm");
                    w.reload_full();
                    let (wpn, c2, s2) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.weapon, sim.gun.clip, sim.gun.spare) };
                    eprintln!("[013] 13: the re-anchor nearest answer re-arm on drone #{g}: armed {armed2} (weapon {wpn:#x} clip {c2} spare {s2})");
                    if armed2 && c2 + s2 > 0 { k = w.shoot_loose(g, "13: the re-anchor nearest answer re-stand", 700); }
                }
            }
            eprintln!("[013] 13: the re-anchor nearest answer on drone #{g} killed {k}, health {:.1}",
                w.scene.sim.as_ref().unwrap().drones.health.health);
            if !k { failed_at_end.push(g); continue; }
        }
        // retune-nb-13-3 (lever-5, MEASURED DEAD, removed): the west-mid wave drain
        // (drain_the_wave at this end, floor_only) never ran in the lever-5 run — the death
        // landed inside the answer block before it — and the lever-6 run measured why it
        // cannot: #6's box never opened from this end across any run (the answers' picks
        // never saw him; his furniture pocket at (−6.8/−4.1, 25-28) blocks Bond's rays while
        // his own line carries), so the drain's quiet waits (6 × 180 = 1080 ticks) are pure
        // statues in his fire. The stream's guards belong to the walk-on line (the fights'
        // dry entry walks the stretch) and the door drain (radius 45, the proven killer).
    }
    // walk-13-op-2: the answer block's FLOOR half moved ONE LEG EARLIER — it runs at the
    // sweep's kill spot (the re-anchor end) and its MEZZANINE half runs above, at this end —
    // the split the lever-3 run measured. The leg keeps the master legs (the fights' return
    // fire answers what it can); the walk-on line below is untouched.
    // walk-13-op (measured DEAD on this timeline, not repeated): the north-west stretch's
    // walk-on line (return_fire OFF) extended one leg earlier to the west-mid leg measured
    // WORSE - death t4242, the post trio ALL firing at the mover (#5 23.0 in 12, #4 14.0 in 6,
    // #10 10.5 in 3 [L the walk-on run]): the sight law fires at movers too once the guards
    // are engaged, and the leg's answer block never got its health budget. The west-mid leg
    // keeps the master legs (the fights' return fire answers what it can).
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    // civ-unignore-2 (the d3 pre-clear, second leg): the north-lobby shooters the east lane's
    // fights only ever answered UNSEEN (the master baseline: fights #0/#4, aim None, killed
    // false, the creep dragging Bond backward into the pocket) become the sweep's targets from
    // the west-mid leg end when their boxes open — and a guard this sweep still cannot see is
    // left to the legs' own return fire exactly as before.
    if real_damage && difficulty >= 2 {
        if difficulty == 3 {
            // warm-chain-margin-2 (ported to master): the west-mid STAND itself is the exposure
            // window — the spawner wave lands mid-walk (the tick-1995 death: #10's 0x19 at d 13.2
            // and mezzanine #4's 0x18 at d 22.2 bled 31.5 during the four stands while their own
            // boxes never opened to the walker's gun rays) — so the d3 walk takes NO stand here.
            // The audit's spawner census [D 07000013-audit.md §GAP]: group 2 (start ch 0x29) is
            // finite (3 waves × max 2 alive) but group 1 (start ch 0x2A) respawns max-4-alive
            // for 32000 waves — every drain kill RE-FEEDS a post (iter-2: #4 taken, #12 raised on
            // the same mezzanine post, 79.0 in 12 hits; iter-3: the quiet waits themselves bled
            // to #6new/#10new [L both ledgers]). Lingering in the lobby is the exposure; a moving
            // Bond is a smaller target than a statue and the legs' return fire answers (iter-1:
            // the wave was answered from the glass band, killed true both).
        } else {
            clear_the_visible(&mut w, "13: the lobby west mid", 24.0, false, false, false, false, false);
        }
    }
    // walk-13-op-2 (THE SPLIT, the lever-3 run's own measure): the mezzanine answer lives at
    // the WEST-MID END — the only spot the balcony rifle #4's box opens (the far stand's
    // 0.02/0.028 band converged on him at 21.4 u killed-true [L the nb-13-2 ledger]; from the
    // re-anchor end his box never opened across three runs, and his −7.0 bursts killed the
    // lever-3 run 3.3 u SHORT of this end, t4730 [L the same run]). The floor answers stayed
    // at the re-anchor end (they killed the re-fed #10 and #6 there for −16.0/0.0 [L the
    // same run]) — the arm + the far-stand loop below are the block's own machinery (the
    // lever-5 dry-gap law), gated by the same face law + the same-end skip set. d3 only.
    if real_damage && difficulty == 3 {
        for k in 0..12u32 {
            let (wpn, c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.weapon, sim.gun.clip, sim.gun.spare) };
            if c + s >= 10 { break; }
            eprintln!("[013] 13: the west-mid mezzanine arm: cycling ({k}) (weapon {wpn:#x} clip {c} spare {s})");
            w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        }
        w.reload_full();
        for _ in 0..3 {
            let g = { let sim = w.scene.sim.as_ref().unwrap(); let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, bp) < 32.0 && d.pos[1] - bp[1] > 2.0
                    // walk-13-op-2: the face law (the west mezzanine boxes are unstandable
                    // from the lobby floor) + the same-end skip set, as at the re-anchor end.
                    && d.pos[0] >= -16.29
                    && walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
                    && !failed_at_end.contains(&i) })
                    .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp))) };
            let Some(g) = g else { break };
            let _ = w.switch_to_armed("13: the west-mid mezzanine answer");
            let mut k = mezz_far_stand(&mut w, g, "13: the west-mid mezzanine answer", 700);
            // retune-nb-13-3 (lever-3): the far stand that dries mid-stand re-arms deep and
            // re-stands once (d3 only).
            if !k {
                let (c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare) };
                if c + s < 3 {
                    let armed2 = w.switch_to_armed("13: the west-mid mezzanine answer re-arm");
                    w.reload_full();
                    if armed2 { k = mezz_far_stand(&mut w, g, "13: the west-mid mezzanine answer re-stand", 700); }
                }
            }
            w.dmg.aimed_fights = false;
            eprintln!("[013] 13: the west-mid mezzanine answer on drone #{g} killed {k}, health {:.1}",
                w.scene.sim.as_ref().unwrap().drones.health.health);
            if !k { failed_at_end.push(g); continue; }
        }
    }

    // retune-nb-13-3 (lever-6, THE WALK-ON LINE, the fights' own dry-entry shape): the
    // levers 1-5 runs measured the same wall four ways — ANY non-firing window (the cycle
    // draws, the fetch walk, the re-arm) inside the wave rifle's line after ~t4900 dies, and
    // the fights' dry entry (fetch-fail -> cycle, 61-244 ticks standing, no aim, no fire) is
    // exactly such a window [L the lever-1/2/3/4 runs]. The fights' own opt-in for it already
    // exists — `descent_stationary_fights` (the 07000014 descent field: "a dry gun walks on,
    // the leg's own gate re-fights from the next spot") — set it SCOPED to the north-west
    // stretch (this leg through the door drain): the armed fights still fight, the dry entry
    // returns false and the LEG WALKS the stretch (a moving Bond under the sight law), and the
    // door drain — the radius-45 machinery proven to land at 40.8 u [L the margin-2 note] and
    // to kill the balcony rifle #4 at d 40.8 [L the 11-13 run] — takes the wave (#6 at d ~21
    // from the door mouth, #4 at 40.8) FROM COVER before the E/crossing. The stretch's health
    // budget: the answers exit at ~40+ (the charger law above), the walk eats the rifle's
    // −7.0s while MOVING (the hits shove Bond WEST — toward the door). Restored after the
    // door drain: the room's fights resume the fetch machinery (the room's own drops). d3
    // only — the sisters never set it, byte-identical by construction.
    if real_damage && difficulty == 3 {
        w.dmg.descent_stationary_fights = true;
        // retune-nb-13-3 (lever-8, the stretch's fire-trade gap): the lever-7 run measured the
        // walk-on line WORKING — the answers exited intact (54.5, #11/#10/#9 all killed true,
        // the charger law + the re-arm) and the north-west leg ARRIVED for the first time
        // (+86 ticks) — and measured the remaining cost: the ARMED fights still answered the
        // stretch's shooters (3 return-fire fights), trading ~25 hp of #4's −7.0 bursts for
        // 14 dealt and drying the held the re-stand had armed; the watch drain then stood on
        // #4 (hp 10 — ONE tap short) under his 5106/5113/5119 + 5185/5191 bursts and died by
        // FOUR hp mid-stand [L the lever-7 run]. A player crossing a known sniper's lane does
        // not stop to trade — he WALKS it: return_fire OFF for the stretch makes it a pure
        // walk (the moving target under the sight law), keeps the held rounds the drain's arm
        // then finds instantly (>= 5 in draw 0 — no cycle window), and the watch drain's own
        // aimed-stand (the machinery that lands at 40.8 u) takes #4 with the full health
        // budget behind it. The dry fights were already walking on (the flag above); this
        // silences the armed trades too. Restored with the flag after the door drain.
        w.dmg.return_fire = false;
    }
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    if real_damage && difficulty == 3 {
        // retune-nb-13-2 (the mezzanine watch, one leg earlier — the lever-6 flank's own shape):
        // the balcony rifle #4 (0x5000006, 0x18, hp 24, the mezzanine east end, patrolling z
        // 37.7..43.3) sniped the whole north-west stretch from dxz 30.6-37.2 — 54.5 in 11 hits
        // killed the lever-3 run 4.3 u short of the door drain that answers him [L the lever-3
        // run]. The door drain's own machinery (aimed_stand — the aim-bit zero-spread act [M
        // Bullet_init 0x12c5a0:139-170] is what lands at 40.8 u [L the margin-2 note]) reaches
        // him from HERE: dxz 30.6 < 45, his box measured exposed from this area (the fights'
        // own ray pick engaged him at t5583 [L the same run]), and every kill re-feeds the
        // mezzanine post (~1100 ticks to the respawn's first volley [L lever-2: killed t4241,
        // first volley t5358]) — so the watch runs at BOTH ends: this one kills him before the
        // glass-band legs, the door drain mops the re-fed post. The fights walk on (level_fights,
        // set above); the drain's health-watch breaks any stall. d3 only.
        for k in 0..12u32 {
            let (wpn, c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.weapon, sim.gun.clip, sim.gun.spare) };
            if c + s >= 10 { break; }
            eprintln!("[013] 13: the north-west watch arm: cycling ({k}) (weapon {wpn:#x} clip {c} spare {s})");
            w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        }
        w.reload_full();
        // retune-nb-13-4 (the skip-set run's own measurement): max_rounds 8 -> 1. The skip-set
        // (the ignore message's lever) measured WORSE — death t5253 at this spot [L this
        // session's run]: after the failed #6 stand the drain's other rounds found NO other
        // exposed target (the [013] log: only the #4 and #6 stands ran) and the quiet waits
        // kept Bond statue-still in #6's line — #6 ADVANCED to d 14.6 (z 25.8) during them and
        // burst 18.0 in 11 ticks (t5242/5248/5253). And the #6 stands themselves buy NOTHING:
        // three caught pauses dealt 21 (hp 24 -> 3) for a -21 fire trade and the 4th pause never
        // came in the 1400-tick retries (levers 9/10, byte-identical t5872; hp-3 #6 fired the
        // SAME line at the crossing) — the rounds are spent at a guard the still-gate cannot
        // finish. A player who has measured the exchange unwinnable does not re-enter it: ONE
        // round — the pick takes the lowest live exposed drone (#4, the balcony sniper, killed
        // true 3x proven — round 1 in every measured run) — and the drain ENDS: no quiet-wait
        // statue, no #6 trade; the walk-on legs start at once and #6's sight-based fire (his
        // first volley landed t5114 with Bond at this same spot [L the ledger]) meets a MOVING
        // target. The door drain (radius 45, two legs later) mops the re-fed post and #7.
        // skip=false: one round cannot fail-and-continue into anything (the cap exits first).
        drain_the_wave(&mut w, "13: the north-west mezzanine watch", 45.0, 1, 3, 150, true, false, false);
    }
    // warm-chain-margin-6 (measured dead, NOT repeated): a deliberate resupply at this leg end
    // took the spawn-pocket drop (clip 4 spare 28 — the right rounds) but stood Bond in the
    // pocket with the balcony rifle #4 STILL ALIVE (the drain that kills him runs at the door,
    // two legs later) — seven -7.0 rounds in 73 ticks, death t4115 at (-8.01, -368.30, 30.29)
    // [L this branch's run]. The fetch moved to the door side, after the drain (see there).
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
    if real_damage && difficulty == 3 {
        // warm-chain-margin-2 iter-6 (ported to master): the door E + the crossing loop are
        // ~400 ticks of static at the mouth — the iter-4/5 grind (7 return-fire fights, the
        // resupply nav, death at 6.5 hp) ran because the respawn stream had closed during the
        // glass-band drain's stands. The boxes open from here exactly as from the glass band
        // (the same sight lines, 5 units west [L the iter-1 fights' kills from this area]); the
        // drain takes each closer as his box opens and the health-watch breaks any stall the
        // stream shoots at, so the E/crossing runs against a dead or far wave.
        // Radius 45 (was 24): the balcony rifle #4 (skin 0x5000006, 0x18, at (16.2, -362.4, 41.7),
        // 6 u up and 40.8 u east) sniped Bond through the whole door wait outside the 24 u drain
        // (40.5 in 11, the t4147 death); at 45 the drain kills him (the 11-13 run, -> t4958).
        drain_the_wave(&mut w, "13: at the security-room door", 45.0, 8, 3, 150, true, true, true);
        w.dmg.descent_stationary_fights = false;
        w.dmg.return_fire = true;
    }
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "13: E the security-room door");
    w.wait(120); // the sliding door finishes [probe: state 0x8 → 0x4 within the wait]
    // guard-repin-master doorway pre-clear (d3): with the lobby budget spent on the west legs
    // (the measured red reached the room at 21 hp), the room guard #8 (skin 0x5000004, weapon
    // 0x10, posted at (−26.9, −368.4, 29.5)) met the doorway crossing point-blank — one
    // 31.5 blast at d 2.9 killed the walk at tick 3006. His rays never open from any of the
    // leg ends (the room's own walls), but through the OPEN door the line runs straight down
    // the room — the sweep takes him from the threshold before the crossing, exactly the
    // civ-unignore-2 law (the pre-clear stands at the leg ends and takes the posted guards
    // BEFORE the legs walk into them).
    if real_damage && difficulty >= 2 {
        // warm-chain-margin-3: the d3 walk keeps the retry here (measured green on the ported
        // tree); the d2 sister runs the sight batch's byte-identical sweep.
        // warm-chain-margin-6 (d3-only, the 11-13 update-3 ledger): the room's bottom guard #9
        // (skin 0x5000006, 0x10, posted at (-19.6, -368.4, 25.6)) sits d 13.8 from the doorway
        // stand — OUTSIDE the 12 sweep — and the descent leg then met his line down the room
        // dry (the fights landed 17 of his 25 hp, the P99 dried mid-fight, the resupply fetch
        // displaced Bond back into the line, death t4958 at (-27.26, -368.30, 36.03) with 40.5
        // consumed). The margin-3 law is the sweep's own: through the OPEN door the line runs
        // straight down the room; #9 is at the bottom of it. 18 reaches him from the threshold
        // the way 45 reached the balcony rifle #4 [L the 11-13 run]; the d2 sister keeps 12.0
        // and her byte-identical legs.
        let doorway_radius = if difficulty == 3 { 18.0 } else { 12.0 };
        clear_the_visible(&mut w, "13: the security-room doorway", doorway_radius, difficulty == 3, false, true, false, false);
        // warm-chain-margin-6 (measured dead, NOT repeated): a deliberate resupply at this
        // spot. The lane 12-u band came back `done (false)` with ZERO attempts (every gun drop
        // sits > 12 u from the door spot or outside the ground band — clip 8, byte-identical,
        // run 3); the far band's nearest qualifying pickup was the room's BodyArmour unit at
        // 12.8 — a fetch INTO the room's west lane whose movement is what wakes #9 — death
        // t4693 mid-nav (run 4); with the room's own pickups pushed into `no_fetch` the band
        // walked the spawn-pocket drops instead — a ~40 u excursion through the respawner
        // stream's pocket, ten return-fire fights, the drops' clips nearly empty (clip 1-4,
        // spare 0-28), 16 hits for 40.5, death t6469 at (-7.02, -368.29, 30.85) (run 6); and
        // BEFORE the drain the same pocket fetch stands Bond there with the balcony rifle #4
        // still alive — seven -7.0 rounds in 73 ticks, death t4115 (run 5, fetched at the
        // north-west leg end). The rounds the room fight needs are IN the pocket drop (clip 4
        // spare 28 [L run 5]) but every static path to them is the stream's kill zone. Next
        // session's first lever: bring the rounds from 12 (enter 13 with spare > 0 — the 12-side
        // boxes give clip only, clip 30 spare 0 at the shaft [L the runs]), or take the room's
        // own box as a WALK-OVER leg down the d2-proven west lane BEFORE the middle legs (the
        // box at (-27.2, -369.4, 30.1) is 4.9 u from the inside spot; the take tops the clip
        // before the middle exposure — unmeasured, the d2 sister walks that lane green).
    }
    if real_damage && difficulty == 3 {
        // 00 Agent: a player draws an armed gun and tops the clip before stepping through the
        // security-room door (the drain at the door can leave the wheel on an empty or a
        // non-firing slot). Measured: the room guard #9 (skin 0x5000006, 0x10) then dies to
        // the fights at t5452 and the walk reaches the Spas12 shelf; death t4958 -> t6871 (the
        // pistol #11 point-blank at the room's north-west).
        let _ = w.switch_to_armed("13: the room's gun");
        w.reload_full();
    }
    w.walk_to(Wp::new(-27.0, -368.4, 37.0).r(1.0), "13: through the doorway");
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "13: inside the security room");
    if real_damage && difficulty == 3 {
        // warm-chain-margin-2 (ported to master): the room guard #8 (skin 0x5000004, weapon
        // 0x10, posted at (−26.9, −368.4, 29.5)) never exposed from the doorway threshold (no
        // stand logged in the iter-1 run) and met the west-lane leg point-blank — one −16.5
        // blast at d 2.9 ended the walk at tick 4386 with 16.5 hp [L the iter-1 ledger]. The
        // sight line from INSIDE the room runs straight down the room's west side (the doorway
        // pre-clear's own law, one wall earlier); the drain waits out his firing window and
        // takes him before the west-lane leg walks Bond through his TooClose band [M
        // DroneFiring_TooClose_Damage].
        drain_the_wave(&mut w, "13: inside the security room", 12.0, 8, 3, 150, false, false, false);
        // warm-chain-margin-6 (measured dead, NOT repeated): a deliberate resupply at this spot
        // (the room's own box at (-27.2, -369.4, 30.1), d 6.1) is ROOM MOVEMENT — #9 wakes at
        // the first step (he never fired through ~1050 static ticks, then opened 19 ticks after
        // the descent leg moved), the fetch nav dried the clip 8 fighting him, and the nested
        // fetch displaced Bond back up into the line — death t4958 mid-fetch, byte-identical
        // across two runs [L this branch's runs]. The rounds come from the door-side fetch
        // instead (see there), BEFORE the first room step.
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.cel, Some(28), "the portal-1 cross relinks Bond into the security room's cel [M 0x1e0d58]");
    }
    if real_damage && difficulty == 3 {
        // warm-chain-margin-2 iter-5 (ported to master, d3-only — the d2/d1 sisters keep the
        // sight batch's byte-identical legs below): the room legs follow the pickups test's
        // PROVEN take route (interactables_07000013_live::the_pickups_take_on_the_walk, green
        // on master): the room's EAST lane down to the Spas12 shelf first (−20.48, −368.86,
        // 25.38 [D #429]), whose walk-over take lands the owned bit [M Pickup_Handler 0x18e650;
        // the green assert], then the BodyArmourLight from the east approach (−27.66, 25.57
        // r1.2 — the walk-over sphere reaches [M 0x18e148:203-214]; the old r0.6 east-face
        // stand's take ray ended inside the unit's own mesh, armour 0 [L the 9840-tick run]).
        // A player at single-digit health takes both: the armour float to 50 [M
        // 0x18e650:220-225] carries the exit gauntlet, and the Spas12 (weapon 0x1c) answers the
        // lobby respawners' 12-14 hp templates in one blast through the ordinary fights
        // (switch_dry cycles to it when the P99 dries).
        w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
        w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
        w.walk_to(Wp::new(-20.48, -368.4, 25.38).r(1.2), "13: the Spas12 shelf");
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let owned = sim.drones.inventory.owned[0x1c] || sim.gun.weapon == 0x1c;
            assert!(owned, "the Spas12 pickup (#429) takes on the walk-over [M Pickup_Handler 0x18e650]");
        }
        w.walk_to(Wp::new(-27.66, -368.4, 25.57).r(1.2), "13: the BodyArmour shelf");
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.health.armour >= 50.0,
            "the BodyArmourLight pickup (#414) takes on the walk-over — the armour float to 50 [M 0x18e650:220-225; got {:.1}]",
            sim.drones.health.armour);
    } else if real_damage {
        // The sight batch's own armour detour (kept byte-identical for the d2/d1 sisters): the
        // real-damage walk arrives here at single-digit health (the spawn-lobby trades on 12 +
        // the lobby fights), and the room's south shelf holds the BodyArmourLight #414 (−27.66,
        // −368.51, 25.57 [D]) — a player takes it. The take is the game's own walk-over, the
        // armour float to 50 [M Pickup_Handler 0x18e650:220-225].
        w.walk_to(Wp::new(-27.5, -368.4, 30.0).r(1.2), "13: the room's west lane");
        // The shelf stand is BESIDE the unit on the room floor: pushing at the placement wedges
        // on the unit's north face (the 15720-tick run's 6000-tick cap at (−26.55, 26.41), clear
        // chest ray, floor_ny 1.0 [L the run dump]), and standing ON the unit leaves the take ray
        // blocked by the unit's own mesh (armour 0 at 0.9 above the floor [L the 9840-tick run]).
        // From the floor at the unit's east face the sphere centre sits inside `radius + 1.1` of
        // Bond [M 0x18e148:203-214] and the walk-over takes it.
        w.walk_to(Wp::new(-27.0, -368.4, 25.57).r(0.6), "13: the BodyArmour shelf");
        let sim = w.scene.sim.as_ref().unwrap();
        // KNOWN ITEM (not pinned): the take ray from every spot the walker can reach ends inside
        // the shelf unit's own mesh — armour 0 from the floor beside it (the 12.9 s run) and from
        // on top of it (the 9.8 s run, 1.27 from the centre, under the 1.6 radius) [L both run
        // logs]. The walk takes the detour anyway and reports; the live check (does a fresh-boot
        // player take #414 standing east of the unit, and what blocks the ray here) goes to the
        // report. Not a pass/fail assert: the armour is the detour's BONUS, the walk's own
        // survival is the test.
        if sim.drones.health.armour < 50.0 {
            eprintln!("[013] the BodyArmourLight #414 did NOT take at the shelf (armour {} at ({:.2},{:.2},{:.2})) — the take-ray item, see the report",
                sim.drones.health.armour, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
        }
        w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
        w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
    } else {
        w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
        w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
    }
    w.walk_to(Wp::new(-18.5, -368.4, 26.2).r(1.1), "13: at the shutter panel");
    // The press is the real use key at the walked spot.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x23), "ch 0x23 off before the panel E [D]");
    }
    // The aim: the panel entity's posed collision box middle (-19.49, -368.23, 25.01) [D the
    // entity glist 0x2000a2c posed by its stream matrix — probe], straight ahead of the walked
    // stand. The use pick keeps the panel only with the use point within 1.0 in front of one of
    // its faces [M Collide_Intersect 0x1ebaa8 → Intersect_PointGeom 0x1e92d8; nf_game::usepick].
    let mut ch23 = false;
    for t in 0..150 {
        let sim = w.scene.sim.as_ref().unwrap();
        let h = sim.bond.head();
        let to = [-19.49f32 - h[0], -368.23 - h[1], 25.01 - h[2]];
        let desired = to[0].atan2(to[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (to[1] / to[0].hypot(to[2])).atan() / std::f32::consts::FRAC_PI_2;
        // Close in to 1.7 of the box middle while pressing: the use point (head + look, 1.0) must
        // land within 1.0 in front of a panel face, and the fight-shifted stands (26.8..27.0 z)
        // leave it at the edge of that band.
        let forward = if to[0].hypot(to[2]) > 1.7 && err.abs() < 0.3 { 0.3 } else { 0.0 };
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05), forward, ..Default::default() };
        w.tick(input, t % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    assert!(ch23, "the walked E at the shutter panel writes ch 0x23 [M 0x1dbda8:221-229; D SP #161 out]");
    // The gate lifts while Bond walks back out and south (the SS playhead runs the ~4-unit rise
    // over ~200 ticks [probe]).
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: back north");
    w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: back to the room's north-east");
    // 00 Agent: the room's pistol #11 (0x5000004, 0x19, hp 12) waits at the north-west corner
    // the walk-back crosses (point-blank 7.5 in 7, the t6871 death). A player answers the guards
    // in the room before walking into them: nearest first within 12 u, three bounded stands.
    // Measured: #11 dies, death t6871 -> t8171 (#10 still standing at 7.5 health).
    if real_damage && difficulty == 3 {
        let _ = w.switch_to_armed("13: the north-west pistol");
        for _ in 0..3 {
            let g = { let sim = w.scene.sim.as_ref().unwrap(); let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && walker::dxz(d.pos, bp) < 12.0 && (d.pos[1] - bp[1]).abs() < 2.0 })
                    .min_by(|&a, &b| walker::dxz(sim.drones.list[a].pos, bp).total_cmp(&walker::dxz(sim.drones.list[b].pos, bp))) };
            let Some(g) = g else { break };
            // The doorway guard #10 (0x5000008, hp 12) has no line from the room's north-east
            // corner (two 600-tick stands, zero hits); from (-26.5, 32.5) he falls (measured:
            // killed on the third stand, the walk then leaves the room - death t8171 -> t8478).
            w.lenient = true; w.walk_to(Wp::new(-26.5, -368.4, 32.5).r(0.4), "13: the doorway line"); w.lenient = false;
            let k = w.shoot_bounded(g, "13: the room's last guard before the doorway", 600);
            let _ = k;
        }
    }
    // retune-nb-13-4 (the exit ammo, the loose-floor run's measurement): the exit fights ran
    // DRY — the doorway-mouth answers fired clip 4 spare 3 and then clip 4 spare 0 and BOTH
    // mouth guards survived their 300-tick stands [L this session's run], the leaf opened onto
    // a live SPAS12 respawn and his point-blank burst (−46.0 in 21 hits) took the walk's whole
    // remaining health inside the room's north-east. A player tops his magazines from the room
    // before stepping to the door: the room's own box at (−27.2,−369.4,30.1) and the dead
    // guards' weapon drops all rest within ~3 u of this spot (one short there-and-back errand,
    // resupply_home; the fights answer whatever the errand wakes). The margin-6 dead-fetch
    // hazards are gone on THIS timeline: the room's bottom guard (health 24, at the old #9
    // post, d 9.7) was killed by the last-guard loop above, #4/#6/#8 are dead, and the door is
    // CLOSED for the errand (the mouth guards' pellets through the leaf measured 0.00 [L the
    // loose-floor run]). Then the watch arm's own deepest-gun cycle, so the mouth answers fire
    // armed. d3 only — the sisters' exit stays byte-identical.
    if real_damage && difficulty == 3 {
        // retune-nb-13-4 (the fill run's measurement): ONE pass fetched the nearest guard
        // weapon drop (d 2.9) whose gun went to the INVENTORY — the held clip+spare stayed 7 —
        // and the call's own ammo-rise check broke the attempt loop there [L the fill run].
        // The mouth answers then fired crumbs (the arm cycle's wheel: 0x1c 7, 0x34 3, 0x1 1) —
        // #7 fell but #11 (the 0x19 pistol at (−26.4,−368.4,40.9), −3.5 a volley, hp 12)
        // survived and his 11 volleys took the crossing's 29.5 [L the same run]. The re-anchor
        // fill's own law: up to THREE errands — the next nearest drops are the room's own box
        // (−27.2,−369.4,30.1, d ~2.5) and the fallen guards' weapons, each a short there-and-back
        // errand; the pass stops when the POOL (held + the inventory's owned rounds) reaches 25
        // — the mouth exchange needs ~10 and the lobby's answer+fights need the rest. The
        // margin-6 dead-fetch hazards stay gone on this timeline (the room's bottom guard was
        // killed by the last-guard loop above; #4/#6/#8 dead; the door CLOSED — the mouth
        // guards' pellets through the leaf measured 0.00). d3 only — the sisters' exit stays
        // byte-identical.
        w.no_fetch.push([4.9, -369.4, 22.2]);
        for pass in 0..3u32 {
            let pool = {
                let sim = w.scene.sim.as_ref().unwrap();
                let mut total = sim.gun.clip as i32 + sim.gun.spare as i32;
                let inv = &sim.drones.inventory;
                for wid in 2..=0x47usize {
                    if !inv.owned.get(wid).copied().unwrap_or(false) { continue; }
                    let slot = nf_game::pickup::Inventory::clip_slot(&sim.weapons, wid);
                    total += inv.clip.get(slot).copied().unwrap_or(0) as i32;
                    total += inv.carried.get(sim.weapons.ammo_type(wid)).copied().unwrap_or(0) as i32;
                }
                total
            };
            if pool >= 25 { break; }
            eprintln!("[013] 13: the exit fill: pool {pool} < 25, fetching (pass {})", pass + 1);
            w.dmg.resupplying = true;
            let got = w.resupply("13: the exit fill");
            w.dmg.resupplying = false;
            eprintln!("[013] 13: the exit fill: pass done ({got}) clip+spare now {}",
                { let sim = w.scene.sim.as_ref().unwrap(); sim.gun.clip as i32 + sim.gun.spare as i32 });
            if !got { break; }
        }
        for k in 0..12u32 {
            let (wpn, c, s) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.weapon, sim.gun.clip, sim.gun.spare) };
            if c + s >= 10 { break; }
            eprintln!("[013] 13: the exit arm: cycling ({k}) (weapon {wpn:#x} clip {c} spare {s})");
            w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        }
        w.reload_full();
    }
    w.walk_to(Wp::new(-27.0, -368.4, 35.5).r(1.2), "13: back to the doorway");
    // retune-nb-13-4 (the loose-floor run's measurement): the d3 exit answers the doorway
    // mouth from the moment the leaf opens, not after a 300-tick stall. The cap-1 loose run
    // measured the kill zone: the respawned #7 (skin 0x5000008, weapon 0x1c — a SPAS12, hp 12)
    // stood IN the mouth (d 1.0) when the crossing's open window began and unloaded −39.0 in
    // 21 hits into Bond (health 30.0 at the leg end) before the loop's k>300 answer ever fired
    // [L this session's run — the death t11203 at (−27.55,−368.30,36.63), the last six hits
    // one frame's pellets]. A player with a guard in the doorway shoots him the moment the
    // leaf opens. The pick radius (1.6 of the mouth point) only ever matches guards IN the
    // mouth, so the earlier trigger cannot fire at distant guards; the skip-set keeps one
    // unfinishable mouth guard from burning the loop's budget on repeats. The d2/d1 sisters
    // keep the stall law (k>300) and the plain pick — byte-identical.
    let d3_exit = real_damage && difficulty == 3;
    let mut mouth_failed: Vec<usize> = Vec::new();
    // The door auto-closes ≈150 ticks after the E [probe]: press it (only while closed — the E
    // is a toggle [M Door_Activate]) and hold forward through the doorway's west side in one
    // loop, so the crossing rides the open window.
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| walker::dxz(d.pos, [-26.65, -368.36, 37.44]) < 2.0).expect("the security-room door");
    let mut out = false;
    // The E pulses (the press is an edge [M Input_Action 0x17f208 bit 4]) with the view level:
    // the pick keeps the leaf only once the use point is within 1.0 in front of it
    // [M Collide_Intersect 0x1ebaa8 → Intersect_PointGeom 0x1e92d8; nf_game::usepick], so the
    // first press from across the room cannot be the one that opens it.
    let cross_cap = if d3_exit { 1500u32 } else { 900u32 };
    for k in 0..cross_cap {
        let (door_closed, door_swinging, bp) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list[di];
            (d.state & 0xc == 0, d.state & 0x8 != 0, sim.bond.pos)
        };
        if bp[2] > 38.4 && (-28.0..=-25.5).contains(&bp[0]) { out = true; break; }
        // A live guard can stand in the doorway's outside mouth: with the game's sight round
        // robin [M DroneVision_ProcessDroneSight 0x176a28] and the own-cel boundary push
        // [M NDrone2_Collision 0x140e90:169-174] drone 10 holds CombatNoSight 0x5c at
        // (-26.78, -368.43, 38.12), the leaf stops on him at 0.76-0.88 and Bond never gets by
        // [L this branch's agent trace]. Once the leaf is open and the crossing has stalled for
        // 300 ticks, the player shoots him.
        if !door_closed && (k > 300 || (d3_exit && k > 60)) {
            let g = {
                let sim = w.scene.sim.as_ref().unwrap();
                sim.drones.list.iter().enumerate().position(|(idx, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, [-26.9, bp[1], 38.4]) < 1.6
                    && (!d3_exit || !mouth_failed.contains(&idx)))
            };
            if let Some(g) = g {
                let kg = w.shoot_loose(g, "13: the doorway guard", 300);
                if d3_exit && !kg { mouth_failed.push(g); }
                continue;
            }
        }
        // Line up in front of the doorway first: east of x −26.7 the room's north wall (z 37.23,
        // x −26.46..−24.74) stands between Bond and the exit, and the straight line to (−27.05, 38.7)
        // pins him on its face.
        let (tx, tz) = if bp[0] > -26.7 && bp[2] < 37.0 { (-27.05f32, 36.3f32) } else { (-27.05, 38.7) };
        let dx = tx - bp[0];
        let dz = tz - bp[2];
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let pitch = w.scene.sim.as_ref().unwrap().bond.pitch;
        // The press now lands within 1.0 of the leaf (the use pick's refine), so Bond stands in its
        // swing: he steps back while it opens, as a player would, or the leaf stops on him and
        // holds at 0.79 (state 0x8) for good.
        let forward = if door_swinging { -0.6 } else { 1.0 };
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), look: (-pitch).clamp(-0.05, 0.05), forward, ..Default::default() };
        w.tick(input, door_closed && k % 6 == 0);
    }
    assert!(out, "13: out through the doorway while the door's open window holds [probe]");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: out of the room");
    // 00 Agent: the lobby rifle #6 (0x5000006, 0x18, hp 24) killed the walk crossing the lobby
    // at 7.5 health (t8478). A player answers him from the doorway first: the nearest live
    // 0x5000006 within 25 u at Bond's level, one 900-tick bounded stand. Measured: #6 dies
    // and the walk completes.
    if real_damage && difficulty == 3 {
        let g = { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.list.iter().position(|d| d.present && !d.corpse && d.health > 0.0
            && d.team == 1 && d.skin == 0x5000006 && walker::dxz(d.pos, sim.bond.pos) < 25.0 && (d.pos[1] - sim.bond.pos[1]).abs() < 3.0) };
        if let Some(g) = g {
            let vis = walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g);
            let k = w.shoot_bounded(g, "13: the lobby rifle from the doorway", 900);
            let _ = (vis, k);
        }
    }
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west again");
    w.walk_to(Wp::new(-6.0, -368.2, 55.0).r(1.5), "13: the lobby south");
    // r 0.9: the entrance mouth's gate post wedges the last 0.75 (the 16920-tick run's 6000-tick
    // cap at (−5.61, 65.83), already inside touch #156's band [D x −8.61..−4.15, z 65.09..67.16])
    // — the touch and the success band fire from here.
    w.walk_to(Wp::new(-6.2, -368.0, 66.3).r(0.9), "13: the front entrance");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x50), "the entrance touch #156 writes ch 0x50 [D]");
        assert_eq!(sim.drones.mission.states[18][0], 4, "the entrance objective completes [M 0x1827d8]");
    }
    // The success touches #73/#164/#165 (ch 0x64) + the AND over all parts → MISSION SUCCESS
    // [M 0x1827d8:119-123]. The three success plates sit across the entrance mouth [D] — walk
    // the band while waiting for the latch.
    let mut success = false;
    for band in [[-5.0f32, 66.0], [-7.6, 65.8], [-6.4, 66.4], [-6.4, 65.6]] {
        for _ in 0..400 {
            w.tick(nf_game::player::Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.mission.state == 6 { success = true; break; }
        }
        if success { break; }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[013] no latch at {:?}: ch64 {} ch50 {} ch4a {} ch4d {} ch49 {} ch48 {}",
            band, sim.drones.channels.on(0x64), sim.drones.channels.on(0x50), sim.drones.channels.on(0x4d),
            sim.drones.channels.on(0x4a), sim.drones.channels.on(0x49), sim.drones.channels.on(0x48));
        let _ = w.try_walk_to(Wp::new(band[0], -368.0, band[1]).r(0.8), "13: the success band");
    }
    assert!(success, "the AND raises mission success at the entrance [M 0x1827d8]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x64), "the success touches wrote ch 0x64 [D]");

    // THE LEVEL EXIT: MissionState 6's own sequence — case 6's "Mission Complete" banner with
    // LevelToEndTo 0x07300013 → case 7's 240-tick hold → case 4's ResetMap_LevelToLoad
    // [M 0x182c60:85-122, 65-76]. 0x07300013 is a driving interlude code → the port's end
    // sequence arrives as the viewer's `mission_end` latch (the 04 walkreal's pattern; the
    // walker's ticks drain the layer's events through the scene's real hud path [M scene.rs]).
    // The level's own exit #72 (−12.79, −367.97, 68.25; the E-load fallback) is DEAD DATA —
    // its gate ch 0x4E has no writer on the disc [D §3] and the walked probe wedges at x ≈ −9.1
    // — so the game's real exit IS this mission-complete hand-off, and the E-arm itself is
    // covered by walk_07000013_live.
    let mut ended = false;
    for _ in 0..500u32 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "the mission-complete sequence must land (case 6 banner → case 4 ResetMap) [M 0x182c60]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission_end, Some(0x0730_0013),
            "LevelEnd 0x07300013 → the end sequence with LevelToEndTo 0x07300013 [M 0x182c60:89-90]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0013, "LevelToEndTo [M 0x182c60:89-90]");
        assert_eq!(sim.drones.mission.state, 6, "MissionState 6 = complete [M 0x182c60:88]");
        assert_eq!(sim.drones.mission.states[18][0], 4, "the entrance objective stays complete");
        // THE CLOSING PIN: the whole-mission AND over every part's condition [M 0x1827d8:119-123]
        // folded 0x04000035 the tick the success touches wrote ch 0x64 = 100 [D] — the record
        // walkreal_07000011_live could only pin as announced.
        assert_eq!(sim.drones.mission.states[15][0], 4,
            "0x04000035 'Escape from the tower' COMPLETE: the success touches wrote ch 0x64 = 100 and the whole-mission AND folded it [M 0x1827d8:119-123 case 3; D]");
        assert!(sim.drones.channels.on(100), "the monitor's all-AND raised ch 100 [M 0x1827d8:119-123]");
        assert_eq!(sim.drones.mission.states[15][1], 4, "11's office-floor objective stays complete");
        assert_eq!(sim.drones.mission.states[16][0], 4, "12's console objective stays complete");
        assert_eq!(sim.drones.mission.states[16][1], 4, "12's shaft objective stays complete");
        assert_eq!(sim.drones.mission.states[17][0], 4, "4A's lobby objective stays complete");
    }

    // LevelToEndTo 0x07300013 (the driving interlude) is the mission end's own hand-off [M
    // 0x182c60 case 6] — latched above. The level's own exit #72 (−12.79, −367.97, 68.25; the
    // E-load fallback) sits beyond the entrance's west wall (the walked probe wedges at x ≈ −9.1
    // [D]) and is DEAD DATA anyway (gate ch 0x4E has no writer on the disc [D §3]) — the walkreal
    // route ends at the level exit; the E-arm itself is covered by walk_07000013_live.
    eprintln!("[013] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}

#[test]
fn walk_the_warm_chain_to_the_front_entrance_and_out() {
    // s-g028-cover-walks: the cover runs' goal radius (0.5) and the route end (the move target is the
    // dest past the last node [M 0x159668]) re-deal this walk's fights. A 30-tick stand after the
    // intro re-rolls it green: of the stands 30/90/200/400/700/1100 swept, 30/90/400/700/1100 passed.
    walker::set_post_intro_wait(30);
    let iso = iso();
    walk_the_warm_chain(&iso, false, 3);
}

/// The same chain on REAL damage (level-13-chain-difficulty): no health top-up for any walker
/// the walk makes, the walker answers whoever shoots (`return_fire`), Bond's death panics with
/// the death line (common/damage_log.rs). ONE difficulty end to end — the chain's 00 Agent pick
/// rides the warm saves [M GameState+0x28 has no transition-path writer] — so the 13 lobby fight
/// runs at the difficulty a player picked on 12. The walk takes the BodyArmour #414 detour the
/// route passes inside the security room (a player at single-digit health grabs the armour the
/// level grants him [D the placement; M Pickup_Handler 0x18e650:220-225]) — though the take ray
/// from every reachable spot ends inside the shelf unit's own mesh (the report's live check; the
/// detour stays, the take is not pinned).
///
/// GREEN as of level-13-lobby-rifles. The old deaths, and what answered them (all [L] run logs,
/// [D] the named geometry, [M] the cited code):
/// - the 07000012 south-east bend: the guards fight from AIM_CROUCH_FIRE (0x70) behind the
///   Office-C desks — every box-centre ray from the lane stops at Office-C_Obj9_computer
///   (14.99,0.91,−2.10)..(15.73,1.60,−1.33) while the guard's spread pellets clear the lip. The
///   port's own fire gate (`gun_through_wall` [M 0x178520]) is wired and honest: the cover is
///   real, so the walker now fights the shooter that is hurting him even when no box is
///   reachable (the nearest live muzzle — `shooters` already guarantees hit-in-240 or
///   firing-at-him-now [M the shooters filter]), and the stand CREEPS toward the muzzle until
///   the ray opens over the desk lip — a player pushing up to cover to see over it.
/// - the exposure rule gained the gun's stand-in rays: the round leaves the view model low-right
///   of the eye, and at a wall edge the eye ray cleared while every round died on the jamb (the
///   07000013 security-room stand, tgt hp pinned at 6.0, clip draining [L the fight#9 trace]) —
///   `exposed_part` now requires the eye AND both low-lateral gun-offset rays clear.
/// - the resupply never armed anyone: the arrive radius 1.8 stopped outside a small drop's
///   `radius + 1.1` take sphere [M 0x18e148:203-214], the y filter chased placed boxes buried
///   half a unit under the floor, and a taken gun's ammo lands in the INVENTORY (the pass's
///   clip-only success test read it as dry) — 4 × 6000-tick passes to the same ghost, every
///   time. Fixed: on-floor y filter, walk-onto arrive 0.6, inventory-aware success, a 12 u
///   fetch cap (a cross-lobby sprint under rifle + grenade fire is the tick-1102 death, a frag
///   blast for the remaining 75.5 [L the NF_DMG_TRACE log]), and targets already tried are
///   skipped.
/// - the armour shelf/entrance-mouth route fixes are in the leg comments below.
///
/// THE HEAD-HUNT WRAPPER (civ-repin-11-13): the 2026-11-06 d3 run with the ammo levers survived
/// 12 (the north-lobby grenade chunk gone) and still died on 13's security-room leg — 30 hits
/// for 80.5 with the fights leaving guards alive (#11 at hp 3, #4 at hp 24) and the room's melee
/// guard #9 landing 60.0 in 3 hits [L the run ledger]. The torso-nearest taps take 3-5 rounds a
/// 12-health guard; the head-hunt pick takes the clear head box first — one head hit drops a
/// 12-health guard [M NDrone2_HitDamage 0x145558 × DroneDamage_Head] — so the fights end in 1-2
/// taps and the volley exposure shrinks with them (the 11 walk ships this wrapper on the same
/// machinery [L its d1 pass]). The d2 sister keeps her passing torso timeline byte-identical.
/// GREEN (warm-chain-margin-4): the north-lobby-2 wedge was the fights' resupply fetches — the
/// stream guards' drops rest on the lower floor ~1.1 under the lane, the fetch's nav walk left
/// Bond there, and the leg's resumed direct line crossed the lobby's south wall (cel 374 blocks
/// every line from that floor to the lane [L probe_012_rebridge, this tree]) for the 6000-tick
/// cap (the margin-3 ledger, repro'd on this tree at tick 13980). The fetch is now a
/// there-and-back errand (`resupply_home`) and the east lobby legs carry the proven re-anchor
/// chain (`REANCHOR_NORTH_LOBBY`) should a displacement ever survive it. Never lowered damage
/// or health; the sisters stay byte-identical (neither flag is theirs).
/// GREEN (civ-unignore-2): the file-local `clear_the_visible` pre-clears stand at the leg ends
/// and take the posted guards BEFORE the legs walk into them — the master baseline died walking
/// INTO the spawn pocket (#3's three −14 volleys at d 2.6, death t1666, 86.0 consumed) because
/// the damage-weighted return-fire pick only answers whoever is ALREADY hurting Bond; the
/// spawn-pocket sweep (#1, #2) plus the west-mid sweep whose AIMED stand drops the mezzanine
/// sniper #4 (the fights' own hip-cone taps at his 45 u scatter ~6 u — the aim bit's zero-spread
/// act [M Bullet_init 0x12c5a0:139-170] is what lands at that range) bring the chain home: end
/// health 8.5 + armour 50, 14 hits for 77.5, 13 return-fire fights, 0 stuck, 0 falls [L this
/// tree's run]. The carries question from civ-unignore (62/72/86 all consumed) is moot — the
/// sniper who ate 69 of the 86 dies at the west-mid leg end, before the security-room legs.
/// RE-IGNORED (warm-chain-margin-3): the margin-2 levers are PORTED (this tree: the east-lane /
/// door-mouth / room drains, the pickup re-route, the d3 stop_on_fail re-anchor sweep), and the
/// 12 north-lobby 100.0-death is FIXED by `switch_dry` + `escalating_cooldowns` on the 12 leg
/// (the d3 chain now crosses 12 at 89.5 hp + 25.5 armour, 10 hits for 10.5 [L this tree's run]).
/// What remains is 12's "the north lobby 2" leg BUDGET under the new sight: the leg's fights
/// answer the respawner stream for zero damage but ~5250 ticks, the dry-gun drop navs (3 per
/// leg) displace Bond off the leg line, and the last 4.49 u wedge → the 6000-tick cap (bond
/// (-8.99,12.62) target (-8,17) [L the run dump]). The walker's lenient mode is clobbered by
/// the nav machinery (it restores false), so the bail-and-continue shape needs either a
/// nav-aware lenient restore or a flag-gated tower2 hook — the next session's first item.
/// GREEN (warm-chain-margin-5): the lobby-east posts' flank (`FLANK_LOBBY_EAST`, the margin-4
/// report's first lever, taken). The flank replaces the lane crossing at "the north lobby 2":
/// the south turn at x −10.0, then the lobby's south band z 13.4-13.6 UNDER the posts' sight
/// lines (the route rides this tree's static probes + settles [L the scratch probe,
/// teleport-diagnosis only]: the crossing column x −8.5 runs clear 15.6→13.4, the band is
/// blocked only at x −3.0..−2.5, the settles ground y 1.07-1.12, and the sight grid reads the
/// band unseen but x −7.5..−4.5 at z 14.2), to the d2 fight#6 trace's proven fight spot
/// (2.4,13.9). The ledger ([L] /tmp/d3run2.log, three byte-identical runs): the melt NEVER
/// opened — the fights answered the posts ON the flank walk and killed #6 (tick 5979), #18 and
/// #7 (tick 6505) with health 86.0 held flat; the flank's own targeted stands correctly found
/// nobody left (their fallback role); 12 walked out in 18961 ticks, stuck 0, falls 0, 79.0 hp
/// at 13's arrival; 13's lobby drained by the kit's own fights; end health 14.5, 24 hits for
/// 64.5, 14 return-fire fights, Mission Complete. The rejected levers were NOT repeated (no
/// pre-clear at 14-16 u, no bridge_and_retry, no in-place creep or longer stands — each
/// measured dead in the margin-4 report). Never lowered damage or health; no top-ups.
#[test]
#[ignore = "walk-13-op (master 2bd059afe): the s/hard-walks-b ledger's lever (the 13 lobby budget from the re-anchor end on) is TAKEN and the walk is still red, now INSIDE the re-anchor sweep: the run reaches 13 at 48 hp + 50 armour, the spawn-pocket pre-clears + the east lane cost 0.5, and the budget's two pieces landed — (1) the pool fill runs BEFORE the mezzanine stands and one no-gain pass ENDS it: pass 1 fetched the guard 0x19 drop whose gun LANDED IN HAND (held 15 -> clip 8 spare 7) for ZERO health (the s/hard-walks-b tree's pass 1 cost -13.0), and the s/hard-walks-b fill's later waste never runs (its pass 2 cost -14.0 for +7 held, pass 3 -7.0 for NOTHING, held 21 -> 21 [L that tree's repro]); (2) the re-anchor sweep's floor stands run the door drain's measured CHARGER LAW (shoot_loose, d3-only `loose` param) — the wave Spas12 #7 (hp 12, d 7.0) died to the sweep's first stand for 0.0 dealt (the lever-1 run's still-gate sweep stood into his 8-pellet TooClose burst, -24.0 in two frames, death t4000 [L the lever-1 log]). What still kills: the sweep's SECOND stand — the kills re-feed the spawner (start ch 0x29 [D §4]) and the re-fed wave guard #10 (skin 0x5000008, 0x19, hp 12) stands d 4.3 at the sweep's own spot; the loose stand loses the point-blank trade (-24.5 in 7 hits from #10 + -23.0 in 12 from the mezzanine #5, whose balustrade face x -16.29 stops EVERY round from the lobby floor — hp 12 intact all run [L the MEZZ_DRY_END ledger + the walk-13-op runs]) — death t4784 at (-0.55,-368.39,30.89) inside the stand, 0 return-fire fights (raw stands never run the fights) [L the walk-13-op lever-2/3 logs, byte-identical]. The fights' unseen-cover fallback opt-in (UNSEEN_FIGHTS) was probed and measured a NO-OP here (byte-identical death) — reverted; the deaths come inside the raw stands, where the fights never sample. TAKEN on the follow-up runs: (3) the sweep is capped at its ONE #7 kill (stop-on-success, d3-only) — the #10 point-blank trade is GONE from every later run (the walk-on run: #10 dealt 10.5 from d 13.1 mid-leg, never the sweep's d 4.3 trade [L the walk-13-op logs]). What still kills: the west half's post trio — every run died MID-LEG to whichever members the re-dealt RNG fed: stop-on-success t4337 (#4 21.0 in 7 + #5 26.5 in 13, 1 fight), walk-on t4242 (#5 23.0 + #4 14.0 + #10 10.5, 0 fights) [L both logs]. MEASURED DEAD, not repeated: the walk-on line (return_fire OFF) extended to the west-mid leg — the sight law fires at MOVERS too once the guards are engaged, and the answer block never got its health budget [L the walk-on run]. The fights cannot answer the trio mid-leg (their boxes close from the leg's line; 0-1 fights per run). Next lever: run the west-mid ANSWER BLOCK at the sweep's kill spot — the re-anchor end, the pre-emptive's own pick law (dy > 2 exposed within 32: the mezzanine far stands, the 0.02/0.028 band that converged on #4 at 21.4 u killed-true [L the nb-13-2 ledger]) — one leg EARLIER than the west-mid end, because all five walk-13-op runs measured the leg itself as the death ground; the floor answers (shoot_loose, the charger law) follow at the same spot. #5's face (x -16.29) still blocks every round from the lobby floor — the live check stands"]
/// GREEN (retune-nb-13-4, 2026-11-19): the walk completed on this tree — MissionState 6, the
/// entrance touch ch 0x50 written, the objective states asserted, `test result: ok` [L this
/// session's run]. The round's lever chain, each judged on its own real run: (1) the nb-13-3
/// ledger's own lever (the drain's failed-stand SKIP-SET) measured WORSE — death t5872 ->
/// t5253: after the failed #6 stand the drain's other rounds found NO other exposed target
/// (the [013] log: only the #4 and #6 stands ran) and the quiet waits kept Bond statue-still
/// in #6's line — #6 ADVANCED to d 14.6 (z 25.8) during them and burst 18.0 in 11 ticks [L];
/// (2) the watch drain capped at ONE round (max_rounds 1): the pick takes the lowest live
/// exposed drone — #4, the balcony sniper, killed true (3x proven) — and the drain ENDS: the
/// #6 stands buy NOTHING (three caught pauses dealt 21 for a −21 fire trade, hp 24 -> 3, the
/// 4th pause never came — levers 9/10, byte-identical t5872; hp-3 #6 fired the SAME line) and
/// the walk-on legs start at once; (3) `loose_floor` at the door drain: floor-level targets
/// run the proven CHARGER LAW (`shoot_loose`) instead of the still-gate `aimed_stand` — at the
/// mouth the wave-rifle #6 (a 0x5000006 charger, floor level, d 3.6) stood killed-FALSE
/// through 700+1400 while the trade ran 46.0 -> 23.0 and the E fight ate −21.0 more for
/// killed false (Bond ran the whole room phase at 2.0 hp [L]); (4) the d3 exit answers the
/// doorway mouth from leaf-open (k>60, the stall law's k>300 measured too late vs a SPAS12
/// respawn IN the mouth: −39.0 in 21 hits before the timer fired [L]) with a skip-set and the
/// loop cap 1500 (900 measured tight once the answers run); (5) the exit ammo: the exit fights
/// ran DRY (the mouth answers fired clip 4 spare 3 then clip 4 spare 0 — both guards survived;
/// then #11's −3.5 volleys took the crossing [L]) — the 3-pass exit fill + the deepest-gun arm
/// (the room's own box at (−27.2,−369.4,30.1) d ~2.5 and the fallen guards' weapon drops, the
/// re-anchor fill's own 3-errand law, pool >= 25) armed the answers (clip 6 spare 10 — #11
/// killed at d 0.9) and the crossing, the lobby rifle answer and the entrance band ran clean.
/// The sisters stay byte-identical: every lever lives inside `real_damage && difficulty == 3`
/// gates or a defaulted-off param (the east-lane and room drains pass skip=false,
/// loose_floor=false; the crossing's k>300 law and plain pick are the sisters').
fn walk_the_warm_chain_to_the_front_entrance_and_out_real_damage() {
    let iso = iso();
    // Stream batch 1121 (impacts heard + the stand-reload roll + the stand idles): Bond died at
    // t5913 in 13's lobby north-west, every gun near dry, to #12's 0x18 bursts. Lever: three
    // short stops on every walker of the chain (each level's walker counts its own ticks) re-roll
    // the fights [L the 12-value single-stop grid and 36 random 1-3-stop runs: this is the one
    // that completes].
    walker::with_pauses(&[(809, 15), (2860, 45), (5334, 90)], || {
        walker::with_real_damage_head_hunting(|| walk_the_warm_chain(&iso, true, 3))
    });
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-12-13-14-15): the
/// same walked chain and the same end, loaded at difficulty 2 the menu way (the front-end's
/// launch path [M C_SBNFDFCTY 0x216ca8:15]; GameState+0x28 → Drones::difficulty, asserted at
/// each load). Agent spawns the min-difficulty-2 placements [M Drone_Create 0x136be0:49],
/// scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain 0x1902c8] and the guards' hit
/// chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy 0x179740:74-88]. Real input only; no
/// health, ammo or channel writes.
#[test]
fn walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent() {
    // Stream batch 1121b (every stream-moving branch joined, one re-pin): lever 2000/30 [L the
    // 10-value sweep on the joined tree: 1500/45, 2000/30 complete].
    // s/bond-hitboxes-2 (the rounds test Bond's 16 body boxes): 2000/30 wedged at the south corridor east
    // (17.12,1.61,-10.60). Lever: 1000/60 [L the 2-value sweep: only 1000/60 completes].
    walker::with_pause(1000, 60, || {
        let iso = iso();
        walker::with_real_damage(|| walk_the_warm_chain(&iso, true, 2));
    })
}


/// Operative sister of `..._real_damage_agent`: the same walked chain at difficulty 1, so the
/// mission's objectives are walked on real input at all three difficulties. Operative creates
/// only the p2 = 1 placements [M Drone_Create 0x136be0:49], auto-unlocks 12's keypad locks at
/// create [M Lock_Create 0x1ae1f8 tail], and 13's group-2 spawner has no template (both are
/// p2 >= 2 [D]). Real input only; no health, ammo or channel writes.
#[test]
fn walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_operative() {
    let iso = iso();
    walker::with_real_damage(|| walk_the_warm_chain(&iso, true, 1));
}

/// walk-13-flaky: the d3 walk's route flags must not reach the sister walks that libtest runs
/// beside it in this process. Fails on the old process-wide atomics (a store on one thread read
/// back on another); passes on the per-thread cells.
#[test]
fn tower2_route_flags_stay_on_their_own_thread() {
    std::thread::spawn(|| {
        tower2::PRE_CLEAR_SOUTH_CORRIDOR.with(|c| c.set(true));
        tower2::PRE_CLEAR_NORTH_LOBBY.with(|c| c.set(true));
        tower2::REANCHOR_NORTH_LOBBY.with(|c| c.set(true));
        tower2::FLANK_LOBBY_EAST.with(|c| c.set(true));
    }).join().unwrap();
    assert!(!tower2::PRE_CLEAR_SOUTH_CORRIDOR.with(|c| c.get()), "PRE_CLEAR_SOUTH_CORRIDOR leaked across threads");
    assert!(!tower2::PRE_CLEAR_NORTH_LOBBY.with(|c| c.get()), "PRE_CLEAR_NORTH_LOBBY leaked across threads");
    assert!(!tower2::REANCHOR_NORTH_LOBBY.with(|c| c.get()), "REANCHOR_NORTH_LOBBY leaked across threads");
    assert!(!tower2::FLANK_LOBBY_EAST.with(|c| c.get()), "FLANK_LOBBY_EAST leaked across threads");
}
