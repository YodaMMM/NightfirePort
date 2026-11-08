//! interactables-4a: the player-touchable KINDS of 0700004a (the Tower-2 elevator) that the
//! existing pins leave open, driven the way a player drives them (the ride, the walk, the use
//! key, the trigger pull), each asserting the game's effect with a sister. The full kind census
//! lives in docs/levels/0700004a.md §7 and `census_0700004a.rs` (the dump); the covered kinds and
//! their live pins:
//! - 0xdb doors — the −52 unlock chain + the patrolling garrison's opens
//!   (`interact_0700004a_live.rs`); THIS file adds the player's own E legs (below).
//! - 0xea plates — the descent firing (`interact_0700004a_live.rs`).
//! - 0xd9 SPs — the lift ride (`fix_finale_live`), the landing cut-scene + handback
//!   (`fix_landing_matrix_live`, `walkreal_0700004a_live`), the lobby scene + the level exit
//!   (`walkreal_0700004a_live`); THIS file adds the twin-sister channel pin (below).
//! - 0xf1 spawners, 0xd2 shooters, 0x33 hint, 0x0f drones — `interact_0700004a_live.rs` (the
//!   drone fire leg is parked there: no clear eye→drone ray from the ride line, 0/2400).
//! - 0x24 start — the spawn (`walkreal_0700004a_live`).
//! - 0x19/0x47 CELs — static geometry, draws (`census_0700004a.rs`'s mesh pass).
//! THIS file (level-4a-interactables-2) adds the legs the model file shape wants per KIND:
//! the top reveal plate under the rider's own WALK, the bare LIFT ride to the landing hold,
//! the player-triggered SimpleScript leg (the bottom plate's walk-in fires the lobby SP and
//! the level end), and the census pin that the absent kinds of the model list are truly
//! absent from m41 [D].
//! Absent from the playable map (m41) [D census + docs/levels/0700004a.md §7]: pickups (0xf0 —
//! the m6/m14 rows are resource-map placements, not instantiated in play), switches/consoles
//! (0x29, m6 only), monitors (0x2d, m13 only), gadget locks (0x2e, m6 only), breakables (0x20),
//! rigid bodies (0xd8). The kind-0x31 row the §7 table missed is the LIFT PLATFORM's own
//! placement (test 1); the kind-0xe9 row is 2 AI points (the AI network, not player-touchable).
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] a live measurement. A
//! teleport is used only where the walk there is a separately documented blocker; the operation
//! itself always runs through the real input.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The disc placement rows the tests cite (the census read at test time — the same parse
/// `census_0700004a.rs` dumps). Returns (flags, params) for every m41 placement of `kind`.
fn m41_rows_of_kind(kind: u8) -> Vec<(u32, Vec<(u32, u32)>, [f32; 3])> {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    lv.maps[mm].placements.iter()
        .filter(|p| p.flags & 0xff == kind as u32)
        .map(|p| (p.flags, p.params.clone(), p.translation))
        .collect()
}

/// The bare ride harness: Bond stands on the platform from the spawn (the walkreal's probe:
/// grounded by t30), rides with no re-seating (the handback fix made the ride hold the key line
/// bare — `walkreal_0700004a_live`), health pinned every 30 ticks (the garrison's fire shrugged
/// off — the fight is not the system under test), `skip_cutscenes` off (the landing scene plays).
struct Ride {
    w: Walker,
    y0: f32,
    t: u32,
}

impl Ride {
    fn new() -> Ride {
        let scene = nf_view::load(&iso(), "0700004a.bin", false).expect("load 0700004a");
        let mut w = Walker::new(scene);
        w.skip_cutscenes = false;
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
        }
        let y0 = w.scene.sim.as_ref().unwrap().bond.pos[1];
        Ride { w, y0, t: 0 }
    }

    /// The script's key line [D 0x06000571 keys: y0 at t90, y0 − 150 at t5040].
    fn key_y(&self, t: u32) -> f32 {
        (self.y0 - ((t as f32 - 90.0).max(0.0)) * (150.0 / 4950.0)).max(self.y0 - 150.0)
    }

    fn step(&mut self, input: Input, use_key: bool) {
        if self.t % 30 == 0 {
            self.w.scene.sim.as_mut().unwrap().drones.health.health = 100.0;
        }
        self.w.tick(input, use_key);
        self.t += 1;
    }
}

