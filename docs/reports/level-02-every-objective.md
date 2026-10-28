# level-02-every-objective — report

Task: make `crates/nf-view/tests/walkreal_07000002_live.rs` walk the WHOLE 07000002 mission from a
fresh load on real input (no teleports past a blocker, no channel/state pokes for the objectives),
every objective the mission data lists asserted complete in player order, ending on the level exit
firing. Branch `s/level-02-every-objective`, from origin/master 87256135.

## What the current tests already covered (the starting inventory)

`walkreal_07000002_live.rs` already walked, on real input: the intro cut-scene's ch 0x47 (the
obj0/obj1 reveal), the garden band, geisha-1's rescue (captor deaths 0x29/0x2A/0x88 [D param 12],
`DroneFunc_HostageSaved` 0x1441c0 → 0x49), AND #397 → 0x3B, the 0x3C touch zone, SEQ → 0x35, the
mid-level scene SP #399, the bunker doorway sensor 0x81, Computer1 #14 shot → 0x48, the ambush lock
(0x81 holding #385-387), geisha-2's rescue → 0x4A, AND #303 → 0x4B, AND #480 → 0x4C, and the real
exit E with the earned 0x4C (`level_exit == 0x07000003`).

**obj0 and obj1 were therefore covered at the channel level; obj2 was NOT covered** — its watch
channel 0x4D was hand-set by a `[S]` channel poke (test step 7) with a warning that ch 0x64 never
closed. Both were wrong, in different ways (below).

## What this task changed

1. **obj2 "Find door to servants quarters" (watch 0x4D) is completed on real input — no poke.**
   The writer is the **arc hand-off**, on the disc in 07000003: TouchOnce trigger **#308 (0xEA,
   param 0 = 0x4D)** at (−9.0, 6.0, −23.2), the level entry right past 07000002's exit
   (−10.05, 6.75, −23.18), with SoundTrigger #341 (0xF9) keyed on 0x4D beside it [D 07000003 param
   sweep, probe `walkreal_probe03handoff`]. The same hand-off shape is proven one part back:
   07000002's intro stream 7 writes 0x47 = part 0's watch channel ("Escort Mayhew to bunker"
   [L c07000001]) [D 07800002 0x06000084]. The test now does the viewer's next-level step into
   07000003 after the exit, the entry touch raises 0x4D through the real Trigger path, and
   `Mission_MonitorObjectives` 0x1827d8 case 3 folds part-1 obj2 to state 4 — asserted. The same
   0x4D also reveals 07000003's obj0/obj1 (enable 0x4D [L c07000003]) — asserted.
2. **The in-level computed writers were ruled out on the disc, not assumed:**
   `Monitor_Update` 0x1afcc8:170-174 raises the viewed sensor's param 8 while the monitor camera
   view is live — both 02 sensors carry param 8 = 0 [D probe `walkreal_probe02mon`]; no 02
   SimpleScript carries 0x4D — `SS_Update` 0x1a9948:21-23 raises SS param 3, and the disc values
   are 0/1/2 [D same probe]; doors write only obj+0x66/0x68 [M 0x136100:97, 0x135be0:58]; no
   constant `switch_channels[77]` write exists anywhere in the decompile.
3. **The objective states are now asserted through the mission layer**, not only the channels:
   `mission.states[row 07000002][0] == 4` after the computer, `[1] == 4` after the geisha chain,
   `[2] == 3` once 0x4C reveals it, `[2] == 4` on the 07000003 entry [M 0x1827d8 cases 1/3;
   L c07000002].
4. **The step-7 warning was wrong and is replaced by a decompile-backed negative**: ch 100 (0x64)
   must NOT close on this walk — the monitor's `bVar9` AND spans every part of mission
   0x07000001, and parts 2/3's watch channels (0x4E/0x51/0x53/0x54) are unwritten here
   [M 0x1827d8:22-38,119-123; L the four mission rows]. The old poke made the warning
   unfailable; the new assert pins the real behaviour.
