//! Driving mission loader and 60 Hz runtime (`DRIVING/*.VIV` track `.crp` + `.ssh` + `nf_game::driving`).

use crate::scene::{Attachment, CharVertex, Draw, Layer, Material, Scene, TextureImage, Vertex};
use anyhow::{Context, Result};
use nf_formats::anim::Mat4;
use nf_formats::crp::Crp;
use nf_formats::crp_mesh::{CollisionTri, MapTrigger, TrackModule};
use nf_formats::disc;
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;
use nf_formats::ssh::Ssh;
use nf_game::driving::{
    ai::AiDriver,
    bounds::ZONES,
    mission::{Mission, MissionState, SimSnapshot},
    physics::Tuning,
    road::RoadNet,
    suspension::{
        axle_of, rotor_wheelspin, watercraft_handbrake_drag, watercraft_longitudinal_thrust,
        SusParams, SusWheel,
    },
    Car, CarInput, CarParams, FlatWorld, Rigid, DT,
};
use std::collections::HashMap;

/// Map the front end's `0x09xxxxxx` driving level code to its `/DRIVING/<VIV>.VIV` stem.
pub fn driving_viv_for_level(level: u32) -> Option<&'static str> {
    match level {
        0x0900_0001 => Some("MIS01"),
        0x0900_0005 => Some("MIS3"),
        0x0900_0006 => Some("MIS4"),
        0x0900_0002 => Some("MIS11"),
        0x0900_0003 => Some("MIS13A"),
        0x0900_0004 => Some("MIS13B"),
        0x0900_0007 => Some("MIS13C"),
        0x0900_0008 => Some("RACE"),
        _ => None,
    }
}

fn player_car_stem(viv_upper: &str) -> &'static str {
    match viv_upper {
        "MIS11" => "vanquishsub",
        "MIS13A" | "MIS13B" => "ultralight",
        "MIS13C" => "patrolboat",
        _ => "vanquish",
    }
}

fn drive_index_for_viv(viv_upper: &str) -> u8 {
    match viv_upper {
        "MIS01" => 0,
        "MIS3" => 1,
        "MIS4" => 2,
        "MIS11" => 3,
        "MIS13A" => 4,
        "MIS13B" => 5,
        "MIS13C" => 6,
        "RACE" => 7,
        _ => 0,
    }
}

/// Spatial XZ grid over `.crp` `CDat` collision triangles for O(1) ground raycasts.
pub struct CollisionGrid {
    tris: Vec<CollisionTri>,
    cells: HashMap<(i32, i32), Vec<usize>>,
}

impl CollisionGrid {
    const CELL: f32 = 16.0;

    pub fn build(raw_tris: Vec<CollisionTri>) -> Self {
        let mut tris = Vec::new();
        let mut cells: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for t in raw_tris {
            let e1 = [t.v[1][0] - t.v[0][0], t.v[1][1] - t.v[0][1], t.v[1][2] - t.v[0][2]];
            let e2 = [t.v[2][0] - t.v[0][0], t.v[2][1] - t.v[0][1], t.v[2][2] - t.v[0][2]];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len < 1e-5 || (ny.abs() / len) < 0.25 {
                continue;
            }
            let idx = tris.len();
            let min_x = t.v[0][0].min(t.v[1][0]).min(t.v[2][0]);
            let max_x = t.v[0][0].max(t.v[1][0]).max(t.v[2][0]);
            let min_z = t.v[0][2].min(t.v[1][2]).min(t.v[2][2]);
            let max_z = t.v[0][2].max(t.v[1][2]).max(t.v[2][2]);
            let cx0 = (min_x / Self::CELL).floor() as i32;
            let cx1 = (max_x / Self::CELL).floor() as i32;
            let cz0 = (min_z / Self::CELL).floor() as i32;
            let cz1 = (max_z / Self::CELL).floor() as i32;
            for cx in cx0..=cx1 {
                for cz in cz0..=cz1 {
                    cells.entry((cx, cz)).or_default().push(idx);
                }
            }
            tris.push(t);
        }
        Self { tris, cells }
    }

    /// Vertical downward raycast at `(x, cur_y + 2.5, z)` returning the highest walkable surface
    /// `y` within `[cur_y - 12.0, cur_y + 3.0]`.
    pub fn ground_y(&self, x: f32, cur_y: f32, z: f32) -> Option<f32> {
        let cx = (x / Self::CELL).floor() as i32;
        let cz = (z / Self::CELL).floor() as i32;
        let list = self.cells.get(&(cx, cz))?;
        let mut best: Option<f32> = None;
        for &ti in list {
            let [a, b, c] = self.tris[ti].v;
            let denom = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2]);
            if denom.abs() < 1e-6 {
                continue;
            }
            let u = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / denom;
            let v = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / denom;
            let w = 1.0 - u - v;
            if u < -1e-4 || v < -1e-4 || w < -1e-4 {
                continue;
            }
            let y = u * a[1] + v * b[1] + w * c[1];
            if y >= cur_y - 12.0 && y <= cur_y + 3.0 {
                best = Some(match best {
                    Some(prev) => prev.max(y),
                    None => y,
                });
            }
        }
        best
    }
}

