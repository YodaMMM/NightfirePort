# audit-voice-triggers-3 — finish the bark draws' walks

Continue from s/audit-voice-triggers-2 and read its report first: the four triggers are right, Mayhew and smg's 1 u control are green, and the smg binary and two full walks are still red with the next step named for each. Merge master, finish those without removing or reordering a draw, then run walkreal 01, walkreal 03 and smg_close_range_live and report each result.

---
claimed-by: sandbox7 at 2026-11-05T10:44Z

---
## Report (done, 2026-11-05T11:50Z)


---
## Report from the worker (done, 2026-11-05T11:50Z)

# audit-voice-triggers-3 — the bark draws' walks finished

Status at the sandbox cutoff (2026-11-05 ~12:00 UTC): **walkreal_07000001_live GREEN 8/8 whole;
walkreal_07000003_live GREEN 6 passed / 0 failed / 1 ignored whole (the full route completes every
objective in 353.56 s); smg_close_range_live still red at the 2 u / 6 u pistol legs, with the
mechanism narrowed and the next step pinned.** No draw was removed, reordered or re-timed anywhere:
every change is test-side tactics or file-local, and the merge resolution kept master's code side.

## The merge

`origin/master` (f68a72216) merged into the branch cut from `origin/s/audit-voice-triggers-2`. One
conflict, `crates/nf-game/src/drone.rs`, three hunks: master's `footstep` → `footstep_ops` rename
(the Vec form, the npc-footsteps task's field) is taken; this branch's `play_script_talk_gate: 0`
field init and the three audit-voice-triggers bark tests are kept; master's new
`play_script_0x168_is_deaf_to_gunfire_until_the_script_ends` test is kept alongside them. The
deaf-window code is master's own and landed unchanged. After the merge `nf-game` builds clean
(49 pre-existing warnings, 0 errors).

## 1. walkreal_07000001_live — GREEN, whole binary 8 passed / 0 failed (51.63 s)

Which draw moved what (carried from the -2 report, unchanged): the bark draws shifted the shared
stream; the hall fight's timeline moved; the drone-#6 shotgun volley (0x1c, 6 pellets in one tick
at 2.5 u) finished a body the return-fire fights had bled to ~18 (39 hits for 100.0, d#6 48.5 +
d#7 30.0 + d#11 15.0). The pinned next step was: raise the d1 personal space to the Agent sister's
6.5 AND thin the attrition before the hall. Both landed (all file-local to
`crates/nf-view/tests/walkreal_07000001_live.rs`):

1. **The personal space is 6.5 on EVERY difficulty** (`walk_to_the_exit_at`): the Operative 4.0
   stood inside the guards' own closing range — they shot on from 4-8 u and the backing never held
   them out. Reproduced first: 6.5 alone still died (t3199, 37 hits for 100.0, the guards close
   through it) — the spacing lever alone is not the answer, which is why the second half landed.
2. **The d1 walks take the Agent hall fight** (the gates `difficulty >= 2` around the hall phase
   are now unconditional): the meet-room pre-clear (LOS-gated, the named guards at 4-9 u are
   outside the TooClose band), the south-door approach + hold line, the per-guard sound lure (the
   bounded 300-tick live-tracked burst [M DroneFunc_SetNoiseHeard — the guards hunt the gunshot
   hearing]), the 3.2 band-edge doorway fight, then the doorway step + west-corridor route
   pre-clear + the pickup line, and the pre-stairwell 6.5 stand-off. The Operative's old
   east-door stage leg into the alerted hall is gone — walking into the funnel is the bleed the
   doorway fight replaces. The kill ledger after the fix: the return-fire fights answer guards
   Bond is already facing at the band's edge, not damage taken while walking.
3. **The doorway step is south-of-aperture-only**: the plain d1 walk's guard-6 flank nav carried
   Bond INTO the hall; the southward step then wedged on the open door's collision from the wrong
   side (bond (5.45, 7.22, 11.67) grounded=false on the door frame, segment hit at t 0.236). From
   north of the aperture the step is skipped — its only job is moving a south-of-the-door Bond
   into the contact's lead release band, which he already holds.

Verification: reproduced red twice (t3253 39-for-100.0, then t3199 37-for-100.0 with 6.5 alone),
then green. The whole binary after the fix: **8 passed; 0 failed; 0 ignored; 0 measured; finished
in 51.63 s** — including `following_mayhew_down_completes_the_level` (the -2 session's green) and
the Agent / 00 Agent real-damage sisters (byte-exact: every change above is either d1-only or on
code paths they already ran).

## 2. walkreal_07000003_live — GREEN, whole binary 6 passed / 0 failed / 1 ignored (382.46 s)

Which draw moved what (the -2 report, unchanged): the shifted stream changed the east-lane fight's
shape — Bond ended DRY (clip 1 spare 0) fighting drone 11 (exposed false at 8.0 u behind cover the
walker's box model cannot see), the resupply refused (no pickup within the walked-path corridor),
10800 ticks burned, then the return legs' middle anchor wedged: bond slid down the moat ditch to
(10.28, 6.34, −36.05) heading to (10.27, 6.65, −30.01), segment_to_target hit at t 0.10, the nav's
sidesteps sliding ALONG the ditch wall, never up it. Reproduced identically this session (the same
STUCK dump, `walk_to the east lane: 6000-tick cap`, 303.21 s run).

The fix (file-local to `crates/nf-view/tests/walkreal_07000003_live.rs`, the east-lane return
loop): the three lane legs re-arm `lenient` per leg (the clear_near/shoot internals reset it on
their way out — the wedged leg panicked at the non-lenient cap instead of reaching any ladder), and
a wedged anchor takes the moat sidestep ladder: sidestep ALONG the moat (the moat legs above walked
it fine) east then west, re-take the anchor from the stepped-aside spot; if both sidesteps wedge,
the anchor is skipped and the NEXT leg re-derives its own path (the anchors steer the nav; a wedged
steering anchor must not strand the route). No draw touched; the ladder runs only on the wedge
path, so the sisters' green timelines are byte-exact — confirmed by the whole-binary run.

Verification: reproduced red (the 303.21 s STUCK), then the whole binary green: **6 passed;
0 failed; 1 ignored; 0 measured; finished in 382.46 s** — the full route completes every objective
(`walk_the_full_route_and_complete_every_objective` alone 353.56 s), the real-damage Agent and
00 Agent sisters included.

## 3. smg_close_range_live — still red at the 2 u / 6 u pistol legs (the 1 u control green, byte-exact)

Which draw moved what (the -2 report, unchanged): the bark draws shifted the stream; the single
pistol pull's unconditional cone draw [M Bullet_init 0x12c5a0:171-180] missed the box the walker
aims at; the hall's combat-state guards (0x73/0x74) re-cover during the stands. The -2 session's
three fixes (the semi-auto cadence, the part-4-torso filter, the candidate retry) carry over and
the 1 u leg stays green byte-exact: **6 rounds, 5 impacts, hp 24.0 → −2.2 (drop 26.250), 107
ticks** — the gate work below is opt-in per leg (`segment_gate` passed only for dist > 1.0), so
that timeline never re-rolls.

This session's three attempts (all in `crates/nf-view/tests/smg_close_range_live.rs`, file-local,
committed as the honest WIP):

1. **The far-from-contact candidate order** (the -2 report's pinned option): the 2 u / 6 u legs
   take candidates farthest from the contact first (> ~8 u cannot be stolen). Result: WORSE — the
   escort does not idle at the meet plate, he FOLLOWS Bond into the hall, and every candidate died
   mid-leg anyway: d#5 12.0 → 0.0, d#6 6.8 → 0.0, d#11 12.0 → 0.0, all ally, 0 player impacts.
   The >8 u rule needs the contact's actual position, which tracks Bond's.
2. **The clear-sight fire gate + the hold-spot legs**: `fire_at` now fires only when the
   eye-to-chest segment is clear (a player does not fire into crates — the old legs poured 6-8
   rounds into cover over 3000 ticks), and the legs beyond 1 u stop chasing a hunting guard (the
   chase-stand walked Bond into the cover pockets and wedged at the 3000-tick cap) — Bond holds
   the proven mid-hall spot and lets the hunters cross. Result: d#6/d#11 never exposed a chest
   (0 rounds in 6000 ticks) — the 0x74 guards shoot from under the crate lip.
3. **The exposed-box aim**: when the chest is covered the gated aim picks whatever box the game
   exposes — the head (part 5, the port's one head part [M damage.rs hit_damage's match]) over the
   lip, else any clear box. Result: rounds now spawn at exposures (d#6 1 round, d#11 4 rounds) but
   every candidate still dies to the escort's volleys before a PLAYER impact registers (the final
   run: d#5 4 rounds 0 impacts 12.0 → 0.0 ally at 479 ticks; d#6 1 round 0 impacts 6.8 → 0.0 ally;
   d#11 4 rounds 0 impacts 12.0 → 0.0 ally).

The remaining mechanism, stated precisely: the contact fights alongside Bond inside the hall and
kills each 12-health candidate in ~300-500 ticks of the leg, while the 0x74 cover discipline gives
Bond only 1-4 exposure ticks per window and the semi-auto cadence lands no round inside one. The
control's claim (the pistol registers damage on guards the game lets you hit) is true — the 1 u
leg proves it every run — but at 2 u / 6 u the game's own ally takes every target first.

Next step (pinned, not started): pin the escort's interference explicitly — assert the steal as
the game's own ally behaviour (a leg whose target dies to ALLY fire mid-leg documents the volley,
like the 01 walk's ledger) and take the 2 u / 6 u legs to a roster the escort cannot reach inside
the window: the courtyard guards on the ch 0x26/0xf channels (d#2/d#3/d#4/d#12/d#17 in the roster
dump), enabled by walking their own plates — a real-input route addition, no channel writes.

## How this session was verified

- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000001_live`: reproduced the
  death twice (t3253, then t3199 under 6.5-only), then the whole binary **8 passed; 0 failed;
  finished in 51.63 s** (run twice; one early run had `the_exit_fade...` red before the
  doorway-step gate landed, green in the final whole-binary run).
- `cargo test --offline --release -j2 -p nf-view --test walkreal_07000003_live`: reproduced the
  east-lane STUCK (303.21 s), then the whole binary **6 passed; 0 failed; 1 ignored; finished in
  382.46 s**.
- `cargo test --offline --release -j2 -p nf-view --test smg_close_range_live --exact
  smg_rounds_damage_a_close_guard_and_the_pistol_sister`: three fix attempts, quoted per run above;
  final state red (the 2 u leg), the 1 u leg green byte-exact.
- The full prescribed suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats
  -p nf-audio -p nf-view -- --test-threads=1`) **did not run this session**: the sandbox reset
  consumed the setup window (the ALSA user prefix, the RUSTFLAGS fingerprint invalidation and the
  disk-full recovery — a full 2-core rebuild of the dep tree was deleted back to the cache) and
  the remaining window went to the three reds. The changed files are three test files + the WORKLOG
  + this report; `nf-game`'s lib built clean post-merge. The next session should run the full
  suite and every `walkreal_*_live` binary before finishing (the 01 and 03 binaries are quoted
  green above from this session's own runs).

## Live checks for the main agent

None required for the 01/03 greens — both are Sim-level and reproducible. The smg leg's open
question (carried from the -2 report, still the one check that would settle it): in PCSX2 at the
meet hall, with the contact fighting alongside, shoot the state-0x74 guards (d#11 at (7.94, 6.35,
17.46), d#6 at (6.6, 6.3, 14.6)) with the pistol from 2-4 u — confirm whether the game's own ally
steals those kills and whether any part shows over the crate lip in his shooting pose (the port
shows the chest permanently covered; the head-over-lip question decides the exposed-box aim's
viability).

## Usage

not visible to me

---
## Report from the worker (done, 2026-11-05T11:53Z) — ntfy failed section appended

## ntfy failed

All three fallbacks failed (2026-11-05 ~11:52 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20000 milliseconds`
2. `python3 -c "... urllib.request ..."` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → no output, killed at the
   60 s tool timeout (the connection never opened).

