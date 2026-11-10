//! Bullet impact effects: the decal FIFO (`Effect_Bullet` 0x1ce8a8 + `Effect_InitLists` 0x1cf268's
//! 200-entry pool at 0x3496e0), the per-material gas puffs (`Effect_Create` 0x1ce688 → `Gas_Init`
//! 0x1aacc8), debris (`Debris_Create` 0x1aaa00, `Debris_Update` 0x1aac00) and ricochets
//! (`Effect_RicochetProb` 0x1cf290 + the reflection in `Bullet_CollisionHandler` 0x12ce80).
//! Per-material data comes from the `EffectInfo` table (0x3037c0, parsed in
//! `nf_formats::sfx::SurfaceSounds::effects`). G-041; docs/atlas/effect.md.
//!
//! The game's decal pool is a free/in-use node pair: `Effect_Bullet` takes a node with
//! `DList_MoveFromFree2InUse` 0x1cccb8 and the eviction path returns it with
//! `DList_RemoveFromInUse2Free` 0x1cc868 [M]; `Effects` models the same two lists (`Decal::node`,
//! the free-node order of `DList_Init` 0x1cc778 + `LList_Add` 0x1cca28).
//!
//! RNG discipline: every `Rand_*` the game draws at an impact is drawn here, in the game's order
//! (the collision handler runs: the material-0x80 roll [not ported — the port's HITDATA carries no
//! +0x51 second byte], then `Effect_Create`'s debris-count draw, then per-debris draws, then
//! `Effect_Bullet`'s `Rand_Rand(2)`, then the ricochet roll). `Gas_Init`'s own 5 draws per puff are
//! NOT consumed — the merged deviation documented in gas.rs (G-033's discipline).

use crate::drone::Rng;
use crate::world::V3;
use nf_formats::sfx::{EffectRecord, SurfaceSounds};

pub type Mat4 = [[f32; 4]; 4];

/// `REC_FRAME_RATE` (0x30d0cc) = 1/60 [M grenade.rs:52].
const REC: f32 = 1.0 / 60.0;
/// The decal's surface offset: `Effect_Bullet` stores `hit + normal · 0.01` [M 0x1ce9f0].
const DECAL_OFFSET: f32 = 0.01;
/// The wall-puff offset: `hit + normal · 0.05` [M 0x1ce980, 0x3d4ccccd].
const PUFF_OFFSET: f32 = 0.05;

/// What a decal node is linked to [M Effect_Bullet 0x1ce8a8:160-200]: a static hit's node goes
/// on its cel's sub-object list (`node+0x18 = cel`, drawn by `View_DrawCelGlist` 0x1e6538 — only
/// while that cel is drawn); a dynamic hit's node goes on the object's list (`node+0x14 = obj`)
/// with its matrix made object-local, and `View_DrawObjectGlist` 0x1e6780 / `View_DrawCelGlist`
/// draw it at `node+0x20 · obj+0x90` — the hole rides the object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecalHost {
    /// The hit's cel (the collision world's cel-scope index; `None` when unknown — drawn always).
    Cel(Option<u32>),
    /// The hit object (`CollisionWorld::set_dynamic` id), the node matrix in its local space and
    /// the surface normal in that space.
    Object { id: u64, local: Mat4, normal: V3 },
}

/// One bullet-hole decal (a pool node at 0x3496e0, 0x60 bytes each): position, surface normal and the
/// `EffectInfo+4` mesh hash, the pool node it occupies and its host.
#[derive(Clone, Debug, PartialEq)]
pub struct Decal {
    /// World position (`hit + normal·0.01`; for an object host, re-posed every tick).
    pub pos: V3,
    pub normal: V3,
    /// The `EffectInfo[material]+4` decal mesh hash (0x02000122 glass, 0x02000126/127 stone/metal,
    /// 0x02000128 wood, 0x02000262 soft-generic [D ee_start_05.bin]; ICE 0x020006fd).
    pub mesh: u32,
    /// The pool node (0..200) — never changes while the hole lives.
    pub node: u8,
    /// The count of holes made before this one (`Effects::made` at creation).
    pub serial: u32,
    pub host: DecalHost,
    /// The world draw matrix: `Mat_Align2Dir` at `pos` for a cel host, `local · obj+0x90` for an
    /// object host [M 0x1e6538:58-66, 0x1e6780:66-74].
    pub m: Mat4,
}

/// A debris piece's draw slot (the port's fixed draw pools; the game draws each piece as its own
/// object, so a piece keeps one slot for life). `slot == u16::MAX`: not yet assigned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawSlot {
    pub slot: u16,
    /// The count of slots handed out before this one (`Effects::debris_made`).
    pub serial: u32,
}

impl Default for DrawSlot {
    fn default() -> Self { DrawSlot { slot: u16::MAX, serial: 0 } }
}

/// The debris draw slots the view builds per debris mesh.
pub const DEBRIS_DRAW_SLOTS: usize = 128;

/// One debris particle (`Debris_Create` 0x1aaa00's class-0x40 object): position, velocity, spin,
/// life in ticks, scale and the `EffectInfo+0x74` mesh.
#[derive(Clone, Debug, PartialEq)]
pub struct Debris {
    pub pos: V3,
    pub vel: V3,
    /// obj data+0x20: the `Rand_FRandHalf_Vec` spin, added to the rotation every tick.
    pub spin: V3,
    /// obj data+0x30: ticks left (`Debris_Update` kills the object at ≤ 0).
    pub life: i16,
    /// obj data+0xe8: 0.4 + FRand(0.1).
    pub scale: f32,
    pub mesh: u32,
    /// obj+0x106: the alpha byte, stepped down by data+0x32 (= 1) per tick in `Debris_Update`.
    pub alpha: u8,
    pub draw: DrawSlot,
}

/// The sim's effect state: the 200-node decal pool (oldest evicted when full [M `Effect_Bullet`
/// 0x1cea20: pool empty → `Effect_UnLink(DAT_003496ec)` = the in-use list's oldest]) and the live
/// debris.
#[derive(Clone, Debug, Default)]
pub struct Effects {
    /// The in-use list, oldest first.
    pub decals: Vec<Decal>,
    pub debris: Vec<Debris>,
    /// The count of decals evicted so far (observability for tests).
    pub evicted: u32,
    /// The count of decals ever made.
    pub made: u32,
    /// Nodes handed out fresh so far: `DList_Init` 0x1cc778 `LList_Add`s nodes 0..199 in order and
    /// `DList_MoveFromFree2InUse` 0x1cccb8 takes from the last-added end, so the fresh nodes go
    /// 199, 198, … 0 [M 0x1cca28, 0x1cccb8].
    fresh_taken: u16,
    /// Nodes returned to the free list (`DList_RemoveFromInUse2Free` 0x1cc868 links them at the
    /// far end from the take end — taken after every fresh node, first returned first).
    returned: std::collections::VecDeque<u8>,
    /// node → index into `decals` (`u16::MAX` free); rebuilt on every change.
    node_index: Vec<u16>,
    /// The count of debris draw slots handed out.
    pub debris_made: u32,
    /// The next-fit cursor of `assign_debris_slots`.
    debris_cursor: u16,
}

/// The decal pool's node count (`DList_Init(0x3496e0, 0x60, 200)` [M Effect_InitLists 0x1cf268]).
pub const DECAL_POOL: usize = 200;

impl Effects {
    /// The pool node of the live decal at `decals[index]`. The game's nodes never move:
    /// `Effect_Bullet` takes a free node, and with none free it unlinks the oldest in-use node and
    /// takes THAT node again [M 0x1cea20..0x1cea58] — a hole keeps its node for life.
    pub fn decal_node(&self, index: usize) -> usize { self.decals[index].node as usize }

    /// The live decal in pool node `node`, if any.
    pub fn decal_in_node(&self, node: usize) -> Option<&Decal> {
        if node >= DECAL_POOL { return None; }
        if self.node_index.len() == DECAL_POOL {
            let i = self.node_index[node];
            return if i == u16::MAX { None } else { self.decals.get(i as usize) };
        }
        self.decals.iter().find(|d| d.node as usize == node)
    }

    /// The pool nodes holding a hole made since the count of holes made was `prev_made`: a node
    /// taken for a new hole is rewritten whole before it draws [M 0x1ce9f0..0x1cea9c], so it must
    /// not be drawn sliding from the node's previous hole to the new one.
    pub fn decal_nodes_reused_since(&self, prev_made: u32) -> Vec<usize> {
        let mut v: Vec<usize> = self.decals.iter().filter(|d| d.serial >= prev_made).map(|d| d.node as usize).collect();
        v.sort_unstable();
        v
    }

    fn reindex(&mut self) {
        self.node_index.clear();
        self.node_index.resize(DECAL_POOL, u16::MAX);
        for (i, d) in self.decals.iter().enumerate() { self.node_index[d.node as usize] = i as u16; }
    }

    /// `DList_MoveFromFree2InUse` 0x1cccb8: the next free node, `None` with the pool full.
    fn take_node(&mut self) -> Option<u8> {
        if (self.fresh_taken as usize) < DECAL_POOL {
            self.fresh_taken += 1;
            return Some((DECAL_POOL - self.fresh_taken as usize) as u8);
        }
        self.returned.pop_front()
    }

