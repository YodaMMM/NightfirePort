//! The walkreal-07000012 route, shared by the 07000012 and 07000013 tests (the warm chain walks
//! the same legs). Every step drives the REAL input path — no position writes on the route.

use crate::walker::Wp;

/// OPT-IN (civ-unignore, set by the 07000013 chain's d3 real-damage walk, checked at the one
/// labeled leg below — no other caller sets it, so the 07000012 and campaign walks are
/// byte-identical): before the south-corridor legs, stand at the "the south corridor east"
/// leg end and kill the corridor shotgun guard #10 (placement pos (19.7,−14.0), weapon 0x1c,
/// hp 12) through the ordinary fight machinery. The measured d3 ledger (NF_DMG_TRACE, master
/// 576909ed6): #10's first fire is ONE tick of 8 × −3.0 pellets at d 3.5 (tick 2723, Bond at
/// (16.8,−12.0) mid "the south corridor" leg) — the pre-engagement −24 the civ-repin-finish
/// report ranked as its first live check; the burst's own fight then killed him at 2764.
/// A player who spots the shotgun man before the corridor does not hand him the opening
/// blast: the stand runs only when the game's own exposed-part probe reads a reachable box
/// from the leg end (d 2.0), and a kill there means the burst never exists.
pub static PRE_CLEAR_SOUTH_CORRIDOR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// OPT-IN (set only by the 07000013 chain's d3 real-damage walk, checked at the one labeled leg
/// below — no other caller sets it, so every other walk is byte-identical): at "the north lobby 2"
/// leg end, stand and take the alive team-1 guards within 12 u that have an exposed part, nearest
/// first (up to three). The guards pursue to 1.0-2.0 [M 0x152548:174-181] and the leg resumed into
/// the pursuit after the fight's own cap left them alive — #6 (rifle 0x18, hp 30) walked Bond down
/// and unloaded 4 × −14.0 at d 2.5-2.9, the TooClose bonus [M DroneFiring_TooClose_Damage], for
/// the tick-5914 death (level-13-warm-chain-margin ledger [L]). The pre-clear takes them from the
/// leg end at d 4.6-8.2 before the north lobby 3 leg, the civ-unignore-2 law (the pre-clears
/// stand at the leg ends and take the posted guards BEFORE the legs walk into them).
pub static PRE_CLEAR_NORTH_LOBBY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// OPT-IN (set only by the 07000013 chain's d3 real-damage walk, checked at the three east
/// north-lobby legs below — no other caller sets it, so every other walk is byte-identical):
/// the north lobby's east legs recover from a fight displacement. The lobby's south wall (static
/// cel 374) blocks EVERY straight line from the lower floor south of the lobby to the lane (the
/// probe scanned x −12..−4 at z 15.75–17.5: all blocked; the wedge repro's own segment read
/// Some(0.127) [L probe_012_rebridge, this tree]), and the nav graph has no cross-path route
/// out (its route to the corridor detours through the y-0.45 level below, whose nodes cannot be
/// walked from y 1.03). What IS proven walkable is the route the level's own fights walked: the
/// lower floor's south lane z 5.2–9.4 (every node arrived on in the resupply runs), the passage
/// (−19.07,5.32) → (−20.09,5.24) through the corridor's east wall, the corridor north, and the
/// lobby mouth (−18.64,15.93). `reanchor_north_lobby` walks exactly those points (each hop
/// lenient: a wedged hop bails and the next hop re-aims), then the leg is re-walked from the
/// lane — a player who fought his way off the lane walks back the way he came and continues.
pub static REANCHOR_NORTH_LOBBY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// OPT-IN (set only by the 07000013 chain's d3 real-damage walk, checked at the one labeled leg
/// below — no other caller sets it, so every other walk is byte-identical): the lobby-east
/// posts flank. The melt ledger (warm-chain-margin-4 [L this tree's NF_DMG_TRACE run]): #6
/// (skin 0x5000006, hp 30) + #7 (skin 0x5000008, hp 12) open fire the moment the lane's west
/// half is crossed — −59.5 in 64 ticks from x −9.1, the fights' own exposure flickering
/// None→Some→None (he patrols 1.1 u behind the east furniture), so no in-place exchange wins;
/// and the pre-clear-from-the-lane-ends lever found nobody exposed at 14-16 u [L the run-8
/// ledger]. The measured lever ([L] the static probes + settles of this tree's scratch probe,
/// teleport-diagnosis only): the lobby's south band IS walkable floor (the settles ground y
/// 1.07-1.12 at (−8,14.5), (−5,14.5), (2.4,13.9)), the crossing column x −8.5 runs clear from
/// the lane (z 15.6) to z 13.4, the deep south band z 13.4-13.6 is blocked only at x
/// −3.0..−2.5, and the z 13.5-14 leg runs UNDER the posts' sight lines (the sight grid: the
/// lane's south edge unseen to x −7.5, the z 14.2 band seen only at x −7.5..−4.5 from #6's
/// then-current spot, the z 13.6 row clear but x −6 from the first probe). The flank (2.4,13.9)
/// is the d2 fight#6 trace's proven fight spot (#6 exposed, the taps landed −16.8/60 ticks
/// [L the d2 trace]) — both posts' boxes read exposed from there (the probe's mutual static
/// rays), #7 nearest (d ≈ 2.9, hp 12 — one fight), #6 from the flank after (d ≈ 3.3, hp 30).
/// The rejected levers are NOT repeated: no pre-clear at 14-16 u, no bridge_and_retry, no
/// in-place creep or longer stands (each measured dead in the margin-4 report).
pub static FLANK_LOBBY_EAST: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// OPT-IN (set only by the 07000011 chain's Agent and 00 Agent walks; the 07000012 and 07000013
/// walks never set it, so their legs are unchanged): "the east wing" leg aims at the corridor's
/// west side (x 23.2; the wall stops Bond near x 24.0). The office civilian #24 stands at
/// (25.35, 1.0, -0.04), 1.3 east of the old leg end, and a guard's round ends on the first body
/// on its line [M Bullet_CollisionHandler 0x12ce80 state 2; NDrone2_DealWithObjHit 0x13fff0:
/// 181-232 — a team-1 shooter lands on team 3]. The chain's Agent run had guard #3 open fire up
/// the wing as Bond stood at that leg end; his rounds killed her 19 ticks later ("Killed
/// Civilians", the restart). With this aim the run measured guard #3 dead at the next leg with
/// her alive. It changes where and when the fight starts; it does not keep her out of every line.
/// Per thread: the 07000011 binary runs its difficulties side by side.
thread_local! { pub static EAST_WING_WEST_SIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

/// OPT-IN (set only by the campaign_flow 12 leg; the 07000011/12/13 walks never set it, so their
/// legs are unchanged): at "the east wing corner" leg end, the wing guard placement #9 (weapon
/// 0x19) stands 2.7 u east of Bond, and on the joined tree he opened fire down the wing at Bond's
/// back as the route walked south; office civilian #24 had stepped into the wing (24.2-24.8, -1.9
/// to -2.6) and his round ended on her first ("Killed Civilians", the restart) — a team-1 round
/// lands on team 3 [M NDrone2_DealWithObjHit 0x13fff0:181-232]. A player deals with the gunman
/// he walks past instead of turning his back on him: the stand runs only while that guard lives
/// within 6 u of the corner.
thread_local! { pub static CLEAR_EAST_WING_GUARD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
/// The 543 west-stand crossing (los-mask-repins-12): opt-in per caller — the d3/d2 route keeps
/// its proven diagonal; only the Operative walk's legs are re-routed so far.
thread_local! { pub static DOOR_543_WEST_STAND: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

/// The displacement recovery for the east north-lobby legs (OPT-IN, `REANCHOR_NORTH_LOBBY`, the
/// d3 chain only): every point below was WALKED by this level's own resupply navs on this tree
/// (the run-5 ledger arrivals), so the chain rides proven clear edges, not a guess. The first
/// hop is a nav walk (the graph routes the lower floor's open band — resupply #3's own route
/// (−9.28,10.22)→(−18.09,7.42) and the probe's hop-1 arrival prove it), the rest are the
/// passage + corridor edges as direct legs, each lenient (try_walk_to: a wedged hop bails and
/// the next hop re-aims — the same self-healing walk_nav_to's per-node walks use).
fn reanchor_north_lobby(w: &mut crate::walker::Walker) {
    w.walk_nav_to([-16.49, 1.03, 5.60], "the re-anchor: the lower floor west", 1.3);
    const CHAIN: [&[f32; 3]; 6] = [
        &[-19.07, 1.03, 5.32],   // the passage lane east end (resupply #1 walked it west→east)
        &[-20.09, 1.03, 5.24],   // the passage through the corridor's east wall
        &[-21.32, 1.03, 6.25],   // the corridor (its west side)
        &[-20.61, 1.10, 9.15],   // the corridor north
        &[-20.26, 1.10, 14.06],  // the corridor mouth
        &[-18.64, 1.10, 15.93],  // the lobby mouth — on the lane
    ];
    for (k, p) in CHAIN.iter().enumerate() {
        let _ = w.try_walk_to(crate::walker::Wp::new(p[0], p[1], p[2]).r(1.3),
            &format!("the re-anchor {k}"));
    }
}

/// The lobby-east posts flank (OPT-IN, `FLANK_LOBBY_EAST`, the d3 chain only). The route rides
/// this tree's own static probes + settles ([L] the scratch probe, teleport-diagnosis only —
/// the PROOF is the real walk's ledger): the south turn at x −10.0 (the crossing 15.6→14.2
/// clear there; the lane end's own x −10.75 is wall at z 14.6), then east under the posts'
/// sight lines — the deep south band z 13.4-13.6, blocked only at x −3.0..−2.5 — to the flank
/// (2.4,13.9), where BOTH posts' boxes read exposed (the d2 fight#6 trace: the taps landed
/// −16.8/60 ticks from exactly this spot). #7 first (hp 12, the nearest — one fight), then #6
/// (hp 30) from the flank, both through the ordinary fight machinery (`stand_and_fire` — the
/// walk's own flags: the lane ray set, the tight gate, the dry switch, the back-off). Every hop
/// is lenient: a wedged hop bails and the next re-aims (the re-anchor's own shape). No position
/// writes, no RNG draws, no health/ammo writes.
fn flank_lobby_east(w: &mut crate::walker::Walker) {
    const HOPS: [&[f32; 2]; 7] = [
        &[-10.0, 15.4],  // east along the lane's south edge (the sight grid: unseen to x −7.5)
        &[-8.5, 13.9],   // the clear column x −8.5 (15.6→13.4 all clear [L the crossing scan])
        &[-6.5, 13.6],   // the deep south band, under the posts' sight lines
        &[-4.0, 13.6],
        &[-2.0, 13.6],
        &[0.6, 13.6],    // under the z 13.75 knee-wall (blocked only at z 13.75; z 13.4 clear)
        &[2.4, 13.9],    // the flank — the d2 fight#6 trace's spot
    ];
    for (k, p) in HOPS.iter().enumerate() {
        let _ = w.try_walk_to(crate::walker::Wp::new(p[0], 1.1, p[1]).r(0.9),
            &format!("the lobby-east flank {k}"));
    }
    // The fights, in the task's order: #7 (skin 0x5000008, hp 12) first, then #6 (skin
    // 0x5000006, hp 30) from the flank. Nearest-alive within 8 u per skin (the skins repeat
    // across the level's roster — the range keeps the pick on the lobby-east pair).
    for skin in [0x5000008u32, 0x5000006u32] {
        let (di, d) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let stand = sim.bond.pos;
            let Some(di) = sim.drones.list.iter().enumerate().filter(|(_, d)| {
                d.present && !d.corpse && d.health > 0.0 && d.team == 1 && d.skin == skin
                    && crate::walker::dxz(d.pos, stand) < 8.0
            })
            .min_by(|a, b| crate::walker::dxz(a.1.pos, stand).total_cmp(&crate::walker::dxz(b.1.pos, stand)))
            .map(|(i, _)| i)
            else { eprintln!("[012] the flank: no alive skin {skin:#x} within 8 u — walking on"); continue };
            (di, crate::walker::dxz(sim.drones.list[di].pos, stand))
        };
        let t0 = w.total_ticks;
        let killed = w.stand_and_fire(di, 2400);
        eprintln!("[012] the flank stand on skin {skin:#x} #{di} at d {d:.1}: killed {killed} in {} ticks, health {:.1}",
            w.total_ticks - t0, w.scene.sim.as_ref().unwrap().drones.health.health);
    }
}

/// The walkreal-12 route: spawn → the keypad burn → the console E. Used again by the 07000013
/// test's warm chain.
pub fn walk_12(w: &mut crate::walker::Walker) {
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48), "the spawn touch #679 fires ch 0x48 [D]");
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "07000011's 'Gain access to lower office floor' completes (the late AND) [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][0], 3,
            "'Activate elevator override control' announces [L record 07000012 obj 0]");
    }

    // Leg 1: south through the lobby, around the shaft core to the keypad by door[19] — walked.
    // (The core's east wall x ≈ 7.0-7.5 walls z 6..16.5 [D probes]; the crossing is at its south
    // end, the keypad wall z ≈ 0.)
    w.walk_to(Wp::new(14.0, 1.1, 6.0).r(1.2), "the lobby south");
    w.walk_to(Wp::new(10.0, 1.1, 2.0).r(1.2), "the floor south 2");
    w.walk_to(Wp::new(9.3, 1.1, -0.6).r(1.0), "the keypad approach");

    // Leg 2: the Q-Decryptor keypad burn — the real G cycle equips 0x4E (the loadout owns it [D];
    // the gadget→gadget prev-slot pollution is fixed), then the held beam burns the lock.
    {
        let mut cycles = 0;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.gun.weapon == 0x4e { break; }
            assert!(cycles < 14, "the G cycle never reached the Q-Decryptor (0x4E)");
            w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); } // the draw cycle
            cycles += 1;
        }
        assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Q-Decryptor is held [M runs_beam]");
    }
    // A keypad opens by its typed code or the decryptor; the laser arm is kind 1 only [M 0x1ae3d8:47, :162].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        if let Some(li) = sim.drones.locks.iter().position(|l| l.out == 0xa && !l.unlocked) {
            let code = sim.drones.key_codes.code(sim.drones.locks[li].code);
            sim.drones.keypad = Some((li, nf_game::lock::Keypad::arm(code, false)));
            for c in code { sim.drones.keypad_digit((c - b'0') as u32); }
        }
    }
    let lock_at = [7.0f32, 1.6, 0.2];
    let start = w.total_ticks;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0xa) { break; }
        assert!(w.total_ticks - start < 600, "the typed code never raised the keypad channel (ch 0xa) [M 0x1ae3d8 case 4]");
        let p = sim.bond.pos;
        let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
        let dx = lock_at[0] - p[0];
        let dz = lock_at[2] - p[2];
        let dy = lock_at[1] - eye;
        let desired = dx.atan2(dz);
        let err = crate::walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / crate::walker::dxz(p, lock_at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.05, 0.05),
            look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        // The burn is the held fire's: each held tick spawns the decryptor's beam shot [M
        // 0x1a5460:601-694's held-fire refire] and Lock_Update 0x1ae3d8:194 burns on its contact
        // (fix-gadgets2's held-fire model — the released fire spawns no shot and the burn stops).
        w.tick_fire(input);
    }
    eprintln!("[012] the keypad lock burned at +{} ticks (ch 0xa) [M 0x1ae3d8:194]", w.total_ticks - start);
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The unlock evidence, difficulty-independent: on Agent (the walkreal load, difficulty 3)
        // the burn's ch 0xa edge opens the door and the assert window is inside the 360-tick
        // auto-close; on Operative (the campaign's difficulty-1 load) the create-unlock's per-tick
        // hold [M Lock_Update 0x1ae3d8:192-196] fires the SAME unlock edge at tick 1 — the door
        // opened then and the 360-tick auto-close [M 0x13660c-0x136700] may legitimately have shut
        // it again by the time the route walks here. The unlock's own marks are what Door_Update's
        // arm leaves behind on both paths: the lock channel cleared + the busy bit
        // [M 0x1363c8:105-107; the 0x1336f8 two-latch snapshot].
        let d19 = &sim.drones.doors.list[19];
        // The unlock evidence, difficulty-independent: ch_lock cleared is what Door_Update's
        // unlock arm leaves on BOTH paths — on Agent (the walkreal load, difficulty 3) the burn's
        // ch 0xa edge, on Operative (the campaign's difficulty-1 load) the create-unlock's per-tick
        // hold [M Lock_Update 0x1ae3d8:192-196] firing the same edge at tick 1 (the door is open
        // from spawn there and the 360-tick auto-close [M 0x13660c-0x136700] may have shut it
        // again long before the route arrives — the state is timing-dependent, the unlock is not).
        // The warm-load busy-flag path (flags 0x3, no 0x10) differs from the cold Agent run — see
        // the report's live checks.
        assert!(d19.ch_lock == 0,
            "door[19] unlocked (the ch 0xa arm clears +0x68 [M 0x1363c8:105-107]); got ch_lock {} flags {:#x} state {:#x}",
            d19.ch_lock, d19.flags, d19.state);
        eprintln!("[012] door[19] plc {} unlocked: ch_lock 0, state {:#x} flags {:#x}", d19.placement, d19.state, d19.flags);
    }

    // Leg 3: through the opened door[19] and north-west to the console — walked. The doorway is
    // the corridor's jamb opening z −1.462..−0.262 at x 6.87..6.92 [D the corridors_a_5 collision]
    // and the security room's own frame, which lines up with it to 4 mm once the room cel sits at
    // the identity [M parseentity_fixup_entity 0x1d00c0 zeroes a 0xc047 room cel's translation;
    // the placement's z 0.2955 had put a 0.904 pinch here]. The r 0.55 capsule [M Player_Collision
    // 0x194df0] crosses on z −0.81..−0.91; the game crosses on z −0.812..−0.794 [L PCSX2
    // 2026-10-31, the walk west along z −0.55 from x 8.4, frames 15-30].
    w.walk_to(Wp::new(6.2, 1.1, -0.86).r(0.6), "through the door[19] doorway");
    w.walk_to(Wp::new(1.0, 1.1, 4.0).r(1.5), "the office floor north");
    // Up to the desk: the keyboard lies 0.9 below the head, so the use sphere reaches it only from
    // within ~1.3 in z (from z 6.22 the use point misses it by 0.02).
    w.walk_to(Wp::new(-1.55, 0.5, 7.2).r(0.4), "the console");

    // The E press on the console (#731 override + #702 lift call, the same volume [D]). The pick
    // tests the console's glist — the keyboard mesh on the desk at y ≈ 0.86..0.91 — against the
    // sphere r 1.0 at head + look [M Player_Activate 0x1a5170:100-108 → Intersect_PointGeom
    // 0x1e92d8], so the player turns to the keyboard (−1.55, 0.88, 8.2) first, as a player does;
    // E pulses one tick in six (the edge) while the view settles on it.
    let kb = [-1.55f32, 0.88, 8.2];
    let mut pressed = false;
    for k in 0..120 {
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
            let err = crate::walker::wrap_pi((kb[0] - p[0]).atan2(kb[2] - p[2]) - sim.bond.yaw);
            let want_pitch = ((kb[1] - eye) / crate::walker::dxz(p, kb).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08), ..Default::default() }
        };
        w.tick(input, k % 6 == 5);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x49) && sim.drones.channels.on(0x59) { pressed = true; break; }
    }
    assert!(pressed, "E at the console writes ch 0x49 (override #731) and ch 0x59 (lift call #702) [D]");
    // The game's frame order: `Mission_Update` (0x182c60) runs from `Game_Run` (0x1c9790:17)
    // BEFORE the object handler (0x1336f8:30-35) — the E press fires at Bond's turn INSIDE the
    // walk, so the objective completes on the channels the NEXT frame. The port's use-trigger
    // half is deferred to the same mid-walk slot (level-0c-complete-2); one more tick is the
    // game's own timing, not slack.
    w.tick(nf_game::player::Input::default(), false);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x49) && sim.drones.channels.on(0x59),
            "ch 0x49/0x59 hold into the next frame (TouchOnce latched, not cleared) [M 0x1b1a78 case 1]");
        assert_eq!(sim.drones.mission.states[16][0], 4, "the override objective completes [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][1], 3, "'Escape through the elevator shaft' announces [L]");
    }
}

