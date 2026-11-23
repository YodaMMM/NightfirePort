# s/walk-13-op-3: the 13 warm-chain real-damage walk — the pool lever taken, the drain's tail and dry mag fixed, the armour take carried the exit phase, the frontier at the exit crossing (t9618), still red

Branch from origin/s/walk-13-op-2 160dda650 (strictly ahead of master 7562f1e70, so the round-2
ledger's tree is the base). Task: read the round-2 ledger in the walkreal_07000013 ignore message,
skip the measured-dead levers, take the next untried one (the door phase's POOL — a fetch at the
north-west end after the watch drain), judge each lever on its own real run, keep the sisters
byte-identical.

## What was done

**The round-2 death reproduced byte-identical first** [L the baseline run, NF_DMG_TRACE/NF_RF_TRACE,
/tmp only]: t6232, #11 out of ammo at +61 twice, the −3.5 kill from the re-fed #6 (hp 24, d 28
BEHIND Bond at (3.4,−368.4,42.1)) during the door drain's quiet waits — the ledger's own lines.

**Five lever states, each judged on its own real run:**

1. **THE POOL (the ledger's lever, taken as ledgered)** — the exit fill's own 3-pass +
   deepest-gun arm machinery at the north-west end after the watch drain (pool >= 25,
   resupply_home, the lane 12-u fetch). Measured: the pool ALREADY reads >= 25 there (the
   inventory depth), the fetch never walks, and the door drain **killed #11 true for the first
   time** (the baseline's #11 stands were clip 0 spare 0 statues — and the lever-1 log shows
   that kill was the lobby crossfire, not Bond's rounds: the stand started clip 0 spare 0).
   Death **t6342** (+110), the same −3.5 #6 volley in the drain's post-kill quiet wait — at
   3.5 hp the health-watch law cannot save the walk [L the lever-1 run].
2. **THE DRAIN-TAIL CUT** — the retry fires only when the stand dealt damage (the lever-9 law's
   own shape: a zero-dealt far stand has no tap-short pause to catch, and its 1400-tick window
   is statue in the shooter's own line — the lever-1 run's #6 retry burned 219 ticks for
   nothing), plus a `max_quiet` param capping the post-kill quiet waits (0 on the door drain;
   the other three drains pass their `max_rounds`, where the cap's break is unreachable — the
   old shape exactly). The first cut misfired (the cap gated the loop itself, the drain never
   ran, death t5350 at the E leg's err-2.9 fight [L the lever-2 run]); the fixed shape drew the
   stream where wounded #11 SURVIVED at hp 9, charged 23 u to d 21.5 and fired the kill —
   death **t6120** [L the lever-2b run].
3. **THE DRY RE-ARM** (`rearm`-gated to the door drain; the re-anchor answer block's own
   dry-gap law) — a mag that dries mid-stand cycles to the deepest gun + reload_full before the
   retry. #6 took 10 (hp 24 → 14), **the walk got INSIDE the security room**, the inside drain
   killed #8 AND #11, and death **t8540** was charged-#6's POINT-BLANK volley (d 4.7 — he
   charged the whole door phase's static window) during the inside drain's tail waits [L the
   lever-3 run].
4. **MEASURED DEAD, NOT REPEATED** — the far-floor STILL-gate swap at the door drain (the loose
   band is 0/11 at #11's d 17.9, so far floor targets should run the still gate): the aimed
   stand's 90-tick no-input settle + raise reshapes the guards' fire stream and #6's in-flight
   volley landed AT the arrival (+4 ticks), twice byte-identical — death **t5034** before the
   first stand's aim ever raised [L the lever-4/5 runs]. The loose gate stays the door drain's
   floor stand at every range.
5. **THE SHIPPED TREE** = the lever-3 shape + the inside drain's tail cut (`max_quiet` 1) +
   **the mouth charger answer** (the nearest live exposed floor drone within 12 after the
   inside drain, armed + `shoot_loose` ×2 — the charger law's own d <= 12 envelope; this
   stream's #6 never reached the mouth, the pick found nothing, the block passed clean).

**The final run's shape** [L the lever-6 run]: the door phase survived, the walk got inside, #8
and #11 killed by the inside drain, the charged #6 answered by the fights TWICE killed-true on
the room legs, the Spas12 take, **the BodyArmourLight take (armour 50 [M 0x18e650:220-225])**,
the shutter panel E (ch 0x23 [D SP #161 out]), the last-guard loop and the exit fights all green
at health 3.5 — **the armour carried the whole exit phase** — and the death landed at the exit
crossing's first stand ("the lobby rifle from the doorway"): the respawned #7 (skin 0x5000008,
weapon 0x1c, hp 9, at (−13.0,−368.4,39.7) d 13.1) fired −1.50 + −2.00 pellet ticks into the
armour-0 health-3.5 walk, **t9618**. That is **3386 ticks past round 2's frontier** (t6232).

## How it was verified

- The ignored d3 walk alone, seven runs (the baseline + five lever states + the final tree),
  each judged on its own run; every log kept in /tmp (not committed).
- The WHOLE walkreal_07000013_live binary on the final tree: **4 passed, 0 failed, 1 ignored**
  (the plain walk, the Agent sister, the Operative sister, the route-flag thread test — the
  sisters stay byte-identical: every lever lives inside `real_damage && difficulty == 3` gates
  or a defaulted-off param; the east-lane/watch/room drains pass skip=false, loose_floor=false,
  rearm=false and the old quiet shape).
- interactables_07000013_live (final tree): **10 passed, 0 failed**.
- `git diff origin/s/walk-13-op-2 --stat`: only crates/nf-view/tests/walkreal_07000013_live.rs
  (+ docs) — the changed code is one test file, so no other binary's behaviour can differ; the
  drain fn is file-local and all four call sites are d3-gated; no sim, draw, RNG, damage,
  health or guard-code change; no top-ups; real input only.

## Suite

The background suite dies between tool calls in this sandbox and the full build does not fit
(the old session's ENOSPC measure), so the fast runner could not complete here. What ran on the
FINAL tree (4de1b304d + the docs): walkreal_07000013_live whole binary 4/0/1 and
interactables_07000013_live 10/0 — the only two binaries that execute the changed file (the
diff is test-local; no other binary includes this code). No FAILED/TIMEOUT lines belong to this
task's change.

## What's left

- The walk is still red at the exit crossing (the ledger above; the refreshed ledger + the next
  lever are in the ignore message): the nb-13-4 mouth answer's radius-1.6 pick only matches
  guards IN the leaf — the respawned SPAS12 #7 sits 13 u out in the lobby and his pellets take
  the crossing's first stand. The next lever: take him from the doorway line first (the
  still-gate far band — the machinery that converged on #4 at 21.4 u) or widen the mouth
  answer's pick to the respawner's own post; the armour's protection is spent by then, so the
  answer must KILL, not trade.

## Live checks for the main agent

1. (carried from round 1) The mezzanine balustrade face x −16.29: from the re-anchor spot
   (3.29,−368.39,31.77), does a FRESH-BOOT player round at the mezzanine pistol guard #5
   (−17.9,−362.4,45.4) land on his box, or stop on the face? Expected per the port: stopped.
2. (carried from round 2) The re-fed wave guards' exposure at the re-anchor end is
   state-sensitive (the round-2 ledger's measure) — the fresh-boot probe (can a player hit a
   lobby guard standing d ~5-8 east of the re-anchor spot?) still decides which reading is the
   port's truth.
3. NEW: #6's charged-charge during the door phase — in lever-3 he ran from (3.4,−368.4,42.1)
   to the door mouth (−25.8,−368.4,40.5) across the drain's static window (~3400 ticks). Does a
   fresh-boot player standing at the door spot see the wave rifle cross the whole lobby north
   side, or does the port's pursuit radius [M 0x152548:174-181] stop him earlier? (The fights'
   two killed-true answers on the room legs suggest the charge is real.)

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed in this
environment; nothing to copy).

## ntfy failed

All three fallbacks failed. Exact outputs:

- curl: `curl: (28) Connection timed out after 20002 milliseconds`.
- python urllib: the `urlopen(..., timeout=20)` call hung; the wrapper `timeout 25` killed it
  (exit 124), no response printed.
- wget: hung the same way; the wrapper `timeout 25` killed it (exit 124), no output.
