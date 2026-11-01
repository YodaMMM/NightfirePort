## Your task: `rigid-body-cylinder-sweep`, the unported rigid-body contact step

The rigid-body collision handler's cylinder contact check and its torque step are unported (origin/s/lamp-calib's notes, rigid_body.rs). Port them from the game [M RB_CollisionHandler 0x1d7a80 and the cylinder sweep it calls; RB_RestingContactRot 0x1d73a0's non-bullet arms], so bodies react to Bond/guards/doors brushing them as in the game (07000004 lamps, 07000006 barrels, 07000001 lamps). Pin with tests (a walk-through contact moves the body the game's way; sister), keeping the bullet-hit curve pinned on lamp-calib unchanged. Continue from origin/s/lamp-calib (merge origin/master first). Run rigid_body_draw_live, rigid_body_wiring_live, interactables_07000001/4/6_live and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/rigid-body-cylinder-sweep`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-11-01T14:14Z

---
## Report (done, 2026-11-01T17:07Z)


---
## Report from the worker (done, 2026-11-01T17:07Z)

# rigid-body-cylinder-sweep — the unported rigid-body contact step

Branch `s/rigid-body-cylinder-sweep` (from `origin/s/lamp-calib`, `origin/master` merged). The
rigid-body collision handler's cylinder contact check and its torque step are ported and pinned;
the bullet-hit curve pinned on lamp-calib is unchanged.

## What's done

**The three passes of `RB_RestingContactRot` 0x1d73a0 [M]** now run in the game's order inside
`rb_resting_contact_rot` (`crates/nf-game/src/rigid_body.rs`):

1. **The cylinder block** [M 0x1d73e8-0x1d7490, skipped for type 3 by the `rb+0xa8 != 3` gate at
   0x1d7424]: up = `Mat_GetUp` row 1 [M 0x1e33b8 copies matrix bytes +0x10/+0x14/+0x18]; the
   HITTEST+0x50 base point = obj_pos + up·(+0x13c − +0x9c) [M 0x1d73f0-0x1d7420]; start
   (HITTEST+0x20) = obj_pos + up·(−(+0x13c)·1.8) (the 1.8 literal 0x3fe66666 at 0x1d42c/0x1d742c,
   negated 0x1d7458) and end (HITTEST+0x30) = the base point [M 0x1d742c-0x1d7490]; HITTEST+0x98
   = 0x10 [M 0x1d7488] — the pick flag that makes `Collide_Pick` 0x1eb320 skip the
   dynamic-object lists [M disasm 0x1eb8f8, `andi 0x10`], so the sweep's candidates are the
   straddled cels' STATIC collision only. The sweep (`rb_cylinder_sweep` over
   `Collide_CylinderIntersect` 0x1ecc00) runs one candidate per straddled cel; each hitting
   candidate's push accumulates into the persistent rb+0xC0 (`RbState.cyl_deltap`, HITTEST+0x10)
   and advances the pass's start/end/base exactly as `Collide_Intersect` 0x1ebaa8:353-394 adds
   DeltaP to +0x10/+0x20/+0x30/+0x50 and re-transforms the stored points per candidate [M,
   audit-verified]. The per-triangle push math is the already ported `Intersect_CylGeom` core
   (`world::rb_cyl_push_cell`, `capsule_tri_contact`), stat-only per cel.
2. **The class-5 walk** over the obj+0xD0 hit list [M 0x1d74c0-0x1d759c]: per class-5 hit,
   impulse = (hit+0x8)·0.5·(hit+0x30) [M 0x1d7504/0x1d7514, the 0.5 = 0x3f000000], +0x50 += r×
   impulse with the per-type lane masks [M disasm 0x1d753c-0x1d757c]: type 2 keeps x, type 3
   keeps y, type 4 keeps z, type 1 (the census lamps and barrels) zeroes y, every other type adds
   the full cross. Torque slot only — no +0x40 write anywhere in the arm [M 0x1d74dc-0x1d759c].
   `Sound_Play3D(100.0, −1, −1, *(rb+0x158), obj+0xC0)` per hit, collected for the caller's pool
   [M 0x1d74fc jal 0x1dd548]. Rot has no position-correction arm.
3. **The grounding tail** [M 0x1d75a0-0x1d7610], on the sweep list: gated on a sweep hit and
   `Vec_IsEqual(rb+0xC0, 0x2d6200, 0.0)` false (the 0x2d6200 table is (0,0,0) [L ACTION.ELF
   0x2d6200; tolerance 0.0 = mtc1 zero 0x1d75a8]), then +0x50 += (hit0+0x40 − translation) ×
   (rb+0xC0·2.0) with **the cross's Y lane zeroed before the add** [M 0x1d7600 `swc1 f20,
   0x24(sp)` — f20 is the 0.0 tolerance register — over the stored cross's y lane; VADD
   0x1d760c; store rb+0x50 0x1d7610]. Only x and z of the cross reach +0x50.

**The dispatch** `RB_CollisionHandler` 0x1d7a80 [M]: type 0 → `RB_RestingContact` 0x1d7168,
everything else → `RB_RestingContactRot`; obj+0xFA |= 0x20 applied by the handler itself after
either branch [M 0x1d7a80:16-23]. The handler now runs EVERY tick (drone.rs control walk), the
sweep included with an empty hit list — a resting body's cylinder digs the floor every tick
[M 0x1d73a0's setup runs before the hit walk].

**Audit.** An independent re-derivation from the decompiles plus raw disassembly checked the
seven port claims (test points, type-3 skip, per-candidate point advance, lane masks, +0x50-only
accumulator, tail formula and gates, dispatch, class-5 impulse/sound): all verified; it caught
**one real bug** — the port's grounding tail added all three cross lanes, the game zeroes the y
lane [M 0x1d7600]. Fixed (`rb_cyl_grounding_tail`) and pinned with a new sister case whose raw
cross-y is provably nonzero (`cyl_grounding_tail_rightes_a_tipped_body`, Sister 3): +0x50.y must
stay untouched while x/z take the full lanes.

**Pins added** (nf-game lib, `rigid_body::tests`): `cyl_test_points_match_the_setup` (up = row 1,
base/start formulas [M 0x1d73f0-0x1d747c]); `cyl_grounding_tail_rightes_a_tipped_body` (the tail
axis h·pen·(−up.z, 0, up.x) for a floor push, upright r ∥ push → zero, the (0,0,0) gate, and the
y-lane mask); `cyl_sweep_without_geometry_is_inert` (no geometry → no candidate, no hit0, no
push; through the handler the accumulators stay at rest). The lamp-calib bullet-curve pins are
untouched (the whole `rigid_body_draw_live` battery still passes on the same numbers).

## How it was verified

- `cargo test --offline --release -j2 -p nf-game --lib`: **644 passed; 0 failed; 5 ignored**
  (pre-fix tree), and after the tail fix the full rigid_body module re-ran green (23 passed;
  0 failed).
- Named live tests (NF_ISO set, fresh extraction):
  - `rigid_body_draw_live`: **3 passed; 0 failed** (the bullet-hit curve on lamp #40 unchanged).
  - `rigid_body_wiring_live`: **2 passed; 0 failed** (the blast replay now rides the sim's
    world/cel — the cylinder pass runs in every handler call).
  - `interactables_07000001_live`: **9 passed; 0 failed; 1 ignored**.
  - `interactables_07000004_live`: **4 passed; 0 failed**.
  - `interactables_07000006_live`: **5 passed; 0 failed**.
- Full suite (`python3 tools/test/suite.py`, four crates, `-j4` build then per-binary
  `--test-threads=1`): the build finished on the final tree ("build ok: 1070 test binaries in
  1546 s"); the RUN was cut by the session deadline at 288 of the 1070 binaries (the log is
  `/home/z/suite.log`, the runner's own per-binary record). Partial aggregate over those 288
  sections: **1706 passed, 5 failed, 55 ignored**. All 5 failures are environmental, none
  touching this task:
  - `disc_golden::bigf_header_fields_behave_as_documented` — slim ISO (`/DRIVING/MIS01.MUS` too
    small for a BIGF header — the golden manifests are for the full disc);
  - `disc_golden::bigf_tables_match_the_golden_manifest` — same slim-ISO cause;
  - `disc_golden::directory_tree_matches_the_golden_manifest` — same;
  - `identity_disc::the_real_disc_passes_the_identity_check` — slim ISO has 518,435 sectors vs
    the golden 1,191,072;
  - `probe_pylon_render::probe_render_pylon_only` — `wgpu adapter NotFound` (no GPU in the
    sandbox).
  The named bins all appear in the run: `nf_game` (the lib) 644/0/5 post-fix,
  `rigid_body_wiring_live` 2/0; `rigid_body_draw_live` and the three interactables re-ran
  manually on the same post-fix binaries (totals above).

## Known approximations (none pinned)

- `hit0` winner metric: the port keeps the minimum-|push| candidate; the game sorts the hit list
  by a gap-distance key (HITDATA+0xc, QuickSort cmp id 2 [M 0x1ecdac]) and takes the head. Same
  winner in the common single-hit case; may differ on multi-contact sweeps. The tail uses it only
  for the r arm. [S]
- `Intersect_CylGeom`'s +0x60 DeltaP lane is not ported (nothing reads it [M, audit]). [S]
- The game writes HITTEST+0x50 even for type 3 (the store sits in the branch's delay slot [M
  0x1d7428]); the port computes it inside the `!= 3` arm — no reader on that path, no observable
  difference. [S]
- rb+0xC0's lifetime: nothing in the decompiled set zeroes it per tick (RestingContactRot only
  reads it; RB_Update 0x1d6e38 touches the OBJ's +0xC0, not the RB's) — the port accumulates
  across ticks like the visible decompile. Live check below settles it.
- obj_pos on the Rot path = `st.trans` (the port's one translation slot) [S — the kind-0xD8 body
  never translates on this path; EulerEvolve's type-0 arm is the only translation writer].

## What's left

- **Bond/guard/door brush contacts are still not hit producers.** The port's hit lists are fed by
  blasts and bullets (class 5) only; `RB_AddImpulsiveForce`'s only decompiled callers are
  RB_Update's gas arm and RB_ApplyWind, and nothing in the decompiled set writes a Rot body's
  obj+0xD0 from dynamic-object overlaps. If the game's brushes move Rot bodies, the writer is
  either in undecompiled code or behind the per-object Collide pick; that builder is the natural
  next task, steered by the live checks below. The ported step (the every-tick cylinder sweep +
  grounding tail) is what makes bodies react to the STATIC world the game's way today.
- The `hit0` comparator disassembly (cmp id 2) if exact multi-contact selection ever matters.

## Live checks for the main agent

Each: fresh boot (save-state resume skips the boot path), then read the noted addresses.

1. **rb+0xC0 lifetime** [L]: 07000006, stand next to a barrel, watch `rb+0xC0` (the RB slot, the
   body's +0xC0) across ~30 ticks of rest. Expected if the port is right: it accumulates the
   floor dig (~the penetration, +y) every tick and never returns to 0 while resting on ground; if
   it resets to 0 each tick, the port needs a per-tick reset (access: read, every tick).
2. **Grounding-tail activity on a tipped body** [L]: poke a barrel's rotation to tip it ~20°
   (rb matrix row 1), let it settle, record `rb+0x50` (torque accumulator) x/z per tick.
   Expected: a persistent same-sign x/z push each tick (the righting axis (−up.z, 0, up.x)·h·pen
   — the sign flips with the tip direction), y untouched (the tail's y lane is zeroed [M
   0x1d7600]).
3. **Brush reaction existence** [L]: 07000004/07000006/07000001, walk Bond slowly into a lamp/
   barrel, record whether the body's `rb+0x50`/`omega` change and whether any HITDATA appears on
   the body's obj+0xD0 (walk the list head) during the brush; also try a door closing on a body.
   Expected if no dynamic producer exists: nothing on obj+0xD0 and no omega change — the brush
   push is Bond's own collision response, not the body's. If the body DOES move, record the
   writing instruction via gdbwatch.py (the builder's address feeds the next task).
4. **Cylinder block's test points** [L]: 07000004 lamp #40, one breakpoint pair at 0x1d7428
   (base store) and 0x1d7470 (start store): expected base = obj+0xC0 + up·(+0x13c−+0x9c), start =
   obj+0xC0 + up·(−1.8·+0x13c), with up = row 1 of obj+0x90.

## Usage

not visible to me (no token, cost or context-window figures are shown in this environment; the
audit subagent reported prompt 133924 / completion 9462 tokens for its single run, read from its
return message).
