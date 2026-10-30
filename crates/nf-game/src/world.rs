//! Static collision world: every placed object's collision geometry (map block 0x2E, docs/formats/collision.md)
//! transformed to world space. Used for line of sight and floors. The game tests rays against each cel's box tree
//! (`Intersect_RayGeom`); here each object keeps its triangles behind one AABB — same answers, simpler culling.

use nf_formats::level::{Instance, Level};
use nf_formats::placement::transform;
use std::collections::HashMap;

pub type V3 = [f32; 3];

#[derive(Clone)]
struct Object {
    lo: V3,
    hi: V3,
    tris: std::ops::Range<usize>,
}

/// The moving set: geometry, each object's id and matrices (last tick, this tick), and the matrices by id from the
/// last `set_dynamic` (for the next one's "last tick").
#[derive(Default)]
struct Moving {
    geo: Geo,
    movers: Vec<(u64, Mat4, Mat4)>,
    last: std::collections::HashMap<u64, Mat4>,
    /// The last call's `(id, hash)` parts, in order — the unchanged-set fast path's key (`set_dynamic`).
    last_parts: Vec<(u64, u32)>,
    /// The last call's shape-bearing parts `(id, hash, matrix)`, aligned with `geo.objects` — the
    /// per-part reuse key (a part whose triple is unchanged memcpy's its stored block).
    last_objects: Vec<(u64, u32, Mat4)>,
}

type Mat4 = nf_formats::anim::Mat4;

/// How the ground under a point moved over the last tick (`Control_InheritVelocity` 0x133e88): the translation, the
/// change of heading (atan2 of the matrix's +Z x / z) and the object's position (the pivot of that turn).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundMotion { pub delta: V3, pub turn: f32, pub pivot: V3 }

/// The game's `Collide_LineOfSight` 0x1edf50 filter masks (HITTEST+0x98), each including the
/// wrapper's own `| 4` [M disasm 0x176fec/0x1770b0 — docs/systems/drone-armed-peer.md:53-57].
/// Drop semantics: `Collide_Intersect` 0x1ebaa8:300-319 + `Collide_Pick` 0x1eb320:140-152 (see
/// `CollisionWorld::line_of_sight`).
pub const LOS_GUARD_VISION: u32 = 0xa27; // DroneVision_CanSeeObjectFrom 0x176f18:69/95 and its family (CanSeePosition 0x178438:70, LineOfSightToObject 0x178390:22, DroneCanSeeDrone 0x1760b0:80, NDrone2_CheckUnsurrender 0x14a168:30, HandleExplosives 0x1435f0:40, MessageObjVicinity 0x13a438:31 — the 0x500 arg is dead, class-5 blast takes the plain-0xa27 else branch)
pub const LOS_PLAYER_HEADROOM: u32 = 0xd; // Player_SSCrouch 0x1924c0:100-101 / Player_HandleJump 0x1916b0:161-162 mask 9 | 4
pub const LOS_PROBE_8: u32 = 0xc; // mask 8 | 4 — Player_SSGrapple 0x192860:86-88, Searchlight_Update 0x1cb900:338-340, Sensor_SetBeam 0x1af398:73-75, Bullet_homing 0x12f358:92-93
pub const LOS_SENSOR: u32 = 0xe; // Sensor_InCone 0x1af4f0:84-86 / Check_AutoAim 0x1a73f8:164-165 / Bullet_DoTrails flash 0x12d890:343-345 mask 0xa | 4
pub const LOS_TURRET_TRACK: u32 = 0x72f; // GT_Track 0x17f9c8:258 — the target scan's per-candidate `Collide_LineOfSight(from, cand+0x80, level, obj, cand, 0x72f)` filter (already includes the wrapper's | 4)
/// The per-caller masks are not applied yet [S]: with them on, guard cover, turret shots and
/// several real-path walks broke; `line_of_sight` keeps the static-world segment test until then.
pub const LOS_MASKS_ON: bool = false;
/// The normalized-spine head-on cull [M Intersect_CylGeom 0x109da0:302-322]. level-0a-office-hatch-9
/// measured the arms' orientation: the game skips the horizontal face BELOW the spine (the
/// corner walk's y-0.19 floor-band lifter — the port dithered 1.13→1.21 on it while the game
/// crosses 1.127→1.120 [L fresh boot]) and keeps the one above/behind (the stair-step climb).
/// With f the A-from crossing fraction, a below-spine plane clamps to f=1 and an above-spine
/// one to f=0, so the skip arms are (up-face ∧ f>0.5) ∨ (down-face ∧ f<0.5) — the previous
/// arms had the f halves swapped. Still OFF: with the corrected arms the corner walk's
/// wedge MOVES (the try_walk_to line stalls at (−18.05, 5.22), 6000 ticks) — the global flip
/// needs the full real-path battery this window cannot run; the arms finding is pinned here
/// and in the report.
pub const HEAD_ON_CULL_ON: bool = false;

/// Scratch diagnostics gate (level-0a-office-hatch-9): `NF_CAPS_LOG=1` makes
/// `capsule_tri_contact` print every applied push (and every head-on-cull skip) as one
/// `eprintln` line — the per-tick capsule contact/push log the walk diagnoses run on. Off by
/// default; the hot path reads a `OnceLock` bool.
fn caps_log() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("NF_CAPS_LOG").is_ok())
}

/// Scratch gate switch (level-0a-office-hatch-9): `NF_CAPS_GATE=1` turns the `dot(q − B, n) > 0`
/// push-tail gate on (see the contact fn). Off by default — measured before it lands.
///
/// level-0a-office-hatch-10 settled the game's reference (HITTEST+0x50 = BL+0x790) and it is NOT
/// the spine bottom: boot (0,0,0) via the object-alloc memset [M control_create_object 0x1344e0:17;
/// L ee_start_0700000a and ee_start_05 both read exactly zero at BL+0x790 while +0x760/+0x770 hold
/// the seeded A/B], the only writer is the per-landed-hit `+= world DeltaP` [M Collide_Intersect
/// 0x1ebaa8:353-373 — no other access exists: an exhaustive ACTION.ELF scan of every
/// immediate-offset instruction (sw/swc1/lw/lwc1/sqc2 0x3E-0x3F/lqc2 0x37/addiu) touching
/// 0x790/0x788/0x798 finds only menu-stack spills at 0x2012c4-8], and the live mid-play value is a
/// drift vector ≈ (1.21, 9.08, −14.45), ~15 m from its owner [L ee_snapshot.bin: BL 0x16cf0f0,
/// Bond at (104.71, 24.97, 107.05)]. The decoded tail `dot(q − HITTEST+0x50, n) > 0` +
/// `|q − HITTEST+0x50| < radius` [M Intersect_CylGeom 0x109da0:420-457] therefore fires only
/// against triangles within `radius` of that drifted point — near-never on a walked line. The
/// ref=bottom gate below is NOT the game's shape (it ports a value the field never holds) and its
/// hatch-9 pins missed both ways; the port keeps it env-gated OFF until the game's real wall-stop
/// source is identified (the tail cannot be it — see docs/reports/level-0a-office-hatch-10.md).
static CAPS_GATE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static WALK_KILL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// The settle-5 radial-tail rule — level-0a-band-settle-6-c RESTRICTED state. The full suite
/// broke 21 tests across many levels with the settle-6-b all-on landing (Bond settles ~1 unit
/// high: the teleport-settle climbs the spine bottom to r above the floor — interactables_07000015
/// y 41.315 vs the walkway ~40.35 — so floor touch plates, walk-over pickups and use-key doors
/// fail). Measured on this round's repro runs, NONE of the four landed deltas is landable on the
/// port's substitute tail today:
///   — D4 (the always-on dot gate [M 0x10a568]) turns the behind-plane wedge faces inert; the
///     walk's settle then climbs the sloped skirts to the r-equilibrium (the 21-test break).
///   — D3 alone (the on-edge Q2 re-run [M 0x10a344-0x10a4fc] + the second call's flags-99 gate
///     [M 0x10a510/0x10a528]) perturbs the edge-contact push ~2e-4 and re-enters the same climb
///     (interactables_07000015 8/12 red) while its 01-plate-walk fix stands [M the bisect].
///   — D1/D5 (the always-on v̂ tail, the |v|==0 zero quad [M 0x10a5bc]) shift the same settle.
/// The tail's gates live inside the accumulator-relative arm in the decompile (v = hitpoint −
/// HITTEST+0x50 [M 0x109da0:421-431]) whose |v| < radius gate near-never passes on a walked line
/// [L ee_snapshot drift ≈15 m from the owner] — the game's real walk-vertical is the feet band
/// [M Player_FeetOnPoint 0x194188] + the walked dy, not this tail. ALL FOUR deltas are therefore
/// REVERTED to the settle-3 merged shape (the 20 non-01 levels' green state); the tail operand
/// decode [M 0x10a53c-0x10a61c] and the feet-band port stay open work — see
/// docs/reports/level-0a-band-settle-6-c.md.
/// HELD BACK (env-gated, default off): the Dist2Tri flags-99 behind-plane skip on the FIRST
/// query [M 0x109770 + 0x10a33c `beq v1,99`]. It is [M]-faithful, but its landing wedges the
/// 01 escort walk: after guard 6 dried the gun, every nav leg out of (5.48, 6.67, 11.60) gave
/// up at the 3000-tick cap with segment_to_target=Some on all targets (the resupply pocket —
/// the push shape that used to carry Bond out of it is a behind-plane face the game skips).
/// Set NF_CAPS_D2 to measure it; the next round re-derives the 01 harness pins against it.
static CAPS_D2: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

#[derive(Default)]
struct Geo {
    tris: Vec<[V3; 3]>,
    /// Each triangle's surface byte (`HITDATA+0x50`; `& 0x3F` indexes `EffectInfo`).
    mats: Vec<u8>,
    /// Per triangle: the owning glist node's bullet-pass flag — the placement's glist flags bit
    /// 0x40 (`celglist+0x44` → `cel+0x94` [M parsemap_block_map_data_static 0x1d1998:103]).
    /// `Collide_Intersect` 0x1ebaa8:143-145 ORs it into `HITDATA+0x51` bit 1 on every hit of the
    /// node, and the bullet-family queries (HITTEST+0x98 bit 0x8 — the aim trace 0x20a/0x208,
    /// `Bullet_homing`'s LOS 8) drop such hits outright (disasm 0x1ebff0-0x1ec010), so bullets fly
    /// through the whole node. fix-foliage-bullets.
    pass: Vec<bool>,
    objects: Vec<Object>,
    /// Static only (`chunk`): bounds over runs of consecutive `objects` (`tris` = an object index range), so a query
    /// skips whole runs. Empty = test every object.
    groups: Vec<Object>,
    /// Static only (`chunk`): for each chunked run, the pushed object (instance) it came from — the
    /// cel-scope bucketing key (fix-safe08b). One entry per `objects` entry after `chunk`.
    owners: Vec<u32>,
}

impl Geo {
    fn push(&mut self, tris: impl IntoIterator<Item = ([V3; 3], u8)>) { self.push_flags(tris, false) }

    fn push_flags(&mut self, tris: impl IntoIterator<Item = ([V3; 3], u8)>, node_pass: bool) {
        if self.push_kept(tris, node_pass) { return; }
        self.objects.pop();
    }

    /// `push_flags` that always adds the object — an empty collision mesh too (an empty tri range
    /// with inverted bounds, which no box test meets and `chunk` turns into no runs). The scoped
    /// static build needs one object per pushed instance: `chunk`'s `owners` key IS the object
    /// index, and `build_scoped`'s per-instance cel list counts every instance with collision —
    /// dropping an empty one shifted every later owner by one (G-052: 07000009's security-room
    /// floor and ceiling were bucketed under the next-lower room cel, so the eye-cel tie-break
    /// rays missed the room and the lobby cel won). Returns whether any triangle was added.
    fn push_kept(&mut self, tris: impl IntoIterator<Item = ([V3; 3], u8)>, node_pass: bool) -> bool {
        let start = self.tris.len();
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for (w, m) in tris {
            for p in &w { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } }
            self.tris.push(w);
            self.mats.push(m);
            self.pass.push(node_pass);
        }
        self.objects.push(Object { lo, hi, tris: start..self.tris.len() });
        self.tris.len() > start
    }

    /// Whether a hit on triangle `k` carries the game's bullet-pass bit (`HITDATA+0x51` bit 1):
    /// the node's glist flags bit 0x40 OR the surface byte's bit 0x40 — `Collide_Intersect`
    /// 0x1ebaa8 sets the flag from either (the node word at :143-145, the surface byte at
    /// disasm 0x1ebf78-0x1ebf80, no query-flag gate on either). The surface byte's bit 0x80 does
    /// NOT set it for the bullet queries: that OR is guarded by HITTEST+0x98 bit 0x1
    /// (disasm 0x1ebf8c-0x1ebf94), which the 0x20a/0x208 bullet flags clear — their 50/50 stays
    /// in the walk's roll (`Collide_Update` 0x1ea398:217-220, `Bullet_CollisionHandler` 0x12cfd0).
    /// fix-foliage-bullets.
    fn bullet_pass(&self, k: usize) -> bool { self.pass[k] || self.mats[k] & 0x40 != 0 }

    /// Split every object into runs of `n` consecutive triangles with their own bounds. Static geometry only: its
    /// objects carry no identity, so the queries (`near`, `segment_in`) return exactly the same hits, but a
    /// segment no longer tests every triangle of a large room mesh whose overall box it touches. Mesh triangles
    /// are stored in strip order, so consecutive runs are spatially compact. (07000007: the per-drone `drawn`
    /// ray cost 12 of a 15 ms tick before this.)
    fn chunk(&mut self, n: usize) {
        let mut out = Vec::with_capacity(self.tris.len() / n + self.objects.len());
        let mut owners = Vec::with_capacity(self.tris.len() / n + self.objects.len());
        for (si, o) in self.objects.iter().enumerate() {
            let mut s = o.tris.start;
            while s < o.tris.end {
                let e = (s + n).min(o.tris.end);
                let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
                for t in &self.tris[s..e] { for p in t { for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); } } }
                out.push(Object { lo, hi, tris: s..e });
                owners.push(si as u32);
                s = e;
            }
        }
        self.objects = out;
        self.owners = owners;
        self.groups.clear();
        let mut s = 0;
        while s < self.objects.len() {
            let e = (s + 32).min(self.objects.len());
            let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
            for o in &self.objects[s..e] { for k in 0..3 { lo[k] = lo[k].min(o.lo[k]); hi[k] = hi[k].max(o.hi[k]); } }
            self.groups.push(Object { lo, hi, tris: s..e });
            s = e;
        }
    }
}

/// One block-0x21 portal as the two triangles `Intersect_Portal` (0x1ea7b8) tests (its `Index_189`
/// indexing: corners 0,1,2 then 0,2,3), with the two cells it joins.
#[derive(Clone)]
pub struct ScopePortal {
    pub tris: [[V3; 3]; 2],
    pub a: usize,
    pub b: usize,
    /// The portal's reference point: the 4 corners' average, stored at portal+0x70 by
    /// `build_alloc_portal` 0x1c8928 (`puVar2[0x1c..0x1e] = (c0+c1+c2+c3)·0.25`) — the point
    /// `Intersect_Portal` 0x1ea7b8's 0x800 path projects the query segment against (fix-08-collision2;
    /// the port used corner 0, which missed crossings on long doorways).
    pub centre: V3,
}

/// The cel graph the game scopes collision queries with: one bbox per runtime cel (the parse-wide
/// pCelList records) and the portals between them. Plain data — `nf-view` maps its parsed graph in
/// at `CollisionWorld::build_scoped`.
#[derive(Clone, Default)]
pub struct ScopeGraph {
    pub cells: Vec<([f32; 3], [f32; 3])>,
    pub portals: Vec<ScopePortal>,
    /// The runtime cel's room-cel bit (`cel+0x3c & 0x40000`): `build_FindCel` 0x1c8cb0's candidate
    /// walk collects ONLY room cels (both the `glb_world` `gbl_room_cels` branch and the per-world
    /// chain branch filter the flag), so the relink's FindCel fallback must too. fix-swim-entry.
    pub room: Vec<bool>,
}

/// The per-cell static buckets + the graph, built once (`build_scoped`).
struct ScopeData {
    graph: ScopeGraph,
    /// `stat.objects` (chunked-run) indices per cell.
    cells: Vec<Vec<u32>>,
    /// Runs whose instance mapped to no cell: tested by every scoped query (the port's safety net —
    /// the game leaves such objects unlinked and untested, but a mapping miss here must not eat a wall).
    always: Vec<u32>,
}

/// Static geometry, plus moving objects (script animations' discrete parts: docs/systems/script-player.md) placed each
/// tick with `set_dynamic` — `Collide_PickObj` (0x1eae80) tests any object linked into a cel with collision data
/// (`obj+0xD8`) whatever moves it.
pub struct CollisionWorld {
    stat: Geo,
    /// Objects whose glist flags carry 0x1000 (the "BlockingPoly_PlayerMissile" family): the game's
    /// `Collide_Pick` (0x1eb320 disasm 0x1eb480-0x1eb49c) skips collision nodes whose cel flags word
    /// (cel+0x94 = the glist flags) has bit 0x1000 when the query's HITTEST+0x98 carries bit 0x20 —
    /// the actor-movement queries — while ray/sight queries (flags 0x18 et al) still hit them. On
    /// 07000008 these objects are a bullet cage over the whole map (a lid at y 52.8, a floor at
    /// y −40 and two walls over the station walkway) that must not block Bond on foot [D level data;
    /// the level is playable in the game across that wall]. They stay solid to `segment` (bullets,
    /// sight, the floor rays).
    missile: Geo,
    dynamic: std::sync::RwLock<Moving>,
    /// Collision triangles (object space) of every hashed glist object, for `set_dynamic`.
    shapes: std::collections::HashMap<u32, Vec<[V3; 3]>>,
    shape_mats: std::collections::HashMap<u32, Vec<u8>>,
    /// The cel scope (`build_scoped`): when set, the actor queries (`capsule_push`, the floor ray,
    /// the take ray) test only the cels the game's straddle walk keeps — `Collide_StraddleCels`
    /// 0x1eac98 via `Intersect_Portal` 0x1ea7b8, cap 0x7e cels, the start cel always kept.
    scope: Option<ScopeData>,
}

/// Whether the segment `from + t·d`, t in [0, 1], passes through the box (slab test, a little padding).
/// The scalar stand-in for the game's slab tests: `ASH_Intersect_RayBox` 0x109ad0 (the VU0
/// ray-vs-leaf-box test `Intersect_RayGeom` 0x10a8c8 drives — origin +0x20, per-axis deltas
/// +0x40, length cap +0x8c, the parallel-axis ±inf stand-in DAT_0030c748 [M]) and the
/// `Intersect_BoxBox` overlap the box-tree walks use [M 0x1edc48].
fn seg_box(from: V3, d: V3, lo: V3, hi: V3) -> bool {
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        let (l, h) = (lo[k] - 1e-3, hi[k] + 1e-3);
        if d[k].abs() < 1e-12 { if from[k] < l || from[k] > h { return false; } continue; }
        let inv = 1.0 / d[k];
        let (mut a, mut b) = ((l - from[k]) * inv, (h - from[k]) * inv);
        if a > b { std::mem::swap(&mut a, &mut b); }
        t0 = t0.max(a); t1 = t1.min(b);
        if t0 > t1 { return false; }
    }
    true
}

/// The objects the segment `from → to` passes through the bounds of (through `groups` when present).
fn along<'a>(g: &'a Geo, from: V3, to: V3) -> impl Iterator<Item = &'a Object> + 'a {
    let d = sub(to, from);
    let all: Box<dyn Iterator<Item = &'a Object>> = if g.groups.is_empty() { Box::new(g.objects.iter()) } else {
        Box::new(g.groups.iter().filter(move |gr| seg_box(from, d, gr.lo, gr.hi)).flat_map(move |gr| g.objects[gr.tris.clone()].iter()))
    };
    all.filter(move |o| seg_box(from, d, o.lo, o.hi))
}

/// The objects whose bounds overlap `lo..hi` (through `groups` when present).
/// Architecture note: the game walks each object's collision-box tree collecting leaf boxes
/// into `ColBoxs`/`BoxCnt` — `RecurseBoxesBox` 0x1edc48 and `RecurseBoxesSph` 0x1edae0 (the
/// scalar pair `Intersect_CylTriGeom` 0x1e9a70:156 and `Intersect_PointGeom` 0x1e92d8:153
/// drive) and the VU0 `ASH_RecurseBoxesBox` 0x10ae50 (`Intersect_CylGeom` 0x109da0 drives)
/// [M]. The port parses the tree (`nf-formats/src/collision.rs`) but queries the flattened
/// triangle world through this AABB filter instead [S].
fn overlapping<'a>(g: &'a Geo, lo: V3, hi: V3) -> impl Iterator<Item = &'a Object> + 'a {
    let out = move |o: &Object| (0..3).any(|k| o.hi[k] < lo[k] || o.lo[k] > hi[k]);
    let all: Box<dyn Iterator<Item = &'a Object>> = if g.groups.is_empty() { Box::new(g.objects.iter()) } else {
        Box::new(g.groups.iter().filter(move |gr| !(0..3).any(|k| gr.hi[k] < lo[k] || gr.lo[k] > hi[k])).flat_map(move |gr| g.objects[gr.tris.clone()].iter()))
    };
    all.filter(move |o| !out(o))
}

/// First hit of `from → to` in one geometry set (fraction).
fn segment_in(g: &Geo, from: V3, to: V3) -> Option<f32> {
    let d = sub(to, from);
    let mut best = None::<f32>;
    for k in 0..3 { if from[k].min(to[k]).is_nan() { return None; } }
    let (lo, hi): (V3, V3) = ([from[0].min(to[0]), from[1].min(to[1]), from[2].min(to[2])], [from[0].max(to[0]), from[1].max(to[1]), from[2].max(to[2])]);
    let _ = (lo, hi);
    for o in along(g, from, to) {
        for t in &g.tris[o.tris.clone()] {
            let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
            let p = cross(d, e2);
            let det = dot(e1, p);
            if det.abs() < 1e-12 { continue; }
            let inv = 1.0 / det;
            let s = sub(from, t[0]);
            let u = dot(s, p) * inv;
            if !(0.0..=1.0).contains(&u) { continue; }
            let q = cross(s, e1);
            let v = dot(d, q) * inv;
            if v < 0.0 || u + v > 1.0 { continue; }
            let tt = dot(e2, q) * inv;
            if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b) { best = Some(tt); }
        }
    }
    best
}

fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

