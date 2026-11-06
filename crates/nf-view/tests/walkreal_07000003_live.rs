//! walkreal: 07000003 "Alpine Escape" (Castle C) walked with the player's own movement — no
//! teleports after the spawn. The route (docs/levels/07000003.md + the disc dump):
//!   the 0x24 start (−8.85, 6.44, −23.19; the 0x4d objective-reveal touch fires under Bond) →
//!   the front gate (door[0] at (14.7, 6.6, −35.4) starts locked behind ch 0x31; the E-plate
//!   kind-1 mask-4 at (11.8, 6.0, −35.6) writes 0x31, which unlocks the door and plays the
//!   gate SS #502, ch 0x31, 2 keys — the "key-gated route") → the 0x21 guard-enable plates →
//!   the servants (skin 0x0500002b ×3, team 3, death channel 0x60 = protected) → the east lawn →
//!   the Load Point #451 (18.11, 6.65, 16.05; E, key gate ch 0x52).
//!
//! The objective channels: the disc's mux family writes the gate —
//!   AND(0x4f, 0x50) → 0x51 (obj1 "Rescue Mayhew's servants") and AND(0x4e, 0x51) → 0x52
//!   (obj2's reveal AND the exit's key gate) [D the kind-3 triggers, out 0x51 list 4f,50 and
//!   out 0x52 list 4e,51]. The LEAF writers are on the disc and ported:
//!   - 0x4e — the Dragon Safe key: E on the case (SP #110, script 0x06000563, mask 4 use-key
//!     [M SP_Activate 0x1dc338:82]) spawns the kind-2 pickup with +0x28 = 0x4e [D script stream 1;
//!     M Pickup_Handler 0x18e650:330-333 writes it on take];
//!   - 0x4f/0x50 — the servants' rescue channels, each placement's param 6 = the game's
//!     Drone+0x135 (#389 carries 0x4f, #388 0x50, #422 none [D probe; 07000002's servants carry
//!     0x49/0x4A the same way [D]]): the enable plate (p3) starts the HOSTAGE cower [D
//!     DroneTypeSettings row 5 start 0xa via dtype 0xe + prop 0x4e; M 0x14c938], the captor — the
//!     nearest armed alive guard at the entering tick [M 0x144030; the port's Drones tick] — dying
//!     moves the Hostage UPDATE to HostageSaved [M 0x15cb78:24-36], whose ENTER raises the channel
//!     [M 0x15ccf0 case 1 → DroneFunc_HostageSaved 0x1441c0:11-18].
//!   0x53 (obj2's watch) is written by 07000004's story script [D 07000003-audit] — in this level
//!   obj2 can only reach state 3 (announced), and the mission ENDS on the exit trigger, not on
//!   mission completion. The short route below still hands the leaves at the moments the level
//!   offers (documented stand-in, not the whole-mission walk); the full-route test drives every
//!   leaf through the real input.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The G-055 flush's kill-loot cycle (g055-4, test-local): the fights' drops rearm the walk —
/// the walk-over take [M Pickup_Handler 0x18e650] files the weapon + its clip, then the deep
/// firearm cycle holds a real gun with rounds (the merged-tree real-damage run spent its health
/// in the servants'-ground fights: 88 of 100 across the trio's crossfire before the exit lawn
/// finished Bond at tick 56836).
fn loot_the_drops_03(w: &mut Walker, label: &str) {
    for _ in 0..4 {
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let t = sim.drones.pickups.iter().filter(|p| !p.gone)
            .filter(|p| (p.m[3][1] - bp[1]).abs() < 1.5)
            .map(|p| [p.m[3][0], p.m[3][1], p.m[3][2]])
            .min_by(|a, b| walker::dxz(*a, bp).total_cmp(&walker::dxz(*b, bp)))
            .filter(|t| walker::dxz(*t, bp) < 8.0);
        let Some(t) = t else { break };
        w.walk_nav_to(t, &format!("{label} loot"), 0.6);
    }
    for _ in 0..12 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let g = &sim.gun;
            if (2..=0x47).contains(&g.weapon) && g.weapon != 0x34 && g.weapon != 0x37
                && g.clip + g.spare >= 5 { return; }
        }
        w.tick(nf_game::player::Input { weapon_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    }
    w.switch_to_armed(label);
}

/// The mission-table row and objective indices for this level [L ee_start_05 @ 0x2a3f18 via
/// mission::c07000003]: obj0 0x4e "Retrieve Dragon Safe contents", obj1 0x51 "Rescue Mayhew's
/// servants", obj2 0x53 "Find door leading to Mayhew" (enable 0x52).
/// Shoot every enabled, alerted hostile within `r` of Bond on his floor, nearest first, through the
/// real fire path. With the placement stats (accuracy / aggression / reaction [M DoModeSettingsNEW
/// 0x14c600:81-107]; bisected: green before that merge, red after) four live guards stood on
/// Bond's next step at (10.4, 6.5, −31.5) in state 0x5C, and the west-lane and east-lane legs
/// pressed into them for whole caps until Bond left the floor. A player clears the guards in his
/// way.
fn clear_near(w: &mut Walker, r: f32, label: &str) {
    for k in 0..8 {
        let near = {
            let sim = w.scene.sim.as_ref().unwrap();
            let b = sim.bond.pos;
            sim.drones.list.iter().enumerate()
                .filter(|(_, d)| d.team == 1 && d.enabled && d.present && !d.corpse && d.health > 0.0
                    && d.state != nf_game::drone::state::HOSTAGE && (d.pos[1] - b[1]).abs() < 2.0 && walker::dxz(d.pos, b) < r)
                .min_by(|a, c| walker::dxz(a.1.pos, b).total_cmp(&walker::dxz(c.1.pos, b)))
                .map(|(i, _)| i)
        };
        let Some(di) = near else { return };
        // The far+unexposed skip (g055-4's 16-walk lever): a guard whose boxes no eye ray
        // reaches (the door-8 cluster #18/#19/#2 behind the leaves [D the 00agent-02-03-2
        // note]) is not a fight — the measured d2 run's flank nav walked INTO the doorway's
        // sweep line and #18's point-blank volleys (38.7 in 16 hits, hp 12 — the taps never
        // landed through the door) finished Bond at 58345. The route's own legs engage the
        // cluster through the open doorway (the green runs fought from (36.2,5.7) THROUGH it).
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if walker::dxz(sim.drones.list[di].pos, sim.bond.pos) > 8.0 && w.exposed_aim(di).is_none() { return; }
        }
        let _ = w.switch_to_armed(label);
        if !w.shoot(di, &format!("{label} guard {k}")) { let _ = w.resupply(label); }
    }
}

