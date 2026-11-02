## Your task: `level-0c-secret-door-reach`, 0c: opening the secret door the way a player does

level-0c-remaining-teleports-to-walks (merged; read its report) found the secret door #544's plates #558 (E-arm) / #559 (body touch) sit in a corridor whose touch volume starts at x 52.24, while the body-touch capsule reaches only x <= 51.84 from the yard side — so the door cannot be opened from the yard, and `walkreal_0700000c_live` still places Bond at the plates. Find the game's way into that corridor [D the corridor's north end, the mezzanine annex vent #180/#182/#184 (ch 0x54), any door/hatch that opens onto it; M the classes involved], walk it on real input, open the door through the plates, and delete the placement. Fix any port gap with evidence. Run walkreal_0700000c_live and interactables_0700000c_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-secret-door-reach`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T07:29Z

---
## Report (blocked, 2026-10-31T08:17Z)


---
## Report from the worker (blocked, 2026-10-31T08:17Z)

# level-0c-secret-door-reach — the report

Branch `s/level-0c-secret-door-reach` (re-created from origin/master 7d0091a0 after the sandbox
reset; the previous session's local branch had no commits). Task: the secret door #544's plates
#558/#559 sit in a corridor the body-touch capsule cannot reach from the yard (the volume starts
x 52.24, the capsule reaches x ≤ 51.84 [D remtel]); find the game's way into that corridor, walk
it on real input, open the door through the plates, and delete the `[plate2 + 0.6]` placement in
`walkreal_0700000c_live`.

## What's done

**The game's way into the corridor is FOUND and evidenced — the corridor's NORTH END.** Four
probes committed (`walkreal_probe0c_secretreach_live.rs` the flood-fill + band maps,
`..._voidspan_live.rs`, `..._upperlane_live.rs`, `..._bridge_live.rs`; all [D] via the port's
parsers, which equal the disc's declared collision [D collaudit probe]):

1. **The corridor's north end is OPEN.** The 0.25-grid flood over x 8..62, z −192..−115 (0.55
   step law, drops ≤ 3.5 one-way with knee checks — the eastclimb7 law) floods the yard + the
   warehouse ground + the corridor as ONE component (comp 2: 15433 cells, x 40.8..62,
   z −186..−139.5, y 0.38..3.38). The corridor floor 0.90 continues north past the vent
   (z ≤ −161.5 at x 53.8) and merges with the warehouse ground 0.69/0.89; at z −165 the corridor
   (x 52.5..55.5) is step-continuous east to the warehouse ground (x 56.0, 0.90 → 0.69).
2. **The mezzanine annex vent (#180/#182/#184, ch 0x54) hangs in the corridor at z −165..−163.5
   under the 5.44 blocking poly** (the scans: x 53.8, z −165.0: [14.29, 5.60, 5.44, 0.90]). The
   mezzanine deck (5.44..6.03, x 52..56, z −176..−152) floods as a SEPARATE component (comp 3)
   unreachable from the ground, and the warehouse upper floor (8.7, x 45.5..47.8 ledge + x 53..62
   floor) floods as islands — so the vent is not the ground route; the north end is.
3. **The display ground (6.27) is sealed east** — the x ~15 wall stands y 6.4..10.5+ (east rays
   hit x 14.90..15.56 at every body height z −155.5..−150.5) — the game's exit is HIGH ("the ways
   in and out of this building are on higher floors" [D USATxt 0x4000063]). The high route east
   is mapped: the ST-1 walkway runs south to z ~−155.5 and steps 14.6 → 14.3 → 13.3 onto the east
   deck (x 18..21, z −162..−156.5), the 9.2 patch (x 22..23, z −162..−157) and the east ramp
   (6.9 → 3.87, x 18..29, z −155..−143) hang below; the ramp ends at x ~29.5 over a FLOORLESS
   pit (x 30..32: no surface from y 8 down to 0) crossable by the engine's own drop-jump carry
   (4.42 m flat measured, 5.2-unit law [M player.rs; the walkreal header]) onto the north-east
   strip (0.06, x 33..40.5, z −152..−143), which joins the warehouse ground (0.38, x 41+) with
   the knee-height rays (y 0.9/1.4) CLEAR at z −151/−149/−145/−143.5 (one post x 41.06 z −147).
4. The lane south (12.3 — the walkreal's own walked territory z −115..−128, legs green) continues
   to z −158.5 [D upperlane scans], joining the walkway/deck system at x 17, z −157..−158.

**The walk legs are NOT yet written** — the session's window went to setup (the sandbox reset
cost clone + downloads + toolchain + builds) and the geometry census. Remaining legs, in order:
the lane south extension (z −128 → −158.5), the walkway step-down (14.6 → 14.3 → 13.3), the drop
onto the 9.2 patch (4.1 m) or the ramp (4.4–6.4 m — the 4.2 m band walk-off [L 2026-10-30 fresh
boot] bounds survivable drops; the 6.4 m variant is [S] unverified and to be avoided by taking
the patch), the ramp walk east (steps 0.1–2.1), the pit jump (a ~3.5 m gap with a 3.8 m fall —
inside the drop-jump carry), the strip walk, the bridge, the warehouse ground north to the
corridor, then the EXISTING real legs 5+ (the plates chain → the door → the yard) unchanged —
and only then the `[plate2 + 0.6]` write dies. No src changes were made this session; the
commit touches only the four probe files + WORKLOG.

## How it was verified

- `walkreal_0700000c_live`: **2 passed, 0 failed** (17.71 s, release, single thread, NF_ISO set)
  — the tree is untouched, so the mission walk stays green.
- `interactables_0700000c_live`: **5 passed, 0 failed** (9.53 s).
- The full suite (`tools/test/suite.py`) was NOT started this session: the reset sandbox consumed
  the first ~20 minutes (clone, work-slim + ISO downloads, rustup, the alsa user-prefix link
  fix), and the runner needs most of the 2-hour window (prior sessions' notes). The next session
  should start it in its first minutes and quote the total line.

## What's left

- Write + tune the walk legs above (the drop chain is the only uncertain part; the patch route
  avoids the 6.4 m variant), delete the placement, re-run the two binaries + the mission sweep.
- The full-suite total line on the final tree.
- Live checks for the main agent:
  1. **The high east route.** Fresh boot, from the ST-1 walkway south end (13.5, 15.7, −152):
     walk the walkway south and step to the 13.3 deck; record whether the game's body makes the
     14.6→13.3 steps and which drop it takes to the ramp (the patch 4.1 or the deck edge 6.4),
     and whether the ramp's pit (x 30..32) is jumped east as mapped. gdbwatch on the pit edges
     if the body stalls.
  2. **The corridor's north end in play.** Fresh boot, put Bond on the warehouse ground
     (50.0, 1.0, −165) and walk north then south into the corridor to the plates: record the
     plates' ch 2/ch 3 firing and the door opening — the port's chain is already green; this
     confirms the game's own approach direction.
  3. **The vent plug (optional).** From the warehouse upper floor (if reachable — itself an
     open question), burn VentLatch #182 and watch SS #180/#184 sweep; the mezzanine remains
     flood-isolated, so this only matters if live play shows the game using the vent.

## Usage

not visible to me (no token, context-window or cost figures are shown in this environment).