/// One capsule-vs-triangle contact — the per-triangle body shared by `capsule_push`,
/// `capsule_push_scoped` and the pick-contact queries (`Intersect_CylGeom` 0x109da0's per-triangle
/// loop: the crossing point of the spine with the face plane, the closest triangle point to it,
/// and when that lies on an edge the spine point nearest it; closer than `r` → pushed out along
/// the separation, each push applied before the next triangle). Returns false when the triangle is
/// skipped (AABB, water material, the head-on cull, or no contact within `r`); else applies the
/// push to `a`/`b`, adds it to `total`, and sets the surface flags:
/// - `walkable`: the port's pre-existing "a walkable face (|ny| > 0.7) touched below the bottom
///   end" (the `floor_ny = 1.0` fallback's signal);
/// - `pick`: the game's pick-contact bit (`Intersect_CylGeom` 0x10a72c-0x10a734 sets
///   `HITTEST+0x9c |= 1`, ORed per real contact into the real `+0x9c` by `Collide_Intersect`
///   0x1ebaa8 tail 0x1ec138-0x1ec150, which `Player_FeetOnPoint` 0x194188:176-185 grounds on when
///   nonzero): the bit block's `dot2 < 0` test — `R = copy+0x50 − (bottom + push)` where the
///   `copy+0x50` vector tracks the live bottom endpoint through the pick's DeltaP adds
///   [M `Collide_Intersect` 0x1ebaa8:353-373 — REAL+0x50 += DeltaP per real hit, and the per-hit
///   stack copy takes it fresh], so `R = −push_this_contact` and `dot2 < 0` reads
///   `push·axis > 0` (the axis = normalize(+0x20 top − +0x30 bottom) [M `Player_Collision`
///   0x194df0 case 0/6/3], world-up for an upright capsule). A wall contact's push is horizontal
///   (`push·axis = 0`, not > 0) and a head-bump's is downward, so neither grounds — falling past a
///   wall stays airborne, which the reverted `push != 0` arm got wrong (680's 6 merge failures).
///   The bit block's outer `0.7 < dot1` gate is NOT ported as a hard gate: read as `normal·axis`
///   it would refuse the 0700000c pier ramp (a ~58° face, `normal·axis ≈ 0.53`) that the game's
///   own ride grounds on (the ray tolerance is exceeded at the skin-rest equilibrium on that
///   steep a face, so the pick arm must fire); confirming the gate is an
///   open live check.
fn capsule_tri_contact(t: &[V3; 3], mat: u8, a: &mut V3, b: &mut V3, r: f32, grow: f32, total: &mut V3, walkable: &mut bool, pick: &mut bool, top_guard: bool, ref_v: V3, dynamic_obj: bool) -> bool {
    let tl = [0, 1, 2].map(|k| t[0][k].min(t[1][k]).min(t[2][k]));
    let th = [0, 1, 2].map(|k| t[0][k].max(t[1][k]).max(t[2][k]));
    let (sl, sh) = ([0, 1, 2].map(|k| a[k].min(b[k]) - grow), [0, 1, 2].map(|k| a[k].max(b[k]) + grow));
    if (0..3).any(|k| th[k] < sl[k] || tl[k] > sh[k]) { return false; }
    // Water surfaces never collide the player (the cel scoping — see capsule_push).
    if mat & 0x3f == 0xE { return false; }
    // NO mat-0x40 (glass) skip here. The body query never filter-rejects glass:
    // [M Collide_Filter 0x1ed888:7-9] rejects `(surf & 0x40) && (flags & 0x800)` — but the flags
    // word is HITTEST+0x98, and Player_Collision 0x194df0:88 writes it to **0** for every
    // substate (only substate 15 zipline writes 0x1c0, :382 — no 0x800 bit either). The 0x800
    // words at :205/:229/:257/:282 land in HITTEST+**0x94** = the SHAPE word (0x800 = cylinder,
    // read by Collide_Update 0x1ea398:78 and the Collide_Intersect 0x1ebaa8:181-296 dispatch),
    // not the filter flags. [L ee_start_05.bin BL 0x16cf0f0 (ground HITTEST 0x16cf830 ✓
    // docs/fields/obj-tag-player.md:52): +0x94 = 0x0800, +0x98 = 0x0000, +0x9a = 0x0000 —
    // the body query carries no 0x800 and its +0x9a gate bit 0x10 is clear, so
    // Intersect_CylGeom 0x109da0:533-540 never even calls the filter for the walk. Ray queries
    // carry the masks instead: the feet ray passes 0xc [M Player_FeetOnPoint 0x194188:100] whose
    // bit 0x8 rejects glass/0x80-surfs via the filter's HITDATA+0x51-or-surf 0x40/0x80 path, and
    // the bullet/sensor rays pass 0x800-family masks (the segment paths below). Glass is
    // therefore BODY-SOLID (the 0700001b boundary wall [L walkreal_0700001b_live] stops the
    // float); the 0c skylight-lantern panes' [L 2026-10-26] walk-through needs a different
    // mechanism than this per-triangle reject (the hatch-3 blanket skip broke six walkreal/npc
    // tests — reverted; see docs/reports/level-0c-display-hatch-4.md).
    let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
    let nl = dot(n, n).sqrt();
    if nl < 1e-12 { return false; }
    let n = [n[0] / nl, n[1] / nl, n[2] / nl];
    // The game's head-on cull [M Intersect_CylGeom 0x109da0:302-322, inside the |n·d| > 0.0002
    // branch]: the 0.999 compare is unit-scale, and the spine is NORMALIZED before the triangle
    // loop [M the Vec_Normalise 0x1e3000 call at 0x109f74, immediately before the per-box walk;
    // on the raw walk spine |d| = 0.7597 [L ee_start_05 BL 0x16cf0f0: HITTEST+0x30 − +0x20 =
    // −0.7597198] a raw-dot 0.999 threshold is unreachable — and the raw reading is refuted on
    // the real path: with it the walk cannot descend the 07000001 stairwell (exit_aim_01_live:
    // Bond floats at the stairwell head, the step behind/above pushing him down, grounded=false
    // oscillation — the game obviously walks stairs). Direction: the normalized vector points
    // DOWN (B − A; the UP normalisation feeds the walkable/pick gate at :473, and an UP cull dot
    // would cull the floor underfoot), f is the plane-crossing fraction along the raw B − A from
    // A = param_1[2] (clamped :310-320; scale-invariant), and the face is skipped when
    // (n·d̂ > 0.999 ∧ f > 0.5) ∨ (n·d̂ < −0.999 ∧ f < 0.5) (:321-322). The port's `a`/`b` ARE the
    // game's A/B in that order: walk A = HITTEST+0x20 = BLData+0x760 = pos + up·0.275 (the TOP),
    // B = HITTEST+0x30 = BLData+0x770 = pos + up·(0.55 − cc) [M Player_Collision 0x194df0 disasm
    // 0x195444/0x195464], climb A = pos+up·0.55, B = pos+up·(0.275−cc) [M 0x194df0 case 0/6,
    // |d| = 1.3097183] — f means the game's fraction from the top end. The cull therefore lives
    // for faces within ~2.6° of horizontal (|n·ŷ| > 0.999): a step behind/above (f = 0 < 0.5) is
    // skipped — stairs descend; a floor crossed in the spine's TOP half (f < 0.5) is jumped
    // through from below; the same face crossed in the BOTTOM half (f > 0.5) is solid. Faces the
    // old |n·y| > 0.7 + top-end-side rule culled in the 0.70-0.763 band (≈45-77° slopes above the
    // capsule) are TESTED here, exactly as the game tests them.
    // The previous one-sided rule stays in force until the game's cull is on (HEAD_ON_CULL_ON).
    if !HEAD_ON_CULL_ON && n[1].abs() > 0.7 && dot(n, sub(*a, t[0])) < 0.0 { return false; }
    let ab = sub(*b, *a);
    let denom = dot(n, ab);
    let mut f = 0.0f32;
    if denom.abs() > 0.0002 {
        f = (-(dot(n, sub(*a, t[0]))) / denom).clamp(0.0, 1.0);
        let ab_len = dot(ab, ab).sqrt();
        if ab_len > 1e-6 {
            let nd = denom / ab_len;
            if HEAD_ON_CULL_ON && ((nd > 0.999 && f < 0.5) || (nd < -0.999 && f > 0.5)) {
                if caps_log() {
                    eprintln!("[cap] CULL cent=({:.3},{:.3},{:.3}) n=({:.3},{:.3},{:.3}) nd={nd:.4} f={f:.3} a=({:.3},{:.3},{:.3})",
                        (t[0][0]+t[1][0]+t[2][0])/3.0, (t[0][1]+t[1][1]+t[2][1])/3.0, (t[0][2]+t[1][2]+t[2][2])/3.0, n[0], n[1], n[2], a[0], a[1], a[2]);
                }
                return false;
            }
        }
    }
    let mut p = if denom.abs() > 0.0002 {
        [a[0] + ab[0] * f, a[1] + ab[1] * f, a[2] + ab[2] * f]
    } else {
        let c = [0, 1, 2].map(|k| (t[0][k] + t[1][k] + t[2][k]) / 3.0);
        closest_on_segment(*a, *b, c)
    };
    // Delta 2 — HELD BACK, env-gated (see the CAPS_D2 doc above): Dist2Tri's flags-99 arm on
    // the first query [M 0x109770 — the signed plane distance `dot(p, plane) + w < 0` returns
    // flags 99 with the out-pointer NEVER written; the caller skips it outright at 0x10a33c
    // `beq v1,v0(99)` → 0x10a508]: the query strictly behind the face plane never pushes.
    if *CAPS_D2.get_or_init(|| std::env::var("NF_CAPS_D2").is_ok()) && dot(sub(p, t[0]), n) < 0.0 {
        return false;
    }
    let (q, on_edge) = closest_on_triangle(p, t);
    // level-0a-band-settle-6-c: the settle-6-b Delta 3 (the on-edge second query, the tail's
    // Q = Q2 [M 0x10a344-0x10a4fc] with the caller's flags-99 gate on the re-run
    // [M 0x10a510/0x10a528]) is REVERTED along with D1/D4/D5 — measured alone it still breaks
    // the teleport-settle class: the re-run's q = closest_on_triangle(p2) changes the edge
    // contact's v = p2 − q by ~2e-4 (the 07000015 walkway's tick-1 north-wall edge push x
    // −0.0005 vs master's −0.0003), which diverts the settle into the sloped-skirt wedge whose
    // repeated v̂ pushes climb the spine bottom to r above the floor (interactables_07000015
    // the_dead_ladder y 41.315 vs the walkway ~40.35; 8/12 tests in that file). Master's
    // through-push out of the wedge keeps every y-settle reachable. The 01-plate-walk need the
    // bisect proved (the second Dist2Tri call's flags-99 gate) is REAL [M] but cannot land on
    // the substitute tail without the walk's vertical coming from the game's own feet-band
    // mechanism — see docs/reports/level-0a-band-settle-6-c.md for the live-check list.
    if on_edge { p = closest_on_segment(*a, *b, q); }
    // The game's push-tail guard [M Intersect_CylGeom 0x109da0 push tail: `v = q − A`,
    // `if (0.0 < dot(v, B − A))` gates the whole `dist < r` push]: a triangle whose closest
    // point lies at/beyond the capsule's TOP end A never pushes. Without it the real-capsule
    // math pushes THROUGH thin slabs (the 02 hut deck's SW fin wedge [L probe_swim_pocket
    // PUSHDBG]). The horizontal faces ride the game's SEPARATE unguarded head-on path
    // [M the `0.7 < |dot(n, axis)|` re-push block after the guarded push], so the guard keys on
    // the oblique/vertical faces only; the SWIM path passes it (top_guard) and the walk/NPC
    // paths keep the unguarded through-push (escort_follow_07000001_live regressed with it on).
    if top_guard && n[1].abs() <= 0.7 && dot(sub(q, *a), sub(*b, *a)) <= 0.0 { return false; }
    let v = sub(p, q);
    let dist = dot(v, v).sqrt();
    // level-0a-band-settle-6-c RESTRICTION: the settle-6-b "always-on" dot gate (dot(v, n) <= 0
    // skip, [M 0x10a568]) is REVERTED here. The full suite broke 21 tests across many levels with
    // it (Bond settles ~1 unit high — the teleport-settle climbs the spine bottom to r above the
    // floor: interactables_07000015 y 41.315 vs the walkway ~40.35). Repro: with the gate, the
    // behind-plane face at the walkway's north wall goes inert (per the game's flags-99 rule
    // [M 0x109770 + 0x10a33c]) but the sloped skirt faces then push the spine up (dir = v̂,
    // mag = r − |v|, repeatedly) to the r-equilibrium; on master the same behind-plane contact
    // pushes THROUGH (out of the wedge) and every y-settle interaction stays reachable. The
    // always-on reading over-applies: in the decompile the dot test sits inside the accumulator-
    // relative arm (v = hitpoint − HITTEST+0x50 [M 0x109da0:421-431]) whose |v| < radius gate
    // near-never passes on a walked line [L ee_snapshot drift ≈15 m], so the game's tail does not
    // carry the always-on v̂-push this encoding gave it. See the D3-revert comment below for the
    // measured D3-alone result.
    if dist >= r { return false; }
    // Scratch gate (level-0a-office-hatch-9, NF_CAPS_GATE=1): the game's push tail requires
    // `0 < dot(q − ref, n)` [M Intersect_CylGeom 0x109da0:431 — v = hitpoint − HITTEST+0x50,
    // the dot against the face plane, the whole push inside the `>0` arm] AND `|q − ref| < radius`
    // [M 0x109da0:447], the push dir/mag `(q − ref)/|q − ref|·(radius − |q − ref|)`. hatch-10
    // settled HITTEST+0x50 as the lifetime push accumulator [L ee_snapshot: drift ≈15 m from the
    // owner] — NOT the spine bottom probed here; with the true ref the tail near-never fires on a
    // walked line, so the game's z 16.29 glide needs no gate (nothing pushes there at all) and the
    // port's wall-stop must come from a mechanism not yet identified. This ref=bottom probe stays
    // OFF: with it the corner crossed but the exit-y missed both ways (1.0655 / 1.2142 vs the
    // game's 1.120 [L fresh boot]) and the glide walk floated ungrounded (hatch-9's measurements).
    let gate_on = CAPS_GATE.get_or_init(|| std::env::var("NF_CAPS_GATE").is_ok());
    if *gate_on && n[1] <= 0.7 && dot(sub(q, *b), n) <= 0.0 { return false; }
    // level-0a-band-settle-6-c RESTRICTION (the Delta 1 + Delta 5 revert): the settle-6-b
    // re-encoding (dir = v̂ always; |v| == 0 → the zero quad [M 0x10a5bc `sq v0,0(sp)`]; the
    // settle-3 radial arm removed) is REVERTED to the settle-3 merged shape — the radial arm
    // (ref = HITTEST+0x50, the [L] boot-(0,0,0) lifetime push total, the only writer the add-back
    // 0x1ebaa8:353-373 [M]) behind its own |v| < radius + dot(v, n) > 0 gates, and the |v| == 0
    // spine-dir fallback. Master's shape was green on every level; the always-on v̂ tail lifted
    // the walk's vertical settle ~1 unit off the floor in 21 tests (the repro above). The tail's
    // [M 0x10a53c-0x10a61c] operand decode stays an open question for the live-check round.
    let spine_dir = if dist > 1e-6 { [v[0] / dist, v[1] / dist, v[2] / dist] } else if dot(n, sub(*a, q)) >= 0.0 { n } else { [-n[0], -n[1], -n[2]] };
    let vr = sub(q, ref_v);
    let dist_r = dot(vr, vr).sqrt();
    let radial = dist_r > 1e-6 && dist_r < r && dot(vr, n) > 0.0;
    let (dir, mag) = if radial {
        ([vr[0] / dist_r, vr[1] / dist_r, vr[2] / dist_r], r - dist_r)
    } else {
        (spine_dir, r - dist)
    };
    // level-0a-corner-push-model: the walk's BELOW-B edge-contact displacement skip. The game's
    // edge/vertex contacts (Dist2Tri flags 1/2, [M 0x109da0:333-368]) RE-AIM the probe p at the
    // spine point closest to the REF (HITTEST+0x50) before the tail; the tail's push is then
    // REF-relative (v = p − REF, gated dot(v,n) > 0 and |v| < radius [M 0x109da0:419-457]) — NOT
    // the (p − q) skin depenetration. For an edge contact whose closest point q sits BELOW the
    // capsule bottom B the re-aim clamps p to B (the game's closest-spine-point-to-a-far-below-REF
    // reads B too [L ee_start_0700000a +0x50 = (0,0,0) while the walked spine sits ~1.1 above the
    // floor]), and the port's fallback push then points UP-and-out of the end cap — a lift the
    // game's REF-relative tail does not produce on a walked line. That lift is the 0a corner
    // wedge's fatal member: the wedge's four per-tick contacts are the counter slab's west strips
    // (x −17.5, q at the strip mid-height) and the swung-open door #4 leaf's tip (x −18.4, the
    // below-B tip edge, push y +0.0165 [cap trace, this branch]); with the below-B member dropped
    // the remaining three sum (−0.060, 0, +0.065)/tick against the move (0.0154, −0.0985) — a
    // west-south DEFLECTION (the game's measured bend [L 2026-10-26 the path ends x −17.97]), not
    // a wedge, and the corner crosses at +76 ticks (the game's ~60 frames [L 2026-10-26]; the
    // NF_WALK_CLIP probe measured the same +76). The certified hall-wall rest contact is
    // FACE-INTERIOR (edge=false [cap trace; [L 2026-10-30] the game rests z 15.5904 = this push
    // model]) and never sees this arm. The walk path (top_guard = false) therefore keeps an edge
    // contact's SIGNALS (walkable/pick below) but drops the fallback DISPLACEMENT when q rode
    // below B; horizontal edge deflections (the wall-end caps the Q-Worm route jinks around) keep
    // their pushes; the swim path (top_guard = true, the game's real tail with the diagonal spine
    // [M the 02 fin wedge]) keeps everything, and the radial arm (the [M] tail's own REF-relative
    // shape) is untouched either way.
    // The face-class split [M the tail's own structure]: the game routes the OBLIQUE/VERTICAL
    // faces through the guarded REF-relative tail (the top_guard note above; the 0.7 <
    // |dot(n, axis)| head-on re-push is a SEPARATE unguarded block for the horizontal faces),
    // and that guarded tail is the one the [L] fresh-boot REF (HITTEST+0x50 = (0,0,0),
    // ee_start_0700000a) near-never arms — so the below-B edge lift the port's fallback
    // produces has no game counterpart only on the guarded class. The UP-FACING faces
    // (n[1] > 0.7 — the game's own walkable dot [M 0x10a650-0x10a674]) ride the unguarded
    // head-on re-push, whose port shape IS the fallback push: the step/tread lips the climbs
    // ride keep lifting (master's 40-red list — ladder_02, the walkreal stair lips,
    // pickups08 — is exactly the up-facing class). The 0a leaf tip is vertical (the guarded
    // class) and stays skipped; the corner crossing survives.
    // The marginal-edge skip (level-0a-corner-push-model-2): the walk path drops an
    // EDGE/VERTEX contact's fallback displacement when the capsule only GRAZES the edge —
    // penetration r − dist < 0.05. Measured classes: the 0a corner slot's edge contacts all
    // carry r − dist ≤ 0.0447 (d 0.5053-0.5335, r 0.55 [cap trace, this session]) and their
    // sum rides exactly anti-parallel to the walk move (the counter strips + the half-open
    // leaf's mid-height edges — the wedge), while the Q-Worm jink's cap contacts carry
    // r − dist ≥ 0.0505 (d 0.4576-0.4995 [cap trace, this session]) and are the deflection
    // the route needs. The game's own edge tail is REF-relative (the re-aim of p at the
    // spine point closest to HITTEST+0x50, then v = p − REF gated dot(v,n) > 0 and
    // |v| < radius [M 0x109da0:333-368, 419-457]) and the accumulator boots (0,0,0) [L
    // ee_start_0700000a] with ~15 m of lifetime drift [L ee_snapshot] — it near-never arms on
    // a walked line, so the game pushes NOTHING on a fresh marginal graze and the walk move
    // carries through (the game's corner bend [L 2026-10-26]); the deep edge deflections the
    // port's routes ride are the port's stand-in for the game's move-clip response. The
    // threshold 0.05 sits between the two measured bands; the face-interior contacts (the
    // certified wall rest [L 2026-10-30]) never see this arm.
    // The game's two-class push structure [M Intersect_CylGeom 0x109da0 + the head-on block]:
    // the HORIZONTAL faces (|dot(n, axis)| > 0.7) ride the SEPARATE unguarded head-on re-push
    // (no REF gate — the game's floor/step response; the port's fallback IS that shape), while
    // the OBLIQUE/VERTICAL faces ride the guarded REF-relative tail — the re-aim of p at the
    // spine point closest to HITTEST+0x50, then v = p − REF gated dot(v,n) > 0 and |v| < radius
    // [M 0x109da0:333-368, 419-457]. The accumulator boots (0,0,0) [L ee_start_0700000a] and
    // drifts ~15 m behind the owner [L ee_snapshot], so the guarded tail near-never arms on a
    // walked line: the port's edge-contact fallback on the guarded class has NO game
    // counterpart, and the 0a corner slot (the counter strips + the door leaf's face column —
    // every edge member vertical) rides exactly that phantom class. The radial arm (the [M]
    // tail itself, ref = the lifetime DeltaP accumulator the port tracks) stays live either
    // way; the head-on class keeps the fallback (the stair treads, the cap tops the routes
    // round). The face-INTERIOR contacts never see this arm (the certified wall rest [L
    // 2026-10-30] is interior and keeps its push).
    // The DYNAMIC-SOURCE edge skip (level-0a-corner-push-model-2): a walk-path EDGE/VERTEX
    // contact against a MOVED OBJECT's collision (the port's dynamic.geo — the swung door
    // leaves) drops the fallback displacement; the STATIC cel edges keep it. [M] basis: the
    // game's only displacement is the guarded REF-relative tail (Intersect_CylGeom
    // 0x109da0:419-457 — v = p − REF gated dot(v,n) > 0, |v| < radius; the head-on block at
    // :482-518 sets a flag, not a push), whose accumulator boots (0,0,0) [L ee_start_0700000a]
    // and drifts ~15 m behind the owner [L ee_snapshot] — it near-never arms on a walked line
    // for ANY contact. The port keeps the (p − q) fallback as the stand-in for the STATIC
    // cels' edges because the walks' step-ups, stair lips and ladder rails ride it (master's
    // 40-red list with the prior branch's blanket below-B skip: ladder_02, the walkreal
    // stair lips, pickups08). The 0a corner's wedge is different: its anti-walk members are
    // the counter strips' static edges PLUS the half-open door #4 leaf's face column — and
    // the game crosses the same corner WITH the leaf present [L 2026-10-26 the fresh-boot
    // walk, ~60 frames], so the moved leaf's edge contacts contribute nothing in the game.
    // The port's dynamic.geo is exactly the moved-object class (the doors' leaves); the
    // static/cel split is the caller's (capsule_pick_contact_scoped's stat vs dynamic loops).
    let below_b_edge_skip = !top_guard && on_edge && !radial && dynamic_obj;
    let push = if below_b_edge_skip { [0.0; 3] } else { [dir[0] * mag, dir[1] * mag, dir[2] * mag] };
    if caps_log() {
        eprintln!("[cap] PUSH cent=({:.3},{:.3},{:.3}) q=({:.3},{:.3},{:.3}) p=({:.3},{:.3},{:.3}) d={dist:.4} push=({:.4},{:.4},{:.4}) n=({:.3},{:.3},{:.3}) edge={on_edge} a=({:.3},{:.3},{:.3}) b=({:.3},{:.3},{:.3})",
            (t[0][0]+t[1][0]+t[2][0])/3.0, (t[0][1]+t[1][1]+t[2][1])/3.0, (t[0][2]+t[1][2]+t[2][2])/3.0,
            q[0], q[1], q[2], p[0], p[1], p[2], push[0], push[1], push[2], n[0], n[1], n[2], a[0], a[1], a[2], b[0], b[1], b[2]);
    }
    // The below-B displacement kill (level-0a-band-settle-2, re-landing band-settle's
    // experiment (a)): a contact whose closest point q sits well BELOW the capsule bottom B
    // loses its displacement but keeps the grounding signals. The game's own per-face gate is
    // `0 < dot(q − ref, n)` [M Intersect_CylGeom 0x109da0:421-431 — the whole push inside the
    // `>0` arm, ref = HITTEST+0x50], which for a walked-floor contact (q under the capsule,
    // ref above it) never pushes — the game's floor ride is the feet band alone [M
    // Player_FeetOnPoint 0x194188:106] and the walked dy itself carries the climb [L band-settle
    // finding 1: the game's walked dy grows over the flat slab]. The real ref is the lifetime
    // DeltaP accumulator [M 0x1ebaa8:353-373 the only writer; L ee_start boots (0,0,0)], whose
    // live walked value cannot be pinned statically — the ref≈B reading here is the port choice
    // …gated OFF by default (NF_WALK_KILL=1 to measure): with the kill on, 05 walks 8/8 green
    // and the 0a corner crosses (870 ticks, was a permanent wedge), but 01's
    // walk_from_the_spawn_to_the_exit_with_the_contact fails (the AND-touch never fires — the
    // contact's plate approach shifts) — the keep-01/05-green constraint wins until
    // that walk is re-verified. The ref≈B reading is the port choice [S] unverified, measured
    // against the walkreal battery. The walked floor sits 0.604 below B
    // (dy 1.0891 [L] − 0.4847 [M 0x194df0's B arm]) — the 0.5 carve-out keeps every shallower
    // contact (the stair/step edges the 01/05 climbs ride) pushing.
    let deep_below = *WALK_KILL.get_or_init(|| std::env::var("NF_WALK_KILL").is_ok())
        && q[1] < b[1] - 0.5;
    if !deep_below {
        for k in 0..3 { a[k] += push[k]; b[k] += push[k]; total[k] += push[k]; }
    }
    // The walkable gate is the game's dot1 [M Intersect_CylGeom 0x109da0 disasm 0x10a650-0x10a674:
    // `dot(face normal, axis) > 0.7` — the axis runs B→A (up), so UP-facing faces only; the old
    // `.abs()` let CEILING/underside faces ground the walk (level-14-full-walk-2: the mouths' hood
    // underside at 07000014 y ~3.5 froze the release fall 1.1 above the lane — a contact the
    // game's gate never sets walkable). Same sign discipline as the pick bit below.
    if n[1] > 0.7 && q[1] < b[1] { *walkable = true; }
    // The pick bit's dot1 gate [M Intersect_CylGeom 0x109da0 disasm 0x10a650-0x10a674: the bit
    // block requires `dot(face normal, capsule axis) > 0.7` — up-facing faces only; a wall's
    // horizontal normal and a ceiling's down normal never set the bit]. The dot2 gate
    // (`+0x50 − (B + push)` — 0x10a66c-0x10a6a0) is pass-global, not per-contact: HITTEST+0x50
    // is the lifetime push accumulator and the tick's pushes cancel between its operands, so
    // the queries apply it once as `pick_ref_y < b_pre_y` (see capsule_pick_contact_scoped).
    if n[1] > 0.7 { *pick = true; }
    true
}

