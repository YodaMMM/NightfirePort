# level-13-guard-barks — report

Branch `s/level-13-guard-barks` (from origin/master). Task: do 07000013's lobby guards
(skins 0x5000004/06/08) speak? Confirm each guard's voice type [D] and the game's gates [M];
make the port speak exactly when the game does — or prove the game is silent too and pin that.

## The answer

The guards' voice type is **3** for every lobby-guard skin — `NDrone2_DefaultInit` 0x14b300's
skin switch stores 3 into `Drone+0xD8` for exactly 0x05000004/0x05000006/0x05000008
(+ 0x050004c) [M :96-103, :253-254; no disc table seeds it]. The port's `voice_type()` and
`drone_setup` already carried the same rows — nothing to fix there. With that settled, the game
has three bark channels, and the port now matches all three:

1. **Crosshair (`bond_talk`): the game is silent too — pinned.** `NDrone2_BondTalk` 0x1446b0
   returns unless `GameState+12 == 0x7000007` [M :30] and its voice-set switch
   {9, 0xd, 0xe, 0x10, 0x11, 0x12} has **no case 3** [M :40]; it is also only called from the
   civilian-family state UPDATEs (Civilian 0x15d118:47, CivilianMission 0x15d860:36,
   CivilianMissionWait 0x15dbb0:41, PartyGirl 0x160140, CivilianGuard 0x1603a8:39) — never
   IDLE/PATROL/ATTACK. The port mirrors this exactly (`bond_talk`'s `d.level != 0x0700_0007`
   return, the same switch, the same caller states). Live test: face-to-face with an Idle lobby
   guard for 240 ticks → no talk queued, no speech audio.
2. **Idle/patrol (`PatrolTalk` 0x13e988): gated on reaching PATROL.** It is called ONLY from
   PATROL's UPDATE [M DSTATE_Patrol 0x15c498:41-44], speaks `0xb3 + Rand(5)` [M :36-40] =
   `Drone_SfxInfo` entries with **facial = 0** [D ACTION.ELF @0x2812d0 + idx·32 — the unported
   FACIAL arm cannot swallow them], behind the global cooldown, a `Rand(2)` roll and the
   `+0xD8 == 3` gate [M :35]. PATROL is only reachable through `InitPatrol` 0x1724f0's
   props 0x24/0x25 + `AssignAIPath(4)`. From the disc placements + the game's own
   `bitDescs`/`bitMasks` (@0x2c7250/0x2c7308, decoded through `behaviour_util_getProperty`
   0x1c7d98): props 0x24/0x25 are **set on #317/#320/#325/#326/#330/#408/#409** (patrol) and
   **clear on #318/#319/#321/#322/#329** (idle) [D 07000013.bin; prop 0x24 = word 1 bit 5,
   0x25 = word 1 bit 6]. At load only the six group-0 guards exist (p4 is the **spawner group
   id**, not a route — the p4=1/2 rows are the reinforcement templates of spawner groups
   1/2 [D golden census: 6 at start, 2 spawner groups]) — of those, #317/#320 are prop-set,
   #318/#319/#329/#330 prop-less. In the live run the prop-set pair (and the two waiters)
   spot Bond across the lobby and go to combat; the prop-less pair sits IDLE. So at spawn the
   game itself plays **no patrol bark** (no guard is in PATROL while Bond stands in the lobby),
   and the live pin holds: 1500 ticks standing at spawn → zero 0x261..0x266 starts.
3. **Attack entry (`AttackTalk` 0x13eac0): the real fix.** Type 3 speaks `0xac + Rand(7)`
   always [M :66-68] but **returns before the draw while `+0x228 & 8`** [M :63-65] — the bit
   `DroneVision_HaveOpponentSight` 0x175ef0:82-86 sets on a reaction-acked sighting **with
   prop 0x3c** (all 12 lobby guards carry 0x3c [D]). The port's `attack_talk` had **no gate**:
   it drew Rand(7) and queued the bark on every attack entry, sighting or not — an RNG-stream
   divergence and a bark the game never plays after the first sight. Fixed: new field
   `ever_seen_3c` (the game's bit exactly — set at the vision pass's sight-ack site under the
   prop gate; the older `ever_seen` folds without it), `attack_talk(level, rng)` returns
   silently before any draw when `voice_type == 3 && ever_seen_3c`. Unit test pins the gate,
   the draw-free return, and the sisters (a never-sighted type-3 guard queues 0xac..0xb2; a
   plain `ever_seen` without 0x3c does NOT gate).

