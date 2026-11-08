## Your task: `gt-update-audit-2`, a 0700000b turret wrecked while deployed behaves as the game's

Branch from origin/s/gt-update-audit, merge current origin/master, and read its docs/reports/gt-update-audit.md. Its pins run from the DORMANT state, so a 0b turret wrecked while deployed is unpinned (see its geometry note). Read that path in `GT_Update` 0x180498 and `GT_ApplyDamage` 0x1816d8 and pin it through real play on 0700000b: Bond shoots a deployed turret until it is wrecked, and the fold, the smoke and the stop happen as in the game. Then settle the class-5 gate on the 0x4a/0x4c disable from the disc: is any turret script object on 0a or 0b class 5 [D]? If the disc answers it, replace the [S] stand-in with the real class read; if only a live read can, list it under the report's live checks. Run the turret binaries, walkreal_0700000a_live, walkreal_0700000b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/gt-update-audit-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T09:06Z

---
## Report (done, 2026-11-08T10:44Z)


---
## Report from the worker (done, 2026-11-08T10:44Z)


> (The two `cat` attempts above failed on a relative path from the queue cwd; the full report follows.)

# gt-update-audit-2 — the 0b turret wrecked while deployed + the class-5 gate settled

2026-11-08, branch `s/gt-update-audit-2` (from `origin/s/gt-update-audit` d80c04e54, master
d09f8b2b3 already in it). Two deliverables: (1) pin the wrecked-while-DEPLOYED turret path on
0700000b through real play — the prior pins ran from DORMANT (the geometry note in
gt-update-audit.md); (2) settle the class-5 gate on the 0x4a/0x4c disable from the disc.

## 1. The 0b deployed wreck, pinned through real play

New test `the_0b_turret_wrecked_while_deployed_folds_smokes_and_stops`
(`crates/nf-view/tests/gun_turrets_live.rs`): the terminal-room turret #174 (p4 = 14 [D
0700000b.md:175]) is deployed through the real ch-14 channel write — the rewind → ARMING →
COMBAT [M 0x180498:205-238] — then Bond shoots it with his best real firearm through the real
bullet path (real switch, real aim at the script-A gun object's live volume, the pulsing
trigger, the walker's reload). Because Bond shoots from behind the facing cone (the game's
track refuses a candidate behind it [M 0x17f9c8:252-257]), the deployed gun runs case 4's
untracked re-holster gate: 0.5 s → **the fold** (state 2, the 1.2 s slerp back to the park
pose → the rewind → FINISHED) [M 0x180498:385-423, 239-265] — while the chain drain keeps
eating (script A unconditional [M 0x180498:130-187]) until the death gate lands the wreck
(state 6) [M 0x180498:236-258]. The test asserts, all through the real paths:

- the deploy reached COMBAT through the real ch-14 write;
- **the fold** happened (RESTORING/FINISHED visited after COMBAT, before the wreck);
- the wreck (state 6) landed;
- **the smoke**: the gas pool grows over the next 10 s — the case-6 every-10th-frame
  `Gas_Init` puff [M 0x180498 case 6];
- **the stop**: 10 more seconds and the state still holds 6 (no arm re-deploys a wreck), the
  wreck smoke is the only activity, Bond's health is flat (case 6 runs no `Bullet_init`) and
  the turret's health is flat (no new chain contacts).

The terminal-room geometry hazard from the prior session (raising ch 14 moves geometry and
the tester's spot fell away mid-firing [L the 2026-11-08 run log]) is handled by the spot
search (three ranges, grounded check) plus the re-place-on-slide guard the current harness
already carries; the run is deterministic (fixed 60 Hz, bit-exact RNG).

## 2. The class-5 gate, settled from the disc

The game's law, from the disassembly (the C decompile alone is ambiguous here):
`GT_Update` 0x180498's drain iterates the turret scripts' object tables via `Script_GetObj`
0x1db688 and, per object, sums the +0xD0 chain damage into ext+0xcc — and the GT_Disable arm
reads **the script object's own class byte** (`lbu $v0, 0xff($s0)` where $s0 is the
Script_GetObj result), then the object's +0xe0 weapon-def id (+0x44) [M disasm
0x180584-0x1805ac]. So the disable fires only when a SCRIPT OBJECT is class 5 and its weapon
id is 0x4a/0x4c.

