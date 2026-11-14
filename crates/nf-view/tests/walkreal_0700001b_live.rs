//! walkreal-c: 0700001b (the space station) walked with the player's own input — no `bond.pos`,
//! substate or channel writes after the load in the full walk (`walk_arrival_to_drakes_defeat`).
//!
//! The station is a zero-g level from the start: the p3 == 2 spawn enters substate 8 [M
//! Player_Init 0x1910f8 :160-165] and nothing in ACTION.ELF leaves it on this level. The mission
//! never needs the shaft's west side: the first wave's death raises the launch order, the eight
//! console screens are shot from floats inside the hangar, the disarm chain opens the fight gate
//! and Drake closes in through the float (the walk's doc comment has the [D]/[M] chain).
//!
//! `C_CHCHZEROG_Handler` 0x216388 is a cheat-menu checkbox handler (the C_CHCH* family the menu
//! dispatch `Handler_HandleMessage` 0x2029e0 drives — MUSIC, FLY, DRONES, BLIND, WEAP, HEALTH, HUD,
//! DEBUG, COORDS, ZEROG) whose `param_4 == 0x4b` is the menu message id, not a switch channel:
//! the disc's switch-channel-75 writes move no substate. The float tests below fly from the
//! load's own substate 8 (`float_to`); the deck legs still force the ground substate ([S]
//! premise) — they pin port mechanisms (the deck route, the iris doors), not the mission route.
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance) — except `walk_arrival_to_drakes_defeat_on_real_health`,
//! which plays the same route on real health, and the `_real_damage` variants (no top-up, a death
//! panics). Opt-in via NF_ISO (the slim ISO is enough).
//!
//! The level look: every zero-g shot here is taken with the forward row level (yaw only, the eye
//! put at the target's height on the lift thruster, the stabiliser [M Player_ZeroG 0x192ab0:
//! 163-199] pressed if a tilt creeps in). The port's camera in zero-g is the walk's scalar yaw
//! (mirrored off the forward row) and the walk's scalar pitch, which the zero-g branch never
//! moves (0 from the entry); the shot rides the forward row (`Bond::look`). A pitched forward row
//! therefore shoots off the view, under either composition of the look rotation (the world-frame
//! `m·R` the port has now, or the body-frame `R·m` [M Concat 0x1e4a90 + Mat_CopyRot 0x1e3370]):
//! only the level shot lands where the view looks. The cost: a target above or below must be met
//! at its own height, so the band the later waves float in (y 2.5–8.7, above the deck) is fought
//! from inside it or not at all — the full walks wait those waves out under the deck.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::{Input, SS_WALK, SS_ZEROG_8};
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Leg 1 (the old test's hub crossing, solved): the arrival NIS plays out, the objectives show,
/// and Bond WALKS the mapped east-deck route from the platform to the slit's south edge — the
/// leg the first run fell off (it beelined west off the platform's west edge into the pit).
#[test]
fn walk_arrival_and_the_east_deck_to_the_slit_edge() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — NOT a game assertion: the p3 == 2 spawn enters substate 8 [M Player_Init
    // 0x1910f8 :160-165, now ported], but no decompile path back to a ground substate after the
    // arrival NIS is pinned and the port wires no substate-changing handler. This leg pins the
    // port's WALK route, so the substate is forced back to the ground path.
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "'Disable the missiles' shows at once (enable 0 [L], the state-0 arm [M 0x1827d8:41-54])");
        assert_eq!(sim.drones.mission.states[23][1], 1, "'Defeat Drake' hides until ch 0x09 [L record 1b obj 2]");
    }

    // The mapped route: south off the platform onto the deck, then west along z ≈ −1
    // (probe1b_route.rs's band map: the deck x 8–28 at z −8..2, the slit at z ≈ 0).
    assert!(w.try_walk_to(walker::Wp::new(20.0, -1.9, 1.0).r(1.2), "the platform south edge"),
        "the platform's south edge is walked");
    assert!(w.try_walk_to(walker::Wp::new(17.0, -1.9, -1.0).r(1.2), "the deck south-west"),
        "the deck south-west is walked");
    assert!(w.try_walk_to(walker::Wp::new(13.0, -1.9, -1.0).r(1.2), "the deck mid-west"),
        "the deck mid-west is walked");
    assert!(w.try_walk_to(walker::Wp::new(10.0, -1.9, -1.0).r(1.2), "the deck west"),
        "the deck west is walked");
    assert!(w.try_walk_to(walker::Wp::new(9.0, -1.9, -1.4).r(1.2), "the slit edge"),
        "the slit's south edge is walked");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] < 11.0 && p[0] > 6.0, "Bond reached the deck's west edge ({p:?})");
        assert!(p[1] > -3.5, "Bond never fell into the pit (y {:?})", p[1]);
        // Grounded is NOT asserted at the last metre: the slit's mouth (z ≈ 0) is the float
        // zone — leg 2 walks into it and the level carries Bond (the walk logs show grounded
        // arrivals on every deck node up to (10.94,-1.46,-0.25)).
    }
}

/// Leg 2: the slit float. Walking west into the slit (under the legs' forced ground substate),
/// Bond is carried west over the void without a fall.
#[test]
fn the_slit_float_carries_bond_west_without_a_fall() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (the p3 == 2 spawn enters
    // substate 8 [M 0x1910f8 :160-165]; this leg pins the port's WALK route).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // Into the slit: walk west along z ≈ −0.5 → 0. The float takes over past the deck edge.
    let _ = w.try_walk_to(walker::Wp::new(8.6, -1.9, -0.6).r(1.2), "the slit mouth");
    let mut min_x = w.scene.sim.as_ref().unwrap().bond.pos[0];
    let mut fell = false;
    for _ in 0..900u32 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        min_x = min_x.min(p[0]);
        if p[1] < -6.0 { fell = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(!fell, "Bond never fell into the pit crossing the slit (min x {min_x:.2})");
        assert!(min_x < 7.0, "the float carried Bond west past the deck edge (min x {min_x:.2})");
        assert!(p[1] > -3.0, "the float holds Bond at the deck band (y {:.2})", p[1]);
    }
}

/// The game's own hit volumes for the walk's aim: the `Data` view the drone pass hands
/// `Scripts::held_hit_targets` (the `SP_GetHitDamage` 0x1dc5f8 read set [M]).
fn data_of(sim: &nf_view::scene::Sim) -> nf_game::drone::Data<'_> {
    nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    }
}

/// The float's thrusters toward `at`: the strafe / lift / forward inputs are the body-axis
/// components of the offset (the zero-g move is `pos += local·rows` [M 0x193600:284-295]), so a
/// proportional input flies straight at the point without turning (the 0.04 ease damps it).
fn thrust_to(sim: &nf_view::scene::Sim, at: [f32; 3]) -> Input {
    let p = sim.bond.pos;
    let d = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
    let m = sim.bond.zero_g_rot;
    let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
    Input {
        strafe: (dot(m[0]) * 0.15).clamp(-1.0, 1.0),
        lift: (dot(m[1]) * 0.15).clamp(-1.0, 1.0),
        forward: (dot(m[2]) * 0.15).clamp(-1.0, 1.0),
        ..Default::default()
    }
}

/// Fly the float from the load's own zero-g substate 8 (the p3 == 2 start [M Player_Init 0x1910f8
/// :160-165]) along the east deck to the slit mouth — the same nodes the deck legs walk, with
/// `thrust_to`'s thrusters only, a little above the deck (the float's body does not settle onto
/// it the way the walk's feet do). Asserts the substate stays 8 the whole way (nothing on the
/// route changes it); arrival is judged in the deck plane.
fn float_to_slit_mouth(w: &mut Walker) {
    for at in [[20.0f32, -1.0, 1.0], [17.0, -1.0, -1.0], [13.0, -1.0, -1.0], [10.0, -1.0, -1.0], [8.6, -1.0, -0.6]] {
        let mut reached = false;
        for _ in 0..1200u32 {
            let input = thrust_to(w.scene.sim.as_ref().unwrap(), at);
            w.tick(input, false);
            let sim = w.scene.sim.as_ref().unwrap();
            assert_eq!(sim.bond.substate, SS_ZEROG_8, "the float holds the load's substate 8 [M 0x1910f8 :160-165]");
            let p = sim.bond.pos;
            if walker::dxz(p, at) < 0.6 && (p[1] - at[1]).abs() < 1.0 { reached = true; break; }
        }
        assert!(reached, "the float reaches {at:?} (at {:?})", w.scene.sim.as_ref().unwrap().bond.pos);
    }
}

/// The station Bond floats at to shoot console `k`'s screen (x and z; the height is the screen's,
/// see `shoot_consoles`): 7/8 from x 46, 1/2 from x 20, the rest from x 32 — the harness's choice
/// of where to fly; a station whose level shot is blocked, or that the float cannot reach, is
/// replaced by `level_station`'s.
fn console_station(k: u32) -> [f32; 3] {
    match k { 7 | 8 => [46.0, -7.0, 0.0], 1 | 2 => [20.0, -2.0, 0.0], _ => [32.0, -1.7, 0.0] }
}

/// The deadline order under the disc countdown (COUNTDOWN_STEP = REC_FRAME_RATE 1/60
/// [M 0x1aa1f0-0x1aa208], the p3 seeds by missile index 105/105/40/20/0/60/105/60 s [D
/// placement params 2026-11-06]): missile k's state-0 exit (MissileDeploy 0→1 [M 0x1aa218])
/// lands at launch + seed·60, its p0 ends P0_LEN[k−1] − 1 later, and state 2 reads the console
/// channel then [M 0x1aa25c] — so console k's read deadline is launch + seed·60 + P0_LEN[k−1] − 1:
/// k5 +1799 (seed 0), k4 +2999 (20), k3 +4199 (40), k6/k8 +5999 (60), k1/k2/k7 +9599 (105).
/// The order below serves the deadlines, weighted by when each console's UI script can first
/// hold (its gate 30+k fires at ITS missile's armed+359 [D P0_GATE] — k5's UI at +359, k4's at
/// +1559, k3's at +2759, k6/k8's at +3959, k1/k2/k7's at +6659): k6 before k8 keeps both inside
/// +5999 without a double crossing (k6 shares the x32 station with 3/4/5 — no travel — and the
/// 6→8 crossing then lands k8 ~+5480). The tail [1, 2, 7] serves the SHARED +9599 deadline in
/// any order, and 1/2 before 7 keeps the real-health walk alive: the station's astronaut wave
/// is on Bond from ~launch+6810 (measured on master's raw aim and here: the ring's 3.0-damage
/// hits at d 3-13 while Bond hovered the x46 nest for 7's UI — 50 hits for the full 100.0,
/// dead before the loop's end; the wave's own spawn sits near the m1/m2/m7 p0 sync writes
/// ch 181..188 @t10 [D P0_SYNC, ~launch+6310] [S — the spawn trigger itself is not pinned]).
/// Waiting for 1/2's holds at their x20 station (25 units from the nest) spends the same
/// window out of the ring, and the single return crossing lands 7 inside its deadline [S —
/// the measured margins are the latch log below].
const CONSOLE_ORDER: [u32; 8] = [5, 4, 3, 6, 8, 1, 2, 7];

/// The level look's yaw error toward `at` (about world up — the forward row's heading), the
/// height of `at` over the eye, and the horizontal range.
fn level_errs(sim: &nf_view::scene::Sim, at: [f32; 3]) -> (f32, f32, f32) {
    let e = sim.bond.head();
    let f = sim.bond.look();
    let yaw_err = walker::wrap_pi((at[0] - e[0]).atan2(at[2] - e[2]) - f[0].atan2(f[2]));
    (yaw_err, at[1] - e[1], ((at[0] - e[0]).powi(2) + (at[2] - e[2]).powi(2)).sqrt())
}

/// The view is level (the forward row's y under 0.01): the master view keeps the entry pitch 0
/// in zero-g, so only a level forward row shoots where the view looks.
fn view_level(sim: &nf_view::scene::Sim) -> bool { sim.bond.look()[1].abs() < 0.01 }

/// The level look's turn toward `at` plus the stabiliser press (action 0xe [M Player_ZeroG
/// 0x192ab0:163-199]) whenever the forward row has picked up a tilt.
fn level_turn(sim: &nf_view::scene::Sim, inp: &mut Input, at: [f32; 3], turn_max: f32) {
    let (e, _, _) = level_errs(sim, at);
    inp.turn = e.clamp(-turn_max, turn_max);
    inp.stabilise = sim.bond.look()[1].abs() > 0.005 && sim.bond.stabiliser == 0;
}

/// The round's start offset from the eye along the body's right/up rows. The laser round leaves
/// from between the eye and the view model's muzzle datum and flies at the view point 256 u out
/// (scene.rs `shot_from`, weapon 0x33's +0x68 0x10000 arm); read back off each fired round (its
/// spawn point = pos − dir·travelled), so the aim can lead the eye by the offset.
#[derive(Clone, Copy, Default)]
struct ShotCal { off: [f32; 2] }

impl ShotCal {
    fn learn(&mut self, sim: &nf_view::scene::Sim, n_before: usize) {
        let Some(b) = sim.drones.player_bullets.get(n_before) else { return };
        let o = [b.pos[0] - b.dir[0] * b.travelled, b.pos[1] - b.dir[1] * b.travelled, b.pos[2] - b.dir[2] * b.travelled];
        let e = sim.bond.head();
        let rel = [o[0] - e[0], o[1] - e[1], o[2] - e[2]];
        let m = sim.bond.zero_g_rot;
        let dot = |a: [f32; 3]| a[0] * rel[0] + a[1] * rel[1] + a[2] * rel[2];
        self.off = [dot(m[0]), dot(m[1])];
    }
    /// The world offset of the round's line from the eye's at range `d` (it closes on the view
    /// line at 256 u).
    fn at(&self, sim: &nf_view::scene::Sim, d: f32) -> [f32; 3] {
        let m = sim.bond.zero_g_rot;
        let k = (1.0 - d / 256.0).max(0.0);
        [0, 1, 2].map(|i| (self.off[0] * m[0][i] + self.off[1] * m[1][i]) * k)
    }
    /// Where the eye aims so the round passes `c`.
    fn eye_target(&self, sim: &nf_view::scene::Sim, c: [f32; 3]) -> [f32; 3] {
        let e = sim.bond.head();
        let d = ((c[0] - e[0]).powi(2) + (c[1] - e[1]).powi(2) + (c[2] - e[2]).powi(2)).sqrt();
        let o = self.at(sim, d);
        [c[0] - o[0], c[1] - o[1], c[2] - o[2]]
    }
    /// The round's predicted line: start and unit direction.
    fn ray(&self, sim: &nf_view::scene::Sim) -> ([f32; 3], [f32; 3]) {
        let e = sim.bond.head();
        let o0 = self.at(sim, 0.0);
        let o = [e[0] + o0[0], e[1] + o0[1], e[2] + o0[2]];
        let f = sim.bond.look();
        let v = [e[0] + f[0] * 256.0 - o[0], e[1] + f[1] * 256.0 - o[1], e[2] + f[2] * 256.0 - o[2]];
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        (o, [v[0] / l, v[1] / l, v[2] / l])
    }
}

/// A shot from `eye` reaches the hit sphere (c, r): the bullet segment's first world contact lies
/// past the sphere's entry (the held script's damage volume [M SP_GetHitDamage 0x1dc5f8]; the
/// bullet pass's world-first rule).
fn sphere_shot_ok(sim: &nf_view::scene::Sim, eye: [f32; 3], c: [f32; 3], r: f32) -> bool {
    let d = ((c[0] - eye[0]).powi(2) + (c[1] - eye[1]).powi(2) + (c[2] - eye[2]).powi(2)).sqrt();
    if d <= r { return true; }
    match sim.world.segment_bullet(eye, c) { None => true, Some(t) => t * d > d - r * 0.9 }
}

/// The stance's line through the game's aim snap: the shot converges on the crosshair ray's
/// first pick stop [M Player_WeaponInitBullet 0x1a4bf0:154-164], so the round reaches the held
/// sphere (c0, r) only when the line from `eye` to the snap point still passes inside it. The
/// same candidates the snap sees gate the planner (the raised launch-rig pieces included —
/// `sphere_shot_ok` is world-only and blind to them): a player moves to a clear line.
/// `aim` is the point the view turns onto (the adjusted target `c`).
fn snap_shot_ok(sim: &nf_view::scene::Sim, eye: [f32; 3], aim: [f32; 3], c0: [f32; 3], r: f32) -> bool {
    let d = data_of(sim);
    let glass = d.weapons.records.get(sim.gun.weapon as usize)
        .map_or(true, |w| u32::from_le_bytes(w[0x6c..0x70].try_into().unwrap()) & 0x800 == 0);
    let v = [aim[0] - eye[0], aim[1] - eye[1], aim[2] - eye[2]];
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-4 { return true; }
    let raw = [eye[0] + v[0] / l * 256.0, eye[1] + v[1] / l * 256.0, eye[2] + v[2] / l * 256.0];
    let (pt, _) = sim.drones.aim_snap_traced(&d, eye, raw, glass);
    let w = [c0[0] - eye[0], c0[1] - eye[1], c0[2] - eye[2]];
    let u = [pt[0] - eye[0], pt[1] - eye[1], pt[2] - eye[2]];
    let ul = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt().max(1e-6);
    let along = (w[0] * u[0] + w[1] * u[1] + w[2] * u[2]) / ul;
    if along <= 0.0 { return false; }
    ((w[0] * w[0] + w[1] * w[1] + w[2] * w[2] - along * along).max(0.0)).sqrt() <= r
}

/// Room for the float's body (the 0.55 capsule [M Player_Collision 0x194df0]) at the station `st`
/// and on the straight way from `from`: the feet and head lines and four 0.7-u spokes clear.
fn station_room(sim: &nf_view::scene::Sim, from: [f32; 3], st: [f32; 3]) -> bool {
    let room = |a: [f32; 3], b: [f32; 3]| sim.world.segment(a, b).is_none();
    let up = [st[0], st[1] + 1.2, st[2]];
    let dn = [st[0], st[1] - 0.6, st[2]];
    room(from, st) && room(st, up) && room(st, dn) && room([from[0], from[1] + 0.9, from[2]], up)
        && (0..4).all(|q| { let a = q as f32 * std::f32::consts::FRAC_PI_2; room(st, [st[0] + 0.7 * a.sin(), st[1], st[2] + 0.7 * a.cos()]) })
}

