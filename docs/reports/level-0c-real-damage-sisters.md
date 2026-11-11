# level-0c-real-damage-sisters — report

Branch `s/level-0c-real-damage-sisters` = origin/s/guard1-rounds-0c-repin 3d78a7a54
(origin/master 784ba2d99 was already an ancestor at branch time; `origin/s/real-health-agent-0c-0d-3`
does not exist — the base branch `origin/s/real-health-agent-0c-0d` 51811c809 holds the wired+ignored
Agent sisters and their lever ledger, and my branch already carries its wiring through the guard1
lineage). Task: the two ignored `walkreal_0700000c_live` sisters — the Operative real-damage walk and
the Agent walk — taken green with the game's guard sight, each ignore message's levers in order.

**Result: BOTH SISTERS GREEN AND UN-IGNORED.** `walkreal_0700000c_live` whole: **4 passed, 0 failed,
0 ignored** (the base top-up walk, the Operative real-damage walk, the Agent walk, ladder495).

- Operative real-damage: objective 3 complete on the 0d arrival — the whole mission walked on real
  damage (0c legs 18714 ticks + 0d 838) [L the run].
- Agent (difficulty 2, the menu-way load, asserted at both loads): objective 3 complete on the 0d
  arrival — the whole mission walked on real damage (0c legs 20462 ticks + 0d 838; the 0d warm carry
  took 81 hp into TR-2) [L the run]. 13 return-fire fights; every ledgered killer answered by a stand
  the route itself reaches. No health, armour or ammo writes anywhere; no top-ups; damage and health
  never lowered.

## What changed (the git diff is exactly two files: `common/walkreal0c.rs` + the 0c test file)

### The Operative sister — the los-mask-repins-0c-2 lever (the real-damage-scoped mid-arc stop)

The measured run on this tree: the whole crossing held to the band's south step-off (9.49,12.87,−220.21)
[L the run], but "the arc west end" overran its arrival circle off the arc's east edge and fell to the
yard (arrived (14.18,2.01,−218.45) [L the run]; the plain run's green arrival rides the west side at
(11.81,−218.53) [L the green run]) and "the ring south-west arc" wedged 6000 ticks on the yard floor.

