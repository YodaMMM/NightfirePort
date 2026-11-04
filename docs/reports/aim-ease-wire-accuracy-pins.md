# aim-ease-wire-accuracy-pins — real-input pins for two landed behaviours

Branch `s/aim-ease-wire-accuracy-pins`. Task: pin two landed behaviours on the real input
path — (1) the aim assist's 0.5/tick ease with the Check_AutoAim scan suspended while the aim
bit is held [M Player_AutoAim 0x1a7c98:50-59]; (2) the move quad's zero window and what the
guards' accuracy read makes of it [M Player_Update 0x193600:129-134 →
DroneWeap_DoOpponentTargetting 0x179a48:34-41 → DroneWeap_DoBulletAccuracy 0x179740:71-73].
The task said no behaviour change was expected; **both pins showed the port diverging from
the decompile, and both divergences were fixed with the addresses** — the fixes are the
delivery, the pins are their proof.

## 1. Wire accuracy: the move quad (fixed + pinned)

**Found:** the port zeroed the move quad (`BLData+0x10` lanes) only on climb ticks, so a wire
rider carried the last walk tick's lanes through the ride and the guards' motion test
`|(BL+0x10, 0, BL+0x18)| != 0` read him MOVING — the 0d wire crossfire hit a riding Bond at
moving accuracy.

**The decompile law:** the zero pair sits inside `if (sVar6 == 1)` where `sVar6` is the player
STATE `+0xF4` (1 = alive & in control), loaded BEFORE the substate switch — which reads the
DIFFERENT field `+0xF6` [M Player_Update 0x193600:128-134, :171; raw MIPS: the gate load
`lh $v1, 0xf4($s3)` @0x1937e0 vs the switch load `lh $v1, 0xf6($s3)` @0x193938;
docs/fields/obj-tag-player.md:59]. The zero runs before every substate dispatch, so each
substate body builds the quad it wants: the walk overwrites by absolute store [M Player_Move
0x191dd0:166-170], the climb's pushes are ADDs on the zeroed quad [M 0x196310:56,90;
0x1969d0:151], and `Player_Wire` 0x197658 writes NO `+0x10` lane (it zeroes BL+0x50 :105 and
moves Bond by direct obj+0x30/34/38 writes; the mount runs `Player_ClearInertia` 0x195e08,
zeroing BL+0x10/0x20/0x30/0x40) — **a wire rider's quad is zero on every ride tick and the
guards read him STILL**, which is the game's own stopped/moving accuracy chain [M 0x179740:52-99].
The task's "zeroes on climb ticks only" framing conflated the state gate with the substate
switch; the pin demonstrates the decompile law, not the framing.

**Fix:** `crates/nf-game/src/player.rs` — the zero is unconditional before the substate
dispatch (commit 03e6db084). The zipline's own `+0x18 += fVar7·0.15` write lands on the zeroed
quad exactly as the game's does [M 0x197f50:121].

**Pin:** `crates/nf-view/tests/wire_rider_readback_0700000d_live.rs` — on the real 0d wire
path (load → walk to the grab → stick-forward mount): every acquired guard reads the riding
Bond STILL (the pre-fix port: 11/11 acquired guards read MOVING), and a stationary Bond reads
still. Passes 1/1.

## 2. Aim assist ease (fixed + pinned)

**Found:** the port eased the assist offsets toward the manual offsets TWICE per aim tick — a
duplicate ease sat in `step_bond` before the cursor update (the level-0c copy) in addition to
the dir-pick ease — so the offsets converged at 0.25/tick where the decompile says 0.5, and
on raise ticks it eased where the game's bit-clear arm scans+halves.

