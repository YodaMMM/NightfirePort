//! The level's runtime cel graph + the game's portal-visible walk — moved here (level-0a-civilian-slide-cels)
//! so the SIM's drawn gate (`Drone::drawn`, the `obj+0xFC` stand-in) can run the game's own room rule
//! (`View_AddCels` 0x1e6ba0's VisCelList: the portal walk from the viewer's cel, no rays) beside the
//! renderer, which keeps using the same types through `nf_view::scene`'s re-export. No draw-path
//! behaviour changes: the code below is nf-view/src/scene.rs's, verbatim.
//!
//! Every function's evidence citations ([M]/[D]/[L]) are the original ones and travel with the code.

/// The level's cel graph, built alongside `parse_room_cels` (which is this function's room half —
/// the two MUST agree; `Scene::cels` is `.0`).
///
/// `cells` spans the WHOLE parse in archive order — the game's pCelList record array is parse-wide
/// (parsemap_block_map_data_static appends one 0x50-byte record per cel placement as each map is
/// parsed, sub-maps first, main map last), and the main map's portal block indexes THAT array.
/// *Measured* [D]: 07000002's main-map portal records point past the main map's own cel placements
/// (22 of 37 endpoints) — they address sub-map cels too.
///
/// The pCelList record index: `parsemap_block_map_data_static` (0x1d1998) addresses the record as
/// `pCelList + placement.index * 0x50` for by-index placements (the by-hash path goes through the
/// entity hashtable, untraced — those records are unreachable here and the portals naming them are
/// counted in `dropped_portals`). Records the port cannot resolve are the *sparse* remainder of the
/// game's array; only the named ones exist as nodes.
#[derive(Clone)]
pub struct CelGraph {
    pub cells: Vec<CelNode>,
    pub portals: Vec<PortalEdge>,
    pub room_to_cell: Vec<Option<usize>>,
    /// Portal records dropped because an endpoint cel placement did not resolve.
    pub dropped_portals: usize,
    /// `View_AddCels` (0x1e6ba0)'s `cel+0x18` chain per cell. The chain threads through the cel
    /// records' +0x18 field [M 0x1e6ba0: the head is the VisCelList cel's +0x18, each node's next
    /// is node+0x18]; only room-class cels own chains and only the generic (runtime class 0x8019/
    /// 0x8018) cels ride them as nodes. Built by `build_chains`: each generic hangs on the room
    /// `build_FindCel` (0x1c8cb0) picks for its world-sphere centre — the bbox candidates plus the
    /// collision up/down ray tie-break [L ee_start_05.bin + ee_snapshot.bin: 8 room owners, 113
    /// members, 0 members outside their owner's bbox, 65/65 multi-bbox owners matching the
    /// FindCel tie-break; L live 07000006: the eye cel's chain empty, the big room 0x0179fb30's
    /// chain = 40 generics all inside its bbox]. Node order within a chain is reverse parse
    /// order [L ground-truth doc: the 40-node dump descends by runtime address] — the builder
    /// prepends. fix-portal4 (the old per-map-block parse-order model is refuted by that data).
    pub chains: Vec<Vec<u32>>,
    /// The main map's placements' own `cells` index, by placement ordinal (`None` = the placement
    /// does not create a cel). A cel placement's glist draws when ITS cel is visible
    /// (`View_AddCels` 0x1e6ba0), not when the room its centre lands in is. fix-portal2.
    pub own_cells_main: Vec<Option<u32>>,
    /// Per portal edge, the linked door's `Doors::list` index — `build_LinkDoors2Portals`
    /// (0x1c8ac8, block 0x21's door half) writes the door object into the runtime portal's +0xC
    /// (both halves; `Door_Link2Portal` 0x136788 gates which doors qualify). `None` = no door on
    /// that portal. The vision walk skips a linked portal while `Door_IsClosed` (0x1367b0) reads
    /// closed [M `Vision_Recurse` 0x1e82e0:65-70]. Built by `link_doors_to_portals` at load.
    pub portal_doors: Vec<Option<u32>>,
}