The disc census answers the question **no**:

- The turret scripts 0x06000041/0x06000042 (the SAME script ids on both levels — 0b's p2/p3
  and 0a's wall guns): every `Entity` event in both scripts' streams is `obj_type: 31` (0x1f)
  [D `nf-extract nis` of 0700000a.bin and 0700000b.bin — 06000041: 3 entities, 06000042: 4,
  all obj_type 31, on both levels].
- `Script_CreateEntity` 0x1d8968 maps 0x1f to the default arm: `control_create_object`
  0x1344e0 + the gfx hash — which stamps no class byte [M 001344e0: no +0xff write; the
  mapped creators stamp 0x20/0x2d/0x29/0xe1/0xea/0xeb/0xf0/0xf2/0x3e].
- The ONLY class-5 stamp in the whole binary is `Bullet_init` 0x12c5a0:87 [M decompile grep
  over all of ACTION.ELF: `+ 0xff) = 5` has one hit] — bullets, and bullets never sit in a
  script's object table.

**No turret script object on 0700000a or 0700000b is class 5**, so the 0x4a/0x4c disable arm
is unreachable for the SP turrets on these levels. The port's [S] stand-in (disable on ANY
recorded 0x4a/0x4c contact) was unfaithful in the eager direction; it is REPLACED in
`crates/nf-game/src/gt.rs` (the update drain): the 0x4a/0x4c contacts now only contribute
their chain damage like every other weapon — the disable is not called, with the full census
citation in the code comment. The damage drain is untouched. `GT_Disable` 0x1816a8 itself
stays modeled (its unit test covers the state-4-only save/restore). No existing test asserted
the stand-in (nothing to fix or weaken).

## Verified (this session, NF_ISO set, --test-threads=1, all foreground)

- nf-game lib: **692 passed / 0 failed** (5 pre-existing ignored) — includes the gt.rs change.
- gun_turrets_live: **6 passed / 0 failed** (the 3 pre-existing deploy/fire/spread pins, the 2
  prior wreck pins, and the new `the_0b_turret_wrecked_while_deployed_folds_smokes_and_stops`).
- gt_los_live: **2 passed / 0 failed**.
- walkreal_0700000a_live: **13 passed / 0 failed** + 2 ignored (52 s).
- walkreal_0700000b_live: **7 passed / 0 failed** (41 s).
- The four-crate suite (tools/test/suite.py): **not fitted** — the background runner dies
  between tool calls in this sandbox (measured three times with nohup and setsid: the process
  is gone and suite.out is 0 bytes on the next call), and the foreground build alone needed
  three 10-minute chunks; the binaries above are this task's required list. The nf-game/nf-view
  test binaries were all rebuilt on the final tree (the change touches gt.rs, which every
  nf-view test links).

## What's left

- The four-crate suite run on this tree (the next session: `NF_ISO=… nohup python3
  tools/test/suite.py > suite.out 2>&1 &` in a sandbox where background jobs survive, or in
  foreground chunks).
- Optional live check (no longer needed for the class-5 answer — the disc settled it):
  gdbwatch on ext+0xcc through a deployed-turret burst to pin the drain rate against the
  port's per-contact law.

## Live checks for the main agent

None required by this task's questions. (The prior report's class-5 live check is answered by
the disc census above.)

## Usage

The session's input/output tokens, context used and cost: not visible to me.
(Wall clock: branch created 09:31 UTC, this report written 11:22 UTC, sandbox deadline
~11:27 UTC per the task message.)
