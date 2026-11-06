//! walkreal: 07000006 (Mayhew B, the castle interior to the party) walked with the player's own
//! movement — no teleports after the truck-ride spawn. The level has NO 0xE8 LOAD_LEVEL placement
//! [D docs/levels/07000006.md §3]: the single exit is the walk-in LoadPoint Touch #746 (0xEB, out
//! ch 0x83) → ScriptPlayer #810 → the dressing cut-scene 0x06000071, whose closing event 0xD [2,1]
//! completes the level's ONLY objective ("Find a way inside to the party", complete ch 0x02,
//! revealed by ch 0x46 [L ee_start_05.bin record 1]) and whose final event 0x12 [0x07000007] loads
//! the party [M Script_EventHandler 0x1d94c0 case 0x12].
//!
//! THE PROVEN ROUTE (every leg walked through `Sim::step_bond` inputs; anchors [D census] +
//! the walked probes): the truck-ride spawn #496 (57.18, 39.73, 22.61, the carried ch-0x1D gate
//! [M Player_Start 0x190fd0]) stands on the WineTruck's bed/dock (Bond ~1.1 above the ground [D
//! #336 WineTruck @(64.48, 38.00, 28.06)]; the probe: east/south walk free, west/north pin on the
//! bed rails at z≈20.5 / the cab at x≈55.5) → THE DISMOUNT off the bed's open south side (the
//! arrival plates' OR #498 raises ch 0x46 at the spawn; the driver-greeting plates #756/#757/#782
//! wake the friendly driver on the way) → the dock's west end → down the west lane → around the
//! well's west rim (the direct line to the well pins at z≈20.5, x≈31 — the well's north fence [D
//! floor grid]) → the garden hedge gap (the hedges wedge nav-assisted legs; the walked lane
//! crosses the z≈8.8 hedge band at x≈24.4 and turns around the x≈22.6 hedge's south end) →
//! the guardhouse courtyard → E-door #47 Door_Top_I1 (36.50, 40.00, −16.25) → the guardroom
//! FuseBox #747 (28.00, 40.00, −18.15): the stealth route's optional Laser cut (group 0 → ch 57
//! searchlights-off + ch 103 [M FuseBox_Update 0x1adfe8:110-115]) → E-door #59 Door_Top_I2
//! (27.88, 40.00, −23.00, auto-close) → the ramp to head #2's balcony and the drop into the
//! searchlight courtyard (the drop crosses plate #664 → ch 0x34 → SP #808 plays the reveal NIS
//! 0x06000066; the ground mouth from the guardhouse court crosses #526 instead — see
//! the_lamps_light_at_the_ground_mouth; ch 55 rises when a head holds Bond — the alarm is NOT a fail
//! channel on this level [L record 1: fail 0xffffffff; M 0x1827d8:98-107]) → E-door #71
//! Door_Tunnel_G9 (3.75, 42.00, −51.13) → the tunnel climb (y 42 → 49) → E-door #72
//! Door_Tunnel_G9_2 (4.00, 49.00, −64.88) → the upper gallery (y≈50.1, the WineTruck building's
//! top corridor, z −65..−70).
//!
//! THE TUNNEL SWITCHBACK (the climb the walk used to die in): the shaft between doors #71/#72
//! is NOT one straight ramp — the disc's stacked floors [D the dense floor map probed on this
//! tree: floor_below column stacks x 2.0..7.0 × z −50.5..−65.5] show the low ramp 42.0 → 44.8
//! over z −51.5..−58.5 ending at the shaft's south-east corner, an east lane (x 6.25–7.0)
//! carrying the turn 44.6 → 46.8 northward, and the upper ramp 47.2 → 48.8 over z −55.5..−59.25
//! running back south to the 49.0 landing (z −59.75.., door #72 at −64.88). A straight shot at
//! (4.0, 49, −62.5) walks under the upper ramp and grinds into the south-east corner (Bond was
//! lifted by the capsule's floor band against the landing's face, jump apex 46.6 < the 48.8
//! slab [D probe trace]); the walk follows the switchback instead — every leg the player's own
//! walk+jump input through climb_stairs.
//!
//! THE OPEN LEG, CLOSED (2026-10-29): the gallery → the dressing room connection does not exist,
//! because the game never crosses there. The gallery deck's north edge is a BALUSTRADE — the deck
//! ends at z −64.8 and balusters stand at z ≈ −64.92 spanning y 49.25..51.3 from x 30 to 36.7
//! [D this tree's full-edge horizontal-ray sweep (probe_06_gapmap): hits at every sampled y
//! 49.25..51.25, gaps < 0.7 wide] — Bond stops dead against it grounded AND airborne (the edge
//! probe: walking north from (35, −65.6) grinds at z −65.50 with and without jump pulses), and
//! the east door #46 Locked_Door_Top_I is locked forever (ch_lock 200 with no writer on the
//! level [D the door dump: state 2, ch_lock 200; M Door_Activate 0x135be0's is_locked
//! 0x136810:10-16; U-044]). The older header text's "the game's own F3 route must use something
//! the walk does not model yet" resolves to: the game's OWN walkable web (map block 0x05
//! [M AIPath_Parse 0x1d2598], dumped on this tree with probe_06_aipaths) routes the room AROUND
//! the gallery — the "Boundary Prime" web chains courtyard 41.15 (nodes 355/371) → the 43.0 step
//! (370 (19.96, 43.27, −49.95)) → the x≈17.4 ledge (363 (17.49, 43.49, −50.34) → 364
//! (17.44, 45.36, −53.74) → 365 (17.37, 45.26, −64.64)) → the 45.25 shelf at z −64.6 east (366
//! (35.49, 45.25, −64.60) → 367 (35.49, 45.25, −62.28)) → through the 46.0 sill's open gap at
//! x ≈ 36 into the room's 43.1 east strip (377/378) and the LoadPoint. The floors under that
//! chain all exist in the port's collision [D probe_06_gapmap bands: the ledge 44.6..45.9 at
//! x 15..17.5, the shelf 45.0..45.3 z −64.5..−62, the sill 46.0 with 45.0 holes at x ≈ 32/36,
//! the room 41.0 (x 21..28) / 43.0 (x 29..37)] — no port collision gap at all; the walk just
//! follows the web now (walk_the_ledge_to_the_party). The tall 53.0→55.6 stepped block at
//! x 36..39.5 (band A) is the east wing's roof behind locked door #46 — not the route.
//!
//! THE OBJECTIVE INVENTORY [L ee_start_05.bin @ 0x2a3e40 — 1 record, re-dumped against the
//! port's c07000006.rs; docs/systems/objectives-all-levels.md]: 07000006 lists exactly ONE
//! objective:
//! | text       | says                             | completes on | revealed by | asserted where |
//! | 0x04000001 | "Find a way inside to the party" | ch 0x02 — the dressing cut-scene's closing event 0xD [2,1] | ch 0x46 — the arrival plates' OR #498 | walk_the_castle_interior_to_the_party (reveal + fold) |
//!
//! THE STAND-INS THE WALK STILL CARRIES (none is a poke; no bond.pos writes, no door/trigger
//! state pokes, no channel writes after the load anywhere in this file):
//! 1. the fabricated 05-exit RamSave `carry(&[0x1d, 0x45, 0x70])` + `Inventory::level_start(0x73,
//!    240)` [S unverified stock — the real 05-exit save's inventory is not sampled; the carried
//!    gate ch 0x1D itself is [M Player_Start 0x190fd0]]. This is the load-time input, the same
//!    shape walkreal_07000005's exit produces.
//! 2. the harness's `top_up_health` (the walker tops health to 100 every tick — walker.rs) and
//!    its `skip_cutscenes` (the game's own skip action [M SP_Update 0x1dbda8:129 → Script_FFwd
//!    0x1dac78]; ffwd's catch-up still fires every event — see Cut-scene law above).
//!
//! Cut-scene law: the walker's skip is the game's own (skip_request → Script::ffwd [M SP_Update
//! 0x1dbda8:129 → Script_FFwd 0x1dac78]), and ffwd's catch-up still fires every event, so the
//! objective channel and the exit survive a skip — pinned green on master by
//! walk_07000006_live.rs's LoadPoint entry (ch 0x83 → ch 0x02 → level_exit 07000007).
//!
//! Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, RamSave};
use walker::{dxz, Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn mission_row() -> usize {
    nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0006).expect("the 06 record")
}

/// The level's single objective state (states[LEVELS row for 07000006][0] [M mission.rs LEVELS
/// table; L ee_start_05.bin @ 0x2a3e40 — 1 record]).
fn obj06(w: &Walker) -> u32 {
    w.scene.sim.as_ref().unwrap().drones.mission.states[mission_row()][0]
}

/// The save the truck route's Trigger_Activate would make at 07000005's exit [M 0x1b2318:108-116]:
/// the carried gate ch 0x1D selects the truck-ride start #496, ch 0x45 (the 05 reveal) + 0x70
/// (the park) ride along.
fn carry(on: &[u32]) -> RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    // [S] unverified: the real 05-exit RamSave carries the player's full 05 stock; the walkreal
    // harness gives the route room to fight the alarm's reinforcements through the real fire path.
    RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 240), &ch)
}

/// The courtyard searchlight heads (kind 1) — four on 07000006 [D census] — as (state, on-channel lit).
fn heads06(w: &Walker) -> Vec<(u8, bool)> {
    let sim = w.scene.sim.as_ref().unwrap();
    sim.drones.searchlights.parts.iter().filter(|p| p.kind == 1)
        .map(|p| (p.state, sim.drones.channels.on(p.on as u32))).collect()
}

/// The ch-0x34 courtyard plates [D trigger census: TouchOnce #524/#525/#526 and the Spotlight
/// Trigger polys #661-#664, all p0 = 0x34].
const PLATES_34: [usize; 7] = [524, 525, 526, 661, 662, 663, 664];


