## Your task: `guard-pane-drain-5`, the 07000001 d3 walk passes with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-4, merge current origin/master, and read its docs/reports/guard-pane-drain-4.md plus the 01 d3 ignore message. Take its SPEND lever: trim the d3 pre-clear and stock passes and keep the wheel armed through the named loop. The phase after the AND gate has never run, so expect escort-phase work once the gate fires, and keep going through it to the exit. Real input only, no top-ups, never lower damage or health, helpers gated to 07000001 and difficulty 3. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary, the bullet binaries, glass_pass_every_level_live and dead_pane_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-09T05:58Z

---
## Report (done, 2026-11-09T06:28Z)

# guard-pane-drain-5 — the SPEND ran; the ledger corrected: the 01 d3 AND blocker is the game's own nav law, not ammo

## What was done

Task: take drain-4's SPEND lever (diet the d3 pre-clear and stock passes, keep the wheel armed
through the named loop), expect escort-phase work once the AND fires, real input only, no
top-ups, never lower damage or health, helpers gated to 07000001 + difficulty 3; remove the
ignore when green. Branch `s/guard-pane-drain-5` — the prior session had already cut it from
`origin/s/guard-pane-drain-4` and merged `origin/master` (63b05cca6, the merge base — master
has not moved since); this session continued on it.

The SPEND is in (all inside `walkreal_07000001_live.rs`, d3-gated, no shared-helper change):

1. **Pre-hall stock: 3 passes → 1** — drain-4's ledger blamed "stock 1" for eating the off-path
   box pair; one resupply call keeps attempting while the held-gun measure stays flat, so pass 1
   consumed both. Pass 0 tops the pistol/SMG and the off-path pair now survives for the
   door-phase detour.
2. **Pre-clear: free shots only** at d3 — no approach walks, no resupply-retry.
3. **The threshold fight (2×1200 ticks) disabled at d3** — run 1 proved it the real ammo
   furnace: ~150 blind taps per round (t%16, no seen-termination) burned the SMG's whole 48
   rounds on guard 6 with no kill.
4. **A threshold pre-open leg** — run 2's approach nav stopped 1.8 u short of the door plane on
   the shifted stream, the aperture never opened; the new leg touches the plane like the old
   fights did, then falls back to the hold line.
5. **`switch_to_armed` before every sustained lure** and tap counters on both lure loops
   (the ledger instruments).
6. **`door_peek_fight` widened to d3** (`>= 2`) — the d2-proven south-door angle fight.

## The ledger the runs produced (all [L], real input, 7.5–14.3 s per run)

- The pre-clear diet costs **zero** rounds: all three named guards have no LOS from the meet
  room (7.5/11.0/9.2 u) — nothing to free-shoot.
- The stock diet works: 1 pass → SMG clip 32 spare 16 armed.
- With the furnace off, the lures fire **armed**: 16–18 burst + 56 sustained taps per guard,
  door plane touched — and the guards **still hold z 15.4/18.4/18.1** (states 0x73/0x74,
  x ≈ 0, z 13.8–18.1 across runs 2–4). No crossing, ever.
- **The law that closes the south-aperture approach**: master's own
  `guards_shut_door_07000001_live` (from 74ee9b749) pins it — the hall guards' NavPath (port
  path 1, 28 nodes) carries only the west door #2 pair; the south door's pair lives on the
  kind-8 Mayhew route, excluded from the A* [M 0x1d2598 `AIPath_Parse`, 0x1571b0
  `AINetwork_DoAStarPath`] — **no armed lure can ever draw these guards through the south
  aperture**; they park at the standoff bands (`DroneFunc_CombatState` 0x148338:45-49, no
  route → 0x5f side-steps). Run 3's armed-taps-with-zero-crossing is that law, not a port bug.
- The south-door peek scan (run 4) found **no exposed part from any of the 20×9 stands** — the
  trio is too deep (x ≈ 0, z 13.8–18.1) for every jamb angle.