    /// `Effect_UnLink` 0x1cf300 + `DList_RemoveFromInUse2Free` 0x1cc868 on `decals[index]`.
    fn free_decal(&mut self, index: usize) {
        let d = self.decals.remove(index);
        self.returned.push_back(d.node);
    }

    /// Takes a node for a new hole — evicting the oldest in-use hole when none is free
    /// [M Effect_Bullet 0x1cea20..0x1cea58] — and links the hole to its host. `pos` is the
    /// world node position (`hit + normal·0.01`), `host` the hit's owner as the collision world
    /// reports it.
    pub fn add_decal(&mut self, pos: V3, normal: V3, mesh: u32, host: crate::world::HitHost) {
        let node = match self.take_node() {
            Some(n) => n,
            None => {
                self.free_decal(0);
                self.evicted += 1;
                self.take_node().expect("an evicted node is free")
            }
        };
        // `Mat_Align2Dir(node+0x20, normal, (0,1,0), (1,0,0))` with `node+0x50 = pos` [M 0x1ce9c0..0x1cea00].
        let mut m = crate::searchlight::align2dir(normal, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        m[3] = [pos[0], pos[1], pos[2], 1.0];
        let host = match host {
            crate::world::HitHost::Cel(c) => DecalHost::Cel(c),
            crate::world::HitHost::Object(id, om) => {
                let (local, ln) = to_object_local(&m, normal, &om);
                DecalHost::Object { id, local, normal: ln }
            }
        };
        self.decals.push(Decal { pos, normal, mesh, node, serial: self.made, host, m });
        self.made = self.made.wrapping_add(1);
        self.reindex();
    }

    /// The per-tick follow of object-hosted holes: each takes its host's live matrix
    /// (`View_DrawObjectGlist` 0x1e6780:66-74 — `node+0x20 · obj+0x90` at every draw). A host no
    /// longer in the world frees its holes (`Effect_RemoveSObjs` 0x1cf360 on the object's
    /// delete — `Destroy_Smash` 0x12bb60:23, `control_movement_object_handler` 0x1336f8:86).
    pub fn follow_hosts(&mut self, host_matrix: impl Fn(u64) -> Option<Mat4>) {
        let mut gone = Vec::new();
        for (i, d) in self.decals.iter_mut().enumerate() {
            let DecalHost::Object { id, local, normal } = d.host else { continue };
            match host_matrix(id) {
                Some(om) => {
                    d.m = concat(&local, &om);
                    d.pos = [d.m[3][0], d.m[3][1], d.m[3][2]];
                    d.normal = rot3(normal, &om);
                }
                None => gone.push(i),
            }
        }
        if gone.is_empty() { return; }
        for &i in gone.iter().rev() { self.free_decal(i); }
        self.reindex();
    }

    /// Hands every unslotted debris piece (the impact pieces, a killed breakable's pieces, the
    /// blast pieces) a draw slot it keeps until it dies: next-fit over the slots no live piece
    /// holds, so a dead piece's slot is not reused while others are free. Drawing piece k of the
    /// concatenated lists through slot k moved every later piece to another slot on each death or
    /// new piece, and the tick blend then drew each such slot part-way between two pieces.
    pub fn assign_debris_slots(&mut self, shatter: &mut [crate::shatter::Debris], blast: &mut [crate::shatter::Debris]) {
        let mut used = [false; DEBRIS_DRAW_SLOTS];
        let slots = self.debris.iter().map(|d| d.draw).chain(shatter.iter().map(|d| d.draw)).chain(blast.iter().map(|d| d.draw));
        for s in slots { if (s.slot as usize) < DEBRIS_DRAW_SLOTS { used[s.slot as usize] = true; } }
        let mut free = used.iter().filter(|u| !**u).count();
        let (mut cursor, mut made) = (self.debris_cursor as usize % DEBRIS_DRAW_SLOTS, self.debris_made);
        let mut give = |draw: &mut DrawSlot| {
            if draw.slot != u16::MAX || free == 0 { return; }
            while used[cursor] { cursor = (cursor + 1) % DEBRIS_DRAW_SLOTS; }
            used[cursor] = true;
            free -= 1;
            *draw = DrawSlot { slot: cursor as u16, serial: made };
            made = made.wrapping_add(1);
            cursor = (cursor + 1) % DEBRIS_DRAW_SLOTS;
        };
        for d in self.debris.iter_mut() { give(&mut d.draw); }
        for d in shatter.iter_mut() { give(&mut d.draw); }
        for d in blast.iter_mut() { give(&mut d.draw); }
        self.debris_cursor = cursor as u16;
        self.debris_made = made;
    }
}

/// Which list a debris draw slot reads: 0 the impact pieces (`Effects::debris`), 1 a killed
/// breakable's pieces, 2 the blast pieces — and the index in it.
pub type DebrisRef = (u8, usize);

/// The piece each debris draw slot shows. Slotted pieces sit in their own slot; pieces not yet
/// slotted (made since the last `assign_debris_slots`) fill the remaining slots in list order.
pub fn debris_slot_map(fx: &[Debris], shatter: &[crate::shatter::Debris], blast: &[crate::shatter::Debris]) -> Vec<Option<DebrisRef>> {
    let mut map: Vec<Option<DebrisRef>> = vec![None; DEBRIS_DRAW_SLOTS];
    let all = fx.iter().map(|d| d.draw).enumerate().map(|(i, s)| ((0u8, i), s))
        .chain(shatter.iter().map(|d| d.draw).enumerate().map(|(i, s)| ((1u8, i), s)))
        .chain(blast.iter().map(|d| d.draw).enumerate().map(|(i, s)| ((2u8, i), s)));
    let mut pending = Vec::new();
    for (r, s) in all {
        match map.get_mut(s.slot as usize) {
            Some(e) if e.is_none() => *e = Some(r),
            _ => pending.push(r),
        }
    }
    let mut free = (0..DEBRIS_DRAW_SLOTS).filter(|&k| map[k].is_none()).collect::<Vec<_>>().into_iter();
    for r in pending { match free.next() { Some(k) => map[k] = Some(r), None => break } }
    map
}

/// The debris draw slots whose piece was slotted since the count of slots handed out was
/// `prev_made` (as `Effects::decal_nodes_reused_since`: a slot never slides between two pieces).
pub fn debris_slots_new_since(fx: &[Debris], shatter: &[crate::shatter::Debris], blast: &[crate::shatter::Debris], prev_made: u32) -> Vec<usize> {
    let mut v: Vec<usize> = fx.iter().map(|d| d.draw).chain(shatter.iter().map(|d| d.draw)).chain(blast.iter().map(|d| d.draw))
        .filter(|s| (s.slot as usize) < DEBRIS_DRAW_SLOTS && s.serial >= prev_made)
        .map(|s| s.slot as usize).collect();
    v.sort_unstable();
    v
}

/// `Concat(a, b)` as the game's draw runs it with b = `obj+0x90` [M Concat 0x1e4a90]: the rows of
/// `a` through b's rotation, then b's translation added to the last row (row-vector order: a, then b).
pub fn concat(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut r = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..3 { r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j]; }
    }
    for j in 0..3 { r[3][j] += b[3][j]; }
    r[3][3] = 1.0;
    r
}

/// `v · rot(m)` (the rows' 3×3 only).
fn rot3(v: V3, m: &Mat4) -> V3 {
    [0, 1, 2].map(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j])
}

/// `Effect_Bullet`'s object arm [M 0x1ce8a8:166-186]: `Mat_Inverse(obj+0x90)` 0x1e3520 (the 3×3
/// transposed, the translation only negated), `Concat` of the node's rotation through it, then the
/// node position by `RotPreTransVec` 0x1e49e8 — `(pos − t) · Rᵀ`. Returns the local node matrix
/// and the normal in the same space.
fn to_object_local(node: &Mat4, normal: V3, obj: &Mat4) -> (Mat4, V3) {
    let mut inv = [[0.0f32; 4]; 4];
    for i in 0..3 { for j in 0..3 { inv[i][j] = obj[j][i]; } }
    let mut local = [[0.0f32; 4]; 4];
    for i in 0..3 {
        let v = rot3([node[i][0], node[i][1], node[i][2]], &inv);
        local[i] = [v[0], v[1], v[2], 0.0];
    }
    let p = [node[3][0] - obj[3][0], node[3][1] - obj[3][1], node[3][2] - obj[3][2]];
    let lp = rot3(p, &inv);
    local[3] = [lp[0], lp[1], lp[2], 1.0];
    (local, rot3(normal, &inv))
}

/// `Rand_FRandHalf(max)` 0x1e3810: one FRand draw, minus `max·0.5` — the signed half-range.
pub fn frand_half(rng: &mut Rng, max: f32) -> f32 {
    nf_formats::ps2float::sub(rng.frand(max), max * 0.5)
}

/// `Rand_FRand_MVar2(min, off)` 0x1e38b0: one FRand draw of `min`, minus `off` — range [−off, min−off].
pub fn frand_mvar2(rng: &mut Rng, min: f32, off: f32) -> f32 {
    nf_formats::ps2float::sub(rng.frand(min), off)
}

