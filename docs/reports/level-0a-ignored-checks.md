# level-0a-ignored-checks — report

Branch `s/level-0a-ignored-checks` (from `origin/master` 81da6c47). Task: the two `#[ignore]`d
tests in `walkreal_0700000a_live` (the doorway walking height, the corner crossing) — read their
notes and the merged capsule work, find what the game does at each spot [D collision; M the
capsule contact rules — head-on cull on the raw denom 0x10a1f0-0x10a2e8, Dist2Tri flags 0/99
0x10a330, Player_FeetOnPoint 0x194188], fix the port gap with evidence or re-derive the
expectation where it was port-derived, un-ignore what can be un-ignored, name the live check for
anything left.

## What's done

1. **The decompile read of both spots (new evidence, all cited in the test files).**
   - The walk path has **no clip, no wall-stop and no step-up**: `Player_Update` integrates the
     walk move unclipped into the root and then calls `Player_Collision` [M 0x193600:284-295,
     361]; the only world queries on a walked line are the per-cel `Collide_Intersect` cyl pass
     [M 0x1ea398:84-94], the FeetOnPoint ray [M 0x1949a8:76-77] and the jump/crouch LOS gates.
   - **The push tail is dead on 0a fresh boot.** The tail is gated on `|closest − HITTEST+0x50| <
     radius` [M 0x109da0:419-457] with HITTEST+0x50 the lifetime DeltaP accumulator (only writer
     = the per-cel add-back [M 0x1ebaa8:353-373]; `_DeltaP` zeroed per call [M 0x109da0:136]).
     [L] `ee_start_0700000a` 0x1b78ee0 (BL = *(obj+0xD4) − 0x740 = 0x1b78750): BL+0x790 = 00×16 —
     the ref boots (0,0,0), every walked contact sits |closest| ≈ 10-25 from the origin ≫ r 0.55,
     so the tail never fires, `pos += BL+0x750` adds zero [M 0x1949a8:131-140] and nothing
     produces the press-y on a fresh boot.
   - The **head-on cull on the raw denom** [M 0x109da0:302-322] is arithmetically inert for the
     walk capsule: the raw spine denom is −0.7597 [L BL+0x780 = 0xbf427ceb] and the cull needs
     |denom| > 0.999. Dist2Tri flags: 99 = query behind the plane (skip), 0 = projection inside
     the tri [M 0x109770:42-64, caller 0x109da0:331,419].
   - The **FeetOnPoint band** [M 0x194188:88-109] is the only vertical mechanism (ray from B down
     cc+0.6, hit counts when `hitDist − 0.1 ≤ cc`, grounded latch, no snap) — docs/systems/
     grounded-band.md mirrors it bit-exactly.
2. **A real port gap found and fixed**: `NF_WALK_CLIP`'s `move_v` never spanned the walk move —
   `pre_move_pos` was captured *after* the walk add (its comment claimed "before the
   integration"), so `move_v = (0, gravity, 0)` on every walked tick and the clip never engaged.
   The fix hoists the capture before every position move (zero-g/swim/walk), matching the game's
   order — contacts read the post-move root [M 0x193600:284-295]. Default path is untouched
   (`move_v` only reads under the flag); the named binaries stayed green.
3. **The 2026-10-30 corner-note premise was wrong and is corrected in the test file.** The
   "corner CROSSES at +76 ticks with NF_WALK_CLIP=1" reading ran on the dead flag path — the
   "crossing" was wall-response-less ghosting. Re-measured with the capture fixed: the flag now
   **wedged the west-hall through-line at the doorway** (walker STUCK at (−8.28, 1.03, 15.83), y
   decaying 1.03→0.71) and the 0a battery reads 1 passed / 6 failed / 2 ignored under it. The
   clip model does not reproduce the game's doorway slide even under its own flag; the flip stays
   unlanded (the earlier flip-2 branch's 21 test regressions + 2 timeouts agree).
4. **Neither test is un-ignorable, and the notes say exactly why now.** Both assert driver-pinned
   settled [L] values (glide 1.1285 ± 0.003, corner exit 1.1159 ± 0.005) that need the unported
   +0.0256 climb settle. On a fresh boot the press-y cannot come from CylGeom (dead tail, above);
   the remaining y writers on a walked line are the FeetOnPoint band [M 0x194188:106] and the
   walk-anim root ride (cc read fresh per tick [M 0x194df0:86]; the port's SP anim set is the
   2-frame stub [D 0x2b91b0]). Re-deriving the expectations from port behaviour would contradict
   the driver's pinned-values rule, so nothing was re-derived.
   - **Named live checks for the main agent** (under "Live checks for the main agent" rules):
     1. Watch BL+0x790 (HITTEST+0x50) per frame through the doorway brush and the corner —
        expected: stays (0,0,0) on 0a (pins the dead tail; access R, address = *(obj+0xD4) −
        0x740 + 0x790, obj+0xD4 @0x1b78714 reads 0x01b78e90 in ee_start).
     2. Trace the walk-anim root motion into AnimObject+0xcc during the walked west-hall glide —
        expected: +0.004/tick growth to peak ≈1.1465 then band-catch descent to 1.1285 (the
        +0.0256 candidate; access RW on AnimObject+0xcc of the player object).

## How it was verified

- Named binaries on the post-fix tree (suite state, release, --test-threads=1):
  - `interactables_0700000a_live`: **7 passed / 0 failed / 0 ignored**
  - `walkreal_0700000a_live`: **7 passed / 0 failed / 2 ignored**
  - `nf_game`: **588 passed / 0 failed / 5 ignored**
- Full suite (`tools/test/suite.py` equivalents — the sandbox kills true background jobs, so the
  chunked foreground runner `tools/test/chunk_par_tmp.py` was used with 2 workers and a fresh
  state file; same build + per-binary runs as suite.py): **332/996 binaries recorded by the
  22:45 UTC stop line = 1572 passed / 0 failed / 99 ignored, zero failures.** The remaining 664
  binaries were NOT RUN before the deadline (see What's left). No FAILED/TIMEOUT lines.
- The flag-on re-measure (documented above) — the negative result that corrected the note.

## What's left

- The remaining 664 suite binaries (the tree is final; a follow-up can resume the tally from the
  recorded state — the 332 done include all priority bins).
- The two tests stay `#[ignore]`d pending the two live checks above; if live check 2 confirms the
  anim-root ride, the climb settle becomes portable and both tests can un-ignore.
- The open decompile question (not blocking): disasm of 0x10a344-0x10a61c to settle whether the
  second Dist2Tri operand is the re-sampled spine point Q or HITTEST+0x50 (the decomp text and
  the port's disasm reading disagree; behaviourally identical on 0a while the tail is dead).

## Usage

`not visible to me` — this session shows no token/credit/context figures for the main agent. The
only figures visible anywhere were the two research subagents' final reports (read in this
conversation): subagent 1 "Tokens -> prompt:214113 completion:10862 total:224975"; subagent 2
"Tokens -> prompt:127655 completion:9346 total:137001".
