//! Deep game-state introspection, real-time performance telemetry, and 1:1 PNG/RGBA visual diffing
//! for rapid agentic debugging on macOS Metal.

use std::path::Path;

use anyhow::{anyhow, Result};

use crate::engine::{json_escape, MacEngine};

/// Per-frame timing and culling telemetry captured during `MacEngine::step` and `render_png`/`bench`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PerfStats {
    pub last_step_ms: f64,
    pub last_skin_ms: f64,
    pub last_render_ms: f64,
    pub last_encode_ms: f64,
    pub last_visible_cels: usize,
    pub total_cels: usize,
}

impl PerfStats {
    pub fn to_json(&self) -> String {
        let frame_ms = self.last_step_ms + self.last_skin_ms + self.last_render_ms;
        let est_fps = if frame_ms > 0.001 {
            1000.0 / frame_ms
        } else {
            0.0
        };
        format!(
            "{{\"step_ms\":{:.3},\"skin_ms\":{:.3},\"render_ms\":{:.3},\"encode_ms\":{:.3},\"frame_ms\":{:.3},\"est_fps\":{:.1},\"visible_cels\":{},\"total_cels\":{}}}",
            self.last_step_ms,
            self.last_skin_ms,
            self.last_render_ms,
            self.last_encode_ms,
            frame_ms,
            est_fps,
            self.last_visible_cels,
            self.total_cels
        )
    }
}

/// Summary returned by `MacEngine::bench_frames`.
#[derive(Clone, Copy, Debug)]
pub struct BenchReport {
    pub frames: u32,
    pub width: u32,
    pub height: u32,
    pub avg_ms: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub est_fps: f64,
    pub avg_sim_ms: f64,
    pub avg_skin_ms: f64,
    pub avg_gpu_ms: f64,
}

impl BenchReport {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"frames\":{},\"width\":{},\"height\":{},\"avg_ms\":{:.3},\"p50_ms\":{:.3},\"p95_ms\":{:.3},\"min_ms\":{:.3},\"max_ms\":{:.3},\"est_fps\":{:.1},\"avg_sim_ms\":{:.3},\"avg_skin_ms\":{:.3},\"avg_gpu_ms\":{:.3}}}",
            self.frames,
            self.width,
            self.height,
            self.avg_ms,
            self.p50_ms,
            self.p95_ms,
            self.min_ms,
            self.max_ms,
            self.est_fps,
            self.avg_sim_ms,
            self.avg_skin_ms,
            self.avg_gpu_ms
        )
    }
}

/// Pixel-level 1:1 visual comparison metrics between two RGBA images.
#[derive(Clone, Copy, Debug)]
pub struct FrameDiff {
    pub width: u32,
    pub height: u32,
    pub mae_rgb: f64,
    pub rmse_rgb: f64,
    pub psnr_db: f64,
    pub max_diff: u8,
    pub pct_over_4: f64,
    pub pct_over_16: f64,
    pub identical_pct: f64,
}

impl FrameDiff {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"width\":{},\"height\":{},\"mae_rgb\":{:.4},\"rmse_rgb\":{:.4},\"psnr_db\":{:.2},\"max_diff\":{},\"pct_over_4\":{:.3},\"pct_over_16\":{:.3},\"identical_pct\":{:.3}}}",
            self.width,
            self.height,
            self.mae_rgb,
            self.rmse_rgb,
            self.psnr_db,
            self.max_diff,
            self.pct_over_4,
            self.pct_over_16,
            self.identical_pct
        )
    }
}

/// Builds a deep JSON snapshot of internal engine state
/// (`all`, `sim`, `drones`, `driving`, `doors`, `pickups`, `objectives`, `scene`, `menu`, `perf`).
pub fn inspect_json(engine: &MacEngine, topic: &str) -> String {
    match topic {
        "sim" => inspect_sim(engine),
        "drones" => inspect_drones(engine),
        "driving" => inspect_driving(engine),
        "doors" => inspect_doors(engine),
        "pickups" => inspect_pickups(engine),
        "objectives" => inspect_objectives(engine),
        "scene" => inspect_scene(engine),
        "menu" => inspect_menu(engine),
        "perf" => engine.perf.to_json(),
        _ => format!(
            "{{\"telemetry\":{},\"perf\":{},\"scene\":{},\"sim\":{},\"drones\":{},\"doors\":{},\"pickups\":{},\"objectives\":{},\"driving\":{},\"menu\":{}}}",
            engine.telemetry().to_json(),
            engine.perf.to_json(),
            inspect_scene(engine),
            inspect_sim(engine),
            inspect_drones(engine),
            inspect_doors(engine),
            inspect_pickups(engine),
            inspect_objectives(engine),
            inspect_driving(engine),
            inspect_menu(engine),
        ),
    }
}