#[derive(Clone, Copy)]
pub struct CelNode {
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// The runtime cel's flags word (`cel+0x3c`), derived per class by `parseentity_fixup_entity`
    /// 0x1d00c0: the 0xc023/0xc047 room classes become **0x804c023/0x804c047** (bit 0x40000 = the
    /// room bit `build_FindCel` 0x1c8cb0 collects; the placement's raw word 0xc047 does NOT carry
    /// it), the generic classes 0x8019. fix-portal.
    pub flags: u32,
    /// The cel's world (centre, radius) sphere — `cel+0x80..0x8c`, the pCelList record's +0x10..0x1c
    /// local sphere transformed by the placement matrix (`parseentity_transform_bounding_box`
    /// 0x1d0fb0 [M]; the port derives it from the placement-transformed bbox corners). This is what
    /// `Vision_InView` (0x1e89e0) tests for the `+0x18` chain pass; `min`/`max` stay raw (the game's
    /// `build_FindCel` 0x1c8cb0 tests the raw glist bbox copied to cel+0x40..0x58). fix-portal2.
    pub sphere: ([f32; 3], f32),
}

/// One block-0x21 portal as two half-edges: `corners` in file order, the FORWARD plane built
/// `Plane_PlaneEq(plane, ref = corner 0, corner 1, corner 2)` [M 0x1d15f8] — its normal points into
/// celA, so the runtime node attached to celB (dest celA) carries it [M 0x1c8928] and the node
/// attached to celA carries its reverse. Side flags per record [M 0x1d15f8]: the celB node walks
/// while `(flags & 2) == 0`, the celA node while `(flags & 1) == 0`.
#[derive(Clone, Copy)]
pub struct PortalEdge {
    pub corners: [[f32; 3]; 4],
    pub n_fwd: [f32; 3],
    pub d_fwd: f32,
    pub cel_a: usize,
    pub cel_b: usize,
    pub flags: u16,
}

/// `Plane_PlaneEq` (0x1df8d0) [M]: n = normalize(cross(v1 − r, v2 − v1)), d = −dot(n, r); the zero
/// vector when the points are collinear (the game stores the zero plane and the walk's dot test then
/// never passes — the same effect here). Value form: `dot(n, p) + d`, zero on the plane.
pub fn plane_eq(r: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 4] {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let e = sub(v2, v1);
    let cr = cross(sub(v1, r), e);
    let len = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
    if len == 0.0 { return [0.0; 4]; }
    let n = [cr[0] / len, cr[1] / len, cr[2] / len];
    [n[0], n[1], n[2], -(n[0] * r[0] + n[1] * r[1] + n[2] * r[2])]
}

/// `Vision_InView` (0x1e89e0) [M]: visible iff the node's (centre, radius) sphere meets each of the
/// four viewer+0x70 side planes. The port reads the game's test as the signed-distance form
/// `n·c + d + r >= 0`. The RAW decompile is `dot((cx,cy,cz,r),(n,d)) < r -> reject` (c.lt.s $f0, $f12
/// + bc1t -> return 0; the decompile's `< -param_1` is a Ghidra polarity artifact), i.e. the literal
/// `n·c + d·r >= r` — but that form is untestable-absurd with the game's OWN stored planes [L
/// ee_start_05.bin: |n| = 1, n·eye + d = 0, d = 51..117 — the fustrum builder 0x1e7df8 anchors every
/// side plane at the eye through the far-quad corners, so d = -n·eye scales with the eye's distance
/// from the world origin]: at the 05 start (|eye| = 153) the d·r term adds 130..300 of slack per
/// plane (the cull degenerates to draw-everything), and at the 07000007 clock-hall spot (the eye
/// looking away from the origin, d = -69.6 on the left plane) it subtracts 184 and rejects the
/// grandfather clock at r = 2.65 that the real game DRAWS [L ground truth]. The two readings agree
/// exactly when |d| << 1, and the signed-distance form is that common limit — the port ships it and
/// lists the open question (what the game's +0x70 quads hold mid-gameplay) under the report's live
/// checks. fix-portal6 (the fix-portal5 regression: every chain node of a visited room failed this
/// gate at the clock hall, dropping the clock + the wall shields the game shows).
/// The port derives (centre, radius) from the cel's glist bbox (the game's sphere comes from the
/// pCelList record's +0x10..0x1c) — documented as the approximation. fix-portal2.
pub fn vision_in_view(centre: [f32; 3], radius: f32, sides: &[[f32; 4]; 4]) -> bool {
    sides.iter().all(|p| p[0] * centre[0] + p[1] * centre[1] + p[2] * centre[2] + p[3] + radius >= 0.0)
}

