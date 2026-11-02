# level-16-pit-squad-faithful-roll — the faithful grenade roll vs the 07000016 finale

## What was found (the premise of the task was inverted)

The briefing said origin/master carries consider-explosive-gate-2. It does not: the gate-2 branch
(`s/consider-explosive-gate-2`, tip 6e5eef87) is unmerged, and master carries only the earlier
consider-explosive-gate work. On master the two 07000016 walks were NOT ignored — they passed —
because the port still had the unfaithful fight laws whose fixes exist only on the gate-2 branch:

1. **The flee state gate** — master's `consider_explosive` refuses the flee from
   `0x28 | 0x2d | 0x2e..=0x4c | 0x52..=0x56 | 0x62..=0x88` (except 0x86). The game refuses only
   {0x28, 0x2d, 0x4a, 0x4b, 0x53-0x55} under state 0x56 and {0x63, 0x86-0x88} at/above it [M
   DroneFunc_ConsiderExplosive 0x1431c0:40-67 — re-verified in this sandbox against
   `work/decomp/ACTION.ELF/001431c0_*.c`: the `< 0x56` arm returns on `0x52 < s ≤ 0x55`, `s == 0x2d`,
   `s == 0x28`, `0x49 < s < 0x4c`; the `>= 0x56` arm on `s == 99 (0x63)` and `0x62 < s < 0x89 &&
   0x85 < s`]. A guard in AimStandFire 0x68 must consider the flee; master's gate refused it.
2. **The aim-stand throw gate** — master's AIM_STAND_FIRE arm pre-gated the throw on
   `grenade_count > 0 && dist 8..20` before calling `DroneWeap_CanThrowGrenade`, so the 50 % roll
   was skipped on every out-of-band aim-stand tick. The game's case 0x10 gates BOTH the throw and
   `NDrone2_ChooseCombatMove` behind one anim check — `*(u64*)(drone+0x568) & 0xffff00ff == 0xa0002`
   (class 2 AND anim id 0xA) [M DroneFunc_CombatState 0x148338 case 0x10, decomp re-verified] — and
   `DroneWeap_CanThrowGrenade` itself draws `Rand_Rand(2)` between the +0xBCC/+0xBCE checks and the
   dist check [M 0x17a798, decomp re-verified: `if (+0xBCC && +0xBCE) { if (Rand(2)==0 && +0x170 &&
   8.0 ≤ +0x1a0 ≤ 20.0) CanDoAnimState(0x49) }`] — the roll is consumed on every gated tick with
   grenades in hand, out of throw range too.
3. **The stun grenade's scare** — master classified 0x35 as (r 8.0, flee 10.0). The game's registry
   switch has NO 0x35 case at all [M Drone_BuildDynamicAwarePoints 0x138b10 — re-verified: the
   class-5 cases are 0x2a/0x2c-0x2f/0x34/0x6c, 0x3b-0x40, 0x36, 0x37; a grep for 0x35 in the file
   returns nothing] — a stun grenade scares no drone.

All three are landed on this branch from the gate-2 branch's drone.rs (that file's delta is exactly
these three regions; master's newer gt.rs/07000008/0700000b walk work was NOT dragged back).

## What was done

