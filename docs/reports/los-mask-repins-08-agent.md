# los-mask-repins-08-agent — report

Branch `s/los-mask-repins-08-agent` = `origin/master` 114737d6d (the Operative real-damage
walk green since los-mask-repins-08). Task: take the Agent ignore message's levers so
`walk_the_whole_mission_on_real_input_real_damage_agent` passes and un-ignore it.

## Done

**The Agent walk is GREEN and un-ignored.** End health 14.9, armour 3.8, 61 hits for 85.1,
the whole mission walked on real input in 37505 ticks, both objectives folded, the exit
hand-off fired, the machine (hp 448.5) destroyed by five rockets through the slit (damage
98.8 → 500.5, dead at fight t1619) [L the run on this branch].

**The fix: the fight from the WEST DOORWAY SLIT** (the ignore message's lever 2). The
ledgered death (tick 37022, 69 hits for 100.0: −1.40 × 34 machine drips through the shot-out
end panes, then the −3.57 killing blast, over the pre-fight 33.6 from burst-pair #19 [L the
NF_DMG_TRACE runs]) was the standing spot between the six shot-out panes: the machine's
rounds fly the same glass-pass holes the rockets fly [M Bullet_update 0x12f048-012f058], so
the open cabin interior is a shooting gallery. The cabin's wall band is the only cover; its
west doorway (the band's z gap −178.25..−180.6, lintel y ≈ 14.31 [D the tri dump]) is the
firing slit. After the pane shoot-out the fight now walks Bond into the slit (real walk
input, the pane block's own bounded sidestep pattern, d2-gated) and the fight's aim and
launch gates read the live spot, so the rockets start through the slit while the band
shields the strafing drips. No damage or health writes, no top-ups; the Operative sister's
green timeline stays byte-exact (d2-gated, re-checked green on this tree).

**Levers tried and retired, each on a measured run** (the evidence stays in the test file's
blocks):

1. The pre-leg unseen lip stand at the descent's end: the half-open door leaves give Bond no
   shot line from the stairwell mouth — every 61-tick episode printed "held (out of sight or
   cap)", no round fired in 430 ticks, and the pair circled into flanking spots north of the
   mouth and burst the legs from the open. Retired with the note in place.
2. The d2 `reload_full` after the stairwell descent: the fights got cheaper (total hits
   100.0 → 76.9, the boarding chunk 74.8 → 32.1 — the drained clips had the fights cycling to
   the 4.0-range laser [L the run's "cycling weapons (0) (weapon 0x4e clip 500 spare 0)"
   prints]), but the R presses re-roll the shared RNG downstream (the walk's own wobble note
   [M Player_HandlePain 0x1902c8:111]) and the re-rolled boarding drove the stuck mitigation
   into a flank that walked Bond off the platform's east edge (the 23.1 fall, death tick
   30399). Retired; the fights keep their measured timelines.
3. The station-hall-mouth proactive stand (25 u, at the ch-0x2a plate): a no-op — no guard is
   visible from the plate at tick 373. Retired.

**Kept (no-op on the measured timelines, d2-gated, the d3-consistent player rules):** the
leg-1 unseen lip stand opened from d3 to d2 (`close_hidden`), the per-leg mid-boarding lip
stand opened from d3 to d2, and the leg-1 hidden-hostile ring widened 12 → 15 u. Each is a
gate change only; the runs with and without them are byte-identical (same ledger, tick 37022
before the slit landed), so the d3 sisters' behaviour is untouched and the Operative walk's
timeline is untouched.

## What ran (release, --test-threads=1)

| binary | result |
|---|---|
| walkreal_07000008_live | 5 passed / 0 failed / 1 ignored (the 00 Agent sister keeps its ledger) |
| interactables_07000008_live | 15 passed / 0 failed |
| nf-game + nf-formats + nf-audio, whole (174 binaries) | 1463 passed / 4 failed / 10 ignored |

The 4 failures are the slim-ISO disc-byte tests on master's own tree: the 3 allowed
`disc_golden` tests (`/DRIVING/MIS01.MUS: too small for a BIGF header`) and
`identity_disc::the_real_disc_passes_the_identity_check`. This branch's diff touches only
`crates/nf-view/tests/walkreal_07000008_live.rs` and `docs/`, so neither can move them.

**What did not run:** the nf-view crate's other test binaries. `cargo test -p nf-view`
relinks every nf-view test binary (~2 GB) and this sandbox has 9.9 GB of disk total (the
repo + work/ + the ISO + a single release build already fill it); per-binary runs of the two
task binaries above are the sweep this window could afford. A full-suite pass on the merged
final tree remains the first thing the next window should run. Only the 08 test file changed;
no `crates/*/src/` and no `tests/common/` files were touched, so no other level's walk can
move and the shared-helper opt-in rule is not triggered.

## What's left

- The 00 Agent (d3) sister keeps its earlier ledger (its own levers are in its ignore
  message); the doorway-slit tactic this window landed is the natural d3 lever to try next —
  the d3 fight already fights from inside the cabin and dies to the machine's own band
  detonations.
- The boarding ledger's #19 33.6 (the leg-1 nav stall window) and the walkway ~36.4 stand as
  measured; the margin is 14.9 + 3.8 armour — thin, deterministic. The stall-window levers
  (the reload's re-roll problem) need a wobble-free mechanism (e.g. a pre-fight route that
  shortens the nav stall without re-rolling the fights).

## Usage

not visible to me — this sandbox shows no token, credit or context-window figures.

## ntfy

1. `curl` → `curl: (28) Connection timed out after 20001 milliseconds`.
2. `python3 urllib.request` → `URLError: <urlopen error [Errno 101] Network is unreachable>`.
3. `wget -q -O- --post-data "los-mask-repins-08-agent: done" --timeout=20 https://ntfy.sh/nightfire-yoda-8k3x`
   → exit 0 (no output; a successful POST), so the notification went out on the third
   fallback. The queue entry (done/30-los-mask-repins-08-agent.md, pushed with the report
   attached) is the notification of record regardless.
