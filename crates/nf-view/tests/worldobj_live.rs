//! The world objects (fix-world-objects): the placement kinds the port drew as static geometry,
//! now created and updated through the real load path (`nf_view::load` → `Sim`).
//! Fail-first: every assertion reads `sim.drones.worldobjs`, which did not exist before the port
//! (the placements were skipped by every creator — levels.md §7's "unhandled placement types");
//! on the pre-fix tree these tests do not compile, and the behaviours (a shooter bullet, a channel
//! forced low, a prop deleted on its channel) had no ported writer at all.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str, archive: &str) -> nf_view::scene::Scene {
    nf_view::load(iso, archive, false).expect("load")
}

/// Shooter_Create 0x17d030 / Shooter_Update 0x17d0c0: 0700004a's two ceiling guns (weapon 0x3d,
/// whose weapon_data class halfword +0x02 is 0x3b) fire one straight-down bullet each on their arm
/// channel (0x22 / 0x4f) and delete themselves (obj+0xFE bit 0 → the delete pass 0x1336f8:86-87).
#[test]
fn shooter_0700004a_fires_straight_down_on_the_arm_edge() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "0700004a.bin");
    let sim = scene.sim.as_mut().expect("sim");
    assert_eq!(sim.drones.worldobjs.shooters.len(), 2, "the two space-laser shooters exist");
    let (pos0, pos1) = (sim.drones.worldobjs.shooters[0].pos, sim.drones.worldobjs.shooters[1].pos);
    assert_eq!(sim.drones.worldobjs.shooters[0].arm, 0x22);
    assert_eq!(sim.drones.worldobjs.shooters[1].arm, 0x4f);
    // Sister first: without the arm channel, nothing fires.
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    assert!(sim.drones.bullets.is_empty(), "no bullet before the arm channel");
    assert!(!sim.drones.worldobjs.shooters[0].fired && !sim.drones.worldobjs.shooters[1].fired);
    // Arm shooter 0's channel: one bullet, straight down (the 0x3b class special case, 0x17d0c0:20-24),
    // from the gun's position; the shooter marks itself dead and never fires again.
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x22, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    assert!(sim.drones.worldobjs.shooters[0].fired, "shooter 0 fired on its arm edge");
    assert!(!sim.drones.worldobjs.shooters[1].fired, "shooter 1's channel is still off");
    let b = sim.drones.bullets.iter().find(|b| b.weapon == 0x3d).expect("the shooter's bullet");
    // The bullet flies on its spawn tick (state 0), so y is already below the muzzle; the straight-down
    // direction keeps x/z at the gun's (the spread radius is 0.0014·s — sub-unit drift).
    assert!((b.pos[0] - pos0[0]).abs() < 0.5 && (b.pos[2] - pos0[2]).abs() < 0.5,
        "the bullet stays on the gun's column, got {:?} vs {:?}", b.pos, pos0);
    assert!(b.pos[1] < pos0[1], "the bullet is below the muzzle ({:?} from {:?})", b.pos, pos0);
    assert!(b.dir[1] < -0.99, "the 0x3b-class weapon fires straight down, got {:?}", b.dir);
    assert!((b.damage - 2000.0).abs() < 1.0, "the weapon record's damage +0x0C (2000)");
    // One shot per placement: the game's delete pass removed the object after the fire tick.
    for _ in 0..5 { sim.step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().expect("sim");
    assert!(!sim.drones.worldobjs.shooters.iter().any(|s| s.arm == 0x22 && !s.fired), "the fired shooter is gone (one-shot)");
    assert_eq!(sim.drones.bullets.iter().filter(|b| b.weapon == 0x3d).count(), 1, "exactly one bullet, no repeats");
    let _ = pos1;
}

/// InverterTrigger_Create 0x1b3040 / Update 0x1b30a8: 07000006's inverter (in 0xb1 → out 0x37)
/// holds its out channel low while the in channel is set.
#[test]
fn inverter_07000006_forces_the_out_channel_low() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000006.bin");
    let sim = scene.sim.as_mut().expect("sim");
    let inv = sim.drones.worldobjs.inverters.first().expect("07000006's inverter").clone();
    assert_eq!((inv.ch_in, inv.ch_out), (0xb1, 0x37), "p0 → in, p1 → out");
    // Sister: with the in channel off, the out channel survives.
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x37, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    assert!(sim.drones.channels.on(0x37), "ch 0x37 stays set while ch 0xb1 is off");
    // The inverter: in on → out forced to 0 the same tick.
    let frame = sim.drones.frame;
    sim.drones.channels.set(0xb1, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    assert!(!sim.drones.channels.on(0x37), "InverterTrigger_Update 0x1b30a8 clears ch 0x37");
    assert!(sim.drones.channels.on(0xb1), "the in channel itself is untouched");
}

