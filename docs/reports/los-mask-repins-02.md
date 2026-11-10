# los-mask-repins-02 — report

**Status: blocked (out of sandbox time mid-fix).** The 07000002 walks' diagnosis is complete and
pushed; the route fix for the one ignored test is identified but not landed. The branch
`s/los-mask-repins-02` carries the evidence and the temporary diagnostics so the next session can
finish the re-route.

## What was done

Branch cut from origin/master c120d7b49 (contains 823ff190b, the game's guard sight). The task's
one ignored test is
`walkreal_07000002_live::walk_the_castle_all_objectives_real_health_00agent`
(d3 sister of the passing d1/d2 walks):

> "the mission ends (level_exit) on the stair-bottom leg at tick 12902 and the doorway sensor's
> ch 0x81 never rises; cause not traced"

### The traced cause (16 s repro on master)

1. The mission **fails**: `level_exit Some(0x07000002)` with `restart=true` —
   `fail_choose_restart` [M death.rs 0x216618: `level_exit = Some(level_current)`]. Not the exit:
   Load Point #206 never fired (`gone=false`, `gate 0x00`), and NF_EXIT_DBG shows no script
   LoadLevel event.
2. The trigger is the scared civilian's death channel **ch 0x60 ON** → FAIL_CIVILIANS — the
   mission fails whatever the killer [M 0x1827d8:98-107; the death channel raises on any death].
   The civilian (placement #464 = drone #22, team 3, skin 0x5000034) is a corpse at
   (-40.6, 6.3, 1.9) — her hide spot [D census] — killed by a **guard** round:
   `hit_by_player false`, hit `(weapon 0x12, part 5, damage 1.5)`.
3. Round trace [L the diagnostic on this tree]: t12330 / t12586 / t12598
   `#10 (placement 452) → #22`; with the stair-top stand removed the killer is
   `#11 (placement 453) → #22` at t12239. Both are the west-end rifle guards (0x12):
   #10 at (-54.6, 5.35, -0.95) lower west, #11 roams to (-69.3, 6.27, -3.46) upper west.
4. Geometry: their lines to a Bond on the **upper-east approach** (y ≈ 6.37, z < 4) pass
   0.05–0.5 u from her hide spot — inside the ±1.5-u miss wobble [M DroneWeap_DoBulletAccuracy
   0x179740:136-148]; from the stair mouth down, or from the west hall, the lines pass 3+ u
   south of her. The exposure window is the final upper-east stretch itself: any sight turn that
   lands while Bond crosses it can kill her, and on this tree both guards' turns do.
5. No pre-clear from the upper hall: the probe [L] at "back from the west-wing armour"
   (-46.37, 5.87, 7.15) shows #9/#10/#11/#12 all `exposed None` (walls).
6. The d3-only stair-top sniper stand (1200 ticks) never fires (the sniper is not exposed from
   the stair top on the sight-corrected tree — clip 32 unchanged across the stand [L]) and only
   parks Bond inside the exposure window. Removing it (committed) moved the kill from #10 t12330
   to #11 t12239 — the window is the problem, not the stand.
7. At d1/d2 (passing runs) the round trace is **empty** — no guard round hits any drone the whole
   walk, so #10/#11 never fire across the lane there. Either their placements are d3-only
   (Drone_Create's gate [M 0x136be0:49: param 2 = minimum difficulty]) or the d1/d2 timeline
   never alerts them before the descent. Not yet determined — next session's first check.

### Commits

- `65be4a69d` branch cut + WORKLOG.
- `13fd56597` the diagnosis: WORKLOG entry + temporary diagnostics in
  `walkreal_07000002_live.rs` (stair-bottom d3 branch prints level_exit/channels/LOAD_LEVEL
  state/team-3 corpses/live guards/round hits; a visibility probe after the west-wing armour
  detour) + the vestigial stair-top stand removed (never fires on this tree; strictly less
  exposure). These diagnostics must be removed once the route passes.

## What is left (the fix)

1. Determine whether placements 452/453 are d3-only (param 2) or alerted late on the d3
   timeline.
2. Re-route so the upper-east crossing is not exposed to them. Candidate levers, in order:
   beat the alert (if alerted late — reach the stair bottom before the first sight turn), clear
   them from the lane side before the descent (they are not exposed from the upper hall, but the
   lower west room's entrance may be reachable from the lane — needs a route probe), or the
   silenced-P99 west-wing kills (change the geisha-1 fight's weapon so the west end never
   alerts; a full d3 re-tune).
3. Remove the temporary diagnostics, run the walk to green, un-ignore, and run every
   `walkreal_*_live` sister + `interactables_07000002_live` again.

## How this was verified

- The 16 s repro of the ignored walk on master (fail at the ch-0x81 assert, tick 12902).
- NF_LEVELLOG (Load Point #206 never fires), NF_EXIT_DBG (no script level event), and the
  test-side diagnostics (level_exit value/restart flag, ch 0x60, the corpse's hit record, the
  round-hit trace, live guard positions, the visibility probe) — all quoted above.
- The d2 comparison run: the d1+d2 walks pass (2 passed / 0 failed / 1 ignored).

## Tests run (release, --test-threads 1)

| binary | result |
|---|---|
| walkreal_07000002_live (d1 + d2) | 2 passed / 0 failed / 1 ignored |
| walkreal_07000002_live (d3, the ignore) | FAILS — the traced cause above |
| interactables_07000002_live | 18 / 0 |
| walk_07000002_live | 2 / 0 |
| hostage_killer_07000002_live | 5 / 0 |
| hostage_save_07000002_live | 3 / 0 |
| findcel_tie_break_07000002_live | 1 / 0 |
| census_07000002_drones | 0 / 0 / 1 ignored (its own long-standing ignore, untouched) |

No `crates/*/src/` changes, no `tests/common/` changes — so no cross-binary re-run obligations
this session. The full suite did not fit: see "Environment limits".

## Live checks for the main agent

1. On 00 Agent (fresh boot, level 0x07000002), watch drone placements 452 and 453 (skin
   0x5000036/0x5000038): do they exist on Agent (difficulty 2) too? Read their placement param 2
   (the minimum-difficulty tag [M 0x136be0:49]) from the level file — gdbwatch on
   `Drone_Create` 0x136be0's gate would settle it.
2. During a real playthrough's upper-east crossing (the west-wing hall → the stair top), do
   guards 452/453 open fire across the courtyard at Bond? If the game's own sight never engages
   them there, the port's cel-walk scope has a leak on this lane portal — that would move the
   fix from "re-route" to "sight bug".

## Environment limits (why the full suite did not run)

- The sandbox disk is 9.9 GB total. `nf-view` alone has 1094 test files (auto-discovered test
  binaries, ~20 MB each) — the full four-crate binary set is far beyond the disk; suite.py's
  build phase hit linker "Bus error" (ENOSPC) repeatedly. I stripped the built binaries to make
  room and ran the 07000002 set instead.
- Background processes are killed between tool calls in this sandbox, so the prescribed
  `nohup python3 tools/test/suite.py &` start does not survive; builds ran in foreground chunks.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.

## ntfy failed

All three fallbacks failed (2026-11-10 ~21:20 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "los-mask-repins-02: blocked …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'los-mask-repins-02: blocked …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "los-mask-repins-02: blocked …" https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 60 s, killed (no output).