/// KIND 0x31 (the row the §7 census table missed): the one m41 kind-0x31 placement is
/// "OBJECT_Lift_At_Floor_67" — hash 0x020005b4 (the lift platform entity), flags 0x2031 (the
/// 0x2000 template bit), NO params [D census] — so `FuseBox_Create`'s script gate skips it and
/// nothing becomes a fusebox; the placement keeps its static handling and the platform is the
/// world Bond spawns and rides on. Sisters: the level's fusebox list is empty at load, and the
/// platform grounds the spawn (the placement's collision is live).
#[test]
fn the_lift_platform_placement_creates_no_fusebox() {
    // The data side [D]: exactly one 0x31 row, the platform's, scriptless with the template bit.
    let rows = m41_rows_of_kind(0x31);
    assert_eq!(rows.len(), 1, "m41 carries exactly one kind-0x31 placement [D census]");
    assert_eq!(rows[0].1.len(), 0, "the 0x31 placement has no params — no script hash [D census]");
    assert_eq!(rows[0].0 & 0x2000, 0x2000, "the 0x2000 template bit is set [D flags 0x2031]");
    // The sim side [M]: FuseBox_Create's script gate (param 0 < 0x06000000 → skip, the scriptless
    // class) leaves the level's fusebox list empty, and the platform's collision is live under
    // the spawn — the same placement grounds Bond.
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).expect("load 0700004a"));
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.fuseboxes.is_empty(),
        "the scriptless 0x2031 placement creates no fusebox [M FuseBox_Create 0x1b0448's script gate]");
    assert!(sim.bond.grounded, "the platform placement's collision is live — the spawn grounds on it");
}

