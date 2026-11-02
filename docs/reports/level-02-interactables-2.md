# level-02-interactables-2 — the rigid-body kind lands in the 07000002 battery

Branch `s/level-02-interactables-2` (from origin/master 6042ee53). Task: census of 07000002
[D every placement kind], one live test per player-touchable kind driven through the real path
(walk/E/shoot) with sisters, port-gap fixes with evidence, and the run totals.

## What was done

1. **The census, re-run from the disc** (`interactables_07000002_probe.rs`, the assertion-free dump,
   opt-in NF_ISO): the player-touchable kind counts match docs/levels/07000002.md row for row —
   breakables 0x20 ×50, guards 0x0F ×25 (AI actors, the separate drones census), TouchOnce 0xEA ×24,
   **rigid bodies 0xD8 ×21**, SimpleScripts 0xE2 ×15, pickups 0xF0 ×8, fuseboxes 0x31 ×8,
   destroyables 0xE1 ×6, sound triggers 0xF9 ×5, AND/SEQ 0xEC/0xED ×8, ladders 0x22 ×4, doors 0xDB ×3,
   Touch 0xEB ×3, monitors 0x30 ×1 + cameras 0x28 ×2, ScriptPlayers 0xD9 ×2, hints 0x33 ×2, wire 0x3E ×1,
   emitter 0xF2 ×1, hurt 0xE4 ×1, LOAD_LEVEL 0xE8 ×1, music 0xFB ×2. Every zero kind of the doc's
   list re-confirmed absent on the disc (switches 0x29, locks 0x2E, copters 0x2F, gadget targets,
   grapple 0x3A, cars 0xDA, searchlights 0xDE, spawners 0xF1, mines 0xFE, …) — and **no 0xDF wind cel
   exists on this level** [D the dump], which makes a resting rigid body bit-still.
2. **The kind coverage audit**: 16 of the battery's tests already covered every player-touchable kind
   of the driver's list (doors, the console family = E-usable SimpleScripts + monitor + LOAD_LEVEL,
   fuseboxes, pickups, breakables/destroyables, plates/touch zones, monitors/cameras, player-triggered
   SimpleScripts; lifts have no placement kind at all and switches/locks are zero [D]). The one
   uncovered player-touchable kind was **rigid bodies 0xD8** — 21 placements the census carries and no
   test drove.