/// DynamicObject_Create 0x18dbf8 / Update 0x18d3b8 (mode 0): 07000002's gated prop deletes itself
/// when its action channel (p1 = 1) fires; 07000014's channel-paired props (chB 0x51 / chA 0x51)
/// gate and delete on the same channel from opposite sides.
#[test]
fn dynobj_mode0_deletes_on_its_action_channel() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000002.bin");
    let sim = scene.sim.as_mut().expect("sim");
    assert_eq!(sim.drones.worldobjs.dynobjs.len(), 1, "07000002's DynamicObject exists");
    assert_eq!(sim.drones.worldobjs.dynobjs[0].ch_b, 1, "its action channel is p1 = 1");
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().expect("sim");
    assert_eq!(sim.drones.worldobjs.dynobjs.len(), 1, "still present while ch 1 is off");
    let frame = sim.drones.frame;
    sim.drones.channels.set(1, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    assert!(sim.drones.worldobjs.dynobjs.is_empty(), "mode 0 + armed ch 1 → deleted (obj+0xFE bit 0)");

    // 07000014's pairs: two props delete on ch 0x51 (chB), two more are gated by it (chA).
    let mut scene = load(&iso, "07000014.bin");
    let sim = scene.sim.as_mut().expect("sim");
    let with_b = sim.drones.worldobjs.dynobjs.iter().filter(|o| o.ch_b == 0x51).count();
    let with_a = sim.drones.worldobjs.dynobjs.iter().filter(|o| o.ch_a == 0x51).count();
    assert_eq!(with_b, 2, "the two chB-0x51 deleters");
    assert_eq!(with_a, 2, "the two chA-0x51 gated props");
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().expect("sim");
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x51, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    assert_eq!(sim.drones.worldobjs.dynobjs.iter().filter(|o| o.ch_b == 0x51).count(), 0, "the chB pair deleted");
    assert_eq!(sim.drones.worldobjs.dynobjs.iter().filter(|o| o.ch_a == 0x51).count(), 2, "the chA pair survives (armed gate runs)");
}

/// DynamicObject_Update 0x18d3b8's distance fade (p2 = the radius): 0700000c's two no-channel
/// props (p2 = 9) write obj+0x106 from 255 at the radius down to 0 at half of it. The viewer here
/// is the player's eye (glb_viewer+0xC0's headless stand-in).
#[test]
fn dynobj_0700000c_distance_fade() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "0700000c.bin");
    let sim = scene.sim.as_mut().expect("sim");
    let fades: Vec<_> = sim.drones.worldobjs.dynobjs.iter().filter(|o| o.dist == 9).collect();
    assert_eq!(fades.len(), 2, "0700000c's two distance-fade props");
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().expect("sim");
    let fade = sim.drones.worldobjs.dynobjs.iter().find(|o| o.dist == 9).expect("still there").clone();
    // Far from it (outside 9 units): untouched 0xFF.
    let far = sim.drones.worldobjs.dynobjs.iter().find(|o| o.dist == 9).unwrap().pos;
    sim.bond.pos = [far[0] + 20.0, far[1], far[2]];
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    let after_far = sim.drones.worldobjs.dynobjs.iter().find(|o| o.dist == 9).unwrap().alpha;
    assert_eq!(after_far, 0xff, "outside the radius the alpha byte is untouched");
    // Inside (d ≈ 6: f = 6/9 − 0.5 = 1/6 → alpha ≈ 85).
    sim.bond.pos = [far[0] + 6.0, far[1], far[2]];
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_mut().expect("sim");
    let after_near = sim.drones.worldobjs.dynobjs.iter().find(|o| o.dist == 9).unwrap().alpha;
    assert!(after_near < after_far && after_near < 200, "closer → lower alpha (fade-out), got {after_near} then {after_far}");
    let _ = fade;
}

/// ThirdCam_Create 0x1bc510 / GetPos 0x1bc590: 07000005 registers four third-person cameras
/// (the wire crossing's among them), each with its 0x19 key-list spline; the spline rides.
#[test]
fn thirdcam_07000005_registers_the_camera_splines() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000005.bin");
    let sim = scene.sim.as_mut().expect("sim");
    let cams: Vec<u16> = sim.drones.worldobjs.thirdcams.iter().map(|c| c.cam_id).collect();
    assert_eq!(cams.len(), 4, "07000005's four ThirdCam placements");
    assert!(cams.contains(&1) && cams.contains(&2) && cams.contains(&3) && cams.contains(&5), "ids {cams:?}");
    for c in &sim.drones.worldobjs.thirdcams {
        assert_eq!(c.flag, 1, "p1 = 1 on every 07000005 camera");
        assert_eq!(c.keys.len(), c.key_count as usize, "the key list matches the level_tag +0x02 count");
        let a = c.get_pos(0.0).expect("spline start");
        let b = c.get_pos(1.0).expect("spline end");
        let d = ((a[3][0] - b[3][0]).powi(2) + (a[3][1] - b[3][1]).powi(2) + (a[3][2] - b[3][2]).powi(2)).sqrt();
        assert!(d > 0.01, "the camera spline moves (translation {:.2})", d);
    }
}

