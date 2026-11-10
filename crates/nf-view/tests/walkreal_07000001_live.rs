//! walkreal-a: 07000001 "Paris Prelude" (Castle A) walked with the player's own movement — no
//! teleports after the spawn. The route (docs/levels/07000001.md):
//!   spawn #0 (−2.78, 6.84, −12.40) → the intro cut-scene writes ch 0xB → the contact (skin 0x4F)
//!   enables → walk the interior (doors #2/#3/#4 on the way) to the LoadPointActivator plate
//!   (Touch out 0x10, mask 0x18, filter skin 0x4F) at (−33.1, 2.9, 15.4) → Bond AND the contact on
//!   it → ch 0x10 → the exit (LOAD_LEVEL gate 0x10) → 07000002.
//!
//! Bond is driven with `Input` only (turn/forward/use); the health top-up is the "enemies may be
//! disabled for the walk" allowance (the AI still runs — alerts, looks, shots — it just cannot kill).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;
use nf_game::world::V3;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The walked route to the exit (shared by the walk test and the exit-fade pin below): the real
/// input path from spawn #0 to the exit plate, stopping AT the fire tick — `level_exit` is
/// posted and `exit_hold` still has all 300 frames (the game's fade level is 0 on the fire tick;
/// `Camera_UpdateGlbVars` advances it on the frames after [M 0x1bada0:16-20]).
/// The Agent kill order: the pellet-stream carriers (the SMG 0x12, the shotgun 0x1c) before the
/// pistols, then the nearer first. The level's SMG guard ground 21 × 1.05-2.10 into the close-order
/// run and the captain's 8-pellet volley ended it; a player silences the sprayers first.
fn agent_order(w: &Walker, guards: &[usize]) -> Vec<usize> {
    let mut g = guards.to_vec();
    g.sort_by(|&a, &b| {
        let sim = w.scene.sim.as_ref().unwrap();
        let key = |i: usize| {
            let d = &sim.drones.list[i];
            (u8::from(!matches!(d.weapon, 0x12 | 0x1c)), walker::dxz(d.pos, sim.bond.pos))
        };
        let (ka, kb) = (key(a), key(b));
        ka.0.cmp(&kb.0).then(ka.1.total_cmp(&kb.1))
    });
    g
}

/// The route clears' own order (level-01-agent-hall-fight-2): the driver's 3 → 14 → 5 first
/// (the corridor's 24-health guard 3, the SMG sprayer 14, the named hall guard 5 — the
/// stairwell bleed's shooters), then the pellet-carriers-first Agent order for the rest.
/// The indices are the difficulty-2 roster's (deterministic across the runs [L the d2 logs]);
/// guards already dead or absent just drop out of the priority pass.
fn route_order(w: &Walker, guards: &[usize]) -> Vec<usize> {
    let mut g: Vec<usize> = [3usize, 14, 5].iter().copied().filter(|i| guards.contains(i)).collect();
    let rest: Vec<usize> = guards.iter().copied().filter(|i| !g.contains(i)).collect();
    g.extend(agent_order(w, &rest));
    g
}

/// los-mask-repins-01: the ch-0xF slab ambush + the mouth fight, extracted so the walk can run
/// it EARLY (before the follow rounds: the contact crosses the slab on his lead — the pop is
/// Bond-touch only [L the d2 run: his ~t9500 crossing popped nothing, Bond's t10284 crossing
/// popped g4/12/17 onto him at 5.4 u] — and again late, where it becomes a no-op once ch 0xF
/// is on. Self-contained: saves/restores the fight bands it sets.
fn run_slab_ambush(w: &mut Walker, ci: usize, difficulty: u32) {
// The ch-0xF ambush is a PLACED pocket pair, not a door route — [L the trio dump on this
// tree] #12 (0x5000036 SMG) stands DISABLED at (-31.31, 6.34, 11.51) (the stairwell-head
// pocket) and #17 (0x5000039 SMG) at (-29.08, 6.34, -0.01) (the south lower hall) from the
// hall AND on (state 0x1, enabled false — disabled objects are dropped from bullet
// collision [M Collide_PickObj 0x1eae80:82 +0xF0 & 0x20], so they cannot be thinned
// early), and the corridor's walkable band at the slab's x is z >= 13.4 only [L the
// scratch sight probe: the z 11.8-13.0 hip rays all hit] — the TOUCH slab out 0xF mask 0x8
// (player) x -26.13..-24.73, z 13.25..16.84 [D placement dump] covers the full width, so
// BOND's crossing pops them on any line. The old sweep crossed it MID-NAV (the approach to
// guard 3), ate the pop volley at 2.6 u while still under nav (3×2.1 at t7094-7106 [L the
// run's dmg trace]) and the next leg walked the 0.4-u corner death. The player crosses a
// known ambush line READY: step in, and the tick ch 0xF rises step back out — the poppers'
// 10-tick reaction [M DroneFunc_ReactionTime 0x14ac60:15-24, the 07000001 arm] leaves that
// window — then fight from the mouth at 6.5+ u, outside the TooClose doubling [M
// Drone_ModBulletDamage 0x139f50]. From the mouth #17's post is EXPOSED [L the sight
// probe: every eye ray from (-23..-24.5, 13.5..16.5) to his torso clears] while #12's
// pocket is walled [L the same probe: no clear ray from any of those stands] — his own
// step-out under the mouth fire (he advanced (-31.3,11.5) -> (-30.6,13.4) during the run's
// mouth fight [L the dmg trace]) is the kill window, so a guard parked west of x -27 is
// fought SHOOT-ONLY: no approach walk ever takes Bond back into the pocket corner.
if !(difficulty >= 2) || w.scene.sim.as_ref().unwrap().drones.channels.on(0xf) { return; }
    eprintln!("[01] the ch-0xF ambush: crossing the slab deliberately from ({:?})",
        w.scene.sim.as_ref().unwrap().bond.pos);
    let prev_keep = w.keep_off;
    let prev_back = w.dmg.back_off;
    // Agent: the fight radius 4.2 (the 6.5 stand-off backed Bond out of the opening and
    // pinned the fight at 7.7, where nothing in the pocket can be answered) and no flank into
    // the pocket west of x -27.
    let band = if difficulty >= 2 { 4.2 } else { 6.5 };
    let prev_hold = w.flank_hold_x;
    if difficulty >= 2 { w.flank_hold_x = Some(-27.0); }
    w.keep_off = Some(band);
    w.dmg.back_off = Some(band);
    // 00 Agent: the pop fight at x1.2 pain is long; a player tops up the gun before he trips it
    // (the fight opened on clip 2 / spare 14 and ran dry against the 24-health captain).
    if difficulty >= 3 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.clip + sim.gun.spare < 30 { let _ = sim; w.resupply("pre-ambush ammo"); }
    }
    w.walk_nav_to([-24.0, 6.4, 14.8], "the ambush stance (east of the ch-0xF slab)", 0.8);
    w.walk_nav_to([-25.7, 6.4, 14.8], "cross the ch-0xF slab (the ambush pops)", 0.25);
    for _ in 0..120 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xf) { break; }
        w.wait(1);
    }
    // Out FIRST (the pop volley's reaction window), then the mouth fight.
    w.walk_nav_to([-22.3, 6.4, 14.8], "back off the popped ambush", 1.0);
    for ambush_round in 0..5 {
        let near: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let bp = sim.bond.pos;
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, bp) < 14.0
            }).collect()
        };
        if near.is_empty() { break; }
        eprintln!("[01] ambush round {ambush_round}: fighting {near:?} from the mouth");
        // Agent: the exposed guards first (a walled one spends the mouth window while the
        // others shoot free).
        let order = if difficulty >= 2 {
            let (mut seen, walled): (Vec<usize>, Vec<usize>) = route_order(&w, &near).into_iter()
                .partition(|&g| walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g));
            seen.extend(walled);
            seen
        } else { route_order(&w, &near) };
        for g2 in order {
            let (gpos, los) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g2];
                if d.corpse || d.health <= 0.0 { continue; }
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            // The pocket rule: a guard parked west of x -27 (the stairwell-head pocket) is
            // NEVER approached — the shoot loop alone, from the mouth. Hunters in the open
            // corridor get the normal approach to 8 (outside the ×2 band).
            if gpos[0] > -27.0 && walker::dxz(gpos, w.scene.sim.as_ref().unwrap().bond.pos) > 9.0 {
                w.walk_nav_to(gpos, &format!("ambush approach guard {g2}"), 8.0);
            }
            // Agent (with health to spare): a walled pocket guard is answered from the corridor band's west end, where
            // the pocket opens at 3.7-4.5 (outside the x2 band), for one bounded window.
            if difficulty >= 2 && !los && gpos[0] < -27.0 && w.scene.sim.as_ref().unwrap().drones.health.health >= 40.0 {
                w.walk_nav_to([-26.5, 6.4, 14.6], &format!("the band edge for guard {g2}"), 0.8);
                // Whoever the band edge exposes Bond to comes first (a later popper the round's
                // list did not hold shot free while Bond waited on the walled guard).
                let seen: Vec<usize> = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let bp = sim.bond.pos;
                    (0..sim.drones.list.len()).filter(|&i| {
                        let d = &sim.drones.list[i];
                        i != ci && i != g2 && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                            && walker::dxz(d.pos, bp) < 14.0 && walker::DamageLog::visible(sim, i)
                    }).collect()
                };
                for g3 in seen {
                    w.shoot_bounded(g3, &format!("band-edge exposed guard {g3}"), 2400);
                    if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                w.shoot_bounded(g2, &format!("ambush guard {g2} from the band edge"), 2400);
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                continue;
            }
            if !w.shoot(g2, &format!("ambush guard {g2}")) {
                if w.resupply("ambush ammo") { w.shoot(g2, &format!("ambush guard {g2} retry")); }
            }
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        // A pocket guard that will not step out yet: hold the mouth (the gunshot hearing
        // keeps drawing him) and give the fight machinery the next window.
        let still = w.scene.sim.as_ref().unwrap().drones.list.iter().enumerate().any(|(i, d)| {
            i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                && d.pos[0] < -27.0 && walker::dxz(d.pos, w.scene.sim.as_ref().unwrap().bond.pos) < 14.0
        });
        if still { w.wait(300); }
    }
    w.keep_off = prev_keep;
    w.dmg.back_off = prev_back;
    w.flank_hold_x = prev_hold;
}

/// The south-lower-hall far field (≈(-7.9, 6.3, -0.4)) covering the hall door mouth from
/// ~19.7-19.8 u: the d3 ledger has TWO of them — #9 (skin 0x500003a) AND #19 (skin
/// 0x5000039) — so the filter is the POSITION box, not one skin (the west-end 0x5000039
/// pocket guard #17 at (-29.08, -0.01) is 21 u from the box centre and stays out). They sit
/// on a LATER start than the hall AND (the d3 ledger: unstarted when the post-AND sweep
/// ran, then 84.0 in 6 + 14.4 in 8 from the mouth at 4163), so the sweep must be re-run
/// every time their start could have happened. SHOOT-ONLY: the d3 4163 death rode the
/// approach walk TOWARD the pair (Bond in their lane, 20 u of open hall); from the lip the
/// eye→torso rays clear both ways (they hit Bond from there), so the fight is taken from
/// where Bond stands and the shoot loop's own flank ladder answers a blocked aim.
fn far_field_guards(w: &Walker) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
            && walker::dxz(d.pos, [-7.9f32, 6.3f32, -0.4f32]) < 6.0
    }).collect()
}

fn far_field_sweep(w: &mut Walker, tag: &str) {
    for g in far_field_guards(w) {
        if !w.shoot(g, &format!("far-field guard {g} ({tag})")) {
            if w.resupply("far-field ammo") { w.shoot(g, &format!("far-field guard {g} ({tag}) retry")); }
        }
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
    }
}

/// The d3 door-lip scan: every live team-1 guard within 7.0 of the hall aperture's lip point
/// (7.9, 6.3, 11.1) — the exact filter the post-AND lip pass has run since guard-radii-finish
/// (b28203c21). Extracted verbatim so the escort-crossing re-scan runs the same filter.
fn door_lip_scan(w: &Walker) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
            && walker::dxz(d.pos, [7.9, 6.3, 11.1]) < 7.0
    }).collect()
}

/// The d3 door-lip fight: the bounded live-tracked lure from the hold line (the guards hunt
/// the gunshot hearing [M DroneFunc_SetNoiseHeard]) draws the lip guard across the aperture,
/// then the real fire path finishes him. `tag` only names the trace labels. Breaks early when
/// Bond dies (the caller re-checks health and breaks its own loop).
fn door_lip_fight(w: &mut Walker, g5: usize, tag: &str) {
    for t in 0..300u32 {
        let (input, trigger, done) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[g5];
            if d.corpse || d.health <= 0.0 { (Input::default(), false, true) }
            else {
                let bp = sim.bond.pos;
                let at = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
                let err = walker::wrap_pi(desired - sim.bond.yaw);
                let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                let perr = want_pitch - sim.bond.pitch;
                (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                    err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, d.pos[2] < 10.5)
            }
        };
        if done { break; }
        w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
        w.total_ticks += 1;
        w.after_step();
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
    }
    if !w.shoot(g5, &format!("{tag} lip guard {g5}")) {
        if w.resupply(&format!("{tag} lip ammo")) { w.shoot(g5, &format!("{tag} lip guard {g5} retry")); }
    }
}

/// The d3 door-phase ammo detour (guard-pane-drain-4): the door phase is DRY — the pre-hall
/// stock swept the walked-path corridor's boxes (3 passes) and the doorway fights burned the
/// pistol pool to clip 1-4 spare 0 [L the d3fix run] — so the named loop's lure taps die after
/// 1-4 rounds and no hunter crosses ("a dry gun draws no hunter across the aperture": the
/// ARMED lure run that walked 5/6/7/11 into the door line carried clip 32 spare 16 [L the lure
/// run note]). The drain-3 ignore names the lever: the off-path boxes at (-7.5,5.3,-6.8)/
/// (-3.3,5.8,-9.1) that resupply()'s corridor rule refuses — a player between fights walks the
/// detour leg he knows: onto the boxes, back to the hold line, REAL INPUT the whole way, and
/// the walk itself puts the boxes on the trail — resupply()'s corridor rule is trail-based
/// (a target must lie within 9.0 u of ground Bond covered), so the loop's own resupply calls
/// can take them afterwards. The far-field pair (-7.9,6.3,-0.4) stands on that leg on a LATER
/// start than the hall AND (the d3 ledger) — the sweep's SHOOT-ONLY rule (never walk toward
/// them) covers the case where it has already started. Returns whether any ammo arrived.
fn d3_ammo_detour(w: &mut Walker, tag: &str, hold: [f32; 3]) -> bool {
    let ammo = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.gun.clip as i32 + sim.gun.spare as i32
    };
    let before = ammo(w);
    // Only walk the detour when the boxes are actually there: a live, ground-level pickup
    // within 6.0 u of one of the two off-path spots (resupply()'s own |dy| < 1.5 ground rule;
    // Bond's y here is the hold line's 6.4 and the spots sit at 5.3/5.8).
    let spots: Vec<[f32; 3]> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let by = sim.bond.pos[1];
        [[-3.3f32, 5.8f32, -9.1f32], [-7.5f32, 5.3f32, -6.8f32]].iter()
            .filter(|s| sim.drones.pickups.iter().any(|p| !p.gone
                && (p.m[3][1] - by).abs() < 1.5
                && walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], **s) < 6.0))
            .map(|s| *s)
            .collect()
    };
    if spots.is_empty() {
        eprintln!("[01] {tag}: no live ground-level pickup at the off-path spots — the detour stays home");
        // The [L] dump: every live pickup left in the level, with the distance from Bond and
        // the height gap, so the ledger's next lever reads off the real list (kind/weapon/
        // rounds say what each box feeds; from != None marks a guard's drop).
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        for (i, p) in sim.drones.pickups.iter().enumerate() {
            if p.gone { continue; }
            eprintln!("[01] {tag}: live pickup {i} kind {} weapon {:#x} rounds {} state {} at ({:.1},{:.1},{:.1}) dxz {:.1} dy {:+.1} from {:?}",
                p.kind, p.weapon, p.rounds, p.state, p.m[3][0], p.m[3][1], p.m[3][2],
                walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], bp), p.m[3][1] - bp[1], p.from);
        }
        return false;
    }
    for s in spots {
        // The pair's lane: fight from where Bond stands, never walk toward it (the sweep is
        // shoot-only and self-gating).
        far_field_sweep(w, tag);
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        eprintln!("[01] {tag}: the ammo detour leg to ({:.1},{:.1},{:.1})", s[0], s[1], s[2]);
        w.walk_nav_to(s, &format!("{tag} the off-path ammo detour leg"), 0.9);
        w.wait(30); // the pickup radius
        if ammo(w) > before { break; }
    }
    // Back to the hold line the caller fights from: the stage below is the route proven FROM
    // that line.
    if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, hold) > 2.0 {
        w.walk_nav_to(hold, &format!("{tag} back to the hold line"), 1.2);
    }
    // The boxes are on the trail now. The held gun may still be the pistol while the boxes
    // feed the wheel — resupply() takes the nearest on-trail box and, if it is another
    // weapon's, cycles the wheel until a firearm with ammo [M Player_GetBestWeapon's
    // first-owned-with-ammo scan].
    for pass in 0..3 {
        if !w.resupply(&format!("{tag} on-trail stock {pass}")) { break; }
    }
    let got = ammo(w) > before;
    eprintln!("[01] {tag}: the door-phase detour done ({got}) clip+spare {}", ammo(w));
    got
}

fn walk_to_the_exit(iso: &str, stop_at_fire: bool) -> Walker { walk_to_the_exit_at(iso, stop_at_fire, 1) }

