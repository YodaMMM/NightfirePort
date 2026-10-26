## Your task: `level-0a-band-settle-5`, the per-face push with its live operands

Start from `origin/s/level-0a-band-settle-4` (merge it, then origin/master) and read its decode. Its three live checks were run on the real game (PCSX2, 0700000a, god mode, Bond walking west from (-8.0, 1.12, 16.29), a breakpoint at 0x10a540 in `Intersect_CylGeom`'s radial block; 12 consecutive hits, all on ONE contact tick with Bond at (-7.8138, 1.1216, 16.4328); s1 = the HITTEST stack copy 0x1fffa30, sp 0x1fff800) [L 2026-10-26]:
1. **copy+0x50 = Dist2Tri's closest point Q** (your finding 2 confirmed): per face (-8.304, 0, 16.2788), (-8.2249, 0, 16.1597), (-8.304, 0.1622, 16.2221), (-8.304, 0.1622, 16.2788), (-7.8138, 0, 16.2788), (-7.8138, 0, 16.4328), (-7.5129, 0, 16.8199), (-6.9183, 0, 16.0449), (-8.304, 2.0206, 16.2787) x2, (-8.304, 0.8562, 15.9498), (-8.304, 0.1793, 16.1899).
2. **copy+0x60 is constant across every face of the tick: (-22.0782, 0.3832, 12.2086)** — it does NOT track pos + the pushes; it is the accumulator's transformed image (the add-back target), unchanged within a tick.
3. **96(sp), the query point, rides Bond's spine:** (-7.8138, 0.6368, 16.4328) on most faces = pos.y - 0.4848 (the B end), (-7.8138, 1.3966, 16.4328) on the two y-2.02 faces = pos.y + 0.275 (the A end), and (-7.8138, 1.0941, 16.4328) on one face = the on-edge projection onto the spine between A and B. x/z = Bond's exactly.
Port the per-face push with these operands (query = the spine point, Q = Dist2Tri's closest point, the accumulator image only in the add-back), reproduce round 3's jamb frame table, then un-ignore the three walkreal_0700000a tests and keep every other walkreal test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-band-settle-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T22:02Z

---
## Report (done, 2026-10-26T23:31Z)


---
## Report from the worker (done, 2026-10-26T23:31Z)

# level-0a-band-settle-5 — the radial tail's operands settled at the metal (v = query − Q, the Dist2Tri flag table, the Q2 re-run, the three accumulators); the port experiment reproduced the 0a doorway slide at the game's rate but regressed 01's plate walk — measured-and-reverted (the tree = the settle-3 merged state byte-identical)

## What was asked

Port the per-face push with the live operands (query = the spine point, Q = Dist2Tri's closest
point, the accumulator image only in the add-back), reproduce round 3's jamb frame table, then
un-ignore the three `walkreal_0700000a` tests and keep every other walkreal test green.

## The decode [M — the decompile + mipsdis over work/elf/ACTION.ELF; settle-4's LQC2/SQC2 flip applied]

1. **v = QUERY − Q — settle-4's finding 3 had the order inverted.** The tail loads vf3 = 96(sp)
   (the query), vf1 = copy+0x50 = Q (Dist2Tri's out-write), vf2 = copy+0x00 = the face plane
   (0x10a53c-0x10a544), computes `_vsub(auVar32,auVar10)` = query − Q (the decompile; the disasm
   stores the result to 0(sp) at 0x10a558 and re-loads it as the dir at 0x10a5e4). The [L] wall
   press decides the direction: v = query − Q points AWAY from the face (outward), `dot(v, n) > 0`
   passes, and the push v̂·(radius−|v|) yields EXACTLY the [L] 0.1/tick equilibrium with the query
   pressed r−0.1 = 0.45 inside and the first touch r−0.539 = 0.01056 [L 2026-10-26]. With
   settle-4's Q − query both gates fail on every wall contact and the game could never stop at a
   wall — impossible.
2. **Dist2Tri 0x109770 decoded end to end.** Signed plane distance first: `dot(p, plane) + w < 0`
   → flags = 99, **the out-pointer is never written**, early return of the negative distance
   [M 0x109770; the caller skips flags == 99 outright at 0x10a528 and its `0.0 <= return` gate at
   0x10a510 double-covers]. Else the three edge half-plane tests (early exit at 2 bits):
   flags 0 = the interior projection (Q = the plane projection, no second query — the floor's
   v = 0 case), 1/2/4 = the clamped edge closest points (with the t>1 fall-through to the shared
   vertex), 3/5/6 = the vertex arms. The RETURN is the true euclidean distance — the common tail
   `SQRT(|out − query|²)` — for every non-99 path.
3. **The second query re-runs the closest point.** When flags ∉ {0,99}, the game projects
   copy+0x50 — which at that moment holds **Q1** (Dist2Tri #1's out-write, 0x10a2fc
   `addiu t2,s1,0x50` before the 0x10a324 jal) — onto the spine A/B clamped [0,1]
   (0x10a3b4/0x10a3c4), and RE-RUNS Dist2Tri from it (0x10a4fc, the out-write again): the tail's
   Q = **Q2**. The port's old `if on_edge { p = closest_on_segment(a, b, q); }` projected Q1 but
   never re-ran — the port kept Q1 in the tail. The [L] 96(sp) dump (the feet/A-end/on-edge
   points riding the spine) plus copy+0x60 constant at (−22.0782, 0.3832, 12.2086) across all 12
   faces [L 2026-10-26] close the "restored accumulator image" reading: the projection target is
   Q1, NOT the accumulator image.
4. **The gates and the push** [M 0x10a568-0x10a61c]: `0 < dot(v, plane)` (0x10a568 c.ole → bc1t
   skip) — a floor's interior crossing (v = 0) and a query behind the plane never push — then
   `|v| < radius` (0x10a5c8; radius = copy+0x8c); dir = v·(1/|v|), and **|v| == 0 → the dir quad
   ≈ 0** (0x10a5bc `sq v0,0(sp)` — the flags-bits denormal, the game's "clear" trick; push ≈ 0,
   NOT a ±n fallback); mag = radius − |v| (0x10a5d8).
5. **The accumulators.** The push adds into THREE vectors: DeltaP 0x2d9bd0 (0x10a5e8 load /
   0x10a608 store — the add-back total), and 16(sp)/32(sp) — which the prologue initializes as
   the query AABB **min(A,B) − (radius+0.1) / max(A,B) + (radius+0.1)** (0x109ddc/0x109dbc,
   fVar28 = radius + 0.1): the broad-phase box tracks the pushed capsule across the tick's faces.
   The spine endpoints move WITH the push (0x10a614/0x10a61c stores to copy+0x20/+0x30), so
   subsequent faces see the updated spine — the port's per-face a/b updates match. The lifetime
   accumulator (HITTEST+0x50) is NOT an operand anywhere in the tail: copy+0x60 (its transformed
   image, constant across the tick [L]) is only the add-back target (Collide_Intersect
   0x1ebaa8:353-373).
6. **The best-contact HITDATA** [M 0x10a738-0x10a7f8]: when a face's |v| beats the best-so-far
   (init = the radius), the game records HITDATA+0x40..0x4c = copy+0x50 (Q), +0x0c = the distance,
   +0x50 = the material byte, and at the return +0x10 = vert2·1e-4 and +0x30 = (a VF00 leftover)
   − vert2·1e-4. The pick-bit block (sVar1 == 0): `dot(axis, plane) > 0.7`, then R = Q − (B+push),
   `dot(R, axis) < 0` → R̂·axis < 0.7 → HITTEST+0x9c |= 1 — the port's pick logic matches.

## What landed, and was reverted

`capsule_tri_contact` (crates/nf-game/src/world.rs) got the five deltas: the settle-3 radial arm
(ref = the accumulator) removed; the behind-plane skip (`dot(p − t0, n) < 0` — the flags-99 arm);
the on-edge second query re-running `closest_on_triangle` (Q2); the always-on dot gate
(`dot(v, n) <= 0` → skip); |v| ≈ 0 → push 0. The NF_CAPS_GATE scratch gate and static were
deleted (the tail's real gates are now decoded); `_ref_v` stays plumbed (the add-back callers).

Measured:

- `cargo test --offline --release -j2 -p nf-game --lib -- --test-threads=1`: **578 passed / 0
  failed / 5 ignored**.
- The 0a battery (`walkreal_0700000a_live`, --test-threads=1): **5 passed / 0 failed / 3
  ignored** — no regression on the active set.
- **The doorway slide reproduced**: the glide test's old route aimed leg 1 at (−8.45, 16.29) — a
  point INSIDE the jamb column's x-range (−8.604..−8.304 [D port world tris]) — and the walker
  blew through the 0.4-arrive window, oscillated through the doorway, and hit the 6000-tick cap
  (~5900 ticks) while its final grind north was still advancing. With the test's legs collapsed
  to the game's single through-line ((−3.9, 15.5) → (−10.0, 16.29), the [L] glide is ONE walk)
  and the new rule in, the walker **crossed the doorway in 155 ticks, exiting at (−9.27, 1.12,
  16.60)** — the slide north around the jamb at ~0.013-0.015/tick, the [L] slide rate (16.29 →
  16.83 over ~36 frames ≈ 0.015/tick [L 2026-10-26]).
- The glide y read **1.1201 vs the [L] settled 1.1285** at x −9.33: the hall glide's +0.0256
  climb settle (the feet-band walked-dy growth [L band-settle finding 1]) is a separate mechanism
  this task did not touch.
- **The regression**: `walkreal_07000001_live` ran **2 passed / 1 FAILED** —
  `walk_from_the_spawn_to_the_exit_with_the_contact` (the AND-touch plate approach, NF_WALK_KILL
  off — the same symptom settle-3 saw with the kill ON). The battery takes ~145 s per run; the
  sandbox's remaining budget allowed no bisect across the five deltas.
- **Reverted** (the settle-4 discipline): `crates/nf-game/src/world.rs` and
  `crates/nf-view/tests/walkreal_0700000a_live.rs` are byte-identical to the merged settle-3
  state (`git diff 5e5909e9` empty on both). Re-run on the reverted tree: nf-game lib **578/0**,
  the 0a battery **5/0 + 3i**. The 01/05 batteries were not re-run on the reverted tree — it is
  the exact settle-3 merged code whose record is docs/reports/level-0a-band-settle-3.md (01 3/0,
  05 8/0, nf-game 925/0 workspace-wide).

## What's left (the next round's plan)

1. **Bisect the five deltas against 01's plate walk** (one env-gated flag per delta, 145 s/run):
   the prime suspects are the dot gate (it skips faces the old fallback pushed — v ⊥ n or the
   query behind a sloped approach face) and the Q2 re-run (the stairwell is all edge contacts).
   If a delta is genuinely the game's ([M]) and the test still shifts, the TEST's plate pins need
   the live re-check (the game's approach with the true gates).
2. **Un-ignore the trio** once 01/0a are co-green: the glide route fix (the through-line) is
   already decoded here; the y-settle climb (1.1285 [L] vs the port's 1.1201) needs the feet-band
   walked-dy growth; the corner and the ring legs ride the same slide.
3. **The handler tail 0x194c70** (`obj+0x30 += BL+0x750` — the wall-press stopper [L
   2026-10-26]) is still not ported as such; the port's push-a/b path stands in.
4. The game's 12-face jamb tick pushes NOTHING under the settled rule (all |v| ≥ 0.63 > radius
   0.55 with the dumped Qs and queries [L]) — the slide pushes live on the neighbouring ticks'
   edge contacts; a per-tick [L] push table around the 12-face tick would pin the rotation.

## Live checks for the main agent

Fresh boot of 0700000a, the west hall:

1. **The slide's per-tick HITTEST+0x10** during the jamb pass (Bond walking west from
   (−8.0, 1.12, 16.29)): dump +0x10 per tick for ~40 ticks. Expect ~0.02-0.07 with the rotating
   x/z split; correlate each nonzero tick with Bond's z — this pins WHICH faces push (the edge
   contacts) and closes the 12-face-tick question above.
2. **The 01 plate approach with the true gates**: the same breakpoint protocol at 0x10a540 on
   07000001's `walk_from_the_spawn_to_the_exit_with_the_contact` route — dump 96(sp) and
   copy+0x50 on the ticks approaching the AND-touch plate. If the game's radial tail really skips
   the faces my dot gate skips, the port's plate shift is the game's shape and the test's pins
   move; if the game pushes them, one of the five deltas is mis-encoded.

## WORKLOG entry (docs/WORKLOG.md was deleted on master — the entry lives here)

level-0a-band-settle-5 (2026-10-26): branch s/level-0a-band-settle-5 from origin/master + merge
origin/s/level-0a-band-settle-4. Decoded the radial tail's operands at the metal (v = query − Q
— settle-4's order inverted, the [L] wall-press equilibrium decides; the Dist2Tri flag table; the
Q2 re-run; the three accumulators; the best-contact HITDATA), landed the five deltas, reproduced
the 0a doorway slide at the game's rate (155-tick crossing, z 16.60), regressed 01's plate walk,
measured-and-reverted to the settle-3 merged state. Next: the five-delta bisect on 01, then the
trio un-ignore with the y-settle climb.

## What I ran

- `cargo test --offline --release -j2 -p nf-game --lib -- --test-threads=1` — 578/0 (with the
  rule) and 578/0 (on the reverted tree).
- `cargo test --offline --release -j2 -p nf-view --test walkreal_0700000a_live --
  --test-threads=1` — 5/0 + 3i (with the rule) and 5/0 + 3i (reverted tree).
- `walkreal_0700000a_live --ignored` (the trio, with the rule): 0/3 (the glide failed only on
  the y pin after the successful 155-tick slide crossing; the corner/ring failed on their legs).
- `walkreal_07000001_live --test-threads=1` (with the rule): 2/1 —
  walk_from_the_spawn_to_the_exit_with_the_contact FAILED (the revert reason).
- The workspace suite (`tools/test/suite.py`): the background start died at the alsa-sys build
  (the sandbox shell drops PKG_CONFIG_PATH between tool calls; the working env
  PKG_CONFIG_PATH=/tmp/alsa-pkg/usr/lib/x86_64-linux-gnu/pkgconfig + LIBRARY_PATH=… was found too
  late for a full restart inside the window). No suite total line; the direct runs above are the
  touched-crate record.

## Suite result

No total line (the alsa-env failure above; the window closed before a restart could finish).
The touched crates' direct runs: nf-game lib 578/0 (both trees), nf-view's 0a battery 5/0+3i
(both trees), nf-view's 01 battery 2/1 with the rule (the revert reason), the trio --ignored 0/3
with the rule.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