/// The Laser drawn with the real weapon-cycle input (the disc's own cutting laser, 0x4E) [D
/// USATxt label 0x05000043; the proven fusebox_06_live draw].
fn draw_laser(w: &mut Walker) {
    let draw = Input { switch_to: Some(0x4e), ..Default::default() };
    // Retry the equip: the request only lands when idle (not firing/reloading/holstering
    // [M Player_WeaponChange 0x1a6f80]), and a fight that just ended can leave the gun
    // mid-anim — the 00j run's 40-tick window raced it and the assert read weapon 2.
    for _ in 0..240 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
        w.tick(draw, false);
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Laser equips [D 0x05000043]");
}

/// The alarm's reinforcement guards (the ch-55 spawner #735 [D census]) body-block the courtyard
/// lane; the player shoots them. Any live guard within 8 units of Bond is taken down with the
/// walker's own real fire path.
fn fight_through(w: &mut Walker) {
    w.switch_to_armed("the courtyard fight");
    for _ in 0..4 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                !d.corpse && d.health > 0.0 && d.pos[1] < 46.0 && dxz(d.pos, sim.bond.pos) < 8.0
            })
        };
        let Some(di) = target else { return };
        eprintln!("[06] fighting guard {di} at {:?}", w.scene.sim.as_ref().unwrap().drones.list[di].pos);
        if !w.shoot(di, "the courtyard guard") {
            // Out of ammo: resupply from the ground pickups (the route carries spares).
            w.resupply("the courtyard fight");
        }
    }
}

/// 00-Agent clear (real-health-00agent-06, measured): fight every alive guard within `radius`
/// until the spot is empty, with a per-target tick cap. One 8-u pass kills at most 4 and cannot
/// reach the guardroom's west end (the room is ~12 u deep) — the 00 roster's min-difficulty-3
/// placements (#6/#10/#18/#28 beyond the Agent pair #3/#4) survived every doorway pass and
/// shredded the exposed cut stand (the 00f/00g ledgers). The 13-u radius reaches the whole
/// room [D the room's floor spans x 27.9..36.5, z −23..−16] but also sweeps in the court-side
/// low guards (y 39.0, #27 at (25.4, −14.5)) whose wall-blocked rays no stand can answer — the
/// 00h run burned 18 × the full shoot budget on #27. So: each target gets a bounded budget; a
/// guard still standing after it is OFF the ray (the walker's own sight/LOS rules) and is skipped
/// for the reactive return fire to handle; the clear returns when every in-radius guard is
/// either dead or skipped. The 6000-tick budget: a peeking cover guard (#27, hp 10 after the
/// doorway phase) only exposes while he fires — a 1500-tick window can land entirely between
/// his bursts (the 00j log's "exposed false" at dist 5.1), while a visible guard in the open
/// dies well inside 1000. (real-health-00agent-06-3 measured BOTH arms: skipping a
/// never-exposed target fast leaves the door-route guard #4 alive to shred the resupply walk
/// (97.2 in 22 hits, the death ledger), while the full window kills him — the burn IS the
/// fight; the clear keeps the plain budget. The NOOK clears are the exception: #27's court
/// post is wall-blocked from INSIDE the room at every phase (two windows burned 287+410 ticks
/// and 18 resupply rounds on him, then the second resupply detour was refused and the walk
/// reached the balcony with clip 0 spare 0 — the stands all bailed DRY and the deck post #29
/// killed the unarmed crossing), so the nook clears take `exposed_only` at 00: a target with
/// NO exposed part at pick time is skipped without his budget and re-checked by the next
/// clear from wherever Bond stands next.)
fn clear_the_spot(w: &mut Walker, radius: f32, exposed_only: bool) {
    w.switch_to_armed("the 00 clear");
    let mut skipped: Vec<usize> = Vec::new();
    for _ in 0..12 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).find(|&i| {
                let d = &sim.drones.list[i];
                !d.corpse && d.health > 0.0 && d.pos[1] < 46.0
                    && dxz(d.pos, sim.bond.pos) < radius && !skipped.contains(&i)
                    && (!exposed_only || w.exposed_aim(i).is_some())
            })
        };
        let Some(di) = target else { return };
        eprintln!("[06] clearing guard {di} at {:?}", w.scene.sim.as_ref().unwrap().drones.list[di].pos);
        if !w.shoot_bounded(di, "the 00 clear", 6000) {
            skipped.push(di);
            // Out of ammo or out of budget: restock so the next target starts full.
            w.resupply("the 00 clear");
        }
    }
}

/// One STILL-GATE aimed stand on drone `di` (real-health-00agent-06-2). The mechanic is the 0c
/// yard stand's, level-local here (shared helpers stay opt-in per the merge lessons; this copy
/// runs only inside this file's difficulty-3 mouth tier). The parts that make long-range stands
/// land, each measured on the 0c branch:
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
///   where the view was when the round lands (the round FLIES [M Bullet_update 0x12e7b0]) and
///   there is no lead;
/// - the STRICT gun-ray exposure re-pick every 10 ticks (`DamageLog::exposed_part_gun_rays` —
///   the muzzle-lane ray set, the fights' own idiom).
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
    eprintln!("[06] the aimed stand ({label}): engaging drone #{di}");
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
                if d.corpse || d.health <= 0.0 || !d.present {
                    eprintln!("[06] the aimed stand ({label}): drone #{di} VANISHED at +{t} (corpse {} health {:.1} present {})", d.corpse, d.health, d.present);
                    return true;
                }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 {
                    eprintln!("[06] the aimed stand ({label}): drone #{di} bailed DRY at +{t} (clip 0 spare 0)");
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
                // 1500: the far deck posts (d 27+) expose in firing windows — the 500 default
                // aborted before a first hit ever landed on the merged-tree runs (the mouth
                // census: #31 rays1 alive through three stand passes). A landing round kills a
                // 10-hp guard, so the wait is the tactic, not a stall (the cap still bounds it).
                if t - last_hit > 1500 {
                    eprintln!("[06] the aimed stand ({label}): drone #{di} bailed NO-HIT at +{t} (still {still}, aiming {})", sim.aiming);
                    return false;
                }
                if t % 10 == 0 { aim = walker::DamageLog::exposed_part_gun_rays(&sim, di); }
                // 200 (real-health-00agent-06-3, run 8's ledger): the roof guard #11's parapet
                // flicker reads rays1 at pick time, rays0 for ~100-tick lean-in gaps — the old
                // 80 give-up bailed mid-window and the pick re-engaged him thirteen times
                // without one landing tap. A lean-out window that lasts 100 ticks still ends
                // in a tap within 200.
                if aim.is_none() { unseen += 1; if unseen > 200 { eprintln!("[06] the aimed stand ({label}): drone #{di} bailed UNSEEN at +{t}"); return false; } } else { unseen = 0; }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let at = aim.unwrap_or([d.pos[0], d.pos[1] + 0.3, d.pos[2]]);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let desired_yaw = dx.atan2(dz);
                let desired_pitch = (dy / h).atan();
                // The cursor drive + the effective-aim error (the 0c convergence — the round
                // obeys the view + the eased offsets, not the view alone).
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
    eprintln!("[06] the aimed stand ({label}): drone #{di} killed {killed}, health {:.1}",
        sim.drones.health.health);
    killed
}

/// 00-Agent mouth tier (real-health-00agent-06-2): AIMED stands on every alive hostile within
/// 40 u whose strict gun rays expose a part — any height. The mouth's killers are HIGH (#11, the
/// tunnel-roof guard y 55.5; the deck posts y 47) and sit outside the y-bounded ground clear by
/// design (the y-60 filter experiment measured NEGATIVE — the perch guard #23 stalls the walk —
/// the prior session's 00m run), and the ch-55 reinforcement pair spawns during the guardroom
/// fights on the merged tree (the courtyard's own volume raises the alarm regardless of the cut
/// [D census]) and grinds from the strip. A player stands at the plaza, aims and drops each one
/// as it pauses. The recent-hitter first: whoever is actually hitting Bond dies before the rest.
fn aimed_stand_the_mouth(w: &mut Walker) {
    // The pick's census (scratch): every alive hostile within 40 u and its ray state.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = sim.bond.pos;
        let near: Vec<String> = (0..sim.drones.list.len()).filter_map(|i| {
            let d = &sim.drones.list[i];
            if d.corpse || d.health <= 0.0 || !d.present { return None; }
            let dxz = ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt();
            if dxz > 40.0 { return None; }
            Some(format!("#{} team{} en{} hp{:.0} y{:.1} d{:.1} rays{}", i, d.team, d.enabled as u8,
                d.health, d.pos[1], dxz, walker::DamageLog::exposed_part_gun_rays(&sim, i).is_some() as u8))
        }).collect();
        eprintln!("[06] the mouth pick census: {}", near.join(" | "));
    }
    for _ in 0..8 {
        let di_opt = {
            let sim = w.scene.sim.as_ref().unwrap();
            let now = sim.drones.frame;
            let b = sim.bond.pos;
            // real-health-00agent-06-3 (measured on master c2c4a6e, run 8): NEAREST first, not
            // recent-hitter first. The tunnel-roof guard #11 grazes the whole mouth/landing
            // band from d 29-41 (0.00 hits — they still register as player_hits), so
            // hitter-first re-engaged him THIRTEEN stands in a row (each a flicker bail: his
            // parts poke in and out of the roof parapet, the 80-tick unseen give-up) while the
            // ch-55 pair walked unopposed into sight and killed the walk. A player triages the
            // CLOSEST threat: the pair at their posts (d ~14.7, outside their 12 u sight) dies
            // to two still-gate taps each, THEN the roof guard gets his window.
            let mut best: Option<(usize, f32)> = None;
            for i in 0..sim.drones.list.len() {
                let d = &sim.drones.list[i];
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1) { continue; }
                let dxz = ((d.pos[0] - b[0]).powi(2) + (d.pos[2] - b[2]).powi(2)).sqrt();
                if dxz > 40.0 { continue; }
                if walker::DamageLog::exposed_part_gun_rays(&sim, i).is_none() { continue; }
                if best.map_or(true, |(_, bd)| dxz < bd) { best = Some((i, dxz)); }
            }
            best.map(|(i, _)| i)
        };
        let Some(di) = di_opt else { break };
        aimed_stand(w, di, "the mouth", 2400);
    }
}