fn mission_row() -> (usize, usize, usize, usize) {
    let li = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0003).expect("the 07000003 mission record");
    (li, 0, 1, 2)
}

#[test]
fn walk_the_alpine_route_to_the_exit() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -8.85).abs() < 2.0 && (p[2] - -23.19).abs() < 2.0, "the 0x24 spawn [D], got {p:?}");
        assert!(!sim.disabled, "no intro blocks this level's start");
    }
    w.wait(60);
    {
        // The spawn's objective-reveal touch (out 0x4d at (−9.0,−23.2) [D dump]) — both first
        // objectives' enable channel, so the monitor announces them [M 0x1827d8:55-67].
        let sim = w.scene.sim.as_ref().unwrap();
        let (li, o0, o1, _) = mission_row();
        assert!(sim.drones.channels.on(0x4d), "the spawn reveal touch fires under Bond [D]");
        assert_eq!(sim.drones.mission.states[li][o0], 3, "obj0 announced through the real monitor");
        assert_eq!(sim.drones.mission.states[li][o1], 3, "obj1 announced through the real monitor");
    }

    // The route: north-east across the courtyard to the Load Point. The nav graph fills between
    // the anchors; the guard-enable plates (0x21/0x23/0x24/0x25) fire under Bond's feet.
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("mid courtyard", [13.0, 5.9, -18.0]),
        ("the east lawn", [22.0, 5.9, 5.0]),
        ("the exit lawn", [18.5, 5.9, 13.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }

    // 6. The exit: E on the Load Point #451 without ch 0x52 refuses [M Trigger_Activate 0x1b2318:76-90].
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E without the key");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x52), "neither objective fired, so the AND(0x4e,0x51) has not written 0x52 [D]");
        assert_eq!(sim.drones.level_exit, None, "the key gate refuses the exit without ch 0x52 [M 0x1b2318:76-90]");
    }
    // The short route's stand-in: hand the LEAF channels at the moments the level offers — the
    // real writers (the servants' p6 rescue channels and the safe key's take) are driven by the
    // full-route test below; 0x51 and 0x52 are the mux family's OUTPUTS (the AND triggers drive
    // them every tick, so a direct hand is overwritten [M 0x1b1a78's unconditional AND write; the
    // recon]). Everything downstream is the disc's own mux family: AND(0x4f,0x50) writes 0x51
    // [D kind-3 out 0x51 list 4f,50], then AND(0x4e,0x51) writes 0x52 [D out 0x52 list 4e,51],
    // which announces obj2 (enable 0x52) and unlocks the exit.
    let sim = w.scene.sim.as_mut().unwrap();
    for &c in &[0x4eu32, 0x4f, 0x50] { sim.drones.channels.set(c, 1, sim.drones.frame); }
    drop(sim);
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let (li, _, o1, o2) = mission_row();
        assert!(sim.drones.channels.on(0x51), "the disc's AND(0x4f,0x50) writes obj1's channel [D]");
        assert!(sim.drones.channels.on(0x52), "the disc's AND(0x4e,0x51) writes the key gate [D]");
        assert_eq!(sim.drones.mission.states[li][o1], 4, "obj1 COMPLETE through the real monitor [M 0x1827d8:78-85]");
        assert_eq!(sim.drones.mission.states[li][o2], 3, "obj2 announced when its enable channel 0x52 fires [M 0x1827d8:55-67]");
    }
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E with the key");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0004), "the exit loads 07000004 through the real reload path");
    eprintln!("[03] walked to the exit in {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[03] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[03] stuck: {s}"); }
}

/// The full play: every objective, every leaf written by the disc's own writers under real input
/// (walking, turning, E, real fire) — the servants rescued through their enable plates + captor
/// kills, the Dragon Safe opened and its key taken, the exit fired. No channel or state pokes.
#[test]
fn walk_the_full_route_and_complete_every_objective() { walk_the_full_route_and_complete_every_objective_at(1) }