/// `Rand_FRandHalf_Vec(scale, out)` 0x1dfef0: three FRandHalf draws into a vector.
pub fn frand_half_vec(rng: &mut Rng, scale: f32) -> V3 {
    [frand_half(rng, scale), frand_half(rng, scale), frand_half(rng, scale)]
}

/// `Vec_NormToRot` 0x1e31b0 [M, the whole 104-byte function]: `out = (−atan2(y, √(x²+z²)),
/// atan2(x, z), 0)` — the euler (pitch, yaw, roll) that rotates the world −z axis onto the
/// vector. `Effect_Bullet`'s wall puffs spawn with it [M 0x1ce8a8:77, :129]; the game's
/// `atan2f` is the C library's — the port's f32 atan2 matches to the ulp [S the PS2 libm's
/// last-bit differences].
pub fn vec_norm_to_rot(v: V3) -> [f32; 3] {
    [-f32::atan2(v[1], (v[0] * v[0] + v[2] * v[2]).sqrt()), f32::atan2(v[0], v[2]), 0.0]
}

/// `Effect_Bullet`'s floor-puff euler for a +0x1b entry [M 0x1ce8a8:78-85, the fStack_120
/// vector]: pitch = −ATAN2_APPROX(−y, √((−x)²+(−z)²)), yaw = ATAN2_APPROX(−x, −z), roll 0 —
/// the negated normal through the same construction as `vec_norm_to_rot`. The game computes
/// it with its approximate `ATAN2_APPROX` 0x1e30f8 [S: the port uses libm atan2; the puff
/// orientation difference is sub-degree].
pub fn negated_normal_euler(v: V3) -> [f32; 3] {
    let nx = -v[0];
    let ny = -v[1];
    let nz = -v[2];
    [-f32::atan2(ny, (nx * nx + nz * nz).sqrt()), f32::atan2(nx, nz), 0.0]
}

/// `Vec_MakeOrthonormalBasis` 0x1e4b78: two vectors orthonormal against `n`; `Effect_Create` stores
/// `u + v` into `HITDATA+0x20` [M 0x1ce740] and `Debris_Create` spreads the debris velocity with it.
pub fn orthonormal_basis_pair(n: V3) -> (V3, V3) {
    if n[0].abs() <= n[1].abs() {
        let il = 1.0 / (n[2] * n[2] + n[1] * n[1]).sqrt();
        let a = [0.0, -n[2] * il, n[1] * il];
        let b = [n[1] * a[2] - n[2] * a[1], -n[0] * a[2], n[0] * a[1]];
        (a, b)
    } else {
        let il = 1.0 / (n[2] * n[2] + n[0] * n[0]).sqrt();
        let a = [n[2] * il, 0.0, -n[0] * il];
        let b = [n[1] * a[2], n[2] * a[0] - n[0] * a[2], -n[1] * a[0]];
        (a, b)
    }
}

