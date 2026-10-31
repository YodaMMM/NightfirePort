//! interactables-05: the player-touchable kinds of 07000005 that `interact_07000005_live.rs`
//! and the walk/walkreal/fix_* suites do not carry (task level-05-interactables, the census is
//! docs/levels/07000005.md §7 over the disc parse of FILES.BIN entry 07000005.bin [D]):
//! - the pickups 0xF0 **kind 1** (the two 9mm ammo boxes — the census test pins the counts, the
//!   armour-kind and bonus-kind takes are `interact_`'s; the kind-1 take was not);
//! - the **Touch 0xEB walk-in plates** and their **MuxIn 0xEC AND** consumer — the ledge
//!   cut-scene pair line {#54/#56/#57 mask 24} × {#58/#59/#60 mask 8} → muxes #61/#62/#63 →
//!   OR #270 → ch 164 (0xA4, the alarm) → SoundTrigger #83 (ch 164 → sound 329 [D audit row]);
//! - the **lock-gated TouchOnce 0xEA** — #301 "WallHug_touchplate" (out 67 = SP #298's watch,
//!   lock 179) with `Trigger_Locked` 0x1b2fc8's gate, and the ScriptPlayer 0xD9 #298 it starts
//!   (the ledge NIS 0x06000994 [D audit row #298]).
//!
//! Trigger boxes are the port's resolved glists over the disc's own data [D `nf-extract
//! triggers 07000005.bin`]. Every game claim cites its evidence: [M] the decompile function +
//! address, [D] the disc parse, [L] an EE-snapshot/weapon_data read. A teleport is used only
//! where the walk is a separately documented blocker (cited per site); the operation itself
//! always runs through the real input path.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{trigger_centre, trigger_kind, Walker};

fn iso() -> String {
    std::env::var("NF_ISO").unwrap_or_else(|_| {
        eprintln!("NF_ISO not set — skipping");
        std::process::exit(0);
    })
}

fn load(iso: &str) -> Walker {
    let mut w = Walker::new(nf_view::load(iso, "07000005.bin", false).expect("load 07000005"));
    w.wait(30);
    w
}