/// The walk body at `difficulty` (1 Operative = the plain load's replay parity; 2 Agent rides the
/// front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]).
fn walk_the_full_route_and_complete_every_objective_at(difficulty: u32) {
    let iso = iso();
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000003.bin", false).expect("load 07000003")
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000003.bin", false, None, difficulty)
            .expect("load 07000003")
    });
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.difficulty, difficulty,
        "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
    // The type-2 guards carry 12 health, captains 24 [L ee_02b.bin]: Bond aims for the head
    // (`Walker::prefer_head`), as a player does, or the scripted fights run the walk dry.
    w.prefer_head = true;
    // Agent tactics (real-health-agent-difficulty), opt-in for the difficulty-2 sister: at
    // DroneFiring_Accuracy_Normal 0.8 the shotgun (weapon 0x1C) guards close in — under 3.0 u
    // every drone round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] and their 8-pellet
    // volleys cost ~3.3 health a pellet at Plr_DMod_Normal 0.7 (the sister's first run died at
    // tick 40108: 52.1 in 16 hits from a guard at 2.8 u). The 02 walk's keep_off/back_off 4.0
    // (the same `Walker` arms its real-health walk pins green).
    if difficulty >= 2 {
        w.keep_off = Some(4.0);
        w.dmg.back_off = Some(4.0);
    }
    if difficulty >= 3 {
        // 00 Agent (real-health-00agent-02-03-2): the west-lawn fights on drone #8 (weapon 0x12)
        // burned five 900-tick caps at "killed false" while his −1.8 chips bled Bond out at
        // t25850 [L the 00agent ledger; the NF_RF_TRACE run: aim Some, blocked true every
        // sample] — the gun line to #8 crosses a NOT-YET-ENABLED guard (the 0x23/0x24/0x25
        // plates fire later on the route) and the stand never shoots through a drone (the
        // blocker hold). The game's own collide drops Enable(0) objects before the hit test
        // [M Collide_PickObj 0x1eae80; Check_AutoAim 0x1a73f8 skips +0xF4 == 1] — a disabled
        // drone never stops a round — so the hold is not game-shaped there. The opt-in
        // fire_through_disabled drops disabled drones from the line; live hostiles on it are
        // still answered by shoot_blockers (the retarget reads team == 1, so the servants
        // (team 3, death ch 0x60) keep their hold and are never answered to).
        w.dmg.shoot_blockers = true;
        w.dmg.fire_through_disabled = true;
        // The lane's dry in-hand fights (the 0x12 clip 0 vs the P99's 15 in reserve) — the
        // game's own mid-fight cycle [L the 00agent-02-03-2 ledger].
        w.dmg.switch_dry = true;
    }
    let (li, o0, o1, o2) = mission_row();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - -8.85).abs() < 2.0 && (p[2] - -23.19).abs() < 2.0, "the 0x24 spawn [D], got {p:?}");
    }
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4d), "the spawn reveal [D]");
        assert_eq!(sim.drones.mission.states[li][o0], 3, "obj0 announced [M 0x1827d8:55-67]");
        assert_eq!(sim.drones.mission.states[li][o1], 3, "obj1 announced [M 0x1827d8:55-67]");
    }

    // The servants' identity snapshot, BEFORE any fight: the p6 rescue bytes are the placements'
    // own words [D p6], static until PreDroneControl consumes one [M 0x148db0:51-90] — and the
    // real-damage run's courtyard fight CAN consume 0x4f early: the approach crosses the east
    // 0x21 twin (measured: ch 0x21 at tick 7165), #389 binds a courtyard captor [M 0x144030], and
    // the fight's kills reach that captor (Hostage case 3's captor-death check [M 0x15cb78:24-36]
    // → HostageSaved → switch_channels[+0x135] [M 0x1441c0:11-18]) — the game's own designed save,
    // just earlier than the scripted leg. Sampling after the fights read a consumed byte (0x0)
    // and the old "the servant carrying 0x4f [D p6]" lookup panicked; the rescue loop's live
    // `channels.on(rescue_ch)` check already skips the captor shot for an early rescue.
    let servants: Vec<(usize, [f32; 3], u8)> = {
        let sim = w.scene.sim.as_ref().unwrap();
        let s: Vec<(usize, [f32; 3], u8)> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.skin & 0xffff == 0x2b && d.team == 3)
            .map(|(i, d)| (i, d.pos, d.second_channel)).collect();
        for (i, p, ch) in &s {
            let d = &sim.drones.list[*i];
            eprintln!("[03] servant #{i} at ({:.1},{:.1},{:.1}) ch {ch:#x} state {:#x} switch {:?} on {}",
                p[0], p[1], p[2], d.state, d.switch_channel, d.channel_on);
        }
        s
    };
    assert_eq!(servants.len(), 3, "three protected servants [D the dump]");

    // The route legs up to the servants' ground (the guard-enable plates 0x21/0x23/0x24/0x25 fire
    // under Bond's feet as he crosses them [D the TouchOnce census]).
    for (name, at) in [
        ("courtyard east", [0.0f32, 5.9, -20.0]),
        ("mid courtyard", [13.0, 5.9, -18.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }

    // 1. "Rescue Mayhew's servants" — the leaves 0x4f (servant #389) and 0x50 (#388) are the
    //    placements' param 6 [D probe]. Each servant's enable plate (p3) enables the servant AND
    //    the guards sharing the channel [D], the servant starts the HOSTAGE cower (DroneTypeSettings
    //    row 5 start 0xa) and the captor binds. Killing the captor completes the save: Hostage
    //    UPDATE → HostageSaved ENTER → DroneFunc_HostageSaved writes switch_channels[+0x135 = p6]
    //    [M 0x15cb78:24-36, 0x15ccf0 case 1, 0x1441c0:11-18].
    //    The enable channel 0x21 hangs off TWO kind-1 plates [D two out-0x21 placements]. The east
    //    twin (centre 14.3, −33.3; footprint x 13.1..15.4, z −35.2..−31.5 [D the trigger lo/hi])
    //    is cut off from every standable lane by the garden moat ditch: the straight walk from the
    //    gate-plate lawn burns the 6000-tick cap and the nav route parks on the ledge above (y
    //    9.37 — the prior session's stuck dump). The WEST twin (centre 8.98, −34.35; footprint x
    //    7.27..10.69, z −35.05..−33.65 [D the trigger lo/hi]) IS crossed on real input: the west
    //    lane's approach to the gate-plate lawn stands inside it at (9.8, 6.37, −34.67) and ch 0x21
    //    fires (a plate probe run on the same legs as below). So the plate is fired
    //    BEFORE the servant walk — #389 is already HOSTAGE with its captor bound when Bond arrives
    //    [M 0x144030's bind at the HOSTAGE entering tick].
    let sim = w.scene.sim.as_ref().unwrap();
    let plate0x25 = sim.drones.triggers.iter()
        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x25).map(|t| t.centre())
        .expect("servant #388's enable plate");
    drop(sim);
    // The 0x21 enable: the west lane to the gate-plate lawn (each leg [D the plate lo/hi] — the
    // last one's nav node stands inside the west twin's footprint).
    for (name, at) in [
        ("west lawn", [0.0f32, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("along the moat", [8.0, 5.9, -36.0]),
        ("gate plate lawn", [11.0, 5.9, -35.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x21),
            "the west-lane approach stands inside the west 0x21 plate's footprint [D lo/hi x 7.3..10.7 z −35.0..−33.7]");
    }
    // Back to the servants' ground by the east lane the "mid courtyard" leg itself walked
    // (x 10.0-10.6 from the moat to the door at (11.8, 6.5, -20.2): the nav stops Bond reached
    // on that leg). The nav's own way "back across the courtyard" to (13, -18) runs through the
    // obstacle at (6.35, -15.7..-17.7) (Bond climbs to y 7.3, airborne) and wedges at every cap, the
    // servant approach and the captor flanks with it — the captor (mode 4, dodging at
    // x 10-15, z -21..-25) stays out of sight across the wall.
    w.lenient = true;
    for (name, at) in [
        ("the east lane south", [10.05f32, 6.56, -31.87]),
        ("the east lane", [10.27, 6.65, -30.01]),
        ("the east lane north", [10.59, 6.47, -22.91]),
    ] {
        clear_near(&mut w, 8.0, name);
        // audit-voice-triggers-3: lenient is re-armed per leg — the clear_near/shoot internals
        // reset it on their way out, and the 2026-11-05 run's wedged leg panicked at the
        // non-lenient cap instead of returning to the ladder below.
        w.lenient = true;
        let mut got = w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, name);
        // The wedge ladder: the lane climb wedges from the moat's south edge (the 2026-11-05
        // run: the mid anchor's nav slid Bond DOWN the ditch to (10.28, 6.34, -36.05), the
        // sidesteps never finding the ramp; segment_to_target hit at t 0.10). A player does not
        // stand in a ditch: sidestep ALONG the moat (the moat legs above walked it fine), then
        // take the anchor again from the stepped-aside spot. Both sides fail: skip the anchor —
        // the anchors steer the nav, the NEXT anchor re-derives its own path and the route
        // continues (the same doctrine as the 01 walk's wedged-lane ladders).
        if got == u32::MAX {
            let bp = w.scene.sim.as_ref().unwrap().bond.pos;
            for (side, sname) in [(1.0f32, "east"), (-1.0f32, "west")] {
                let to = [bp[0] + side * 2.5, bp[1], bp[2] + side * 1.5];
                w.walk_nav_to(to, &format!("{name}: the moat sidestep {sname} (the wedged nav's ladder)"), 1.2);
                w.lenient = true;
                got = w.walk_to(Wp { at, act: Act::None, arrive: 1.0 }, &format!("{name} (after the {sname} sidestep)"));
                if got != u32::MAX { break; }
            }
            if got == u32::MAX {
                eprintln!("[03] {name}: both sidesteps wedged — skipping the anchor, the next leg re-derives the path");
            }
        }
    }
    w.lenient = false;
    if difficulty >= 3 {
        // 00 Agent (real-health-00agent-02-03-2): the servants' ground is a running fight (the
        // 0x24/0x25 guards and their captains) and the real-damage hook's resupply is the lane
        // walks' arm, not this walk's. A player scavenges the dead guards' drops between
        // contacts: the dry P99 (clip 1 spare 0) met shotgun captain #5 at 1.6 u and died to
        // his volley at t42315 [L the 00agent-02-03-2 ledger]. resupply walks the covered path
        // only (the 9-u corridor rule), so this stays on the route.
        let _ = w.resupply("before the servants' rescue (00agent)");
    }
    for (plate, want_ch) in [(None, 0x4fu8), (Some(plate0x25), 0x50u8)] {
        let (si, at, rescue_ch) = *servants.iter().find(|(_, _, ch)| *ch == want_ch)
            .unwrap_or_else(|| panic!("the servant carrying {want_ch:#x} [D p6]"));
        // #389's 0x21 plate already fired on the lane approach; #388's 0x25 plate is walked here.
        if let Some(plate) = plate {
            if difficulty >= 3 {
                // 00 Agent: rounds before the plate walk (the running fight on the servants'
                // ground; the dry-gun death above) [L the 00agent-02-03-2 ledger].
                let _ = w.resupply(&format!("before the {want_ch:#x} plate (00agent)"));
            }
            w.walk_nav_to(plate, &format!("onto the {want_ch:#x}-servant's enable plate"), 1.8);
            if !w.scene.sim.as_ref().unwrap().drones.channels.on(0x25) {
                // An awake guard standing in the double doors' sweep on this leg (doors 4/5 at
                // (19.9/21.8, −20.25)) turns the leaves back each tick — they shuttle between
                // pos01 0.71 and 0.79 and the doorway never clears [L NF_DOORTRACE]. A player
                // shoots him: drop every live armed guard within 3 of the doorway, then re-walk.
                let blockers: Vec<usize> = {
                    let sim = w.scene.sim.as_ref().unwrap();
                    sim.drones.list.iter().enumerate()
                        .filter(|(_, d)| d.present && !d.corpse && d.health > 0.0 && d.team != 3
                            && ((d.pos[0] - 20.9).powi(2) + (d.pos[2] + 20.25).powi(2)).sqrt() < 3.0)
                        .map(|(i, _)| i).collect()
                };
                for gi in blockers {
                    w.shoot(gi, "the guard in the 0x25 doorway");
                }
                w.walk_nav_to(plate, &format!("onto the {want_ch:#x}-servant's enable plate (again)"), 1.8);
            }
        }
        w.wait(10);
        if difficulty >= 3 {
            // 00 Agent: rounds before the servant walk (the same running-fight rule) [L the
            // 00agent-02-03-2 ledger].
            let _ = w.resupply(&format!("before servant {si} (00agent)"));
        }
        // The G-055 flush's second-block guards hold the servants' ground posts and cross-fire
        // the approach (the merged-tree real-damage run: the trio #14/#2/#18 each cost 24-34
        // health during the servant navs, 88 total, and the exit lawn finished the last 6.8).
        // A player clears the ground one guard at a time from where he stands and scoops the
        // drops (the kill-loot cycle) — the fights shorten with every kill.
        clear_near(&mut w, 9.0, "the servants' ground");
        loot_the_drops_03(&mut w, "the servants' ground");
        // gas-rng-draws: the door-8 cluster (#18/#19/#2 behind the leaves at (38.7, 0.2)) can
        // outlive the ground clear — clear_near's far+unexposed skip passes them by while the
        // leaves hide their boxes, and the servant nav then walks THROUGH the opening door
        // into #18's point-blank volley (the fresh d2 ledger: killed at 56606 at (40.30, 0.29),
        // 1.6 u, health 6.2 entering the leg). A player at a door that just hissed open does
        // not step through blind: he fights what shows through the mouth from the hinge side.
        // Self-gating on the cluster being alive keeps every stream that already cleared it
        // byte-identical (the green runs fought the cluster through this doorway earlier).
        {
            let cluster_alive = {
                let sim = w.scene.sim.as_ref().unwrap();
                // The anchor is the cluster's LOAD band [L the gas-rng-draws-6 probe:
                // #18 (27.92, 6.25, −5.57) hp 12 weapon 0x1c, #2 (30.69, 6.25, −7.55) hp 12
                // weapon 0xe, #19 (37.01, 6.25, −15.04) hp 12 weapon 0xe, all disabled state
                // 0x1, and door #8 (35.62, 6.51, 2.71) closed state 0x2 pos01 0.50; no other
                // hostile stands within 8 of the band centre]. The first anchor — the
                // POST-open pocket (38.7, 0.2) r 6.0 they walk to AFTER the door opens —
                // never matched the load spots, so the block never fired on the failing
                // stream. No `enabled` in the filter: the door-8 members load DISABLED behind
                // the leaves (the 01 ch-0xF pocket law — disabled objects are dropped from
                // bullet collision [M Collide_PickObj 0x1eae80:82]) and pop when the door
                // opens — which is exactly why the mouth walk must run first. Only a corpse
                // or a finished fight keeps the block a no-op (the green streams cleared
                // them).
                sim.drones.list.iter().any(|d| {
                    d.team == 1 && !d.corpse && d.health > 0.0
                        && walker::dxz(d.pos, [31.9f32, 6.25f32, -9.4f32]) < 8.0
                })
            };
            if cluster_alive {
                w.walk_nav_to([35.6f32, 6.4f32, 3.4f32], "the door-8 mouth (the cluster holds)", 1.0);
                // The mouth stand opens door #8 the way the mid-nav approach did [L the d2
                // ledger — the door opened under the servant nav]; the popped members then
                // walk their pocket route (to (38.7, 0.2) [L the d2 ledger]) — ~6.9 u from
                // the mouth, inside clear_near's 9 band once they arrive. The settle loop
                // gives them that walk and fights them from the hinge side (the green runs'
                // own fight shape from (36.2, 5.7) THROUGH the doorway), instead of stepping
                // off into the servant nav with the fight unanswered.
                for round in 0..5 {
                    w.wait(90);
                    let at_pocket = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        sim.drones.list.iter().filter(|d| d.team == 1 && !d.corpse && d.health > 0.0
                            && walker::dxz(d.pos, [38.7f32, 6.2f32, 0.2f32]) < 7.0).count()
                    };
                    clear_near(&mut w, 9.0, "the door-8 mouth");
                    if at_pocket == 0 && round >= 1 { break; }
                }
            }
        }
        w.walk_nav_to([at[0], at[1] - 0.3, at[2]], &format!("servant {si}"), 1.7);
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[si];
        assert!(!d.corpse, "servant {si} alive when Bond arrives");
        assert_eq!(d.death_channel, 0x60, "servant {si} is protected [D death_ch]");
        let captor = d.captor;
        let ch_on = sim.drones.channels.on(rescue_ch as u32);
        drop(sim);
        if !ch_on {
            // The captor (nearest armed alive guard at the HOSTAGE entering tick [M 0x144030; the
            // port's Drones tick]) — kill it with real fire; its death is the save's trigger.
            if let Some(ci) = captor {
                {
                    let sim = w.scene.sim.as_ref().unwrap();
                    let c = &sim.drones.list[ci];
                    eprintln!("[03] servant {si}'s captor #{ci} at ({:.1},{:.1},{:.1}) state {:#x} hp {:.1} corpse {}",
                        c.pos[0], c.pos[1], c.pos[2], c.state, c.health, c.corpse);
                }
                // 00 Agent (real-health-00agent-02-03-3): the east captor's room guards (the
                // door-8 cluster at (35.5-37.5, -0.6) — #19 hp 12, #18 hp 12, #2 hp 24 shotgun)
                // shot Bond out of the captor flank's REACTIVE fights: under the guards' door
                // routes the flipped kill order (#2 first) left #18 live into a second fight and
                // his 28.8 volley (40.8 -> 12.0) was the whole green margin — the
                // pre-door-routes run answered the same cluster from (36.2,5.7) for 12.0 total
                // and finished at 28.8 [L the two NF_RF_TRACE ledgers, identical to t48500]. A
                // player takes the room's fight spot FIRST (the flank-1 nav-3 ground the green
                // run fought from — parts expose from there; from the servant ground they do
                // not, and the flank walk into the shotgun's 3.0 band died at 26.4 [L this
                // branch's first pre-clear run]) and clears the cluster hp-ascending with real
                // fire before the duel. si==0 only — the west (0x4f) leg's reactive sequence is
                // green as-is and is not touched.
                if difficulty >= 3 && si == 0 {
                    w.walk_nav_to([36.5, 5.9, 5.7], "the captor room rally", 1.5);
                    let escorts: Vec<usize> = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let mut es: Vec<(f32, usize)> = sim.drones.list.iter().enumerate()
                            .filter(|(i, d)| Some(*i) != captor && d.present && !d.corpse && d.health > 0.0
                                && d.team == 1
                                && walker::dxz(d.pos, [36.5, 5.9, 5.7]) <= 9.0)
                            .map(|(i, d)| (d.health, i)).collect();
                        es.sort_by(|a, b| a.0.total_cmp(&b.0));
                        es.into_iter().map(|(_, i)| i).collect()
                    };
                    for gi in escorts {
                        let (alive, gp, hp) = {
                            let sim = w.scene.sim.as_ref().unwrap();
                            let g = &sim.drones.list[gi];
                            (!g.corpse && g.health > 0.0, g.pos, g.health)
                        };
                        if !alive { continue; }
                        eprintln!("[03] captor room guard #{gi} at ({:.1},{:.1},{:.1}) hp {hp:.1} — pre-clear", gp[0], gp[1], gp[2]);
                        if !w.shoot(gi, &format!("servant {si} captor room guard {gi}")) {
                            if w.resupply("captor room ammo") {
                                w.shoot(gi, &format!("servant {si} captor room guard {gi} (retry)"));
                            }
                        }
                    }
                }
                let mut killed = w.shoot(ci, &format!("servant {si}'s captor"));
                // The mode-4 dodging captor pattern (the east-lane legs' own note: a captor
                // "stays out of sight across the wall"): a player chases him — walk to his LIVE
                // position, top the clip back up (clear_near's own resupply-on-fail: the earlier
                // fights drain it) and reshoot, until the line clears.
                for round in 0..4 {
                    if killed { break; }
                    let _ = w.resupply(&format!("servant {si}'s captor chase {round}"));
                    let (cpos, gone) = {
                        let sim = w.scene.sim.as_ref().unwrap();
                        let c = &sim.drones.list[ci];
                        (c.pos, c.corpse || c.health <= 0.0)
                    };
                    if gone { killed = true; break; }
                    eprintln!("[03] captor {ci} chase round {round} at ({:.1},{:.1},{:.1})", cpos[0], cpos[1], cpos[2]);
                    w.walk_nav_to(cpos, &format!("onto servant {si}'s captor (chase {round})"), 3.5);
                    killed = w.shoot(ci, &format!("servant {si}'s captor (chase {round})"));
                }
                assert!(killed, "servant {si}'s captor #{ci} died to the chase");
            } else {
                eprintln!("[03] servant {si}: no captor bound (ch {rescue_ch:#x} off) — the save rule reads an empty link as saved [M 0x15cb78:24-36]");
            }
            w.wait(30);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[si];
        assert!(!d.corpse, "servant {si} survived the rescue");
        assert!(d.state != 0x01 && d.state != 0x0a,
            "servant {si} left WAIT_SWITCH/HOSTAGE [M the save path], got {:#x}", d.state);
        eprintln!("[03] servant {si} post-save state {:#x}", d.state);
        assert!(sim.drones.channels.on(rescue_ch as u32),
            "the save raised servant {si}'s rescue channel {rescue_ch:#x} [D p6; M 0x1441c0:11-18]");
        if difficulty >= 3 && si == 0 {
            // 00 Agent (real-health-00agent-02-03-3): the green run's post-rescue west-lawn leg
            // started from the chase's end (38.34,0.72) and its nav threads the building's east
            // side through the (39.0,12.9) door; from the rally the nav detours through the
            // (35.6,8.7) door instead and that E stalls the leg 29x3000 ticks [L the first rally
            // run's ledger]. The player walks the chase's last leg: restore the green
            // continuation point (the flank's own nav link, walked there in the green run).
            w.walk_nav_to([38.4, 5.9, 0.8], "the east flank end", 1.6);
        }
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x51), "the disc's AND(0x4f,0x50) writes obj1's channel [D]");
        assert_eq!(sim.drones.mission.states[li][o1], 4, "obj1 COMPLETE through the real monitor [M 0x1827d8:78-85]");
    }

    // 2. The front gate. The plate (kind 1, mask 4 = E-usable [M Trigger_Activate 0x1b2318:38-49])
    //    writes 0x31; the door starts locked behind it and the gate SS #502 listens on 0x31. The
    //    approach is the west lane (z≈−33..−36): the straight courtyard line crosses the garden
    //    moat ditch ((11.8,−34.5) and (16,−31) drop into the water [D the dump's 5 water cels]).
    for (name, at) in [
        ("west lawn", [0.0f32, 5.9, -30.0]),
        ("the west lane", [1.0, 5.9, -33.0]),
        ("along the moat", [8.0, 5.9, -36.0]),
    ] {
        w.walk_nav_to(at, name, 1.6);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    let gate_plate = sim.drones.triggers.iter()
        .find(|t| t.kind == walker::trigger_kind::TOUCH_ONCE && t.out == 0x31).map(|t| t.centre())
        .expect("the gate plate");
    let gate_door = sim.drones.doors.list.iter()
        .position(|d| d.ch_lock == 0x31).expect("the locked gate door");
    let gate_ss = sim.drones.sss.list.iter()
        .position(|s| s.channel == 0x31).expect("the gate SS");
    drop(sim);
    w.walk_to(Wp { at: gate_plate, act: Act::None, arrive: 1.4 }, "to the gate plate");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x31), "the gate is shut before the plate [sister]");
        assert_eq!(sim.drones.doors.list[gate_door].state, 2, "the gate door sits closed at create [M Door_Create 0x136100 +0xf4 = 2]");
    }
    // Sister first: E on the locked door refuses (the ch_lock refusal [M Door_Activate 0x135be0 —
    // the ch_lock gate + the hit feedback]).
    let sim = w.scene.sim.as_ref().unwrap();
    let gate_at = sim.drones.doors.list[gate_door].pos;
    drop(sim);
    w.walk_to(Wp { at: gate_at, act: Act::Use { use_ticks: 90 }, arrive: 1.3 }, "E the locked gate door");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[gate_door];
        assert!(d.pos01 < 0.5 && d.state != 4, "the locked door refuses E while ch 0x31 is off [M 0x135be0] — state {:#x} pos01 {}", d.state, d.pos01);
    }
    // The plate: E writes 0x31, the door's lock channel turns on, the gate SS plays its keys —
    // a 2-key one-shot (wrap flag non-zero → the wrap disarms to state 0 [M 0x1a9948:119-128]) whose
    // pose HOLDS the end key, so pin the posed matrix leaving its rest pose.
    let ss_rest = w.scene.sim.as_ref().unwrap().drones.sss.list[gate_ss].matrix;
    w.walk_to(Wp { at: gate_plate, act: Act::Use { use_ticks: 90 }, arrive: 1.4 }, "E on the gate plate");
    w.wait(20);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x31), "the plate writes ch 0x31 [D out 0x31 mask 4]");
        let ss = &sim.drones.sss.list[gate_ss];
        assert!(ss.matrix != ss_rest || ss.t > 0.0 || ss.state == 1,
            "the gate SS plays its keys on 0x31 [D ss #502 ch 0x31], got state {:#x} t {}", ss.state, ss.t);
    }
    w.wait(90);
    // Now the door opens to E and Bond walks through to the gate's north side.
    w.walk_to(Wp { at: gate_at, act: Act::Use { use_ticks: 120 }, arrive: 1.3 }, "E the unlocked gate door");
    w.walk_nav_to([14.7, 5.9, -36.8], "through the gate", 1.6);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] < -36.0, "Bond is behind the front gate [route], got {:?}", sim.bond.pos);
    }

    // 3. obj0 "Retrieve Dragon Safe contents" through the REAL chain: behind the gate, E on the
    //    Dragon Safe (SP #110, script 0x06000563 [D]) spawns the kind-2 key pickup with +0x28 =
    //    0x4e [M Pickup_Create 0x18dda0:122 + the script's Entity params], the script's stream 0/1
    //    lifts it above the counter [D the keys; M Script_Interp 0x1d9a08:94-110], and stepping
    //    back clears the take's line of sight — the take writes ch 0x4e [M 0x18e650:330-333; L
    //    PCSX2 2026-10-22: the first back-step took it].
    //    The items test's repro spot faces the display case; walking there through the real path.
    w.walk_nav_to([5.90, 5.9, -42.41], "to the dragon safe", 0.35);
    {
        // Aim at the case (SP #110's body at (6.7, 6.1, −42.6) [D the SP dump]) with the real view
        // (turn/look inputs). The use is EDGE-triggered — the world-use block fires on the action
        // bits' press edge, `Input_Action(p, 0xe, 4)`, and a held E activates the world ONCE [M
        // 0x1a5460:395-403; the action machine 0x17ee88 latches per press] — so the view converges
        // FIRST with no use (the earlier hold-E loop spent its one edge while still facing away:
        // no spawn), then ONE press tick at the converged view: the proven doublecross_items_live
        // shape [L PCSX2 2026-10-22: (5.90,5.97,−42.41) yaw 1.710 pitch −0.458, one E press → the
        // card].
        let case = [6.7f32, 6.1, -42.6];
        for _ in 0..240u32 {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { break; }
            let bp = sim.bond.pos;
            let dx = case[0] - bp[0];
            let dz = case[2] - bp[2];
            let dy = case[1] - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(bp, case).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let settled = err.abs() < 0.02 && perr.abs() < 0.02;
            let input = nf_game::player::Input {
                turn: if settled { 0.0 } else { err.clamp(-0.08, 0.08) },
                look: if settled { 0.0 } else { perr.clamp(-0.08, 0.08) },
                ..Default::default()
            };
            drop(sim);
            w.tick(input, false); // converge the view only — the edge is NOT spent yet
            if settled { break; }
        }
        // The single press at the converged view (one tick = one edge, the repro's shape).
        w.tick(Default::default(), true);
        // The stream reaches the Entity command within a couple of seconds [M 0x1d8968's 0xF0 arm].
        let mut live = false;
        for _ in 0..120 {
            w.tick(Default::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.pickups.iter().any(|p| p.kind == 2 && !p.gone) { live = true; break; }
        }
        assert!(live, "the converged single E press spawned the key pickup [D script 0x06000563 s1]");
    }
    for _ in 0..130 { w.tick(Default::default(), false); } // the reveal plays; the card lifts
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let k = sim.drones.pickups.iter().find(|p| p.kind == 2 && !p.gone).expect("the keycard is live");
        assert!(k.m[3][1] > 6.04, "the stream lifted the keycard to its last key (y 6.05) [D 0x06000563 s0/s1], got y {:.3}", k.m[3][1]);
        assert_eq!(k.switch_channel, 0x4e, "the keycard's take channel is obj0's leaf [D params]");
    }
    // Standing at the counter the take's LOS is blocked (as in the game [L]); step back (−x, the
    // case faces +x) and the take writes ch 0x4e [M 0x18e650:330-333].
    {
        let back = nf_game::player::Input { forward: -1.0, ..Default::default() };
        let mut taken = false;
        for _ in 0..120 {
            w.tick(back, false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4e) { taken = true; break; }
        }
        assert!(taken, "backing off the counter, Bond takes the keycard and ch 0x4e goes on [L + M]");
    }
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[li][o0], 4, "obj0 COMPLETE through the real take [M 0x1827d8:78-85]");
        assert!(sim.drones.channels.on(0x52), "the disc's AND(0x4e,0x51) writes the key gate [D]");
        assert_eq!(sim.drones.mission.states[li][o2], 3, "obj2 announced by its enable channel [M 0x1827d8:55-67]");
        assert!(!sim.drones.channels.on(0x60) && !sim.drones.channels.on(0x61) && !sim.drones.channels.on(0x63),
            "no fail hook fired on the route [M 0x1827d8:98-117]");
        // The safe's alarm responders: guards #456/#462's enable channel IS 0x4e [D p3].
        for di in sim.drones.list.iter().filter(|d| d.team == 1 && d.switch_channel == 0x4e) {
            assert!(di.enabled, "guard with enable 0x4e responded to the key [D p3]");
        }
    }

    // 4. The exit: E on the Load Point loads 07000004 through the real reload path (the exit
    //    lawn first — the direct line rides the slope without settling).
    let sim = w.scene.sim.as_ref().unwrap();
    let exit = sim.drones.triggers.iter().find(|t| t.kind == walker::trigger_kind::LOAD_LEVEL).map(|t| t.centre()).expect("the Load Point");
    drop(sim);
    w.walk_nav_to([18.5, 5.9, 13.0], "the exit lawn", 1.6);
    w.walk_to(Wp { at: exit, act: Act::Use { use_ticks: 90 }, arrive: 1.6 }, "E the exit");
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.level_exit, Some(0x0700_0004), "the exit loads 07000004 [D param 0]");
    eprintln!("[03] full walk: {} ticks; falls: {:?}", w.total_ticks, w.report.falls);
    for f in &w.report.falls { eprintln!("[03] fall: {f}"); }
    for s in &w.report.stuck_events { eprintln!("[03] stuck: {s}"); }
}

