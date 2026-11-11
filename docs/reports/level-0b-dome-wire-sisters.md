# level-0b-dome-wire-sisters

Branch `s/level-0b-dome-wire-sisters` = origin/s/walker-sidestep-decay cb992a3bd (origin/master
already merged in upstream — the merge was "Already up to date"). The walker-sidestep-decay
report read before any code moved. **Test-file changes only** — no `crates/*/src/` change
anywhere; the sim, the walker helpers and every other level's walks are untouched.

## What's done

1. **The sisters' real failure re-measured, twice over.** On this tree the four 0b
   real-damage sisters did NOT die at the dome wire first: they wedged at the desk-corner
   doorway leg (`files n26→n25`, the 6000-tick cap at (−20.70,48.06,30.64), health 100, zero
   hits — all four, whole-binary order [L this session's first whole run]) — the ledger's
   "they round the desk corner on their own fight timing" relied on guard contact this
   timeline never produces. **Repair (level-gated to this file): the west flank is no longer
   gated off the real walks** — `files n26 flank west` runs for them too.
2. **The "dome wire" is identified from the runs, not the ledger's phrase.** There are NO
   fuseboxes and NO 0xE4 hurt volumes and no folded `stat_hurts` entries on 0700000b [L the
   temp fold/hurt dump at the deck key: `stat_hurts (0)`, no hurt rows, no fusebox rows — the
   temp diag was removed before the final commit]. The −2.40 world zaps are the terminal
   lane's wall guns' rounds: the ceiling gun **#291 (−1.14, 49.06, −0.19)** plus the north
   pair **#174/#175 (±x, 49.10, −21.60)** — owner `usize::MAX` ("the world" in the damage
   log) at −2.40 per round = the wall guns' 4.0 damage × Easy 0.6 [M DroneWeap_DoBulletAccuracy
   0x179740:74-88's difficulty table; L the NF_DMG_TRACE runs]. The head-on lane costs were
   re-measured on the decayed timeline: **the approach 14 × 2.4, the return 25 × 2.4 = 93.6 of
   100 — dead at the deck key (tick 16013), the ledger's own death reproduced** [L the d1
   run].
3. **The lane repair (the code's own west-crossing lever, re-measured on this timeline; gated
   to `real` = the sisters, this file only):**
   - the approach: the northwest diagonal `lane_sprint` to (−7.0, the plate's z) then east
     into the spawned plate — **5 rounds (12.0)** [L];
   - the return: west-exit (−7.0, −13.9), rejoin the corridor at z −5 on the east lane,
     sprint the mouth, n16 — **~16-17 rounds vs 25 head-on** [L].
   The direct west L-shape pins on the x −6.3 wall row (measured at z 1.5 and z −6.6 [L]) —
   the wall row's z −6.6..1.5 span is recorded in the route comments.
4. **Result: whole `walkreal_0700000b_live` 5 passed / 4 failed** — the plain walks stay
   green, and the four sisters' deaths moved from the wire (t16013) to the ROOF ENDGAME:
   d1 t16479 at (−1.48,58.14,29.41) (38 hits), 00agent ×2 t16835 at (−12.27,57.85,10.96)
   (24 hits), agent t16872 at (−11.95,58.07,14.55) [L the final whole run]. The wire toll is
   no longer what kills them; the roof fights (drone #4 point-blank 12.6-16.8, #5/#7/#8
   volleys) take the remainder.

## How it was verified

- Whole `walkreal_0700000b_live` (`--test-threads=1`, release, final tree): **5 passed /
  4 failed** — the four sisters above [L].
- `interactables_0700000b_live` (whole): **13 passed / 1 failed** — `qworm_console_installs_
  on_the_gadget_shot` (line 464, "the gadget shot lands and installs") — **pre-existing on
  the base tree**: the file's last change is 2998b6a76 (before this branch), the binary does
  not compile this task's file, and no `crates/*/src/` or shared-helper file changed [L the
  run; `git log -1 --` the file].
- `nf-game` src is untouched (no `crates/*/src/` diff on the branch); it compiled clean as a
  dependency of every test build this session.
- The fast suite (`tools/test/suite.py`) did NOT run: this sandbox kills background jobs
  between tool calls (the established session behaviour the predecessor reports record), the
  remaining window went to the walk repairs, and the full four-crate build does not fit the
  9.9 GB disk alongside the ISO and the toolchains (the same verdict the
  walker-sidestep-decay, los-mask-repins and real-health sessions recorded). Every binary
  that compiles the changed file ran whole.

## What's left

- The four sisters are still red — **the ledger and the next lever**: the lane toll is now
  ~50 (21 × 2.4 on d1: 5 approach + ~16 return), and the roof fights kill from there (d1
  died to #4 point-blank 12.6 + #7 10.5 + #5/#8 volleys). Levers, in order:
  1. shave the return's remaining ~16 rounds — the tracked gun follows across the corridor
     (the east lane x −0.3 measured the same as the west lane x −1.1 [L]); the unexplored
     shape is a stop behind the corridor mouth's furniture row before the n15 dash;
  2. the d1 roof package has no early r1 (the `agent_run` gate is `difficulty >= 2`) — the
     #4-on-the-dash-line close (28.0 point-blank [L the sprint-4 report]) is the d1 killer;
     porting a d1-legal close (the sprint-5 dart-at-5.0 lever, already in the file for the
     sisters) onto the P3→P4 phase is the next route lever;
  3. d2/d3 died at (−12.27,57.85,10.96) / (−11.95,58.07,14.55) — the deck west band after
     the P1 sprint — their 6-round band window and take-slot hold ride the d3 gate; re-time
     those against the west-band arrival on the new (earlier) timeline.
- The qworm interactables red above (pre-existing; not this task's file).
- The fast suite on the final tree (per-binary pass counts), next window.

## Live checks for the main agent

None new — the fix is in the level's own test routes; the game-facing facts rest on the
predecessor rounds' live checks. If a render pass is cheap: the corridor gun #291 at
(−1.14,49.06,−0.19) should show a ceiling-mounted wall gun over the terminal lane, and the
x −6.3 wall row (z −6.6..1.5) should show the west room's east wall the return doglegs
around.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~09:27 UTC, stop-new-work 09:07), `git` hashes, test outputs, process exit codes. No
input/output token counters, no credit balance, no dollar cost.

## ntfy failed

The sandbox has no reliable outbound route to ntfy.sh this session (the same outcome the
walker-sidestep-decay, los-mask-repins and guard-noise sessions recorded). The three
fallbacks, in order, with their exact outcomes:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-0b-dome-wire-sisters: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 urllib.request` →
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x` → **exit 0, no output** —
   but a verification rerun without `-q` hangs at `Connecting to ntfy.sh|159.203.148.75:443...`
   until its timeout (the same artifact the predecessor sessions measured), so the exit-0 is
   a connection artifact, not a delivery. Treat the notification as NOT sent.

Everything else was delivered: the branch (the report commit + the WORKLOG checkpoints), and
the queue's done item `done/10-level-0b-dome-wire-sisters.md` with the report body attached
(the claim was filed through `claim.sh` — the task sat unclaimed in `pending/`; `finish.sh`
first ran with a relative report path and marked done without the body, then the body was
attached with the absolute path and pushed).
