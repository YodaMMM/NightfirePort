# Guard radii and glass: join the three lines

Three lines carry this work and none contains the others: `origin/s/guard-batch-repin`, `origin/s/guard-radii-01-d3`, `origin/s/guard-radii-16-chasers`. Branch from the first, merge the other two and current `origin/master`, push to `s/guard-lines-finish`.

Read blocked/10-guard-batch-repin.md for the two non-walk reds:
1. `enemy_accuracy_live astronauts_hit_like_the_chain`: diagnosed as the harness's stream-calibrated expectation, because rounds passing glass consume the shared RNG stream. Fix the harness so it measures acquisition without depending on that stream position.
2. The kiko pit-squad shoot red: reproduced, not diagnosed. Find the cause and cite it.

Then run the walks for 01, 16 and 1b plus enemy_accuracy_live and quote counts. Keep the d2/d3 ignores on 16 unless they go green on their own.

---
claimed-by: sandbox1 at 2026-11-06T07:44Z

---
## Report (done, 2026-11-06T09:34Z)


---
## Report from the worker (done, 2026-11-06T09:34Z)

# guard-lines-finish

Branch `s/guard-lines-finish` = `origin/s/guard-batch-repin` merged with `origin/s/guard-radii-01-d3`,
`origin/s/guard-radii-16-chasers` and current `origin/master` (cd758f109). The WORKLOG conflict kept both
lineages' entries (the guard-batch lineage had created the file fresh; nothing dropped, nothing reordered).

## Done

1. **The astronaut red (`enemy_accuracy_live astronauts_hit_like_the_chain`) — fixed in the harness.**
   The blocked report's diagnosis held up: the pre-branch stream is what the 3..=4 expectation was
   calibrated on, and the fire-path glass pass shifts that stream (the glass-passed 0x6A rounds live
   their 14400-tick fuse through the settle and window, their DoTrails draws consuming the shared RNG
   the sight-acquisition rolls ride [M Bullet_init 0x12c5a0:199-211; Bullet_DoTrails 0x12d890;
   Collide_Update 0x1ea398:216-229]). Reproduced first (diff 2 d 5: 1 shots in 10 s), then
   `measure()` in `crates/nf-view/tests/enemy_accuracy_live.rs` now resets the shared RNG to a fixed
   seed and clears the settle's stray rounds at the window start — every case measures the chain from
   the same stream state, independent of what flew before it. Consequentials re-derived, not guessed:
   - the 21-cell pin matrix re-measured on the fixed stream (values moved one calibration step, e.g.
     0x12 diff-3 5-u 24 → 29); the pins are harness measurements ± max(2, 20 %), no [L] pin touched;
   - the rifle test's "misses are landing" bound now follows the roll law: a rolled miss still flies
     the wobble-offset bullet and can clip Bond's bounds [M 0x179740's miss arm], so landed hits may
     exceed rolled hits but never the roll count (`hits ≤ rolls`, the Tally carries `rolls`); and the
     landed bound's slack is ±4 for the acquisition gate (a rolled-true shot beyond TooClose clears to
     a miss while `seen_ticks < NewSighting_TimeToHit·60` [M 0x179740:93-99] — the window opens on
     exactly those early shots; measured 18/22 on the diff-3 20-u cell).

2. **The kiko pit-squad shoot red (`kiko_button_real_walk_live.rs:319`) — diagnosed and cited, fixed
   test-side.** Chain of evidence:
   - Kill trace: member #14 sits at hp 10.0 for the whole 10800-tick budget, `last_dmg 0.0` — not one
     hit. Gun-state trace: Bond holds weapon **0x1 (the fists), clip 1 spare 0**; the view converges
     (err ±0.0005 for hundreds of ticks, blocked false) and the fire script plays (`fire_state`, the
     23–30-tick swing cycle), but the fists' hit bullet has range **1.8 u** [L weapon_data] and the
     member stands 4.4–5.7 u off in its AIM_STAND_FIRE (0x68) / STEP_AIM (0x73/0x74) standoff —
     the guardroom-guard invulnerability the 07000006 walkreal fought (level-06-guardroom-guard).
   - Why the fists: the carry hands weapon 0x47 (the Ronin pseudo) at load, which instantly swaps to
     0x1 at tick 0 on EVERY tree (master included). Master's green run rode a pit member dying
     ADJACENT to the shelter door whose weapon drop (0x14, clip 21 spare 50) auto-equipped Bond via
     the pickup take (the instant 0x1→0x14 swap at tick 9477) — after that Bond mows the squad.
   - Bisect: reverting ONLY the five `glass_pass` sites in drone.rs turns the test green on the merged
     tree (11.5 s). The merged tree's guard-side glass pass (a [M]-verified law) shifts the phase's
     entry state — the return-2 leg already stalls +545 ticks (master +172) from the merged guards'
     positions — so no member dies adjacent to Bond, no drop, no gun, fists forever, budget red.
   - **Not a game-code fault**: the glass law matches the decomp and the fists standoff is the game's
     own combat law. The test rode a drop landing at Bond's feet.
   - Fix (real input, no pos writes): the pit loop calls `w.switch_to_armed("the pit squad")` before
     every target — Bond cycles the wheel to his own owned firearm (the P2K, 7/41) exactly as a player
     would holding fists with a fight ahead. Whole binary 1 passed / 0 failed (11.5 s).

