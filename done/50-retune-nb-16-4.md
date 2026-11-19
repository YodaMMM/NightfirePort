## Your task: `retune-nb-16-4`, 16 chasers-defeat 00 Agent real-damage walk green again (round 4)

Branch from current origin/master; if origin/s/retune-nb-16-3 is not on master, merge it in first. Read docs/reports/retune-nb-16-3.md there and the ignore message on walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. If no untried lever is left, scan the armour and health pickups near the death first, then sweep the idle wait before the fatal fight (each wait shifts the guards' roll). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000016_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-16-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-19T11:56Z

---
## Report (done, 2026-11-19T13:28Z)


---
## Report from the worker (done, 2026-11-19T13:28Z)

# retune-nb-16-4 — 16 chasers-defeat 00 Agent real-damage walk, round 4

## What was done

Round 4 of `walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent`
(branch `s/retune-nb-16-4`, from origin/master 2be83dc10; no `origin/s/retune-nb-16-2` or `-3`
branch or report exists anywhere on the remote, so master's walk file and its ignore message
were the ledger truth).

**The lever-(a) wire trace ran, and the wire theory died.** A `WireWatch` helper (file-local to
`walkreal_07000016_live.rs`, gated `difficulty >= 3 && !tops_up()`, reads and prints only — no
walk, no RNG draw) diffs every hurt volume's disabled bit at 1-tick resolution across the whole
chaser stage, and a loot-scan print names which volume (if any) claims each nearby pickup.
Measured on this tree: **the loot window has ZERO live-claimed spots**. The only stage flip is
the LiftCrush hurt #1 (gate 0xcb) rising at +3185 — its poly (0.5..4.4, −162..−158) is the bay
lift, 12+ u east of the blast room. The ShuttleLaunch ch-0xf3 pulses (script ticks 1611..2170
and 6810..7320 [D `blast_room_shelter_16_live::the_blast_doors_are_shut...`]) arm only the
launch script's own window, not the chaser stage. The round-2 ledger's "its spot measures
LIVE-WIRE-CLAIMED during the loot window" is stale on this tree.

**The real blocker was the round-2 `v.truncate(2)`.** The loot scan's kind-0x1 class holds two
boxes: the Ammo9 w 0x15 at (−15.5,−170.8) — the ledger's "med-kit" — and the Ammo556 w 0x1a at
(−16.8,−167.9). `Pickup_Handler`'s kind-1 arm is `Player_EquipAmmo` [M 0x18e650:210-214]; w 0x15
(MP5/Storm) is unowned, so that box **refuses forever** — the "+10.0 health buffer" the round-2
ledger built its zero-margin arithmetic on never existed (the run confirms: health 19.5 through
the whole stage, no +10 anywhere). Both boxes outranked the armour (kind 0x3) in the kit-first
sort, `truncate(2)` dropped the armour, and every run looted ammo only, then died at t12882 to
the chaser's three landed rounds (−10.00 f12709, −5.00 f12863, −4.50 f12882) with armour 0.0 —
reproduced byte-exact on the first run of this round.

**The fix** (all `difficulty >= 3 && !tops_up()`, file-local; every other walk byte-identical):
the loot pass moved ahead of the room-soldier fight, the sort is med-kit-tag → armour → ammo
(distance within class), `truncate(3)`. Measured: the loot window is 156 ticks (the armour hop
lands at +90 — kind 3 sets the pool flat 50.0 [M 0x18e650:220-224], the Commando box +50 →
spare 64), the room-soldier fight costs **0 health** (the armour absorbs it), the kill loop
takes her down at +3508 with health 19.5 untouched (her punches at the d 1.9 punch band land on
the armour), Bond sits pulse 2 out in the shelter, and the squad stands are entered at 19.5.

**New death, new frontier: t17282.** ONE round −19.50 from squad clone #12 (skin 0x500001e,
weapon 0x14, hp 100) at d 1.4 at (−12.62,−21.97,−169.69) with armour 0.0 — the 50.0 pool spent
across the soldier fight, the 3508-tick punch-band kill and the squad's early fire. Health 19.5
is the stage entry and **no health pickup exists in the pit room** [D the audit's 17-pickup
table: the pit pickups are the armour ×2 (#648/#649), Ammo556 ×2, Ammo9 ×2, 007 ×2 — every
kind-1 is ammo]. The refreshed ledger and the next levers are in the test's ignore message:
(a) the east armour #649 (21.62,−22.97,−168.83) [D] fetched during the squad stage (kind 3
refuses only at pool ≥ 50 [M 0x18e650:220-221]); (b) shorten the 3508-tick punch-band kill;
(c) hold the re-arm legs inside the room so the close stands answer a closing clone.

## How it was verified

- The ledgered walk was run 4× on this tree (instrumented, then the fix, then with
  `NF_PICKUPLOG=1`, then final): the first reproduced the round-2 death byte-exact; the fixed
  tree moves the death from the chaser stage (t12882) to the squad stage (t17282) — the furthest
  measured point of any round.
- `walkreal_07000016_live` **whole binary**: **18 passed / 0 failed / 1 ignored** (161 s,
  `--test-threads=1`, NF_ISO the onfoot ISO) — the d1/d2/tops-up walks byte-identical, the
  gates hold.
- The test file is the only code change; no `crates/*/src/` files touched, so no other crate's
  tests are affected by the diff.
- Suite: the background `tools/test/suite.py` was started in the first minutes per the task but
  this sandbox kills background jobs between tool calls — it died with an empty `suite.out`
  before the build finished. The foreground nf-view build then hit two disk walls (the sandbox
  has ~10 GB; the ISO + work + target filled it; a stale 785 MB `target-scoped` from a previous
  session was deleted to recover). The full four-crate suite did not fit the remaining window —
  the same outcome the round-1 report documented for the nf-view binary set. What ran: the whole
  `walkreal_07000016_live` binary above.

## What's left

The walk is still red — one landed round short. The next round should: skip the w-0x15 tag in
the loot scan (it can never take; saves its 66-tick hop at the stage's start), then take the
east armour #649 during the squad stage (lever (a) in the ignore message), or shorten the
punch-band kill (lever (b)), or hold the re-arm legs inside the room (lever (c)). The
`WireWatch` and the loot-scan claimer print stay in the tree — they are cheap and they settle
the wire question for good.

## Usage

not visible to me — this session shows no token, credit or context-window figures (no meter in
the environment).
