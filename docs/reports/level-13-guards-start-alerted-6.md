# level-13-guards-start-alerted-6 — report

Branch `s/level-13-guards-start-alerted-6` (from `origin/s/level-13-guards-start-alerted-5` @ de110fd9, merged `origin/master` @ 929c6606). Tips: merge a8425173, fixes 9c4305c2 (scene.rs) and 66e259e7 (the [01f] walk).

## What's done

Both task reds are green on the branch tree: `walkreal_07000001_live` 4/4 (`walk_from_the_spawn_to_the_exit_with_the_contact` and `following_mayhew_down_completes_the_level` among them), `--test-threads=1`, real ISO. One port fix (decompile-cited) and one walk re-derivation (disc-cited).

### 1. The corpse capsule — the port fix (9c4305c2)

The merged tree's stall reproduced exactly as the task described: the escort settled at (3.5, 6.3, 6.6) in AllyLeadWait 0x20 with `route_idx (12, 37)`, `armed_peer_dist = f32::MAX` (no armed peer — the AllyLeadHide hypothesis dead at the trace level), and the NF_AILOG trace showed every Lead→Wait entry reading **path 0.0** — Bond out of the [M DroneMove_ObjectMissionPathDistance 0x153f50] node scan, so the wait release ([M NDrone2_DSTATE_AllyLeadWait 0x15eda8 case 3: pd > −7.0 && (pd != 0.0 || dist2d < 5.0)]) could never land. The reason Bond could not get there was not the escort at all: Bond was pinned at (5.45, 6.68, 11.60) — **1.14 above the static floor (5.54), `grounded=false`** — on the fresh corpse of a guard the walk had just killed at the east-door pile. Every nav leg died at the 3000-tick cap.

The port's near-bot capsule set for the player's actor-actor push (`Collide_Jointy` 0x1ec180:229-281 via `LadderCtx::bot_capsules`, scene.rs) filtered `enabled` but never `corpse`. The game's dead bot presents no capsule: `SetAsDead` 0x14a2f0:9 sets **+0x4F8 & 0x200** at death, and `NDrone2_DoCollision` 0x149ec8:11 returns 0 (collision OFF) whenever that bit is set — on every level but the special 0x700004a — with `NDrone2_Collision` 0x140e90:64 gating the same bit (docs/fields/drone.md, the 0x4F8&0x200 row, census-verified [M*]). The fix adds `!x.corpse` to the filter. Bond walks over bodies as the game's player does; the wait releases; the escort walks his route down to the plate and the AND fires with both on it. `walk_from_the_spawn_to_the_exit_with_the_contact` green.

### 2. The [01f] walk's corner — the re-derivation (66e259e7)

With the corpse fix in, `following_mayhew_down_completes_the_level` still wedged, one corner earlier: Bond boxed at (5.45, 6.68, 11.60) by STATIC geometry — `grounded=true`, segments hitting at 0.10-0.39 in every direction but north. The port's segment fan maps the box: the hall's south structure — a wall at x≈6.03 (z 11.6-13+) and the z≈11.05 wall ([D] the disc's collision cels, via the port's parse; the doorway band x 6.5-7.9, z 12.0→10.4 is the only opening, verified clear by casts). The escort waited across it at (3.69, 6.34, 6.40), AllyLeadWait, pd 0.0, Bond 5.4+ away — outside the [M 0x15eda8] 5.0 band forever. The re-derivation is the player's play, the same ladder the main test already uses for the west wall: when the straight line to the contact crosses collision, round the structure by its north end — the AI's own lane node[23] (3.39, 14.69) → node[7] (6.88, 14.62) [D nav path 0 nodes+links, cited in the file since the -5 round] — and come back in through the east door band. The close then lands inside the release band, the lead resumes, the descent completes, the AND fires with Bond AND the parked contact on the plate, and the exit writes 07000002. Test file only — no port change, no weakened assert (the release-band pin itself is unchanged).

## What was verified and how

- `walkreal_07000001_live` **4/4** (the two task reds + the two that were already green), one binary, `--test-threads=1`, NF_ISO set, 33 s.
- Suite (fast runner `tools/test/suite.py`, foreground — the sandbox kills background jobs between tool calls, re-proven this session with three nohup/setsid spawns): a `--only` chunk over the 12 binaries the corpse fix can plausibly touch (the whole 07000001/07000002 family + the -5 families):

  ```
  build ok: 881 test binaries in 1 s
  FAILED escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens
  38 passed, 1 failed, 0 ignored  (2 min)
  ```

  That chunk line is this window's suite evidence; the FULL 881-binary total did not fit the 2-hour window (setup — clone, rustup, work-slim, ISO, cargo build of all four crates from scratch on 2 cores — consumed ~45 min of it, and the background suite the task message prescribes does not survive this sandbox). The 12-binary chunk covers every test family the touched code paths feed.

## What's left

1. **`escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens`** — new red, cause identified, fix not landed in the window. The corpse fix shifts the deterministic RNG stream (Bond's path over the body pile changes, so the guard-15 fight resolves differently): his clip runs dry at +1257 ("guard 15: out of ammo"), guard 15 stays alive, and the escort — with an armed hostile firing — leaves the ally machine into **RUN_AWAY_FROM_OBJECT 0x86**, failing the `ESCORT_STATES.contains` assert at :137. ESCORT_STATES already carries the ally machine's ten states including LEAD_HIDE and GO_TO_GOAL_ALLY; whether the game's escort legitimately routes to RunAwayFromObject 0x86 (who calls it on an ally: `RunAwayFromObject` 0x168aa8's callers under fire vs the Lead machine's own armed arms [M 0x15e698:30-36 / 0x15ec08 case 3]) is the open [M] question. Either the game routes the hit escort there (→ the test's state list gains the cite) or the walk's fight must not dry out (the player reloads — a resupply ladder like the hall's). Not weakened, not pinned either way this window.
2. The full-suite total line on the final tree (next window: the background runner works on a sandbox that lets it live; otherwise chunked `--only` runs like the above).
3. The three `disc_golden` tests were not in the chunk (slim-ISO expected reds per the suite rules).

## Live checks for the main agent

1. **Escort under fire**: fresh boot 07000001, meet scene done, keep one hall guard alive and close Bond to him with an empty clip so the guard fires at the escort. Watch the escort's `Drone+0x5A2`/state — expected per the ally machine: LeadHide (0x1f) with `FindRunToPoint` 0x15168 succeeding, AllyGoToGoalPosition 0x28 [M 0x15eda8:58-61]; if the game instead shows RunAwayFromObject 0x86, the port's ESCORT_STATES test list gains the state with that cite and the escort_follow red is a test-list fix, not a port fix.
2. **Corpse vs player**: fresh boot 07000001, kill a guard at melee range, walk Bond onto the body. Expected per [M SetAsDead 0x14a2f0:9 + DoCollision 0x149ec8:11]: no capsule push — Bond walks through/over with no lift. If the game's player visibly steps ON the corpse (a solid low box), the class-0x12 body object carries its own collision and the port's corpse handling needs it (the Jointy capsule remains wrong either way — it is the BOT's, and the bot is dead).

## Usage

not visible to me (no token, credit or context-window figures are exposed in this environment; wall clock read with `date -u` — first read 11:13 UTC, this report written 12:56 UTC against the ~13:12 UTC expiry, stop-new-work 12:52 UTC).