/// THE FULL ROUTE ON REAL HEALTH (`damage_log.rs::with_real_damage`): the same walked route,
/// the top-up off — no health, armour or ammo writes anywhere — and Bond's death panics with
/// the damage log (who, weapon, range). Auto-aim is the app default; the fights are the
/// walker's own plain return fire (the same fights every other real-damage walk is pinned
/// against).
#[test]
fn walk_the_full_route_and_complete_every_objective_real_damage() {
    walker::with_real_damage(walk_the_full_route_and_complete_every_objective);
}

/// Sister of the full walk's key gate: one objective's channel alone never opens the gate —
/// the AND needs both inputs [D the kind-3 out-0x52 list 4e,51].
#[test]
fn the_key_gate_stays_shut_without_both_objectives() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    // obj0's channel only: the AND(0x4e,0x51) holds 0x52 off [D].
    sim.drones.channels.set(0x4e, 1, sim.drones.frame);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(!sim.drones.channels.on(0x52), "0x4e alone never opens the gate [D AND list 4e,51]");
    assert!(!sim.drones.channels.on(0x51), "0x51's own AND(0x4f,0x50) is still missing an input [D]");
    // obj1's sub-channels alone: same.
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("reload");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.channels.set(0x4f, 1, sim.drones.frame);
    sim.drones.channels.set(0x50, 1, sim.drones.frame);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.channels.on(0x51), "AND(0x4f,0x50) completes obj1's channel [D]");
    assert!(!sim.drones.channels.on(0x52), "0x51 alone never opens the gate [D AND list 4e,51]");
}

