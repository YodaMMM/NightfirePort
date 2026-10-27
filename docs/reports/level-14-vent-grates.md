# level-14-vent-grates — the duct-mouth grates drew off their frames because the SP placement pose used the wrong matrix builder; ported `View_RotTransMatrix` and the wall grates now sit flush in the frames

Branch `s/level-14-vent-grates` = `origin/master` + this round (commit 76df53dd).

## What is done

1. **The objects identified [D placements].** The duct-mouth grates are the nine 0xd9
   ScriptPlayer placements of object `0x02000ab4` "Vent_StoreRoom_Graphic" (a thin panel,
   object-space bounds x ±0.74, y ±0.025, z −1.44..0 — the panel hangs BACK from its origin).
   The driver's two view points see pl 848 (30.10, 3.58, 46.42) rot (−π/2, −π/2, 0) — the west
   storage-room duct mouth — and pl 849 (51.58, 3.58, 44.94) rot (−π/2, +π/2, 0) — the east
   corridor's duct mouth; the family also covers the vent-room's flat floor grates (yaw-only)
   and the pitched ones at (38.45/34.49, 3.58, ~36) and the ceiling one at (40.80, 2.37, 69.65).
   Evidence: `crates/nf-view/examples/probe14_grates.rs` / `probe14_grates2.rs` (committed).
2. **The game's pose rule [M].** `parsemap_create_dynamic_objects` 0x1d04f0 **case 0xd9** →
   `SP_CreateScriptPlayer(pos, rot, ...)` 0x1db9a8 → `control_create_object(0x3c, pos, rot, 0)`
   0x1344e0, whose param_4 == 0 arm builds the object matrix with **`View_RotTransMatrix`
   0x1e5168(rot triple, pos)** — NOT `RotMatrix` 0x1e2368, which is what the port used for every
   placement. The grate script itself (param 0 = 0x06000a3b [D placement params], decoded in
   `probe14_grates3.rs`) creates the LID `0x02000750` at t0 and the ladder `0x02000751` at t61
   with all four keys IDENTITY — so the grate's pose IS its placement matrix, built by the View
   composition. There is no open/kicked state, no hinge: the grate is a static-faced SP object
   (the kick opens the LID the script creates over it — a separate mechanic, already ported).
3. **The View composition ported, bit-exact discipline.** The decompile's PS2Sinf3 output slots
   do not line up with its local names (the outputs land at sp+0/4/8 and sp+0xc/0x10/0x14 while
   the named locals sit elsewhere); the disassembly pins them (two `jal PS2Sinf3` calls with
   a0/a1/a2 = sp, sp+4, sp+8 and sp+0xc, sp+0x10, sp+0x14; inputs f12 = rx+π/2 / rz+π/2, f14 =
   ry+π/2 / ry, f13 = rx / rz; the π/2 literal 0x3fc90fdb on the move). Rows:
   `row0 = [cy·cz, sz, −cz·sy]`, `row1 = [sx·sy − cx·cy·sz, cx·cz, cy·sx + cx·sy·sz]`,
   `row2 = [cx·sy + cy·sx·sz, −cz·sx, cx·cy − sx·sy·sz]`, translation row 3 — ported as
   `placement::view_rot_trans_matrix` (crates/nf-formats/src/placement.rs) with the same
   pf::mul/add_noguard discipline as `object_matrix`.
4. **Used where the game uses it [M].** `Player::from_placement` (the SP base — the created
   entities and the SP's own runtime object ride it) and the static draw's **0xd9 family** in
   `level.rs instances()`. Other placement classes keep the RotMatrix reading until each creator
   is pinned; **for rx = rz = 0 the two compositions collapse to the same rows**, so every
   yaw-only placement (the overwhelming majority, and every existing pinned test pose) is
   bit-identical under the swap.
5. **Verified against the geometry [D-derived].** pl 848's panel under the OLD reading spanned
   x 30.10..31.54 — 1.44 units proud of the west wall plane, hanging into the room (the
   driver's "hanging inside the duct, off their frames"). Under the ported composition it spans
   **x = 30.12 (the wall plane), z 45.68..47.16, y 2.14..3.58** — flush in the frame, filling
   the duct mouth. pl 849 likewise (x 51.55, z 44.20..45.68, y 2.14..3.58). pl 266 (flat,
   yaw-only) is unchanged. The probe printing these corners is committed
   (`probe14_grates4.rs`). The visible confirmation is the main agent's render.

## How it was verified

- `cargo test --offline --release -p nf-formats`: **94 green across the unit suites**; the
  `disc_golden` binary shows the allowed 3 slim-ISO failures (11 passed, 3 failed) — the
  baseline.
- The committed probes (assertion-free, the evidence chains above).
- The background `tools/test/suite.py` (started 09:06 UTC on the pre-fix tree — this branch at
  master + docs): still running at report time; so far 2 CRASHes (mp_stress_explosives_live,
  mp_online_failures — exit −9, the sandbox's memory kill, the same pattern the previous
  session's run showed on a different tree) and no FAILED/TIMEOUT lines besides. No total line
  yet; the fix landed after that build, so the tally reflects the pre-fix tree (the touched
  code is nf-formats/nf-game placement matrices — the suite's own build had already pinned the
  pre-fix binaries).

## What is left

- The main agent's render of the two mouths (`--bond` spots: (32.61, 2.80, 45.27) looking at
  (30.10, 3.58, 46.42); (56.03, 0.65, 46.89) looking at (51.58, 3.58, 44.94)) — each grate
  should sit flush in its frame, panel in the wall plane, no more proud-of-wall plate.
- Whether the OTHER pitched placement classes (0x19 non-main-map, 0x2a/0x22 doors/markers,
  pickups) also build through `control_create_object` (they call their own creators —
  `Door_Create`, `Ladder_Create`, `Pickup_Create` — which may pass a quaternion or their own
  matrix; each needs its own [M] pin before switching). Yaw-only placements are unaffected by
  the question.
- The suite's total line on the final tree (this window's background run predates the fix).

## Live checks for the main agent

Fresh boot 07000014:

1. Stand at (32.61, 2.80, 45.27) facing the west duct mouth: is the vent grate FLUSH in the
   wall frame (a plate in the wall plane filling the mouth), not hanging into the room?
2. Stand at (56.03, 0.65, 46.89) facing the corridor's duct mouth: same check for the second
   grate.
3. gdbwatch the SP object's matrix at obj+0x24 for the grate object after load (the created
   0x3c object): rows should match `view_rot_trans_matrix` for rot (−π/2, −π/2, 0) —
   row0 ≈ (0, 0, 1), row1 ≈ (1, 0, 0), row2 ≈ (0, 1, 0) with trans (30.10, 3.58, 46.42).

## Usage

not visible to me (this environment exposes no token, credit or context-window figures).
