## Your task: `level-4a-ride-bond-float`, Bond stands on the lift in the ride cut-scene

In Phoenix Fire Part 3's (0700004a) lift-ride landing cut-scene (pack 0x060006cf, SP #289 base (0, -53.53, -3.38)) Bond's body floats above where he should stand. `s/level-4a-ride-cast-b` (on master) decoded the anim-object placement (the root track seeds obj+0x30 [M Script_AnimStart 0x1d8f40:69,94; AnimFrameResolve 0x1c35f8]; no base compose in the per-tick anim stream [M Script_Interp 0x1d9a08 case 2]) and pinned the car-local roots [D 0x040005fd root (2.35, 0.85, -0.64)], but the compose that puts the cast in the car is still open. Find the Bond actor's stream, its sequence roots [D] and the game's rule that places him [M], and port it scoped to what the game does so Bond stands on the deck; the 07000014 scenes stay unchanged (a global SP-base compose broke them once).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-ride-bond-float`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-27T15:52Z

---
## Report (done, 2026-10-27T18:44Z)

# level-4a-ride-bond-float — the float's compose site found; the world-alone rule landed scoped to the ride pack

## What was asked

In Phoenix Fire Part 3's (0700004a) lift-ride landing cut-scene (pack 0x060006cf, SP #289 base
(0, -53.53, -3.38)) Bond's body floats above where he should stand. Find the Bond actor's stream,
its sequence roots [D] and the game's placement rule [M], port it scoped so the 07000014 scenes
stay unchanged.

## The decode [M — decompile + mipsdis over work/elf/ACTION.ELF, all re-verified this session]

1. **The script base si+0x00 is the SP placement at load**: `Script_Load` 0x1d8248 builds it with
   `View_RotTransMatrix(param_3, param_2, si)` at :148-149. The port's `Script::load` already
   seeds `self.matrix` the same way.
2. **The camera op (0x0d) overwrites si+0x00 with the identity, unconditionally and forever**:
   `Script_CameraStart` 0x1d9140 calls `Mat_IdentityT(si)` — the disasm shows a0 untouched from
   entry to the jal at **0x1d916c** (before the flag-0x20 check) — and `Script_CameraEnd`
   0x1db348 never restores it (it only kills the stream and clears +0xab5 bit 0x10).
3. **An unparented case-1 stream composes si+0x00 every tick**: `Script_GetInterp`'s 10th
   argument is `(stream.parent == -1)` — `lb v1,37(s1)`, `nor`+`sltiu`, `sw v1,0(sp)` at
   **0x1d9a7c-0x1d9a98** — and GetInterp's tail is `if (param_10) Concat(si, out)` [M
   0x1da158:195-197]. So the compose's VALUE rides the lifecycle: the placement before the
   camera op, the identity after it.
4. **Parented streams compose si once at the chain top**: the chain-walk GetInterp calls pass 0
   [M 0x1d9b30 `sw zero,0(sp)`] and `Script_Interp` concats si explicitly when the walk reaches
   an ancestor with parent == -1 [M 0x1d9b9c]. The port's flat per-stream `mul(&m, &self.matrix)`
   for parented streams is NOT camera-gated yet — scoped follow-up (no pinned scene is affected).
5. **The anim cast composes nothing**: `Script_Interp` case 2 (0x1d9a08:140-199, re-read in full)
   advances the frame, calls `AnimObjectUpdate`, flags 0x20, footsteps — no GetInterp, no
   matrix. `Script_AnimStart` 0x1d8f40 creates the object at the origin, seeds obj+0x30 from the
   root track, and its own GetInterp output stays unread [M disasm 0x1d90f0-0x1d911c]. The cast
   draws on its raw root track in every reading.

## The pack's data [D — the opt-in probes, this branch]

- `ride_pack_raw_probe` (nf-formats): pack 0x060006cf, camera true, length 575, 28 streams.
  **s0's first commands are `0d 00 00 00 7d 00` — the camera op at t0**, before every entity/anim
  stream keys (s1/s2/s3 anims at keys 125-130, s7 satchel 133..594, s13 floor 1405..1408,
  s25 marker 1409, s26 = event 6 PlayerEnable, s27 = event 0xD ch 0x22).
- `ride_pack_keys_probe`: the raw keys — the floor s13 t0 (0, 0.990, 0) → t575 (0, −5.334, 0);
  the satchel s7 t0 (3.429, 1.488, −0.836) → t575 (3.226, −0.776, −2.002); the camera s0
  t0 (2.889, −7.829, −4.583) → t575 (2.094, −9.921, −2.073); the marker key 1409 at t575
  (1.799, −54.128, −3.453) — the raw world hand-back the [L] pin sits on.
- `ride_cast_roots_probe` (nf-view): the three cast scripts' body windows — guard 0x040005fd
  root (2.353, 0.852, −0.644) → (2.497, 0.598, −0.945); **Bond 0x040005fe root (2.389, −6.949,
  −2.530) → (2.010, −13.455, −3.243) — the rope descend, ~8 below the floor the whole scene**;
  officer 0x040005ff (3.379, 1.153, −0.791) → (3.292, 0.550, −0.842). The facial windows carry
  zero roots. The whole stage (camera, floor, satchel, cast) shares ONE raw frame.
- `ride_cab_keys_probe`: the lift script 0x06000571 — **camera false** (no camera op, so the
  placement compose stays there), 6 streams, the cab entity 0x020005b4 keys (0,0,0) →
  (0,−150,0) over t0..t5040 — local offsets. The ride itself is untouched by this task's rule.
- `ride08_cam_probe`: 07000008's only NIS is 0x06000077 — camera op in s0, entity/anim streams
  s1..s17 — the surface that exposed the scoping below.

## The fix

`crates/nf-game/src/nis.rs` interp case 1: `keys_alone = camera_started && parent < 0` scoped to
the scenes it is proven in — the ride pack **0x060006cf** plus master's existing arms (the
0x0600062f kind-0x31 case, and `ehash != 0 && ekind == 0xff` unparented). Under the pack arm the
port no longer composes the SP base into the pack's post-camera entities, so the floor lands at
the raw keyed 0.99 → −5.334 **under a cast that was always drawn keys-alone** — before, the
composed floor sat at −52.5..−58.9 while the cast stayed at raw y ≈ 0.85, i.e. Bond floated
~53 units above the port's own floor (the symptom). The marker class (hash-0 kind-0x19) keeps
its compose skip, so the [L]-pinned hand-back at the raw key (1.7994, −3.4534) is untouched.

**The scoping is honest about what is not proven**: the general rule (every unparented
post-camera stream keys-alone) broke 07000008's gondola — Bond nav-deadlocks at the gondola
mouth (143.8, 46.35, −106.85; the descent to 112, 27.4, −122 never opens) in
`walkreal_07000008_live`'s two walks; with 0x06000077 excluded from the rule the same tests go
green in 28.7 s. The decompile's case-1 path has no per-hash branch [M 0x1d8968:35-173], so the
game-side discriminator (the gondola script's stream parents, EntityEnd times vs the op tick, or
a loop's si reseed) is a named open item, marked `[S] unverified` in the code comment. Nothing
else changed vs master: every non-pack script keeps master's exact compose behavior.

## What was verified (all through the real input path, NF_SILENT=1, --test-threads=1)

- `ride_cutscene_objects_live` — extended with the world-alone pins: the floor mid-scene y within
  the raw key span [−5.334, 0.990], the satchel within [−0.776, 1.488], and the cast roots on the
  raw tracks (guard ≈ 0.72±0.4, Bond within −6.949..−13.455 — measured −10.25, the rope —
  officer within 1.153..0.550). **1 passed / 0 failed.**
- `fix_landing_matrix_live` (the [L] raw-marker hand-back) — 1 passed / 0 failed.
- `walkreal_0700004a_live`, `lifts_ride_live` (2), `interact_0700004a_live` (5) — all passed.
- `walkreal_07000008_live` — 4 passed / 0 failed (green again under the scoped rule).
- `walkreal_07000014_live` (2), `exit_aim_01_live`, `cutscene_guard_models_live` (3),
  `cutscene_models_live` (4), `fix_safe08_live` (5), `walkreal_07000005_live` (8),
  `interact_07000001_live` (10) — all passed.
- The fast runner (tools/test/suite.py, daemonized): **build ok: 868 test binaries in 753 s** on
  the branch merged with origin/master (08f06953). The RUN phase did not reach a total line
  inside this sandbox window (it was stopped at ~17:54 for the 07000008 A/B above); the totals
  below are the targeted runs. **Suite total line: not produced this window** — the final tree
  differs from the built one only by the scoped `keys_alone` expression (one line + comments),
  and every compose-sensitive binary in the suite was run individually as listed above.

## What's left

- **The live check that settles the scoping** (below) — it turns the `[S]` carve-out into the
  game's real rule, or replaces it.
- The parented-stream chain-top compose is not camera-gated in the port [M 0x1d9b9c] — no
  pinned scene is affected; land it with the discriminator.
- The guard's drawn-y diagnostic in `ride_cutscene_objects_live` still prints 0.56 (raw) while
  the recorded live sighting is "the guard in the car, deck ≈ −52.7" — if the live check rules
  the cast compose rides the SP base (not si), that is the next port step.
- The suite total line on the final tree (the runner record above).

## Live checks for the main agent

1. **07000008 intro script 0x06000077, the gondola entity stream** (find the stream whose keys
   carry the carriage; entity glist from the decode): during the intro, read the entity's
   obj+0x90 translation per tick across the camera-op tick and the EntityEnd tick. Expected: if
   the matrix keeps the placement offset AFTER the camera op (composed) while an unparented
   stream interp is still running, the game's compose outlives the op for this script — hook the
   writer and name the function; if instead the stream has a non-(-1) parent byte or ends before
   the op, the port's scoping condition should carry that same shape instead of the hash.
2. **0700004a pack 0x060006cf, stream s13 (NIS_lift 0x020006ee)**: read obj+0x90's translation
   at t≈300 of the scene. Expected under this task's reading: ≈ the raw key interp
   (0, ~−1.7, 0) — NOT (0, ~−55.2, 0). A read of ≈ −55 would refute the pack arm of the rule
   and mean the float's real fix is the cast compose instead.

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
