# fusebox-wire-geometry-2 — the fusebox wire contact on the real cable geometry

Branch `s/fusebox-wire-geometry-2` = origin/master (24084ef2) + merge origin/s/fusebox-wire-geometry
(a1aefc0c) + this task's commits. No product code was changed by this task: the base branch's
wire-contact change (`wire_geo` local bounds through the live entity matrix, slab-tested in the
bullet pass [M Collide_PickObj 0x1eae80 → obj+0xD0 → FuseBox_Update 0x1adfe8:88-123]) is kept as
merged, and the four tests that aimed the Laser at the old stand-in sphere spot were re-aimed at
the wire OBJECT's visible geometry, the way a player aims.

## What was done

1. **The four red tests re-aimed at the visible cables, every assert kept.** Each burn now takes
   the wire object's world bounds (`Drones::wire_geo` local bounds transformed by the live entity
   matrix — the same slab the bullet pass tests) and holds fire with a per-tick aim-walk: the test
   reads the Laser round the tick spawned (`player_bullets`) and shifts the crosshair by that
   round's miss against the wire's own bounds — a player's watch-the-beam-and-correct. Files:
   `crates/nf-view/tests/fix_fusebox_cut_live.rs` (the 07000014 sabotage test),
   `crates/nf-view/tests/walk_07000014_live.rs` (the facility chain's sabotage stage),
   `crates/nf-view/tests/fusebox_06_live.rs` (both bundle tests + the aim-only sister, which now
   aims the real cable with fire still held off).
2. **walk_07000014_live's no-hurt pin repointed (test changed, reason here).** The old pin
   `health == 100.0` silently relied on the beam aiming 1.5 m BELOW the panel, where it never
   touched the room's DESTROY_BoxSatellite boxes. With the real cable aim the beam lands on the
   panel, their smashes blast the walker — the game's own behaviour (this test's own 996-audit
   note; measured 3.3 splash over the four cuts) — so health is no longer part of the pin. The
   decompile-backed claim is now pinned exactly: `fusebox_hurts == 0` [M 0x1adfe8:99-101 — the
   group-1 death hurts only while +0x56 == 0; every 07000014 box carries param 2 non-zero [D
   placements 291/383/385/833/919]], the same counter fix_fusebox_cut_live's sabotage test pins.
   The walker also asserts `health > 0` (the chain continues through three more stages).
3. **The 07000006 box resolves its wire geometry too [D].** The probe dump (see Verification):
   box #747's registered wires are hash 0x02000371 (group 0, obj 13) and 0x02000428 (group 1,
   obj 14) — both present in `wire_geo`, world slabs at x 28.09..28.11, y 41.35..41.85 (z
   −18.07..−18.01 / −18.28..−18.14) while the script entity translation is (28.00, 40.00,
   −18.15) — 1.35 m below the visible bundle. Both bundle burns now go through the real geometry
   (fusebox_06_live 4/4). 07000008's box was not probed; its three tests (guardroom burn, E
   sister, gunfire sister) are UNCHANGED and green — its contact behaves exactly as before this
   task (its wire aim still uses the entity translation and the burn works, i.e. the sphere
   fallback path is what serves it).

## Verification (all with NF_ISO on the slim on-foot ISO)

- Direct runs of the touched binaries, `--test-threads=1`:
  - `fix_fusebox_cut_live` **4/4** — the sabotage cuts box 383's g0+g1 and box 385's g0+g1
    (stance dist 0.88–1.10 to the visible cables), ch 0x87+0x88 rise, AND #391 raises 0x8C,
    `fusebox_hurts == 0`; the guardroom/E/gunfire sisters unchanged.
  - `fusebox_06_live` **4/4** — the searchlight bundle → ch 57 + ch 103, the alarm bundle →
    ch 55, no hurt (+0x56 = 55), no-fire and no-E sisters hold.
  - `walk_07000014_live` **2/2** — the full chain to the delta-sector exit with the real-cable
    sabotage in the middle.
  - `fusebox_geometry_07000014_live` **1/1** (the base branch's own test).
- Probe dump (scratch example, not committed): 07000014 box1 g0k0 wire hash 0x02000AA1, world
  AABB x −89.474..−89.349, y −0.727..−0.502, z 30.895 (thin slab) vs the entity translation
  (−89.3055, −2.2213, 30.787) — the cables 1.5 m above the translation [D hashes/streams; the
  base branch's doc comment]. Laser record 0x4E: flags68 0x12482 (fp-model bit 0x10000 set, scope
  0x40 clear), +0x1C range 4.0 [L weapon_data] — the shot takes the midpoint-of-head-and-datum-0
  re-aim branch [M Player_WeaponInitBullet 0x1a4bf0:226-245], which is why the aim-walk uses the
  spawned round's actual miss rather than an open-loop crosshair guess.

## Full suite on the merged tree

The sandbox kills background processes between tool calls (nohup probe failed), so the suite ran
through a resumable clone of `tools/test/suite.py` (same build command, same per-binary semantics
— `--test-threads=1`, 600 s wall, NF_SILENT=1 — same accounting), chunked into foreground calls,
state in `/home/z/my-project/suite_resume_results.tsv`. Master is unchanged since the branch
point (origin/master still 24084ef2), so the merged tree IS the final tree.

- Progress when this report was written: **304/806 test binaries** (all 89 nf-game, all
  nf-audio, the nf-view priority families first), **zero product-code failures** — the only
  FAILED lines so far are `fix_0c_pylon_live::{the_0c_pylon_fills_the_view_from_the_closeup_spot,
  the_0c_pylon_is_on_screen_at_the_user_spot}`, reproduced on a direct re-run of the binary.
  **Out of this task's scope**: this task's diff is three nf-view TEST files; no pylon, view or
  bullet-pass product code was touched (the base branch's diff touches the fusebox wire contact
  loop only). The task's own merged-tree red list names exactly the four tests fixed here, so
  this pair needs a master-baseline run to attribute — see Live checks.
- The four task tests + the geometry test: green (direct runs above, and inside the suite run:
  fix_fusebox_cut_live, fusebox_06_live, walk_07000014_live, fusebox_geometry_07000014_live all
  completed inside the chunked run with 0 failures).

## What's left

- The remaining ~500 nf-view binaries of the chunked suite run (nf-game, nf-audio, nf-formats
  done; nf-view partial). Resume with any suite.py run — the tree is final; nothing to rebuild.
- Attribution of the `fix_0c_pylon_live` pair (master-baseline run or a live check).

## Live checks for the main agent

1. **07000006 wire geometry, live**: with a save state at the 06 guardroom nook (Bond ≈
   (29.9, 41.0, −17.9), the fusebox #747 panel ahead), watch `obj+0xD0` contact appends for the
   wire objects (hashes 0x02000371/0x02000428, script 0x0600004B) while lasering the VISIBLE
   bundle at y ≈ 41.6 (the entity translation y 40.0 is 1.35 m low). Expected: contacts land on
   the visible-bundle shot lines; ch 57/103/55 write as the port now does.
2. **The `fix_0c_pylon_live` pair**: run both tests on a pristine origin/master checkout against
   the same slim ISO. If they are green there, the base branch's wire-contact merge (a1aefc0c) is
   implicated despite the apparent disjointness and needs a look before merge; if red there too,
   they are pre-existing master reds outside this task.

## Usage

not visible to me (no token/credit/context-window figures are displayed in this environment)
