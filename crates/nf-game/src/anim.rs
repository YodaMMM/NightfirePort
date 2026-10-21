//! The animation engine's pose side (docs/systems/anim-engine.md): a script's pose at a fractional time
//! (`AnimProcessScriptCmds` op 0 → `AnimSeqSet` → `AnimFrameSet`) and the blend of several playing scripts
//! (`AnimFrameResolve` → `AnimFrameBlend`).

use nf_formats::anim::{local_pose, Script, Sequence};
use nf_formats::character::{Skeleton, Skin};
use std::collections::HashMap;

/// Per bone: local translation and rotation quaternion (x, y, z, w).
pub type Pose = Vec<([f32; 3], [f32; 4])>;

/// `Quat_Slerp_Acc` (0x1e4188), verbatim [M work/decomp/ACTION.ELF/001e4188]: dot = q0·q1; `neg` = dot < 0 and |dot|
/// used; the slerp branch runs while `0.01 < 1 − dot` — ω = acosf(dot), ωt = ω·t, w1 = PS2Sinf(ωt)·(1/PS2Sinf(ω)),
/// w0 = PS2Sinf(ω−ωt)·(1/PS2Sinf(ω)) (a reciprocal multiply, one division); else linear w0 = 1−t, w1 = t. `neg`
/// negates **w0** (the q0 coefficient — NOT q1), and there is **no renormalisation**: out = w0·q0 + w1·q1 as is. A
/// rounded dot past 1.0 gives 1 − dot ≤ 0 < 0.01 and lands in the linear branch, so no clamp is needed.
pub fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let neg = d < 0.0;
    if neg { d = -d; }
    let (s0, s1) = if 0.01 < 1.0 - d {
        // G-046: the game's slerp weights are Spline_Interp 0x1e1858:285-300 [M]: acosf (0x232c48 →
        // __ieee754_acosf 0x233228), wt = angle·t (mul.s), and the weights are PS2Sinf(wt)·(1/PS2Sinf(angle))
        // — a div.s reciprocal then mul.s, not a per-weight division.
        let w = crate::trig::acosf_libm(d);
        let wt = nf_formats::ps2float::mul(w, t);
        let inv = nf_formats::ps2float::div(1.0, crate::trig::sinf(w));
        (crate::trig::sinf(crate::trig::ee_sub(w, wt)) * inv, crate::trig::sinf(wt) * inv)
    } else {
        (1.0 - t, t)
    };
    let s0 = if neg { -s0 } else { s0 };
    [0, 1, 2, 3].map(|k| a[k] * s0 + b[k] * s1)
}

/// The root translation at a fractional frame — the game's root motion never samples the quintics between frames:
/// `AnimSeqTick` (0x1c28d0) takes the blended pose buffer's root, which `AnimFrameSet` (0x1c75a0) filled as frames
/// ⌊f⌋ and ⌊f⌋+1 blended by `AnimFrameBlend` (0x1c2b80). So: single frame while |frac| ≤ 0.0002 or at the last
/// frame, else the two integer frames' translations lerped by the fraction, clamped into the next frame's direction
/// (AnimFrameBlend's translation rule).
pub fn root_translation_at(seq: &Sequence, frame: f32) -> [f32; 3] {
    let last = seq.frames.max(1) as u32;
    let f0 = (frame.floor().max(1.0) as u32).clamp(1, last);
    let frac = frame - frame.floor();
    let at = |f: u32| [seq.value(0, f), seq.value(1, f), seq.value(2, f)];
    let a = at(f0);
    if frac.abs() <= 0.0002 || f0 >= last { return a; }
    let b = at(f0 + 1);
    let mut r = a;
    for k in 0..3 {
        let d = b[k] - a[k];
        let v = a[k] + d * frac;
        r[k] = if (d >= 0.0 && v > b[k]) || (d < 0.0 && v < b[k]) { b[k] } else { v };
    }
    r
}

/// `AnimDistanceTableCreate(id, axisMask, looping)` (0x1c4060): the cumulative |root Δ| per frame a speed-mode-1
/// script plays by. Table = `[0]` total, `[1]` segment count N (frames, or frames−1 without `looping`), `[2]` 0.0,
/// then cum[k] for k = 1..=N−1 at index k+2 — segment k spans `[cum[k−1], cum[k]]` (`cum[0]` = the reserved 0.0).
/// Each segment's delta = the root translation at frame k+1 minus frame k (via `AnimFrameCopy`, i.e. the integer
/// quintic samples), components masked by `axis_mask` bits 0/1/2 = x/y/z; a segment under 1e-05 becomes 0.01
/// [M 001c4060: `fVar8 = 1e-05`, `if fVar7 < fVar8 → 0.01`]. The walk sets build z-only tables (axis mask 4,
/// `AnimDistanceTableGet` 0x1c76f0) with `looping` set [M AnimSeqSet 0x1c7460 attaches the table to seq+0x80].
/// (`AnimDistanceTableInit` 0x1c76e8 — called only from `AnimPreLoadInit` 0x1c63b8 — clears the game's global
/// `nDistanceTables` counter on level load; the port builds tables lazily per sequence below, so there is no
/// registry to reset — the fresh-per-load model covers it.)
pub struct DistanceTable {
    pub total: f32,
    pub segments: u32,
    /// cum[k] = distance walked by the end of segment k (index k−1 in the game's float array + 3).
    pub cum: Vec<f32>,
}

impl DistanceTable {
    // unported-misc-5 — the game's anim skin data goes through the cel hashtable:
    // `hashtable_modify` 0x1ca648 (`AnimProcessSkinData` 0x1bd500), `hashtable_celglist_to_hashcode`
    // 0x1ca840 (`AnimObjectNew` 0x1be910), `hashtable_get_hashtype_count` 0x1ca938 /
    // `hashtable_get_hashtype_data` 0x1ca9a0 (`AnimPostLoadInit` 0x1be1a8, `AnimDebugCreate`
    // 0x1bcfa8, `CreateSpaceAndPackTextures` 0x11dcf8). The port keys its anim/weapon meshes by
    // hash directly (HashMap/BTreeMap over the same hashes), so the sectioned hashtable is a
    // Rust map here — see scene.rs's note for the render side of the same table.
    pub fn build(seq: &Sequence, axis_mask: u32, looping: bool) -> Option<DistanceTable> {
        let frames = seq.frames.max(1) as i32;
        let n = if looping { frames } else { frames - 1 };
        if n < 2 { return None; }
        let at = |f: i32| {
            let f = f.clamp(1, frames) as u32;
            [seq.value(0, f), seq.value(1, f), seq.value(2, f)]
        };
        let mut cum = Vec::with_capacity((n - 1) as usize);
        let mut acc = 0.0;
        for k in 1..n {
            let (a, b) = (at(k), at(k + 1));
            let mut d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            if axis_mask & 1 == 0 { d[0] = 0.0; }
            if axis_mask & 2 == 0 { d[1] = 0.0; }
            if axis_mask & 4 == 0 { d[2] = 0.0; }
            let m = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            acc += if m < 1e-05 { 0.01 } else { m };
            cum.push(acc);
        }
        Some(DistanceTable { total: acc, segments: (n - 1) as u32, cum })
    }

