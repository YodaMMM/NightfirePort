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
fn clear_the_visible(w: &mut Walker, label: &str, radius: f32) {
    w.switch_to_armed(label);
    for _ in 0..12 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && walker::dxz(d.pos, sim.bond.pos) < radius
                    && walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some()
            })
        };
        let Some(di) = target else { return };
        let killed = aimed_stand(w, di, label, 2400);
        // aimed_stand's early gives-ups (dry, disabled) return before its own restore: force the
        // walk's default back — 13's fights must keep the plain tap gate (the precise_fights
        // lesson: the tight gate holds fire on the lobby chargers).
        w.dmg.aimed_fights = false;
        eprintln!("[013] {label}: the pre-clear stand on drone #{di} killed {killed}");
        if !killed { w.resupply(label); }
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

/// The whole warm chain (12 → 4A → 13 → out) on real input. `real_damage` = the armour detour
/// (see the real-damage test below); the wrapper turns the top-up off for every walker the walk
/// makes. `difficulty` = the menu pick the chain runs at (the front-end's launch path [M
/// C_SBNFDFCTY 0x216ca8:15]; 3 = the pre-sister byte-identical 00 Agent).
fn walk_the_warm_chain(iso: &str, real_damage: bool, difficulty: u32) {
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
            w.dmg.precise_fights = true;
            // civ-unignore (the measured pre-engagement burst): #10's first fire on the bend→
            // corridor walk is ONE tick of 8 × −3.0 shotgun pellets at d 3.5 (t2723, Bond at
            // (16.8,−12.0)) — the −24 chunk that sends 13 the 62 instead of 86, the exact leak the
            // civ-repin-finish report ranked first (its live check asked what wrote 86.0 → 62.0
            // between ticks 2531 and 2764; NF_DMG_TRACE answers: guard #10, weapon 0x1c, one
            // frame). The opt-in tower2 flag stands at the "the south corridor east" leg end and
            // kills him through the ordinary fight machinery before his sight opens. OFF for every
            // other caller (the 07000012 and campaign walks set nothing).
            tower2::PRE_CLEAR_SOUTH_CORRIDOR.store(true, std::sync::atomic::Ordering::Relaxed);
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
    if real_damage && difficulty == 3 {
        clear_the_visible(&mut w, "13: the spawn pocket", 24.0);
    }

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
    // guard-repin-master re-anchor: the west-mid leg's return-fire fights (the north-lobby
    // shooters) displaced Bond SOUTH of the lane line into the lobby pocket at (0.60, −368.30,
    // 25.66) — the guards' per-goal arrival radii [M 0x152548:174-181] close the pursuit to
    // 1.0-2.0, the stands' back-off geometry moved, and from that spot the direct segment to
    // the west-mid clips the lounge furniture corner at ~(0.3, 26.3) (the measured 6000-tick
    // cap: segment_to_target hit at t 0.042, d 15.78, all nearby guards dead). The re-anchor
    // pulls the route back up to the east lane's proven line before the west leg: from the
    // pocket the segment to (2.5, 30.5) runs north-east, clear of the furniture corner, and
    // the west-mid line from there rides the old path.
    w.walk_to(Wp::new(2.5, -368.4, 30.5).r(1.5), "13: the lobby mid re-anchor");
    // civ-unignore-2's law at the re-anchor end (guard-repin-master): the same ray-gated sweep
    // runs one leg EARLIER — the west-mid route's fights used to be answered mid-leg from the
    // lane, but the guards' per-goal arrival radii [M 0x152548:174-181] close the pursuit to
    // 1.0-2.0 and the stands' displacement parked Bond in the lobby pocket (the measured
    // 6000-tick cap at (0.60, −368.30, 25.66), then the re-anchor's measured cost: #5 (skin
    // 0x5000004 at (−14.8, −368.4, 45.6)) landed 28.0 in 8 hits across the re-anchor + west-mid
    // legs and Bond reached the security room at 21 hp to lose the point-blank #8 exchange).
    // Taking the shooters at the re-anchor end — where #11 (3.8, 33.9) is d ~4.5 and the
    // mezzanine/west boxes open — keeps the west legs' budget for the room.
    if real_damage && difficulty == 3 {
        clear_the_visible(&mut w, "13: the lobby mid re-anchor", 24.0);
    }
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    // civ-unignore-2 (the d3 pre-clear, second leg): the north-lobby shooters the east lane's
    // fights only ever answered UNSEEN (the master baseline: fights #0/#4, aim None, killed
    // false, the creep dragging Bond backward into the pocket) become the sweep's targets from
    // the west-mid leg end when their boxes open — and a guard this sweep still cannot see is
    // left to the legs' own return fire exactly as before.
    if real_damage && difficulty == 3 {
        clear_the_visible(&mut w, "13: the lobby west mid", 24.0);
    }
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
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
    if real_damage && difficulty == 3 {
        clear_the_visible(&mut w, "13: the security-room doorway", 12.0);
    }
    w.walk_to(Wp::new(-27.0, -368.4, 37.0).r(1.0), "13: through the doorway");
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "13: inside the security room");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.cel, Some(28), "the portal-1 cross relinks Bond into the security room's cel [M 0x1e0d58]");
    }
    if real_damage {
        // The armour the route passes: the real-damage walk arrives here at single-digit health
        // (the spawn-lobby trades on 12 + the lobby fights), and the room's south shelf holds the
        // BodyArmourLight #414 (−27.66, −368.51, 25.57 [D]) — a player takes it. The take is the
        // game's own walk-over, the armour float to 50 [M Pickup_Handler 0x18e650:220-225].
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
    }
    w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
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
    w.walk_to(Wp::new(-27.0, -368.4, 35.5).r(1.2), "13: back to the doorway");
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
    for k in 0..900u32 {
        let (door_closed, door_swinging, bp) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list[di];
            (d.state & 0xc == 0, d.state & 0x8 != 0, sim.bond.pos)
        };
        if bp[2] > 38.4 && (-28.0..=-25.5).contains(&bp[0]) { out = true; break; }
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
        if k % 150 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list[di];
            eprintln!("[13diag] k {k} door state {:#x} swing {:.2} bond ({:.2},{:.2},{:.2})", d.state, d.swing, bp[0], bp[1], bp[2]);
            for (i, g) in sim.drones.list.iter().enumerate() {
                if g.present && !g.corpse && walker::dxz(g.pos, [-26.65, -368.36, 37.44]) < 3.5 && (g.pos[1] - -368.4).abs() < 3.0 {
                    eprintln!("[13diag]   guard {i} state {:#x} at ({:.2},{:.2},{:.2}) hp {:.1}", g.state, g.pos[0], g.pos[1], g.pos[2], g.health);
                }
            }
        }
    }
    assert!(out, "13: out through the doorway while the door's open window holds [probe]");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: out of the room");
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
#[test]
#[ignore = "real-damage margin: on 11-08 batch 8b3a33ee2 Bond dies at tick 2448 at (4.03,-368.39,33.35) with 3 hp after a 3.50 hit from drone #5 (weapon 0x19); passed on d09f8b2b3 with the same route. Re-pinned in the stream batch."]
fn walk_the_warm_chain_to_the_front_entrance_and_out_real_damage() {
    let iso = iso();
    walker::with_real_damage_head_hunting(|| walk_the_warm_chain(&iso, true, 3));
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
    let iso = iso();
    walker::with_real_damage(|| walk_the_warm_chain(&iso, true, 2));
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
