//! level-07-party-move-scale: `Player_Move` 0x191dd0:139-141 — on 07000007 while switch channel
//! 0x48 is clear, the move mult param scales ×0.9 before it eases BOTH horizontal wish lanes.
//!
//! THE GATE [M disasm 0x191dd0-0x19230c]: at 0x1921d4-0x19220c the handler reads
//! GameState @0x2a3774 == 0x07000007 AND switch_channels[0x48] @0x26fcd8 == 0, then
//! `mul.s f23,f23,f0` with f0 = 0.9 (0x3F666666) — f23 is Player_Move's mult param — and the
//! scaled f23 feeds BOTH horizontal lanes: `mul.s f21,f21,f23` @0x192230 → BL+0x10 (strafe)
//! @0x1922d4, and `mul.s f20,f20,f23` @0x192250 → BL+0x18 (forward) @0x1922e0; the swim path
//! scales the same two lanes. The turn rate (BL+0x34 @0x1922c8) and the +0x20/+0x28 smoother
//! stores are NEVER scaled (no such store in the 335-instruction dump). The scale applies AFTER
//! the ±0.1 clamps and BEFORE position integration.
//!
//! THE OBSERVABLE [M]: the port's `Bond.move_local` is the BL+0x10/+0x18 wish quad. With
//! forward held, the forward smoother eases toward target_f = 0.1·fwd²+0.005 = 0.105, clamped
//! ±0.1 [M 0x1921d0 the clamp tail], so the converged wish lane is 0.1 × mult × 0.9 = **0.09**
//! with the gate and **0.1** without it. The strafe lane: target_s = 0.075 (unclamped), converged
//! 0.075 × 0.9 = **0.0675** vs 0.075. Both asserts are red on the pre-fix port and green with the
//! gate. The same ch-0x48 predicate drives the jump refusal [M Player_HandleJump 0x1916b0:106]
//! and the weapon lock [M Player_Weapon 0x1a24c8:244] — already ported; this branch is the move.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_party_move_scale_is_0x9_while_channel_0x48_is_clear() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x48),
            "the party boots with ch 0x48 clear (the hall plate #412 fires it later) [D #411/#412]");
        eprintln!("[007] spawn {:?} ch0x48 {}", sim.bond.pos, sim.drones.channels.on(0x48));
    }
    // The forward lane: hold forward on real input from the spawn; the wish quad converges
    // within ~40 ticks (the 0.2 ease). 120 ticks is 4× the settle time.
    for t in 0..120 {
        w.tick(Input { forward: 1.0, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let vf = sim.bond.move_local[1];
        eprintln!("[007] forward converged: move_local {:?} at {:?}", sim.bond.move_local, sim.bond.pos);
        assert!((vf - 0.09).abs() < 0.003,
            "the party's forward wish lane converges to 0.1×0.9 = 0.09 [M Player_Move 0x191dd0:139-141, disasm 0x19220c/0x192250], got {vf}");
    }
    // The strafe lane: same scale on BL+0x10 [M 0x192230/0x1922d4]. Forward released first so
    // its smoother drains toward 0 — 60 ticks is enough for the strafe ease to settle.
    for t in 0..60 {
        w.tick(Input { strafe: 1.0, ..Default::default() }, false);
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let vs = sim.bond.move_local[0];
        eprintln!("[007] strafe converged: move_local {:?} at {:?}", sim.bond.move_local, sim.bond.pos);
        assert!((vs - 0.0675).abs() < 0.003,
            "the party's strafe wish lane converges to 0.075×0.9 = 0.0675 [M Player_Move 0x191dd0:139-141, disasm 0x192230/0x1922d4], got {vs}");
    }
}