    /// `AnimDistanceTableDistanceToFrame2` (0x1c4270): the speed a mode-1 script plays at this tick. `walked` is the
    /// walked accumulator (`animObj+0x6C` = obj+0xDC) plus the stride (`animObj+0x64` = obj+0xD4); it is wrapped into
    /// `[0, total)` and **stored back** (the decomp writes `param_2 + 0x6c`); the frame is the segment plus the
    /// distance into it; the returned speed is that frame minus the script time, wrapped in the stride's direction.
    pub fn distance_to_frame(&self, walked: &mut f32, time: f32, forward: bool) -> f32 {
        if self.segments == 0 || self.total <= 0.0 { return 0.0; }
        let mut w = *walked;
        while self.total <= w { w -= self.total; }
        while w < 0.0 { w += self.total; }
        // Segment search: the estimate then the walk (0x1c4270: i = (int)(w·N/total), down while w < cum[i−1], up
        // while cum[i] ≤ w).
        let mut i = ((w * self.segments as f32) / self.total) as i32;
        let cum_at = |k: i32| -> f32 { if k <= 0 { 0.0 } else { self.cum[(k as usize).min(self.cum.len()) - 1] } };
        while i > 0 && w < cum_at(i) { i -= 1; }
        while (i as u32) < self.segments && cum_at(i + 1) <= w { i += 1; }
        let seg_end = cum_at(i + 1);
        let seg_start = cum_at(i);
        let seg_len = seg_end - seg_start;
        let frame = (i + 1) as f32 + if seg_len > 0.0 { (w - seg_start) / seg_len } else { 0.0 };
        *walked = w;
        let speed = frame - time;
        if forward {
            if speed < 0.0 { self.total - time + frame } else { speed }
        } else if speed >= 0.0 {
            -(self.total - frame + time)
        } else {
            speed
        }
    }
}

/// `AnimScriptTick`'s speed for a script (`+0xB4`), which the tick then adds to the script time [M 001c2630]:
/// mode 0 — `speed × FRAME_RATE_MUL` (1.0 at the port's fixed 60 Hz); mode 1 — the distance table's frame minus the
/// time (flags 0x80000000); mode 2 — synced to the master script `+0x4C`: Δ = (M.time−1)/(M.len−1) − (t−1)/(len−1),
/// wrapped in the master speed's direction, `speed = Δ·(len−1) + 1.0`; mode 3 — `(len+1)·phase(+0x8C) − time` (no
/// writer found in ACTION.ELF); any other byte keeps the speed.
pub fn speed_for_mode(mode: u8, speed: f32, time: f32, length: f32, phase: f32, master: Option<(f32, f32, f32)>) -> f32 {
    match mode {
        0 => speed,
        1 => speed, // replaced by the distance table's `distance_to_frame` at the call site (needs the walked state)
        2 => match master {
            Some((m_time, m_len, m_speed)) if length > 1.0 && m_len > 1.0 => {
                let mn = (m_time - 1.0) / (m_len - 1.0);
                let tn = (time - 1.0) / (length - 1.0);
                let d = mn - tn;
                let d = if m_speed >= 0.0 {
                    if d < 0.0 { (1.0 - tn) + mn } else { d }
                } else if d >= 1.0 {
                    (tn + 1.0) - mn
                } else {
                    d
                };
                d * (length - 1.0) + 1.0
            }
            _ => speed,
        },
        3 => (length + 1.0) * phase - time,
        _ => speed,
    }
}

/// `AnimFrameBlend(t, out, a, b)`: per bone, translation a + (b − a)·t without overshooting b, rotation slerp; t = 0
/// keeps a, t = 1 copies b.
pub fn blend(a: &mut Pose, b: &Pose, t: f32) {
    if t == 0.0 { return; }
    if t == 1.0 { a.clone_from(b); return; }
    for (x, y) in a.iter_mut().zip(b) {
        for k in 0..3 {
            let d = y.0[k] - x.0[k];
            let v = x.0[k] + d * t;
            x.0[k] = if (d >= 0.0 && v > y.0[k]) || (d < 0.0 && v < y.0[k]) { y.0[k] } else { v };
        }
        x.1 = slerp(x.1, y.1, t);
    }
}

/// The sequence and fractional frame a script shows at script time `time` (from 1): the opcode-0 command covering
/// ⌊time⌋, frame = time − first + 1 (`AnimSeqSet` / `AnimSeqTick`); the last command past the end. When several
/// opcode-0 windows cover the time (e.g. the 07000005 cut-scene scripts pair a 22-channel face track with the
/// 222-channel body track), `AnimProcessScriptCmds` (0x1c1b38) starts **each** as a playing anim in command order
/// and the palette lets the later-started one own the shared bones — so the **last** covering command wins.
pub fn script_frame(script: &Script, time: f32) -> Option<(u32, f32)> {
    let t = time.floor().max(1.0) as u16;
    let (a, _, q) = script.sequences().filter(|(a, b, _)| *a <= t && t <= *b).last().or_else(|| script.sequences().last())?;
    Some((q, (time - a as f32 + 1.0).max(1.0)))
}

/// `AnimFrameSet`: the pose at fractional frame f — frames ⌊f⌋ and ⌊f⌋ + 1 (clamped to the last) blended by the
/// fraction.
pub fn sequence_pose(seq: &Sequence, skel: &Skeleton, skin: &Skin, frame: f32) -> Pose {
    let last = seq.frames.max(1) as u32;
    let f0 = (frame.floor() as u32).clamp(1, last);
    let frac = (frame - frame.floor()).clamp(0.0, 1.0);
    let mut p = local_pose(seq, skel, skin, f0);
    if frac > 0.0002 && f0 < last {
        let q = local_pose(seq, skel, skin, f0 + 1);
        blend(&mut p, &q, frac);
    }
    p
}

/// A script's pose at `time` (`None` when the script or its sequence is not loaded, or the sequence
/// cannot pose this skeleton).
pub fn script_pose(scripts: &HashMap<u32, Script>, seqs: &HashMap<u32, Sequence>, skel: &Skeleton, skin: &Skin, hash: u32, time: f32) -> Option<Pose> {
    let (q, f) = script_frame(scripts.get(&hash)?, time)?;
    // The skeleton-id match is the historical strictness; the pose machinery itself
    // (`local_pose_with`) reads the channels by the SKEL's translation mask, never by
    // `seq.skeleton`. Shared human tracks ride on other rigs' ids: 07000004's seated target
    // (skin 0x0500001e, skeleton 8) plays 0x0400028a, whose record byte is skeleton 0 — the
    // strict filter left him with no palette and no hit boxes (the bullets could never land).
    // [The game's playback confirms the cross-rig read: `AnimFrameCopy` 0x1c5668 takes the walk's
    // skeleton from the SEQUENCE tag's +0xa9 byte (:52) and gates the writes on the SKIN's bone
    // flags (param_4+0x24) — ninja-boss-cutscene. The port's skin-skeleton walk is the standing
    // [S] convention; the channel-count guard keeps short tracks (a 22-channel face track on a
    // body need) rejected.]
    let seq = seqs.get(&q).filter(|s| {
        s.skeleton == skin.skeleton
            || {
                let need: usize = (0..skel.bones as usize).map(|b| if skel.has_translation(b) { 6 } else { 3 }).sum();
                s.channels.len() >= need
            }
    })?;
    Some(sequence_pose(seq, skel, skin, f))
}

