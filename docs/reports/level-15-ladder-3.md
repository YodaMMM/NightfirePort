# level-15-ladder-3 — 07000015 ladder #382: settle whether the game's pick ever admits it

**Done.** `Collide_Pick` 0x1eb320 was read in full (C + disassembly): the branch `s/level-15-ladder-2`
reading is right and the master reading misquoted the decompile. The unlinked ladder is never a pick
candidate — an empty straddle set admits nothing. Master's ignored route test was replaced by the
branch's pin + sister (both green); the census row and the test doc now carry the corrected chain.

## The verdict, with the lines

`Collide_Pick` 0x1eb320 [M], candidates, in order:

1. **:125** `iStack_cc = Collide_StraddleCels(param_1, auStack_f0)` — the straddle set and its count.
2. **:128** `if (auStack_f0[0] != 0) { ... }` — the whole straddled-cel loop is gated on a NON-ZERO
   count. Disassembly: `0x1eb410 lhu $v0, 0xb0($sp)` (the count) / **`0x1eb414 beql $v0, $zero,
   0x1eba38`** — an EMPTY set branches STRAIGHT to LAB_001eba38. Nothing else runs.
3. Inside the loop the candidates are the straddled cels' object chains only: **:134**
   `iVar12 = *(int *)(*(int *)(iStack_c0 + iStack_cc) + 0x18)` (cel+0x18 inline chain; disasm
   `0x1eb438 lw $s3, ($v0)` / `0x1eb444 lw $s3, 0x18($s3)`) and the cel+8 `Collide_PickObj` walk
   (:313-328, disasm `0x1eb8e8` ff., `jal Collide_PickObj` 0x1eae80 per object).
4. **LAB_001eba38 (:369-375)** — the ONLY other source: the ForcedList. Disasm `0x1eba44 lui $v0,
   0x27` / `0x1eba4c lw $s0, 0x590($v0)` — head = *(0x270590), `Collide_PickObj` per node, walk +4.

So master's claim — "the pick admits the unlinked ladder only from an empty straddle set [M
Collide_Pick 0x1eb320:125-134]" — quotes those lines backwards: a NON-empty straddle set is the
requirement for the cel-chain candidates, and the empty set falls straight through to the
ForcedList, admitting nothing by itself.

The ladder is in neither candidate source:

- `Ladder_Create` 0x1966c0:20-23 sets class 0x2d, calls `build_LinkToRoom(obj, 0, glb_world)` then
  `Controls_StraddleTest(obj)`.
- `build_LinkToRoom` 0x1c9680:45-59 links ONLY on a `build_FindCel` hit (the link, the channel
  bytes and the StraddleTest are all inside `if (lVar3 != 0)`); the miss path returns 0 with
  obj+0x20 = 0 [M control_create_object 0x1344e0 memset] and adds no ForcedList entry. The
  ForcedList arm (:63-67) needs the obj+0xf0 bit 0x20000000, which a ladder never has.
- `build_FindCel` 0x1c8cb0:103-119 keeps room cels whose entity-params box holds the point; the
  ladder's world-sphere centre (84.529, 31.733, −2.288) is in no box [D the 07000015 entity
  params: the east room's box is y 39.28..45.14, every shaft cel's box ends at x 81.87] — and the
  port's own load already shows `cell == None` on all three ladders [L the port sim].
- `control_handle_cel_change` 0x133288:63-67 retries `build_LinkToRoom` for a cel-less object, but
  on the same centre — the same miss forever (class 0x2d ≠ 2, so not even the Drone_FellOutMap arm
  runs). No later relink can ever put the ladder on a chain.
- `Controls_StraddleTest` 0x134778: BFS start = **obj+0x20** [M disasm `0x1347b8 lw $a2, 0x20($s0)`,
  delay slot `0x1347d0 sw $a2, 0x80($sp)` — the hittest's start-cel field] = 0, so
  `Collide_StraddleCels` 0x1eac98 skips its BFS (0x1eac98:19 `iVar2 != 0` guard) and appends just
  the start cel (count 1, :82-84); count < 2 [disasm `0x1347d8 sltiu`] takes the clear arm
  (`0x13482c-0x134860`: `sw $zero,($s0)` / `sw $zero,4($s0)` / `0xf0 &= ~0x10008000`) — NOT added
  to the ForcedList 0x270590. The ≥2 arm's `LList_Add(0x270590, obj)` (0x134804) never runs.

`Collide_PickObj` therefore never runs for the ladder → `Player_DealWithObjHit` 0x194458 never sees
class 0x2d → `Player_CollLadder` 0x196118 can never fire. Same for #900 and #901 [D the same
boxes]: **no ladder on 07000015 can be grabbed from any spot, standing or mid-fall.**

## The test change