/// civ-unignore-2: the aimed-mouth sweep with the pick GATED on "actually shot Bond" — a recent
/// hitter (any range to `radius`) the strict gun rays expose, recent-hitter first. The mouth
/// sweep's plain ray pick engages ANY exposed post in 40 u, which at the dock would answer the
/// friendly driver's cell (the driver never fires; the measured lesson above). Here only a
/// hostile who has hit Bond in the last 600 frames is a target — the player's own rule, the
/// fights' shooters window widened to the guards' real reach (the plain 20 u window is why the
/// dock guard plinked free: 27-35 u is outside it but inside his own).
fn stand_shooters_down(w: &mut Walker, label: &str, radius: f32) {
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
                if walker::DamageLog::exposed_part_gun_rays(&sim, i).is_none() { continue; }
                best = Some(i);
                break;
            }
            best
        };
        let Some(di) = di_opt else { break };
        aimed_stand(w, di, label, 2400);
    }
}

/// The walk from the spawn to the searchlight courtyard's tunnel-door approach (the point where
/// the game's own web [D Boundary Prime] splits: west into the tunnel, or the ledge route east
/// to the dressing room). `cut` does the stealth route's fusebox Laser cut in the guardroom
/// (group 0 → ch 57 + 103); without it the searchlights stay live and the courtyard raises the
/// ch-55 alarm — neither blocks the walk (no fail channel on this level [L record 1; M
/// 0x1827d8:98-107]).
fn walk_to_the_courtyard(w: &mut Walker, cut: bool) { walk_to_the_courtyard_at(w, cut, 1) }

/// `difficulty` threads the sister's Agent tactics (the door #59 mouth step); 1 = the old body.
fn walk_to_the_courtyard_at(w: &mut Walker, cut: bool, difficulty: u32) {
    walk_to_the_guardhouse_court(w, difficulty);
    walk_from_the_guardhouse_court_at(w, cut, difficulty);
}

/// The spawn → the guardhouse court east of the searchlight courtyard, with its idle guard down.
fn walk_to_the_guardhouse_court(w: &mut Walker, difficulty: u32) {
    // THE DISMOUNT: off the truck bed's open south side (the spawn's own facing walks onto the
    // bed; the probe drops grounded at z≈29).
    w.walk_nav_to([60.0, 38.3, 31.5], "off the truck, south", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] > 28.0, "the dismount clears the truck's south side, got {:?}", sim.bond.pos);
    }
    // civ-unignore-2 (the Agent dock shooters): the dock's east guard (#14) posts at the truck
    // d 27-35 from every early leg — OUTSIDE the fights' plain 20 u shooters window — so the
    // return fire never answers him while he plinks the whole west lane (the measured d2
    // ledger: 25.0 in 13 hits, the walk's biggest single dealer, still hp 10 at the strip
    // death, and his rounds bled every west-lane stand). A player answers whoever is shooting
    // him. (A BLIND radius clear here is WRONG — measured this session: clear_the_spot(8.0) at
    // the dismount shot the friendly driver #17 (hp 1, (66.1,39.2,28.9), never a shooter) and
    // his death latched the level exit 303 ticks later, killing the walk.)
    if difficulty >= 2 {
        stand_shooters_down(w, "the dock shooters", 40.0);
    }
    // Across the driver-greeting plates (#756/#757/#782 → ch 0xB9) on the ground west of the dock.
    w.walk_nav_to([45.0, 38.3, 24.0], "across the greeting plates", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x46), "the arrival plates' OR #498 reveals the objective (ch 0x46) [D #498 out 0x46]");
    }
    // The dock's west end, down the west lane, around the well's west rim (the nav graph has no
    // coverage here; each anchor was walked by the probe).
    w.walk_nav_to([24.3, 38.3, 27.6], "the dock's west end", 1.6);
    w.walk_nav_to([24.6, 38.5, 21.5], "down the west lane", 1.6);
    // civ-unignore-2: answer the dock guard now that he has opened fire (his first hits land
    // across these legs at d 27-35; see the ledger note at the dismount).
    if difficulty >= 2 {
        stand_shooters_down(w, "the west lane shooters", 40.0);
    }
    w.walk_nav_to([26.5, 39.1, 13.1], "the well courtyard's west side", 1.6);
    if difficulty >= 2 {
        stand_shooters_down(w, "the well shooters", 40.0);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dxz(p, [26.5, 39.1, 13.1]) < 3.0, "the well courtyard's west side reached, got {p:?}");
    }
    // The garden hedges wedge nav-assisted legs (the global roadmap routes east into the z≈8.8
    // hedge); the probe's walked lane crosses the hedge band at x≈24.4, stands at the x≈22.6
    // hedge face, then turns southeast around its south end to the guardhouse courtyard.
    w.walk_to(Wp::new(23.2, 39.1, 7.0), "the hedge gap");
    w.walk_to(Wp::new(27.2, 39.1, -2.4), "around the hedge end");
    w.walk_to(Wp::new(33.7, 39.1, -8.9), "the guardhouse court");
    // The idle guard #438 (27.51, 39.18, −10.19) [D census] blocks the lane to door #47; the
    // real fire path kills him (10 hp — the fire-path pins in fix_06_fire_live.rs walk this
    // same fight with every press firing).
    fight_through(w);
}

/// The guardroom route into the searchlight courtyard: door #47, the fusebox nook, door #59, the
/// west ramp up to head #2's balcony (floor 46.0 at x 7..10.8, z −26..−22 [D floor probe]) and
/// off its west edge onto the 41.0 courtyard floor — the drop passes through the Spotlight
/// Trigger poly #664 (y 45.93, x −0.8..6.8, z −25.7..−21.0 [D trigger dump]).
fn walk_from_the_guardhouse_court(w: &mut Walker, cut: bool) { walk_from_the_guardhouse_court_at(w, cut, 1) }