fn inspect_scene(engine: &MacEngine) -> String {
    let s = &engine.scene;
    let triangles = s.indices.len() / 3;
    let particles: usize = s.drive_particles.iter().map(|(_, _, v)| v.len()).sum();
    format!(
        "{{\"vertices\":{},\"triangles\":{},\"materials\":{},\"textures\":{},\"cels\":{},\"drive_particles\":{},\"drive_hud_verts\":{},\"drive_reflection\":{},\"mirror_x\":{}}}",
        s.vertices.len(),
        triangles,
        s.materials.len(),
        s.textures.len(),
        s.cels.len(),
        particles,
        s.drive_hud.v.len(),
        s.drive_reflection.is_some(),
        s.mirror_x,
    )
}

fn inspect_sim(engine: &MacEngine) -> String {
    let Some(sim) = engine.scene.sim.as_ref() else {
        return "null".to_string();
    };
    let closed_flags = sim.drones.doors.closed_flags();
    let doors_total = closed_flags.len();
    let doors_closed = closed_flags.iter().filter(|&&c| c).count();
    let pickups_total = sim.drones.pickups.len();
    let pickups_active = sim
        .drones
        .pickups
        .iter()
        .filter(|p| !p.gone && !p.hidden)
        .count();
    let active_channels: Vec<String> = sim
        .drones
        .channels
        .v
        .iter()
        .enumerate()
        .filter(|&(_, &val)| val != 0)
        .take(32)
        .map(|(ch, _)| ch.to_string())
        .collect();
    let ch_str = active_channels;
    format!(
        "{{\"frame\":{},\"pos\":[{:.3},{:.3},{:.3}],\"yaw\":{:.4},\"pitch\":{:.4},\"substate\":{},\"crouch_drop\":{:.3},\"eye\":{:.3},\"health\":{:.2},\"armour\":{:.2},\"weapon\":{},\"clip\":{},\"spare\":{},\"disabled\":{},\"script_camera\":{},\"view_override\":{},\"doors_total\":{},\"doors_closed\":{},\"pickups_total\":{},\"pickups_active\":{},\"active_channels\":[{}]}}",
        sim.drones.frame,
        sim.bond.pos[0],
        sim.bond.pos[1],
        sim.bond.pos[2],
        sim.bond.yaw,
        sim.bond.pitch,
        sim.bond.substate,
        sim.bond.crouch_drop,
        sim.bond.eye,
        sim.drones.health.health,
        sim.drones.health.armour,
        sim.gun.weapon,
        sim.gun.clip,
        sim.gun.spare,
        sim.disabled,
        sim.script_camera().is_some(),
        sim.view_override().is_some(),
        doors_total,
        doors_closed,
        pickups_total,
        pickups_active,
        ch_str.join(","),
    )
}

fn inspect_drones(engine: &MacEngine) -> String {
    let Some(sim) = engine.scene.sim.as_ref() else {
        return "null".to_string();
    };
    let bp = sim.bond.pos;
    let total = sim.drones.list.len();
    let mut alive = 0usize;
    let mut alerted = 0usize;
    let mut entries: Vec<(f32, usize)> = Vec::with_capacity(total);
    for (idx, d) in sim.drones.list.iter().enumerate() {
        if d.health > 0.0 {
            alive += 1;
            if d.alert > 0.0 {
                alerted += 1;
            }
        }
        let dx = d.pos[0] - bp[0];
        let dy = d.pos[1] - bp[1];
        let dz = d.pos[2] - bp[2];
        entries.push(((dx * dx + dy * dy + dz * dz).sqrt(), idx));
    }
    entries.sort_by(|a, b| a.0.total_cmp(&b.0));
    let nearest: Vec<String> = entries
        .into_iter()
        .take(8)
        .map(|(dist, idx)| {
            let d = &sim.drones.list[idx];
            format!(
                "{{\"idx\":{},\"skin\":\"0x{:08x}\",\"state\":{},\"hp\":{:.1},\"alert\":{:.2},\"pos\":[{:.2},{:.2},{:.2}],\"dist\":{:.2}}}",
                idx, d.skin, d.state, d.health, d.alert, d.pos[0], d.pos[1], d.pos[2], dist
            )
        })
        .collect();
    format!(
        "{{\"total\":{},\"alive\":{},\"alerted\":{},\"nearest\":[{}]}}",
        total,
        alive,
        alerted,
        nearest.join(",")
    )
}