/// Returns `u + v` (the only form the callers use).
pub fn orthonormal_basis_sum(n: V3) -> V3 {
    let (a, b) = orthonormal_basis_pair(n);
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// `Bullet_CollisionHandler` 0x12ce80's per-hit pass rules [M disasm 0x12cee0..0x12cfb8]: the walk
/// decides whether a bullet PASSES THROUGH the surface it hit (a pierce keeps the bullet alive —
/// state 1, the handler walks the trace's NEXT HITDATA and spawns the exit-hole effects — while a
/// stop dies with `BU+0xF0 |= 0x10` [M 0x12d81c..0x12d82c]). In the game's order:
/// - `HITDATA+0x51` bit 1 — set by `Collide_Intersect` 0x1ebaa8 when the glist node's flags carry
///   the pass bit — forces pierce (the caller's `node_pass`);
/// - the object-hit rules (`obj+0xf0 & 0x60`, the shooter skip, class-0x28 windows vs
///   `weapon_data+0x70 & 0x100` [M 0x12cf10..0x12cf3c]) — the port's world walk sees world hits
///   only, no object arm;
/// - **the glass rule [M 0x12cf44..0x12cf64]**: `if (!(weapon_data+0x6c & 0x800) &&
///   (HITDATA+0x50 & 0x3f) == 0x10) pass = 1` — material index 0x10 (the 07000008 corridor
///   windows' surface byte) lets ANY bullet through unless the weapon's `+0x6c` carries 0x800
///   (the Laser 0x4E's +0x6c = 0x900 [L] burns the pane instead); no RNG draw;
/// - `weapon_data+2` (the weapon id) 0x50/0x51 (the Q-Claw) then skip the surface-byte rules —
///   no draw [M 0x12cf68 `if (uVar2 - 0x50 < 2)`];
/// - surface byte `& 0x40` → forced pierce, no draw [M 0x12cf7c];
/// - surface byte `& 0x80` → ONE `Rand_Rand(0x3f)` draw; bit 0 of the roll ORs into pierce
///   [M 0x12cf9c `Rand_Rand(0x3f)`, `bVar7 = bVar7 | bVar6 & 1`].
/// The material's 0x40/0x80 bits are per-triangle bytes in the level collision blob
/// (`Intersect_RayGeom` 0x10a8c8:326 copies the whole byte to `HITDATA+0x50`).
/// `flags6c` is the shooter's `weapon_data+0x6c` word (the glass-rule gate). fix-rocket-glass-08.
pub fn penetration_roll(rng: &mut Rng, material: u8, weapon_id: u16, flags6c: u32) -> bool {
    penetration_roll_node(rng, material, weapon_id, false, flags6c)
}

/// The roll with the hit's node pass flag (`HITDATA+0x51` bit 1's glist-flags source —
/// `Collide_Intersect` 0x1ebaa8:143-145): a node-flagged hit passes WITHOUT a draw (the game's
/// walk sets the pass from the flag and the 0x40 byte before any roll), and the surface byte's
/// 0x80 roll is drawn exactly when the byte carries the bit — even on a node-flagged hit, whose
/// result is then OR'd in (`Collide_Update` 0x1ea398:216-220). fix-foliage-bullets.
pub fn penetration_roll_node(rng: &mut Rng, material: u8, weapon_id: u16, node_pass: bool, flags6c: u32) -> bool {
    let mut pierce = node_pass
        // the glass rule [M 0x12cf44..0x12cf64]: material 0x10 passes before the Q-Claw's
        // type-word check, so even the grapple bolt flies through a pane (no draw).
        || flags6c & 0x800 == 0 && material & 0x3f == 0x10;
    if weapon_id == 0x50 || weapon_id == 0x51 { return pierce; }
    pierce |= material & 0x40 != 0;
    if material & 0x80 != 0 {
        pierce |= rng.rand(0x3f) & 1 != 0;
        if std::env::var_os("NF_RNGLOG").is_some() {
            eprintln!("[rnglog] penetration_roll +1 (draw #{})", rng.draws);
        }
    }
    pierce
}

impl Effects {
    pub fn new() -> Effects { Effects { decals: Vec::with_capacity(200), ..Default::default() } }

    /// The `EffectInfo` record of `material` (`HITDATA+0x50 & 0x3f`), None when the table is absent or
    /// the material is 0 (the game's `Effect_Create` guard [M 0x1ce6f8]).
    pub fn record<'a>(surfaces: Option<&'a SurfaceSounds>, material: u8) -> Option<&'a EffectRecord> {
        let s = surfaces?;
        let m = (material & 0x3f) as usize;
        if m == 0 { return None; }
        s.effects.get(m)
    }

    /// One bullet hit on the world, in `Bullet_CollisionHandler` 0x12ce80's order:
    /// `Effect_Create` 0x1ce688 (debris) → `Effect_Bullet` 0x1ce8a8 (gas puffs + the decal) → the
    /// ricochet roll (`Effect_RicochetProb` 0x1cf290) and reflection [M 0x12d6f8..0x12d7b8].
    ///
    /// `flags70` is `weapon_data+0x70`: bit 2 (0x2) enables the effects (the flags become 0x61 =
    /// bullet + emitter + sound), bit 4 (0x10) enables ricochets. `dir` is the bullet's direction
    /// (BU+0x00), `bounces` its ricochet count (BU+0x50, killed at 3). `owner_is_player`: the shooting
    /// object is class 3 — `Effect_Bullet` skips its `Rand_Rand(2)` for the player's own shots
    /// [M 0x1ce94c: `obj+0xe0 → +0x30`, byte `+0xff == 3`]. Returns the reflected direction when the
    /// bullet ricochets (the caller re-aims the bullet and bumps its bounce count).
    ///
    /// The impact sound itself is `EffectInfo+0x14` (`SurfaceSounds::impact`), already played by the
    /// callers' `impact_sound`.
    pub fn bullet_impact(&mut self, rng: &mut Rng, surfaces: Option<&SurfaceSounds>, gas: &mut crate::gas::GasPool,
                         fx: &mut crate::world_fx::WorldFx,
                         pos: V3, normal: V3, material: u8, dir: V3, flags70: u32, bounces: u8, owner_is_player: bool,
                         ricochet_mask: u32, gas_host: crate::gas::HostCel) -> Option<V3> {
        self.bullet_impact_on(rng, surfaces, gas, fx, pos, normal, material, dir, flags70, bounces, owner_is_player, ricochet_mask, crate::world::HitHost::Cel(None), gas_host)
    }

    /// `bullet_impact` with the hit's host (`CollisionWorld::segment_surface_host*`): the decal
    /// node links to the hit's cel or object [M Effect_Bullet 0x1ce8a8:160-200].
    #[allow(clippy::too_many_arguments)]
    pub fn bullet_impact_on(&mut self, rng: &mut Rng, surfaces: Option<&SurfaceSounds>, gas: &mut crate::gas::GasPool,
                         fx: &mut crate::world_fx::WorldFx,
                         pos: V3, normal: V3, material: u8, dir: V3, flags70: u32, bounces: u8, owner_is_player: bool,
                         ricochet_mask: u32, host: crate::world::HitHost, gas_host: crate::gas::HostCel) -> Option<V3> {
        // `Bullet_CollisionHandler` 0x12ce80: the effect flags start 0; weapon_data+0x70 & 2 → 0x61
        // [M 0x12cf58 init, 0x12d198 `| 0x61`]. Without bit 2 the game passes flags 0: no decal, no
        // gas, no debris — only the +0x14 sound (which `Effect_Bullet`'s 0x40 arm plays; the port
        // plays it unconditionally via `impact_sound`).
        let flags: u32 = if flags70 & 0x2 != 0 { 0x61 } else { 0 };
        // Laser-flagged weapons (`+0x6c & 0x100` / `+0x70 & 0x1000`) pass flags 0x10 instead — the
        // port's laser/watch beams are hitscan stand-ins and do not route through here.
        let Some(rec) = Self::record(surfaces, material) else { return None };
        // `Effect_Create` 0x1ce688: GLASS (material & 0x3f == 0x10) negates the normal for the debris
        // basis [M 0x1ce728], restored before `Effect_Bullet`.
        let glass = (material & 0x3f) == 0x10;
        let n = normal;
        if flags & 0x20 != 0 {
            // The emitter arm [M Effect_Create 0x1ce688:81-88, the flags & 0x20 block's FIRST
            // action]: `Emitter_Create(0, 0, EffectInfo+0x84, hit_pos, normal, 0, 0, 0)` 0x17ae00 —
            // a ONE-SHOT burst emitter (rate argument 0 → the +0x4b one-shot latch) at the hit
            // point, oriented by the surface normal, spawning `EffectInfo[material]+0x84`'s def.
            // `Emitter_Create` itself consumes NO RNG (a failed def lookup → no object at all);
            // the draws come on the emitter's OWN update ticks (`Emitter_Update` 0x17b080, 4 ×
            // `Rand_FRandHalf(1.0)` per spawned particle — yaw, pitch, speed, life, that order).
            if rec.emitter != 0 {
                fx.spawn_impact(rec.emitter, pos, n);
            }
        }
        if flags & 0x20 != 0 && rec.debris_enable != 0 && rec.debris_mesh != 0 {
            // `Rand_Rand(EffectInfo+0x83)` → count [M 0x1ce7b0]; `Debris_Create` 0x1aaa00 per particle:
            let count = rng.rand(rec.debris_count as u32) + 1;
            if std::env::var_os("NF_RNGLOG").is_some() {
                eprintln!("[rnglog] debris_count +1 (draw #{})", rng.draws);
            }
            let basis = orthonormal_basis_sum(if glass { [-n[0], -n[1], -n[2]] } else { n });
            for _ in 0..count {
                let scale = rng.frand(0.1) + 0.4;                       // data+0xe8 [M 0x1aaad8]
                let life = (rec.debris_life >> 1) as u32 + rng.rand((rec.debris_life >> 1) as u32); // data+0x30 [M 0x1aaaf4]
                let half = rec.debris_speed * 0.5;
                let jitter = [frand_half(rng, rec.debris_speed), frand_half(rng, rec.debris_speed), frand_half(rng, rec.debris_speed)];
                let vel = [
                    (half * n[0] + basis[0] * jitter[0]) * REC,
                    (half * n[1] + basis[1] * jitter[1]) * REC,
                    (half * n[2] + basis[2] * jitter[2]) * REC,
                ];
                let spin = frand_half_vec(rng, rec.debris_spin);        // data+0x20 [M 0x1aab68]
                self.debris.push(Debris { pos, vel, spin, life: life as i16, scale, mesh: rec.debris_mesh, alpha: 0xff, draw: DrawSlot::default() });
            }
        }
        if flags & 0x1 != 0 {
            // `Effect_Bullet`'s gas loop: `Rand_Rand(2)` unless the shooter is the player [M 0x1ce94c].
            if !owner_is_player && rng.rand(2) != 0 {
                // The 1-in-2 kill skips only the puffs; the decal below still runs.
            } else {
                for g in &rec.gas {
                    if g.mesh == 0 { continue; }
                    // Flag +0x1a: wall puffs (offset along the normal) are skipped on floor/ceiling hits
                    // [M 0x1ce990: `if (0.3 < |normal.y|) skip`].
                    let wall = g.wall_offset != 0;
                    if wall && n[1].abs() > 0.3 { continue; }
                    let p = if wall { [pos[0] + n[0] * PUFF_OFFSET, pos[1] + n[1] * PUFF_OFFSET, pos[2] + n[2] * PUFF_OFFSET] } else { pos };
                    // The spawn rotation [M 0x1ce8a8:122-130]: a wall puff (+0x1a) orients along the
                    // normal (`Vec_NormToRot` 0x1e31b0); a floor puff with the +0x1b byte set takes the
                    // negated-normal euler [M :78-85], else the zero vector.
                    let rot = if wall {
                        vec_norm_to_rot(n)
                    } else if g.orient_normal != 0 {
                        negated_normal_euler(n)
                    } else {
                        [0.0; 3]
                    };
                    let rgba = [((g.rgba[0] >> 24) & 0xff) as u8, ((g.rgba[0] >> 16) & 0xff) as u8, ((g.rgba[0] >> 8) & 0xff) as u8, (g.rgba[0] & 0xff) as u8];
                    gas.init_ex_rot(rng, g.mesh, p, 0.0, [0.0; 3], g.scale[0], g.scale[1], g.kind as u8, rgba, g.rgba[1], g.life as f32, rot, gas_host);
                }
            }
            // The decal: `EffectInfo+4` != 0 → take a pool node, evicting the OLDEST when the 200 are
            // in use [M 0x1cea20..0x1cea58]; `Mat_Align2Dir` + `hit + normal·0.01` [M 0x1ce9f0]. No RNG.
            if rec.decal != 0 {
                self.add_decal([pos[0] + n[0] * DECAL_OFFSET, pos[1] + n[1] * DECAL_OFFSET, pos[2] + n[2] * DECAL_OFFSET], n, rec.decal, host);
            }
        }
        // The ricochet [M 0x12d6f8..0x12d7b8]: weapon_data+0x70 & 0x10, the bullet's bounce count < 3,
        // `Effect_RicochetProb(material, mask)` — mask 8 (the port has no hit-object for world hits,
        // so the 9 variant's `obj+0x3c & 0x400000` never applies). The roll `Rand_Rand(N) & mask`
        // draws only when `EffectInfo+0x1a != 0` [M 0x1cf290].
        if flags70 & 0x10 != 0 && bounces < 3 && rec.ricochet_n != 0 {
            // The mask: 8 for a world hit, 9 when the hit object carries +0x3c & 0x400000, and 0
            // on a PASS/PIERCE contact — and `Effect_RicochetProb` 0x1cf290 returns 0 for mask 0
            // BEFORE its `Rand_Rand` (the `else if (param_2 == 0)` arm), so a pass contact draws
            // NOTHING here; the old port drew anyway — an RNG draw the game never makes
            // (bullet-pass-water).
            if ricochet_mask != 0 && rng.rand(rec.ricochet_n as u32) as u8 & ricochet_mask as u8 != 0 {
                if std::env::var_os("NF_RNGLOG").is_some() {
                    eprintln!("[rnglog] ricochet_roll +1 (draw #{})", rng.draws);
                }
                // Reflect: `dir + normal·(−(rest+1)·dot(dir,normal))`, normalised, with the restitution
                // read as raw f32 bits from `EffectInfo+0x1c` [M 0x12d704..0x12d7a8].
                let rest = f32::from_bits(rec.restitution_bits);
                let dot = dir[0] * n[0] + dir[1] * n[1] + dir[2] * n[2];
                let k = -(dot * rest + dot);
                let r = [dir[0] + n[0] * k, dir[1] + n[1] * k, dir[2] + n[2] * k];
                let l = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
                if l > 1e-12 {
                    return Some([r[0] / l, r[1] / l, r[2] / l]);
                }
            }
        }
        None
    }

    /// `Debris_Update` 0x1aac00 per particle: life −1, pos += vel, vel += WldGravity·(1/60)² while
    /// state 0, rotation += spin, the alpha byte steps down by 1 per tick; kill at life ≤ 0.
    pub fn update(&mut self) {
        for d in self.debris.iter_mut() {
            d.life -= 1;
            d.pos = [d.pos[0] + d.vel[0], d.pos[1] + d.vel[1], d.pos[2] + d.vel[2]];
            d.vel = [d.vel[0], d.vel[1] + 0.0 - 9.8 * REC * REC, d.vel[2]];
            d.alpha = d.alpha.saturating_sub(1);
        }
        self.debris.retain(|d| d.life > 0);
    }
}

