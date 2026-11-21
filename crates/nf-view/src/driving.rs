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
        // RACE: the archive's only player vehicle file is cobra_player.atr, whose
        // RENDER_FILENAME.s is paradis_car.crp [D RACE.VIV]; RACE carries no vanquish.crp.
        "RACE" => "paradis_car",
        // MIS3 carries no vanquish.crp either; supersnow (8 colours, a bullet-streak weapon type) is
        // taken as the player's over small_snowmobile (1 colour, no weapon) [S: the player vehicle
        // pick is not traced].
        "MIS3" => "supersnow",
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
        let y_before = self.car.pos[1];
        self.car.step(&input, &self.world, &self.rigid, DT);
        // `Car::step` floors the height at 0 (its flat-ground model); on tracks that lie below 0
        // (MIS13C's river at y ~-587) that snaps the car up out of the world, out of the ground
        // follow's -12..+3 window. Keep the height the car had; the collision follow below sets it.
        if y_before < 0.0 && self.car.pos[1] == 0.0 { self.car.pos[1] = y_before; }

        // The boundary sheets (DRIVING.ELF 0x331a60) are the SUB_PHYSICS pass (FUN_00190f88 via
        // FUN_00188200 0x188bf8), not the land car's: the vanquish's .atr sets no SUB_PHYSICS, so
        // no sheet pass runs here (docs/systems/driving-world-bounds.md).

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
    light: Option<(&nf_formats::rendertun::LightEnv, &nf_formats::rendertun::CarRender)>,
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

    // The car's parts through the corrected draw chain (nf_formats::carmesh: each placement names its
    // model set [M DRIVING.ELF 0x1ad2e0 +0x1c], the vd lists hold placement indices [M 0x1ad758]),
    // every drawn part placed in car space; normals ride the vertex nrm (the car batches carry no
    // vertex colours [D]). The old reader stays as the fallback when the car file does not decode.
    // The base-pass car light (docs/formats/car-model.md §10, bondrm.o microcode):
    // min(255, 127·(D·L1·max(0,n·d1) + L2·max(0,n·d2) + Ground·max(0,n·up) + A·Sky)) with the strip's
    // CarRender class (A, D) for this vehicle, PS2 scale (0x80 = 1.0). [S] the World light set, shade
    // 1.0 (its writer is not traced), the normals at the car's own frame (the colours are baked once,
    // not re-lit as the car turns); the specular and reflection passes are not drawn.
    let entry = light.and_then(|(_, cr)| cr.entries.iter().find(|e| e.name.eq_ignore_ascii_case(car_stem)).or_else(|| cr.entries.iter().find(|e| e.name == "default")));
    let mut strip_rgba: Vec<Vec<[u8; 4]>> = Vec::new();
    let strips: Vec<nf_formats::crp_mesh::MeshStrip> = match nf_formats::carmesh::car_mesh(&crp_bytes) {
        Ok(car) => {
            let mut v = Vec::new();
            for part in car.drawn(None).into_iter().filter(|part| !part.hidden()) {
                for st in &part.strips {
                    let rgba: Vec<[u8; 4]> = match (light, entry, st.lighting) {
                        (Some((env, _)), Some(e), Some(class)) if st.normals.len() == st.positions.len() => {
                            let pr = e.props(class);
                            st.normals.iter().map(|&n| {
                                let c = env.car_light(n, pr.ambient, pr.diffuse, 1.0);
                                let b = |x: f32| ((127.0 * x).clamp(0.0, 255.0) * 2.0).min(255.0) as u8;
                                [b(c[0]), b(c[1]), b(c[2]), 255]
                            }).collect()
                        }
                        _ => Vec::new(),
                    };
                    strip_rgba.push(rgba);
                    v.push(nf_formats::crp_mesh::MeshStrip {
                        positions: st.positions.iter().map(|&q| part.apply(q)).collect(),
                        uvs: st.uvs.clone(),
                        colors: st.normals.iter().map(|n| [(n[0] * 127.0) as i8 as u8, (n[1] * 127.0) as i8 as u8, (n[2] * 127.0) as i8 as u8]).collect(),
                        texture: st.texture.clone(),
                    });
                }
            }
            v
        }
        Err(_) => car_mod.extract_car_mesh(&car_crp, car_stem),
    };
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

    for (si, strip) in strips.into_iter().enumerate() {
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
                rgba: strip_rgba.get(si).and_then(|v| v.get(k)).copied().unwrap_or([255, 255, 255, 255]),
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
    // The track's drawn geometry: every Map in#0 placement's baked model, road, ground, props
    // and sky alike (nf_formats::geoprim, docs/formats/geoprim.md; the forward placement
    // matrices [M DRIVING.ELF 0x22faf8]).
    let geometry = nf_formats::geoprim::track_geometry(&crp_bytes)?;
    let placements = nf_formats::geoprim::map_placements(&crp).unwrap_or_default();
    // Draw batching: strips sorted by (sky, texture, 64-u map cell) so that each run of one
    // texture in one cell merges into a single draw below (paris: ~58,700 strips) [port choice].
    let cell_of = |ws: &nf_formats::geoprim::WorldStrip| -> (i32, i32) {
        let p = ws.positions.first().copied().unwrap_or([0.0; 3]);
        ((p[0] / 64.0).floor() as i32, (p[2] / 64.0).floor() as i32)
    };
    let mut strips = geometry.strips;
    strips.sort_by(|a, b| {
        let ka = (a.article.to_ascii_lowercase().starts_with("sky"), a.texture.clone(), cell_of(a));
        let kb = (b.article.to_ascii_lowercase().starts_with("sky"), b.texture.clone(), cell_of(b));
        ka.cmp(&kb)
    });
    let m0 = materials.len();
    let mut group: Vec<(bool, Option<String>, (i32, i32))> = Vec::new();
    for ws in strips {
        let gkey = (ws.article.to_ascii_lowercase().starts_with("sky"), ws.texture.clone(), cell_of(&ws));
        // The sky articles (`sky*`) are a small dome (paris: +-30 u as placed, docs/formats/geoprim.md):
        // drawn around the viewer like the on-foot camera skies (Layer::SkyCamera, before the world,
        // no depth write) with their vertices relative to their placement [S: the driving engine's
        // sky pass is not traced].
        let sky = ws.article.to_ascii_lowercase().starts_with("sky");
        let origin = if sky { placements.get(ws.placement).map_or([0.0; 3], |p| p.origin) } else { [0.0; 3] };
        let positions = if sky { ws.positions.iter().map(|q| [q[0] - origin[0], q[1] - origin[1], q[2] - origin[2]]).collect() } else { ws.positions };
        let vcols = ws.colors;
        let strip = nf_formats::crp_mesh::MeshStrip { positions, uvs: ws.uvs, colors: Vec::new(), texture: ws.texture };
        let n = materials.len();
        let (b0, b1) = (bmin, bmax);
        let v0 = vertices.len();
        push_strip(&strip, None, &mut vertices, &mut indices, &mut materials, &mut bmin, &mut bmax);
        // The part's per-vertex colours light the track (the night lighting is baked into them;
        // PS2 scale, 0x80 = 1.0) [D docs/formats/geoprim.md; the microcode's pass-through is unproven].
        if !vcols.is_empty() && vertices.len() - v0 == vcols.len() {
            for (v, c) in vertices[v0..].iter_mut().zip(&vcols) {
                let f = |x: u8| (x as u32 * 2).min(255) as u8;
                v.rgba = [f(c[0]), f(c[1]), f(c[2]), 255];
            }
        }
        if sky {
            (bmin, bmax) = (b0, b1); // the dome does not widen the track bounds
            for m in &mut materials[n..] { m.layer = Layer::SkyCamera; m.draw.pass = 0; m.draw.zwrite = false; m.radius = 1.0e6; }
        }
        for _ in n..materials.len() { group.push(gkey.clone()); }
    }
    // Merge consecutive same-group materials whose index ranges touch (one draw per texture per cell).
    {
        let mut out: Vec<Material> = Vec::with_capacity(materials.len() - m0);
        let mut out_g: Vec<(bool, Option<String>, (i32, i32))> = Vec::new();
        for (m, g) in materials.drain(m0..).zip(group.into_iter()) {
            if let (Some(last), Some(lg)) = (out.last_mut(), out_g.last()) {
                if *lg == g && last.key == m.key && last.layer == m.layer && last.first_index + last.index_count == m.first_index {
                    // Bounding sphere of both.
                    let d = [m.centre[0] - last.centre[0], m.centre[1] - last.centre[1], m.centre[2] - last.centre[2]];
                    let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if dist + m.radius > last.radius {
                        if dist + last.radius <= m.radius { last.centre = m.centre; last.radius = m.radius; }
                        else if dist > 0.0 {
                            let r = 0.5 * (dist + last.radius + m.radius);
                            let t = (r - last.radius) / dist;
                            for k in 0..3 { last.centre[k] += d[k] * t; }
                            last.radius = r;
                        }
                    }
                    last.index_count += m.index_count;
                    continue;
                }
            }
            out.push(m);
            out_g.push(g);
        }
        materials.extend(out);
    }

    let col_tris = crp.extract_collision_tris().unwrap_or_default();

    if bmin[0] > bmax[0] {
        bmin = [-100.0; 3];
        bmax = [100.0; 3];
    }

    // The ground query skips class-bit-0x04 triangles [M DRIVING.ELF 0x2138b0].
    let collision = CollisionGrid::build(col_tris.iter().filter(|t| !t.ground_query_skips()).cloned().collect());

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
    // The game's start: the one Map Trgr whose action list holds EStartMission (0x16), fired at
    // world init [M DRIVING.ELF 0x22cd30, 0x15c920]; the car goes to its x/z at the ground under
    // it + 1.0, facing its +0x30 direction [M 0x15c738, 0x1fee68] (nf_formats::trackstart).
    // The road-node pick above stays only as the fallback when no such trigger decodes.
    if let Ok(sp) = nf_formats::trackstart::track_spawn(&crp, &col_tris) {
        start_pos = sp.pos;
        start_yaw = sp.yaw;
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
    // The mission's light set (Render/Lighting/<track>.tun, World) and the car material table
    // (Render/CarRender/default.tun) for the base-pass car light.
    let read_tun = |want: &str| -> Option<String> {
        let e = bigf.entries.iter().find(|x| { let n = x.name.to_ascii_lowercase(); n.contains("render") && n.contains(want) && n.ends_with(".tun") })?;
        let raw = iso.read_bytes(viv_base + e.offset as u64, e.size as u64).ok()?;
        maybe_decompress(&raw).ok().map(|t| String::from_utf8_lossy(&t).into_owned())
    };
    let car_lighting = read_tun("lighting").and_then(|t| nf_formats::rendertun::Lighting::parse(&t).ok());
    let car_render = read_tun("carrender").and_then(|t| nf_formats::rendertun::CarRender::parse(&t).ok());
    let car_light_ref = car_lighting.as_ref().zip(car_render.as_ref()).map(|(l, c)| (&l.world, c));
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
            car_light_ref,
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
                                car_light_ref,
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
    // The mission's fog (the Render/Fog/<track>.tun tuning text, nf_formats::rendertun): its colour is the
    // frame's background where nothing draws (the open sky overhead, past the sky bands), as the
    // on-foot levels clear to theirs. The track's draws keep fog 0, so only the background changes.
    if let Some(e) = bigf.entries.iter().find(|x| { let n = x.name.to_ascii_lowercase(); n.contains("render") && n.contains("fog") && n.ends_with(".tun") }) {
        if let Ok(raw) = iso.read_bytes(viv_base + e.offset as u64, e.size as u64) {
            if let Ok(t) = maybe_decompress(&raw) {
                if let Ok(f) = nf_formats::rendertun::Fog::parse(&String::from_utf8_lossy(&t)) { scene.fog = Some((f.colour, f.end)); }
            }
        }
    }

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

        // The wheel roll is turns per frame at the integer frame rate, floored at 0.01 and capped
        // at 1.0 [M DRIVING.ELF 0x192498:126-142]: a car two ticks off the line rolls under the
        // floor, so build speed first.
        state.viv = "MIS13A".into();
        for _ in 0..120 { state.step(1.0, 0.0, 0.0, false); }
        assert!(state.rotor_spin > 0.0 && state.rotor_spin <= 1.0, "spin {}", state.rotor_spin);
    }
}