The fix (walkreal0c.rs, gated `!w.tops_up()` — the real-damage tree only; every top-up walk byte-exact):
the **mid-arc stop** (11.0,12.4,−219.3) r 0.7 before "the arc west end" — mid-band floor on the run's own
[0c-arc] rows (z −219: x 10-14 hold 11.2-11.4 [L the run's map]) — shortens the final run-up so the
circle is met on its west side, the same treatment the mid-band stop gave the strip diagonals. Result
[L the run]: the stop held (11.46,12.82,−218.78) at +87 ticks, the arc west end arrived on the west side
(11.82,12.86,−218.52) at +20, and the mission walked. Placed POST-ride so it cannot re-shift the ride,
per the ledger.

### The Agent sister — the real-health-agent-0c-0d ledger's levers, in order, each on its own run

1. **The d2-gated stair-head down-shot stand** (after "the lane south to the stair head", before the
   stair approach): from the stair head (13.2,12.4,−124) the hall floor's own opening (x 9..12.4 for
   z −115.5..−131.5 [D probe rays]) gives the down line at the display-ground residents. The first
   stand killed #24 (the traced impact `DRONE #24 part 35` [L the NF_RF_TRACE run]) and the persistent
   sweep (6 bounded scan rounds — a patroller steps into the line) cleared the ground before the
   stair-approach fights could pay their blocked-up-ray cost (the pre-lever fights paid 23.4 + 17.5
   there [L the pre-lever run]).
2. **The d2-gated display-pocket stand** (the photograph leg's own spot, the display eye): the first
   stand killed #25 (impact part 4 [L the traced run]); the sweep then found "nobody left below" —
   the below-floor shooter class the ledger named (#31) died to the fights at 6.3 instead of 16.8.
3. **The NF_RF_TRACE read of the blocked fights** (the ledger's third lever) named the two remaining
   killers, both answered with the same player laws the route already uses:
   - the deck snipers #19/#20 (28.0 + 12.1 at the deck step-down; the fights on them `killed false`
     from the deck legs) — the **d2-gated deck-step sweep** (`yard_sniper_stands "deck-step"`, the
     at-the-door sweep's own 64-u range law, taken before "the deck south");
   - the lane pick's UNSEEN-FALLBACK CREEP on the west sniper #13 at the patrol ring's corner: the
     fight's creep walked Bond 7.7 u off the choreography ((10.5,−204.6) → (3.1,−208.2) [L the traced
     run]) and the aftermath fell off the walkway. Answered with the **west-half return-fire
     suppression** (the strip crossing's own STRIP_NO_FIGHTS law, armed early at the corner bend; the
     existing west-edge and ring-corner sweeps answer the west snipers from the stable deck — the
     Operative run's own fight killed #13 from the west edge at 0 cost [L the run]) plus the **corner
     bend routed east of the #1 corpse**: the [0c-ring] map (added to the d2 gate) shows the deck bends
     east at the corner (z −197..−200 holds x 13-14 [L the run's own map]) while the #1 corpse lay
     mid-band at (12.61,12.09,−196.03); the straight corner line's slide direction decided the run (the
     Operative's west slide recovered, the Agent's east slide fell to the yard (15.06,1.03,−202.45)
     [L the run]). Guards and corpses are solid [M 0a9d39435]; the two bend legs
     ((13.4,−196.4) r 0.6 → (13.4,−199.0) r 0.7) pass the body on its east side along the mapped shelf.
4. The Agent test now carries the Operative sister's own crossing opt-ins (`with_ride_release_past_a` +
   `with_strip_p2k_shot`) — it calls `_at(2)` directly, so the base test's wrappers did not apply; the
   past-A release lands ON the walkable spine and the strip crossing walks the no-fight scope with the
   P2K landing shot, exactly the Operative baseline's machinery.

**Gating (the task's helper rule):** every new stand/sweep/leg is gated to `difficulty >= 2` inside
walkreal0c.rs (a 0700000c-only module — its five consumers are the 0c-family binaries listed below),
and the mid-arc stop to `!tops_up()`. The Operative sister's own chain re-ran green unchanged through
every lever round; every d1 walk and every top-up walk keeps its byte-exact chain by construction.

## What ran (foreground — the fast runner cannot live in this sandbox)

`suite.py` was started in the background per the task (setsid, cargo on PATH, the alsa pkg-config fixed
into a user prefix) and died between tool calls with the whole background process group — the same wall
the guard1-rounds-0c-repin report recorded; a `sleep`-loop control experiment confirmed nothing
backgrounded survives the tool-call boundary. The chunks below are the same binaries on the same tree:

| binary / chunk | result |
|---|---|
| walkreal_0700000c_live (whole) | **4 passed, 0 failed, 0 ignored** (59.3 s) |
| walkreal_0700000c_eastroute_live | 1 passed, 0 failed (14.6 s) |
| walkreal_0700000c_barrelrow_live | 1 passed, 0 failed (8.3 s) |
| interactables_0700000c_live | 7 passed, 0 failed (14.0 s) |
| guard1_rounds_0700000c_live | 2 passed, 0 failed (23.8 s) |
| upward_shot_0700000c_live | 2 passed, 0 failed (16.5 s) |
| comms_chatter_0c_live | 1 passed, 0 failed (14.5 s) |
| walkreal_07000002_live | 4 passed, 0 failed, 1 ignored (95.3 s; master's own ignore) |
| walkreal_0700000d_live | 1 passed, 0 failed, 2 ignored (22.7 s; the other tasks' ignores) |
| walkreal_07000013_live | 2 passed, 1 failed, 1 ignored — see below |
| nf-game lib | 782 passed, 0 failed, 4 ignored |
| nf-view lib | 138 passed, 0 failed |
| nf-formats lib | 92 passed, 0 failed |
| nf-audio lib | 27 passed, 0 failed |
| nf-formats + nf-audio integration | all ok except the 3 `disc_golden` tests the slim ISO allows |

The five walkreal0c consumers (the blast radius of my diff) all ran whole and green. The remaining
walkreal_*_live level binaries (01, 03-07, 08-16, 1b, 41, 4a and the probe set) did not fit the window;
my diff cannot reach them (walkreal0c.rs is included only by the 0c-family files — the consumer grep
and the two-file diff are the evidence), so their chains are byte-identical by construction.

**walkreal_07000013_live's top-up walk failure is PRE-EXISTING on this branch's lineage, not this
task's:** `walk_the_warm_chain_to_the_front_entrance_and_out` fails at its own door-window probe
("13: out through the doorway while the door's open window holds", walkreal_07000013_live.rs:718)
while its `_real_damage_agent` and `_real_damage_operative` siblings pass. My two commits touch only
walkreal0c.rs and the 0c test file (the diff vs the branch base and vs origin/master isolates them);
the suspect set is the branch base's own lineage deltas vs master (drone.rs +147, walker.rs +111,
walker_b2.rs +36, damage_log.rs +68 — the pretrans-arms-5/0c-11 aim-fix/root-zero work, already on the
base when I cut it). Master's tip claims the Chain Reaction walks green (0ee8478fd); a master-vs-branch
bisect of the 13 top-up walk is the merge owner's cheap follow-up.

## What's left

- Nothing on the task's own bar: both sisters are green, un-ignored, and pushed with their ledgers in
  the test doc comments.
- The full four-crate suite total line did not fit the window (the background wall above); the chunks
  cover every binary my diff can reach, plus nf-game/nf-formats/nf-audio/nf-view libs.
- The 13 top-up walk pre-existing failure (above) is the one red the merge owner will see outside this
  task's files.

## Live checks for the main agent

1. 0700000c, Agent difficulty: stand at the stair head (≈13.2, 12.4, −124) and take the aimed down-shot
   at a display-ground guard (≈(8.1,7.3,−114.7)) through the hall floor's opening — expected: the round
   lands (the port's traced impact did: DRONE #24 part 35 at (9.32,7.95,−114.30) from bond
   (13.18,12.36,−123.08) [L the traced run]); confirm the game's own bullet class behaves the same
   (gdbwatch on the guard's hp at the aim-bit tap).
2. 0700000c, Agent difficulty: at the patrol ring's north-west corner after guard #1 dies on the ring
   diagonal, confirm the game's own corpse capsule blocks the straight corner line the way the port's
   does (the port measured the corpse mid-band at (12.61,12.09,−196.03) [L the run's dump]) — i.e.
   whether the game's player also slides off the east edge on the straight line, or steps around.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the session
(no usage panel is exposed to the agent).