fn inspect_doors(engine: &MacEngine) -> String {
    let Some(sim) = engine.scene.sim.as_ref() else {
        return "null".to_string();
    };
    let bp = sim.bond.pos;
    let closed = sim.drones.doors.closed_flags();
    let mut rows: Vec<(f32, String)> = sim
        .drones
        .doors
        .list
        .iter()
        .enumerate()
        .map(|(idx, d)| {
            let dx = d.pos[0] - bp[0];
            let dy = d.pos[1] - bp[1];
            let dz = d.pos[2] - bp[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let is_closed = closed.get(idx).copied().unwrap_or(false);
            (
                dist,
                format!(
                    "{{\"idx\":{},\"state\":{},\"closed\":{},\"pos\":[{:.2},{:.2},{:.2}],\"dist\":{:.2}}}",
                    idx, d.state, is_closed, d.pos[0], d.pos[1], d.pos[2], dist
                ),
            )
        })
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let list: Vec<String> = rows.into_iter().take(16).map(|(_, s)| s).collect();
    format!(
        "{{\"total\":{},\"closed\":{},\"nearest\":[{}]}}",
        sim.drones.doors.list.len(),
        closed.iter().filter(|&&c| c).count(),
        list.join(",")
    )
}

fn inspect_pickups(engine: &MacEngine) -> String {
    let Some(sim) = engine.scene.sim.as_ref() else {
        return "null".to_string();
    };
    let bp = sim.bond.pos;
    let mut rows: Vec<(f32, String)> = sim
        .drones
        .pickups
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.gone && !p.hidden)
        .map(|(idx, p)| {
            let pos = [p.m[3][0], p.m[3][1], p.m[3][2]];
            let dx = pos[0] - bp[0];
            let dy = pos[1] - bp[1];
            let dz = pos[2] - bp[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            (
                dist,
                format!(
                    "{{\"idx\":{},\"kind\":{},\"weapon\":{},\"rounds\":{},\"pos\":[{:.2},{:.2},{:.2}],\"dist\":{:.2}}}",
                    idx, p.kind, p.weapon, p.rounds, pos[0], pos[1], pos[2], dist
                ),
            )
        })
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let active = rows.len();
    let list: Vec<String> = rows.into_iter().take(16).map(|(_, s)| s).collect();
    format!(
        "{{\"total\":{},\"active\":{},\"nearest\":[{}]}}",
        sim.drones.pickups.len(),
        active,
        list.join(",")
    )
}

fn inspect_objectives(engine: &MacEngine) -> String {
    let Some(sim) = engine.scene.sim.as_ref() else {
        return "null".to_string();
    };
    let m = &sim.drones.mission;
    let lang = engine.lang.as_ref().or(sim.lang.as_ref());
    let n = m.num_vis_objectives();
    let mut items = Vec::with_capacity(n);
    for i in 0..n {
        if let Some(o) = m.objective_state(i) {
            let label = lang
                .and_then(|l| l.bind(o.text))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| format!("0x{:x}", o.text));
            items.push(format!(
                "{{\"idx\":{},\"state\":{},\"text_id\":\"0x{:x}\",\"text\":{}}}",
                i,
                o.state,
                o.text,
                json_escape(&label)
            ));
        }
    }
    format!(
        "{{\"ended\":{},\"count\":{},\"items\":[{}]}}",
        m.ended,
        items.len(),
        items.join(",")
    )
}

