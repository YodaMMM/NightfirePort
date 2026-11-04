# real-health-agent-16-1b-2 — finish the 16 and 1b Agent walks

Read docs/reports/real-health-agent-difficulty-16-1b-41-4a.md on origin/s/real-health-agent-difficulty-16-1b-41-4a. On master walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent and walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_agent are ignored: Bond dies on real health (tactics, not a port gap). Re-route the way a player would. Real input only; no top-ups; helpers opt-in. Remove each ignore when green; every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox1 at 2026-11-04T03:58Z

---
## Report (blocked, 2026-11-04T04:31Z)

# real-health-agent-16-1b-2 — report (2026-11-04)

Branch `s/real-health-agent-16-1b-2`, cut from origin/master ecfaca48. Round 2 of the 16 and 1b
Agent (difficulty 2) real-health walks. **Status: blocked (not finished) — both sisters remain
red; this round measured the kill chains exactly and ruled four levers in or out. The next
session continues from the branch.**

## What's done

**16 — the kill chain is now attributed and four levers are measured** ([L] this branch's run
ledgers, `NF_DMG_TRACE` on the master-state test):

- The death ledger's 71.4 of the 100 (drone #3 35.7 in 17 hits, #5 16.8 in 4, #2 10.5 in 5,
  #1 8.4 in 4) is the **south gallery**: four AR guards (skins 0x500008c/0x50000b2/0x50000b1,
  weapon 0x1a, 10-25 hp) patrolling z −102..−112 at y 6.7 (#1 at y 0.8). Their FIRST landed
  round is tick 455 — at the ramp foot (Bond (11.8, 0.9, −87.4) from (−6.7, 6.7, −86.6)); the
  spawn legs (ticks 0-450) land nothing, so their fields never reach the spawn. They sit above
  the plain shooters window's dy 6.0, so the ramp-foot fights (vs the close ground pair
  #4/#0/#1/#2) run while the gallery plinks free.
- Lever (a) — a load-time (48, 18) shooters window: the fights engaged the gallery ACROSS THE
  PIT from the lane east (tick 823, one fight, "killed false"), the fights then never
  re-engaged, dead at 1271. The wide pool breaks the early geometry.
- Lever (b) — the (48, 18) window gated from the ring's east end (y 5.9, dy 0.8 to the
  gallery): the fights DID take the gallery 1v1 (head-hunting on) and killed #5/#1/#3/#1 on
  the ring — but the ring trades cost 44 (Bond 51.7 → 7.6 by tick 3495) and the chaser fight
  finished him at 8435. The mechanism works; the trades are too expensive to fit after the
  ramp foot's 48.3.
- Lever (c) — zero-movement `shoot_point` stands on every z < −95 guard from the spawn: every
  guard fails `DamageLog::visible`'s eye-ray gate from there (the ring structure blocks), all
  stands skipped, the timeline byte-identical to master's.
- Lever (d) — static-spot stands at the ring east: Bond died AT the stands (tick 2189) under
  the gallery crossfire; the stands outlast the health they save.
- The 16 test file keeps master's code with an updated ignore message carrying the law and the
  levers. Binary whole: **14 passed / 0 failed, 1 ignored** (73 s).

**1b — the wave-ring ledger and one lever measured**:

- The current master state: 36 hits for 100.0, death at 9856 (the prior branch's 32-hit state
  was its own tree — master's merged tactics measure worse here). The death ledger: the WAVE
  RING finishes him — six 0x5000078 astronauts (weapon 0x6a, −3.5 a hit) at 7.9-10.0 u firing
  in the last 215 ticks; Drake (#19, 0x50000ba, weapon 0x30) dealt 28.0 in only 2 hits earlier.
  Drake's 0x30 gun at −14.0 a hit (the prior round's measurement) remains the single worst
  item; the ring outnumbers the fight's one-target dps.
- Lever — the prior round's named "shorter dry window" (the recharge resume 60 → 30, gated to
  `waves_first`): measured **37 hits / death at 9405** — neutral-to-worse (the earlier resume
  pulls Bond out of cover sooner, not more kills). REVERTED; the file is master-identical.
- The next lever, named for the next round: the exposure scoring against the spawn BAND — the
  waves spawn/fight at y 10-12 while the shelter floats at y 3-6 and every death-round came
  from ABOVE (dy +4..+9); a hold the down-shooters cannot see is the shape `hide_and_snipe`'s
  y-scan (−6..+2 × 1.5) already searches, but its `score` weights clarity 3×/10× — a
  d2-gated re-weight toward "cannot be seen from y > 9" is the one-line trial. Binary whole:
  **17 passed / 0 failed, 1 ignored** (121 s).

## How it was verified

- The two touched binaries ran whole on the branch HEAD (release, `--test-threads=1`):
  `walkreal_07000016_live` **14/0 + 1 ignored**, `walkreal_0700001b_live` **17/0 + 1 ignored**.
- No shared file changed this round: `crates/nf-view/tests/common/` is untouched, no
  `crates/*/src` changed — the other 23 `walkreal_*_live` binaries are untouched by
  construction (rule 10's grep over the changed names returns only these two files).
- The full 4-crate suite did not fit the window (the same disk/time envelope the prior round
  documented); the branch's diff is comment-and-ignore-message only in the two level files.

## What's left

Both sisters green, in priority order:

1. **1b** (the closer one — Drake 28.0 in 2 hits + the ring): the d2-gated exposure re-weight
   against the y > 9 spawn band (above), or a Drake-specific early window (his r < 8 band
   already exists at r−60 — extend it to the first 500 ticks after his spawn at any range:
   both 28.0 hits landed at 9.3 u, outside the band).
2. **16** (the gallery): a cover-shaped gallery approach — the fights' own flank machinery on
   the gallery from the ring, one guard at a time, using the ring's south rim as the break —
   or the main agent's live scout: **is the gallery reachable on foot** (the station's west
   side? the trench's east wall?) — if the player can take the gallery level (y 6.7) the four
   guards become the same 1v1 trades the ring fights won, from a spot their fire cannot reach.

## Live checks for the main agent

1. **07000016, the south gallery's foot access**: a `--bond -16.3,5.9,-93.5` render looking
   south along the station (toward z −102..−112, y 6.7) — is there a walkway/stair from the
   station deck onto the gallery, or does the gallery face north over an unclimbable drop?
   That settles the next round's approach lever.
2. **0700001b, the spawn band's sight-lines**: a `--bond 13.98,3.10,-2.72` render at the
   measured death spot looking up at (16.9, 10.6, −1.5) — confirm whether the deck rails the
   `hide_and_snipe` y-scan trusts actually cut the y 10-12 shooters' lines from the shelter
   band, or whether the shelter spot itself is the leak.

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).

## ntfy failed

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "…" https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20001 milliseconds`
2. python3 urllib → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x` → no response within the
   30 s window (killed by the tool cap, no output).

(ntfy.sh egress is blocked from this sandbox — the same record the real-health-00agent-02-03-3
and real-health-00agent-12-13-14-15 sessions left in their reports.)
