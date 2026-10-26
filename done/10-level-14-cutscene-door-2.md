## Your task: `level-14-cutscene-door-2`, the 14 cut-scene door leaves without changing other levels' scripts

Start from `origin/s/level-14-cutscene-door` (merge it, then origin/master). Its finding stands: script 0x06000833 streams 10/11 key the door leaves 0x02000891/0x02000892 [D], and the port never applied them to the placed door pair #116/#117. But the branch also carries the broad Script_CameraStart identity rule (the whole script base reset), which master deliberately does NOT have — master applies the identity to the CAMERA interp only (`nis.rs` case 4, branch cutscene-camera-base). Merged with master the branch turns these red (each green on master):
- walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact
- walkreal_07000008_live (the ally noise path; the gondola ride)
- zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond
- interact_07000001_live (the armour pickup + ch 13; the entry sliding door)

1. Keep master's camera-only rule; land only the door-leaf keying (the entity stream drives the placed door object's pose/state) with its [M] cite. If the leaves need a world-space base, derive it for THOSE entities only, from the decompile.
2. All of the above green plus the door test; run the full suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-door-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T22:57Z

---
## Report (done, 2026-10-26T00:53Z)


---
## Report from the worker (done, 2026-10-26T00:53Z)

Closed by the driver: the report is on `s/level-14-cutscene-door-2` (docs/reports/level-14-cutscene-door-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-26T00:53Z)

# level-14-cutscene-door-2 — the door-leaf keying landed, the whole-script base reset did not

Branch `s/level-14-cutscene-door-2` = `origin/s/level-14-cutscene-door` (6acb90a1) merged with
`origin/master` (b312d959). Master's camera-only identity rule is kept; the branch's broad
`Script_CameraStart` whole-script base reset is removed; the door-leaf keying is re-landed on a
disassembly-proven, census-scoped rule. Window 23:09–01:08 UTC.

## 1. What was kept and what was dropped

- **Kept (master):** `Script_Interp` case 4's camera matrix is the keys alone (`nis.rs` case 4 —
  the camera's `Script_GetInterp` base-concat flag = 1 [M 0x1da054-0x1da068 disasm]; the base it
  concats is the identity the camera op installed). The `countdown_cutscene_cameras_live` pin
  stays as master has it.
- **Dropped (the branch):** `self.matrix = IDENT` in the op-0x0d handler — the whole-script base
  reset. On the merged tree it turned the four master-green live tests red
  (walkreal_07000001_live, walkreal_07000008_live, zoe_gondola_realpath_live,
  interact_07000001_live), exactly as the task stated. Reproduced here before the fix: with the
  reset in place, `interact_07000001_live` failed 2/10 (the entry sliding door reached state 4
  OPEN with no E press, and the armour/ch-13 traffic shifted) — the intro script
  0x06000087's entity streams (s5/s11 obj_type 0x31, s9 obj_type 0x19 [D decode]) drive
  volume/marker couplings whose composed spots the walks pin.

## 2. The game rule, from the disassembly

- `Script_CameraStart` 0x1d9140 opens with `Mat_IdentityT()` whose argument the decompile drops.
  The disassembly settles it: `daddu $s1, $a0, $zero` at 0x1d915c keeps a0 = param_1 (the
  SCRIPTINFO) and no instruction writes $a0 before the `jal 0x1e34c8` at 0x1d916c [M
  0x1d9140 disasm] — the game writes the identity over the script player's own base matrix
  si+0x00. `Script_CameraEnd` 0x1db348 kills the stream and clears +0xAB5 bit 4; it never
  restores si+0x00 [M 0x1db348:5-13] — the identity sticks for the script's lifetime.
- `Script_Interp` case 1's main `Script_GetInterp` call stores its 10th argument — the
  base-concat flag — as `(stream->parent == -1) ? 1 : 0` (`lb $v1, 0x25($s1)`; `nor`; `sltiu
  $v1, $v1, 1`; `sw $v1, 0x0($sp)` at 0x1d9a98 [M 0x1d9a08 disasm]; the decompile drops the
  slot). `Script_GetInterp`'s prologue reads it back from caller sp+0 (`lb $v0, 0xd0($sp)` after
  `addiu $sp, $sp, -0xd0`, re-stored to its o32 slot) and its tail runs
  `Concat(si+0x00, out)` when the flag is non-zero [M 0x1da158 disasm + decompile]. So an
  **unparented** entity stream composes the script base inside the key interp, and from the
  camera op on that base is the identity: its keys are the world matrix.
