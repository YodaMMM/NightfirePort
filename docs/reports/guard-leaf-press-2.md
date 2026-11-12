# guard-leaf-press-2 — the 13-level leaf press without breaking the Double Cross exit

Status: **not done** — the window ended mid-fix. The branch carries the merge, the full
diagnosis, one decompile-backed attempt (reverted), and the evidence trail the next window
needs. Branch `s/guard-leaf-press-2`, head 293a90e1b.

## What's done

- Branch from `origin/s/guard-leaf-press-07` 758a89ddc, merged `origin/master` 08a276ce0 —
  clean, 16 files, 1049 insertions. `origin/s/guard-leaf-press-07` is NOT an ancestor of
  master, so the merged tree is the real candidate. WORKLOG entry `guard-leaf-press-2` added.
- **Reproduced the regression** (rule 4). On the merged tree,
  `fix_mission_live::the_07000001_exit_works_end_to_end` fails NOT at the shoji-OPEN assert but
  at the last one: `level_exit == Some(0x07000001)` (the level's OWN id — a mission-fail
  RESTART) instead of `Some(0x07000002)`. The restart fires mid-ambush: Mayhew is killed
  BEFORE the ch-0x99 release ("the hold releases (Mayhew … state 0x47)"), the escort-death
  script LOAD_LEVELs 07000001, and every later walker leg aborts at "+0 ticks".
- **Killer identified** (temporary on_step probe, reverted before commit): the escort, held at
  (7.02, 12.59) — past the courtyard shoji, in the crossfire lane — is hit at probe-t 20977
  (hp 2.5, guard g10 alive at (6.98, 8.48) st 0x1, 4.1 away on the hall side, firing north
  through the open shoji) and dead at t 21039 (hp −5, DEATH_ANIM 0x44). g10 is a leftover the
  sweep reaches only after the fatal burst.
- **Trigger confirmed by experiment**: with 07's `near_pos_fn` list tier disabled (one-line
  local revert), the SAME test passes (45 s run). The tier itself is decompile-backed
  [M NDrone2_DroneNearPos 0x143e68] and stays; something that coexists with it in the game is
  missing in the port.
- **Geometry evidence** (probe v1/v2, reverted): with the tier OFF (green run) the escort is
  held on the HALL side working the shoji in `DSTATE_OpenDoor` 0x81 at (7.04, 9.9→10.2), takes
  one hit (hp 10), survives, opens, crosses, exits. His hold position with the tier on is
  CORRECT — route node 33 at (2.91, 7.57) vs the game trace's (2.94, 7.22) [L
  tests/traces/01-doors.trace] — but the route then ADVANCES past the shoji mid-fight (idx 15→16,
  node 6 at (6.99, 11.78), probe-t 20153, while the ambush is still running), he enters the
  courtyard, and courtyard-side AIM_STAND_FIRE guards (g14/g17, st 0x5c/0x68/0x73) drain him
  (hp 4.38 at t 24629 → −19.81, DEATH_ANIM at t 33882, before the release).
- **The game's-way fix implemented, then reverted.** [M Drone_CollisionHandler 0x140968:204-208
  + Collide_Intersect 0x1ebaa8:353-373]: the collide phase's DeltaP — obj+0x30 += ext+0x3e0
  whenever the 0x800 capsule's contact list (obj+0xD0) is non-empty; the port has NO
  drone-vs-drone capsule resolution at all, so with the node-reach tier guards can rest inside
  each other / on route nodes where the game's bodies are pushed to 0.8 separation. Commit
  cc3600b24 added `Drones::contact_pushout` (r 0.4 [M NDrone2_Collision 0x140e90:91-93], y band
  2.0 [M 0x140e90:91-121], class-2 fold present/enabled/non-corpse/hp>0, level-gated to
  07000001 per the leaf-press "set grows one level at a time" precedent). It moved the fight
  the RIGHT direction (the escort held at the shoji in 0x81 like the green run) but did NOT
  converge: the exit test still red (death moved later — at the release, state 0x48, the escort
  crossed at t 20153 and courtyard guards drained him). And it broke
  `walkreal_07000001_live`: **5 passed 2 failed WITH it, 7 passed 0 failed WITHOUT it** (same
  binary, `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent` among the
  failures) — the separation fold's detail (full 0.8 both-movers push, tick-end placement,
  order) is not the game's yet. Commit 293a90e1b reverts it with this evidence in the message.

## What's left (next window)

1. Make the pushout match the game's collide order (per-object, during the walk — the port
   applies it tick-end; the game's DeltaP lands inside the object walk) and re-check
   walkreal_07000001_live first, then the exit test.
2. If the pushout alone still doesn't stop the mid-fight shoji crossing, probe WHY the escort's
   route advances idx 15→16 mid-fight under the tier (the door node reach — which peer stood
   within 0.4 of which node when idx 14→15/15→16 fired; the v2 probe's per-advance peer dump is
   the template, in this report's history on the branch).
3. Then: every walkreal_*_live one level per tool call, nf-game lib + npc_doors_live +
   fix_guard_misc_live (lesson 10), the full suite (see below), and the report re-run.

## What ran

- `fix_mission_live` (built + `the_07000001_exit_works_end_to_end` run 5 times: merged red,
  tier-off green, pushout red ×2, plus the probe runs) — per-run lines above.
- `walkreal_07000001_live`: 7/0/1i on the merged tree (pre-pushout), 5/2/1i with the pushout.
- The full suite did NOT run: the sandbox kills background jobs between tool calls (the
  nohup'd first build died silently at 03:07; only foreground chunks survive), and the
  foreground window fit the diagnosis + one fix attempt + the walks above. `tools/test/suite.py`
  exists on the merged tree (`--only` chunks work) — the next window should chunk it.

## Live checks for the main agent

- With `gdbwatch.py` on 07000001, watch Mayhew's route node index (the mission AI path) during
  the ambush: does the real game ever advance past the courtyard-shoji door node while the
  ambush is still running? Expected: no (he holds at the ally-wait node (2.94, 7.22) [L
  01-doors.trace] until ch 0x99).
- `fieldprobe.py`: during the ambush, dump the courtyard shoji's state byte and obj+0xD0
  contact lists of the hall guards nearest (6.98, 8.48) — expect contact entries (DeltaP
  sources) whenever two bodies stand within 0.8.

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).

## ntfy failed

(to be filled by the finish step — see below)
## ntfy failed

(see the commit tail — filled by the finish step)
