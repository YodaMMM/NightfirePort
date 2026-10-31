# level-04-interactables-2 — 04: every player-touchable kind behaves like the game

Branch `s/level-04-interactables-2` (from master e4550f2e, merged into the branch at 2a2fcbfc).

## The census (re-run over the disc, [D])

Re-ran the census for `level-04-interactables-2` with a scratch parser over `nf_formats::level::Level::parse`
of FILES.BIN entry `07000004.bin` — main map `010000c5`, **70 placements**, grouped by the kind byte
(`flags & 0xff`, the same word `parsemap_create_dynamic_objects` 0x1d04f0 dispatches on [M]):

| kind | count | player-touchable? |
|---|---|---|
| 0x24 Start | 1 | spawn |
| 0x0F Drone (THE target, skin 0x050001e, death ch 0x54 = param 12) | 1 | yes — the shot |
| 0xD9 ScriptPlayers (0x06000082 story, 0x06000083 intro) | 2 | skipped by the player |
| 0xF0 Pickups (3 × StormUzi kind 0 w18 60rd respawn 2; 1 × BodyArmourHeavy kind 3 respawn 0) | 4 | yes — walk-over |
| 0xEE TouchOnce ("MultiplexOut" out 0x54 → list [0x64], no collision mesh) | 1 | position-independent mux |
| 0xD8 Rigid bodies (2 × rb_whitelamp) | 2 | yes — the shot |
| 0x19 statics 40, 0x21 ripples 4, 0x2A sky 3, 0x2B leafgens 2, 0x32 rainboxes 3, 0x47 room meta 2, 0xDF env 1, 0xF3 trees 2, 0xFC corona 2 | 59 | no (world fx / meta) |

**No doors, switches/consoles, fuseboxes, lifts, locks/keypads, monitors, breakable doors, SimpleScripts,
plates (0xEA), copters or mines exist on this level** — every number matches docs/levels/07000004.md's
2026-10-17 table, so that table needed no correction.

## What was added (and what was already there)

The per-kind coverage across the level's two interactables files:

- `interactables_07000004_live.rs` (this task's file) — now **4 tests**:
  - the two lamp tests from `level-04-interactables` (create/rest bit-identical; the shot that tips the
    body, sister stays);
  - **NEW** `the_uzi_respawn_rearms_after_the_state2_wait` — the pickup kind's respawn tail: the take
    parks the record hidden in the state-2 wait [M Pickup_Handler 0x18e650:341-347], still hidden at 600
    ticks, re-armed takeable (`hidden` cleared, state 1) past 60·2·10 = 1200 ticks [M Pickup_Update
    0x18e148:182-189; L FRAME_RATE_INT 60 in SP]; sister asserts the record rests visible before the walk
    and the disc's respawn word 2 / rounds word 60 [D params 5/2]. Driven by the real walk-over of the
    lawn uzi (−10.66, 4.39, 42.66 [D]).
  - **NEW** `the_armour_refuses_the_take_at_the_cap_then_takes` — the armour kind's cap refusal:
    `Pickup_Handler` 0x18e650:220-225 returns while `BLData+0x8B0 >= 50.0` before the shared tail, so the
    pickup stays visible/takeable; with the cap cleared the same standing contact takes it and respawn 0
    deletes it [M :338-341]. The assassination target is killed through the real fire path first (the
    mission's own step) — see the finding below.
- `interact_07000004_live.rs` (`level-04-interactables`, untouched) — the target-drone shot, the base
  weapon/armour takes, the TouchOnce mux, the ScriptPlayers settle, and the world-fx kinds (ripples,
  leafgens, trees, env, the template-map phantom check).

No port gap was found: both new behaviours are already ported correctly (`Inventory::take`'s
`kind::ARMOUR` refusal at pickup.rs, the respawn tail in `Sim::take_pickup` at drone.rs:15640-15647,
`Pickup::respawn_wait` at pickup.rs:259-269) — the tests now pin them.

## The one finding worth recording

The armour-refusal test initially failed with the pickup consumed (`gone=true`) despite the poked cap.
The take log (`NF_PICKUPLOG=1`) showed `Some(Armour)`: the hostile target stands 3.5 units from the
post-settle stand [D (−2.64, 5.50, 30.08)] and his rounds drop the poked `+0x8B0` below 50 during the
walk, so the take legitimately grants [M 0x18e650:220 reads the live armour]. Not a port gap — the test
now kills him through the real fire path (the mission's own step) before the beats. Note this also means
`interact_07000004_live.rs`'s armour test can be granted *after* taking target damage; its `>= 50`
assertion is compatible either way, so it was left as is.

## Runs and totals (all with `NF_ISO` set, `--offline --release -- --test-threads=1`)

- `cargo test -p nf-view --test interactables_07000004_live` — **4 passed; 0 failed**
- `cargo test -p nf-view --test interact_07000004_live` (regression) — **10 passed; 0 failed**
- `cargo test -p nf-view --test walkreal_07000004_live` — **7 passed; 0 failed**
- `cargo test -p nf-game` — **953 passed; 0 failed** (98 test binaries)

The full 4-crate fast runner did NOT produce a total line in this sandbox: background jobs are killed
between tool calls here (the nohup'd `tools/test/suite.py` died with an empty suite.out), and the
cold-start rebuild of the four crates' test binaries (rustc lld, ~25 min under `-j4` across several
tool-call windows) did not leave a window where the whole 4-crate run could complete before the
sandbox deadline. The three quoted targeted runs plus the full `nf-game` crate (the largest of the
four, 953 tests) are the totals above; `nf-formats` and `nf-audio` were not re-run this session.

## What's left for a follow-up

- Nothing on the census: every player-touchable kind of 07000004 is pinned end-to-end across the two
  files, each through the real input path with sisters.
- Optional: re-run `tools/test/suite.py` to completion on the final tree in a sandbox where background
  jobs survive, and quote its total line.

## Live checks for the main agent

None required — the census came from the disc parse and both new behaviours are pinned by the port's
own sim. If convenient, a fresh-boot check that the intro NIS does NOT consume the BodyArmourHeavy
(#9, (−1.85, 4.37, 46.07)) before player control (watch PICKUPINFO+0x20 / obj+0xF0 bit 0x10 during the
intro) would confirm the cutscene-touch discipline the port's gate (`alive`) stands in for.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the session.
