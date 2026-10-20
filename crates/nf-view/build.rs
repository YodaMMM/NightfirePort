// Bakes the third-party licence notices into the exe (a release is just the exe — the user, 2026-10-19). The release
// build (tools/release/make-release.ps1) writes them with notices.py and points NF_NOTICES_FILE at the file; a dev
// build gets a one-line placeholder.
fn main() {
    println!("cargo:rerun-if-env-changed=NF_NOTICES_FILE");
    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("notices.txt");
    let text = match std::env::var("NF_NOTICES_FILE") {
        Ok(p) if !p.is_empty() => {
            println!("cargo:rerun-if-changed={p}");
            std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("NF_NOTICES_FILE {p}: {e}"))
        }
        _ => "Third-party licence notices are included in release builds (tools/release/notices.py).\n".to_string(),
    };
    std::fs::write(out, text).unwrap();

    // The DLL-hijack guard (the user, 2026-10-19: NightfirePort.exe run from Downloads loaded a game mod's
    // `dxgi.dll` sitting beside it, which popped "UGameViewportClient::DrawTransition signature failed" boxes).
    // Windows looks in the exe's own folder first for every DLL that isn't a KnownDLL, and these are linked at load
    // time, before `main` can say otherwise. Delay-load them: their first use comes after `main` has restricted the
    // search to System32 (`bin/NightfirePort.rs`, SetDefaultDllDirectories).
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        for dll in DELAY_LOADED {
            println!("cargo:rustc-link-arg-bin=NightfirePort=/DELAYLOAD:{dll}");
        }
        println!("cargo:rustc-link-arg-bin=NightfirePort=delayimp.lib");
        // The exe's icon (the user, 2026-10-19: "also no icon for the exe"): an original gun-barrel mark drawn here
        // (no game art ships), written as a .res the MSVC linker takes directly — no rc.exe, no extra crate.
        let res = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("nightfireport_icon.res");
        std::fs::write(&res, icon::res()).unwrap();
        println!("cargo:rustc-link-arg-bin=NightfirePort={}", res.display());
    }
    // The same art as an .ico (the window icon, `include_bytes!`, and a preview).
    let ico = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("nightfireport.ico");
    std::fs::write(ico, icon::ico()).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

mod icon {
    /// The sizes Windows picks from (Explorer, the taskbar, Alt-Tab, the title bar).
    const SIZES: [u32; 5] = [16, 24, 32, 48, 256];

    /// One size as straight RGBA, top row first: a gun-barrel view — a gold rim, the dark bore with six rifling
    /// grooves twisting inward, and the light at the end of the barrel in the middle. 4×4 supersampled edges.
    fn rgba(s: u32) -> Vec<u8> {
        let mut out = vec![0u8; (s * s * 4) as usize];
        let n = 4;
        for y in 0..s {
            for x in 0..s {
                let mut acc = [0f32; 4];
                for sy in 0..n {
                    for sx in 0..n {
                        let fx = (x as f32 + (sx as f32 + 0.5) / n as f32) / s as f32 * 2.0 - 1.0;
                        let fy = (y as f32 + (sy as f32 + 0.5) / n as f32) / s as f32 * 2.0 - 1.0;
                        let r = (fx * fx + fy * fy).sqrt();
                        if r > 1.0 { continue; }
                        let c: [f32; 3] = if r > 0.88 {
                            // The gold rim, lit from the top left.
                            let lit = 0.75 + 0.25 * (-(fx + fy) * 0.7).clamp(-1.0, 1.0);
                            [0.95 * lit, 0.72 * lit, 0.25 * lit]
                        } else if r > 0.84 {
                            [0.10, 0.07, 0.03]
                        } else {
                            let th = fy.atan2(fx);
                            let groove = ((6.0 * th + 9.0 * r).sin() * 0.5 + 0.5).powf(6.0);
                            let base = 0.05 + 0.10 * (1.0 - r);
                            let bore = base + 0.10 * groove * r;
                            // The light at the end of the barrel.
                            let glow = (1.0 - (r / 0.26)).max(0.0).powf(1.5);
                            [bore + 0.95 * glow, bore + 0.90 * glow, bore + 0.80 * glow]
                        };
                        acc[0] += c[0]; acc[1] += c[1]; acc[2] += c[2]; acc[3] += 1.0;
                    }
                }
                let i = ((y * s + x) * 4) as usize;
                if acc[3] > 0.0 {
                    for k in 0..3 { out[i + k] = ((acc[k] / acc[3]).clamp(0.0, 1.0) * 255.0) as u8; }
                    out[i + 3] = (acc[3] / (n * n) as f32 * 255.0) as u8;
                }
            }
        }
        out
    }