3. **The rigid-body kind battery** (2 new tests in `crates/nf-view/tests/interactables_07000002_live.rs`,
   now 18):
   - `the_rigid_bodies_rest_at_their_placements_and_a_walk_touch_moves_nothing` — the census asserts
     (21 bodies; every one type 1 [D param 2]; masses bucket 3 / lamps 1 / pans 2 [D param 0, the
     +0x2c+4k level_tag layout [M 0x1d1b68:64-75; L the RB_Create breaks on 07000004/06]]); the rest
     hold (240 settled ticks leave every body's rows at the RB_SetState identity and
     trans == pos [M 0x1d7da0; no wind [D], the buoyancy gate shut [M 0x1d6e88]]); then the real walk
     into the bucket (`bucketswing` #4 [D]): the bucket's rows stay at rest and it never translates —
     the player's contact is class 3 (obj+0xFF = 3 [M Player_Init 0x1910f8:76]) and
     `RB_RestingContactRot` 0x1d73a0's hit walk has NO arm for it [M the decompile: only the class-5
     arm writes] — while the body itself BLOCKS the walk query (Bond cannot stand in it) [M Collide_Pick
     0x1eb320:314-329 + Player_Collision 0x194df0's 0x800 query]. Sisters: the far lamp is untouched by
     the walk; the rest hold fails if any wind/buoyancy/sweep path writes a resting body.
   - `gunfire_swings_a_rigid_body_without_moving_its_anchor` — a real P99 round (the `shoot_at` aim
     loop, the real trigger taps) swings the bucket: the matrix leaves the identity, the anchor never
     moves (trans == pos to 1e-6 — the type-0-only position arm of RB_EulerEvolve 0x1d7810 [M]), the
     state stays finite. The class-5 record the arm consumes: hit+0x8 = the round's damage,
     hit+0x30 = the flight direction [L 07000004 lamp #40, a break at the class-5 arm 0x1d74dc];
     the impulse = (hit+0x8)·0.5·vel with the type-1 cross's y lane zeroed [M 0x1d73a0:
     0x1d753c-0x1d757c]. Sisters: the bucket rests at the identity before the shots; the far kitchen
     pan (#145) is untouched by the garden gunfire.
   - **The one subtle game rule the tests pin on the way**: the aim runs THROUGH the body (the
     eye→centre line carried 1.2 m past). The bodies are thin shells and the game's ray rule is
     single-sided — a triangle whose plane faces along the ray is skipped [M Intersect_RayGeom
     0x10a8c8:168] — so a segment that STOPS at the broad-sphere centre can end inside the cavity
     with no front-face crossing (measured: the centre-aim ray lands nothing on panswing1's 88
     triangles). A through-segment crosses the shell's exit wall, which is how the game's own rounds
     land on lamps [L 07000004 lamp #40].
4. **The doc correction**: docs/levels/07000002.md's rigid-body row still said "unported (drawn at
   rest)" — stale since the evolution landed (the `s/rigid-body-cylinder-sweep` merge this branch
   is cut from): the row now carries the census detail (masses/types), the class-3/class-5 touch
   rules and the works status with the test pointers. The old row contradicted the shipped
   `rigid_body.rs` (RB_Update/EulerEvolve/RestingContact(Rot)/ApplyWind all ported and wired per
   tick) and the existing `rigid_body_wiring_live.rs`/`rigid_body_draw_live.rs` batteries.

## Port gaps the tests exposed

None behavioural. The audit's one candidate — the port never mirrors a player's walk contact onto a
body's obj+0xD0 chain (only blasts and bullets push hits) — is behaviourally identical to the game on
every disc body type involved: the Rot walk (0x1d73a0, every 07000002 body is type 1 [D]) falls
through non-class-5 records, and the type-0 position-correction arm of RB_RestingContact 0x1d7168 has
no candidate on this level (no type-0 placements [D]). If a future level places type-0 bodies next to
GUARDS (class ≠ 3/0x1a/5 → the correction arm), that mirror becomes load-bearing; flagged below.

## How it was verified

- `interactables_07000002_live`: **18 passed, 0 failed** (was 16; +2 rigid-body tests).
- `walkreal_07000002_live`: **2 passed, 0 failed**.
- `nf-game` (full crate, --test-threads=1): **1028 passed, 0 failed, 9 ignored**.
- Fast suite (`tools/test/suite.py`, this tree = master + the task commits, nothing left to merge):
  see the total line below.
- The census numbers above are from the probe dump run against the slim ISO (NF_ISO set) this session.

## Suite total

This sandbox kills background jobs between tool calls (confirmed three times this session, the same
way the previous session's WORKLOG recorded) and the workspace's test-binary count is now 1070, so a
single full-suite pass does not fit the tool window either. What ran, completely:

- Per-crate rule-5 runs (`cargo test --offline --release -- --test-threads=1`), every crate of the
  rule-5 line:
  - `nf-game`: **1028 passed, 0 failed, 9 ignored**;
  - `nf-formats` + `nf-audio` (--no-fail-fast): **229 passed, 4 failed, 3 ignored** — the 4 are the
    slim-image class: the 3 `disc_golden` golden-manifest tests the rules allow, plus
    `identity_disc::the_real_disc_passes_the_identity_check`, which asserts the RETAIL image's
    1,191,072 sectors and measured this sandbox's slim on-foot image (518,435 sectors, volume
    NIGHTFIRE/SLUS-20579/NTSC/1.00 all matching — the failure is the sector-count line only);
  - `nf-view`'s task binaries: `interactables_07000002_live` **18/0**, `walkreal_07000002_live` **2/0**.
- The fast runner (`tools/test/suite.py`, this tree = master + the task commits, nothing left to
  merge): its 1070-binary run phase was capped twice by the tool window and once by the background
  killer; the completed prefix before the cap: **73 binaries, 309 passed, 0 failed, 22 ignored**,
  zero FAILED/TIMEOUT/CRASH lines (`suite.log`). A background continuation was attempted after the
  push and died between tool calls like its predecessors (four kills this session), so the quoted
  prefix is this session's suite record; its totals are summable from the per-binary
  `test result:` lines in `suite.log`.

## What's left

Nothing for the task's kind list. Open observations for a future task, not this level's gaps:
- The player-contact mirror onto obj+0xD0 (Collide_Update 0x1ea398:160-233) is only load-bearing for
  type-0 bodies touched by non-player classes; it is unported and unexercised on the disc levels so
  far [S the carry-forward note, no assertion].

## Live checks for the main agent

- One render spot-check (lesson 6: the drawn pose is the render layer's): with Bond at
  `--bond -35.85,6.3,-28.6,<yaw facing the bucket>,0` near the garden bucket (-35.85, 6.83, -31.02),
  fire at it — the bucket should visibly tip/swing about its hang point while staying on its anchor
  (the sim-side swing is pinned by `gunfire_swings_a_rigid_body_without_moving_its_anchor`; the drawn
  matrix path is `scene.rs`'s rbodies read at obj+0x90).

## Usage

Token/credit/context-window figures for this session: not visible to me (the environment shows no
token, cost or context-usage counters to copy; no input/output token figures are exposed).