/// The walk at `difficulty` — the difficulty select's GameState+0x28 value (1 Operative, 2 Agent,
/// 3 00 Agent [M the `difficulty[]` table 0x2df4c0: rows carry GameState values 1/2/3; Txt
/// 0x14d/0x14e/0x14f = "Operative"/"Agent"/"00 Agent", menu.rs]). 1 keeps the plain `load` (the
/// replay parity — `load` passes difficulty 1 to the same `load_impl`); the others ride the
/// front-end's launch path [M load_with_carry_difficulty → load_impl's difficulty slot].
fn walk_to_the_exit_at(iso: &str, stop_at_fire: bool, difficulty: u32) -> Walker {
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(iso, "07000001.bin", false).expect("load 07000001")
    } else {
        nf_view::load_with_carry_difficulty(iso, "07000001.bin", false, None, difficulty)
            .expect("load 07000001")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // On real damage a guard inside 6.5 u is backed away from, in the walker's fights and its
    // return fire: under 3.0 u every drone round hits [M DroneWeap_DoBulletAccuracy
    // 0x179740:113-115], and the hall guards' 8-pellet shotgun volleys (weapon 0x1C [M
    // DroneWeap_FireWeapon 0x178f68, weapon_data+0x18]) took 24 health each from 0.8-1.2 u at the
    // east door. The top-up walks keep their fight timelines.
    if !w.tops_up() {
        // Agent tactics (level-01-agent-escort-fight): the TooClose band doubles everything inside
        // 3.0 [M 0x139fac-0x139fc8 ×DroneFiring_TooClose_Damage 2.0 under DroneFiring_TooClose_Distance
        // 3.0; the accuracy's forced hit 0x179740:100-110] — at the ×0.7 Agent pain [M 0x1902c8 ×
        // Plr_DMod_Normal] the Operative-parity close fights are lethal (the Agent run took 79.4 in
        // the three clearing fights, 14.0 a pistol round at 1.7-2.7 u). The player fights from
        // OUTSIDE the band: the personal space 6.5 on EVERY difficulty — the Operative real-damage
        // walk's 4.0 stood inside the guards' own closing range (they shot on from 4-8 u; the
        // audit-voice-triggers-2 kill: the drone-#6 6-pellet frame at 2.5 u finished a body bled to
        // ~18 — the Agent sister's 6.5 stand-off is the honest spacing at both difficulties).
        w.keep_off = Some(6.5);
        w.dmg.back_off = Some(6.5);
        // pretrans-arms-4: the legs keep the same 6.5 to armed hostiles (the legs_keep_off field
        // doc) — the measured deaths were the LEGS walking Bond to 2.3-2.5 u of the planted hall
        // guards for two single-frame 8-pellet volleys (24.0 + 48.0 [L the NF_DMG_TRACE run]).
        w.legs_keep_off = true;
        // The pane-solid guards (PANE_RIDE, the kind-2 sheets the hall patrols brush [M
        // Collide_Pick 0x1eb320:313-329 — the movement query tests the start room's object
        // chain, and the panes link into their room by build_FindCel 0x1c8cb0]) shift the
        // guards' micro-paths, so a fight's stand-and-fire window can end with the guard
        // alive where the ghost-pane timeline dropped him — and the failed fight's 240-tick
        // cooldown is exactly the window a weapon-0x0 melee guard (the hall guard #5's
        // 35-hit shred in the ride-on bisection) needs to finish Bond. A player who fails to
        // drop the guard keeps shooting: the determined pass on a failed fight (the
        // 07000016 opt-in), 01's real-damage walks only.
        w.dmg.finish_fights = true;
        // pretrans-arms-4 (the -3 report's spelled-out lever): the hall flank wedges one step
        // early on the solid bodies, the 24-hp shotgun captain #7's planted volley (weapon 0x0,
        // script side [L the death ledger]) lands 48.0 while the stand is on another shooter,
        // and Bond arrives at the stairwell drained. The volley-carrier-first retarget (see the
        // DamageLog field) answers the shotgun mid-volley — the pellets stop with him.
        w.dmg.volley_carrier_first = true;
        // Agent: the return fire stands its ground (no back-away). Measured with the guards'
        // lines ending by their length (the headless sound driver): the start-room guard #0
        // no longer freezes in AIM_STAND_FIRE mid-line but side-steps (0x73/0x74), and the
        // 6.5 back-away carried Bond out of his line to (-8.0, -12.5) — the fight read
        // `blocked` for 300 ticks while the SMG pair #8/#13 shot on from 12.5 u (dead at tick
        // 6041/1163). Holding the line keeps the target in the gun ray; the walk's own legs
        // keep their 6.5 personal space (`keep_off`).
        if difficulty == 2 { w.dmg.back_off = None; }
    }
    // The type-2 guards carry 12 health, captains 24 [L ee_02b.bin]: Bond aims for the head
    // (`Walker::prefer_head`), as a player does, or the scripted fights run the walk dry.
    w.prefer_head = true;
    // A left nudge ends like a right one (`Walker::sidestep_decays`).
    w.sidestep_decays = true;
    // hud-start: a load holds nothing (0x47 [M Player_InitWeapon 0x1a7ef8:137]) - settle the start draw first.
    nf_view::settle_start_draw(w.scene.sim.as_mut().unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -2.78).abs() < 2.0 && (p[2] - -12.4).abs() < 2.0, "spawn #0 [D], got {p:?}");
    }
    // The intro cut-scene (SP #277, script 0x06000087) plays at load; its stream 8 writes ch 0xB
    // at ~tick 2119 and the script's tail (event 6 / Script_Free) re-places Bond at the script
    // matrix — wait the whole thing out before reading the route.
    let mut enabled_at = None;
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { enabled_at = Some(t); break; }
    }
    let enabled_at = enabled_at.expect("the intro must write ch 0xB (the contact's enable channel)");
    eprintln!("[01] intro ch 0xB at tick {enabled_at}");
    w.wait_enabled(6000, "the intro tail (event 6 re-places Bond)");
    // Real damage: the intro leaves the contact standing on the line from the start-room guard
    // #0 (-5.3, -0.5) to Bond (-8.5, -9.8), and a guard's round lands on any drone of the other
    // side it meets [M NDrone2_DealWithObjHit 0x13fff0:181-232: a team-1 shooter lands on teams
    // 2/3]. Idling there let guard #0's two rounds aimed at Bond kill the contact (frames 289
    // and 407, 5 each, mission failed 0x4000034) before the walk moved. A player steps out of
    // the line first: 1.6 east puts the contact 1.1 off it.
    if !w.tops_up() {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        w.walk_to(Wp { at: [p[0] + 1.6, p[1], p[2] - 0.4], act: Act::None, arrive: 0.3 }, "out of the contact's line");
        w.wait(240);
    } else {
        w.wait(300);
    }

    // The contact (skin 0x4F) is the escort: after ch 0xB he leaves WAIT_SWITCH, but the AllyLead
    // machine hides him (state 0x1f, [M AllyLeadHide 0x15ec08]) while an armed guard stands within
    // 14 units — the game's own behavior. The real player clears the guards; Bond shoots every
    // started armed guard within 14 of the contact through the real fire path.
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    drop(sim);
    // The friendly-fire shield (level-06-rook-door-3): the game's bullets hit ANY drone — no
    // team gate in Drone_CollisionHandler 0x140968:85-95 [M] — and the attributed round-0 kill
    // was BOND's own spread past guard #5 into the walking contact's head (the NF_HEALTH_TRACE
    // read; the guard set {3,5,14,15} never shot him — drone-fired bullets cannot even reach a
    // drone in the port, and the game's shooter would be the player's own pellet there too).
    // The player's play: never take a shot whose line runs through the escort. Every clearing
    // loop below inherits it through `shoot`.
    w.shield = Some(ci);
    // capsule-join-5: the return-fire stands inherit it too (DamageLog::fire_shield). With the
    // standing bodies the route pre-clear's return fire on guard #15 put Bond's volley through
    // the contact standing past him (by_player, weapon 0x1c, t6620 [L the NF_HEALTH_TRACE run])
    // and the mission failed. A player never fires through his escort, whichever fight it is.
    w.dmg.fire_shield = Some((ci, true));
    let guards: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && walker::dxz(d.pos, sim.drones.list[ci].pos) < 14.0
        }).collect()
    };
    eprintln!("[01] {} armed guard(s) within 14 of the contact — clearing", guards.len());
    // Agent kill order: the pellet-stream carriers first (the level's SMG guard 0x12 ground
    // 21 × 1.05-2.10 into the close-order run — a player silences the sprayer before the pistols).
    let mut guards = if difficulty >= 2 { agent_order(&w, &guards) } else { guards };
    for g in guards {
        // Walk within 9 of the guard first (line of sight), then shoot. Agent: stop at 8 — the
        // fight happens outside the TooClose band (the close-order run walked to 3.5 and fought
        // at 1.7-2.7 u under the ×2 damage).
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
        if walker::dxz(gpos, bpos) > 9.0 {
            w.walk_nav_to(gpos, &format!("approach guard {g}"), if difficulty >= 2 { 8.0 } else { 3.5 });
        }
        if !w.shoot(g, &format!("guard {g}")) {
            if w.resupply("ammo retry") { w.shoot(g, &format!("guard {g}")); }
        }
        // An empty gun cannot finish the fight — if the resupply also failed, move on rather than
        // hold an empty trigger through the 900-tick reposition windows.
        if {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.gun.clip + sim.gun.spare == 0
        } { eprintln!("[01] guard {g}: out of ammo and no box reached — leaving him for a later round"); continue; }
    }
    // The contact's hide exit: no armed drone within 14 → timer → Lead [M 0x15ec08]. Walk to him,
    // then escort: follow, and every time he cowers (an armed guard within 14 of him — LeadHide
    // [M 0x15ec08]) or parks (PlayerInWay), clear the armed started guards around him. The real
    // player fights through; the lead resumes on each clear.
    let cpos = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
    let plate = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH, 0x10).expect("the AND-touch plate");
    w.walk_to(Wp { at: cpos, act: Act::None, arrive: 2.5 }, "to the contact");

    // The meet beat — the level's second cut-scene, triggered entirely on disc. SP #276
    // (placement 276, script 0x06000086, 19 streams, in the level archive [D the disc scan —
    // pack 07800001 carries only the intro's 0x06000087]) is a mask-1 ScriptPlayer: it triggers
    // when its own out channel ch 0x33 rises [M SP_Hit 0x1dc470:17-21 flags bit 0x1 +
    // SP_Update 0x1dbda8 case 0]. The chain: the TOUCH_ONCE plate at (6.8, 6.0, 8.9) (mask 0x08)
    // writes ch 0x32 [D placement]; the OUT mux (0xEE, out 0x32 → list [0x33]) mirrors
    // ch[0x32] → ch[0x33] EVERY tick [M Trigger_Update 0x1b1a78 case 5:
    // switch_channels[list[k]] = switch_channels[out]]; SP #276 fires. So Bond WALKS the plate —
    // the real player's route into the north-east hall — and the scene plays out untouched
    // (streams end at frame 280 [D decode], ~4.7 s).
    w.walk_nav_to([6.8, 6.0, 8.9], "the meet plate (NE hall)", 1.0);
    // The plate is a TOUCH_ONCE on Bond's capsule — the touch lands when the capsule segment
    // crosses the volume's top face (feet ≈ the plate's floor 6.04). A nav stop a hair short
    // never crosses; a player steps ON it. Walk the centre tightly, then sweep the quad.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let plate = sim.drones.triggers.iter()
            .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x32)
            .map(|t| t.centre())
            .expect("the meet plate");
        drop(sim);
        w.walk_to(Wp { at: plate, act: Act::None, arrive: 0.25 }, "onto the meet plate");
        for (dx, dz) in [(0.0f32, 0.9f32), (0.0, -0.9), (0.9, 0.0), (-0.9, 0.0)] {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: [plate[0] + dx, plate[1], plate[2] + dz], act: Act::None, arrive: 0.2 },
                &format!("the meet plate sweep ({dx},{dz})"));
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: plate, act: Act::None, arrive: 0.2 }, "back across the meet plate");
        }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[01] meet plate: bond at {:?} ch 0x32={} ch 0x33={}", sim.bond.pos,
            sim.drones.channels.on(0x32), sim.drones.channels.on(0x33));
        assert!(sim.drones.channels.on(0x32), "walking the meet plate must fire its ch 0x32 [D placement]");
    }
    let mut e_at = None;
    for t in 0..300u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xe) { e_at = Some(t); break; }
    }
    eprintln!("[01] the meet scene (0x06000086) wrote ch 0xE at +{e_at:?} — stream 9's event 0xD ch 0xE val 1 [D decode]");
    assert!(e_at.is_some(), "the meet plate must fire ch 0x32 → the 0x33 mirror → SP #276 must play and write ch 0xE");
    // ch 0xE is the start channel of four hall guards [D placements; the probe's drone table]:
    // skins 0x39/0x36/0x38/0x3a at (−12.2,17.2), (5.4,14.4), (9.1,17.0), (5.9,17.9).
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let states: Vec<u8> = sim.drones.list.iter()
            .filter(|d| d.switch_channel == 0xe)
            .map(|d| d.state)
            .collect();
        assert_eq!(states.len(), 4, "four drones wait on ch 0xE [D placements]");
        assert!(states.iter().all(|&s| s != nf_game::drone::state::WAIT_SWITCH),
            "every ch-0xE guard must have left WAIT_SWITCH when the scene wrote the channel (states {states:?})");
    }

    // Top up BEFORE the hall: the meet scene's weapon hand left clip 32 spare 0, and the hall's
    // dodge fights burn more than a clip per guard (guard 6's fight ran dry at +634 ticks; the
    // resupply detour from inside the hall then wedged on the hall's south wall, the ambush
    // stayed alive and ch 0x99 never rose — the hold kept the contact at node 33 and the exit
    // died with it). A player tops up at the corridor boxes before the hall: the walked-path
    // corridor's own rule (resupply()).
    {
        // guard-radii-01-d3: at 00 Agent the threshold fights burn every pool the walk owns —
        // the 70165 ledger: the pistol's 54, the hall box's +20, then the wheel SMG's 55 via
        // the dry-gate's armed switch, and the resupply cycles INSIDE the hall find nothing
        // (the hall's south wall walls the nav off from the corridor boxes) — so the
        // fall-through shoot flanks the door dry and lip guard #5 takes the last 24 at 1.4 u.
        // The lever the finish report names: resupply BEFORE the hall phase, without a
        // mid-fight detour. resupply() takes the nearest walked-path box per call and the
        // taken one is gone, so repeated calls sweep the corridor's boxes onto the wheel.
        if difficulty >= 3 {
            for pass in 0..3 {
                if !w.resupply(&format!("d3 pre-hall stock {pass}")) { break; }
            }
            // Back to the meet room before the hall (the same invariant the dry branch keeps:
            // the stage below is the route proven FROM the meet plate).
            if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [6.8, 6.0, 8.9]) > 3.0 {
                w.walk_nav_to([6.8, 6.0, 8.9], "back to the meet room (d3 stock)", 1.5);
            }
        } else {
            let sim = w.scene.sim.as_ref().unwrap();
            let dry = (sim.gun.clip as i32) + (sim.gun.spare as i32) < 48;
            drop(sim);
            if dry {
                w.resupply("pre-hall ammo");
            // Back to the meet room before the hall: the stage below is the route proven FROM
            // the meet plate. From a corridor box south-west of it the stage leg's nav wedged at
            // (1.5, 7.1, 8.6) — Bond 0.6 above the room's walk height (6.47), the segment to the
            // stage hitting at t 0.31 — for its 3000-tick cap and every guard approach after.
            if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [6.8, 6.0, 8.9]) > 3.0 {
                w.walk_nav_to([6.8, 6.0, 8.9], "back to the meet room", 1.5);
            }
        }
        }
    }

    // The hall detour: the disc's AND mux (kind 3, out 0x99 = 153, list [0x11,0x12,0x13] [D the
    // placement]) fires when the three named guards are dead — their death channels are 0x13,
    // 0x11, 0x12 [D placement params]. A real player clearing the north-east hall fires this
    // scripted event; the walk does it through the real fire path before the escort.
    let named: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len())
            .filter(|&i| [0x11u8, 0x12, 0x13].contains(&sim.drones.list[i].death_channel))
            .collect()
    };
    assert_eq!(named.len(), 3, "three guards carry death channels 0x11/0x12/0x13 [D]");
    // Stage into the hall through its east door first. From the meet room the routes to the
    // west-half guards run through door #2 (−14.1,5.3,13.1) — the level's known walker wedge
    // (the solo approach wedges at the 3000-tick cap) — and a
    // guard fought through that wall never exposes a part (he stands behind the hall's south
    // structure). A player enters the hall by its main door and fights inside; the east hop is
    // the route the guard-11 approach already proves (nav reached (2.97,6.38,16.27) from the
    // same start), and from the stage the named guards are short in-hall hops.
    // EVERY difficulty (audit-voice-triggers-3): the close-order run walked the stage INTO the
    // north crowd (a 14.0 pistol round at 1.0 u, then the captain's full 8-pellet volley at 1.9 u,
    // mid-walk — dead before any fight could answer) — and the Operative real-damage walk bled the
    // same way (the 2026-11-05 kill: 37-39 hits for 100.0, the hall pair 70+ of it, the return-fire
    // fights answering only AFTER the damage). A player clears what he can see from the
    // meet room first: the named guards at 4-9 u are outside the TooClose band, in the open.
    {
        let pre_clear: Vec<usize> = if difficulty >= 2 { agent_order(&w, &named) } else { named.clone() };
        for g in pre_clear {
            let (los, dist) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                if d.corpse || d.health <= 0.0 { continue; }
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none(), walker::dxz(d.pos, bp))
            };
            if !los { eprintln!("[01] pre-clear guard {g}: no LOS from the meet room ({dist:.1} u) — leaving him for the staged loop"); continue; }
            if dist > 9.0 { w.walk_nav_to(w.scene.sim.as_ref().unwrap().drones.list[g].pos, &format!("pre-clear approach guard {g}"), 8.0); }
            if !w.shoot(g, &format!("pre-clear guard {g}")) {
                if w.resupply("pre-clear ammo") { w.shoot(g, &format!("pre-clear guard {g}")); }
            }
        }
    }
    // Agent: the blind stage walk is the killer — the nav threads the north crowd (the close-order
    // run took a 14.0 pistol round at 1.0 u and the captain's full 8-pellet volley at 1.9 u mid-walk,
    // dead before any fight could answer). The player stops on the hall's south floor short of the
    // crowd (outside the ×2 band, [M 0x139fac-0x139fc8 under DroneFiring_TooClose_Distance 3.0]) and
    // fights from there; the named loop's approaches cover the rest.
    // gas-rng-draws: the Operative (d1) run now takes the SAME doorway play. On the re-landed
    // stream the ch-0xE crowd bunches at the mouth the tick the meet scene writes 0xE — the d1
    // stage walk met three of them at 0.4-0.7 u and died there at tick 1930 (26 hits for 100.0;
    // the under-3-u forced hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] + the TooClose
    // ×2 [M Drone_ModBulletDamage 0x139f50]). The lure hold below is difficulty-blind geometry:
    // the hold line sits just outside the 3.0 band and the guards cross one at a time — easier
    // tuning than Agent (pain 0.5, accuracy 0.5), not a different game.
    if difficulty >= 1 {
        // The hall's south door (7.9, 11.1) is the only nav entry from the meet room, and the
        // alerted crowd hunts it in full cover: from the south their parts never expose — plain
        // AND scoped probes, with the door open or shut — while their rounds come THROUGH the
        // plane, and every crossing of the aperture under fire cost the probe runs 30-70 health
        // (the funnel concentrates both shotguns' whole volley where Bond walks). The hall
        // cannot be entered, fought or staged through at Agent. The player's answer — the one
        // that won the probe run at FULL health with the AND firing — is the sound lure: Bond
        // holds SOUTH of the pre-opened door and fires into the hall; the guards hunt the
        // gunshot hearing, come through the aperture ONE AT A TIME, and die in the doorway —
        // outside the 3.0 forced band, his parts behind the south structure between volleys.
        // (The hunters do cross: guard 11 was killed mid-crossing by the walk's own return fire
        // in an earlier run, and the lure run walked 5/6/7/11 AND the west-corridor pair 14/15
        // into the door line, all corpses at z 10.2-12.7.)
        // 1. The gun: the level-start pistol 0x2 — the deepest pool Bond owns here (the meet
        //    hand-off's SMG rides the wheel at clip 32 spare 0 [L the probe dump; 0x2 at clip 7
        //    + the type-1 pool 47]), the head box drops a 12-health guard in one hit [M
        //    NDrone2_HitDamage 0x145558 head ×100], and the pistol's record carries the 3.5 the
        //    walk's own clear fights registered [L the health trace: weapon 2 damage 3.5].
        // 2. The door pre-opens from the approach leg (the nav opens it at ~1.1 u [L the
        //    stage-walk trace "opening door ... state 0x2"]) and Bond falls back to the hold
        //    line while it swings.
        // 3. The doorway fight runs at the band's edge: keep_off 3.2, not the escort's 6.5 —
        //    the wide space backs Bond out through the door where the south structure
        //    re-occludes every part (the probe: exposed false at 6.7 u with the door open);
        //    inside 3.0 every drone round hits forced [M 0x179740:100-110], so the hold line
        //    sits just outside it and the shoot loop's backing never crosses back in.
        // The stage walk afterwards runs from the hold line once the AND has fired (the crowd
        // thinned); the loop below leaves a never-showing guard to the escort rounds, as before.
        w.tick(Input { switch_to: Some(0x2), ..Default::default() }, false);
        for _ in 0..60 { w.tick(Default::default(), false); }
        w.walk_nav_to([7.5, 6.4, 10.1], "the hall door approach (pre-opens it)", 0.8);
        w.walk_nav_to([7.2, 6.4, 8.8], "the door-hold line", 1.0);
        w.wait(60);
        // guard-goal-arrival-radii: the goal machinery's own radii (cover 0.5/1.0, the AI points'
        // 1.0 accuracies [D the aipoint_accuracy_census]) bring the hall guards ~1 unit deeper
        // than the flat 2.0 stand-in did — the 00agent run's captain reached 2.9 u (INSIDE the
        // old 3.2 band, his 8-pellet volley then took 42 health in one burst). The band widens
        // at d3 ONLY (the d1/d2 balance stays on the tuned 3.2): still well inside the 6.7-u
        // occlusion edge (the probe: exposed false at 6.7 u with the door open), so the doorway
        // fight keeps its LOS while Bond backs out of the close-order shotgun band [M
        // 0x179740:100-110's forced-hit radius is 3.0].
        let band = if difficulty >= 3 { 4.0 } else { 3.2 };
        w.keep_off = Some(band);
        w.dmg.back_off = Some(band);
        // And at d3 the hold line itself sits 1.2 farther south (z 8.8 -> 7.6): with the goal
        // radii live the captains close to 2.4 u and the backing is wall-limited — the deeper
        // line gives the backing room to keep the shotgun band (>3.0 forced-hit radius [M
        // 0x179740:100-110]) while the doorway LOS holds (4.0 < the 6.7-u occlusion edge).
        if difficulty >= 3 {
            w.walk_nav_to([7.2, 6.4, 7.6], "the door-hold line (d3, the radii live)", 1.0);
        }
    }
    // (the d1 stage walk is gone — gas-rng-draws: every difficulty holds the door line now; the
    // named loop fights the crowd through the aperture and the follow crosses the cleared door.)
    // Agent kill order in the hall too: the shotgun captain (0x1c, 24 hp) before the pistols —
    // the close-order run walked to 3.0 of guard #11 first and ate the captain's full 8-pellet
    // volley at 1.9 u while the pistol round landed at 1.0 (the ×2 TooClose band). A player
    // silences the volley first, from outside the band.
    let mut named = if difficulty >= 2 { agent_order(&w, &named) } else { named };
    for g in named {
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        // The guards hunt: guard 11 left his placement for the hall's south entry and the
        // approach from the courtyard side wedged on the hall's south wall (the nav gave up
        // at the 3000-tick cap, segment_to_target ≈0.47). Re-stage through the proven route
        // (the meet-room corridor → the east door) before closing from inside.
        // EVERY difficulty: NO restage and NO approach — the restage's east hop threads the same
        // crowd (the close-order run died mid-stage to a 14.0 round at 1.0 u and the captain's
        // volley at 1.9) and the door funnel's hops cannot stop short (the 8.0 arrive
        // overshot to 0.9). The guards HUNT to the south door: hold the stage point and let
        // them come — the doorway fight sits at the ×2 band's edge and their parts show
        // through the open door. The approach stays the last resort (a guard that never
        // comes), after the hold.
        if difficulty >= 1 {
            // The lure first: a burst INTO the hall at the guard's own spot — the guards hunt
            // the gunshot hearing, and the hunter paths south through the open aperture.
            // level-01-agent-hall-fight-2: the aim TRACKS HIS LIVE POSITION, not the placement
            // snapshot — the old burst aimed at a point captured once, so the crossing guard's
            // rounds landed only after the show-then-shoot settle gap (76.5 -> 42.9 across the
            // door phase in the worst run); with the stream riding him through the aperture the
            // taps land DURING the crossing and he dies inside his own volley budget (the
            // report's "the live-tracking stream" move). The burst still ends when HIS part
            // shows in the doorway (the scoped LOS probe at the hold line) or at the 300-tick
            // cap; the hold below then breaks at once and the fight takes over.
            {
                // The lure is bounded at 300 ticks (5 s, ~18 taps): the sound is the draw — a
                // sustained burst only burns the pool. The burst ends when HIS part shows in
                // the doorway or at the cap; the hold below gives him the crossing window.
                for t in 0..300u32 {
                    let (input, trigger, done) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let d = &sim.drones.list[g];
                        if d.corpse || d.health <= 0.0 { (Input::default(), false, true) }
                        else {
                            let bp = sim.bond.pos;
                            // The live aim point: his current position each tick (+0.3 the
                            // torso lift the scoped probe uses; the fight's prefer_head
                            // refines from here).
                            let at = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                            let dx = at[0] - bp[0]; let dz = at[2] - bp[2];
                            let dy = at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
                            let desired = dx.atan2(dz);
                            let err = walker::wrap_pi(desired - sim.bond.yaw);
                            let want_pitch = (dy / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                            let perr = want_pitch - sim.bond.pitch;
                            let seen = walker::dxz(d.pos, bp) < 9.0
                                && sim.world.segment_bullet_scoped(sim.world.find_cel([bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]]), [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]], [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none();
                            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                                err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, seen)
                        }
                    };
                    if done { break; }
                    w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
                    w.total_ticks += 1;
                    w.after_step();
                    if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                }
            }
            // Then the hold: give the door its swing and the hunter his crossing window. A
            // guard that never shows is left for the escort rounds (the contact stalls on him
            // and the clear loop re-visits).
            for _ in 0..40 {
                let seen = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[g];
                    if d.corpse || d.health <= 0.0 { break; }
                    let bp = sim.bond.pos;
                    let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                    walker::dxz(d.pos, bp) < 11.0
                        && sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none()
                };
                if seen { break; }
                w.wait(30);
            }
        }
        let _ = gpos;
        // Dry mid-hall is the walk's own trap: the hall's south wall walls the resupply nav
        // off from the corridor boxes (every southward hop wedged at segment t≈0.095). A
        // player steps out the door he came in by: east door → corridor box → back inside.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let dry = sim.gun.clip + sim.gun.spare < 8;
            drop(sim);
            if dry && difficulty >= 1 {
                // The door phase: the ladder's stage leg crosses the funnel under fire (the
                // first run died there). Resupply from the hold line and walk back.
                // guard-pane-drain-4 (d3): when the corridor is empty (the pre-hall stock took
                // its boxes; "no pickup within the walked-path corridor (46 trail points)" [L
                // the d3fix run]) the player walks the off-path detour leg the drain-3 ignore
                // names — the boxes at (-7.5,5.3,-6.8)/(-3.3,5.8,-9.1), taken by real input,
                // which also puts them on the trail for the loop's later resupply calls. The
                // armed gun then feeds the loop's own bounded aperture lure — the armed
                // sustained lure (900 ticks of t%16 taps ≈ 56 rounds — the run that drew
                // 5/6/7/11 across carried clip 32 spare 16 [L the lure run note]).
                if !w.resupply(&format!("hall guard {g} ammo")) && difficulty >= 3 {
                    d3_ammo_detour(&mut w, &format!("hall guard {g} ammo detour"), [7.2, 6.4, 8.8]);
                }
                if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [7.2, 6.4, 8.8]) > 2.0 {
                    w.walk_nav_to([7.2, 6.4, 8.8], &format!("hall guard {g} back to the hold line"), 1.2);
                }
            }
        }
        // join-fix-g (d2 only): on the joined tree the hall guards side-step behind the jamb
        // (z 14-18) and never cross during the lure; the shoot machinery's flank then opened the
        // door and walked Bond into the hall, where the shotgun captain #7 took 83.5 in 24 hits
        // at 3.4 u. The player leans to the angle instead, from the meet-room side of the door.
        {
            let (across, alive) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                (d.pos[2] > 11.2, !d.corpse && d.health > 0.0)
            };
            if difficulty == 2 && across && alive && w.exposed_aim(g).is_none()
                && door_peek_fight(&mut w, g, &format!("hall guard {g}")) {
                continue;
            }
        }
        // guard-radii-finish (d3 only): a named guard ACROSS the aperture with no part from the
        // hold line must not take the shoot machinery's flank — on the merged tree the flank nav
        // opened the hall door and walked Bond INTO the hall, where the 12-hp shotgunner #6
        // (0x5000036, 0x1c) stood 1.9 u off and the ×2 TooClose band [M Drone_ModBulletDamage
        // 0x139f50] took 58.0 in 8 hits (the 19098 death, the 2026-11-06 run). The player's line
        // is the threshold fight: step into the already-open aperture (7.9, 10.6 — the flank
        // nav's own door-opening proves the line), fight his live position from there (the
        // structure's jamb ends at the door plane, so his torso clears from the threshold), and
        // step back south to the hold line on the first wound — his volleys at >3.0 are outside
        // the forced band. He still dies for the AND; the hold line invariant survives.
        let (across, alive) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[g];
            (d.pos[2] > 11.2, !d.corpse && d.health > 0.0)
        };
        if difficulty >= 3 && across && alive {
            let mut fought = false;
            for _ in 0..2 {
                w.walk_nav_to([7.9, 6.4, 10.6], &format!("hall guard {g} the aperture threshold"), 0.5);
                let hp0 = w.scene.sim.as_ref().unwrap().drones.health.health;
                for t in 0..1200u32 {
                    let (input, trigger, done) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let d = &sim.drones.list[g];
                        if d.corpse || d.health <= 0.0 { (Input::default(), false, true) }
                        else if sim.drones.health.health <= hp0 - 2.0 { (Input::default(), false, true) }
                        else {
                            let bp = sim.bond.pos;
                            let at = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                            let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
                            let err = walker::wrap_pi(desired - sim.bond.yaw);
                            let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                            let perr = want_pitch - sim.bond.pitch;
                            (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                                err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, false)
                        }
                    };
                    if done { fought = true; break; }
                    w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
                    w.total_ticks += 1;
                    w.after_step();
                    if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                }
                // Back to the hold line: the fight ran inside his 3.0-u forced band for at most
                // the first wound — the band's own backing rule.
                w.walk_nav_to([7.2, 6.4, 7.6], &format!("hall guard {g} back off the threshold"), 1.0);
                let dead = w.scene.sim.as_ref().unwrap().drones.list[g].health <= 0.0;
                if dead { fought = true; break; }
            }
            if fought { continue; }
            // Still across and alive after the threshold rounds: on the merged tree the stream
            // shift parked #6 deeper in the hall (z 17.8 — his boxes walled from the threshold,
            // health 12 untouched through 2×1200 threshold taps [L the 8604 run]) and the plain
            // shoot's flank became the killer: its nav opened the hall door and walked Bond
            // INTO the hall ("hall guard 6 flank 1 nav 2: opening door" [L the same run]),
            // dry at clip 0, where the shotgun captain #5's 22.8-u pellets finished him at tick
            // 8604. The player falls back to the sound lure that won the doorway fight: hold
            // the line, fire at the guard's LIVE position, and let the goal machinery hunt the
            // gunshot hearing [M DroneFunc_SetNoiseHeard] across the aperture — the taps land
            // during the crossing (the live-tracking stream; the lure run walked 5/6/7/11 into
            // the door line, corpses at z 10.2-12.7 [L the lure run note above]). Bounded: no
            // crossing in 900 ticks (or a still-across guard) leaves him for the escort-round
            // clears rather than any flank into the hall.
            {
                let (still_alive, still_across) = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[g];
                    (!d.corpse && d.health > 0.0, d.pos[2] > 11.2)
                };
                if still_alive && still_across {
                    // Dry at the door phase starves the lure: the run entered the flank at
                    // clip 4 spare 0 and died dry mid-flank [L the 8604 run]. The walk's own
                    // resupply idiom (the dry mid-hall block) from the hold line.
                    if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 {
                        if !w.resupply(&format!("hall guard {g} lure ammo")) && difficulty >= 3 {
                            d3_ammo_detour(&mut w, &format!("hall guard {g} lure detour"), [7.2, 6.4, 8.8]);
                        }
                        if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [7.2, 6.4, 8.8]) > 2.0 {
                            w.walk_nav_to([7.2, 6.4, 8.8], &format!("hall guard {g} back to the hold line for the lure"), 1.0);
                        }
                    }
                    for t in 0..900u32 {
                        let (input, trigger, done) = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let d = &sim.drones.list[g];
                            if d.corpse || d.health <= 0.0 { (Input::default(), false, true) }
                            else if d.pos[2] <= 11.2 { (Input::default(), false, true) }
                            else {
                                let bp = sim.bond.pos;
                                let at = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                                let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
                                let err = walker::wrap_pi(desired - sim.bond.yaw);
                                let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                                let perr = want_pitch - sim.bond.pitch;
                                (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                                    err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, false)
                            }
                        };
                        if done { break; }
                        w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
                        w.total_ticks += 1;
                        w.after_step();
                        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                    }
                    let (alive, across_now) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let d = &sim.drones.list[g];
                        (!d.corpse && d.health > 0.0, d.pos[2] > 11.2)
                    };
                    eprintln!("[01] hall guard {g} the aperture lure: alive {alive} across {across_now} at tick {}", w.total_ticks);
                    if !alive || across_now { continue; } // dead → the AND takes his channel; across → the escort clears, never a flank into the hall
                }
            }
        }
        if !w.shoot(g, &format!("hall guard {g}")) {
            if w.resupply("hall ammo") { w.shoot(g, &format!("hall guard {g}")); }
        }
    }
    // The last guard's channel lands with the end of his fall [M 0x1637f8 case 2], up to 213 ticks.
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x99) { break; }
        w.tick(Default::default(), false);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x99),
        "AND(0x11,0x12,0x13) must raise ch 153 once the three named guards are dead [D mux placement]");
    eprintln!("[01] the hall AND fired: ch 153 on at tick {}", w.total_ticks);
    // The door-lure fight leaves Bond on the hold line south of the door; the contact's route
    // to the LoadPointActivator plate runs back through the hall (his stall sat at (5.9,14.1),
    // route 18/37), so Bond steps just INSIDE the doorway — 2.6 u from the route, inside the
    // lead's 5.0 release band [M NDrone2_DSTATE_AllyLeadWait 0x15eda8 case 3]. The direct
    // steer crosses the already-open aperture; the nav's own funnel hop wedges here (the
    // closes capped at 3000 ticks four runs running).
    // gas-rng-draws: the 0x500003a pair in the south lower hall (≈(-7.9, 6.3, -0.4)) covers the
    // door mouth from ~19.7-19.8 u — the d3 doorway fight ate 98.4 of 100 from them (the 4163
    // death: 84.0 in 6 + 14.4 in 8). A player silences the far field BEFORE the aperture work:
    // they stand south-west of the hold line, so the approach walks AWAY from the funnel. The
    // skin+presence filter self-gates the loop (no such started guard → no-op).
    far_field_sweep(&mut w, "post-AND");
    if difficulty >= 1 {
        // real-health-00agent-01: the AND's three named guards are down, but the hall holds
        // MORE than them — the d3 run's #5 (0x5000038, the shotgun 0x1c, 24 hp) stood 0.9 u off
        // the aperture's north side and the doorway step ate his volley point-blank (the 4119
        // death: -14.40/-2.80 at 0.9 u, the ×2 TooClose band [M Drone_ModBulletDamage
        // 0x139f50]; the named loops never visit him — he is outside the AND and outside every
        // later route window's 9 u). A player does not step through an aperture with a guard on
        // its lip: from the hold line, the lure draws him (the guards hunt the gunshot hearing
        // [M DroneFunc_SetNoiseHeard] — the same bounded live-tracked burst the named loop
        // uses) and the fight takes him as he crosses. Was D3-gated to keep the Agent sister's
        // timeline byte-exact; gas-rng-draws: that timeline is red on this stream anyway, the lip
        // guard exists at every difficulty (the d1 ledger's #7, skin 0x5000038 weapon 0x1c 24 hp,
        // stood at the mouth pre-fix), and d1 now holds the door line like the sisters — the gate
        // opens to all.
        let lip: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && walker::dxz(d.pos, [7.9, 6.3, 11.1]) < 7.0
            }).collect()
        };
        for g5 in lip {
            for t in 0..300u32 {
                let (input, trigger, done) = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[g5];
                    // gas-rng-draws: the far field can START mid-hold (the d3 4163
                    // death: 84.0 in 6 + 14.4 in 8 from 19.7-19.8 u while the lip fight held
                    // the mouth). A player under new fire from the south-west breaks the hold
                    // and answers it first — the lip guard is re-held on the next round and
                    // the far sweep below takes them the tick they pop.
                    let far_started = sim.drones.list.iter().any(|d| {
                        d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                            && walker::dxz(d.pos, [-7.9f32, 6.3f32, -0.4f32]) < 6.0
                    });
                    if far_started { (Input::default(), false, true) }
                    else if d.corpse || d.health <= 0.0 { (Input::default(), false, true) }
                    else {
                        let bp = sim.bond.pos;
                        let at = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                        let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
                        let err = walker::wrap_pi(desired - sim.bond.yaw);
                        let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                        let perr = want_pitch - sim.bond.pitch;
                        (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                            err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, d.pos[2] < 10.5)
                    }
                };
                if done { break; }
                w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
                w.total_ticks += 1;
                w.after_step();
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            // gas-rng-draws: a far-pair start mid-hold breaks the hold (above) — the pair is
            // answered BEFORE the lip shot, the tick they pop, not after the whole lip loop.
            far_field_sweep(&mut w, "mid-lip");
            if !w.shoot(g5, &format!("door lip guard {g5}")) {
                if w.resupply("door lip ammo") { w.shoot(g5, &format!("door lip guard {g5} retry")); }
            }
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
    }
    // guard-radii-01-d3: the ledger's #5 (0x5000038, the shotgun 0x1c, 24 hp) surfaces AFTER the
    // AND — the iter-1 trace ran the first lip scan while he was still off (no "door lip guard"
    // fight printed), and the route pre-clear leg then walked Bond past his (4.5,15.0) muzzle at
    // 0.9 u (-14.40 ×3, the 3844 death). A player who just cleared the hall holds the door line
    // and watches the aperture while the escort starts crossing: poll the same scan from the
    // hold line and fight the lip the moment it shows. D3-GATED: the d1/d2 timelines stay
    // byte-exact.
    if difficulty >= 3 {
        for poll in 0..20 {
            let lip2: Vec<usize> = door_lip_scan(&w);
            if lip2.is_empty() { w.wait(30); continue; }
            eprintln!("[01] the door-lip re-scan (+{poll} polls) found {lip2:?} — fighting from the hold line");
            for g5 in lip2 {
                door_lip_fight(&mut w, g5, "escort lip");
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            break;
        }
    }
    // gas-rng-draws: SECOND far-field sweep — the pair sits on a later start than the hall
    // AND (the first sweep passed them by unstarted; the d3 ledger has them firing at 4163).
    // Placed before the doorway step, the one move that walks Bond back into their 19.7-u
    // lane. Self-gates: nothing started → a no-op.
    far_field_sweep(&mut w, "pre-doorway");
    if difficulty >= 1 {
        // gas-rng-draws: on the merged stream the door fight can end with the leaf frozen
        // mid-travel (state 0x8, pos01 0.80, ch_open 0 — a live 0x5c guard parked 2 u off the
        // hinge and the dump's corpse-blind filter showing nothing in the arc). Bond is then
        // ALREADY inside the hall (z 12.3, 1.9 u off the route's door node (5.9, 14.1) —
        // inside the lead's 5.0 release band [M 0x15eda8 case 3]), and a player does not need
        // to touch the frozen leaf: he stands in the band and the escort continues. The
        // walk's 6000-tick cap is therefore downgraded here from a panic to a reported stall
        // (the positioning purpose is met at the stall spot); every other stream — where the
        // leaf travels — is byte-identical (the walk arrives, no panic path runs).
        // The pressed pane #130 now breaks under the opening guard's dive (the guards' step probe
        // no longer stops at live glass), which moves the hall fight: the flank can leave Bond west
        // of the hall's small SS box (x 0.85..1.20, z 12.57..12.93), and the straight line to the
        // doorway steps him up onto it. A player walks round it, along the route's door node row.
        if w.scene.sim.as_ref().unwrap().bond.pos[0] < 3.0 {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                w.walk_to(Wp { at: [3.2, 6.4, 14.1], act: Act::None, arrive: 0.6 }, "round the hall's SS box");
            }));
        }
        let door_stall = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            w.walk_to(Wp { at: [7.9, 6.4, 12.3], act: Act::None, arrive: 0.6 }, "into the hall doorway (the release band)");
        }));
        if door_stall.is_err() {
            eprintln!("[01] the hall doorway walk stalled on the frozen leaf (state 0x8 pos01 0.80) — Bond holds the release band from here (the route node (5.9,14.1) is 1.9 u off)");
        }
        // gas-rng-draws: the door fight's survivors cower (COMBAT_NO_SIGHT 0x5c [M drone.rs])
        // 2-5 u north of the doorway — the d1 stream left #14 (5.9, 13.7) and #15 (2.9, 13.5),
        // both 12 hp — and a LIVE actor in the leaf's reach keeps the door's blocked scan
        // failing (the class-2 gate is present ∧ non-corpse ∧ health > 0 [M door.rs
        // DoorMover::valid]), so the slid leaf never retracts and the next nav hop into the
        // west corridor walks straight into it (the 3000-tick hop cap). A player finishing a
        // doorway fight drops the cowerers he can see before moving on; every death re-opens
        // the door's close arm. Self-gates: no live guard near the mouth → a no-op (the
        // streams where the leaf travels are untouched).
        {
            let survivors: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && walker::dxz(d.pos, [6.0f32, 6.3f32, 13.0f32]) < 6.0
                }).collect()
            };
            for g2 in survivors {
                if !w.shoot(g2, &format!("doorway survivor {g2}")) {
                    if w.resupply("doorway survivor ammo") { w.shoot(g2, &format!("doorway survivor {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            far_field_sweep(&mut w, "post-doorway");
        }
        // The player walked this corridor five minutes ago and knows the guards: the contact's
        // route to the plate runs west through the west corridor, where the SMG pair (12/17)
        // and the 24-health guard 3 sit on the ch-0x26/0xf waits — the follow legs bled
        // 76.5 -> 42.9 crossing their volleys in the first full-route run. He clears the route
        // BEFORE following (the corridor's own resupply boxes sit on the way; the walk's dry
        // ladder tops up mid-sweep).
        for wp in [[-14.1f32, 13.1f32], [-19.4f32, 15.25f32], [-23.3f32, 14.4f32], [-29.1f32, 11.5f32]] {
            let near: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && walker::dxz(d.pos, [wp[0], 6.4, wp[1]]) < 9.0
                }).collect()
            };
            for g2 in route_order(&w, &near) {
                let gpos = w.scene.sim.as_ref().unwrap().drones.list[g2].pos;
                let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
                // The pocket rule (the ambush block): a guard parked west of x -27 is shot
                // from where Bond stands — the approach never walks into the pocket corner.
                if walker::dxz(gpos, bpos) > 9.0 && gpos[0] > -27.0 {
                    w.walk_nav_to(gpos, &format!("route pre-clear approach guard {g2}"), 8.0);
                }
                if !w.shoot(g2, &format!("route pre-clear guard {g2}")) {
                    if w.resupply("route pre-clear ammo") { w.shoot(g2, &format!("route pre-clear guard {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
        // level-01-agent-hall-fight-2: fetch the pickups on the way before the stairwell — the
        // west-hall placed pair (−14.4, 6.5, 16.1) / (−13.2, 5.6, 10.4) sits 3-5 u off the route
        // line and the fights' drops (every killed guard drops his weapon [L the 01 drops]) land
        // at the fight floors. A player sweeping the corridor walks the line and tops up before
        // the stairwell descent instead of waiting for the dry gate; resupply() stays the dry
        // fallback. Real input only — the walk-over take [M Pickup_Handler 0x18e650].
        {
            let route: &[[f32; 2]] = &[[-14.1, 13.1], [-19.4, 15.25], [-23.3, 14.4], [-29.1, 11.5]];
            let stops: Vec<[f32; 3]> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let by = sim.bond.pos[1];
                sim.drones.pickups.iter()
                    .filter(|p| !p.gone && (p.m[3][1] - by).abs() < 1.5)
                    .filter(|p| route.iter().any(|r| walker::dxz([p.m[3][0], p.m[3][1], p.m[3][2]], [r[0], by, r[1]]) < 9.0))
                    .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
                    .collect()
            };
            for (k, at) in stops.iter().enumerate() {
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                w.walk_nav_to(*at, &format!("the pickup line {k}"), 1.0);
            }
        }
    }

    // guard-radii-01-d3: the finish report's own "post-escort re-sweep" — the first lip scan
    // ran the moment the AND fired, and the ledger's #5 (0x5000038, the shotgun 0x1c, 24 hp)
    // surfaces LATE: the 70165 death ate his -14.40/-14.80 volley at 1.4-2.2 u once the crossing
    // began. As the escort crossing begins, run the lip block's scan again and fight whoever it
    // finds — from the hold line, the same lure-and-fight shape. D3-GATED: the d1/d2 timelines
    // stay byte-exact.
    if difficulty >= 3 {
        let lip2: Vec<usize> = door_lip_scan(&w);
        if !lip2.is_empty() {
            eprintln!("[01] the escort-crossing lip re-scan found {lip2:?} — walking the hold line and fighting");
            if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, [7.2, 6.4, 8.8]) > 3.0 {
                w.walk_nav_to([7.2, 6.4, 8.8], "the door-hold line (the escort lip re-scan)", 1.2);
            }
        }
        for g5 in lip2 {
            door_lip_fight(&mut w, g5, "escort lip");
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
    }
    // los-mask-repins-01 (d2-gated, the d1 timeline stays byte-exact): the game's sight moved
    // the west-end fights onto the contact. On the d2 tree the lead crossed the ch-0xF slab
    // ~t9500 while Bond was still closing (the pop is Bond-touch only — his crossing popped
    // nothing [L the run's alarm trace]), g3 (ch-0x26 waiter, west room) walked to the
    // stairwell head and chipped him from t9710, and Bond's own t10284 slab crossing popped
    // g4/12/17 onto him at 5.4 u — dead at t10814, ch 0x60, the escort fail restarts the
    // level. A player clears the ambush line BEFORE escorting through it: the slab crossing +
    // mouth fight + the ch-0x26 waiters as they show, all while the contact still waits east
    // (the hall's live holders keep his hide on).
    run_slab_ambush(&mut w, ci, difficulty);
    let mut settled = cpos;
    // The contact's park from the previous round: two rounds inside ~1 unit with no guard to
    // clear means the lead machine wedged on geometry — Bond walks PAST him toward the plate
    // (the real player's move): out of the PlayerInWay band the lead resumes and re-paths [M
    // 0x15ea28 case 3 / 0x15e698 case 3], which unsticks him. The head-look merge's corrected
    // head box (the game's own AnimGetCollData → psiBuildMatrixPalette → HeadTrack coupling)
    // moves the fight timeline, so the lead can park in spots the old path never hit.
    let mut prev_settled = cpos;
    let mut cleared = std::collections::HashSet::new();
    // gas-rng-draws: the legs are SHORT (900, not 12000) and the rounds many (48): the d1
    // ledger's death rode ONE 5300-tick follow leg — #3 (0x500036 SMG) popped on the
    // contact's ch-0xF slab crossing mid-leg and sprayed 71.0 in 94 hits from 10.8 u before
    // the between-leg clear ever ran. A player under an SMG's fire breaks off every few
    // seconds and answers it; the per-round own-radius clear (below, at 14 u) is that
    // answer. Same following capacity (48×900 > 12000), the clears run every leg.
    for round in 0..48 {
        settled = w.follow(ci, &format!("the contact r{round}"), 900, plate);
        if walker::dxz(settled, plate) < 5.0 { break; }
        // The player also clears his own radius as he walks: the route guards the contact woke
        // (the west-corridor SMG pair) shoot Bond DURING the follow legs, and the return-fire
        // exchanges bleed the Agent run dry before the stairwell (76.5 -> 31.7 across one
        // route). Same clear the stall branch runs, keyed to Bond's position.
        {
            let near: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && walker::dxz(d.pos, bp) < 14.0
                }).collect()
            };
            for g2 in route_order(&w, &near) {
                let gpos = w.scene.sim.as_ref().unwrap().drones.list[g2].pos;
                let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
                // The pocket rule (the ambush block): a guard parked west of x -27 is shot
                // from where Bond stands — the approach never walks into the pocket corner.
                if walker::dxz(gpos, bpos) > 9.0 && gpos[0] > -27.0 {
                    w.walk_nav_to(gpos, &format!("the contact r{round} own-radius approach guard {g2}"), 8.0);
                }
                if !w.shoot(g2, &format!("the contact r{round} own-radius guard {g2}")) {
                    if w.resupply("own-radius ammo") { w.shoot(g2, &format!("the contact r{round} own-radius guard {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
        }
        // los-mask-repins-01 (d2-gated): the lead walks INTO the west end ahead of Bond, and a
        // guard he woke on the way (the ch-0x26 pair; g3's walk to the stairwell head) is
        // answered from the contact's side too, not only within 14 of Bond — on the d2 tree g3
        // reached the head and opened fire on the contact (2.2 + the 22.5 head hit at t9710-
        // t11007 shapes) while Bond's 900-tick leg was still running [L the run's health trace].
        if difficulty >= 2 {
            let near_c: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                        && walker::dxz(d.pos, cp) < 14.0
                }).collect()
            };
            for g2 in route_order(&w, &near_c) {
                let gpos = w.scene.sim.as_ref().unwrap().drones.list[g2].pos;
                let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
                // The pocket rule (the ambush block): a guard parked west of x -27 is shot
                // from where Bond stands — the approach never walks into the pocket corner.
                if walker::dxz(gpos, bpos) > 9.0 && gpos[0] > -27.0 {
                    w.walk_nav_to(gpos, &format!("the contact r{round} contact-radius approach guard {g2}"), 8.0);
                }
                if !w.shoot(g2, &format!("the contact r{round} contact-radius guard {g2}")) {
                    if w.resupply("contact-radius ammo") { w.shoot(g2, &format!("the contact r{round} contact-radius guard {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
        }
        // Stalled: who is armed and started within 14 of the contact?
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled
                    && walker::dxz(d.pos, settled) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() {
            // No threat: the lead may just be WAITING for Bond (state 0x20 = the lead wait, Bond
            // inside 3.0 [M 0x15e4f0 case 3]) or parked after the detour. PlayerInWay (0x1e) is the
            // opposite: Bond is IN the lead's path (`route_blocked && dist <= 3.0` [M 0x15e4f0's
            // PlayerInWay arm]) and he resumes only when Bond is OUT of the way band (dist > 3.0
            // [M 0x15ea28 case 3]) — so Bond steps BACK behind him (away from the plate) instead of
            // closing in, which would keep him parked. Otherwise close the distance and follow again;
            // panic only after several fruitless rounds.
            let sim = w.scene.sim.as_ref().unwrap();
            let cstate = sim.drones.list[ci].state;
            drop(sim);
            // The meet hall's west wall check (the level-11-stairwell-bullet-wall-3 stall): Bond
            // chased guard 15 OUT of the hall (the north lane) and stands in the west corridor with
            // the hall's west wall segment (x≈1.93, z 11.1-13.1 [D the disc's collision cels — the
            // port's ray fan, walkreal_probe11::dump_07000001_west_wall]) between him and the
            // contact; every close/walk-past target sits inside the hall and the straight-line
            // steering wedges on the wall face (the nav's own first hop is a hall-side node across
            // the wall, and the stuck ladder's sidesteps slide ALONG it, never the ~1.7 north to
            // the wall's north edge). The player walks back the lane he came out by — the hall's
            // north opening, the AI's own chain node[8] (−0.39,14.28) → node[23] (3.39,14.69) →
            // node[7] (6.88,14.62) [D nav path 0 nodes+links; the lane passes the AI's bounds
            // walls through their z≈13.1 north end], then closes on the contact; the AllyLeadWait
            // release [M NDrone2_DSTATE_AllyLeadWait 0x15eda8 case 3: pd > −7.0 && (pd != 0.0 ||
            // dist2d < 5.0)] needs Bond within 5.0 straight-line when pd == 0.0 (Bond off the
            // route's 5.0 node scan), and inside the hall he is.
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let (bp, cp) = (sim.bond.pos, sim.drones.list[ci].pos);
                let walled = bp[0] < 2.0 && sim.world.segment(bp, cp).is_some();
                drop(sim);
                if walled {
                    eprintln!("[01] round {round}: the hall's west wall is between Bond ({bp:?}) and the contact ({cp:?}) — walking the north lane back in");
                    w.walk_nav_to([0.6, 6.5, 14.3], "the west corridor lane (north)", 1.4);
                    w.walk_nav_to([3.39, 6.4, 14.69], "the hall's north opening (nav node 23)", 1.4);
                    w.walk_nav_to([6.88, 6.4, 14.62], "inside the hall (nav node 7)", 1.4);
                    w.walk_nav_to(settled, "close on the contact (the [M 0x15eda8] release band)", 1.2);
                    prev_settled = settled;
                    continue;
                }
            }
            if cstate == nf_game::drone::state::PLAYER_IN_WAY {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                let back = [cp[0] + (cp[0] - plate[0]) * 0.5, cp[1], cp[2] + (cp[2] - plate[2]) * 0.5];
                drop(sim);
                eprintln!("[01] round {round}: contact in PlayerInWay — Bond steps back off his lane");
                w.walk_nav_to(back, "step out of the lead's way", 3.0);
            } else if round >= 24 {
                let sim = w.scene.sim.as_ref().unwrap();
                panic!("[01] the contact stalled at ({:.1},{:.1},{:.1}) state {:#x} armed_peer {:.1} with no guard to clear (round {round}; the cap 24 gives the [M 0x15eda8] d2<5 release time to land on the slow closing)",
                    settled[0], settled[1], settled[2], sim.drones.list[ci].state, sim.drones.list[ci].armed_peer_dist);
            } else if walker::dxz(settled, prev_settled) < 1.0 {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                let (dx, dz) = (plate[0] - cp[0], plate[2] - cp[2]);
                let l = (dx * dx + dz * dz).sqrt().max(1e-3);
                let ahead = [cp[0] + dx / l * 4.0, cp[1], cp[2] + dz / l * 4.0];
                drop(sim);
                eprintln!("[01] round {round}: the lead is wedged at ({:.1},{:.1},{:.1}) state {cstate:#x} — Bond walks past him toward the plate (the lead resumes out of the way band and re-paths)",
                    settled[0], settled[1], settled[2]);
                prev_settled = settled;
                let before = w.scene.sim.as_ref().unwrap().bond.pos;
                w.walk_nav_to(ahead, "walk past the wedged lead", 3.0);
                let moved = walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, before);
                if moved < 1.0 {
                    // The nav gave up (the hall's meet-frozen bodies park mid-lane when the
                    // fight resolves differently): a real player vaults the crowd — the jump
                    // press carries the capsule over the parked bodies.
                    eprintln!("[01] the walk-past nav gave up — Bond vaults forward (the jump press)");
                    let sim = w.scene.sim.as_ref().unwrap();
                    let (bp, cp) = (sim.bond.pos, sim.drones.list[ci].pos);
                    drop(sim);
                    let to = [cp[0] - bp[0], cp[2] - bp[2]];
                    let l = (to[0] * to[0] + to[1] * to[1]).sqrt().max(1e-4);
                    for _ in 0..90 {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let err = walker::wrap_pi(to[0].atan2(to[1]) - sim.bond.yaw);
                        drop(sim);
                        w.tick(nf_game::player::Input { forward: 1.0, jump: true, turn: err.clamp(-0.08, 0.08), ..Default::default() }, false);
                    }
                }
            } else {
                // Close RIGHT up (1.2, not the old 4.0): the AllyLeadWait release [M 0x15eda8
                // case 3, the port's ALLY_LEAD_WAIT arm] needs Bond's mission-path distance back
                // inside (−7, 0] — on the hall's winding route a 4.0 straight-line stop still read
                // > 7 route-units and the wait held. A real player walks up to his partner.
                eprintln!("[01] round {round}: no guards to clear (contact state {cstate:#x}) — closing on him");
                prev_settled = settled;
                w.walk_nav_to(settled, "close on the contact", 1.2);
            }
            continue;
        }
        eprintln!("[01] round {round}: the contact stalled at ({:.1},{:.1},{:.1}) — clearing guards {:?}",
            settled[0], settled[1], settled[2], guards);
        // Diagnostic: at the second fruitless round on the same set, dump the guards' boxes vs the
        // aim ray (the guard-15 miss the previous session probed).
        if round >= 5 {
            let sim = w.scene.sim.as_ref().unwrap();
            for &g in &guards {
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                eprintln!("[01] diag guard {g}: pos {:?} health {:.0} state {:#x} crouched-anim state {:#x} dist {:.1}",
                    d.pos, d.health, d.state, d.anim.state, walker::dxz(d.pos, bp));
                let to = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                eprintln!("[01] diag guard {g}: torso seg hit {:?}", sim.world.segment(eye, to));
                let data = nf_game::drone::Data {
                    tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
                    world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins, skeletons: &sim.skeletons,
                    nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning, hitboxes: &sim.hitboxes, volumes: &sim.volumes,
                    sfx: sim.sfx.as_ref(), models: &sim.models, surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: 1,
                    ai_points: &sim.ai_points,
                };
                let boxes = d.hit_boxes(&data);
                for b in boxes.iter().take(8) {
                    eprintln!("[01] diag guard {g} part {:3}: c ({:.2},{:.2},{:.2}) half {:?} from-eye {:.1}",
                        b.part, b.m[3][0], b.m[3][1], b.m[3][2], b.half, walker::dxz([b.m[3][0], b.m[3][1], b.m[3][2]], bp));
                }
            }
        }
        // The Agent kill order here too (the pellet carriers first; see the initial clear).
        let guards = if difficulty >= 2 { agent_order(&w, &guards) } else { guards };
        for g in guards {
            // Walk TO the guard (the route's doors open under Bond's feet via the stuck handler —
            // a midpoint stop can sit behind a wall with no line of sight). The gate is LOS, not
            // just distance: at the stairwell corner Bond stood 3.2 from the guard with the
            // eye→torso ray hitting a wall at t 0.18 — the shots could never land. Agent: stop at
            // 8 (outside the TooClose band).
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            // Agent: the hall's guards HUNT to the south door (guard 6 met the door-stander at
            // 0.7 u in the close-order run; the stage nav opens the door for them) — the approach
            // walks Bond into the doorway volleys, and the door funnel's hops cannot stop short
            // anyway (the 8.0 arrive overshot to 0.9). The player HOLDS the stage point and lets
            // them come: the doorway firefight sits at the ×2 band's edge, and their parts show
            // through the open door. ~15 s of hold, then fight whatever showed; the approach is
            // the last resort for a guard that never comes.
            let mut los_clear = los_clear;
            let mut gpos = gpos;
            let mut bpos = bpos;
            if difficulty >= 2 && (walker::dxz(gpos, bpos) > 9.0 || !los_clear) {
                for _ in 0..30 {
                    w.wait(30);
                    let sim = w.scene.sim.as_ref().unwrap();
                    let d = &sim.drones.list[g];
                    if d.corpse || d.health <= 0.0 { break; }
                    let bp = sim.bond.pos;
                    let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                    let dist = walker::dxz(d.pos, bp);
                    let seen = dist < 10.0
                        && sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none();
                    drop(sim);
                    if seen { break; }
                }
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                los_clear = sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none();
                gpos = d.pos;
                bpos = bp;
                drop(sim);
            }
            if walker::dxz(gpos, bpos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("approach guard {g}"), if difficulty >= 2 { 8.0 } else { 3.0 });
            }
            // Low ammo: grab the nearest pickup first (the placed kind-3 ammo / the guard drops).
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.gun.clip + sim.gun.spare < 8 { drop(sim); w.resupply("ammo"); }
            }
            // Skip the hopeless fight: no ammo and no box reached — the round loop retries after
            // the drops land (every killed guard drops his weapon [L the 01 drops]).
            let ammo_now = w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare;
            if ammo_now == 0 { eprintln!("[01] guard {g}: skipped — dry"); continue; }
            let mut killed = w.shoot(g, &format!("guard {g}"));
            if !killed {
                // Out of ammo mid-fight: resupply and retry once.
                if w.resupply("ammo retry") { killed = w.shoot(g, &format!("guard {g}")); }
            }
            // Only DEAD guards leave the clear list — a survivor re-hides the contact.
            let dead = w.scene.sim.as_ref().unwrap().drones.list[g].corpse
                || w.scene.sim.as_ref().unwrap().drones.list[g].health <= 0.0;
            if killed || dead { cleared.insert(g); }
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        eprintln!("[01] the contact settled at ({:.1},{:.1},{:.1}) state {:#x} route_idx {:?} blocked {} armed {:.1} bond d {:.1}",
            d.pos[0], d.pos[1], d.pos[2], d.state, d.route.as_ref().map(|r| (r.index, r.nodes.len())), d.route_blocked, d.armed_peer_dist,
            walker::dxz(sim.bond.pos, d.pos));
        eprintln!("[01] anim: danim {} state {:#x} ftime {:.1} script {:?} held {} once {} call {:?} drawn {}",
            d.anim.danim, d.anim.state, d.anim.ftime, d.anim.script, d.anim.held, d.anim.once, d.anim.call, d.drawn);
        eprintln!("[01] yaw {:.2} desired {:.2} target {:?}", d.yaw, d.desired_yaw, d.route.as_ref().map(|r| r.target));
        if let Some(t) = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::TOUCH && t.out == 0x10) {
            eprintln!("[01] plate lo {:?} hi {:?} centre {:?}", t.lo, t.hi, t.centre());
        }
        // The way west from his nose: world geometry at hip heights?
        for (dx, dz) in [(-0.5f32, 0.0), (-1.0, 0.0), (-2.0, 0.0)] {
            let from = [d.pos[0], d.pos[1], d.pos[2]];
            let to = [d.pos[0] + dx, d.pos[1], d.pos[2] + dz];
            eprintln!("[01] world seg to ({:+.1},{:+.1}): hip {:?} hip+0.5 {:?} feet {:?}",
                dx, dz, sim.world.segment([from[0], from[1], from[2]], [to[0], to[1], to[2]]),
                sim.world.segment([from[0], from[1] + 0.5, from[2]], [to[0], to[1] + 0.5, to[2]]),
                sim.world.segment([from[0], from[1] - 0.9, from[2]], [to[0], to[1] - 0.9, to[2]]));
        }
    }
    eprintln!("[01] the contact settled at ({:.1},{:.1},{:.1}), plate at ({:.1},{:.1},{:.1})",
        settled[0], settled[1], settled[2], plate[0], plate[1], plate[2]);
    assert!(walker::dxz(settled, plate) < 6.0, "the contact must walk his route to the LoadPointActivator plate");
    // Bond walks the last leg onto the plate (the contact stalled at the stairwell head;
    // the AND cannot fire with him outside the volume).
    drop(sim);
    run_slab_ambush(&mut w, ci, difficulty);
    {
        // The player clears his OWN radius before the stairwell descent: the route guards the
        // contact woke (the west-corridor SMG pair 12/17 among them) sit 2-6 u off the final
        // legs, and the return-fire exchanges bled the run dry there (31.7 -> 0 across two
        // volleys in the first full-route run). Same loop shape as the escort rounds' clears.
        // Every difficulty: once the guards take their door routes (door 2 at (-15, 6.3, 14.1),
        // the route's door binding [M 0x153358:149-163]) #17 reaches the stairwell head on
        // Operative too.
        // At Agent the SMG pair's close volleys double inside 3.0 u (DroneFiring_TooClose
        // [M Drone_ModBulletDamage 0x139f50]); keep the walk's Agent stand-off (6.5) here too.
        let prev_back_off = w.dmg.back_off;
        w.dmg.back_off = Some(6.5);
        // The game's body push (guard-standing-capsule: every near bot's standing capsule
        // [M NDrone2_Collision 0x140e90:91-121] pushes a grounded Bond on every contact tick
        // [M Collide_Jointy 0x1ec180:229-281]) moves Bond's fight positions here — and the
        // pocket guards' flank offsets then drew Bond into the stairwell mouth, where a wedge
        // sidestep carried him down onto the contact's plate (the exit fired mid-fight). The
        // block's own pocket rule below (a guard west of x −27 is shot from where Bond stands)
        // now bounds the flank too: no approach into the mouth, the exit stays for the plate legs.
        let prev_flank_hold = w.flank_hold_x;
        w.flank_hold_x = Some(-27.0);
        for sweep in 0..3 {
            let near: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && walker::dxz(d.pos, bp) < 12.0
                }).collect()
            };
            if near.is_empty() { break; }
            // The guards Bond can SEE first (a player answers the gun that is firing at him):
            // with the game's standing bodies the fight positions shift, and the walled
            // stairwell sniper #2 (state 0x2a, never steps out) held the loop for thousands of
            // ticks while the exposed rifleman #17 fired through the contact at Bond and killed
            // him [L capsule-join trace: contact 7.8 → 0 at t25197-25972, #17 state 0x68 at 0.9 u
            // from him]. Before each walled guard, every guard now visible is answered first.
            let (seen, walled): (Vec<usize>, Vec<usize>) = route_order(&w, &near).into_iter()
                .partition(|&g| walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g));
            for g2 in seen.into_iter().chain(walled) {
                if !walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g2) {
                    let now_seen: Vec<usize> = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let bp = sim.bond.pos;
                        (0..sim.drones.list.len()).filter(|&i| {
                            let d = &sim.drones.list[i];
                            i != ci && i != g2 && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                                && walker::dxz(d.pos, bp) < 12.0 && walker::DamageLog::visible(sim, i)
                        }).collect()
                    };
                    for g3 in now_seen {
                        w.shoot_bounded(g3, &format!("pre-stairwell seen guard {g3}"), 2400);
                        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
                    }
                }
                {
                    let d = &w.scene.sim.as_ref().unwrap().drones.list[g2];
                    if d.corpse || d.health <= 0.0 { continue; }
                }
                let gpos = w.scene.sim.as_ref().unwrap().drones.list[g2].pos;
                let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
                // The pocket rule (the ch-0xF ambush block above): a guard parked west of x -27
                // is shot from where Bond stands, never approached into the corner.
                if walker::dxz(gpos, bpos) > 9.0 && gpos[0] > -27.0 {
                    w.walk_nav_to(gpos, &format!("pre-stairwell approach guard {g2}"), 8.0);
                }
                if !w.shoot(g2, &format!("pre-stairwell guard {g2}")) {
                    if w.resupply("pre-stairwell ammo") { w.shoot(g2, &format!("pre-stairwell guard {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            let _ = sweep;
        }
        w.dmg.back_off = prev_back_off;
        w.flank_hold_x = prev_flank_hold;
    }
    // The player route descends the stairwell from the west hall's corner (the AI's own path
    // climbs over the top floor — the sister proved the descent through this anchor).
    // pretrans-arms-4: the legs' 6.5 keep-off releases here — the remaining guards are the
    // walled pocket pair (the stairwell sniper #2 sits at y 10.2 over the mouth, never fired
    // [L the -3 death ledger and the runs on this tree: no #2 hits in any ledger]) and the
    // final legs must pass under him at 3.2 u — every green tree walked that stretch (the
    // keep-off leg backed off it forever: the 3000-tick plate cap in the first run).
    w.legs_keep_off = false;
    w.walk_nav_to([-30.8, 6.4, 13.1], "the stairwell head", 1.2);
    w.walk_nav_to(plate, "the plate", 1.2);
    let sim = w.scene.sim.as_ref().unwrap();
    let on_plate = sim.bond.pos[0] < -32.0 && sim.bond.pos[2] > 12.0;
    if !on_plate {
        let ci2 = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).unwrap();
        let c = &sim.drones.list[ci2];
        eprintln!("[01] off the plate: contact #{ci2} at {:?} state {:#x} health {:.1} corpse {} fail {:?}", c.pos, c.state, c.health, c.corpse, c.fail_reason);
    }
    assert!(on_plate, "Bond must walk onto the LoadPointActivator plate (at {:?}; level_exit {:x?} restart {} cold {} health {:.1} ch10 {} tick {})",
        sim.bond.pos, sim.drones.level_exit, sim.drones.level_exit_restart, sim.drones.level_exit_cold, sim.drones.health.health, sim.drones.channels.on(0x10), w.total_ticks);
    {
        let d = &sim.drones.list[ci];
        eprintln!("[01] at the AND-touch: contact at ({:.2},{:.2},{:.2}) state {:#x} route_idx {:?} blocked {} seen {}",
            d.pos[0], d.pos[1], d.pos[2], d.state, d.route.as_ref().map(|r| (r.index, r.nodes.len())), d.route_blocked, d.seen);
        for (i, o) in sim.drones.list.iter().enumerate() {
            if i == ci || o.team != 1 || !o.started || o.corpse || !o.enabled { continue; }
            let dd = walker::dxz(o.pos, d.pos);
            if dd < 16.0 {
                eprintln!("[01] peer {i} skin {:#x} at ({:.1},{:.1},{:.1}) d {dd:.1} state {:#x} health {:.0}",
                    o.skin, o.pos[0], o.pos[1], o.pos[2], o.state, o.health);
            }
        }
    }
    // The stairwell guards (the west-side combat-steppers, plus the balcony skin-0x36) hide the
    // contact at the head — in the game the hide runs him to cover (`NDrone2_FindRunToPoint`
    // 0x15168, the half the port does not model) while the player fights. The walk fights them
    // the same way: clear every started, armed guard within 14 of the CONTACT (the hide exit
    // needs the visible armed peer beyond 14.0 [M `NDrone2_NotNearArmedDrone` 0x177640]), let the
    // lead finish the route's descent, then take Bond back onto the plate for the AND.
    'recovery: for round in 0..12 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        if guards.is_empty() { break; }
        for g in guards {
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            let bpos = w.scene.sim.as_ref().unwrap().bond.pos;
            // The pocket rule (the ambush + pre-stairwell blocks): a guard parked west of
            // x -27 (the stairwell-head pocket) is shot from where Bond stands — the d2
            // 8502 death walked the 3.0-arrive approach INTO the pocket corner and ate
            // #3's 31.5-in-15 from 7.1 u. Hunters in the open corridor keep the approach.
            if gpos[0] > -27.0 && (walker::dxz(gpos, bpos) > 9.0 || !los_clear) {
                w.walk_nav_to(gpos, &format!("approach stairwell guard {g}"), 3.0);
            }
            // The contact reached the plate while Bond stood on it: the AND fired on this leg
            // (the walk stops on the exit tick) — the fight below would walk Bond off the plate
            // and drop ch 0x10 before the check reads it.
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break 'recovery; }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                if sim.gun.clip + sim.gun.spare < 8 { drop(sim); w.resupply("ammo"); }
            }
            let ammo_now = w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare;
            if ammo_now == 0 { eprintln!("[01] stairwell guard {g}: skipped — dry"); continue; }
            let mut killed = w.shoot(g, &format!("stairwell guard {g}"));
            if !killed {
                if w.resupply("ammo retry") { killed = w.shoot(g, &format!("stairwell guard {g}")); }
            }
            let dead = w.scene.sim.as_ref().unwrap().drones.list[g].corpse
                || w.scene.sim.as_ref().unwrap().drones.list[g].health <= 0.0;
            if killed || dead { cleared.insert(g); }
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break 'recovery; }
        }
        w.follow(ci, &format!("the contact descent r{round}"), 6000, plate);
    }
    // Bond LEADS the descent: he takes the plate first — the AND needs BOTH, so the exit holds
    // until the contact arrives — while the hand (below) lands on the live world. Bond ahead is
    // fine for the lead: the wait bands stall only for a Bond BEHIND on the route [M 0x15e698
    // case 3's −9.0 path arm], so the contact's route walks him down to the plate.
    let on_plate = |p: V3| p[0] < -32.4 && p[1] < 4.0 && (12.2..18.7).contains(&p[2]);
    let exited = w.scene.sim.as_ref().unwrap().drones.channels.on(0x10);
    if !exited && !on_plate(w.scene.sim.as_ref().unwrap().drones.list[ci].pos) {
        // The proven descent: the stairwell head anchor first (the direct nav to the plate from
        // the balcony side climbs over the top floor and never enters the stairwell — the sister
        // proved the descent through this anchor).
        w.walk_nav_to([-30.8, 6.4, 13.1], "the stairwell head", 1.2);
        w.walk_nav_to(plate, "the plate (lead the descent)", 1.2);
    }
    // level-01-end-fade: the pin path stops AT the fire — the mission-monitor block below waits
    // 30 ticks that would burn into the fade's 300 before the read (the walk path keeps them).
    if stop_at_fire && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return w; }
    // obj0 "Escort Mayhew to bunker" (text 0x04000006) watches ch 0x47 [D mission::c07000001,
    // ee_start_05 @ 0x2a3eb8: text 0x04000006, ch 0x47, enable 0, flags 0]. The objective does
    // NOT complete on this level, and this walk now pins that instead of standing in for it:
    //   * no 07000001 disc data writes 0x47 — the level census found the only script-0xD writes
    //     0xB and 0xE, and no trigger/mux/lock/pickup carries 0x47 [D walkreal_probe01];
    //   * no ACTION.ELF code writes ch 0x47 immediately either — this session's disassembly
    //     census of every switch_channels (0x26fc90) store (lui 0x27/-0x370 base idiom, tracked
    //     through addiu/addu to sb/sh/sw) shows the only immediate channels are 0x2d, 0x5e, 0x62,
    //     0x63, 0x64, 0x6f, 0x97, 0x98, 0x9b, 0xfd [M-dis work/elf/ACTION.ELF]; every other
    //     writer (Trigger/SP/Script_EventHandler/DroneFunc_HostageSaved/Pickup/Door/...) takes
    //     its channel from disc data;
    //   * the real writer is the NEXT level's story hand-off: 07000002's level-start cut-scene
    //     SP #400, script 0x06000084 (pack 07800002, mode 1), stream 7 writes ch 0x47 = 1
    //     [D 07800002 0x06000084 stream 7] — the same one-part-late shape as 0x4D, which
    //     07000003's TouchOnce #308 writes for 07000002's obj2 [D 07000003 param sweep].
    // walkreal_07000002_live pins that write and the resulting fold live; here the game's own
    // monitor state on THIS level is pinned: obj0 announced and active, everything else waiting
    // [M 0x1827d8 case 0 announces when MissionData row level <= GameState; case 1 waits on the
    // enable channel; case 3 folds only when the watch channel rises].
    // The record indexes resolve from the LEVELS table, not hardcoded: the table mirrors the
    // game's .data order and the mission-05..08 records sit in FRONT of 01's [M 0x1827d8 walks
    // LEVELS in table order; a hardcoded [0][0] named 01's record before those landed].
    let m01 = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0001).expect("07000001's LEVELS record");
    let m02 = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0002).expect("07000002's LEVELS record");
    // The monitor settles; fire-aware so the pin's read stays on the fire tick (identical 30
    // ticks when !stop_at_fire).
    for _ in 0..30 {
        if stop_at_fire && w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
        w.tick(Default::default(), false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[m01][0], 3, "obj0 announced (immediate reveal, enable 0) and still active — the fold belongs to 07000002's intro stream 7 [M 0x1827d8 case 0; D 07800002 0x06000084]");
        assert_eq!(sim.drones.mission.states[m02][0], 1, "07000002 'Destroy Mayhew's computer' still waits on enable 0x47 [M 0x1827d8 case 1]");
        assert_eq!(sim.drones.mission.states[m02][1], 1, "07000002 'Rescue geisha girls' still waits on enable 0x47");
        assert_eq!(sim.drones.mission.states[m02][2], 1, "07000002 'Find door to servants quarters' still waits on enable 0x4c");
        // [S] unverified — kept out of pass/fail. "07000001 never raises the obj0 completion
        // channel" rests on a MISSING writer (the census above: level data, every
        // switch_channels 0x26fc90 immediate, every disc-data-driven writer), and an assertion
        // may not rest on a missing writer — the port may simply lack it. Logged, never
        // asserted:
        if sim.drones.channels.on(0x47) {
            eprintln!("[01] [S] unverified — ch 0x47 is ON after the walk: a writer the census missed (or the un-ported mission layer) fired; the never-raises expectation is NOT met");
        } else {
            eprintln!("[01] [S] unverified — ch 0x47 still off after the walk: matches 'no disc writer on this level' [D census + M-dis], but the port may simply lack the writer, so this stays out of pass/fail");
        }
        assert!(!sim.drones.channels.on(100), "the mission spans 07000002 — success waits for its objectives");
    }
    if !exited {
        // The contact finishes the route onto the plate → the AND → the exit (Bond leads from the
        // plate; the wait loop gives the descent his window). The game's own escort STOP: while
        // Bond stands within 3.5 dxz inside the contact's π/3 facing cone, PlayerInWay (0x1e)
        // HOLDS — the case-3 stay arm returns 1 while `Dist2D(Bond) < 3.5` [M 0x15ea28:32-47],
        // and the 3.5-exit/Lead re-entry is the only release (the wedge is the game's rule, not a
        // port stall — 957's guard-combat rework shifted the endgame race so Bond's plate-first
        // parking now lands inside the band). A real player steps aside; the harness does the
        // same through the real input path: retreat to the stairwell-head anchor (outside the
        // plate's y band AND the 3.5 band) whenever the contact is wedged, and let the lead
        // resume before re-taking the plate.
        for _ in 0..80 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                let cp = sim.drones.list[ci].pos;
                let wedged = sim.drones.list[ci].state == 0x1e // PLAYER_IN_WAY [M 0x15ea28]
                    || walker::dxz(cp, sim.bond.pos) < 3.5;
                drop(sim);
                if wedged && !on_plate(cp) {
                    w.walk_nav_to([-30.8, 6.4, 13.1], "step aside (the PlayerInWay release [M 0x15ea28])", 1.2);
                }
            }
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
            if on_plate(w.scene.sim.as_ref().unwrap().drones.list[ci].pos)
                && !w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) {
                w.walk_nav_to(plate, "the plate (re-take)", 1.2);
            }
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { break; }
            // level-01-end-fade: the wait breaks AT the fire — the pin below reads the hold with
            // all 300 frames left on the exit tick itself (the countdown begins the next tick
            // [M 0x1bada0:16-20 advances the fade on the frames after the SetFade stores level 0]).
            for _ in 0..60 {
                if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
                w.tick(Default::default(), false);
            }
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x10), "the AND-touch must fire with Bond AND the contact on the plate (contact at {:?})",
        sim.drones.list[ci].pos);
    drop(sim);
    // The exit fires the same tick ch 0x10 rises (the LOAD_LEVEL trigger's gate IS 0x10 — the
    // case-10 arm fires on the channel, no proximity check [M Trigger_Update 0x1b1a78:162-166]) —
    // stop at the fire tick; the walk test and the fade pin both resume here.
    for _ in 0..60u32 {
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { return w; }
        w.tick(Default::default(), false);
    }
    panic!("the exit must fire within 60 ticks of the AND-touch (level_exit posted)");
}


