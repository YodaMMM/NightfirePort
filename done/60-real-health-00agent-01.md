# 60-real-health-00agent-01

walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent` (ignored):
on 00 Agent the walk leaves the ch-0xF pocket at 57.4 health but only #17 dies; #12 and #3 survive and the
contact is killed in the pocket at t50960 (mission fail). First check whether the contact's death is the port
doing something the game does not (guard target choice against the contact, his health, cover), then re-route.
Un-ignore when green.

---
claimed-by: sandbox1 at 2026-11-22T17:20Z

---
## Report (done, 2026-11-22T19:17Z)

Closed by the driver: the report is on `s/real-health-00agent-01` (docs/reports/real-health-00agent-01.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-22T19:23Z)


---
## Report from the worker (done, 2026-11-22T19:23Z)

# real-health-00agent-01

Task: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`
(the 01 00 Agent real-damage contact walk) — the driver's order: first check whether the
contact's death is the port doing something the game does not, then re-route. Un-ignore when
green.

## Done

1. **Branch + ledger continuation.** `s/real-health-00agent-01` created from master
   (dbf17a37d); `origin/s/hard-walks-d` merged (its ignore note is the newest ledger on this
   test and was not on master yet). WORKLOG entries appended (never rewritten).

2. **The port-vs-game check: FAITHFUL — the contact's death is not a port difference.**
   Evidence (decomp read on this branch; every claim in the test's ledger comment too):
   - Guard target choice: `NDrone2_FindOpponent` 0x1424e0:100-121 — the SP `team != 2` arm
     takes the player object unconditionally and returns (no sight test); `SetOpponent`
     0x1422d0:19-20 stores it with no team filter. No branch lets a team-1 guard target a
     team-2 ally. [L] `work/ee_start_07000001.bin`: all 14 team-1 guards hold +0x170 =
     Bond's obj (0x18f5790); the contact (skin 0x0500_004f) holds 0. Port drone.rs:21302-21313.
   - Crossfire landing: `NDrone2_DealWithObjHit` 0x13fff0 case 5 — a team-1 shooter's round
     lands on any victim not team 1 (bVar3/bVar4 fold; teams 0/≥4 count as team 1's side);
     delivered via `Drone_CollisionHandler` 0x140968:84-95/209-220. Port drone.rs:27417-27430,
     gate-for-gate.
   - Contact health: no placement health field — `NDrone2_DefaultInit` 0x14b300:52 sets
     +0xAC = 10.0; difficulty scales damage only (`NDrone2_HitDamage` 0x145558:135-156), never
     spawn health. [L] the same snapshot: contact team 2, health 10.0, dtype 12, voice 9 —
     2-3 guard rounds kill him.
   - Cover: `NDrone2_NotNearArmedDrone` 0x177640 (sole caller `AllyLeadHide` 0x15ec08:30,
     radius 14.0) — hides while an armed team-1 peer is within 14 u and seen/latched; the
     port matches (drone.rs:21727-21786 + LEAD_HIDE 10598-10626).

3. **The reproduce runs changed the story** ([L] 4 real runs on the merged tree, full
   NF_HEALTH_TRACE/NF_ALARM_TRACE): the contact overtook Bond during the ambush approach
   (west-armour exit t24200 at (-23.97,14.52) → stairwell head by t24900) and died to **four
   PLAYER rounds** (3.5 each, part 4, `by_player true`, t25015-25440) — the 624-tick #12
   stand's full-bloom P2K spray landing 0.66-0.70 u off the aim line at 2.5-3 u = 0.22-0.28
   rad, i.e. the spread cap [M bullet::player_spread's cap]. His center sat 0.03-0.32 u off
   EVERY lane line from the mouth. The ledger's t50960 was the mission-fail restart
   processing, not the death.

4. **The route fix (all in walkreal_07000001_live.rs, d3-gated, no shared-code change):**
   - `contact_blocks_line` — the alive contact blocks any guard whose eye line passes within
     0.2464·dist + 0.12 (the cap cone; a fresh-barrel cone gate stays open under sustained
     fire). Applied at every d3 ambush fire site (the lane fight, the band-edge arms, the
     bounded-rounds arm) and as the recovery-loop break condition.
   - The stairwell-head fight stance (-30.8,6.4,13.1): when the contact holds the lane, Bond
     takes the head — the lines from there clear the parked contact (#12 0.99, #3 0.75) —
     and fights the pocket pair from the ×2 band edge with head aim.

5. **Measured results:**
   - run2 (gate on): every lane stand skipped; **zero by_player damage on the contact for the
     whole run** — Bond's friendly-fire kill is closed. Bond then bled out during the escort
     stall (t56212) to the surviving #3/#12.
   - run3/4 (gate + head fight): **#12 killed from the head** (t27257-27273, two 3.5 part-4
     rounds) but the ×2 close trade + #17's approach fire cost 36.4 of 57.4, and #3's 3.6 u
     SMG grind finished Bond at t27705. The parked contact then took guard spray (by_drone
     4/17, weapon 18) — his cowering boxes sit in the spray shadow of every lane-adjacent
     stance.
   - run5 (the window lever): dead — died t25396 with no window kill; run6/7: the restored
     state reproduces run4 exactly (dies t27705).

6. **Verification of what shipped:** the whole `walkreal_07000001_live` binary ran:
   **7 passed, 0 failed, 1 ignored** — the d1/d2 real-damage sisters
   (`..._real_damage`, `..._real_damage_agent`) and all other tests in the binary pass, so
   the d3-gated changes are byte-identical below difficulty 3. No file outside
   `crates/nf-view/tests/walkreal_07000001_live.rs` and docs was changed; no
   `crates/*/src` change, so no other binary needs a re-run. The walk test itself stays
   IGNORED with the measured ledger and the next lever in the ignore message.

## What's left

- The walk is still red. The measured wall: while the contact is parked at the stairwell
  head, #3 (the 24-hp captain) is unkillable from any lane-adjacent stance without firing
  through him, the guards' own spray crosses him, and the #12-class close trades cost ~18.
- The pre-arrival window lever was TRIED and is MEASURED DEAD ([L run5]): removing the -19
  retreat put #12 in the x2 band at 2.8 u — Bond died earlier (t25396 vs t27705), no window
  kill landed, the parked contact took guard spray again. Restored, the state reproduces run
  4 exactly (dies t27705 [L run7]).
- Structural finding ([M] the walk's placement dump + [L] runs 1-7): the pop slab covers the
  corridor's full width and is Bond-touch only, so the contact can never descend before the
  pop; while he parks at the head, every pocket guard is unkillable under the bloom gate, and
  the pocket must be cleared for his hide to release (the descent stalls otherwise).
- The next levers (in the ignore message): (1) the FAST-KILL budget — bait #3's advance out
  of the pocket (measured to (-30.7,14.7) in run2, where the mouth line clears the parked
  contact at 0.9 u) and clear the pocket within the contact's 10-hp spray budget; (2) a
  walker-side guard-spray shield for the contact (the mirror of the player-side gate).
- The full 4-crate suite did not fit in the sandbox window (the dep tree rebuilt from scratch
  after the reset + the binary runs above took the remainder). The binary that contains the
  changed file ran whole and green apart from the target walk's ignore.

## Live checks for the main agent

1. `tools/trace/gdbwatch.py` on the contact's health word during the 01 pocket fight (contact
   drone +0xAC): expected — the game's contact takes the same crossfire (guard rounds aimed
   at Bond landing on him, 1.5-3.5 per hit) when the player fights the pocket from the lane;
   this confirms the port's case-5 landing is game-faithful under real play, not just
   decompile-reading.
2. `tools/trace/fieldprobe.py` poke check: write the contact's +0x170 (opponent) while a
   pocket guard is sighted-attacking — expect the game to keep it 0/unchanged (team-2 allies
   never acquire opponents), matching `NDrone2_FindOpponent` 0x1424e0's early return.
3. A render/spot check at `--bond -29.93,6.0,16.35` (the lane mouth) with the contact parked
   at (-30.24,6.34,13.92) and guard #12 at (-30.0,6.34,12.5): confirm visually that the
   contact's cowering model fills the lane mouth's field of view to the pocket (the walk's
   geometry claim behind the gate).

## Usage

Not visible to me for this session as a whole (no session-level token/credit/cost readout is
exposed in this environment). The two research subagents reported, at their final messages
(where shown in their transcripts): Explore agent 1: prompt 195591, completion 11526, total
207117 tokens; Explore agent 2: prompt 184244, completion 7969, total 192213 tokens.