`crates/nf-view/tests/interactables_07000015_live.rs`: master's ignored `the_ladder_takes_the_climb`
(a real-input fall+strafe route test whose premise contradicted the admission chain above — the
TEST was wrong, per the task rules) is replaced by s/level-15-ladder-2's pair, re-landed on today's
master with the corrected chain in the doc:

- `the_dead_ladder_never_grabs_at_point_blank` — pins the premise (find_cel None at all three
  ladder centres; every loaded ladder unlinked), then a real-input walk (forward ticks only) WEST
  across the ladder plane's x 84.529 on the south walkway with the grab-favourable facing
  (|AngleDiff| = π > π/2 [M 0x196118:20]): the climb NEVER engages while Bond's pre-push capsule
  crosses the plane's reach (min pre-collide x < 84.6).
- `sister_the_dead_ladder_mesh_reaches_the_point_blank_stander` (can fail) — at the point-blank
  stand the ladder's own pick WOULD hit (`Ladder::capsule_hits_default`): the blocker is the
  admission chain, not the geometry. Any relink, box change, ForcedList port or pick-math change
  flips these.

Supporting: `CollisionWorld::debug_scope_cells` (a debug introspection accessor alongside
`debug_straddle`/`debug_bucket_owners`) and the `docs/levels/07000015.md` §7 ladder row correction
(the old "one climbed here" was wrong — all three are admission-dead).

## How it was verified

- `cargo test --offline --release -p nf-view --test interactables_07000015_live -- --test-threads=1`
  — **11 passed, 0 failed** (twice, including the two new tests).
- The four 07000015 nf-view binaries on the final tree (direct per-binary runs, suite semantics):
  interactables_07000015_live 11/0, census_07000015 (2 pre-existing ignores), walk_07000015_live
  2/0, walkreal_07000015_live 1/0 — all green.
- The full suite (tools/test/suite.py semantics, per binary: same env NF_SILENT=1 +
  CARGO_MANIFEST_DIR, `--test-threads=1`, 600 s limit), run as FOREGROUND CHUNKS because this
  sandbox kills background jobs between tool calls (three nohup/setsid starts all died within
  seconds; tools/test/suite.py's documented background start is not survivable here):
  - **nf-game: 85 binaries — 881 passed, 0 failed, 6 ignored.**
  - **nf-formats: — 4 failed**: the 3 documented slim-ISO `disc_golden` allowances PLUS
    `identity_disc::the_real_disc_passes_the_identity_check` — the same environmental root cause,
    not a regression: it asserts `info.sectors == 1_191_072` (the full 2.4 GB disc); this sandbox's
    slim ISO is 1,061,754,880 bytes. nf-formats is untouched by this branch.
  - **nf-audio: green** (included in the totals below).
  - **nf-view: PARTIAL — 275 of the 769 total suite binaries ran** (nf-view alone has ~500), all
    green except the nf-formats-disc-dependent ones counted above. The rest did not fit the
    session: the target dir filled the 9.9 GB rootfs mid-build (`No space left on device (os error
    28)` at 100%), and after reclaiming space the 2-core box rebuilt the third-party stack (the
    stale-artifact prune was forced by the disk-full corruption) but the remaining ~494 binaries
    need more wall time than the sandbox had.
  - Cumulative over the run binaries: **1705 passed, 4 failed (all four the slim-ISO environmental
    cause above), 57 ignored.** No failure touches this branch's change (nf-game's full set is
    green; the changed test file is green).
- Every assertion above cites [M] decompile/disassembly addresses or [D] disc data; the port-side
  green runs are the live [L] evidence for the port matching the game.

## The PCSX2 read that would settle it (for the main agent)

The decompile settles the admission question; the reads below falsify/verify the chain end-to-end
on a FRESH boot of 07000015 (a save-state resume skips the create path):

1. Break on `Ladder_Create` 0x1966c0 (ACTION.ELF execution) — it returns 3 times (#382/#900/#901);
   capture `$v0` (the ladder object pointer P) after each return.
2. Read `P+0xFF` (u8): expect 0x2d [M 0x1966c0:20]. Read `P+0x20` (u32): expect **0** — the
   FindCel miss [M 0x1c9680 miss path]. Read `P+0xF0` (u32): expect bit **0x10008000 clear** — not
   forced [M 0x134778 count<2 arm]. Watch writes to `P+0x20` for the rest of the level: expect
   none [M 0x133288 retries the same miss].
3. The task's named read — a fresh-boot fall past x 84.53 off the south lip (the master route's
   approach), watching the player substate for the climb: expect the climb substate to NEVER
   engage for the whole fall (the pick cannot see the ladder), i.e. the fall ends on the death
   plate / in the void with no grab window.

Any non-zero `P+0x20` or a set 0x10008000 falsifies the chain and this test fix — please report it.

## What's left

- The remaining ~494 nf-view suite binaries (environment-limited, see above); they are untouched by
  this branch and green in their last full-suite state on master.
- Nothing else for the task.

## Usage

not visible to me — this session exposes no token, credit, cost or context-window figures.