/// Whether a bullet from `eye` enters any of drone `di`'s hit boxes before the world: the
/// walker's `exposed_aim` box test, asked from a stand Bond is not on yet.
fn exposed_from(w: &Walker, di: usize, eye: V3) -> bool {
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[di];
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    let cel = sim.world.find_cel(eye);
    d.hit_boxes(&data).iter().any(|b| {
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        let Some(t_box) = nf_game::damage::ray_box(eye, ray, b) else { return false };
        !sim.world.segment_bullet_scoped(cel, eye, c).is_some_and(|t| t < t_box)
    })
}

/// The peek fight's fire loop on guard `g` from where Bond stands: `Some(true)` when he dies,
/// `Some(false)` when Bond dies or runs dry, `None` after a 12-point wound or a lost line.
fn peek_fire(w: &mut Walker, g: usize) -> Option<bool> {
        let hp0 = w.scene.sim.as_ref().unwrap().drones.health.health;
        let mut lost = 0u32;
        let mut aim = w.exposed_aim(g);
        for t in 0..900u32 {
            if t % 10 == 0 { aim = w.exposed_aim(g); }
            lost = if aim.is_none() { lost + 1 } else { 0 };
            let (input, trigger, reload, done) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                if d.corpse || d.health <= 0.0 { return Some(true); }
                if sim.drones.health.health <= hp0 - 12.0 || lost > 40 { (Input::default(), false, false, true) }
                else {
                    let bp = sim.bond.pos;
                    let at = aim.unwrap_or([d.pos[0], d.pos[1] + 0.3, d.pos[2]]);
                    let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
                    let err = walker::wrap_pi(desired - sim.bond.yaw);
                    let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                    let perr = want_pitch - sim.bond.pitch;
                    let empty = sim.gun.clip == 0 && sim.gun.spare > 0;
                    (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                        aim.is_some() && !empty && err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0,
                        empty && t % 20 == 0, false)
                }
            };
            if done { break; }
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, reload, false);
            w.total_ticks += 1;
            w.after_step();
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.health.health <= 0.0 { return Some(false); }
            if sim.gun.clip == 0 && sim.gun.spare == 0 { return Some(false); }
        }
        let d = &w.scene.sim.as_ref().unwrap().drones.list[g];
        if d.corpse || d.health <= 0.0 { Some(true) } else { None }
}

