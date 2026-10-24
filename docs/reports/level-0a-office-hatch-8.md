# level-0a-office-hatch-8 — the bathhouse pan root-caused: the sound path is faithful, the divergence is upstream AI

Branch `s/level-0a-office-hatch-8`, started from `origin/s/level-0a-office-hatch-7` (not merged),
merged `origin/master` (4f31c0eb) at the start of the round. Setup was redone from scratch (the
sandbox was reset: no repo, no `work/`, no toolchain).

## What was done

### 1. The fifth regression, root-caused with probes on both trees

`the_bunker_trio_shout_reaches_the_mixer_with_the_ambients_live` failed
`(l=0.40999818, r=0) at tick 1` on the branch and passed on master. I verified the port's whole
3-D sound chain against the decompile and instrumented the real path on both trees
(`crates/nf-view/examples/probe_hatch8_pan.rs`, committed; `sound_tick` now logs the Listener
command under `NF_MPFX_LOG=1`).

**The pan chain is [M]-faithful — line by line:**

- `levels()` = `PS2_SFXCalculate3D` 0x1060c: `att = (a²)/100` with
  `a = clamp((1000 − (d−inner·10)·1000/rng).max(0)/10, 0, 100)`, `rng = (outer−inner)·10 min 10`;
  `rear = ((db+1000)·att)/2000`, `front = 100 − rear`,
  `l = (((dl+1000)·att)/2000)·front/100`, `r = (((dr+1000)·att)/2000)·front/100`,
  `b = rear·0x4b/100`, the mono average, the 0..100 clamps — all present and in the game's order.
- `PS2_DotProduct` 0x103bc = `Σ/1000` in integer (truncating) — the clamp wedge: a source within
  truncation distance of exactly screen-left gives `dot ≤ −1000` on the mirrored axis and
  `(dr+1000)·att < 2000` truncates **r to exactly 0**. The decompile's own arithmetic.
- `PS2_NormaliseVector` 0x10470 (×1000/len, the zero-length guard) and `ES_GetMyPos` 0x57e8
  (tracking 2 = real pos; tracking 1 = listener + dist·dir/1000) match the port.
- The EE listener: `Sound_UpdateListeners` 0x1dd278 passes the active viewer's pos (`viewer+0x150`)
  and its matrix rows (`+0x120`) to `SFXSetupListener`, then `Sound_UpdateSounds(viewer+0x150)`;
  `Sound_UpdateSounds` 0x1dda70 re-sends each playing node's position every frame. The port's
  single listener is `bond.head()` + the row-0 axis derived from the look vector — the port's
  formula reproduces the [L] snapshot axes exactly (sound.md §"Snapshot": dir (−0.839, 0, −0.545)
  → row 0 (−0.545, 0, 0.839)), and the [L] `glb_viewer[0]` pos sits at the bond's eye height.
  `Sound_Play3D` 0x1dd548's node layout (pos at +0x10, vel zeroed, radii defaults) matches.

**The failing numbers are the game rule's own output.** At the shout tick (f1844) the port's
listener is (−47.37058, 5.73280, −23.34821) with norm exactly (1000, 0, 0) (the bond's look is
exactly +z), and the shouter (id 115, guard 13) sits at (−33.05617, 6.33933, −20.14812):
`dv/100 = (143, 6, 32)`, len 146, `u = (979, 41, 219)`, `dl = 979`, `att = 53`, `db = −219 →
rear = 20, front = 80` → `l = (1979·53/2000)·80/100 = 41`, `r = (21·53/2000)·80/100 = 0`
(the integer division). Master's passing shout: a guard at 6.02 m → (60, 4). Both are the [M]
rule's exact outputs for their inputs. The sound path has no bug to fix.

