//! control-funcs-collide — `ThirdIcon_Update` 0x196e80's kill-channel branch (the collide slot of
//! class 0x42; update slot NULL): `switch_channels[inst+0x06] != 0` → `obj+0xFE |= 1` and the
//! delete pass removes the icon [M 0x196e80:21-27] — a consumed icon never arms `BLData+0x95F`
//! again. Red on the old code: the port's icon arm loop read only the on-channel, so 07000041's
//! killed ctx-2 icon kept arming the HUD context forever.
//!
//! The disc places exactly ONE killable icon: 07000041 map 99's ctx-2 icon at (8.59, 0, −29.47),
//! on-channel 1, kill channel 0x6e (110) [D the 0xFA census]; the real writer is the TouchOnce
//! trigger (0xEA) at (15.60, 0, −33.22) whose out channel is 110 [D the same map's rows] — the
//! test writes that channel through the port's `Channels` (the exact `switch_channels` write the
//! trigger's fire lands in [M 0x1b1a78's Out copy → the port's Channels::set]) and asserts the
//! icon stops arming. Opt-in via `NF_ISO`.

use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_kill_channel_consumes_the_icon() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000041.bin", false).expect("load 07000041");
    {
        let sim = scene.sim.as_ref().unwrap();
        let ik = sim.icons.iter().position(|i| i.kill_channel != 0)
            .expect("07000041 carries the one killable icon [D census]");
        assert_eq!(sim.icons[ik].ctx, 2, "the killable icon is the ctx-2 HUD context [D param 0]");
        assert_eq!(sim.icons[ik].kill_channel, 0x6e, "kill channel 0x6e [D param 2]");
        eprintln!("[kill] icon {ik} centre {:?} r {}", sim.icons[ik].centre, sim.icons[ik].radius);
    }
    let sim = scene.sim.as_mut().unwrap();
    let ik = sim.icons.iter().position(|i| i.kill_channel != 0).unwrap();
    let centre = sim.icons[ik].centre;
    // Arm the on-channel the way a channel write does (the icon's own param-1 gate).
    sim.drones.channels.set(u32::from(sim.icons[ik].on_channel), 1, sim.drones.frame);
    // Bond stands at the icon.
    sim.bond = Bond::new(centre, 0.0);
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    assert_eq!(sim.bond.icon_ctx, 2, "the icon arms its ctx-2 while alive [M 0x196e80:36-52] (got {})", sim.bond.icon_ctx);
    // The kill channel fires (the trigger 0xEA's out write — [M 0x1b1a78] → the port's set).
    sim.drones.channels.set(0x6e, 1, sim.drones.frame);
    for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    assert_ne!(sim.bond.icon_ctx, 2, "the consumed icon never arms again [M 0x196e80:21-27] (got {})", sim.bond.icon_ctx);
}