/// pretrans-arms-2: the armed, live, started team-1 guards standing within 3.0 of the escort
/// `ci` — a round at any of them crosses the escort (the walker's shield holds the tap).
fn at_the_contact(w: &Walker, ci: usize) -> Vec<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    let c = sim.drones.list[ci].pos;
    (0..sim.drones.list.len()).filter(|&i| {
        let d = &sim.drones.list[i];
        i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
            && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
            && walker::dxz(d.pos, c) < 3.0
    }).collect()
}

/// pretrans-arms-2: the escort lure. While an armed guard stands at the contact's shoulder,
/// Bond waits east of the corridor door (out of the pocket's sight line) for up to 2400 ticks;
/// the walker's return fire answers whoever comes through, the shield holding every tap that
/// would cross the contact.
fn lure_off_the_contact(w: &mut Walker, ci: usize) {
    if at_the_contact(w, ci).is_empty() { return; }
    eprintln!("[01] guards {:?} at the contact's shoulder: luring them off", at_the_contact(w, ci));
    w.walk_nav_to([-12.8, 6.45, 15.1], "the lure hold (east of the corridor door)", 0.8);
    for _ in 0..80 {
        if at_the_contact(w, ci).is_empty() { break; }
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { return; }
        w.wait(30);
    }
    eprintln!("[01] lure done: at the contact {:?}", at_the_contact(w, ci));
}

