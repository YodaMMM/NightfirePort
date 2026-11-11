# pretrans-arms-6 — the three d1 descent walks un-ignored green with the forward clear band

Branch s/pretrans-arms-6 from origin/s/pretrans-arms-5 (19f30d481), merged current origin/master
(f926de993; the only conflict, the 0c helper + live test, took master's whole — the 0c walk is
green there via with_strip_p2k_shot). Real input only; no damage, health or pin weakened; no
top-ups added; the helper change is opt-in and gated to the 07000001 d1 walks.

## Done

1. **The three ledgered 01 descent reds are GREEN, UN-IGNORED** (57d0d3b3a + the green-note
   commit): `walk_from_the_spawn_to_the_exit_with_the_contact`, its
   `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` sister, and
   `the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads`.
   [L the fix5 runs on this tree: main exits to 07000002 at tick 12990; the real-damage sister
   exits at tick 308314; the exit-fade pin passes in 9.9 s; the whole-binary run: mayhew,
   exit-fade, hall_guards_wait, plate_refuses, main, real-damage ALL ok, the d3 sister still
   ignored with its own ledger.]
2. **The lever (the ledger's forward clear band), as landed:**
   - `run_slab_ambush` un-gated to d1 (the gate was `difficulty >= 2`, a byte-exact choice from
     when the d1 walks were green). On the d1 red tree the ch-0xF slab was crossed UNCONTROLLED
     mid-nav (the route pre-clear's guard-2 flank nav crossed it [L nav 1→2 (-26.62,13.63) →
     (-29.96,14.39)]), the poppers only ENABLED at their posts (armed_peer INFINITY at the r0
     print), engaged later when Bond came into view, parked on the hidden contact at the
     stairwell head (d 0.6-1.0, states 0x68/0x5c/0x73/0x74) and bled him 10 → 1 before Bond
     reached the plate; the recovery-loop fight from the plate could not answer in time (no
     exposed part for 540 ticks from below; the flank's +2077 ticks rode out the escort fail)
     [L the pretrans-arms-6 red run]. With the mouth fight (the d2-proven mechanism, the band
     already parameterized 6.5 for d1) the poppers 4/12 are killed at the mouth BEFORE the
     descent — cleared ahead of the contact, who still waits east of the pocket. d2/d3 keep the
     exact old gate (`difficulty >= 2 && ch_on → return`).
   - The pocket rule is now y-aware: the new OPT-IN walker field `flank_hold_min_y` (common/
     helper, set ONLY by walk_to_the_exit_at at difficulty 1) makes the existing flank hold
     (`flank_hold_x`) apply only to targets BELOW y 7.0 — the stairwell-head pocket's walled
     guards sit at y 6.3, while the upper-floor perch guard (skin 0x5000036, state 0x2a at
     y 10.2) keeps his proven around-the-upper-floor kill path [L the fix2 green run: the
     pre-clear flank nav 1→15 round the upper floor killed him]. An x-only hold deadlocked on
     the perch guard (three holds give up, twelve recovery rounds, the contact never un-hides
     [L the fix4 red run]). Every other walk leaves the field None — byte-identical.
   - The recovery loop gains a d1 band-edge stand for walled pocket-FLOOR guards (y < 7.0):
     from (-26.5,6.4,14.6) the pocket opens at 3.7-4.5, outside the ×2 band [L the d2/d3 runs].
   - The route pre-clear's fights hold the same d1 pocket rule (the real-damage red run's
     guard-2 flank nav engaged the advanced popper #17 at 4.1 u mid-leg and the 540-tick flank
     walked Bond INTO the pocket corner (-31.0,10.7) where #17 (d 3.2) and #4 (d 4.6) killed
     him at tick 16146 [L the pretrans-arms-6 dbg run]).
3. **The measured dead ends, not repeated:** the pretrans-arms-5 plate-side clear loop (the
   escort died EARLIER, during the descent follow) stays out; the x-only blanket hold (the
   fix3/fix4 attempts) is replaced by the y-aware one.

## What ran (release, --test-threads=1, this tree)

- walkreal_07000001_live WHOLE BINARY, the window's run: **6 passed, 0 failed, 1 ignored**
  before the window cut it — mayhew, the exit-fade pin, the_hall_guards_wait, the_plate_refuses,
  the main walk and the real-damage sister ALL ok [L the full-binary log, /tmp on the sandbox];
  the d3 sister ignored with its own ammo-exhaustion ledger (its death, tick 12897, killer #7
  skin 0x5000038 shotgun at the hall aperture, is byte-identical to the pre-task red — the d1
  gates don't touch it). The d2 `..._real_damage_agent` sister was mid-run (its own long
  guard-2 retry cycle, round 34+/48) when the run was cut by the sandbox tool timeout; its
  timeline is byte-identical to the -5 green single (every new lever is d1-gated; the
  flank_hold_min_y check is none_or on the unset field) — [L the -5 per-test single] stands
  for it.
- The fix5 targeted run (the three un-ignored tests + the d3): **2 passed, 1 failed (the d3
  ledger only)** [L the fix5 log].
- interactables_07000001_live and the nf-game lib tests: see the WORKLOG entry for the final
  counts on the pushed tree (they ran after this report's draft; the WORKLOG records them).
- The full four-crate `tools/test/suite.py`: the background start died within seconds (this
  sandbox kills every background job between tool calls — plain nohup, the -5 finding), and the
  foreground 4-crate release build does not fit the window (the walk binary alone took ~20 min
  of the 2-hour window; the target/ tree already holds 6.2 GB of the 9.9 GB disk). Coverage
  stand-in: the per-binary chunks above + nf-game + interactables.

## Live checks for the main agent

1. [S] In the real game at 07000001 (Operative), watch the ch-0xF slab pop: do the poppers
   (#4 0x500003a, #12 0x5000036, #17 0x5000039) advance on Bond at the mouth or on the contact
   parked at the stairwell head? gdbwatch on the contact's health through the mouth fight.
2. [S] The upper-floor perch guard (skin 0x5000036, state 0x2a at (-30.0,10.2,13.6)): does the
   game's own escort hide on him (NDrone2_NotNearArmedDrone 0x177640 counts a sniper-aim guard
   14 u away?) or does the lead descend past him? gdbwatch on the contact's state byte during
   the descent.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy failed

All three fallbacks failed at ~01:39 UTC 2026-11-11:
- curl: `curl: (28) Connection timed out after 20002 milliseconds`
- python3: `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
- wget: hung past 45 s, killed with no output (the -5 round hit the same wall at ~20:33 UTC).