/// Where to float for a level shot at the sphere (c, r): the eye at the centre's height on a ring
/// around it (6–14 u out, 24 bearings), the shot clear (`sphere_shot_ok`) AND unstealable by the
/// aim snap (`snap_shot_ok` — the same candidates the snap sees), room for the body (`station_room`
/// — from the float's own spot, or from `pref` the console's default anchor when the float has
/// wedged and its straight lines are all bank-clipped), not a station the float already stalled
/// short of; the nearest to `pref`.
/// `hard` (the 00 Agent sister, level-1b-console-latch-00agent): the score adds the station's
/// exposure — the number of live astronauts whose muzzle ray reaches the stance's eye — weighted
/// over the walk distance. The measured 00 Agent death [L 1b-run1.log]: the launch order (ch 19
/// [D spawner #163 p6 = 194, AND #202]) fires MID-WAVE (five skin-0x5000078 survivors at hp
/// 40/37 keep their posts at 6-19 u), and the console phase ran with NO wave defense for the mid
/// group (the wave_close detour is tail-only) — the survivors' 0x6a rounds (5.0 [D weapon_data
/// +0x0c] × pain Plr_DMod_Hard 1.0 [M 0x1902c8; 0x1d3a80 case 0x700001b]) landed ~1 hit per 250
/// ticks and ground the walk down from tick 1775 to death at 6251 (18 hits for 100.0, 0
/// return-fire fights) while consoles 6/8/1/2/7 were still unlatched. A player fighting through
/// a hub whose gate has already fired takes the stance the fewest shooters can line before he
/// shoots the screen — same shot rule, fewer lined muzzles.
fn level_station(sim: &nf_view::scene::Sim, c: [f32; 3], r: f32, c0: [f32; 3], from: [f32; 3], pref: [f32; 3], banned: &[[f32; 3]], hard: bool) -> Option<[f32; 3]> {
    let eye_off = sim.bond.head()[1] - sim.bond.pos[1];
    let mut best: Option<(f32, [f32; 3])> = None;
    for rad in [6.0f32, 8.0, 10.0, 12.0, 14.0] {
        for k in 0..24 {
            let a = k as f32 * std::f32::consts::TAU / 24.0;
            let eye = [c[0] + rad * a.sin(), c[1], c[2] + rad * a.cos()];
            let st = [eye[0], eye[1] - eye_off, eye[2]];
            if !sphere_shot_ok(sim, eye, c, r) || !snap_shot_ok(sim, eye, c, c0, r)
                || !(station_room(sim, from, st) || station_room(sim, pref, st)) { continue; }
            if banned.iter().any(|b| dist3(*b, st) < 1.0) { continue; }
            let mut dd = dist3(st, pref);
            if hard {
                if !hurt_free(sim, st) { continue; }
                dd += 2.0 * exposure(sim, eye).0 as f32;
            }
            if best.map_or(true, |b| dd < b.0) { best = Some((dd, st)); }
        }
    }
    best.map(|b| b.1)
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

/// 00 Agent (hard): whether the standing capsule at `at` touches any armed hurt volume — the
/// same contact law the port's damage arm runs (the feet..+1.8 r 0.55 capsule [M
/// Player_DealWithObjHit 0x194458; `Hurt::update`'s disabled/fired gates [M Hurt_Update
/// 0x17e5d0's 0x20 bit]) read through the pub fields so the probe cannot start, fire or retire
/// anything. The run2 death [L 1b-run2.log]: 95.0 of the 100.0 to the world (the −20.0 plates
/// at 8502 and the fatal −39.3 at (46.01,−8.08,0.00) at 10424) once the exposure-hedged
/// stances moved Bond onto the level's own hurt plates — a stance that trades lined muzzles
/// for a 20.0 plate is not a player stance; the pick now refuses it.
fn hurt_free(sim: &nf_view::scene::Sim, at: [f32; 3]) -> bool {
    let a = [at[0], at[1], at[2]];
    let b = [at[0], at[1] + 1.8, at[2]];
    !sim.hurts.iter().any(|h| !h.disabled && !h.fired
        && nf_game::triggers::capsule_hits_tris(a, b, 0.55, &h.tris))
}

/// The float's thrust toward `at` with the lift on a damped speed target (the eased thruster lags
/// ~25 ticks [M 0x192ab0:92-159], so the plain proportional lift creeps the last half-unit).
fn float_to(sim: &nf_view::scene::Sim, at: [f32; 3]) -> Input {
    let mut inp = thrust_to(sim, at);
    let dy = at[1] - sim.bond.pos[1];
    let vstar = (dy * 0.03).clamp(-0.05, 0.05);
    inp.lift = ((vstar + (vstar - sim.bond.zero_g_lift) * 2.0) * 10.0).clamp(-1.0, 1.0);
    // The sag guards: the eased descent overshoots its brake and dropped the float past the
    // low stations toward the room floor (console 8's end of the bank), where the collision
    // settles him into a pocket under the bank. The pocket does NOT end zero-g by itself
    // [M Player_ZeroG 0x192ab0 has no ChangeSubState — and the driver's no-input measurement:
    // PCSX2 and the port both settle there and float-hold], and the old comment here was
    // WRONG: it is not walk-mode pinning him and the thrusters are NOT dead — Input.lift is
    // the game's real vertical thruster (actions 7−8 [M 0x192ab0:136-137], the +0x14 lane the
    // smoother eases [M 0x192ab0:92-159]) and it works; the ROUTE was the trap: the straight
    // thrust at the goal drives into the bank face while the near-floor creep holds the
    // height, so the net motion is the hold the no-input measurement shows. Below the goal,
    // demand the climb; near the floor AND near the goal from above, creep the last stretch
    // down instead of braking to a stall. (The route's way out is the escape phase in
    // `shoot_consoles`: leave the pocket up-and-back first, then re-plan from a spot whose
    // straight lines are not bank-clipped.)
    if dy > 0.3 {
        inp.lift = 1.0;
    } else {
        let near_floor = sim.world.segment(sim.bond.pos, [sim.bond.pos[0], sim.bond.pos[1] - 2.2, sim.bond.pos[2]]).is_some();
        let vstar = if near_floor && dy > -1.5 { (dy * 0.03).clamp(-0.012, 0.05) } else { vstar };
        inp.lift = ((vstar + (vstar - sim.bond.zero_g_lift) * 2.0) * 10.0).clamp(-1.0, 1.0);
    }
    inp
}


/// One-shot pocket probe (1b-zero-g-floor diagnostics): ray the enclosure from the wedged spot
/// in 26 directions (6 u out) and print the hit distances — the pocket's shape decided the
/// route fix (the two-phase crossing). Fires once per shoot_consoles run.
fn pocket_probe(sim: &nf_view::scene::Sim, goal: [f32; 3], k: u32, tt: u32, done: &mut bool) {
    let p = sim.bond.pos;
    if !*done {
        *done = true;
        eprintln!("[pocket] con{k} t{tt} at ({:.2},{:.2},{:.2}) goal ({:.2},{:.2},{:.2})", p[0], p[1], p[2], goal[0], goal[1], goal[2]);
        let mut dirs: Vec<[f32; 3]> = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]];
        for sx in [-1.0f32, 1.0] { for sy in [-1.0, 1.0] { for sz in [-1.0, 1.0] {
            let l = (sx * sx + sy * sy + sz * sz).sqrt();
            dirs.push([sx / l, sy / l, sz / l]);
        } } }
        for (i, d) in dirs.iter().enumerate() {
            let far = [p[0] + d[0] * 6.0, p[1] + d[1] * 6.0, p[2] + d[2] * 6.0];
            let hit = sim.world.segment(p, far).map(|t| t * 6.0);
            eprintln!("[pocket]   dir{i} ({:+.2},{:+.2},{:+.2}) hit {}", d[0], d[1], d[2], hit.map_or("clear".into(), |h| format!("{h:.2}")));
        }
    }
}

/// Shoot the eight console screens in deadline order with the level look: each console UI script
/// carries three holds (event 3 on streams 1/2/3 [D the probe1b_scripts decode]) and resumes from
/// each on 20 bullet damage — one rifle hit per hold [M SP_UnPause 0x1dca58 / SP_GetHitDamage
/// 0x1dc5f8], then writes the console channel [D stream 4 ev0x0d]. The aim is the held script's
/// largest hit sphere (the game's own damage volume — held_hit_targets): Bond floats to the
/// console's station at the screen's own height (the lift thruster), yaws onto it and taps when
/// the round's line (`ShotCal`) passes through the middle of that sphere with the world clear:
/// the largest one is OBJECT_DissarmCollision's, two triangles the round has to meet [M
/// Collide_Intersect 0x1ebaa8]. The trigger is
/// tapped in bursts: the laser's clip only recharges (+1 per 4 frames while no fire script runs
/// [M Player_Update 0x193600 disasm 0x193eb8-0x19405c]; each tap drains 12), and starting a burst
/// at 48 rounds keeps the holds' 3 hits together so the recharge runs between consoles. Returns
/// the frame each console latched.
/// The ticks the walk keeps for the tail consoles' work once the wave detour stops (the
/// measured k1/k2/k7 run from the fight's end to the last latch was ~1510 ticks).
const TAIL_RESERVE: u32 = 2400;

/// Ticks until the first branch read of a tail missile (1/2/7) whose console is still clear:
/// state 0 counts its seed down at `REC_FRAME_RATE` per gated tick, state 1 reads at its tick
/// L−1 [M 0x1aa1c8-238, 0x1aa23c-258; D P0_LEN]. None when no such read is ahead.
fn tail_read_in(s: &nf_view::scene::Sim) -> Option<u32> {
    use nf_game::space_missile::P0_LEN;
    s.drones.space_missiles.iter()
        .filter(|m| matches!(m.index, 1 | 2 | 7) && !s.drones.channels.on(m.index as u32))
        .filter_map(|m| {
            let len = P0_LEN[(m.index - 1) as usize];
            match m.state {
                0 => Some((m.countdown.max(0.0) * 60.0) as u32 + len - 1),
                // The p0's stream clock: the create beat ran update 1, each state-1 tick one more
                // [M 0x1a9ef0:61, 0x1aa23c], so the ticks spent in state 1 are clock − 1.
                1 => {
                    let clock = m.info.and_then(|h| s.drones.scripts.owned(h)).map_or(0.0, |sc| sc.clock);
                    Some((len - 1).saturating_sub((clock as u32).saturating_sub(1)))
                }
                _ => None,
            }
        })
        .min()
}

/// Ticks until missile `k`'s state-2 read of console k's channel — the console's read deadline,
/// the same clock `tail_read_in` reads for the tail: state 0 counts its seed down at
/// `REC_FRAME_RATE` per gated tick, state 1 reads at its tick L−1 [M 0x1aa1c8-238,
/// 0x1aa23c-258; D P0_LEN]. None once the console is latched or the missile is past the read.
fn read_deadline_in(s: &nf_view::scene::Sim, k: u32) -> Option<u32> {
    use nf_game::space_missile::P0_LEN;
    if s.drones.channels.on(k) { return None; }
    s.drones.space_missiles.iter()
        .filter(|m| m.index as u32 == k)
        .filter_map(|m| {
            let len = P0_LEN[(k - 1) as usize];
            match m.state {
                0 => Some((m.countdown.max(0.0) * 60.0) as u32 + len - 1),
                1 => {
                    let clock = m.info.and_then(|h| s.drones.scripts.owned(h)).map_or(0.0, |sc| sc.clock);
                    Some((len - 1).saturating_sub((clock as u32).saturating_sub(1)))
                }
                _ => None,
            }
        })
        .min()
}

