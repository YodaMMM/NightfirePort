//! level-4a-lift-2 probe: the post-enable hand-back. Three parts, diagnostic only (--nocapture):
//! A [D] the disc's drone/spawner placements near the hand-back y (the mid-landing garrison's xz —
//!   where the guards #250/#251 stand = real floor);
//! B [L] the port's collision column map at the composed hand-back spot (1.80,−6.83) — where
//!   floor_below/segment find floors between −104 and the shaft bottom;
//! C [L] the ride with the real path — board, ride, no input, dump Bond + the platform through
//!   the landing cut-scene's end and the fall after it (t2150..3700).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_post_enable_handback() {
    // ---- A: the disc's placements near the hand-back y ----
    {
        use nf_formats::{disc, iso9660::IsoImage, level::Level};
        let img = IsoImage::open(iso().as_str()).unwrap();
        let (entries, bin) = disc::files_bin_entries(&img).unwrap();
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).unwrap();
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let lv = Level::parse(&data).unwrap();
        let mm = lv.main_map().unwrap();
        eprintln!("== A: placements near the hand-back y (main map m{mm}, |y+105| < 9) ==");
        for m in lv.maps.iter() {
            for p in &m.placements {
                let t = p.translation;
                if (t[1] + 105.0).abs() < 9.0 {
                    eprintln!("  #{:<4} flags {:06x} hash {:08x} at ({:.2},{:.2},{:.2}) rot ({:.2},{:.2},{:.2}) params {}",
                        p.index, p.flags, p.hashcode, t[0], t[1], t[2], p.rotation[0], p.rotation[1], p.rotation[2],
                        p.params.iter().take(6).map(|(a, b)| format!("({a:#x},{b:#x})")).collect::<String>());
                }
            }
        }
    }

    // ---- B: the port's collision column at the composed hand-back spot ----
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== B: floor_below in the column (x, z) from y −104 and −100, depth 60 ==");
        for x in [0.6f32, 1.2, 1.8, 2.4, 3.0, 3.6, 4.2] {
            for z in [-7.5f32, -6.8, -6.0, -5.2, -4.5] {
                let f1 = sim.world.floor_below([x, -104.0, z], 60.0);
                let f2 = sim.world.floor_below([x, -100.0, z], 60.0);
                eprintln!("  x {x:5.2} z {z:5.2}: below−104 {f1:?}  below−100 {f2:?}");
            }
        }
        // The boarding spot's own floor for comparison (the platform top at the spawn).
        eprintln!("  spawn column x 4.26 z −4.76 from −6: {:?}", sim.world.floor_below([4.26, -6.0, -4.76], 60.0));
    }

    // ---- C: the ride, the landing scene's end, the fall ----
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    w.skip_cutscenes = false;
    eprintln!("== C: the ride + the post-enable fall (bond / platform / channels) ==");
    for k in 0..3620u32 {
        w.tick(Default::default(), false);
        let t = k + 84;
        if t < 2150 { continue; }
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let sp = sim.drones.scripts.players.iter().find(|sp| sp.scripts.contains(&0x06000571));
        let py = sp.map(|sp| sp.live_matrix[3][1]).unwrap_or(f32::NAN);
        let ch22 = sim.drones.channels.on(0x22);
        let ground = sim.world.floor_below([p[0], p[1] + 0.5, p[2]], 8.0);
        eprintln!("  t{t} bond ({:.2},{:.2},{:.2}) grounded {} disabled {} hp {:.0} | platform y {py:.2} | ch22 {ch22} | floor8 {:?}",
            p[0], p[1], p[2], sim.bond.grounded, sim.disabled, sim.drones.health.health, ground);
        if t > 3500 { break; }
    }
}
