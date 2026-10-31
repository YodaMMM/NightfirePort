//! Probe: every placement of every 0700001b map with its kind, position and params (hex), one
//! line each — the channel map for the mission's writer hunt. Assertion-free; run with
//! --ignored --nocapture. [D via nf-formats::level.]

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0700001b_every_placement() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let data = read_archive(&iso, "0700001b.bin");
    let lv = Level::parse(&data).expect("parse");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let mut ps = String::new();
            for (j, v) in &p.params { ps.push_str(&format!(" p{j}={v:#x}")); }
            println!("map{mi} #{i} kind={:#04x} flags={:#x} hash={:#x} pos=({:.2},{:.2},{:.2}){ps}",
                p.flags & 0xff, p.flags, p.hashcode, p.translation[0], p.translation[1], p.translation[2]);
        }
    }
}

#[path = "common/walker.rs"]
#[allow(dead_code)]
mod walker;

/// The natural flow: load, no substate force, Bond idle in the start's float; log every channel
/// value change per tick for N ticks (NF_1B_TICKS, default 12000), plus Bond's substate/pos.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_idle_channels() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let n: u32 = std::env::var("NF_1B_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(12000);
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    for t in 0..n {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let v = &sim.drones.channels.v;
        for c in 0..256 {
            if v[c] != prev[c] {
                println!("t{t} f{} ch {c} ({c:#x}) {} -> {}  bond ss {} pos ({:.2},{:.2},{:.2}) dis {} mstat {}",
                    sim.drones.frame, prev[c], v[c], sim.bond.substate, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
                    sim.disabled, sim.drones.mission.status());
            }
        }
        prev = v.clone();
        if t % 1000 == 0 {
            println!("t{t} bond ss {} pos ({:.2},{:.2},{:.2}) disabled {} missiles {:?}", sim.bond.substate,
                sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], sim.disabled,
                sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>());
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !d.present { continue; }
                println!("   drone {i} skin {:#x} pos ({:.2},{:.2},{:.2}) en {} corpse {} state {:#x} hp {:.1}",
                    d.skin, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.corpse, d.state, d.health);
            }
        }
    }
}

fn log_changes(w: &walker::Walker, prev: &mut Vec<u8>, t: u32) {
    let sim = w.scene.sim.as_ref().unwrap();
    let v = &sim.drones.channels.v;
    for c in 0..256 {
        if v[c] != prev[c] {
            println!("t{t} f{} ch {c} ({c:#x}) {} -> {}  bond ss {} pos ({:.2},{:.2},{:.2}) mstat {} hp {:.0}",
                sim.drones.frame, prev[c], v[c], sim.bond.substate, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2],
                sim.drones.mission.status(), sim.drones.health.health);
        }
    }
    *prev = v.clone();
}

/// The natural flow with the real fire: wait out the arrival, kill every spawned astronaut,
/// then idle and log.
#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn trace_0700001b_kill_wave() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    let mut prev = w.scene.sim.as_ref().unwrap().drones.channels.v.clone();
    let mut t = 0u32;
    while t < 1600 { w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1; }
    for round in 0..20 {
        let target = {
            let sim = w.scene.sim.as_ref().unwrap();
            sim.drones.list.iter().position(|d| d.present && !d.corpse && d.enabled && d.health > 0.0 && d.skin & 0xffff == 0x78)
        };
        let Some(di) = target else { println!("round {round}: no live astronaut"); break };
        let ok = w.shoot(di, "astronaut");
        log_changes(&w, &mut prev, t);
        let sim = w.scene.sim.as_ref().unwrap();
        println!("round {round}: shoot d{di} -> {ok}; bond ss {} pos {:?} hp {:.0} frame {}", sim.bond.substate, sim.bond.pos, sim.drones.health.health, sim.drones.frame);
    }
    let n: u32 = std::env::var("NF_1B_TICKS").ok().and_then(|s| s.parse().ok()).unwrap_or(6000);
    for _ in 0..n {
        w.tick(Default::default(), false); log_changes(&w, &mut prev, t); t += 1;
        if t % 1000 == 0 {
            let sim = w.scene.sim.as_ref().unwrap();
            println!("t{t} missiles {:?} deploy {:?}", sim.drones.space_missiles.iter().map(|m| m.state).collect::<Vec<_>>(), sim.drones.missile_deploy);
        }
    }
}

#[test]
#[ignore = "assertion-free trace; run with --ignored"]
fn zerog_axis_signs() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "arrival");
    w.tick(Default::default(), false);
    let r0 = w.scene.sim.as_ref().unwrap().bond.zero_g_rot;
    println!("rot0 {:?} active {}", r0, w.scene.sim.as_ref().unwrap().bond.zero_g_active);
    w.tick(nf_game::player::Input { turn: 0.1, ..Default::default() }, false);
    let r1 = w.scene.sim.as_ref().unwrap().bond.zero_g_rot;
    println!("after turn +0.1: fwd {:?} (was {:?}), right {:?}", r1[2], r0[2], r0[0]);
    w.tick(nf_game::player::Input { look: 0.1, ..Default::default() }, false);
    let r2 = w.scene.sim.as_ref().unwrap().bond.zero_g_rot;
    println!("after look +0.1: fwd {:?} (was {:?}), up {:?}", r2[2], r1[2], r1[1]);
    let sim = w.scene.sim.as_ref().unwrap();
    println!("look() {:?} eye {} crouch {}", sim.bond.look(), sim.bond.eye, sim.bond.crouch_drop);
}
