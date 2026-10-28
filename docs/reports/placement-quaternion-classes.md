# placement-quaternion-classes

Branch `s/placement-quaternion-classes` (from origin/master @ fcf6f334; master did not move during the session, so the
branch tree **is** the merged final tree). Task: take the last two placement classes off the port's Euler fallback —
0xf2 (emitters, the dispatcher's inline build) and 0x34 (`GunImp_Create`) — by parsing the placement's +0x24
quaternion, porting `Quat_QuatToMat` exactly, building both classes from it, and pinning the disc rows against the
decompile.

## What's done

**The quaternion, parsed and ported.**
- `Placement::quaternion()` returns the +0x24..+0x34 words as (x, y, z, w). The dispatcher
  `parsemap_create_dynamic_objects` copies exactly those four words into the `quaternion_tag` it hands the creators
  that take one [M 0x1d04f0 prologue: +0x24/+0x28/+0x2c/+0x30 → the quaternion stack slots]. `docs/formats/placement.md`
  now documents the field.
- `nf_formats::placement::quat_quat_to_mat` ports `Quat_QuatToMat` 0x1e3e80 store for store: 2y/2z as doubling adds
  (fVar1/fVar2 [M :18,:20]), (2x)·x and (2x)·w (fVar6/fVar8 [M :21-22]), the nine rotation entries in the decompile's
  order [M :23-31], through the EE FPU's no-guard adds and chopped muls. `nf_game::drone::quat_quat_to_mat` (the
  version already verified bit-exact against the 40 recorded game calls) now delegates to it — one implementation; the
  delegation is bit-identical (same `add_noguard`/`mul` ops) and `diff_calls` + `trig_live` re-verified green.

**Both classes build from it.**
- `object_matrix_for_class` gained the quaternion argument; for 0x34/0xf2 (non-`creates_cel`) it returns the quat
  matrix with the placement translation in the bottom row — `Mat_SetTrans` 0x1e3478 writes only +0x30..+0x38, and the
  0xf2 dispatcher stores the position into the matrix's row 3 itself [M 0x1d04f0:476-477]. For 0x34 the prebuilt matrix
  then overwrites the created object's matrix at `Control_CreateObjEx` 0x1343c8:18-23. Call sites updated:
  `level.rs` static Instances and `props.rs` bounds both pass `p.quaternion()`.
- **0x34 / GunImp:** the drawn turret-base Instance now uses the quat builder. The port's runtime base stays the
  documented yaw-only model (`yaw_base`): every 0x34 disc row is yaw-only with a consistent pure-yaw quat (identity or
  a π yaw; max |quat matrix − euler matrix| = 4.8e-7 over all 37 rows [D]) — so `yaw_base = rotation[1]` equals the
  quat's yaw on all disc data. The census pins this per row. gunimp.rs's module/base docs cite the chain.
- **0xf2 / emitters (the `world_fx.rs` inline note, now resolved):** the decompile chain is longer than "build the
  matrix": the dispatcher builds the quat matrix [M 0x1d04f0:474-478], `Emitter_CreatePlist` passes **only its up row**
  (`Mat_GetUp`, 0x17b848; matrix +0x10 [M Mat_GetUp 0x1e33b8]) plus the +0x30 translation row as the position, and
  `Emitter_Create` rebuilds the spawn basis from that up vector exactly like the impact path —
  `Vec_MakeOrthonormalBasis` + `Mat_SetDir/SetNorm/SetUp` [M 0x17ae00:104-107; rows Norm = v, Up = up, Dir = u]. The
  port now runs that chain at the spawn site. A pure-yaw quat's matrix row 1 is (0, 1, 0), so every yaw-only placement
  spawns on the reconstructed identity basis; the 0xf2 particle spawn position (the translation row) and all RNG
  discipline are untouched.

## The disc data (census, this ISO — 99 level bins, 2,230 maps)