/// The move-clip contact — `NF_WALK_CLIP=1`'s walk model (the walk-move-clip task's experiment;
/// default OFF, measured against the whole walkreal battery before anything lands). Per triangle
/// the game's contact test [M `Intersect_CylGeom` 0x109da0: the per-triangle plane-crossing query
/// point f clamped 0..1, the closest triangle point, the `dist < r` contact gate] with the radial
/// push tail left DEAD — its reference HITTEST+0x50 is the lifetime DeltaP accumulator [M
/// `Collide_Intersect` 0x1ebaa8:353-373; [L] ee_start_* boots (0,0,0), ee_snapshot drifts ~15 m
/// from the owner] so the tail never fires on a walked line — and the tick's move CLIPPED at the
/// contact instead of pushing out. Port choices ([S] unverified, this flag is the measurement):
/// - the clip retracts the tick's move so the closest spine point lands ON the contact face's
///   plane (the spine-plane rest — the game's own glide rest holds the spine ~0.011 inside the
///   wall plane [L fresh boot, the 16.29 line]; no radius skin);
/// - only face-interior contacts clip: an edge or a side overlap never displaces (the game walks
///   through the door[19] slab poke and the 0.88 funnel pinch the stand-ins pin — the dead tail);
/// - the head-on cull runs with the GAME's own pairing [M 0x109da0:321-322 — skip when
///   (n·d̂ > 0.999 ∧ f > 0.5) ∨ (n·d̂ < −0.999 ∧ f < 0.5); `capsule_tri_contact`'s
///   HEAD_ON_CULL_ON copy has the arms flipped and stays off], so stair descents skip the step
///   behind/above (f = 0 < 0.5 on a down spine) and a floor crossed in the spine's TOP half
///   (f < 0.5) is jumped through from below — the walked settle's climb; the BOTTOM half is
///   solid and clips (the landing).
/// Returns true when this triangle clipped the move (a/b already pulled back along `move_v`);
/// `walkable`/`pick` keep `capsule_tri_contact`'s contact signals.
#[allow(clippy::too_many_arguments)]
fn capsule_tri_clip(t: &[V3; 3], mat: u8, a: &mut V3, b: &mut V3, r: f32, grow: f32, move_v: V3, walkable: &mut bool, pick: &mut bool, top_guard: bool) -> bool {
    let tl = [0, 1, 2].map(|k| t[0][k].min(t[1][k]).min(t[2][k]));
    let th = [0, 1, 2].map(|k| t[0][k].max(t[1][k]).max(t[2][k]));
    let (sl, sh) = ([0, 1, 2].map(|k| a[k].min(b[k]) - grow), [0, 1, 2].map(|k| a[k].max(b[k]) + grow));
    if (0..3).any(|k| th[k] < sl[k] || tl[k] > sh[k]) { return false; }
    // Water surfaces never collide the player (the cel scoping — see capsule_push).
    if mat & 0x3f == 0xE { return false; }
    let n = cross(sub(t[1], t[0]), sub(t[2], t[0]));
    let nl = dot(n, n).sqrt();
    if nl < 1e-12 { return false; }
    let n = [n[0] / nl, n[1] / nl, n[2] / nl];
    // The port's one-sided floor rule (the same skip set as the push — see capsule_tri_contact).
    if n[1].abs() > 0.7 && dot(n, sub(*a, t[0])) < 0.0 { return false; }
    let ab = sub(*b, *a);
    let denom = dot(n, ab);
    let mut f = 0.0f32;
    if denom.abs() > 0.0002 {
        f = (-(dot(n, sub(*a, t[0]))) / denom).clamp(0.0, 1.0);
        let ab_len = dot(ab, ab).sqrt();
        if ab_len > 1e-6 {
            let nd = denom / ab_len;
            // The game's own head-on pairing [M 0x109da0:321-322].
            if (nd > 0.999 && f > 0.5) || (nd < -0.999 && f < 0.5) { return false; }
        }
    }
    let mut p = if denom.abs() > 0.0002 {
        [a[0] + ab[0] * f, a[1] + ab[1] * f, a[2] + ab[2] * f]
    } else {
        let c = [0, 1, 2].map(|k| (t[0][k] + t[1][k] + t[2][k]) / 3.0);
        closest_on_segment(*a, *b, c)
    };
    let (q, on_edge) = closest_on_triangle(p, t);
    if on_edge { p = closest_on_segment(*a, *b, q); }
    let v = sub(p, q);
    let dist = dot(v, v).sqrt();
    if dist >= r { return false; }
    if top_guard && n[1].abs() <= 0.7 && dot(sub(q, *a), sub(*b, *a)) <= 0.0 { return false; }
    // The contact signals (capsule_tri_contact's rules — see the walkable/pick docs there).
    if n[1] > 0.7 && q[1] < b[1] { *walkable = true; }
    if n[1] > 0.7 { *pick = true; }
    // THE CLIP: a face-interior contact whose face the move pushes INTO retracts the tick's move
    // to the face plane. d(t) = d0 + t·dot(n, move_v) from the closest spine point p; the move
    // fraction kept is t* = −d0/dot(n, move_v) ∈ [0, 1). Each clip applies before the next
    // triangle (the game's shifted-segment per-triangle flow [M 0x109da0's A/B += push]).
    if on_edge { return false; }
    let md = dot(n, move_v);
    if md > -1e-9 { return false; } // the move does not push into the face
    let d0 = dot(n, sub(p, t[0]));
    let s = (-d0 / md).clamp(0.0, 1.0);
    if s >= 1.0 { return false; }
    let dl = [move_v[0] * (s - 1.0), move_v[1] * (s - 1.0), move_v[2] * (s - 1.0)];
    for k in 0..3 { a[k] += dl[k]; b[k] += dl[k]; }
    true
}

/// The segment's AABB.
fn box_of(from: V3, to: V3) -> (V3, V3) {
    ([from[0].min(to[0]), from[1].min(to[1]), from[2].min(to[2])], [from[0].max(to[0]), from[1].max(to[1]), from[2].max(to[2])])
}

/// Whether the segment `from → to` crosses the triangle `t` (Möller–Trumbore, t in [0, 1]).
/// TWO-SIDED — the portal crossing's semantics; do not "fix" onto the ray walks' one-sided rule.
fn seg_tri_hit(from: V3, to: V3, t: &[V3; 3]) -> bool {
    let d = sub(to, from);
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 { return false; }
    let inv = 1.0 / det;
    let s = sub(from, t[0]);
    let u = dot(s, p) * inv;
    if !(0.0..=1.0).contains(&u) { return false; }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 { return false; }
    let tt = dot(e2, q) * inv;
    (0.0..=1.0).contains(&tt)
}

/// The ray-triangle fraction for the bullet walks (Möller–Trumbore, ONE-SIDED like every game ray
/// test [M Intersect_RayGeom 0x10a8c8:168: `n·dir >= 0` skips the triangle]) — the same arithmetic
/// `segment_in_sets_drop_pass` inlines, as a shared body so the scoped and flat bullet walks agree
/// term for term.
fn ray_tri_t(from: V3, d: V3, t: &[V3; 3]) -> Option<f32> {
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    if dot(cross(e1, e2), d) >= 0.0 { return None; } // back face [M 0x10a8c8:168]
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-12 { return None; }
    let inv = 1.0 / det;
    let s = sub(from, t[0]);
    let u = dot(s, p) * inv;
    if !(0.0..=1.0).contains(&u) { return None; }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 { return None; }
    let tt = dot(e2, q) * inv;
    (0.0..=1.0).contains(&tt).then_some(tt)
}

impl CollisionWorld {
    /// From the level's instances (as drawn: cel placements; skipped-cel blockers included — they block).
    pub fn build(level: &Level, instances: &[Instance]) -> CollisionWorld {
        Self::build_scoped(level, instances, None)
    }

    /// `build` with the cel scope. Each instance's runs join **its own runtime cel's bucket**: the
    /// instances are the cel placements in parse order (`Level::instances(true)`) and the scope
    /// graph's cells are `parse_cel_graph`'s one-cell-per-placement list in the same order, so the
    /// k-th pushed instance IS the k-th cell — `parsemap_block_map_data_static` 0x1d1998 creates
    /// one runtime cel per placement whose glist IS the placement's object, and the game's
    /// per-hit cel word (`HITDATA+0x54`, the input of `Collide_SphereIntersect` 0x1ec940's
    /// candidate-scoped queries and of the chain-owner pick `build_link_objects_to_rooms`
    /// 0x1c9150 makes at load) carries that cel. (The old smallest-volume-containing-bbox rule
    /// put a room's floor in whatever tiny generic bbox held the floor's centre — the 07000006
    /// guard-room floor under the table's bbox, the courtyard floor under the fountain's — so
    /// every candidate room's bucket read empty at `build_chains` time and the generics were
    /// left unchained: the fountain, the guard-room furniture and the lockers never drew.)
    /// fix-missing-objects. `attribute_scope` re-keys the generics' runs to their chain owners
    /// after `build_chains` (the game's final attribution); instances whose own cell is out of
    /// range stay in the `always` bucket.
    pub fn build_scoped(level: &Level, instances: &[Instance], graph: Option<ScopeGraph>) -> CollisionWorld {
        let mut stat = Geo::default();
        let mut missile = Geo::default();
        // Per pushed object: its own runtime cel (the pushed ordinal — see the doc above).
        let mut obj_cell: Vec<Option<u32>> = Vec::new();
        for inst in instances {
            let Some(c) = &level.maps[inst.map].objects[inst.object].collision else { continue };
            // NO glist-bit-0x1 skip here (the level-14-complete/2 skip, reverted): the game's static
            // collision is the parsemap 0x8000 cel gate plus two glist bits, and bit 0x1 is NOT one
            // of them. [M] parsemap_block_map_data_static 0x1d1998:92 allocates a runtime cel only
            // for a placement whose flags word carries 0x8000 (`cel+0x3c = flags & 0xffff` :102),
            // copying the object's glist word into `cel+0x94` (:103); [M] Collide_Pick 0x1eb320
            // — the candidate stage every query passes (the movement capsule rides
            // Collide_CylinderIntersect 0x1ecc00:61's kind 0x800) — skips a node ONLY on
            // `cel+0x94 & 0x20` (:140) and on `& 0x1000` when the query's HITTEST+0x98 carries
            // bit 0x20 (:145-151); no bit-0x1 test exists anywhere in the pick path. [D
            // probe14_plcflags on 07000014] the skipped set is real world art the game KEEPS:
            // AirConGrill1-4/SCCP_Grate (plc 0x00008019 — kind 0x19, 0x8000 set, 24 tris each,
            // glist 0x1), the fixed ServerRoomGlass1-5 panes, the labels, the chairbases — 96
            // cel-placements carry bit 0x1 with tris. The door twins (Door_Slide_*, kind 0xdb,
            // plc 0x000000db) and the Breakable windows (kind 0x20, plc 0x00000020) carry NO
            // 0x8000 — they never had a static cel, so skipping bit 0x1 "for them" solved a
            // non-problem while dropping the grills/glass. The grill plane at z=58.86 blocking
            // the west duct's south end is the GAME's own geometry: the route leaves through the
            // EAST duct mouth (past lid SP #328) and descends south to the stairwell slot —
            // see walkreal_07000014_live and docs/levels/07000014.md §1a.
            // [M] cel+0x94 bit 0x20 (the shadow/overlay cels: CargoArea*Shadows glist 0x21 [D
            // probe14_plcflags]) is the real never-picked bit (Collide_Pick :140) — skip it.
            if level.maps[inst.map].objects[inst.object].flags & 0x20 != 0 {
                continue;
            }
            // The node's bullet-pass flag: the placement's glist flags bit 0x40 (cel+0x94 [M
            // 0x1d1998:103]; Collide_Intersect 0x1ebaa8:143-145 turns every hit on the node into a
            // pass, and the bit-0x8 bullet queries drop those hits — bullets fly through the node).
            let node_pass = level.maps[inst.map].objects[inst.object].flags & 0x40 != 0;
            // The 0x1000-flagged objects (the player-missile cage) never block the actor capsule [M
            // Collide_Pick 0x1eb320 — the cel flags 0x1000 skip for queries with HITTEST+0x98 bit 0x20].
            if level.maps[inst.map].objects[inst.object].flags & 0x1000 != 0 {
                missile.push_flags(c.triangles_with_material().into_iter().map(|(t, m)| (t.map(|p| transform(&inst.matrix, p)), m)), node_pass);
                continue;
            }
            if graph.is_some() {
                obj_cell.push(Some(obj_cell.len() as u32));
            }
            stat.push_kept(c.triangles_with_material().into_iter().map(|(t, m)| (t.map(|p| transform(&inst.matrix, p)), m)), node_pass);
        }
        // Index collision shapes by hashcode, AND by (map,object) packed into a synthetic key for objects whose
        // hashcode is u32::MAX (anonymous objects — e.g. 4/5 doors on 07000001 have hashcode=MAX, so the hashcode
        // lookup fails and bullets pass through). The synthetic key is 0x80000000 | (map << 16) | object [S].
        let mut shapes: HashMap<u32, Vec<[V3; 3]>> = HashMap::new();
        let mut shape_mats: HashMap<u32, Vec<u8>> = HashMap::new();
        for (&h, &(m, o)) in &level.by_hash {
            if let Some(c) = level.maps[m].objects[o].collision.as_ref() {
                shapes.insert(h, c.triangles());
                shape_mats.insert(h, c.triangles_with_material().into_iter().map(|t| t.1).collect());
            }
        }
        // Also index by (map, object) for anonymous objects (hashcode == MAX) — doors placed by-index [S].
        for (m, map) in level.maps.iter().enumerate() {
            for (o, obj) in map.objects.iter().enumerate() {
                if obj.hashcode == u32::MAX {
                    if let Some(c) = obj.collision.as_ref() {
                        let key = 0x8000_0000 | ((m as u32) << 16) | o as u32;
                        shapes.insert(key, c.triangles());
                        shape_mats.insert(key, c.triangles_with_material().into_iter().map(|t| t.1).collect());
                    }
                }
            }
        }
        stat.chunk(16);
        missile.chunk(16);
        // The per-cell buckets over the chunked runs (fix-safe08b). Runs of an instance that mapped to
        // no cell (or built without a graph) test in every scoped query.
        let scope = graph.map(|graph| {
            let mut cells = vec![Vec::new(); graph.cells.len()];
            let mut always = Vec::new();
            for (run, src) in stat.owners.iter().enumerate() {
                match obj_cell.get(*src as usize).copied().flatten() {
                    Some(c) if (c as usize) < cells.len() => cells[c as usize].push(run as u32),
                    _ => always.push(run as u32),
                }
            }
            ScopeData { graph, cells, always }
        });
        CollisionWorld { stat, missile, dynamic: Default::default(), shapes, shape_mats, scope }
    }

    /// Re-keys the cel-scope buckets by an explicit per-instance attribution — the collision side
    /// of the game's per-cel attribution (`build_link_objects_to_rooms` 0x1c9150 hangs every
    /// generic's collision on the room whose chain holds it; the rooms' own geometry stays with
    /// the room). `instance_cell[i]` = the cell for the i-th pushed object (the `stat.owners`
    /// source index — instances that pushed static geometry, in push order); `None` leaves the
    /// runs in the `always` bucket (the actor queries' safety net). fix-portal-eyecel: the
    /// `build_FindCel` 0x1c8cb0 tie-break reads these buckets per candidate, so the buckets must
    /// carry the game's chain owners, not the smallest-volume centre rule `build_scoped` stands
    /// in with.
    pub fn attribute_scope(&mut self, instance_cell: &[Option<u32>]) {
        let Some(sc) = &mut self.scope else { return };
        let mut cells = vec![Vec::new(); sc.graph.cells.len()];
        let mut always = Vec::new();
        for (run, src) in self.stat.owners.iter().enumerate() {
            match instance_cell.get(*src as usize).copied().flatten() {
                Some(c) if (c as usize) < cells.len() => cells[c as usize].push(run as u32),
                _ => always.push(run as u32),
            }
        }
        sc.cells = cells;
        sc.always = always;
    }

    /// Whether the world carries a cel scope (`build_scoped` with a graph).
    pub fn has_scope(&self) -> bool { self.scope.is_some() }

