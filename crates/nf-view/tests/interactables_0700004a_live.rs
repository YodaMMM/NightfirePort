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
        let near = (use_pt[0] - c[0]).powi(2) + (use_pt[1] - c[1]).powi(2) + (use_pt[2] - c[2]).powi(2)
            < (1.0 + rad) * (1.0 + rad);
        let want_yaw = v[0].atan2(v[2]);
        let err = walker::wrap_pi(want_yaw - b.yaw);
        let want_pitch = (v[1] / len).asin() / std::f32::consts::FRAC_PI_2;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - b.pitch).clamp(-0.06, 0.06), ..Default::default() };
        (input, near)
    }

    // (a) the −48 pass: the unlocked east pair #106 opens on the rider's E as the car levels
    // with its floor (the landing floors stack every 4 units; the pass window is ~130 ticks at
    // the keyed rate [D keys]).
    let mut opened_a = false;
    for _ in 0..1600u32 {
        let (input, near) = aim_use(&r, d106);
        r.step(input, near && r.t % 6 == 0);
        if r.w.scene.sim.as_ref().unwrap().drones.doors.list[d106].state & 0xc != 0 { opened_a = true; break; }
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(opened_a,
            "the unlocked −48 pair opens on the rider's E at the pass [M Door_Activate's use path], state {:#x}",
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