/// The drone path's pose: every started opcode-0 window plays together (`AnimProcessScriptCmds`
/// 0x1c1b38 starts each window as its own playing anim) and per bone the LAST-started window that
/// covers it owns it — [L] the 07000006 truck driver (start script 0600011d, body 04000064 then the
/// 22-channel face track 04000065, both covering 1..880) holds his position to 0.01 over the whole
/// 700-frame trace window while the body track's root runs 0.57: the face track's all-zero root owns
/// his root bone, so the body's root motion never reaches the object. A single-window script (the
/// 07000001 guard's 06000169) keeps its root motion — [L] the same trace file's 01-doors segment
/// shows the guard moving 1.1 units inside his start script. Bones no window covers fall to the
/// widest one (the game's frame copy leaves them at the anim object's previous values). A shared
/// human track riding another rig's record byte (0x0400028a, record 0, on skeleton-8 skin —
/// drone-port-check #3) counts as full-coverage when its channels cover the skeleton. Never panics:
/// each window poses only the bones its channels reach (`local_pose_bones`).
pub fn script_pose_drone(scripts: &HashMap<u32, Script>, seqs: &HashMap<u32, Sequence>, skel: &Skeleton, skin: &Skin, hash: u32, time: f32) -> Option<Pose> {
    script_pose_drone_with(scripts, seqs, skel, skin, hash, time, &mut |seq, f, n| nf_formats::anim::local_pose_bones(seq, skel, skin, f, n))
}

/// mp-perf-16: a memo of whole-frame bone samples (`local_pose_bones`) keyed by (sequence,
/// skin, frame, bone count) — the render's MP bodies re-sample the same integer frames tick
/// after tick (a fractional frame blends two of them, the walk advances ~half a frame a tick),
/// and the PS2-rounded channel sums are the bulk of a body's palette cost. A sample is a pure
/// function of its key (`local_pose_bones` reads only the sequence, skeleton and skin), so a
/// cached pose is the identical pose. Render-only: the sim's drone poses keep the direct path.
#[derive(Default)]
pub struct PoseSampleCache {
    map: HashMap<(u32, u16, u32, u32, [u32; 3], u32, usize), Pose>,
}

impl PoseSampleCache {
    /// The cap: past it the memo starts over (a 16-body arena keeps a few hundred live entries).
    const CAP: usize = 4096;
    pub fn sample(&mut self, seq: &Sequence, skel: &Skeleton, skin: &Skin, frame: u32, bones: usize) -> Pose {
        if self.map.len() >= Self::CAP { self.map.clear(); }
        let key = (seq.hashcode, seq.frames, skin.hashcode, skel.id as u32, skin.scale.map(f32::to_bits), frame, bones);
        self.map.entry(key)
            .or_insert_with(|| nf_formats::anim::local_pose_bones(seq, skel, skin, frame, bones))
            .clone()
    }
}

/// `script_pose_drone` through the memo (see [`PoseSampleCache`]) — the same pose.
pub fn script_pose_drone_cached(scripts: &HashMap<u32, Script>, seqs: &HashMap<u32, Sequence>, skel: &Skeleton, skin: &Skin, hash: u32, time: f32, cache: &mut PoseSampleCache) -> Option<Pose> {
    script_pose_drone_with(scripts, seqs, skel, skin, hash, time, &mut |seq, f, n| cache.sample(seq, skel, skin, f, n))
}

fn script_pose_drone_with(scripts: &HashMap<u32, Script>, seqs: &HashMap<u32, Sequence>, skel: &Skeleton, skin: &Skin, hash: u32, time: f32, sample: &mut dyn FnMut(&Sequence, u32, usize) -> Pose) -> Option<Pose> {
    let script = scripts.get(&hash)?;
    let t = time.floor().max(1.0) as u16;
    let wins: Vec<(u16, u32)> = script.sequences().filter(|(a, _, _)| *a <= t).map(|(a, _, q)| (a, q)).collect();
    // fix-facial3 / fix-cutscene-faces: a face track is NOT a bone pose — `AnimFacialBlend`
    // 0x1be648 samples those channels as the morph weights [M]. The classifier is the game's own:
    // the SEQUENCE's flags byte bit 0x8 [M `AnimProcessScriptCmds` 0x1c1b38:
    // `*(int *)(lVar6 + 0xc) >> 3 & 1`], not the channel count (the counts match by construction:
    // [D measured] 07000007's face seq 0x04000467, flags 0x08, 22 channels == skin 0x0500003b's
    // facial count; measured on the disc: 07000006's driver face track 04000065 channels 0..2
    // reach ~1.0 — weight values the old bone read turned into root/face-bone translations, the
    // weights-as-bones bug). One rule everywhere (the count probe was fix-facial3's stand-in).
    let is_face = |q: u32| -> bool { seqs.get(&q).is_some_and(|s| s.flags & 0x8 != 0) };
    // The bones a window's channels reach; the shared-track arm mirrors `script_pose`'s filter.
    let covered = |q: u32| -> usize {
        if is_face(q) { return 0; }
        let Some(s) = seqs.get(&q) else { return 0 };
        if s.skeleton == skel.id as u8 {
            nf_formats::anim::bones_covered(s, skel)
        } else {
            let need: usize = (0..skel.bones as usize).map(|b| if skel.has_translation(b) { 6 } else { 3 }).sum();
            if s.channels.len() >= need { skel.bones as usize } else { 0 }
        }
    };
    // The base: the widest started window — it also stands in for the bones no window reaches.
    let (base_i, base_n) = wins.iter().enumerate()
        .filter_map(|(i, (_, q))| { let n = covered(*q); (n > 0).then_some((i, n)) })
        .max_by_key(|&(_, n)| n)?;
    let (base_first, base_q) = wins[base_i];
    let base_f = (time - base_first as f32 + 1.0).max(1.0);
    let base_seq = seqs.get(&base_q)?;
    let mut acc = sequence_pose_n_with(base_seq, base_f, base_n, sample);
    // mp-perf-16: the base window's own pass below re-sampled the very same (sequence, frame,
    // bone count) — the identical values; keep them instead of decoding the sequence twice (the
    // PS2-rounded channel sums are the bulk of an MP body's palette cost).
    let base_vals: Vec<_> = acc.iter().take(base_n).copied().collect();
    if acc.len() < skel.bones as usize {
        while acc.len() < skel.bones as usize { acc.push(*acc.last().unwrap()); }
    }
    // Then every started window in command order overwrites the bones it reaches — the last one
    // wins; the base's own pass (first pass over its bones) is its place in that order.
    for (wi, (first, q)) in wins.iter().enumerate() {
        if wi == base_i {
            for (i, v) in base_vals.iter().enumerate() { acc[i] = *v; }
            continue;
        }
        let n = covered(*q);
        if n == 0 { continue; }
        let Some(seq) = seqs.get(q) else { continue };
        let f = (time - *first as f32 + 1.0).max(1.0);
        let p = sequence_pose_n_with(seq, f, n, sample);
        for (i, v) in p.iter().enumerate().take(n) { acc[i] = *v; }
    }
    Some(acc)
}