fn inspect_driving(engine: &MacEngine) -> String {
    let Some(drv) = engine.driving.as_ref() else {
        return "null".to_string();
    };
    let speed = (drv.car.vel[0].powi(2) + drv.car.vel[2].powi(2)).sqrt();
    let player_drives = drv.intro.as_ref().map_or(true, |(i, _)| i.player_drives());
    let on_spline = drv
        .intro
        .as_ref()
        .is_some_and(|(i, p)| i.spline_position(p).is_some());
    let physics_mode = if drv.sub.is_some() {
        "sub"
    } else if drv.land.is_some() {
        "land"
    } else {
        "flat"
    };
    let triggers_hit = drv.triggered.iter().filter(|&&t| t).count();
    let (hostiles_total, hostiles_active, damage_dealt) = drv
        .hostiles
        .as_ref()
        .map(|h| (h.world.db.elements.len(), h.assigned.len(), h.damage_dealt))
        .unwrap_or((0, 0, 0.0));
    let wheels = [
        drv.wheels[0].compression,
        drv.wheels[1].compression,
        drv.wheels[2].compression,
        drv.wheels[3].compression,
    ];
    format!(
        "{{\"viv\":{},\"ticks\":{},\"difficulty\":{},\"pos\":[{:.3},{:.3},{:.3}],\"vel\":[{:.3},{:.3},{:.3}],\"speed\":{:.3},\"yaw\":{:.4},\"player_drives\":{},\"on_spline\":{},\"physics_mode\":{},\"mission_state\":{},\"progress\":{},\"triggers_hit\":{},\"triggers_total\":{},\"road_nodes\":{},\"ai_cars\":{},\"hostiles_total\":{},\"hostiles_active\":{},\"damage_dealt\":{:.2},\"wheels\":[{:.3},{:.3},{:.3},{:.3}]}}",
        json_escape(&drv.viv),
        drv.ticks,
        drv.difficulty,
        drv.car.pos[0],
        drv.car.pos[1],
        drv.car.pos[2],
        drv.car.vel[0],
        drv.car.vel[1],
        drv.car.vel[2],
        speed,
        drv.car.yaw,
        player_drives,
        on_spline,
        json_escape(physics_mode),
        json_escape(&format!("{:?}", drv.mission_state)),
        drv.progress,
        triggers_hit,
        drv.triggers.len(),
        drv.road_nodes.len(),
        drv.ai_cars.len(),
        hostiles_total,
        hostiles_active,
        damage_dealt,
        wheels[0],
        wheels[1],
        wheels[2],
        wheels[3],
    )
}

fn inspect_menu(engine: &MacEngine) -> String {
    let Some(fe) = engine.fe.as_ref() else {
        return "null".to_string();
    };
    let req = fe
        .requested_level
        .map(|l| format!("\"0x{l:08x}\""))
        .unwrap_or_else(|| "null".to_string());
    let notice = fe
        .notice
        .as_deref()
        .map(json_escape)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"page\":{},\"cursor\":{},\"unlock_all\":{},\"requested_level\":{},\"notice\":{}}}",
        json_escape(&format!("{:?}", fe.page)),
        fe.cursor,
        fe.unlock_all,
        req,
        notice,
    )
}

/// Computes pixel-by-pixel RGB difference metrics between two RGBA8 buffers and optionally
/// writes an amplified visual diff PNG (4x error in red/yellow over darkened grayscale base).
pub fn diff_rgba(
    w: u32,
    h: u32,
    a: &[u8],
    b: &[u8],
    out_diff_png: Option<&Path>,
) -> Result<FrameDiff> {
    let expected = (w as usize) * (h as usize) * 4;
    if a.len() != expected || b.len() != expected {
        return Err(anyhow!(
            "buffer size mismatch: expected {expected}, got {} and {}",
            a.len(),
            b.len()
        ));
    }
    let pixels = (w as usize) * (h as usize);
    let mut sum_abs = 0u64;
    let mut sum_sq = 0u64;
    let mut max_diff = 0u8;
    let mut over_4 = 0usize;
    let mut over_16 = 0usize;
    let mut identical = 0usize;
    let mut heatmap = out_diff_png.map(|_| vec![0u8; expected]);

    for i in 0..pixels {
        let o = i * 4;
        let dr = a[o].abs_diff(b[o]);
        let dg = a[o + 1].abs_diff(b[o + 1]);
        let db = a[o + 2].abs_diff(b[o + 2]);
        let m = dr.max(dg).max(db);
        if m == 0 {
            identical += 1;
        }
        if m > 4 {
            over_4 += 1;
        }
        if m > 16 {
            over_16 += 1;
        }
        if m > max_diff {
            max_diff = m;
        }
        sum_abs += dr as u64 + dg as u64 + db as u64;
        sum_sq += (dr as u64).pow(2) + (dg as u64).pow(2) + (db as u64).pow(2);
        if let Some(hm) = heatmap.as_mut() {
            if m == 0 {
                let g = ((a[o] as u16 + a[o + 1] as u16 + a[o + 2] as u16) / 6) as u8;
                hm[o] = g;
                hm[o + 1] = g;
                hm[o + 2] = g;
                hm[o + 3] = 255;
            } else {
                let amp = (m as u16 * 4).min(255) as u8;
                hm[o] = 255;
                hm[o + 1] = 255u8.saturating_sub(amp);
                hm[o + 2] = 0;
                hm[o + 3] = 255;
            }
        }
    }

    let channels = (pixels * 3).max(1) as f64;
    let mae_rgb = sum_abs as f64 / channels;
    let mse = sum_sq as f64 / channels;
    let rmse_rgb = mse.sqrt();
    let psnr_db = if mse <= 1e-12 {
        99.99
    } else {
        10.0 * (255.0_f64 * 255.0 / mse).log10()
    };

    if let (Some(path), Some(hm)) = (out_diff_png, heatmap) {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, nf_formats::png::encode_rgba(w, h, &hm))?;
    }

    let n = pixels.max(1) as f64;
    Ok(FrameDiff {
        width: w,
        height: h,
        mae_rgb,
        rmse_rgb,
        psnr_db,
        max_diff,
        pct_over_4: 100.0 * over_4 as f64 / n,
        pct_over_16: 100.0 * over_16 as f64 / n,
        identical_pct: 100.0 * identical as f64 / n,
    })
}

