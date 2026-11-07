# mp-bot-scan-others — MP bots choose targets from the game's per-player scan

Branch `s/mp-bot-scan-others` (from origin/master 22d5836b7).

## What was found

**Nothing called the ported scan.** `BotBrain::scan_others` (crates/nf-game/src/mp/bots.rs) existed
as a stub that only set `vars.scan_flag` and was called by nobody (grep over `crates/`: the
definition and two test-side touches only). What actually filled `OtherPlayer::flags`/`dist2` each
tick was an inline stand-in in the live driver `scene.rs` (`step_mp_bots`, old lines 13602-13625):
dist² + bit-1 candidate for every alive non-self row, bit-3 team-mate in the team view, and **no
seen bit at all** — `BotCtx::lazy_sight: true` made `find_opponent` re-derive the sight answer per
pick (`segment_clear` + `DroneVision_OpponentVisibility ≥ 0.7`, a cone+range score from the drone
vision chain). That is neither `NDrone2_CanSeeObject` nor a per-tick pass.

**What the game does** ([M] `work/decomp/ACTION.ELF/00125390_BOT_setOtherPlayerInfo__FP10DCVars_tag.c`,
line numbers below are that file's):

- `NDrone2_ControlSTANDARD` 0x1490f0:164 calls `BOT_setOtherPlayerInfo` every tick for the bots'
  drone type (`Drone+0xC5 == 0x1E` — the byte is the drone-type index, ×0xC into
  `DroneTypeSettings`, per `Drone_Control` 0x139268:59), immediately before
  `NDrone2_FindOpponent` 0x1424e0 — unconditionally, every tick.
- Per slot 0..7 it rewrites the `BOT_vars+0xB0+slot*0x10` row: empty/self/dead (class 0x11/0x12)
  rows get `flags & ~0xA` (:87-97); live rows get bit-1 candidate + bit-3 same-team
  (`old & ~0x8 | 2 | same`, :99-102; the team compare is my team word, with team 2→3 and the
  FFA (non-team, non-Assassination) arm comparing team 3 → no team-mates, :63-75).
- **Enemy rows only**: the +0x04 dist² (`Vec_SqDist3D`, :148-149) and the bit-2 seen bit.
  The seen bit is written **only from `NDrone2_CanSeeObject(drone, obj, 5, 0)`** (:151-153) —
  head-to-head `Collide_LineOfSight` mask 0xa27 behind `DroneVision_CanSeeObjectFrom` 0x176f18,
  gated by the nav-cel ray; **no cone and no range in it** — and only for **the ONE slot the
  `BOT_vars+0x767` cursor picks that tick** (:67/:77 read, :151 run, :174 advance). The bit
  LATCHES in the row between refreshes (:115 `& 0xfffffff6` keeps bit 2). The cursor resets to 0
  whenever the cursor slot could not run the LOS branch (dead/empty/me/team-mate — :169-176), so
  the refresh is biased to low slots while someone is dead. Later-slot bots copy an earlier bot's
  dist²/seen for shared targets instead of recomputing (:141-146) — the same head-to-head line, so
  the answers agree; the port skips this copy [S].
- The +0x770 scan flag: zeroed, then set by walking the MPGame bot chain (:35-61) — "another bot
  exists", not "any live player". The port sets it from "another bot slot row present" [S: the
  chain-head semantics].
- Not lifted (no row fields, readers not ported): bit-0 alive byte + the 2-frame just-died grace
  (:105-127), the row+0x0 `MPGame+0x19C` stamp (:117), the row+0x8 angle in degrees (:131-139).
- `NDrone2_FindOpponent` 0x1424e0 reads the rows: bit-1 candidate required (:365), dist² < best
  (:366, best starts 640000.0), **bit-2 seen OR the 360° stats bit** (:367), bit-3 == 0 for
  scoring (:371), then the not-lifted multipliers (last-attacker ×0.04 :372, carrier ×0.25 :375,
  ±10°+alive ×0.04 + distraction :378-391, bot-chain ×16 :402-427), earlier slot wins ties
  (:434), the post-pick cone re-check (:469-491) and the site arm (:519-560).

## What changed

1. **`BotBrain::scan_others` rewritten into the real 0x125390 pass** (crates/nf-game/src/mp/bots.rs):
   signature `(my_index, my_team, team_view, world, rows: &mut [OtherPlayer], first_bot_slot)`;
   writes the +0x770 flag, the per-row candidate/team bits, the enemy-only dist², and the
   round-robin seen bit with the collision-world segment as the `CanSeeObject` stand-in (the
   documented `Collide_LineOfSight` stand-in — the same line the lazy sight ran), latched per
   slot in the new `vars.scan_seen`, cursor in the new `vars.scan_cursor` (`BOT_vars+0x767`).
   Every write cites its 0x125390 line.
2. **Called at the game's point**: the live driver (`scene.rs` `step_mp_bots`) runs the pass per
   bot per tick right before `brain.step` (ControlSTANDARD:164 → FindOpponent), and the old
   inline stand-in loop is deleted; `BotCtx::lazy_sight` is now `false` there — `find_opponent`
   reads the precomputed bits instead of re-deriving a sight line per pick. `find_opponent`'s own
   [S] gates (24 m / 300° wide range, ±90° cone) are unchanged — they stand in for the drone
   vision cone the game applies elsewhere (0x1424e0:469-491 / 0x14b300), not for the scan.