/// 07000016's five DynamicObjects (the audit's kind 0x3b): two complementary display pairs swap on
/// their lock channels, and the lift-shaft blockers unhide on the pit-squad death channel.
/// `DynamicObject_Create` 0x18dbf8: obj+0xF0 = the glist's +0x44 word, |0x30 when p0 != 0 —
/// bit 0x10 = not drawn [M docs/atlas/object.md +0xF0; `View_AddObjects` 0x1e62d8 skips & 0x8010].
/// `DynamicObject_Update` 0x18d3b8: watch A (p0) ON → restore the stored word (0x30 cleared =
/// drawn); watch B (p1), mode 0: ON → obj+0xFE bit 0 (the delete pass removes it — the port's
/// `dead`), OFF → the distance fade. Placements [D census]: #682 Screen_door_unlocked (p0 = 0x24),
/// #683 Screen_door_locked (p1 = 0x24), #503 Door2_switch (p1 = 0x94), #504 Door2_switch_locked
/// (p0 = 0x94), #532 Blocking_polys_lift_shaft1 (p0 = 0xcc).
#[test]
fn dynobj_07000016_display_pairs_swap_on_their_lock_channels() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = load(&iso, "07000016.bin");
    {
        let sim = scene.sim.as_ref().expect("sim");
        assert_eq!(sim.drones.worldobjs.dynobjs.len(), 5, "the five 0x3b placements");
        // The create states [M 0x18dbf8:31]: watch-A objects hidden (0x30 set), watch-B drawn.
        for ch in [0x24u16, 0x94, 0xcc] {
            let a = sim.drones.worldobjs.dynobjs.iter().find(|o| o.ch_a == ch).expect("watch-A object");
            assert_eq!(a.flags & 0x30, 0x30, "watch-A ch {ch:#x} hidden at create");
        }
        for ch in [0x24u16, 0x94] {
            let b = sim.drones.worldobjs.dynobjs.iter().find(|o| o.ch_b == ch).expect("watch-B object");
            assert_eq!(b.flags & 0x10, 0, "watch-B ch {ch:#x} drawn at create");
        }
    }
    for _ in 0..10 { scene.sim.as_mut().expect("sim").step_bond(Default::default(), false, false, false); }
    // door[0]'s SecurityStation rises ch 0x24: the unlocked screen appears, the locked one is deleted.
    let mut sim = scene.sim.as_mut().expect("sim");
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x24, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_ref().expect("sim");
    let a24 = sim.drones.worldobjs.dynobjs.iter().find(|o| o.ch_a == 0x24).expect("watch-A 0x24");
    assert_eq!(a24.flags, a24.base_flags, "watch-A 0x24 restored (drawn)");
    assert!(sim.drones.worldobjs.dynobjs.iter().all(|o| o.ch_b != 0x24), "watch-B 0x24 deleted (obj+0xFE bit 0)");
    // The tower pair on 0x94 (no disc writer — but the channel arm must hold when it does rise).
    let mut sim = scene.sim.as_mut().expect("sim");
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x94, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_ref().expect("sim");
    let a94 = sim.drones.worldobjs.dynobjs.iter().find(|o| o.ch_a == 0x94).expect("watch-A 0x94");
    assert_eq!(a94.flags, a94.base_flags, "watch-A 0x94 restored (drawn)");
    assert!(sim.drones.worldobjs.dynobjs.iter().all(|o| o.ch_b != 0x94), "watch-B 0x94 deleted");
    // The lift-shaft blockers: drone #698's death channel 0xcc unhides them.
    let mut sim = scene.sim.as_mut().expect("sim");
    let frame = sim.drones.frame;
    sim.drones.channels.set(0xcc, 1, frame);
    sim.step_bond(Default::default(), false, false, false);
    let sim = scene.sim.as_ref().expect("sim");
    let acc = sim.drones.worldobjs.dynobjs.iter().find(|o| o.ch_a == 0xcc).expect("watch-A 0xcc");
    assert_eq!(acc.flags, acc.base_flags, "the blockers restored (drawn) on 0xcc");
}
