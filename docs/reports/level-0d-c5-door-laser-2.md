# level-0d-c5-door-laser-2 — the sustained beam drains the C5 door to the smash

Branch `s/level-0d-c5-door-laser-2` (base `s/level-0d-c5-door-laser` 3c0838b3 merged with
`origin/master` 480aa3db). **Both task items are done and the drain test is green un-ignored;
the full four-crate suite did not fit this sandbox's window** (details under Verification) — the
next message should run it first thing on this tree.

## Done

1. **The sustained-fire defect — root cause: the harness's aim convergence, not the port's fire
   path.** A per-tick probe through the real mount showed the stall exactly: the drain test's aim
   loop measured convergence on the SMOOTHED aim (ext+0x80/0x84, a 0.02-rad window) while feeding
   the RAW aim (ext+0x74/0x70). The smoothed mirror eases onto the raw at 0.15/tick [M GunImp_Update
   0x1320d8:136-161], so during a saturating approach the raw leads the smoothed by rate/0.15 — the
   loop broke with the raw ~0.1 rad past the target, and the zero-input ticks then eased the barrel
   past the door: the settled beam dug into the deck ~4 units short of bigdoor02 (probe trace: beam
   far y 3.985 vs the door's y-lo 3.9875), only the settle-sweep shots landed (~5 contacts =
   10000 − 9750), and the health froze while the beam quad stayed up and the heat climbed. The
   decompile-checked fire path is faithful and needed no change: the 0.15 ease with the game's own
   no-overshoot clamp (inert for a 0.15 lerp) [M :136-161], the round spawned from the smoothed cel
   matrix's direction and the TurretGuns muzzle [M :325 Mat_GetDir + :196-211], and the every-tick
   re-trigger under a held trigger (cooldown refills TurretGuns+0x14 = 1.0, drains 1.0/tick [M
   :107-112, :324]) — which is exactly the [L] PCSX2 shape (~50/frame of contact ≈ one 50-damage
   contact per tick at one shot a tick). The fix is in the TEST, per the rules (the harness
   contradicted the game's ease dynamics): converge on the RAW aim — the player's stick-rest
   position — then coast the ease 40 ticks (0.85^40 ≈ 1.5e-3) before the trigger, matching the
   [L] stick-at-rest bursts that ran with the beam already on the door.
2. **The drain test un-ignored, green** (`the_beam_drains_the_c5_door_at_50_a_tick_and_the_smash_
   fires_at_zero`, 3/3 in `0700000d_gunimp_door_live`). The full game shape now rides out in the
   test's own trace: six 30-shot bursts drain −1500 each (health 10000 → 8650 → … → 1150, the
   colour pair 255 → 0 under the beam [M Destroy_CollisionHandler 0x12bd48:61-95]), then the heat
   arithmetic [M 0x1320d8 — += 2.0 a shot, −1.0 a tick, a shot every tick] puts the hot byte up at
   shot ~201 with the door at EXACTLY 0 (200 contacts), the game's hot gate blocks the fire arm for
   the ~200-tick recovery (the byte clears when the heat drains to 0 [M the 0x1320d8 head]; the
   test's burst trace shows the colour recovering 0 → 157 → 201 through the pause), and the kill
   shot after recovery takes the death catch to −50 [L measured; M 0x12bd48:46-49] — bigdoor01
   smashes the SAME tick at full health 1000 [M Destroy_Update 0x12c4f8:29-33; L], ch 0x1f latches
   [M 0x12bb60:40], and objective 6 completes to state 4 the tick after [M 0x1827d8:78-85].
3. **The destroyed-glist swap's draw side.** The statics bake at build time (nothing is dead yet),
   so the hide is a per-frame gate: `Scene::destroy_centres` records each DRAWN 0xe1 leaf's world
   centre (the same `transform(matrix, obj.centre)` the group's cull sphere carries — the bits
   match the material's `centre`) with its destroy placement [M props.create's placement index];
   lib.rs feeds `Renderer::destroy_dead` each frame from the sim's `props.destroy[..].dead`; the
   renderer's per-frame draw filter drops every World draw whose centre bits match a dead leaf.
   That is `Destroy_Smash` 0x12bb60's `Control_SetGList(obj, *(tag+0x00))` swap's draw semantics
   [M 0x12bbb8 asm + 0x1d1b68]: the C5 leaves swap to the parsemap fallback slot, which has no
   batches [D probe lod_word −1 for every 0d 0xe1], so the doorway ends open [L]. A leaf WITH a
   LOD variant would draw the LOD mesh instead — no disc row on 0d needs it [S unverified].
4. **The merged-tree build fix**: master's `the_remote_kill_cascade_propagates_one_link_per_tick_
   newest_first` constructed `Destroy` without the predecessor branch's new fields — the two test
   initializers now carry `mask: 0, colour: [0xff, 0xff], hit67: false` (the parse initializer's
   shape: p4 = 0 → mask 0). The four-crate BUILD is green on the merged tree.

## Verification (what ran)

- `cargo test --offline --release -p nf-view --test 0700000d_gunimp_door_live`: **3 passed,
  0 failed, 0 ignored** — the drain test un-ignored green, plus the handheld filter and the
  beam-stop tests still green.
- `cargo check --offline --release -p nf-view` and the four-crate `cargo test --no-run`:
  **BUILD GREEN** on the merged tree (after the props.rs fix above).
- `cargo test --offline --release -p nf-game --lib -- --test-threads=1`: **582 passed, 0 failed,
  5 ignored** — the destroy/explode/gunimp unit families (the drain machinery's home crate) are
  green on the final merged tree.
- **Not run: the full four-crate suite total line.** This sandbox kills background processes
  between tool calls (three detached runners died — setsid/nohup/disown all included), and the
  foreground `python3 tools/test/suite.py` run spent its whole 10-minute tool window inside the
  build step (nf-view's ~50 test binaries) without reaching the run phase. **Next message: run
  `NF_ISO=<abs> python3 tools/test/suite.py` first thing on this tree and quote the total line.**
  The touched-crate runs above are the only test evidence from this session; nf-game's and
  nf-view's full binaries are untested on the final tree.

## Live checks for the main agent

1. **The doorway render** — the destroyed-glist hide is drawing code, so please render before and
   after the smash. `--bond 137.27,12.0,-340.0,3.14,0.1` (in front of the deck, facing the
   doorway): BEFORE the drain both C5 door meshes (bigdoor01 #93, bigdoor02 #96) fill the
   doorway; AFTER bigdoor02's death (the drain test's end state) BOTH meshes are gone and the
   doorway reads as an open passage [L doorway ends open; D the leaves carry no LOD variant].
   A second spot inside the next room, `--bond 137.0,6.0,-285.0,0.0,0.0` looking back at the
   doorway, should show the passage open with no door slivers left at the frame edges.
2. While the beam is ON the door (mid-drain), the beam quad should stretch muzzle → the door face
   and the impact flare should sit at the hit — the existing `the_beam_stops_at_the_surface_it_
   hits` pins the sim side.

## What's left

- The full four-crate suite total line on this final tree (first thing, next message).
- p6 > 0 deferred channel writes in `Destroy_Update`'s countdown arm (carried from the
  predecessor's report; the C5 doors are p6 = 0 [D], the port's immediate-write path is exact
  there) — needs the live check before porting.

## Usage

not visible to me