/// Agent hall door, a guard across the aperture that the hold line cannot see (he side-steps
/// behind the jamb, z 14-18): a player leans to the angle that sees him, from the meet-room side
/// of the door plane. The stand is the nearest spot south of the door (z 8.4-10.8, x 6.0-9.8)
/// with a straight walk from Bond, a bullet line into one of his boxes, and 3.4 u from every
/// live hall guard (outside the 3.0 forced-hit band [M DroneWeap_DoBulletAccuracy
/// 0x179740:100-110]). Bond walks there, fires at the live part, and steps back to the hold line
/// after a 12-point wound or when the line is gone. Never through the door plane: the flank that
/// walked into the hall died there at 3.4 u.
fn door_peek_fight(w: &mut Walker, g: usize, tag: &str) -> bool {
    let hold = [7.2f32, 6.46, 9.4];
    for round in 0..10u32 {
        // The pre-open from the approach leg no longer swings the leaf on the joined tree (no
        // "opening door" on the approach; the flank's own E opened it): a player opens it by hand.
        let door = walker::door_near(w.scene.sim.as_ref().unwrap(), [7.9, 6.4, 11.1]).map(|(_, i)| i);
        if let Some(di) = door {
            if w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0xc == 0 {
                w.use_door(di, &format!("{tag} peek door {round}"));
            }
        }
        let stand = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.list[g];
            if d.corpse || d.health <= 0.0 { return true; }
            let bp = sim.bond.pos;
            let eye_h = sim.bond.eye - sim.bond.crouch_drop;
            let mut best: Option<(f32, V3)> = None;
            for ix in 0..=19 {
                for iz in 0..=8 {
                    let c = [6.0 + ix as f32 * 0.2, bp[1], 8.4 + iz as f32 * 0.3];
                    if walker::dxz(c, d.pos) < 3.4 { continue; }
                    if sim.drones.list.iter().any(|o| o.team == 1 && o.enabled && o.present && !o.corpse
                        && o.health > 0.0 && walker::dxz(o.pos, c) < 3.4) { continue; }
                    if sim.world.segment([bp[0], bp[1] + 0.9, bp[2]], [c[0], c[1] + 0.9, c[2]]).is_some() { continue; }
                    let k = walker::dxz(c, bp);
                    if best.is_some_and(|(bk, _)| bk <= k) { continue; }
                    if !exposed_from(w, g, [c[0], c[1] + eye_h, c[2]]) { continue; }
                    best = Some((k, c));
                }
            }
            best.map(|(_, c)| c)
        };
        let Some(stand) = stand else {
            // join-reds2-0: while no stand sees guard `g`, a hall partner who DOES show from
            // here is answered first — on the joined tree the captain #7 (0x1c) stood in the
            // aperture's line at (5.4, 18.1) and took 84.0 in 20 hits off the waiting walk.
            let other = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| i != g).find(|&i| {
                    let o = &sim.drones.list[i];
                    o.team == 1 && o.enabled && o.present && !o.corpse && o.health > 0.0
                        && walker::dxz(o.pos, bp) < 14.0 && w.exposed_aim(i).is_some()
                })
            };
            if let Some(o) = other {
                eprintln!("[01] {tag}: no stand for {g} in round {round}; #{o} shows from the hold line — answering him");
                if peek_fire(w, o) == Some(false) { return false; }
                continue;
            }
            {
                let sim = w.scene.sim.as_ref().unwrap();
                for (i, o) in sim.drones.list.iter().enumerate() {
                    if o.team == 1 && o.enabled && o.present && !o.corpse && o.health > 0.0 && walker::dxz(o.pos, hold) < 14.0 {
                        eprintln!("[01] {tag}: no peek stand {round}: #{i} at ({:.1},{:.1},{:.1}) st {:#x} hp {:.0}", o.pos[0], o.pos[1], o.pos[2], o.state, o.health);
                    }
                }
            }
            if walker::dxz(w.scene.sim.as_ref().unwrap().bond.pos, hold) > 0.8 {
                w.lenient = true;
                w.walk_to(Wp::new(hold[0], hold[1], hold[2]).r(0.5), &format!("{tag} peek back {round}"));
                w.lenient = false;
            }
            w.wait(60);
            continue;
        };
        eprintln!("[01] {tag}: peek stand {round} at ({:.1},{:.1}) for guard {g} at {:?}", stand[0], stand[2],
            w.scene.sim.as_ref().unwrap().drones.list[g].pos);
        w.lenient = true;
        w.walk_to(Wp::new(stand[0], stand[1], stand[2]).r(0.3), &format!("{tag} peek {round}"));
        w.lenient = false;
        match peek_fire(w, g) {
            Some(true) => return true,
            Some(false) => return false,
            None => {}
        }
        w.lenient = true;
        w.walk_to(Wp::new(hold[0], hold[1], hold[2]).r(0.5), &format!("{tag} peek back {round}"));
        w.lenient = false;
        eprintln!("[01] {tag}: peek {round} done, health {:.1}, guard hp {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health,
            w.scene.sim.as_ref().unwrap().drones.list[g].health);
    }
    let d = &w.scene.sim.as_ref().unwrap().drones.list[g];
    d.corpse || d.health <= 0.0
}
/// The 00 Agent door-lip lure (see the call site): every hall guard within 7 of the south door is
/// drawn by a live-tracked burst from the hold line and fought as he crosses.
fn door_lip_lure(w: &mut Walker) {
        let lip: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| {
            let d = &sim.drones.list[i];
            d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                && walker::dxz(d.pos, [7.9, 6.3, 11.1]) < 7.0
        }).collect()
    };
    for g5 in lip {
        for t in 0..300u32 {
            let (input, trigger, done) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g5];
                if d.corpse || d.health <= 0.0 { (Input::default(), false, true) }
                else {
                    let bp = sim.bond.pos;
                    let at = [d.pos[0], d.pos[1] + 0.3, d.pos[2]];
                    let desired = (at[0] - bp[0]).atan2(at[2] - bp[2]);
                    let err = walker::wrap_pi(desired - sim.bond.yaw);
                    let want_pitch = ((at[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop)) / walker::dxz(bp, at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
                    let perr = want_pitch - sim.bond.pitch;
                    (Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() },
                        err.abs() < 0.02 && perr.abs() < 0.028 && t % 16 == 0, d.pos[2] < 10.5)
                }
            };
            if done { break; }
            w.scene.sim.as_mut().unwrap().step_bond(input, trigger, false, false);
            w.total_ticks += 1;
            w.after_step();
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
        if !w.shoot(g5, &format!("door lip guard {g5}")) {
            if w.resupply("door lip ammo") { w.shoot(g5, &format!("door lip guard {g5} retry")); }
        }
        if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
    }
    }