/// Decodes an 8-bit RGBA PNG written by `nf_formats::png::encode_rgba` (filter 0, stored/fixed deflate).
pub fn decode_nf_png(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>)> {
    if bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n") {
        return Err(anyhow!("not a PNG file"));
    }
    let mut pos = 8usize;
    let mut w = 0u32;
    let mut h = 0u32;
    let mut color_type = 0u8;
    let mut idat = Vec::new();
    while pos + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        let ctype = &bytes[pos + 4..pos + 8];
        let chunk = bytes
            .get(pos + 8..pos + 8 + len)
            .ok_or_else(|| anyhow!("truncated PNG chunk"))?;
        pos += 12 + len;
        match ctype {
            b"IHDR" => {
                if chunk.len() < 13 {
                    return Err(anyhow!("short IHDR"));
                }
                w = u32::from_be_bytes(chunk[0..4].try_into().unwrap());
                h = u32::from_be_bytes(chunk[4..8].try_into().unwrap());
                color_type = chunk[9];
            }
            b"IDAT" => idat.extend_from_slice(chunk),
            b"IEND" => break,
            _ => {}
        }
    }
    if color_type != 6 || w == 0 || h == 0 {
        return Err(anyhow!(
            "unsupported PNG format (expected 8-bit RGBA, got color_type={color_type}, {w}x{h})"
        ));
    }
    if idat.len() < 6 {
        return Err(anyhow!("truncated zlib stream"));
    }
    let raw = inflate_stored_or_fixed(&idat[2..idat.len() - 4])?;
    let stride = 1 + (w as usize) * 4;
    if raw.len() != stride * (h as usize) {
        return Err(anyhow!(
            "decoded scanline length mismatch: got {}, expected {}",
            raw.len(),
            stride * (h as usize)
        ));
    }
    let mut rgba = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h as usize {
        let row = &raw[y * stride..(y + 1) * stride];
        if row[0] != 0 {
            return Err(anyhow!("unsupported PNG filter {} on row {y}", row[0]));
        }
        rgba.extend_from_slice(&row[1..]);
    }
    Ok((w, h, rgba))
}

