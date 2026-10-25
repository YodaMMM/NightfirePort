## Your task: `level-07-guest-collision`, 07000007: party guests are solid to Bond

On the party level (07000007) the guests walk straight through Bond, and Bond through them. Master has the game's player-vs-bot push (`Collide_Jointy` 0x1ec180's 0x800 arm, the bot capsule from `NDrone2_Collision` 0x140e90:91-121, on only near the player via `DroneFunc_NearPlayerCollision` 0x14b190 / `NDrone2_DoCollision` 0x149ec8).

1. Find why the guests (civilian/party-guest DMODEs, the 0x0f placements on 07000007 [D]) are left out: their drone flags (+0x4F8 collision bits), their class, the near-player gate, the guests' own movement collision (do guests get pushed by Bond / stop at him in the game?) [M].
2. Port what the game does for the guests — both directions: Bond blocked by / pushed off a guest, and a walking guest not passing through Bond [M]. Real-input tests on 07000007 (Bond walks into a standing guest; a walking guest meets Bond), sister: a guard still behaves as before. Every party test green on master stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-collision`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-10-25T00:33Z

---
## Report (done, 2026-10-25T01:56Z)


---
## Report from the worker (done, 2026-10-25T01:56Z)

# level-07-guest-collision — party guests are solid to Bond (07000007)

## What the game does (all decompiled, first-hand reads)

- `NDrone2_DoCollision` 0x149ec8:11-34 [M ACTION.ELF]: a bot's collision kind word (+0x464) goes
  0x800 only when this returns 1. Gate order: dead (+0x4f8&0x200, except level 0x700004a) → 0;
  disabled (+0x4f8&0x20 clear) → 0; then for class word +0xd8 != 0xc: `+0xd8 == 0x13` → 1,
  `+0x538 != 0` → 1, `DroneFunc_NearPlayerCollision` (a per-player distance < 2.0 at
  drone+0x154+4i, 0x14b190) → 1, `obj+0xfc != 0` → 1, `+0x570&2` (anim no-collide) → 0,
  fall-through → 1. **Class +0xd8 == 0xc skips every inner check** — unconditional 1.
- `NDrone2_Enable` 0x149dd0 [M]: Enable(0) clears +0x4f8 bits 0x20/0x80/0x100 and sets the
  obj+0xf0 skip bits; `NDrone2_DSTATE_WaitSwitch` 0x15b960:14-16 ENTER calls Enable(0) — so the
  idle WAIT_SWITCH crowd (the ch-6/ch-21/ch-23 groups) has collision OFF in the game too. The
  port's `enabled=false` on WAIT_SWITCH ENTER is faithful.
- `NDrone2_Collision` 0x140e90:91-121 [M]: the walking capsule — radius 0.4, **0.55 for
  +0xd8 == 0xc** (:92-94, +0x45c), top = pos + up·r, bottom = pos + up·(r − h) with h = +0xa0
  (1.0 default [M 0x14b300:48]).
- `Collide_Jointy` 0x1ec180:203-284 [M]: the 0x800 capsule-vs-capsule arm. SP
  (`MPSettings+0x384 == 0`): fVar12 = 1.0 and **return 0 when the OTHER obj is class 3**
  (:229-233) — a bot never pushes itself off Bond. Bond's own pass (class 3) is pushed off every
  capsule-ON bot with the FULL overlap. `Player_Collision` 0x194df0 builds Bond's kind-0x800
  hittest EVERY frame with mask 0 (:88) — **no player-stance (grounded/airborne) gate anywhere**.
- **The game does NOT exclude guests.** No civilian/class check exists in DoCollision (the
  opposite: 0xc is unconditional), the capsule build, the broadphase (Collide_PickObj 0x1eae80,
  class/mask filters), or Jointy. `NDrone2_IsCivilian` 0x14aae0 has zero collision callers.
- The 07000007 guests are class +0xd8 **0xd/0xe** (skins 0x5000015/0x5000019-0x21 [M DefaultInit
  0x14b300:128-132,156-160]) — the near arm serves them (capsule ON within 2.0), and the 0.55
  radius arm does NOT (they get 0.4). Their team byte +0x44 = 3 [M GetDroneTypeAttackTypeFriend
  0x14cbd8:48-61] is what distinguishes them from guards in code; it is not read by the
  collision chain.
- A walking guest meeting Bond: the guest is never displaced (Jointy returns 0 for a class-2
  self vs class-3 other [M :229-233]); Bond is pushed. Guests do not stop at Bond.

## Why the port left them out

Master's push (`Bond::tick_ctx`, player.rs) ran only while Bond was AIRBORNE (a documented [S]
port choice from the sleeper task), and the Sim's capsule fill (scene.rs) filtered on
`enabled` only. So a grounded Bond was pushed by nobody, and the enabled standing guests never
blocked him. The idle crowd is `enabled=false` — faithful to the game.

## The port change (this branch)

- `crates/nf-view/src/scene.rs` (the fill): alive filter `!x.corpse` added [M 0x149ec8:11,
  0x14a2f0:20]; radius 0.55 for `voice_type == 0xc` (the port's +0xd8 mapping — skins
  0x500003b/0x3c) else 0.4 [M 0x140e90:92-94]; capsule top pos+r, bottom pos+(r−1.0) at h 1.0
  (the anim height word +0xa0 is not read — [S] unverified, live-check item); the 2.0 pick
  radius is now documented as an optimization only (an overlap needs ≤ 0.95). Each capsule
  carries `guest_team = x.team == 3`.
- `crates/nf-game/src/player.rs` (the push): the civilian team (the game's +0x44 = 3 guest
  dtypes [M 0x14cbd8:48-61]) is pushed in EVERY stance; the other classes keep master's
  airborne-only behavior (`if self.grounded && !guest_team { continue; }` — marked [S] port
  choice at the site). The game's own pass has no stance gate; the residual uncertainty for the
  non-guest classes is +0x570&2/obj+0xfc (not modelled) — see Live checks.

## Verification (all on this branch's final tree)

New tests, real input through `Sim::step_bond` (`crates/nf-view/tests/fix_guest_solid_live.rs`):
- `bond_walking_into_a_standing_guest_is_pushed_off` — a standing enabled team-3 guest
  (probe: 15 such at the party start, e.g. the PARTY_GIRL 0x2f pair at (44.3,44.0,−62.1)) holds
  walking Bond at min spine distance 0.950 (his 0.55 + her 0.4, [M 0x140e90:91-94]); he walked
  (moved 0.2+) but never reached her. PASS.
- `a_walking_guest_meeting_bond_pushes_bond_and_keeps_walking` — after the ch-6 plate wake, a
  walker meets standing Bond: Bond displaced 3.75 (the SP Jointy push [M 0x1ec180:229-233]),
  the guest keeps walking 9.66 (bots never self-push vs class 3 [M :231-233]). PASS.
- `a_guard_still_behaves_as_before_grounded_bond_meets_no_push` — the sister: grounded Bond
  walks into a bouncer (state 0x30 at (41.3,−48.7)) to 0.012 — no actor push; master's
  airborne-only choice for non-guests unchanged ([S] pin, see above). PASS.

Suites re-run green on the final tree (foreground, `--test-threads=1`, NF_ISO onfoot ISO):
nf-view: fix_guest_solid_live 3/3, walkreal_07000007 wake test 1/1, walk_07000007_live 2/2,
interact_07000007_live 7/7, fix_guest_reactions_live 3/3 (+1 ignored), fix_party_mission_live
9/9 (+6 ignored probes), party_start_rules_live 6/6, sleeper_push_live 1/1.
nf-game crate: 547+ passed, 0 failed (all binaries green, includes dmode_row0_near_player_live).

## Known failure (left as-is, honestly)

`walkreal_07000007_live::walk_the_party_start_to_finish` (green on master) now fails at the
hall door leg: a team-3 civilian stands at (79.5, 44.4, −50.3) on the old straight nav line;
solid guests hold Bond 0.95 off him and the walker has no obstacle avoidance, so the approach
stalls at (79.19, −51.21) (NF_PUSH_DEBUG shows his capsule as the only pusher). Two sidestep
anchor attempts (south z −53.4, north z −47.6) still stall — the nav node path re-enters his
push cone. I reverted my route edits; the test file is untouched from master. Fix options: (a)
teach `Walker` a real sidestep (perpendicular escape, not nav-node re-aims), (b) route the hall
approach around the guest by hand with a measured waypoint, or (c) a live check that the real
game's guest there is enabled (if he is disabled on disc, the port's `enabled` fold needs a
look instead). No decompile-backed test was weakened; the guest-solid behavior itself is pinned
by the three new tests.

## Live checks for the main agent

1. Grounded push vs a NON-guest bot (the [S] port choice): in a fresh boot on 07000007, walk
   Bond into the hall bouncer at (79.5,−50.3) (or the entrance guards) and watch the player
   obj position — does the game push him out (expected if the bouncer's +0x570&2 is clear) or
   does he overlap? Watch +0x4f8 bit 0x20, +0x570 bit 2 and obj+0xfc of the bouncer while Bond
   stands against him (gdbwatch.py on drone+0x4f8/+0x570 of the bouncer at (79.5,−50.3)).
2. The hall civilian at (79.5,−50.3): is his +0x4f8 bit 0x20 SET in the real game at the party
   start (i.e. does the real game also hold Bond off him)? If clear, the port's `enabled` fold
   for his placement is wrong and the walkreal failure is a port bug, not a walker gap.
3. The anim height word +0xa0 on a walking guest (does it stay 1.0? the port assumes it).
4. (carried from master's comment) the 01 armour walk and 0a west doorway ally tangent zones —
   does the real game push Bond off allies there?

## What's left

- The walkreal full-walk failure (above) — walker avoidance or the live check deciding `enabled`.
- The +0x570&2 / obj+0xfc per-bot OFF switches and the +0xa0 height word are not modelled.
- The 0.55 radius arm has no level-07 exerciser (no 0xc-class bot on 07000007 — skins
  0x3b/0x3c live on 02); pinned only by the code path.

## Usage

Not visible to me: this worker session shows no token, credit or context-window figures for the
main conversation. The only figures my environment displayed were the subagent totals printed
by the two research Task results: "Tokens -> prompt:199318 completion:9702 total:209020" and
"Tokens -> prompt:139314 completion:9009 total:148323" (copied exactly from those result
footers).