/// KIND 0xdb, the player's own E legs — the legs the unlock/garrison pins leave open, all
/// through the real use key from the car (the only standing spot the ride gives; the doors'
/// reach is 2.5 [M 0x1a5170; door.rs player_use], the door plane sits 2.5 ahead of the stand):
/// (a) an UNLOCKED pair opens on the E as the car passes its floor (#106/#107 at −48, ch_lock 0
/// [D #106 p3]); (b) the LOCKED −52 pair (#110/#111, ch_lock 0x40 [D #110 p3]) refuses the same
/// E while ch 0x40 is low — the pass is before t1610 (the script's ch-0x40 event [D keys]);
/// (c) after the ride's unlock + landing scene (the handback), the SAME pair opens on the E —
/// Door_Update's lock rising edge cleared the door's ch_lock [M 0x1363c8]; and the twin
/// cut-scene SP #273 (out ch 0x83, mask 1 [D #273]) never fires — its trigger is its own out
/// channel [M SP_Hit 0x1dc470:17-19] and nothing writes ch 0x83 on this level [D scans,
/// docs/systems/tower2-lift-ride.md §1], while the landing SP #289 (out ch 0x40) DOES fire on
/// the ride's ch-0x40 write.
#[test]
fn the_riders_e_opens_unlocked_landing_doors_and_refuses_on_locked() {
    let mut r = Ride::new();
    let (d106, d110) = {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let di = |pl: usize| sim.drones.doors.list.iter().position(|d| d.placement == pl).expect("door");
        (di(106), di(110))
    };
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[d106];
        assert_eq!(d.ch_lock, 0, "the −48 pair has no lock channel [D #106 p3]");
        let d = &sim.drones.doors.list[d110];
        assert_eq!(d.ch_lock, 0x40, "the −52 pair locks on ch 0x40 at load [D #110 p3]");
        assert!(sim.drones.doors.list.iter().all(|d| d.state == nf_game::door::CLOSED),
            "sister: every door closed at load");
        assert!(!sim.drones.channels.on(0x83), "sister: ch 0x83 off at load [D #273's trigger is its own out]");
    }

    // The game's use pick per tick [M Player_Activate 0x1a5170:100-133 + Collide_PickObj
    // 0x1eae80's 0x100/0x101 arm]: the use sphere r 1.0 at head + view-dir·1.0 vs the door's
    // live world sphere (obj+0x80/0x8C — the port's live_sph). The rider's look/turn steer the
    // view dir onto the sphere centre (the walkreal's real-aim pattern); the press fires the E
    // on edge ticks. Returns the tick's input and whether the pick would land now.
    fn aim_use(r: &Ride, di: usize) -> (Input, bool) {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[di];
        let (c, rad) = d.live_sph();
        let b = &sim.bond;
        let eye = [b.pos[0], b.pos[1] + b.eye - b.crouch_drop, b.pos[2]];
        let v = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-4);
        let use_pt = [eye[0] + v[0] / len, eye[1] + v[1] / len, eye[2] + v[2] / len];
        // The game's pick: the world-sphere candidate AND the use point within 1.0 in front of
        // one of the leaf's collision faces [M Collide_PickObj 0x1eae80; Collide_Intersect
        // 0x1ebaa8 → Intersect_PointGeom 0x1e92d8; nf_game::usepick].
        let near = (use_pt[0] - c[0]).powi(2) + (use_pt[1] - c[1]).powi(2) + (use_pt[2] - c[2]).powi(2)
            < (1.0 + rad) * (1.0 + rad)
            && nf_game::usepick::picks_in(&sim.world, use_pt, (c, rad), &d.matrix, d.glist);
        let want_yaw = v[0].atan2(v[2]);
        let err = walker::wrap_pi(want_yaw - b.yaw);
        let want_pitch = (v[1] / len).asin() / std::f32::consts::FRAC_PI_2;
        // Step to the car's edge toward the leaf while out of the pick's reach (the use point
        // must come within 1.0 of the leaf's face).
        let far = v[0].hypot(v[2]) > 1.85;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - b.pitch).clamp(-0.06, 0.06), forward: if far && err.abs() < 0.3 { 0.6 } else { 0.0 }, ..Default::default() };
        (input, near)
    }

    // (a) the −48 pass: the unlocked east pair #106 opens on the rider's E as the car levels
    // with its floor (the landing floors stack every 4 units; the pass window is ~130 ticks at
    // the keyed rate [D keys]).
    let mut opened_a = false;
    let mut closest = f32::MAX;
    for _ in 0..1600u32 {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list[d106];
            let h = sim.bond.head();
            let tris: Vec<[[f32; 3]; 3]> = sim.world.shape(d.glist).map(|t| t.iter().map(|q| q.map(|p| nf_formats::placement::transform(&d.matrix, p))).collect()).unwrap_or_default();
            // the smallest radius at which a sphere at the head meets a leaf face from in front
            let mut rr = 0.25f32;
            while rr < 6.0 && !nf_game::world::CollisionWorld::sphere_meets_tris(h, rr, &tris) { rr += 0.25; }
            closest = closest.min(rr);
        }
        let (input, near) = aim_use(&r, d106);
        r.step(input, near && r.t % 6 == 0);
        if r.w.scene.sim.as_ref().unwrap().drones.doors.list[d106].state & 0xc != 0 { opened_a = true; break; }
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(opened_a,
            "the unlocked −48 pair opens on the rider's E at the pass [M Door_Activate's use path], state {:#x}, the head's closest approach to a leaf face {closest}",
            sim.drones.doors.list[d106].state);
    }

    // (b) the −52 pass, still before t1610: the same E on the LOCKED pair #110 refuses — the
    // lock arm returns before any motion [M Door_Activate's `ch_lock` gate]; the car levels
    // with the −52 floor ~t1550 (the key line [D keys]) and ch 0x40 fires at t1610 [D].
    let mut pressed_b = false;
    for _ in 0..450u32 {
        let (input, near) = aim_use(&r, d110);
        let pressing = near && r.t % 6 == 0;
        if pressing { pressed_b = true; }
        r.step(input, pressing);
        if r.w.scene.sim.as_ref().unwrap().drones.channels.on(0x40) { break; } // t1610 arrived
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(pressed_b, "the E was pressed inside the locked pair's live sphere at the −52 pass");
        assert_eq!(sim.drones.doors.list[d110].state, nf_game::door::CLOSED,
            "sister: the locked pair refuses the E while ch 0x40 is low [M Door_Activate's lock arm; D #110 p3]");
        assert_eq!(sim.drones.doors.list[d110].ch_lock, 0x40, "the lock still armed before t1610");
    }

    // (c) the SP-kind sisters through the landing: the script's t1610 event writes ch 0x40 [D
    // keys] (the unlock the −52 pair test pins), the landing scene disables Bond at its t0 and
    // the handback re-enables him at its t575 [D 0x060006cf s9/s26; M Script_Free 0x1da880:46-49]
    // — and the twin SP #273 (the SAME script 0x060006cf, out ch 0x83, mask 1 [D #273]) never
    // fires: its trigger is its own out channel [M SP_Hit 0x1dc470:17-19] and nothing on this
    // level writes ch 0x83 [D scans, docs/systems/tower2-lift-ride.md §1]. The lobby scene's ch
    // 0x84 likewise awaits the bottom plate [D #62]. (The post-handback E on the −52 pair is NOT
    // a player move: the resumed car holds the key line 1.75 below the −52 floor already at the
    // handback — the key line [D keys; the port's ride] — so the door's live sphere is out of the
    // use reach for good; the rider's real E legs are the passes above.)
    let mut saw_disable = false;
    let mut enabled_again = false;
    for _ in 0..1400u32 {
        r.step(Input::default(), false);
        let sim = r.w.scene.sim.as_ref().unwrap();
        if sim.disabled { saw_disable = true; }
        if saw_disable && !sim.disabled { enabled_again = true; break; }
    }
    assert!(saw_disable && enabled_again,
        "the landing scene disables and the handback re-enables [D 0x060006cf s9/s26; M Script_Free 0x1da880]");
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x40), "ch 0x40 held through the scene [D keys]");
        assert!(!sim.drones.channels.on(0x83) && !sim.drones.channels.on(0x84),
            "sister: the twin SP #273 (out 0x83) never fired and the lobby scene (0x84) awaits the bottom plate [M SP_Hit 0x1dc470's out-channel trig]");
        // The doors: the −52 pair unlocked (the lock edge [M 0x1363c8] — the −52 landing test's
        // pin), and the ride-level door state untouched by the scene's own animation (the scene
        // keys no door stream [D 0x060006cf streams]).
        assert_ne!(sim.drones.doors.list[d110].ch_lock, 0x40,
            "the lock edge cleared the door's ch_lock [M Door_Update 0x1363c8]");
    }
}

