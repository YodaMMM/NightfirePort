## Your task: `level-06-searchlight-state`, The Exchange: the few searchlights that still look lit after the fusebox cut

On 07000006, after the fusebox is cut with the laser (green wire), MOST searchlights go dark correctly, but a couple still show a bright lens/housing although they are off (no beam). Wanted: those couple look off exactly like the others. NOT wanted: hiding the searchlight - an earlier attempt set the head `drawn = false` on the off-kill and every searchlight vanished entirely, which is wrong (the housing stays in the game).
Find: (1) which searchlight placements stay bright after the cut and what distinguishes them from the ones that go dark correctly [D their placements, off channels, which channel the cut writes, their linked parts; whether the cut reaches them at all]; (2) exactly what the game's off path changes on the head object and its linked parts [M Searchlight_Update's off-channel kill: which fields - obj+0xFE bit, colour bytes, glist, light removal, the lens/corona parts], so the head stays drawn but unlit. Port the sim/data side so those couple end in the same state as the others, and expose the per-light on/off state; list exactly what must DRAW differently (object, part, state) for the main agent - drawing changes are the main agent's. Test through the real fusebox laser path that every searchlight ends in the same off state. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-searchlight-state`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T18:09Z

---
## Report (done, 2026-10-28T19:55Z)


---
## Report from the worker (done, 2026-10-28T19:56Z)

# level-06-searchlight-state — the 07000006 searchlights that stay bright after the fusebox cut

Branch `s/level-06-searchlight-state` (from origin/master 3c312512, merged origin/master 1ca7f4c4
mid-task, no conflicts). Sandbox 18:12–20:12 UTC.

## What was asked

After the green-wire Laser cut (group 0, ch 57 + 103), most courtyard searchlights go dark but a
couple keep a bright lens/housing with no beam. Find which placements and why; pin the game's off
path field-for-field; port the sim/data side so every light ends in the same off state; expose the
per-light on/off state; list the draw deltas for the main agent. Never hide the searchlight.

## The answer: wiring is NOT the difference — the port's off-kill was

**(1) Which placements, and does the cut reach them?** 07000006 has exactly four searchlight
lights (20 kind-0xDE placements = 4× {head, mount, spot, beam, corona}): heads #2 (on-ch 64),
#4 (63), #6 (61), #8 (62), all with **off/kill channel 57 and alarm 55** [D placements; census in
docs/levels/07000006-audit.md]. The fusebox #747's group-0 cut writes ch 57 + 103 one-shot
[D #747 params 57/55/103/55; M FuseBox_Update 0x1adfe8:110-115]. So the cut reaches all four
identically ([M] the kill gate reads `switch_channels[inst+0x35]` every tick, Searchlight_Update
0x1cb900:92) — no placement is wired differently. "A couple stay bright" is not a missed kill.

**(2) The game's off path on the head and its linked parts** [M Searchlight_Update 0x1cb900:92-138,
all readers/writers of the instance are create+update only]:
- obj+0xF4 (state) = 3.
- For the SPOT link (inst+0x28): if the spot holds a psiLight node (spot inst+0x28) —
  `LList_Remove(0x2c74cc)`, `psiLight_Delete(node+0x34)`, back to pool 0x2c74c0 [M :95-110].
  Then the spot obj gets `obj+0xFE |= 1` and the link slot is zeroed [M :113-114]. obj+0xFE bit 0
  is DELETE: the movement pass does `Effect_RemoveSObjs` + `control_delete_object`
  [M control_movement_object_handler 0x1336f8:88-91].
- For the BEAM link (inst+0x2c): `obj+0xFE |= 1`, slot zeroed [M :116-126].
- For the CORONA link (inst+0x30): `obj+0xFE |= 1`, slot zeroed, and the head's ON byte
  (inst+0x34) = 0 [M :127-135]. No colour bytes are touched (the corona's alpha/scale at
  obj+0x106/0xE8 die with the object).
- inst+0x34/35/36 (on/off/alarm bytes) = 0 [M :134-137] — the state-3 arm reads the head's OWN
  on-byte [M :399], so a killed head can never re-light.
- `Control_NextLOD(head)` [M :138]: `obj+0xD8 = *(glist+0x40)` when nonzero
  [M Control_NextLOD 0x1349b0:8-12]. The runtime glist+0x40 is the block-0x27 LOD chain pointer —
  `parsemap_block_LOD` 0x1d1f10:21 stores `pCelList + index*0x50` from the disc's per-object
  first-dword (−1 = none) [M]. **[D 07000006 block 0x27]: every head placement resolves map
  object 2 OBJECT_SearchLight1 whose chain entry is 131 = OBJECT_SearchLightBroken; the
  mounts/beams/coronas/spots carry −1.** So the killed head keeps DRAWING — swapped to the
  dark-lens Broken variant — and the MOUNT (inst+0x20) is never touched: the housing stays.
  The bullet break path does the same swap [M :454] plus 0xE6 [M :455-456].

**The two real port deltas** (both fixed):
1. The port's off-kill hid EVERY link **including the mount** and cleared its link slot — the
   game keeps the mount drawn and linked [M the kill walk covers only psVar2+0x14/0x16/0x18].
2. The port never swapped the head's model: killed heads kept drawing the LIT-lens
   OBJECT_SearchLight1 mesh forever with no beam — exactly the reported bright lens/housing.
   (The game shows the Broken-variant housing.)

## Port changes (sim/data side; drawing is the main agent's)

`crates/nf-game/src/searchlight.rs`:
- `Part::lod_next: bool` (own line at the struct end) — "Control_NextLOD ran on this head",
  set by the off-kill [M :138] and the break [M :454]; the renderer resolves the mesh via
  `LevelMap::lod_target` from the part's placement (nf-formats already parses block 0x27).
