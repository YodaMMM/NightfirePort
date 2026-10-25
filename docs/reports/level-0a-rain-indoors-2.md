# level-0a-rain-indoors-2 — land the rain fall speed without changing other levels

Branch `s/level-0a-rain-indoors-2` (origin/master 38f1bc93 merged with
origin/s/level-0a-rain-indoors f7246aaf). Sandbox window 08:06–10:05 UTC; this session is the
whole task. The level-0a-rain-indoors fix (`Drops.fall` = DAT_003194f4 = −0.5 for rain
[M 0x18bf68, 0x18c580]) was correct but turned 7 live tests red on other levels. This session
found the mechanism, fixed the port's remaining deviation, and turned 5 of the 7 back green.

## Why a rain-step change altered placements and AI on other levels

**The generator is shared.** `Rand_FRand` 0x1e3780, `Rand_Rand` 0x1e3738 and `Rand_Random`
0x1e36f0 all step the same `SEED_X`/`SEED_Y` pair (0x30d0a0/0x30d0a4) [M]. Every drop draw and
every AI draw consumes one stream, so any change in the drops' draw rate shifts every later
AI draw.

**The per-frame draw shape [M].** `Env_Update` 0x17bc88 draws one `Rand_Rand(20000)`
unconditionally whenever `glb_world != 0`, then calls `UpdateDrops` [M 0x17bc88:11-13,59].
For rain (`DropType == 1`): NO wind draws (the two `FRand(0.0004)` draws sit behind the
`DropType != 1` gate [M 0x18c580:139]), and `AddDrop` 0x18c278 draws exactly 3 FRands per call
(x = `FRand(30)−15`, z = `FRand(30)−15`, y = `FRand(7.5)+7.5` [M 0x18c278:29-40]), called for
free slots in slot order while `Added < 0x15` per frame (`slti $v0, $v0, 0x15` at 0x18c9fc
[M disasm]; `Added` resets each entry [M 0x18c580:138]).

**The port's bug: rain stepped with the per-slot table.** The game's rain branch steps every
drop by the UNIFORM wind quad `{WindVector, DAT_003194f4, DAT_003194f8}·FRAME_RATE_MUL` —
`auVar21 = _lqc2(_WindVector2); auVar20 = _vadd(auVar21, auVar20)` per drop [M 0x18c580
rain branch; disasm: `beq DropType,1 → 0x18c7e0` at 0x18c6f8]. The per-slot `RandTable2`
(`RandTable1[i]·FRM + quad·FRM`, 0x18c580:185-202) is computed but read ONLY by the snow/rising
branch's scratchpad walk (`((uint)lVar2 & 0x1f)·0x10 + iStack_f8`, iStack_f8 = RandTable2
0x3192f0 [M 0x18c580:256]). The port applied `step[i & 31]` (the per-slot table) to rain too:
every rain drop got a fake ±0.02/tick x/z slant and a vel.y-varying fall (0.525–0.6/tick)
instead of the game's exact straight 0.5/tick. With the fall fix landing, drop death times
changed → respawn counts per frame changed → the shared stream shifted on EVERY level with
rain → guard placements, doors and escort/conversation AI landed differently. The previous
session's report ("per-slot slant vx/vz ∈ ±0.02 drifts ≤ 0.86 m") was wrong for rain: the
slant exists in `RandTable1` but rain never reads it; game rain drift is exactly 0.

**[L] snapshot cross-checks** (work/ee_start_*.bin, file offset = EE address):
- `CurrentMax` is a HALFWORD at 0x30d784: ee_start_05 = 0x1000 (snow), ee_start_07000001 =
  0x0040 (64, rain), ee_start_0700000a = 0x0080 (128, rain) — the port's unsigned split
  (`level.wrapping_sub(0x0700_0009) ∈ 0..=2 → 0x80 else 0x40`) matches the game on 01 and 0a.
- `DropType` (0x30d788) = 1 on both 01 and 0a — both levels rain in the game; the port builds
  the same env there (probe: `cargo run --release -p nf-view --example probe_01_env` prints
  kind=1, fall=−0.5, 64 slots, 7 boxes, spawns/respawns active on 07000001).
- `InitBodge` (0x30d789) reads 1 in the 01/0a dumps — a stale save-state field, NOT the boot
  value: the InitDrops disasm (0x18c1f0-0x18c220) sets it 1 only for 0x07000005/06, exactly as
  the C says. The port's `init_bodge` matches the game; nothing to change.