/// The servants are protected: killing one raises ch 0x60 and the standing fail hook fails the
/// mission with "Killed Civilians" [M Mission_MonitorObjectives 0x1827d8:98-104; the death channel
/// is the placement's own [D the dump]].
#[test]
fn killing_a_servant_fails_the_mission() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    let si = sim.drones.list.iter().position(|d| d.skin & 0xffff == 0x2b && d.team == 3).expect("a servant");
    let at = sim.drones.list[si].pos;
    drop(sim);
    w.walk_nav_to([at[0], at[1] - 0.3, at[2]], "to the servant", 2.0);
    w.shoot(si, "the servant");
    for _ in 0..120 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.list[si].corpse, "the servant died to real fire");
    assert!(sim.drones.channels.on(0x60), "the placement's death channel 0x60 rose [D]");
    // The mission layer's own records (the events vec is drained per step by the view path — the
    // persistent globals are the evidence): the standing hook's label, the fail state, ch 99.
    assert_eq!(sim.drones.mission.fail_label, 0x0400_0034, "the Killed Civilians fail label [M 0x1827d8:98-104]");
    assert_eq!(sim.drones.mission.state, 2, "MissionState 2 (the fail arm) [M 0x182c60:45-57]");
    assert!(sim.drones.channels.on(99), "the mission-fail channel rose [M mission.rs ch.set(99)]");
}

