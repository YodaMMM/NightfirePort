# level-01-exit-after-the-wait — the 07000001 exit chain after Mayhew's wait

Branch `s/level-01-exit-after-the-wait` (from origin/master @ fcf6f334). Task: on Double Cross
Part 1, Mayhew now waits at the door correctly (the ally-wait hold, node 33 [M NDrone2_ReachedDestNode
0x153358:85-94]) and leads on after the ambush, but the level reportedly cannot be finished at the
end (Bond at (-33.44, 6.47, 8.16) at the bottom stairwell, Mayhew walking down ahead, frame ~7138,
the exit never completes). Reproduce on real input from a fresh load, find the dead link in the
exit chain, fix the port.

## Result

**The exit chain completes on the current master, on both the walker's tuned route and the driver's
follow route.** No dead link found. The failing state in the task brief reproduces as a TRANSIENT
escort stall whose release is the game's own band arithmetic (details below) — the exit fires once
the player advances. Every link of the chain was verified end to end through the real input path.

## The chain, verified link by link

1. **The intro** (SP #277, script 0x06000087, pack 07800001) writes ch 0xB at tick 46 of the walk
   (the port's measured timing; the intro tail's event 6 re-places Bond) — the contact's enable
   [D script decode; the port's walkreal runs log it every run].
2. **The contact's mission route** (drone #161, skin 0x0500004F): `AssignAIPath(8)` [M 0x15e4f0
   case 1] → the kind-8 path's link-walk chain from the spawn node 16. The chain ends at **node 36
   (-32.947, 3.695, 12.837), INSIDE the LoadPointActivator plate volume** (plate lo (-33.80, 0.64,
   12.17) hi (-32.41, 3.96, 18.68)) [D the nav dump — `crates/nf-view/tests/probe01_nav_dump.rs`].
   Node 33 (2.912, 5.739, 7.571) carries flag 0x20 — the ally-wait hold on ch 0x99 (the hall
   ambush's AND) [D placement; M 0x153358:85-94]; node 18 carries 0x80000 (a MayhewTalk 0x47 bark)
   plus the stairwell door bit [D nav dump].
3. **AllyLeadDone parks him ON the plate** [M NDrone2_DSTATE_AllyLeadDone 0x15f2e8: ENTER plays the
   alert idle, UPDATE returns 1 — no movement]. The follow-route run logged the park at
   (-33.03, 3.94, 12.93) with route (37, 37) — the chain's end — and the plate's `touched_by` read
   TRUE for his toucher at that spot.
4. **The AND-touch** (0xEB placement #180, out 0x10, mask 0x18, filter skin 0x4F) fires ch 0x10
   with BOTH touchers on the volume [M Trigger_Hit 0x1b1fc8:56-123; the port's drone touchers carry
   `skin: x.skin` — drone.rs 16719] — observed in all runs (`ch 0x10 fired at Some(0)` on the
   follow route: the same tick Bond's capsule entered the volume with Mayhew parked).
5. **The exit** (0xE8 placement #18, next 0x07000002, gate 0x10) fires on the gate with no touch
   check [M Trigger_Update 0x1b1a78 case 10] → `level_exit = Some(0x0700_0002)` in every run.
6. obj0 (text 0x04000006) watches ch 0x47, whose writer is not on disc (the un-ported mission/
   dialogue layer); the level exit does not depend on it [D the census in docs/levels/07000001.md].

## The two real-path runs

- **The tuned lead route** (`walk_from_the_spawn_to_the_exit_with_the_contact`): Bond clears the
  guards, takes the plate first on the final descent. **4/4 passes, bit-identical** (exit at tick
  71401 in every run; the log sizes match to the byte across processes).
- **The follow route** (`following_mayhew_down_completes_the_level` — NEW, this task): Bond clears
  the ambush, then trails Mayhew at 4.0 down the whole stairwell — never the plate-first lead.
  **3/3 passes**, exit at tick 27466. Mayhew stalls five times on the way (the meet hall's exit, the
  west corridor at x −5/−11/−18.8, the stairwell door, mid-stairs) in the game's AllyLeadWait, and
  every stall released when Bond closed in — the release band [M 0x15eda8 case 3] is position-driven.

## The task-brief snapshot, explained

Bond parked at (-33.44, 6.47, 8.16) with Mayhew "walking down ahead" is the **stairwell-door
AllyLeadWait stall**. The passing follow run hit exactly it: Mayhew stood at (-33.01, 5.94, 9.69)
in state 0x20 with Bond behind, and the port's `object_mission_path_distance` logged **path -10.3**
while Bond was ~1.2–5 units away in a straight line. That value is the game's own arithmetic, not a
port divergence:

- The pd scan walks the mission route's node list for the node nearest Bond, gated by 3D distance
  ≤ 30, |dy| ≤ 4.0, the slope band, and **AINetwork_BoundsTest only** (no collision-world LOS) [M
  0x153f50:89-127; the DAT constants are [L ee_snapshot] and the port's `Network::node_scan_ok`
  mirrors the chain]. At the stairwell head the boundary fence around the floor opening rejects the
  near nodes (14/15), so the scan lands on node 13 — 10.3 chain-units behind the drone's node.
- The distance is `AINetwork_GetRouteDistanceBetweenNodes` [M 0x156160]: the summed stored link
  lengths between the two list positions, negative when Bond's node is behind. The port's
  `Route::for_path` builds the same link-walk chain the game's `InitAIPath` builds (the kind-8
  open-chain rule, nav.rs 569-604), and the pd sums the same links with the same sign [M].
- AllyLeadWait holds while `pd ≤ −7.0`, or `pd == 0.0` with Bond ≥ 5.0 away; it releases at
  `pd > −7.0 && (pd != 0.0 || dist < 5.0)` [M 0x15eda8 case 3]. The port's arm is that exact
  predicate (drone.rs 8940-8942).

So the escort waits at the door whenever Bond trails at a chain distance > 7 — and releases the
moment Bond advances past the door (his scan node becomes 18/35). In the passing follow run the
release landed when Bond's close-in walked him to (-32.86, 5.89, 10.32), past the door: 0x20 →
0x1d → the route ran to node 36 → 0x23 Done → the AND on Bond's next step. A player standing at the
door corner waiting for Mayhew to keep going sees a level that never ends — but the same stand is
the game's own behavior by the decompile. The second stall-holder is **AllyLeadHide**: a live armed
guard within 14 parks him (guard 12 at 8.9 units held the hide at the stairwell head in one run
until the clears landed) [M 0x15ec08 case 3 + NotNearArmedDrone 0x177640] — the player's fight.

## Tests added

- `crates/nf-view/tests/walkreal_07000001_live.rs::following_mayhew_down_completes_the_level` — the
  driver's real path end to end: fresh load → the intro → clear around the contact → the meet plate
  (ch 0x32 → 0x33 → SP #276 → ch 0xE) → the hall's three named guards → ch 0x99 → follow the
  contact (4.0 trail, close-ins on the game's wait stalls) → the contact parks in AllyLeadDone on
  the plate → Bond steps on → ch 0x10 → the exit to 07000002. Dumps the park position, the plate
  bounds, both touchers' `touched_by`, and every live armed guard within 20 on the way.
- `crates/nf-view/tests/probe01_nav_dump.rs` — the kind-8 path dump (node positions + flags) that
  pins the route-end-inside-the-volume claim [D].

No decompile-backed assertion was weakened; no production code changed (the exit chain verified
correct as-is).

## Verification

- Direct runs (final tree, `--test-threads=1`, NF_ISO set): `walkreal_07000001_live` main test 4/4
  (tick 71401, bit-identical), the new follow test 3/3 (tick 27466).
- The fast runner (`tools/test/suite.py`) was launched detached at 00:32 UTC; build ok: 874 test
  binaries in 1073 s. Its nf-view test build picked up the new test file, and **the suite's own
  `walkreal_07000001_live` run passed 4/4 (76.52 s), including
  `following_mayhew_down_completes_the_level`** — the final tree validated inside the runner.
  The FAILED set so far is exactly the 4 allowed slim-ISO failures
  (`identity_disc::the_real_disc_passes_the_identity_check`, 3 × `disc_golden`); the pass was
  still working through the remaining binaries at report time.

## Live checks for the main agent

To close the loop on the original observation against the real game (PCSX2, fresh boot — a
save-state resume skips the boot path):

1. Walk 07000001 to the stairwell with Mayhew leading; stop Bond at the door corner
   (-33.4, 6.5, 8.2) while Mayhew stands at/near the door node and do not advance for ~20 s.
   Watch `drone+0x970` (the mission AIRoute): `+0x9f0` (the current node index), `+0x9f2` (the
   count), the state machine id (0x1d/0x20/0x1e/0x1f/0x23), and SM+0x20 (the stored pd). EXPECT
   (per the decompile): state 0x20 (AllyLeadWait) with SM+0x20 ≤ −7.0 — the same hold the port
   shows (the port logged −10.3). If the real game instead resumes the lead with Bond parked at
   the corner, the port's pd scan diverges and that is the fix to chase.
2. Then walk Bond through the door and down one flight: EXPECT the state 0x20 → 0x1d within a few
   ticks and the route index advancing past the door nodes — the release the port shows.
3. At the bunker: with Mayhew parked (state 0x23) at (-33.0, 3.9, 12.9), step Bond onto the plate:
   EXPECT ch 0x10 (switch_channels[0x10]) to latch and the 07000002 load to begin.

## What is left

- The suite's final total line (the runner was still executing at report time; the 4 slim-ISO
  failures are the allowed set). The next session can re-run `tools/test/suite.py` on the final
  tree for the formal total.
- If the live checks above show the real game releasing the escort where the port holds him, the
  candidate deltas are the pd scan's bounds-gate inputs (the AI boundary walls around the stairwell
  hole) and the `Route::for_path` chain order vs the game's InitAIPath on this path — both pinned
  by the probe and the decompile cites above.