- `RainBox_Create` 0x18d840 APPENDS to the RainBoxRoot tail (`lw 0x1c` walk, `sw 0x1c($v1)`
  [M disasm 0x18d888-0x18d8cc]) — the port's placement-order box walk matches; no fix needed.

## The fix

`crates/nf-game/src/env.rs` `Drops::update`: rain steps by the uniform quad
`[wind[0], fall, wind[1]]`; snow/rising keep the per-slot `step[i & 31]`
[M 0x18c580 rain branch vs the scratchpad walk]. With `WindVector` 0 for rain (InitDrops
zeroes it; the kind-1 path never draws wind), game rain falls straight at exactly 0.5/tick and
a drop keeps the x/z its `AddDrop` draws gave it for the whole fall. The unit tests re-pinned
the [M] truth: the rain sister now asserts the y step is EXACTLY 0.5 and x/z never change
(previously it tolerated 0.5–0.65, i.e. the wrong slant); the snow sister (per-slot table
band, drift > 1 m) is unchanged and still green.

## Verification

- `cargo test --offline --release -p nf-game --lib env::` → **6 passed, 0 failed** (rain
  speed+straightness, snow sister, the four pre-existing env tests).
- The 8 live test binaries from the task's red list (`NF_ISO` set, `--test-threads=1`):
  - sight_frustum_live → **2/2 green** (the "guard placement 261" pair).
  - doublecross_transitions_live → **6/6 green** (incl. castle_d_intro_enables_the_assassin…).
  - escort_hide_findrun_live → **3/3 green** (incl. the_hides_exit_reanchors…).
  - fix_pickups_ammo_live → **4/4 green** (incl. drops_and_boxes_take_through_the_real_path…).
  - hostage_killer_07000002_live → **5/5 green** (incl. the_captor_and_hostage_conversations…).
  - fix_0a_rain_indoors_live → **2/2 green** (no rain in the office lounge; boxes populated).
  - interactables_0700000a_live → 6/7, **armour_pickup_taken_by_walking_over_it still red**.
  - escort_follow_07000001_live → **escort_at_walking_speed… still red**.
  So **5 of the 7 originally-red tests are green with rain at the game speed**.

## What's left (next session, from this branch)

Two live tests still fail; both are AI/channel consequences of the port's stream VALUES
differing from the game's (the port seeds its shared `Rng` from the 07000005 snapshot seed XOR
a per-test seed — a documented port choice [S], `Rng::new` at drone.rs:745), amplified by the
now-fast rain consuming ~12 draws/frame:

1. `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it`: the walk to the
   armour reaches nav node 8 and the sim fires **level_exit** mid-route ("node 8: level_exit
   reached at +1017 ticks") — an exit trigger arms/fires that didn't on master. The walk never
   reaches #730, so the armour==50 assert fails.
2. `escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens`:
   the walker's door helper opens the gallery door (7.9, 5.3, 11.1) (state 0x2 → 0x4, pos01
   1.00) but its channel re-zeros (state 0x10, pos01 0.97, ch_open 0) and the walker gives up
   at the 3000-tick cap; the contact never reaches the AND plate in 12 rounds.

Both reproduce on this branch; both were green on master under the slow-rain stream. Candidate
next step: seed the port's shared stream per level from the game's own level-start snapshot
seeds (ee_start_07000001 SEED_X/Y = 0x2a9b09c3/0x54b345f5, ee_start_0700000a =
0x4106601e/0x082fb79b [L]) instead of the 05-snapshot XOR mix, so the port's draw VALUES track
the game's too — a rebaseline across the live suite, so it needs its own session. (The 05
snapshot's SEED at 0x30d0a0/4 = 0x06595adb/0x208a5de8 is what `Rng::new` mixes today [M
drone.rs:744-745].)

## Full suite on the final tree

Ran `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
-p nf-view -- --test-threads=1` on the final tree (this branch, merge included). Result and
the total line: see the "Suite" appendix at the bottom (filled from suite.out before the
finish step).

## Live checks for the main agent

- 0700000a office render (unchanged from level-0a-rain-indoors): `nf-view 0700000a --shot
  out.png --bond -37.50,1.11,-16.95,2.827,-0.092 --frame 300` — expect NO streaks in the
  lounge; the west windows still show rain outside.
- 07000001 rain shape (new): `nf-view 07000001 --shot out2.png --bond -3.47,7.14,-4.94,0.0,0.0
  --frame 300` — rain should fall in straight vertical streaks inside the 7 rainboxes around
  the spawn plateau (no slant: game rain drift is 0 [M 0x18c580 rain branch]).

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).