fn walk_from_the_guardhouse_court_at(w: &mut Walker, cut: bool, difficulty: u32) {
    w.walk_to(Wp::new(36.0, 39.1, -13.4), "before door #47");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(dxz(p, [36.0, 39.1, -13.4]) < 3.0, "door #47's approach reached, got {p:?}");
    }
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [36.5, 40.0, -16.25])
            .expect("door #47 Door_Top_I1 in the doors list [D census]");
        eprintln!("[06] door {di} at {pos:?}");
        w.use_door(di, "door #47");
    }
    // 00-Agent tactics (real-health-00agent-06, measured): crossing the guardroom under fire
    // costs −71.6 at 00 — the reactive return fire only answers AFTER each hit, and the two
    // guardroom guards (#4, #3 — #3 is the difficulty≥2 spawn at (29.9, 41.0, −19.5), 7.3 u
    // from the doorway, inside the fight radius) land 31 hits while Bond walks to the nook
    // (91.8 → 20.2, the 00 run's ledger; at Agent the same crossing costs ≤4 — accuracy 0.9 →
    // 1.3 and pain 0.7 → 1.0 [M ReadTuningVars 0x1d3a80 case 0x7000006]). The player clears
    // the room from the doorway BEFORE the crossing, and again at the nook (any survivor).
    // civ-unignore-2: the guardroom pre-clears run at Agent too — the d2 ledger measures the
    // crossing fight against the room's own #3 spawn (30.3,41.0,−17.0) landing 3 × −3.85 at
    // d 2.1-2.8 (point-blank, inside the fight radius) before the reactive fight dropped him.
    if difficulty >= 2 {
        clear_the_spot(w, 13.0, false);
    }
    // 00-Agent tactics (real-health-00agent-06-3, measured on master c2c4a6e): the pre-clear's
    // budget windows outrun door #47's 360-tick auto-close [M Door_Update] — the nook nav then
    // met the leaf at pos01 0.50 state CLOSING and burned 4 × 3000 stuck ticks at the jamb
    // (the stuck ladder's 2.8 u door-find radius missed the door at d 2.96, so nothing re-pressed
    // E), and the 5th attempt's recovery diverged to the hedge under the deck snipers (the deck
    // posts' sight 48-49 u [D census #446 sight 48/#448 sight 49] reaches the whole guardhouse
    // court; the run's ledger: 45.0 in 1 hit at d 35.9 from the y-47 deck). Two real-input
    // moves: walk back to the proven pre-door spot first (the nav leg then starts from the same
    // node the green d1/d2 runs start from — the stuck run's nav 5 re-routed through door #59
    // from the clear's displaced stand), and re-press E at the door before walking through.
    if difficulty >= 3 {
        w.walk_to(Wp::new(35.5, 39.1, -12.7).r(1.5), "back to the door-#47 approach");
        if let Some((_, di)) = walker::door_near(w.scene.sim.as_ref().unwrap(), [36.5, 40.0, -16.25]) {
            w.use_door(di, "door #47 re-opened after the pre-clear");
        }
    }
    // Through #47 into the guardroom; the nav graph covers this room [D leg probe: 4 nodes].
    w.walk_nav_to([29.9, 40.5, -17.5], "the guardroom fusebox nook", 1.4);
    // 00-Agent tactics (real-health-00agent-06, measured): at 00 the two guardroom guards
    // (#4, #3) shoot the exposed Laser cut to pieces — the nook fights landed −22.0 (9 hits)
    // and −44.0 while the walk stood at the fusebox (the 00 death ledger: 91.8 → 20.2 across
    // ticks 2761-2872; at Agent the same fights cost ≤4 total). The player clears the room at
    // range BEFORE standing at the fusebox, and again before the pilaster lane's mouth step.
    if difficulty >= 2 {
        clear_the_spot(w, 13.0, difficulty >= 3);
    }
    if cut {
        if difficulty >= 3 {
            // 00-Agent tactics (real-health-00agent-06, measured): the doorway fight drops #3
            // at (30.26, 41.0, −16.95) — 0.9 u off the nook stand's ray to the group-0 wire
            // (28.00, 40.00, −18.15) — and the corpse eats the Laser beam (the 00e run's
            // "shoot_point cap": the wire never burned off the nook stand, while
            // fusebox_06_live burns the same wire from a corpse-free room). The player steps
            // north-east off the bodies and cuts from there — the ray to the wire stays clear.
            w.walk_to(Wp::new(30.9, 40.5, -16.2), "the cut's clear stand, off the fight corpses");
            // Re-clear at the stand: the doorway skips (#27 — outside-north post (25.4, −14.5),
            // wall-blocked from every inside stand) CHASE Bond and come back on-ray here —
            // the 00i ledger: #27 followed him to the stand and machine-gunned the whole cut
            // (22 hits for 61.0) with hp 10 left. From the stand its corner post is visible.
            clear_the_spot(w, 13.0, false);
            // Back to the nook for the cut itself — the fusebox_06_live-proven stand for the
            // group-0 wire ray (28.00, 40.00, −18.15). The 00k run cut from the offset stand
            // and capped (the ray blocked there); the 00e run cut from the nook with #27 still
            // alive and the reactive fights interrupted the taps — with the room clear and the
            // chaser dead, the nook cut runs unmolested.
            w.walk_to(Wp::new(29.9, 40.5, -17.5), "back to the nook for the cut");
        }
        draw_laser(w);
        let wire = {
            let sim = w.scene.sim.as_ref().unwrap();
            let fb = sim.drones.fuseboxes.first().expect("the 06 fusebox #747 [D census]");
            fb.alive_wires().filter(|(g, _, _)| *g == 0)
                .filter_map(|(_, _, o)| nf_game::drone::Drones::script_obj_pos(&sim.drones.scripts, &sim.data(), o))
                .next().expect("the group-0 (searchlight) bundle is registered at load [M FuseBox_Register 0x1b0970]")
        };
        eprintln!("[06] burning the group-0 wire at {wire:?}");
        assert!(w.shoot_point(wire, "the fusebox laser cut",
            |sim| sim.drones.channels.on(57) || sim.drones.channels.on(103)),
            "the Laser cuts the searchlight bundle [M 0x1adfe8:110-115]");
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(57), "ch 57 — the searchlights-off gate — is on [D #747 p1]");
        assert!(sim.drones.channels.on(103), "the bundle's second word ch 103 fires [D #747 p3]");
    }
    // 00-Agent tactics (real-health-00agent-06-3, measured): the nook phase leaves the gun
    // dry (the doorway windows + the #27 burns drained 240 rounds by the balcony in the run-3
    // ledger — the stands all bailed DRY and the deck post #29 killed the unarmed crossing).
    // The room's OWN ammo boxes sit at its south-east [D census #666 AmmoPickUp300Soft
    // (34.27,40.01,−24.12) + #770 Ammo_5_56mm (34.72,40.00,−24.45)] — a player low on rounds
    // loots them on the way to the pilaster lane. The walker's resupply refuses off-corridor
    // detours, so the loot is a walked leg of its own, then the resupply takes the box.
    if difficulty >= 3 {
        let dry = { let sim = w.scene.sim.as_ref().unwrap(); sim.gun.clip + sim.gun.spare < 60 };
        if dry {
            w.try_walk_to(Wp::new(34.5, 40.5, -24.2).r(1.4), "the guardroom's ammo boxes");
            w.resupply("the guardroom's ammo boxes");
        }
    }
    // 00-Agent tactics: second guardroom sweep after the (optional) cut — anything the
    // doorway fight left alive (or that wandered in behind Bond) dies here, before the
    // pilaster lane's mouth step.
    if difficulty >= 2 {
        clear_the_spot(w, 13.0, difficulty >= 3);
    }
    // West out of the guardroom through the auto-closing door #59 Door_Top_I2 (27.88, 40.00, −23.00).
    if difficulty >= 2 {
        // [D coll-scan of the guardroom's west wall, this branch] the wall carries a pilaster
        // whose east face is the plane x 28.61 over z −19.71..−21.36 (vertical face strips at
        // x 28.61, ledge bands y 40.00..40.22) with its solid top ≈42.2 (the floor grid's 42.2
        // hits at x 28.5, z −20..−21); the guardroom floor is 40.0 throughout — there is NO
        // upper level here. The direct southwest steer to (27.9, −21.0) grazes the pilaster's
        // north-east corner and corner-catches Bond exactly at it + the body radius:
        // (28.55, −19.16), d 1.95, through the whole 6000-tick cap. The stuck samples'
        // "floating" y (41.56/41.83 ungrounded) are the ladder's own jump pulses [M
        // walker.rs stuck 3/4/5 jump] — the grounded sample reads 41.13 = the 40.0 floor +
        // 1.1, and nothing standable exists at that xz (the open hatch #748's leaf parks at
        // z −18.53..−18.42, 0.6 north; the pilaster top is 0.5 east). At Agent the extra
        // spawn drone #3 fights from (29.9, 41.0, −19.5) — 1.4 from the catch point — and
        // the fight delivers Bond into it. The player rounds the pilaster through the east
        // lane (clearance > 0.55 off the x 28.61 face), south of its −21.36 end, then to the
        // door #59 mouth.
        w.walk_to(Wp::new(29.7, 40.6, -20.9), "the pilaster's east lane");
        w.walk_to(Wp::new(29.4, 40.6, -22.3), "south of the pilaster");
    } else {
        w.walk_nav_to([27.9, 40.0, -21.0], "before door #59", 1.4);
    }
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [27.88, 40.0, -23.0])
            .expect("door #59 Door_Top_I2 in the doors list [D census]");
        eprintln!("[06] door {di} at {pos:?}");
        w.use_door(di, "door #59");
    }
    // THROUGH the doorway (level-06-castle-fire): without this waypoint the mouth leg aims
    // through the guardroom's west wall (the auto-closing doorway is a nav blind spot).
    // Agent tactics, opt-in for the difficulty-2 sister: the Agent run's fights stretch the
    // guardroom legs, and by the time the through-leg walks the auto-closing door #59 may
    // have begun its close cycle. The player closes the distance to the mouth before the
    // leaf shuts: step into the doorway first, then through.
    if difficulty >= 2 {
        w.walk_to(Wp::new(27.9, 40.2, -22.4), "to the doorway mouth");
    }
    w.walk_to(Wp::new(26.6, 40.6, -23.6), "through door #59");
    // West up the ramp to head #2's balcony and down into the courtyard (the nav leg drops off the
    // balcony's west edge through plate #664 → ch 0x34 → SP #808's reveal NIS 0x06000066).
    // 00-Agent tactics (real-health-00agent-06-2, measured on the merged tree): Bond dies ON
    // the balcony BEFORE the mouth tier ran — the death spot (8.54, 47.13, −21.81) is the
    // drop's edge, and the ch-55 reinforcement guard (#20, (6.7, 42.0, −33.7), d ~13) plus the
    // tunnel-roof guard (#11, (−0.4, 55.5, −51.0), d ~32) shoot the drop itself (the merged
    // tree's ledger: 27.5 in 5 hits from #20, 11.0 in 2 from #11). So the stands run at the
    // balcony's west reach — a real floor spot on the drop's edge (the death spot is grounded
    // proof the floor holds there) — BEFORE the drop, with the whole courtyard in the gun rays.
    if difficulty >= 3 {
        w.walk_nav_to([10.5, 47.1, -21.8], "the balcony's west reach", 1.6);
        aimed_stand_the_mouth(w);
        // THE DROP (real-health-00agent-06-3, measured on master c2c4a6e): with the stands
        // done the player walks west off the balcony — the game's own route (the d1 pin: the
        // route's first plate IS the drop poly #664). Without the walked drop the mouth nav
        // from the balcony follows the nav graph's ramp lane back down east on this master —
        // no ch-0x34 ever fires and the courtyard pin fails with the whole courtyard already
        // crossed clean (run 4: 64.2 hp at the tunnel approach, the deck posts gone).
        w.try_walk_to(Wp::new(3.0, 41.0, -23.0).r(1.5), "the balcony's west-edge drop through #664");
    }
    if difficulty <= 2 {
        // WHY THE ROUTE MOVED (the civ-repin-06 diagnosis): the mouth leg below has NO nav chain
        // from here — leg() returns the bare two-point steer (the session probe: nav_route and
        // nav_route_global both fail from the door-#59 arrival to the mouth, every ±1 u start
        // variant included) — so the leg is a straight steer whose stuck-jump wandering decides
        // which high ledge Bond captures. The top-up run's wandering climbed the west ramp and
        // fell off the balcony through the reveal plate #664 [L the passing run: rise ch 52 at
        // t3313 at (3.45,46.44,−21.54) touching 664]; the real-damage runs' fights (the
        // civ-alarm branch's earlier acks land during the guardroom legs) displace the wedge
        // position, the same wandering captures the north-east deck instead, and the first
        // ch-0x34 plate crossed became #662 — its box spans the whole mouth's east at y 45.30,
        // z −21.08..−10.55 [D trigger dump; the failing run's rise at (8.47,45.88,−15.10)
        // touching 662]. #661/#662/#663/#664 all write the reveal ch 52 [D trigger dump probe:
        // out 52 on all four]. The player walks the ramp ON PURPOSE: the door-#59 west lane IS
        // the ramp (floor 40.8 → 41.6 → 43.0 → 44.6 → 46.0 along z −23..−24, x 28 → 13 [D
        // world floor probe this session]), the balcony runs west at 46.0-46.9 to its edge at
        // x ~7, and the edge drop falls through #664's box (y 45.93, x −0.79..6.77, z
        // −25.70..−21.05 [D]) into the 41.0 courtyard — south of #662's z range the whole way.
        // Short legs with tight radii: a fight strafe displaces one leg and the next re-aims
        // (the wedge-luck is gone).
        w.try_walk_to(Wp::new(21.0, 42.0, -23.5).r(1.2), "the ramp's lower lane");
        w.try_walk_to(Wp::new(14.0, 44.8, -23.5).r(1.2), "the ramp's upper lane");
        w.try_walk_to(Wp::new(9.5, 46.0, -23.0).r(1.0), "the balcony's west reach");
        // The drop target sits WELL west of the edge with a tight radius: a nearer/wider target
        // arrived ON the edge lip (the first cut's r 2.0 fired at x 6.44 while Bond was still on
        // the jump arc at y 48.28 [L this session's run]) and the mouth leg's north-east steer
        // then walked him off the balcony's NORTH edge — z −21.0 is #662's own z boundary — and
        // the descent started from the wrong courtyard spot. From (3.0,−23) the westward walk
        // steps off at x ~6.8, the fall crosses #664's plane (y 45.93) inside its box, and the
        // landing (x ~3-5, z −23) re-aims the mouth leg from the 41.0 floor — 3+ units below
        // #662's plane, untouchable.
        w.try_walk_to(Wp::new(3.0, 41.0, -23.0).r(1.5), "the balcony's west-edge drop through #664");
    }
    w.walk_nav_to([8.4, 42.0, -12.0], "the searchlight courtyard mouth", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[06] at the mouth: ch52 {} ch55 {}", sim.drones.channels.on(52) as u8, sim.drones.channels.on(55) as u8);
    }
    fight_through(w);
    // 00-Agent tactics: a ground clear at the mouth before the descent. The phase's real
    // killers are HIGH (#12, the 00-roster deck guard at y 47.0, weapon 0x25, ONE 45.0 hit at
    // d 26.9; #11, the tunnel-roof guard at y 55.5) and stay outside any y-bounded ground
    // clear by design — raising the filter to y 60 pulled in the unreachable perch guard #23
    // (y 51.0) and stalled the walk 40k ticks (the 00m run), so the high posts need the
    // aimed-stand tier (see the report's what's-left), not this sweep. This clear handles the
    // ground band only.
    // 00-Agent tactics (real-health-00agent-06-2): the AIMED stands first (any height within
    // 40 u, the recent hitter first) — they answer #11/#12-class posts and the ch-55
    // reinforcement pair the way a player does: from the plaza, hold aim, drop each one as it
    // pauses. The ground clear below keeps its y bound.
    // The civ-repin-06 extension (d1/d2 REAL DAMAGE only): the balcony route trips the west
    // head's sweep cone EARLY (the ch-55 rise moved ahead of the mouth — [L this session's run:
    // ch 55 already on at the mouth print]), so #735's reinforcement pair stands at the strip
    // posts BEFORE the descent and charged it point-blank (the death ledger: #17 39.6 in 6 hits
    // at d 1.1, #18 28.4 in 5 at d 1.8, both still at full hp 20 — the return fire never settled
    // on a charger). The 00 tier's own answer, measured at d3, runs for the real-damage d1/d2
    // sisters too: the aimed stands engage the posts from the plaza and the strip (a charger
    // never pauses; a posting guard does), and the ground clear sweeps the lane band. The
    // topped-up walk keeps its plain lane (the fights never run there; the stands would only
    // churn its pinned timeline).
    let real_tactics = difficulty >= 3 || (difficulty <= 2 && walker::no_topup_env());
    if real_tactics {
        // 00-Agent tactics (real-health-00agent-06-3, measured on master c2c4a6e): the mouth
        // phase goes SILENT at 00. The deck posts (#12/#29/#30) that this tier answered are
        // already DEAD by the mouth on this master (they chase during the guardroom fights and
        // fall — runs 4-7 censuses), the only mouth-band target left is the harmless court
        // guard #27 (0.0 in 3 hits), and EVERY burst here pulls the ch-55 pair west off their
        // strip posts while they spawn-walk (runs 6/7: the landing then met #19 at d 10-12
        // INSIDE his sight, and the duel killed the walk — 64.2 in 12 hits at 5.5 a hit). The
        // player crosses the mouth quietly and takes the pair at their posts from the landing.
        if difficulty < 3 {
            aimed_stand_the_mouth(w);
            clear_the_spot(w, 13.0, false);
        }
        // The mouth rays DON'T reach the strip posts — the mouth pick census reads rays0 for
        // every hostile at 18-38 u [L this session's run: #17 d 24.2, #18 d 25.6, both rays0],
        // so both stands broke instantly and the strip-approach leg walked Bond into the pair's
        // 12 u sight mid-nav — they charged to d 1.5/1.9 before the strip stands could run and
        // the chargers' 66 damage landed (the death ledger). The courtyard's open WEST does
        // reach (the strip census: #25 rays1 at d 16 [L the same run]). The player walks back
        // to the drop landing and takes the pair at their POSTS — 13.9/14.6 u, outside their
        // 12 u sight [D props], static, so the tight-gate shots settle and two rounds a guard
        // end them before the strip approach can enter the charge band.
        w.walk_to(Wp::new(4.0, 41.5, -23.5).r(1.4), "back to the drop landing's lane");
        if difficulty >= 2 {
            // d2's alarm runs LATE ([L the agent run: ch 55 still off at the mouth print, on by
            // the back-walk]) — the pair is mid-spawn-walk when the landing engagement starts,
            // and the sweep's first burst PULLS them (noise) into a charge that landed 50.0
            // (#19 11.5 in 3 + #20 38.4 in 6, the death ledger). The still-gate stands go
            // FIRST: hold aim, drop each one as it pauses at its post — no burst before the
            // settle, no charge. The sweep below only mops up what never posted.
            // 00-Agent tactics (real-health-00agent-06-3, measured on master c2c4a6e): the
            // posted pair CHARGES once inside its sight (run 6: the sweep-first order met #19
            // at d 10.1 — the duel killed the walk), so at 00 the still-gate stands go first
            // and the sweep only mops up what a stand left. The pair's own sight is 12 u [D
            // props] and their posts sit 14.7 u from the landing — outside it: the stand
            // waits safely for the pause and the tap lands.
            aimed_stand_the_mouth(w);
        }
        clear_the_spot(w, 20.0, false);
        aimed_stand_the_mouth(w);
    }
    // South across the courtyard to the tunnel door's approach (3.75, 42.00, −51.13) [D leg
    // probe: the nav graph covers this courtyard, 7 nodes]. The courtyard floor here is the
    // web's 41.0 band [D Boundary Prime nodes 303-306: (3.4..5.3, 41.0, −48..−50)].
    // 00-Agent tactics (real-health-00agent-06-2, measured): the pair (#735's reinforcements)
    // spawns at the strip posts when the mouth plate's reveal raises the alarm — the ground
    // plaza rays reach them here (the balcony stands above found nothing: the lip blocks the
    // muzzle rays DOWNWARD — the strict ray set's honest miss), and the descent's deck guard
    // (the merged-tree ledger: #31, weapon 0x25, 69.0 in 2 hits at d 8) dies AIMED from range
    // before he is point-blank. The leg is SPLIT at the strip (the pair's posts (11.6, 42.0,
    // −35.1)/(13.2, 41.7, −34.8) [D census]): the stands run from the strip before the last
    // stretch walks Bond into the deck guard's d-8 lane (the merged-tree run 2 died at
    // (4.39, 42.03, −45.56) to #31 mid-nav).
    if real_tactics {
        // 00-Agent tactics (real-health-00agent-06-3, measured, runs 4+9): the strip nav from
        // the landing is the lane — it walked straight down on run 4 (nav 1 (5.96,−23.71) →
        // nav 2 (5.99,−32.72), no give-up, no door-#59 loop). The mouth RETURN that 06-2's
        // lever 2 replaced is a trap on this master: the mouth plaza is the tunnel-roof guard
        // #11's killing field at d 41 (run 9: standing there ate 42.2 in 8 FULL hits) and the
        // pair's advance lane. The stands at the strip arrival answer the rest.
        aimed_stand_the_mouth(w);
        // 00-Agent tactics (real-health-00agent-06-3, runs 10-12 MEASURED NEGATIVE): the pair's
        // posts read rays0 from the landing (d 21) AND from the sight's edge (5.48,−24.92: d
        // 17.0/16.4, rays0) and rays1 only at d ≤ 10-13 — the muzzle-ray opening zone on the
        // courtyard floor lies ENTIRELY inside their 12 u sight band, so no floor stand can
        // take them fairly; the arrive (5.99,−32.72) races three shooters at d 8.7 and loses.
        // The edge stand is kept as the next session's measuring stick: only a spot ELEVATED
        // over their posts (the deck y 48, 06-2's lever 1) out-ranges their sight.
        if difficulty >= 3 {
            w.walk_nav_to([6.0, 42.0, -26.0], "the strip lane's sight edge", 1.2);
            aimed_stand_the_mouth(w);
        }
        w.walk_nav_to([6.0, 42.0, -34.0], "the courtyard's strip approach", 1.6);
        aimed_stand_the_mouth(w);
        fight_through(w);
    }
    w.walk_nav_to([4.4, 42.0, -48.5], "down the searchlight courtyard", 1.6);
    if real_tactics {
        aimed_stand_the_mouth(w);
    }
    fight_through(w);
}