## How verified

- **Unit (nf-game lib, `cargo test --offline --release -p nf-game --lib`): 563 passed / 0
  failed**, new `attack_talk_type3_gates_on_the_ever_sighted_bit` (+ sisters).
- **Live (NF_ISO, `guard_barks_07000013_live`): 4 passed / 0 failed** —
  `the_lobby_guards_are_voice_type_3` (every loaded lobby guard voice 3 + DTYPE pair 3/0xc),
  `the_disc_props_split_the_loaded_guards` (6 create; the two prop-less guards Idle, the rest
  combat-family), `the_first_sighting_chain_barks_nothing` (1500 ticks: no 0xac..0xb2, no
  0x261..0x266), `crosshair_on_a_lobby_guard_is_silent` (240 ticks face-to-face: no talk, no
  speech).
- **Regression:** `interactables_07000015_live` 11/11, `fix_blast_scripts_live` 3/3.
- The `tools/test/suite.py` fast runner does not survive the background kill between tool calls
  in this sandbox (measured again this session); the crates the change touches were run as
  above, plus the shared live suites listed.
- Broad slice: `cargo test --offline --release -j2 -p nf-game -p nf-formats -- --test-threads=1`
  — nf-formats 89 lib + 5 test bins green; `disc_golden` 11 passed / 3 failed = exactly the 3
  slim-ISO `disc_golden` failures the task allows (bigf_header/bigf_tables/directory_tree
  golden manifests); nf-game's own runs are the 563/0 lib + the live suites above.

## Corrections made to the docs

`docs/levels/07000013-audit.md`: GAP-13-5 closed with the pins above; the kind-0x0f param row
corrected [D placement dump] — all 12 guards carry **p15 = 3** (the audit's "none carries
p15/p16" was wrong; dtype 3 still falls to mode 0 in `GetDTYPENEW` 0x14c938 cases 0-9 with
alert 0, so the start-state rule stays the props), and **p4 is the spawner group id** (the
audit's "p4 route" misread; the level doc's §4 skin claim 0x05000038/3a is stale the same way —
superseded by the audit census).

## What's left

- The **never-seen attack bark end-to-end** (a noise/peer alert on a guard who has not yet seen
  Bond → 0xac..0xb2 through the audio): the unit test pins the talk-queue half; a live pin
  needs a spot where Bond can make muzzle noise without entering the Idle pair's sight (my one
  attempt at the west desks got him seen — `ever_seen_3c` set, gate engaged, silence correct
  but the positive path unexercised).
- The spawner-template guards (groups 1/2, #321/#322/#325/#326/#408/#409) become bark-eligible
  when their channels (0x29/0x2a) fire — same rules, worth a live walk in a later pass.
- **Observation for a later task:** guard #317 reached a combat-aim state while the port's
  sight-ack (`seen_ticks >= reaction_time`) had not yet fired — the state entry and the ack use
  different latches in the port; the game's ordering (HaveOpponentSight acks THEN FirstAttack
  talks) may differ. Listed as a live check below.

## Live checks for the main agent

1. `gdbwatch.py` on **Drone+0x228** (u32, the vision word) for one lobby guard on 07000013 from
   a fresh boot, standing at spawn 10 s: expected — bit 4 (0x4 seen) and bit 8 (0x8 ever-seen)
   set within ~1-2 s of the intro release (prop 0x3c is on [D]), and `AttackTalk` 0x13eac0's
   type-3 arm returning at :64 on every attack entry afterwards.
2. `gdbwatch.py` on the **global PatrolCooldown 0x298FC4**: expected — 0 at load; after a guard
   first enters PATROL, either `now + 300` (lost Rand(2)) or `now + 600` (a 0xb3+ line played).
   Confirms whether the game's lobby guards EVER reach PATROL in a normal playthrough before
   ch 0x28 fires.
3. `fieldprobe.py` poke: fire a weapon near the west-desk Idle pair without line of sight
   (e.g. bond teleported behind the shutter wall) — expected: `AttackTalk` 0x13eac0 called with
   `+0x228 & 8 == 0` → one 0xac+Rand(7) line through `Sound_Play3D`. This is the positive bark
   the live sandbox could not reach.

## Usage

not visible to me — this session exposes no token, context-window or cost figures (no usage
panel in the environment; nothing to copy).