fn shoot_consoles(w: &mut Walker, alive: &impl Fn(&nf_view::scene::Sim) -> bool, hard: bool) -> Vec<(u32, u32)> {
    // 00 Agent (hard): the level's own contact plates, once — the [L] ledger of where the
    // −20.0/−39.3 world hits came from (the death at (46.01,−8.08,0.00) sat INSIDE the x46
    // plate's box).
    if hard {
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, h) in sim.hurts.iter().enumerate() {
            eprintln!("[1b] hurt {i}: kind {} damage {:.1} gate {} disabled {} fired {} lo ({:.1},{:.1},{:.1}) hi ({:.1},{:.1},{:.1}) tris {}",
                h.kind, h.damage, h.gate, h.disabled, h.fired, h.lo[0], h.lo[1], h.lo[2], h.hi[0], h.hi[1], h.hi[2], h.tris.len());
        }
    }
    let mut latched = Vec::new();
    let mut last_shot = 0u32;
    let mut in_burst = false;
    let mut climbing_out = false;
    // aim-route-3 (the aim-route-finish §2 recipe): `answered[pi]` counts the script player's
    // SP_UnPause activations — the damage tail's one-tick flag [M 0x1dc2f8-0x1dc318] fires once
    // per 20-damage block, and the console UI scripts carry exactly three holds (event 3 on
    // streams 1/2/3 [D probe1b_scripts]) resumed each by 20 bullet damage — so the count IS the
    // answered-hold count, straight from the game's own unpause path. `missile_start[k−1]` pins
    // the frame missile k left state 0 (MissileDeploy 0→1 [M 0x1aa218]) — its p0's tick-0 frame
    // S_k — so console k's read deadline is S_k + P0_LEN[k−1] − 1 [M 0x1aa23c-258; D P0_LEN].
    let n_sp = w.scene.sim.as_ref().unwrap().drones.scripts.players.len();
    let mut answered = vec![0u32; n_sp];
    let mut missile_start = [0u32; 8];
    let mut ch19_frame = 0u32;
    let mut ch19_was = false;
    let mut ch19_drop_at = 0u32;
    let mut preflied: Vec<u32> = Vec::new();
    // level-1b-console-latch-00agent (hard): the mid-phase defense's cooldown anchor.
    let mut last_defense = 0u32;
    // real-health-00agent-1b-7: consecutive zero-kill FULL-CAP fights — the fight-loop
    // signature (see the mid-defense block below).
    let mut no_progress_streak = 0u32;
    let mut plan: (u32, Option<[f32; 3]>) = (0, None);
    let mut cal = ShotCal::default();
    let mut banned: Vec<[f32; 3]> = Vec::new();
    // 1b-zero-g-floor item 1: the pocket under the bank cannot be LEFT by any thrust —
    // measured this session: the wedge at (44.75, −8.66, 1.69) held the position to ±0.05
    // through 3000+ ticks of full climb (lift 1.0, the real actions-7−8 input [M 0x192ab0
    // :136-137]) plus sustained back thrust, and the [pocket] ray dump maps the enclosure
    // (ceiling 0.83 straight up — the bank's underside, sloping DOWN toward +x/+z at 0.79-1.75
    // on every up diagonal; the riser 4.73 at +x; −x/±z/down clear). The port's radial push
    // from that slope cancels the whole move — the game's own radial push tail is provably
    // dead [M Intersect_CylGeom 0x109da0:447's HITTEST+0x50 guard; the game CLIPS the move at
    // the face instead, which slides], and the task forbids collision changes. So the fix is
    // the route: never enter the pocket — the two-phase goal (the report's own next lever).
    let mut pocket_dumped = false;
    let mut pocket_probe_done = false;
    let mut moved_at = 0u32;
    let mut last_goal = [0.0f32; 3];
    let mut last_goal_dist = f32::MAX;
    let mut dead_ticks = 0u32;
    let mut last_esc_pos = [0.0f32; 3];
    let mut dumped: Vec<u32> = Vec::new();
    // aim-route-4: the disc countdown (seed·60 gated ticks) spreads the console phase — the
    // last deadline group (k1/k2/k7, armed at launch+6300) sits at launch+9599 and the last
    // latch lands ~8300 ticks after ch 19 (measured below), past the old 6000 cap.
    // The stuck guard (level-1b-drake-walk-time): once Bond is dead or a missile has launched
    // (mission status > 1) the phase can never progress — the console channels' only writers
    // are Bond's own shots, the launched missile's state is final, and the wave only grows —
    // so every further tick is the unbounded grind the task reproduces: a dead Bond at Agent
    // re-entered the k1 wave fight 12000×2800 ticks (33.6M, ~590 s+ of wall, no result) because
    // 15 astronauts never thin near a body that cannot shoot and k1 never latches. A passing
    // walk never sits at status > 1 here (the fight-gate assert below pins `status() <= 1` on
    // the route), so the guard is inert on green runs; on red runs it hands back a fast,
    // ledger-rich failure instead of hanging the suite.
    let stuck = |s: &nf_view::scene::Sim| !alive(s) || s.drones.mission.status() > 1;
    for tt in 0..12000u32 {
        // The wave defense, TAIL CONSOLES ONLY (aim-route-5, ported from master's raw-aim fix):
        // the station's astronaut wave follows Bond across the map (measured: 3.0-damage hits
        // at d 3-13 from ~launch+6810 — the real-health walk took 50 hits for the full 100.0
        // hovering the [7,1,2] x46 nest). The early chains have no wave to fight (first contact
        // ~launch+6810) and no margin to spend: k5/k4/k3/k6/k8 sit ~560-1150 inside their
        // one-tick branch reads (+1799/+2999/+4199/+5999), and an unrestricted detour measured
        // +829 on k3 on master — m3 then read its console CLEAR at the +4199 branch and took
        // the LAUNCH arm (MissileDeploy [2,2,3,…], mission fail). Whenever the wave closes
        // inside 25 u on a tail console, fight it with the proven first-wave killer — the
        // fight's own dry-clip cover discipline (float away at <12, back out at 60) paces the
        // laser's 100-round pool against the 12-a-tap drain [M 0x193eb8-0x19405c] — and resume
        // the console work the tick it thins out. The shared +9599 tail deadline absorbs the
        // detour (the measured margins were ~1500-2200).
        {
            let sim = w.scene.sim.as_ref().unwrap();
            walk_log::leg("consoles", sim);
            if stuck(sim) {
                eprintln!("[1b] consoles STUCK guard: frame {} hp {:.1} armour {:.1} deploy {:?} mst {} astro {} — the phase cannot progress (dead Bond or a launched missile), ending the leg",
                    sim.drones.frame, sim.drones.health.health, sim.drones.health.armour, sim.drones.missile_deploy,
                    sim.drones.mission.status(), live_astronauts(sim).len());
                break;
            }
            let all_latched = CONSOLE_ORDER.iter().all(|&k| sim.drones.channels.on(k));
            let k = if all_latched { 0 } else { *CONSOLE_ORDER.iter().find(|&&k| !sim.drones.channels.on(k)).unwrap() };
            // The detour is bounded by the tail branch read: a wave that does not thin is left
            // standing once the last unlatched tail console's read is TAIL_RESERVE ticks away —
            // a player shoots the consoles before the missiles leave (measured on the stream join:
            // a wave that never thinned held the walk in the fight 6902..11264, past the
            // k1/k2/k7 read at ~11253, and m1/m2/m7 took the launch branch).
            let tail_ok = |s: &nf_view::scene::Sim| tail_read_in(s).map_or(true, |n| n > TAIL_RESERVE);
            let wave_close = (k == 1 || k == 2 || k == 7) && tail_ok(sim)
                && live_astronauts(sim).iter().any(|&i| dist3(sim.drones.list[i].pos, sim.bond.pos) < 25.0);
            if wave_close {
                let f0 = sim.drones.frame;
                fight(w, console_station(k), None,
                    |s| stuck(s) || !tail_ok(s) || !live_astronauts(s).iter().any(|&i| dist3(s.drones.list[i].pos, s.bond.pos) < 25.0),
                    2800);
                eprintln!("[1b] wave fight at k{k}: frames {f0}..{}", w.scene.sim.as_ref().unwrap().drones.frame);
                continue;
            }
        }
        // 00 Agent (hard): the mid-group defense. The tail detour above only guards k 1/2/7; on
        // the measured stream the first wave's survivors line Bond during the MID consoles too
        // (the run's shooters at 6-19 u from tick 1775 on), 2+ lined at once around the x32
        // station — the exposure-hedged stance below cuts the lined count, and when it still
        // reads 2+ the player breaks off and kills the nearest threat before going back to the
        // screen: the fight's own waves_first cover discipline + target banding, bounded at 700
        // ticks, on a 1200-tick cooldown so the per-console deadlines keep their margins [L
        // 1b-run1.log: death at 6251 with consoles 6/8/1/2/7 unlatched, 18 hits for 100.0, 0
        // return-fire fights].
        if hard {
            // real-health-00agent-1b-5 (the 300-cycle re-balance against the REAL read times):
            // the 1200-tick cooldown starved the thinning — the -4 round measured 300 as the
            // edge (150 starved the holds outright, m3 launched, hp 2.2 at 6977 [L the -4 runs
            // A/B/150]; 300 kept the lane fights thinning the wave 7 -> 3) — so the cycle is 300.
            let lined = exposure(w.scene.sim.as_ref().unwrap(), w.scene.sim.as_ref().unwrap().bond.head()).0;
            // real-health-00agent-1b-6 (measured, iteration 8 [L /tmp/lever8.log]): the cooldown
            // must count SIM FRAMES, not loop iterations — fight_opts_y burns its whole window
            // (450) inside ONE iteration, so the iteration counter read ~0 right after a fight
            // and the defense slept ~300 real ticks while the station grind drained 13.9 hp in
            // the 6548..6600 window (then the fight never re-fired before the k6 chain began).
            let cooled = w.scene.sim.as_ref().unwrap().drones.frame.saturating_sub(last_defense) >= 300;
            // The cap from the pending console's REAL branch read: the state-2 channel read at
            // S_k + P0_LEN − 1 [M 0x1aa23c-258; D P0_LEN] — `read_deadline_in` reads the same
            // clock the missiles run (state 0's countdown and state 1's owned-script clock, the
            // cadence probe pins it 1:1 per sim tick [M 0x1aa23c per-tick Script_Update]). The
            // fight may spend the slack the console's chain does not need: cap = deadline − 800,
            // clamped 300..1400. The −400 reserve let the fights eat k3's chain (m3 launched,
            // death t6580); −800 latches k3 in time (m3/m4/m5 all disarmed, death t7494);
            // −1000 disarms too (t7402), −1200 starves the fights (t6505) [L the 11-13 runs].
            let deadline = {
                let sim = w.scene.sim.as_ref().unwrap();
                CONSOLE_ORDER.iter().find(|&&c| !sim.drones.channels.on(c))
                    .and_then(|&c| read_deadline_in(sim, c))
            };
            let cap = deadline.map_or(300u32, |n| n.saturating_sub(800).clamp(300, 1400));
            // real-health-00agent-1b-6 (measured, iteration 6 [L /tmp/lever6.log]): the 1200
            // slack gate already disables the defense through every k6/k8 chain tick (their
            // read sits at 7774: deadline_in crosses 1200 at 6574 and the walk ground down
            // fight-free from 6574 to death 7541) — the gate must cover the MEASURED fight
            // window (450, below) + k6's measured 230-tick chain (hold#1 7289 -> latch 7519
            // [L iteration 4]) + re-acquisition ≈ 880, so 900: k5/k4/k3's tight windows (the
            // remaining deadlines at their measured fight starts: 469/446/461) stay pinned.
            // [L iteration 7: dropping this gate to 900 CASCADES — the extra pre-chain fight at
            // 3734 (deadline_in 1040) shifted the route, the k4-window fight then collided with
            // the f4200 spawn wave (astro 4 -> 6 mid-fight, −45.0 hp in one window, k4's latch
            // printed +297 late) and the walk died at 6784. The pre-chain fights stay gated at
            // 1200: the measured green slots are the spawn-locked ones (3105/4328/5513/6098).
            // The chain WINDOW relief is separate: the fence below admits fights inside a
            // STARTED chain on the missile's own live clock (read_deadline_in reads the same
            // state-1 clock the state-2 read fires on [M 0x1aa23c-258] — the measured live
            // k4 read sits at ~5974, 1200 past the modeled S_k + P0_LEN − 1: the p0 starts
            // ~1200 after the MissileDeploy 0->1, so the live clock is the game's truth).]
            // [L iteration 10: the k8 chain window's fight is blocked by THIS gate alone — the
            // idle log at f7200 reads lined 6 / cooled true / chain_clear true / slack_ok FALSE
            // with the live deadline_in at 1174 (m8's live read ~8374), and the walk died at
            // 7541 with hp 15.5 still grinding k8's chain. 950 admits it: 450 fight + ~300
            // chain remainder + re-acquisition ≈ 1000 < 1174 [L the measured chain shapes].]
            let slack_ok = deadline.map_or(true, |n| n > 1200);
            // The hold-chain fence (the console5 k5 lesson): a defense fight must never start
            // inside a TIGHT chain — the chain's holds ride the same read deadline, and a fight
            // inside the chain steals the window (the measured red run: the fight 2988..3305 sat
            // between hold#1 2917 and hold#2 3593, the latch landed +93 past 3574, missile 5
            // launched). `answered[pi] == 0` = the chain has not started (the SP_UnPause
            // activation count, the aim-route-3 recipe).
            // real-health-00agent-1b-6 (measured, iteration 5 [L /tmp/lever5.log]): the fence as
            // an ABSOLUTE chain stop is too blunt — the walk died at 7541 INSIDE k6's fight-free
            // chain: the station grind costs 11-20 hp a console window (the fight gaps
            // 3435..4328: −11.1, 4716..5513: −20, 5799..6098: −15) and the fence leaves every
            // chain unprotected. The chains are NOT all tight: k6's read sits at 7774 and its
            // measured chain is 230 ticks (hold#1 7289 -> latch 7519 [L iteration 4]) — a
            // slack-rich chain absorbs the bounded fight window (450 below) plus the chain
            // remainder plus re-acquisition with ~300 to spare, so the fence now only pins
            // chains whose remaining read deadline is under 1000 (at the measured fight starts
            // the remaining deadlines were k5 469 / k4 446 / k3 461 [L iteration 5 caps] — all
            // still pinned).
            let chain_clear = {
                let sim = w.scene.sim.as_ref().unwrap();
                CONSOLE_ORDER.iter().find(|&&c| !sim.drones.channels.on(c)).map_or(true, |&c| {
                    sim.drones.scripts.players.iter().position(|p| p.gate as u32 == 30 + c)
                        .map_or(true, |pi| answered[pi] == 0 || read_deadline_in(sim, c).map_or(false, |n| n > 900))
                })
            };
            // real-health-00agent-1b-6 (measured, iteration 1 [L /tmp/lever1.log]): the round-5
            // ledger's "yield at the pending station" gate is WRONG here — suppressing the
            // station fights removed every defense after the first crossing (the fights 2-5 of
            // the baseline all start near x32) and the un-thinned wave ground Bond down at
            // 5649, 1845 ticks BEFORE the baseline death. The station fights are the survival
            // mechanism: fight_opts_y's exposure-hedged cover floats break the shooters' sight
            // (the baseline's fight 1 ended after 53 ticks on a sight break). The yield lever
            // only survives as the fight's own SHAPE below: the hold moves OUT to the wave's
            // approach, one kill bounds the exposure, and the walk returns to the screen.
            // Gated like every lever here: the 00 Agent path only (`hard`), and never when the
            // walker tops up (the walker for this test always runs with top_up_health off —
            // `!w.tops_up()` — so the lever is inert on any allowed-top-up variant).
            // real-health-00agent-1b-6 (measured, iteration 13 [L /tmp/lever13.log]): the
            // melee override — the thinned wave's last two astronauts CLOSED inside the 3 u
            // TooClose band at the k6 station (every round hits for twice the damage [M
            // DoBulletAccuracy 0x179740:113-115; Drone_ModBulletDamage 0x139fac-0x139fc8]) and
            // shredded the chain stand: hp 41.8 -> 0.0 in the 7200..7562 window (4+ doubled
            // hits) with the fight gated off by slack_ok. A closing melee pair is more urgent
            // than the read deadline (death trumps it), so any live astronaut inside 8 u
            // forces the fight past slack_ok; the started-chain fence below still applies.
            let melee = live_astronauts(w.scene.sim.as_ref().unwrap()).iter()
                .any(|&i| dist3(w.scene.sim.as_ref().unwrap().drones.list[i].pos, w.scene.sim.as_ref().unwrap().bond.pos) < 12.0);
            if (lined >= 2 && slack_ok || melee) && cooled && chain_clear && !w.tops_up() {
                last_defense = w.scene.sim.as_ref().unwrap().drones.frame;
                let f0 = w.scene.sim.as_ref().unwrap().drones.frame;
                let here = w.scene.sim.as_ref().unwrap().bond.pos;
                // real-health-00agent-1b-6, lever 2 (the round-5 ledger's run A shape): the
                // |z| 1.6 lane at Bond's own x sat behind the panel rows — the wave posts at
                // z ±2.7-4.3 there and out-ranges the capped stance, so the fights ran their
                // FULL caps without thinning ([L] this run: astro 6 at the t7494 death, five
                // fights ~4000 ticks, zero kills). Run A held the lane the wave CROSSES to
                // reach Bond and thinned 7 → 3 in one window: the lane sits ON the approach —
                // 40% of the way from Bond to the nearest shooter — still off-plane
                // |z| >= 1.6 on the shooter's side (the panel rows at z ±4.62 [D the glist]
                // clear the lane's faces), and the fire prio is the nearest shooter itself
                // (the crossed threat; the prio banding r−50 / r−60 inside 8 u concentrates
                // the laser there).
                let n0 = live_astronauts(w.scene.sim.as_ref().unwrap()).len();
                let (lane, prio_i) = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let live = live_astronauts(sim);
                    let near = live.iter().copied().min_by(|&a, &b| {
                        dist3(sim.drones.list[a].pos, sim.bond.pos).total_cmp(&dist3(sim.drones.list[b].pos, sim.bond.pos))
                    });
                    // [L iteration 11: the raw approach lane is a CHASE TRAP once the shooters
                    // roam — the fights dragged Bond to x39.4/z 10.8 (lanes printed), the k6
                    // chain starved (holds unanswered past its live read), m6/m8 LAUNCHED
                    // (deploy [1,1,2,2,2,3,1,3], status 2). The lane stays pinned to the
                    // pending console's station (±5 u xz, |z| >= 1.6): the wave crosses the
                    // route to reach Bond there, and the walk never leaves its console leg.]
                    let pending = CONSOLE_ORDER.iter().find(|&&c| !sim.drones.channels.on(c)).copied();
                    match near {
                        Some(i) => {
                            let sp = sim.drones.list[i].pos;
                            let st = pending.map(console_station).unwrap_or(here);
                            let lx = (here[0] + 0.4 * (sp[0] - here[0])).clamp(st[0] - 5.0, st[0] + 5.0);
                            let mut lz = (here[2] + 0.4 * (sp[2] - here[2])).clamp(st[2] - 5.0, st[2] + 5.0);
                            lz = if lz >= 0.0 { lz.max(1.6) } else { lz.min(-1.6) };
                            ([lx, here[1], lz], Some(i))
                        }
                        None => (here, None),
                    }
                };
                // One kill bounds the exposure (real-health-00agent-1b-6 [L iteration 1: the
                // same fight with only the lined<=1 exit ran its full 869-tick cap in the open
                // and the whole run died 1845 ticks earlier]) — but ONE kill a fight loses the
                // count race [L iteration 3: the fights are SPAWN-LOCKED, a fight only re-lines
                // when the next scripted spawn wave lands (3 astronauts a wave, the fight gaps
                // 3105→4328→5527 sit on the f4200/f5100 spawn waves), so one kill a wave nets
                // +2 live a wave and Bond died at 6199 with astro 9]. The spawn waves land
                // BUNCHED at one post, so the raised lift (below) can take TWO of them inside
                // the same exposure window: the fight ends on the second kill (or the first
                // sight break — the exposure exit below).
                // The height cap rides along: 6.0 guarded the STATION aim lift — the beam band
                // BigLaserBeamHURT #259 (dmg 50, gate ch 18) spans y 9.0-9.9 at z −0.4..0.4
                // [L the hurt dump; the station-lift death parked feet 7.53 at z 0] — but this
                // lane holds |z| >= 1.6, clear of the band's whole z range — but the fight's
                // cover ring wanders ±2.5 in z (see the iteration 9 note below), so the height
                // cap itself must stay band-safe, not just the lane. The laser hits 20.0
                // [D weapon_data 0x33 +0x0c] = one dead astronaut a hit.
                // [L iteration 4: fight 4 (6098..6974, cap 876) stalled — astro 5 -> 5, zero
                // kills — and the full window cost 28.0 hp (43.9 -> 16.1) plus the k6 chain's
                // start; the walk died at 7631 with k6 JUST latched (7519, −255). The kill
                // fights end inside 286-388 ticks (fights 2-3, two kills each), so a 450-tick
                // window keeps every measured kill shape and cuts a stalled fight's pure loss.]
                // [L iteration 9: max_y 12 is BAND-UNSAFE — the k4 screen stance had already
                // lifted Bond to feet 7.375 (capsule top 9.175 [M Player_DealWithObjHit
                // 0x194458: feet..+1.8 r 0.55]) and the fight's cover ring (±2.5 in z) walked
                // him under the live band's z −0.4..0.4: two −50.0 deliveries, hp 100 -> 0.0 in
                // a 14-tick fight at 3443. The band-safe body cap is feet 7.1 (top 8.9 < 9.0):
                // max_y 7.0 keeps the capsule out of the band at ANY z, reaches the ~8.3-8.8
                // aim heights the descending shooters sit at (the ledger's "wait the band
                // shooters down to the <=8.3 aim heights"), and the ceiling-band shooters
                // (aim 9+) wait out across the 300-tick frame cooldown.]
                let fight_cap = cap.min(300);
                let hp0 = w.scene.sim.as_ref().unwrap().drones.health.health;
                // real-health-00agent-1b-7 (the round-6 ledger's lever 1, measured [L
                // /tmp/baseline.log, /tmp/lever1.log, /tmp/lever2.log]): the melee override is
                // a FIGHT LOOP — with the wave parked within 12 u the fights re-fire with NO
                // 2-kill progress (astro 3 -> 3 -> 3 in fights 6714..7014/7014..7314/7314..7442,
                // no k6 hold EVER answered, death 7442) because `cooled` counts from the
                // fight's START and a cap-long fight ends exactly when the cooldown expires:
                // consecutive fights leave ZERO console ticks and the pending console's chain
                // starves. The fix is the SCREEN RESUMPTION on the loop SIGNATURE: two
                // consecutive zero-kill FULL-CAP fights (the parked-wave shape — the wave is
                // unreachable, the fight is pure loss) hand the window back to the console
                // work: last_defense moves to fight_end + 300, so the next defense (the melee
                // override with it) waits 600 real ticks while the pending console's holds
                // answer (k6's measured chain is 230 ticks, hold#1 7289 -> latch 7519 [L
                // round-6 iteration 4], and k6 shares the x32 station with 3/4/5 — no travel);
                // a kill or an early (exposure) exit resets/keeps the cadence. MEASURED
                // ANTI-LEVERS: (lever1.log) a TIME-based no-progress exit (a fight with no kill
                // by 150 ticks ends there) is WRONG — the measured kill shape needs 286-388
                // ticks (iters 3-4), so the 150 exit truncated every kill fight (astro 3 -> 3
                // where the baseline thinned 3 -> 1), the wave grew 3 -> 9 un-thinned, the
                // grind cost the whole health bar and m3 LAUNCHED (deploy [0,0,3,2,2,1,0,1],
                // death 6580 — 862 before baseline); (lever2.log) resuming after the FIRST
                // zero-kill full-cap fight is also WRONG — the baseline's first parked fight
                // (3105..3405, kills 0) is followed by the productive thinning fight
                // (3405..3556, 3 -> 1), and the 300-tick delay dropped that kill (the wave
                // grew 3 -> 10, death 6530). The SECOND consecutive zero-kill full-cap fight
                // is the measured loop signature: it first occurs at 7314 in the baseline
                // (after 6714..7014 and 7014..7314, both full-cap zero-kill), exactly where
                // the chain window is needed.
                fight_opts_y(w, lane, prio_i, true,
                    |s| stuck(s) || exposure(s, s.bond.head()).0 <= 1 || live_astronauts(s).len() + 2 <= n0, fight_cap, 7.0);
                {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let end = sim.drones.frame;
                    let kills = n0.saturating_sub(live_astronauts(sim).len());
                    if kills > 0 {
                        no_progress_streak = 0;
                    } else if end.saturating_sub(f0) >= fight_cap {
                        no_progress_streak += 1;
                        if no_progress_streak >= 2 {
                            last_defense = end + 300;
                        }
                    }
                    eprintln!("[1b] mid-phase defense: frames {f0}..{end} (lined was {lined}, cap {fight_cap} of {cap}, chain_clear {chain_clear}, astro {n0} -> {}, hp {hp0:.1} -> {:.1}, lane {lane:?}, kills {kills}, streak {no_progress_streak}{})",
                        live_astronauts(sim).len(), sim.drones.health.health,
                        if no_progress_streak >= 2 && kills == 0 { ", screen window" } else { "" });
                }
                continue;
            } else if lined >= 1 && w.scene.sim.as_ref().unwrap().drones.frame % 600 == 0 {
                let sim = w.scene.sim.as_ref().unwrap();
                eprintln!("[1b] defense idle f{}: lined {lined} cooled {cooled} slack_ok {slack_ok} chain_clear {chain_clear} hp {:.1} astro {}",
                    sim.drones.frame, sim.drones.health.health, live_astronauts(sim).len());
            }
        }
        let (input, fire) = {
            let sim = w.scene.sim.as_ref().unwrap();
            for &k in &CONSOLE_ORDER {
                if sim.drones.channels.on(k) && !latched.iter().any(|&(c, _)| c == k) {
                    latched.push((k, sim.drones.frame));
                    let dl = missile_start[(k - 1) as usize] as i64 + nf_game::space_missile::P0_LEN[(k - 1) as usize] as i64 - 1;
                    eprintln!("[1b] latch k{k} frame {} deadline {dl} margin {}",
                        sim.drones.frame, sim.drones.frame as i64 - dl);
                }
            }
            let Some(&k) = CONSOLE_ORDER.iter().find(|&&k| !sim.drones.channels.on(k)) else { break };
            let pi = sim.drones.scripts.players.iter().position(|p| p.gate as u32 == 30 + k && p.hit_points > 0.0)
                .expect("the console's UI script player (gate 30 + k, hit points 20 [D SP#213-220])");
            let tg = sim.drones.scripts.held_hit_targets(&data_of(sim));
            let best = tg.iter().filter(|x| x.0 == pi).max_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).map(|x| (x.1, x.2));
            let eye_off = sim.bond.head()[1] - sim.bond.pos[1];
            let mut station = console_station(k);
            let mut fire = false;
            let mut aim = None;
            let mut cross_high = false;
            if let Some((c0, r)) = best {
                let c = cal.eye_target(sim, c0);
                station[1] = c[1] - eye_off;
                let se = [station[0], station[1] + eye_off, station[2]];
                // The pick's candidate dump for the evidence log (the second-half consoles —
                // the first wave's death has raised the rigs by then), plus the per-bearing
                // ring scan: which stances the snap leaves alive and who steals the rest.
                let dump_n = dumped.iter().filter(|&&x| x == k).count();
                if matches!(k, 8 | 7 | 1 | 2) && (dump_n == 0 || tt % 2000 == 0) && dump_n < 3 {
                    dumped.push(k);
                    let dp = data_of(sim);
                    eprintln!("[cands] console {k} — the pick's candidates at t{tt}:");
                    for (ci, c2, r2, m2) in sim.drones.scripts.sp_bullet_targets(&dp) {
                        let p = &sim.drones.scripts.players[ci];
                        let hsh = m2.map_or(0, |(hh, _)| hh);
                        eprintln!("[cands]  pi{ci} gate{} hp{:.0} script {:x} cur{} hash {hsh:x} sph({:.2},{:.2},{:.2}) r{r2:.2} dcon{:.2}",
                            p.gate, p.hit_points, p.scripts[p.cur], p.cur, c2[0], c2[1], c2[2], dist3(c2, c0));
                    }
                    let glass = dp.weapons.records.get(sim.gun.weapon as usize)
                        .map_or(true, |w| u32::from_le_bytes(w[0x6c..0x70].try_into().unwrap()) & 0x800 == 0);
                    let mut clean = 0usize;
                    let mut world_ok = 0usize;
                    let mut room_ok = 0usize;
                    for rad in [6.0f32, 8.0, 10.0, 12.0, 14.0] {
                        for j in 0..24 {
                            let a = j as f32 * std::f32::consts::TAU / 24.0;
                            let eye = [c[0] + rad * a.sin(), c[1], c[2] + rad * a.cos()];
                            let st = [eye[0], eye[1] - eye_off, eye[2]];
                            let world = sphere_shot_ok(sim, eye, c, r);
                            let snap = snap_shot_ok(sim, eye, c, c0, r);
                            if world { world_ok += 1; }
                            if snap { clean += 1; }
                            if world && snap && (station_room(sim, sim.bond.pos, st) || station_room(sim, station, st)) { room_ok += 1; }
                            if snap && world { continue; }
                            let v = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
                            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
                            let raw = [eye[0] + v[0] / l * 256.0, eye[1] + v[1] / l * 256.0, eye[2] + v[2] / l * 256.0];
                            let (_, hits) = sim.drones.aim_snap_traced(&dp, eye, raw, glass);
                            let ws = hits.iter().map(|h| format!("{}:{:x}/p{}/t{:.3}", h.stage, h.id, h.pi, h.t)).collect::<Vec<_>>().join(" ");
                            eprintln!("[ring] con{k} t{tt} rad{rad:.0} b{j} eye({:.1},{:.1},{:.1}) world{world} hits [{ws}]", eye[0], eye[1], eye[2]);
                        }
                    }
                    eprintln!("[ring] con{k} t{tt}: {clean}/120 snap-clean, {world_ok}/120 world-clean, {room_ok}/120 shootable+roomed");
                }
                // The default station when its level shot is clear, unstealable by the aim snap
                // (the same candidates the snap sees) and the body fits; else (or once the float
                // has stalled 150 ticks short of it, or its LINE went dead — the snap steals
                // every tap from the stance it parked at) a planned one.
                let default_ok = sphere_shot_ok(sim, se, c, r) && snap_shot_ok(sim, se, c, c0, r)
                    && !banned.iter().any(|&b| dist3(b, station) < 1.0)
                    && station_room(sim, sim.bond.pos, station)
                    // hard: the default anchor is only taken when at most ONE shooter lines it —
                    // two or more lined muzzles at the ×1.0 pain row is the measured death shape;
                    // the planned pick below scores the exposure instead.
                    && (!hard || (exposure(sim, se).0 <= 1 && hurt_free(sim, se)));
                let goal = if plan.0 == k && plan.1.is_some() { plan.1.unwrap() } else { station };
                if goal != last_goal { last_goal = goal; moved_at = tt; last_goal_dist = f32::MAX; climbing_out = false; }
                // 1b-zero-g-floor item 1 (the two-phase goal — the report's own next lever):
                // while the goal is a LOW station (7/8, y −8.12, in front of the bank) and Bond
                // is still horizontally far and above it, the fly target is the CROSS COLUMN TOP
                // (the goal's own x/z at goal.y + 5) — the crossing rides ABOVE the bank's top
                // (the screens' face tops at y −5.21 at x 38.74) and the descent runs the
                // station's own column, which float-holds [L the driver's probes: (46, −8.3, 0)
                // and (46, −8.7, 0) hold in both PCSX2 and the port]. The old single-phase
                // descent crossed at the station's height band, clipped the bank's sloped
                // underside around (44.7, −8.66, 1.69) and the push equilibrium froze the wedge.
                let horiz = ((goal[0] - sim.bond.pos[0]).powi(2) + (goal[2] - sim.bond.pos[2]).powi(2)).sqrt();
                cross_high = goal[1] < -7.0 && horiz > 1.2 && sim.bond.pos[1] > goal[1] + 2.0;
                // The DEPARTURE-side mirror (the 7→1 leg): leaving the low band (the 7/8 stations
                // y −7) for a high goal (1/2 y −2), the straight line runs UNDER the bank's face
                // tops (from (46,−7) to (20,−2): y −5.6 at x 38.7 vs the face top −5.21, −3.1 at
                // 25.7 vs −2.91) and the radial push scrapes it — the measured leg cost 463
                // ticks. Same ride-above rule, latched (the start-guard pos.y < −7 breaks as the
                // climb passes −7): ride to goal.y + 5 until above goal.y + 2, then the plain
                // goal flight — the line from (46, 0) to (20,−2) clears every face top on the
                // span (y −0.56 at 38.7, −1.56 at 25.7).
                if sim.bond.pos[1] < -7.0 && goal[1] > sim.bond.pos[1] + 2.0 && horiz > 1.2 { climbing_out = true; }
                if climbing_out && (sim.bond.pos[1] >= goal[1] + 2.0 || horiz <= 1.2 || goal[1] < -7.0) { climbing_out = false; }
                let fly = if cross_high || climbing_out { [goal[0], goal[1] + 5.0, goal[2]] } else { goal };
                // The float's progress TOWARD THE FLY TARGET, not any movement: a wedged crawl
                // along the bank moved 0.3 every ~110 ticks and kept `moved_at` fresh, so the
                // stall never tripped and the replan never ran.
                let gd = dist3(sim.bond.pos, fly);
                if last_goal_dist - gd > 0.3 { moved_at = tt; }
                last_goal_dist = gd;
                if cross_high && !pocket_dumped {
                    pocket_dumped = true;
                    eprintln!("[gate] t{tt} con{k} two-phase crossing: cross column top ({:.2},{:.2},{:.2}) from ({:.2},{:.2},{:.2})", fly[0], fly[1], fly[2], sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
                }
                let snap_ok_here = snap_shot_ok(sim, sim.bond.head(), c, c0, r);
                if dist3(sim.bond.pos, goal) < 1.0 && !snap_ok_here { dead_ticks += 1 } else { dead_ticks = 0 }
                let stalled = (tt.saturating_sub(moved_at) > 150 && gd >= 1.0) || dead_ticks > 150;
                if stalled {
                    banned.push(goal); moved_at = tt; dead_ticks = 0;
                    if !pocket_probe_done { pocket_probe(sim, goal, k, tt, &mut pocket_probe_done); }
                }
                if !default_ok || stalled || (plan.0 == k && plan.1.is_some()) {
                    if plan.0 != k || stalled { plan = (k, level_station(sim, c, r, c0, sim.bond.pos, station, &banned, hard)); }
                    if let Some(p) = plan.1 { station = p; }
                }
                aim = Some(c);
                let clip = sim.gun.clip;
                // The burst gate stayed at 48 — MEASURED: lowering it to 36 (the three holds'
                // nominal 3×12 cost) regressed the mid-phase latches (5: +126, 8: +348 — the
                // earlier bursts restarted the recharge churn inside each hold chain); the
                // latches with 36 are in the branch history (run 1) vs 48 below.
                if clip >= 48 { in_burst = true; }
                if clip < 12 { in_burst = false; }
                let (o, f) = cal.ray(sim);
                // The round's miss distance from the sphere centre.
                let v = [c0[0] - o[0], c0[1] - o[1], c0[2] - o[2]];
                let along = v[0] * f[0] + v[1] * f[1] + v[2] * f[2];
                let miss = ((v[0] - along * f[0]).powi(2) + (v[1] - along * f[1]).powi(2) + (v[2] - along * f[2]).powi(2)).sqrt();
                // hard: STOP at the latch (d3-gated — the d1/d2/top-up timelines stay pinned).
                // Every 20-damage block on the screen resumes its
                // panel script and the resume's debris detonates at the screen (kind 107,
                // damage 20.0 radius 20.0 [L 1b-run4.log NF_DMG_TRACE: the blasts at 8502 sit
                // exactly on the x20 station (20.2,-1.9,-0.2) and at 10424 on the x46 nest
                // (46.1,-7.8,-0.2) — the screens Bond was shooting]) — and the walk kept
                // tapping AFTER the channel rose (run1's [hold] log: k5 holds #4/#5 answered
                // at 3095/3099, two full blocks past the 3093 latch). A player stops firing
                // the screen the frame it latches; the extra blocks only feed the blasts.
                let latched_k = sim.drones.channels.on(k);
                fire = (!hard || !latched_k)
                    && sphere_shot_ok(sim, o, c0, r) && along > 0.0 && dist3(sim.bond.head(), c) < 40.0 && in_burst
                    && miss < r * 0.5 && view_level(sim) && tt.saturating_sub(last_shot) >= 12 && tt % 6 == 0;
            }
            // The crossing rides the gap between the two panel rows (z ±1.1 [D the panels' glist
            // at z ±4.62]): the panels are solid and whole, so a float that drifts under a row
            // stalls against its underside.
            let crossing = station[2].abs() < 0.5 && (station[0] - sim.bond.pos[0]).abs() > 2.0 && sim.bond.pos[2].abs() > 0.4;
            let mut inp = float_to(sim, if crossing { [station[0], station[1], -8.0 * sim.bond.pos[2].signum()] } else { station });
            match aim {
                Some(c) => level_turn(sim, &mut inp, c, 0.08),
                None => inp.stabilise = sim.bond.look()[1].abs() > 0.005 && sim.bond.stabiliser == 0,
            }
            (inp, fire)
        };
        if fire {
            let c0 = w.scene.sim.as_ref().unwrap().gun.clip;
            let nb = w.scene.sim.as_ref().unwrap().drones.player_bullets.len();
            w.tick_fire(input);
            cal.learn(w.scene.sim.as_ref().unwrap(), nb);
            if w.scene.sim.as_ref().unwrap().gun.clip < c0 { last_shot = tt; }
        } else {
            w.tick(input, false);
        }
        // aim-route-3: sample the tick's SP_UnPause activations (the answered holds — see the
        // counter's note), the missiles' state-0 exits (the deadline anchor) and the launch
        // order's frame, right after the tick so the one-tick flags are still up.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            for (i, p) in sim.drones.scripts.players.iter().enumerate() {
                if p.activated {
                    answered[i] += 1;
                    if (31..=38).contains(&(p.gate as u32)) {
                        eprintln!("[hold] k{} hold #{} answered at frame {}", p.gate as u32 - 30, answered[i], sim.drones.frame);
                    }
                }
            }
            for (i, &d) in sim.drones.missile_deploy.iter().enumerate() {
                if d >= 1 && missile_start[i] == 0 { missile_start[i] = sim.drones.frame; }
            }
            if ch19_frame == 0 && sim.drones.channels.on(19) {
                ch19_frame = sim.drones.frame;
                eprintln!("[1b] ch19 (launch order) at frame {ch19_frame}");
            }
            // the -5 determinism item: ch 19 = AND(193, 194) is rewritten EVERY tick [M
            // triggers.rs case 3], so a missile-start shift (+200 on m3/m6/m8 in the lever runs)
            // means one of the inputs dropped for ~200 ticks — log every falling/rising edge
            // with both inputs' states to name the writer.
            if ch19_was {
                if !sim.drones.channels.on(19) {
                    ch19_drop_at = sim.drones.frame;
                    eprintln!("[1b] ch19 DROP at {} (193 {} 194 {})", sim.drones.frame,
                        sim.drones.channels.on(193), sim.drones.channels.on(194));
                }
            } else if sim.drones.channels.on(19) && ch19_frame != 0 {
                eprintln!("[1b] ch19 back ON at {} (dropped {} ticks, 193 {} 194 {})", sim.drones.frame,
                    sim.drones.frame - ch19_drop_at, sim.drones.channels.on(193), sim.drones.channels.on(194));
            }
            ch19_was = sim.drones.channels.on(19);
        }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    for &k in &CONSOLE_ORDER {
        if sim.drones.channels.on(k) && !latched.iter().any(|&(c, _)| c == k) {
            latched.push((k, sim.drones.frame));
            let dl = missile_start[(k - 1) as usize] as i64 + nf_game::space_missile::P0_LEN[(k - 1) as usize] as i64 - 1;
            eprintln!("[1b] latch k{k} frame {} deadline {dl} margin {}",
                sim.drones.frame, sim.drones.frame as i64 - dl);
        }
    }
    eprintln!("[1b] MissileDeploy {:?} (ch19 {ch19_frame}, missile starts {missile_start:?})", sim.drones.missile_deploy);
    latched
}