5. **Docs corrected with the evidence** (the audit's own GAP-2 precedent): `07000002.md` and
   `07000002-audit.md` claimed "the writers of 0x49/0x4A/0x4D are NOT on the disc" — 0x49/0x4A are
   the hostage drones' param-6 rescue channels raised by `DroneFunc_HostageSaved` 0x1441c0 [M],
   and 0x4D's writer is 07000003's #308 [D]. `07000003.md` Start section now records the entry
   touch. No port code change was needed: the port's trigger, mission and viewer next-level
   machinery already implemented all of it; only the test's stand-in was blocking the proof.

## How it was verified

- `walkreal_07000002_live` — 1 passed / 0 failed (51.6 s): the full route above, ending on the
  07000003 entry with obj2 folded to 4.
- The 02 family: `walk_07000002_live` 2/0 (the exit refusal + load), `destroy_channels_02_live`
  1/0, `hurt_02_live` 1/0, `hostage_killer_07000002_live` 5/0, `hostage_save_07000002_live` 2/0.
- Probes (diagnostic, --nocapture): `walkreal_probe02mon` (monitors/sensors/SS params),
  `walkreal_probe03handoff` (fresh-load 0x4D at tick 1, the doors dump, the param-77/SP sweep) 3/0.
- Crates' own suites: nf-game lib 584/0 (5 ignored — includes the mission.rs monitor suite, the
  LEVELS-table test and the every-level ch-100 test), nf-formats lib 92/0, nf-audio lib 23/0;
  nf-game `sensor_live` 1/0, `sensor_radius_live` 1/0, `nis_handback_census` 1/0; nf-view
  `music_triggers_live` 1/0.
- **Total across the runs: 719 passed / 0 failed.**

### The full-suite total line — honest status

The full 885-binary suite did not run in this sandbox window:
- `tools/test/suite.py` (the fast runner) was started in the background at 22:52 UTC per the task,
  but this sandbox kills background jobs between tool calls (the level-0d WORKLOG documented the
  same); it died silently — `suite.out` holds only the build output.
- `tools/test/chunk_suite.py` (the sandbox-proof runner) was invoked with the warm target dir, but
  its build gate (`cargo test --no-run` over all four crates) ENOSPCs: the sandbox disk is 9.9 GB
  and the release `deps` tree alone is ~5 GB; 4 nf-view test binaries (`stuck_guard_combat_05_live`,
  `interact_07000005_live`, `fix_ladder_water_live`, `net_resync_live`) cannot link in the remaining
  space (211 MB of stale duplicate dep artifacts were pruned to get that far). No state-file
  results were recorded; the binaries that DID run are listed above — all green.

Nothing in this task's changes touches `nf-game`/`nf-formats`/`nf-audio` src (test + docs only),
so the risk to unrun binaries is the risk of the pre-existing tree; the next message can run
chunk_suite to ALL DONE once disk allows.

## Live checks for the main agent

One optional confirmation (the port evidence is already [D]/[M]-complete; this would be the
on-hardware confirmation):

- **The arc hand-off on the EE.** Watch address `switch_channels[0x4D]` = **0x0026fcdd**
  (base `switch_channels` 0x0026fc90 [M symtab] + 0x4D), byte, write access, during a play-through
  of 07000002's exit into 07000003. Expected: 0 written while 07000002 plays; a write of 1 within
  the first second of the 07000003 boot (Bond spawns inside TouchOnce #308's quad), before any
  player input. The same window should show `switch_channels[100]` = 0x0026fcf4 stay 0 through
  both levels.
- The known-visual list is unchanged from the audit: the 21 rigid-body props (GAP-1) are
  draw-only — not this task's blocker; no objective goes through them.

## What's left

- The full suite total line (disk-blocked here — see above).
- The rest of mission 07000001's parts (07000003/04) are other tasks' levels; this test now hands
  off to 07000003's entry exactly as the disc does.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the only
proxies I can see are wall-clock — setup from a warm sandbox ~22:50 UTC, test pass 23:58 UTC —
and the sub-agent report's own token line, which is not this session's usage).