    /// One icon image as the DIB an .ico / RT_ICON holds: BITMAPINFOHEADER (height doubled), BGRA rows bottom-up,
    /// then an all-zero AND mask (the alpha channel carries the shape).
    fn dib(s: u32) -> Vec<u8> {
        let px = rgba(s);
        let mask_row = s.div_ceil(32) * 4;
        let mut d = Vec::new();
        for v in [40u32, s, s * 2] { d.extend_from_slice(&v.to_le_bytes()); }
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&32u16.to_le_bytes());
        for v in [0u32, s * s * 4 + mask_row * s, 0, 0, 0, 0] { d.extend_from_slice(&v.to_le_bytes()); }
        for y in (0..s).rev() {
            for x in 0..s {
                let i = ((y * s + x) * 4) as usize;
                d.extend_from_slice(&[px[i + 2], px[i + 1], px[i], px[i + 3]]);
            }
        }
        d.extend(std::iter::repeat(0u8).take((mask_row * s) as usize));
        d
    }

    fn dir_entry(s: u32, len: usize) -> Vec<u8> {
        let b = if s >= 256 { 0u8 } else { s as u8 };
        let mut e = vec![b, b, 0, 0];
        e.extend_from_slice(&1u16.to_le_bytes());
        e.extend_from_slice(&32u16.to_le_bytes());
        e.extend_from_slice(&(len as u32).to_le_bytes());
        e
    }

    /// The .ico file: ICONDIR + entries (with file offsets) + the DIBs.
    pub fn ico() -> Vec<u8> {
        let imgs: Vec<Vec<u8>> = SIZES.iter().map(|&s| dib(s)).collect();
        let mut f = vec![0, 0, 1, 0];
        f.extend_from_slice(&(SIZES.len() as u16).to_le_bytes());
        let mut off = 6 + 16 * SIZES.len();
        for (k, &s) in SIZES.iter().enumerate() {
            f.extend(dir_entry(s, imgs[k].len()));
            f.extend_from_slice(&(off as u32).to_le_bytes());
            off += imgs[k].len();
        }
        for i in &imgs { f.extend_from_slice(i); }
        f
    }

    fn res_entry(out: &mut Vec<u8>, ty: u16, name: u16, flags: u16, data: &[u8]) {
        for v in [data.len() as u32, 32] { out.extend_from_slice(&v.to_le_bytes()); }
        for v in [0xffffu16, ty, 0xffff, name] { out.extend_from_slice(&v.to_le_bytes()); }
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&0x0409u16.to_le_bytes());
        for v in [0u32, 0] { out.extend_from_slice(&v.to_le_bytes()); }
        out.extend_from_slice(data);
        while out.len() % 4 != 0 { out.push(0); }
    }

    /// The .res: the empty lead entry, one RT_ICON (3) per size, then RT_GROUP_ICON (14) #1 — the first group icon
    /// is the one Explorer shows for the exe.
    pub fn res() -> Vec<u8> {
        let mut out = Vec::new();
        res_entry(&mut out, 0, 0, 0, &[]);
        let mut grp = vec![0, 0, 1, 0];
        grp.extend_from_slice(&(SIZES.len() as u16).to_le_bytes());
        for (k, &s) in SIZES.iter().enumerate() {
            let d = dib(s);
            res_entry(&mut out, 3, (k + 1) as u16, 0x1010, &d);
            grp.extend(dir_entry(s, d.len()));
            grp.extend_from_slice(&((k + 1) as u16).to_le_bytes());
        }
        res_entry(&mut out, 14, 1, 0x1030, &grp);
        out
    }
}

/// The hijack-prone (non-KnownDLL) system DLLs the exe imports — the names mod loaders and overlays ship as proxies.
/// tools/release/check-imports.py verifies the release exe imports none of them directly.
const DELAY_LOADED: [&str; 5] = ["dxgi.dll", "dwmapi.dll", "uxtheme.dll", "opengl32.dll", "mmdevapi.dll"];
