# real-health-00agent-06 — report

Task: finish 07000006 on real health at 00 Agent — add the 00 Agent sister
(`walk_the_castle_interior_to_the_party_real_damage_00agent`, difficulty 3, `load_with_carry_difficulty`,
asserting `Drones::difficulty == 3` at load), check the difficulty-3 rows the game reads [M], fix the
port where it differs (with addresses) and the tactics elsewhere. Real input only; no health, ammo or
channel writes; no teleports; never lower damage or health.

## Status: NOT finished — the 00 sister walks 80% of the level and dies at the courtyard mouth

The next message can continue from this branch; everything below is committed and pushed.

## What's done

1. **Branch + merge.** `s/real-health-00agent-06` cut from `origin/master` (68298698) with
   `origin/s/level-06-agent-guardroom-descent` (b32cee97) merged in — one modify/delete conflict on the
   descent's report file (master had deleted it in the batch flow; master's deletion kept). WORKLOG entry
   added. Pushed at `eacb0312`.
2. **The difficulty-3 rows the game reads are verified in the port, from the decompile [M]:**
   - Pain scale: `ReadTuningVars` (0x1d3a80) case 0x7000006 sets `Plr_DMod_Hard = 0x3f800000` = 1.0 —
     Bond takes full damage at 00 Agent. Port: `damage_factor` (bullet.rs) 05–08 → (0.6, 0.7, 1.0),
     difficulty 3 → Hard ✓; applied in the `Player_HandlePain` path [M 0x1902c8].
   - Guard accuracy: `DroneWeap_DoBulletAccuracy` (0x179740:74-88) — GameState+0x28 = 3 → ×
     `DroneFiring_Accuracy_Hard`; case 0x7000006 sets it to `0x3fa66666` = 1.3. Port: drone.rs:11896-98
     (`1 → Easy, 3|4 → Hard, _ → Normal`) ✓.
   - `DroneDamage_*` scale BOND's shots ON guards (`NDrone2_HitDamage` 0x145558; damage.rs
     `hit_damage`, difficulty 3 → `DroneDamage_Hard`); case 0x7000006 sets `DroneDamage_Hard =
     0x3f4ccccd` = 0.8 ✓ in the port's tuning.rs (bit-exact, generated from 0x1d3a80, covered by
     tests/tuning_live.rs).
   - Min-difficulty spawns: `Drone_Create`'s gate `param2 <= difficulty` [M 0x136be0:49] — the port's
     level_drones gate (drone.rs:11807/11840) ✓; the load threads the difficulty through
     `Drone_Data` (scene.rs — the descent's fix), so the 00 load builds the full roster.
   - 00-only gates: the Lock_Create auto-unlock is Operative-only [M 0x1ae1f8:67-69];
     `Lock::create`/`SoundTriggers::create` take the difficulty ✓.
   - The difficulty pick itself: GameState+0x28 = 3 via the front-end launch path [M C_SBNFDFCTY
     0x216ca8:15; the difficulty[] table 0x2df4c0 row 2 carries 3 [D]]. The sister asserts
     `drones.difficulty == 3` at load.
   - **One test-side fix:** the Agent sister's doc comment claimed `DroneFiring_Accuracy_Normal 1.1`;
     the decompile says case 0x7000006 Normal = `0x3f666666` = 0.9. The comment contradicted the
     decompile — fixed with the address (comment-only; no pass/fail change).
3. **The sister added** (`walkreal_07000006_live.rs`): `walk_the_castle_interior_to_the_party_real_damage_00agent`
   → `walker::with_real_damage(|| walk_the_castle_interior_to_the_party_at(3))`, on the descent branch's
   difficulty-parameterized walk body. All new tactics are gated `difficulty >= 3`, so the Operative and
   Agent walks keep master's choreography byte-exact (proven: the whole 06 binary passes 6/7 with them
   green).