- **0x34: 37 rows, all yaw-only**, every row's quat matrix equals the Euler matrix within 1e-6 (measured max 4.8e-7).
  The one SP mount (#225 in 0700000d) has the identity quat.
- **0xf2: 69 rows; 17 yaw-only** (all quat==euler within 1e-6), **33 rows quat==euler within 1e-6**, **36 rows where
  the quat diverges from the Euler reading (>1e-3; every one has rz≠0)**. Example (map 54 row 987): the Euler words
  say rz = +π/2 while the quat is a −90° z-rotation — the emitter's up row is (+1, 0, 0) by quat vs (−1, 0, 0) by
  euler, i.e. the port's interim Euler reading had those emitters' spawn directions mirrored/rotated. The game reads
  the quat [M], so those rows now sit where the game puts them.
- The task text said "16 of 69 disc rows carry pitch/roll"; the census columns are **rx≠0 = 16, rz≠0 = 52** (the old
  `world_fx.rs` note's "16/52" was the same pair). Recorded here because the wording was ambiguous; the hard test pins
  per-row properties, not the totals.

## Test changes (one existing test was wrong against the decompile)

- `placement.rs`'s class-predicate unit test pinned 0x34/0xf2 to the RotMatrix fallback — that contradicted
  [M 0x1d04f0:474-478 and 0x131f20:66-69]. Fixed: those classes now expect `Quat_QuatToMat`(+0x24 quat) with the
  placement translation; the addresses are cited in the test comment.
- New hard census test `placement_rot_census::census_quat_classes_build_from_the_placement_quaternion`, per row over
  every map (holds on any disc subset): (1) the class builder returns the quat matrix + translation; (2) every
  yaw-only row's quat matrix stays within 1e-6 of the Euler matrix (the "yaw-only emitters stay where they are" pin);
  (3) every 0x34 row closes onto RotY(rotation[1]) — the GunImp yaw-model equivalence; (4) the parse sees the four
  words the dispatcher copies. Totals print informationally.
- New unit test `quat_to_mat_matches_the_decompile_and_closes_on_yaw` replays the decompile's locals bit-for-bit and
  closes a pure-yaw quat onto the RotY rows.

## How it was verified

- `nf-formats`: lib 92/92 + placement/census suites (the 3 `disc_golden` failures are the allowed set on this image);
  the new census test green (0x34 37/37; 0xf2 33 + 17 pinned, 36 divergent).
- `nf-game`: lib 578/578; `diff_calls` 6/6 (the recorded `Quat_QuatToMat` calls — proves the moved implementation
  bit-exact); `trig_live` 4/4; `blast_smoke_live` 4/4 (emitter path).
- `nf-view`: `load_scene_equality` 4/4 (hash pins unchanged — the 0x34/0xf2 instance moves are ≤ 1e-6 or the instances
  don't resolve); `gunimp_mounted_live`, `0700000d_gunimp_beam_lifetime_live`, `0700000d_gunimp_rate_live`,
  `0700000d_gunimp_beam_axis_live` 5/5 (the mount path).
- Full suite (`tools/test/suite.py`) on the final tree: build ok, 874 test binaries; the run's total line is quoted
  at the end of this report. Failures, all accounted for:
  - the 3 `disc_golden` tests — the allowed set on this image;
  - `fix_0c_pylon_live` (2 tests) + `probe_pylon_render` (1 test) — **environmental, not the change**: all three panic
    at wgpu adapter creation (`adapter: NotFound { active_backends: Backends(0x0) ... }`, `crates/nf-view/tests/fix_0c_pylon_live.rs:19`)
    — this sandbox has the Vulkan loader (`libvulkan.so.1`) but **no ICD** (`/usr/share/vulkan/icd.d` absent) and no
    display for GL, so wgpu finds no adapter in ANY build here; the panics happen before any placement-derived data is
    used (the scene load itself succeeded). No root in the sandbox, so the ICD could not be installed.

Full-suite total (final tree): **the run did not reach its total line before this sandbox's hard deadline** (the
runner was still executing the 874-binary sweep at cutoff). State at cutoff (00:31 UTC, suite started 23:33 UTC):
**335 of the binaries had completed, with exactly the six failures accounted for above** — the three `disc_golden`
tests (allowed on this image; also reproduced directly: `cargo test -p nf-formats` shows them as the only failures)
and the three GPU-less pylon renders. Everything else green, including `nis_handback_census` (the previous task's
census stays green), `diff_calls`, `trig_live`, `blast_smoke_live`, `load_scene_equality`, all gunimp tests, and the
new `placement_rot_census` pins. The remaining ~539 binaries were still running when the sandbox expired; they are
the same battery the branch's targeted runs covered (nf-view sims and renders; the renders would hit the same missing-ICD
wall). The per-binary log went to `../suite.log`; `tail -1 suite.out` had no total line at cutoff — none is quoted
because none existed.

## What's left

- Nothing known. The 36 divergent 0xf2 rows now emit where the game's quats put them; a main-agent render of one such
  scene would be a nice visual confirmation — the rows are listed by (map, row) in the census output, and the
  up-tilt signs are the visible change (e.g. map 54's rows 987-1014: emitters whose spawn direction flips x).

## Live checks for the main agent

- Optional visual: render a scene containing a divergent 0xf2 emitter (map 54 rows 987-1014 — find the level by
  loading each level bin and counting maps; the census prints map indices). Expected: the smoke/spawn column leans
  along the quat's up row (e.g. +x where the old build leaned −x).
- `tools/trace/gdbwatch.py` on `Quat_QuatToMat` 0x1e3e80 during a fresh boot into a level with 0xf2 emitters:
  expect a1 (the out matrix) = the dispatcher's `auStack_80` and a0 pointing at the placement copy at sp+0x90 area
  [M 0x1d04f0:474-478]; then on `Emitter_CreatePlist` 0x17b848, expect `Mat_GetUp`'s out (sp+0x40 area) to equal the
  matrix's +0x10 row — confirming the up-row-only consumption.

## Usage

not visible to me (this environment shows no token, credit or context-window figures for the session; no input/output
token or cost counters are exposed to the agent).