/// The tunnel climb and the upper gallery: door #71 → the two-ramp switchback (see the header's
/// floor map [D]) → door #72 → the gallery deck. The gallery is where the ORIGINAL walk stopped:
/// its north edge is a balustrade (the deck ends z −64.8 with balusters at z −64.92, y
/// 49.25..51.3, x 30..36.7 [D the full-edge ray sweep on this tree]) and its east door #46 is
/// locked (ch_lock 200, nothing on the level writes 200 [D door dump; M 0x136810:10-16; U-044]),
/// so the gallery is scenic — the game's own web routes the dressing room around it (see
/// walk_the_ledge_to_the_party).
fn walk_the_tunnel_and_gallery(w: &mut Walker) {
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [3.75, 42.0, -51.13])
            .expect("door #71 Door_Tunnel_G9 in the doors list [D census]");
        eprintln!("[06] door {di} at {pos:?}");
        w.use_door(di, "door #71");
    }
    // Through the doorway to the inside spot (trigger #674 (4.42, 42.66, −52.94) [D census]).
    w.walk_to(Wp::new(4.42, 42.7, -52.9), "into the tunnel");
    // THE TUNNEL SWITCHBACK up to the upper door #72 Door_Tunnel_G9_2 (4.00, 49.00, −64.88).
    // Legs follow the disc's own surfaces (see the header's floor map [D]); each is the
    // player's own walk+jump input. The shaft's south-east corner is a dead end under the
    // upper ramp — the old straight shot grinded there (jump apex 46.6 < the 48.8 slab face).
    w.climb_stairs([4.3, 44.6, -58.0], "up the low ramp", 1.4, 2400);
    w.climb_stairs([6.7, 45.1, -58.3], "the shaft's east gap", 1.2, 1600);
    // The corner's walkable lane narrows to x 6.75-7.0 at z −57.0 (the upper ramp's east face
    // tapers east as it rises [D floor map]) — tight arrive radii so each leg actually reaches
    // its mark before the next re-aims; the shallow lane→ramp step (46.8 → 47.4) is at z −56.0.
    w.climb_stairs([6.85, 45.9, -57.0], "the east lane north", 0.5, 2000);
    w.climb_stairs([6.45, 46.8, -56.1], "the corner shelf", 0.5, 2000);
    w.climb_stairs([5.6, 47.5, -56.0], "onto the upper ramp", 0.7, 1600);
    // The upper ramp and the seam into the landing are walked WITHOUT the jump pulses: an
    // airborne seam crossing re-links Bond onto the ramp cel's phantom plane under the landing
    // slab and the shaft void swallows him [D the seam probe on this tree: grounded the whole
    // crossing at root ~50.0; the jump-pulsed legs fell past y 47 into the slab's underside].
    w.walk_slope([4.0, 48.9, -59.0], "south up the upper ramp", 0.9, 2000);
    w.walk_slope([4.0, 49.0, -63.0], "the landing, up to door #72", 0.8, 1600);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[1] > 47.0, "the switchback gains the upper level (y {:.2} at {:?})",
            sim.bond.pos[1], sim.bond.pos);
    }
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [4.0, 49.0, -64.88])
            .expect("door #72 Door_Tunnel_G9_2 in the doors list [D census]");
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(dxz(sim.bond.pos, pos) < 4.0, "door #72 is E'd from beside it, not through the wall (bond {:?} door {pos:?})", sim.bond.pos);
        eprintln!("[06] door {di} at {pos:?}");
        w.use_door(di, "door #72");
    }
    // Through the doorway to the inside spot (trigger #521 (4.62, 49.80, −66.65) [D census]) —
    // the gallery walk needs the doorway crossed first or the steering slams the wall beside it.
    w.walk_to(Wp::new(4.6, 49.8, -66.6), "through door #72");
    // East into the upper gallery (y≈50.1 — the WineTruck building's top corridor, z −65..−70).
    w.walk_to(Wp::new(20.0, 50.1, -67.0), "east into the gallery");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[1] > 49.0, "Bond stands on the upper gallery (y {:.2})", p[1]);
    }
}

