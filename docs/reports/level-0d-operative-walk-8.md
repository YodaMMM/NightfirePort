# level-0d-operative-walk-8

Branch `s/level-0d-operative-walk-8` (origin/master 385ebb66d + `s/level-0d-operative-walk-7`
merged, clean). Task: the OPERATIVE-WALK-7 ledger's lever (1) — the #10 bay-act bullet audit at
d 55 on `walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`; real input only, no
top-ups, never lower damage or health; the plain replay and the d2 twin stay byte-identical.

## Status: still red — but lever (1) is ANSWERED with the flight's own law

The walk still dies in the ladder-top phase (this session's final-tree roll: tick 4160 at
(210.86, 16.12, −238.71), 10 hits for 100.0, 0 return-fire fights — inside the documented
3733–4683 dice band). The audit tool is new and stays on the branch.

## The audit tool (NF_0D_AUDIT=1)

`shoot_far_aim` grew an env-gated, read-only flight trace (`audit_bullets`): every tick it
prints each live `player_bullets` round's spawn pos/dir/weapon/speed/range and its death
state, with the flight's own verdicts computed at the last known point — `drone_ray_part`
over a 60 u reach (the flight's posed-box law) and `segment_flight_scoped` with the glass
pass (the flight's world law), stop point in absolute coords. No RNG draws, no sim writes;
unset = byte-identical (the plain replay and the d2 twin ran green on the same tree).

## Lever (1) — the #10 audit, ANSWERED [M/L citations in the ignore ledger]

- The spread is NOT the cause: the P2K 0x1e record's +0x24 = 100.0 base spread
  [L ee_start_05.bin @0x2bf150 + 0x1e·0x10C, both snapshots] is zeroed by the aimed mul
  (0x400000 flag + the aim bit) [M Bullet_init 0x12c5a0:139-170; the port's `player_spread`].
  Range 1200, speed 13.2 u/tick, damage 40, 1 pellet [L same record].
- The muzzle midpoint is NOT the cause: the P2K carries no 0x10000 flag, so the spawn is the
  eye [M Player_WeaponInitBullet 0x1a4bf0:226-245 — the port's None arm].
- THE CAUSE: `Bullet_update`'s per-tick START-CEL re-feed [M 0x12f0cc-0x12f0d8] — the flight's
  scoped pick runs from the bullet's own cel, and it stops EVERY line to #10 on the bay's
  north-wall strip east of the strip mouth:
  - head line: STOP (214.45, 12.89, −288.28), before the posed head box the same query sees
    past it (part Some(5, 0.805) over the remaining 60 u);
  - torso line: STOP (212.25, 11.93, −287.90), before the torso box (part Some(4, 0.704)).
  - the AIM gate (the eye's cel, `segment_bullet_scoped`) reads the same lines bullet_scoped
    CLEAR — the [0d:ray] lines — so the act fired converged, "clear" rounds into a wall the
    aim pick cannot see.
- #18's line passes the strip MOUTH at x ~204 (her stop z −291.02 is PAST her torso hit
  t 0.744 — the contact order is box first), which is why her act lands at +195 and his never
  does. **#10 is act-proof from the ladder top on this roll family.** The bay-duo loop keeps
  ONE pass (the measured +195 #18 kill) and the two #10 hunts are OUT (the two 400-tick
  passes were pure cadence exposure under the west-roof pair's fire).

## What else was wired and measured

- **The south-roof pair → Operative** (the d2 law verbatim: the present-pick + the no-drain
  box act at (169.6, −271.5) ± (5,3), cap 400 ×2, real-damage-gated). The baseline roll's
  #14 (skin 0x5000087, (166.3, 21.2, −271.4)) landed 22.0 with no Operative act on them.
  Both lever rolls the pick found the box EMPTY (the dice re-rolled them dead pre-phase) —
  the act self-gates at zero cost. STAYS.
- **The head-line gamble (ledger lever (2)) — MEASURED DEAD TWICE, OUT.** One 200-tick head
  act on the all-stalled roll's nearest ANY-box candidate grazed both runs (0 lands, 400
  ticks): the run-4379 graze law holds on this tree. Removed.
- **The hoisted fixed chest leg — MEASURED DEAD, OUT.** The fixed g28-spot chest leg at the
  phase start burned its 450 cap on a 0.972-BLOCKED chest ray (the run-4380 mid-band law
  verbatim, the guards settled at −243.42/−245.55). Roll A's +236 g29 catch was the leg
  catching her CLEAR CROSSING (the sweep law), not the fixed spot. Removed.

## The rolls (each on its own real run, shared stream, every act change re-rolls the dice)

1. the walk-7 tree (baseline reproduction): death 4514 — the walk-7 roll verbatim.
2. + levers v1 (bay-duo 1-pass, south-roof arm, gamble): death 4121 — #29 alone delivered
   70.0 on her 180-tick cadence; the fixed g29 leg landed her at +236 — after her 5th round.
3. + the hoisted chest leg: death 4345 — the leg burned 450 on the 0.972-blocked ray; the
   pair delivered 86.0.
4. final tree (the two dead legs removed): death 4160 — g29 landed at +197 (the fixed leg's
   crossing catch), the pair still delivered 94.0, and #10's late wake-kill 6.0 was the
   killing shot at 4160.

The act-fire dice correlation (the walk-6 law) held all session: every act change re-rolls
the incoming stream, and the 100.0 total lands before the pair dies.

## The refreshed ledger (in the ignore message)

NEXT LEVERS: (1) the pair's CROSSING SWEEP with the box act run BEFORE the retarget loop
(the -3 kills' own law: caps 900, the pass catches the crossing of a clear-ray spot — the
+222/+754/+193/+194 kills and roll A's +236); (2) the port-side capsule fix
[M NDrone2_Collision 0x140e90:91-121] (its own task); (3) the yard levers.

## Verification

- `walkreal_0700000d_live` whole binary: **2 passed, 0 failed, 1 ignored** — the plain
  top-up replay and the d2 real-damage twin green and byte-identical (all changes gate on
  `real_damage`/difficulty 1 or the audit env).
- `interactables_0700000d_live`: **11 passed, 0 failed**.
- The ignored Operative real-damage walk: dies at tick 4160 on the final tree (measured on
  its own run; 10 hits for 100.0, 0 return-fire fights).
- The full suite did NOT fit: this sandbox reset at session start (repo, ISO and toolchain
  gone — setup + from-scratch build consumed the first ~40 minutes), and the background
  runner cannot survive the sandbox's between-call process kill (suite.out stayed empty;
  everything had to run foreground). What ran whole: both task-mandated binaries above.
  What's left for the next session: `python3 tools/test/suite.py` on this tree after the
  crossing-sweep lever lands.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).

## ntfy failed

All three fallbacks failed in order:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-0d-operative-walk-8: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'...', headers={'Title':'NightfirePort'}), timeout=25)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "level-0d-operative-walk-8: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out and was killed after 60 s (no output; the plain retry with exit-code capture
   printed exit code 124 — the timeout kill).