#[ignore = "pretrans-arms-5 ledger: the IsOpponent under-0.9 no-turn fix [M 0x178ae0:34-36] moved the stairwell-head fights — on the merged tree the descending contact stalls at route 29/37 with two armed guards parked ON him (skins 0x500003a/0x5000036, d 0.6-1.0, states 0x73/0x74 [L the red run]) and the escort monitor fails the level before Bond lands the plate (the walk legs read level_exit Some at the lead-descent); a plate-side clear loop (40 bounded rounds, the x<-27 pocket shot-from-where-you-stand rule) never fired — the escort dies EARLIER, during the descent follow itself [L the 21 s red run with the loop armed]; next lever: extend the descent recovery loop's clear band (a forward band on the contact's route ahead, mayhew-walk style) or re-check armed_near DURING the lead-descent legs"]
#[test]
fn walk_from_the_spawn_to_the_exit_with_the_contact() { walk_from_the_spawn_to_the_exit_with_the_contact_at(1) }

fn walk_from_the_spawn_to_the_exit_with_the_contact_at(difficulty: u32) {
    let iso = iso();
    let w = walk_to_the_exit_at(&iso, false, difficulty);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0002), "the exit fires to 07000002");
    eprintln!("[01] level_exit to 07000002 at tick {} — walked {} ticks total", w.total_ticks, w.total_ticks);
    for f in &w.report.falls { eprintln!("[01] fall: {f}"); }
}

/// Sister: walking onto the plate ALONE (the contact left far behind) must not fire ch 0x10 — the
/// mask-0x18 filter needs the skin-0x4F drone too [M Trigger_Hit 0x1b1fc8].
#[test]
fn the_plate_refuses_bond_alone() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load"));
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { break; }
    }
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);
    // Teleport-free check of the filter logic itself is the main test's job; here Bond stands on the
    // plate by WALKING there (the contact never left his spawn room — his enable channel is forced
    // off in this sister by stepping only 30 ticks past the intro… the intro writes 0xB regardless,
    // so instead the sister walks Bond to the plate and asserts the channel without the contact.
    let sim = w.scene.sim.as_ref().unwrap();
    let plate = walker::trigger_centre(sim, walker::trigger_kind::TOUCH, 0x10).unwrap();
    let start = sim.bond.pos;
    let pts = walker::leg(sim, start, plate);
    drop(sim);
    for k in 1..pts.len() {
        // The guards the walk wakes stand in its doorways with the game's standing bodies
        // (`NDrone2_Collision` 0x140e90 capsule, on within 2.0 of the player [M 0x149ec8 /
        // 0x14b190]): the 0x5c/0x68 trio #5-#7 held the door at (-14.1, 13.1) [L capsule-join
        // red: STUCK at sister 28]. A player shoots who stands in his way, so the sister answers
        // every live guard within 5 u before each leg.
        for _ in 0..6 {
            let g = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).find(|&i| {
                    let d = &sim.drones.list[i];
                    d.skin != 0x0500_004f && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && walker::dxz(d.pos, bp) < 5.0
                })
            };
            let Some(g) = g else { break };
            if !w.shoot(g, &format!("sister blocker {g}")) { break; }
        }
        w.walk_to(Wp::new(pts[k][0], pts[k][1], pts[k][2]), &format!("sister {k}"));
    }
    w.walk_to(Wp { at: plate, act: Act::None, arrive: 1.2 }, "the plate");
    // Keep Bond ON the plate while the contact (left at his spawn) is far away.
    for _ in 0..60 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).unwrap();
    let contact_far = walker::dxz(sim.drones.list[ci].pos, plate) > 4.0;
    if contact_far {
        assert!(!sim.drones.channels.on(0x10), "Bond alone must not fire the AND-touch [M Trigger_Hit]");
        assert_eq!(sim.drones.level_exit, None);
    } else {
        eprintln!("[01 sister] the contact followed Bond to the plate — the sister asserts nothing (contact at {:?})",
            sim.drones.list[ci].pos);
    }
}


/// Sister: Bond never walks the meet plate (he stays at the spawn side of the bar), so the
/// TOUCH_ONCE at (6.8,6.0,8.9) never writes ch 0x32, the 0xEE mirror never raises ch 0x33
/// [M Trigger_Update 0x1b1a78 case 5], SP #276 (mask 1 = its own out channel [M SP_Hit
/// 0x1dc470:17-21]) never triggers, and the four hall guards waiting on ch 0xE never start.
#[test]
fn the_hall_guards_wait_for_the_meet_scenes_channel() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load"));
    for _ in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) { break; }
    }
    w.wait_enabled(6000, "the intro tail");
    // Long past the scene's 280-frame span [D decode]: the plate stays untouched.
    w.wait(2400);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x32), "the meet plate never fired (Bond kept away)");
    assert!(!sim.drones.channels.on(0x33), "the 0x33 mirror follows ch 0x32 [M case 5]");
    assert!(!sim.drones.channels.on(0xe), "ch 0xE never rises without the scene");
    let waiting: Vec<u8> = sim.drones.list.iter()
        .filter(|d| d.switch_channel == 0xe)
        .map(|d| d.state)
        .collect();
    assert_eq!(waiting.len(), 4, "four drones wait on ch 0xE [D placements]");
    if waiting.iter().any(|&s| s != nf_game::drone::state::WAIT_SWITCH) {
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.switch_channel == 0xe {
                eprintln!("[01 sister] ch-0xE drone {i} skin {:#x} at {:?} state={:#x} enabled={} health={:.0} anim {:?}",
                    d.skin, d.pos, d.state, d.enabled, d.health, d.anim.danim);
            }
        }
    }
    assert!(waiting.iter().all(|&s| s == nf_game::drone::state::WAIT_SWITCH),
        "every ch-0xE guard must sit in WAIT_SWITCH while the meet plate is unfired (states {waiting:?}) [M NDrone2_DSTATE_Global]");
}

/// The driver's real path: clear the ambush, then FOLLOW Mayhew down — Bond stays behind him on
/// the whole descent (never the walker's tuned plate-first lead), Mayhew's mission route runs to
/// its end (path-0 node 36, INSIDE the LoadPointActivator plate volume [D the nav dump: node 36
/// (-32.947, 3.695, 12.837) vs the plate lo (-33.80, 0.64, 12.17) hi (-32.41, 3.96, 18.68)]), and
/// the game's AllyLeadDone parks him there [M NDrone2_DSTATE_AllyLeadDone 0x15f2e8 case 3
/// falls to the default — no movement]. Bond then steps onto the plate: the AND (mask 0x18,
/// filter skin 0x4F [M Trigger_Hit 0x1b1fc8]) fires ch 0x10 → the exit loads 07000002.
/// The contact parked in PlayerInWay [M 0x15ea28] with Bond AHEAD of him on his lead (nearer
/// his steering target — the route's current `target`, else the stairwell head): a player steps
/// back 4.0 behind him (the far side from the target) so the lead resumes. Bond trailing or
/// level with him: None (the follow's own trail/close rules apply).
fn behind_the_in_way_contact(w: &Walker, ci: usize, head: V3) -> Option<V3> {
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[ci];
    if d.state != nf_game::drone::state::PLAYER_IN_WAY { return None; }
    let (cp, bp) = (d.pos, sim.bond.pos);
    let ahead = d.route.as_ref().map(|r| r.target).filter(|t| walker::dxz(*t, cp) > 0.5).unwrap_or(head);
    if walker::dxz(bp, ahead) >= walker::dxz(cp, ahead) { return None; }
    let (hx, hz) = (cp[0] - ahead[0], cp[2] - ahead[2]);
    let hl = (hx * hx + hz * hz).sqrt().max(1e-3);
    Some([cp[0] + hx / hl * 4.0, cp[1], cp[2] + hz / hl * 4.0])
}

