# level-0a-office-hatch-4 — 0700000a: the spawn floor, the hall height and the corner

## Status

**The −0.0109 spawn surface was hunted and NOT found in the port-reachable disc data; the three
sisters and the main walk stay `#[ignore]`d; no src changes landed this round.** The hunt's
evidence trail (four probes, committed) narrows the next session to one live check.

## What was done

### The −0.0109 surface hunt (fix 1 — not landed)

The game's spawn root is 1.0891 [L ee_start_0700000a.bin 0x18694e0] = the ray floor −0.0109 +
1.1 [M Player_StandAtNewPosition 0x190d88 disasm 0x190ee0-0x190f18: build_FindCel from pos+0.1,
build_PointOnFloor's (0,−1,0)·3.0 ray [M 0x1c95a8], then +1.1f; the port's `Bond::stand_at`
player.rs:505 = floor_below + 1.1]. The port's flat ray reads 0.0000, so the port spawns at
1.1000. The task's two fix options both need the surface's identity, so the round hunted it:

1. **The placed collision tris** — every instance of every map (`Level::instances(false)`, the
   world-transformed tris), two filters: (a) a face with ALL vertices at world y −0.02..0.004
   within 4 of the spawn xz; (b) the PLANE-y at the exact spawn xz (−3.93, 10.40) for every tri
   whose bounds overlap the column, band −0.08..0.12 (covering both the −0.0109 reading and the
   cc-settle alternative 0.0544 = 1.0891 − cc 1.0347). **Result: only three faces, ALL
   0.0000** — map37 obj458 Room-M_lifts tri36/tri253 and obj0 Player_1 tri2 [D dump
   hatch4_probe1].
2. **The stored plane records** — the game's `Intersect_RayGeom` (0x10a8c8 disasm: the tri's
   plane halfword → the leaf words: 3 s16 normal ×2⁻¹⁴ then the distance as the 4th s16 loaded
   RAW through `vitof0` and scaled (1e-4,1e-4,1e-4,1.0) — the distance scale is 1.0, the
   vertical-ray hit point y = −d) — the port's decoder drops the distance. obj458's spawn-column
   floor tris store **d = 0** → the plane = the vertices → 0.0000, not −0.0109 [D dump
   hatch4_probe2, the raw words printed].
3. **The dynamic set** — 0 tris in the spawn box (−6..0, −0.06..0.06, 8..13) [L probe
   hatch4_probe4]. The SS door pieces (#524/#525, room-m_LIFTDOOR2/1) bottoms sit at world
   −0.0055/−0.0336 [D probe1] — neither is −0.0109.
4. **The 0x1000 missile cage** (the set the actor queries skip) — the bullet-set ray reads a
   uniform **0.1154** along the whole route (the cage's lid/floor), not −0.0109 [L probe
   hatch4_probe3].
5. **hatch-3's "the SS cab placement #1 mesh at (−3.9026, 0.69766, 10.9626)"** maps on the disc
   to **plc#58, kind 0xEA, the TouchOnce arrival plate** (obj `oDroneTrigger1`); its collision
   spans world y 0.0000..1.3953 [D probe1] — no −0.0109 surface in it. (The level has NO
   cab/lift/elevator-named object; map37 obj1 is an AI volume at (−7.47, 0, 14.76).)

The port-side scoped answers agree: `find_cel` + `segment_hit_scoped` at the spawn return
0.00002 [L hatch-3], and the port's floors are ~0.0000 along the whole walked route (the cab
mouth, the lobby, the doorway, the hall, the corner) [L probe3] — the port's world has NO
raised 0.0256 hall floor either, so the climb settle (1.1147 = −0.0109 + 0.0256 + 1.1, the
hatch-3 ledger — the arithmetic re-verified) cannot land without the spawn surface either.

### What this means for the fixes

