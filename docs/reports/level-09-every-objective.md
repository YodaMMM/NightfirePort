# level-09-every-objective — 07000009 completable end to end on real input

Branch `s/level-09-every-objective` (from origin/master ad2f0791). Sandbox 20:01–22:00 UTC.

## The coverage survey (the task's first ask)

[D mission/c07000009.rs — the level's 3 objective records; L ee_start_05.bin @0x2a3f78]:
0x4000016 (ch 0x61, flags 2 = fail-objective), 0x4000017 (ch 0x46), 0x4000018 (ch 0x47, enable
0x46). **The existing `walkreal_07000009_live.rs` already walks every one of them on real input,
in player order, ending on the level exit firing — there were no missing legs to add:**

- **obj 0x4000016** ("Don't let the security guards raise the alarm"): the walk asserts ch
  0x7C/0x7D/0x61 never rise from the spawn through the exit (state 2 = un-failed at the end
  [M 0x1827d8:32-51 flags&2]); the FAIL arm is pinned twice more — `the_alarm_switch_fails_
  objective_1` (a real E press on a ch-124 alarm switch → OR mux #278 → ch 0x61 → ch 99 → state 5)
  and `security_alarm_09_live` (a guard's full real sight chain: seen → CivilianChallenge →
  RunToAlarm → PressAlarm → ch 0x61 → the mission fails).
- **obj 0x4000017** ("Get to the lobby and activate the main elevator system"): asserted COMPLETE
  (state 4) on the REAL panel E-press — the class-0 script entity of SP #70
  [M Player_Activate 0x1a5170:121-122 → SP_Activate; D script 0x0600059B] → ch 0x39 → AND mux
  #274 → ch 0x46 [D].
- **obj 0x4000018** ("Take main elevator to office floor"): asserted ANNOUNCED (state 3) at the
  exit — its enable ch 0x46 fired at the panel [M 0x1827d8:55-67]; ch 0x47 has **no writer on
  07000009** (it is written on 0700000a), so it cannot complete on this level [D census + the
  level doc's objective table]. It completes on the next level's walk.
- The exit: TOUCH_ONCE #67 in the cab → ch 0x32 → LOAD_LEVEL #63 → `level_exit = 0x0700000a`,
  the 300-tick fade [M 0x1b2318:112], the end-marker music jump [M UpdateMFX_03TowerA 0x113558].

Verified GREEN on this merged tree: 3/3 tests pass (the exit fired 28 ticks into the cab walk;
4010 ticks start-to-finish; 0 stuck, 0 falls).

## The one remaining bar and what it found

The walk still uses the documented `hold_npcs = true` dev hold for its guard AI — a state hold,
which the task's "no state pokes" bar excludes. I attempted the live-guard walk (v1–v3) and hit
a real finding; the walk is REVERTED to the committed passing version (the hold is the same
documented facility `elevator_buttons_09_live`, `security_alarm_09_live`, `campaign_flow_live`
and `hold_npcs_live` use), and the live-guard work is the recorded open item:

- The car-park sentry (−3.9, −376.0, 36.0 [D census]) never sights the spawn: the walk sat 732
  ticks beside him with the alarm low — Bond's east route to the stairs is safe unshot.
- The stair-head pinch guard (drone 4, mid-stairwell at (20.59, −372.08, 22.00)) holds
  CIVILIAN_CHALLENGE [M NDrone2_DSTATE_CivilianChallenge 0x162558; state 0x3E] and takes **ZERO
  damage from 15+ rounds at 1.2–7.7 u point blank** across three runs — the rounds fire and drain,
  no damage, no flinch, health stays 10 (k15 13 [D drone-props]). Either the port's bullet-vs-drone
  pick skips a challenge-state drone or the mid-stair pose sits behind geometry the pick ignores.
  **This is the open port-gap item for the next session** — the hooks to read: the port's
  bullet-vs-drone pick, `NDrone2_BulletImpact` 0x1461c8, the challenge state's hit handling, and
  whether the game lets challenged Bond fire at all (a live check below).
- The sight chain's timing gives a real kill window (the challenging guard must RUN to a switch
  before PressAlarm [M 0x162558 case 1 RunToAlarm 0x89]), so a fixed pick would make the
  live-guard walk feasible with the ammo the 09 start loadout carries.

## What must DRAW differently

Nothing new this session (the level's open draw gaps are the audit's GAP-1/2/4: the switch press
anims, the hologram rotor spin, the glass-shatter shard NIS — all listed in
docs/levels/07000009-audit.md for the main agent).

## Live checks for the main agent

1. **Can challenged Bond damage the challenging guard?** In PCSX2, stand in front of the
   stairwell guard until he challenges (state 0x3E), fire point blank, and watch the guard's
   health float (`watch` the drone's health word) — the game's answer decides whether the port's
   zero-damage symptom is a pick gap or correct behaviour.
2. **The challenging guard's hit response** — `gdbwatch` the guard's `NDrone2_BulletImpact`
   0x1461c8 entry while firing: does the pick ever reach it from a challenged state?

## Verification summary (what ran)

- `walkreal_07000009_live` (all 3 tests) — GREEN on the final tree (quoted above).
- Suite slices (suite.py, `--only`, final tree): units + the 09 family
  (nf_game/nf_formats/nf_audio/nf_view, walk/walkreal 09, security_alarm_09, elevator_buttons_09,
  lift_indicators_09, interact_07000009, census_07000009, hold_npcs, mission_outros, progress):
  **`830 passed, 0 failed, 7 ignored  (2 min)`**; neighbour walkers + mission flows
  (walk/walkreal 0700000a, walk 07000008, campaign_flow, level_end, gameflow_stack_sp_run,
  mission_stats_flow, level_carry): **`39 passed, 0 failed, 3 ignored  (9 min)`**.
  Zero FAILED/TIMEOUT/CRASH lines in any slice. The full 883-binary suite needs ~3 h on this
  sandbox (measured ~44 binaries per 9.5 min last session) and did not fit the 2-hour window;
  the untouched tail is the next session's first run.

## What's left

1. The live-guard walk: root-cause the challenge-state zero-damage symptom (item above), then
   re-land the walk without `hold_npcs` (the v3 shape is in this branch's history: clear each
   patrol beat from a standoff, walk-to-guard before shooting, resupply on dry).
2. The full-suite tail (~800 binaries).

## Usage

My own session's token/context/cost figures are not visible to me (no such readout in this
environment). The only figures I could read were the per-subagent turn readouts of the delegated
shell runs (the main session's Bash wedged mid-session), e.g. the last one:
`Tokens -> prompt:9347 completion:482` — per-call subagent counts, not session totals.

## ntfy failed

All three fallbacks failed at 21:36 UTC — ntfy.sh rejected every publish with the daily-quota 429
(the report went out via the queue entry instead):

```
{"code":42908,"http":429,"error":"limit reached: daily message quota reached; increase your limits with a paid plan, see https://ntfy.sh","link":"https://ntfy.sh/docs/publish/#limitations"}
```

- curl: HTTP 429, the JSON above.
- python3 urllib: `urllib.error.HTTPError: HTTP Error 429: Too Many Requests`
- wget: empty output, same server-side 429.