/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty): the same walked
/// route at the menu's Agent pick — GameState+0x28 = 2 through the front-end's launch path [M
/// C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 1 carries 2; Txt 0x14e =
/// "Agent"]. On this level group the pick scales Bond's pain by Plr_DMod_Normal 0.7 instead of
/// Easy 0.5 and the guards' hit chance by DroneFiring_Accuracy_Normal 0.8 instead of Easy 0.5
/// [M Player_HandlePain 0x1902c8; DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 01-04
/// group]; the min-difficulty placements pass Drone_Create's gate [M 0x136be0:49]. Real input
/// only — the load path is the only difference.
#[test]
fn walk_the_full_route_and_complete_every_objective_real_damage_agent() {
    walker::with_real_damage(|| walk_the_full_route_and_complete_every_objective_at(2));
}

/// 00 Agent-difficulty sister of `..._real_damage_agent` (real-health-00agent-02-03-04): the
/// same walked route at the menu's 00 Agent pick — GameState+0x28 = 3 through the front end's
/// launch path [M C_SBNFDFCTY 0x216ca8:15; the `difficulty[]` table 0x2df4c0 row 2 carries 3;
/// Txt 0x14f = "00 Agent"]. On this level group the pick scales Bond's pain by Plr_DMod_Hard
/// 1.2 instead of Agent's Plr_DMod_Normal 0.7, the guards' hit chance rides
/// DroneFiring_Accuracy_Hard 1.0 instead of 0.7 and their rounds by DroneDamage_Hard 0.8; the
/// autoaim bit closes at 00 Agent (Autoaim_HardMul 0) [M Player_HandlePain 0x1902c8;
/// DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs 01-04 group]; the min-difficulty
/// placements pass Drone_Create's gate [M 0x136be0:49]. Real input only — the load path is the
/// only difference.
#[test]
#[ignore = "00 Agent: the east captor-room cluster under the door routes — the pre-clear rally kills the door-8 guards (#18/#19/#2) at zero fight cost but the rally APPROACH crosses their gun line (40.8 -> 15.6-19.2) and the exit lawn then needs ~29; next levers: an approach that stays north of door 8 (the green run fought from (36.2,5.7) THROUGH the open doorway), the 87000-tick (35.6,8.7) door stall on the post-rescue west-lawn leg, and the exit-lawn #16 post-kill volley; follow-up real-health-00agent-02-03-3"]
fn walk_the_full_route_and_complete_every_objective_real_damage_00agent() {
    walker::with_real_damage(|| walk_the_full_route_and_complete_every_objective_at(3));
}