/// Leg 4: the lift call button #699 (ch 0x58 — the security-centre shutters), then the REAL walk
/// into the shaft lobby and the gated exit (the shared tail of the route).
pub fn walk_12_exit(w: &mut crate::walker::Walker) {
    // The lift call button #699 (E-touch, out 0x58, mask 4 [D trig dump]: lo (0.87, 0.86, 8.08)
    // hi (1.37, 0.91, 8.34)) sits on the north wall's face, east of the override console (#702
    // shares the console volume, out 0x59). Walk to its face and hold E: ch 0x58 swaps BOTH
    // SimpleScript pairs (sh_shuttersclosed_LEFT #657 ↔ sh_shuttersopen_LEFT #676 and
    // sh_screen-shutter_down #697 ↔ _up #698 [D census §5]).
    let ss_at = |w: &crate::walker::Walker, plc: usize| -> [f32; 3] {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sim.drones.sss.list.iter().find(|s| s.placement == plc)
            .unwrap_or_else(|| panic!("ss #{plc} is an animated SS [D census §5]"));
        [s.matrix[3][0], s.matrix[3][1], s.matrix[3][2]]
    };
    let (sh_closed_0, sh_open_0, scr_down_0, scr_up_0) =
        (ss_at(w, 657), ss_at(w, 676), ss_at(w, 697), ss_at(w, 698));
    w.walk_to(Wp::new(1.1, 0.9, 7.4).r(0.5), "the lift call button #699");
    let mut shutters = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x58) { shutters = true; break; }
    }
    assert!(shutters, "E at #699 writes ch 0x58 [D trig 699 out 0x58; M Trigger_Activate 0x1b2318 type 1]");
    // The swap needs the SS tick: wait, then assert both pairs exchanged (complementary
    // keyframes — the playhead runs to the last key while the channel is on [M SS_Update 0x1a9948]).
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), false); }
    {
        let (sh_closed_1, sh_open_1, scr_down_1, scr_up_1) =
            (ss_at(w, 657), ss_at(w, 676), ss_at(w, 697), ss_at(w, 698));
        let moved = |a: [f32; 3], b: [f32; 3]| (a[1] - b[1]).abs() > 0.5 || (a[2] - b[2]).abs() > 0.5;
        assert!(moved(sh_closed_0, sh_closed_1) && moved(sh_open_0, sh_open_1),
            "ch 0x58 swaps the shutter pair #657/#676 [M SS_Update; D census §5] (closed {sh_closed_0:?}→{sh_closed_1:?}, open {sh_open_0:?}→{sh_open_1:?})");
        assert!(moved(scr_down_0, scr_down_1) && moved(scr_up_0, scr_up_1),
            "ch 0x58 swaps the screen-shutter pair #697/#698 [D census §5] (down {scr_down_0:?}→{scr_down_1:?}, up {scr_up_0:?}→{scr_up_1:?})");
    }

    // The lift is entered from the NORTH lobby: the leaves #677/#678 shut at (2.21, 1.30, 13.18)
    // [D census §5] face north, the shaft touch #683 (z 12.10..13.08) and the gated exit #669
    // (z 9.47..10.45) lie south of them. The security room has no way north — its north wall
    // (south face z 8.80, x 2.19..5.53) and the round pilaster close it once the room cel sits at
    // the identity [M parseentity_fixup_entity 0x1d00c0]; the game agrees [L PCSX2 2026-10-31:
    // walked north from (5.0, 6.9) Bond stops at z 7.49, from (6.0, 6.8) at z 8.16]. (The old
    // "0.88 pinch into a lobby ramp" was the room's 0.2955 z shift opening a false gap against
    // the lift-lobby wall.) So the route goes back out through door[19] and round the floor's
    // south and west wings to the north lobby: the free-space path the capsule fits [D the static
    // collision, a 0.25 m grid flood], every door on it opened with E.
    w.walk_to(Wp::new(4.0, 1.1, 5.0).r(0.6), "back across the security room");
    w.walk_to(Wp::new(6.0, 1.1, -0.86).r(0.6), "door[19], the west side");
    w.walk_to(Wp::new(8.4, 1.1, -0.6).r(0.6), "back through door[19]");
    let legs: &[(f32, f32, &str)] = &[
        (10.75, -0.50, "the corridor"), (11.25, 1.25, "the corridor north"), (13.25, 3.25, "the east lobby"),
        (21.00, 3.50, "the east lobby east"), (24.25, 2.75, "the east wing corner"), (24.00, -1.00, "the east wing"),
        (23.50, -4.50, "the east wing south"), (22.25, -7.25, "the south-east bend"), (20.75, -9.75, "the south-east bend 2"),
        (19.00, -12.00, "the south corridor east"), (15.00, -12.00, "the south corridor"), (11.00, -12.00, "the south corridor west"),
    ];
    for &(x, z, label) in legs {
        let x = if label == "the east wing" && EAST_WING_WEST_SIDE.with(|c| c.get()) { 23.2 } else { x };
        // 00 Agent real health: the stuck handler opens door plc 543 (9.9, -14.5) on this leg and
        // its swinging leaf pins Bond at (9.26, -12.67), 1.9 u short of the target; the next leg
        // walks through that same door, so the leg is lenient there (no cap panic).
        let lenient = label == "the south corridor west" && !w.tops_up()
            && w.scene.sim.as_ref().unwrap().drones.difficulty >= 3;
        let prev = w.lenient;
        if lenient { w.lenient = true; }
        w.walk_to(Wp::new(x, 1.1, z).r(1.0), label);
        w.lenient = prev;
        if label == "the east wing corner" && CLEAR_EAST_WING_GUARD.with(|c| c.get()) {
            let di = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
                d.placement == 9 && d.present && !d.corpse && d.health > 0.0 && d.team == 1
                    && crate::walker::dxz(d.pos, w.scene.sim.as_ref().unwrap().bond.pos) < 6.0
            });
            if let Some(di) = di {
                // The loadout's pistol (0x10, clip 7 on this chain) — the keypad burn left the
                // Q-Decryptor in hand. A real wheel request, then the draw.
                for _ in 0..4 {
                    if w.scene.sim.as_ref().unwrap().gun.weapon == 0x10 { break; }
                    w.tick(nf_game::player::Input { switch_to: Some(0x10), ..Default::default() }, false);
                    for _ in 0..60 {
                        let g = &w.scene.sim.as_ref().unwrap().gun;
                        if g.weapon == 0x10 && g.switch_state == 0 { break; }
                        w.tick(nf_game::player::Input::default(), false);
                    }
                }
                let armed = w.scene.sim.as_ref().unwrap().gun.weapon == 0x10;
                let t0 = w.total_ticks;
                let killed = armed && w.stand_and_fire(di, 600);
                eprintln!("[012] the east wing guard #{di} (placement 9) from the corner: armed {armed} (gun {:#x}) killed {killed} in {} ticks",
                    w.scene.sim.as_ref().unwrap().gun.weapon, w.total_ticks - t0);
            }
        }
        if label == "the south-east bend 2" && PRE_CLEAR_SOUTH_CORRIDOR.load(std::sync::atomic::Ordering::Relaxed) {
            let di = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
                d.present && !d.corpse && d.health > 0.0 && d.team == 1
                    && crate::walker::dxz(d.pos, [19.7, 1.0, -14.0]) < 3.0
            });
            if let Some(di) = di {
                let exposed = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    crate::walker::DamageLog::exposed_part(sim, di)
                };
                if exposed.is_some() {
                    eprintln!("[012] pre-clearing the corridor shotgun guard #{di} from the bend-2 leg end (d {:.1})",
                        crate::walker::dxz(w.scene.sim.as_ref().unwrap().drones.list[di].pos, [20.75, 1.0, -9.75]));
                    let killed = w.stand_and_fire(di, 600);
                    eprintln!("[012] the corridor pre-clear: killed {killed}");
                } else {
                    eprintln!("[012] the corridor shotgun guard has no exposed part from the leg end — walking on (the burst ledger as measured)");
                }
            }
        }
    }
    // Door plc 543 (kind 0 swing, hinge (9.95, 1.10, -14.46) [M the shut matrix], in the corridor's
    // west wall, hole z -13.86..-15.15 [D the static collision]). The game lets Bond through on the
    // north approach: [L PCSX2 2026-11-10, fresh load, guards asleep] Bond at (9.9, 1.1, -13.0)
    // facing south, Use pressed, stick forward 240 frames → (6.21, 1.13, -25.40), 11 u past the
    // door; the port reproduces the same walk to (6.25, 1.13, -25.39). The stand point sits in the
    // wall's north recess: walked from the corridor Bond rests at (9.28, -12.84) [L the d1 run] —
    // still west of the hinge line 9.95, the same swing side. The old (8.5, -13.5) diagonal's
    // stuck-E landed EAST of the hinge and swung the leaf into the corridor's west mouth; the leg
    // wedged 6000 ticks [L the d1 run]. Opt-in: DOOR_543_WEST_STAND (the d1 route only — the d3/d2
    // diagonal is the proven route there; this one shifts their fight timelines).
    if DOOR_543_WEST_STAND.with(|c| c.get()) {
        // los-mask-repins-12-2: the approach walks the geometry the wall plane admits — the
        // corridor's west wall is SOLID north of the hole (z −13.86..−15.15 [D the static
        // collision]), so a west steer at z −12.85 pins Bond against it (the d1 run: 6000-tick
        // cap at (10.62,−12.85) [L]; the stuck ladder's door answer no-ops on the open leaf and
        // never escalates), and the old run only passed the approach because the walker's
        // never-decaying nudge left slid him south into the band by accident. The band is the
        // crossing: south along the corridor to z −14.40 first, then through to the west stand.
        w.walk_to(Wp::new(10.55, 1.10, -12.85).r(0.5), "door plc 543, the corridor wall");
        w.walk_to(Wp::new(10.55, 1.10, -14.40).r(0.5), "door plc 543, the hole band south");
        w.walk_to(Wp::new(9.30, 1.10, -12.85).r(0.4), "door plc 543, the west stand");
        {
            let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.placement == 543)
                .expect("door plc 543 [D]");
            w.use_door(di, "door plc 543");
            for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); }
            assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state, nf_game::door::OPEN,
                "E from west of the hinge opens door plc 543 [M Door_Activate 0x135be0; L the live fact]");
        }
        // The crossing follows the live fact's line — STRAIGHT SOUTH from the stand [L the game's
        // Bond held the stick south and drifted to (6.21, -25.40); a SW-steered waypoint digs the
        // stuck-mitigation sidestep into the wall's east dead-end (10.62, -15.73) [L the d1 run]].
        w.walk_to(Wp::new(9.00, 1.10, -17.50).r(1.3), "through door plc 543");
    } else {
        w.walk_to(Wp::new(8.50, 1.1, -13.50).r(1.0), "through door plc 543");
    }
    let legs: &[(f32, f32, &str)] = &[
        (6.50, -15.50, "the south passage"), (6.50, -19.50, "the south passage 2"),
        (5.75, -22.75, "the south hall"), (-9.75, -22.25, "the south hall west"),
    ];
    for &(x, z, label) in legs {
        w.walk_to(Wp::new(x, 1.1, z).r(1.0), label);
    }
    // Door plc 545 (−10.58, −19.29) opens on E only (the game's too: it stays shut on approach and
    // opens on Square [L PCSX2 2026-10-31]); Bond then crosses on the game's own line, z −18.70..−18.81
    // from x −10.0 to −12.4 [L same run].
    w.walk_to(Wp::new(-9.90, 1.1, -20.60).r(0.4), "door plc 545, the approach");
    {
        let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter().position(|d| d.placement == 545)
            .expect("door plc 545 [D]");
        w.use_door(di, "door plc 545");
        for _ in 0..30 { w.tick(nf_game::player::Input { forward: -0.6, ..Default::default() }, false); } // clear the swing
        for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
        assert_eq!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state, nf_game::door::OPEN,
            "E opens door plc 545 [M Door_Activate 0x135be0]");
    }
    let legs: &[(f32, f32, &str)] = &[
        (-10.00, -18.80, "through door plc 545"), (-12.40, -18.70, "door plc 545, the west side"),
        (-18.75, -19.25, "the west hall"), (-21.75, -18.25, "the west hall 2"), (-23.75, -16.25, "the south-west bend"),
        (-25.25, -13.75, "the west wing south"), (-25.25, -1.75, "the west wing"), (-22.50, -0.50, "the west wing bend"),
        (-20.75, 1.75, "the west wing bend 2"), (-20.50, 5.50, "the west corridor"), (-20.50, 13.50, "the west corridor north"),
        (-18.75, 15.75, "the north lobby west end"), (-10.75, 15.75, "the north lobby"), (-8.00, 17.00, "the north lobby 2"),
        (-4.00, 17.00, "the north lobby 3"), (-1.00, 16.00, "the north lobby east"),
    ];
    for &(x, z, label) in legs {
        // warm-chain-margin-5 (OPT-IN `FLANK_LOBBY_EAST`, the d3 walk only): the lobby-east
        // posts' flank replaces the lane crossing at "the north lobby 2" — the melt ledger
        // (margin-4) killed every in-place answer from the lane. The flank walks the south band
        // under the posts' sight lines to the (2.4,13.9) fight spot, takes #7 then #6, and the
        // walk rejoins at "the lift doors" — the remaining lane legs (2/3/east) are behind him
        // (the posts are the lane's east guards; with them dead the lane is walked out).
        // Every other caller runs master's plain walk_to byte-identical.
        if label == "the north lobby 2" && FLANK_LOBBY_EAST.load(std::sync::atomic::Ordering::Relaxed) {
            flank_lobby_east(w);
            continue;
        }
        if matches!(label, "the north lobby 3" | "the north lobby east")
            && FLANK_LOBBY_EAST.load(std::sync::atomic::Ordering::Relaxed) {
            continue;
        }
        // warm-chain-margin-4 (OPT-IN `REANCHOR_NORTH_LOBBY`, the d3 walk only): the east
        // north-lobby legs' displacement recovery — try the leg leniently first; if the fights'
        // resupply fetches left Bond south of the lobby wall (the leg's direct line blocked,
        // the 6000-tick wedge of the margin-3 ledger), walk the proven re-anchor chain back to
        // the lane and re-walk the leg from there. Every other caller runs master's plain
        // walk_to byte-identical.
        let reanchored = REANCHOR_NORTH_LOBBY.load(std::sync::atomic::Ordering::Relaxed)
            && matches!(label, "the north lobby 2" | "the north lobby 3" | "the north lobby east");
        if reanchored {
            let already = { let sim = w.scene.sim.as_ref().unwrap(); crate::walker::dxz(sim.bond.pos, [x, 1.1, z]) < 1.0 };
            if !already {
                let _ = w.try_walk_to(Wp::new(x, 1.1, z).r(1.0), label);
            }
            let arrived = { let sim = w.scene.sim.as_ref().unwrap(); crate::walker::dxz(sim.bond.pos, [x, 1.1, z]) < 1.0 };
            if !arrived {
                reanchor_north_lobby(w);
                w.walk_to(Wp::new(x, 1.1, z).r(1.0), label);
            }
        } else {
            w.walk_to(Wp::new(x, 1.1, z).r(1.0), label);
        }
        // warm-chain-margin-3 (the level-13-warm-chain-margin-2 lever, ported to master): the
        // flag-gated north-lobby pre-clear — only the 07000013 chain's d3 real-damage walk sets
        // PRE_CLEAR_NORTH_LOBBY, so every other caller walks the legs byte-identical.
        // warm-chain-margin-4: the sweep fires at BOTH lane ends — the "the north lobby" end
        // (BEFORE the east legs walk into the lobby-east posts' sight; the d3 melt ledger [L
        // this tree: #6 rifle −7/hit + #7 −3.5/hit landed −59.5 in 64 ticks from x −9.1, the
        // fight's own gun-ray exposure flickered None→Some→None and the taps landed −3.5 per
        // ~4 rounds — no in-place exchange wins that) — and the "the north lobby 2" end (the
        // late spawns the east legs meet). The radius opens to 16.0 (the posts stand 14-15.5 u
        // from the west end: #6 (3.3,17.2) d 14.6, #7 (4.7,15.5) d 15.5 [L the fight#6 trace])
        // and the stands cap at 2400 (the d3 stands' 700-cap law is the 13 lobby's unseen-
        // respawner bleed; these posts are 14-16 u lanes whose kill budget is ~9 body taps —
        // a 600-cap stand cannot finish one). The flag gates every line — no other caller
        // changes.
        if matches!(label, "the north lobby" | "the north lobby 2")
            && PRE_CLEAR_NORTH_LOBBY.load(std::sync::atomic::Ordering::Relaxed) {
            for _ in 0..4 {
                let (di, d) = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let stand = sim.bond.pos;
                    let Some(di) = sim.drones.list.iter().enumerate().filter(|(_, d)| {
                        d.present && !d.corpse && d.health > 0.0 && d.team == 1
                            && crate::walker::dxz(d.pos, stand) < 16.0
                    })
                    .min_by(|a, b| crate::walker::dxz(a.1.pos, stand).total_cmp(&crate::walker::dxz(b.1.pos, stand)))
                    .map(|(i, _)| i)
                    else { break };
                    if crate::walker::DamageLog::exposed_part(sim, di).is_none() { break }
                    (di, crate::walker::dxz(sim.drones.list[di].pos, stand))
                };
                let killed = w.stand_and_fire(di, 2400);
                eprintln!("[012] the north lobby pre-clear: guard #{di} at d {d:.1} killed {killed}");
                if !killed { break }
            }
        }
    }
    w.walk_to(Wp::new(2.25, 1.1, 15.25).r(0.6), "the lift doors");
    w.walk_to(Wp::new(2.2, 0.7, 12.55).r(0.4), "into the shaft entrance");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x1f), "the shaft touch #683 holds ch 0x1F [D]");
        assert_eq!(sim.drones.level_exit, Some(0x0700_004a),
            "the gated exit #669 loads 0700004A [M 0x1b1a78 case 10]");
        let door1 = sim.drones.sss.list.iter().position(|s| s.placement == 677)
            .expect("lft_LIFTDOOR1 (#677) is an animated SS [D]");
        let x = sim.drones.sss.list[door1].matrix[3][0];
        assert!((x - 3.070).abs() < 0.06,
            "the lift door leaf stands at key 1 (x ≈ 3.070 [D]) after the call, got {x}");
    }
    eprintln!("[012] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}
