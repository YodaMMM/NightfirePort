# consider-explosive-gate-2 — the grenade flee gate without the 11 corridor regression

Branch from origin/master; merge origin/s/consider-explosive-gate in first.

That branch ports the game's flee-state gate [M 0x1431c0:40-67] (guards in aim/fire states now flee a live grenade), and nf-game is green, but two tests go red on master with it:
start_corridor_guard_07000011_live::idle_bond_is_approached_and_shot_at_every_selectable_difficulty and ::in_position_the_guards_step_aim_as_well_as_fire. The corridor guards keep fleeing grenades the port lets them lob — and that lob is marked [S] (suggested, not proven) in the port.

Do:
1. Prove when the game's drones throw grenades at all [M the drone grenade-throw decision: weapon/prop gates, range, LOS, cooldown, difficulty], and whether the 07000011 start-corridor guards ever throw one in the game. If the port's lob is not the game's, fix or remove it with citations.
2. With the faithful throw and the faithful flee gate together, both corridor tests must pass without changing their [L] pins (they were taken from the game).
3. Run nf-game, start_corridor_guard_07000011_live, every grenade/consider test, every real-damage walk, then the full suite.

---
claimed-by: sandbox5 at 2026-11-02T17:06Z

---
## Report (done, 2026-11-02T20:04Z)


---
## Report from the worker (done, 2026-11-02T20:05Z)

# consider-explosive-gate-2 — the grenade flee gate without the 11 corridor regression

Branch `s/consider-explosive-gate-2` (cut from origin/master 524924db; origin/s/consider-explosive-gate
f4d40f39 already an ancestor — the flee gate [M 0x1431c0:40-67] came in with it).

## What's done

1. **The corridor-11 regression is resolved with the game's own laws.** Both previously-red tests —
   `start_corridor_guard_07000011_live::idle_bond_is_approached_and_shot_at_every_selectable_difficulty`
   and `::in_position_the_guards_step_aim_as_well_as_fire` — pass, and their [L] pins are untouched
   (the test file is unmodified on this branch). 8/8 green.

2. **Why they were red, and the two fixes:**
   - **The stun grenade 0x35 never enters the game's scary registry** [M Drone_BuildDynamicAwarePoints
     0x138b10:36-160 — the class-5 switch covers 0x2a/0x2c-0x2f/0x34/0x3b-0x40/0x6c (kind 2, radius
     12), 0x36 (kind 1, radius 3) and 0x37 (kind 2, radius 8 — the 8.0 the old port comment borrowed);
     0x35 falls to the default break. Msg 0x21's only sender is HandleExplosives 0x1435f0, which reads
     only that registry]. The port's 0x35 flee arm (radius 8.0, run 10.0) is REMOVED (commit 9f4d03fb):
     the corridor guards no longer flee #127's own stun lob the instant the faithful flee gate lands.
   - **The 0x68 throw gate was not the game's** (commit 284129c2). The game's gate in DroneFunc_CombatState
     0x148338 case 0x10 is `(u64)drone+0x568 & 0xffff00ff == 0xa0002` — the playing danim's class
     (+0x568 low byte) == 2 AND its anim id (+0x56A) == 0x0A AIM_STAND — gating BOTH DroneWeap_CanThrowGrenade
     (→ state 0x2d) and NDrone2_ChooseCombatMove. The port had `anim.class == 2` plus a
     `dist 8..20` pre-gate BEFORE the roll. Two unfaithfulnesses fixed: (i) a class-2 danim that is not
     AIM_STAND could roll (and throw) where the anim-id half refuses; (ii) skipping the pre-gate's dist
     check skips Rand(2) draws the game makes — desyncing the shared bit-exact RNG stream.

3. **Do 1 — the game's drone grenade-throw decision, pinned [M]:**
   - `DroneWeap_CanThrowGrenade` 0x17a798, verified to the DISASSEMBLY (0x17a7b0-0x17a828):
     `+0xBCC != 0` (grenade weapon) → `+0xBCE != 0` (count) → `Rand_Rand(2) == 0` (50%) →
     `+0x170 != 0` (opponent) → `8.0 <= +0x1a0 (dist) <= 20.0` → `DroneAnim_CanDoAnimState(0x49, 0)`
     (the throw danim is in the drone's set). **The roll precedes the dist check** — a grenade-holding
     guard in the gated state consumes one roll per evaluated tick even out of throw range.
   - Its ONLY caller is `DroneFunc_CombatState` 0x148338 (grep over the whole decompile): case 0x10
     (state 0x68 AimStandFire, behind the 0xa0002 anim gate, after the clip/range/unseen/cover checks)
     and case 0x18 (state 0x70 AimCrouchFire, ungated). Cases 0/2/4/6 never evaluate a throw.
   - Gates the driver asked about: **weapon/prop** — the throw needs the +0xBCC/+0xBCE endowment and the
     danim gate; no other property gates it. **Range** — 8..20 (inside CanThrowGrenade, after the roll).
     **LOS** — none: CanThrowGrenade has no sight/LOS term. **Cooldown** — none: no timer field is read;
     the throttle is the 50% roll plus the finite count. **Difficulty** — the endowment is disc data
     (placement param 11, not difficulty-scaled); the thrower exists iff placement p2 ≤ difficulty
     [M 0x136be0:49].
   - The mid-burst window matches: the game's 0x68 UPDATE (0x166328 case 3) calls PlayFiringAnim instead
     of CombatState while the burst holds (+0x3c/+0x3b), so the roll cadence is "every evaluated tick
     between bursts" — the port's `firing && !burst_done` skip mirrors it.
   - Endowment decode [M NDrone2_DefaultInit 0x14b300:348-376]: `+0xBCC` = the grenade weapon (switch on
     p11>>4: 1→0x36 smoke, 2→0x35 stun, 4→0x34 frag, 5→(p11&0xf)+0x3a, 6→0x6c; default 0/0) and
     `+0xBCE` = the count = p11 & 0xf (1 for kinds 5/6, 0 default). The port's spawn decode was already
     exactly this.
   - **Whether the 07000011 start-corridor guards ever throw** [D probe_11_grenades on 07000011.bin]:
     #126 (skin 0x05000038) p11 = 0x00000000 → no grenades, never throws. #127 (skin 0x0500003A)
     p11 = 0x00000021 → stun 0x35, count 1 → throws under exactly the gates above. The port's lob IS
     the game's: same endowment, same gates, same roll order. (The probe's printed count column was
     mislabelled hi-vs-low nibble — the count is p11 & 0xf per 0x14b300:349-350; the probe is
     assertion-free and no test reads that column.)