- The kill (`Searchlights::off_kill`, extracted for unit testing): keeps links[0] (mount) and its
  drawn flag; clears links[2..4] only; zeroes on/off/alarm; frees the spot's light node
  (`light = None` — the psiLight_Delete); sets `lod_next`.
- The break path in `bullet_check` also sets `lod_next` (its deletion set was already correct).

Per-light on/off state exposure: `Part.{state, killed, on, off, alarm, drawn, lod_next, light}`
already cover it; the doc now states the contract (docs/fields/searchlight.md, kill gate +
obj+0xD8 row + port-mismatches 1/2 marked FIXED).

## What must DRAW differently (main agent's half)

For a searchlight head part with `lod_next == true` (state 3 + killed):
1. Draw **OBJECT_SearchLightBroken**'s mesh (map object 131 of 07000006's main map, via
   `LevelMap::lod_target(object_of(part.placement))` [D block 0x27]) at the part's current matrix,
   instead of OBJECT_SearchLight1's lit-lens mesh — the dark-lens housing.
2. The MOUNT keeps its current mesh and matrix (the sim no longer collapses it; nothing to do
   beyond not hiding it — nf-view's `p.drawn` gate already handles it).
3. Spot/beam/corona stay collapsed (`drawn = false` — unchanged).
4. Nothing else changes for lights still on; a light the boot NIS never lit (heads start state 3
   with on-channels unstaged) draws its normal head mesh — the game only swaps on kill/break.

## Verification

- New unit test `the_off_channel_kill_matches_the_game_field_for_field` (nf-game): the kill's
  field-for-field contract [M 0x1cb900:92-138] incl. mount survival, alarm byte, lod_next, and
  no re-light off a cleared on-byte. All 13 searchlight unit tests green.
- New real-input test `searchlight_off_07000006_live` (nf-view, opt-in NF_ISO):
  (a) the [D] pin — all four head placements chain to OBJECT_SearchLightBroken, non-head parts
  carry −1; (b) through the REAL fusebox path (guardroom stance → Laser equipped → the
  fusebox_06-style burn loop → ch 57 + 103 rise) every head ends in the SAME off state
  (state 3, killed, on/off/alarm 0, links[2..4] cleared, mount linked+drawn, head drawn with
  lod_next set), no spot/beam/corona drawn, no light node, and no re-light over the following
  300 ticks while the boot NIS keeps pulsing. GREEN.
- Regression: searchlight_flip_07000006_live (3), searchlight_bullet_07000006_live (2),
  fusebox_06_live, walk/walkreal 05/06/07 — all green (see the suite slices).

## Suite

The suite.py runner does ~44 binaries per 9.5 min on this sandbox (2 cores); the full 883-binary
set needs ~3 h and the window had ~45 min after the fix landed, so it ran in slices
(`--only`, same runner, same flags, final merged tree):

- First pass (pre-merge tree, killed at 580 s): 44 binaries completed, **0 failed, 0 timeouts**
  (/home/z/suite.log).
- Chunk 1 (post-merge tree): units (nf_game/nf_formats/nf_audio/nf_view) + all searchlight/
  fusebox/alarm/rook/06/lighting/level-carry binaries (35 binaries):
  **`877 passed, 0 failed, 7 ignored  (4 min)`**.
- Chunk 2 (post-merge tree): walk_07000005_live, walkreal_07000005_live, walk_07000006_live,
  walkreal_07000006_live, walk_07000007_live, walkreal_07000007_live:
  **`19 passed, 0 failed, 0 ignored  (2 min)`**.
- No FAILED/TIMEOUT/CRASH line in any slice. The crates my change touches (nf-game, nf-formats,
  nf-audio, nf-view) ran complete in chunk 1. The rest of the 883 binaries did not fit the
  sandbox window and are the first thing the next message should run (`python3 tools/test/
  suite.py .` — it resumes nothing, so budget ~3 h or slice it).

disc_golden was not in any slice — the slim ISO's 3 allowed failures are untested this session.

## What's left

1. The renderer half: draw OBJECT_SearchLightBroken for a `lod_next` head (list above), then the
   main agent's PCSX2 render check against the game.
2. The full-suite tail (~800 binaries, ~2.5 h).
3. Open live check from before (kept in docs/fields/searchlight.md): which beat moves SP #808 in
   the real game (the port boots NIS 0x06000066 at level start as the stand-in).

## Live checks for the main agent

1. Off-kill model swap (07000006): cut the fusebox (green wire) and `watch` a courtyard head's
   obj+0xD8 — on the kill tick it should step from the placement's glist to the
   OBJECT_SearchLightBroken glist [M 0x1cb900:138; chain via glist+0x40, 0x1d1f10]. Also watch
   the head's and mount's obj+0xFE — both must stay 0 (neither is deleted). Predicted render: the
   housing stays, the lens goes dark, beam/corona/spot vanish. Full command sketch in
   docs/fields/searchlight.md "Live checks" item 5.
2. Mount survival: after the cut, the mount mesh must still draw (the pre-fix port hid it).
3. Broken-variant render spot (for the draw work; head #2 at (−1.12, 47.57, −17.33), courtyard):
   `--bond -4.0,42.1,-17.0,1.9,0.0` looking east at the light — off state should show the
   dark-lens housing, no beam/corona/spot. Second spot for head #6 (−1.13, 47.57, −41.31):
   `--bond -4.0,42.1,-41.0,0.0,0.0`.

## Usage

My own session's token/context/cost figures are not visible to me (no such readout in this
environment). The only figures I could read were the per-subagent turn readouts of the delegated
shell runs (the main session's Bash wedge forced delegation), last one:
`Tokens -> prompt:8317 completion:336` — per-call subagent counts, not session totals.