/// KIND 0xea, the player's own WALK leg: the first real steps on the platform carry Bond into
/// the top reveal plate #277's volume (0.90–5.18, −9.87..−7.65, −5.74..−1.01 [D #277]) —
/// mask 8 (the player walk-in bit of Trigger_Hit's 0x18 player|drone gate [M 0x1b1fc8:56]) —
/// ch 0x4d rises, the objective reveals, and the TouchOnce deletes itself (the glist-0x10
/// delete [M Trigger_Update 0x1b1a78:72-74 case 1 + 0x1b1d78-0x1b1d90]). Sisters: ch 0x4d low
/// and the plate present-and-live at load — the walk, not the load, fires it.
#[test]
fn the_riders_first_steps_walk_onto_the_top_reveal_plate() {
    let mut r = Ride::new();
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x4d), "sister: ch 0x4d low at load [D #277 out]");
        let t = sim.drones.triggers.iter().find(|t| t.placement == 277).expect("plate #277");
        assert_eq!(t.out, 0x4d, "the reveal plate's out channel [D #277]");
        assert_eq!(t.mask & 0x8, 0x8, "the player walk-in mask bit [D #277; M Trigger_Hit 0x1b1fc8:56]");
        assert!(!t.gone, "sister: the TouchOnce is live before the walk");
        assert!(sim.bond.grounded, "Bond stands on the platform before the steps");
    }
    // The real steps (the walkreal board window's pattern): walk north to the volume's north
    // edge and back — turn + forward only, no teleports, no position writes.
    // Sister (measured 2026-11-02, this harness): 90 bare ticks of standing in the spawn
    // region (the script's whole pre-descent window) leave ch 0x4d LOW — the bare stand does
    // not fire the plate (walkreal-a's "the stand-in toucher misses on a pure vertical ride");
    // the player's own WALK onto the plate fires it. A port that fires the plate without the
    // walk fails here.
    for _ in 0..90u32 { r.step(Input::default(), false); }
    let bare_fired = r.w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d);
    assert!(!bare_fired,
        "sister: the bare stand does not fire the reveal plate — the walk does [M Trigger_Hit 0x1b1fc8:56-123's arm chain]");
    for k in 0..84u32 {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() };
        r.step(input, false);
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4d),
            "the walk-in fires the reveal plate's ch 0x4d (bare-stand first: {bare_fired}) [D #277; M Trigger_Update 0x1b1a78:72-74 case 1]");
        assert_eq!(sim.drones.mission.states[17][0], 3, "ch 0x4d reveals 'Gain access to lobby' [L record 4a]");
        let t = sim.drones.triggers.iter().find(|t| t.placement == 277).expect("plate #277 after");
        assert!(t.gone, "the TouchOnce deleted itself after firing [M 0x1b1d78-0x1b1d90's glist-0x10 delete]");
    }
}