3. **The merges** — all four lines joined; WORKLOG kept whole; every checkpoint pushed
   (5a54317b3 → 127d51114).

## Verified (each binary whole, quoted)

| binary | result |
|---|---|
| `enemy_accuracy_live` | **5 passed / 0 failed / 4 ignored** (394 s) |
| `kiko_button_real_walk_live` | **1 passed / 0 failed** (11.5 s) |
| `walkreal_07000016_live` | **14 passed / 0 failed / 2 ignored** (77.6 s) |
| `walkreal_07000001_live` | **8 passed / 0 failed** (41.0 s) |
| `walkreal_0700001b_live` | **17 passed / 0 failed / 2 ignored** (126.6 s) |

The 16 d2/d3 sisters: forced-run (`--ignored`) and BOTH still red (real-health deaths — the chasers'
measured ledgers from guard-radii-16-chasers stand); **the ignores stay** as instructed.

## What's left

- The full suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view
  -- --test-threads=1`) and the remaining `walkreal_*_live` binaries did not fit before the 09:28 UTC
  stop-work mark (the sandbox expires ~09:48). The five task binaries above ran whole and green; the
  next session should run the full suite plus every `walkreal_*_live` before merging.

## Live checks for the main agent

1. 07000016, fresh boot with the carry channels on: walk Bond to the pit floor's shelter phase and let
   a pit-squad member approach the west blast-room door — does a member die adjacent to Bond in the
   real game's flow (the drop-adjacency the green master run rode)? Watch `switch_channels` around the
   door and Bond's held weapon (BLData weapon id) through the pit phase.
2. 0700001b diff 2: an astronaut 5 u from a standing Bond — the port's post-fix acquisition acquires
   and fires 3 shots/10 s from a fixed stream; the blocked report's gdbwatch ask stands (the real
   game's acquisition timing when Bond stands still settles whether the port's sight roll cadence
   matches).

## Usage

No token, credit, cost or context-window figures are visible to me in this environment (no usage meter
is shown in the session UI or the tool outputs). not visible to me.

---
## Report from the worker (done, 2026-11-06T09:36Z)

# guard-lines-finish

Branch `s/guard-lines-finish` = `origin/s/guard-batch-repin` merged with `origin/s/guard-radii-01-d3`,
`origin/s/guard-radii-16-chasers` and current `origin/master` (cd758f109). The WORKLOG conflict kept both
lineages' entries (the guard-batch lineage had created the file fresh; nothing dropped, nothing reordered).

## Done

1. **The astronaut red (`enemy_accuracy_live astronauts_hit_like_the_chain`) — fixed in the harness.**
   The blocked report's diagnosis held up: the pre-branch stream is what the 3..=4 expectation was
   calibrated on, and the fire-path glass pass shifts that stream (the glass-passed 0x6A rounds live
   their 14400-tick fuse through the settle and window, their DoTrails draws consuming the shared RNG
   the sight-acquisition rolls ride [M Bullet_init 0x12c5a0:199-211; Bullet_DoTrails 0x12d890;
   Collide_Update 0x1ea398:216-229]). Reproduced first (diff 2 d 5: 1 shots in 10 s), then
   `measure()` in `crates/nf-view/tests/enemy_accuracy_live.rs` now resets the shared RNG to a fixed
   seed and clears the settle's stray rounds at the window start — every case measures the chain from
   the same stream state, independent of what flew before it. Consequentials re-derived, not guessed:
   - the 21-cell pin matrix re-measured on the fixed stream (values moved one calibration step, e.g.
     0x12 diff-3 5-u 24 → 29); the pins are harness measurements ± max(2, 20 %), no [L] pin touched;
   - the rifle test's "misses are landing" bound now follows the roll law: a rolled miss still flies
     the wobble-offset bullet and can clip Bond's bounds [M 0x179740's miss arm], so landed hits may
     exceed rolled hits but never the roll count (`hits ≤ rolls`, the Tally carries `rolls`); and the
     landed bound's slack is ±4 for the acquisition gate (a rolled-true shot beyond TooClose clears to
     a miss while `seen_ticks < NewSighting_TimeToHit·60` [M 0x179740:93-99] — the window opens on
     exactly those early shots; measured 18/22 on the diff-3 20-u cell).

2. **The kiko pit-squad shoot red (`kiko_button_real_walk_live.rs:319`) — diagnosed and cited, fixed
   test-side.** Chain of evidence:
   - Kill trace: member #14 sits at hp 10.0 for the whole 10800-tick budget, `last_dmg 0.0` — not one
     hit. Gun-state trace: Bond holds weapon **0x1 (the fists), clip 1 spare 0**; the view converges
     (err ±0.0005 for hundreds of ticks, blocked false) and the fire script plays (`fire_state`, the
     23–30-tick swing cycle), but the fists' hit bullet has range **1.8 u** [L weapon_data] and the
     member stands 4.4–5.7 u off in its AIM_STAND_FIRE (0x68) / STEP_AIM (0x73/0x74) standoff —
     the guardroom-guard invulnerability the 07000006 walkreal fought (level-06-guardroom-guard).
   - Why the fists: the carry hands weapon 0x47 (the Ronin pseudo) at load, which instantly swaps to
     0x1 at tick 0 on EVERY tree (master included). Master's green run rode a pit member dying
     ADJACENT to the shelter door whose weapon drop (0x14, clip 21 spare 50) auto-equipped Bond via
     the pickup take (the instant 0x1→0x14 swap at tick 9477) — after that Bond mows the squad.
   - Bisect: reverting ONLY the five `glass_pass` sites in drone.rs turns the test green on the merged
     tree (11.5 s). The merged tree's guard-side glass pass (a [M]-verified law) shifts the phase's
     entry state — the return-2 leg already stalls +545 ticks (master +172) from the merged guards'
     positions — so no member dies adjacent to Bond, no drop, no gun, fists forever, budget red.
   - **Not a game-code fault**: the glass law matches the decomp and the fists standoff is the game's
     own combat law. The test rode a drop landing at Bond's feet.
   - Fix (real input, no pos writes): the pit loop calls `w.switch_to_armed("the pit squad")` before
     every target — Bond cycles the wheel to his own owned firearm (the P2K, 7/41) exactly as a player
     would holding fists with a fight ahead. Whole binary 1 passed / 0 failed (11.5 s).

3. **The merges** — all four lines joined; WORKLOG kept whole; every checkpoint pushed
   (5a54317b3 → 127d51114).

## Verified (each binary whole, quoted)

| binary | result |
|---|---|
| `enemy_accuracy_live` | **5 passed / 0 failed / 4 ignored** (394 s) |
| `kiko_button_real_walk_live` | **1 passed / 0 failed** (11.5 s) |
| `walkreal_07000016_live` | **14 passed / 0 failed / 2 ignored** (77.6 s) |
| `walkreal_07000001_live` | **8 passed / 0 failed** (41.0 s) |
| `walkreal_0700001b_live` | **17 passed / 0 failed / 2 ignored** (126.6 s) |

The 16 d2/d3 sisters: forced-run (`--ignored`) and BOTH still red (real-health deaths — the chasers'
measured ledgers from guard-radii-16-chasers stand); **the ignores stay** as instructed.

## What's left

- The full suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view
  -- --test-threads=1`) and the remaining `walkreal_*_live` binaries did not fit before the 09:28 UTC
  stop-work mark (the sandbox expires ~09:48). The five task binaries above ran whole and green; the
  next session should run the full suite plus every `walkreal_*_live` before merging.

## Live checks for the main agent

1. 07000016, fresh boot with the carry channels on: walk Bond to the pit floor's shelter phase and let
   a pit-squad member approach the west blast-room door — does a member die adjacent to Bond in the
   real game's flow (the drop-adjacency the green master run rode)? Watch `switch_channels` around the
   door and Bond's held weapon (BLData weapon id) through the pit phase.
2. 0700001b diff 2: an astronaut 5 u from a standing Bond — the port's post-fix acquisition acquires
   and fires 3 shots/10 s from a fixed stream; the blocked report's gdbwatch ask stands (the real
   game's acquisition timing when Bond stands still settles whether the port's sight roll cadence
   matches).

## ntfy failed

All three fallbacks in order, exact output:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-lines-finish: done …" https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "…urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-lines-finish: done …', headers={'Title':'NightfirePort'}), timeout=20)"` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "guard-lines-finish: done …" https://ntfy.sh/nightfire-yoda-8k3x` →
   timed out (killed at the 30 s shell limit with no output).

## Usage

No token, credit, cost or context-window figures are visible to me in this environment (no usage meter
is shown in the session UI or the tool outputs). not visible to me.