/// A script's pose with EVERY started opcode-0 window playing together. `AnimProcessScriptCmds`
/// (0x1c1b38) starts each window whose time the script enters as its own playing anim — the pack
/// cut-scene scripts pair a narrow 22-channel face track with the full 222-channel body track
/// (060004f1/060004f2 of the 07000007 Kimiko scene: windows [1..0x10e6] seq …467/…468 then
/// [1..0x10e7] seq …332/…3f9; the 07000005 ledge's 06000996/97: [1..0x5dc] 040007fa then
/// [1..0x579] 040007f8) [D `nf-extract script-info` dumps]. Per bone the FIRST-started window that
/// covers it owns it and the later windows fill only the bones the earlier ones do not reach [S:
/// the direction follows the play reference — the game's mouths move while the body animates, which
/// the old single-window read (last wins, fix-drone2) cannot do; both windows start at frame 1 here,
/// so command order breaks the tie]. A window past its end holds its last frame (`AnimScriptTick`
/// 0x1c2630); `sequence_pose`'s clamp provides that. The AI/drone path keeps `script_pose`
/// (last-wins, fix-drone2's pinned behaviour) — this is the cut-scene character path only.
pub fn script_pose_together(scripts: &HashMap<u32, Script>, seqs: &HashMap<u32, Sequence>, skel: &Skeleton, skin: &Skin, hash: u32, time: f32) -> Option<Pose> {
    let script = scripts.get(&hash)?;
    let t = time.floor().max(1.0) as u16;
    let wins: Vec<(u16, u32)> = script.sequences().filter(|(a, _, _)| *a <= t).map(|(a, _, q)| (a, q)).collect();
    // The game's facial window: the SEQUENCE's flags byte bit 0x8 [M `AnimProcessScriptCmds`
    // 0x1c1b38: `uVar5 = *(int *)(lVar6 + 0xc) >> 3 & 1` on the raw sequence record — a facial
    // window sets the script node's 0x40000 flag and plays in its +0x54 facial slot,
    // docs/systems/anim-engine.md "+0x50 / +0x54 (body; facial)"]. A facial seq's channels are
    // the skin's facial MORPH weights, never a bone pose: `AnimFacialBlend` 0x1be648 reads the
    // playing sequence's frame buffer as the weight array, and the counts match by construction —
    // [D measured on this scene] skin 0x0500003b's facial count 22 == face seq 0x04000467's 22
    // channels (flags 0x08) whose values stay in [0, 1], while the body seq 0x04000332 (flags
    // 0x00) carries 222 = 3·73 + 3·1 channels (the skeleton's per-bone walk) whose root channels
    // hold the actor's world position (83..91, 43.7, −35..−31). Read as bones, the face track's
    // leading [0,1] weights zero the root — the missing-cast bug. The window picker here must
    // never hand a facial window to the bone walk; the weights path (`Scene::facial_weights_now`)
    // is what consumes it. This flags rule (fix-cutscene-faces) is THE classifier everywhere —
    // `script_pose_drone` uses it too (fix-facial3's channel-count probe was the stand-in;
    // one rule, the game's own).
    let is_facial = |q: u32| seqs.get(&q).is_some_and(|s| s.flags & 0x8 != 0);
    // ninja-boss-cutscene: the window-acceptance rule, one with `script_pose`/`script_pose_drone`'s
    // channel-count guard. `AnimFrameCopy` 0x1c5668 reads the channel stream positionally over the
    // SEQUENCE's own record skeleton (`iVar6 = pSkeletons[*(byte *)(sAnimSeq + 0xa9)]`, :52; the
    // translation-mask test :190, the stand-in offsets :194-198) and gates each bone's write on the
    // SKIN's bone-flag bytes (`param_4 + 0x24`, :186-189) — no id equality anywhere. Shipped
    // cut-scenes ride cross-rig: 07000004's ninja (skin 0x0500001e, skeleton byte 8) plays 0x06000507
    // whose only window is 0x0400047a (record byte 0; both skeletons 73 bones, mask 0x1/0/0) [D
    // 07000004.bin kind-7/5, 07F00004.bin kind-4] — the strict `s.skeleton == skin.skeleton` filter
    // found no base window and the cut-scene's ninja actor never posed (the user's "ninja boss is
    // missing from last cutscene", playtest 1017). The gameplay path has posed the same pairing since
    // its own fix (see `script_pose`'s comment). A guarded window covers the skin skeleton's FULL walk
    // or nothing (a short track — a face track past the flags rule — still never poses bones).
    let full_need: usize = (0..skel.bones as usize).map(|b| if skel.has_translation(b) { 6 } else { 3 }).sum();
    // The base: the widest started non-facial window that can pose the skin, at its own frame — it
    // also stands in for the bones no window reaches. A matching window keeps the old `fits` gate
    // (a lone narrow track hides — kept); a guarded window needs the full walk.
    let (base_i, base_n) = wins.iter().enumerate()
        .filter_map(|(i, (_, q))| {
            let Some(s) = seqs.get(q) else { return None };
            if is_facial(*q) { return None; }
            let n = if s.skeleton == skin.skeleton {
                (nf_formats::anim::fits(s, skel)).then(|| nf_formats::anim::bones_covered(s, skel)).unwrap_or(0)
            } else {
                (s.channels.len() >= full_need).then_some(skel.bones as usize).unwrap_or(0)
            };
            (n > 0).then_some((i, n))
        })
        .max_by_key(|&(_, n)| n)?;
    if base_n == 0 { return None; }
    let (base_first, base_q) = wins[base_i];
    let base_f = (time - base_first as f32 + 1.0).max(1.0);
    let Some(base_seq) = seqs.get(&base_q) else { return None };
    let mut acc = sequence_pose_n(base_seq, skel, skin, base_f, base_n);
    if acc.len() < skel.bones as usize {
        // Pad to the full bone count with the base's last bone pose (never read: the body covers all).
        while acc.len() < skel.bones as usize { acc.push(*acc.last().unwrap()); }
    }
    // fix-cutscene-draw: the base (widest, `fits`) track OWNS the bones it covers. The game maps
    // each window's channels to bones through the skin's channel table (`AnimFrameCopy` is passed
    // the sAnimSkin [M AnimFrameResolve 0x1c35f8 → AnimFrameCopy; psiBuildMatrixPalette 0x111498
    // reads sAnimObject+0x24 = the skin (decompile line 110, `*(int *)(iVar7 + 0x24)` → the
    // pSkeletons/parents/matrices reads); the +0x94 skin reads sit on other bases —
    // (obj+0xdc)+0x94 in AnimListBuild 0x1c0fc0:114, obj+0x94 in AnimObjectDraw 0x1c4490:117]),
    // so a pack cut-scene's narrow 22-channel face track
    // reaches only the face bones and never the body's root — the 222-channel body track (whose
    // root channel carries the actor's absolute world position, `AnimGetFirstTrans` 0x1c7348
    // initialises obj+0x30 from its frame-0 root) covers every bone and wins them. The port's
    // sequential channel walk cannot express that sparse map: letting the face window fill first
    // ("first-writer-wins", fix-cutscenes2) handed bones 0..5 of the body track to the face
    // track's leading channels — the root collapsed to identity-at-the-origin and every script
    // actor drew as a mannequin at the world origin, off-camera (the missing cut-scene cast).
    // Narrow windows now fill only the bones the base does not reach (none when a full body
    // track runs; the ledge's narrow pairs without a full track keep the first-writer fill).
    let mut owned = vec![true; base_n];
    for (first, q) in &wins {
        if *q == base_q { continue; }
        let Some(seq) = seqs.get(q) else { continue };
        // A facial window is the morph-weight source (above) — it owns no bones. The classifier is
        // the flags bit 0x8, the game's own, everywhere (fix-cutscene-faces; see the base picker).
        if is_facial(*q) { continue; }
        // Matching windows keep the old narrow fill (`bones_covered` leading bones — the ledge's
        // narrow pairs); guarded windows fill all-or-nothing, the same rule as the base picker.
        let n = if seq.skeleton == skin.skeleton {
            nf_formats::anim::bones_covered(seq, skel)
        } else {
            (seq.channels.len() >= full_need).then_some(skel.bones as usize).unwrap_or(0)
        }
        .min(base_n);
        if n == 0 { continue; }
        let f = (time - *first as f32 + 1.0).max(1.0);
        let p = sequence_pose_n(seq, skel, skin, f, n);
        for (i, v) in p.iter().enumerate().take(n) {
            if !owned[i] { acc[i] = *v; owned[i] = true; }
        }
    }
    Some(acc)
}