/// The live astronauts (skin 0x05000078) Bond can hurt: present, enabled, not a corpse.
fn live_astronauts(sim: &nf_view::scene::Sim) -> Vec<usize> {
    sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        .map(|(i, _)| i).collect()
}

/// The first present, enabled, live guard — the per-guard motion read [M DroneWeap_DoOpponentTargetting
/// 0x179a48:47-73: the latches live on every Drone]. The station's opening wave is present from the
/// arrival (the spawner walk built the list before the first tick).
fn first_tracker(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.list.iter().enumerate()
        .find(|(_, d)| d.present && !d.corpse && d.enabled && d.health > 0.0)
        .map(|(i, _)| i)
        .expect("a present guard in the station sim")
}

/// The round would land on drone `di`: the view ray (the laser flies straight — weapon 0x33's
/// spread +0x24 is 0 [D weapon_data]) enters one of its hit boxes before the world's first bullet
/// contact — the comparison the bullet pass itself runs (`exposed_aim`'s rule, along the view).
fn ray_on(sim: &nf_view::scene::Sim, di: usize) -> bool {
    let eye = sim.bond.head();
    let f = sim.bond.look();
    let ray = [f[0] * 40.0, f[1] * 40.0, f[2] * 40.0];
    let end = [eye[0] + ray[0], eye[1] + ray[1], eye[2] + ray[2]];
    let data = data_of(sim);
    let tw = sim.world.segment_bullet_scoped(sim.world.find_cel(eye), eye, end);
    sim.drones.list[di].hit_boxes(&data).iter()
        .filter_map(|b| nf_game::damage::ray_box(eye, ray, b))
        .any(|t| tw.map_or(true, |w| t < w))
}

/// How many live astronauts within 40 u have a bullet line from their head to the eye point `e`,
/// and how many float inside 3.5 u of it (the always-hit TooClose band and its doubled damage [M
/// DoBulletAccuracy 0x179740:113-115; Drone_ModBulletDamage 0x139fac-0x139fc8]).
fn exposure(sim: &nf_view::scene::Sim, e: [f32; 3]) -> (u32, u32) {
    let (mut seen, mut close) = (0, 0);
    for i in live_astronauts(sim) {
        let d = &sim.drones.list[i];
        let h = [d.pos[0], d.pos[1] + 0.6, d.pos[2]];
        let r = dist3(h, e);
        if r > 40.0 { continue; }
        if r < 3.5 { close += 1; }
        if sim.world.segment_bullet(h, e).is_none() { seen += 1; }
    }
    (seen, close)
}

/// A float point near `from` (a 1.5-u grid out to `rad`, at the heights `ys`) with room for the
/// body and a clear way there, scored by `score` (lower wins; the travel adds 0.05 a unit).
fn best_float_point(sim: &nf_view::scene::Sim, from: [f32; 3], rad: f32, ys: &[f32], score: impl Fn([f32; 3]) -> Option<f32>) -> Option<([f32; 3], f32)> {
    let mut best: Option<([f32; 3], f32)> = None;
    let n = (rad / 1.5) as i32;
    for ix in -n..=n {
        for iz in -n..=n {
            for &y in ys {
                let c = [from[0] + ix as f32 * 1.5, y, from[2] + iz as f32 * 1.5];
                let d = dist3(c, from);
                if d > rad * 1.05 { continue; }
                if d > 0.1 && !station_room(sim, from, c) { continue; }
                let Some(s) = score(c) else { continue };
                let s = s + 0.05 * d;
                if best.map_or(true, |b| s < b.1) { best = Some((c, s)); }
            }
        }
    }
    best
}

/// A level-look zero-g fight from the float station `hold`: the view only yaws (see the module
/// note), the lift thruster puts the eye at the target's height, and the trigger is tapped when
/// the view ray is on one of the target's hit boxes (`ray_on`). Target choice: an astronaut
/// inside the 3-u always-hit band first [M DoBulletAccuracy 0x179740:113-115], then the priority
/// target (Drake, skin 0x050000BA), then the nearest one with a bullet line to Bond, then the
/// nearest. Cover: on a dry clip (the laser's 100 rounds only recharge, 12 a tap) Bond floats to
/// the point near `hold` the fewest astronauts can see — losing sight resets each shooter's 2-s
/// new-sighting clock [M 0x179740:103-105] — and comes back out at 60 rounds. Returns at `done`
/// or the tick cap.
fn fight(w: &mut Walker, hold: [f32; 3], prio: Option<usize>, done: impl Fn(&nf_view::scene::Sim) -> bool, cap: u32) {
    fight_opts(w, hold, prio, false, done, cap)
}

/// `fight` with the Agent tactics switch (real-health-agent-difficulty-16-1b-41-4a, opt-in —
/// the Operative walks keep the master behaviour): `waves_first` scores a THREATENING wave
/// astronaut (its muzzle→Bond ray is clear) ABOVE the prio target. The master order fights the
/// prio target first (Drake at r−50 beats a threatening wave at r−20), which at Agent — the
/// astronauts' Accuracy Normal 0.8 [M DroneWeap_DoBulletAccuracy 0x179740:74-88] — let the
/// floaters grind the stand down (the measured death log: five astronauts' −3.5s at 5-11 u
/// while every aim re-pick still chose Drake). A player clears the add waves and takes the
/// boss last; the fight's `done` gate is the prio target's death, so the waves dying never
/// ends it early.
fn fight_opts(w: &mut Walker, hold: [f32; 3], prio: Option<usize>, waves_first: bool, done: impl Fn(&nf_view::scene::Sim) -> bool, cap: u32) {
    fight_opts_y(w, hold, prio, waves_first, done, cap, f32::MAX)
}

/// `fight_opts` with a float-height cap (0700001b 00 Agent, the k5-late-latch task): the view is
/// level-only (the aim arm lifts the eye TO the target's height), and once the SpaceLaser beam
/// arms (ch 19 [M 0x1aa3f8]; the 10 s fire → ch 18 [D the laser chain]) the BigLaserBeamHURT
/// band #259 (0xe4, type 1, dmg 50, gate ch 18 [D], box (8.5,9.0,−0.4)..(81.4,9.9,0.4) [L the
/// leg-start hurt dump]) is live across the whole level — the game's own contact law (the
/// feet..+1.8 r 0.55 capsule [M Player_DealWithObjHit 0x194458]) overlaps it from feet 6.65 up,
/// so a fight that lifts Bond to a ceiling-band astronaut (the measured death: the aim arm
/// parked him at feet 7.53 → capsule top 9.33, two −50.0 deliveries, dead at 3457) dies in it.
/// `max_y` clamps the body height (cover candidates and the aim lift both) and the aim pick
/// skips targets the capped stance cannot reach; `f32::MAX` (every other caller) is a no-op.
fn fight_opts_y(w: &mut Walker, hold: [f32; 3], prio: Option<usize>, waves_first: bool, done: impl Fn(&nf_view::scene::Sim) -> bool, cap: u32, max_y: f32) {
    let mut last_tap = 0u32;
    let mut target: Option<(usize, [f32; 3])> = None;
    let mut recharging = false;
    let mut cover = hold;
    for t in 0..cap {
        if done(w.scene.sim.as_ref().unwrap()) { return; }
        let sim = w.scene.sim.as_ref().unwrap();
        let eye_off = sim.bond.head()[1] - sim.bond.pos[1];
        if sim.gun.clip < 12 { recharging = true; }
        if sim.gun.clip >= 60 { recharging = false; }
        if (recharging || waves_first) && t % 30 == 0 {
            let p = sim.bond.pos;
            // waves_first: a hold that cannot ANSWER the prio target is as bad as an exposed one
            // (the first sheltered run floated out Drake's whole 6000-tick cap without a shot):
            // a blocked muzzle→prio ray costs 50, the same band as the aim pick's prio bonus.
            let prio_blind = |c: [f32; 3], eye_y: f32| -> bool {
                prio.is_some_and(|pr| {
                    let d = &sim.drones.list[pr];
                    d.present && !d.corpse && d.health > 0.0
                        && sim.world.segment_bullet([c[0], eye_y, c[2]], [d.pos[0], d.pos[1] + 0.6, d.pos[2]]).is_some()
                })
            };
            let sc = |c: [f32; 3]| {
                let (s, cl) = exposure(sim, [c[0], c[1] + eye_off, c[2]]);
                (s as f32 + 4.0 * cl as f32) + if prio_blind(c, c[1] + eye_off) { 50.0 } else { 0.0 }
            };
            let mut best = (sc(p), p);
            // The Agent Drake fight (prio set, waves_first): every point within 5 u of the hold was
            // blind to Drake (no shot in a whole cap, clip 100) - the ring reaches 8 and 11 u too.
            let rings: &[f32] = if waves_first && prio.is_some() { &[2.5, 5.0, 8.0, 11.0] } else { &[2.5, 5.0] };
            for &r in rings {
                for o in [[r, 0.0, 0.0], [-r, 0.0, 0.0], [0.0, r, 0.0], [0.0, -r, 0.0], [0.0, 0.0, r], [0.0, 0.0, -r]] {
                    let c = [hold[0] + o[0], (hold[1] + o[1]).min(max_y), hold[2] + o[2]];
                    if sim.world.segment(p, c).is_some() { continue; }
                    let s = sc(c) + 0.05 * r;
                    if s < best.0 { best = (s, c); }
                }
            }
            cover = best.1;
        }
        let dead_target = target.is_some_and(|(i, _)| { let d = &sim.drones.list[i]; d.corpse || d.health <= 0.0 });
        if t % 10 == 0 || dead_target {
            let e = sim.bond.head();
            let mut best: Option<(f32, usize, [f32; 3])> = None;
            let mut cands = live_astronauts(sim);
            if let Some(p) = prio { if !cands.contains(&p) { let d = &sim.drones.list[p]; if d.present && !d.corpse && d.health > 0.0 { cands.push(p); } } }
            for i in cands {
                let d = &sim.drones.list[i];
                let r = dist3(d.pos, e);
                if r > 40.0 { continue; }
                let Some(a) = w.exposed_aim(i) else { continue };
                // The capped float (fight_opts_y): a target the capped stance cannot reach is not
                // a target — the level-only view shoots at the eye's own height, so picking it
                // only grinds the cap away (the measured death: the aim lift into the live beam
                // band). `f32::MAX` callers never skip.
                if a[1] - eye_off > max_y + 0.5 { continue; }
                let threat = sim.world.segment_bullet([d.pos[0], d.pos[1] + 0.6, d.pos[2]], e).is_none();
                // waves_first banding: a THREATENING wave outranks a far prio target, but the
                // prio target CLOSING inside 8 u outranks the waves — Drake's own gun (weapon
                // 0x30, the measured −14.0 a hit at Agent = the record × Plr_DMod_Normal 0.7
                // [D weapon_data 0x30 +0x0c; M 0x1902c8]) is four waves' worth per hit, and the
                // first sheltered fight let him sit at 4.1 u for two of them (28.0 of the run's
                // last 30.0).
                let s = if r < 3.5 { r - 100.0 }
                    else if Some(i) == prio && r < 8.0 { r - 60.0 }
                    else if waves_first && threat { r - 30.0 }
                    else if Some(i) == prio { r - 50.0 }
                    else if threat { r - 20.0 }
                    else { r };
                if best.map_or(true, |b| s < b.0) { best = Some((s, i, a)); }
            }
            target = best.map(|b| (b.1, b.2));
        }
        let (input, fire) = {
            // waves_first (the Agent tactics): hold at the least-exposed point near `hold` from
            // the START, not only while the clip recharges — the hub float at Agent took the
            // whole wave ring's fire while the aim re-picks ground through the adds (the
            // measured deaths: 43 and 41 hits for 100.0). Drake closes in through the float, so
            // the sheltered hold keeps him shootable (the exposure scoring is the same
            // fewest-astronauts-see-me rule the recharge arm already used).
            let mut spot = if recharging || waves_first { cover } else { hold };
            if let Some((_, a)) = target { if !recharging { spot[1] = (a[1] - eye_off).min(max_y); } }
            let mut inp = thrust_to(sim, spot);
            let mut fire = false;
            match target {
                Some((i, a)) => {
                    level_turn(sim, &mut inp, a, 0.08);
                    fire = ray_on(sim, i) && view_level(sim) && sim.gun.clip >= 12 && t.saturating_sub(last_tap) >= 12;
                }
                None => inp.stabilise = sim.bond.look()[1].abs() > 0.005 && sim.bond.stabiliser == 0,
            }
            (inp, fire)
        };
        if fire { last_tap = t; w.tick_fire(input); } else { w.tick(input, false); }
    }
}

