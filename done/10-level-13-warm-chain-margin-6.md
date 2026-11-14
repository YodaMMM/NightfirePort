## Your task: `level-13-warm-chain-margin-6`, 07000013's warm-chain real-damage walk gets past door 543 and out the front

Branch from current origin/master and read the ignore message on walk_the_warm_chain_to_the_front_entrance_and_out_real_damage in crates/nf-view/tests/walkreal_07000013_live.rs, plus docs/reports/level-13-warm-chain-margin-5.md on origin/s/level-13-warm-chain-margin-5. The 12 leg's corridor fights carry Bond into door 543's recess, the door opens from east of the hinge and the leaf pins him. Find a route or timing so Bond is clear of the leaf's swing when 543 opens (the game's own door logic is not to be changed for this), then carry the walk out the front entrance. The ignore lists what was tried; don't repeat those. Helpers stay gated to 07000013 and the difficulty, so the operative, agent and plain sisters stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever into the ignore message. Run the whole walkreal_07000013_live binary and interactables_07000013_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-warm-chain-margin-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-14T01:02Z

---
## Report (done, 2026-11-14T02:06Z)

# level-13-warm-chain-margin-6 — the security-room fight is decided by ammo; the ledger and the next lever are on the ignore message

**Branch** `s/level-13-warm-chain-margin-6`, cut from `origin/master` 42aff3fcb. The diff is one
file: `crates/nf-view/tests/walkreal_07000013_live.rs` (+41/-1 — the d3-gated doorway radius and
the measured-lever ledger comments; the ignore message carries the update-4 ledger). No
`crates/*/src/` changes, no shared-helper change shipped (the one opt-in flag tried this session,
`lane_fetch_far`, was measured and reverted — `walker_b2.rs`/`damage_log.rs` are byte-identical to
master). No damage or health numbers touched; real input only; no top-ups.

## What was done

The ignore's frontier (11-13 update 3) reproduced byte-identical on this tree [L /tmp/d3run0.log]:
the walk dies at t4958 (13 local) at (-27.26, -368.30, 36.03) inside the security room, killed by
#9 (skin 0x5000006, weapon 0x10) — 11 hits for 40.5 + 59.5 into armour, 5 return-fire fights. The
run's ledger pins the mechanism:

