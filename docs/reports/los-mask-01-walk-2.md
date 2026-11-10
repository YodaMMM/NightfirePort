# los-mask-01-walk-2 — report

Task: take `los-mask-01-walk`'s "What's left" in order on the merged tree (branch
`s/los-mask-01-walk-2` = origin/s/los-mask-01-walk d8c32a91d merged with current origin/master;
the one merge conflict in `walkreal_07000002_live.rs` resolved both-ways: the masked-tree d2
Mayhew shield block kept + master's `real-health-00agent-02-6` d3 hooks kept, the superseded
d3 scoop+arm_fullest hook dropped).

## Done

1. **08 first, per the prior report.** The captured whole-binary rerun on the merged tree names
   exactly ONE failure — the two unledgered ones now pass:
   - `walk_the_whole_mission_on_real_input_real_damage` wedged at "the stair head": the
     walkway-head return-fire fights (the back-off fight's own forward −1.0
     [M damage_log.rs crowd]) herded Bond onto the walkway door (160.6,45.0,−100.1) WEST frame
     pocket — (159.6-160.3, 46.1-46.9, −99.0 to −99.4), grounded false, 0.74 above the crossing
     deck, head_bump on the frame — where every southward target wedged at the threshold for
     seven 3000-tick caps [L the captured rerun; the plain no-fight crossing walks the same lane
     at y 46.08 through the open door]. FIX: the gated settle (real-damage walks only, after the
     walkway-head leg) walks Bond back out of the pocket along the lane he came in by — the
     plain crossing's nav-3 arrival (159.75,46.08,−95.86) [L] — and the rest of the lane is
     retaken with the fights done. No damage/health/RNG changes, real input only.
   - With the settle the walk reaches the HIND FIGHT (rocket 1 lands 199.5 of the machine's 299,
     one rocket left in the clip) and dies at the bottom terminal t53951: the world 39.4 in 22
     hits — five −1.2 zaps every 20-25 ticks (frames 53881/53906/53929/53935/53948, the
     live-wire/hurt-volume class) at the ride's end (111.08,13.52,−179.72), then −20.09
     (a 5.3-u detonation, 11 ticks after rocket 1's impact) — plus the masked long-range drains
     #5 10.8 in 6 hits at d 80.6 and #14 9.6 at d 98.2 (ranges the strict pre-mask ray blocked
     [M 0x1eae80:70-87 + 0x1ebaa8:300-319]). **Ledgered into its ignore** with three next
     levers: (1) the terminal walk-out (the retired 16-ray exit arm in the fight loop), (2) the
     05 pre-engage lever for the long-range drain placements, (3) a strafe dodge of the
     machine's incoming rockets (Input.strafe exists). The Agent sister's ignore updated (its
     captured-rerun lever is done). Whole binary: **3 passed / 0 failed / 3 ignored**, 109 s.

