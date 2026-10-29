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
//! (27.88, 40.00, −23.00, auto-close) → the searchlight courtyard (the entry plates #525/#526 →
//! 0x52 stage SP #808's boot NIS; ch 55 rises when a head holds Bond — the alarm is NOT a fail
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

/// The Laser drawn with the real weapon-cycle input (the disc's own cutting laser, 0x4E) [D
/// USATxt label 0x05000043; the proven fusebox_06_live draw].
fn draw_laser(w: &mut Walker) {
    let draw = Input { switch_to: Some(0x4e), ..Default::default() };
    for _ in 0..40 { w.tick(draw, false); }
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

/// The walk from the spawn to the searchlight courtyard's tunnel-door approach (the point where
/// the game's own web [D Boundary Prime] splits: west into the tunnel, or the ledge route east
/// to the dressing room). `cut` does the stealth route's fusebox Laser cut in the guardroom
/// (group 0 → ch 57 + 103); without it the searchlights stay live and the courtyard raises the
/// ch-55 alarm — neither blocks the walk (no fail channel on this level [L record 1; M
/// 0x1827d8:98-107]).
fn walk_to_the_courtyard(w: &mut Walker, cut: bool) {
    // THE DISMOUNT: off the truck bed's open south side (the spawn's own facing walks onto the
    // bed; the probe drops grounded at z≈29).
    w.walk_nav_to([60.0, 38.3, 31.5], "off the truck, south", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] > 28.0, "the dismount clears the truck's south side, got {:?}", sim.bond.pos);
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
    w.walk_nav_to([26.5, 39.1, 13.1], "the well courtyard's west side", 1.6);
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
    // Through #47 into the guardroom; the nav graph covers this room [D leg probe: 4 nodes].
    w.walk_nav_to([29.9, 40.5, -17.5], "the guardroom fusebox nook", 1.4);
    if cut {
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
    // West out of the guardroom through the auto-closing door #59 Door_Top_I2 (27.88, 40.00, −23.00).
    w.walk_nav_to([27.9, 40.0, -21.0], "before door #59", 1.4);
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [27.88, 40.0, -23.0])
            .expect("door #59 Door_Top_I2 in the doors list [D census]");
        eprintln!("[06] door {di} at {pos:?}");
        w.use_door(di, "door #59");
    }
    // THROUGH the doorway (level-06-castle-fire): without this waypoint the mouth leg aims
    // through the guardroom's west wall (the auto-closing doorway is a nav blind spot).
    w.walk_to(Wp::new(26.6, 40.6, -23.6), "through door #59");
    // West up the steps to the searchlight courtyard mouth (the entry plates #525/#526 → 0x52;
    // SP #808's boot NIS fires and the walker skips it with the game's own skip action).
    w.walk_nav_to([8.4, 42.0, -12.0], "the searchlight courtyard mouth", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[06] at the mouth: ch52 {} ch55 {}", sim.drones.channels.on(52) as u8, sim.drones.channels.on(55) as u8);
    }
    fight_through(w);
    // South across the courtyard to the tunnel door's approach (3.75, 42.00, −51.13) [D leg
    // probe: the nav graph covers this courtyard, 7 nodes]. The courtyard floor here is the
    // web's 41.0 band [D Boundary Prime nodes 303-306: (3.4..5.3, 41.0, −48..−50)].
    w.walk_nav_to([4.4, 42.0, -48.5], "down the searchlight courtyard", 1.6);
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
fn walk_the_ledge_to_the_party(w: &mut Walker) {
    // East along the courtyard: the web's 41.1 east strip [D nodes 246 (10.74, 41.22, −34.28) /
    // 455-456 (14.0, 40.71, −34..−36.7) / 214 (25.27, 41.41, −40.6) / 215 (27.61, 41.15, −40.71)].
    w.walk_nav_to([10.5, 42.4, -34.3], "the courtyard's east strip", 1.6);
    w.walk_nav_to([14.0, 41.9, -36.7], "the courtyard's south-east corner", 1.6);
    w.walk_nav_to([25.3, 42.6, -40.6], "the east courtyard floor", 1.6);
    // South to the web's 41.15 band and the step's foot [D nodes 355/457/371].
    w.walk_nav_to([25.2, 42.3, -47.3], "the 43-step's foot", 1.6);
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
fn walk_the_castle_interior_to_the_party() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false,
        Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 57.18).abs() < 2.0 && (p[2] - 22.61).abs() < 2.0,
            "the carried ch-0x1D gate selects the truck-ride spawn #496 [M Player_Start 0x190fd0], got {p:?}");
    }
    // The level-start NIS: waited out through the game's own skip action.
    w.wait_enabled(3000, "the intro NIS");
    w.wait(30);

    walk_to_the_courtyard(&mut w, false);
    eprintln!("[06] reached the courtyard at tick {}", w.total_ticks);
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
    walk_the_ledge_to_the_party(&mut w);
    eprintln!("[06] the whole castle walked in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[06] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[06] stuck: {s}"); }
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