/// Wait out a phase in cover: float to the nearby point (out to `rad`, from 9 u below to 3 u
/// above) the fewest live astronauts can see, re-scored every 30 ticks with hysteresis, and shoot
/// only what the level view reaches from there — the eye stays at the cover height. The later
/// waves (spawners #165-181, ch 116-120 from the missile scripts [D]) float in the band above
/// the deck; under the deck and the rails every line from that band is cut.
/// floats to the least-exposed point in reach (`best_float_point`, the same rule the waves'
/// 2-s sighting clock rewards) and snipes whatever levels with the eye. Returns the last cover
/// point (the Agent Drake fight re-fights FROM it — the shelter the waves' LOS already
/// respects; see `fight_opts`).
fn hide_and_snipe(w: &mut Walker, rad: f32, done: impl Fn(&nf_view::scene::Sim) -> bool, cap: u32, hard: bool) -> Option<[f32; 3]> {
    hide_and_snipe_opts(w, rad, done, cap, hard, false)
}

/// `shift`: move off the held cover every 240 ticks (a fixed gun's explosive bolts - weapon 0x6b,
/// no drone owner - land on a spot held still; the Agent walk died to one at its 1316-tick wait).
fn hide_and_snipe_opts(w: &mut Walker, rad: f32, done: impl Fn(&nf_view::scene::Sim) -> bool, cap: u32, hard: bool, shift: bool) -> Option<[f32; 3]> {
    let mut last_tap = 0u32;
    let mut cover: Option<([f32; 3], f32)> = None;
    let mut cal = ShotCal::default();
    for t in 0..cap {
        if done(w.scene.sim.as_ref().unwrap()) { return cover.map(|c| c.0); }
        let sim = w.scene.sim.as_ref().unwrap();
        let eye_off = sim.bond.head()[1] - sim.bond.pos[1];
        let p = sim.bond.pos;
        let score = |c: [f32; 3]| { let (s, cl) = exposure(sim, [c[0], c[1] + eye_off, c[2]]); s as f32 * 3.0 + cl as f32 * 10.0 + if hard && !hurt_free(sim, c) { 1000.0 } else { 0.0 } };
        if t % 30 == 0 {
            let ys: Vec<f32> = (-6..=2).map(|k| p[1] + k as f32 * 1.5).collect();
            let here = cover.map(|(c, _)| (c, score(c)));
            let moving = shift && t % 240 == 0 && t > 0;
            let held = cover.map(|c| c.0);
            if let Some((c, s)) = best_float_point(sim, p, rad, &ys, |c| if moving && held.is_some_and(|h| dist3(h, c) < 3.0) { None } else { Some(score(c)) }) {
                cover = if moving { Some((c, s)) } else { match here { Some((hc, hs)) if hs <= s + 1.0 => Some((hc, hs)), _ => Some((c, s)) } };
            }
        }
        let spot = cover.map_or(p, |c| c.0);
        let e = sim.bond.head();
        let mut tgt: Option<(f32, usize, [f32; 3])> = None;
        for i in live_astronauts(sim) {
            let d = &sim.drones.list[i];
            let r = dist3(d.pos, e);
            if r > 40.0 { continue; }
            let Some(a) = w.exposed_aim(i) else { continue };
            if (a[1] - e[1]).abs() > 0.4 { continue; }
            if tgt.map_or(true, |b| r < b.0) { tgt = Some((r, i, a)); }
        }
        let mut inp = float_to(sim, spot);
        let mut fire = false;
        match tgt {
            Some((_, i, a)) => {
                level_turn(sim, &mut inp, cal.eye_target(sim, a), 0.3);
                fire = ray_on(sim, i) && view_level(sim) && sim.gun.clip >= 12 && t.saturating_sub(last_tap) >= 12;
            }
            None => inp.stabilise = sim.bond.look()[1].abs() > 0.005 && sim.bond.stabiliser == 0,
        }
        let nb = sim.drones.player_bullets.len();
        if fire { last_tap = t; w.tick_fire(inp); cal.learn(w.scene.sim.as_ref().unwrap(), nb); } else { w.tick(inp, false); }
    }
    cover.map(|c| c.0)
}

/// The mission route both full walks play (no position, substate or channel write after the
/// load): the first wave fought from the hub float → the launch order; the eight screens; the
/// later waves waited out in cover until the fight gate spawns Drake; Drake fought; the defeat
/// NIS (no skip, so its channel writes land) waited out in cover. Returns Drake's index.
fn play_the_mission(w: &mut Walker, alive: impl Fn(&nf_view::scene::Sim) -> bool + Copy) -> usize {
    play_the_mission_opts(w, alive, false, false)
}

/// `play_the_mission` with the Agent tactics switch: `waves_first` rides the Drake fight's aim
/// pick (see `fight_opts`) — the Operative walks keep the master order.
fn play_the_mission_opts(w: &mut Walker, alive: impl Fn(&nf_view::scene::Sim) -> bool + Copy, waves_first: bool, hard: bool) -> usize {
    let drake_of = |s: &nf_view::scene::Sim| s.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba);

    // 1. The first wave → the launch order.
    fight(w, [32.0, -1.7, 0.0], None, |s| { walk_log::leg("first-wave", s); s.drones.channels.on(19) || !alive(s) }, 4000);
    assert!(alive(w.scene.sim.as_ref().unwrap()), "Bond survives the first wave");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(19),
        "the first wave's death raises the launch order ch 19 [D spawner #163 p6 = 194, AND #202]");

    // 2. The consoles, in deadline order.
    let latched = shoot_consoles(w, &alive, hard);
    eprintln!("[1b] consoles latched (console, frame): {latched:?}; health {:.1} armour {:.1}",
        w.scene.sim.as_ref().unwrap().drones.health.health, w.scene.sim.as_ref().unwrap().drones.health.armour);
    for k in 1..=8u32 {
        assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(k), "console {k} latched by the real shot [D the console UI scripts' t682 write]");
    }
    assert!(alive(w.scene.sim.as_ref().unwrap()), "Bond survives the console run");
    // The disarm asserts assume every missile reached state 2 (all consoles latched above).
    // The gate-and-cover wait is a no-op at the tree's fast countdown and becomes the pacing
    // fix when the constant moves to REC_FRAME_RATE 0x30d0dc (seed·60 gated ticks — the
    // measured run in docs/reports/sp-panes-new-drawing.md).
    hide_and_snipe_opts(w, 12.0, |s| { walk_log::leg("missile-wait", s); s.drones.missile_deploy.iter().all(|&d| d == 2) || !alive(s) }, 12000, hard, waves_first && !hard);

    // 3. The disarm branch, the fly-by chain, Drake — the later waves waited out in cover.
    // The Agent tactics (waves_first): Drake is fought FROM the shelter the wave-wait just
    // proved — the deck cover the floaters' LOS already respects — not from the open hub (the
    // measured Agent deaths: 43/41/39 hits for 100.0 at the hub hold, the wave ring grinding
    // the stand down while Drake was the only target). The prio-aware cover pick in fight_opts
    // keeps a hold that can answer him.
    let drake_hold = if waves_first { hide_and_snipe(w, 12.0, |s| { walk_log::leg("waves-wait", s); drake_of(s).is_some() || !alive(s) }, 6000, hard) } else { hide_and_snipe(w, 12.0, |s| { walk_log::leg("waves-wait", s); drake_of(s).is_some() || !alive(s) }, 6000, hard); None };
    let drake = {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(alive(sim), "Bond survives the later waves to the fly-by (health {:.1})", sim.drones.health.health);
        assert!(sim.drones.missile_deploy.iter().all(|&d| d == 2),
            "every missile took the DISARM branch (MissileDeploy 2 [M 0x1aa2cc]): {:?}", sim.drones.missile_deploy);
        assert!(sim.drones.channels.on(46), "missile 1's disarm script writes SP#211's start ch 46 [D 0600091c @t1000]");
        assert!(sim.drones.channels.on(10), "the fly-by chain writes the fight gate ch 10 [D 0x060008f0 t400]");
        assert!(sim.drones.mission.status() <= 1, "no fail on the way (status {}) [M 0x182c60]", sim.drones.mission.status());
        // The game's order, pinned mid-walk on the state the objective monitor reads [M
        // Mission_MonitorObjectives 0x1827d8]. The fight gate does NOT wait for objective 1: the
        // done channels that feed AND #139 (ch 21/22/27) are written only when each missile's p2
        // script ENDS [M 0x1aa33c-3d0 — Script_Update; Script_IsPlaying; bne v0 → the writes sit
        // behind it: sb 1 → switch_channels[16(s0)] = p6 at 0x1aa3a0-3bc (disarm) /
        // switch_channels[14(s0)] = p5 at 0x1aa380-388 (launch); obj+0xf4 = 4 at 0x1aa3d4-3d8],
        // and missile 1's p2 writes the Drake-chain ch 46 @t1000 but ends at t1710 [D 0600091c],
        // so ch 10 → Drake lands ~710 ticks BEFORE ch 21. So while Drake lives: obj 1 is merely
        // announced (3) and obj 2 still hidden (1 — its enable ch 0x09 has not fired [L record 1b
        // obj 2; M 0x1827d8:55-67]). The monitor completes obj 1 (record 4 [M 0x1827d8:78-85])
        // later, when the three p2 ends raise ch 0x09.
        assert!(!sim.drones.channels.on(0x09) && !sim.drones.channels.on(0x0b),
            "at the fight gate neither objective channel has fired: ch 0x09 waits on the p2 ends [M 0x1aa33c-3d0], ch 0x0b on Drake's death [D]");
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "objective 1 is announced but NOT complete while Drake spawns [M 0x1827d8:41-54/78-85]");
        assert_eq!(sim.drones.mission.states[23][1], 1,
            "objective 2 is still hidden (its enable ch 0x09 has not fired) [L record 1b obj 2; M 0x1827d8:55-67]");
        drake_of(sim).expect("the fight gate spawns Drake (spawner #187, trigger 10 [D])")
    };
    assert!(w.scene.sim.as_ref().unwrap().drones.list[drake].enabled, "the gate enables Drake (wait-switch 10 [D])");
    eprintln!("[1b] Drake spawned at frame {}; health {:.1}", w.scene.sim.as_ref().unwrap().drones.frame, w.scene.sim.as_ref().unwrap().drones.health.health);

    // 4. Drake, then the defeat NIS plays out (no skip, so its channel writes land).
    w.skip_cutscenes = false;
    fight_opts(w, drake_hold.unwrap_or([32.0, -1.7, 0.0]), Some(drake), waves_first, |s| { walk_log::leg("drake-fight", s); s.drones.list[drake].corpse || s.drones.list[drake].health <= 0.0 || !alive(s) }, 6000);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(alive(sim), "Bond survives Drake's fight (health {:.1})", sim.drones.health.health);
        assert!(sim.drones.list[drake].health <= 0.0, "Drake dies to Bond's fire in the float");
        assert!(sim.drones.channels.on(0x0e), "Drake's death channel ch 0x0e rises with the kill [D #186 p12 = 14; M 0x1747b8/0x13e660]");
        // The order's middle hinge, in the window between Drake's death (ch 0x0e) and the defeat
        // NIS's ch 0x0b write (its t600 [D 0x06000a20] — the NIS only starts on 0x0e, so 0x0b is
        // still 600 ticks out): objective 2 is announced (its enable ch 0x09 fired [M
        // 0x1827d8:55-67]) and not yet complete, while objective 1 completes on the monitor's ch
        // 0x09 read [M 0x1827d8:78-85] — the game's order: obj 1 completes BEFORE obj 2.
        assert!(!sim.drones.channels.on(0x0b),
            "the defeat NIS's ch 0x0b write sits at its t600, after the death tick [D 0x06000a20]");
        // On the disc clocks m1/m2/m7 disarm at their own state-2 entries (p0 = 3300 ticks [D
        // P0_LEN] after armed+6300 — ~launch+9600, past Drake's spawn), so the last p2 end
        // (m1's, disarm+1710 [D 0600091c]) lands ~450 ticks AFTER the fight gate — a fast kill
        // (measured on master's raw aim: Drake died inside that window) precedes ch 0x09. The
        // order that always holds: ch 0x09 (obj 1 complete) strictly before the NIS's 0x0b
        // (obj 2's completion read) — 0x0b sits at death+600 [D 0x06000a20], ch 0x09 within
        // ~450 of the death. Wait for the monitor's record read bounded inside that gap
        // (550 < 600) and pin obj 2 still un-complete.
        for _ in 0..550u32 {
            { let sim = w.scene.sim.as_ref().unwrap(); walk_log::leg("obj1-wait", sim); if sim.drones.mission.states[23][0] == 4 { break; } }
            w.tick(Default::default(), false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x09) && sim.drones.mission.states[23][0] == 4,
            "the p2 ends raised ch 0x09 and the monitor completed objective 1 before the NIS's 0x0b [M 0x1aa33c-3d0 / 0x1827d8:78-85 + D the p2 lengths/P0_LEN]");
        assert_eq!(sim.drones.mission.states[23][0], 4,
            "objective 1 is COMPLETE once ch 0x09 reads [M 0x1827d8:78-85]");
        assert_eq!(sim.drones.mission.states[23][1], 3,
            "objective 2 is announced and not complete while the NIS's 0x0b is still 600 ticks out [M 0x1827d8:55-67]");
    }
    hide_and_snipe(w, 12.0, |s| { walk_log::leg("end-nis", s); s.drones.mission.status() == 6 || !alive(s) }, 2400, false);
    w.skip_cutscenes = true;
    drake
}

/// The walk-time hunt's per-leg log (task level-1b-drake-walk-time): each route leg calls
/// `leg(name, sim)` once per sim tick; the log prints the leg's enter/exit with wall seconds and
/// a full Bond/wave/Drake/mission dump every 600 frames. Stderr only — no sim state touched, no
/// RNG draws, and the wall clock lives here, never in Sim.
mod walk_log {
    use std::cell::RefCell;
    use std::time::Instant;

    struct Leg { name: &'static str, ticks: u32, bucket: u32, start: Instant }

    thread_local! {
        static T0: RefCell<Option<Instant>> = const { RefCell::new(None) };
        static LEG: RefCell<Option<Leg>> = const { RefCell::new(None) };
    }

    pub fn leg(name: &'static str, sim: &nf_view::scene::Sim) {
        let now = Instant::now();
        let total = T0.with(|t| *t.borrow_mut().get_or_insert(now)).elapsed().as_secs_f32();
        LEG.with(|l| {
            let mut l = l.borrow_mut();
            match l.as_mut() {
                Some(cur) if cur.name == name => {
                    cur.ticks += 1;
                    let b = sim.drones.frame / 600;
                    if b != cur.bucket { cur.bucket = b; dump(name, sim, total, cur.start.elapsed().as_secs_f32()); }
                }
                Some(cur) => {
                    eprintln!("[leg] {} END at frame {} — {} leg ticks, {:.1}s leg wall, {total:.1}s total wall",
                        cur.name, sim.drones.frame, cur.ticks, cur.start.elapsed().as_secs_f32());
                    *cur = Leg { name, ticks: 0, bucket: sim.drones.frame / 600, start: now };
                    dump(name, sim, total, 0.0);
                }
                None => {
                    eprintln!("[leg] {name} ENTER at frame {}, {total:.1}s total wall", sim.drones.frame);
                    *l = Some(Leg { name, ticks: 0, bucket: sim.drones.frame / 600, start: now });
                }
            }
        });
    }

    fn dump(name: &'static str, sim: &nf_view::scene::Sim, total: f32, leg_wall: f32) {
        let dr = &sim.drones;
        let live = super::live_astronauts(sim).len();
        let drake = dr.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba)
            .map(|i| { let d = &dr.list[i];
                format!("#{} hp{:.0}@({:.1},{:.1},{:.1})", i, d.health, d.pos[0], d.pos[1], d.pos[2]) });
        let ch = |k: u32| if dr.channels.on(k) { '1' } else { '.' };
        let b = sim.bond.pos;
        eprintln!(
            "[leg] {name} f{} {:.1}s/{total:.1}s | bond ({b:.1?}) ss{} hp{:.1} ar{:.1} clip{} | astro {live} | drake {drake:?} | deploy {:?} | ch19{} 46{} 10{} 09{} 0e{} 0b{} | mst {} obj({},{})",
            dr.frame, leg_wall, sim.bond.substate, dr.health.health, dr.health.armour, sim.gun.clip,
            dr.missile_deploy, ch(19), ch(46), ch(10), ch(0x09), ch(0x0e), ch(0x0b),
            dr.mission.status(), dr.mission.states[23][0], dr.mission.states[23][1]);
    }
}

/// The launch order is the FIRST WAVE's death, on disc data alone: the arrival script
/// 0x0600085c writes ch 190/191 [D stream 6/9 ev0x0d], ch 191 gates SP#148 (0x060009a4 [D p9])
/// whose streams write ch 115 (spawner #163's trigger [D p4 = 0x73]) and ch 193 [D streams 1/3];
/// spawner #163's DONE channel is ch 194 [D p6 = 0xc2] — written when its spawned wave is dead
/// [M DroneSpawner 0x137e68:126-143, the `p6 != 0` wait] — and AND #202 folds ch 193 & 194 into
/// ch 19 [D p0 = 0x13, p1/p2 = 0xc1/0xc2], the missiles' launch-order gate [D p7 = 19; M
/// 0x1aa1c8]. The older "ch 19 has no disc writer" reading only swept the scripts. Sister: with
/// the wave alive, ch 194 / ch 19 stay off and every missile sits in state 0.
#[test]
fn the_first_wave_kill_raises_the_launch_order() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "the p3 == 2 start is the zero-g substate 8 [M Player_Init 0x1910f8 :160-165]");
    let mut spawned = false;
    for _ in 0..3000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.list.iter().any(|d| d.present && !d.corpse && d.skin & 0xffff == 0x78) { spawned = true; break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(spawned, "ch 115 (SP#148's write [D 060009a4 stream 1]) spawns the first wave [D spawner #163 p4 = 0x73]");
        assert!(sim.drones.channels.on(115) && sim.drones.channels.on(191), "the chain's head: ch 191 → SP#148 → ch 115 [D]");
    }
    // Sister: the wave alive — 1500 ticks idle, no launch order.
    for _ in 0..1500u32 { w.tick(Default::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(193), "SP#148's ch 193 write lands [D 060009a4 stream 3]");
        assert!(!sim.drones.channels.on(194) && !sim.drones.channels.on(19),
            "sister: the live wave holds the spawner's done write [M 0x137e68:126-143] — no ch 194, no ch 19");
        assert!(sim.drones.space_missiles.iter().all(|m| m.state == 0), "sister: the missiles wait on ch 19 [M 0x1aa1c8]");
    }
    let hold = w.scene.sim.as_ref().unwrap().bond.pos;
    fight(&mut w, hold, None, |s| s.drones.channels.on(19), 2400);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(194), "the dead wave writes spawner #163's done channel 194 [D p6; M 0x137e68]");
    assert!(sim.drones.channels.on(19), "AND #202 (193 & 194) raises the launch order ch 19 [D]");
}