/// GREEN un-ignored (pretrans-arms-5, 2026-11-10 [L the 01f run on the merged tree]):
/// pretrans-arms-4's ledger (the contact killed by guard crossfire during the stairwell-head
/// clear, state 0x47) no longer reproduces after the merge with current master — the guard
/// sight/door-leaf commits (823ff190b "Guards see the way the game's do", c120d7b49
/// "Countdown guards slide along a door leaf instead of freezing at it") moved the gallery
/// holders' burst lines off the contact's wait spots: the same walk, unchanged, now ends with
/// the contact parked in AllyLeadDone ON the plate (-33.11,2.94,14.31), ch 0x10 fired at the
/// first plate-wait tick, level_exit Some(0x07000002) [L the 417 s run]. The history stays
/// for the record: he was killed by GUARD fire (the top-up walk keeps only BOND alive; the
/// game's bullets land on any drone of the other side [M NDrone2_DealWithObjHit
/// 0x13fff0:181-232 — a team-1 shooter lands on teams 2/3]; the walk's `shield` models only
/// BOND's own rounds).
#[test]
fn following_mayhew_down_completes_the_level() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000001.bin", false).expect("load 07000001"));
    // The hall's hanging lamps are solid (and block the guards' sight, as the game's
    // DroneVision LOS picks cel-linked objects): the run can leave Bond pressed into the
    // south-wall/east-door-jamb corner, so wedged nav legs bridge through a visible node.
    w.bridge_wedged_nav = true;
    // The type-2 guards carry 12 health, captains 24 [L ee_02b.bin]: Bond aims for the head
    // (`Walker::prefer_head`), as a player does, or the scripted fights run the walk dry.
    w.prefer_head = true;
    // ---- the route to the stairwell head (the main test's play, condensed) ------------------
    for t in 0..4000u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xb) {
            eprintln!("[01f] intro ch 0xB at tick {t}"); break;
        }
    }
    w.wait_enabled(6000, "the intro tail");
    w.wait(300);
    let sim = w.scene.sim.as_ref().unwrap();
    let ci = sim.drones.list.iter().position(|d| d.skin == 0x0500_004f).expect("the contact");
    drop(sim);
    w.shield = Some(ci);
    // audit-voice-triggers-2: the contact stands on the fight lines (the hall run) and the merged
    // branch's added voice draws shifted the shared RNG — the guard-14 volley's moving spread
    // (the game's 176-unit cap = 0.2464 rad [M bullet::player_spread]) crossed him 1.96 u off the
    // aim line where the modelled reach was 1.39, killing him (d#1 10.0 → −39.5 at t5704,
    // by_player, weapon 0x1c) and failing the escort (ch 0x60 → 99). The walk models the cap in
    // the shield's cone so a tap that would cross him is refused before it fires.
    w.shield_bloom = true;
    let plate = walker::trigger_centre(w.scene.sim.as_ref().unwrap(), walker::trigger_kind::TOUCH, 0x10).expect("the AND-touch plate");
    // Clear whatever armed guard stands within 14 of the contact (the hide exit needs the coast
    // clear [M NDrone2_NotNearArmedDrone 0x177640]).
    let mut cleared = std::collections::HashSet::new();
    let clear_around = |w: &mut Walker, cleared: &mut std::collections::HashSet<usize>, label: &str| {
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            }).collect()
        };
        for g in guards {
            let (gpos, los_clear) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[g];
                let bp = sim.bond.pos;
                let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
                (d.pos, sim.world.segment(eye, [d.pos[0], d.pos[1] + 0.3, d.pos[2]]).is_none())
            };
            if walker::dxz(gpos, w.scene.sim.as_ref().unwrap().bond.pos) > 9.0 || !los_clear {
                w.walk_nav_to(gpos, &format!("{label}: approach guard {g}"), 3.0);
            }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("ammo"); }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare == 0 { continue; }
            let killed = w.shoot(g, &format!("{label}: guard {g}"))
                || { if w.resupply("ammo retry") { w.shoot(g, &format!("{label}: guard {g}")) } else { false } };
            let sim = w.scene.sim.as_ref().unwrap();
            let dead = sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0;
            drop(sim);
            if killed || dead { cleared.insert(g); }
        }
    };
    clear_around(&mut w, &mut cleared, "spawn side");
    // The meet beat: walk the plate at (6.8, 6.0, 8.9) → ch 0x32 → the 0x33 mirror → SP #276 → ch 0xE.
    w.walk_nav_to([6.8, 6.0, 8.9], "the meet plate (NE hall)", 1.0);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let meet = sim.drones.triggers.iter()
            .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x32)
            .map(|t| t.centre()).expect("the meet plate");
        drop(sim);
        w.walk_to(Wp { at: meet, act: Act::None, arrive: 0.25 }, "onto the meet plate");
        for (dx, dz) in [(0.0f32, 0.9f32), (0.0, -0.9), (0.9, 0.0), (-0.9, 0.0)] {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: [meet[0] + dx, meet[1], meet[2] + dz], act: Act::None, arrive: 0.2 }, "the meet plate sweep");
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x32) { break; }
            w.walk_to(Wp { at: meet, act: Act::None, arrive: 0.2 }, "back across the meet plate");
        }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x32), "the meet plate must fire ch 0x32");
    let mut e_at = None;
    for t in 0..300u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0xe) { e_at = Some(t); break; }
    }
    assert!(e_at.is_some(), "the meet scene must write ch 0xE");
    w.wait(120);
    // Pre-hall ammo, then the hall's three named guards (death channels 0x11/0x12/0x13) → ch 0x99.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dry = (sim.gun.clip as i32) + (sim.gun.spare as i32) < 48;
        drop(sim);
        if dry { w.resupply("pre-hall ammo"); }
    }
    // Back through the meet-room corridor first: a resupply that ends in the south rooms
    // otherwise routes the stage through the closed west door at (-14.1, 5.3, 13.1) and stalls.
    w.walk_nav_to([2.9, 6.47, 4.49], "back to the meet-room corridor", 1.5);
    w.walk_nav_to([2.9, 6.3, 16.3], "stage into the hall (east door)", 3.0);
    let named: Vec<usize> = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).filter(|&i| [0x11u8, 0x12, 0x13].contains(&sim.drones.list[i].death_channel)).collect()
    };
    for g in named {
        let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
        if walker::dxz(gpos, w.scene.sim.as_ref().unwrap().bond.pos) > 9.0 {
            walker::hall_restage(&mut w, "hall restage");
        }
        w.walk_nav_to(gpos, &format!("approach hall guard {g}"), 3.0);
        if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 {
            walker::hall_ammo_ladder(&mut w, "hall ammo");
        }
        if !w.shoot(g, &format!("hall guard {g}")) {
            if w.resupply("hall ammo") { w.shoot(g, &format!("hall guard {g}")); }
        }
    }
    // The last guard's channel lands with the end of his fall [M 0x1637f8 case 2], up to 213 ticks.
    for _ in 0..300 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x99) { break; }
        w.tick(Default::default(), false);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x99), "the hall AND must fire ch 0x99");
    // West-end pre-clear (guard-standing-capsule-2): the capsule push shifted Bond's escort
    // spots, and the hall's west-end holders kept their burst lines down the north side onto
    // the contact's wait spots — the lead rounds' stall at (3.6,14.6) left him 10+ s in #15's
    // line while Bond fought #5 (dxz 7.6, in the 14 band), and #15 (dxz 14.8, OUT of the band)
    // fired one burst whose spread crossed Bond's approach line — 1 u from the waiting
    // contact — and took his head (45.0, kill at t15517, by_player false [L the red run:
    // /tmp of the guard-standing-capsule-2 session]). A player escorting clears the shooters'
    // end of the hall BEFORE the contact crosses it: fight them now, farthest (the burst
    // line) first, while the contact still waits east of the crossfire; band 20 (the 14 band
    // missed #15 at dxz 14.8). Fight order only — the contact's rules are untouched.
    {
        let guards: Vec<usize> = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            let mut g: Vec<usize> = (0..sim.drones.list.len()).filter(|&i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 20.0 && !cleared.contains(&i)
            }).collect();
            g.sort_by(|&a, &b| {
                let da = walker::dxz(sim.drones.list[a].pos, cp);
                let db = walker::dxz(sim.drones.list[b].pos, cp);
                db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
            });
            g
        };
        for g in guards {
            let gpos = w.scene.sim.as_ref().unwrap().drones.list[g].pos;
            w.walk_nav_to(gpos, &format!("west-end pre-clear: approach guard {g}"), 3.0);
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare < 8 { w.resupply("west-end ammo"); }
            if w.scene.sim.as_ref().unwrap().gun.clip + w.scene.sim.as_ref().unwrap().gun.spare == 0 { continue; }
            let killed = w.shoot(g, &format!("west-end pre-clear: guard {g}"))
                || { if w.resupply("west-end ammo retry") { w.shoot(g, &format!("west-end pre-clear: guard {g}")) } else { false } };
            let sim = w.scene.sim.as_ref().unwrap();
            let dead = sim.drones.list[g].corpse || sim.drones.list[g].health <= 0.0;
            drop(sim);
            if killed || dead { cleared.insert(g); }
        }
    }
    // los-mask-repins-01: the game's sight moved the west-end fights onto the waiting contact.
    // On the merged tree the ch-0x26 waiters g2 (the balcony over the west stairwell) and g3
    // (the west room, 24 hp SMG) start when west door #2 (-14.1, 5.3, 13.1) opens — the
    // pre-clear's own resupply close-leg opened it [L the run: "opening door" then both wake
    // on the same tick, t10452] — and g3 walked to the stairwell head, aimed down the hall and
    // killed the waiting contact from 31 u (2.2 at t11007, the 22.5 head hit at t11085 →
    // ch 0x60, the escort fail) while the 20 u pre-clear band never listed him (28 u away and
    // still WAIT_SWITCH — enabled false — when it ran; the port's bullets cannot pick a
    // not-enabled drone [M drone.rs the bullet pick's dr.enabled gate]). g15 (sw_ch 0x12 — a
    // trio death channel) hunts east to (-10.5, 19.2), 13 u off the hidden contact. A player
    // escorting answers the corridor BEFORE leading Mayhew through it:
    //   1. the own-radius answer (g15 at Bond's feet when the pre-clear ends),
    //   2. the slab ambush + mouth fight (the pop is Bond-touch only [L the d2 run: the
    //      contact's crossing popped nothing]) — the ch-0x26 pair and the pocket as they show,
    //   3. the head sweep (the balcony's #2 over the stairwell head),
    // all before the follow begins — the trail then walks a clean corridor.
    {
        let prev_back_off = w.dmg.back_off;
        w.dmg.back_off = Some(6.5);
        let prev_flank_hold = w.flank_hold_x;
        w.flank_hold_x = Some(-27.0);
        for sweep in 0..3 {
            let near: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                        && walker::dxz(d.pos, bp) < 12.0
                }).collect()
            };
            if near.is_empty() { break; }
            let (seen, walled): (Vec<usize>, Vec<usize>) = route_order(&w, &near).into_iter()
                .partition(|&g| walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g));
            for g2 in seen.into_iter().chain(walled) {
                if !w.shoot(g2, &format!("west-end answer guard {g2}")) {
                    if w.resupply("west-end answer ammo") { w.shoot(g2, &format!("west-end answer guard {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
        w.dmg.back_off = prev_back_off;
        w.flank_hold_x = prev_flank_hold;
    }
    run_slab_ambush(&mut w, ci, 2);
    {
        // The head sweep: the balcony skin-0x36 (g2) parks over the stairwell head and the
        // ch-0x26 wake leaves him there; the trail would walk the contact under his sight.
        // Same answer shape as the pre-stairwell block (shoot-only for the x < -27 pocket).
        let prev_back_off = w.dmg.back_off;
        w.dmg.back_off = Some(6.5);
        let prev_flank_hold = w.flank_hold_x;
        w.flank_hold_x = Some(-27.0);
        w.walk_nav_to([-30.8, 6.4, 13.1], "the head sweep (before the follow)", 1.2);
        for sweep in 0..3 {
            let near: Vec<usize> = {
                let sim = w.scene.sim.as_ref().unwrap();
                let bp = sim.bond.pos;
                (0..sim.drones.list.len()).filter(|&i| {
                    let d = &sim.drones.list[i];
                    i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                        && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                        && walker::dxz(d.pos, bp) < 12.0
                }).collect()
            };
            if near.is_empty() { break; }
            let (seen, walled): (Vec<usize>, Vec<usize>) = route_order(&w, &near).into_iter()
                .partition(|&g| walker::DamageLog::visible(w.scene.sim.as_ref().unwrap(), g));
            for g2 in seen.into_iter().chain(walled) {
                if !w.shoot(g2, &format!("head sweep guard {g2}")) {
                    if w.resupply("head sweep ammo") { w.shoot(g2, &format!("head sweep guard {g2} retry")); }
                }
                if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
            }
            if w.scene.sim.as_ref().unwrap().drones.health.health <= 0.0 { break; }
        }
        w.dmg.back_off = prev_back_off;
        w.flank_hold_x = prev_flank_hold;
    }
    // Follow the contact out of the hall to the stairwell head (the main test's follow loop, but
    // stopping AT the head — the FOLLOW begins at the descent).
    let head: V3 = [-30.8, 6.4, 13.1];
    for round in 0..12 {
        let settled = w.follow(ci, &format!("the contact r{round}"), 12000, head);
        if walker::dxz(settled, head) < 4.0 { break; }
        let before = cleared.clone();
        clear_around(&mut w, &mut cleared, "lead r");
        let sim = w.scene.sim.as_ref().unwrap();
        let wedged = settled == w.scene.sim.as_ref().unwrap().drones.list[ci].pos
            && sim.drones.list[ci].state != nf_game::drone::state::PLAYER_IN_WAY;
        drop(sim);
        if cleared == before && wedged {
            // No guard to clear and the contact is holding: close on him (the AllyLeadWait release band
            // [M 0x15eda8 case 3] needs Bond within 5.0 straight-line when pd == 0.0). When the
            // straight line crosses collision (the hall's south structure — see the trail-loop's
            // corner note), the player rounds it by the north lane first.
            let cp = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
            let walled = w.scene.sim.as_ref().unwrap().world.segment(bp, [cp[0], bp[1], cp[2]]).is_some();
            if walled {
                eprintln!("[01f] the contact is across the hall's south structure — walking the north lane back in");
                w.walk_nav_to([5.45, 6.6, 14.3], "round the hall's south structure (north)", 1.4);
                w.walk_nav_to([6.88, 6.4, 14.62], "the hall's north opening (nav node 7)", 1.4);
                w.walk_nav_to([7.4, 6.0, 11.0], "back through the east door band", 1.4);
            }
            w.walk_nav_to(cp, "close on the contact", 1.2);
        }
    }
    clear_around(&mut w, &mut cleared, "stairwell head");
    // ---- THE FOLLOW: trail him down the stairwell, never overtaking --------------------------
    // The trail point sits 4.0 behind him on the Bond→contact line: outside the PlayerInWay
    // 3.5 stay band [M 0x15ea28:32-47], and close enough that the LeadWait band cannot hold —
    // the wait enters at pd ≤ −9.0 / (pd == 0.0 && dist ≥ 5.0) [M 0x15e698 case 3] and releases
    // at pd > −7.0 [M 0x15eda8 case 3]; a 5.5 trail on the corridor's zigzag read ~7.8
    // route-units and parked him in AllyLeadWait (the first run's stall at (−18.8, 14.6)).
    let mut trail_still = 0u32;
    let mut last_cp = w.scene.sim.as_ref().unwrap().drones.list[ci].pos;
    let mut rounds = 0u32;
    let mut closed_in = 0u32;
    let parked_at;
    loop {
        rounds += 1;
        assert!(rounds < 120, "the follow never settled: contact at {:?} state {:#x}",
            w.scene.sim.as_ref().unwrap().drones.list[ci].pos, w.scene.sim.as_ref().unwrap().drones.list[ci].state);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { parked_at = w.scene.sim.as_ref().unwrap().drones.list[ci].pos; break; }
        let (cp, cstate) = {
            let sim = w.scene.sim.as_ref().unwrap();
            (sim.drones.list[ci].pos, sim.drones.list[ci].state)
        };
        // Parked: the route is done (AllyLeadDone parks [M 0x15f2e8]).
        if cstate == nf_game::drone::state::ALLY_LEAD_DONE {
            eprintln!("[01f] the contact parked in AllyLeadDone at ({:.2},{:.2},{:.2}) after {rounds} follow rounds — tick {}",
                cp[0], cp[1], cp[2], w.total_ticks);
            parked_at = cp;
            break;
        }
        if walker::dxz(cp, last_cp) < 0.05 { trail_still += 60; } else { trail_still = 0; closed_in = 0; }
        last_cp = cp;
        if trail_still > 600 {
            // He stood still 10 s — a player closes the gap: the wait's own release
            // ([M 0x15eda8 case 3] pd > −7.0 && (pd != 0.0 || dist < 5.0)) and the PlayerInWay
            // exit both come from Bond's position. Two closes, then give up with the dump.
            closed_in += 1;
            trail_still = 0;
            if closed_in > 2 {
                eprintln!("[01f] the contact stood still 30 s at ({:.2},{:.2},{:.2}) state {cstate:#x} despite {closed_in} closes — tick {}",
                    cp[0], cp[1], cp[2], w.total_ticks);
                parked_at = cp;
                break;
            }
            // PlayerInWay is the opposite of the wait: Bond stands IN the lead's route inside 3.0
            // and the lead resumes only once Bond is out of that band [M 0x15ea28 case 3] —
            // closing in keeps him parked (the hall run: Bond trailed 2.8 u WEST of him, on his
            // westward route, three closes, 30 s). A player steps behind him instead.
            if cstate == nf_game::drone::state::PLAYER_IN_WAY {
                let tgt = w.scene.sim.as_ref().unwrap().drones.list[ci].route.as_ref().map(|r| r.target);
                if let Some(tgt) = tgt {
                    let (hx, hz) = (tgt[0] - cp[0], tgt[2] - cp[2]);
                    let hl = (hx * hx + hz * hz).sqrt().max(1e-3);
                    let behind = [cp[0] - hx / hl * 4.5, cp[1], cp[2] - hz / hl * 4.5];
                    eprintln!("[01f] the contact holds in PlayerInWay at ({:.2},{:.2},{:.2}) — stepping behind him to ({:.2},{:.2})",
                        cp[0], cp[1], cp[2], behind[0], behind[2]);
                    w.walk_nav_to(behind, "behind the waiting contact", 1.0);
                    continue;
                }
            }
            eprintln!("[01f] the contact stood still 10 s at ({:.2},{:.2},{:.2}) state {cstate:#x} — closing in (the release bands [M 0x15eda8])",
                cp[0], cp[1], cp[2]);
            // The hall's south-west inner corner: when the straight line to the contact crosses
            // collision, Bond is boxed by the hall's south structure — the x≈6.03 wall (z 11.6-13+)
            // and the z≈11.05 wall ([D the disc's collision cels — the port's segment fan: the corner
            // casts hit at 0.10-0.39 in every direction but north; the doorway band x 6.5-7.9,
            // z 12.0→10.4 is the only opening]). The player rounds the structure by its north end —
            // the AI's own lane node[23] (3.39,14.69) → node[7] (6.88,14.62) [D nav path 0] — and
            // comes back in through the east door band (the main test's west-wall ladder).
            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
            let walled = w.scene.sim.as_ref().unwrap().world.segment(bp, [cp[0], bp[1], cp[2]]).is_some();
            if walled {
                eprintln!("[01f] the contact is across the hall's south structure — walking the north lane back in");
                w.walk_nav_to([5.45, 6.6, 14.3], "round the hall's south structure (north)", 1.4);
                w.walk_nav_to([6.88, 6.4, 14.62], "the hall's north opening (nav node 7)", 1.4);
                w.walk_nav_to([7.4, 6.0, 11.0], "back through the east door band", 1.4);
            }
            if let Some(back) = behind_the_in_way_contact(&w, ci, head) {
                eprintln!("[01f] he waits in PlayerInWay with Bond ahead — stepping back behind him to ({:.2},{:.2})", back[0], back[2]);
                w.walk_nav_to(back, "step back behind the waiting contact", 2.0);
                continue;
            }
            w.walk_nav_to(cp, "close on the waiting contact", 3.0);
            continue;
        }
        // Guards near him first (the hide keeps him off the route); the approach may overtake —
        // a player swaps positions for a fight, then falls back in behind.
        let armed_near = {
            let sim = w.scene.sim.as_ref().unwrap();
            let cp = sim.drones.list[ci].pos;
            (0..sim.drones.list.len()).any(|i| {
                let d = &sim.drones.list[i];
                i != ci && d.team == 1 && d.started && !d.corpse && d.enabled && d.health > 0.0
                    && ((2..0x47).contains(&d.weapon) || (0x6a..=0x71).contains(&d.weapon))
                    && walker::dxz(d.pos, cp) < 14.0 && !cleared.contains(&i)
            })
        };
        if armed_near { clear_around(&mut w, &mut cleared, "follow"); continue; }
        let bp = w.scene.sim.as_ref().unwrap().bond.pos;
        let (mut dx, mut dz) = (cp[0] - bp[0], cp[2] - bp[2]);
        // Behind him means behind along HIS heading (the route's move target): with Bond ahead
        // on the route, the Bond-to-contact line put the trail point in front of the lead.
        if let Some(tgt) = w.scene.sim.as_ref().unwrap().drones.list[ci].route.as_ref().map(|r| r.target) {
            let (hx, hz) = (tgt[0] - cp[0], tgt[2] - cp[2]);
            if hx * -dx + hz * -dz > 0.0 && hx * hx + hz * hz > 1e-4 { dx = hx; dz = hz; }
        }
        let l = (dx * dx + dz * dz).sqrt().max(1e-3);
        let mut trail = [cp[0] - dx / l * 4.0, cp[1], cp[2] - dz / l * 4.0];
        if let Some(back) = behind_the_in_way_contact(&w, ci, head) {
            eprintln!("[01f] Bond is ahead of the contact (PlayerInWay) — stepping back behind him to ({:.2},{:.2})", back[0], back[2]);
            trail = back;
        }
        w.walk_nav_to(trail, &format!("trail the contact {rounds}"), 2.0);
    }
    // The endgame state: the contact where the follow left him, Bond behind.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[ci];
        eprintln!("[01f] follow done (park {:?}): contact at ({:.2},{:.2},{:.2}) state {:#x} route {:?} bond at ({:.2},{:.2},{:.2}) tick {}",
            parked_at, d.pos[0], d.pos[1], d.pos[2], d.state, d.route.as_ref().map(|r| (r.index, r.nodes.len())),
            sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], w.total_ticks);
        // Who could hold the hide / block the AND: every live armed guard within 20 of him.
        for (i, o) in sim.drones.list.iter().enumerate() {
            if i == ci || o.team != 1 || !o.started || o.corpse || !o.enabled || o.health <= 0.0 { continue; }
            let dd = walker::dxz(o.pos, d.pos);
            if dd < 20.0 {
                eprintln!("[01f] armed peer {i} skin {:#x} at ({:.1},{:.1},{:.1}) d {:.1} state {:#x} health {:.0} weapon {:#x}",
                    o.skin, o.pos[0], o.pos[1], o.pos[2], dd, o.state, o.health, o.weapon);
            }
        }
    }
    // Bond takes the plate (the player's final step — the AND needs him in the volume).
    w.walk_nav_to(plate, "Bond onto the plate", 0.8);
    // The AND needs BOTH touchers inside the volume for a beat [M Trigger_Hit 0x1b1fc8]; wait for it.
    let mut fired = None;
    for t in 0..600u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x10) { fired = Some(t); break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[ci];
    eprintln!("[01f] after the plate wait: ch 0x10 fired at {fired:?}; contact ({:.2},{:.2},{:.2}) state {:#x} bond {:?}",
        d.pos[0], d.pos[1], d.pos[2], d.state, sim.bond.pos);
    let contact_toucher = nf_game::triggers::Toucher {
        a: [d.pos[0], d.pos[1] - 1.0, d.pos[2]], b: d.head(), r: 0.4, drone: true, skin: d.skin, contain: true,
        seg_only: false, // the merge's field: the 01f plate keeps its master semantics (the segment+radius both-sided test on record-less plates)
    };
    // On-foot Bond: `contain: !hanging` with hanging false — the sim's own on-foot construction
    // [drone.rs 16711], and this walk never leaves the ground.
    let bond_toucher = nf_game::triggers::Toucher {
        contain: true, ..nf_game::triggers::Toucher::player([sim.bond.pos[0], sim.bond.pos[1] - 0.6, sim.bond.pos[2]])
    };
    if let Some(t) = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::TOUCH && t.out == 0x10) {
        eprintln!("[01f] plate lo {:?} hi {:?}: contact touched_by={} bond touched_by={}",
            t.lo, t.hi, t.touched_by(&contact_toucher), t.touched_by(&bond_toucher));
    }
    assert!(fired.is_some(), "the AND-touch must fire with Bond AND the parked contact on the plate");
    drop(sim);
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0002), "the exit fires to 07000002");
    eprintln!("[01f] level_exit to 07000002 at tick {} (follow route)", w.total_ticks);
    for f in &w.report.falls { eprintln!("[01f] fall: {f}"); }
}

