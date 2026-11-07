//! fx-muzzle-and-trail row 12: the bullet trail's BIG puff through the real Sim path (nf-view
//! load + the sim tick). The facts under test:
//! - `Bullet_DoTrails` 0x12d890:397-400 latches obj+0xf6 = 1 (with state +0xf4 = 1) on the
//!   bullet's FIRST bounce — the path only records without the explode-on-hit bit (+0x70 bit
//!   0x20000) reach [M decomp :393-407]; `Bullet_trail_effect_smoke` 0x12cba8:29-53 then takes the
//!   `+0xf6 != 0` arm: `(GameState + (u8)obj[0xec]) % 15 == 0` → `Gas_Init(0.2, 5.0, …, 0x200001f,
//!   kind 0xb, 0x404040ff → 0x404040ff)` with the 6 shared draws [M :29-53 + Gas_Init 0x1aacc8].
//!   obj+0xec = GameState at creation [M control_init_object 0x134630:15], read as a byte.
//! - The smoke grenade 0x36 carries the trail bit (+0x6c & 0x2 [L ee_start_05.bin @ 0x2bf150 +
//!   0x36·0x10c + 0x6c = 0x8c0a]) AND the bounce bit (+0x70 & 0x20000 [L same +0x70 = 0x20280]),
//!   so a bounced 0x36 IS the game's big-puff site. Before the first bounce the trail runs the
//!   kind-7 fast arm (& 3 == 0, life 120) [M 0x12cba8:15-27].
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(iso: &str) -> nf_view::scene::Scene {
    nf_view::load(iso, "07000005.bin", false).expect("load 07000005")
}

#[test]
fn a_bounced_smoke_grenade_switches_the_trail_to_the_big_puff() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = load(&iso);
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = Bond::new([100.0, 26.0, 108.0], sim.bond.yaw);
    sim.disabled = false;
    for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    // The 0x36 record: trail bit + bounce bit, no exhaust bit [L ee_start_05.bin sweep].
    let f6c = u32::from_le_bytes(sim.weapons.records[0x36][0x6c..0x70].try_into().unwrap());
    assert_eq!(f6c & 0x2, 0x2, "0x36 carries the trail bit");
    assert_eq!(f6c & 0x20, 0, "0x36 carries no exhaust bit");
    // Drop a smoke grenade from head height onto the courtyard floor: it falls, bounces (the
    // 0x20000 path — never explodes on the hit), and the trail follows it through both arms.
    // The fuse stays long so the test ends before the smoke cloud pops (that is a different site).
    let born = sim.drones.frame as u8; // obj+0xec = GameState at creation [M 0x134630:15]
    sim.drones.grenades.push(nf_game::grenade::Grenade {
        loop_handle: 0,
        pos: [100.0, 31.0, 108.0], velocity: [0.0, -1.0, 0.0], speed: 1.0, travelled: 0.0,
        range: 100000.0, damage: 0.0, radius: 0.0,
        weapon_type: 0x36, mesh: 0, owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false,
        resting: false, stuck: false, seek_target: None, phase: None,
        burn: 0, defused: false, born,
    });
    let mut bounced_at = None;
    let mut big_at: Vec<u32> = Vec::new();
    let mut fast_at: Vec<u32> = Vec::new();
    for t in 0..140u32 {
        sim.step_bond(Input::default(), false, false, false);
        let frame = sim.drones.frame;
        if bounced_at.is_none() && sim.drones.grenades[0].bounces > 0 { bounced_at = Some(frame); }
        // Fresh puffs this tick: the pool update (retain_mut) runs at the head of the tick, so a
        // puff pushed by this tick's drain still carries its FULL spawn life (255 big / 120 fast).
        for g in &sim.drones.gas.puffs {
            if g.kind == 0xb && g.life == 255.0 { big_at.push(frame); }
            if g.kind == 7 && g.life == 120.0 { fast_at.push(frame); }
        }
    }
    let bounced = bounced_at.expect("the grenade lands and bounces");
    // Before the bounce: the fast arm on the (frame + born) & 3 == 0 ticks, never the big one.
    assert!(fast_at.iter().any(|&f| f < bounced), "the fast trail runs in flight");
    for &f in &fast_at { if f < bounced { assert_eq!((f.wrapping_add(born as u32)) & 3, 0, "fast arm tick {f}"); } }
    assert!(big_at.iter().all(|&f| f >= bounced), "no big puff before the first bounce");
    // After the bounce: the big arm on the (frame + born) % 15 == 0 ticks only.
    assert!(big_at.len() >= 3, "the % 15 gate fires repeatedly after the bounce (got {big_at:?})");
    for &f in &big_at {
        assert_eq!((f.wrapping_add(born as u32)) % 15, 0, "big puff tick {f} off the % 15 gate");
    }
    // The values on one live big puff [M 0x12cba8:29-53].
    let g = sim.drones.gas.puffs.iter().find(|g| g.kind == 0xb && g.life == 255.0).expect("a fresh big puff this tick");
    assert_eq!(g.mesh, 0x0200_001f);
    assert_eq!(g.scale, 0.2);
    assert_eq!(g.rgba, [0x40 as f32, 0x40 as f32, 0x40 as f32, 0xff as f32]);
    // The velocity draws: vx MVar2(0.3, 0.15)+0.6, vy MVar2(0.1, 0.05)+0.5, vz MVar2(0.3, 0.15)+0.3,
    // each ·REC_FRAME_RATE (1/60) [M 0x12ccc4/0x12cce4/0x12cd08 + the given-vel store 0x1ab26c].
    let rec = 1.0 / 60.0;
    assert!((0.45 * rec..=0.75 * rec).contains(&g.vel[0]), "vx in the drawn band, got {}", g.vel[0]);
    assert!((0.45 * rec..=0.55 * rec).contains(&g.vel[1]), "vy in the drawn band, got {}", g.vel[1]);
    assert!((0.15 * rec..=0.45 * rec).contains(&g.vel[2]), "vz in the drawn band, got {}", g.vel[2]);
    // Sister (can fail): the pre-fix gate ran the fast arm off `self.frame & 3` with no +born
    // phase and never emitted the big arm — a fresh-0xb tick off % 15 or none at all fails above.
}