1. **#9 is a 25-hp-class guard** posted at the room's bottom east corner (-19.6, -368.4, 25.6),
   and **he wakes on movement only**: he never fired through the inside drain's ~1050 static
   ticks (the drain stood #8 down and waited out seven quiet windows), then opened 19 ticks after
   the descent leg's first step. The doorway sweep cannot reach him either — see below.
2. **The fight is won by rounds, not exposure**: the fights answered from the room mouth and
   landed 17 of his 25 hp; the in-hand clip was 8 and it DRIED mid-fight — the leg's own resupply
   fetch then displaced Bond back up into #9's line and the second hit (−7.5) killed.
3. **The doorway pre-clear at 18 is a measured no-op**: no box opens through the open door — not
   even #8's at d 7.9 (the sweep stood silently; the death stayed byte-identical t4958). The
   margin-3 "the line runs straight down the room" law does not reach #9's east-corner post from
   the threshold. Kept in the tree, d3-only, documented (it is the same shape that reaches #4).

## The levers measured this session (all d3-gated, six real runs, all ledgers [L] /tmp/d3run0-6.log)

- **run 1** — doorway sweep radius 12→18: byte-identical no-op (t4958, same totals). Kept.
- **run 2/3** — a deliberate resupply after the inside drain (the room's own ammo box at
  (-27.2, -369.4, 30.1), d 6.1, within the lane 12-u band): the fetch IS room movement — #9 woke
  at the first step, the nav's fights dried the clip, the nested fetch displaced Bond — death
  t4958 mid-fetch, byte-identical across both runs. Reverted.
- **run 3 (door side)** — a deliberate resupply at the security-room door, lane 12-u band:
  `resupply done (false)` with ZERO attempts — every gun drop sits > 12 u from the door spot or
  outside the ground band. Zero ticks. Reverted.
- **run 4** — the same door-side fetch under a far band (a temporary opt-in `lane_fetch_far` on
  the lane resupply's 12-u limit): the nearest qualifying pickup was the room's **BodyArmour unit
  at 12.8** — a fetch INTO the room's west lane; #9 woke mid-nav; death t4693 (earlier than
  master's t4958). Reverted; the flag reverted out of the helpers.
- **run 5** — the fetch moved to the north-west leg end (before the drain): the pocket drop was
  taken — **clip 4 spare 28, the rounds the room fight needs** — but Bond stood in the spawn
  pocket with the balcony rifle #4 STILL ALIVE (the drain that kills him runs at the door, two
  legs later): seven −7.0 rounds in 73 ticks, death t4115 at (-8.01, -368.30, 30.29). Reverted.
- **run 6** — the far-band fetch after the drain with the room's own pickups pushed into
  `no_fetch`: the band walked the spawn-pocket drops instead — a ~40 u excursion through the
  respawner stream's pocket, TEN return-fire fights, the drops' clips nearly empty (clip 1-4,
  spare 0-28), 16 hits for 40.5, death t6469 at (-7.02, -368.29, 30.85). Reverted; the flag and
  the no_fetch pushes reverted.

**The synthesis**: the rounds the room fight needs ARE on the floor (the pocket drop, spare 28)
but every static path to them crosses the respawner stream's kill zone, and the dropped guns
themselves carry nearly-empty clips — the SPARE pool is the value. The next lever is written into
the ignore message: **bring the rounds from 12** (enter 13 with spare > 0; the 12-side boxes give
clip only — clip 30 spare 0 at the shaft [L the runs]) **or take the room's own box as a
WALK-OVER leg down the d2-proven west lane BEFORE the middle legs** (the box at (-27.2, -369.4,
30.1) is 4.9 u from the inside spot; the take tops the clip before the middle exposure —
unmeasured; the d2 sister walks that lane green, and a walk-over take is not a fetch nav: no
fight machinery intervenes).

The 543 pin (the task's original phrasing) was already passed before this session (update 1);
the walk's frontier is the security room, and the ledger above is what the next session needs.

## Verification (what ran, on this tree, all `--test-threads=1`)

- `walkreal_07000013_live` WHOLE: **3 passed, 0 failed, 1 ignored** — the cold walk, the Agent
  sister, the Operative sister (their routes byte-identical — every change this session is
  d3-gated), the d3 walk ignored with the update-4 ledger [L /tmp/wr13full.log].
- `interactables_07000013_live`: **10 passed, 0 failed** [L /tmp/ia13.log].
- The fast runner (`tools/test/suite.py`) could NOT run in this sandbox: the background start
  died instantly between tool calls (this sandbox kills background processes — the margin-5
  report documented the same wall), and the foreground full-suite build is the disk/time wall the
  margin-5 report measured (the ~130 statically-linked test binaries do not fit beside the repo,
  ISO and work/; 3.5 G free at build time). What stands instead: the two binaries this task
  names, whole, green; the changed file's other tests (the three sisters) green in the same run;
  and no shared-helper change shipped (both helper files byte-identical to master), so no other
  binary's behaviour can move.

## What is left

- The d3 walk itself — red at t4958, the ledger and the next lever in the ignore message
  (11-14 update 4). The first items: the rounds from 12, or the room box as a walk-over leg down
  the west lane before the middle legs.
- The fast-runner total line (disk-bound out, as in margins 3/4/5).

## Live checks for the main agent

1. Fresh boot of 07000013 at 00 Agent: stand at the security-room door after opening it and
   confirm the room's bottom-east guard (#9's post, (-19.6, -368.4, 25.6)) NEVER opens his box to
   the gun-ray set from the threshold — the port reads no exposure at 18 u (if the real game's
   player can shoot him through the door, the threshold stand is the cheap kill).
2. Fresh boot of 07000012 at 00 Agent: confirm a picked-up dropped rifle's SPARE pool carries
   across the level exit into 0700004a/07000013 (the port's `Inventory` carries it; the room
   fight's next lever spends that pool — if the real game zeroes spares at the shaft, the lever
   is dead).

## ntfy failed

All three fallbacks, in order, with their exact errors:
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-13-warm-chain-margin-6: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'level-13-warm-chain-margin-6: …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "level-13-warm-chain-margin-6: done …" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output within 45 s (killed by the timeout wrapper; no bytes received).

## Usage

not visible to me (no token/credit/context figures are exposed in this environment).
