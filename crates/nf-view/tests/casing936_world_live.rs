//! standins-casing-936 real path: the game's `Collide_LineOfSight` masks all carry bit 0x8 —
//! the pass-flag drop [M Player_HandleJump 0x1916b0 mask 9, Player_SSCrouch 0x1924c0 mask 9,
//! Player_SSGrapple 0x192860 mask 8, Sensor_InCone 0x1af4f0 / Check_AutoAim 0x1a73f8 mask 0xa,
//! Bullet_homing 0x12f358 mask 8] — so on the disc's foliage (07000005's trees carry glist
//! flags 0x41 [D, the segment_bullet doc's census]) there are sight lines the raw segment
//! blocks and the bit-0x8 query sees through. `Doors::line_of_sight` ran the raw
//! `segment_clear` before this task. Opt-in: NF_ISO.

#[path = "common/walker.rs"]
mod walker;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn los_sees_through_the_discs_pass_flagged_foliage() {
    if std::env::var_os("NF_ISO").is_none() { eprintln!("skip: NF_ISO not set"); return; }
    let mut w = Walker::new(nf_view::load(&iso(), "07000005.bin", false).unwrap());
    w.wait_enabled(3000, "level start");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    let world = &sim.world;
    // Sweep horizontal rays from around the spawn: find one the RAW segment blocks but the
    // bit-0x8 query (segment_bullet's drop) sees through — pass-flagged foliage. Sister: the
    // first plainly-blocked ray (both queries hit) must stay blocked for line_of_sight.
    let (mut foliage, mut wall) = (None, None);
    'outer: for ox in [-4.0f32, 0.0, 4.0] {
        for oz in [-4.0f32, 0.0, 4.0] {
            for oy in [0.6f32, 1.2, 2.0] {
                for k in 0..72 {
                    let yaw = k as f32 * std::f32::consts::TAU / 72.0;
                    let (sy, cy) = (yaw.sin(), yaw.cos());
                    for dist in [12.0f32, 24.0] {
                        let from = [sim.bond.pos[0] + ox, sim.bond.pos[1] + oy, sim.bond.pos[2] + oz];
                        let to = [from[0] + cy * dist, from[1], from[2] + sy * dist];
                        let raw = world.segment(from, to).is_some();
                        let dropped = world.segment_bullet(from, to).is_none();
                        if raw && dropped && foliage.is_none() { foliage = Some((from, to)); }
                        if raw && !dropped && wall.is_none() { wall = Some((from, to)); }
                        if foliage.is_some() && wall.is_some() { break 'outer; }
                    }
                }
            }
        }
    }
    let (f, t) = foliage.expect("no raw-blocked/bit-8-clear ray found from the spawn grid — \
        the probe saw no pass-flagged foliage; 07000005's trees carry glist flags 0x41 [D]");
    assert!(world.line_of_sight(f, t),
        "the bit-0x8 LOS query must see through the pass-flagged foliage at {f:?} → {t:?}");
    eprintln!("[los] clear through foliage: {f:?} → {t:?}");
    if let Some((f, t)) = wall {
        assert!(!world.line_of_sight(f, t),
            "sister: plain geometry (raw and bit-0x8 both hit) still blocks the LOS at {f:?} → {t:?}");
        eprintln!("[los] sister wall blocks: {f:?} → {t:?}");
    }
}