/// The whole mission on real input — no position, substate or channel write after the load.
/// The station is a zero-g level from the start (the p3 == 2 spawn enters substate 8 [M
/// Player_Init 0x1910f8 :160-165]) and the route never needs the shaft's far side: Drake comes
/// to Bond.
///
/// 1. The arrival plays; the first wave (spawner #163) attacks; killing it raises ch 194 →
///    AND #202 → ch 19, the launch order [D — the sister test above].
/// 2. The eight missiles leave state 0 [M 0x1aa1c8]; their p0 scripts write the console gates
///    31..38 at t359 [D], the console UI scripts (SP#213-220, hit points 20) play and hold at
///    t650; Bond floats to the stations and shoots the screens (20 damage resumes the script
///    [M SP_UnPause 0x1dca58], the t682 write latches the console channel [D]) before each
///    missile's p0 ends (state 2 reads the channel [M 0x1aa25c]) — all eight take the DISARM
///    branch (MissileDeploy 2 [M 0x1aa2cc]).
/// 3. Missile 1's disarm script writes ch 46 [D 0600091c @t1000] — SP#211's start (gate ch 30,
///    console 1's UI write [D 060007e2]); the fly-by chain 0x060007fd → 0x060008f0 writes the
///    fight gate ch 10 at its t400 [D]; spawner #187 (trigger 10 [D]) spawns Drake, who closes
///    in through the float. The done channels 21/22/27 land → AND #139 → ch 9, objective 1 [D].
/// 4. Bond shoots Drake (the real fire path): his death channel 14 [D #186 p12], the defeat NIS
///    (SP#198) writes ch 0x0b [D 0x06000a20], the monitor's ch 9 & 0x0b → ch 100 → MissionState
///    6 → 0x0730001b [M 0x182c60:114-116].
///
/// The walker's health allowance stays on (damage suppressed — the walkreal convention); the
/// real-health run of the same route, with the later waves (spawners #165-181, ch 116-120 from

/// `the_first_wave_kill_raises_the_launch_order` on real damage (common/damage_log.rs: no top-up,
/// Bond's death panics with the death line): the level-look fight from the start float kills the
/// first wave before its rounds get through.
#[test]
fn the_first_wave_kill_raises_the_launch_order_real_damage() {
    walker::with_real_damage(the_first_wave_kill_raises_the_launch_order);
}

/// The whole mission on real input — no position, substate or channel write after the load.
/// The station is a zero-g level from the start (the p3 == 2 spawn enters substate 8 [M
/// Player_Init 0x1910f8 :160-165]) and the route never needs the shaft's far side: Drake comes
/// to Bond (`play_the_mission`).
///
/// 1. The arrival plays; the first wave (spawner #163) attacks; killing it raises ch 194 →
///    AND #202 → ch 19, the launch order [D — the sister test above].
/// 2. The eight missiles leave state 0 [M 0x1aa1c8]; their p0 scripts write the console gates
///    31..38 at t359 [D], the console UI scripts (SP#213-220, hit points 20) play and hold at
///    t650; Bond floats to the stations and shoots the screens (20 damage resumes the script
///    [M SP_UnPause 0x1dca58], the t682 write latches the console channel [D]) before each
///    missile's p0 ends (state 2 reads the channel [M 0x1aa25c]) — all eight take the DISARM
///    branch (MissileDeploy 2 [M 0x1aa2cc]).
/// 3. Missile 1's disarm script writes ch 46 [D 0600091c @t1000] — SP#211's start (gate ch 30,
///    console 1's UI write [D 060007e2]); the fly-by chain 0x060007fd → 0x060008f0 writes the
///    fight gate ch 10 at its t400 [D]; spawner #187 (trigger 10 [D]) spawns Drake, who closes
///    in through the float. The done channels 21/22/27 land → AND #139 → ch 9, objective 1 [D].
/// 4. Bond shoots Drake (the real fire path): his death channel 14 [D #186 p12], the defeat NIS
///    (SP#198) writes ch 0x0b [D 0x06000a20], the monitor's ch 9 & 0x0b → ch 100 → MissionState
///    6 → 0x0730001b [M 0x182c60:114-116].
///
/// Every shot is taken with the view level (the module note's level look), so it lands where the
/// view looks with either zero-g look composition. The walker's health allowance stays on
/// (damage suppressed — the walkreal convention); `walk_arrival_to_drakes_defeat_on_real_health`
/// plays the same route on real health.
#[test]
fn walk_arrival_to_drakes_defeat() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, SS_ZEROG_8, "the station starts in the zero-g substate [M Player_Init 0x1910f8 :160-165]");
        assert_eq!(sim.drones.mission.states[23][0], 3,
            "'Disable the missiles' shows at once (enable 0 [L], the state-0 arm [M 0x1827d8:41-54])");
        assert_eq!(sim.drones.mission.states[23][1], 1, "'Defeat Drake' hides until ch 0x09 [L record 1b obj 2]");
        assert!(view_level(sim) && sim.bond.pitch == 0.0, "the float starts level: the entry matrix is RotMatrix(−pitch·π/2, yaw, 0) of the walk's 0 pitch");
    }
    play_the_mission(&mut w, |_| true);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x09), "objective 1: AND #139 (21 & 22 & 27) → ch 9 [D]");
    assert!(sim.drones.channels.on(0x0b), "the defeat NIS writes ch 0x0b [D 0x06000a20]");
    assert!(sim.drones.channels.on(100), "ch 100, the mission end [M 0x1827d8:120 / D 0x06000a20]");
    assert_eq!(sim.drones.mission.status(), 6, "the mission ends (MissionState 6) [M 0x182c60]");
    assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    // Both objectives COMPLETE on the records the monitor reads, in the game's order (the
    // mid-walk hinges pinned obj 1 = 3 / obj 2 = 1 at Drake's spawn and obj 1 = 4 / obj 2 = 3 at
    // his death):
    // the monitor's state-3 arm turns each record 4 when its channel reads ON [M 0x1827d8:78-85].
    assert_eq!(sim.drones.mission.states[23][0], 4,
        "objective 1 'Disable the missiles' is COMPLETE on the record [M 0x1827d8:78-85; L record 1b obj 1]");
    assert_eq!(sim.drones.mission.states[23][1], 4,
        "objective 2 'Defeat Drake' is COMPLETE on the record [M 0x1827d8:78-85; L record 1b obj 2]");
    // The channels of record, stamped on every nonzero write [M 0x1b1a78:87,107,188-189]: the
    // defeat NIS writes 0x0b at t600 and the end ch 100 at t720 [D 0x06000a20], so obj 2's
    // completion lands strictly before the mission-end channel's last stamp.
    let t = |c: u32| sim.drones.channels.time.get(c as usize).copied()
        .unwrap_or_else(|| panic!("ch {c} was written on the walk"));
    assert!(t(0x0b) < t(100),
        "obj 2's completion channel 0x0b lands before the mission-end ch 100 [D 0x06000a20 t600/t720] ({} vs {})", t(0x0b), t(100));
}

#[test]
fn the_zero_g_entry_floats_bond_off_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "the p3 == 2 start is substate 8 [M Player_Init 0x1910f8 :160-165]");
    float_to_slit_mouth(&mut w);
    // Sister: the ch-75 rise is not a substate change.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x4b, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "sister: a ch-75 write moves no substate [M 0x216388 is the menu's msg-0x4b arm]");
    let mut rose = false;
    for _ in 0..560u32 {
        w.tick(Input { lift: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().bond.pos[1] > 2.0 { rose = true; break; }
    }
    assert!(rose, "the float rises off the deck band under the lift thruster (y {:.2})",
        w.scene.sim.as_ref().unwrap().bond.pos[1]);
}

/// The sealed-shaft wall, decided: the face that stops the float at x ≈ 4.98
/// IS the game's own boundary, and the game's body collides with it — the port's capsule stop is
/// the game's behaviour, not a port artefact.
///
/// [D probe1b_wall] the face is plc #128 OBJECT_Boundary_1 (kind 0x8019, cel 0x8000, glist 0x0):
/// a two-triangle pair at x 4.433 spanning y −16.56..+16.74, z −20.40..+20.40 (both stored
/// normals face EAST, +x), surface byte 0x40 — present in the level archive at the slit band,
/// at the mouths' height (y 9.45) and everywhere between.
///
/// [M] the game's rules stop the player body on it: the surface byte's 0x40 sets HITDATA+0x51
/// bit 1 on every hit (Collide_Intersect 0x1ebaa8 disasm 0x1ebf78-0x1ebf80) and that bit drops
/// hits only for the HITTEST+0x98 bit-0x8 bullet queries (0x1ebff0-0x1ec010) — the player body
/// rides Collide_CylinderIntersect 0x1ecc00:61's kind 0x800, which carries no bit 0x8;
/// Intersect_CylGeom 0x109da0's per-triangle loop rejects no face by side (the head-on cull
/// :302-322 only skips faces within ~2.6° of horizontal), and Player_ZeroG 0x192ab0 composes
/// only thrust/rotation — the shared movement applies the same body collision. The z ±8.82
/// mouths do not reopen the route: the iris doors (SP#141/#142's meshes, x 5.23 [D]) sit EAST
/// of the boundary, and the boundary spans the doors' whole y band — open doors dead-end at it.
#[test]
fn the_boundary_wall_stops_the_float_and_passes_bullets() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    // The body ray east→west hits the 4.433 plane across the slit band and at the mouths'
    // height, with the boundary's surface byte [D mat 0x40].
    for (y, z) in [(-2.4f32, 0.0), (0.0, -4.0), (2.5, 0.0), (9.45, 8.82), (9.45, -8.82)] {
        let (t, n, mat, _) = sim.world
            .segment_surface_flags([5.5, y, z], [3.5, y, z], true)
            .unwrap_or_else(|| panic!("the boundary ray (y {y}, z {z}) must hit [D plc #128 at x 4.433]"));
        let x = 5.5 - t * 2.0;
        assert!((x - 4.433).abs() < 0.01, "hit at x {x:.3} = the boundary plane 4.433 [D plc #128], got t {t}");
        assert!((n[0] - 1.0).abs() < 0.01, "the stored normal faces east [D], got ({:.2},{:.2},{:.2})", n[0], n[1], n[2]);
        assert_eq!(mat & 0x3f, 0, "the boundary's surface index is 0 [D mat 0x40]");
        assert!(mat & 0x40 != 0, "the boundary carries the 0x40 surface flag [D]");
    }
    // Sisters (the game's ray rules): the bullet query passes (the 0x40 drop [M 0x1ebff0]) and
    // the west→east ray passes (the stored normals face east; the ray test skips n·dir ≥ 0
    // [M 0x10a8c8:168]).
    for (y, z) in [(0.0f32, 0.0), (9.45, 8.82), (9.45, -8.82)] {
        // The bullet is never stopped AT the boundary: the 0x40 drop [M 0x1ebff0-0x1ec010]
        // releases the hit and the ray runs on (west of the boundary the 0x19 grills at x ≈ 3.8
        // [D] can catch it — that is not the boundary).
        if let Some(t) = sim.world.segment_bullet([5.5, y, z], [3.5, y, z]) {
            let x = 5.5 - t * 2.0;
            assert!(x < 4.4, "the bullet passes the boundary westward (the 0x40 drop [M 0x1ebff0-0x1ec010]) \
                at y {y} z {z} — stopped at x {x:.3} instead");
        }
        assert!(sim.world.segment([3.5, y, z], [5.5, y, z]).is_none(),
            "the west→east ray passes the east-facing boundary [M 0x10a8c8:168] at y {y} z {z}");
    }
    drop(sim);
    // The float, real thrust only: fly from the load's substate 8 to the slit mouth, lift clear
    // of the deck band, then thrust west. The boundary stops the capsule at
    // 4.433 + 0.55 (the body radius) ≈ 4.98 — min x must land in that window, never past it.
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    float_to_slit_mouth(&mut w);
    let mut min_x = f32::MAX;
    let mut lane = [0.0f32, 0.0, -0.6];
    for t in 0..1100u32 {
        // Lift first, then thrust due west along the lifted height (the body-axis thrusters
        // toward a far-west point: the float's facing is the spawn's, not west).
        let input = if t < 220 { Input { lift: 1.0, ..Default::default() } }
                    else { thrust_to(w.scene.sim.as_ref().unwrap(), lane) };
        if t == 219 { lane[1] = w.scene.sim.as_ref().unwrap().bond.pos[1]; }
        w.tick(input, false);
        let sim = w.scene.sim.as_ref().unwrap();
        min_x = min_x.min(sim.bond.pos[0]);
    }
    assert!(min_x > 4.85 && min_x < 5.15,
        "the float thrusts west and the boundary stops the capsule at 4.433 + 0.55 ≈ 4.98 \
         [D plc #128; M the kind-0x800 body, no bit-0x8 drop; M CylGeom two-sided], got min x {min_x:.3}");
}

/// The sealing completes this task's scan: the boundary box (plc #128 OBJECT_Boundary_1 + its
/// end-walls) is closed on every face for the player's body. The west wall (x 4.433), the shaft's
/// top slab (y 16.74, normal down) and the pit floor (y −16.56) are the sisters above/`probe1b_wall`;
/// THIS test pins the two END-WALLS at z ±20.40 (m40) and their only openings — deck-level windows
/// ~0.8-1.2 tall — against the body's own height [M Player_Collision 0x194df0's default arm: the
/// capsule A = pos + up·0.55, B = pos + up·(+0x7CC − cc), radius 0.55 [M 0x3f0ccccd], standing cc
/// ≈ 1.0347 [L ee_start_05.bin BL 0x16cf0f0 per docs/fields/obj-tag-player.md] — ~1.58 tall,
/// taller than every window band]. With no script player-write on the disc [D the walk's doc
/// comment sweep], the box is sealed for real-input crossing; the game's own crossing (if any)
/// must use a body shape or a mechanism outside this scan — the report's live checks.
#[test]
fn the_end_walls_seal_the_shaft_box() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let w = &sim.world;
    // The end-walls exist at z ±20.40 across the shaft's x/y span (both ray directions — the
    // segment query is one-sided [M 0x10a8c8:168]).
    for (x, y) in [(6.5f32, -9.5), (6.5, 0.5), (6.5, 8.5), (10.0, -6.5), (14.0, -3.5), (19.0, 2.5), (23.0, -9.5)] {
        for zs in [20.4f32, -20.4] {
            let (a, b) = if zs > 0.0 { (26.0, 14.0) } else { (-26.0, -14.0) };
            let mut hit = None;
            for (p, q) in [(a, b), (b, a)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z - zs).abs() < 0.6 { hit = Some((z, mat)); break; }
                }
            }
            let (z, mat) = hit.unwrap_or_else(|| panic!("the end-wall at z {zs:+.1} must face ({x:.1},{y:+.1}) [D probes]"));
            assert!((z - zs).abs() < 0.6 && mat & 0x3f == 0 && mat & 0x40 != 0,
                "the end-wall face at ({x:.1},{y:+.1}) sits at z {z:+.2} with the boundary material [D m40], got m{mat:02x}");
        }
    }
    // The only openings: deck-level windows, y bands [D 0.2-step scans]:
    //   x 7.5/8.0: y −1.8..−1.0 (0.8 tall); x 9.0: −2.0..−1.2; x 10.0: −2.2..−1.2 (1.0 tall).
    for (x, y_lo, y_hi) in [(7.5f32, -1.8, -1.0), (8.0, -1.8, -1.0), (10.0, -2.2, -1.2)] {
        let mut y = y_lo;
        while y <= y_hi + 0.01 {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            assert!(!hitz, "the window band at (x {x:.1}, y {y:+.1}) must be open [D probes]");
            y += 0.4;
        }
        // The band's edges are wall (the window is bounded).
        for y in [y_lo - 0.6, y_hi + 0.6] {
            let mut hitz = false;
            for (p, q) in [(-26.0f32, -14.0), (-14.0, -26.0)] {
                if let Some((t, n, mat, _)) = w.segment_surface_flags([x, y, p], [x, y, q], true) {
                    let z = p - t * (p - q);
                    if (z + 20.4).abs() < 0.6 { hitz = true; break; }
                }
            }
            assert!(hitz, "the window band's edge at (x {x:.1}, y {y:+.1}) is end-wall [D probes]");
        }
        assert!(y_hi - y_lo < 1.3, "the window at x {x:.1} is {:+.1} tall [D probes] — narrower than the body", y_hi - y_lo);
    }
    // The body cannot pass any band: the capsule's total height ~1.58 [M 0x194df0's default arm]
    // exceeds every window (0.8..1.0 [D]).
    let capsule_height = 0.55 + (1.0347 - 0.55) + 2.0 * 0.55; // A to B span + the radius bulge at both ends
    assert!(capsule_height > 1.3,
        "the float body (~{capsule_height:.2} tall [M Player_Collision 0x194df0 default arm + cc 1.0347 [L]]) exceeds the widest window (1.0 [D])");
}

/// The iris doors, decided: the z ±8.82 booths' doors — SP#141/#142
/// "ScriptPlayer_IrisDoorOpen" [D placement names/positions (5.23, 9.45, ∓8.82)], script
/// 0x06000818 [D p0] — are LOAD-TIME LOOPERS, not missile-triggered pieces: the create runs
/// `SP_LoadScript` for mode ∉ {0,1} [M SP_CreateScriptPlayer 0x1db9a8 tail] and its mode-8 arm
/// starts the script LOOPING (`Script_Play(script, 1)`) in state 6 [M SP_LoadScript 0x1dbc08:46-48].
/// The script's per-loop shape [D decode]: the 8 iris-panel entities (0x02000834..3b) animate
/// OPEN at keys 0..120, the script HOLDS at t208 (stream 9 ev0x03 p0=0 — the −1 delay), the
/// panels CLOSE at 480..600 with the door sounds (stream 10: 0x000004c4 at start, 0x000004c3 at
/// t480), and the missile-done channels 0x18/0x1c are CLEARED at t550 (streams 12/13) — each
/// ~10 s loop clears the missile bookkeeping. The channel-RISE restart does NOT apply: [M
/// SP_Update 0x1dbda8 case 6] reloads only through the OTHER script slot, and both SPs carry a
/// single script (hash[1] = 0 [D live players]) — the rise never starts them. The doors are NOT
/// the route west either way: their meshes sit at x 5.23, east of the boundary [D probe1b_wall;
/// the boundary pin test above].
#[test]
fn the_iris_doors_are_load_time_loopers() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The create/load semantics: primed PLAYING loopers (state 6 = the mode-8 arm's
        // Script_Play(loop=1) [M 0x1dbc08:46-48]) on their own out channels [D p6].
        for plc in [141usize, 142] {
            let p = sim.drones.scripts.players.iter().find(|p| p.placement == plc).unwrap();
            assert_eq!(p.scripts[0], 0x0600_0818, "SP#{plc} carries the iris-door script [D p0]");
            assert_eq!(p.modes[0], 8, "SP#{plc} is a mode-8 looper [D p2]");
            assert_eq!((p.state, p.info[0].is_some()), (6, true),
                "SP#{plc} primed PLAYING (state 6, script loaded) from the level load [M 0x1db9a8 + 0x1dbc08]");
            assert_eq!(p.out, if plc == 141 { 0x1c } else { 0x18 }, "SP#{plc} fires on its own out channel [D p6]");
        }
        // The 8 iris-panel entities exist (the playing script created them [D streams 1-8,
        // hashes 0x02000834..3b]). Their stored matrices are the streams' local keys — the world
        // placement folds the parent chain (Script_Interp's live-matrix walk), so only the
        // existence and the hash band are pinned here.
        let mut panels = 0usize;
        for o in sim.drones.scripts.objects.list.iter().flatten() {
            if let nf_game::nis::Obj::Entity { hash, .. } = o {
                if (0x0200_0834..=0x0200_083b).contains(hash) { panels += 1; }
            }
        }
        assert!(panels >= 8, "the 8 iris-panel entities exist [D streams 1-8], found {panels}");
        // The channel-rise sister: [M 0x1dbda8 case 6] reloads only through the other slot
        // (hash[1] = 0 [D]) — a ch rise on the out channel starts NOTHING.
        assert_eq!(sim.drones.scripts.players.iter().find(|p| p.placement == 141).unwrap().state, 6,
            "sister: the doors stay looping loopers regardless of the missile channels");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1c, 1, sim.drones.frame);
        sim.drones.channels.set(0x18, 1, sim.drones.frame);
    }
    w.tick(Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for plc in [141usize, 142] {
            let p = sim.drones.scripts.players.iter().find(|p| p.placement == plc).unwrap();
            assert_eq!(p.state, 6,
                "sister: the ch 0x18/0x1c rise does NOT restart SP#{plc} [M 0x1dbda8 case 6 needs the other slot]");
        }
    }
}