/// Which pool an `EffectLightPick` slot belongs to — the view's `Attachment::effect` kinds
/// (0 decal / 1 casing / 2 debris / 3 smoke cloud) plus 4 for the gas puffs, which ride the
/// attachment's own `gas` field and so have no effect kind of their own.
pub const EFFECT_PICK_DECAL: u8 = 0;
pub const EFFECT_PICK_CASING: u8 = 1;
pub const EFFECT_PICK_DEBRIS: u8 = 2;
pub const EFFECT_PICK_SMOKE_CLOUD: u8 = 3;
pub const EFFECT_PICK_GAS_PUFF: u8 = 4;

/// One effect draw slot's per-object light pick. The game picks the lights of EVERY drawn object
/// at the object's own world sphere, per frame, in the draw-list walk: `View_DrawObjList`
/// 0x1e69a0:73-81 runs `Lights_CalcClosestLights(obj+0x80, *(u16*)(obj+0xF0), &count)` then
/// `psiLight_SetLights(list, count, 1, 1, 1)` for every non-cel, non-animobject entry, BEFORE the
/// per-class draw dispatch — debris, casings, gas puffs and smoke clouds all ride it [M]. The
/// port's effect draws lit whole mesh groups from ONE pick at the load-time (identity) mesh
/// centre — the world origin — so pieces near different lights drew each other's light.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectLightPick {
    /// The pool (`EFFECT_PICK_*`).
    pub kind: u8,
    /// The slot within that pool (the casing/debris draw slot, the puff/cloud list index).
    pub slot: u16,
    /// The mesh the slot draws — the hash whose glist the game linked at create ([M Debris_Create
    /// 0x1aaa00:51, Bullet_init_casing 0x12fe2c, Gas_Init 0x1aacc8, Occlude_Create 0x17ea80]).
    pub mesh: u32,
    /// The game's obj+0x80 world sphere (centre xyz, w = radius): `Control_BuildWorldSph`
    /// 0x133070 transforms the glist's local sphere by the object matrix and scales the radius by
    /// |obj+0xE8| — for a debris piece the piece position and the mesh radius × 0.4..0.5. The
    /// effect meshes' glist local centres measure ≤ 0.03 units ([D] archive census, 07000005:
    /// debris 0x0200012c (−0.007, 0.000, −0.001), gas 0x0200001f/0x02000039 exactly 0), so the
    /// port feeds the piece position for them; the smoke cloud mesh's 1.1-unit centre offset is
    /// carried ([D] 0x02000273 centre (−0.005, 1.097, 0.117)).
    pub sphere: [f32; 4],
    /// obj+0xF0's low 16 bits — the mask `Lights_CalcClosestLights` gates kinds with: kinds 0/2
    /// need bit 0x80, kind 1 is skipped under bit 0x100 [M 0x1cb2e0:19-31]. The class creators OR
    /// their bits in after `Control_SetGList` copies the glist word [M Debris_Create 0x1aaa00:53
    /// `|= 0x20000020`; Gas_Init 0x1aacc8:83 `|= 0x20020120`], so a puff's mask is the mesh word
    /// | 0x120 — which excludes kind 1 on every gas mesh (0x229/0x329 → 0x329 [D census]).
    pub flags: u16,
    /// The chosen lights, nearest first: up to two indices into the resolved list the pick ran
    /// against (the game sorts by the falloff `d/(radius·0.5)` and uploads the first two [M
    /// 0x1cb2e0:52-77 + `psiLight_SetLights` 0x10e588:11-14]). `None` = nothing accepted — the
    /// 0x0321/0x329/0x150 meshes accept no kind at any centre ([D] archive census).
    pub lights: [Option<u32>; 2],
}