/// THE DRESSING ROOM via the game's own route [D Boundary Prime node chain 371 → 370/363 → 364
/// → 365 → 366 → 367 and the room ring 373-378/440-448]: the courtyard's 41.0 south-east floor
/// (nodes 371 (24.12, 41.15, −50.0) / 355 (25.23, 41.15, −47.27)) → the 43.0 step at the
/// courtyard's south wall (370 (19.96, 43.27, −49.95), 361 (20.28, 43.13, −47.37), band C z −48:
/// 43.0 at x 18..20) → the ledge's north end (363 (17.49, 43.49, −50.34)) → SOUTH along the x≈17.4
/// ledge rising 43.5 → 45.4 (364 (17.44, 45.36, −53.74) → 365 (17.37, 45.26, −64.64)) → EAST
/// along the 45.25 shelf at z −64.6 under the gallery's balustrade (366 (35.49, 45.25, −64.60)) →
/// north through the 46.0 sill's open gap at x ≈ 36 (367 (35.49, 45.25, −62.28); the sill is
/// 46.0 across z −62 with 45.0 holes at x ≈ 32 and x ≈ 36 [D fine band C]) into the room's
/// 43.1 east strip (377 (35.36, 43.14, −58.39) / 378 (35.40, 43.14, −50.45)) → the LoadPoint #746
/// volume (centre (35.53, 44.50, −54.09) [D trig dump]) → ch 0x83 → ScriptPlayer #810 → the
/// dressing cut-scene 0x06000071 → its event 0xD [2,1] folds the level's only objective (ch 0x02
/// [L ee_start_05 record 1]) and its final event 0x12 [0x07000007] loads the party [M
/// Script_EventHandler 0x1d94c0 case 0x12]. The alarm-silencing TouchOnce #765 (out 177) shares
/// the volume's spot [D trig dump] — the walk trips both, exactly as the walkthrough does.
fn walk_the_ledge_to_the_party_at(w: &mut Walker, difficulty: u32) {
    // civ-unignore-2 (the Agent ledge-leg stands): the east strip legs walk Bond PAST the strip
    // posts (#24 (28.1,42.0,−36.0) hp 20 / #5 (27.8,42.0,−35.7)) whose rounds reach across the
    // whole east courtyard (the measured d2 ledger: #5's hits at d 31.6/24.2/23.8, #24's at
    // d 16.8/12.0) while Bond's own muzzle rays read blocked from the west anchors at 21-28 u
    // [L four mouth censuses] — the ray-open spot the reactive fight proved is the east floor
    // side (~d 9.7 from (18.56,−37.04), and the route's own east-floor anchor (25.3,−40.6) sits
    // EAST of the posts). A player crossing a post's lane stops at each leg end and takes what
    // the aim exposes: aimed_stand_the_mouth (the file's own ray-gated sweep, recent-hitter
    // first, any height) runs after each leg at Agent real health; a guard the rays still
    // block is skipped (the stand's 80-tick unseen give-up), never chased. This runs AFTER the
    // descent's mouth tier (the ch-55 pair is already dead there — the config-4 lesson from
    // docs/reports/civ-repin-finish.md: pocket stands run while the alarm pair lives bait its
    // charge), so the only targets are the posts, the deck/roof posts and the walk-in lane's
    // own guards.
    let ledge_stands = difficulty == 2;
    w.walk_nav_to([10.5, 42.4, -34.3], "the courtyard's east strip", 1.6);
    if difficulty >= 3 {
        // 00-Agent tactics (real-health-00agent-06, measured): the ch-55 alarm's reinforcement
        // pair (#19/#20) converges on the strip AFTER the alarm's mouth fight — at 00 they met
        // Bond point-blank (d 3.7/5.2) mid-leg and the reactive return fire bled 77.5 hp
        // (15 hits, the 00c death ledger: #20 −44.5 in 9, #19 −33.0 in 6, hp 20 left). The
        // player keeps them at range: fight at every node of the descent, so the pair dies
        // while it is still 8 u out instead of inside the gun's TooClose band.
        clear_the_spot(w, 8.0, false);
    }
    if ledge_stands { aimed_stand_the_mouth(w); }
    w.walk_nav_to([14.0, 41.9, -36.7], "the courtyard's south-east corner", 1.6);
    if difficulty >= 3 {
        clear_the_spot(w, 8.0, false);
    }
    if ledge_stands { aimed_stand_the_mouth(w); }
    w.walk_nav_to([25.3, 42.6, -40.6], "the east courtyard floor", 1.6);
    if difficulty >= 3 {
        clear_the_spot(w, 8.0, false);
    }
    if ledge_stands { aimed_stand_the_mouth(w); }
    // South to the web's 41.15 band and the step's foot [D nodes 355/457/371].
    w.walk_nav_to([25.2, 42.3, -47.3], "the 43-step's foot", 1.6);
    if difficulty >= 3 {
        clear_the_spot(w, 8.0, false);
    }
    if ledge_stands { aimed_stand_the_mouth(w); }
    w.walk_nav_to([24.1, 42.3, -50.0], "south along the step's foot", 1.4);
    // Up the 43.0 step (band C z −48: 43.0 at x 18..20 [D]; node 370 (19.96, 43.27, −49.95))
    // and onto the ledge's north end (node 363 (17.49, 43.49, −50.34)).
    w.climb_stairs([20.0, 44.5, -50.0], "up the 43.0 step", 0.9, 1600);
    w.climb_stairs([17.5, 44.7, -50.3], "the ledge's north end", 0.8, 1600);
    // South along the x≈17.4 ledge as it rises 43.5 → 45.3 (nodes 364/365 [D]).
    w.climb_stairs([17.4, 46.6, -56.5], "south up the ledge", 0.8, 2400);
    w.climb_stairs([17.4, 46.5, -63.5], "the ledge's south end", 0.8, 1600);
    // East along the 45.25 shelf at z −64.6, under the gallery deck (node 366 [D]).
    w.walk_to(Wp::new(35.4, 46.5, -63.5), "east along the shelf under the gallery");
    // North through the sill's open gap at x ≈ 36 (node 367 [D]) into the room's east strip
    // (nodes 377/378: 43.1 at x 35.4, z −50.5..−58.4).
    w.walk_to(Wp::new(36.0, 46.3, -61.2), "through the sill gap into the room");
    w.walk_to(Wp::new(35.4, 44.4, -54.2), "north across the dressing room");
    // The walk-in folds ch 0x83 → the dressing scene; poll in case the volume fired mid-leg.
    let mut scene = false;
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x83) { scene = true; break; }
        w.tick(Input::default(), false);
    }
    assert!(scene, "the LoadPoint #746 raises ch 0x83 on the walk-in [D #746]");
    // The dressing cut-scene: the game's own skip runs its events (the Cut-scene law).
    w.wait_enabled(3000, "the dressing cut-scene");
    // The objective fold: ch 0x02 (the scene's closing event 0xD [2,1]) → state 4 [M 0x1827d8:55-67].
    let mut done = obj06(w);
    for _ in 0..240 {
        done = obj06(w);
        if done == 4 { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(done, 4, "the level's only objective completes on the dressing scene's ch 0x02 [L record 1]");
    // The scene's final event 0x12 [0x07000007] IS the level's exit (no 0xE8 placement [D]).
    let mut exited = w.scene.sim.as_ref().unwrap().drones.level_exit;
    for _ in 0..240 {
        exited = w.scene.sim.as_ref().unwrap().drones.level_exit;
        if exited.is_some() { break; }
        w.tick(Input::default(), false);
    }
    assert_eq!(exited, Some(0x0700_0007), "the dressing scene's event 0x12 loads the party [M 0x1d94c0 case 0x12]");
}

/// The FULL mission through the real input path: the truck-ride spawn → the dismount → the well
/// courtyard's hedge lane → E-doors #47/#59 (the guardroom) → the searchlight courtyard → the
/// ledge route (the game's own web [D Boundary Prime]: the 43.0 step, the x≈17.4 ledge, the
/// 45.25 shelf, the sill gap) → the dressing room → the LoadPoint #746 walk-in → the dressing
/// cut-scene folds the level's ONLY objective (ch 0x02 → state 4) and its event 0x12 fires the
/// level exit to the party. Every door on the route is E-opened by the player's own use key, and
/// the objective is revealed by the route itself (ch 0x46) — the whole chain 1 (armed) → 3
/// (announced) → 4 (complete) asserted on the walk [M 0x182270:98-105 / 0x1827d8:55-67].
/// History: the OLD blocker (the fire fault) does not reproduce on master (fix_06_fire_live).
/// The tunnel wedge was fixed by the switchback legs (see walk_the_tunnel_and_gallery). The
/// gallery → dressing-room gap was the second wedge — the gallery's own north edge is a
/// balustrade and door #46 is locked [D], so the game routes the room around it: see the header's
/// OPEN LEG note and walk_the_ledge_to_the_party.
#[test]
fn walk_the_castle_interior_to_the_party() { walk_the_castle_interior_to_the_party_at(1) }

/// The walk body at `difficulty` (1 Operative = the old `load_with_carry` path's replay parity —
/// its save resolves to 1; 2 Agent rides the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 →
/// load_with_carry_difficulty]; the difficulty global survives the level change untouched, so a
/// warm load re-seeds the same value [M player.rs RamSave::difficulty doc]).
fn walk_the_castle_interior_to_the_party_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70])), difficulty).expect("load 07000006"));
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 57.18).abs() < 2.0 && (p[2] - 22.61).abs() < 2.0,
            "the carried ch-0x1D gate selects the truck-ride spawn #496 [M Player_Start 0x190fd0], got {p:?}");
    }
    // courtyard-lamps: watch the reveal channel (SP #808's p6, ch 0x34) and the alarm (ch 55, the
    // heads' p3 [D]) for the whole walk.
    w.rise_watch = vec![0x34, 55];
    // The level-start NIS: waited out through the game's own skip action.
    w.wait_enabled(3000, "the intro NIS");
    w.wait(30);
    // THE LAMPS ARE DARK AT THE START: SP #808 waits in state 0 for ch 0x34 [M SP_Update 0x1dbda8
    // case 0; SP_Hit 0x1dc470:16-18], so the reveal NIS 0x06000066 has not raised any head's
    // on-channel and every head sits in state 3 (off) [M Searchlight_Create: p1 != 0 → state 3].
    {
        let h = heads06(&w);
        assert_eq!(h.len(), 4, "07000006 has four courtyard heads [D census]");
        assert!(h.iter().all(|&(st, on)| st == 3 && !on), "the courtyard lamps are dark at the start: {h:?}");
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x34), "no plate crossed on the truck");
        assert!(sim.drones.searchlights.parts.iter().filter(|p| p.kind == 4).all(|p| !p.drawn), "no corona at the start");
    }

    // 00-Agent tactics (real-health-00agent-06, measured): the 00 walk takes the stealth
    // route's fusebox Laser cut. Without it the courtyard's ch-55 plate raises the alarm and
    // the ch-55 spawner #735 drops the reinforcement pair (#19/#20) at their strip posts
    // (11.6, 42.0, −35.1) / (13.2, 41.7, −34.8) — measured at 00: the pair bled 77.5 hp
    // point-blank on the descent (the 00c/00d ledgers), because the guards' ×1.3 accuracy and
    // ×1.0 pain [M ReadTuningVars 0x1d3a80 case 0x7000006] make the un-silenced descent
    // unwalkable. The cut writes ch 57 — the heads' off-channel — and Searchlight_Update
    // force-states them to 3 before the tracking code runs, so no head ever writes ch 55 and
    // #735 never spawns [M Searchlight_Update 0x1cb900:88-92; D #747 p1]. The guardroom is
    // pre-cleared by the 00 fights below, so the cut's exposed window is safe.
    // 00-Agent tactics (real-health-00agent-06-2, re-measured on the merged tree): the cut is
    // GONE at 00 too. The prior session's reason (the cut silences #735's reinforcement pair)
    // no longer holds: the merged master's door-route guards re-trip the courtyard's ch-55
    // volume during the guardroom fights (the merged-tree ledger: #20 skin 0x500002f hit at
    // frames 10563-10639, DURING the cut stand, with ch 57 on) — the pair spawns anyway — and
    // the cut's exposed window itself cost 51.0 hp (the #28 fight at the nook: 89 → 38, the
    // merged-tree run 1). The alarm is NOT a fail channel [L record 1; M 0x1827d8:98-107] and
    // the lit heads' cones missed every walked crossing (the sweep pin below), so the route
    // takes the door #59 lane without the fusebox stand and answers the pair at the balcony
    // with the aimed stands (the mouth tier below).
    walk_to_the_courtyard_at(&mut w, false, difficulty);
    eprintln!("[06] reached the courtyard at tick {}", w.total_ticks);
    for r in &w.rises { eprintln!("[06] rise: ch {} at tick {} bond {:?} touching {:?}", r.0, r.1, r.2, r.3); }
    // THE WALKED BOND TURNS THEM ON AT THE ENTRANCE: the first ch-0x34 rise is the walk's own
    // plate crossing at the courtyard mouth, and the reveal has lit all four heads by now.
    {
        let first = w.rises.iter().find(|r| r.0 == 0x34).expect("the walk crosses a ch-0x34 plate");
        assert!(!first.3.is_empty() && first.3.iter().all(|p| PLATES_34.contains(p)),
            "ch 0x34 rises under Bond's own feet on a courtyard plate, got {:?} at {:?}", first.3, first.2);
        // The balcony drop: Bond falls through the Spotlight Trigger poly #664 [D] (measured on
        // this walk: Bond ≈ (3.4, 46.5, −21.5) mid-drop).
        assert_eq!(first.3, vec![664], "the guardroom route's first plate is the balcony-drop poly #664");
        let h = heads06(&w);
        // The heads LIT at every difficulty now (the cut route retired at 00 — the measured
        // reason above): the reveal NIS lights all four at the entrance plate, and the lit
        // heads' cones missed every walked crossing (the pin below).
        assert!(h.iter().all(|&(st, on)| st != 3 && on), "all four heads lit after the entrance plate: {h:?}");
        // The lit heads still spot Bond at Operative: the alarm channel 55 (written only while a
        // head tracks [M Searchlight_Update 0x1cb900:142-144]) rises in the courtyard, after the
        // plate. Operative-only pin [D coll-scan of the head cones, this branch]: the catch is a
        // narrow window, not a law — head6 (5.50, 55.67, −50.15, yaw cone ±0.218, pitch cone
        // 1.004±0.218) reaches Bond's sphere only where its pitch edge meets body height,
        // z ≈ −38..−35 × x ≈ 2.4..8.6 (the Operative alarm measured exactly there, (4.55,
        // 42.18, −36.30), t 15528); the other three heads' cones (east/west pies at z −19.8..−14.9
        // and z −21.7..−25.5) missed BOTH walks' crossings on the sweep phase. The Agent descent
        // runs a fight-displaced west lane — x ≈ 0.75 at z −36, 0.6 u outside head6's yaw band —
        // and the game's own sweep misses it too (the same deterministic cones, verified against
        // the Operative catch). The alarm is NOT a fail channel on this level [L record 1; M
        // 0x1827d8:98-107] and dodging the beams is the player's own best line at Agent real
        // health (no ch-55 reinforcement spawn #735) — the sister pins the lit heads and the
        // route, not the spot.
        if difficulty == 1 {
            let alarm = w.rises.iter().find(|r| r.0 == 55).expect("a lit head spots the walked Bond (ch 55)");
            assert!(alarm.1 > first.1, "the alarm follows the reveal (plate t{}, alarm t{})", first.1, alarm.1);
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The route's own channel, written by the walk itself (the fusebox cut is pinned by
        // fusebox_06_live; the walk crosses the courtyard fighting the alarm's reinforcements).
        assert!(sim.drones.channels.on(0x46), "the objective was revealed by the route [D #498]");
        // The objective's state chain so far: armed-then-announced (1 or 3), never complete —
        // the mission monitor folds ch 0x02 only [L record 1].
        let s = sim.drones.mission.states[mission_row()][0];
        assert!(s == 1 || s == 3, "the objective is announced but not complete at the courtyard (state {s}) [M 0x182270:98-105 / 0x1827d8:55-67]");
        assert_eq!(sim.drones.level_exit, None, "no exit before the LoadPoint chain");
    }
    walk_the_ledge_to_the_party_at(&mut w, difficulty);
    eprintln!("[06] the whole castle walked in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[06] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[06] stuck: {s}"); }
}