4. **The 00 tactic tier (all measured on this branch's runs, real input only):**
   - `clear_the_spot(w, radius)`: fight every alive guard within the radius with a bounded budget per
     target (6000 ticks) and a skip list — a guard still standing after the budget is off the ray and
     is left for the reactive return fire. 13 u for the guardroom (the room spans x 27.9..36.5 [D]),
     8 u on the open descent. This was needed because the 00 roster's min-difficulty-3 placements
     (#6/#10/#18/#28) pack the guardroom beyond what the Agent-calibrated fights handle.
   - Doorway + nook + cut-stand clears; the Laser cut moved to the stealth route (`cut = difficulty >= 3`).
   - The Laser draw retries the equip up to 240 ticks — the request only lands when idle
     [M Player_WeaponChange 0x1a6f80], and a just-finished fight leaves the gun mid-anim (the 00j run's
     40-tick window raced it).
5. **Measured progress of the 00 sister (the run ledgers, this port):**
   - Original state (no 00 tier): died tick 13669 in the guardroom/courtyard, 34 hits for 100; the
     guardroom crossing alone cost −71.6 (91.8 → 20.2) vs ≤4 at Agent — the direct consequence of
     accuracy 0.9 → 1.3 and pain 0.7 → 1.0, no port bug.
   - With the tier: the guardroom is SOLVED — the doorway clear + the chaser kill (#27: an
     outside-north post the stands can't answer, it chased Bond and machine-gunned the cut for 61.0
     with hp 10 left in run 00i; the cut-stand re-clear handles it) + the fusebox cut completing
     (ch 57 on, heads stay dark [M Searchlight_Update 0x1cb900:88-92 — the off-channel check precedes
     the tracking machine], no searchlight alarm). Bond crossed the pilaster lane and the balcony drop
     to the mouth.
   - The remaining death: tick 13143 at the courtyard mouth (7.61, 42.03, −14.73), 16 hits for 100:
     #12 (skin 0x500002e, weapon 0x25) — a 00-roster guard on the tunnel deck at y 47.0 — lands ONE
     45.0 hit; #11 (the tunnel-roof guard, y 55.5) chips 33.5 over 7 hits; #19/#20 (the ch-55
     reinforcement pair) spawn anyway — the courtyard's own ch-55 volume (touching [], a volume not a
     searchlight) raises the alarm regardless of the cut, so the stealth route does not silence THEM.

## What's left (the next session's named next step)

The mouth phase needs its own 00 tier before the descent clears matter:
1. An aimed-stand mechanic vs the high-y guards (#12 at y 47.0 on the tunnel deck, #11 at y 55.5 on
   the roof): `clear_the_spot`'s `pos[1] < 46.0` filter excludes them by design, and the reactive fire
   cannot out-trade a 45.0-damage weapon at 00. Measured negative result to build on: raising the
   clear's y filter to 60 pulled in the unreachable perch guard #23 (y 51.0 at (38.2, 51.0, −32.3)),
   stalled the walk ~40k ticks in futile stands and died worse (run 00m) — the filter is correct and
   was restored; the high posts need stands from spots whose gun rays clear them (the 0c sister's
   aimed-stand model: settle, hold aim to the bit, cursor-aware convergence), not ground sweeps.
2. The reinforcement pair: with the searchlights cut they still spawn from the courtyard ch-55 volume —
   either fight them at range from the mouth plaza before they close (they spawn at the strip posts
   (11.6, 42.0, −35.1)/(13.2, 41.7, −34.8)), or re-time the mouth crossing ahead of the alarm.
3. Then re-run: the whole `walkreal_07000006_live` binary, `rook06_realpath_live`, `nf_game` (all green
   here already except the 00 test itself), plus the walkreal sweep and suite chunks.

## Verification (this session, on the final tree = master 68298698 + the descent merge)

| Binary | Result |
| --- | --- |
| `walkreal_07000006_live` (whole, 7 tests) | 6 passed, 1 failed — the failure IS the new 00 sister (dies at the mouth, above); Operative + Agent real-damage sisters and all pins stay green |
| `rook06_realpath_live` | 1 passed, 0 failed |
| `nf_game` | 656 passed, 0 failed |
| 00 sister iterations | 13 measured runs (00a–00m), each rebuilt first; the ledgers quoted above are in the session logs (not committed) |

The full suite did not fit: the sandbox kills background jobs between tool calls, so only foreground
chunks ran; the 06/rook/nf-game groups above are the lesson-10 set for the touched file (test-only —
no `crates/*/src/` change this task; the branch's diff vs master touches the 06 test file, WORKLOG,
this report, and the merge).

## Live checks for the main agent

1. From a fresh 00 Agent boot of 07000006: at the courtyard mouth, confirm the tunnel-deck guard
   (y ≈ 47) fires the weapon the disc calls 0x25 and that one hit on Bond costs ≈45 × 1.0 (Plr_DMod_Hard)
   — pins the #12 hit against the real damage path.
2. Confirm the courtyard's ch-55 volume raises the alarm even with the fusebox cut (ch 57 on): walk the
   mouth with the searchlights dead and watch ch 55 — the port says the volume, not the heads, writes it.

## Usage

Not visible to me: this environment exposes no token, cost or context-window figures for the session.

## ntfy failed

All three prescribed fallbacks failed; exact outputs:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20002 milliseconds`.
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'real-health-00agent-06: blocked', headers={'Title':'NightfirePort'}), timeout=20)"` →
   no output within the window (the previous task's identical call failed with
   `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>` — ntfy.sh is not
   reachable from this sandbox's network).
3. `wget -q -O- --post-data "real-health-00agent-06: blocked" https://ntfy.sh/nightfire-yoda-8k3x` →
   connection hang, killed at the 60 s tool timeout.

Everything else (GitHub push, finish.sh) succeeded.