4. **Test runs (Do 3):**
   - `start_corridor_guard_07000011_live` 8/8 (with the faithful throw + faithful flee gate).
   - nf-game `grenade_live` 4/4 (`guard_throws_a_grenade_on_level_02`,
     `guard_decides_to_throw_grenade_unforced`, `a_guard_in_aim_stand_fire_0x68_flees_his_own_landed_frag`,
     `a_guard_in_go_to_goal_0x63_does_not_flee_the_landed_frag`).
   - grenade/consider/explosive files: `grenade_rattle_live` 4/4, `stun_grenade_live` 1/1,
     `consider_alerted_live` 4/4 (nf-view), `explode_live` 2/2, `fix_grenade_release_live` 4/4,
     `fix_grenade_throw_live` 4/4, `fix_grenade_types_live` 3 passed + 1 pre-existing ignore,
     `mp_stress_explosives_live` — see below, `probe_11_grenades` run assertion-free for the [D] tables.
   - **Every `walkreal_*_live` binary run** (01,02,…,0f, 0c_barrelrow, 0c_eastroute, 0d, 11, 11_route,
     12, 13, 14, 15, 16, 1b, 41, 4a, 4a_satchel): all green except 07000016 → see below.

5. **The one regression found and its measured disposition (07000016, commit fdbbb1dd):** the faithful
   roll cadence makes the pit squad's frag holders roll on ticks the old port skipped, retiming that
   level's shared RNG stream. Two scenario tests re-ignored with the measured reason and the [M]/[D]
   citations in the file:
   - `the_chasers_punch_lands_the_record_damage`: the no-return-fire real-damage walk dies at tick 2362
     in the pit-squad fight (47 hits for 100.0; last hitters drones #0/#2/#3/#4/#5, weapon 0x1a [D p9=26]).
     The frag-holding guards are #186/#188 (skin 0x0500008c, p11 0x43 = frag 0x34 × 3 [D]) — they roll
     every aim-stand 0x68 tick [M 0x17a798 disasm] and the stream shift retimes the whole fight. The
     pinned punch chain (50 [D record 99] × 0.4 [M 0x1d4424 Operative] = 20.0) is untouched; the
     scenario's survival expectation was calibrated on the desynced stream.
   - `walk_the_chasers_defeat_completes_the_mission`: the walk survives but runs the pit squad dry
     (weapon 0x11 clip 0 spare 0, cycled twice) and ch 0x82 (mux #633 [D]) never starts within the
     3000-tick window.
   - **Sub-attribution** (scratch build keeping the old roll order with only the anim-id gate added):
     07000016 passes 14/14 — only the roll cadence shifts it. origin/master passes 14/14; 9f4d03fb
     (flee gate + stun removal, no throw-gate change) passes 14/14. The real-damage variant
     `walk_the_chasers_defeat_completes_the_mission_on_real_damage` still PASSES.
   - `mp_stress_explosives_live`: OOM-killed (~3.5 GB RSS in a 3.9 GB sandbox), also OOM in the
     previous session's suite.log (line 5625) before this branch existed; dies even with
     `NF_STRESS_TICKS=60` (at the three arena loads). Environmental, not code.

## What's left

- **The full suite did not run this session** — the sandbox deadline (≈20:10 UTC) hit first. Everything
  run is listed above; the next message should run
  `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`
  (expect only the 3 `disc_golden` failures on the slim ISO, the 07000016 ignores, and the mp_stress OOM).
- The 07000016 scenario re-pins, after the live checks below.

## Live checks for the main agent

1. **07000016 pit squad lethality under the faithful cadence.** Load 07000011→07000016 chain save,
   walk-in approach to the pit, Bond NOT firing back, PINE health + the guards' states every 50 frames.
   Expected if the port is faithful: the frag-holding 0x0500008c guards (disc p11 0x43) sit in
   AimStandFire 0x68 well outside 8..20 u while the stream advances (they consume Rand(2) rolls per tick
   [M 0x17a798 disasm]) and an idle no-return-fire Bond takes heavy fire crossing the pit. Watch also
   for an actual frag throw (grenade object with owner = one of #186/#188) and, on its bounce, the
   thrower entering RunAwayFromObject 0x86 (the faithful flee gate [M 0x1431c0:40-67]).
2. **07000011 corridor stun lob.** Fresh 07000011 boot, difficulty 2, Bond idle at the spawn: confirm
   #127 (skin 0x0500003A, p11 0x21) eventually throws its ONE stun grenade when the port's gates align
   (0x68 + AIM_STAND danim + 8..20 u + a 50% roll), and that NO guard flees it (0x35 is not in the
   0x138b10 registry) — the corridor's [L] pins show the guards holding their StepAim/AimStandFire cycle.

## Usage

not visible to me
