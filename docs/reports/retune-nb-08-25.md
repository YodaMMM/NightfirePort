# retune-nb-08-25 — 08 whole-mission 00 Agent real-damage walk, round 25

Task: get `walk_the_whole_mission_on_real_input_real_damage_00agent` green again (round 25).
Branch: `s/retune-nb-08-25`, created from origin/master (7562f1e70) with s/retune-nb-08-24 merged
(7e7a96ef8 — the branch-point tree already contained the round-24 merge and its test-file ledger
commit 25bcc2a4e). The round-23 shape is the starting point: death t29609 at the crouched anchor.

The sandbox was reset mid-task; the setup was redone (clone, work-slim, the onfoot ISO, rust,
cargo fetch, the queue worktree) and the branch + WORKLOG were pushed first.

## What was tried (each lever judged on its own real run)

The ledger's lever (1): **the dive-window launch package re-armed WITH the aggro answer** — six
real runs (v1–v6), every one through the real input path from the level load, no top-ups, no
damage/health changes, no guard-code changes, all arms d3-gated (the sisters' timelines
untouched by construction).

### 1. The draw settle — TRIED and PROVEN (the round-24 disproof does not hold)

The round-24 verdict claimed the draw finishes before the fight loop opens and the settle adds
zero ticks. The v2 run's `[08 draw]` line measured **switch_state 3 at f27455** — the loop opens
MID-DRAW, and tick_fire's drawing guard ("a press under a draw is spent on the view-state gate"
[M Player_WeaponFiring 0x1a5460; walker.rs tick_fire]) ate every fire press of the dive's
all-green t10–t35 window (the `[08 gates]` t10 row: line_ok/slit/eye_sees/lane/cast/closing/
band_x/dive_win ALL true with cp[1] 15.39 — zero launches in the whole run [L the v1/v2 logs]).

With the settle re-shipped (hold the aim, tick until `switch_state == 0`, d3-gated), the round-24
launch reproduced EXACTLY [L the v3–v6 logs]:

- `[08 draw] settled f27490 switch_state 0 after 35 settle ticks` — the round-24 shape;
- `[08 rocket] t18 launched — dist 42.8` — the dive geometry admitted what the l <= 20 cap refuses;
- `[08 rocket] t211 gone — damage 100.4` — the first Hind landing since rounds 11/13, hp 598 → 497.6.

### 2. The aggro answer — TRIED in four shapes, ALL DEAD by the depth criterion