/// The per-object light pick of every live effect draw slot, against `resolved` — the sim-side
/// data the draw consumes instead of one origin-centred pick per mesh group. `mesh_sphere` maps
/// a mesh hash to its glist bounds (local centre, radius, the +0x44 flags word) — the same
/// records the level parse's `MapObject` carries. Deterministic: the walk and its sort draw no
/// RNG [M Lights_CalcClosestLights 0x1cb2e0].
///
/// Decals are deliberately absent: the game never picks with the decal node's own glist word.
/// The node draws inside its host's list entry, so the pick runs at the HOST's sphere with the
/// HOST's word — cel+0x80 / cel+0x94 for a cel host [M View_DrawCelGlist 0x1e6538:58-60],
/// obj+0x80 / obj+0xF0 for an object host [M View_DrawObjectGlist 0x1e6780] — which the port's
/// world draws already reproduce; the decal meshes' own 0x0321 word never enters any pick.
pub fn effect_light_picks(
    drones: &crate::drone::Drones,
    resolved: &crate::light::ResolvedLights,
    mesh_sphere: &dyn Fn(u32) -> Option<([f32; 3], f32, u16)>,
) -> Vec<EffectLightPick> {
    let mut out = Vec::new();
    let mut buf = [(0usize, 0.0f32); 30];
    let mut push = |kind: u8, slot: u16, mesh: u32, centre: [f32; 3], radius: f32, flags: u16| {
        let mut p = EffectLightPick {
            kind,
            slot,
            mesh,
            sphere: [centre[0], centre[1], centre[2], radius],
            flags,
            lights: [None, None],
        };
        // The game's walk + falloff sort, capped at 30 [M 0x1cb2e0], then the first two upload [M 0x10e588].
        let n = resolved.closest_into(p.sphere, flags as u32, &mut buf);
        for (s, &(i, _)) in buf[..n].iter().take(2).enumerate() {
            p.lights[s] = Some(i as u32);
        }
        out.push(p);
    };
    // Casings (class 0x40, list 0): the flying ones draw, so the flying ones pick [M
    // Bullet_InitCasingLists 0x12fd48 parks the armed slots View_SetDrawInNoViews;
    // Bullet_init_casing_ex 0x12f678 re-shows them]. Mask: the casing mesh's own glist word (no
    // obj+0xF0 OR in the casing creators [M grep of every obj+0xF0 store]).
    for (slot, c) in drones.casings.list.iter().enumerate() {
        if c.state != 0 { continue; }
        let Some((_, radius, flags)) = mesh_sphere(c.mesh) else { continue };
        push(EFFECT_PICK_CASING, slot as u16, c.mesh, c.pos, radius, flags);
    }
    // Debris (class 0x10): every slotted piece — the effect debris, the breakable pieces and the
    // blast debris share the 128 draw slots (`debris_slot_map`). Mask: mesh word | 0x20 [M
    // 0x1aaa00:53]; radius = mesh radius × obj+0xE8 (0.4 + FRand(0.1) [M 0x1aaa00:30-33]).
    let map = debris_slot_map(&drones.effects.debris, &drones.breaks.shatter.debris, &drones.blast_debris);
    for (slot, r) in map.iter().enumerate() {
        let Some((src, i)) = r else { continue };
        let i = *i;
        let (mesh, pos, scale) = match src {
            0 => match drones.effects.debris.get(i) {
                Some(d) => (d.mesh, d.pos, d.scale),
                None => continue,
            },
            1 => match drones.breaks.shatter.debris.get(i) {
                Some(d) => (d.model, d.pos, d.e8),
                None => continue,
            },
            2 => match drones.blast_debris.get(i) {
                Some(d) => (d.model, d.pos, d.e8),
                None => continue,
            },
            _ => continue,
        };
        let Some((_, radius, flags)) = mesh_sphere(mesh) else { continue };
        push(EFFECT_PICK_DEBRIS, slot as u16, mesh, pos, radius * scale, flags | 0x20);
    }
    // Smoke clouds (class 0x2A, `Occlude_Create` 0x17ea80): a static object at obj+0x30; the
    // mesh's glist centre offset rides the sphere ([D] 0x02000273 local centre y 1.097). Mask:
    // the mesh word (no OR in Occlude_Create [M grep]).
    for (slot, c) in drones.smoke_clouds.iter().enumerate() {
        let Some((centre, radius, flags)) = mesh_sphere(c.mesh) else { continue };
        push(EFFECT_PICK_SMOKE_CLOUD, slot as u16, c.mesh,
            [c.pos[0] + centre[0], c.pos[1] + centre[1], c.pos[2] + centre[2]], radius, flags);
    }
    // Gas puffs: the live ones (a hidden puff sits parked out of the draw lists [M
    // View_SetDrawInNoViews]). Mask: mesh word | 0x120 [M 0x1aacc8:83]; radius = mesh radius ×
    // the puff's own scale.
    for (slot, g) in drones.gas.puffs.iter().enumerate() {
        if g.hidden { continue; }
        let Some((_, radius, flags)) = mesh_sphere(g.mesh) else { continue };
        push(EFFECT_PICK_GAS_PUFF, slot as u16, g.mesh, g.pos, radius * g.scale, flags | 0x120);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use nf_formats::sfx::EffectGas;

    fn rec(decal: u32, ricochet_n: u16) -> EffectRecord {
        EffectRecord {
            decal, gas_mesh: 0x0200_0215, ricochet_n, restitution_bits: 0x3f,
            gas: [EffectGas { mesh: 0x0200_0039, kind: 9, rgba: [0x7f7f_7f7f, 0], scale: [0.005, 0.02], life: 32, wall_offset: 1, orient_normal: 0 }, Default::default(), Default::default()],
            debris_mesh: 0x0200_012c, debris_speed: 6.2, debris_spin: 0.785, debris_life: 55, debris_enable: 1, debris_count: 5,
            emitter: 0,
            foot_decal_4: 0,
            foot_decal_8: 0,
        }
    }

    /// fx-scrape-and-impacts row 13: `Vec_NormToRot` 0x1e31b0's values and the wall puff's spawn
    /// rotation — `Effect_Bullet` orients the +0x1a wall puff with `Vec_NormToRot(normal)`
    /// [M 0x1ce8a8:129], the floor puff with the negated-normal euler by +0x1b [M :122-127] and
    /// zero without it. Sister: the pre-fix shape (both zero) fails on the wall arm.
    #[test]
    fn wall_puffs_spawn_oriented_along_the_normal() {
        // The pure forms: axis normals through Vec_NormToRot.
        assert_eq!(vec_norm_to_rot([0.0, 0.0, 1.0]), [0.0, 0.0, 0.0]);
        assert!((vec_norm_to_rot([1.0, 0.0, 0.0])[1] - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        assert!((vec_norm_to_rot([0.0, 1.0, 0.0])[0] + std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // The euler's negated form mirrors the pitch and the yaw.
        let e = negated_normal_euler([0.0, 1.0, 0.0]);
        assert!((e[0] - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
        // Through the real path: a wall hit (|normal.y| ≤ 0.3) spawns the puff with the
        // Vec_NormToRot rotation; a floor puff follows its +0x1b byte.
        let mut r = rec(0x0200_0126, 0);
        r.gas[0].orient_normal = 1;
        let surf = table(r);
        let mut gas = crate::gas::GasPool::default();
        let mut e2 = Effects::new();
        let mut rng = Rng::new(3);
        // Normal [0.8, 0, 0.6]: |y| = 0 → the wall puff runs (and the floor puffs' slots are
        // empty, mesh 0 — the first entry is the only one).
        e2.bullet_impact(&mut rng, Some(&surf), &mut gas, &mut fx(), [0.0, 0.0, 0.0], [0.8, 0.0, 0.6], 10, [0.0, 0.0, -1.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        assert_eq!(gas.puffs.len(), 1);
        let want = vec_norm_to_rot([0.8, 0.0, 0.6]);
        assert_eq!(gas.puffs[0].rot, want, "the wall puff carries the Vec_NormToRot euler");
        // The +0x1b floor puff: normal [0,1,0] is a floor hit for the WALL entry (skipped), so
        // make the entry a floor puff (+0x1a clear) and re-shoot.
        let mut r = rec(0, 0);
        r.gas[0] = EffectGas { mesh: 0x0200_0039, kind: 0, rgba: [0x7f7f_7f7f, 0], scale: [0.005, 0.02], life: 32, wall_offset: 0, orient_normal: 1 };
        let surf = table(r);
        let mut gas2 = crate::gas::GasPool::default();
        let mut e3 = Effects::new();
        e3.bullet_impact(&mut rng, Some(&surf), &mut gas2, &mut fx(), [1.0, 1.0, 1.0], [0.0, 1.0, 0.0], 10, [0.0, -1.0, 0.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        assert_eq!(gas2.puffs[0].rot, negated_normal_euler([0.0, 1.0, 0.0]));
        // +0x1b clear → the zero vector.
        let mut r = rec(0, 0);
        r.gas[0] = EffectGas { mesh: 0x0200_0039, kind: 0, rgba: [0x7f7f_7f7f, 0], scale: [0.005, 0.02], life: 32, wall_offset: 0, orient_normal: 0 };
        let surf = table(r);
        let mut gas3 = crate::gas::GasPool::default();
        let mut e4 = Effects::new();
        e4.bullet_impact(&mut rng, Some(&surf), &mut gas3, &mut fx(), [1.0, 1.0, 1.0], [0.0, 1.0, 0.0], 10, [0.0, -1.0, 0.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        assert_eq!(gas3.puffs[0].rot, [0.0; 3]);
    }

    /// A 23-entry `EffectInfo` table with `r` at material 10 (STONE) — the parse is indexed by the
    /// material byte (`HITDATA+0x50 & 0x3f`).
    fn table(r: EffectRecord) -> SurfaceSounds {
        SurfaceSounds { impact: vec![], footstep: vec![], body: vec![],
            effects: (0..23).map(|k| if k == 10 { r } else { Default::default() }).collect(), ..Default::default() }
    }

    fn fx() -> crate::world_fx::WorldFx { crate::world_fx::WorldFx::default() }

    /// G-041: a hit on a decal-carrying material leaves one decal at `hit + normal·0.01` with the
    /// material's mesh. Sister: material 0 (NONE) and flags without weapon bit 2 leave none.
    #[test]
    fn decal_at_offset_with_material_mesh() {
        let mut e = Effects::new();
        let mut gas = crate::gas::GasPool::default();
        let r = rec(0x0200_0126, 0);
        let surf = table(r);
        let mut rng = Rng::new(1);
        let out = e.bullet_impact(&mut rng, Some(&surf), &mut gas, &mut fx(), [10.0, 2.0, 3.0], [0.0, 0.0, 1.0], 10, [0.0, 0.0, -1.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        assert!(out.is_none());
        assert_eq!(e.decals.len(), 1);
        let d = &e.decals[0];
        assert_eq!(d.mesh, 0x0200_0126);
        assert!((d.pos[2] - (3.0 + DECAL_OFFSET)).abs() < 1e-6, "the decal sits 0.01 along the normal");
        assert_eq!(d.pos[0], 10.0);
        // Sister: no weapon bit 2 → flags 0 → nothing at all (and no RNG draws for gas/debris — the
        // only draws would be the ricochet's, which is gated on the flag too).
        let before = (rng.x, rng.y);
        let mut e2 = Effects::new();
        e2.bullet_impact(&mut rng, Some(&surf), &mut gas, &mut fx(), [1.0, 1.0, 1.0], [0.0, 1.0, 0.0], 10, [0.0, -1.0, 0.0], 0x0, 0, true, 8, crate::gas::HostCel::default());
        assert!(e2.decals.is_empty());
        assert_eq!((rng.x, rng.y), before, "flags 0: the game draws nothing at this impact");
    }

    /// The 200-entry FIFO: the 201st decal evicts the first [M Effect_InitLists 0x1cf268, Effect_Bullet
    /// 0x1cea20 — pool full → the OLDEST in-use node is unlinked].
    #[test]
    fn the_201st_decal_evicts_the_first() {
        let mut e = Effects::new();
        let mut gas = crate::gas::GasPool::default();
        let r = rec(0x0200_0126, 0);
        let surf = table(r);
        let mut rng = Rng::new(7);
        for k in 0..201 {
            e.bullet_impact(&mut rng, Some(&surf), &mut gas, &mut fx(), [k as f32, 0.0, 0.0], [0.0, 1.0, 0.0], 10, [0.0, -1.0, 0.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        }
        assert_eq!(e.decals.len(), 200, "the pool holds 200");
        assert_eq!(e.decals[0].pos[0], 1.0, "the first decal (k=0) was evicted");
        assert_eq!(e.decals[199].pos[0], 200.0, "the 201st (k=200) is the newest tail");
        assert_eq!(e.evicted, 1);
    }

    /// A hole keeps its pool node for life: the fresh nodes go 199, 198, … 0 (`DList_Init`'s
    /// `LList_Add` order, taken from the last-added end [M 0x1cc778, 0x1cca28, 0x1cccb8]); the
    /// 201st hole takes the evicted first hole's node 199, every other live hole stays in its
    /// node, and only that node is reported new [M Effect_Bullet 0x1cea20..0x1cea58].
    #[test]
    fn a_hole_keeps_its_pool_node_and_only_the_evicted_node_is_reused() {
        let mut e = Effects::new();
        let mut gas = crate::gas::GasPool::default();
        let surf = table(rec(0x0200_0126, 0));
        let mut rng = Rng::new(7);
        let shoot = |e: &mut Effects, rng: &mut Rng, gas: &mut crate::gas::GasPool, k: usize| {
            e.bullet_impact(rng, Some(&surf), gas, &mut fx(), [k as f32, 0.0, 0.0], [0.0, 1.0, 0.0], 10, [0.0, -1.0, 0.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        };
        for k in 0..200 { shoot(&mut e, &mut rng, &mut gas, k); }
        let before: Vec<f32> = (0..DECAL_POOL).map(|n| e.decal_in_node(n).unwrap().pos[0]).collect();
        assert_eq!(before[199], 0.0, "the first hole took node 199");
        assert_eq!(before[0], 199.0, "the 200th hole took node 0");
        shoot(&mut e, &mut rng, &mut gas, 200);
        assert_eq!(e.decal_in_node(199).unwrap().pos[0], 200.0, "the new hole took the evicted hole's node");
        for n in 0..DECAL_POOL - 1 {
            assert_eq!(e.decal_in_node(n).unwrap().pos[0], before[n], "node {n} kept its hole");
        }
        assert_eq!(e.decal_nodes_reused_since(200), vec![199], "only node 199 changed hands");
        assert_eq!(e.decal_node(e.decals.len() - 1), 199, "the newest hole's node");
        // Sister: the Vec index (the old draw-slot mapping) DOES move every hole.
        assert_ne!(e.decals[7].pos[0], e.decal_in_node(7).unwrap().pos[0], "the Vec index is not the node");
        shoot(&mut e, &mut rng, &mut gas, 201);
        shoot(&mut e, &mut rng, &mut gas, 202);
        assert_eq!(e.decal_nodes_reused_since(201), vec![197, 198]);
        assert_eq!(e.decal_in_node(197).unwrap().pos[0], 202.0);
    }

    fn rot_y(a: f32, t: V3) -> Mat4 {
        let (s, c) = a.sin_cos();
        [[c, 0.0, -s, 0.0], [0.0, 1.0, 0.0, 0.0], [s, 0.0, c, 0.0], [t[0], t[1], t[2], 1.0]]
    }

    /// A hole on a moving object rides it [M Effect_Bullet 0x1ce8a8:166-186 — node matrix made
    /// object-local; View_DrawObjectGlist 0x1e6780:66-74 — drawn at `node · obj+0x90`]: a door
    /// leaf hinged at the origin swings 90° and the hole swings with it. Sister: a cel-hosted hole
    /// at the same spot stays put.
    #[test]
    fn a_hole_on_a_door_swings_with_the_door() {
        let mut e = Effects::new();
        let door0 = rot_y(0.0, [0.0, 0.0, 0.0]);
        // A hole on the leaf's +z face, 1 m out from the hinge.
        e.add_decal([1.0, 1.5, 0.01], [0.0, 0.0, 1.0], 0x0200_0128, crate::world::HitHost::Object(0x100_0000_0007, door0));
        e.add_decal([1.0, 1.5, 0.01], [0.0, 0.0, 1.0], 0x0200_0128, crate::world::HitHost::Cel(Some(3)));
        let door1 = rot_y(std::f32::consts::FRAC_PI_2, [0.0, 0.0, 0.0]);
        e.follow_hosts(|id| (id == 0x100_0000_0007).then_some(door1));
        let on_door = &e.decals[0];
        // rot_y(90°) carries local +x to world −z and +z to world +x.
        assert!((on_door.pos[0] - 0.01).abs() < 1e-5 && (on_door.pos[1] - 1.5).abs() < 1e-5 && (on_door.pos[2] + 1.0).abs() < 1e-5,
            "the hole moved with the leaf: {:?}", on_door.pos);
        assert!((on_door.normal[0] - 1.0).abs() < 1e-5, "the normal turned with the leaf: {:?}", on_door.normal);
        assert_eq!(on_door.m[3][0], on_door.pos[0], "the draw matrix carries the new position");
        assert_eq!(e.decals[1].pos, [1.0, 1.5, 0.01], "sister: a cel-hosted hole never moves");
        // The leaf back at rest: the hole is back where it was made.
        e.follow_hosts(|_| Some(door0));
        assert!((e.decals[0].pos[0] - 1.0).abs() < 1e-5 && (e.decals[0].pos[2] - 0.01).abs() < 1e-5);
    }

    /// A host that leaves the world frees its holes (`Effect_RemoveSObjs` 0x1cf360 on the object's
    /// delete); the freed node joins the free list behind the never-used nodes
    /// (`DList_RemoveFromInUse2Free` 0x1cc868 links it at the far end from the take end), so the
    /// next hole takes a fresh node. Sister: a kept host keeps its hole.
    #[test]
    fn a_deleted_host_frees_its_holes_and_fresh_nodes_go_first() {
        let mut e = Effects::new();
        let m = rot_y(0.0, [5.0, 0.0, 5.0]);
        e.add_decal([5.0, 1.0, 5.01], [0.0, 0.0, 1.0], 1, crate::world::HitHost::Object(1, m)); // node 199
        e.add_decal([6.0, 1.0, 5.01], [0.0, 0.0, 1.0], 1, crate::world::HitHost::Object(2, m)); // node 198
        e.follow_hosts(|id| (id == 2).then_some(m));
        assert_eq!(e.decals.len(), 1, "host 1 is gone: its hole is freed");
        assert_eq!(e.decals[0].node, 198, "sister: host 2's hole stays in its node");
        assert!(e.decal_in_node(199).is_none());
        e.add_decal([0.0; 3], [0.0, 1.0, 0.0], 1, crate::world::HitHost::Cel(None));
        assert_eq!(e.decals[1].node, 197, "a fresh node before the returned one");
    }

    /// A debris piece keeps its draw slot until it dies: the death of an earlier piece leaves the
    /// later ones in their slots, and a new piece takes a slot no live piece holds. Sister: the
    /// list index (the old slot mapping) shifts on the death.
    #[test]
    fn a_debris_piece_keeps_its_draw_slot() {
        let mut e = Effects::new();
        let piece = |x: f32, life: i16| Debris { pos: [x, 0.0, 0.0], vel: [0.0; 3], spin: [0.0; 3], life, scale: 0.45, mesh: 7, alpha: 0xff, draw: DrawSlot::default() };
        e.debris.extend([piece(0.0, 2), piece(1.0, 50), piece(2.0, 50)]);
        let mut blast = vec![crate::shatter::Debris { model: 9, pos: [3.0, 0.0, 0.0], rot: [0.0; 3], vel: [0.0; 3], spin: [0.0; 3], life: 50, fade: 0xff, e8: 1.0, draw: DrawSlot::default() }];
        e.assign_debris_slots(&mut [], &mut blast);
        let slot_of = |e: &Effects, x: f32| e.debris.iter().find(|d| d.pos[0] == x).map(|d| d.draw.slot);
        assert_eq!((slot_of(&e, 0.0), slot_of(&e, 1.0), slot_of(&e, 2.0), blast[0].draw.slot), (Some(0), Some(1), Some(2), 3));
        let made0 = e.debris_made;
        for _ in 0..2 { e.update(); }
        assert_eq!(e.debris.len(), 2, "the first piece died");
        // Sister: the piece at list index 0 is now a different piece.
        assert_eq!(e.debris[0].pos[0], 1.0, "the list shifted");
        assert_eq!(slot_of(&e, 1.0), Some(1), "piece 1 kept slot 1");
        e.debris.push(piece(9.0, 50));
        e.assign_debris_slots(&mut [], &mut blast);
        assert_eq!(slot_of(&e, 9.0), Some(4), "next-fit: the new piece does not take the just-freed slot 0");
        let map = debris_slot_map(&e.debris, &[], &blast);
        assert_eq!(map[1], Some((0, 0)));
        assert_eq!(map[3], Some((2, 0)));
        assert_eq!(map[0], None, "the dead piece's slot draws nothing");
        assert_eq!(debris_slots_new_since(&e.debris, &[], &blast, made0), vec![4], "only the new piece's slot is new");
    }

    /// `Debris_Create` 0x1aaa00's draw count and physics: count = Rand_Rand(5)+1, per particle 5 draws
    /// (scale, life, 3 velocity) + 3 spin = 8. Sister: debris dies at life 0 and gains gravity.
    #[test]
    fn debris_draw_order_and_update() {
        let mut e = Effects::new();
        let mut gas = crate::gas::GasPool::default();
        let r = rec(0, 0);
        let surf = table(r);
        let mut rng = Rng::new(3);
        e.bullet_impact(&mut rng, Some(&surf), &mut gas, &mut fx(), [0.0; 3], [0.0, 1.0, 0.0], 10, [0.0, -1.0, 0.0], 0x2, 0, true, 8, crate::gas::HostCel::default());
        assert!(!e.debris.is_empty(), "stone debris spawns");
        assert!(e.debris.len() <= 6, "count = Rand_Rand(5)+1 ≤ 6");
        for d in &e.debris {
            assert!((0.4..=0.5).contains(&d.scale), "scale = 0.4 + FRand(0.1) [M 0x1aaad8]");
            assert_eq!(d.alpha, 0xff);
        }
        let n = e.debris.len();
        for _ in 0..60 { e.update(); }
        assert!(e.debris.is_empty(), "life ≤ 55 ticks: all debris dead after 60");
        let _ = n;
    }

    /// `Effect_RicochetProb` 0x1cf290 + the reflection [M 0x12d704]: one `Rand_Rand(N)` draw when
    /// N != 0; a hit reflects the direction about the normal with the +0x1c restitution. Sister: a
    /// zero N draws nothing.
    #[test]
    fn ricochet_draws_once_and_reflects() {
        let mut gas = crate::gas::GasPool::default();
        // Force a hit: a big N and a fixed seed — the roll is Rand_Rand(N) & 8.
        let r = rec(0, 0xffff);
        let surf = table(r);
        // Debris disabled so only the ricochet draw happens.
        let r2 = EffectRecord { debris_enable: 0, ..rec(0, 0xffff) };
        let surf2 = table(r2);
        // A grazing shot (dot = −0.0995): the reflection removes the normal component (restitution ≈
        // 0 — the +0x1c bits are a denormal), keeping the tangent. A head-on shot would stop
        // (v' ≈ 0) — the game's own e≈0 does that, which is why ricochets read on graze hits.
        let dir = [1.0, 0.0, -0.1];
        let n = [0.0, 0.0, 1.0];
        let mut found = None;
        for seed in 0..64u32 {
            let mut e = Effects::new();
            let mut rng = Rng::new(seed);
            let before = (rng.x, rng.y);
            let out = e.bullet_impact(&mut rng, Some(&surf2), &mut gas, &mut fx(), [0.0; 3], n, 10, dir, 0x12, 0, true, 8, crate::gas::HostCel::default());
            assert_ne!((rng.x, rng.y), before, "N != 0: exactly one Rand_Rand draw (seeds advanced)");
            if let Some(d) = out { found = Some((seed, d)); break; }
        }
        let (_, d) = found.expect("some seed must roll a ricochet with N=0xffff");
        // The tangent (x) is preserved, the normal component is gone: dot(d, n) ≈ 0 and d points back
        // off the wall (d·dir's normal part flipped).
        assert!((d[0] - 0.995).abs() < 0.01, "the tangential component survives");
        assert!(d[2].abs() < 0.01, "the normal component is removed (e ≈ 0)");
        assert!((d[0] * d[0] + d[1] * d[1] + d[2] * d[2] - 1.0).abs() < 1e-4, "normalised");
        // Sister: N = 0 → no ricochet draw, no reflection. The stream still moves: the record's
        // kind-9 gas puff spawns through `Gas_Init`, whose spin and sign are shared-stream draws
        // [M 0x1aacc8:72-80 — unconditional for every puff; kind 9 takes the no-draw position arm
        // :105-141]. gas-rng-draws: the old pin read 0 total draws — that was only true while the
        // port's Gas_Init consumed nothing; Effect_RicochetProb itself still draws nothing, so the
        // twin replay of the puff's 2 draws must land on the exact state the call leaves.
        let r3 = EffectRecord { debris_enable: 0, ..rec(0, 0) };
        let surf3 = table(r3);
        let mut e3 = Effects::new();
        let mut rng3 = Rng::new(1);
        let mut twin = Rng::new(1);
        twin.frand(std::f32::consts::FRAC_PI_2 * crate::gas::REC_FRAME_RATE); // the spin [M 0x1ab060]
        twin.rand(0x100); // the sign coin [M 0x1ab08c]
        assert!(e3.bullet_impact(&mut rng3, Some(&surf3), &mut gas, &mut fx(), [0.0; 3], n, 10, dir, 0x12, 0, true, 8, crate::gas::HostCel::default()).is_none(), "N = 0: no ricochet, no gas");
        assert_eq!((rng3.x, rng3.y), (twin.x, twin.y), "N = 0: exactly the puff's 2 Gas_Init draws, no ricochet draw");
        assert_eq!(rng3.draws, 2);
    }

    /// `Vec_MakeOrthonormalBasis` 0x1e4b78: the basis sum is orthogonal to the normal.
    #[test]
    fn basis_sum_is_tangential() {
        for n in [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.6, 0.8, 0.0]] {
            let s = orthonormal_basis_sum(n);
            let dot = s[0] * n[0] + s[1] * n[1] + s[2] * n[2];
            assert!(dot.abs() < 1e-5, "u+v lies in the surface plane");
        }
    }

    /// The per-object light pick (`effect_light_picks`): a piece between two kind-1 lights picks
    /// THOSE two at its own position, in falloff order — not whatever is nearest the world origin,
    /// which is what the old per-group draw pick ran at. The empty-mask meshes accept none even
    /// with lights in range, and the gas puff's `| 0x120` mask excludes kind 1 [M 0x1aacc8:83].
    #[test]
    fn effect_light_pick_selects_at_the_piece_not_the_origin() {
        let mut drones = crate::drone::Drones::new(Vec::new(), Rng::new(1));
        // Two pieces: one of the kind-1-admitting debris mesh 0x0200012c (flags 0x020 [D census]),
        // one of the empty-mask mesh 0x02000200 (flags 0x321 [D census]).
        drones.effects.debris.push(Debris {
            pos: [10.5, 0.0, 0.0], vel: [0.0; 3], spin: [0.0; 3], life: 100, scale: 0.45,
            mesh: 0x0200_012c, alpha: 255, draw: DrawSlot::default(),
        });
        drones.effects.debris.push(Debris {
            pos: [10.5, 0.0, 0.0], vel: [0.0; 3], spin: [0.0; 3], life: 100, scale: 0.45,
            mesh: 0x0200_0200, alpha: 255, draw: DrawSlot::default(),
        });
        drones.effects.assign_debris_slots(&mut [], &mut []);
        // Three kind-1 lights: near the piece, farther from it, and at the world origin.
        let rl = |pos: [f32; 4]| crate::light::ResolvedLight {
            pos, track_pos: pos, rgb: [255.0, 200.0, 120.0], intensity: 2.0,
            inv_r2: 1.0 / 36.0, kind: 1, active: true, track: None,
        };
        let resolved = crate::light::ResolvedLights { lights: vec![
            rl([10.0, 0.0, 0.0, 1.0]),   // 0.5 from the piece
            rl([12.5, 0.0, 0.0, 1.0]),   // 2.0 from the piece
            rl([0.0, 0.0, 0.0, 1.0]),    // AT the origin — the old per-group pick centre
        ] };
        let sphere = |h: u32| match h {
            0x0200_012c => Some(([0.0; 3], 0.1, 0x020u16)),
            0x0200_0200 => Some(([0.0; 3], 0.1, 0x321u16)),
            0x0200_0273 => Some(([0.0, 1.097, 0.117], 1.742, 0x150u16)),
            0x0200_0039 => Some(([0.0; 3], 4.139, 0x329u16)),
            _ => None,
        };
        let picks = effect_light_picks(&drones, &resolved, &sphere);
        // Debris slot 0 (0x0200012c, mask 0x020): the two piece-nearest lights, nearest first.
        let d = picks.iter().find(|p| p.kind == EFFECT_PICK_DEBRIS && p.mesh == 0x0200_012c).expect("the 0x020 debris piece is picked");
        assert_eq!(d.slot, 0);
        assert_eq!(d.flags, 0x020, "the mesh word | Debris_Create's 0x20 [M 0x1aaa00:53]");
        assert!((d.sphere[0] - 10.5).abs() < 1e-4 && (d.sphere[3] - 0.045).abs() < 1e-4,
            "the pick runs at the piece's own sphere: centre = pos, radius = mesh radius x scale");
        assert_eq!(d.lights, [Some(0), Some(1)], "the two lights nearest the PIECE, in falloff order");
        // The pre-fix witness: at the world origin (the old group-pick centre) the SAME list picks
        // the origin light first — a different piece answer is the whole bug.
        let (lp, _) = resolved.pick2([0.0, 0.0, 0.0, 0.1], 0x20);
        assert_eq!(lp[0], [0.0, 0.0, 0.0, 1.0], "the origin-centred pick lands on the origin light");
        // The empty-mask piece accepts nothing at any centre.
        let e = picks.iter().find(|p| p.kind == EFFECT_PICK_DEBRIS && p.mesh == 0x0200_0200).expect("the 0x321 debris piece is picked");
        assert_eq!(e.lights, [None, None], "0x321 admits no kind: kind 0/2 lack 0x80, kind 1 is under 0x100 [M 0x1cb2e0:19-31]");
        // A smoke cloud carries the mesh's 1.1-unit local centre in the sphere [D census].
        drones.smoke_clouds.push(crate::grenade::SmokeCloud { pos: [5.0, 1.0, 5.0], mesh: 0x0200_0273, ticks: 100 });
        let picks = effect_light_picks(&drones, &resolved, &sphere);
        let c = picks.iter().find(|p| p.kind == EFFECT_PICK_SMOKE_CLOUD).expect("the cloud is picked");
        assert!((c.sphere[1] - 2.097).abs() < 1e-3, "cloud sphere centre = pos + the mesh's local centre, got {}", c.sphere[1]);
        assert_eq!(c.lights, [None, None], "the 0x150 cloud mask accepts nothing [D census]");
        // A gas puff's mask is the mesh word | 0x120: kind 1 is excluded even in range [M 0x1aacc8:83].
        drones.gas.puffs.push(crate::gas::Gas {
            pos: [10.5, 0.0, 0.0], vel: [0.0; 3], mesh: 0x0200_0039, scale: 1.0, scale_rate: 0.0,
            rgba: [128.0; 4], rgba_rate: [0.0; 4], spin: 0.0, life: 40.0, kind: 0,
            billboard: false, xray_only: false, hidden: false, born: 0, ceil_y: None, child: None,
            rot: [0.0; 3],
        });
        let picks = effect_light_picks(&drones, &resolved, &sphere);
        let g = picks.iter().find(|p| p.kind == EFFECT_PICK_GAS_PUFF).expect("the puff is picked");
        assert_eq!(g.flags, 0x329, "0x329 | 0x120 = 0x329: the Gas_Init OR carries 0x100");
        assert_eq!(g.lights, [None, None], "the puff mask excludes kind 1, so no light even in range");
    }
}
