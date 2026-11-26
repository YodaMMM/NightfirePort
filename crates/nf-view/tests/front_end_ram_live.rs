//! The front-end map's load cost: the boot load of 07000048 (`nf_view::load`, the window's first
//! scene) and the return-to-menu load (`load_front_end_gc`, the async loader's arm) measured for
//! wall time and the process's peak private bytes / working set.
//! Measurement only (`--ignored`, one test per process: the peak is a process-wide counter).

fn iso() -> Option<String> {
    let p = std::env::var("NF_ISO").ok()?;
    std::path::Path::new(&p).is_file().then_some(p)
}

/// (peak working set, peak private bytes) in MB.
fn peak_mb() -> (f64, f64) {
    #[cfg(windows)] {
        #[repr(C)] #[derive(Default)]
        struct Pmc { cb: u32, faults: u32, peak_ws: usize, ws: usize, a: usize, b: usize, c: usize, d: usize, pf: usize, peak_pf: usize }
        #[link(name = "kernel32")] extern "system" { fn K32GetProcessMemoryInfo(p: isize, c: *mut Pmc, cb: u32) -> i32; fn GetCurrentProcess() -> isize; }
        let mut m = Pmc { cb: std::mem::size_of::<Pmc>() as u32, ..Default::default() };
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut m, m.cb); }
        (m.peak_ws as f64 / 1048576.0, m.peak_pf as f64 / 1048576.0)
    }
    #[cfg(not(windows))] { (0.0, 0.0) }
}

fn report(tag: &str, s: &nf_view::scene::Scene, secs: f64) {
    let (ws, pf) = peak_mb();
    eprintln!("[fe-ram] {tag}: {secs:.2} s, peak working set {ws:.0} MB, peak private {pf:.0} MB, {} vertices, {} indices, {} materials",
        s.vertices.len(), s.indices.len(), s.materials.len());
    for (n, t) in nf_view::phases_take() { eprintln!("[fe-ram]   {n}: {t:.2} s"); }
}

#[test]
#[ignore]
fn boot_load() {
    let Some(iso) = iso() else { return };
    nf_view::phases_reset();
    let t = std::time::Instant::now();
    let s = nf_view::load(&iso, "07000048.bin", false).expect("07000048");
    report("boot load", &s, t.elapsed().as_secs_f64());
}

#[test]
#[ignore]
fn return_load() {
    let Some(iso) = iso() else { return };
    nf_view::phases_reset();
    let t = std::time::Instant::now();
    let s = nf_view::load_front_end_gc(&iso, "07000048.bin", false, None, false, 2, 0, false).expect("07000048");
    report("return load", &s, t.elapsed().as_secs_f64());
}

/// The front-end map builds no weapon-model or combat/effect geometry: 07000048 places no world
/// instance and no character, Bond owns nothing there and the sim never ticks under the menus, so
/// the session weapon-mesh merge (44 maps from 8 story archives) and the draw pools it fed were
/// dead weight (old tree: 6353686 vertices, 12632 pool attachments, ~1.3 GB peak private bytes,
/// ~1.7 s per front-end load). Both front-end load paths are checked: the boot load and the
/// return-to-menu load the async loader runs.
#[test]
fn front_end_builds_no_combat_geometry() {
    let Some(iso) = iso() else { return };
    for (tag, s) in [
        ("boot", nf_view::load(&iso, "07000048.bin", false).expect("07000048")),
        ("return", nf_view::load_front_end_gc(&iso, "07000048.bin", false, None, false, 2, 0, false).expect("07000048")),
    ] {
        assert!(s.sim.is_some(), "{tag}: the front-end map keeps its sim (Player_Start runs there)");
        assert!(s.hud.is_some(), "{tag}: the menus' font/HUD tables still load");
        assert!(s.attachments.is_empty(), "{tag}: {} pool attachments built on the front end", s.attachments.len());
        assert!(s.characters.is_empty(), "{tag}: {} characters on the front end", s.characters.len());
        assert_eq!(s.vertices.len(), 0, "{tag}: merged weapon meshes / pools left vertices on the front end");
    }
    // A story level still builds its pools (the gate is the front end only).
    let s = nf_view::load(&iso, "07000001.bin", false).expect("07000001");
    assert!(s.attachments.len() > 1000, "07000001 lost its draw pools ({} attachments)", s.attachments.len());
}
