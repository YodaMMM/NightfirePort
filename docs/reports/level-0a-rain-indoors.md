# level-0a-rain-indoors — rain falls inside the office (0700000a)

Branch `s/level-0a-rain-indoors` (from origin/master 38f1bc93). Sandbox window 05:40–07:39 UTC;
this session is the whole task.

## Symptom

In play on 0700000a, rain streaks draw indoors in the dark office lounge at Bond
(−37.50, 1.11, −16.95, yaw 2.827, pitch −0.092), frame 2719. Rain belongs outside the windows
only (the user's live observation; 0b's rain already checked as right).

## How the game confines its rain

- Rain exists only inside **authored RainBox volumes**: placement type 0x32 → `RainBox_Create`
  0x18d840 [M], which stores the placement's referenced object's celglist (`obj+0xd8`) and links
  the box into `RainBoxRoot`. `parsemap_block_map_data_dynamic` 0x1d1b68 only creates a dynamic
  object when the placement resolves to an object and `flags & 0xa000 == 0` [M 0x1d1b68].
- Each frame `UpdateDrops` 0x18c580 [M] cube-clips every box to the viewer ±15 via
  `RainBox_Clip` 0x18d0d0 [M] (box min = celg+0x20 + obj pos, max = celg+0x30 + obj pos; active
  when the clipped x and z ranges are both non-empty) and returns early when **no** box is active.
  `AddDrop` 0x18c278 [M] spawns a drop only inside an active box's x/z footprint and above its
  clipped floor; the kill height is max(box floor, viewer_y − 7.5).
- **The step**: `UpdateDrops` builds `RandTable2[i] = RandTable1[i]·FRAME_RATE_MUL + wind_quad·FRAME_RATE_MUL`
  where the wind quad is `{WindVector, DAT_003194f4, DAT_003194f8}` [M 0x18c580, the `_vmulbc`
  broadcasting FRAME_RATE_MUL (x), `_vadd` into the position]. `InitDrops` 0x18bf68 [M] sets
  **`DAT_003194f4 = −0.5` for DropType 1 (rain)** and 0 for snow/rising. So rain falls at
  ~0.55/tick (≈33 units/s) with a per-slot slant vx/vz ∈ ±0.02/tick (fixed per slot), drift ≤
  0.86 m over a full 15–22.5 m fall. `DAT_003194f4`'s only other reader is `AddDrop`'s spawn wind
  shear, dead for every kind (WindVector stays 0 for rain; DAT_003194f4 = 0 for the kinds that
  have wind) [M 0x18c278:49-52]. Rain draws per drop through the object pipeline
  (`DrawDrops` 0x18cb90 → `psiDrawObjectMatrix`, depth-tested; a second darker camera-facing pass
  with mesh 0x2000039 and `psiSetTweakARGB(0x24,0x20,0x20,0x20)` on 07000009–0b) [M 0x1e6f50 call
  site: per active viewer, after the solid pass]. No room/indoor gate exists in the draw — the
  confinement is entirely the boxes + the fast fall.

## Why the port rained indoors

[D] 0700000a: one Env placement (#530, param 0 = 1 → rain [M `Env_Create` 0x17b968 reads
level_tag+0x2C into `InitDrops`]) over seven rainboxes (`Effect_rainbox1..7`, placements
#791–#797, glist bounds + translation — full table reproducible with
`crates/nf-view/examples/probe_0a_rain.rs`). The office lounge is cel bounds x −42.44..−10.72,
z −38.77..−13.82 (probe_0a_rain2 dump); rainbox5's east face is −42.27 and the reported spot is
**4.77 m east of any box**.

The port's `Drops` (crates/nf-game/src/env.rs) modelled the boxes, the cube clip, the footprint
gate and the kill height exactly — but its per-tick step was `vel + wind` only: **it missed
`DAT_003194f4 = −0.5`**, so port rain fell at the table's snow speed (0.025–0.1/tick). A 22.5 m
fall then takes 150–900 ticks and the per-slot slant drifts drops up to ~6 m: measured pre-fix
(probe_0a_rain2, 300 ticks at the office state) the drop bounding box reached x −37.63 — 4.6 m
east of rainbox5's face, inside the office — with the near-office count growing over time.

## The fix

`crates/nf-game/src/env.rs`: `Drops` gained `fall: f32` (`DAT_003194f4`), set −0.5 for kind 1
and 0 otherwise [M 0x18bf68], folded into the per-tick step's y [M 0x18c580]. Rain now falls at
the game's ~0.55/tick, drift stays ≤ 0.86 m, and no drop reaches the office.

## Verification

- `crates/nf-game/src/env.rs` unit tests: `rain_falls_at_the_game_speed_and_stays_in_its_box`
  (the −0.5 step pinned exactly; 600 ticks of drops never leave their box by > 1.0 m) and sister
  `snow_keeps_the_slow_fall_and_drifts` (snow keeps DAT_003194f4 = 0, still drifts past 1 m).
  `cargo test --offline --release -p nf-game --lib env::` → **6 passed, 0 failed**.
- `crates/nf-view/tests/fix_0a_rain_indoors_live.rs` (needs NF_ISO), through the real load +
  `Sim::step_bond` path: `no_rain_draws_inside_the_office_lounge` (the [D] census: kind 1, seven
  boxes, fall −0.5; 301 ticks at the reported state; zero alive drops at x > −41.0, which is
  ≥ 1.3 m inside the office from the nearest box face — unreachable under the [M] drift bound)
  and sister `rain_still_draws_outside_its_boxes` (rainbox5's strip west of the office wall and
  rainbox1's courtyard strip at the spawn stay populated). → **2 passed, 0 failed**.
- Pre-fix reproduction on the same paths: probe_0a_rain2 showed drops inside the office region
  (above), and the unit drift bound fails without the fix (drops drift 3+ m).
- Headless render spot for the main agent: `--bond -37.50,1.11,-16.95,2.827,-0.092` after ~300
  frames (`nf-view 0700000a --shot out.png --bond -37.50,1.11,-16.95,2.827,-0.092 --frame 300`) —
  expect no streaks in the lounge; the west windows still show rain outside.
- Fast suite runner (`tools/test/suite.py`) started at 05:46 UTC on the session-start tree
  (origin/master 38f1bc93, before this fix): build ok, 792 binaries in 1489 s. Failures seen so
  far on that baseline tree: 3 `disc_golden` (allowed on the slim ISO) + `identity_disc::
  the_real_disc_passes_the_identity_check` + `probe_render_pylon_only` — **master's baseline, not
  this branch's** (the runner's build predates the fix; no time to re-run 792 binaries inside the
  window). This branch's touched crates were run directly (above). The next session should re-run
  the suite on the merged tree.

## What's left

- Re-run the fast suite on this branch merged with origin/master; confirm `identity_disc` /
  `probe_pylon_render` against the baseline.
- [L] live PCSX2 confirmation would pin the game's indoor-rain absence and the rain fall speed
  (see below).

## Live checks for the main agent

1. At the office state (Bond −37.50, 1.11, −16.95, yaw 2.827), watch `DAT_003194f4` (ACTION.ELF
   load address 0x003194f4): expect −0.5 (0xbf000000) during 0a rain, 0 during 0b/05 snow — read
   access from `UpdateDrops` 0x18c580.
2. A rain drop's per-frame Δy near the office: expect ≈ −0.5…−0.6 per frame (fall + vel.y), not
   −0.025…−0.1 (watch any `DropVector` slot at 0x2a8100 + 16·i, frame-to-frame).

## Usage

not visible to me (this session exposes no token, context-window or cost figures; nothing to copy).