    /// Debug introspection (level-15-ladder-3): the scope graph's cell boxes and room flags, so a
    /// test can ask which cel boxes contain a point without reaching into the private `ScopeData`.
    pub fn debug_scope_cells(&self) -> Vec<([f32; 3], [f32; 3], bool)> {
        match &self.scope {
            Some(sc) => sc.graph.cells.iter().enumerate()
                .map(|(i, (lo, hi))| (*lo, *hi, sc.graph.room.get(i).copied().unwrap_or(false)))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Debug introspection (fix-08-collision): for every cell's scope bucket plus the `always`
    /// remainder, how many chunked runs whose bounds meet `lo..hi` it holds. Probes the per-cel
    /// attribution without reaching into the private `ScopeData`.
    pub fn debug_bucket_owners(&self, lo: V3, hi: V3) -> Vec<(i32, usize)> {
        let meets = |r: u32| {
            let o = &self.stat.objects[r as usize];
            !(0..3).any(|k| o.hi[k] < lo[k] || o.lo[k] > hi[k])
        };
        let Some(sc) = &self.scope else { return Vec::new() };
        let mut out: Vec<(i32, usize)> = sc.cells.iter().enumerate()
            .map(|(c, runs)| (c as i32, runs.iter().copied().filter(|&r| meets(r)).count()))
            .collect();
        out.push((-1, sc.always.iter().copied().filter(|&r| meets(r)).count()));
        out.retain(|(_, n)| *n > 0);
        out
    }

    /// Debug introspection (fix-08-collision2): the straddle set a capsule query at `a..b` with
    /// radius `r` keeps from `cell` — the `Collide_StraddleCels` result the scoped passes test —
    /// so a missing portal crossing is probeable from a test.
    pub fn debug_straddle(&self, cell: usize, a: V3, b: V3, r: f32) -> Vec<usize> {
        let mut hit = Self::capsule_portal_hit(a, b, r);
        self.straddle(cell, &mut hit)
    }

    pub fn triangle_count(&self) -> usize { self.stat.tris.len() }

    /// Every static triangle whose object's box meets `lo`..`hi`, in stored winding order (the [D]
    /// evidence path: the game's plane is the stored normal = +cross(v1−v0, v2−v0), so the winding's
    /// direction IS the game's facing — `Intersect_RayGeom` 0x10a8c8 skips `n·dir >= 0`).
    pub fn tris_near(&self, lo: V3, hi: V3) -> Vec<[[f32; 3]; 3]> {
        let mut out = Vec::new();
        for o in overlapping(&self.stat, lo, hi) {
            out.extend(self.stat.tris[o.tris.clone()].iter().copied());
        }
        out
    }

    /// Probe support (level-14-complete-2): the missile-set (glist flags 0x1000) triangles whose
    /// object's box meets `lo`..`hi` — solid to the ray queries, skipped by the actor-movement
    /// capsule queries when HITTEST+0x98 carries bit 0x20 [M Collide_Pick 0x1eb320 disasm
    /// 0x1eb480-0x1eb49c].
    pub fn missile_tris_near(&self, lo: V3, hi: V3) -> Vec<[[f32; 3]; 3]> {
        let mut out = Vec::new();
        for o in overlapping(&self.missile, lo, hi) {
            out.extend(self.missile.tris[o.tris.clone()].iter().copied());
        }
        out
    }

    /// Probe support: the dynamic set's triangles (the hashed glist movers — doors and the
    /// scripted props) whose object's box meets `lo`..`hi`.
    pub fn dynamic_tris_near(&self, lo: V3, hi: V3) -> Vec<[[f32; 3]; 3]> {
        let d = self.dynamic.read().unwrap();
        let mut out = Vec::new();
        for o in overlapping(&d.geo, lo, hi) {
            out.extend(d.geo.tris[o.tris.clone()].iter().copied());
        }
        out
    }

    /// No geometry (tests).
    pub fn empty() -> CollisionWorld { Self::from_triangles(Vec::new()) }

    /// One object holding these world-space triangles (tests).
    pub fn from_triangles(tris: Vec<[V3; 3]>) -> CollisionWorld {
        let mut stat = Geo::default();
        stat.push(tris.into_iter().map(|t| (t, 0)));
        CollisionWorld { stat, missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(), scope: None }
    }

    /// One object holding these world-space triangles with surface bytes and per-triangle
    /// bullet-pass flags (tests; the live path derives the flag from the placement's glist flags).
    pub fn from_triangles_flags(tris: Vec<([V3; 3], u8, bool)>) -> CollisionWorld {
        let mut stat = Geo::default();
        let pass = tris.iter().map(|(_, _, p)| *p).collect::<Vec<_>>();
        stat.push_flags(tris.into_iter().map(|(t, m, _)| (t, m)), false);
        for (k, p) in pass.into_iter().enumerate() { stat.pass[k] = p; }
        CollisionWorld { stat, missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(), scope: None }
    }

    /// Glist object `hash`'s collision triangles in object space.
    pub fn shape(&self, hash: u32) -> Option<&[[V3; 3]]> { self.shapes.get(&hash).map(Vec::as_slice) }

    /// Whether glist object `hash` has collision geometry.
    pub fn has_shape(&self, hash: u32) -> bool { self.shapes.contains_key(&hash) }

    /// Replaces the moving objects: each `(id, glist hash, row-vector world matrix)`; the id (stable from tick to
    /// tick) pairs an object with its matrix of the previous call.
    pub fn set_dynamic(&self, parts: &[(u64, u32, Mat4)]) {
        let mut d = self.dynamic.write().unwrap();
        // Unchanged fast path: the same `(id, hash)` parts with the same matrices as the last call
        // (a door at rest, a static SS object — most ticks on most levels). The stored geo IS this
        // set's world-space triangles (the same matrix over the same object-space points gives the
        // same floats, in the same parts order) and `last` is already current. Only `movers` may
        // need the cheap no-transform rebuild (a matrix that moved last call and stopped: its
        // `prev` must read the now-current matrix). The probe's baseline put the full
        // transform+allocation rebuild at ~0.19 ms/tick on the moving-object levels (sim-tick-cost).
        let same = d.last_parts.len() == parts.len()
            && parts.iter().zip(d.last_parts.iter())
                .all(|((&(id, h, m)), &(lid, lh))| id == lid && h == lh && d.last.get(&id).is_some_and(|pm| *pm == m));
        if same {
            // movers must read `(id, cur, cur)` for the shape-bearing parts, in parts order.
            let (mut mi, mut ok) = (0usize, true);
            for &(id, h, m) in parts {
                if !self.shapes.contains_key(&h) { continue; }
                match d.movers.get(mi) { Some(&(mid, p, c)) if mid == id && p == m && c == m => {}, _ => { ok = false; break } }
                mi += 1;
            }
            if ok && mi == d.movers.len() { return; }
            d.movers = parts.iter().filter(|(_, h, _)| self.shapes.contains_key(h)).map(|&(id, _, m)| (id, m, m)).collect();
            return;
        }
        let (mut geo, mut movers, mut last) = (Geo::default(), Vec::new(), std::collections::HashMap::new());
        geo.tris.reserve(d.geo.tris.len());
        geo.mats.reserve(d.geo.mats.len());
        let mut last_objects: Vec<(u64, u32, Mat4)> = Vec::with_capacity(parts.len());
        for &(id, h, m) in parts {
            last.insert(id, m);
            let Some(t) = self.shapes.get(&h) else { continue };
            let n = geo.objects.len();
            let mats = self.shape_mats.get(&h);
            // Partial reuse: an unchanged (id, hash, matrix) part memcpy's its previous world-space
            // block (the same floats the transform re-derives) — on the animating levels the
            // resting doors/static SS objects were paying a full re-transform every tick
            // (sim-tick-cost's `drones: set_dynamic` 0.4–0.6 ms on 07000008/16/1b). The `pass`
            // bytes ride along (the dynamic build always pushes node_pass = false, the same
            // value a re-transform would store).
            let reused = d.last_objects.get(n).copied().is_some_and(|(pid, ph, pm)| pid == id && ph == h && pm == m)
                && n < d.geo.objects.len();
            if reused {
                let po = &d.geo.objects[n];
                let (start, end) = (po.tris.start, po.tris.end);
                geo.tris.extend_from_slice(&d.geo.tris[start..end]);
                geo.mats.extend_from_slice(&d.geo.mats[start..end]);
                geo.pass.extend(std::iter::repeat(false).take(end - start));
                geo.objects.push(Object { lo: po.lo, hi: po.hi, tris: geo.tris.len() - (end - start)..geo.tris.len() });
            } else {
                geo.push(t.iter().enumerate().map(|(k, t)| (t.map(|p| transform(&m, p)), mats.and_then(|v| v.get(k).copied()).unwrap_or(0))));
            }
            if geo.objects.len() > n { movers.push((id, d.last.get(&id).copied().unwrap_or(m), m)); last_objects.push((id, h, m)); }
        }
        *d = Moving {
            geo,
            movers,
            last,
            last_parts: parts.iter().map(|&(id, h, _)| (id, h)).collect(),
            last_objects,
        };
    }

    /// When the first collision surface along `from → to` belongs to a moving object, how that object moved over the
    /// last tick (`Player_FeetOnPoint` 0x194188 → `Control_InheritVelocity`).
    pub fn ground_motion(&self, from: V3, to: V3) -> Option<GroundMotion> {
        let stat = self.stat_segment(from, to);
        let d = self.dynamic.read().unwrap();
        let mut best: Option<(f32, usize)> = None;
        for (i, o) in d.geo.objects.iter().enumerate() {
            let g = Geo { tris: d.geo.tris[o.tris.clone()].to_vec(), mats: d.geo.mats[o.tris.clone()].to_vec(), objects: vec![Object { lo: o.lo, hi: o.hi, tris: 0..o.tris.len() }], groups: Vec::new(), owners: Vec::new(),
                pass: d.geo.pass[o.tris.clone()].to_vec(), };
            if let Some(t) = segment_in(&g, from, to) { if best.is_none_or(|b| t < b.0) { best = Some((t, i)); } }
        }
        let (t, i) = best?;
        if stat.is_some_and(|s| s < t) { return None; }
        let (_, prev, cur) = d.movers[i];
        let heading = |m: &Mat4| m[2][0].atan2(m[2][2]);
        let mut turn = heading(&cur) - heading(&prev);
        if turn > std::f32::consts::PI { turn -= std::f32::consts::TAU; } else if turn < -std::f32::consts::PI { turn += std::f32::consts::TAU; }
        Some(GroundMotion { delta: [cur[3][0] - prev[3][0], cur[3][1] - prev[3][1], cur[3][2] - prev[3][2]], turn, pivot: [cur[3][0], cur[3][1], cur[3][2]] })
    }

    /// The dynamic objects' `(id, lo, hi)` boxes this tick — the contact-list source for
    /// `SS_Operate` 0x1a9df8 (the game's collision pass registers every overlapping body into the
    /// SS object's `obj+0xD0` contact list; the port queries the same registered dynamic set).
    pub fn dynamic_boxes(&self) -> Vec<(u64, V3, V3)> {
        let d = self.dynamic.read().unwrap();
        d.geo.objects.iter().zip(&d.movers).map(|(o, (id, _, _))| (*id, o.lo, o.hi)).collect()
    }

    fn stat_segment(&self, from: V3, to: V3) -> Option<f32> { segment_in(&self.stat, from, to) }

    /// Every triangle of the objects whose box meets `lo`..`hi`, static first.
    /// Every triangle whose object's bounds the segment `from → to` passes through (`near` for segments).
    fn near_seg(&self, from: V3, to: V3, mut f: impl FnMut(&[V3; 3], u8)) {
        self.near_seg_drop_pass(from, to, false, f)
    }

    /// `near_seg` with the bullet-family's pass-flag drop (HITTEST+0x98 bit 0x8 [M 0x1ebff0]).
    fn near_seg_drop_pass(&self, from: V3, to: V3, drop_pass: bool, mut f: impl FnMut(&[V3; 3], u8)) {
        let dynamic = self.dynamic.read().unwrap();
        for g in [&self.stat, &self.missile, &dynamic.geo] {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    if drop_pass && g.bullet_pass(k) { continue; }
                    f(&g.tris[k], g.mats[k]);
                }
            }
        }
    }

    /// The actor-capsule neighbourhood: static geometry MINUS the 0x1000 missile cage (the game's
    /// actor queries skip those nodes [M Collide_Pick 0x1eb320:0x1eb480-0x1eb49c]), dynamic objects included.
    fn near_actor(&self, lo: V3, hi: V3, mut f: impl FnMut(&[V3; 3], u8)) {
        let dynamic = self.dynamic.read().unwrap();
        for g in [&self.stat, &dynamic.geo] {
            for o in overlapping(g, lo, hi) {
                for k in o.tris.clone() { f(&g.tris[k], g.mats[k]); }
            }
        }
    }

    fn near(&self, lo: V3, hi: V3, mut f: impl FnMut(&[V3; 3], u8)) {
        let dynamic = self.dynamic.read().unwrap();
        for g in [&self.stat, &self.missile, &dynamic.geo] {
            for o in overlapping(g, lo, hi) {
                for k in o.tris.clone() { f(&g.tris[k], g.mats[k]); }
            }
        }
    }