2. **The walkreal sweep, one level per tool call** (whole binaries, `--test-threads=1`, real
   input, the slim on-foot ISO):
   - GREEN: 0a **13/0/2**, 0c **3/0/1**, 0d **1/0/2**, 15 **7/0/2**, 16 **15/0/3**, 1b **16/0/3**,
     41 **15/0/0**, 4a **4/0/0** — the masks shifted nothing on these levels.
   - 11: the Operative + 00 Agent walks wedge on the top floor ("across the top floor to the
     stairwell door end" oscillates between (12.23,47.89,14.06) and (12.98,47.89,12.20), the
     sidestep ladder's two spots undoing each other, target d 15.3-16.0, segment 0.036, health
     100, resupply rounds 0-3 all re-wedge [L]) — **ledgered into ignores** with the 03-law
     re-time lever. Binary green 2/0/2.
   - 12: the Operative walk wedges at "the floor south 2" (three spots across resupply 0-3,
     segment 0.328, cel 50) and "the south corridor east" (NOT grounded, cel 339), health 100 —
     **ledgered** with the re-time lever. Binary green 3/0/1.
   - 14: the tripbomb route freezes ON the tilted sill row ("the sill row up" d 0.82, segment
     None — no motion at all — floor ny 0.35 slope reads; "under-slab west" crouched, ny 0.32),
     health 100 — **ledgered** (a waypoint past the sill's lip, or the walker's step-up gate
     reading the scoped slope). Binary green 6/0/4.
   - 0b: the d2/d3 real-damage sisters die 21 hits for 100.0 with **0 return-fire fights** —
     this is the roof-sprint lineage's own red (branch `s/level-0b-roof-sprint-4` ended with
     "the sisters stay red, the base state"); left red with the lineage, not mask-attributed.
   - 09: `the_alarm_switch_fails_objective_1` — the nav to the ch-124 switch (18.74,−368.05,
     43.17) ping-pongs WEST 18 segments (x −20.6→−27.3), crosses the level-exit volume around
     nav 17, and every hop afterwards returns "level_exit reached at +0 ticks"; guard 4's racer
     state (ever_seen true, alarm_run false) rides the timeline. Left red — a nav-route class
     (the guard-door-park-2 NodesForPosition cel-adjacency fix 0x1586a8:103-131 is the natural
     first check), not mask-attributed this session.
   - 13: the PLAIN `walk_the_warm_chain_to_the_front_entrance_and_out` fails the out-doorway
     probe (the leaf holds on Bond; both real-damage sisters PASS the same probe) — left red,
     ambiguous provenance (the merged master's solid bodies are a suspect).
   - 03/05: the prior session's ledger-ignores stand untouched (not re-litigated this session).

3. **The ai/turret/glass battery, one `-p nf-view --test <name>` at a time** (the cross-package
   name-clash workaround the prior report asked for): ai_traces_live **4/0**, turret_hits_live
   **3/0**, gun_turrets_live **3/0**, glass_breach_live **1/0**, glass_pass_every_level_live
   **3/0/3**, glass_dive_07000001_live **1/0**, dynobj_glass_live **2/0**, rocket_glass08_live
   **2/0/1** — all green. Two exceptions:
   - guard_rounds_glass_live **0/1**: "guard rounds stopped at the glass on [\"07000003\"]" —
     a real red, provenance unknown (never run in the mask sessions); left red for attribution.
   - glass_guards_14_live cannot run in this sandbox (wgpu `request_adapter`/`expect("device")`
     — no GPU); not a code red.

4. **The fast suite** (`tools/test/suite.py`): background starts are killed between tool calls
   in this sandbox — re-verified this session (the process vanished between two calls with an
   empty suite.out). Not run; the per-binary totals above are the final tree's state. A final
   full-suite pass belongs to the next session's first minutes (or a `--only`-chunked run).

## What's left

1. The 08 Operative ledger's three levers (the terminal walk-out first), then the Agent/00agent
   sisters' descent re-tune.
2. The 11/12/14 re-time levers written in their ignore messages (mid-floor/waypoint + pinned
   fight sites, each gated to its level).
3. 09's alarm-switch nav route (check NodesForPosition's cel search for the switch's route) and
   13's plain-walk doorway probe (door-swing vs the merged solid bodies) — attribution first.
4. guard_rounds_glass's 07000003 glass stop — attribute (mask change vs master's merge), then
   fix or ledger.
5. The 03/05 ignores' own levers (the 03 moat re-time, the 05 pre-engage) from the prior
   session's report.
6. The fast suite on the final tree, chunked if the background keeps dying.
7. glass_guards_14_live needs a GPU sandbox run before its next merge.

## Verification method

Every run above: `cargo test --offline --release -j2 -p nf-view --test <binary> --
--test-threads=1` with `NF_ISO` on the slim on-foot ISO; diagnoses used `--nocapture` captures
kept under /tmp (not committed). The 08 settle is gated to the real-damage walks
(`!w.tops_up()`), cites its [L] evidence in code, moves no damage/health/RNG constants, and the
fixed leg is walked on real input from the level start (no teleports). nf-game/nf-formats/
nf-audio src were not touched by this branch (test files + docs only), so their batteries are
master's own; nf-game --lib was green on the prior tree and the merge added no src changes from
this branch.

## Live checks for the main agent

1. On 07000008, at the bottom terminal (111.08,13.5,−179.72): which hurt volume or live wire
   ticks −1.2 every ~20-25 frames at the ride's end (fieldprobe on the ride-end spot) — that
   pins whether the Operative walk's zaps are the terminal's wire (the walk-out lever's
   justification) or the Hind's blast tails.
2. On 07000003: whether a GUARD's round breaks+passes a pane (the glass_drain path) — the
   guard_rounds_glass red says it stops; gdbwatch on DroneWeap_DoBullet's collide at a pane on
   03 settles mask-change vs master-merge attribution.
3. On 07000009: the nav route the walk follows from the lobby to the ch-124 switch — whether
   the game's own nav would route east (the ping-pong west is a port nav bug, not the game's).

## Usage

not visible to me — this session's driver shows no token, cost or context-window figures to the
worker (the IM gateway metadata carries only session/chat ids; no usage counters appeared in
any tool result).
