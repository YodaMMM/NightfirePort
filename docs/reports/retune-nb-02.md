# retune-nb-02 — 02 castle 00 Agent real-health walk re-tune (nb-scan-all)

Branch `s/retune-nb-02` (from origin/master 6fe4339a1). The task: re-tune
`walk_the_castle_all_objectives_real_health_00agent` for the 3412a89a3 guard changes
(the spacing scan live on every level + the close-range AltAttack from the fire states)
and un-ignore it. **Not finished: the test is still red, on a LATER death than the one
the ignore recorded.** The geisha-1 phase the ignore died in is now green; the walk dies
past the stair bottom. Everything below is measured on real runs of the real input walk.

## What was measured (the repro, before any change)

A fresh run of the ignored test on the task's tree reproduced the ignore's death exactly:
Bond died t10472 at (-34.68,6.47,3.70) armour 0.0 — on the ch-0x17 plate, during the
walk back to the geisha-1 room [L 2026-11-18 repro1 run]. The watch trace (NF_DRONE_WATCH=5,6,7)
shows what changed under the spacing scan:

- The give-ground retreat (the old v12 lever) no longer funnels the captors. Captain d#5
  holds a step-aim camp at the plate's west side (-36.8..-38.9, 3.6..7.2) and d#6 holds
  the west end (-45.0..-45.6, 15.6..17.8); BOTH cycle 0x5f COMBAT_NO_ROUTE → 0x73/0x74
  STEP_AIM (the scan's own strafe options — ChooseCombatMove reads the scan's side bits
  now [M 0x150c18:58]) → 0x5c COMBAT_NO_SIGHT, for 3100 ticks, and neither ever closes
  into the stand-off ring [L the repro3 watch trace].
