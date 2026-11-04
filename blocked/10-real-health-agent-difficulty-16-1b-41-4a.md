## Your task: `real-health-agent-difficulty-16-1b-41-4a`, finish levels 16, 1b, 41, 4a on real health at Agent

Branch from origin/master, then merge origin/s/real-health-agent-difficulty-01-07 in if it is not on master yet. It adds the difficulty-parametrized load (`load_with_carry_difficulty(..., d)`, the front end's launch path [M C_SBNFDFCTY 0x216ca8:15 → GameState+0x28]); read its report docs/reports/real-health-agent-difficulty-01-07.md first. Agent = difficulty 2 and 00 Agent = difficulty 3. The pick scales Bond's pain (Player_HandlePain 0x1902c8, Plr_DMod_*) and the guards' hit chance (DroneWeap_DoBulletAccuracy 0x179740:74-88). It also spawns the min-difficulty placements (Drone_Create 0x136be0:49) and changes the stats the difficulty rows set [M].

Add an Agent-difficulty sister (`..._agent`) of the real-health walk in walkreal_070000XX_live for each of 16, 1b, 41, 4a: the same route and the same end, loaded at difficulty 2 the menu way, asserting `Drones::difficulty == 2` at load. Where Bond dies or wedges, read the damage log against the game's per-difficulty pipeline. Fix the port where it differs, with the address. Otherwise improve the walk's tactics: cover, pickups, kill order, timing. Real input only: no health, ammo or channel writes, and no teleports. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Every new sister must pass, and the existing Operative walks stay green. Run each touched walkreal_*_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-agent-difficulty-16-1b-41-4a`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-03T23:02Z

---
## Report (blocked, 2026-11-04T00:15Z)


---
## Report from the worker (blocked, 2026-11-04T00:15Z)

# real-health-agent-difficulty-16-1b-41-4a — Agent sisters for levels 16, 1b, 41, 4a

Branch `s/real-health-agent-difficulty-16-1b-41-4a`, cut from origin/master 38052bf6
(2026-11-03 23:08 UTC; origin/s/real-health-agent-difficulty-01-07 is already merged —
merge-base confirms, so no extra merge was needed). **Status: blocked (not finished) — the
four sisters are wired and verified where they pass; 1b and 16's Agent sisters are still red
(tactics, not port bugs); the full suite did not fit.** The next session continues from the
branch.

## What's done

**The pin [M]** (from the merged 01-07 work, re-used as-is): Agent = GameState 2 via
`C_SBNFDFCTY 0x216ca8:15 → GameState+0x28 → Drones::difficulty`; the per-difficulty damage
scales are already in the port (`Player_HandlePain 0x1902c8` × Plr_DMod_*,
`DroneWeap_DoBulletAccuracy 0x179740:74-88`, `Drone_Create 0x136be0:49`'s min-difficulty
placements). The four walk bodies were parametrized `_at(difficulty: u32)`; the difficulty-1
call sites keep the master's load expression byte-for-byte; the sisters load via
`load_with_carry_difficulty(..., 2)` — the menu way, real input only, no health/ammo/channel
writes, no teleports — and each body asserts `Drones::difficulty == difficulty` at load. The
original `#[test]` surfaces are preserved (plain wrappers call `_at(1)`).

**Sister results (real health, real input):**

- **16 `walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent` — RED (death)**.
  Bond dies at tick 8493/8571 near the chaser fight: 43-44 hits for 100.0. The ledger is the
  game's own math — the corridor guards' AR rounds at the records × Normal (drone #3 35.7 in
  17 hits, #5 16.8/4, #2 10.5/5, #1 8.4/4 = ~86 on the APPROACH) plus the pit-door guard
  (skin 0x5000093, weapon 0x14) −15.0 through the chaser stand. Landed on the branch: the
  pit-door guard is pre-shot before the chaser (found by proximity to the death log's spot,
  gated `difficulty >= 2`). The approach's dragged fights (17 hits from one guard = the fight
  never won the trade) are the remaining work.
- **1b `walk_arrival_to_drakes_defeat_on_real_health_agent` — RED (death), three documented
  iterations**: the Drake fight's aim pick ordered Drake (r−50) over threatening wave
  astronauts (r−20), and the wave ring ground Bond down (43 hits for 100.0). Iterations on the
  branch, all opt-in via `waves_first = difficulty >= 2` inside the level's own file:
  waves-first scoring → 41 hits; the exposure-scored shelter hold → Drake starved (the cap,
  no shot — a prio-blind hold is now penalized 50); the shelter = `hide_and_snipe`'s cover
  point as the Drake hold → 39; the Drake-close band (r < 8 → r−60; his weapon 0x30 gun lands
  −14.0 = the record × 0.7 — four waves' worth per hit, measured at 4.1 u twice) → **32 hits,
  death at 10695** (from 8705/43). ~8-15 health short; the next lever is Bond's own dps
  (the reload cadence) or holding the waves' sight-lines longer.
- **41 — GREEN, both sisters** (`walk_from_the_plate_to_the_tether_crossing..._agent`,
  `the_plate_first_walk_real_damage_agent`): the routes complete at Agent with zero damage
  (no armed hazards on these lanes at either difficulty).
- **4a — GREEN** (`ride_the_lift_from_the_spawn_and_walk_into_the_lobby_real_damage_agent`):
  the ride completes at Agent with zero damage, 0 hits.

**No port divergence found.** Every hit in the four death/damage logs is the game's own
per-difficulty pipeline: 1b's astronauts 5.0 × Plr_DMod_Normal 0.7 = 3.5 [D weapon 0x6a +0x0c;
M 0x1902c8; 0x1d3a80 case 0x700001b — Easy 0.6/Normal 0.7, Accuracy Easy 0.6/Normal 0.8], the
armour-first pain path [M 0x1902c8:100-105], Drake's −14.0 = the 0x30 record × 0.7 [D]; 16's
rounds the records × Normal. Nothing to fix in crates/*/src.

## Verification

- The four touched binaries ran on the branch: walkreal_0700004a_live 3/3, walkreal_07000041_live
  6/6 (both filters' three tests each: plain, real_damage, agent), walkreal_0700001b_live the
  real-health pair (d1 GREEN at 84.0 health / 22 hits, the Agent sister RED as documented),
  walkreal_07000016_live the real-damage pair (d1 GREEN at 34.2, the Agent sister RED).
- All four d1 walks are green on the branch — the Operative surface is unchanged (the
  difficulty-1 call sites are byte-identical; the tactics are gated `difficulty >= 2`).
- **The full 4-crate suite did not fit.** The chain of events is in the WORKLOG: the
  background `tools/test/suite.py` was killed between tool calls (the sandbox's known
  background-job behaviour, empty suite.out); the chunk runner's `CARGO_TARGET_DIR
  /home/z/target-cache` build filled the disk (100%, 3.8 MB free — the linker failed; the
  cache wiped, 4.5 GB back); the repo-target 4-crate `--no-run` build alone exceeded the
  window's remainder (10 min at -j4, killed mid-build). No shared helper changed
  (crates/nf-view/tests/common/ is untouched; no crates/*/src changed), so the blast radius is
  exactly the four level files above — the other levels' walks are untouched by construction.
  The next session should run `tools/test/chunk_suite.py` (re-invoking until ALL DONE) with
  `CARGO_TARGET_DIR` pointed at the repo target and enough disk.

## What's left

1. 16's Agent sister: the approach fights' attrition (~86) — the dragged fights need the
   fight machinery's study (cover between trades, or arming the guards' own drops earlier);
   then re-run with the pit-door pre-shoot already on the branch.
2. 1b's Agent sister: ~8-15 health short — Bond's dps in the Drake stand (the reload cadence:
   the fight recharges at clip < 12 to 60 — a shorter dry window), or holding the waves'
   sight-lines down through the kill.
3. The chunked full suite (the total line), then re-verify the four binaries whole (no ignores
   were added — the sisters are new tests).

## Live checks for the main agent

None required — every observed hit matches the port's per-difficulty pipeline at the game's
addresses. If the 16 approach fights stay unwinnable after the tactics work, a live check
would be: the corridor guards (skins 0x500008c/0x50000b2/0x50000b1) at Agent on the real game
— whether the player is expected to fight them at all on the corridor leg (their alert states
at Bond's passing) — that would tell whether the route should be stealth-shaped instead.

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