So the drain-3/4 framing ("ammo-exhausted; the SPEND will arm the lure") is superseded: the
lever chain terminates at the nav law. The walk stays `#[ignore]`d — red at the AND assert —
and the ignore message now carries this ledger and the next lever.

## The next lever (in the ignore message)

**The WEST door #2 peek** — the only door pair on the guards' own NavPath: stage to the
corridor side of #2 (−14.1,5.3,13.1) (the known walker wedge — short lenient legs), open by
hand, scan stands down the hall's west line (the trio is ~14 u east along the hall's long
axis), fight SHOOT-ONLY. Second lever: the east stage point (2.97,6.38,16.27) — inside the
hall ~3 u from the trio — as a shoot-only perch behind the mouth. The escort phase past the
AND has still never run on any tree.

## How it was verified

Every run is the real-input path from the level load (`--ignored --nocapture`,
`NF_ISO` + `NF_SILENT=1`, `--test-threads=1`), no teleports, no health/damage writes; the
edits are test-file-only and d3-gated (the door_peek widening is `>= 2`, which leaves d2's
behaviour bit-identical — the whole binary confirms it).

| binary | result |
|---|---|
| walkreal_07000001_live (whole) | **7 passed / 0 failed / 1 ignored** (the d3 00-Agent ignore) |
| bullet_range_live | 3 / 0 |
| fix_foliage_bullets_live | 3 / 0 |
| glass_pass_every_level_live | 3 / 0 (3 pre-existing geometry ignores, same as drain-4) |
| dead_pane_live | 3 / 0 |

The fast runner (`tools/test/suite.py`) still cannot survive this sandbox — background
processes (setsid'd nohup included) are reaped between tool calls here, so no suite total line
exists this session either; every binary the task names ran directly with its count above.
nf-game/nf-formats/nf-audio sources are untouched by this branch (the diff is the 01 walk test
file and docs), so the earlier sessions' green lib runs stand.

## What's left

1. The d3 walk green via the west-door-peek lever above, then the un-ignore; the escort phase
   past the AND behind it.
2. One clean full-suite pass on a sandbox that lets background processes live.

## Live checks for the main agent

1. **The west-door route, on hardware.** 07000001 d3: alert the hall crowd (fire from the
   meet room), then watch one hall guard's nav during 40 s of sustained fire from the hold
   line. Expected per the pinned law: he holds the standoff band and never paths to the south
   door — confirming run 3 matches the game. A gdbwatch on his route+0xF4 node list during the
   hunt should show path 1 nodes only (no south-door pair).
2. **The west door #2 crossing.** Open #2 from the corridor side and fire into the hall.
   Expected: the guards CAN acquire a route through #2's pair (n21/n22 on their NavPath) —
   if they still hold, the west-door-peek lever dies too and the east perch becomes primary.
3. **The trio's exposure from the west line.** A fieldprobe ray sweep from the corridor side
   of #2 at the trio's standoff spots (x ≈ 0, z 14–18): which stands expose a part — the same
   survey the peek fight's scan does in-port.

## Usage

- Context window / tokens / cost: **not visible to me** — no token, credit or context figures
  are exposed anywhere in this environment.
- Wall clock (from the session's `date -u`): the driver message arrived ~06:01 UTC; the four
  walk runs 06:20–07:25 UTC; this report written ~07:37 UTC, inside the ~08:00 UTC expiry.

## ntfy failed

All three fallbacks, in order, with their exact output:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "guard-pane-drain-5: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "… urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'guard-pane-drain-5: done …', headers={'Title':'NightfirePort'}), timeout=25)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "guard-pane-drain-5: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out (exit 124, no output)

Outbound HTTPS to ntfy.sh is unreachable from this sandbox (the same block the armour-absorb-int
session hit at ~07:00 UTC). The queue `done/` file and this report are the delivery.