- **Fix 1 (the spawn)**: the surface's identity needs the live hitdata. The port's parser and
  both its query sets (static incl. the cels_only and the all-instance placements, dynamic,
  missile) demonstrably hold no −0.0109 face at the spawn — the surface is either outside the
  port's collision parse (a glist/room chain piece the parser doesn't reach) or the game's
  spawn ray hit comes from a query mode the port's ray families don't model (build_PointOnFloor
  passes HITTEST flags **0x70c** [M 0x1c95a8: `Collide_RayIntersect(..., 0, 0x70c)`] — the
  port's `floor_below` uses the flat `segment` with no flags; the 0x70c bits' node-class gates
  are unmodelled).
- **Fixes 2-3 (the hall, the corner)** ride fix 1: with the port's spawn at 1.1000 the climb
  would settle 1.1256, not the game's 1.1147 (± 0.003 test band), and the corner entry needs
  the 1.1147 glide. The corner's push landing [M Player_CollisionHandler 0x1949a8:136-140 —
  the `obj+0x30 += hit − stored` DeltaP add] remains the -2 report's open item.

## Live checks for the main agent (fresh boot — save-state resumes don't count)

1. **The spawn-ray hitdata**: at the spawn tick, break `Collide_RayIntersect` (0x1ec690) on the
   build_PointOnFloor call (the caller 0x190f0c-ish) and read the returned HITDATA: +0x40/44/48
   (the hit point — expect y −0.0109), +0x54 (the static cel — WHICH cel), and the hit's object/
   node pointer — then map that cel to its placement/object. Expected: a piece the port's world
   build does not push (the cab's floor mesh); its placement number + object name is what the
   port must register in the spawn cel's chain.
2. **The walk flags**: confirm the build_PointOnFloor call's HITTEST flags word is 0x70c and
   read HITTEST+0x98's effective value — if the 0x70c gates open a node class the port's flat
   ray skips, the port's floor query needs that mask before any surface registration matters.
3. **The hall floor**: a Collide_RayIntersect straight down at (−10.79, 1.0, 16.26) — is the
   game's hall floor 0.0000 (and the 1.1147 is the climb residue riding) or a raised surface
   (0.0147/0.0256)? The port's world says 0.00007 [L probe3]; the game's answer decides whether
   the climb settle is a residue or a floor step.

## Tests

- No test changed state this round: the three sisters and the main walk stay `#[ignore]`d with
  their evidence messages intact; the existing non-ignored walkreal_0700000a_live tests were
  run (they passed until the run was OOM-killed by the concurrent suite — no failure observed).
- New: `hatch4_probe1-4` (assertion-free dumps, `--ignored`) — the evidence trail for the hunt
  above, committed.

## What ran

- `cargo test --offline --release -p nf-view --test walkreal_0700000a_live` (the visible tests;
  the run was killed by OOM under the concurrent suite after the ignored tests printed — no
  failure observed) and the four probe binaries.
- **The full suite** was started in the background at 00:12 UTC and is still running at the
  report time; its interim lines: 4 FAILED (identity_disc::the_real_disc_passes_the_identity_check
  + disc_golden::bigf_header_fields_behave_as_documented + disc_golden::bigf_tables_match_the_
  golden_manifest + disc_golden::directory_tree_matches_the_golden_manifest — the disc-dependent
  set on the slim ISO) and 1 CRASH (mp_online_failures, exit −9 — OOM under this sandbox's
  concurrent probe builds, the same failure mode lift-2 recorded). No source changes landed, so
  no port behaviour changed this round; the next session should re-run the suite clean.

## What's left (the next session's order)

1. The live checks above → the surface's identity → register it (or port the 0x70c flags).
2. `Bond::stand_at` matches 1.0891 → un-ignore `the_spawn_height_matches_the_game_snapshot`.
3. The doorway line over the 0.1622 wall base → the 1.1147 glide → un-ignore the hall sister.
4. The corner: the 1.1147 entry + the DeltaP push landing [M 0x1949a8:136-140] → un-ignore the
   corner sister → the main walk.

## Usage

not visible to me