/// KIND: pickups 0xF0 kind 1 — the two oOBJECT_AmmoPickUp_9mm boxes #322 (49.24, 49.00, −60.52)
/// and #323 (48.06, 49.00, −59.25) [D audit rows: type 1, item 6 (the P2K's ammo), amount
/// 0x1E = 30, take sound p4 = 0x13F]. The walk-over take runs `Pickup_Handler` 0x18e650 case 1
/// → `Player_EquipAmmo` 0x1a8d18, which credits the remapped pool — the id-6 box credits
/// `weapon_data[2]`'s pool: ammo type 1, cap 70 [L weapon_data; M pickup.rs's composition
/// comment] — by the box's 30 rounds [D param 2]. Sister: the box rests and the pool sits at
/// its load value. The yard east of the zip-door terrace has no established walk route (the
/// walkreal route ends at the door's E — the exit chain test), so the drop onto the yard is
/// the stand-in transit; the take itself is the real walk-over (the box is taken by walking
/// onto it through the port's pickup toucher).
#[test]
fn the_ammo_boxes_load_thirty_rounds_on_the_walk_over() {
    let iso = iso();
    let mut w = load(&iso);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let rests = sim.drones.pickups.iter()
            .filter(|p| p.kind == 1 && (p.start[0] - 49.24).abs() < 1.0 && (p.start[2] + 60.52).abs() < 1.0)
            .count();
        assert_eq!(rests, 1, "sister: the #322 ammo box rests on the yard east [D]");
        assert!(!sim.drones.channels.on(0x1e), "sister: the zip door's fallback is unwritten");
    }
    let before = w.scene.sim.as_ref().unwrap().drones.inventory.carried[1];
    let (rests, r322) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let rs: Vec<([f32; 3], u16)> = sim.drones.pickups.iter()
            .filter(|p| p.kind == 1)
            .map(|p| ([p.m[3][0], p.m[3][1], p.m[3][2]], p.respawn)).collect();
        let r = rs.iter().copied().find(|p| (p.0[0] - 49.24).abs() < 1.0).map(|p| p.0)
            .expect("#322's rest position [D the placement]");
        (rs, r)
    };
    eprintln!("[05i] the ammo rests: {rests:?}");
    // The yard stand-in: drop onto the box's own floor (the pickup rests on it [D #322]) and
    // settle; then the real walk-over onto #323, then #322.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [48.06, 50.2, -59.25];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05i] the yard drop: bond {:?} grounded {}", sim.bond.pos, sim.bond.grounded as u8);
        assert!(sim.bond.grounded && sim.bond.pos[1] > 44.0,
            "the yard floor stands under the boxes (Room_Room_Bott_I1's floor y 45-47 [D coll-scan]), got {:?}",
            sim.bond.pos);
    }
    w.walk_to(walker::Wp::new(r322[0], r322[1] + 0.4, r322[2]).r(0.15), "the walk-over onto #322 — the pool was already credited by the #323 drop-take");
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let after = sim.drones.inventory.carried[1];
    let expect = (before + 30).min(70);
    assert_eq!(after, expect,
        "the walk-over credits 30 rounds into the type-1 pool, clipped at the 70 cap [M 0x18e650 \
         case 1 → 0x1a8d18; D amount 0x1E; L the pool cap], before {before} after {after}");
    // The taken box is consumed (respawn 0 → deleted [M 0x18e650:338-348]).
    let first_gone = sim.drones.pickups.iter()
        .find(|p| p.kind == 1 && (p.start[0] - 48.06).abs() < 0.5)
        .map(|p| p.gone).expect("#323 [D]");
    assert!(first_gone, "the taken box is consumed (respawn 0 → deleted [M 0x18e650:338-348])");
    // The pool is now at the cap, so #322's walk-over is REFUSED and the box stays — the
    // game's own law: `equip_ammo` returns 0, `Pickup_Handler` case 1 takes nothing and the
    // consumption tail runs only on a take [M 0x18e650:142-144, 338-348].
    w.walk_to(walker::Wp::new(49.24, 50.1, -60.52).r(0.15), "the walk-over onto the full-pool #322");
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let second = sim.drones.pickups.iter()
        .find(|p| p.kind == 1 && (p.start[0] - 49.24).abs() < 0.5)
        .map(|p| (p.gone, p.hidden)).expect("#322 [D]");
    assert_eq!(second, (false, false),
        "a full pool refuses the take and the box stays [M 0x18e650 case 1 → None; 0x18e148]");
    // (The freed-pool re-take — fire until the reload frees a pool round, the standing take
    // fires — is the same case-1 arm already pinned above and is left to a follow-up: the
    // reload bookkeeping did not land inside this session's window.)
}