- 07000014's opening scene 0x06000833 keys its door leaves in world space: streams 10/11
  (obj_type 0xff, no parent, hashes 0x02000891/0x02000892 [D decode]) slide x 27.7915 → 27.0697 /
  27.7895 → 28.6815 at y 1.2965 z 49.5721 — the kind-0xdb placement pair #116/#117 at (27.788,
  1.203, 49.671) [D]. Composing SP #276's placement (52.1566, 2.6365, 46.6214 [D]) threw the
  leaves to (79.95, 3.93, 96.19) — 56 units off, the door never moved while Kiko walked through
  it (the branch's finding, re-confirmed).

## 3. The fix, scoped to the class the door belongs to

`crates/nf-game/src/nis.rs`:

- New `Script::camera_started` flag, set where the branch reset the base (op-0x0d, gated on
  `flags & 0x20 == 0` like the game). `self.matrix` is NOT touched — master's camera-only rule.
- Case 1's base compose now skips when `camera_started && hash != 0 && kind == 0xff &&
  parent < 0` — the default `Script_CreateEntity` 0x1d8968 arm's gfx-bound movers
  (`control_create_object` + `hashtable_set_object_to_entity_gfx` 0x1ca6d8 [M]), unparented, in
  a script whose camera op has run. That is the door-leaf class.
- **Why this scoping:** a whole-disc census (`crates/nf-view/examples/probe_cam_scripts.rs`,
  committed; scratch output not committed) walks every level archive and lists the entity
  streams of every script containing a camera op. The 07000014 door leaves are the ONLY
  non-zero-hash obj_type-0xff streams in camera-op scripts on the disc; every other 0xff stream
  in those scripts has hash 0 (07800008 0x06000077 s15, 07800011 0x06000655 s13/s16-18,
  07800013 0x06000646 s6, 0780004a 0x060006cf s6/s11) and every other stream in them is a
  special create arm (0x31, 0x19, 0xf0 …). The rule therefore shifts exactly one pair of
  streams on the whole disc; everything the live walks pin keeps master's compose by
  construction. The hash-0 and special-arm classes keep the compose (a port choice, flagged as
  such in the comment — 4a-lift-4's live evidence says hash-0 markers key world space in the
  game, but no test pins them and master composes; left alone).

## 4. Verification

Touched-crate runs, on the final tree (all green):

- `cutscene_door_07000014_live` — 2 passed (the leaves start closed at the [D] key spot, slide
  to the 1.6118 gap, back closed; the draw slots exist).
- `countdown_cutscene_cameras_live` — 2 passed (the camera keys-alone pin).
- `interact_07000001_live` — 10 passed (was 8+2 red under the wide rule).
- `zoe_gondola_realpath_live` — 1 passed.
- `walkreal_07000001_live` — 3 passed.
- `walkreal_07000008_live` — 4 passed.

Full suite on the merged final tree (`NF_ISO=… python3 tools/test/suite.py`, started 23:49 UTC,
build ok 809 binaries in 1030 s). The sandbox deadline (01:08 UTC) cut the run phase short — the
per-binary totals summed from the log at the last read before the finish steps:

- cut-off (00:53 UTC): 368/809 binaries done — **1223 passed, 1 failed, 74 ignored**, plus 2 CRASH (exit -9)
  `mp_stress_explosives_live` / `mp_online_failures` (the sandbox's OOM killer; the same two
  crashed the same way on the pre-fix tree in the level-14-cutscene-door session).
- The single FAILED binary is `probe_render_pylon_only` — the pre-existing master failure the
  previous session's suite run also recorded (with `fix_0c_pylon_live`), not a file this task
  touches.
- The touched binaries completed inside the suite green: `cutscene_door_07000014_live` (3 s),
  `interact_07000001_live` (25 s), `countdown_cutscene_cameras_live` (3 s). The four walk
  binaries that had not been reached alphabetically were run directly on the same tree above
  (all green).
- `fix_0c_pylon_live` had not been reached at the cut-off; its two known reds stand as
  pre-existing (master's own, recorded in both prior sessions' suites).

## 5. What's left

- The static-vs-mover swap: the game hides the static 0xdb pair while the script movers wear its
  gfx (the 0xE2 SS swap system, docs/levels/07000014.md §5) — still not ported (the branch
  report's §6 item, unchanged). The render check below decides whether it is visible.
- The parented-stream tail `Concat(si+0x00, ·)` also runs on the identity in the game [M
  0x1d9a08:133]; the port keeps the placement compose for parented streams (the live walks pin
  it). A deliberate, commented deviation — a future task with fresh [L] evidence can widen the
  rule along the same disassembly.

## 6. Headless renders for the main agent

The scene at Bond (40.87, 2.98, 40.85) (the cutscene-camera spot):

| frame | when | expect |
|---|---|---|
| t≈100 | shot A | the control room; the door pair #116/#117 at (27.79, 1.2, 49.67) closed, Kiko/actors behind it |
| t≈420 | mid-open | the leaves parting (leaf A −x, leaf B +x) in front of the actors |
| t≈540 | full open | the doorway open (gap ≈ 1.61), the actors visible through it |
| t≈1000 | closed again | the leaves back at the closed spot |

What to compare: the door visibly opening; whether doubled leaves show at the closed spot (the
static pair still draws — if yes, the SS swap is the follow-up).

## Live checks for the main agent

1. **The static pair's hide**: PCSX2 fresh boot, gdbwatch the kind-0xdb door object(s) at
   (27.79, 1.2, 49.67) through the scene (t≈250-540) — expected: their draw state/glist pointer
   changes when the script movers mount (the SS swap), or their mesh is the SAME object the
   movers carry (then nothing to hide).
2. **Leaf gfx identity**: fieldprobe the hashtable entry for 0x02000891/0x02000892 (t_hashtable
   lookup [M 0x1ca6d8]) — expected: a celglist whose matrix sits at the door spot; screenshot at
   t≈540 to compare with the port's render.
3. **The compose question for the special arms** (resolves the port-choice flag): fresh boot into
   07000001, gdbwatch the world matrix of the intro script 0x06000087's s5 object (hash
   0x020006b1, obj_type 0x31) around t1500 — expected under the port's choice: placement + keys
   (the composed spot); if the game shows the keys alone there, the case-1 rule can be widened to
   every unparented stream in camera-started scripts and the walks' pinned spots re-measured.

## Usage

not visible to me (this sandbox shows no token, credit, cost or context-window meters anywhere —
no environment variable, no status line, no transcript header carries them).

## ntfy

