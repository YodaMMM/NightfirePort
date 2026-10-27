## Your task: `drone-bump-collision`, guards and Bond stop passing through each other

Guards walk straight through Bond on every level (and through each other): the drone collide step of the control pass is not ported. Port it from origin/master:
- `control_funcs` class 2 collide = `Drone_CollisionHandler` 0x140968 [M table @0x26f8a8]; the probe is the two-end capsule ext+0x3F0/0x400 with radius ext+0x45C = 0.4 (0.55 in state 0xC, 0.3 in the +0x2F variant), armed per tick by `NDrone2_Collision` 0x140e90:59-124 under the `NDrone2_DoCollision` 0x149ec8 gates (the 0x20/0x200 flag arms, the 0700004A special case, `DroneFunc_NearPlayerCollision` 0x14b190's 2.0 near-player arm).
- Contacts mirror into both sides' +0xD0 [M Collide_Update 0x1ea398:160-233]; the consumer dispatch sends the player bump msg 0x22, bullets 6/7/0x19/0x17, and the default arm (walls/doors) = velocity zero + deflect [M 0x13fff0:127-147]. Port the msg-0x22 arm's behaviour from its handler (find and cite the address).
- Bond's side: the port's only actor push is Bond-side and airborne-only (player.rs, `!self.grounded`, marked [S]); replace it with the game's rule.
Read `origin/s/level-07-guest-collision-5`'s report first: rounds 1-4 of a narrower attempt broke the walkreal routes (party walk on 07, 0a armour pickup) because a push shifted guard paths and RNG; measure each walkreal/escort test before and after and fix causes, never re-pin a walk to make it pass. Tests: a guard walking a line into stationary Bond stops or deflects at the capsule distance [M]; Bond walking into a standing guard is blocked.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