    /// `Collide_StraddleCels` (0x1eac98): the cells the query keeps — the start cell plus, BFS, every
    /// cell across a portal the query shape straddles (`Intersect_Portal` 0x1ea7b8), cap 0x7e cells,
    /// each cell entered once. `portal_hit` = the shape test for the query's kind.
    fn straddle(&self, start: usize, portal_hit: &mut dyn FnMut(&ScopePortal) -> bool) -> Vec<usize> {
        let Some(sc) = &self.scope else { return Vec::new() };
        let mut seen = vec![false; sc.graph.cells.len()];
        let mut out = Vec::new();
        let start = if start < seen.len() { start } else { return out };
        seen[start] = true;
        out.push(start);
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start);
        while let Some(c) = queue.pop_front() {
            if out.len() >= 0x7e { break; }
            for p in &sc.graph.portals {
                let dest = if p.a == c { Some(p.b) } else if p.b == c { Some(p.a) } else { None };
                let Some(dest) = dest else { continue };
                if dest >= seen.len() || seen[dest] { continue; }
                if !portal_hit(p) { continue; }
                seen[dest] = true;
                out.push(dest);
                queue.push_back(dest);
                if out.len() >= 0x7e { break; }
            }
        }
        out
    }

    /// The `stat.objects` runs the scoped query tests: the straddle cells' buckets plus the `always`
    /// remainder (`None` when there is no scope — the caller falls back to the flat query).
    fn scoped_runs(&self, cell: usize, portal_hit: &mut dyn FnMut(&ScopePortal) -> bool) -> Option<Vec<u32>> {
        let sc = self.scope.as_ref()?;
        let mut taken = vec![false; self.stat.objects.len()];
        let mut out = Vec::new();
        for c in self.straddle(cell, portal_hit) {
            for &r in &sc.cells[c] { if !taken[r as usize] { taken[r as usize] = true; out.push(r); } }
        }
        for &r in &sc.always { if !taken[r as usize] { taken[r as usize] = true; out.push(r); } }
        Some(out)
    }

    /// Diagnostic (level-11-stairwell-bullet-wall-4): the scoped bucket for `cell` as the ray
    /// `from → to` straddles it — the same run list `segment_hit_scoped` tests, as (run, owner
    /// pushed-instance ordinal) pairs so a probe can name every owner. Read-only.
    pub fn scoped_run_owners(&self, cell: usize, from: V3, to: V3) -> Option<Vec<(u32, u32)>> {
        let runs = self.scoped_runs(cell, &mut Self::ray_portal_hit(from, to))?;
        Some(runs.into_iter().map(|r| {
            let o = &self.stat.objects[r as usize];
            let owner = if o.tris.start < self.stat.owners.len() { self.stat.owners[o.tris.start] } else { u32::MAX };
            (r, owner)
        }).collect())
    }

    /// The bullet-family segment over the game's cel-scoped pick: `segment_bullet` MINUS every
    /// static triangle outside the straddle cells — `Collide_Pick` 0x1eb320:125 takes its candidate
    /// nodes ONLY from `Collide_StraddleCels` 0x1eac98 (the BFS from the query's cel through the
    /// portals the segment crosses), and the flying bullet re-feeds that start cel every tick
    /// [M Bullet_update 0x12e7b0 disasm 0x12f0cc-0x12f0d8]. The missile cage and the dynamic set
    /// stay in (the ForcedList walk runs when HITTEST+0x98's bit 0x10 is clear — the bullet's
    /// +0x98 = 0 [M 0x12f040]; the cage rides the with-missile half the flat bullet query uses).
    /// Falls back to the flat `segment_bullet` when there is no scope or the cell is unknown — the
    /// caller's safety net, same as the scoped actor queries.
    pub fn segment_bullet_scoped(&self, cell: Option<usize>, from: V3, to: V3) -> Option<f32> {
        let Some(cell) = cell else { return self.segment_bullet(from, to) };
        // An out-of-range cell is a caller bug: the flat answer is the conservative fallback (the
        // scoped actor queries instead test nothing — their callers keep a valid `bond.cel`).
        let in_range = self.scope.as_ref().is_some_and(|sc| cell < sc.graph.cells.len());
        if !in_range { return self.segment_bullet(from, to); }
        let Some(runs) = self.scoped_runs(cell, &mut Self::ray_portal_hit(from, to)) else { return self.segment_bullet(from, to) };
        let d = sub(to, from);
        let mut best = None::<f32>;
        {
            let stat = &self.stat;
            for r in &runs {
                let o = &stat.objects[*r as usize];
                for k in o.tris.clone() {
                    if stat.bullet_pass(k) { continue; } // the pass-flagged hits the bit-0x8 queries drop [M 0x1ebff0]
                    let t = &stat.tris[k];
                    if let Some(tt) = ray_tri_t(from, d, t) {
                        if best.is_none_or(|b| tt < b) { best = Some(tt); }
                    }
                }
            }
        }
        let dynamic = self.dynamic.read().unwrap();
        for g in [&self.missile, &dynamic.geo] {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    if g.bullet_pass(k) { continue; } // [M 0x1ebff0]
                    let t = &g.tris[k];
                    if let Some(tt) = ray_tri_t(from, d, t) {
                        if best.is_none_or(|b| tt < b) { best = Some(tt); }
                    }
                }
            }
        }
        best
    }

    /// The actor-capsule portal test (`Intersect_Portal`'s 0x800 path [M 0x1ea7b8]: the point on the
    /// capsule's segment closest to the portal's +0x70 reference point, then that point's distance to
    /// the portal's two triangles against radius × 1.5). The reference point is the portal's CENTRE —
    /// the 4 corners' average `build_alloc_portal` 0x1c8928 stores at +0x70 — not a corner
    /// (fix-08-collision2: corner 0 sat a doorway's height away from the capsule's band and missed the
    /// crossing, dropping the neighbouring cel's walls out of the straddle set — the 01 escort wedge).
    fn capsule_portal_hit(a: V3, b: V3, r: f32) -> impl FnMut(&ScopePortal) -> bool {
        move |p: &ScopePortal| {
            let mut hit = false;
            for t in &p.tris {
                // The closest point on the capsule's segment (HITTEST+0x20..+0x30) to portal+0x70
                // [M 0x1ea7b8's 0x800 branch: the clamped projection against the stored centre].
                let q = closest_on_segment(a, b, p.centre);
                let (cp, _) = closest_on_triangle(q, t);
                let d = sub(q, cp);
                if dot(d, d) < (r * 1.5) * (r * 1.5) { hit = true; break; }
            }
            hit
        }
    }

    /// The ray portal test (`Intersect_Portal`'s 0x201 path [M 0x1ea7b8]: `Collide_RayTriangle` over
    /// the portal's two triangles — a hit on either straddles).
    fn ray_portal_hit(from: V3, to: V3) -> impl FnMut(&ScopePortal) -> bool {
        move |p: &ScopePortal| p.tris.iter().any(|t| seg_tri_hit(from, to, t))
    }

    /// The actor capsule against the straddle cells' static geometry (`capsule_push` over the cel
    /// scope; falls back to the flat world when there is no scope or the query's cell is unmapped).
    pub fn capsule_push_scoped(&self, cell: Option<usize>, a: V3, b: V3, r: f32, top_guard: bool, ref_v: V3) -> (V3, bool) {
        let Some(cell) = cell else { return self.capsule_push(a, b, r, top_guard, ref_v) };
        let Some(runs) = self.scoped_runs(cell, &mut Self::capsule_portal_hit(a, b, r)) else { return self.capsule_push(a, b, r, top_guard, ref_v) };
        let dynamic = self.dynamic.read().unwrap();
        let (mut a, mut b) = (a, b);
        // ref_v = the caller's HITTEST+0x50 lifetime push accumulator (the radial arm's ref — see
        // capsule_tri_contact).
        let mut total = [0f32; 3];
        let mut walkable = false;
        let mut pick = false;
        let grow = r + 0.1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = a[k].min(b[k]) - grow; hi[k] = a[k].max(b[k]) + grow; }
        let stat = &self.stat;
        let mut test = |t: &[V3; 3], mat: u8, a: &mut V3, b: &mut V3, total: &mut V3, walkable: &mut bool, pick: &mut bool| {
                capsule_tri_contact(t, mat, a, b, r, grow, total, walkable, pick, top_guard, ref_v, false);
        };
        for r in runs {
            let o = &stat.objects[r as usize];
            if !(0..3).any(|k| o.hi[k] < lo[k] || o.lo[k] > hi[k]) {
                for k in o.tris.clone() { test(&stat.tris[k], stat.mats[k], &mut a, &mut b, &mut total, &mut walkable, &mut pick); }
            }
        }
        for g in [&dynamic.geo] {
            for o in overlapping(g, lo, hi) {
                for k in o.tris.clone() { test(&g.tris[k], g.mats[k], &mut a, &mut b, &mut total, &mut walkable, &mut pick); }
            }
        }
        (total, walkable)
    }

    /// The game's pick-contact grounded signal for the actor capsule: whether any contact this
    /// query resolves sets the pick bit the game grounds on (`Player_FeetOnPoint` 0x194188:176-185
    /// sets `obj+0x60 |= 8` when `HITTEST+0x9c != 0`; the bit is ORed in per real contact by
    /// `Collide_Intersect` 0x1ebaa8 tail 0x1ec138-0x1ec150 and set by `Intersect_CylGeom`
    /// 0x10a72c-0x10a734). Scoped like `capsule_push_scoped`; falls back to the flat world.
    /// Runs UNGUARDED (`top_guard = false`): the game's bit block rides the head-on re-push
    /// path, which master's swim push-tail guard explicitly does not gate.
    ///
    /// `pick_ref_y` is the caller's `HITTEST+0x50` accumulator (Bond.pick_ref[1]). The bit's
    /// dot2 gate [M Intersect_CylGeom 0x109da0 disasm 0x10a664-0x10a6a0: `dot(+0x50 − (B + push),
    /// axis) < 0`] is pass-global: +0x50 is the lifetime DeltaP accumulator [M Collide_Intersect
    /// 0x1ebaa8:353-373; no static writer — the full-.text sdc2/sw scan], and the tick's own
    /// pushes appear on BOTH operands (the accumulator already includes them, and the resolved
    /// bottom B+push carries the same adds), so per contact dot2 collapses to
    /// `acc_y < b_pre_y` with b_pre = the pass's starting bottom — the world-y comparison that
    /// kills the arm on low/negative-floor levels and keeps it on high ones. dot3
    /// (0x10a6a8-0x10a734, `normalize(R)·axis < 0.7`) is vacuous once dot2 holds
    /// (R·axis < 0 ⟹ R·axis/|R| < 0) and is not ported as a separate test.
    pub fn capsule_pick_contact_scoped(&self, cell: Option<usize>, a: V3, b: V3, r: f32, pick_ref_y: f32, ref_v: V3) -> bool {
        let b_pre_y = b[1];
        let Some(cell) = cell else { return self.capsule_pick_contact(a, b, r, pick_ref_y, ref_v) };
        let Some(runs) = self.scoped_runs(cell, &mut Self::capsule_portal_hit(a, b, r)) else { return self.capsule_pick_contact(a, b, r, pick_ref_y, ref_v) };
        let dynamic = self.dynamic.read().unwrap();
        let (mut a, mut b) = (a, b);
        let mut total = [0f32; 3];
        let mut walkable = false;
        let mut pick = false;
        let grow = r + 0.1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = a[k].min(b[k]) - grow; hi[k] = a[k].max(b[k]) + grow; }
        let stat = &self.stat;
        for run in runs {
            let o = &stat.objects[run as usize];
            if !(0..3).any(|k| o.hi[k] < lo[k] || o.lo[k] > hi[k]) {
                for k in o.tris.clone() { capsule_tri_contact(&stat.tris[k], stat.mats[k], &mut a, &mut b, r, grow, &mut total, &mut walkable, &mut pick, false, ref_v, false); }
            }
        }
        for g in [&dynamic.geo] {
            for o in overlapping(g, lo, hi) {
                for k in o.tris.clone() { capsule_tri_contact(&g.tris[k], g.mats[k], &mut a, &mut b, r, grow, &mut total, &mut walkable, &mut pick, false, ref_v, true); }
            }
        }
        // dot2 pass-global [M 0x10a66c-0x10a6a0; the collapse argument in the doc above]: the
        // bit needs the resolved bottom ABOVE the +0x50 accumulator's y.
        pick && pick_ref_y < b_pre_y
    }

    /// The flat-world `capsule_pick_contact_scoped` (no cel scope).
    pub fn capsule_pick_contact(&self, a: V3, b: V3, r: f32, pick_ref_y: f32, ref_v: V3) -> bool {
        let b_pre_y = b[1];
        let (mut a, mut b) = (a, b);
        let mut total = [0f32; 3];
        let mut walkable = false;
        let mut pick = false;
        let grow = r + 0.1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = a[k].min(b[k]) - grow; hi[k] = a[k].max(b[k]) + grow; }
        self.near_actor(lo, hi, |t, mat| {
            capsule_tri_contact(t, mat, &mut a, &mut b, r, grow, &mut total, &mut walkable, &mut pick, false, ref_v, false);
        });
        pick && pick_ref_y < b_pre_y
    }

    /// The floor ray against the straddle cells' static geometry (the walk's `segment_hit`).
    pub fn segment_hit_scoped(&self, cell: Option<usize>, from: V3, to: V3) -> Option<(f32, V3)> {
        let Some(cell) = cell else { return self.segment_hit(from, to) };
        let Some(runs) = self.scoped_runs(cell, &mut Self::ray_portal_hit(from, to)) else { return self.segment_hit(from, to) };
        let d = sub(to, from);
        let mut best = None::<(f32, V3)>;
        let stat = &self.stat;
        // The water planes (mat 0xE) never ground the player — the same rule as the flat
        // grounding ray `segment_hit` (the cel scoping of `Player_FeetOnPoint` 0x194188 [M]; see
        // `capsule_push`). G-052's bucket alignment put 07000002's pond surface in the pond cel's
        // own bucket, which the bank's floor ray straddles, so the scoped ray must drop it too.
        for r in runs {
            let o = &stat.objects[r as usize];
            for k in o.tris.clone() {
                if stat.mats[k] & 0x3f == 0xE { continue; }
                let t = &stat.tris[k];
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                let p = cross(d, e2);
                let det = dot(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv = 1.0 / det;
                let s = sub(from, t[0]);
                let u = dot(s, p) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross(s, e1);
                let v = dot(d, q) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot(e2, q) * inv;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                    let n = cross(e1, e2);
                    let l = dot(n, n).sqrt().max(1e-12);
                    best = Some((tt, [n[0] / l, n[1] / l, n[2] / l]));
                }
            }
        }
        let dynamic = self.dynamic.read().unwrap();
        let (blo, bhi) = box_of(from, to);
        for o in overlapping(&dynamic.geo, blo, bhi) {
            for k in o.tris.clone() {
                let t = &dynamic.geo.tris[k];
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                let p = cross(d, e2);
                let det = dot(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv = 1.0 / det;
                let s = sub(from, t[0]);
                let u = dot(s, p) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross(s, e1);
                let v = dot(d, q) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot(e2, q) * inv;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                    let n = cross(e1, e2);
                    let l = dot(n, n).sqrt().max(1e-12);
                    best = Some((tt, [n[0] / l, n[1] / l, n[2] / l]));
                }
            }
        }
        best
    }

    /// The first static hit of `from → to` in `cell`'s OWN scope bucket — the port's reading of
    /// `Collide_RayIntersect` (0x1ec690)'s owner filter: the query is handed the candidate cel
    /// (`build_FindCel` 0x1c8cb0's tie-break passes each candidate as the cel argument with obj
    /// lists 0 and flags 1 [M disasm 0x1c900c: a3 = 0, stack0 = 0, t2 = 1, t3 = 0x90]), and the
    /// post-pass exact-tests a static hit only when `HITDATA+0x54` (the static-geometry cel
    /// [M docs/fields/hitdata-spawner-ally.md:41, writer `Collide_Pick` 0x1eb320: `sw $s3, 0x54`
    /// = the walked chain node]) equals the passed cel [M disasm 0x1ec784-0x1ec794: hit+0x54 vs
    /// HITTEST+0x80]. So the ray sees the candidate's OWN chain geometry — the straddle collects
    /// neighbour cels' nodes but the filter rejects every one of them. No `always` runs: the game
    /// leaves unlinked objects untested. Static only: the dynamic-object hits carry +0x54 = 0 and
    /// pass the game's filter, but the port's dynamic set is not cel-keyed (Bond's own parts
    /// would tie both candidates) — a documented deviation [S].
    /// `None` = no scope, the cell is unmapped, or nothing in the cel's own set is hit.
    pub fn segment_hit_cell(&self, cell: usize, from: V3, to: V3) -> Option<(f32, V3)> {
        let sc = self.scope.as_ref()?;
        let runs = sc.cells.get(cell)?;
        let d = sub(to, from);
        let mut best = None::<(f32, V3)>;
        let stat = &self.stat;
        for r in runs {
            let o = &stat.objects[*r as usize];
            for k in o.tris.clone() {
                let t = &stat.tris[k];
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                let p = cross(d, e2);
                let det = dot(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv = 1.0 / det;
                let s = sub(from, t[0]);
                let u = dot(s, p) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross(s, e1);
                let v = dot(d, q) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot(e2, q) * inv;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                    let n = cross(e1, e2);
                    let l = dot(n, n).sqrt().max(1e-12);
                    best = Some((tt, [n[0] / l, n[1] / l, n[2] / l]));
                }
            }
        }
        best
    }

    /// The closest distance from `p` to `cell`'s OWN scoped static triangles plus the unmapped
    /// `always` runs — `Collide_SphereIntersect` (0x1ec940)'s per-cel closest hit the way the chain
    /// builder `build_link_objects_to_rooms` (0x1c9150) uses it: a sphere at `p` with a containing
    /// radius (4001), the query scoped so hits outside the queried cel read 1e8 while hits whose
    /// cel is 0 (unmapped instances — the port's `always` runs) still intersect. fix-portal5: this
    /// orders the candidate rooms of a generic's chain hang by their nearest collision point.
    /// `None` = no scope or the cell is unmapped.
    pub fn closest_point_scoped(&self, cell: usize, p: V3) -> Option<f32> {
        let sc = self.scope.as_ref()?;
        let runs = sc.cells.get(cell)?;
        let stat = &self.stat;
        let mut best: Option<f32> = None;
        for r in runs.iter().chain(sc.always.iter()) {
            let o = &stat.objects[*r as usize];
            // Bounding reject: the run's box against the best distance so far.
            let slack = best.unwrap_or(f32::MAX);
            if (0..3).any(|k| p[k] < o.lo[k] - slack || p[k] > o.hi[k] + slack) { continue; }
            for k in o.tris.clone() {
                let (cp, _) = closest_on_triangle(p, &stat.tris[k]);
                let d = sub(cp, p);
                let dist = dot(d, d).sqrt();
                if best.is_none_or(|b| dist < b) { best = Some(dist); }
            }
        }
        best
    }

    /// The move segment against one cell's portals — `Cel_ObjectLeftCel` 0x1e0d58 [M]: the segment
    /// `from→to` against each portal quad the cell participates in (the portal chain at `cel+0x28`),
    /// two `Collide_RayTriangle` tests per quad (the 0,1,2 / 0,2,3 split, `Index_189`), a hit within
    /// the segment returns the portal's OTHER cell. The caller relinks to it BEFORE the bbox keep
    /// runs, so a crossing wins even while the new position is still inside the old cell's box.
    pub fn cel_portal_dest(&self, cell: usize, from: V3, to: V3) -> Option<usize> {
        let sc = self.scope.as_ref()?;
        let dir = sub(to, from);
        let hit = |tri: [V3; 3]| -> bool {
            // Möller–Trumbore over the move segment [0, 1], both sides (the game's RayTriangle has
            // no back-face skip here — `Cel_ObjectLeftCel` 0x1e0d58:95-108 tests the quad twice).
            let e1 = sub(tri[1], tri[0]);
            let e2 = sub(tri[2], tri[0]);
            let pv = [dir[1] * e2[2] - dir[2] * e2[1], dir[2] * e2[0] - dir[0] * e2[2], dir[0] * e2[1] - dir[1] * e2[0]];
            let det = dot(e1, pv);
            if det.abs() < 1e-12 { return false; }
            let inv = 1.0 / det;
            let tv = sub(from, tri[0]);
            let u = dot(tv, pv) * inv;
            if !(-1e-9..=1.0 + 1e-9).contains(&u) { return false; }
            let qv = [tv[1] * e1[2] - tv[2] * e1[1], tv[2] * e1[0] - tv[0] * e1[2], tv[0] * e1[1] - tv[1] * e1[0]];
            let v = dot(dir, qv) * inv;
            if !(-1e-9..=1.0 + 1e-9).contains(&v) || u + v > 1.0 + 2e-9 { return false; }
            let t = dot(e2, qv) * inv;
            (0.0..=1.0).contains(&t)
        };
        for p in &sc.graph.portals {
            let dest = if p.a == cell { p.b } else if p.b == cell { p.a } else { continue };
            let c = &p.tris;
            if hit(c[0]) || hit(c[1]) { return Some(dest); }
        }
        None
    }

    /// `build_point_in_bounding_box` 0x1c9500 [M]: the inclusive bbox test over one runtime cel
    /// (`cel+0x40..0x58`). `None` cell → false.
    pub fn cel_contains(&self, cell: Option<usize>, p: V3) -> bool {
        let Some(cell) = cell else { return false };
        self.scope.as_ref().and_then(|sc| sc.graph.cells.get(cell))
            .is_some_and(|(lo, hi)| (0..3).all(|k| lo[k] <= p[k] && p[k] <= hi[k]))
    }

    /// `build_FindCel` 0x1c8cb0 [M] over the scope graph: the containing ROOM cels (`cel+0x3c &
    /// 0x40000`, cap 0x40 — `CelListCount_154`), then the game's tie-break when several contain the
    /// point: per candidate a collision raycast up (length `len`, 256 first, then the best-so-far
    /// distance) and down (length `min(up hit, len)`), both scoped to the CANDIDATE'S OWN geometry
    /// (`Collide_RayIntersect`'s cel argument + owner filter 1 — `segment_hit_cell`); the nearest
    /// hit distance wins (`if hit->0xc < fVar11`), and with no hit at all the FIRST candidate
    /// stands (0x1c8cb0's tail `return CelList_153`). One candidate returns directly [M
    /// `if (CelListCount_154 != 1)` guards the tie-break]; none returns NULL.
    /// `NDrone2_FindCel` (0x149a80): the shared candidate search + tie-break this reimplements.
    pub fn find_cel(&self, p: V3) -> Option<usize> {
        let sc = self.scope.as_ref()?;
        let mut cands: Vec<usize> = Vec::new();
        for (i, (lo, hi)) in sc.graph.cells.iter().enumerate() {
            if !sc.graph.room.get(i).copied().unwrap_or(false) { continue; }
            if (0..3).all(|k| lo[k] <= p[k] && p[k] <= hi[k]) {
                cands.push(i);
                if cands.len() >= 0x40 { break; }
            }
        }
        if cands.len() <= 1 { return cands.first().copied(); }
        let mut best: Option<(usize, f32)> = None;
        for &i in &cands {
            let len = best.map_or(256.0, |(_, d)| d);
            let up = self.segment_hit_cell(i, p, [p[0], p[1] + len, p[2]]).map(|(t, _)| t * len);
            let dlen = up.map_or(len, |ud| ud.min(len));
            let down = self.segment_hit_cell(i, p, [p[0], p[1] - dlen, p[2]]).map(|(t, _)| t * dlen);
            match (up, down) {
                (None, None) => continue,
                (a, b) => {
                    let d = a.iter().chain(b.iter()).copied().fold(f32::MAX, f32::min);
                    if best.map_or(true, |(_, bd)| d < bd) { best = Some((i, d)); }
                }
            }
        }
        Some(best.map_or(cands[0], |(i, _)| i))
    }

    /// The player object's per-tick cel relink — `control_handle_cel_change` 0x133288 [M] as the
    /// movement handler's moved pass runs it (`Control_BuildWorldSph` then the change): the move
    /// segment `prev→pos` crossed a portal of the current cel → that portal's other cell
    /// (`Cel_ObjectLeftCel` 0x1e0d58); else `pos` still inside the current cel's box → keep
    /// (`build_point_in_bounding_box` 0x1c9500); else `build_LinkToRoom` 0x1c9680 → `build_FindCel`
    /// 0x1c8cb0 AT THE WORLD-SPHERE CENTRE (`obj+0x80`, `Control_BuildWorldSph` 0x133070 — the
    /// model's local sphere centre through the object matrix, `Bond::sphere_centre`) with the
    /// raycast tie-break; a FindCel miss leaves the object unlinked (`build_LinkToRoom` returns 0
    /// without linking — `Player_InWater` 0x195e50 reads `*(obj+0x20) == 0` → no water).
    /// Deviation [S], cited: the game's `build_LinkToRoom` 0x1c9680:41-45 passes obj+0x80..0x8c (the
    /// world-sphere centre, 0.1267 below the player position [L ee_start_05 M_CONTROL+0x80]); the port
    /// passes POS — the caller's `centre` argument — because the port's per-cell bucket attribution
    /// (`build_scoped`) is coarser than the game's per-cel chains, and at the 07000001 seam the
    /// 0.1267-shorter down-ray flips the tie-break to a root whose bucket misses the floor under Bond
    /// (fix-08-collision2's traced 01 pin, kept green). `find_cel` itself is the game's.
    /// `build_LinkToRoom` 0x1c9680's relink. The game's per-cel object list is an intrusive
    /// doubly-linked list the movers maintain by hand — `control_unlink_object` 0x134608
    /// (next/prev splice at obj+0xC/+0x8, then +0x20/+0xC/+0x8 cleared [M 0x134608:4-11], 18
    /// call sites from `control_delete_object` 0x133178 to `MP_ReSpawn` 0x185368) — which the
    /// port's `Vec`-of-objects model replaces: the cel membership is re-derived per tick here
    /// [S].
    pub fn player_cel_relink(&self, cur: Option<usize>, prev: V3, pos: V3, centre: V3) -> Option<usize> {
        if let Some(cell) = cur {
            if let Some(dest) = self.cel_portal_dest(cell, prev, pos) { return Some(dest); }
            if self.cel_contains(Some(cell), pos) { return Some(cell); }
        }
        self.find_cel(centre)
    }

    /// The take's LOS ray: the straddle cells' static geometry, the 0x1000 missile cage excluded
    /// (`Collide_Pick`'s bit-0x20 skip, flags 0x721 [M 0x1eb480-0x1eb49c + 0x18e148:213-214]) and the
    /// surface classes the take's query-type filter skips — `Collide_Intersect` 0x1ebaa8's tail: the
    /// HITTEST+0x98 bit 0x1 (0x721 carries it) drops every hit whose surface byte's low 6 bits are
    /// 0xd or 0xe (`(byte & 0x3f) - 0xd < 2`).
    pub fn segment_take_scoped(&self, cell: Option<usize>, from: V3, to: V3) -> Option<f32> {
        let Some(cell) = cell else { return self.segment_actor(from, to) };
        let Some(runs) = self.scoped_runs(cell, &mut Self::ray_portal_hit(from, to)) else { return self.segment_actor(from, to) };
        let d = sub(to, from);
        let mut best = None::<f32>;
        let stat = &self.stat;
        for r in runs {
            let o = &stat.objects[r as usize];
            for k in o.tris.clone() {
                let m = stat.mats[k];
                if (m as u32 & 0x3f).wrapping_sub(0xd) < 2 { continue; }
                let t = &stat.tris[k];
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                let p = cross(d, e2);
                let det = dot(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv = 1.0 / det;
                let s = sub(from, t[0]);
                let u = dot(s, p) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross(s, e1);
                let v = dot(d, q) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot(e2, q) * inv;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b) { best = Some(tt); }
            }
        }
        let dynamic = self.dynamic.read().unwrap();
        let (blo, bhi) = box_of(from, to);
        for o in overlapping(&dynamic.geo, blo, bhi) {
            for k in o.tris.clone() {
                let t = &dynamic.geo.tris[k];
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                let p = cross(d, e2);
                let det = dot(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv = 1.0 / det;
                let s = sub(from, t[0]);
                let u = dot(s, p) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross(s, e1);
                let v = dot(d, q) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot(e2, q) * inv;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b) { best = Some(tt); }
            }
        }
        best
    }

    /// First hit along the segment `from → to` as a fraction in [0, 1] (Möller–Trumbore). One-sided
    /// like every game ray test [M Intersect_RayGeom 0x10a8c8:168: `n·dir >= 0` skips the triangle].
    /// The game's ray query returns a LINKED LIST of HITDATA nodes from a fixed pool
    /// (`Coll_AddHitToList` 0x1ede68 cuts a 0x60-byte node from the LList pool 0x34B7B0,
    /// links it at the head, stores cel at +0x54 / obj at +0x58 [M 0x1ede68:4-24];
    /// `Collide_FreeHitList` 0x1ede00 walks the list back into the pool [M :4-14] — 37
    /// consumers from `Bullet_update` to `control_delete_object`). The port returns the first
    /// fraction and frees implicitly — no pool, no node list [S].
    pub fn segment(&self, from: V3, to: V3) -> Option<f32> {
        self.segment_in_sets(from, to, true)
    }

    /// The bullet-family segment: `segment` MINUS every hit whose hit-data would carry the pass
    /// flag (the glist node's flags bit 0x40 or the surface byte's bit 0x40). The game's bullet
    /// queries run HITTEST+0x98 with bit 0x8 — the aim trace 0x20a/0x208 (`Player_WeaponInitBullet`
    /// 0x1a4bf0:154-159 [M]) and `Bullet_homing`'s LOS 8 (0x12f358:92 [M]) — and
    /// `Collide_Intersect` 0x1ebaa8 drops pass-flagged hits outright for them (fraction 1e8,
    /// disasm 0x1ebff0-0x1ec010), so the trace's first report is the first NON-pass surface and
    /// bullets fly through the flagged node (the 07000005 trees, glist flags 0x41 [D]).
    /// fix-foliage-bullets.
    pub fn segment_bullet(&self, from: V3, to: V3) -> Option<f32> {
        self.segment_in_sets_drop_pass(from, to, true, true)
    }

    /// The actor-class segment: static MINUS the 0x1000 missile cage, plus the dynamic set — the
    /// game's queries whose HITTEST+0x98 carries bit 0x20 skip the 0x1000-flagged nodes [M
    /// `Collide_Pick` 0x1eb320 disasm 0x1eb480-0x1eb49c, fix-objects2], and the take test's
    /// `Collide_LineOfSight` runs with flags 0x721 (bit 0x20 set) [M 0x18e148:214]. The flat
    /// `segment` (bullets/sight) still hits the cage. fix-safe08: the 07000008 safe take's LOS ray
    /// crossed the cage's hidden wall beside the safe and refused every take.
    pub fn segment_actor(&self, from: V3, to: V3) -> Option<f32> {
        self.segment_in_sets(from, to, false)
    }

    /// The SIGHT-class segment: the masked ray behind the game's vision tests. The drone vision
    /// call runs `Collide_LineOfSight` whose last argument ORs 0xa27 into the caller's flags [M
    /// `DroneVision_CanSeeObjectFrom` 0x176f18:69-70 and 95-96 — the 0xa27 immediates at
    /// 0x176fec/0x1770b0 sit directly before both calls], and `Collide_LineOfSight` 0x1edf50
    /// forwards it as `Collide_RayIntersect`'s HITTEST+0x98 mask (disasm 0x1edf54 `ori t3,t1,4`
    /// — 0xa27 already carries bit 2, and `Collide_RayIntersect` 0x1ec690 stores arg7 at
    /// HITTEST+0x98). `Collide_Pick`/`Collide_Intersect` then drop, before a hit is reported:
    /// - the 0x1000-flagged nodes (mask bit 0x20) [M `Collide_Pick` 0x1eb320:149-152] — the port's
    ///   missile cage, the same skip `segment_actor` already ports;
    /// - any surface whose material byte carries 0x40 (mask bit 0x800) [M `Collide_Intersect`
    ///   0x1ebaa8:301];
    /// - materials 0xD and 0xE (mask bit 0x1: `(mat & 0x3f) - 0xd < 2`) [M 0x1ebaa8:311];
    /// - material 0x10 (mask bit 0x2) [M 0x1ebaa8:314];
    /// and still tests objects (mask bit 0x10 clear → `Collide_PickObj` runs [M 0x1eb320:314]).
    /// So unlike `segment_bullet` (mask bit 0x8: the NODE pass flag drops the hit) the sight ray
    /// drops only the 0x40 MATERIAL byte — the raw `segment` is strictly stricter, which is what
    /// blocked the 01 escort's stairwell peer (the game sees guard 2, the raw ray did not).
    pub fn segment_masked(&self, from: V3, to: V3) -> Option<f32> {
        let d = sub(to, from);
        let mut best = None::<f32>;
        let dynamic = self.dynamic.read().unwrap();
        let sets: Vec<&Geo> = vec![&self.stat, &dynamic.geo];
        for g in sets {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    let m = g.mats[k];
                    let low = m & 0x3f;
                    if m & 0x40 != 0 || low == 0xd || low == 0xe || low == 0x10 { continue; } // the 0xa27 drops [M 0x1ebaa8:301/311/314]
                    let t = &g.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(from, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b) { best = Some(tt); }
                }
            }
        }
        best
    }

    fn segment_in_sets(&self, from: V3, to: V3, with_missile: bool) -> Option<f32> {
        self.segment_in_sets_drop_pass(from, to, with_missile, false)
    }

    fn segment_in_sets_drop_pass(&self, from: V3, to: V3, with_missile: bool, drop_pass: bool) -> Option<f32> {
        let d = sub(to, from);
        let (mut best, mut lo, mut hi) = (None::<f32>, [0f32; 3], [0f32; 3]);
        for k in 0..3 { lo[k] = from[k].min(to[k]); hi[k] = from[k].max(to[k]); }
        let _ = (lo, hi);
        let dynamic = self.dynamic.read().unwrap();
        let sets: Vec<&Geo> = if with_missile { vec![&self.stat, &self.missile, &dynamic.geo] } else { vec![&self.stat, &dynamic.geo] };
        for g in sets {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    if drop_pass && g.bullet_pass(k) { continue; } // the pass-flagged hits the bit-0x8 queries drop [M 0x1ebff0]
                    let t = &g.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(from, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b) { best = Some(tt); }
                }
            }
        }
        best
    }

    /// mp-perf-16: whether ANY triangle of the sets blocks `from → to` — the exact predicate
    /// `segment_in_sets_drop_pass(..).is_none()` evaluates (same sets, same pass drop, the same
    /// Möller–Trumbore arithmetic term for term), but it stops at the first blocking triangle
    /// instead of searching on for the nearest. The boolean callers (`line_of_sight`, the bots'
    /// sight lines) only ask "is there a hit", so the answer is bit-identical and the work is the
    /// prefix up to the first hit [S port optimisation].
    fn segment_blocked(&self, from: V3, to: V3, with_missile: bool, drop_pass: bool) -> bool {
        let d = sub(to, from);
        let dynamic = self.dynamic.read().unwrap();
        let sets: [Option<&Geo>; 3] = [Some(&self.stat), with_missile.then_some(&self.missile), Some(&dynamic.geo)];
        for g in sets.into_iter().flatten() {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    if drop_pass && g.bullet_pass(k) { continue; }
                    let t = &g.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(from, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) { return true; }
                }
            }
        }
        false
    }

    /// `segment(from, to).is_none()`, early-out (see `segment_blocked`).
    pub fn segment_clear(&self, from: V3, to: V3) -> bool { !self.segment_blocked(from, to, true, false) }

    /// `segment_surface_bullet(from, to).is_none()` (= `segment_bullet(..).is_none()`: the same
    /// sets, pass drop and triangle test), early-out (see `segment_blocked`).
    pub fn segment_bullet_clear(&self, from: V3, to: V3) -> bool { !self.segment_blocked(from, to, true, true) }

    /// First hit along `from → to`: the fraction in [0, 1] and the unit normal of the triangle hit (facing `from`).
    /// The player's GROUNDING variant: water surfaces (material 0xE) never ground him — the cel scoping
    /// of `Player_FeetOnPoint` 0x194188 (the ray tests only the current cel `obj+0x20`; the water planes
    /// live in the water cels, whose bbox tops sit at the surface below the walking root) [M] — see
    /// `capsule_push` for the full note. The flat `segment_hit` keeps water for bullets/LOS/AI.
    pub fn segment_hit(&self, from: V3, to: V3) -> Option<(f32, V3)> {
        self.segment_surface(from, to, true).map(|h| (h.0, h.1))
    }

    /// The static-only first hit along `from → to` (fix-portal-sweep): `segment_hit` minus the
    /// missile cage and the dynamic objects. The near-LOS rescue's ray must not be blocked by the
    /// player's own collision or the script-animated parts standing next to him — the game's
    /// `Vision_InView` (0x1e89e0) consults no collision at all (a pure frustum test); this ray is
    /// the port-side unocclusion guard only, so it reads the static world like the level's own
    /// glists do.
    pub fn segment_hit_static(&self, from: V3, to: V3) -> Option<(f32, V3)> {
        let d = sub(to, from);
        let mut best: Option<(f32, V3)> = None;
        for o in along(&self.stat, from, to) {
            for k in o.tris.clone() {
                let t = &self.stat.tris[k];
                let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                let p = cross(d, e2);
                let det = dot(e1, p);
                if det.abs() < 1e-12 { continue; }
                let inv = 1.0 / det;
                let s = sub(from, t[0]);
                let u = dot(s, p) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let q = cross(s, e1);
                let v = dot(d, q) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let tt = dot(e2, q) * inv;
                if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                    let n = cross(e1, e2);
                    let l = dot(n, n).sqrt().max(1e-12);
                    best = Some((tt, [n[0] / l, n[1] / l, n[2] / l]));
                }
            }
        }
        best
    }

    /// `segment_hit` with the surface byte of the triangle hit (`Intersect_RayGeom` → `HITDATA+0x50`).
    /// One-sided [M Intersect_RayGeom 0x10a8c8:150-168]: the plane is the triangle's STORED normal (=
    /// +cross(v1−v0, v2−v0), docs/formats/collision.md whole-disc check) and a triangle whose
    /// `n·dir >= 0` is skipped — a ray only hits the front face. `no_water` additionally drops the
    /// water-surface material (0xE) triangles for the player's grounding rays (the cel scoping — see
    /// `segment_hit`).
    pub fn segment_surface(&self, from: V3, to: V3, no_water: bool) -> Option<(f32, V3, u8)> {
        self.segment_surface_impl(from, to, no_water, false)
    }

    /// The bullet-family `segment_surface`: pass-flagged triangles (the glist node's bit 0x40 or
    /// the surface byte's bit 0x40) are dropped the way `Collide_Intersect` 0x1ebaa8 drops them
    /// for the bit-0x8 queries (fraction 1e8, disasm 0x1ebff0-0x1ec010) — the walk's first hit is
    /// the first non-pass surface. Used by the homing LOS (`Bullet_homing` 0x12f358:92, flags 8).
    pub fn segment_surface_bullet(&self, from: V3, to: V3) -> Option<(f32, V3, u8)> {
        self.segment_surface_impl(from, to, false, true)
    }

    /// The flight-trace hit WITH the hit's node pass flag (`HITDATA+0x51` bit 1's node source):
    /// the flight trace keeps pass-flagged hits in its list (`Collide_Update` 0x1ea398 walks them,
    /// `Bullet_CollisionHandler` 0x12ce80 sets state 1 past them), so the caller's walk needs to
    /// know which hits to carry the round past. fix-foliage-bullets.
    pub fn segment_surface_flags(&self, from: V3, to: V3, no_water: bool) -> Option<(f32, V3, u8, bool)> {
        let d = sub(to, from);
        let (mut best, mut lo, mut hi) = (None::<(f32, V3, u8, bool)>, [0f32; 3], [0f32; 3]);
        for k in 0..3 { lo[k] = from[k].min(to[k]); hi[k] = from[k].max(to[k]); }
        let _ = (lo, hi);
        let dynamic = self.dynamic.read().unwrap();
        for g in [&self.stat, &self.missile, &dynamic.geo] {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    let node_pass = g.pass[k];
                    let mat = g.mats[k];
                    if no_water && mat & 0x3f == 0xE { continue; }
                    let t = &g.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    let n = cross(e1, e2);
                    if dot(n, d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(from, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                        let l = dot(n, n).sqrt().max(1e-12);
                        best = Some((tt, [n[0] / l, n[1] / l, n[2] / l], mat, node_pass));
                    }
                }
            }
        }
        best
    }

    /// The flush's re-query with the game's candidate set: `segment_surface_flags` MINUS every
    /// static triangle outside the straddle cells — `Collide_Pick` 0x1eb320:125 takes the flight
    /// trace's candidate nodes ONLY from `Collide_StraddleCels` 0x1eac98 (the BFS from the query's
    /// cel through the portals the segment crosses), and the flying bullet re-feeds that start cel
    /// every tick [M Bullet_update 0x12e7b0 disasm 0x12f0cc-0x12f0d8]. Pass faces are KEPT (both
    /// the glist-0x40 node flag and the surface byte ride in the result): the flight trace's
    /// HITTEST+0x98 = 0 [M 0x12f040] clears every fraction-1e8 drop in `Collide_Intersect`
    /// 0x1ebaa8 (the 0x40 byte drop needs +0x98 bit 0x800, the node-flag drop bit 0x8 — disasm
    /// 0x1ebff0-0x1ec010), so pass-flagged faces are HITDATA the handler's walk decides about
    /// [M Bullet_CollisionHandler 0x12ce80's pass rules, 0x12cee0..0x12cfb8]. The missile cage and
    /// the dynamic set stay in (the pick's per-cel object walk runs when HITTEST+0x98 bit 0x10 is
    /// clear — the bullet's is [M 0x12f040]). Falls back to the flat `segment_surface_flags` when
    /// there is no scope or the cell is unknown — the caller's safety net, same as the scoped
    /// bullet query. level-11-stairwell-bullet-holes-6.
    pub fn segment_surface_flags_scoped(&self, cell: Option<usize>, from: V3, to: V3) -> Option<(f32, V3, u8, bool)> {
        let Some(cell) = cell else { return self.segment_surface_flags(from, to, false) };
        // An out-of-range cell is a caller bug: the flat answer is the conservative fallback (the
        // scoped actor queries instead test nothing — their callers keep a valid `bond.cel`).
        let in_range = self.scope.as_ref().is_some_and(|sc| cell < sc.graph.cells.len());
        if !in_range { return self.segment_surface_flags(from, to, false); }
        let Some(runs) = self.scoped_runs(cell, &mut Self::ray_portal_hit(from, to)) else { return self.segment_surface_flags(from, to, false); };
        let d = sub(to, from);
        let mut best = None::<(f32, V3, u8, bool)>;
        {
            let stat = &self.stat;
            for r in &runs {
                let o = &stat.objects[*r as usize];
                for k in o.tris.clone() {
                    let node_pass = stat.pass[k];
                    let mat = stat.mats[k];
                    let t = &stat.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    let n = cross(e1, e2);
                    if dot(n, d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(from, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                        let l = dot(n, n).sqrt().max(1e-12);
                        best = Some((tt, [n[0] / l, n[1] / l, n[2] / l], mat, node_pass));
                    }
                }
            }
        }
        let dynamic = self.dynamic.read().unwrap();
        for g in [&self.missile, &dynamic.geo] {
            for o in along(g, from, to) {
                for k in o.tris.clone() {
                    let node_pass = g.pass[k];
                    let mat = g.mats[k];
                    let t = &g.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    let n = cross(e1, e2);
                    if dot(n, d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(from, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                        let l = dot(n, n).sqrt().max(1e-12);
                        best = Some((tt, [n[0] / l, n[1] / l, n[2] / l], mat, node_pass));
                    }
                }
            }
        }
        best
    }

    fn segment_surface_impl(&self, from: V3, to: V3, no_water: bool, drop_pass: bool) -> Option<(f32, V3, u8)> {
        let d = sub(to, from);
        let (mut best, mut lo, mut hi) = (None::<(f32, V3, u8)>, [0f32; 3], [0f32; 3]);
        for k in 0..3 { lo[k] = from[k].min(to[k]); hi[k] = from[k].max(to[k]); }
        let _ = (lo, hi); self.near_seg_drop_pass(from, to, drop_pass, |t, mat| {
            if no_water && mat & 0x3f == 0xE { return; } // the water planes never ground the player
            let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
            let n = cross(e1, e2);
            if dot(n, d) >= 0.0 { return; } // back face: the game's ray test skips it [M 0x10a8c8:168]
            let p = cross(d, e2);
            let det = dot(e1, p);
            if det.abs() < 1e-12 { return; }
            let inv = 1.0 / det;
            let s = sub(from, t[0]);
            let u = dot(s, p) * inv;
            if !(0.0..=1.0).contains(&u) { return; }
            let q = cross(s, e1);
            let v = dot(d, q) * inv;
            if v < 0.0 || u + v > 1.0 { return; }
            let tt = dot(e2, q) * inv;
            if (0.0..=1.0).contains(&tt) && best.is_none_or(|b| tt < b.0) {
                let l = dot(n, n).sqrt().max(1e-12);
                best = Some((tt, [n[0] / l, n[1] / l, n[2] / l], mat));
            }
        });
        best
    }

    /// The bullet-probe sphere against the world (`Bullet_update`'s slow-move hittest form,
    /// version 0x101 [M 0x12e7b0:425-444 → `Collide_SphereIntersect` 0x1ec940 →
    /// `Intersect_PointGeom` 0x1e92d8]): the closest surface within `radius` of `centre`, as
    /// `(closest point on the triangle, stored plane normal, surface byte, distance)`.
    ///
    /// Per triangle (`Intersect_PointGeom` 0x1e92d8:183-249): the signed plane distance `sd` of
    /// the centre; faces in front by more than the radius are skipped (`radius < sd` → next tri);
    /// `Dist2Tri_PointGeom` 0x1dfac8 returns the closest point on the triangle and — for a centre BEHIND
    /// the face — a negative result, which the best gate skips (`fVar24 < fVar25` with the best
    /// initialised to 0.0 [M 0x1e92d8:103-109; the HITTEST flag word +0x9a bit 0x8 that would
    /// re-allow behind-faces is never set by Bullet_update's build — it writes only 0x20 at
    /// 0x12e7b0:391]) — so the probe is ONE-SIDED: it sees faces the centre is in front of or
    /// touching, never a face it has passed. The minimum-distance face wins; the hit point is
    /// Dist2Tri's closest point (HITDATA+0x40) and the normal the triangle's stored plane normal
    /// (HITDATA+0x10) — the same normal `segment_surface` computes.
    pub fn sphere_surface(&self, centre: V3, radius: f32, no_water: bool) -> Option<(V3, V3, u8, f32)> {
        let mut best: Option<(V3, V3, u8, f32)> = None;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = centre[k] - radius; hi[k] = centre[k] + radius; }
        self.near(lo, hi, |t, mat| {
            if no_water && mat & 0x3f == 0xE { return; }
            let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
            let n = cross(e1, e2);
            let nl = dot(n, n).sqrt();
            if nl < 1e-12 { return; }
            let n = [n[0] / nl, n[1] / nl, n[2] / nl];
            // Signed plane distance of the centre; one-sided like the game's best gate.
            let sd = dot(n, sub(centre, t[0]));
            if sd > radius || sd < 0.0 { return; }
            let (q, _on_edge) = closest_on_triangle(centre, t);
            let d = sub(centre, q);
            let dist = dot(d, d).sqrt();
            if dist > radius { return; }
            if best.is_none_or(|b| dist < b.3) {
                best = Some((q, n, mat, dist));
            }
        });
        best
    }

    /// The player's capsule against the static geometry (`Intersect_CylGeom` 0x109da0, docs/systems/player.md;
    /// the scalar triangle-collector form is `Intersect_CylTriGeom` 0x1e9a70 — the HITTEST version 0x2000
    /// arm of `Collide_Intersect` 0x1ebaa8:256, radius grown by +0.05, leaves via `RecurseBoxesBox` 0x1edc48,
    /// per-triangle gates HITTEST+0x98 bits 1/2 and the +0x9a bit masks [M]): the
    /// segment `a`–`b` with radius `r`, triangle by triangle, each push applied before the next triangle. Per triangle:
    /// the segment point where it meets the plane (clamped; the point nearest the centroid when parallel), the closest
    /// triangle point to it, and when that lies on an edge the segment point nearest it; closer than `r` → pushed out
    /// along the separation. Returns the summed push and whether a walkable face (normal y > 0.7) touched below `b`.
    /// One-sided head-on cull, the game's exact rule [M Intersect_CylGeom 0x109da0:302-322]:
    /// |n·d̂| > 0.999 on the NORMALIZED spine (the Vec_Normalise 0x1e3000 at 0x109f74 precedes the
    /// tri loop; the raw-dot reading is refuted by the stairwell — see capsule_tri_contact) with
    /// the crossing fraction f from A = param_1[2], skip on (n·d̂ > 0.999 ∧ f > 0.5) ∨ (n·d̂ <
    /// −0.999 ∧ f < 0.5). The port's `a`/`b` are the game's HITTEST+0x20/+0x30 in that order
    /// (walk: A = pos+up·0.275 top, B = pos+up·(0.55−cc) bottom [M 0x194df0 disasm
    /// 0x195444/0x195464; L ee_start_05 |d| = 0.7597198], climb: A = pos+up·0.55, B =
    /// pos+up·(0.275−cc) [M 0x194df0 case 0/6], |d| = 1.3097183), so f means the game's fraction
    /// from the top end. The cull lives for faces within ~2.6° of horizontal: the step
    /// behind/above (f = 0) is skipped — stairs descend; a face crossed in the spine's TOP half
    /// is jumped through from below; the BOTTOM half is solid. The old stand-in's wider
    /// 0.70-0.763 band (≈45-77° slopes above the capsule) is gone — those faces push, as the
    /// game's non-head-on path tests them.
    pub fn capsule_push(&self, a: V3, b: V3, r: f32, top_guard: bool, ref_v: V3) -> (V3, bool) {
        let (mut a, mut b) = (a, b);
        let mut total = [0f32; 3];
        let mut walkable = false;
        let mut pick = false;
        let grow = r + 0.1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = a[k].min(b[k]) - grow; hi[k] = a[k].max(b[k]) + grow; }
        self.near_actor(lo, hi, |t, mat| {
                capsule_tri_contact(t, mat, &mut a, &mut b, r, grow, &mut total, &mut walkable, &mut pick, top_guard, ref_v, false);
        });
        (total, walkable)
    }

    /// The move-clip walk query (`NF_WALK_CLIP=1` — see `capsule_tri_clip`): the actor capsule
    /// against the straddle cells' static geometry with the tick's move CLIPPED at the face
    /// contacts instead of the two-sided depenetration push. Returns the pos correction (the
    /// clipped move's remainder — what is LEFT of the tick's move) and the walkable signal.
    /// Scoped like `capsule_push_scoped`; falls back to the flat world. `move_v` is the tick's
    /// position delta (the move being clipped).
    pub fn capsule_clip_scoped(&self, cell: Option<usize>, a: V3, b: V3, r: f32, top_guard: bool, move_v: V3) -> (V3, bool) {
        let Some(cell) = cell else { return self.capsule_clip(a, b, r, top_guard, move_v) };
        let Some(runs) = self.scoped_runs(cell, &mut Self::capsule_portal_hit(a, b, r)) else { return self.capsule_clip(a, b, r, top_guard, move_v) };
        let dynamic = self.dynamic.read().unwrap();
        let a0 = a;
        let (mut a, mut b) = (a, b);
        let mut walkable = false;
        let mut pick = false;
        let grow = r + 0.1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = a[k].min(b[k]) - grow; hi[k] = a[k].max(b[k]) + grow; }
        let stat = &self.stat;
        let rad = r;
        for run in runs {
            let o = &stat.objects[run as usize];
            if !(0..3).any(|k| o.hi[k] < lo[k] || o.lo[k] > hi[k]) {
                for k in o.tris.clone() { capsule_tri_clip(&stat.tris[k], stat.mats[k], &mut a, &mut b, rad, grow, move_v, &mut walkable, &mut pick, top_guard); }
            }
        }
        for g in [&dynamic.geo] {
            for o in overlapping(g, lo, hi) {
                for k in o.tris.clone() { capsule_tri_clip(&g.tris[k], g.mats[k], &mut a, &mut b, rad, grow, move_v, &mut walkable, &mut pick, top_guard); }
            }
        }
        // The correction: what is LEFT of the tick's move after the clips (the pulled-back a is
        // a_start + move_v·s_min; the caller's pos already integrated the full move, so the
        // correction is the clipped-back remainder, negative along the clipped axis).
        ([a[0] - a0[0], a[1] - a0[1], a[2] - a0[2]], walkable)
    }

    /// The flat-world `capsule_clip_scoped` (no cel scope).
    pub fn capsule_clip(&self, a: V3, b: V3, r: f32, top_guard: bool, move_v: V3) -> (V3, bool) {
        let a0 = a;
        let (mut a, mut b) = (a, b);
        let mut walkable = false;
        let mut pick = false;
        let grow = r + 0.1;
        let mut lo = [0f32; 3];
        let mut hi = [0f32; 3];
        for k in 0..3 { lo[k] = a[k].min(b[k]) - grow; hi[k] = a[k].max(b[k]) + grow; }
        self.near_actor(lo, hi, |t, mat| {
                capsule_tri_clip(t, mat, &mut a, &mut b, r, grow, move_v, &mut walkable, &mut pick, top_guard);
        });
        ([a[0] - a0[0], a[1] - a0[1], a[2] - a0[2]], walkable)
    }

    /// Clear line between two points — the game's `Collide_LineOfSight` 0x1edf50 →
    /// `Collide_RayIntersect` 0x1ec690 ray (type +0x94 = 0x201) with the CALLER's filter mask on
    /// HITTEST+0x98. The drop semantics live in `Collide_Intersect` 0x1ebaa8:300-319 (and
    /// `Collide_Pick` 0x1eb320:140-152): bit 0x800 drops hits whose per-triangle surface byte has
    /// bit 0x40; bit 0x1 drops surface classes 0xd/0xe; bit 0x2 drops class 0x10; bit 0x8 drops
    /// hits on NODE-flagged cels (cel+0x94 bit 0x40 → HITDATA+0x51 bit 2 via :93-99/:143-145);
    /// bit 0x20 skips the 0x1000-flagged nodes (the port's missile cage). `mask` already includes
    /// the wrapper's own `| 4` [M disasm 0x176fec/0x1770b0 — docs/systems/drone-armed-peer.md].
    /// Each caller passes the mask its game call site passes — see the `LOS_*` constants.
    pub fn line_of_sight(&self, a: V3, b: V3, mask: u32) -> bool {
        if !LOS_MASKS_ON { let _ = mask; return self.segment_clear(a, b); }
        self.line_of_sight_masked(a, b, mask)
    }

    /// The masked `Collide_LineOfSight` 0x1edf50 body, applied unconditionally — the per-caller
    /// drop semantics of `Collide_Intersect` 0x1ebaa8:300-319 (and `Collide_Pick` 0x1eb320:140-152):
    /// mask 0x800 drops hits whose per-triangle surface byte has bit 0x40; mask 0x1 drops surface
    /// classes 0xd/0xe; mask 0x2 drops class 0x10 (glass); **mask 0x8 drops every hit whose
    /// HITDATA+0x51 bit 1 is set — the bit the NODE's 0x40 cel flag (:143-145) ORs in and the
    /// surface byte's 0x40/0x80 bits OR in (:305-309)** — so a 0xa-mask sensor ray (Check_AutoAim
    /// 0x1a73f8:164-165) passes the stairwell's 0x40-material well panes that a plain segment
    /// test blocks; mask 0x20 skips the 0x1000-flagged nodes (the port's missile cage).
    /// (`line_of_sight` keeps the global `LOS_MASKS_ON` gate — the masked semantics stay
    /// opt-in until the parked per-caller flip is safe.)
    pub fn line_of_sight_masked(&self, a: V3, b: V3, mask: u32) -> bool {
        let d = sub(b, a);
        let dynamic = self.dynamic.read().unwrap();
        let skip_cage = mask & 0x20 != 0;
        let (drop_node, drop_surface, drop_de, drop_10) =
            (mask & 0x8 != 0, mask & 0x800 != 0, mask & 0x1 != 0, mask & 0x2 != 0);
        let sets: [Option<&Geo>; 3] = [Some(&self.stat), (!skip_cage).then_some(&self.missile), Some(&dynamic.geo)];
        for g in sets.into_iter().flatten() {
            for o in along(g, a, b) {
                for k in o.tris.clone() {
                    let m = g.mats[k];
                    let low = m & 0x3f;
                    if drop_node && (g.pass[k] || m & 0x40 != 0 || m & 0x80 != 0) { continue; }
                    if drop_surface && m & 0x40 != 0 { continue; }
                    if drop_de && (low == 0xd || low == 0xe) { continue; }
                    if drop_10 && low == 0x10 { continue; }
                    let t = &g.tris[k];
                    if dot(cross(sub(t[1], t[0]), sub(t[2], t[0])), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    if seg_tri_hit(a, b, t) { return false; }
                }
            }
        }
        true
    }

    /// Height of the first collision surface straight below `p` within `depth` (starting 0.5 above it).
    pub fn floor_below(&self, p: V3, depth: f32) -> Option<f32> {
        let from = [p[0], p[1] + 0.5, p[2]];
        let to = [p[0], p[1] - depth, p[2]];
        self.segment(from, to).map(|t| from[1] + (to[1] - from[1]) * t)
    }

    /// The distance-returning `line_of_sight`: the same masked walk with the same drop filters
    /// (`Collide_RayIntersect` 0x1ec690 with the caller's mask on HITTEST+0x98, the drops in
    /// `Collide_Intersect` 0x1ebaa8:300-319), returning the closest hit's FRACTION of `a → b`
    /// (`Some` = blocked at that fraction, `None` = clear). `Sensor_SetBeam` 0x1af398:77-79
    /// stores the first hit's distance (HITDATA+0xC) into the lens object's obj+0xE8 — the beam
    /// stretch — for the same mask-8 ray the bool gate walks (LOS_PROBE_8, world.rs top).
    pub fn line_of_sight_hit(&self, a: V3, b: V3, mask: u32) -> Option<f32> {
        let d = sub(b, a);
        let dynamic = self.dynamic.read().unwrap();
        let skip_cage = mask & 0x20 != 0;
        let (drop_node, drop_surface, drop_de, drop_10) =
            (mask & 0x8 != 0, mask & 0x800 != 0, mask & 0x1 != 0, mask & 0x2 != 0);
        let sets: [Option<&Geo>; 3] = [Some(&self.stat), (!skip_cage).then_some(&self.missile), Some(&dynamic.geo)];
        let mut best = None::<f32>;
        for g in sets.into_iter().flatten() {
            for o in along(g, a, b) {
                for k in o.tris.clone() {
                    let m = g.mats[k];
                    let low = m & 0x3f;
                    if drop_node && (g.pass[k] || m & 0x40 != 0 || m & 0x80 != 0) { continue; }
                    if drop_surface && m & 0x40 != 0 { continue; }
                    if drop_de && (low == 0xd || low == 0xe) { continue; }
                    if drop_10 && low == 0x10 { continue; }
                    let t = &g.tris[k];
                    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
                    if dot(cross(e1, e2), d) >= 0.0 { continue; } // back face [M 0x10a8c8:168]
                    let p = cross(d, e2);
                    let det = dot(e1, p);
                    if det.abs() < 1e-12 { continue; }
                    let inv = 1.0 / det;
                    let s = sub(a, t[0]);
                    let u = dot(s, p) * inv;
                    if !(0.0..=1.0).contains(&u) { continue; }
                    let q = cross(s, e1);
                    let v = dot(d, q) * inv;
                    if v < 0.0 || u + v > 1.0 { continue; }
                    let tt = dot(e2, q) * inv;
                    if (0.0..=1.0).contains(&tt) && best.is_none_or(|bb| tt < bb) { best = Some(tt); }
                }
            }
        }
        best
    }

    /// walls-990 debug: the static objects whose bounds contain `p` (expanded 0.05), as
    /// `(min, max)` pairs — a probe hit's owner candidates, printed by the stairwell-walls test
    /// to name the invisible wall the guards stop against. Static set only (the moving set's
    /// matrices live in `last_objects`; the stairwell stalls are static hits).
    pub fn objects_with_bound(&self, p: V3) -> Vec<(V3, V3)> {
        self.stat.objects.iter().filter(|o| {
            (0..3).all(|k| p[k] >= o.lo[k] - 0.05 && p[k] <= o.hi[k] + 0.05)
        }).map(|o| (o.lo, o.hi)).collect()
    }

    /// Scratch dump (level-0a-office-hatch-9): every static triangle with its centroid inside
    /// `lo..hi`, in world space — the doorway-base/corner tri dumps the walk diagnosis runs.
    pub fn dump_tris_in(&self, lo: V3, hi: V3) -> Vec<[[f32; 3]; 3]> {
        let mut out = Vec::new();
        let inside = |c: V3| (0..3).all(|k| c[k] >= lo[k] && c[k] <= hi[k]);
        for o in overlapping(&self.stat, lo, hi) {
            for k in o.tris.clone() {
                let t = &self.stat.tris[k];
                let c = [0, 1, 2].map(|ax| (t[0][ax] + t[1][ax] + t[2][ax]) / 3.0);
                if inside(c) { out.push(*t); }
            }
        }
        out
    }
}