/// Live driving mission simulation state.
pub struct DrivingState {
    pub viv: String,
    pub car: Car,
    pub world: FlatWorld,
    pub rigid: Rigid,
    pub crp_bytes: Vec<u8>,
    pub road_nodes: Vec<[f32; 3]>,
    pub collision: CollisionGrid,
    pub ai_cars: Vec<(AiDriver, u32)>,
    pub player_attach_slot: Option<u32>,
    pub mission: Mission,
    pub mission_state: MissionState,
    pub triggers: Vec<MapTrigger>,
    pub triggered: Vec<bool>,
    pub progress: u32,
    pub rotor_spin: f32,
    pub wheels: [SusWheel; 4],
    pub sus_params: SusParams,
    pub ticks: u32,
}

const IDENT_MAT: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn yaw_pos_mat(pos: [f32; 3], yaw: f32) -> Mat4 {
    let (s, c) = yaw.sin_cos();
    [
        [c, 0.0, -s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [s, 0.0, c, 0.0],
        [pos[0], pos[1], pos[2], 1.0],
    ]
}

impl DrivingState {
    pub fn step(&mut self, throttle: f32, brake: f32, steer: f32, handbrake: bool) {
        self.ticks = self.ticks.wrapping_add(1);
        let input = CarInput {
            throttle: throttle.clamp(0.0, 1.0),
            brake: brake.clamp(0.0, 1.0),
            steer: steer.clamp(-1.0, 1.0),
            handbrake,
        };
        self.car.step(&input, &self.world, &self.rigid, DT);

        // Apply world-boundary sheet containment (G-056, DRIVING.ELF unk_331A60).
        let p4 = [self.car.pos[0], self.car.pos[1], self.car.pos[2], 1.0];
        if let Some(zi) = nf_game::driving::bounds::ZoneQuad::first_hit(p4) {
            let z = &ZONES[zi];
            let vn = self.car.vel[0] * z.normal[0]
                + self.car.vel[1] * z.normal[1]
                + self.car.vel[2] * z.normal[2];
            if vn < 0.0 {
                let f = z.boundary_force(vn, [0.0; 4], 1500.0, DT, 1.0);
                self.car.vel[0] += f[0] * DT;
                self.car.vel[2] += f[2] * DT;
            }
            let d = z.signed_dist(p4);
            if d < z.thickness * 0.25 {
                let push = z.thickness * 0.25 - d;
                self.car.pos[0] += z.normal[0] * push;
                self.car.pos[2] += z.normal[2] * push;
            }
        }

        // Ground height from 4-wheel CDat collision raycasts + SusWheel suspension (fallback to RoadNet).
        let (s, c) = self.car.yaw.sin_cos();
        let wheel_offsets = [
            [-0.85f32, self.car.params.wheel_zf_offset],
            [0.85, self.car.params.wheel_zf_offset],
            [-0.85, self.car.params.wheel_zr_offset],
            [0.85, self.car.params.wheel_zr_offset],
        ];
        let mut hit_sum = 0.0f32;
        let mut hit_cnt = 0u32;
        for (wi, &[lx, lz]) in wheel_offsets.iter().enumerate() {
            let wx = self.car.pos[0] + lx * c + lz * s;
            let wz = self.car.pos[2] - lx * s + lz * c;
            let hy = self.collision.ground_y(wx, self.car.pos[1], wz);
            let ray_t = hy.map(|y| (self.car.pos[1] - y).max(0.0));
            let _ = self.wheels[wi].step(0.0, ray_t, &self.sus_params, axle_of(wi));
            self.wheels[wi].end_tick();
            if let Some(y) = hy {
                hit_sum += y;
                hit_cnt += 1;
            }
        }
        if hit_cnt > 0 {
            self.car.pos[1] = hit_sum / hit_cnt as f32;
        } else if let Some(hy) = self.collision.ground_y(self.car.pos[0], self.car.pos[1], self.car.pos[2]) {
            self.car.pos[1] = hy;
        } else if !self.road_nodes.is_empty() {
            let p = self.car.pos;
            let mut best_d2 = f32::MAX;
            let mut best_y = p[1];
            for n in &self.road_nodes {
                let dx = n[0] - p[0];
                let dz = n[2] - p[2];
                let d2 = dx * dx + dz * dz;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best_y = n[1];
                }
            }
            self.car.pos[1] = best_y;
        }

        // Step AI traffic cars.
        if !self.ai_cars.is_empty() {
            if let Ok(crp) = Crp::parse(&self.crp_bytes) {
                if let Ok(rn) = RoadNet::parse_crp(&crp) {
                    for (drv, _) in &mut self.ai_cars {
                        let _ = drv.step(&rn, DT);
                        if let Some(hy) = self.collision.ground_y(drv.car.pos[0], drv.car.pos[1], drv.car.pos[2]) {
                            drv.car.pos[1] = hy;
                        }
                    }
                }
            }
        }

        // Watercraft (`MIS11`, `MIS13C`) and aerial rotor (`MIS13A`, `MIS13B`) passes.
        let mut speed = (self.car.vel[0].powi(2) + self.car.vel[2].powi(2)).sqrt();
        match self.viv.as_str() {
            "MIS11" | "MIS13C" => {
                let thrust = watercraft_longitudinal_thrust(
                    input.throttle,
                    self.car.params.maxacc,
                    input.brake,
                    self.car.params.maxbrake,
                );
                let drag = watercraft_handbrake_drag(
                    speed,
                    self.car.params.handbrake_force,
                    input.handbrake,
                );
                let (sy, cy) = self.car.yaw.sin_cos();
                self.car.vel[0] += sy * thrust * DT * 0.25;
                self.car.vel[2] += cy * thrust * DT * 0.25;
                if drag > 0.0 && speed > 1e-4 {
                    let damp = (1.0 - (drag * DT / speed).min(1.0)).max(0.0);
                    self.car.vel[0] *= damp;
                    self.car.vel[2] *= damp;
                }
                speed = (self.car.vel[0].powi(2) + self.car.vel[2].powi(2)).sqrt();
            }
            "MIS13A" | "MIS13B" => {
                self.rotor_spin = rotor_wheelspin(speed, self.car.params.wheel_radius, DT);
            }
            _ => {}
        }

        // Checkpoint proximity detection against `Map` `Trgr` volumes.
        for (i, t) in self.triggers.iter().enumerate() {
            if self.triggered.get(i).copied().unwrap_or(true) {
                continue;
            }
            let dx = self.car.pos[0] - t.pos[0];
            let dy = self.car.pos[1] - t.pos[1];
            let dz = self.car.pos[2] - t.pos[2];
            let r = t.radius.max(4.0);
            if dx * dx + dy * dy + dz * dz <= r * r {
                self.triggered[i] = true;
                self.progress = self.progress.saturating_add(1);
            }
        }

        // Step mission rules & timers.
        let snap = SimSnapshot {
            health: 100.0,
            damage: 0.0,
            speed,
            pos: self.car.pos,
            died: None,
            collided: false,
            progress: self.progress,
            difficulty: 1,
        };
        let _events = self.mission.step(DT, &snap);
        self.mission_state = self.mission.state;
    }

    /// Write player and AI car world transforms into `Scene` attachment palette slots.
    pub fn apply_to_palette(&self, attach_base: usize, pal: &mut [Mat4]) {
        if let Some(slot) = self.player_attach_slot {
            if let Some(m) = pal.get_mut(attach_base + slot as usize) {
                *m = yaw_pos_mat(self.car.pos, self.car.yaw);
            }
        }
        for (drv, slot) in &self.ai_cars {
            if let Some(m) = pal.get_mut(attach_base + *slot as usize) {
                *m = yaw_pos_mat(drv.car.pos, drv.car.yaw);
            }
        }
    }

    /// Third-person chase camera behind the player car: `(pos, yaw, pitch)`.
    pub fn chase_camera(&self) -> ([f32; 3], f32, f32) {
        let yaw = self.car.yaw;
        let fwd = [yaw.sin(), 0.0, yaw.cos()];
        let cam_pos = [
            self.car.pos[0] - fwd[0] * 6.5,
            self.car.pos[1] + 2.4,
            self.car.pos[2] - fwd[2] * 6.5,
        ];
        (cam_pos, yaw, -0.14)
    }
}