/// `sequence_pose` for the first `bones` bones only (a narrow sequence — the face tracks — whose
/// channel list `local_pose` would read past).
pub fn sequence_pose_n(seq: &Sequence, skel: &Skeleton, skin: &Skin, frame: f32, bones: usize) -> Pose {
    sequence_pose_n_with(seq, frame, bones, &mut |seq, f, n| nf_formats::anim::local_pose_bones(seq, skel, skin, f, n))
}

/// `sequence_pose_n` with the whole-frame sampler supplied (`local_pose_bones` or its memo).
fn sequence_pose_n_with(seq: &Sequence, frame: f32, bones: usize, sample: &mut dyn FnMut(&Sequence, u32, usize) -> Pose) -> Pose {
    let last = seq.frames.max(1) as u32;
    let f0 = (frame.floor() as u32).clamp(1, last);
    let frac = (frame - frame.floor()).clamp(0.0, 1.0);
    let mut p = sample(seq, f0, bones);
    if frac > 0.0002 && f0 < last {
        let q = sample(seq, f0 + 1, bones);
        blend(&mut p, &q, frac);
    }
    p
}

/// A script that is fading out (`AnimScriptAppendBlend`: fade −1, flags 0x500000): it keeps playing while its blend
/// clock runs down; weight = clock / length. `AnimScriptCallOnStop` (0x1c7200) stores the game's +0x6c stop
/// callback on that node — `DroneAnim_SetHTAnim` 0x13b1c8 arms it to `DroneAnim_AnimEndCallback` 0x13e3e8 for
/// stop-at-end scripts, which only clears the drone's +0x579 one-shot-pending flag when the node dies; the port
/// has no stop-callback slot, covering that clear with its per-Animator `once`/`held`/`early_end` reset on
/// script replacement. The old script's OnStop is cleared (0) by the same call [M 0x13b1c8:84-88].
#[derive(Clone, Debug)]
pub struct Outgoing {
    pub script: u32,
    /// Script time (from 1) and speed (frames per tick); a one-shot stops at its end, a looping script wraps.
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
    pub clock: f32,
    pub length: f32,
    /// Root yaw moved into the object (anim rotation, `+0x78 & 0x20000`): removed from this script's root.
    pub unyaw: f32,
}

impl Outgoing {
    pub fn weight(&self) -> f32 { if self.length > 0.0 { (self.clock / self.length).clamp(0.0, 1.0) } else { 0.0 } }

    /// One tick (`AnimScriptTick` + `AnimScriptEnd` + the fade in `AnimFrameResolve`); false once faded out.
    pub fn tick(&mut self, len: f32) -> bool {
        if len >= 1.0 {
            let t = self.time + self.speed;
            self.time = if t <= len { t } else if self.looping { t - len } else { len };
        }
        self.clock -= 1.0;
        self.clock > 0.0
    }
}

/// The yaw (about +Y) of a root rotation: its forward (+Z) turned, `atan2(x, z)` (`Mat_GetDir` + `Vec_DirToRot`).
pub fn root_yaw(q: [f32; 4]) -> f32 {
    let [x, y, z, w] = q;
    let f = [2.0 * (x * z + w * y), 1.0 - 2.0 * (x * x + y * y)];
    // G-046: Vec_DirToRot 0x1e3218:8 is the PS2 libm atan2f [M], not host atan2.
    crate::trig::atan2f_libm(f[0], f[1])
}

/// Removes `yaw` about +Y from a root rotation (pre-multiplies by the inverse turn).
pub fn unyaw(q: [f32; 4], yaw: f32) -> [f32; 4] {
    // G-046: the inverse-turn quat is the game's AA2Q idiom — w = PS2Sinf(θ/2 + 1.5707964), y =
    // PS2Sinf(θ/2) [M Quat_AxisAngleToQuat 0x1e3f20's PS2Sinf2 lanes; the θ = −yaw mapping [S] — the
    // game counterpart of this helper is otherwise unresolved (G-046's UNCLEAR note)].
    let ha = -yaw * 0.5;
    let s = crate::trig::sinf(ha);
    let c = crate::trig::sinf(crate::trig::ee_add(ha, crate::trig::HALF_PI));
    let r = [0.0, s, 0.0, c];
    // r ⊗ q
    [r[3] * q[0] + r[0] * q[3] + r[1] * q[2] - r[2] * q[1],
     r[3] * q[1] - r[0] * q[2] + r[1] * q[3] + r[2] * q[0],
     r[3] * q[2] + r[0] * q[1] - r[1] * q[0] + r[2] * q[3],
     r[3] * q[3] - r[0] * q[0] - r[1] * q[1] - r[2] * q[2]]
}

/// `AnimFrameResolve`'s order: the outgoing scripts (oldest first) then the current one; result = first, then for each
/// next: blend(result, next, 1 − weight(previous)). The current script has full weight.
pub fn blend_playing(current: Pose, outgoing: &[(Pose, f32)]) -> Pose {
    let Some(((first, _), rest)) = outgoing.split_first() else { return current };
    let mut out = first.clone();
    let mut prev_w = outgoing[0].1;
    for (p, w) in rest {
        blend(&mut out, p, 1.0 - prev_w);
        prev_w = *w;
    }
    blend(&mut out, &current, 1.0 - prev_w);
    out
}

