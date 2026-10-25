//! level-14-cutscene-fall-2 probe: the 01/08 script structure under the camera-start base
//! reset. The merged tree (origin/master + origin/s/level-14-cutscene-fall) turns 6 test groups
//! red; each was green on master. This probe dumps, read-only:
//! 1. every ScriptPlayer placement on 07000001 and 07000008 (scripts, modes, gate, base);
//! 2. every script in the lib: camera flag, length, per stream first/last key + decoded cmds;
//! 3. which streams carry op 0x0d (camera) vs entity/anim blocks, in stream order, so the
//!    reset's reach (which keys sample after it, in which SCRIPTINFO) can be pinned against
//!    the decompile [M Script_CameraStart 0x1d9140 → Mat_IdentityT(SCRIPTINFO) 0x1d916c;
//!    Script_Update 0x1da998 runs every stream of the SCRIPTINFO every tick].
//!
//! Opt-in via `NF_ISO`. Run with `--nocapture --test-threads=1`.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn probe_script_structure_01_08() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    for level in ["07000001.bin", "07000008.bin"] {
        let scene = nf_view::load(&iso, level, false).unwrap_or_else(|e| panic!("load {level}: {e}"));
        let sim = scene.sim.as_ref().unwrap();
        eprintln!("######## {level} ########");
        eprintln!("=== ScriptPlayer placements ===");
        for p in &sim.drones.scripts.players {
            eprintln!(
                "SP #{} scripts {:08x}/{:08x} modes {:?} gate {:#x} out {:#x} stay {} base\n  [{:.3} {:.3} {:.3}][{:.3} {:.3} {:.3}][{:.3} {:.3} {:.3}] t ({:.2},{:.2},{:.2})",
                p.placement, p.scripts[0], p.scripts[1], p.modes, p.gate, p.out, p.stay,
                p.matrix[0][0], p.matrix[1][0], p.matrix[2][0],
                p.matrix[0][1], p.matrix[1][1], p.matrix[2][1],
                p.matrix[0][2], p.matrix[1][2], p.matrix[2][2],
                p.matrix[3][0], p.matrix[3][1], p.matrix[3][2],
            );
        }
        eprintln!("=== scripts in lib: {} ===", sim.drones.scripts.lib.len());
        let mut hashes: Vec<u32> = sim.drones.scripts.lib.keys().copied().collect();
        hashes.sort();
        for h in hashes {
            let n = &sim.drones.scripts.lib[&h];
            eprintln!(
                "--- NIS {h:08x}: camera-flag {} length {} loop_start {} streams {} keys {}",
                n.camera, n.length, n.loop_start, n.streams.len(), n.keys.len()
            );
            for (i, s) in n.streams.iter().enumerate() {
                let cmds = s.decode();
                // Flag the ops that matter for the base-reset question.
                let mut marks = Vec::new();
                for c in &cmds {
                    match c {
                        nf_formats::nis::Cmd::Camera { .. } => marks.push("CAM".into()),
                        nf_formats::nis::Cmd::Entity { hash, parent, obj_type, .. } => marks.push(format!("ENT {hash:08x} parent {parent:?} kind {obj_type:#x}")),
                        nf_formats::nis::Cmd::Anim { script, .. } => marks.push(format!("ANIM {script:08x}")),
                        nf_formats::nis::Cmd::SubScript { hash, stream, .. } => marks.push(format!("SUB {hash:08x} stream {stream}")),
                        _ => {}
                    }
                }
                eprintln!("  s{i}: keys {}..{} cmds {:?}", s.first_key, s.last_key, cmds);
                if !marks.is_empty() { eprintln!("      marks: {}", marks.join(" | ")); }
            }
        }
    }
}

#[path = "common/walker.rs"]
mod walker2;
use walker2::Walker;

#[test]
fn probe_entity_matrices_after_intro() {
    let Some(_iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&_iso, "07000001.bin", false).expect("load"));
    for t in 0..4600u32 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        if let Some(nf_game::nis::Obj::Entity { hash, kind, matrix, .. }) = o {
            eprintln!("ENT {i} {hash:08x} kind {kind:#x} m00 {:.9} t ({:.5},{:.5},{:.5})",
                matrix[0][0], matrix[3][0], matrix[3][1], matrix[3][2]);
        }
    }
    eprintln!("BOND ({:.5},{:.5},{:.5})", sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
}

#[test]
fn probe_first_divergence_01() {
    let Some(_iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&_iso, "07000001.bin", false).expect("load"));
    for t in 0..4600u32 {
        w.tick(Default::default(), false);
        let s = w.scene.sim.as_ref().unwrap();
        let mut h: u64 = 0;
        for d in &s.drones.list { h = h.wrapping_mul(0x100000001b3).wrapping_add(d.pos[0].to_bits() as u64 ^ d.state as u64); }
        eprintln!("T {t} B {:.4} {:.4} {:.4} D {} H {h:016x}",
            s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.doors.list[0].state);
    }
}

#[test]
fn probe_drones_at_divergence() {
    let Some(_iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&_iso, "07000001.bin", false).expect("load"));
    for t in 0..258u32 {
        w.tick(Default::default(), false);
        if (250..=256).contains(&t) {
            let s = w.scene.sim.as_ref().unwrap();
            for (i, d) in s.drones.list.iter().enumerate() {
                eprintln!("T{t} D{i} skin {:08x} state {:x} p ({:.9},{:.9},{:.9})",
                    d.skin, d.state, d.pos[0], d.pos[1], d.pos[2]);
            }
        }
    }
}