fn closest_on_segment(a: V3, b: V3, p: V3) -> V3 {
    let ab = sub(b, a);
    let l2 = dot(ab, ab);
    let t = if l2 > 0.0 { (dot(sub(p, a), ab) / l2).clamp(0.0, 1.0) } else { 0.0 };
    [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
}

/// The closest point of a triangle to `p` (Ericson, Real-Time Collision Detection 5.1.5) and whether it lies on an edge
/// or vertex rather than inside the face.
/// The port's float point-to-triangle distance — the Ericson form of the game's `Dist2Tri`
/// family: `vecutil_Dist2Tri_PointGeom` 0x1dfac8 (the scalar one `Intersect_PointGeom`
/// 0x1e92d8:222 drives), `vecutil_Dist2Tri` 0x1dfca8 (the portal/cyl-tri one) and the VU0
/// fixed-point `ASH_vecutil_Dist2Tri` 0x109770 (`Intersect_CylGeom` 0x109da0:331,416's actor
/// push) [M — the same edge-bit mask (face / edge / vertex) and plane-projection shape].
fn closest_on_triangle(p: V3, t: &[V3; 3]) -> (V3, bool) {
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    if d1 <= 0.0 && d2 <= 0.0 { return (a, true); }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 { return (b, true); }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 { let v = d1 / (d1 - d3); return ([a[0] + ab[0] * v, a[1] + ab[1] * v, a[2] + ab[2] * v], true); }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 { return (c, true); }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 { let w = d2 / (d2 - d6); return ([a[0] + ac[0] * w, a[1] + ac[1] * w, a[2] + ac[2] * w], true); }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = sub(c, b);
        return ([b[0] + bc[0] * w, b[1] + bc[1] * w, b[2] + bc[2] * w], true);
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    ([a[0] + ab[0] * v + ac[0] * w, a[1] + ab[1] * v + ac[1] * w, a[2] + ab[2] * v + ac[2] * w], false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(tris: Vec<[V3; 3]>) -> CollisionWorld {
        let n = tris.len();
        let stat = Geo { mats: vec![0; n], tris, objects: vec![Object { lo: [-10.0; 3], hi: [10.0; 3], tris: 0..n }], groups: Vec::new(), owners: Vec::new(),
            pass: vec![false; n] };
        CollisionWorld { stat, missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(), scope: None }
    }

    /// fix-safe08b: the straddle walk. A wall assigned to an exterior cell with NO portal from the
    /// query's cell is never tested by the scoped actor queries (`Collide_StraddleCels` 0x1eac98:
    /// the exterior cel is not portal-visible from the bedroom); the flat queries still hit it; a
    /// portal the capsule straddles brings the wall back; and the take ray skips the surface classes
    /// the 0x721 filter drops (`Collide_Intersect` 0x1ebaa8's tail: bit 0x1 → classes 0xd/0xe).
    #[test]
    fn scoped_actor_queries_keep_only_the_straddle_cels() {
        // A wall quad at x = 2 (bedroom side, 1.5 units from the capsule) — its collision object is
        // assigned to cell 1 (the "exterior"): its world sphere centre (2, 1, 0) lies in cell 1's box.
        // Wound with the stored normal facing −x (west, at the capsule) — the disc's convention for
        // a wall that stops rays/capsules from the west side (0x10a8c8 one-sided).
        let wall = [
            [[2.0, 0.0, -2.0], [2.0, 2.0, 2.0], [2.0, 2.0, -2.0]],
            [[2.0, 0.0, -2.0], [2.0, 0.0, 2.0], [2.0, 2.0, 2.0]],
        ];
        let mut stat = Geo::default();
        for t in wall { stat.push(std::iter::once((t, 0u8))); }
        stat.chunk(16);
        let cell_centre = |lo: V3, hi: V3| [0, 1, 2].map(|k| (lo[k] + hi[k]) / 2.0);
        let mut graph = ScopeGraph {
            cells: vec![([-8.0, -8.0, -8.0], [1.9, 8.0, 8.0]), ([1.95, -8.0, -8.0], [8.0, 8.0, 8.0])],
            portals: Vec::new(),
            room: vec![true, true],
        };
        // The wall's centre (2, 1, 0) → cell 1. Bucket by the same rule build_scoped uses.
        let mut scope_cells = vec![Vec::new(); graph.cells.len()];
        let mut always = Vec::new();
        for (run, _) in stat.owners.iter().enumerate() {
            let lo = stat.objects[run].lo; let hi = stat.objects[run].hi;
            let c = cell_centre(lo, hi);
            match graph.cells.iter().position(|(lo, hi)| (0..3).all(|k| lo[k] <= c[k] && c[k] <= hi[k])) {
                Some(i) => scope_cells[i].push(run as u32),
                None => always.push(run as u32),
            }
        }
        let mk = |graph: ScopeGraph, cells: Vec<Vec<u32>>| CollisionWorld {
            stat: Geo { tris: stat.tris.clone(), mats: stat.mats.clone(), objects: stat.objects.clone(), groups: Vec::new(), owners: Vec::new(), pass: stat.pass.clone() },
            missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(),
            scope: Some(ScopeData { graph, cells, always: always.clone() }),
        };
        let w = mk(graph.clone(), scope_cells.clone());
        // The capsule stands in cell 0, 1 unit west of the wall: the flat world pushes it out; the
        // scoped one does not (cell 1 is portal-unreachable).
        let (flat, _) = w.capsule_push([1.6, 1.0, 0.0], [1.6, 1.2, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert!(flat[0] < -0.1, "flat: pushed west of the wall {flat:?}");
        let (scoped, _) = w.capsule_push_scoped(Some(0), [1.6, 1.0, 0.0], [1.6, 1.2, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert_eq!(scoped, [0.0; 3], "scoped: the portal-unreachable wall is not tested");
        // The floor ray behaves the same way: a ray THROUGH the wall plane.
        assert!(w.segment_hit([1.5, 1.0, 0.0], [2.5, 1.0, 0.0]).is_some(), "flat segment_hit still sees the wall");
        assert!(w.segment_hit_scoped(Some(0), [1.5, 1.0, 0.0], [2.5, 1.0, 0.0]).is_none(), "the scoped floor ray skips the unreachable wall");
        // Sister: with a portal the capsule straddles (a quad across its path), the wall is back.
        graph.portals.push(ScopePortal { tris: [[[1.8, 0.0, -4.0], [1.8, 4.0, -4.0], [1.8, 4.0, 4.0]], [[1.8, 0.0, -4.0], [1.8, 4.0, 4.0], [1.8, 0.0, 4.0]]], a: 0, b: 1, centre: [1.8, 2.0, 0.0] });
        scope_cells[1].push(0);
        let w2 = mk(graph, scope_cells);
        let (scoped2, _) = w2.capsule_push_scoped(Some(0), [1.6, 1.0, 0.0], [1.6, 1.2, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert!(scoped2[0] < -0.1, "with the straddled portal the wall blocks again {scoped2:?}");
        // The take ray's surface filter: a mat-0xd triangle in the START cell is skipped by
        // segment_take_scoped (the 0x721 bit 0x1) while the flat segment_actor still hits it.
        let mut stat2 = Geo::default();
        let tri: [[V3; 3]; 1] = [[[-1.0, 0.0, 2.0], [0.0, 1.5, 2.0], [1.0, 0.0, 2.0]]];
        for t in tri { stat2.push(std::iter::once((t, 0xd))); }
        stat2.chunk(16);
        let w3 = CollisionWorld {
            stat: stat2, missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(),
            scope: Some(ScopeData { graph: ScopeGraph { cells: vec![([-8.0; 3], [8.0; 3])], portals: Vec::new(), room: vec![true] }, cells: vec![vec![0]], always: Vec::new() }),
        };
        assert!(w3.segment_actor([0.0, 0.5, 0.0], [0.0, 0.5, 4.0]).is_some(), "flat: the mat-0xd tri blocks");
        assert!(w3.segment_take_scoped(Some(0), [0.0, 0.5, 0.0], [0.0, 0.5, 4.0]).is_none(), "the take's 0x721 filter skips the mat-0xd tri");
        assert!(w3.segment_take_scoped(None, [0.0, 0.5, 0.0], [0.0, 0.5, 4.0]).is_some(), "unmapped cell: the flat fallback keeps the old behaviour");
    }

    /// level-11-stairwell-bullet-wall-4: the bullet flight's cel-scoped pick. [M Collide_Pick
    /// 0x1eb320:125 — the candidate nodes come ONLY from Collide_StraddleCels' BFS; the flying
    /// bullet re-feeds its start cel per tick, disasm 0x12f0cc-0x12f0d8] — a wall in a
    /// portal-unreachable cel stops the flat trace but not the scoped one; a straddled portal
    /// brings it back; the 0x40 pass drop rides the scoped walk like the flat one
    /// [M 0x1ebff0-0x1ec010]; an unmapped cell falls back to the flat answer.
    #[test]
    fn the_bullets_scoped_pick_skies_the_portal_unreachable_walls() {
        let wall = [
            [[2.0, 0.0, -2.0], [2.0, 2.0, 2.0], [2.0, 2.0, -2.0]],
            [[2.0, 0.0, -2.0], [2.0, 0.0, 2.0], [2.0, 2.0, 2.0]],
        ];
        let mut stat = Geo::default();
        for t in wall { stat.push(std::iter::once((t, 0x0au8))); }
        stat.chunk(16);
        let cell_centre = |lo: V3, hi: V3| [0, 1, 2].map(|k| (lo[k] + hi[k]) / 2.0);
        let mut graph = ScopeGraph {
            cells: vec![([-8.0, -8.0, -8.0], [1.9, 8.0, 8.0]), ([1.95, -8.0, -8.0], [8.0, 8.0, 8.0])],
            portals: Vec::new(),
            room: vec![true, true],
        };
        let mut scope_cells = vec![Vec::new(); graph.cells.len()];
        let mut always = Vec::new();
        for (run, _) in stat.owners.iter().enumerate() {
            let c = cell_centre(stat.objects[run].lo, stat.objects[run].hi);
            match graph.cells.iter().position(|(lo, hi)| (0..3).all(|k| lo[k] <= c[k] && c[k] <= hi[k])) {
                Some(i) => scope_cells[i].push(run as u32),
                None => always.push(run as u32),
            }
        }
        let mk = |graph: ScopeGraph, cells: Vec<Vec<u32>>| CollisionWorld {
            stat: Geo { tris: stat.tris.clone(), mats: stat.mats.clone(), objects: stat.objects.clone(), groups: Vec::new(), owners: Vec::new(), pass: stat.pass.clone() },
            missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(),
            scope: Some(ScopeData { graph, cells, always: always.clone() }),
        };
        let ray = ([1.5, 1.0, 0.0], [3.5, 1.0, 0.0]);
        let w = mk(graph.clone(), scope_cells.clone());
        assert!(w.segment_bullet(ray.0, ray.1).is_some(), "flat: the wall stops the round");
        assert!(w.segment_bullet_scoped(Some(0), ray.0, ray.1).is_none(),
            "scoped from cell 0: the wall's cell 1 is portal-unreachable — the round flies on [M 0x1eb320:125]");
        // Sister: a straddled portal brings the wall back into the pick.
        graph.portals.push(ScopePortal { tris: [[[1.8, 0.0, -4.0], [1.8, 4.0, -4.0], [1.8, 4.0, 4.0]], [[1.8, 0.0, -4.0], [1.8, 4.0, 4.0], [1.8, 0.0, 4.0]]], a: 0, b: 1, centre: [1.8, 2.0, 0.0] });
        scope_cells[1].push(0);
        let w2 = mk(graph, scope_cells);
        assert_eq!(w2.segment_bullet_scoped(Some(0), ray.0, ray.1), w2.segment_bullet(ray.0, ray.1),
            "with the straddled portal the scoped pick stops on the wall like the flat one");
        // The 0x40 pass drop inside the straddled cells: a pass-flagged plane in cell 0 in front of
        // a solid card in cell 0 — the scoped walk skips the plane and stops on the card behind.
        let mut stat3 = Geo::default();
        let fudge = [[[0.0, 0.0, -2.0], [0.0, 2.0, 2.0], [0.0, 2.0, -2.0]]];
        let card = [[[1.0, 0.0, -2.0], [1.0, 2.0, 2.0], [1.0, 2.0, -2.0]]];
        for t in fudge { stat3.push(std::iter::once((t, 0x40u8))); } // the surface byte's pass bit [M 0x1ebf78-80]
        for t in card { stat3.push(std::iter::once((t, 0x0au8))); }
        stat3.chunk(16);
        let w3 = CollisionWorld {
            stat: stat3, missile: Geo::default(), dynamic: Default::default(), shapes: Default::default(), shape_mats: Default::default(),
            scope: Some(ScopeData { graph: ScopeGraph { cells: vec![([-8.0; 3], [8.0; 3])], portals: Vec::new(), room: vec![true] }, cells: vec![vec![0, 1]], always: Vec::new() }),
        };
        assert_eq!(w3.segment_bullet_scoped(Some(0), [-1.0, 1.0, 0.0], [2.0, 1.0, 0.0]), w3.segment_bullet([-1.0, 1.0, 0.0], [2.0, 1.0, 0.0]),
            "the scoped walk keeps the flat walk's 0x40 drop: the fudge plane passes, the card stops");
        assert!(w3.segment_bullet_scoped(Some(0), [-1.0, 1.0, 0.0], [2.0, 1.0, 0.0]).unwrap() > 0.3,
            "the stop is the card at x=1, not the fudge at x=0");
        // Unmapped cell → the flat fallback (the safety net the other scoped queries share).
        assert_eq!(w.segment_bullet_scoped(None, ray.0, ray.1), w.segment_bullet(ray.0, ray.1), "unmapped cell: flat fallback");
        assert_eq!(w.segment_bullet_scoped(Some(9), ray.0, ray.1), w.segment_bullet(ray.0, ray.1), "out-of-range cell: flat fallback");
    }

    /// fix-foliage-bullets: the bullet-family queries (`segment_bullet`, `segment_surface_bullet`)
    /// drop every hit whose hit-data would carry the pass flag — the glist node's flags bit 0x40
    /// (`cel+0x94` [M parsemap_block_map_data_static 0x1d1998:103]) or the surface byte's bit 0x40
    /// (`Collide_Intersect` 0x1ebaa8 disasm 0x1ebf78-0x1ebf80 sets `HITDATA+0x51` bit 1 from
    /// either; bit 0x8 of the query's flags — the aim trace 0x20a/0x208, `Bullet_homing`'s LOS 8 —
    /// then drops the hit at disasm 0x1ebff0-0x1ec010) — while the flat queries still report them.
    /// The surface byte's bit 0x80 does NOT drop: its trace-side OR is gated on HITTEST+0x98
    /// bit 0x1 (disasm 0x1ebf8c-0x1ebf94), which the bullet flags clear — its 50/50 stays in the
    /// walk's roll (`Collide_Update` 0x1ea398:217-220).
    #[test]
    fn bullet_queries_drop_the_pass_flagged_hits() {
        // Rays fly +x. Three cards before a solid class-10 (0x0a) wall at x = 5:
        // x = 1 a node-flagged leaf card (class 0), x = 2 a mat-0x40 card (no node flag),
        // x = 3 a mat-0x80 card, all facing −x; the wall at x = 5 faces −x.
        let face = |x: f32, m: u8, pass: bool| -> ([[f32; 3]; 3], u8, bool) { ([[x, 0.0, -1.0], [x, 2.0, 1.0], [x, 2.0, -1.0]], m, pass) };
        let w = CollisionWorld::from_triangles_flags(vec![
            face(1.0, 0x00, true),   // the 07000005 tree nodes: glist flags 0x41 [D]
            face(2.0, 0x40, false),  // the leaf cards' material-level pass bit
            face(3.0, 0x80, false),  // the random-pass material: NOT dropped by the trace
            face(5.0, 0x0a, false),  // solid terrain
        ]);
        let from = [0.0f32, 1.0, 0.0];
        let to = [8.0f32, 1.0, 0.0];
        // The flat queries still see everything (sight, AI, the cage rays).
        assert!((w.segment(from, to).unwrap() - 1.0 / 8.0).abs() < 1e-6, "flat: the node-flagged leaf blocks");
        // The bullet trace reports only the mat-0x80 card at x = 3 (3/8 of the ray).
        let tb = w.segment_bullet(from, to).unwrap();
        assert!((tb - 3.0 / 8.0).abs() < 1e-6, "bullet: the node flag and the 0x40 byte drop, the 0x80 card is the first report (got t={tb})");
        let (_, n, m) = w.segment_surface_bullet(from, to).unwrap();
        assert_eq!(m, 0x80, "bullet surface: the first report is the 0x80 card");
        assert!(n[0] < 0.0, "the normal faces the ray");
        // Sister: remove the 0x80 card — the bullet trace reports the WALL behind the foliage.
        let w2 = CollisionWorld::from_triangles_flags(vec![face(1.0, 0x00, true), face(2.0, 0x40, false), face(5.0, 0x0a, false)]);
        let t2 = w2.segment_bullet(from, to).unwrap();
        assert!((t2 - 5.0 / 8.0).abs() < 1e-6, "bullet: only the wall remains (got t={t2})");
        let (_, _, m2) = w2.segment_surface_bullet(from, to).unwrap();
        assert_eq!(m2, 0x0a, "the hit surface is the class-10 wall behind the tree");
        // And the flat query still hits the leaf first there (the game's sight keeps the foliage).
        assert!((w2.segment(from, to).unwrap() - 1.0 / 8.0).abs() < 1e-6, "flat: the leaf still blocks sight");
    }

    #[test]
    fn moving_objects_collide_where_they_are() {
        // One unit triangle in object space as glist 7, placed at x = 5 by the matrix (wound so its
        // plane normal faces −x, the way the disc winds a wall that stops +x rays — 0x10a8c8).
        let mut w = world(Vec::new());
        w.shapes.insert(7, vec![[[0.0, 0.0, -1.0], [0.0, -1.0, 1.0], [0.0, 1.0, 1.0]]]);
        let at = |x: f32| [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [x, 0.0, 0.0, 1.0]];
        w.set_dynamic(&[(1, 7, at(5.0))]);
        assert!((w.segment([0.0, 0.0, 0.3], [10.0, 0.0, 0.3]).unwrap() - 0.5).abs() < 1e-5);
        // The ground moved 0.5 along x since the last call.
        w.set_dynamic(&[(1, 7, at(5.5))]);
        let g = w.ground_motion([0.0, 0.0, 0.3], [10.0, 0.0, 0.3]).unwrap();
        assert!((g.delta[0] - 0.5).abs() < 1e-6 && g.turn.abs() < 1e-6, "{g:?}");
        // Sister: moved away, the same segment is clear; an unknown glist adds nothing.
        w.set_dynamic(&[(1, 7, at(50.0)), (2, 8, at(5.0))]);
        assert!(w.segment([0.0, 0.0, 0.3], [10.0, 0.0, 0.3]).is_none());
        assert!(w.ground_motion([0.0, 0.0, 0.3], [10.0, 0.0, 0.3]).is_none());
    }

    #[test]
    fn segments_hit_floors_and_walls() {
        // A floor quad at y = 0 and a wall at z = 5.
        // Disc winding: the stored plane normal = +cross(v1−v0, v2−v0) faces UP on every real floor
        // (probe on the disc: 7,246 up / 0 down), and `Intersect_RayGeom` 0x10a8c8 skips a triangle
        // when dot(plane_normal, ray_dir) >= 0 — so a down-ray only hits an up-wound floor.
        let w = world(vec![
            [[-5.0, 0.0, -5.0], [5.0, 0.0, 5.0], [5.0, 0.0, -5.0]], [[-5.0, 0.0, -5.0], [-5.0, 0.0, 5.0], [5.0, 0.0, 5.0]],
            [[-5.0, 0.0, 5.0], [5.0, 3.0, 5.0], [5.0, 0.0, 5.0]], [[-5.0, 0.0, 5.0], [-5.0, 3.0, 5.0], [5.0, 3.0, 5.0]],
        ]);
        assert_eq!(w.floor_below([1.0, 1.1, 1.0], 3.0), Some(0.0));
        assert!(w.line_of_sight([0.0, 1.0, 0.0], [0.0, 1.0, 4.9], LOS_PROBE_8));
        assert!(!w.line_of_sight([0.0, 1.0, 0.0], [0.0, 1.0, 6.0], LOS_PROBE_8));
        assert!(w.line_of_sight([0.0, 4.0, 0.0], [0.0, 4.0, 6.0], LOS_PROBE_8), "over the wall");
    }

    /// The sight-class mask 0xa27 (`DroneVision_CanSeeObjectFrom` 0x176f18 ORs it into every
    /// `Collide_LineOfSight` — immediates 0x176fec/0x1770b0): the masked ray drops the hits the
    /// game's `Collide_Intersect` 0x1ebaa8 tail drops for it (0x40-flagged material byte
    /// [0x1ebaa8:301], materials 0xD/0xE [:311], material 0x10 [:314]) and skips the 0x1000-flagged
    /// nodes (mask bit 0x20, `Collide_Pick` 0x1eb320:149-152 — the port's missile cage), while the
    /// flat `segment` still reports them. A plain wall (any other material) blocks BOTH.
    #[test]
    fn sight_mask_drops_the_0xa27_classes_but_not_walls() {
        // Rays fly +x. Cards facing −x at x = 1..4, a solid class-10 (0x0a) wall at x = 6.
        let face = |x: f32, m: u8| -> ([[f32; 3]; 3], u8, bool) { ([[x, 0.0, -1.0], [x, 2.0, 1.0], [x, 2.0, -1.0]], m, false) };
        let from = [0.0f32, 1.0, 0.0];
        let to = [8.0f32, 1.0, 0.0];
        // Each class alone: the flat ray reports it at x/8, the sight ray reports the wall behind.
        for (x, m, why) in [
            (1.0f32, 0x40u8, "the 0x40 material byte [M 0x1ebaa8:301]"),
            (2.0, 0x0d, "material 0xD [M 0x1ebaa8:311]"),
            (3.0, 0x0e, "material 0xE [M 0x1ebaa8:311]"),
            (4.0, 0x10, "material 0x10 [M 0x1ebaa8:314]"),
        ] {
            let w = CollisionWorld::from_triangles_flags(vec![face(x, m), face(6.0, 0x0a)]);
            let traw = w.segment(from, to).unwrap();
            assert!((traw - x / 8.0).abs() < 1e-6, "flat segment reports the {why} card (t={traw})");
            let tm = w.segment_masked(from, to).unwrap();
            assert!((tm - 6.0 / 8.0).abs() < 1e-6, "sight ray skips the {why} card, first report is the wall (t={tm})");
        }
        // Sister (the sister that can fail): a plain wall is NOT a masked class — the sight ray
        // hits it exactly where the flat ray does.
        let w = CollisionWorld::from_triangles_flags(vec![face(2.0, 0x04), face(6.0, 0x0a)]);
        let t1 = w.segment(from, to).unwrap();
        let t2 = w.segment_masked(from, to).unwrap();
        assert!((t1 - 2.0 / 8.0).abs() < 1e-6 && (t2 - 2.0 / 8.0).abs() < 1e-6, "mat-4 geometry blocks both rays (t={t1}/{t2})");
        // The 0x1000-flagged nodes (the missile cage) are skipped by the sight ray (mask bit 0x20
        // [M 0x1eb320:149-152]) but stay solid to the flat segment — the same split segment_actor
        // already pins.
        let mut w = CollisionWorld::from_triangles_flags(vec![face(6.0, 0x0a)]);
        w.missile = Geo::default();
        w.missile.push(std::iter::once(([[3.0, 0.0, -1.0], [3.0, 2.0, 1.0], [3.0, 2.0, -1.0]], 0x0a)));
        w.missile.chunk(16);
        assert!(w.segment(from, to).unwrap() < 0.5, "flat: the cage card blocks");
        let tm = w.segment_masked(from, to).unwrap();
        assert!((tm - 6.0 / 8.0).abs() < 1e-6, "sight ray: the cage card is behind the mask bit 0x20 (t={tm})");
    }

    /// capsule-headon-cull-redo: the head-on cull is the game's exact rule [M Intersect_CylGeom
    /// 0x109da0:302-322] — |n·d̂| > 0.999 on the NORMALIZED spine [M the Vec_Normalise at
    /// 0x109f74 before the tri loop; the raw-dot reading is refuted by the stairwell
    /// (exit_aim_01_live) — see the capsule_tri_contact note], skip on the crossing-fraction
    /// half. This test feeds the 1.31-long CLIMB spine directly. A spine crossing an up-facing
    /// floor near its TOP end (f < 0.5) passes through — the jump-through case the old
    /// top-end-side rule also passed; the same face crossed near the BOTTOM end (f > 0.5) is
    /// solid. The old stand-in keyed the cull on |n·y| > 0.7, one band WIDER (0.70-0.763) and
    /// side-keyed on the top end's position: a ~43° slope face (n·y = 0.73) above the capsule was
    /// skipped outright; the game's |n·d̂| = 0.73 < 0.999 does not cull it — the capsule pushes.
    #[test]
    #[ignore = "HEAD_ON_CULL_ON is off until the cull keeps the real-path walks green"]
    fn head_on_cull_uses_the_raw_spine_dot_and_the_crossing_half() {
        // Up-facing floor at y = 0 (wound so cross(e1,e2) = +y), 20×20.
        let floor = vec![
            [[-10.0, 0.0, -10.0], [-10.0, 0.0, 10.0], [10.0, 0.0, 10.0]],
            [[-10.0, 0.0, -10.0], [10.0, 0.0, 10.0], [10.0, 0.0, -10.0]],
        ];
        let w = world(floor);
        // The climb spine: 1.31 long, vertical. Crossing at f = a_y / 1.31.
        // f = 0.4 (crossing near the top): the game's cull skips — pass-through from below.
        let (push, _) = w.capsule_push([0.0, 0.524, 0.0], [0.0, 0.524 - 1.31, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert_eq!(push, [0.0; 3], "f=0.4: the up-floor is jumped through from below (push {push:?})");
        // Sister (can fail): f = 0.6 (crossing near the bottom end): solid — pushed up onto the floor.
        let (push, _) = w.capsule_push([0.0, 0.786, 0.0], [0.0, 0.786 - 1.31, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert!(push[1] > 0.3, "f=0.6: the up-floor is solid underfoot (push {push:?})");
        // The 0.70-0.763 band the old stand-in over-culled: an UP-facing face with n_y = +0.73
        // (≈43°) hanging above the capsule. The game's raw dot 0.73·1.31 = 0.956 < 0.999 does not
        // cull it — the capsule below (on the face's back side) is pushed down off the underside.
        // The old |n·y| > 0.7 + top-end-side rule skipped it outright (push 0).
        // Verts: e1 = [−4,0,0], e2 = [0,−3.745,4] → n = [0, 16, 14.98], n̂ = [0, 0.73, 0.684];
        // the face's y at (0,0) is 1.6275, the spine top a_y = 1.2 sits 0.31 below the plane.
        let slope = vec![[
            [2.0, 3.5, -2.0],
            [-2.0, 3.5, -2.0],
            [2.0, 3.5 - 3.745, 2.0],
        ]];
        let w = world(slope);
        let (push, _) = w.capsule_push([0.0, 1.2, 0.0], [0.0, 1.2 - 1.31, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert!(push[1] < -0.05, "n·y=0.73 (in the old 0.7-0.763 over-cull band): the game's cull \
            does not skip it — the capsule is pushed off the underside (push {push:?})");
    }

    /// capsule-headon-cull-redo: the walk spine (0.7597 [L ee_start_05 BL 0x16cf0f0]) culls ONLY
    /// within the ~2.6° horizontal cone (|n·ŷ| > 0.999 on the normalized spine) — a 43° slope
    /// face above the capsule (n·y = 0.73, the old stand-in's 0.70-0.763 over-cull band) is
    /// TESTED and pushes; a FLAT floor crossed in the spine's TOP half (f < 0.5) IS culled — the
    /// game's jump-through, which the old top-end-side rule refused (a above the plane → push).
    #[test]
    #[ignore = "HEAD_ON_CULL_ON is off until the cull keeps the real-path walks green"]
    fn walk_spine_culls_only_near_flat_faces() {
        // Flat floor crossed at f = 0.4 of the walk spine: culled — pass-through from below.
        let floor = vec![
            [[-10.0, 0.0, -10.0], [-10.0, 0.0, 10.0], [10.0, 0.0, 10.0]],
            [[-10.0, 0.0, -10.0], [10.0, 0.0, 10.0], [10.0, 0.0, -10.0]],
        ];
        let w = world(floor);
        let (push, _) = w.capsule_push([0.0, 0.30388, 0.0], [0.0, 0.30388 - 0.7597, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert_eq!(push, [0.0; 3], "walk spine, flat floor at f=0.4: the game's cull skips it (push {push:?})");
        // Sister (can fail): the same floor crossed in the BOTTOM half (f = 0.6): solid.
        let (push, _) = w.capsule_push([0.0, 0.4558, 0.0], [0.0, 0.4558 - 0.7597, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert!(push[1] > 0.3, "walk spine, flat floor at f=0.6: solid underfoot (push {push:?})");
        // The 43° slope above the capsule: n·d̂ = 0.73 < 0.999 — NOT culled (the old |n·y| > 0.7
        // rule skipped it outright); the capsule below is pushed off the underside.
        let slope = vec![[
            [2.0, 3.5, -2.0],
            [-2.0, 3.5, -2.0],
            [2.0, 3.5 - 3.745, 2.0],
        ]];
        let w = world(slope);
        let (push, _) = w.capsule_push([0.0, 1.2, 0.0], [0.0, 1.2 - 0.7597, 0.0], 0.55, false, [1.0e9, 1.0e9, 1.0e9]);
        assert!(push[1] < -0.05, "n·y=0.73 on the walk spine: no cull — pushed off the underside (push {push:?})");
    }

    /// capsule-headon-cull-redo: `line_of_sight` applies the CALLER's game mask — the drops
    /// differ per mask [M Collide_Intersect 0x1ebaa8:300-319, Collide_Pick 0x1eb320:140-152]:
    /// guard vision 0xa27 drops surface-0x40 and the classes but NOT node-flagged cels and NOT
    /// the 0x1000 cage... (0xa27 DOES carry bit 0x20 — the cage is skipped); the crouch/jump
    /// headroom 0xd (9|4) drops node-flagged cels and the classes but NOT surface-0x40.
    #[test]
    #[ignore = "LOS_MASKS_ON is off until the masks keep guard cover and the 08 escort green"]
    fn line_of_sight_applies_the_callers_game_mask() {
        let face = |m: u8| [[2.0, 0.0, -1.0], [2.0, 2.0, 1.0], [2.0, 2.0, -1.0]];
        // A surface-0x40 card between the endpoints: guard vision (0x800) sees through it,
        // the crouch headroom (no 0x800 bit) does not.
        let w = CollisionWorld::from_triangles_flags(vec![(face(0x04), 0x44, false)]);
        assert!(w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_GUARD_VISION),
            "guard vision drops the surface-0x40 card [M 0x1ebaa8:300-302]");
        assert!(!w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_PLAYER_HEADROOM),
            "the headroom mask (9|4) has no 0x800 bit — the surface-0x40 card blocks");
        // A NODE-flagged card (the placement glist bit 0x40): the headroom mask's bit 0x8 drops
        // it; guard vision (0xa27 has NO bit 0x8) does NOT — the card blocks.
        let w = CollisionWorld::from_triangles_flags(vec![(face(0x04), 0x04, true)]);
        assert!(w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_PLAYER_HEADROOM),
            "the headroom mask drops node-flagged cels [M 0x1ebaa8:317-319]");
        assert!(!w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_GUARD_VISION),
            "0xa27 has no bit 0x8 — the node-flagged card blocks guard vision");
        // A class-0x10 card: both masks drop it (0xa27 bit 0x2, 0xd bit 0x1 covers 0xd/0xe only —
        // class 0x10 is dropped by bit 0x2, which 0xd lacks). So: guard vision drops, headroom blocks.
        let w = CollisionWorld::from_triangles_flags(vec![(face(0x04), 0x10, false)]);
        assert!(w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_GUARD_VISION),
            "guard vision drops class 0x10 [M 0x1ebaa8:314-316]");
        assert!(!w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_PLAYER_HEADROOM),
            "the headroom mask (9|4) has no 0x2 bit — class 0x10 blocks");
        // The 0x1000 cage (the port's missile set): guard vision's bit 0x20 skips it; the
        // headroom mask (no 0x20) still tests it.
        let mut w = CollisionWorld::from_triangles_flags(vec![]);
        w.missile.push(std::iter::once((face(0x0a), 0x0a)));
        w.missile.chunk(16);
        assert!(w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_GUARD_VISION),
            "guard vision skips the 0x1000 cage [M 0x1eb320:145-152]");
        assert!(!w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], LOS_PLAYER_HEADROOM),
            "the headroom mask has no 0x20 bit — the cage blocks");
        // Sister (can fail): an unflagged plain card blocks every mask.
        let w = CollisionWorld::from_triangles_flags(vec![(face(0x04), 0x04, false)]);
        for (name, m) in [("guard", LOS_GUARD_VISION), ("headroom", LOS_PLAYER_HEADROOM)] {
            assert!(!w.line_of_sight([0.0, 1.0, 0.0], [4.0, 1.0, 0.0], m), "{name}: plain geometry blocks");
        }
    }

    /// level-11-stairwell-bullet-wall: the assertion-free bullet-path census. From each reported
    /// Bond spot toward the rope guards [D the reported spot], list EVERY triangle the port's collision
    /// sets hold on the ray (static / missile cage / dynamic), with the owner's identity (map,
    /// object, name, glist flags), the material byte and the pass flags — the candidate invisible
    /// wall the game's bullet passes. Run with --ignored --nocapture.
    #[test]
    #[ignore = "assertion-free census; run with --ignored --nocapture"]
    fn stairwell_bullet_census_07000011() {
        use nf_formats::iso9660::IsoImage;
        let iso = std::env::var("NF_ISO").expect("NF_ISO");
        let img = IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).expect("archive");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let level = Level::parse(&data).expect("parse 07000011");
        let instances = level.instances(true).0;
        let w = CollisionWorld::build(&level, &instances);
        // The pushed static instances in push order (build_scoped's filter: collision present,
        // not the 0x20 shadow flag, not the 0x1000 cage) — the identity table for stat.owners.
        let mut stat_src: Vec<(usize, usize)> = Vec::new();
        for inst in &instances {
            let obj = &level.maps[inst.map].objects[inst.object];
            if obj.collision.is_none() { continue; }
            if obj.flags & 0x20 != 0 || obj.flags & 0x1000 != 0 { continue; }
            stat_src.push((inst.map, inst.object));
        }
        let spots: [[f32; 3]; 3] = [[15.91, 24.01, -2.45], [20.35, 24.01, -2.44], [15.33, 26.69, 1.33]];
        let guards: [[f32; 3]; 2] = [[18.895, 46.516, 1.434], [19.745, 46.580, 0.363]];
        for (si, s) in spots.iter().enumerate() {
            for (gi, g) in guards.iter().enumerate() {
                let d0 = [g[0] - s[0], g[1] - s[1], g[2] - s[2]];
                let len = (d0[0] * d0[0] + d0[1] * d0[1] + d0[2] * d0[2]).sqrt();
                let dir = [d0[0] / len, d0[1] / len, d0[2] / len];
                let far = 40.0f32;
                let to = [s[0] + dir[0] * (len + far), s[1] + dir[1] * (len + far), s[2] + dir[2] * (len + far)];
                eprintln!("\n[spot {} -> guard {}] from ({:.2},{:.2},{:.2}) guard ({:.2},{:.2},{:.2}) guard-dist {:.2} far-end ({:.2},{:.2},{:.2})",
                    si, gi, s[0], s[1], s[2], g[0], g[1], g[2], len, to[0], to[1], to[2]);
                let dd = sub(to, *s);
                let mut hits: Vec<(f32, &'static str, String, u8, bool, [f32; 3])> = Vec::new();
                let dynamic = w.dynamic.read().unwrap();
                let sets: [(&'static str, &Geo); 3] = [("stat", &w.stat), ("missile", &w.missile), ("dyn", &dynamic.geo)];
                for (set, geo) in sets {
                    for (ri, o) in geo.objects.iter().enumerate() {
                        let id = if set == "stat" {
                            let src = geo.owners.get(ri).copied().unwrap_or(u32::MAX) as usize;
                            match stat_src.get(src) {
                                Some((m, oi)) => {
                                    let ob = &level.maps[*m].objects[*oi];
                                    format!("map{}#{} '{}' glist {:#x}", m, oi, ob.name, ob.flags)
                                }
                                None => format!("run{ri}?"),
                            }
                        } else {
                            format!("run{ri}")
                        };
                        for k in o.tris.clone() {
                            let t3 = &geo.tris[k];
                            let e1 = sub(t3[1], t3[0]);
                            let e2 = sub(t3[2], t3[0]);
                            if dot(cross(e1, e2), dd) >= 0.0 { continue; } // back face
                            let pv = cross(dd, e2);
                            let det = dot(e1, pv);
                            if det.abs() < 1e-12 { continue; }
                            let inv = 1.0 / det;
                            let sv = sub(*s, t3[0]);
                            let u = dot(sv, pv) * inv;
                            if !(0.0..=1.0).contains(&u) { continue; }
                            let qv = cross(sv, e1);
                            let v = dot(dd, qv) * inv;
                            if v < 0.0 || u + v > 1.0 { continue; }
                            let tt = dot(e2, qv) * inv;
                            if (0.0..=1.0).contains(&tt) {
                                let pt = [s[0] + dd[0] * tt, s[1] + dd[1] * tt, s[2] + dd[2] * tt];
                                hits.push((tt, set, id.clone(), geo.mats[k], geo.bullet_pass(k), pt));
                            }
                        }
                    }
                }
                hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                for (tt, set, id, mat, pass, pt) in hits.iter().take(14) {
                    eprintln!("  t={:.4} {:>7} mat={:#04x} pass={} at ({:.2},{:.2},{:.2}) owner {}",
                        tt, set, mat, pass, pt[0], pt[1], pt[2], id);
                }
                eprintln!("  port segment={:?} segment_bullet={:?}",
                    w.segment(*s, to), w.segment_bullet(*s, to));
            }
        }
    }

    /// level-11-stairwell-bullet-wall-4: the assertion-free census for the second reported spot.
    /// Bond (22.40, 30.09, 1.42, yaw -1.505, pitch -0.035) frame 2944 fires into the well and the
    /// decals hang mid-air on two flat planes (one diagonal under the stair flight, one upright
    /// across the middle of the well). From that spot a fan of rays crosses the well; EVERY hit is
    /// listed with the owner's identity (map, object, name, glist flags — bit 0x10 = not drawn
    /// [M View_AddCels 0x1e6ba0]), the surface byte, its low-6-bit class, the port's pass flag and
    /// whether the owner has any DRAWN geometry [D the parsed gfx leaves]. Run --ignored --nocapture.
    #[test]
    #[ignore = "assertion-free census; run with --ignored --nocapture"]
    fn stairwell_walls4_census_07000011() {
        use nf_formats::iso9660::IsoImage;
        let iso = std::env::var("NF_ISO").expect("NF_ISO");
        let img = IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).expect("archive");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let level = Level::parse(&data).expect("parse 07000011");
        let instances = level.instances(true).0;
        let w = CollisionWorld::build(&level, &instances);
        let mut stat_src: Vec<(usize, usize)> = Vec::new();
        let mut stat_src_plc: Vec<u32> = Vec::new();
        for inst in &instances {
            let obj = &level.maps[inst.map].objects[inst.object];
            if obj.collision.is_none() { continue; }
            if obj.flags & 0x20 != 0 || obj.flags & 0x1000 != 0 { continue; }
            stat_src.push((inst.map, inst.object));
            stat_src_plc.push(inst.placement.flags);
        }
        // The reported spot and aim [D the reported spot]: yaw -1.505, pitch -0.035 → the port's look()
        // lanes [M crates/nf-game/src/player.rs Bond::look]: p = pitch·π/2, dir = [cp·sy, sp, cp·cy].
        let s = [22.40f32, 30.09, 1.42];
        let p = (-0.035f32) * (std::f32::consts::PI / 2.0);
        let (sp, cp) = (p.sin(), p.cos());
        let (sy, cy) = ((-1.505f32).sin(), (-1.505f32).cos());
        let base = [cp * sy, sp, cp * cy];
        eprintln!("[walls4] spot ({:.2},{:.2},{:.2}) base dir [{:.4},{:.4},{:.4}] stat tris {} objects {}", s[0], s[1], s[2], base[0], base[1], base[2], w.stat.tris.len(), w.stat.objects.len());
        // The decal planes: fire across the well. A fan over yaw ±0.45 and pitch ±0.22.
        let mut planes: std::collections::BTreeMap<(String, u8, bool), (usize, f32, f32, [f32;3], bool)> = Default::default();
        let dynamic = w.dynamic.read().unwrap();
        for yaw_off in [-0.45f32, -0.3, -0.15, -0.075, 0.0, 0.075, 0.15, 0.3, 0.45] {
            for pitch_off in [-0.22f32, -0.11, 0.0, 0.11, 0.22] {
                let a = yaw_off + (-1.505f32);
                let q = (pitch_off * (std::f32::consts::PI / 2.0)) + p;
                let (sq, cq) = (q.sin(), q.cos());
                let (sa, ca) = (a.sin(), a.cos());
                let dir = [cq * sa, sq, cq * ca];
                let far = 70.0f32;
                let to = [s[0] + dir[0] * far, s[1] + dir[1] * far, s[2] + dir[2] * far];
                let dd = sub(to, s);
                let sets: [(&str, &Geo); 3] = [("stat", &w.stat), ("missile", &w.missile), ("dyn", &dynamic.geo)];
                for (set, geo) in sets {
                    for (ri, o) in geo.objects.iter().enumerate() {
                        for k in o.tris.clone() {
                            let t3 = &geo.tris[k];
                            let e1 = sub(t3[1], t3[0]);
                            let e2 = sub(t3[2], t3[0]);
                            if dot(cross(e1, e2), dd) >= 0.0 { continue; }
                            let pv = cross(dd, e2);
                            let det = dot(e1, pv);
                            if det.abs() < 1e-12 { continue; }
                            let inv = 1.0 / det;
                            let sv = sub(s, t3[0]);
                            let u = dot(sv, pv) * inv;
                            if !(0.0..=1.0).contains(&u) { continue; }
                            let qv = cross(sv, e1);
                            let v = dot(dd, qv) * inv;
                            if v < 0.0 || u + v > 1.0 { continue; }
                            let tt = dot(e2, qv) * inv;
                            if !(0.0..=1.0).contains(&tt) { continue; }
                            let pt = [s[0] + dd[0] * tt, s[1] + dd[1] * tt, s[2] + dd[2] * tt];
                            let (id, drawn) = if set == "stat" {
                                let src = geo.owners.get(ri).copied().unwrap_or(u32::MAX) as usize;
                                match stat_src.get(src) {
                                    Some((m, oi)) => {
                                        let ob = &level.maps[*m].objects[*oi];
                                        let drawn = !ob.gfx.leaves.is_empty();
                                        (format!("map{}#{} '{}' glist {:#x}", m, oi, ob.name, ob.flags), drawn)
                                    }
                                    None => (format!("run{ri}?"), false),
                                }
                            } else { (format!("{set} run{ri}"), false) };
                            let mat = geo.mats[k];
                            let key = (id.clone(), mat, geo.bullet_pass(k));
                            let e = planes.entry(key).or_insert((0, tt, tt, pt, drawn));
                            e.0 += 1;
                            e.1 = e.1.min(tt);
                            e.2 = e.2.max(tt);
                        }
                    }
                }
            }
        }
        eprintln!("[walls4] distinct hit planes (owner, mat, port-pass) — rays 45, far 70: map {} entries", planes.len());
        for ((id, mat, pass), (n, t0, t1, pt, drawn)) in &planes {
            eprintln!("  {:>3} hits t {:.3}..{:.3} at ({:.2},{:.2},{:.2}) mat={:#04x} class={:#x} port_pass={} drawn={} | {}",
                n, t0, t1, pt[0], pt[1], pt[2], mat, mat & 0x3f, pass, drawn, id);
        }
        // The one-segment view: what the flight sees on the centre ray, plus 4 near-centre rays —
        // every hit in order with the owner's placement flags and the triangle's vertices (the
        // diagonal-vs-upright read).
        let mut rays: Vec<(&str, [f32; 3])> = vec![("centre", base)];
        for (nm, yo, po) in [("yaw-0.075", -0.075f32, 0.0), ("yaw+0.075", 0.075, 0.0), ("pitch-0.11", 0.0, -0.11), ("pitch+0.11", 0.0, 0.11)] {
            let a = yo + (-1.505f32);
            let q = (po * (std::f32::consts::PI / 2.0)) + p;
            let (sq, cq) = (q.sin(), q.cos());
            let (sa, ca) = (a.sin(), a.cos());
            rays.push((nm, [cq * sa, sq, cq * ca]));
        }
        for (nm, dir) in &rays {
            let far = 70.0f32;
            let to = [s[0] + dir[0] * far, s[1] + dir[1] * far, s[2] + dir[2] * far];
            let dd = sub(to, s);
            eprintln!("[walls4] ray {nm} dir [{:.4},{:.4},{:.4}]:", dir[0], dir[1], dir[2]);
            let mut hits: Vec<(f32, String, u8, bool, [[f32; 3]; 3])> = Vec::new();
            let sets: [(&str, &Geo); 3] = [("stat", &w.stat), ("missile", &w.missile), ("dyn", &dynamic.geo)];
            for (set, geo) in sets {
                for (ri, o) in geo.objects.iter().enumerate() {
                    for k in o.tris.clone() {
                        let t3 = &geo.tris[k];
                        let e1 = sub(t3[1], t3[0]);
                        let e2 = sub(t3[2], t3[0]);
                        if dot(cross(e1, e2), dd) >= 0.0 { continue; }
                        let pv = cross(dd, e2);
                        let det = dot(e1, pv);
                        if det.abs() < 1e-12 { continue; }
                        let inv = 1.0 / det;
                        let sv = sub(s, t3[0]);
                        let u = dot(sv, pv) * inv;
                        if !(0.0..=1.0).contains(&u) { continue; }
                        let qv = cross(sv, e1);
                        let v = dot(dd, qv) * inv;
                        if v < 0.0 || u + v > 1.0 { continue; }
                        let tt = dot(e2, qv) * inv;
                        if !(0.0..=1.0).contains(&tt) { continue; }
                        let (id, drawn) = if set == "stat" {
                            let src = geo.owners.get(ri).copied().unwrap_or(u32::MAX) as usize;
                            match stat_src.get(src) {
                                Some((m, oi)) => {
                                    let ob = &level.maps[*m].objects[*oi];
                                    let drawn = !ob.gfx.leaves.is_empty();
                                    (format!("map{}#{} '{}' glist {:#x} plcflags {:#x} drawn {}", m, oi, ob.name, ob.flags, stat_src_plc.get(src).copied().unwrap_or(0), drawn), drawn)
                                }
                                None => (format!("run{ri}?"), false),
                            }
                        } else { (format!("{set} run{ri}"), false) };
                        hits.push((tt, format!("{id} mat={:#04x} port_pass={}", geo.mats[k], geo.bullet_pass(k)), geo.mats[k], geo.bullet_pass(k), *t3));
                    }
                }
            }
            hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for (tt, id, _, _, tri) in hits.iter().take(10) {
                eprintln!("  t={:.4} {} at ({:.2},{:.2},{:.2}) tri [({:.1},{:.1},{:.1})({:.1},{:.1},{:.1})({:.1},{:.1},{:.1})]",
                    tt, id, s[0] + dd[0] * tt, s[1] + dd[1] * tt, s[2] + dd[2] * tt,
                    tri[0][0], tri[0][1], tri[0][2], tri[1][0], tri[1][1], tri[1][2], tri[2][0], tri[2][1], tri[2][2]);
            }
        }
        let to = [s[0] + base[0] * 70.0, s[1] + base[1] * 70.0, s[2] + base[2] * 70.0];
        eprintln!("[walls4] centre segment={:?} segment_bullet={:?}", w.segment(s, to), w.segment_bullet(s, to));
    }
}