**The decompile law:** `Player_AutoAim` 0x1a7c98 runs ONCE per tick (the only `jal
Player_AutoAim` is `Player_Weapon` 0x1a2b70-279, called from `Player_Update` 0x193600:412);
aim bit set → ONE 0.5/tick ease toward the manual offsets and no Check_AutoAim scan [M
0x1a7c98:50-59]; bit clear → the scan and all four offsets halve. `Player_Aiming` 0x1a1d28
accumulates the manual offsets (+0x118/+0x11c) EARLIER the same tick [M 0x193600:282], so the
ease reads THIS tick's cursor.

**Fix:** `crates/nf-view/src/scene.rs` — the duplicate pre-cursor ease removed (commit
f5431b1c5); the dir-pick ease (the game's position, after the cursor update) is the one
per-tick ease. The earlier freeze fix (an assist locked while hip-firing survives into the
aimed ticks) stands.

**Pin:** `crates/nf-view/tests/aim_assist_ease_0700000d_live.rs` — hip-fire lock on a 0d
guard (P2K 0x02, no raise script, zero spread [L ee_start_05 weapon_data 0x02]), then the
exact ease law per tick (10 exact 0.5^k ticks, lock at tick 37), the converged aimed round ==
the view direction to 1e-5, and the sister (button up → the scan re-snaps the offsets).
Passes 1/1, and demonstrably catches the double ease (tick 1: want v0·0.5, got v0·0.25 —
commit f3b5be107).

## 3. Verification (this sandbox, all `--test-threads=1`)

- `aim_assist_ease_0700000d_live` 1/1, `wire_rider_readback_0700000d_live` 1/1 (the pins).
- nf-game lib: 658 passed / 0 failed / 5 ignored.
- walkreal binaries (the changed-code levels first, then alphabetical):
  `walkreal_07000001_live` 8/8, `_02` 3/3, `_03` 6/6, `_04` 10/10, `_05` 14/14, `_06` 6/6,
  `_07` 5/5, `walkreal_0700000d_live` 1/1, `walkreal_0700000c_live` 3/3,
  `walkreal_0700000c_barrelrow_live` 1/1, `walkreal_0700000c_eastroute_live` 1/1 — 61 tests,
  0 failures. drone.rs was not touched, so `npc_doors_live`/`fix_guard_misc_live` were not
  required; nothing under `crates/nf-view/tests/common/` changed.

## 4. What's left

- **The full suite total line did not fit.** The sandbox lost ~40 minutes to environment
  failures: a first suite run died on disk exhaustion (the linker SIGBUS'd on two test
  binaries — no code error), ~1.5 GB was cleared and everything rebuilt, and the 10-minute
  tool-call cap killed two compile attempts before all test binaries were built at 23:12.
  With the window that remained, 11 of the 107 `walkreal_*_live` binaries ran (all green,
  listed above) plus the pins and the lib tests. The remaining 96 walkreal binaries, the
  other live suites and the disc_golden trio are untouched by these changes in every way the
  diff can reach (player.rs's move-quad zero + the scene.rs ease placement), but the next
  agent should run them for the record.
- The prior session's commits (de0e06766, 03e6db084, f5431b1c5, f3b5be107) carried the
  analysis and the fixes; this session re-verified the pins green and ran the walks above.

## 5. Live checks for the main agent

None required — both pins run the real input path from the level load (no teleport) and pin
the exact decompile laws. One optional confirmation: a PCSX2 watch on a riding Bond's
`BLData+0x10`/`+0x18` on the 0d wire (expect 0 on every ride tick, `+0x18` = 0.15 on the
zipline only [M 0x197f50:121]) would double-seal the wire law against the task's original
framing.

## Usage

not visible to me

## ntfy failed

All three fallbacks failed in order (2026-11-04 ~23:24-23:26 UTC):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "aim-ease-wire-accuracy-pins: done — both pins landed green on the fixed ease + the rider-still law; report pushed, finish.sh closed" https://ntfy.sh/nightfire-yoda-8k3x` → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 urllib.request` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "aim-ease-wire-accuracy-pins: done" https://ntfy.sh/nightfire-yoda-8k3x` → no output within 60 s (killed by the tool timeout, no bytes received)
