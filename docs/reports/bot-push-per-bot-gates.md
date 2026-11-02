# bot-push-per-bot-gates — the every-tick bot push with the game's per-bot gates

## What's done

**The fix.** `crates/nf-game/src/player.rs`: the actor-actor capsule push (`Collide_Jointy`
0x1ec180:229-281) no longer waits for Bond to be airborne. The disassembly has no ground gate — the
block reads only the two spines (two rounds of `vecutil_calculate_closest_point_on_line`
0x1df7e0, :234-239), the radii sum squared vs the dist² (:251-252, `+0x8c` of both hittests), and
fVar12; the old `!self.grounded` port choice is gone. `crates/nf-view/src/scene.rs`
(`step_bond`): the `bot_capsules` fill now ports the bot hittest's own arming:

- ALIVE bots arm only where `NDrone2_DoCollision` 0x149ec8 returns nonzero (the call at 0x14106c
  feeding the kind-0x800 store at 0x14107c/0x141084 in `NDrone2_Collision` 0x140e90): bit
  +0x4f8 0x20 must be set (DefaultInit 0x14b300's fold ORs 0x80|0x40|0x20|0x2 into every new drone;
  SetAsDead 0x14a2f0 clears 0x20/0x80 and sets 0x200 at death), kind +0xd8 0x13 returns 0
  (0x149f24-28), kind 0xc returns 1 (0x149f1c-20), then ANY of +0x538 != 0 (0x149f30-34 — the port's
  `anim.blend_out`), `DroneFunc_NearPlayerCollision` 0x14b190 (a player slot's +0x154 distance
  < 2.0 — the port's `dist_to_player`, filled per tick [M NDrone2_PreDroneControl 0x148db0:35-48]),
  obj+0xFC != 0 (0x149f4c-54 — the draw counter, the port's `drawn_grace`), or +0x570 & 2
  (0x149f5c-68 — unmodelled: the port has no +0x570 writer, so the arm never fires [S unverified]).
- DEAD bots (+0x4f8 & 0x200) arm UNCONDITIONALLY (0x140f8c writes 0x464 = 0x800 on the +0x200
  fall-through — the old `corpse`-exclusion comment cited DoCollision, which is the bot's own
  physics gate, not the hittest arm) with the BONE capsule: `AnimGetBoneWorldTrans` 0x1c3120 bones
  5/0x33 (0x80000005/0x80000033 at 0x140f0c/0x140f50), r 0.4 (f20 = 0x3ecccccd at 0x140ee0) — a
  lying-down segment near the floor, so Bond steps over bodies instead of standing on the old
  upright stand-in pillar.
- The arms: top = pos + up·r, bottom = pos + up·(r − h) (0x14101c `r − h` with f22 = +0xa0, the
  animated hip — the port's `hip_anim`), r 0.55 for kind 0xc else 0.4 (0x14104c-68).
- Every capsule that could overlap Bond's (0.55 + 0.4/0.55 < 2.0) is inside the near band, so the
  near fill loses no overlap.

**The test.** `crates/nf-view/tests/flank_fall_repro.rs` `the_old_courtyard_flank_stays_on_the_floor`
is un-ignored and PASSES: the overlap now discharges the tick it opens (a fraction of a unit per
tick, like the game) and Bond walks the gatehouse wall's end-edge past the two guards without being
carried across the plane x 10.915. Its sister `pressed_against_the_west_lip_bond_stays_grounded`
needed a pin fix: `ungrounded == 0` contradicted the decompile — the courtyard guard idles at
(12.74, −19.23) and the press line passes inside the Jointy reach, so the game's own push (geometry
only, no ground preservation [M 0x1ec180:229-281]) lifts Bond for a few ticks (measured: 5 ticks,
t13090-13095, apex y 6.49). The pin now asserts the hang signature instead: no ungrounded run over
20 ticks (the old band hang was unbounded), the press still reaches the lip, Bond lands.

**Walker mitigation.** `crates/nf-view/tests/common/walker.rs`: the stuck ladder ORBITS a bot
capsule that sits on the bond→waypoint segment (strafe on the away side + slight forward, until the
lane clears or 900 ticks) — as the fallback AFTER the opt-in `body_block_response` shot arm. The
every-tick push cancels a blind walk's progress into an armed bot, where master's no-push
trajectory walked through.

**Merge.** origin/master 88d87c9c merged clean (tests + docs only; crates untouched).

## How it was verified

Targeted runs (release, `--test-threads=1`, slim ISO `NF_ISO`):

- `flank_fall_repro`: **2 passed, 0 failed** (the un-ignored repro + the relaxed lip press).
- `walkreal_07000003_live`: **4 passed, 0 failed** (`walk_the_full_route_and_complete_every_objective`
  and the rest).
- `exit_aim_01_live::the_exit_resolves_to_the_gate_plate_and_e_is_refused_there`: **passed** (after
  the walker orbit — the walk crosses the hall crowd the game itself pushes around).
- `walkreal_07000001_live`: **2 passed, 4 failed** (`the_hall_guards_wait_for_the_meet_scenes_channel`
  and `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` pass; `following_mayhew_down`,
  `the_exit_fade_starts`, `the_plate_refuses_bond_alone`, `walk_from_the_spawn_to_the_exit_with_the_contact`
  fail — see What's left).
- `nf-game` lib tests: **654 passed, 0 failed, 5 ignored**.

## What's left

1. **The 01 escort walks need their routes re-tuned for the faithful push.** The four failures are
   the escort/crowd interaction: the contact (skin 0x4F, kind 9 — NOT a 0x13 exclusion; the game
   pushes Bond off him too) walks his own 37-node route and his AllyLead/escort AI reacts to Bond's
   pushed position; the walker's `clear_around` shooting spots and the follow legs were tuned over
   many sessions to the old no-push trajectory. The orbit mitigation gets the contact to route node
   15/37 (from 12) but the flow still stalls around the hall crowd (four guards at (6-7, 6.3,
   2-6.9), all armed correctly). This is route re-tuning, not a port bug — every bot involved is
   armed exactly as the decompile arms it.
2. **The full suite did not produce a total line in this window.** `tools/test/suite.py` was started
   three times on the final tree (merged with origin/master 88d87c9c); each build hit
   `No space left on device` (the sandbox has a 10 GB rootfs; ISO 1.06 GB + toolchain 1.3 GB +
   target/ 5 GB leaves too little headroom to link all ~200 nf-view test binaries + examples). Each
   attempt was resumed after pruning stale duplicate artifacts (freed ~1.5 GB across attempts);
   the last was killed at the 20-min deadline to free the disk for the required targeted runs.
   No FAILED/TIMEOUT lines were ever produced (the failure was at BUILD, not at test time). The
   next session should run the suite with a pre-pruned target or a bigger disk.
3. **Live checks for the main agent** (none run — no PCSX2 here):
   - Drone +0x4f8 bit 0x20 on a normal alive guard during play (expect SET, e.g. watch
     `gdbwatch.py` on the +0x4f8 word of a courtyard guard in 07000003), and CLEAR on his corpse
     after a kill, with 0x200 SET — confirming the arming polarity this fix ports.
   - Bond brushing the 07000003 courtyard guard at (12.74, −19.23): expect a small lift every tick
     he overlaps (the game's Jointy push, no ground gate) — the transient the relaxed lip-press pin
     now allows.
   - kind (+0xd8) of the 01 hall crowd drones at (6-7, 6.3, 2-6.9): expect values != 0x13 (armed) —
     confirming the crowd is a legitimate pusher.

## Usage

not visible to me

## ntfy failed

All three transports timed out from this sandbox (2026-11-02T09:32Z):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "...urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', ...), timeout=20)"`
   → no output, the call failed — the same 20 s timeout, ntfy.sh unreachable from this network.
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → failed the same way
   (connection timeout, no output).