/// The anim-object flag word at `animobj+0xc8` (the object the port folds into state lists — see
/// `drone.rs` `aims_torso` for the aim side and the `tracking` fold for the head side).
/// `AnimObjectIsAiming` 0x1c6900 tests `& 0x180`, `AnimObjectIsHeadTracking` 0x1c6850 tests `& 0x30` [M
/// work/decomp/ACTION.ELF/001c6900, 001c6850].
pub const ANIM_FLAGS_AIMING: u32 = 0x180;
/// The head-tracking flag bits (`AnimObjectIsHeadTracking` 0x1c6850's mask) [M 001c6850].
pub const ANIM_FLAGS_HEAD_TRACKING: u32 = 0x30;

/// `AnimObjectIsAiming` 0x1c6900 [M 001c6900]: `(animobj+0xc8 & 0x180) != 0` once the object and its anim
/// exist — the callers pass a live object, so the null arms (`obj == 0 || obj+0xdc == 0 → false`) live at the
/// callers (`NDrone2_DoTracking` 0x1479d8 dereferences first). Reached only through the drone AI tick.
pub fn is_aiming(flags: u32) -> bool {
    flags & ANIM_FLAGS_AIMING != 0
}

/// `AnimObjectIsHeadTracking` 0x1c6850 [M 001c6850]: the 0x30 mask — `AnimObjectSetHeadTracking` 0x1c6748
/// sets these bits (`drone.rs` tracking fold cites it); `NDrone2_DoHeadTracking` 0x1472c8 early-returns on it.
pub fn is_head_tracking(flags: u32) -> bool {
    flags & ANIM_FLAGS_HEAD_TRACKING != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Quat_Slerp_Acc` negates the w0 coefficient and never renormalises [M 0x1e4188:30-36]: for opposite-w quats
    /// (the port's channel decode can produce both signs) the result keeps q1's sign — the old port negated q1
    /// instead and renormalised, giving +1 where the game gives −1.
    #[test]
    fn slerp_negates_w0_and_never_renormalises() {
        // dot = −1: |dot| = 1 → the linear branch (1 − 1 = 0 is not > 0.01); neg flips w0.
        let q = slerp([0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, -1.0], 0.5);
        assert_eq!(q, [0.0, 0.0, 0.0, -1.0], "neg must negate w0 (the q0 weight), not q1");
        // No renormalisation: the game adds w0·q0 + w1·q1 as is (0x1e4188:33-36) — a length-2 quat stays length 2.
        let two = [0.0, 0.0, 0.0, 2.0];
        let q = slerp(two, two, 0.5);
        assert_eq!(q, two, "a renormalised result would shrink the length-2 quat");
        // The slerp branch (dot 0 → ω = π/2): w0 = w1 = PS2Sinf(π/4)·(1/PS2Sinf(π/2)) = √½.
        let id = [0.0, 0.0, 0.0, 1.0];
        let half_turn_y = [0.0, 1.0, 0.0, 0.0];
        let q = slerp(id, half_turn_y, 0.5);
        assert!((q[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5 && (q[3] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5, "{q:?}");
    }

    /// The distance table plays frames by walked distance [M 0x1c4060/0x1c4270]: equal walked steps advance the
    /// implied root distance equally — the feet-don't-slide property the walk sets need.
    #[test]
    fn distance_table_maps_walked_distance_to_frames() {
        use nf_formats::anim::{Sequence, Segment};
        // 5 frames whose root z walks 1.0 per frame (channels: z = t, t = frame − 1); every channel
        // carries a segment (`AnimFrameCopy` evaluates the tracks the sequence carries, and an
        // empty track's `segs.last()` has no neutral stand-in on the read side).
        let mk = |end: u32, z: f32| vec![Segment { end, c: [0.0, z, 0.0, 0.0, 0.0, 0.0] }];
        let seq = Sequence {
            hashcode: 1,
            flags: 0,
            frames: 5,
            skeleton: 0,
            channels: vec![mk(4, 0.0), mk(4, 0.0), mk(4, 1.0), mk(4, 0.0), mk(4, 0.0), mk(4, 0.0)],
        };
        let t = DistanceTable::build(&seq, 4, true).expect("a 5-frame looping walk has a table");
        assert_eq!(t.segments, 4, "looping → frames−1 segments [M 001c4060]");
        assert!((t.total - 4.0).abs() < 1e-4, "each frame walks 1.0, total 4.0, got {}", t.total);
        // walked 0.5 → halfway through segment 1 → frame 1.5; the frame advance per walked unit stays 1.
        let mut w = 0.5;
        let speed = t.distance_to_frame(&mut w, 1.0, true);
        assert!((speed - 0.5).abs() < 1e-5, "frame 1.5 − time 1.0 = 0.5, got {speed}");
        assert!((w - 0.5).abs() < 1e-6, "the wrapped walked value is stored back [M 001c4270]");
        // Wrapping: walked total + 0.25 plays like 0.25.
        let mut w = t.total + 0.25;
        let _ = t.distance_to_frame(&mut w, 1.0, true);
        assert!((w - 0.25).abs() < 1e-6, "walked wraps into [0, total), got {w}");
        // Backward stride wraps the other way (0x1c4270's negative branch).
        let mut w = 0.25;
        let speed = t.distance_to_frame(&mut w, 4.5, false);
        assert!(speed < 0.0, "a backward stride plays the walk in reverse, got {speed}");
    }

    #[test]
    fn slerp_and_blend_follow_the_weights() {
        let id = [0.0, 0.0, 0.0, 1.0];
        let half_turn_y = [0.0, 1.0, 0.0, 0.0];
        let q = slerp(id, half_turn_y, 0.5);
        assert!((q[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5 && (q[3] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5, "{q:?}");
        let a: Pose = vec![([0.0; 3], id)];
        let b: Pose = vec![([2.0, 0.0, 0.0], half_turn_y)];
        // An outgoing script at weight 0.75 under the current one: 25 % of the way to the current pose.
        let p = blend_playing(b.clone(), &[(a.clone(), 0.75)]);
        assert!((p[0].0[0] - 0.5).abs() < 1e-6);
        // Faded out (weight 0): the current pose; no outgoing: the current pose.
        assert_eq!(blend_playing(b.clone(), &[(a, 0.0)])[0].0, b[0].0);
        // An outgoing one-shot holds its last frame while fading over 8 ticks.
        let mut o = Outgoing { script: 0, time: 9.0, speed: 1.0, looping: false, clock: 8.0, length: 8.0, unyaw: 0.0 };
        let mut n = 0;
        while o.tick(10.0) { n += 1; }
        assert_eq!((n, o.time), (7, 10.0));
    }
}

/// The facial state script tables [D ACTION.ELF .data 0x2c6ea8/0x2c6ec0, read via the ELF symbols]:
/// `AnimFacialUpdate` (0x1be310) appends table[0] as the idle face script (marker 0xe0000000) and
/// table[4] as the blink script (marker 0xe0000001); the gender select is bit 10 of the anim
/// object's +0x58 flags word. Each hash is an animation script (prefix 06).
pub const MALE_FACIAL_STATE_ANIMS: [u32; 5] = [0x0600_00f8, 0x0600_00f9, 0x0600_00f5, 0x0600_00f6, 0x0600_00f3];
pub const FEMALE_FACIAL_STATE_ANIMS: [u32; 5] = [0x0600_036d, 0x0600_036f, 0x0600_036a, 0x0600_036b, 0x0600_0370];

/// One facial script slot's contribution: the playing sequence's evaluated frame (the sAnimSeq's
/// +0x84 buffer [M `AnimSeqNew` 0x1c7248 / `AnimFrameSet` 0x1c75a0]) — for a face track the 22
/// channels ARE the morph weights, which `AnimFacialBlend` (0x1be648) reads through
/// `script+0x54 → seq+0x84`.
///
/// `facial_weights` samples a face-track sequence at script time `time` the way the sequence
/// player does (`script_frame`'s window → frame = time − first + 1; `AnimFrameCopy` evaluates
/// each channel at the integer frame, the fractional part lerps f/f+1 — [S] for the weights
/// buffer: `AnimFrameBlend`'s float-path lerp; the bone path's overshoot clamp/slerp does not
/// apply to scalars). `count` = the skin's facial count (disc +0x12).
pub fn facial_weights(seq: &Sequence, time: f32, count: usize) -> Option<Vec<f32>> {
    if count == 0 || seq.channels.len() < count { return None; }
    let (q, f) = script_frame_of(seq, time)?;
    let last = seq.frames.max(1) as u32;
    let f0 = (f.floor() as u32).clamp(1, last);
    let frac = (f - f.floor()).clamp(0.0, 1.0);
    let mut w: Vec<f32> = (0..count).map(|c| seq.value(c, f0)).collect();
    if frac > 0.0002 && f0 < last {
        for (c, v) in w.iter_mut().enumerate() {
            let nxt = seq.value(c, f0 + 1);
            *v += (nxt - *v) * frac;
        }
    }
    let _ = q;
    Some(w)
}

/// The covering opcode-0 window and fractional frame for a sequence played from script time
/// `time` from 1 (the `script_frame` arithmetic against a bare sequence: the whole sequence is
/// the window, frame = time, clamped to the length — `AnimSeqTick` holds the last frame).
fn script_frame_of(seq: &Sequence, time: f32) -> Option<(u32, f32)> {
    let last = seq.frames.max(1) as f32;
    let f = if time < 1.0 { 1.0 } else { time.min(last) };
    Some((seq.hashcode, f))
}

/// `AnimFacialBlend` (0x1be648) over the three facial script slots and the anim object's eye/lid
/// bytes (+0x97, +0x98): slot 0 (marker 0xe0000000, the idle/base face) copies all `out.len()`
/// weights straight; slot 1 (0xe0000001, blink) and slot 2 (0xe0000002, talk/forced) override a
/// weight only when |w| > 0.01; then +0x97 writes indices 14 (value/255) and 13 (1 − value/255),
/// +0x98 writes indices 11 and 12 the same way, each guarded by the same 0.01 threshold
/// (0.003921569 = 1/255). Under a script camera (`ScriptCam != 0`) only slot 2 applies, as a
/// straight copy. `out` = the object's morph weight array (animData+0xb0, `count` floats).
/// (`AnimFacialAppend` 0x1c6490 is the game's slot-2 feeder — it appends the talk script via
/// `AnimScriptAppend` with marker 0xe0000002, stop-at-end +0xb5 = 3, and sets the anim object's
/// speaking byte +0x109 = 2. The port's slot-2 consumer is this `slot2` parameter, but no
/// gameplay append fills it: `Drones::speech` skips facial lines — see the note there.)
pub fn facial_blend(out: &mut [f32], slot0: Option<&[f32]>, slot1: Option<&[f32]>, slot2: Option<&[f32]>, eyes: [u8; 2], script_cam: bool) {
    if script_cam {
        if let Some(s2) = slot2 {
            for (o, &w) in out.iter_mut().zip(s2) { *o = w; }
        }
        return;
    }
    if let Some(s0) = slot0 {
        for (o, &w) in out.iter_mut().zip(s0) { *o = w; }
    }
    for s in [slot1, slot2].into_iter().flatten() {
        for (o, &w) in out.iter_mut().zip(s) {
            if w.abs() > 0.01 { *o = w; }
        }
    }
    let write_lid = |out: &mut [f32], b: u8, v_idx: usize, inv_idx: usize| {
        let v = b as f32 / 255.0;
        if v > 0.01 { out[v_idx] = v; }   // the value: +0x38 → 14 / +0x2c → 11
        let inv = (255 - b as i32) as f32 / 255.0;
        if inv > 0.01 { out[inv_idx] = inv; } // the inverse: +0x34 → 13 / +0x30 → 12
    };
    write_lid(out, eyes[0], 14, 13); // animObject+0x97
    write_lid(out, eyes[1], 11, 12); // animObject+0x98
}

/// `control_AnimUpdate` 0x1341f0 (28 B) — the class-0x15/0x1F update dispatch slot (the .data
/// table at 0x26f98c/0x26fa04 [D]; docs/atlas/object.md:113,119) — is a one-line forwarder to
/// `AnimObjectUpdate` 0x1befd8 [M 0x1341f0:6], so the port's anim-object tick here does that
/// job under this name.
///
/// `AnimFacialUpdate` (0x1be310) — the idle/blink driver, one call per skinned object per frame
/// (`AnimObjectUpdate` 0x1befd8:127-136, gated on the skin's facial count, the big-skeleton flag
/// and `AnimObjectIsClose(obj, 0xc)`; frozen under `ScriptCam` and the +0x58 bit-0x10000 facial-off
/// flag). Pure state half: the state byte (low 7 bits, bit 0x80 = the female double-blink flag)
/// and the u16 frame timer live on the anim object at sub+0x99/sub+0x92.
///
/// Per state [M decomp 0x1be310, disasm 0x1be3e0-0x1be5cc]:
/// state 0 (init): the state stores 1 BEFORE the draw; one `Rand_Rand(180)` → timer = d + 90
/// (90..269 frames); the caller appends the table[0] idle face (marker 0xe0000000).
/// state 1 (idle/blink): an alert-level change re-appends the alert row's idle face and returns
/// (0 draws); the timer counts down unless the drone's alert level is 3 (combat — blinking frozen);
/// on wrap: one `Rand_Rand(120)` → timer = d + 120 (calm) / d + 60 (alerted), the caller appends
/// table[4] (the blink, marker 0xe0000001); a female then rolls `Rand_Rand(100)`: ≤ 50 done,
/// 51..75 → `Rand_Rand(6)` → timer = d + 6 (the double-blink comes at once, bit 0x80 set),
/// ≥ 76 → `Rand_FRand(0.2) + 0.2` = the blink script's start-time offset (0.2..0.4 s); with bit
/// 0x80 already set the next wrap clears it and rolls nothing.
/// state 2 (talk): stay while a talk script (marker 0xe0000002) plays; else state 1 + the alert
/// changed flag (0 draws — the next frame re-appends the idle face).
///
/// The game's gender select is anim-object flag bit 10 (raw+0xC8 & 0x400); the port passes its
/// `Drone+0x15` female byte (the skin-mask stand-in — flagged in the report). Returns the table
/// rows to append this frame.
#[derive(Debug, Default, PartialEq)]
pub struct FacialCmd {
    /// Append the idle face, table row (marker 0xe0000000) — the init's row 0 or the alert row.
    pub idle_row: Option<usize>,
    /// Append the blink, table[4] (marker 0xe0000001).
    pub blink: bool,
    /// The blink script's start-time offset (`Rand_FRand(0.2) + 0.2`, the ≥ 76 female roll) —
    /// seconds; 0.0 when the blink starts at once.
    pub blink_delay: f32,
}

pub fn facial_update(
    state: &mut u8, timer: &mut u16, female: bool, alert: u8, alert_changed: &mut bool, talking: bool,
    rng: &mut crate::drone::Rng,
) -> FacialCmd {
    let mut cmd = FacialCmd::default();
    match *state & 0x7f {
        0 => {
            *state = 1; // stored BEFORE the draw [M disasm 0x1be3e0 → the jal 0x1be3f0]
            *timer = (rng.rand(180) + 90) as u16;
            cmd.idle_row = Some(0);
        }
        1 => {
            if *alert_changed {
                *alert_changed = false;
                cmd.idle_row = Some(alert as usize);
                return cmd;
            }
            // The timer counts down unless the drone's alert level is 3 (combat freeze).
            if alert != 3 {
                if *timer == 0 {
                    // The wrap: the blink. One `Rand_Rand(120)`, timer = d + 120 (calm) / d + 60.
                    let d = rng.rand(120);
                    *timer = (if alert != 0 { 60 + d } else { 120 + d }) as u16;
                    cmd.blink = true;
                    // The female double-blink roll [M disasm 0x1be52c-0x1be5cc].
                    if female {
                        if *state & 0x80 == 0 {
                            let r = rng.rand(100);
                            if r > 50 {
                                if r <= 75 {
                                    let d2 = rng.rand(6);
                                    *timer = (6 + d2) as u16;
                                    *state |= 0x80;
                                } else {
                                    cmd.blink_delay = rng.frand(0.2) + 0.2;
                                }
                            }
                        } else {
                            *state &= !0x80;
                        }
                    }
                } else {
                    *timer -= 1;
                }
            }
        }
        2 => {
            // Stay while a talk script plays; else back to idle and re-append the face next frame.
            if !talking { *state = 1; *alert_changed = true; }
        }
        _ => {}
    }
    cmd
}

#[cfg(test)]
mod facial3_tests {
    use super::*;

    /// `facial_update` (0x1be310, fix-facial3): the state machine's transitions and the draw
    /// accounting — the init's state-before-draw, the wrap's `Rand(120)` + the female rolls.
    #[test]
    fn facial_update_state_machine_and_draws() {
        let mut rng = crate::drone::Rng::new(0x1234_5678);
        // State 0 (init): state stores 1 BEFORE the draw; one Rand(180) → timer 90..269; row 0.
        let (mut st, mut tm) = (0u8, 0u16);
        let cmd = facial_update(&mut st, &mut tm, false, 0, &mut false, false, &mut rng);
        assert_eq!(st, 1);
        assert!((90..=269).contains(&tm), "timer = Rand(180)+90, got {tm}");
        assert_eq!(cmd, FacialCmd { idle_row: Some(0), blink: false, blink_delay: 0.0 });
        // State 1, alert changed: the idle row re-appends with 0 draws (the timer untouched).
        let mut changed = true;
        let cmd = facial_update(&mut st, &mut tm, false, 2, &mut changed, false, &mut rng);
        assert_eq!(cmd.idle_row, Some(2));
        assert!(!changed);
        // State 2 (talk): stay while talking; exit sets state 1 + the re-append flag, 0 draws.
        let mut st2 = 2u8;
        facial_update(&mut st2, &mut tm, false, 0, &mut false, true, &mut rng);
        assert_eq!(st2, 2);
        facial_update(&mut st2, &mut tm, false, 0, &mut false, false, &mut rng);
        assert_eq!(st2, 1);
        // State 1 wrap: force the timer to 0 → the blink; the calm timer = Rand(120)+120.
        let mut stf = 1u8;
        let mut tmf = 0u16;
        let cmd = facial_update(&mut stf, &mut tmf, true, 0, &mut false, false, &mut rng);
        assert!(cmd.blink, "the wrap blinks");
        assert!((120..=239).contains(&tmf), "the calm wrap timer = Rand(120)+120, got {tmf}");
        // The combat freeze: alert 3 never wraps (0 draws, the timer frozen).
        let mut stc = 1u8;
        let mut tmc = 0u16;
        let cmd = facial_update(&mut stc, &mut tmc, false, 3, &mut false, false, &mut rng);
        assert!(!cmd.blink && tmc == 0, "alert 3 freezes the blink");
    }
}

#[cfg(test)]
mod anim_object_flag_queries {
    use super::*;

    /// `AnimObjectIsAiming` 0x1c6900: the 0x180 mask — either bit alone aims, both aim, and the
    /// head-tracking bits 0x30 are NOT aim bits [M 001c6900: `(*(uint *)(iVar1 + 200) & 0x180) != 0`].
    /// Sister: the same word with only the 0x30/0x40/0x200 bits set does not aim.
    #[test]
    fn is_aiming_follows_the_0x180_mask() {
        assert!(is_aiming(0x180), "both aim bits set [M 001c6900]");
        assert!(is_aiming(0x080), "the 0x80 bit alone aims");
        assert!(is_aiming(0x100), "the 0x100 bit alone aims");
        assert!(is_aiming(0x180 | 0x30), "aim bits win over the track bits");
        // The sister: the neighbouring flags must not leak into the aim answer.
        assert!(!is_aiming(0x30), "head-track bits 0x30 are not aim bits");
        assert!(!is_aiming(0x40), "0x40 (outside both masks) is not an aim bit");
        assert!(!is_aiming(0x200), "0x200 is outside the 0x180 mask");
        assert!(!is_aiming(0), "the idle flag word does not aim");
    }

    /// `AnimObjectIsHeadTracking` 0x1c6850: the 0x30 mask — 0x10 and 0x20 each track; the aim bits
    /// 0x180 are NOT track bits [M 001c6850: `(*(uint *)(iVar1 + 200) & 0x30) != 0`].
    /// Sister: the aim flag word with no track bits does not head-track.
    #[test]
    fn is_head_tracking_follows_the_0x30_mask() {
        assert!(is_head_tracking(0x30), "both track bits set [M 001c6850]");
        assert!(is_head_tracking(0x10), "the 0x10 bit alone tracks");
        assert!(is_head_tracking(0x20), "the 0x20 bit alone tracks");
        assert!(is_head_tracking(0x30 | 0x180), "track bits win over the aim bits");
        // The sister: the aim bits must not leak into the track answer.
        assert!(!is_head_tracking(0x180), "aim bits 0x180 are not track bits");
        assert!(!is_head_tracking(0x40), "0x40 is outside the 0x30 mask");
        assert!(!is_head_tracking(0), "the idle flag word does not track");
    }
}