/// The other entrance: from the guardhouse court straight west up the ground ramp (floor 38.0 →
/// 41.0 along z −12..−13 [D floor probe]) into the courtyard mouth, real input all the way. The
/// first plate under Bond's feet is #526 (x 9.17..10.67, z −13.86..−11.35 — the east one of the
/// mouth pair [D trigger dump]); the reveal NIS then lights all four heads within its 167 waits
/// [D 0x06000066 stream 0]. Nothing is lit before the plate.
#[test]
fn the_lamps_light_at_the_ground_mouth() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    w.rise_watch = vec![0x34];
    w.wait_enabled(3000, "the intro NIS");
    w.wait(30);
    walk_to_the_guardhouse_court(&mut w, 1);
    {
        let h = heads06(&w);
        assert!(h.iter().all(|&(st, on)| st == 3 && !on), "still dark in the guardhouse court: {h:?}");
        assert!(w.rises.is_empty(), "no ch-0x34 plate on the way to the guardhouse court");
    }
    w.walk_to(Wp::new(22.0, 39.1, -12.3), "the guardhouse court's west end");
    w.walk_to(Wp::new(12.0, 41.5, -12.3), "up the ground ramp");
    w.walk_to(Wp::new(6.0, 42.0, -12.3), "into the courtyard mouth");
    let first = w.rises.first().cloned().expect("the mouth crosses a ch-0x34 plate");
    eprintln!("[06] mouth rise at tick {} bond {:?} touching {:?}", first.1, first.2, first.3);
    assert_eq!(first.3, vec![526], "the ground mouth's first plate is #526");
    for _ in 0..240 { w.tick(Input::default(), false); }
    let h = heads06(&w);
    assert!(h.iter().all(|&(st, on)| st != 3 && on), "all four heads lit after the mouth plate: {h:?}");
}