| shape | death | measurement |
| --- | --- | --- |
| v3: the wall-read cast (dodge only rounds whose line the world leaves clear) | t28369 | the cast DISARMED the drive: `segment_flight` from the round to the chest returned a block the round's own flight refutes (it detonated AT Bond through the same line); Bond frozen at (113.08,13.57,-179.01) through the t830–870 approach, dh 24 held by the cannon rounds; the single -13.00/-8.00 pair at miss 0.59 [L the v3 log] |
| v4: the miss-hat pick (the away-perpendicular of the round's line) + the widened drive bands (miss < 6.0, detection from the spawn tick d <= 60) | t28717 | the drive ran but the kill still landed: -13.00 then -8.00 [L the log] |
| v5: + the wedge re-pick (the deterministic pick loops back into the same wedged bearing) | t28821 | -10.85/-3.23/-6.92 spread over three salvos at miss ~1.0–1.06; the standing strafe grinds ~0.03/tick [L the log] |
| v6: + the per-salvo wedged-side exclusion | t28821 | byte-identical to v5 — the grind is under the 0.15-u window threshold [L the log] |

**The measurement that settles the class:** the anchor pocket is ~0.5–0.8 u across and the
standing strafe grinds ~0.03/tick against the plate lip — the dodge CANNOT build the miss >= ~6
the ledger asked for at this anchor (the round-7 episodes' 0.7–2.3 u were the unwedged episodes;
the aggro salvos' bearings wedge both sides). The stalemate's shield IS the machine's distance
— re-measured through six runs.

Every behavioural arm was reverted; the tree's behaviour is byte-identical to the round-23 tree
(re-confirmed below). The verdict comments live at the gates (`ready`, the fight header), and the
ignore message carries the round-25 block with the next levers.

## Byte-exactness proof

- The reverted tree re-run on the 00agent walk: **death t29609 (frame 29611) at (112.95, 13.09,
  -179.32)** — identical to the branch-point shape [L the revert run].
- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 0 failed / 1 ignored**
  (274 s) — the round-23 signature exactly [L the run].

## What was run

- `walk_the_whole_mission_on_real_input_real_damage_00agent` seven times on this sandbox
  (v1–v6 plus the revert proof) — each through the real input path from the level load.
- The whole `walkreal_07000008_live` binary on the final tree: **6/0/1**.
- The seven walk/08 canaries on the final tree — **28 passed, 0 failed, 2 ignored**, the
  round-24 signature exactly:
  - `walkreal_0700000c_eastroute_live` 1 passed;
  - `walkreal_0700001b_live` 17 passed / 2 ignored;
  - `walkreal_probe0c_barrelrow_live` 1 passed;
  - `walkreal_probe0c_door6_live` 1 passed;
  - `walkreal_probe11_48_live` 2 passed;
  - `zipline_07000008_live` 1 passed;
  - `pickups08_live` 5 passed.
- The fast runner (`tools/test/suite.py`) not completable in this sandbox: the background runner
  is killed between tool calls (the sandbox's own rule) and its build step ENOSPCs on this
  9.9 GB disk (the round-24 report's documented constraint — the four crates carry ~1300 test
  targets, ~5.4 GB of binaries). Its 4-minute leash wrote ~1.5 GB of partial artifacts and died
  (`ps` shows no runner after the first tool call). The run step is therefore replicated fresh
  on the final tree: the whole 08 binary + the seven canaries above; round-24's 165-binary
  replication (823 passed / 19 failed / 48 ignored, all 18 unique fails the slim-ISO driving-VIV
  stubs) remains valid — the only source this tree touches is the 08 test file, comments and
  eprintln text only since the revert.
- Nothing under `crates/*/src/` changed; the changed file is used by exactly one test binary,
  which ran clean whole (6/0/1). `nf-game`/`nf-formats`/`nf-audio` sources are untouched.

## The ledger (pushed into the ignore message)

1. **THE ARMOUR ENTRY** (the 08-23 item (2), still untried): the fight's zaps drained the entry's
   20.0 armour before the salvos landed and the salvo rows met HEALTH 21 (-13/-8 = death, the
   v3 run's exact rows). The boarding pair's drain (30.0) is the gate: kill the boarding pair
   for less than 18.0 armour (the deck-head arm-up shape; the walkway pair already dies at ZERO
   cost) and the entry keeps ~32+, the salvo rows meet armour, the stalemate grinds to its
   deeper death or another landing.
2. **THE SALVO SOURCE** — a live check for the main agent: gdbwatch the machine's attack-state
   open (Copter_Update 0x17dbb8 case 1) vs its route-path index (Copter_Move 0x17d628:63-113,
   the damage-fraction advance) across a landing. If the landing's fraction re-rolls the route
   into the close-salvo class, the launch must be delivered while the machine sits FAR (the band
   between — its own salvos then pay the -2.00 x2 residue instead of reaching the chest).
3. **The pocket dodge at this anchor is MEASURED DEAD** (six runs) — any further dodge lever
   needs a different anchor (the north-strip variant was measured worse, the round-5 ledger) or
   a wedge-free bearing the plate lip does not cover.

## Live checks for the main agent

1. **The salvo source** (ledger item 2): with the game running the 08 fight, gdbwatch
   `Copter_Update` 0x17dbb8 case 1's attack-state open against the route-path index that
   `Copter_Move` 0x17d628:63-113 derives from the damage fraction (route_path_count *
   damage/(0.9*hp) + 0.5). Expected result: whether the index advances at the landing (damage
   100.4, hp 497.6 → fraction 0.168) and whether the close-salvo passes (the t830–t1330 rows)
   follow the advance — this decides the far-side hold's shape.

## What's left (for round 26)

- The two ledgered levers above; the armour entry is the cheapest concrete gate.
- The full four-crate suite on a sandbox with the disk for the whole build (the round-24/25
  documented ENOSPC constraint; or the runner's build step taught to skip examples — a
  shared-file change, not made here).

## Usage

not visible to me

## ntfy failed

All three fallbacks failed (2026-11-23 06:07–06:11 UTC, this sandbox could not reach ntfy.sh —
the same constraint round 24 documented):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-08-25: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds` (rc 28).
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'retune-nb-08-25: done ...', headers={'Title':'NightfirePort'}), timeout=20)"`
   → the python call exceeded its 40 s shell timeout and was killed (no response from ntfy.sh).
3. `timeout 25 wget -q -O- --post-data "retune-nb-08-25: done ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → rc 124 (timeout, no output).