fn maybe_decompress(raw: &[u8]) -> Result<Vec<u8>> {
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb {
        Ok(refpack::decompress(raw)?)
    } else {
        Ok(raw.to_vec())
    }
}

/// Triangulates a PS2 triangle strip into `indices`, skipping strip-break jumps
/// whose edges exceed `sqrt(max_edge_sq)`.
fn push_strip_tris(
    positions: &[[f32; 3]],
    base_v: u32,
    max_edge_sq: f32,
    indices: &mut Vec<u32>,
) {
    let dist_sq = |a: [f32; 3], b: [f32; 3]| -> f32 {
        (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
    };
    for k in 0..positions.len().saturating_sub(2) {
        let (p0, p1, p2) = (positions[k], positions[k + 1], positions[k + 2]);
        if dist_sq(p0, p1) > max_edge_sq
            || dist_sq(p1, p2) > max_edge_sq
            || dist_sq(p0, p2) > max_edge_sq
        {
            continue;
        }
        let (i0, i1, i2) = if k % 2 == 0 {
            (base_v + k as u32, base_v + k as u32 + 1, base_v + k as u32 + 2)
        } else {
            (base_v + k as u32 + 1, base_v + k as u32, base_v + k as u32 + 2)
        };
        indices.push(i0);
        indices.push(i1);
        indices.push(i2);
    }
}

fn load_car_into_scene(
    iso: &IsoImage,
    viv_base: u64,
    car_stem: &str,
    crp_off_sz: (u32, u32),
    ssh_off_sz: Option<(u32, u32)>,
    vertices: &mut Vec<Vertex>,
    char_verts: &mut Vec<CharVertex>,
    indices: &mut Vec<u32>,
    materials: &mut Vec<Material>,
    textures: &mut Vec<TextureImage>,
    attachments: &mut Vec<Attachment>,
) -> Option<u32> {
    let crp_raw = iso
        .read_bytes(viv_base + crp_off_sz.0 as u64, crp_off_sz.1 as u64)
        .ok()?;
    let crp_bytes = maybe_decompress(&crp_raw).ok()?;
    let car_crp = Crp::parse(&crp_bytes).ok()?;
    let car_mod = TrackModule::load(&car_crp).ok()?;

    let mut car_tex_by_name: HashMap<String, usize> = HashMap::new();
    if let Some((ssh_off, ssh_sz)) = ssh_off_sz {
        if let Ok(ssh_raw) = iso.read_bytes(viv_base + ssh_off as u64, ssh_sz as u64) {
            if let Ok(ssh_bytes) = maybe_decompress(&ssh_raw) {
                if let Ok(ssh) = Ssh::parse(&ssh_bytes) {
                    for t in ssh.textures {
                        let idx = textures.len();
                        car_tex_by_name.insert(t.name.to_ascii_lowercase(), idx);
                        textures.push(TextureImage {
                            key: (0, idx as u32),
                            width: t.width,
                            height: t.height,
                            rgba: t.rgba,
                            more: Vec::new(),
                            rate: 1,
                        });
                    }
                }
            }
        }
    }

    let strips = car_mod.extract_car_mesh(&car_crp, car_stem);
    if strips.is_empty() {
        return None;
    }

    let slot = attachments.len() as u32;
    attachments.push(Attachment {
        character: 0,
        bone: 0,
        local: IDENT_MAT,
        flash: None,
        tracer: None,
        projectile: None,
        dropped: false,
        script: None,
        aims_light: None,
        gas: None,
        effect: None,
        silencer: false,
        quiver_bolt: None,
        swap_piece: false,
        watch_hand: None,
        pda_digit: None,
    });

    for strip in strips {
        if strip.positions.len() < 3 {
            continue;
        }
        let base_v = vertices.len() as u32;
        let first_i = indices.len() as u32;
        for (k, &p) in strip.positions.iter().enumerate() {
            let uv = strip.uvs.get(k).copied().unwrap_or([0.0, 0.0]);
            let n = strip.colors.get(k).copied().unwrap_or([0, 127, 0]);
            vertices.push(Vertex {
                pos: p,
                uv,
                rgba: [255, 255, 255, 255],
                nrm: [n[0] as i8, n[1] as i8, n[2] as i8, 0],
            });
            char_verts.push(CharVertex {
                character: 0,
                pos: p,
                weight: 1.0,
                slots: [0, 0],
                attachment: Some(slot),
                searchlight: None,
                door: None,
                pickup: None,
                morph: -1,
            });
        }
        push_strip_tris(&strip.positions, base_v, 3.5 * 3.5, indices);
        let index_count = indices.len() as u32 - first_i;
        if index_count == 0 {
            continue;
        }
        let tex_key = strip
            .texture
            .as_ref()
            .and_then(|t| car_tex_by_name.get(&t.to_ascii_lowercase()).copied())
            .map(|idx| (0usize, idx as u32));
        materials.push(Material {
            layer: Layer::Character,
            draw: Draw {
                pass: 1,
                blend: 0,
                zwrite: true,
                atest: 0,
                fog: 0,
            },
            centre: [0.0, 0.0, 0.0],
            radius: 10_000.0,
            cel: None,
            chain: u32::MAX,
            node: u32::MAX,
            key: tex_key,
            lflags: 0,
            skin: None,
            sky_slot: None,
            first_index: first_i,
            index_count,
        });
    }
    Some(slot)
}

/// Load a driving track `.crp` + `.ssh` + player/AI car models + mission rules from `/DRIVING/<viv>.VIV`.
pub fn load_driving_scene(iso_path: &str, viv_name: &str) -> Result<(Scene, DrivingState)> {
    let viv_upper = viv_name
        .trim_end_matches(".VIV")
        .trim_end_matches(".viv")
        .to_ascii_uppercase();
    let iso = IsoImage::open(iso_path)?;
    let entry = iso
        .find(&format!("/DRIVING/{viv_upper}.VIV"))
        .with_context(|| format!("missing /DRIVING/{viv_upper}.VIV"))?;
    let bigf = disc::read_bigf(&iso, entry)?;
    let viv_base = entry.lba as u64 * 2048;

    let crp_entry = bigf
        .entries
        .iter()
        .find(|x| {
            x.name.to_ascii_lowercase().contains("data\\track\\")
                && x.name.to_ascii_lowercase().ends_with(".crp")
        })
        .with_context(|| format!("no track .crp in {viv_upper}.VIV"))?;
    let crp_raw = iso.read_bytes(viv_base + crp_entry.offset as u64, crp_entry.size as u64)?;
    let crp_bytes = maybe_decompress(&crp_raw)?;
    let crp = Crp::parse(&crp_bytes)?;

    let mut textures = Vec::new();
    let mut tex_by_name: HashMap<String, usize> = HashMap::new();
    if let Some(ssh_entry) = bigf.entries.iter().find(|x| {
        x.name.to_ascii_lowercase().contains("data\\track\\")
            && x.name.to_ascii_lowercase().ends_with(".ssh")
    }) {
        let ssh_raw = iso.read_bytes(viv_base + ssh_entry.offset as u64, ssh_entry.size as u64)?;
        let ssh_bytes = maybe_decompress(&ssh_raw)?;
        if let Ok(ssh) = Ssh::parse(&ssh_bytes) {
            for (idx, t) in ssh.textures.into_iter().enumerate() {
                tex_by_name.insert(t.name.to_ascii_lowercase(), idx);
                textures.push(TextureImage {
                    key: (0, idx as u32),
                    width: t.width,
                    height: t.height,
                    rgba: t.rgba,
                    more: Vec::new(),
                    rate: 1,
                });
            }
        }
    }

    let module = TrackModule::load(&crp)?;
    let articles = crp.track_articles()?;

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut materials: Vec<Material> = Vec::new();
    let mut bmin = [f32::MAX; 3];
    let mut bmax = [f32::MIN; 3];

    let mut art_strips: HashMap<String, Vec<nf_formats::crp_mesh::MeshStrip>> = HashMap::new();
    for art in &articles {
        let strips = module.extract_article_mesh(&art.name, art.bbox_min, art.bbox_max);
        art_strips.insert(art.name.clone(), strips);
    }

    let push_strip = |strip: &nf_formats::crp_mesh::MeshStrip,
                      inst: Option<&nf_formats::crp_mesh::MapInstance>,
                      vertices: &mut Vec<Vertex>,
                      indices: &mut Vec<u32>,
                      materials: &mut Vec<Material>,
                      bmin: &mut [f32; 3],
                      bmax: &mut [f32; 3]| {
        if strip.positions.len() < 3 {
            return;
        }
        let base_v = vertices.len() as u32;
        let first_i = indices.len() as u32;
        let mut smin = [f32::MAX; 3];
        let mut smax = [f32::MIN; 3];
        for (k, &lp) in strip.positions.iter().enumerate() {
            let p = inst.map_or(lp, |m| m.transform(lp));
            for a in 0..3 {
                bmin[a] = bmin[a].min(p[a]);
                bmax[a] = bmax[a].max(p[a]);
                smin[a] = smin[a].min(p[a]);
                smax[a] = smax[a].max(p[a]);
            }
            let uv = strip.uvs.get(k).copied().unwrap_or([0.0, 0.0]);
            let n = strip.colors.get(k).copied().unwrap_or([0, 127, 0]);
            vertices.push(Vertex {
                pos: p,
                uv,
                rgba: [255, 255, 255, 255],
                nrm: [n[0] as i8, n[1] as i8, n[2] as i8, 0],
            });
        }
        push_strip_tris(&strip.positions, base_v, 40.0 * 40.0, indices);
        let index_count = indices.len() as u32 - first_i;
        if index_count == 0 {
            return;
        }
        let centre = [
            0.5 * (smin[0] + smax[0]),
            0.5 * (smin[1] + smax[1]),
            0.5 * (smin[2] + smax[2]),
        ];
        let radius = ((smax[0] - centre[0]).powi(2)
            + (smax[1] - centre[1]).powi(2)
            + (smax[2] - centre[2]).powi(2))
        .sqrt()
        .max(1.0);
        let tex_key = strip
            .texture
            .as_ref()
            .and_then(|t| tex_by_name.get(&t.to_ascii_lowercase()).copied())
            .map(|idx| (0usize, idx as u32));

        materials.push(Material {
            layer: Layer::World,
            draw: Draw {
                pass: 1,
                blend: 0,
                zwrite: true,
                atest: 0,
                fog: 0,
            },
            centre,
            radius,
            cel: None,
            chain: u32::MAX,
            node: u32::MAX,
            key: tex_key,
            lflags: 0,
            skin: None,
            sky_slot: None,
            first_index: first_i,
            index_count,
        });
    };

    let map_instances = crp.extract_map_instances().unwrap_or_default();
    if !map_instances.is_empty() {
        let mut seen_inst = std::collections::HashSet::new();
        for inst in &map_instances {
            if inst.article.to_ascii_lowercase().starts_with("sky") || inst.inv_scale2 > 10.0 {
                continue;
            }
            let key = (
                inst.article.as_str(),
                (inst.tr[0] * 10.0).round() as i32,
                (inst.tr[1] * 10.0).round() as i32,
                (inst.tr[2] * 10.0).round() as i32,
            );
            if !seen_inst.insert(key) {
                continue;
            }
            if let Some(strips) = art_strips.get(&inst.article) {
                for strip in strips {
                    push_strip(strip, Some(inst), &mut vertices, &mut indices, &mut materials, &mut bmin, &mut bmax);
                }
            }
        }
    } else {
        for art in &articles {
            if let Some(strips) = art_strips.get(&art.name) {
                for strip in strips {
                    push_strip(strip, None, &mut vertices, &mut indices, &mut materials, &mut bmin, &mut bmax);
                }
            }
        }
    }

    // Build CDat collision grid and emit upward-facing road/street terrain triangles
    // (GeoPrim road strips are runtime-generated in eaglrm.o, so CDat provides the full
    // drivable street/bridge/ramp mesh).
    let col_tris = crp.extract_collision_tris().unwrap_or_default();
    {
        let first_i = indices.len() as u32;
        for t in &col_tris {
            let e1 = [t.v[1][0] - t.v[0][0], t.v[1][1] - t.v[0][1], t.v[1][2] - t.v[0][2]];
            let e2 = [t.v[2][0] - t.v[0][0], t.v[2][1] - t.v[0][1], t.v[2][2] - t.v[0][2]];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len < 1e-5 || (ny.abs() / len) < 0.45 {
                continue;
            }
            let shade = if (ny.abs() / len) > 0.92 {
                [68, 72, 78, 255]
            } else {
                [96, 100, 106, 255]
            };
            let bv = vertices.len() as u32;
            for p in t.v {
                let wp = [p[0], p[1] - 0.03, p[2]];
                for a in 0..3 {
                    bmin[a] = bmin[a].min(wp[a]);
                    bmax[a] = bmax[a].max(wp[a]);
                }
                vertices.push(Vertex {
                    pos: wp,
                    uv: [0.0, 0.0],
                    rgba: shade,
                    nrm: [0, 127, 0, 0],
                });
            }
            indices.push(bv);
            indices.push(bv + 1);
            indices.push(bv + 2);
        }
        let index_count = indices.len() as u32 - first_i;
        if index_count > 0 {
            let centre = [
                0.5 * (bmin[0] + bmax[0]),
                0.5 * (bmin[1] + bmax[1]),
                0.5 * (bmin[2] + bmax[2]),
            ];
            let radius = ((bmax[0] - centre[0]).powi(2)
                + (bmax[1] - centre[1]).powi(2)
                + (bmax[2] - centre[2]).powi(2))
            .sqrt()
            .max(1000.0);
            materials.push(Material {
                layer: Layer::World,
                draw: Draw {
                    pass: 1,
                    blend: 0,
                    zwrite: true,
                    atest: 0,
                    fog: 0,
                },
                centre,
                radius,
                cel: None,
                chain: u32::MAX,
                node: u32::MAX,
                key: None,
                lflags: 0,
                skin: None,
                sky_slot: None,
                first_index: first_i,
                index_count,
            });
        }
    }

    if bmin[0] > bmax[0] {
        bmin = [-100.0; 3];
        bmax = [100.0; 3];
    }

    let collision = CollisionGrid::build(col_tris);

    // Parse RoadNet & compute player spawn.
    let mut road_nodes = Vec::new();
    let mut start_pos = [
        0.5 * (bmin[0] + bmax[0]),
        bmin[1] + 2.0,
        0.5 * (bmin[2] + bmax[2]),
    ];
    let mut start_yaw = 0.0f32;
    let rn_opt = RoadNet::parse_crp(&crp).ok();
    if let Some(rn) = &rn_opt {
        road_nodes = rn.nodes.iter().map(|n| n.pos()).collect();
        if let Some(p0) = road_nodes.first().copied() {
            start_pos = p0;
            if let Some(p1) = road_nodes.get(1).copied() {
                start_yaw = (p1[0] - p0[0]).atan2(p1[2] - p0[2]);
            }
        }
    }

    let find_car_entry = |stem: &str| -> Option<((u32, u32), Option<(u32, u32)>)> {
        let target_crp = format!("data\\car\\model\\{stem}.crp");
        let crp_e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&target_crp))?;
        let target_ssh = format!("data\\car\\model\\{stem}.ssh");
        let ssh_e = bigf.entries.iter().find(|x| x.name.eq_ignore_ascii_case(&target_ssh));
        Some(((crp_e.offset, crp_e.size), ssh_e.map(|s| (s.offset, s.size))))
    };

    // Load player & AI car 3D meshes into Scene dynamic attachment slots.
    let char_vertex_start = vertices.len();
    let mut char_verts = Vec::new();
    let mut attachments = Vec::new();
    let p_stem = player_car_stem(&viv_upper);
    let player_attach_slot = find_car_entry(p_stem).and_then(|(c, s)| {
        load_car_into_scene(
            &iso,
            viv_base,
            p_stem,
            c,
            s,
            &mut vertices,
            &mut char_verts,
            &mut indices,
            &mut materials,
            &mut textures,
            &mut attachments,
        )
    });

    // Load CarParams (.atr) and Rigid (.tun).
    let car_params = bigf
        .entries
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case(&format!("data\\sim\\attrib\\pvehicle\\{p_stem}.atr")))
        .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| CarParams::parse_atr(&s))
        .unwrap_or_else(CarParams::vanquish);

    let rigid = bigf
        .entries
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case("data\\tuning\\Physics\\Rigid\\default.tun"))
        .and_then(|e| iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .map(|s| Rigid::from_tun(&Tuning::parse(&s)))
        .unwrap_or_else(Rigid::game_defaults);

    // Spawn up to 4 AI traffic cars on RoadNet segments.
    let mut ai_cars = Vec::new();
    if let Some(rn) = &rn_opt {
        if rn.segs.len() >= 4 {
            let traffic_stem = bigf
                .entries
                .iter()
                .filter_map(|e| {
                    let low = e.name.to_ascii_lowercase();
                    let rest = low.strip_prefix("data\\car\\model\\")?.strip_suffix(".crp")?;
                    if rest != p_stem { Some(rest.to_string()) } else { None }
                })
                .next();
            if let Some(t_stem) = traffic_stem {
                if let Some((c, s)) = find_car_entry(&t_stem) {
                    let step_seg = (rn.segs.len() / 5).max(1);
                    for k in 1..=4 {
                        let seg = (k * step_seg) % rn.segs.len();
                        if let Ok(drv) = AiDriver::on_segment(car_params.clone(), rn, seg, 0, k as i8) {
                            if let Some(slot) = load_car_into_scene(
                                &iso,
                                viv_base,
                                &t_stem,
                                c,
                                s,
                                &mut vertices,
                                &mut char_verts,
                                &mut indices,
                                &mut materials,
                                &mut textures,
                                &mut attachments,
                            ) {
                                ai_cars.push((drv, slot));
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse mission Rule / MSet from track .crp.
    let rule_bytes = crp
        .find(0x5275_6c65)
        .ok()
        .flatten()
        .or_else(|| crp.pool_find(0x5275_6c65).ok().flatten())
        .and_then(|r| crp.payload(&r).ok())
        .unwrap_or(&[]);
    let mset_bytes = crp
        .find(0x4d53_6574)
        .ok()
        .flatten()
        .or_else(|| crp.pool_find(0x4d53_6574).ok().flatten())
        .and_then(|r| crp.payload(&r).ok());
    let mission = Mission::load(rule_bytes, mset_bytes, drive_index_for_viv(&viv_upper));
    let mission_state = mission.state;

    // Extract `Map` `Trgr` checkpoint triggers and emit subtle translucent ground beacon rings
    // (spaced >=35m apart, capped at 16, using normal alpha blend so they never blow out the scene).
    let triggers = crp.extract_map_triggers().unwrap_or_default();
    let triggered = vec![false; triggers.len()];
    {
        const SEGMENTS: usize = 12;
        const BEACON_H: f32 = 0.8;
        let first_i = indices.len() as u32;
        let mut placed_beacons: Vec<[f32; 3]> = Vec::new();
        for t in &triggers {
            if !(2.0..=12.0).contains(&t.radius) {
                continue;
            }
            if placed_beacons.iter().any(|p| {
                let dx = p[0] - t.pos[0];
                let dz = p[2] - t.pos[2];
                dx * dx + dz * dz < 35.0 * 35.0
            }) {
                continue;
            }
            placed_beacons.push(t.pos);
            let r = t.radius.clamp(2.5, 6.0);
            let bv = vertices.len() as u32;
            for s in 0..SEGMENTS {
                let ang = (s as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
                let (sa, ca) = ang.sin_cos();
                let x = t.pos[0] + ca * r;
                let z = t.pos[2] + sa * r;
                vertices.push(Vertex {
                    pos: [x, t.pos[1], z],
                    uv: [0.0, 0.0],
                    rgba: [40, 220, 255, 96],
                    nrm: [0, 127, 0, 0],
                });
                vertices.push(Vertex {
                    pos: [x, t.pos[1] + BEACON_H, z],
                    uv: [0.0, 1.0],
                    rgba: [255, 220, 60, 24],
                    nrm: [0, 127, 0, 0],
                });
            }
            for s in 0..SEGMENTS {
                let s1 = (s + 1) % SEGMENTS;
                let b0 = bv + (s * 2) as u32;
                let t0 = b0 + 1;
                let b1 = bv + (s1 * 2) as u32;
                let t1 = b1 + 1;
                indices.extend_from_slice(&[b0, b1, t0, t0, b1, t1]);
            }
            if placed_beacons.len() >= 16 {
                break;
            }
        }
        let index_count = indices.len() as u32 - first_i;
        if index_count > 0 {
            let centre = [
                0.5 * (bmin[0] + bmax[0]),
                0.5 * (bmin[1] + bmax[1]),
                0.5 * (bmin[2] + bmax[2]),
            ];
            let radius = ((bmax[0] - centre[0]).powi(2)
                + (bmax[1] - centre[1]).powi(2)
                + (bmax[2] - centre[2]).powi(2))
            .sqrt()
            .max(1000.0);
            materials.push(Material {
                layer: Layer::World,
                draw: Draw {
                    pass: 2,
                    blend: 2,
                    zwrite: false,
                    atest: 0,
                    fog: 0,
                },
                centre,
                radius,
                cel: None,
                chain: u32::MAX,
                node: u32::MAX,
                key: None,
                lflags: 0,
                skin: None,
                sky_slot: None,
                first_index: first_i,
                index_count,
            });
        }
    }

    let stats = format!(
        "{viv_upper}: {} articles, {} verts, {} tris, {} textures, {} col tris",
        articles.len(),
        vertices.len(),
        indices.len() / 3,
        textures.len(),
        collision.tris.len()
    );
    let mut scene = Scene::from_static_world(
        vertices,
        indices,
        materials,
        textures,
        (bmin, bmax),
        Some((start_pos, start_yaw)),
        stats,
    );
    scene.char_vertex_start = char_vertex_start;
    scene.char_verts = char_verts;
    scene.attachments = attachments;

    let car = Car::new(car_params, start_pos, start_yaw);
    let state = DrivingState {
        viv: viv_upper,
        car,
        world: FlatWorld::paved(),
        rigid,
        crp_bytes,
        road_nodes,
        collision,
        ai_cars,
        player_attach_slot,
        mission,
        mission_state,
        triggers,
        triggered,
        progress: 0,
        rotor_spin: 0.0,
        wheels: [SusWheel::default(); 4],
        sus_params: SusParams::VANQUISH,
        ticks: 0,
    };
    Ok((scene, state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_all_driving_mission_codes() {
        assert_eq!(driving_viv_for_level(0x0900_0001), Some("MIS01"));
        assert_eq!(driving_viv_for_level(0x0900_0005), Some("MIS3"));
        assert_eq!(driving_viv_for_level(0x0900_0006), Some("MIS4"));
        assert_eq!(driving_viv_for_level(0x0900_0002), Some("MIS11"));
        assert_eq!(driving_viv_for_level(0x0900_0003), Some("MIS13A"));
        assert_eq!(driving_viv_for_level(0x0900_0008), Some("RACE"));
    }

    #[test]
    fn collision_grid_raycasts_sloped_triangle() {
        let tri = CollisionTri {
            v: [[0.0, 0.0, 0.0], [10.0, 5.0, 0.0], [0.0, 0.0, 10.0]],
            flags: 1,
        };
        let grid = CollisionGrid::build(vec![tri]);
        let y = grid.ground_y(5.0, 3.0, 2.0).expect("hit sloped tri");
        assert!((y - 2.5).abs() < 1e-4);
    }

    #[test]
    fn checkpoints_and_watercraft_rotor_passes_step_cleanly() {
        let mut state = DrivingState {
            viv: "MIS11".into(),
            car: Car::new(CarParams::vanquish(), [0.0, 0.0, 0.0], 0.0),
            world: FlatWorld::paved(),
            rigid: Rigid::game_defaults(),
            crp_bytes: Vec::new(),
            road_nodes: Vec::new(),
            collision: CollisionGrid::build(Vec::new()),
            ai_cars: Vec::new(),
            player_attach_slot: None,
            mission: Mission::load(&[], None, 3),
            mission_state: MissionState::Playing,
            triggers: vec![MapTrigger {
                pos: [0.0, 0.0, 1.0],
                radius: 4.0,
                flags: 0x0407_0101,
                id: 0x656c_0001,
            }],
            triggered: vec![false],
            progress: 0,
            rotor_spin: 0.0,
            wheels: [SusWheel::default(); 4],
            sus_params: SusParams::VANQUISH,
            ticks: 0,
        };
        state.step(1.0, 0.0, 0.0, false);
        assert_eq!(state.progress, 1);
        assert!(state.triggered[0]);
        assert!(state.car.vel[2] > 0.0);

        state.viv = "MIS13A".into();
        state.step(1.0, 0.0, 0.0, false);
        assert!(state.rotor_spin > 0.0);
    }
}