/// A cel's (centre, radius) sphere from its bbox — the `Vision_InView` input.
fn cel_sphere(c: &CelNode) -> ([f32; 3], f32) { c.sphere }

/// The game's World-glist visibility: the VisCelList walk PLUS `View_AddCels` (0x1e6ba0)'s `+0x18`
/// chain pass — for every visited cel, each chain node draws when `Vision_InView` passes, skipping
/// 0x8018-class nodes while the viewer's cel is 0xc023-class [M 0x1e6ba0:47-56]. `sides` = the
/// frustum's four side planes (the viewer+0x70 set). Returns the per-`cells` mask; the strict
/// VisCelList mask (what `View_DrawObjList`'s object cull keys on — chain nodes never enter the
/// VisCelList, `Vision_AddCelToDraw` 0x1e8908 is the only inserter) stays
/// `portal_visible_cels_from_cell`. fix-portal2.
pub fn portal_visible_cels_game_from_cell(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], sides: &[[f32; 4]; 4]) -> Option<Vec<bool>> {
    portal_visible_cels_game_from_cell_doors(graph, start, eye, depth0, sides, None)
}

/// The game mask with the door links honoured (`Vision_Recurse` 0x1e82e0's `Door_IsClosed` gate,
/// G-035) — the draw path's walk. `door_closed` = `Doors::closed_flags()` at this tick; `None` =
/// the doors-less walk.
pub fn portal_visible_cels_game_from_cell_doors(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], sides: &[[f32; 4]; 4], door_closed: Option<&[bool]>) -> Option<Vec<bool>> {
    let chain = portal_cel_chain_from_cell_doors(graph, start, eye, depth0, door_closed)?;
    let mut vis = vec![false; graph.cells.len()];
    for &c in &chain { vis[c] = true; }
    let start_class = graph.cells[start].flags & 0xffff;
    for &c in &chain {
        for &n in &graph.chains[c] {
            let n = n as usize;
            if vis[n] { continue; }
            let node = &graph.cells[n];
            if node.flags & 0xffff == 0x8018 && start_class == 0xc023 { continue; }
            let (c3, r) = cel_sphere(node);
            if vision_in_view(c3, r, sides) { vis[n] = true; }
        }
    }
    Some(vis)
}