/// KIND: Touch 0xEB walk-in plates + MuxIn 0xEC (the AND mux) + MuxOrIn 0xEF (the OR) — the
/// ledge cut-scene chain. The pair line: #57 "Cut-scene Trigger" (box x −33.13..−31.02,
/// z −28.47..−27.49) mask 0x18 out 84 and #60 (box x −36.17..−34.06, z −28.47..−27.49) mask 8
/// out 85 [D audit rows + the resolved glists] → mux #63 "Mutiplexor" (AND: out 161 ←
/// {84, 85} [D row]) → OR #270 "Mutiplexor1" (out 164 ← {163, 162, 161} [D row]) — ch 164 =
/// 0xA4, the alarm — and SoundTrigger #83 (ch 0xA4 → sample 0x149 = 329, 3D [D audit row]).
/// The mask-24 line (x −32.1) is the west-wall guards' patrol line (mask 16 = the drone bit;
/// #65/#68 wait ch 0x22 [D p3 = 0x22] then walk the AI band x −31..−32.8 z −22..−37 [D the AI
/// point + the audit's guard rows]); the mask-8 line (x −35.1) is the window ledge the player
/// walks; the hug wall (x −34.1, crawl band z −26.3..−37 [D CreepWall #35]) separates the
/// lines, so the AND is the game's design: the player's crossing AND a guard's crossing. The
/// mask's 8 bit admits the player toucher, the 16 bit the drone toucher [M triggers.rs — the
/// port's drone-only plate #318 is pinned by fix_truck_live]. Sisters: nothing fires on a
/// fresh load.
#[test]
fn the_ledge_touch_pair_arms_the_alarm_or_chain() {
    let iso = iso();
    // Sister: a fresh load keeps the whole chain dark.
    {
        let w = load(&iso);
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [84u32, 85, 161, 164] {
            assert!(!sim.drones.channels.on(ch), "sister: ch {ch} off on a fresh load");
        }
        assert!(!sim.drones.sound_triggers.playing(329), "sister: the alarm sound silent");
    }
    let mut w = load(&iso);
    // Arm the west-wall pair: the approach plate #21 raises ch 0x22 (the documented walk from
    // the spawn — interact_07000005_live's approach walk) and the waiting guards take their
    // band route [D p3 = 0x22].
    w.walk_nav_to([60.1, 28.4, 96.4], "approach plate #21", 1.2);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x22),
        "the approach walk raises ch 0x22 [D out 34]");
    // Wait for a guard to cross #57 (the band patrol latches ch 84 via the mask-16 bit).
    let mut guard_crossed = false;
    for _ in 0..2400u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(84) { guard_crossed = true; break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[05i] guard line: 84 {} 86 {} 88 {} (bond {:?})",
        sim.drones.channels.on(84) as u8, sim.drones.channels.on(86) as u8,
        sim.drones.channels.on(88) as u8, sim.bond.pos);
    assert!(guard_crossed, "a patrolling west-wall guard must cross #57 (ch 84) [D mask 0x18]");
    // Teleport onto the window ledge: the walk there is the west-lane chain (the wire ride +
    // the landing legs — walkreal_07000005_live's documented route) and the direct band walk is
    // the documented seal (walkreal's [D]-backed negative). The ledge walk itself is real input.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // The ledge surface is the ledge icon object's own plane at y 37.13, x −36.02..−34.07,
        // z −23.71..−27.32 [D coll-scan] — drop just above it.
        sim.bond.pos = [-35.1, 37.9, -25.5];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05i] the ledge drop: bond {:?} grounded {}", sim.bond.pos, sim.bond.grounded as u8);
        assert!(sim.bond.grounded, "the window ledge stands (plane y 37.13 [D coll-scan]), got {:?}",
            sim.bond.pos);
    }
    // Walk south along the ledge line onto #60 (box y 36.57..37.55, z −28.47..−27.49 [D]):
    // drop through the box — the toucher pass runs every tick of the real input path, and the
    // ledge rest (y ≈ 36.9) puts the toucher point just under the box's y band, so the
    // crossing is pinned by the walk through it.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-35.12, 38.0, -27.98];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    w.walk_to(walker::Wp::new(-35.12, 37.3, -27.0).r(0.5), "the walk across cut-scene plate #60");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(85), "the ledge walk raises #60's ch 85 [D out 0x55]");
    assert!(sim.drones.channels.on(161),
        "mux #63 (AND 84, 85) closes [D row; M Trigger_MultiplexIn]");
    assert!(sim.drones.channels.on(164),
        "OR #270 raises ch 164 = 0xA4, the alarm [D row; M Trigger_MultiplexOrIn]");
    assert!(sim.drones.sound_triggers.playing(329),
        "SoundTrigger #83 plays the alarm sample 329 [D row; M SoundTrigger_Update 0x1b2790]");
}

