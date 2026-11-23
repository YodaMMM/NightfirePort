# retune-nb-08-26 — 08 whole-mission 00 Agent real-damage walk, round 26

Task: get `walk_the_whole_mission_on_real_input_real_damage_00agent` green again (round 26).
Branch: `s/retune-nb-08-26`, created from origin/master (7562f1e70) with s/retune-nb-08-25 merged
(d8aed67e0 — the round-25 ledger/report). The round-23 shape was the starting point (death
t29609). The walk is still red, and it is now **709 ticks deeper** with the round's lever
shipped.

## What was tried (each lever judged on its own real run)

The ledger's lever (1): **THE ARMOUR ENTRY** — taken as ledgered first, then re-derived on the
run's own measurements. Every run went through the real input path from the level load, no
top-ups, no damage/health changes, no guard-code changes, every arm d3-gated (the sisters'
timelines untouched by construction).

### 1. The boarding-pair premise — MEASURED DEAD (the ledger's drain is somewhere else)

An instrumented trace (armour probes at the boarding, ride, drop-block, standoff, reload and
pane-window landmarks — eprintln only, the timeline byte-exact) settled where the fight entry's
20.0 armour comes from [L the 2026-11-23 armour-trace run]:

- The ride pays **ZERO**: armour 50.0 at every ride stamp t0/t300/t600/t900/t1200.
- The boarding/ride windows carry **no fights** (no [real-damage] rows between the ledge pickup
  and the drop block).
- The 30.0 the round-25 ledger charged to the boarding pair actually drains while the walk
  **STANDS at the rim** through the scoop/arm/switch/reload + pane windows: 50.0 -> 42.0 at
  reload f27229 -> 20.0 at the fight entry — the standing capsule (feet 13.470) is back on the
  step edge the hurt volume's zap band covers, while the pit floor (feet 13.086) is the zap-free
  plate [L the round-5/8 runs].
- The armour pickups hard-set 50.0 [M Pickup_Handler 0x18e650; pickup.rs take_ex 732-733: type-3
  armour refused at armour >= 50.0, else `*armour = 50.0`], so no pre-pickup saving can raise the
  entry — the ride-window drain is the only term, and it is not guard fire.

So the ledgered shape (kill the boarding pair for < 18.0) is dead as written: the pair is not
the drain, and the pickup cap pins the entry at 50 − tail drain.

### 2. The armour entry RE-DERIVED and SHIPPED — the crouched tail

The tail stays **crouched** through scoop/arm/switch/reload + pane shooting (the tail stand
press skipped, d3-gated, in `walkreal_07000008_live.rs` only): the pit floor is the zap-free
plate, and the pane aims recompute from the crouched eye each tick (the pane loop's own
yaw/pitch err). MEASURED [L the lever-1 run]:

- ALL panes SHOT OUT after 1 round from the crouch — the glass shoot costs nothing.
- The fight opens **armour 44.0 / health 21.0** (was 20.0/21.0).
- The fight's first health row moved t28034 -> t28821 (the 44.0 paid ~1200 fight ticks of grind
  before health paid a row).
- The death moved **t29609 -> t30318 (frame 30320) at (112.80,13.57,-177.29)** — **+709**, the
  deepest measured shape since round 23. The closing killer: the machine's **-13.90 missile
  blast** at t30184 (health 18.1 -> 4.2) at (113.0,13.5,-180.9), caught standing off the pit
  (the shipped dodge walked Bond ~2 u north, z -179.3 -> -177.3).

### 3. The crouched FIGHT OPEN — tried, measured shallower, reverted

The dive-stand press skipped (the fight stays on the zap-free pit floor): death **t29696** —
+87 over the shipped t29609 and **622 SHALLOWER** than the shipped-stand run. The rows tell why
[L the lever-2 run]: the machine's close-circuit grind ran -2.00 × 10 + -1.00 over just 370
ticks (t29327-t29696) — the crouch holds Bond ON the pit where the close circuit's volume pays
fastest, while the shipped stand's fight drives the dodge episodes that spread the rows. The
dive-stand press RESTORED.

## Byte-exactness proof

- The whole `walkreal_07000008_live` binary on the final tree: **6 passed / 0 failed / 1
  ignored** (267.11 s) — the d1/d2 sisters, the plain walks and both ally tests byte-exact; the
  round-23 signature exactly [L the final-tree run].
- The 00agent walk re-run on the post-ledger tree: FAILED (red) at the shipped shape [L the
  re-verify run].

## What was run

- `walk_the_whole_mission_on_real_input_real_damage_00agent` five times on this sandbox (the
  armour-trace baseline, lever 1, lever 2, the final-tree confirmation, the post-ledger
  re-verify) — each through the real input path from the level load.
- The whole `walkreal_07000008_live` binary on the final tree: **6/0/1**.
- Nothing under `crates/*/src/` changed; the only changed file is the 08 test file, used by
  exactly one test binary, which ran clean whole (6/0/1). The shared helpers (walker.rs,
  damage_log.rs) are untouched, so the other levels' walks are byte-unaffected by construction.
- The fast runner (`tools/test/suite.py`) not completable in this sandbox: background jobs are
  killed between tool calls (the sandbox's own rule — a background build started first thing
  died within one tool call, re-verified) and its build step ENOSPCs on this 9.9 GB disk (the
  round-24/25 documented constraint — ~5.4 GB of binaries on 5.8 GB free). The run step is
  replicated fresh on the final tree by the whole 08 binary above; round-24's 165-binary
  replication (823/19/48, all 18 unique fails the slim-ISO driving-VIV stubs) remains valid for
  the untouched crates.

## What's left (for round 27)

- The two ledgered levers in the ignore message:
  1. **THE -13.90 BLAST** — the lever run's closing killer is the missile blast caught standing
     off the pit; the pocket-dodge class at this anchor is measured dead (the round-25 verdict
     stands), so the answer is a drive-off law (the dodge must not leave the pit floor while the
     machine is close — score, not forbid, its own run) or the salvo-source far-side hold.
  2. **THE SALVO SOURCE live check** for the main agent (below).
  3. The fight-entry pane drain (even crouched the pane windows cost 6.0 through the SHOT-OUT
     panes) — shooting the panes last, or from the east band out of the volume's arc, may keep
     the full 50.0 at the entry (its own run).
- The full four-crate suite on a sandbox with the disk for the whole build (the round-24/25/26
  documented ENOSPC constraint).

## Live checks for the main agent

1. **The salvo source** (the carried ledger item): with the game running the 08 fight, gdbwatch
   `Copter_Update` 0x17dbb8 case 1's attack-state open against the route-path index that
   `Copter_Move` 0x17d628:63-113 derives from the damage fraction (route_path_count *
   damage/(0.9*hp) + 0.5). Expected result: whether the index advances at a landing (damage
   ~100, hp 497.6 -> fraction ~0.17) and whether the close-salvo passes follow the advance —
   this decides the far-side hold's shape (item 1's second arm).

## Usage

not visible to me

## Notification

curl timed out at 20 s (`(28) Connection timed out after 20002 milliseconds`), the python
urllib fallback timed out at 30 s, and the wget fallback SUCCEEDED (exit 0) — the notification
was delivered on the third fallback; no "ntfy failed" section required.