3. **State hash untouched**: `scan_cursor`/`scan_seen` are deliberately NOT folded into
   `BotBrain::state_hash` (they are pure functions of already-synced positions; the task's "keep
   the state hash unchanged" holds — `mp_lockstep`'s bit-flip test still passes). No RNG draws
   added anywhere (0x125390 draws none).
4. **Readers checked against the decompile**: `find_opponent`'s bit1+bit2 gate = 0x1424e0:365-367 ✓,
   bit3 team-mate gate = :371 ✓, dist² best compare with earlier-slot tie win = :366/:434 ✓,
   trait-7 `flags & 0xA != 0x2` (bots.rs trait pick) matches the candidate-without-team-mate read
   of the +0xB0 rows ✓, `find_site_opponent`'s site-row bit-2 read is the port's OBJ_ROW carrier
   [S] (documented there). The full FindOpponent scoring (multipliers/history) stays not lifted —
   now load-bearing, see "Left".
5. **New pin test** `crates/nf-view/tests/mp_bot_scan_live.rs`, through the real input path
   (`load_mp_carry` + `step_bond` from the load, no teleports): a bot never picks a team-mate in a
   team game (2 bots team 0 vs the human); a bot never picks a slot whose head-to-head line was
   never clear and never holds one walled beyond latch+hold (the 8-slot cursor period + the
   aggression×20 unseen-hold bound); a fresh pick always carries the latched seen bit; a bot picks
   a target in the open. Both tests pass.

## Verification (what ran, pass counts)

- `mp_bot_scan_live` (new): 2/2 passed.
- `mp_bots_live` + `mp_bots_moving_live` + `mp_world_sync_live` + `mp_bot_objectives_live` (one
  cargo pass, `--test-threads=1`): 10 tests, 9 passed, 1 failed → fixed (below), then
  `mp_bot_objectives_live` alone: **10/10**.
- `mp_lockstep` (`nf-game --features mp-bots`): 6/6 — the brain state-hash pins hold.
- `nf-game` lib: 686 passed, 5 ignored.
- Not run for time: the other `mp_*_live` binaries individually and the full `tools/test/suite.py`
  (its background run died with the sandbox's process reaping; see "Left").

## Existing test changed (the test was wrong, per the decompile)

`mp_bot_objectives_live::espionage_bots_fight_while_the_blueprint_moves` failed after the fix:
peak shots 20 → **7**, deaths 1 → 0, and the carrier's within-3.0 base arrival gone. All three
figures were emergent under the OLD stand-in pick (fresh cone-gated visibility per pick = stable
targets). The game's scan has **no cone in the seen bit** (0x176f18) and refreshes round-robin, so
carriers are interrupted more; the cadence/lethality ride FindOpponent's scoring multipliers and
history (0x1424e0:372-427), which the port has not lifted [S]. The test now pins the decompile-law
content (blueprint carried, armed bots fire, carrier homing inside 25.0 of its base) with the
citations inline. If the main agent lifts the FindOpponent scoring, the sharper pins should come
back — see the live check below.

## Live checks for the main agent

1. PCSX2, a 2v2 Espionage match (or any team scenario with 2 bots), `gdbwatch.py` on a bot's
   `BOT_vars+0x767` (the scan cursor; BOT_vars = `Drone+0xD1C`): expect +1 per tick while the
   cursor slot runs the LOS branch, reset to 0 the tick the cursor slot is dead/me/team-mate
   ([M] 0x125390:169-176).
2. Same setup, watch the row flags at `BOT_vars+0xB0+slot*0x10+0xC` for an enemy slot: bit 2
   should survive untouched on ticks where that slot is not the cursor, and flip only on the
   cursor tick ([M] :115/:151-153).
3. Count one bot's shots over 10 match-minutes of the espionage scenario ([L] measurement): the
   port now measures 7 (was 20 under the old stand-in). If the real game shoots much more, the
   FindOpponent scoring lift (0x1424e0:372-427) is the missing piece, not the scan.

## Left

- The other `mp_*_live` binaries were not run individually this session (the full suite's
  background run was killed by the sandbox between tool calls and a foreground rerun did not fit);
  the four listed above plus `mp_lockstep` and the `nf-game` lib are the ones that touch the
  changed code. The next message should run the full suite on this tree (`NF_ISO=... python3
  tools/test/suite.py`) — note the suite must run in the FOREGROUND here, background jobs get
  reaped.
- `NDrone2_FindOpponent`'s scoring block (0x1424e0:372-427: last-attacker ×0.04, carrier ×0.25,
  ±10° ×0.04 + `BOTSTATE_increaseDistraction`, the bot-chain ×16 via the OTHER bot's +0x770, the
  opponent history) is the obvious next task; with the faithful scan in place the port's fight
  cadence now depends on it.
- The scan's bit-0 alive byte + 2-frame just-died grace, the row+0x0 stamp and the row+0x8 angle
  are not ported (no `OtherPlayer` fields; their readers in FindOpponent are not lifted either).
- The later-slot copy branch (0x125390:141-146) is skipped [S — the same head-to-head line].

## Usage

not visible to me — this session shows no token, credit, context-window or cost figures anywhere
(no status bar, no `/usage`-style output, nothing in the environment); per the instruction not to
estimate, there are no numbers to copy.