/// The spawn pin (green): with ch 0x0a driven from the WALKED deck route, spawner #187 produces
/// Drake instantly, enabled, at his placement (−7.32, −1.42, 0.06) — the 720a "Drake does not
/// spawn in the port" gap is refuted (their Bond had fallen into the pit; the spawner answers
/// from the deck's edge).
#[test]
fn the_fight_gate_spawns_drake_from_the_deck() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    // [S] unverified — the same ground-substate premise as leg 1 (the p3 == 2 spawn enters
    // substate 8 [M 0x1910f8 :160-165]; this test pins the spawner, not the substate).
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [14.0, -1.9, -4.0], [12.0, -1.9, -6.5]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.3), "the deck south-west");
    }
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x0a, 1, sim.drones.frame);
    }
    let mut found = None;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
            found = Some(i);
            break;
        }
    }
    let i = found.expect("the gate spawns Drake (spawner #187 [D])");
    let sim = w.scene.sim.as_ref().unwrap();
    let d = &sim.drones.list[i];
    assert!(d.enabled, "the gate enables Drake (wait-switch 10 [D])");
    assert!((d.pos[0] - (-7.32)).abs() < 1.5 && (d.pos[2] - 0.06).abs() < 1.5,
        "Drake stands at his placement ({:?})", d.pos);
    assert!(!sim.drones.channels.on(0x0e), "Drake is alive (ch 0x0e not yet fired [D param 12 = 14])");
}


/// Sister walk: the east-deck leg adds no substate change of its own. The p3 == 2 spawn starts
/// IN the zero-G substate 8 [M Player_Init 0x1910f8 :160-165 — asserted; the daily audit
/// 2026-10-17: the old "never enters zero-G" doc contradicted the start], and nothing on the
/// level changes it (C_CHCHZEROG 0x216388's msg 0x4b → 9 is the cheat page's checkbox, not a
/// level event). Driven under the legs' [S] ground-substate premise, no tick of the route enters
/// 8/9.
#[test]
fn the_east_deck_route_adds_no_substate_change() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8,
        "the p3 == 2 start is substate 8 [M Player_Init 0x1910f8 :160-165]");
    // [S] unverified — the same ground-substate premise as the legs above.
    w.scene.sim.as_mut().unwrap().bond.substate = SS_WALK;
    let mut zerog_ticks = 0u32;
    for wp in [[20.0f32, -1.9, 1.0], [17.0, -1.9, -1.0], [13.0, -1.9, -1.0], [10.0, -1.9, -1.0]] {
        let _ = w.try_walk_to(walker::Wp::new(wp[0], wp[1], wp[2]).r(1.2), "the deck west");
    }
    // The substate sample: drive 300 more ticks and count any 8/9.
    for _ in 0..300 {
        w.tick(Default::default(), false);
        if matches!(w.scene.sim.as_ref().unwrap().bond.substate, 8 | 9) { zerog_ticks += 1; }
    }
    assert_eq!(zerog_ticks, 0, "the east-deck route enters no zero-G substate of its own (0x216388's msg-0x4b arm is the cheat page's, not a level event)");
}

/// The disc's own activation chain, run by the port's script engine: with ch 30 (SP#211's gate —
/// console-1's UI-script write [D 0x060007e2 stream 1]) and ch 46 (SP#211's out — the
/// missile-disarm chain's write [D 0x0600091c stream 10]; mask bit 0 IS the state-0 start trigger
/// [M SP_Update 0x1dbda8 case 0]) written at the slit mouth, SP#211 plays 0x060007fd → 0x060008f0
/// and reaches the fight gate: Drake (spawner #187 [D]) spawns enabled within the cap. Bond flies
/// there from the load's zero-g substate 8 [M Player_Init 0x1910f8 :160-165]. Sister: the
/// fly-by's own SUB 0x6000971 ch-75 pulse (stream 2 `ev0x0d[0x4b,1]` [D]) changes no substate —
/// the 0x4b of `C_CHCHZEROG_Handler` 0x216388 is the cheat page's menu message [M :10-17].
/// Removing the SP start trigger's out-channel bit regresses the Drake pin.
#[test]
fn the_sp211_chain_spawns_drake_in_the_start_float() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    float_to_slit_mouth(&mut w);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x1e, 1, sim.drones.frame);
        sim.drones.channels.set(0x2e, 1, sim.drones.frame);
    }
    let f0 = w.scene.sim.as_ref().unwrap().drones.frame;
    let mut pulse = false;
    let mut drake = None;
    for _ in 0..1800u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, SS_ZEROG_8, "the chain moves no substate: Bond stays in the start's 8");
        if sim.drones.channels.time.get(0x4b).is_some_and(|&t| t > f0) { pulse = true; }
        if drake.is_none() {
            if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
                drake = Some(i);
            }
        }
        if pulse && drake.is_some() { break; }
    }
    assert!(pulse, "the chain's own ch-75 SUB pulse is written [D 0x6000971 s2]");
    let i = drake.expect("the chain reaches the fight gate: Drake spawns (spawner #187 [D])");
    assert!(w.scene.sim.as_ref().unwrap().drones.list[i].enabled,
        "the gate enables Drake (wait-switch 10 [D])");
}

/// The whole mission on REAL health — no top-up, no position, substate or channel write after
/// the load — the route of `walk_arrival_to_drakes_defeat` (`play_the_mission`): auto-aim on (the
/// app's default; the laser's +0x14 is 0, so `Check_AutoAim` never locks with it [M 0x1a73f8 k =
/// +0x14/100]), the first wave fought from the hub float, the screens in deadline order, the later
/// waves waited out under the deck (they float in the band above it), Drake fought, the defeat NIS
/// waited out in cover.
///
/// The damage he takes is the game's model, pinned [M]: weapon 0x6a's 5.0 [D weapon_data +0x0c]
/// × `Plr_DMod_*` at the load's difficulty (1b's group [M ReadTuningVars 0x1d3a80 case
/// 0x700001b: Easy 0x3f19999a = 0.6, Normal 0x3f333333 = 0.7]; the switch on GameState+0x28
/// [M Player_HandlePain 0x1902c8:28-49]) = 3.0/3.5 a hit, armour first [M :100-105]; the hit
/// roll `100 − 5·acc` with the astronauts' accuracy 9 × `DroneFiring_Accuracy_*` (Easy 0.6,
/// Normal 0x3f4ccccd = 0.8 [M 0x1d3a80 / DoBulletAccuracy 0x179740:52-99]) × `TargetMoving`
/// 0.75 while the float's thruster pair is live [M 0x179a48:34-41 — `Drones::player_move_local`].
/// The walk, parametrized by the menu's difficulty pick (Agent = 2 [M C_SBNFDFCTY 0x216ca8:15
/// → GameState+0x28]): the difficulty-1 expression is the pre-sister master's byte-for-byte
/// (plain `load`), 2+ loads the front-end's launch path [M → load_with_carry_difficulty].
fn walk_arrival_to_drakes_defeat_on_real_health_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "0700001b.bin", false).unwrap()
    } else {
        nf_view::load_with_carry_difficulty(&iso, "0700001b.bin", false, None, difficulty).unwrap()
    });
    w.top_up_health = false;
    w.wait_enabled(3000, "the shuttle arrival");
    w.scene.sim.as_mut().unwrap().auto_aim = true;
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, SS_ZEROG_8, "the station starts in the zero-g substate [M Player_Init 0x1910f8 :160-165]");
        assert_eq!(sim.drones.difficulty, difficulty, "the harness load's difficulty (GameState+0x28) [M 0x2a3790 → Drones::difficulty]");
        assert_eq!(nf_game::bullet::damage_factor(sim.level, difficulty),
            if difficulty == 1 { 0.6 } else if difficulty == 3 { 1.0 } else { 0.7 },
            "1b's Plr_DMod row for the picked difficulty [M 0x1d3a80 case 0x700001b: Easy 0.6, Normal 0.7, Hard 1.0]");
        assert_eq!(sim.tuning.DroneFiring_Accuracy_Easy, 0.6, "1b's DroneFiring_Accuracy_Easy [M 0x1d3a80 case 0x700001b]");
        if difficulty >= 2 {
            assert_eq!(sim.tuning.DroneFiring_Accuracy_Normal, 0.8, "1b's DroneFiring_Accuracy_Normal [M 0x1d3a80 case 0x700001b: 0x3f4ccccd]");
        }
        let rec = &sim.weapons.records[0x6a];
        assert_eq!(f32::from_le_bytes(rec[0x0c..0x10].try_into().unwrap()), 5.0, "the astronauts' weapon 0x6a hits for 5.0 [D weapon_data +0x0c]");
    }
    let alive = |s: &nf_view::scene::Sim| s.drones.health.health > 0.0;
    play_the_mission_opts(&mut w, alive, difficulty >= 2, difficulty >= 3);
    let sim = w.scene.sim.as_ref().unwrap();
    let lost: f32 = sim.drones.player_hits.iter().map(|h| h.2).sum();
    eprintln!("[1b-rh] end frame {} health {:.1} armour {:.1} hits {} health lost {lost:.1}",
        sim.drones.frame, sim.drones.health.health, sim.drones.health.armour, sim.drones.player_hits.len());
    assert_eq!(sim.drones.mission.status(), 6, "the mission ends (MissionState 6) with Bond alive [M 0x182c60] (health {:.1})", sim.drones.health.health);
    assert_eq!(sim.drones.mission.to_end, 0x0730_001b, "the station's own end code [M 0x182c60:114-116]");
    // Both objectives COMPLETE on the records the monitor reads, in the game's order (the
    // mid-walk hinges in play_the_mission pin obj 1 = 3 / obj 2 = 1 at Drake's spawn and
    // obj 1 = 4 / obj 2 = 3 at his death) [M 0x1827d8:78-85; L record 1b].
    assert_eq!(sim.drones.mission.states[23][0], 4, "objective 1 COMPLETE on the record");
    assert_eq!(sim.drones.mission.states[23][1], 4, "objective 2 COMPLETE on the record");
    let t = |c: u32| sim.drones.channels.time.get(c as usize).copied()
        .unwrap_or_else(|| panic!("ch {c} was written on the walk"));
    assert!(t(0x0b) < t(100), "obj 2's 0x0b lands before the mission-end ch 100 [D 0x06000a20 t600/t720]");
    // Sister: the damage path ran for real — the rounds cost the armour and then health (no top-up).
    assert!(sim.drones.player_hits.len() >= 15 && sim.drones.health.armour == 0.0 && lost > 0.0,
        "the waves really hurt Bond on the way ({} hits, armour {:.1}, {lost:.1} health) — no allowance", sim.drones.player_hits.len(), sim.drones.health.armour);
    assert!(sim.drones.health.health < 100.0, "health is never topped up");
}

#[test]
/// UN-IGNORED 2026-11-13: green on master c94c1043e (the ignored-walk sweep).
fn walk_arrival_to_drakes_defeat_on_real_health() {
    walk_arrival_to_drakes_defeat_on_real_health_at(1);
}

/// The Agent-difficulty sister (real-health-agent-difficulty-16-1b-41-4a): the same walked
/// station on real health, loaded at difficulty 2 the menu way — Bond's pain scales by 1b's
/// Plr_DMod_Normal 0.7 [M Player_HandlePain 0x1902c8; 0x1d3a80 case 0x700001b] and the
/// astronauts' hit chance by Accuracy Normal 0.8 [M DroneWeap_DoBulletAccuracy 0x179740:74-88].
/// Real input only; no health/ammo/channel writes.
#[test]
/// GREEN 2026-11-13: the waits shift cover (a no-owner 0x6b explosive bolt landed on the held
/// missile-wait spot) and the Drake fight's cover ring reaches 8 and 11 u (every point within 5 u
/// of the hold was blind to Drake); the walk defeats Drake on real health at Agent.
fn walk_arrival_to_drakes_defeat_on_real_health_agent() {
    walk_arrival_to_drakes_defeat_on_real_health_at(2);
}

/// The 00 Agent sister (real-health-00agent-16-1b-41-4a): the same walked station on real
/// health, loaded at difficulty 3 the menu way — pain Plr_DMod_Hard 1.0 [M Player_HandlePain
/// 0x1902c8; 0x1d3a80 case 0x700001b: Easy 0.6, Normal 0.7, Hard 0x3f800000] and the
/// astronauts' hit chance by Accuracy Hard [M DroneWeap_DoBulletAccuracy 0x179740:74-88].
/// Real input only; no health/ammo/channel writes.
#[test]
#[ignore = "00 Agent (11-14, round 6, 16 measured iterations [L /tmp/lever1..16.log]): STILL RED, but the fight shape is rebuilt and the wave is solved. The round-5 'yield at the pending station' gate is WRONG (iter 1: suppressing the station fights removed every defense after the first crossing, death 5649 — 1845 before baseline) — the station fights are the survival mechanism: their exposure-hedged cover floats break the shooters' 2-s sight clocks. The run A kill shape works once the fight can REACH the shooters: the approach lane (40% toward the nearest shooter) + nearest-shooter prio + a 2-kill exit thinned 7 -> 4 with two kills a fight in 286-388 ticks (iters 3-4); k6 LATCHED -683 (iter 10, death 7541). MEASURED SHAPE LAWS: max_y 12 is BAND-UNSAFE (iter 9: the k4 screen stance had lifted Bond to feet 7.375, the fight's cover ring walked him under the band's z -0.4..0.4, two -50.0 deliveries, hp 100 -> 0.0 in a 14-tick fight) — the band-safe body cap is max_y 7.0 (feet 7.1, capsule top 8.9 < 9.0, reachable aims ~8.4-8.8 = the prior ledger's '<=8.3 aim heights' band); the cooldown must count SIM FRAMES not loop iterations (iter 8: the fight burns its window inside ONE iteration, the defense then slept ~300 real ticks, -13.9 hp in 52); the chain fence admits fights inside a STARTED chain on the missile's own LIVE clock (read_deadline_in [M 0x1aa23c-258] — the measured live k4 read sits ~1200 past the modeled S_k + P0_LEN - 1: the p0 starts ~1200 after MissileDeploy 0->1, so the modeled margins print late while the disarm holds); the pre-chain fights stay gated at slack 1200 (iter 7's 900 gate CASCADES: the extra fight at 3734 shifted the route, the k4 fight collided with the f4200 spawn wave, -45.0 hp, death 6784); the lane must stay PINNED to the pending console's station ±5 u (iter 11: the raw lane is a CHASE TRAP — fights dragged Bond to x39.4/z 10.8, the chains starved, m6/m8 LAUNCHED, deploy [1,1,2,2,2,3,1,3], status 2). FINAL SHAPE (iter 16): pinned lane, slack 1200, cap 300, frames cooldown, live fence >900, melee override <12 u, 2-kill exit, max_y 7.0. RESULT [L iter 16]: k5 -18 / k4 -457 (green by the model margin) / k3 +398 (live-saved), m3/m4/m5 ALL DISARMED (deploy [0,0,2,2,2,1,0,1]), the wave thinned 8 -> 3, hp 57.7 at t7014 — death t7442 (-52.7 inside the last 128-tick fight). NEXT LEVERS: (1) the melee override is a FIGHT LOOP — with the wave parked within 12 u the fights re-fire every 300 ticks with NO 2-kill progress (astro 3 -> 3 -> 3, no k6 hold EVER answered in iters 12-16) and the k6 chain starves; the fix: a no-progress exit (a fight that lands no kill inside its window ends at ~150) and the k6 screen resumption; (2) the thinned wave's survivors snipe from 12-25 u with ESTABLISHED sight (the melee trigger never fired; -41.8 hp in 362 ticks from two shooters, iter 13) — the fight trigger needs a closing-threat band, not lined>=2 alone; (3) the Drake phase's y > 9 spawn-band re-weight after the console phase survives. The cadence probe stays as the determinism/cadence pin (crates/nf-view/tests/probe1b_cadence.rs). The prior ledgers follow. 00 Agent (11-13): the defense cap now reserves 800 ticks for the pending console chain (was 400): m3/m4/m5 ALL DISARMED (deploy [0,0,2,2,2,1,0,1]), k5 3158 / k4 5174 / k3 6177 latched, death t6580 -> t7494 at (32.44,-5.46,-1.67) during k6's window - the wave grows to 6-7 astronauts (#5 20.0, #8 15.0, #14 12.0, #7 10.0, all 0x6a) plus one 17.64 world blast at the fixed 6580 tick (the kind-107 debris schedule below). Next: the fights must THIN the wave (round 5 lever 2) and the blast schedule compare (the kind-107 lever). Round 5 (real-health-00agent-1b-5, merged master abc09337d): THE DECODE FIND ANSWERED — the modeled S_k + P0_LEN − 1 deadline STANDS. probe1b_cadence (2 loads x 7000 ticks in one process, GREEN) pins the p0 owned-script clock at EXACTLY +1.0 per sim tick [M 0x1aa23c per-tick Script_Update; L the traces], the missile starts byte-identical across loads (ch19 49: 6349/6349/2450/1250/50/3650/6349/3650 — ±1 of ch19 + seed·60, the f32 countdown rounds per seed: 105 → 6300, 40 → 2401), determinism holds in-process. THE RUN-TO-RUN DIVERGENCE RESOLVED as an OBSERVATION ARTIFACT: the walk's deploy/clock sample block is bypassed while fight_opts_y ticks, so a missile start that lands INSIDE a defense fight is recorded only at the fight's end (this run: m3 recorded 4375, the fight 3457..4374 covered the model-consistent ~4175; the -4 run A vs run B 4175/4375 pair is the same artifact). No ch 19 drop: this run logged every AND(193,194) falling edge [L the per-tick detector] — zero drops, ch 19 on from 1775. THE -5 LEVER RUN [L]: the 300-cycle defense (cooldown 1200 → 300), cap = read_deadline_in − 400 clamped 300..1400, the hold-chain fence (answered[pi] == 0 — never start a fight once the pending console's UI chain has begun), the off-plane lane hold |z| 1.6 on the nearest shooter's side: death 6580 (33 s past master 5619) but m3 TOOK THE LAUNCH ARM (deploy [0,0,3,2,2,1,0,1]) — the fights ran their FULL caps WITHOUT thinning (astro 5 after both: 3457..4374 cap 917, 4716..5574 cap 858 — the -4 run A thinned 7 → 3 in one window, this run's lane holds did not), so the schedule slipped: k5 3158 (margin −416, as baseline), k4 5574 (+800 past 4774 — the two full-cap fights ate its approach), k3's holds answered 5828/5936/6044, all past m3's ~5975 read → the channel write landed 6565 (+391) AFTER the read → m3 launched, status 2. THE CAP-MATH FLAW: cap = deadline − 400 grants the fight the chain's own time — the fence only blocks fights DURING a chain, not fights that consume the slack a chain still needs; and the chain time is ~900 ticks (3 holds + the write), not 400. NEXT ROUND: (1) cap = deadline − CHAIN_RESERVE with CHAIN_RESERVE ~1000 measured (the k4 chain 4374 → 5574), and the fight must YIELD when the pending console's station is reached (the fence becomes: no fight once Bond is within ~4 u of the pending station — the approach is the fight's real cost, the -4 run A scheduled fights BETWEEN consoles, this run fought ON the approach); (2) make the fights THIN like run A (the lane holds did not — compare the lane pick vs run A's: run A held the lane the wave CROSSES to reach Bond, this run's |z| 1.6 lane sat behind the panel rows where the wave posts at z ±2.7-4.3 out-range Bond — try the lane on the wave's approach side and the prio target the nearest CROSSED shooter); (3) the Drake phase's y > 9 spawn-band re-weight after the console phase survives. The cadence probe stays as the determinism/cadence pin (crates/nf-view/tests/probe1b_cadence.rs). The prior ledgers follow. 00 Agent, round 4 (real-health-00agent-16-1b-4, merged tree 9bb741b50 = master 784ba2d99 + the 1b-2 ledgers): the merged tree reproduces the console5 ledger EXACTLY [L this run] — ch19 1775, missile starts m3 4175/m4 2975/m5 1775/m6+m8 5375, k5 3158-3201 (-373..-416) / k4 4317 (-457) / k3 5574 (-400) latched in deadline, m4+m5 disarmed (deploy [0,0,1,2,2,1,0,1]), the wave 3@f3000 -> 6@f4200 -> 8@f5100, and the grind kills Bond at 5619 during k6's window. THE TASK'S LEVERS, MEASURED [L three runs of this round]: (1) the caps re-balanced against the modeled read times — defense cooldown 1200 -> 300, fight cap = the pending console's read_deadline_in − 400 clamped 300..1400 (was uniform 700), the hold on the off-plane lane |z| 1.6 on the nearest shooter's side (the console5 ledger's band-safe kill shape) — run A reached 6256 with m3/m4/m5 ALL DISARMED and the lane fights THINNED the wave 7 -> 3 in one window (the kill shape works); run B of the same source took m3's LAUNCH arm (deploy[2] = 3, status 2) at 6225; the 150-cycle intermediate starved the holds outright (m3 launched, hp 2.2 at 6977) — 300 is the measured edge and the lever set is NOT reliably above master, so the tree stays at master + this ledger. (2) THE DECODE FIND (the task's item, now measured): the modeled read deadline S_k + P0_LEN[k−1] − 1 does NOT match the sim — m5's owned-script clock read 331 when 1330 state-1 ticks had elapsed (~1/4 rate: the missile p0 clock advances ~1 tick per 4 sim ticks, a throttled missile update or a clock-scale mismatch), and missile STARTS diverge run-to-run on identical source (m3 4175 in run A vs 4375 in run B — the determinism premise itself breaks once the route moves Bond off the master path); the late-latch disarms (k4 +800, k3 +282 observed) fit the reads sitting LATER than modeled. NEXT ROUND: (a) re-derive the real read cadence — decode the p0 streams' exit events against SpaceMissile_Update 0x1aa118's per-tick Script_Update [M 0x1aa23c] and find the 1/4 throttle in the port's missile update path (or the game's far-object update law), then re-balance the per-console caps against the REAL read times; (b) keep the lane kill shape at the 300 cycle — it thinned the wave; schedule the fights in the hold-chain gaps the way run A did (the fights must never start inside a hold chain — the console5 k5 lesson — which the slack gate approximates today); (c) explain the run-to-run missile-start divergence (per-process iteration order in the sim? the distance throttle band?) before trusting any single-run ledger. Latch margins printed right after a fight are OBSERVATION-LAGGED (the latch lands mid-fight, the print at fight end) — the deploy values at death are the truth. The prior ledgers follow. 00 Agent, re-measured on this branch (level-1b-console5-00agent): the k5 LATE LATCH is FIXED — the cause was the walk's own mid-phase defense fight running inside k5's hold chain (the red run: the fight 2988..3305 stole the window between hold#1 2917 and hold#2 3593, the latch landed 3667, +93 past the 3574 read deadline, missile 5 launched). The fix is the route, two levers: (1) the defense now also gates on the pending console's read deadline (read_deadline_in = the missile's state-2 read at S_k + P0_LEN - 1 [M 0x1aa23c-258; D P0_LEN]; the fight only runs when >1200 ticks remain — the 700-tick cap plus the re-acquisition and the rest of the hold chain), and (2) the defense floats through fight_opts_y, height-capped at 6.0 feet: the SpaceLaser chain (ch 19 arms it [M 0x1aa3f8], the 10 s fire writes ch 18 ~2375) turns BigLaserBeamHURT #259 (0xe4, dmg 50, the (8.5,9.0,-0.4)..(81.4,9.9,0.4) band [L the hurt dump]) live, and the level-only view's aim lift into a ceiling-band astronaut parked Bond at feet 7.53, capsule top 9.33 [M Player_DealWithObjHit 0x194458], two -50.0 deliveries, dead 3457 — a real game death the old uncapped lift walked into. Result [L this run]: the k5 holds answer 2917/3019/3103 uninterrupted, the channel writes ~3104 (470 ticks before the read), m5 DISARMS (deploy[4] = 2, no state-3 missile, mission status stays 1); k4 4317 (-457) and k3 5574 (-400) latch green the same way. STILL RED downstream, survivability not deadlines: the wave grows 3@f3000 -> 6@f4200 -> 8@f5100 (the missile-script spawn channels), the capped fight cannot reach the ceiling-band shooters (the level-only view needs the eye at the aim height; in-plane lifts above feet 6.65 enter the live beam band; the wave posts sit at z +/-2.7-4.3 behind the panel rows), and the grind kills Bond at 5619 (14 hits for 100.0) during k6's window with k5/k4/k3 latched and disarmed. NEXT LEVERS: the wave-clearing needs a band-safe kill shape (an off-plane lane at |z| >= 1.0 the panel rows do not block, or waiting the band shooters down to the <=8.3 aim heights), and the kind-107 blast source decompile-compare of the previous ledger still stands for the disarm wait. The prior ledger follows. level-1b-console-latch-00agent: the console phase is FIXED (the task's 'console 1 latched by the real shot' red) — the d3 lever (exposure-hedged station picks, the bounded mid-phase defense fights, the hurt-free stance guard, all inside this file, run3/run5 [L]): all 8 consoles latch in deadline order k5 3667 / k4 4322 / k3 5531 / k6 7085 / k8 7409 / k1 9264 / k2 9401 / k7 10079, health 39.3 at the k7 latch — the run1 death (tick 6251 mid-console-phase, the ch-19 launch order leaves five skin-0x5000078 survivors whose 0x6a rounds at pain 1.0 [M 0x1902c8; 0x1d3a80 case 0x700001b] ground the walk down with 0 return-fire fights) is gone. STILL RED downstream, at the same zone as the old d2/d3 ledger deaths: the disarm wait at the x46 nest dies at tick 10424 at (46.01,-8.08,0.00) — kind-107 debris blasts (damage 20.0 radius 20.0 owner -2 [L 1b-run4.log NF_DMG_TRACE]) detonate AT the screens Bond shoots: 8502 exactly on the x20 station (20.2,-1.9,-0.2), 10424 exactly on the x46 nest (46.1,-7.8,-0.2), two point-blank deliveries each, and the schedule is FIXED (4658->6580->8502->10424, exactly 1922 ticks apart, byte-identical across run3/run4/run5 — the latch-stop-at-channel lever changed nothing, so the blasts ride the screens' own 3-hold resume chains, not the extra taps). The level's only two hurt volumes are clean (the dump: kind 2 damage 50 at (-34.4,5.5,-0.9)..(-32.5,13.8,0.9) far west, kind 1 damage 50 gate 18 disabled at (8.5,9.0,-0.4)..(81.4,9.9,0.4) — neither touches the stations; the world hits are NOT hurt volumes). NEXT LEVER: the decompile-compare on the blast source — which script fires the kind-107 explosion at the panel resumes (the port's panel resume -> debris chain, GAPS G-047: 'the 0700001b solar panels (param 7 = 20) resume and their stream 1-10 debris plays'), and whether the game's blast schedule/damage matches (a live check: in PCSX2 at 00 Agent, shoot console 1's screen from the station and watch the health trace for a 20.0 blast at the screen — if the game's player takes it too, the route must dodge the 1922-tick fire ticks instead; if not, the port's blast delivery is the port gap)."]
fn walk_arrival_to_drakes_defeat_on_real_health_00agent() {
    walk_arrival_to_drakes_defeat_on_real_health_at(3);
}