/// The scenic leg the ORIGINAL walk covered: the tunnel switchback (the two-ramp climb the walk
/// used to die in [D floor map]) and the upper gallery. The gallery is NOT the dressing-room
/// route — its north edge is balustraded and door #46 is locked [D walk_the_tunnel_and_gallery's
/// note] — but it is the zip-wire arrival floor (start #511) and the walk's own proven ground.
#[test]
fn walk_the_castle_interior_to_the_upper_gallery() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    w.wait_enabled(3000, "the intro NIS");
    w.wait(30);
    walk_to_the_courtyard(&mut w, false);
    walk_the_tunnel_and_gallery(&mut w);
    eprintln!("[06] reached the gallery at tick {}", w.total_ticks);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x46), "the objective was revealed by the route [D #498]");
        let s = sim.drones.mission.states[mission_row()][0];
        assert!(s == 1 || s == 3, "announced but not complete at the gallery (state {s}) [M 0x182270:98-105 / 0x1827d8:55-67]");
        assert_eq!(sim.drones.level_exit, None, "no exit from the gallery itself");
    }
    eprintln!("[06] walked to the gallery in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[06] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[06] stuck: {s}"); }
}

/// Sister: the same walk that stops at the courtyard (before the tunnel AND before the ledge
/// route) never ends the level — no alarm-channel dependency, no ch 0x83, no objective fold, no
/// exit. The walk's asserts depend on the walked route, not on any earlier scripted hand-out.
#[test]
fn stopping_before_the_tunnel_never_ends_the_castle() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    w.wait_enabled(3000, "the intro NIS");
    w.wait(30);
    // The route WITHOUT the fusebox cut, halted at the tunnel-door approach (before the tunnel
    // and before the ledge route's 43.0 step).
    walk_to_the_courtyard(&mut w, false);
    eprintln!("[06] sister walked to the courtyard in {} ticks (no cut)", w.total_ticks);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x83), "the LoadPoint stays cold far from its volume [D #746]");
    assert_ne!(sim.drones.mission.states[mission_row()][0], 4, "the objective is not complete");
    assert_eq!(sim.drones.level_exit, None, "the castle does not end away from the LoadPoint");
    let _ = Act::None;
}

/// Real damage: the same walk with the health top-up off for every walker it makes (the
/// tests' own `top_up_health = true` included) and the walker's return fire on — Bond's death
/// panics with the death spot and the last hits (common/damage_log.rs), so a pass pins that the
/// walk completes on the port's own damage.
#[test]
fn walk_the_castle_interior_to_the_party_real_damage() {
    walker::with_real_damage(walk_the_castle_interior_to_the_party);
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty): the same walked
/// truck-spawn → castle-interior → party route at the menu's Agent pick — GameState+0x28 = 2
/// through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table
/// 0x2df4c0 row 1 carries 2; Txt 0x14e = "Agent"]. On this level group the pick scales Bond's
/// pain by Plr_DMod_Normal 0.7 instead of Easy 0.6 and the guards' hit chance by
/// DroneFiring_Accuracy_Normal 0.9 instead of Easy 0.8 [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; ReadTuningVars 0x1d3a80 case 0x7000006:
/// Normal 0x3f666666 = 0.9 — this comment said 1.1, which contradicted the decompile; fixed by
/// real-health-00agent-06]; the min-difficulty
/// placements pass Drone_Create's gate [M 0x136be0:49] and the Lock_Create auto-unlock keeps
/// tagged locks shut [M 0x1ae1f8:67-69 — Operative-only]. Real input only — the load path is the
/// only difference.
/// GREEN (civ-unignore-2): three Agent-tier levers, all file-local and difficulty-gated (the d1
/// walks byte-identical). (1) the west-lane shooter sweeps (`stand_shooters_down`) — the dock
/// guard posts d 27-35 from every early leg, OUTSIDE the fights' plain 20 u shooters window, and
/// the measured baseline's biggest dealer (25.0 in 13 hits) plinked free the whole walk; a
/// BLIND radius clear here is measured wrong (it shot the friendly driver #17, hp 1, and his
/// death latched the level exit). (2) the guardroom pre-clears at Agent (the d3 tier's own
/// `clear_the_spot(13)` arms) — the room's #3 spawn landed point-blank during the crossing.
/// (3) the ledge-leg aimed stands — the east strip legs walk past the strip posts whose rounds
/// reach d 30+ while Bond's muzzle rays read blocked from the west anchors; the ray-gated sweep
/// takes what opens (the posts #24/#25 died in the mouth tier this run) and skips what stays
/// blocked. Measured: the whole castle walked in 13031 ticks, end health 3.7, 51 hits for 96.3
/// (the residual is #5's walk-in lane at hp 3 — his rays never opened before the 43-step) [L
/// this tree's run].
#[test]
fn walk_the_castle_interior_to_the_party_real_damage_agent() {
    walker::with_real_damage(|| walk_the_castle_interior_to_the_party_at(2));
}

/// 00-Agent sister of `..._real_damage` (real-health-00agent-06): the same walked truck-spawn →
/// castle-interior → party route at the menu's 00 Agent pick — GameState+0x28 = 3 through the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 2
/// carries 3; Txt 0x14f = "00 Agent"; Drones::difficulty assert at the load below]. The
/// difficulty-3 rows the game reads on this group [M ReadTuningVars 0x1d3a80 case 0x7000006;
/// Player_HandlePain 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88 — 3 → Hard]:
/// Plr_DMod_Hard 0x3f800000 = 1.0 (Bond takes FULL damage — no Easy/Normal relief), the guards'
/// hit chance × DroneFiring_Accuracy_Hard 0x3fa66666 = 1.3 and their bullets × DroneDamage_Hard
/// 0x3f4ccccd = 0.8; Drone_Create's min-difficulty gate spawns every placement tagged ≤ 3
/// [M 0x136be0:49], and the Lock_Create auto-unlock stays off beyond Operative
/// [M 0x1ae1f8:67-69]. Real input only — the
/// load path is the only difference; no health, ammo or channel writes, no teleports.
#[test]
#[ignore = "00 Agent: the mouth->strip lane on master c2c4a6e — the ch-55 pair (#19/#20) charges the landing engagement from inside its 12 u sight (runs 6/7: the duel at d 10.1, 64.2 in 12 hits) and the tunnel-roof guard #11 (y 55.5) holds the whole mouth-strip band in a full-damage lane (runs 8-10: 38-42 per phase at d 23-41) while the still-gate stand cannot land on a mover or his parapet flicker; the session's test-side levers (the door-#47 re-open + the d1-aligned guardroom nav, the guardroom ammo-box loot, the d3 balcony drop, the nearest-first stand triage — #20 died to it) got the walk from the guardroom wedge to the strip at 64.2 hp; follow-up real-health-00agent-06-4"]
fn walk_the_castle_interior_to_the_party_real_damage_00agent() {
    walker::with_real_damage(|| walk_the_castle_interior_to_the_party_at(3));
}