/// The door-8 cluster's LOAD-state probe (gas-rng-draws-6; run with `--ignored --nocapture`):
/// report-5's named first step — the staged pre-servant-nav mouth-clear's anchor
/// (`dxz < 6.0` of (38.7, 6.2, 0.2)) never fired because the cluster (#18/#19/#2) loads
/// DISABLED behind the leaves at its OWN spots and only walks to (38.7, 0.2) after the door
/// opens. This probe dumps every hostile's load position/state at tick 100 (before any leg)
/// so the filter can be re-anchored to the real spots. Cheap (no walking).
#[test]
#[ignore = "probe: dump the door-8 cluster's load state at tick 100"]
fn probe_door8_cluster_load_state() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.wait(100);
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        if d.team != 1 { continue; }
        eprintln!("[03-probe] #{i} skin {:#x} at ({:.2},{:.2},{:.2}) enabled {} present {} corpse {} state {:#x} hp {:.1} weapon {:#x}",
            d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.present, d.corpse, d.state, d.health, d.weapon);
    }
    let door8 = sim.drones.doors.list.iter().enumerate()
        .find(|(_, dr)| walker::dxz(dr.pos, [35.6f32, 2.1, 2.7]) < 4.0);
    if let Some((di, dr)) = door8 {
        eprintln!("[03-probe] door #{di} at ({:.2},{:.2},{:.2}) state {:#x} pos01 {:.2}",
            dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.pos01);
    } else {
        eprintln!("[03-probe] no door within 4 of (35.6, 2.1, 2.7) — dump all doors:");
        for (di, dr) in sim.drones.doors.list.iter().enumerate() {
            eprintln!("[03-probe] door #{di} at ({:.2},{:.2},{:.2}) state {:#x} pos01 {:.2}",
                dr.pos[0], dr.pos[1], dr.pos[2], dr.state, dr.pos01);
        }
    }
}