/// KIND 0x31 + the 0xd9 lift SP #42 — the LIFT ride itself, bare: the player's only act is
/// standing on the platform. SP #42 "SCRIPTPLAYER_Lift" (mode 1 auto-start, out ch 0x22 [D
/// #42]) plays 0x06000571 and the platform descends the key line (y0 at key t90, y0 − 150 at
/// key t5040 [D keys]); at t1610 the script writes ch 0x40 [D keys], the −52 landing cut-scene
/// SP #289 triggers on it [M SP_Hit 0x1dc470:17-19's out-channel bit] and disables Bond; the
/// hand-back re-enables him ON the held platform [M Script_Free 0x1da880:46-49] while the
/// script's pause (event 3 [D 0x06000571]) keeps the car at the landing hold. Sisters: at load
/// ch 0x40 is low and every door closed — the ride, not the load, lands.
#[test]
fn the_bare_ride_carries_the_player_to_the_landing_hold() {
    let mut r = Ride::new();
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x40), "sister: ch 0x40 low at load [D keys — the t1610 writer]");
        assert!(sim.drones.doors.list.iter().all(|d| d.state == nf_game::door::CLOSED),
            "sister: every door closed at load");
    }
    let mut saw_disable = false;
    let mut enabled_again = false;
    for _ in 0..4000u32 {
        r.step(Input::default(), false);
        let sim = r.w.scene.sim.as_ref().unwrap();
        if sim.disabled { saw_disable = true; }
        if saw_disable && !sim.disabled { enabled_again = true; break; }
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let y0 = r.y0;
        let drop = y0 - sim.bond.pos[1];
        assert!(saw_disable && enabled_again,
            "the landing scene disables and the hand-back re-enables [D 0x060006cf s9/s26; M Script_Free 0x1da880]");
        assert!(sim.drones.channels.on(0x40), "the lift script's t1610 event wrote ch 0x40 [D keys]");
        assert!(sim.bond.grounded, "the hand-back grounds Bond on the held platform [M 0x194df0's cc rule]");
        assert!((42.0..50.0).contains(&drop),
            "the car holds at the landing: the key line put it y0 − 46.06 at t1610 and the script's pause keeps it [D keys] — drop {drop:.2}");
    }
}