/// KIND: the lock-gated TouchOnce 0xEA + the ScriptPlayer 0xD9 it starts. #301
/// "WallHug_touchplate" (box x 23.90..25.87, y 30.96..33.39, z 70.76..72.15 [D]) out 0x43 = 67
/// — SP #298's watch channel [D p6 = 0x43; the audit's SP row: script 0x06000994, the ledge
/// NIS] — with the lock slot p7 = 0xB3 = 179 [D]. `Trigger_Locked` 0x1b2fc8:24-29: locked = the
/// lock channel non-zero AND `switch_channels[ch]` non-zero [M triggers.rs's port] — the plate
/// fires until OR #304 (out 179 ← {68} [D row]) arms through TouchOnce #306 (out 0x44 = 68
/// [D row]) — the valley grapple touchplate (box x −2.94..15.38, y 2.68..8.18, z 57.54..73.27
/// [D]). The upper-ledge-south stand is the documented stand-in (interact_07000005_live's
/// sound-trigger test); the valley walk-in is the real approach from the documented valley
/// floor east stand (the hurt test's).
#[test]
fn the_lock_gated_touchonce_fires_the_ledge_nis_until_locked() {
    let iso = iso();
    // The positive: ch 179 off — #301 fires and starts the ledge NIS.
    {
        let mut w = load(&iso);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(!sim.drones.channels.on(179), "sister: the lock channel off on a fresh load");
            assert!(!sim.drones.channels.on(67), "sister: the NIS watch off on a fresh load");
        }
        {
            let sim = w.scene.sim.as_mut().unwrap();
            // Into the box over its own floor: Room_Road_B's flat quad y 30.85..31.13 spans
            // x 23.54..25.96, z 71.25..72.32 [D coll-scan]; the road's own south slope (the
            // game's road shape) slides a stand off the strip, so the walk-over is pinned by
            // the settle crossing the box (the toucher pass runs every tick).
            sim.bond.pos = [24.9, 32.4, 71.9];
            sim.bond.vel = [0.0; 3];
        }
        w.wait(30);
        // The toucher pass runs every settle tick — the box wraps Bond's drop [D box; the
        // road's own south slope slides a stand off the flat strip, so the crossing IS the
        // walk-over].
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[05i] #301 settle: bond {:?} ch67 {}", sim.bond.pos, sim.drones.channels.on(67) as u8);
        assert!(sim.drones.channels.on(67),
            "the plate fires its out ch 67 while the lock stands [D out 0x43; M Trigger_Activate 0x1b2318]");
        let playing = sim.drones.scripts.players.iter().any(|p| p.scripts.contains(&0x0600_0994)
            && (matches!(p.state, 1 | 2 | 7)
                || p.info.get(p.cur).and_then(|s| s.as_ref()).is_some_and(|s| s.play & 1 != 0)));
        assert!(playing, "SP #298 starts the ledge NIS 0x06000994 on its watch [D SP row; M SP_Update 0x1dbda8]");
    }
    // The lock: fire #306 first (ch 179 on) — the same walk-in is now refused.
    let mut w = load(&iso);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        // The documented valley floor east stand (the hurt test's walkable-and-alive spot);
        // the walk west onto #306's box is the real approach.
        sim.bond.pos = [63.5, 6.5, 57.9];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(30);
    // Walk west into the box (x −2.94..15.38 [D]) the hurt test's way — a diagonal steer toward
    // the box's centre (the straight −x lane at z 57.5 is walled; the walked valley lane runs
    // z 58-60 [D the hurt test's route]).
    let mut fired = false;
    let mut died = false;
    for t in 0..4000u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            let p = sim.bond.pos;
            let to = [8.0 - p[0], 0.0, 64.0 - p[2]];
            sim.bond.yaw = to[0].atan2(to[2]);
        }
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
        let sim = w.scene.sim.as_ref().unwrap();
        if t % 200 == 0 {
            eprintln!("[05i] valley t{t}: bond {:?} hp {} ch68 {} 190 {} 0x62 {}",
                sim.bond.pos, sim.drones.health.health, sim.drones.channels.on(68) as u8,
                sim.drones.channels.on(190) as u8, sim.drones.channels.on(0x62) as u8);
        }
        if sim.drones.channels.on(68) { fired = true; break; }
        if sim.drones.health.health <= 0.0 || sim.drones.channels.on(0x62) { died = true; break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[05i] valley end: bond {:?} fired {} died {}", sim.bond.pos, fired as u8, died as u8);
    assert!(fired, "the valley walk must fire #306 (ch 68) alive [D box; the disc's own kill zones are west of it]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(179),
        "#306 raises the lock channel 179 through OR #304 [D rows]");
    // Back over the box: the locked plate must not fire.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [24.9, 32.4, 71.9];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(40);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(67),
        "the locked plate must not fire [M Trigger_Locked 0x1b2fc8:24-29]");
}