fn inflate_stored_or_fixed(d: &[u8]) -> Result<Vec<u8>> {
    struct Bits<'a> {
        d: &'a [u8],
        pos: usize,
    }
    impl Bits<'_> {
        fn bit(&mut self) -> Result<u32> {
            let b = *self
                .d
                .get(self.pos / 8)
                .ok_or_else(|| anyhow!("deflate: unexpected EOF"))?;
            let v = (b >> (self.pos % 8)) & 1;
            self.pos += 1;
            Ok(v as u32)
        }
        fn bits(&mut self, n: u32) -> Result<u32> {
            let mut v = 0;
            for k in 0..n {
                v |= self.bit()? << k;
            }
            Ok(v)
        }
        fn code(&mut self, n: u32) -> Result<u32> {
            let mut v = 0;
            for _ in 0..n {
                v = (v << 1) | self.bit()?;
            }
            Ok(v)
        }
    }

    const LEN_BASE: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [u32; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u32; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];

    let mut b = Bits { d, pos: 0 };
    let mut out = Vec::new();
    loop {
        let last = b.bits(1)?;
        match b.bits(2)? {
            0 => {
                b.pos = (b.pos + 7) & !7;
                let byte_at = b.pos / 8;
                let len = u16::from_le_bytes(
                    d.get(byte_at..byte_at + 2)
                        .ok_or_else(|| anyhow!("deflate: short stored len"))?
                        .try_into()
                        .unwrap(),
                ) as usize;
                let data_start = byte_at + 4;
                let chunk = d
                    .get(data_start..data_start + len)
                    .ok_or_else(|| anyhow!("deflate: short stored block"))?;
                out.extend_from_slice(chunk);
                b.pos = (data_start + len) * 8;
            }
            1 => loop {
                let c7 = b.code(7)?;
                let sym = if c7 <= 0x17 {
                    256 + c7
                } else {
                    let c8 = (c7 << 1) | b.bit()?;
                    if (0x30..=0xbf).contains(&c8) {
                        c8 - 0x30
                    } else if (0xc0..=0xc7).contains(&c8) {
                        280 + c8 - 0xc0
                    } else {
                        let c9 = (c8 << 1) | b.bit()?;
                        if (0x190..=0x1ff).contains(&c9) {
                            144 + c9 - 0x190
                        } else {
                            return Err(anyhow!("deflate: invalid fixed huffman symbol"));
                        }
                    }
                };
                if sym < 256 {
                    out.push(sym as u8);
                } else if sym == 256 {
                    break;
                } else {
                    let li = (sym - 257) as usize;
                    let length = LEN_BASE[li] + b.bits(LEN_EXTRA[li])? as usize;
                    let di = b.code(5)? as usize;
                    let dist = DIST_BASE[di] + b.bits(DIST_EXTRA[di])? as usize;
                    if dist == 0 || dist > out.len() {
                        return Err(anyhow!("deflate: bad backreference dist {dist}"));
                    }
                    for _ in 0..length {
                        let byte = out[out.len() - dist];
                        out.push(byte);
                    }
                }
            },
            t => return Err(anyhow!("deflate: unsupported block type {t}")),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

/// Compares two PNG files produced by `nf-view` / `nf-macos-harness` on disk.
pub fn diff_png_files(a: &Path, b: &Path, out_diff: Option<&Path>) -> Result<FrameDiff> {
    let (wa, ha, ra) = decode_nf_png(&std::fs::read(a)?)?;
    let (wb, hb, rb) = decode_nf_png(&std::fs::read(b)?)?;
    if wa != wb || ha != hb {
        return Err(anyhow!(
            "image dimensions differ: {}x{} vs {}x{}",
            wa,
            ha,
            wb,
            hb
        ));
    }
    diff_rgba(wa, ha, &ra, &rb, out_diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_rgba_and_png_roundtrip_work_with_sister_check() {
        let mut a = vec![0u8; 16 * 16 * 4];
        for i in 0..(16 * 16) {
            a[i * 4] = (i % 256) as u8;
            a[i * 4 + 1] = 80;
            a[i * 4 + 2] = 160;
            a[i * 4 + 3] = 255;
        }
        let png = nf_formats::png::encode_rgba(16, 16, &a);
        let (w, h, decoded) = decode_nf_png(&png).expect("decode roundtrip");
        assert_eq!((w, h), (16, 16));
        assert_eq!(decoded, a);

        let same = diff_rgba(16, 16, &a, &decoded, None).unwrap();
        assert_eq!(same.max_diff, 0);
        assert!((same.identical_pct - 100.0).abs() < 1e-6);

        // Sister assertion: mutating one pixel produces non-zero error and max_diff = 30.
        let mut b = a.clone();
        b[0] = b[0].wrapping_add(30);
        let changed = diff_rgba(16, 16, &a, &b, None).unwrap();
        assert_eq!(changed.max_diff, 30);
        assert!(changed.mae_rgb > 0.0);
        assert!(changed.pct_over_16 > 0.0);
    }
}
