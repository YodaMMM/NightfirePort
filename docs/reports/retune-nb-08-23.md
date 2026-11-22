# retune-nb-08-23 — the 08 whole-mission 00 Agent real-damage walk, round 23

Branch `s/retune-nb-08-23` = origin/master (7562f1e70) + `s/retune-nb-08-22` merged, per the
task brief. The merge: the mirrored files (README, sfx.rs, drone.rs, collide_push_live.rs, the
01/0a/14 walkreal tests) took master's landed state (08-22's copies of those diffs are other
tasks' work that landed on master through different commits); the 08 test file took 08-22's
rounds 20-22 line with master's stream re-times applied (the 550/60 re-roll stop wraps the
rounds' lever wrappers; the crouch drop keeps master's game-lockout shape [M Player_SSCrouch
0x1924c0:45-55,84-92]); `damage_log.rs` auto-merged (the round-22 fight_weave intact, 7 refs).

## What's done

Three levers shipped, each judged on its own real run of the real-input walk (no top-ups, no
damage/health changes, no guard-code changes; all changes are inside
`crates/nf-view/tests/walkreal_07000008_live.rs`):

1. **The launch distance cap (l <= 20.0, d3-gated).** The merged-tree baseline died t21550 to
   Bond's OWN round: the t227 launch at l 44.7 (a momentary line through the west wall band)
   blew at (111.0,14.1,-182.1), 3.7 u out, owner -1 kind 0x2c — the documented self-blast
   signature [L the merged-tree NF_DMG_TRACE baseline; the 08-15 note's owner -1 kind 0x2c
   class]. The far launch's 174-tick guided cruise curves into the band the eye ray cleared
   for one tick. The cap (the measured landings all fired into the machine's own reach — the
   low passes dist 17.6-28.9, rarely beyond 16 u [L the round-4 flight logs]) held the launch:
   the self-blast row is GONE, death t21708 (+158).
2. **The dive-stand weave.** The held window exposed the next binder exactly as the round-22
   ledger predicted: #22's 0x15 sniper grind on the standing dive-stance (21.0 in 24 hits,
   full-p rolls on the scripted shape). The fight loop's held window (no dodge driving, no
   launch ready) now holds the round-22 ±0.5/6-tick strafe pattern — the roll reads moving
   ×0.1875 [M DroneWeap_DoBulletAccuracy 0x179740:52-54,71-73; DroneWeap_DoOpponentTargetting
   0x179a48:44-58]. #22's grind fell out of the damage tail (−1.83/−3.00 sparse rows), death
   t22500 (+950 over the baseline).
3. **The 6300/60 boarding re-roll stop** (joins master's shipped 550/60): fight entry armour
   20.0 / health 21.0, death **t29609** at the crouched anchor — +8059 over the merged-tree
   baseline. The sweep pins it as a sharp peak [L the runs]: 6000/60 t21437, 6600/60 t21330,
   6250/60 t22143, 6350/60 t21766, 6300/30 t22632, 6300/90 t25010.

Measured and reverted/ledgered this round:

- The salvo dodge's close band widened to 7.0 (the 6.3-miss wall-band detonation pairs): WASH —
  death t22502, the same deal to the tick. Reverted.
- The weave's launch-band hold-off (no weave at l <= 25, so the aim settles for a launch):
  +188 deeper (t29797) but it STOOD Bond into the volume band during the close passes (the
  standing death class at y 13.47) and ZERO launches still fired (the circuit's 30-tick
  samples never showed a settled l <= 25 bearing — yaw_err 2.5..3.1 [L the run]). Reverted.
- The machine-fight-entry stops: +20500/60 reached t30054 but died in the STANDING west-band
  death class at (111.0,13.47,-180.95) — not shipped; +20800/60 t29503, +21000/60 t27953.

The refreshed ledger (all runs above + the next levers) is appended to the target test's
ledger comments before `fn walk_the_whole_mission_on_real_input_real_damage_00agent`.

## How it was verified

- Every lever judged on its own real run; the numbers above are [L] this branch's runs (the
  logs are sandbox scratch: /tmp/r23base.log, /tmp/r23trace.log, /tmp/r23lever1..5.log).
- The whole `walkreal_07000008_live` binary on the final tree, twice:
  **6 passed / 0 failed / 1 ignored** (the target walk), 274.52 s and 269.89 s. The sisters
  and every other test in the file stayed green across all five lever states (the whole
  binary also ran clean at the two-lever state before the stop shipped).
- The change surface is the one test file; no shared code was touched, so no other walkreal
  binary's timeline can move.

## What's left

The walk is still red: death t29609, the volume's machine-proximity zap rows grinding out the
21.0-health entry over an 8300-tick stalemate with ZERO launches (the machine's circuit never
gave a settled l <= 20 window on this deal). The next levers are in the ignore ledger: (1) the
volume's machine-proximity burst rows — the round-17 live check (is the volume machine-powered?
a gdbwatch on the volume's damage tick vs the copter's range) decides whether ANY local park
can outlast the stalemate; (2) the launch band itself — the aim never settles under the weave;
a settled-aim window needs the circuit's per-tick dist series, not the 30-tick samples;
(3) the upstream banks (the boarding chunk's remaining rows: #5 −6.0, #26 −4.0, the pit pair's
deal variance).

## What was run, and what was not

- Ran: the whole 08 binary twice on the final tree (6/0/1 above); every lever's individual
  real run; `cargo test --offline --release -p nf-view --test walkreal_07000008_live --no-run`
  builds nf-game/nf-formats/nf-audio/nf-view clean.
- Did NOT run: the fast runner (`tools/test/suite.py`) or the chunked fallback. The full suite
  needs ~1.5 h+ and the background runner is killed between tool calls in this sandbox (the
  rounds 13-22 kill pattern, re-measured this session); the merge + three lever rounds
  consumed the window. The touched-code rule is satisfied: the whole change is the one test
  file and its whole binary ran clean twice on the final tree.

## Usage

not visible to me.

## ntfy

In order, 2026-11-22 ~22:41 UTC:

1. curl: `curl: (28) Connection timed out after 20002 milliseconds` (rc 28).
2. python3 urllib (timeout=20): **exited 0, no exception** — treated as delivered; the confirmation print was not captured by the shell pipeline. Note: in the previous session (walk-1b-agent, ~21:58 UTC) all three channels timed out from this sandbox, so ntfy.sh reachability here is intermittent; the python attempt is the only one that returned cleanly.
3. wget: not needed (the python fallback exited 0).