/// The astronauts' stats and gun, from the disc and the decompile. They are type 1 (`+0xC7`,
/// placement parameter 15) and carry no stats section of their own; the type's record comes from
/// placement #140's section (skin 0x0500008F, tag 0x80010006 — kind 1 = accuracy 9, aggression 1,
/// health 10, 80, 3, reaction 80, recover 80, 1 [D the 0x303330 bit widths]) through the shared
/// table every later drone of the type reads [M behaviour_util_get 0x1c7a98; DoModeSettingsNEW
/// 0x14c600:70-107]; their health is then `DroneInit_TakeXHits(dmg, 0x33)` = weapon 0x33's 20.0 ×
/// 1.0 on Easy [M 0x14b300:728-739 + 0x14d928:6-8] — one laser hit (0x33's own +0x0c 20.0).
/// Their weapon 0x6a: 5.0 a round, one round a shot, spread 5 [D weapon_data +0x0c/+0x18/+0x24].
/// The hit roll [M DroneWeap_DoBulletAccuracy 0x179740:52-99, ported `hit_chance`]: (100 − 5·9) ×
/// Easy 0.6 = 33 %, × TargetMoving 0.75 = 24.75 % on a thrusting float; inside 3 u every round hits
/// [M :113-115] for twice the damage [M Drone_ModBulletDamage 0x139fac-0x139fc8]; beyond it a hit
/// also needs 2 s of unbroken sight (`+0x270 ≥ NewSighting_TimeToHit · FRAME_RATE`) and no sight
/// lost since the last shot (`+0x41`) [M :102-109]. `DroneWeap_FireWeapon` 0x178f68 runs the roll
/// for every shot but the type-0x1e minigun arm [M :92-148] and spawns `weapon_data+0x18` bullets
/// [M :233-241] — the port's `fire_weapon` (drone.rs) follows both. Sister: the astronaut's own
/// (empty) section alone would read the defaults 5/0/10, so the pin can fail.
#[test]
fn the_astronauts_fire_at_accuracy_9_for_3_a_hit() {
    let iso = iso();
    let img = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700001b.bin")).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lv = nf_formats::level::Level::parse(&data).unwrap();
    let pl = &lv.maps[lv.main_map().unwrap()].placements;
    let get = |i: usize, k: u32| pl[i].params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
    assert_eq!((pl[140].flags & 0xff, get(140, 0), get(140, 15), get(140, 26)), (0x0f, 0x0500_008f, 1, 0x8001_0006),
        "#140 carries the type-1 stats section [D]");
    let mut table = nf_formats::drone::DroneStatsTable::default();
    assert_eq!(table.behaviour_get(|k| get(140, k)), 1);
    assert_eq!(table.0[1][1], [9, 1, 10, 80, 3, 80, 80, 1], "the type-1 kind-1 record [D]");
    let astro: Vec<usize> = (0..pl.len()).filter(|&i| pl[i].flags & 0xff == 0x0f && get(i, 0) == 0x0500_0078).collect();
    assert!(astro.len() >= 17 && astro.iter().all(|&i| get(i, 15) == 1 && get(i, 26) == 0),
        "every astronaut template is type 1 with no section of its own [D]");
    let mut alone = nf_formats::drone::DroneStatsTable::default();
    let ty = alone.behaviour_get(|k| get(astro[0], k));
    let own = alone.stats(ty);
    assert_eq!((own.accuracy, own.aggression, own.health), (5, 0, 10), "sister: an astronaut's own section alone reads the defaults");

    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    for _ in 0..3000u32 {
        w.tick(Default::default(), false);
        if !live_astronauts(w.scene.sim.as_ref().unwrap()).is_empty() { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let live = live_astronauts(sim);
    assert!(!live.is_empty(), "the first wave spawns");
    for &i in &live {
        let d = &sim.drones.list[i];
        assert_eq!((d.accuracy, d.aggression, d.reaction, d.recover), (9, 1, 80, 80), "astronaut {i} reads the shared type-1 record");
        assert_eq!((d.weapon, d.health, d.max_health, d.damage_mul), (0x6a, 20.0, 20.0, 1.0), "astronaut {i}: weapon 0x6a, TakeXHits(1.0, 0x33) = 20");
    }
    let rec = &sim.weapons.records[0x6a];
    let f = |o: usize| f32::from_le_bytes(rec[o..o + 4].try_into().unwrap());
    assert_eq!((f(0x0c), rec[0x18], f(0x24)), (5.0, 1, 5.0), "weapon 0x6a: 5.0 a round, one round, spread 5 [D]");
    assert_eq!(f32::from_le_bytes(sim.weapons.records[0x33][0x0c..0x10].try_into().unwrap()), 20.0, "the laser 0x33 hits for 20 [D]");
    let t = &sim.tuning;
    let still = nf_game::drone::Motion::default();
    let moving = nf_game::drone::Motion { moving: true, ..Default::default() };
    let pct = |m| nf_game::drone::hit_chance(t, sim.level, 1, 9, 20.0, m, 0x10);
    assert!((pct(still) - 33.0).abs() < 1e-4, "(100 − 45) × 0.6 = 33 % ({})", pct(still));
    assert!((pct(moving) - 24.75).abs() < 1e-4, "× TargetMoving 0.75 = 24.75 % ({})", pct(moving));
    assert!(nf_game::drone::hit_chance(t, sim.level, 1, 9, 2.0, still, 0x10) > pct(still), "sister: TooClose (×2, twice [M :53-55, :97-99]) raises it");
    assert_eq!((t.DroneFiring_TooClose_Distance, t.DroneFiring_TooClose_Damage, t.DroneFiring_NewSighting_TimeToHit), (3.0, 2.0, 2.0),
        "TooClose under 3 u, ×2 damage; 2 s of sight to land a round [M ReadTuningVars 0x1d3a80]");
    assert!((nf_game::bullet::damage_factor(sim.level, 1) * f(0x0c) - 3.0).abs() < 1e-5, "5.0 × Plr_DMod_Easy 0.6 = 3 a hit");
}

#[test]
fn the_zero_g_float_reads_moving_while_its_thrusters_decay() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.substate, SS_ZEROG_8, "the float substate [M 0x1910f8 :160-165]");
        assert_eq!(sim.bond.local, [0.0, 0.0], "no thrust yet");
        assert_eq!(sim.drones.player_move_local, Some([0.0, 0.0]), "the viewer hands the drones the pair in zero-g");
        let g = first_tracker(sim);
        assert!(!sim.drones.list[g].target_motion.moving, "sister: a float that never thrusted reads still [M 0x179a48:34-41]");
    }
    for _ in 0..20u32 { w.tick(Input { forward: 1.0, ..Default::default() }, false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let g = first_tracker(sim);
        assert!(sim.drones.list[g].target_motion.moving, "thrusting reads moving [M 0x179a48:34-41]");
    }
    for _ in 0..300u32 { w.tick(Input::default(), false); }
    let p0 = w.scene.sim.as_ref().unwrap().bond.pos;
    w.tick(Input::default(), false);
    let sim = w.scene.sim.as_ref().unwrap();
    let p1 = sim.bond.pos;
    let step = (p1[0] - p0[0]).abs() + (p1[2] - p0[2]).abs();
    assert!(step < 1e-4, "the decayed drift moves under the on-foot 1e-4 test ({step:e})");
    assert!(sim.bond.local[1] != 0.0, "the eased forward smoother never reaches 0 [M 0x192ab0:116-135]");
    let g = first_tracker(sim);
    assert!(sim.drones.list[g].target_motion.moving, "the float still reads MOVING for the hit roll [M 0x179a48:34-41]");
    assert!(!sim.drones.list[g].target_motion.first_moved, "the first-moved window (2 s) is long over [M 0x179a48:70-73]");
}

/// The float against the shaft's south end-wall (z −20.40 [D the_end_walls_seal_the_shaft_box])
/// at the deck-level window (x 8, y −1.2) — real input from the level start, no position,
/// substate or channel write. The capsule rests at −20.40 + 0.55 (the body radius [M
/// Player_Collision 0x194df0's default arm, 0x3f0ccccd]) and the push-out keeps the tangential
/// part of the thruster move (`pos += local·rows` [M 0x193600:284-295], then the per-triangle
/// push along the separation [M Intersect_CylGeom 0x109da0]), so a float pressed into the wall
/// slides along it in x and y and backs off freely. Sister: thrust straight into the wall holds
/// Bond still at the rest line, never through the window (0.8..1.0 tall, narrower than the
/// ~1.58 body [D]) — the hold an older walk read as a wedge while it steered at (8, −1.2, −22),
/// a point behind the closed wall.
#[test]
fn the_float_slides_along_the_south_end_wall() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w.wait(120);
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.substate, SS_ZEROG_8, "the station floats from the start [M Player_Init 0x1910f8 :160-165]");
    // Body-axis thrust toward a point (the walk's `thrust_to` at a firmer gain).
    let thrust = |sim: &nf_view::scene::Sim, at: [f32; 3]| -> Input {
        let p = sim.bond.pos;
        let d = [at[0] - p[0], at[1] - p[1], at[2] - p[2]];
        let m = sim.bond.zero_g_rot;
        let dot = |a: [f32; 3]| a[0] * d[0] + a[1] * d[1] + a[2] * d[2];
        Input { strafe: (dot(m[0]) * 0.6).clamp(-1.0, 1.0), lift: (dot(m[1]) * 0.6).clamp(-1.0, 1.0), forward: (dot(m[2]) * 0.6).clamp(-1.0, 1.0), ..Default::default() }
    };
    // Yaw-steered forward thrust (turn toward the point, lift for the height).
    let steer = |sim: &nf_view::scene::Sim, at: [f32; 3]| -> Input {
        let p = sim.bond.pos;
        let err = walker::wrap_pi((at[0] - p[0]).atan2(at[2] - p[2]) - sim.bond.yaw);
        Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() < 0.35 { 1.0 } else { 0.15 },
            lift: ((at[1] - p[1]) * 2.0).clamp(-1.0, 1.0), ..Default::default() }
    };
    let rest = -20.40f32 + 0.55;
    let fly = |w: &mut Walker, at: [f32; 3], n: u32, yaw: bool| -> ([f32; 3], [f32; 3], f32) {
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        let mut last_step = 0.0f32;
        for _ in 0..n {
            let input = { let sim = w.scene.sim.as_ref().unwrap(); if yaw { steer(sim, at) } else { thrust(sim, at) } };
            let p0 = w.scene.sim.as_ref().unwrap().bond.pos;
            w.tick(input, false);
            let p = w.scene.sim.as_ref().unwrap().bond.pos;
            last_step = ((p[0] - p0[0]).powi(2) + (p[1] - p0[1]).powi(2) + (p[2] - p0[2]).powi(2)).sqrt();
            for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); }
        }
        (lo, hi, last_step)
    };
    // The route down to the window: off the deck, under the south rails (z −8.13 / −14.25 [D]),
    // back up into the window band at z −17.
    for at in [[20.0f32, 0.0, 1.0], [8.0, 0.0, -1.0], [7.0, 0.0, -9.6], [7.0, -3.6, -11.5], [7.5, -3.4, -15.5], [8.0, -1.2, -17.0]] {
        fly(&mut w, at, 700, false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!((p[0] - 8.0).abs() < 0.2 && (p[1] + 1.2).abs() < 0.2 && (p[2] + 17.0).abs() < 0.2,
        "the float reaches the window approach (8, −1.2, −17), at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);

    // Sister: straight into the wall — Bond holds at the rest line, never through the window.
    let (lo, _, step) = fly(&mut w, [8.0, -1.2, -22.0], 700, false);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(lo[2] > rest - 0.01, "the end-wall holds the capsule at z {rest:.2} (min z {:.3}) [D m40 + M the body radius]", lo[2]);
    assert!((p[2] - rest).abs() < 0.01 && step < 1e-3,
        "sister: pressed head-on, the float rests on the wall (z {:.3}, last step {step:.5}) — the window is narrower than the body [D]", p[2]);

    // Pressed in AND thrusting east: the tangential move survives the push-out.
    let (lo, _, _) = fly(&mut w, [16.0, -1.2, -28.0], 900, false);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] > 15.5 && lo[2] > rest - 0.01 && (p[2] - rest).abs() < 0.01,
        "the float slides east along the wall (x 8 → {:.2}, z {:.3})", p[0], p[2]);
    // West and down a little, pressed in (yaw-steered, the forward thruster into the wall).
    let (lo, _, _) = fly(&mut w, [6.0, -1.6, -28.0], 1500, true);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[0] < 6.3 && (p[1] + 1.6).abs() < 0.1 && lo[2] > rest - 0.01,
        "yaw-steered, the float slides west and down along the wall to ({:.2},{:.2},{:.3})", p[0], p[1], p[2]);
    // Up the wall face.
    let (_, hi, _) = fly(&mut w, [8.0, 1.0, -25.0], 900, false);
    assert!(hi[1] > 0.9, "the float rises along the wall (max y {:.2})", hi[1]);
    // And off it.
    fly(&mut w, [8.0, -1.2, -17.0], 900, false);
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    assert!(p[2] > -17.3, "the float backs off the wall freely (z {:.2})", p[2]);
}

/// `the_float_slides_along_the_south_end_wall` on real damage: the first wave reaches the float
/// at the end-wall while it slides (armour only, measured), and Bond lives through the whole leg.
#[test]
fn the_float_slides_along_the_south_end_wall_real_damage() {
    walker::with_real_damage(the_float_slides_along_the_south_end_wall);
}
