# level-15-mines-shot-from-range — shoot the laser mines from a distance on real input

Branch from origin/master.

On 07000015 the floating laser mines (the nine 0xFE placements, Mine_Update 0x17c930) should explode when shot from range; merges level-15-mines and mines-tripwires changed this, but no test reaches a mine through real input from a distance.

Do:
1. Walk a real route (no teleport after load, no ammo/health writes) to a spot with a clear line to a mine at range (>8 units), aim with the level's own weapons (what the level gives Bond [D loadout/pickups]), and fire.
2. The mine must take the hit through the game's bullet path [M the bullet-vs-object pick] and explode (its beam goes, the blast plays, the mine is gone). If it does not, find why and fix it, citing the decompile.
3. Add the live test; run interactables_07000015_live, walkreal_07000015_live, mine_bob_live, then the full suite.

---
claimed-by: sandbox3 at 2026-11-02T09:48Z

---
## Report (done, 2026-11-02T13:32Z)


# level-15-mines-shot-from-range — report

Branch `s/level-15-mines-shot-from-range` (from `origin/master` c628a2fa, one commit of test code + this report).

## What was done

The task asked for the shot the merges never pinned: on 07000015 a floating laser mine (the 0xFE
placements, `Mine_Update` 0x17c930) must explode when a bullet reaches it from range (>8 units) on
real input — no teleport after load, no ammo/health writes.

Research first: the mine's bullet contact is already wired in the sim. The flight segment picks the
mine's mesh via `Mine::segment_t` [M drone.rs:14652-14663], `Collide_Update` 0x1ea398:160-233
mirrors the hit onto the mine's list with +0x58 = the bullet, and `Mine_Update` 0x17c930 explodes on
that referenced entry with cause `MINE_BULLET`. The gap was test-side: existing legs shot at
1.6-5.9 u only. **No sim code needed to change** — the fix is a new test.

New live test `a_p2k_round_from_10_units_explodes_the_door38_fence` in
`crates/nf-view/tests/walkreal_07000015_live.rs` (+77 lines, the branch's only code change):

- Real walk on real input: spawn gantry → nav-graph silo crossing → the spine. No teleports, no
  health writes, no ammo top-ups; the guards are held (the file's dev freeze) so the only damage in
  the run is the mines' own blasts.
- The level's own weapon: the P2K (0x02) from the 07000015 loadout [D].
- The target: the doorway fence #383, the floor-to-ceiling beam at x 35.6, z 17.47 [D], ~10 u south
  of the shot spot. door[38] (z 10.5) sits on the line, so the test opens the leaf from its E reach
  (head→pos d² < 6.25 [M Player_Activate 0x1a5170]) and fires inside the door's 360-tick auto-close
  window [M door.rs 0x13660c-0x136700].
- The assert: `d > 8.0` at the shot (it lands at 10.38 u), then `#383.exploded` with
  `cause == MINE_BULLET` [M 0x1ea398:160-233 → 0x17c930; gone bit 0x17cb7c-0x17cb88].
- Blast proof: neighbours #384/#385 explode with `cause == MINE_BLAST`, which can only come from the
  blast's `Explode_Propagate` case-0x4B entries [M 0x17c518; 0x17cde0] — there is no other writer of
  MINE_BLAST.
- On the way in, the silo-crossing beam #462 is shot from the lane as route clearing and dies at
  13.58 u — itself a >8 u bullet kill.

## How it was verified

- `walkreal_07000015_live` 4/4 (including the new range-shot test) — GREEN.
- `interactables_07000015_live` 17/17 — GREEN.
- `mine_bob_live` 3/3 — GREEN.
- `nf-game` 125 tests — GREEN.
- `nf-formats` — GREEN except exactly the 3 allowed `disc_golden` failures on the slim ISO
  (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`).
- `nf-audio` — GREEN.
- `nf-view` partial suite (sandbox window): 263 of 927 test binaries run, 260 green, including all
  24 priority suites matching mine/worldobj/explode/blast/bullet/07000015/gunimp — 24/24 green.

## What's left

- The rest of the nf-view suite: ~661 binaries not reached (alphabetically from `fix_pickups_trace_live`
  onward) — the sandbox disk and window could not fit the whole build+run. The change is test-only,
  and nothing outside the touched test file could be affected, but the next session can resume the
  sweep for completeness.
- Three nf-view failures, all environment- or timing-bound, none related to the change:
  - `fix_0c_pylon_live`: panics at `request_adapter().expect("adapter")` — no GPU in the sandbox;
    needs a rendered check on the main agent's machine.
  - `campaign_flow_live` and `enemy_accuracy_live`: exceed the 150 s per-binary cap (slow front-end
    play / long sim), not assertion failures.

## Live checks for the main agent

Not needed for the merge (no visible change; the explosion visuals are already proven by the
existing suites). One optional confirmation on the real game, if convenient:

- 07000015, real play: walk to about (34.6, 41.4, 7.5) on the spine, open door[38], step back south,
  shoot the doorway fence mine at (35.6, 42.3, 17.47) from >8 u. Expected: the mine explodes from
  the gunshot (beam gone, blast plays), and its two neighbours explode in the blast chain.

## Usage

not visible to me

---
## ntfy failed

1. curl: (28) Connection timed out after 20001 milliseconds
2. python urllib: URLError <urlopen error [Errno 101] Network is unreachable>
3. wget: no output, killed after 30 s (connection never established)