/// KIND 0xd9, the SimpleScript the player TRIGGERS: the ride's end carries the player into the
/// bottom plate #62's volume (out ch 0x84 [D #62]); the lobby cut-scene SP #276 (script
/// 0x060006d3, out 0x84, mask 1 [D #276]) waits on its own out channel [M SP_Hit
/// 0x1dc470:17-19], runs (state 2 [M SP_Update 0x1dbda8's case 1→2]) under the banner, and the
/// mission's LevelEnd follows [M 0x182c60:73]. Sisters: at load ch 0x84 is low, the #276 player
/// waits (present, not deleted, not running), and no exit stands.
#[test]
fn the_bottom_plate_walk_in_triggers_the_lobby_script_and_the_level_end() {
    let mut r = Ride::new();
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x84), "sister: ch 0x84 low at load [D #62 out]");
        assert!(sim.drones.level_exit.is_none(), "sister: no level end at load");
        let p = sim.drones.scripts.players.iter().find(|p| p.scripts[0] == 0x0600_06d3)
            .expect("the lobby SP #276 exists at load");
        assert!(!p.deleted, "the lobby SP waits at load");
        assert_ne!(p.state, 2, "sister: the lobby script is not running at load [M SP_Update 0x1dbda8]");
    }
    // The bare ride to the bottom (the descent resumes after the landing hold): the car
    // carries Bond into plate #62's volume — the player's trigger is the ride's own walk-in.
    let mut saw_84 = false;
    let mut ran_276 = false;
    let mut exited = false;
    for _ in 0..9000u32 {
        r.step(Input::default(), false);
        let sim = r.w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x84) { saw_84 = true; }
        if sim.drones.scripts.players.iter().any(|p| p.scripts[0] == 0x0600_06d3 && p.state == 2) {
            ran_276 = true;
        }
        if sim.drones.level_exit.is_some() { exited = true; break; }
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(saw_84, "the bottom plate #62 fired ch 0x84 under the ride's walk-in [D #62]");
        assert!(ran_276,
            "the lobby script #276 ran under the banner [D #276; M SP_Update 0x1dbda8 case 1→2]");
        assert!(exited,
            "the mission's LevelEnd follows the bottom plate [M 0x182c60:73; walkreal's run: 0x84 then the exit ~800 ticks later]");
    }
}

/// The model list's kinds that m41 does NOT place [D census, docs/levels/0700004a.md §7]:
/// pickups (0xf0 — the m6/m14 rows are resource-map placements, not instantiated in play),
/// switches/consoles (0x29, m6 only), monitors (0x2d, m13 only), keypad locks (0x2e, m6 only),
/// breakables (0x20), rigid bodies (0xd8), SimpleScripts (0xe2) — the absent kinds have no
/// live leg on this map because the map places none of them. Sister: the kinds the map DOES
/// place keep the §7 counts (a parse or data regression fails here first).
#[test]
fn m41_places_none_of_the_pickup_switch_monitor_lock_breakable_rigid_or_simplescript_kinds() {
    for k in [0xf0u32, 0x29, 0x2d, 0x2e, 0x20, 0xd8, 0xe2] {
        assert!(m41_rows_of_kind(k as u8).is_empty(), "kind {k:#04x}: absent from m41 [D census]");
    }
    assert_eq!(m41_rows_of_kind(0xea).len(), 13, "13 TouchOnce plates on m41 [D census]");
    assert_eq!(m41_rows_of_kind(0xdb).len(), 162, "162 doors on m41 [D census]");
    assert_eq!(m41_rows_of_kind(0xd9).len(), 4, "4 SPs on m41 [D census]");
    assert_eq!(m41_rows_of_kind(0x31).len(), 1, "1 platform row on m41 [D census]");
}