- **The plain walk** `walk_the_chasers_defeat_completes_the_mission` — with the faithful laws it
  failed exactly as the driver described (dry at the squad, then the lift-out wait timing out). Two
  fixes: (a) the wait read `ch 0x82 && disabled` — the port's movie player races a disabled-Bond
  script through in ~46 frames ([L] the pack cinema 0x06000631 ran 748 ticks across frames
  4938..4984; the lift movie 0x06000632 ran ticks 0..1059 across frames 13205..13251 — both runs),
  so when the squad fight runs long the last death's 0x82 edge fires AND the hand-back completes
  before the walk's first check. The wait now reads the latched 0x82 [D mux #633 `params
  [0:0x82 1:0xbf 2:0xc0]`] and the movie's own play-proof sits below it (the hand-back at key 1868
  (5.39, 1.34, −158.31) [D stream 13 event 6] + the 0x8d gate clear [D stream 16]); **14/14 green**.
  (b) documented where the fight actually goes: nobody reached the throw state 0x2d and nobody fled
  in either walk [L the NF_ALARM_TRACE runs — zero `-> 0x2d` and zero `-> 0x86` transitions]; Bond's
  ammo goes the chaser fight → dry → the walker's cycle → the squad fight → dry → the pit's own
  pulse finishes the last squad members ([L] ch 0x17 rises with Bond dry).
- **The grenade pins** — grenade_live +2 from the gate-2 branch (the flee gate's both sides, the
  pin-then-run convention): 4/4 green; ai_live 37/37 (the throw + the flee-family tests);
  grenade_rattle 4/4; stun_grenade 1/1; nf-game lib 655 passed / 5 ignored; fix_finale_live 4/4.
- **The punch walk** `the_chasers_punch_lands_the_record_damage` — the faithful cadence makes the
  trench hall's camp (7-8 guards: 0x500008c/0x50000b1, weapon 0x1a [D #533-#538/#691], the frag
  holders #533/#535 with 3× frag [D p11 0x43]) lethal to a passive Bond: 47 hits for 100.0, died at
  tick 2362, the heavy hitters two 0x50000b1 riflemen at d 1.9-8.6 (under 3.0 every round hits [M
  0x179740:113-115], doubled [M 0x139fac]). The player's tactic built on the walk (`clear_the_ring_guard`,
  file-local, no common/ changes): the kill order (frag holders first), the head-box kills
  (`prefer_head`, the existing 6 u opt-in window), the airborne-grenade jink (strafe flips every 30
  ticks while any guard grenade is airborne — the lob aims at Bond's throw-time position), the drop
  loot walk (Bond reaches the chaser holding the looted 0x1a at clip 25 spare 43 [L]), the vest
  #596 guard on the loot walk ([D PICKUP_BodyArmourHeavy #596 at (-19.03, 6.67, −94.48)] — the
  recorded delta is the HEALTH loss [M drone.rs's `player_hits.push((frame, owner, before -
  self.health.health))`], an armour plate eats the punch whole, so the pin needs bare health), the
  per-leg advance-fight rhythm with the leg re-walks (a displaced start crossed the lane hole at
  (5.7, -103) [L the trench's west-end wedge]), and the descent walked not fought (the ramp-edge
  wedge).

## State: `blocked`, and what's left

The whole camp clears — the best run reaches the chaser with **health 10.4, armour 12.4, the
looted 0x1a (25+43)** [L the run's logs] — but the pin needs health > 30 to run her window and
bare health for the chain, so:

1. **The vest take** — armour 12.4 at the chaser spawn even with the #596 loot guard: a second
   take point this session did not locate (the candidates are pit-vest #648 at (-14.41, -22.97,
   −172.75) on the shelter walk, or a fight displacement crossing #596 at < 4 u). `NF_PICKUPLOG=1`
   prints the `[take]` line with the tick; route the offending leg around it.
2. **The health budget** — the camp fight costs ~90 under the faithful cadence (the volley from
   the posts during each fight; the failed 900-tick range fight on #5 alone cost 33.6 [L]). Ideas
   for the next session: fight from the station's north wall cover instead of the open walkway, or
   tighten the tap window's re-aim cadence (aim re-picked every 30 ticks).
3. **The full suite** — NOT run (the sandbox deadline). Run first next session:
   `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio -p nf-view -- --test-threads=1`
   On the slim ISO only the 3 `disc_golden` tests may fail. Also run EVERY `walkreal_*_live`
   binary before finishing (the rule for common/ — this branch changes NONE of `tests/common/`,
   but the drone.rs fight laws touch every level's fights).
4. Then un-ignore nothing — neither walk carries an `#[ignore]` on this branch; both must simply be
   green.

## Live checks for the main agent

1. In the real game (07000016, Operative, the walk-in from the 15 hand-off): does a walked Bond
   who NEVER fires survive the trench hall to the chaser? The faithful port says no (47 hits for
   100.0, dead at tick 2362). If the real game's passive walk survives, the accuracy/fire cadence
   ports need a look against it.
2. Do the frag holders #533/#535 (p11 0x43, 3× frag [D]) actually throw at the walked Bond
   anywhere in the approach (the port never reached 0x2d in any run — the roll is 50 %/aim-stand
   tick, so the real machine's answer pins the whole RNG question)?
3. Does the game's movie player really race a cut-scene through in ~46 frames when the player has
   no input, or does the port's walk harness expose a pacing divergence (the pack cinema's 748
   ticks over 46 frames [L])?

## Verification summary

- `walkreal_07000016_live`: 13 passed / 1 failed (the punch walk, the state above) — the plain
  walk green 14/14 with the fix at its best run.
- `fix_finale_live` 4/4; `grenade_live` 4/4; `grenade_rattle_live` 4/4; `stun_grenade_live` 1/1;
  `ai_live` 37/37; nf-game lib 655/0/5.
- The decompile re-verified in this sandbox for all three cited laws (the files listed above).
- The full suite: not run (deadline) — first item next session.

## Usage

- Session tokens / cost / context-window figures: not visible to me (no token or cost telemetry is
  exposed in this sandbox).
- Wall clock: the session ran 2026-11-02 ~20:41 → ~21:55 UTC (the branch cut at 20:45, the
  checkpoint 42abb756 pushed ~21:55).