- The stand-off loop's blind 300-tick hold_for_sight on the unseen nearest captain was
  the free-fire window: health 87.0 → 39.0 while waiting [L the repro run's fight traces].
- The walk-back to the room then crossed the plate mouth at 0.8 u from the camped captain
  — inside the ≤3.0-u every-round-hits band [M DroneWeap_DoBulletAccuracy 0x179740:113-115]
  and inside the 1.5-u AltAttack range [M 0x15fd88 case 3] — and died there.

## Lever 1 (landed): the plate vantage + the lean-hold duel (d3-only)

`crates/nf-view/tests/walkreal_07000002_live.rs`, inside the `difficulty >= 3` geisha-1
block (the d1/d2 paths are byte-identical — no shared helper touched):

1. After the give-ground walk, re-arm the silenced P99 (0x02 — it still carries ~20 rounds
   [L the fight#7 trace: clip 7 spare 13]; the give-ground fight's quiet fallback had armed
   the dropped 0x1c, whose fire sound is loud [D 0x1de]) and walk back to the pre-plate
   vantage (-35.2, 6.5, 9.7) — the walk's own "north of the ch-0x17 plate" route leg's spot.
   The return path stays z >= 9, north of the plate gap: it never crosses the 0.8-u mouth.
2. In the stand-off loop, replace the blind 300-tick hold with: hold_for_sight 240 → on a
   miss, one lean leg to (-35.2, 6.5, 8.2) (the route's own pre-plate spot) and hold 240
   again → break only if both fail.

**Measured result [L the 2026-11-18 lever-1 run]:** the geisha-1 phase is green — the
vantage duel kills (the stand-off round 0 fired at t10952), the captors die, ch 0x49/0x29/
0x2A/0x88 rise, the walk proceeds through the touch zone, the west-wing armour detour
(armour 50 taken), the stair top, the stair bottom (t27557), the sniper wait (anchored
+60), and dies t27653 at (-36.09,5.92,-0.46) — the lane crossing — with 7.8 health: the
courtyard sniper's −48.0 body round [L the known −48.0 at the crossing, the 00agent ledger].

The new ledger (health at each phase, the lever-1 run):
- t10852 the plate vantage: 87.0 (P99 0x2 clip 7 spare 8)
- t10952 the stand-off round 0: 87.0
- t21876 after the post-loop pursuer sweep fight 0: 51.0
- t23901 after fight 1: 33.0 — d#6 survived, fight 2 gave up on it
- t25734 the west-wing armour detour done: 7.8 health + 50.0 armour (the detour cost 25.2)
- t27557 the stair bottom: 7.8 + 46.4 armour (the sniper wait anchored, −3.6 armour)
- t27653 the lane crossing: death (one sniper body round through 7.8)

## What is left (the next lever, for the next message)

The crossing pool must be ≥ ~48.0 + margin when Bond descends. Two measured sinks:
1. **The post-loop pursuer sweep** (`clear_pursuers` "after the geisha-1 stand-off", radius
   16) fought three pursuers through stand_and_fire caps + loot walks for 87.0 → 33.0 over
   ~13000 ticks [L the lever-1 run's [02] lines]. Lever 2-a: run the sweep's targets through
   the vantage shape instead (settle_aim + short holds, or a tighter radius so the far
   captor #6 is met later from the room approach), or extend the stand-off loop's rounds so
   the sweep finds fewer live shooters.
2. **The west-wing armour detour** cost 25.2 (33.0 → 7.8) but buys 50 armour — net +24.8,
   keep it, but its timing can move earlier (before the stand-off) so the fights run on the
   50 armour instead of draining health to 7.8 before the crossing.

Both levers stay d3-gated and judged on their own real runs, per the task rules. The
ignore message on the test carries this ledger and the next lever.

## What was run (binaries, pass counts)

- `walkreal_07000002_live` (whole binary): **4 passed, 0 failed, 1 ignored** (the re-tune
  target is the ignored one) [L 2026-11-18, 89.7 s].
- `interactables_07000002_live` (whole binary): **18 passed, 0 failed** [L 2026-11-18,
  17.7 s].
- The ignored target itself, run with `--ignored` for the measurements above (red, as
  described).

## The full suite

**The fast runner could not complete in this sandbox.** The sandbox disk is 9.9 GB total;
the ISO (1.0 GB) + work/ + the toolchain leave ~4.6 GB for cargo. `crates/nf-view` alone
has 1111 test files — the full nf-view test build writes them all as ~13-20 MB binaries
(≈ 5+ GB per full build) and every re-link duplicates the set under a new fingerprint dir;
the first build attempt filled the disk mid-link (bus errors at 100% full, verified with
df). I freed caches, pruned stale fingerprint dirs, and rebuilt per-test-binary
(`cargo test -p nf-view --test <name> --no-run`) for everything the task touches. The
suite runner itself ran twice: once to `BUILD FAILED` (disk), once killed by the tool-call
wall-clock mid-build. `nf-game`, `nf-formats`, `nf-audio` test binaries did build (the
build completed once for those three before the nf-view wipe). **No suite total line is
available from this sandbox — the per-binary results above are the verification that ran.**

## Evidence tags used

- [L] every health/position/state figure above: the 2026-11-18 runs of the real-input walk
  in this sandbox (repro1/repro3/lever1 logs, kept out of the commit as scratch).
- [M] the accuracy band 0x179740:113-115, AltAttack 0x15fd88, ChooseCombatMove 0x150c18 —
  cited from the code/decompile addresses as in the test file.
- [D] the 0x1c fire sound 0x1de, the plate trigger quad — from the disc data the test cites.

## Live checks for the main agent

None required for lever 1 (all its facts are sim-measured). For the next lever, if the
post-loop sweep stays expensive: confirm on PCSX2 that a 07000002 captain at ~15 u in the
west wing (d#6, placement 0x45x group) step-aims instead of closing — the port's
ChooseCombatMove now reads the scan's side bits ([M 0x150c18:58-79] + the 0x31C scan), and
a live watch of his state (expect 0x5f/0x73/0x74/0x5c cycles, no 0x68/0x70) would pin the
siege behaviour as game-true rather than a port artefact.

## Usage

Environment figures for this session: input/output token and context-window figures are
not visible to me; cost is not visible to me.