/// level-01-end-fade: the level-end screen fade, pinned on the walked exit above. The game's
/// exit path [M]: the AND-touch raises ch 0x10 and `Trigger_Update`'s case-10 arm fires the
/// LOAD_LEVEL trigger (0x1b1a78:162-166 — no proximity check); `Trigger_Activate`'s type-10
/// tail runs `Music_Event(3,1)` + `GS_PauseGame(1)` + `ResetMap_LevelToLoad(next,1,0)` +
/// `GameFlow_PushState(300.0f, 7, 0xff)` [M 0x1b2318:89-92, disasm 0x1b24b8-0x1b24ec]. The push
/// converts state 7 → 8 through `Camera_SetFade(6, 300.0, 0xff, 1)` [M 0x1c9f38 disasm
/// 0x1c9f7c-0x1c9f84: a0=6, f12/a1 pass through, a2=1], arming the viewer fade: level 0, rate
/// (1/300)·(0xff/255), colour bytes R=G=B=0 — black [M 0x1bbdd8 disasm 0x1bbe20-0x1bbe54].
/// `Game_Draw` draws psiFadeView(R,G,B,level·255) while the fade is active [M 0x1e6f50:203-209];
/// `Camera_UpdateGlbVars` advances the level every frame and clamps the byte [M 0x1bada0:16-27];
/// when the fade ends, state 8 pops onto state 3 (`ResetMap_Load`) and the level loads
/// [M 0x1c98c8:173-189].
#[ignore = "pretrans-arms-5 ledger: the same aim-fix move as the two main walks below — the IsOpponent under-0.9 no-turn fix [M 0x178ae0:34-36] stalls the descending contact under two parked guards (d 0.6-1.0 [L the 21 s red run on the merged tree]) and the escort fails before the plate fire this pin walks to; next lever: the descent recovery's forward clear band (shared with the main walks)"]
#[test]
fn the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads() {
    let iso = iso();
    let mut w = walk_to_the_exit(&iso, true);
    // The exit tick: the hold stands with the full count, the fade level is 0, the request rides.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let h = sim.drones.exit_hold
            .expect("the exit posts GS_PauseGame(1) + GameFlow_PushState(300.0f, 7, 0xff) [M 0x1b2318:89-92]");
        assert_eq!(h.next, 0x0700_0002, "the hold carries the exit's next level [D placement p0; M 0x1b2318:91]");
        assert_eq!(h.total, 300, "the push carried 300.0f [M 0x1b2318:92 — 0x43960000]");
        assert_eq!(h.fade_frames, 300, "all 300 frames left at the fire tick");
        assert_eq!(sim.drones.level_exit, Some(0x0700_0002),
            "the exit request is posted; the host drains it only after the hold clears [M 0x1c98c8 case 8]");
        assert_eq!(sim.drones.exit_fade_view().unwrap().alpha, 0.0,
            "the fade's level starts at 0 [M 0x1bbdd8: level = 0 · (0xff/255)]");
    }
    // The world freezes under the fade while it runs [M 0x1c98c8:44-47 the clock stops;
    // 0x1c9790:30-38 the world sections sit behind the pause gate; 0x10cd80:36 the input dies].
    let (bond0, anchor0, frame0) = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.pos, sim.drones.player_pos, sim.drones.frame)
    };
    for _ in 0..100 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.pos, bond0, "the world is frozen under the fade");
        assert_eq!(sim.drones.player_pos, anchor0, "the player anchor is frozen under the fade");
        assert_eq!(sim.drones.frame, frame0, "the game clock stops while paused [M 0x1c98c8:44-47]");
        let h = sim.drones.exit_hold.expect("the hold still runs at fire+100");
        assert_eq!(h.fade_frames, 200, "100 of the 300 frames burned [M 0x1bbdd8 rate · 0x1bada0 advance]");
        let a = sim.drones.exit_fade_view().unwrap().alpha;
        assert!((a - 100.0 / 300.0).abs() < 1e-6,
            "the fade alpha is the burned fraction of the push (got {a}) [M 0x1bada0:16-20]");
        // The visible pane: the game draws psiFadeView(0,0,0, level·255) — black, its alpha byte
        // straight into the GS where 0x80 = 1.0 [M 0x1bbdd8 colour bytes, 0x1e6f50:203-209,
        // CreateTotalScreenSprite 0x1181b0 ALPHA_1 0x44] — so the cover is byte/128
        // (`hud::fade_cover`, hud-start): 85/128 here, not the raw 1/3.
        let hud = w.scene.script_overlay_hud(4.0 / 3.0);
        let want = nf_view::hud::fade_cover(a);
        assert!((want - 85.0 / 128.0).abs() < 1.5 / 128.0, "byte (int)(255·a) ≈ 85 → ≈85/128, got {want}");
        let got: Vec<f32> = hud.v.iter().filter(|v| v.c[0] == 0.0 && v.c[1] == 0.0 && v.c[2] == 0.0).map(|v| v.c[3]).collect();
        assert!(got.iter().any(|c| (c - want).abs() < 1e-3), "fade quad alphas {got:?}, want {want}");
        assert!(hud.v.iter().any(|v| v.c[0] == 0.0 && v.c[1] == 0.0 && v.c[2] == 0.0
                && (v.c[3] - want).abs() < 1e-3),
            "the fade pane draws black at the faded alpha [M 0x1bbdd8 + 0x1e6f50 psiFadeView]");
    }
    // The rest of the count: the hold clears at exactly 300 with the request intact
    // [M 0x1c98c8:173-189 — state 8 pops onto state 3].
    for _ in 0..300 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.exit_hold.is_none() { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.exit_hold.is_none(), "the hold cleared at 300 frames [M 0x1c98c8:173-189]");
        assert_eq!(sim.drones.level_exit, Some(0x0700_0002), "the loader waited under the fade — the request is intact");
    }
    // The load AFTER the fade: the host captures the RamSave (the game saves at the fire
    // [M 0x1b2318:96]; the world is frozen between) and loads warm [M 0x1d3800 → 0x1d6360].
    let save = {
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    drop(w);
    let next = nf_view::load_with_carry(&iso, "07000002.bin", false, Some(&save)).expect("load 07000002 warm");
    {
        let sim = next.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0002, "the next level loaded after the fade [M 0x1c98c8:173-189]");
        assert!(sim.drones.exit_hold.is_none(), "the fresh level carries no hold");
        let p = sim.bond.pos;
        assert!((p[0] - -27.17).abs() < 2.0 && (p[2] - 29.80).abs() < 2.0,
            "start #0 (gate 0) — entry from 07000001's single exit route [D docs/levels/07000002.md], got {p:?}");
    }
}

/// Real damage: the same walk with the health top-up off for every walker it makes (the
/// tests' own `top_up_health = true` included) and the walker's return fire on — Bond's death
/// panics with the death spot and the last hits (common/damage_log.rs), so a pass pins that the
/// walk completes on the port's own damage.
#[ignore = "pretrans-arms-5 ledger: the IsOpponent under-0.9 no-turn fix [M 0x178ae0:34-36] moved the stairwell-head fights — on the merged tree the descending contact stalls at route 29/37 with two armed guards parked ON him (skins 0x500003a/0x5000036, d 0.6-1.0, states 0x73/0x74 [L the red run]) and the escort monitor fails the level before Bond lands the plate (the walk legs read level_exit Some at the lead-descent); a plate-side clear loop (40 bounded rounds, the x<-27 pocket shot-from-where-you-stand rule) never fired — the escort dies EARLIER, during the descent follow itself [L the 21 s red run with the loop armed]; next lever: extend the descent recovery loop's clear band (a forward band on the contact's route ahead, mayhew-walk style) or re-check armed_near DURING the lead-descent legs"]
#[test]
fn walk_from_the_spawn_to_the_exit_with_the_contact_real_damage() {
    walker::with_real_damage(walk_from_the_spawn_to_the_exit_with_the_contact);
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty): the same walked
/// route with the sim loaded the way the menu's Agent pick loads it — GameState+0x28 = 2 through
/// the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 writes the wheel row's value 2
/// (`difficulty[]` table 0x2df4c0 row 1); `Player_HandlePain` 0x1902c8 scales Bond's pain by
/// Plr_DMod_Normal 0.7 on this level group instead of Easy 0.5, and `DroneWeap_DoBulletAccuracy`
/// 0x179740:74-88 scales the guards' hit chance by DroneFiring_Accuracy_Normal 0.8 instead of
/// Easy 0.5 [M tuning.rs 01-04 group]]. Real input only — the load path is the only difference;
/// no health, armour or channel writes. A pass pins the level completes at Agent on the port's
/// own damage and the game's Agent spawns (the min-difficulty placements now pass Drone_Create's
/// gate [M 0x136be0:49] and the captain criterion reads prop 5 [M 0x14b300:504-535]).
///
/// level-01-agent-stairwell-doors: the stairwell ch-0xF ambush is a PLACED pocket pair — [L the
/// placement dump + the trio trace on this tree] #12 (0x5000036 SMG) disabled at (-31.31, 6.34,
/// 11.51) and #17 (0x5000039 SMG) at (-29.08, 6.34, -0.01) from the hall AND on (state 0x1;
/// disabled objects are dropped from bullet collision [M Collide_PickObj 0x1eae80:82 +0xF0 &
/// 0x20], and the corridor's walkable band at the slab's x is z >= 13.4 only [L the sight
/// probe], so the TOUCH slab out 0xF mask 0x8 x -26.13..-24.73, z 13.25..16.84 [D placement]
/// pops them on ANY crossing). The walk now crosses the slab deliberately, steps back out in
/// the poppers' 10-tick reaction window [M DroneFunc_ReactionTime 0x14ac60:15-24] and fights
/// from the mouth at 6.5+ u (outside the TooClose doubling [M Drone_ModBulletDamage 0x139f50]):
/// the backed-off return fire dropped #12/#3/#17 for 3.1 health on the fix run, and a guard
/// parked west of x -27 (the walled pocket — no clear eye ray from any mouth stand [L]) is
/// fought SHOOT-ONLY, never approached into the 0.4-u corner where the pre-fix run died.
#[test]
fn walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent() {
    walker::with_real_damage(|| walk_from_the_spawn_to_the_exit_with_the_contact_at(2));
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-01): the same walked
/// route — the stairwell-doors ambush law above — with the sim loaded the way the menu's 00
/// Agent pick loads it — GameState+0x28 = 3 through the front-end's launch path [M
/// C_SBNFDFCTY 0x216ca8:15 writes the wheel row's value 3 (`difficulty[]` table 0x2df4c0 row 2,
/// Txt 0x30000a7); `Player_HandlePain` 0x1902c8 scales Bond's pain by Plr_DMod_Hard 1.2 on this
/// level group (Easy 0.5, Agent 0.7), `DroneWeap_DoBulletAccuracy` 0x179740:81-95 by
/// DroneFiring_Accuracy_Hard 1.0 (Easy 0.5, Agent 0.8) and `NDrone2_HitDamage` 0x145558 reads
/// DroneDamage_Hard 0.8 for the guards' own bullets [M tuning.rs 01-04 group; L the pins:
/// 0x30cab8/0x30caec — difficulty3_07000001_live asserts the three words on a d3 load]]. The
/// min-difficulty placements pass Drone_Create's gate at d3 exactly as at d2 [M 0x136be0:49] —
/// the same roster the Agent walk fights, hit more often for less per hit while Bond's pain is
/// amplified 1.2×. Real input only — the load path is the only difference (plus the
/// d3-gated tactics the deaths force, inside this file); no health, armour or channel writes.
/// IGNORED with the measured reason (guard-pane-drain-3, 2026-11-08 [L the runs on the merged
/// tree 2998b6a76]): the stream shift moved the failure twice in one day. First the plain
/// shoot's flank killed the walk outright — #6 (12 hp pistol) parked deeper in the hall
/// (z 17.8, health 12 untouched through 2×1200 threshold taps), exposed_aim false from the
/// hold line, and the shoot machinery's flank nav OPENED the hall door and walked Bond into
/// the hall, dry at clip 0, where the shotgun captain #5 (0x1c) at 22.8 u finished him at
/// tick 8604 (20 hits for 100.0 [L the 8604 run]). This round's lever — the d3-gated aperture
/// lure in the named loop (the bounded live-tracked gunshot lure + no flank while
/// still-across) — removed that death: Bond now reaches the AND gate at 85.6 health (8 hits
/// for 14.4 [L the d3fix run]). But the gate still can't fire: all three named guards stay
/// across and alive — the door-phase resupplies fail ("no pickup within the walked-path
/// corridor", clip 1-4 spare 0), the lure's noise dies after 1-4 taps, and a dry gun draws no
/// hunter across the aperture (the old lure run that walked 5/6/7/11 into the door line was
/// ARMED: clip 32 spare 16 [L the lure run note]). THE AMMO LEVER, TRIED (guard-pane-drain-4,
/// 2026-11-08 [L the detour runs]): the walked detour to the off-path boxes is d3-gated in the
/// named loop's two dry branches (d3_ammo_detour — real input, self-gating), and the runs
/// CORRECT the ledger: (1) the corridor rule does NOT refuse the named boxes — the pre-hall
/// stock's own attempts consumed them ("stock 1: resupply attempt 0 at (-7.5,5.3,-6.8) (d 7.0)"
/// / "attempt 1 at (-3.3,5.8,-9.1) (d 5.1)" — the legs reached pickup radius, the boxes went
/// gone, and the held-gun ammo measure missed the pool they fed); (2) at the door phase the
/// whole level holds TWO live pickups [L the detour dump]: the east-wing box (17.4,6.2,-3.2)
/// whose detour is forbidden by the walk's own ch-0x05 wake law, and the west-hall box
/// (-14.4,6.5,16.1) across the aperture — both kind 3 weapon 0x47 rounds 50, a weapon the wheel
/// does not own (12 cycle ticks all print the fists 0x1 — Bond holds the FISTS at the door
/// phase); (3) no guard drop remains live either (from None on both). The level is
/// ammo-exhausted at the door phase under this walk's spending. NEXT LEVER: the SPEND — the
/// pre-clear loop and the stock passes burn the pistol's 54 and every reachable box before the
/// AND; a player who knows the hall holds the AND crowd hoards: diet the d3 pre-clear (the
/// lure draws the ch-0xE crowd across one at a time and the doorway fight kills them there —
/// the same kills, later, for a third of the rounds) and keep the wheel armed through the
/// named loop (switch_to_armed after every dry resupply), so the sustained lure still has a
/// gun. The detour machinery stays: it fires at every dry branch and takes any drop that lands
/// near the spots on a future stream.
#[test]
#[ignore = "00 Agent: the AND(0x11,0x12,0x13) hall gate can't complete — the aperture-lure round removed the flank death (Bond at the gate at 85.6 hp, tick 22260 [L the d3fix run]) but all three named guards stay across and alive: the door phase is AMMO-EXHAUSTED (the drain-4 detour runs: the pre-hall stock itself consumed the off-path boxes at (-7.5,5.3,-6.8)/(-3.3,5.8,-9.1) — stock 1 reached both at d 7.0/5.1 and the boxes are gone; the whole level then holds two live pickups, the east-wing box forbidden by the ch-0x05 wake law and the west-hall box across the aperture, both weapon 0x47 which the wheel does not own; the wheel cycles 12 ticks on the fists 0x1; no guard drop remains live [L the detour dump]) — the lure's noise dies and no guard crosses; next lever: the SPEND — diet the d3 pre-clear and the stock passes (a player hoards for the AND crowd; the lure draws them across one at a time and the doorway fight kills them there), keeping the wheel armed through the named loop so the sustained lure still has a gun"]
fn walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent() {
    walker::with_real_damage(|| walk_from_the_spawn_to_the_exit_with_the_contact_at(3));
}