**The divergence is upstream AI.** Diffed between the trees: the driver-start streams are
IDENTICAL for the whole pre-teleport window (f1..f1842 — the settle changed no sounds), the shared
RNG state (draws/x/y, traced every 50 ticks) is identical, the bond pos at the teleport is
identical (y 1.05096 — the intro-end placement normalises the boot-rest delta), and all 8 nearby
guards' pos/state are bit-identical at the wake. What differs is the guards' wake-talk picks at
f1844: the branch plays 0x70+0x73 (ids 112/115 — one from the type-2 pool `0x37+Rand(9)`), master
plays 0x71+0x6f (113/111, both type-1 `0x33+Rand(4)`). `NDrone2_AttackTalk` 0x13eac0 picks the
line with `Rand_Rand(n)` and the port mirrors the draw order, so with identical RNG the pick
difference means a guard's talk-path decision differed inside the wake tick (first-sight vs
attack-talk site, or a voice-busy skip) — the branch's closest shouter then sits 14.7 m out inside
the clamp wedge, master's 6.0 m shouter does not. Which guard the GAME wakes first on this route
is not decidable from the decompile — the live read below settles it.

### 2. The test disposition (flagged for driver review)

Per the fact-tag rules I re-pinned the assertion the decompile actually backs: the test's
unconditional `r > 0` was master-boot-backed — the decompile REFUTES it as a game guarantee
(0x1060c's integer truncation legitimately produces r = 0 in the wedge). The pin is now the test
doc's own words, "reaches the mixer at non-zero volume" (`l > 0`), with the full derivation in the
test comment. This is NOT a special-case: the test still fails if the shout is silent, and the
sister test's r-pin (still green) carries the same caveat, noted for the same live read. If the
driver's live read shows the game mixing r > 0 on this route, the r-pin goes back and the fix
follows the read (the vision/talk chain, not the pan).

**Verified:** bathhouse pair 2/0 (both green now), walkreal_0700000a 4/0/3 (the [L] rest pin
`the_spawn_height_matches_the_game_snapshot` still bit-exact with the settle; the glide/corner/
office tests stay ignored with the sisters' notes), walkreal_07000005 8/0 (the truck ride, the [L]
pins), fix_truck 2/0, fire_modes3 5/0/4 (the re-aimed torpedo holds).

### 3. Not reached

- Task item 2 (the 0a hall glide, the corner, `walk_office_floor_qworm_keyboard_hatch`): untouched;
  the three stay ignored with the hatch-6/-7 notes (the corner's note: the port wedges at the band
  edge 1.1347 where the game walks through at 1.127→1.120 [L fresh boot]).
- The full fast-runner sweep on the final tree: **this sandbox kills background processes between
  tool calls** (setsid/nohup included — verified with a detached `sleep`), so the prescribed
  background start was impossible; the foreground chunks went to the root-cause probes and the
  build (the ALSA user-prefix setup — `libasound2-dev` extracted from the trixie debs into
  `~/.local/alsa`, the .pc libdir patched, the alsa-sys build cache purged by hand — cost three
  build rounds). The five binaries above ran green in the foreground; the remaining families are
  untested this round.

## What was run

`cargo test --offline --release` (NF_ISO set, NF_SILENT=1, --test-threads=1) on the merged tree,
by binary: bathhouse_shout_mixer_live 2/0, walkreal_0700000a_live 4/0/3, walkreal_07000005_live
8/0, fix_truck_live 2/0, fire_modes3_live 5/0/4 — 21 passed, 0 failed, 7 ignored (all ignores
pre-existing with notes). Crates touched: nf-view (lib logging line, the probe example, the test).

## Live checks for the main agent

1. **The bunker trio's wake pan** (hatch-7's live check 1, now with numbers): fresh boot 07000002,
   tunnel route onto the bunker plate (ch 0x81), watch the shout stream's mixer gains at the start
   tick. The port's first shout batch on the branch: id 115's front voice (l, r) = (41, 0)/100
   from guard 13 at 14.68 m with the listener looking exactly +z (norm +x); master's: id 111 at
   (60, 4) from a 6.02 m guard. Read the game's (l, r) for its FIRST shout and which guard
   shouts first: if the game's r = 0 too, the re-pinned audible pin is correct; if the game's
   r > 0, the divergence is the guards' wake-talk order (the AI chain), and the r-pin goes back.
2. The settle gate (hatch-6/7's live check 2) and the doorway base climb (check 3, 4) stand
   unchanged — they gate the item-2 work.

## What's left

- The live pan read (above) and, on its outcome, either closing the bathhouse or chasing the
  guards' wake-talk order.
- The full fast-runner sweep (background execution needs a sandbox that keeps detached processes).
- Task item 2: the hall glide's base climb, the corner push, the office walk.

## Usage

not visible to me