pub fn portal_visible_cels(graph: &CelGraph, eye_room: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<bool>> {
    let start = graph.room_to_cell.get(eye_room).copied().flatten()?;
    portal_visible_cels_from_cell(graph, start, eye, depth0)
}

/// The walk from an already-resolved runtime cel (the `viewer+4` the game hands
/// `Vision_Portal_Recurse` 0x1e86e8 — `Camera_CheckLocation` 0x1bae90's state, not a per-frame
/// re-pick). fix-portal.
pub fn portal_visible_cels_from_cell(graph: &CelGraph, start_cell: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<bool>> {
    let chain = portal_cel_chain_from_cell(graph, start_cell, eye, depth0)?;
    let mut visible = vec![false; graph.cells.len()];
    for c in chain { visible[c] = true; }
    Some(visible)
}

/// `portal_visible_cels_from_cell` with the door links honoured (`Vision_Recurse` 0x1e82e0's
/// `Door_IsClosed` gate — G-035): `door_closed` = `Doors::closed_flags()` at this tick.
pub fn portal_visible_cels_from_cell_doors(graph: &CelGraph, start_cell: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], door_closed: Option<&[bool]>) -> Option<Vec<bool>> {
    let chain = portal_cel_chain_from_cell_doors(graph, start_cell, eye, depth0, door_closed)?;
    let mut visible = vec![false; graph.cells.len()];
    for c in chain { visible[c] = true; }
    Some(visible)
}

/// The game's `VisCelList` (0x34b3b0) chain order — what `View_AddCels` (0x1e6ba0) walks when it
/// hands out the −90000 + 10·k list-0 keys. `Vision_Portal_Recurse` (0x1e86e8) inserts the viewer's
/// cel at depth 0, `Vision_Recurse` (0x1e82e0) inserts each portal-reached cel at **first reach**
/// with depth = parent+1 (`*(short *)(&DAT_0034b3b4 + uVar9 * 8) = (short)iVar11`), then
/// `QuickSort` (0x1d30a8, comparator id 3) orders the array and it is threaded through `cel+0x20`.
/// Returns the sorted `cells` indices in that order (the viewer's cel first), or None when the walk
/// cannot run (same guards as `portal_visible_cels`, whose visibility mask this also is).
pub fn portal_cel_chain(graph: &CelGraph, eye_room: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<usize>> {
    let start = graph.room_to_cell.get(eye_room).copied().flatten()?;
    portal_cel_chain_from_cell(graph, start, eye, depth0)
}

/// The chain from a resolved cell — see `portal_cel_chain`.
pub fn portal_cel_chain_from_cell(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<usize>> {
    portal_cel_chain_from_cell_doors(graph, start, eye, depth0, None)
}

/// The chain from a resolved cell, honouring the door links (`build_LinkDoors2Portals` 0x1c8ac8):
/// `door_closed` is the per-door `Door_IsClosed` (0x1367b0) snapshot (`Doors::closed_flags`), read
/// per portal the same order the game reads it — after the eye-side plane test, before the clip
/// [M `Vision_Recurse` 0x1e82e0:50-70]. `None` = the doors-less walk (the pre-G-035 behaviour).
///
/// This chain is the game's capture pipeline's tail, whose three never-mentioned members run
/// inside `View_CaptureScene` 0x1e6df0 [M its decomp — the only caller of all three]:
/// `Vision_Init_Portal_Recurse` 0x1e89a8 (reset `VisCelCount = 0`, `Mat_GetDir(viewer+0x120)`,
/// `Vision_Calculate_View_Fustrum` — the port's `depth0` plane build + chain start), the
/// `Vision_Portal_Recurse`/`Vision_Recurse` walk below, and `vision_generate_display_list`
/// 0x1e8848 (`QuickSort` comparator 3 over `VisCelList` by recursion depth, then the cels
/// threaded through `cel+0x20` with the sorted order — exactly the sort + order this fn
/// returns; the viewer+0x20 accumulate bit is the game's own draw bookkeeping). The eye
/// position itself is `vision_GetCamPos` 0x1e8990 (copies the 4 words at `ViewPoints`
/// 0x2d67d0 — the current viewer position — into a caller vector) — the port's `eye`
/// parameter, threaded from the frame camera by the renderer [M 001e8990/001e89a8/001e8848
/// decomp; ViewPoints in the symtab 0x2d67d0 size 80].
pub fn portal_cel_chain_from_cell_doors(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], door_closed: Option<&[bool]>) -> Option<Vec<usize>> {
    let mut visited = vec![false; graph.cells.len()];
    let mut planes: Vec<Vec<[f32; 4]>> = vec![Vec::new(); 48];
    planes[0] = depth0.to_vec();
    let mut budget = 0usize;
    // The VisCelList insertion order: the viewer's cel first (depth 0 — `Vision_Portal_Recurse`
    // inserts it before `Vision_Recurse` runs), then each first-reach cel at depth = parent+1.
    // `Vision_Recurse` [M 0x1e82e0]: the recursion budget (VisionRecurseCount > 100) and the depth
    // cap (45) are the game's own guards; VisCelCount caps at 0x40 cels.
    let mut order: Vec<(usize, u16)> = vec![(start, 0)];
    fn recurse(
        graph: &CelGraph, eye: [f32; 3], cell: usize, depth: usize,
        visited: &mut Vec<bool>, planes: &mut Vec<Vec<[f32; 4]>>, budget: &mut usize, order: &mut Vec<(usize, u16)>,
        door_closed: Option<&[bool]>,
    ) {
        *budget += 1;
        if *budget > 100 || depth > 45 { return; }
        let n0 = planes[depth].len();
        let dbg = { static F: std::sync::OnceLock<bool> = std::sync::OnceLock::new(); *F.get_or_init(|| std::env::var_os("NF_PORTAL_DEBUG").is_some()) };
        for (pi, p) in graph.portals.iter().enumerate() {
            let (side_open, dest, fwd) = if cell == p.cel_a {
                ((p.flags & 1) == 0, p.cel_b, false)
            } else if cell == p.cel_b {
                ((p.flags & 2) == 0, p.cel_a, true)
            } else { continue };
            if !side_open {
                if std::env::var_os("NF_PORTAL_DEBUG").is_some() { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: side closed (flags {:x})", p.flags); }
                continue;
            }
            // The side's plane: the node attached to celB carries the forward plane, the celA node
            // its reverse [M 0x1c8928 / 0x1d15f8]. `Vision_Recurse` processes a portal only when the
            // eye is on its negative side [M 0x1e82e0:50].
            let (n, d) = if fwd { (p.n_fwd, p.d_fwd) } else { ([-p.n_fwd[0], -p.n_fwd[1], -p.n_fwd[2]], -p.d_fwd) };
            if n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2] + d >= 0.0 {
                if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: eye on the wrong side of the portal plane"); }
                continue;
            }
            // `Vision_Recurse` 0x1e82e0:65-70: after the plane test, `Door_IsClosed` (0x1367b0)
            // on the portal's linked door (portal+0xC) ends this portal — a shut door's room is
            // not walked into (G-035). The game's walk runs the test for EVERY portal (a NULL
            // door reads open, `Door_IsClosed` 0x1367b0's `param_1 == 0` arm).
            if let Some(closed) = door_closed {
                if let Some(&Some(di)) = graph.portal_doors.get(pi) {
                    if closed.get(di as usize).copied().unwrap_or(false) {
                        if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: linked door {di} is shut"); }
                        continue;
                    }
                }
            }
            // The quad as the runtime node stores it: forward corners on the celB node, reversed
            // (c3,c2,c1,c0) on the celA node [M 0x1c8928's amemcpy sources].
            let mut poly: Vec<[f32; 3]> = if fwd { p.corners.to_vec() } else { vec![p.corners[3], p.corners[2], p.corners[1], p.corners[0]] };
            for k in 0..n0 {
                match clip_positive(&poly, planes[depth][k]) {
                    None => {}
                    Some(clipped) => { poly = clipped; }
                }
                if poly.is_empty() { break; }
            }
            if poly.is_empty() {
                if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: poly empty after clip (n0 {n0})"); }
                continue;
            }
            // The VisCelList insert cap: `if (VisCelCount < 0x40)` guards the dedup + insert
            // [M 0x1e82e0]; the recursion itself continues past the cap.
            if !visited[dest] && order.len() < 0x40 { visited[dest] = true; order.push((dest, (depth + 1) as u16)); }
            if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: poly {} (n0 {n0}, budget {budget}, {} new)", poly.len(), !visited[dest] as u8); }
            if poly.len() >= 3 {
                // The next depth's planes: one per clipped-polygon edge, ref = the eye (ViewPoints[0]
                // [M 0x1e82e0:146]), plus the portal's own plane last [M 0x1e82e0:151-155].
                let m = poly.len();
                let mut next: Vec<[f32; 4]> = (0..m).map(|i| plane_eq(eye, poly[i], poly[(i + 1) % m])).collect();
                next.push([n[0], n[1], n[2], d]);
                planes[depth + 1] = next;
                recurse(graph, eye, dest, depth + 1, visited, planes, budget, order, door_closed);
                if *budget > 100 { return; }
            }
        }
    }
    visited[start] = true;
    recurse(graph, eye, start, 0, &mut visited, &mut planes, &mut budget, &mut order, door_closed);
    quicksort_viscel(&mut order);
    Some(order.into_iter().map(|(c, _)| c).collect())
}

/// `QuickSort` (0x1d30a8) with comparator id 3 — jumptable 0x304fc0[3] → 0x1d3430, disassembled:
/// `lhu $a0, 4($a0); lhu $v1, 4($a1)` — the ascending **u16 portal depth** at element+4. Unstable:
/// Hoare partition, pivot = element `count>>1` swapped to the front, the smaller side pushed to the
/// stack (the decomp's byte-size test `(j − (lo+1)) < (hi − i)` at stride 8 is index-space
/// `(j − lo) <= (hi − i)`). The tie order feeds `View_AddCels`' −90000 + 10·k ordinals, so the port
/// sorts exactly like the game over the same insertion order.
pub fn quicksort_viscel(items: &mut [(usize, u16)]) {
    let n = items.len();
    if n < 2 { return; }
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let (mut lo, mut hi) = (0usize, n - 1);
    loop {
        // Pivot: element count>>1 swapped to the front (0x1d31a8's `Swap(((hi−lo)/stride+1)>>1·stride+base, base)`).
        let count = hi - lo + 1;
        items.swap(lo, lo + count / 2);
        let mut i = lo + 1;
        let mut j = hi + 1;
        loop {
            // `while (uVar3 <= uVar5 && Comp(uVar3, param_1) < 1)` — Comp <= 0 stays left.
            while i <= hi && items[i].1 <= items[lo].1 { i += 1; }
            j -= 1;
            // `while (param_1 < uVar2 && Comp(uVar2, param_1) > -1)` — Comp >= 0 stays right.
            while lo < j && items[j].1 >= items[lo].1 { j -= 1; }
            if j < i { break; }
            items.swap(i, j);
            i += 1;
        }
        items.swap(lo, j);
        if j - lo <= hi.saturating_sub(i) {
            // Left side smaller (or equal): push the right, loop on the left [LAB_001d32a8].
            if i < hi { stack.push((i, hi)); }
            if j > lo + 1 {
                hi = j - 1;
            } else {
                // Left is a single element: pop (LAB_001d32d0).
                match stack.pop() { Some((l, h)) => { lo = l; hi = h; } None => return }
            }
        } else {
            // Right side smaller: push the left, loop on the right.
            if lo + 1 < j { stack.push((lo, j - 1)); }
            lo = i;
            if lo >= hi {
                // The do-while's `i < hi` failed: pop.
                match stack.pop() { Some((l, h)) => { lo = l; hi = h; } None => return }
            }
        }
    }
}

/// One `ClipPortalWithPlane` (0x1e7fa0) pass: Sutherland–Hodgman keeping `value >= 0`, with the game's
/// compaction (a vertex is kept only when its successor is farther than √1e-7). `None` = every vertex
/// inside (the game's return 0 — the polygon passes through unchanged), `Some(v)` = the clipped
/// polygon (possibly empty = fully outside).
fn clip_positive(poly: &[[f32; 3]], plane: [f32; 4]) -> Option<Vec<[f32; 3]>> {
    let val = |p: &[f32; 3]| plane[0] * p[0] + plane[1] * p[1] + plane[2] * p[2] + plane[3];
    if !poly.iter().any(|p| val(p) < 0.0) { return None; }
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let (cur, nxt) = (poly[i], poly[(i + 1) % n]);
        let (cin, nin) = (val(&cur) >= 0.0, val(&nxt) >= 0.0);
        let denom = plane[0] * (cur[0] - nxt[0]) + plane[1] * (cur[1] - nxt[1]) + plane[2] * (cur[2] - nxt[2]);
        match (cin, nin) {
            (true, true) => out.push(nxt),
            (true, false) => {
                if denom != 0.0 {
                    let t = -val(&nxt) / denom;
                    out.push([nxt[0] + t * (cur[0] - nxt[0]), nxt[1] + t * (cur[1] - nxt[1]), nxt[2] + t * (cur[2] - nxt[2])]);
                }
            }
            (false, true) => {
                if denom != 0.0 {
                    let t = -val(&nxt) / denom;
                    out.push([nxt[0] + t * (cur[0] - nxt[0]), nxt[1] + t * (cur[1] - nxt[1]), nxt[2] + t * (cur[2] - nxt[2])]);
                }
                out.push(nxt);
            }
            (false, false) => {}
        }
    }
    // The game's compaction: keep vertex i only when its (wrapped) successor differs by more than
    // √1e-7 (Vec_SqDist3D over the out buffer, 0x1e82e0's copy-back loop).
    let n = out.len();
    let mut comp = Vec::with_capacity(n);
    for i in 0..n {
        let nx = if i + 1 != n { i + 1 } else { 0 };
        let d = [out[i][0] - out[nx][0], out[i][1] - out[nx][1], out[i][2] - out[nx][2]];
        if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 1e-7 { comp.push(out[i]); }
    }
    Some(comp)
}
