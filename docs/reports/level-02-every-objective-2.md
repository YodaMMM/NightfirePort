# level-02-every-objective-2 — report

Task: list every stand-in, position write, health top-up or channel poke `walkreal_07000002_live`
(and the shared walker helpers it uses) still carries on 07000002, and replace each with the
real-input leg, fixing any port gap the real leg hits. Branch `s/level-02-every-objective-2`,
from origin/master 531c08fb.

## The stand-in inventory (the starting state)

1. **`clear_captors` teleported Bond beside every captor** — `sim.bond.pos = [at[0] + 2.5, at[1],
   at[2] + 0.4]`, run twice (geisha 1, geisha 2). A position write.
2. **The ch-0x17 plate tr[003] raise was hand-set** — after the walker wedged, the test ran
   `channels.set(0x17, 1, frame)`. A channel poke.
3. **The geisha-1 touch zone 0x3C (tr[032]) was hand-set** — same pattern. A channel poke.
4. **The ch-0x19 plate tr[009] raise was hand-set** — same pattern. A channel poke.
5. **The walker's `top_up_health` default (true)** floored Bond's health at 100 every tick — the
   walk crossed the whole castle under god-mode. A health top-up (walker default, this test never
   turned it off).

## What replaced them (all on the branch)

- **The three pokes are GONE.** `probe02quads` (new, on the branch) dumped each quad's triangles,
  ran approach-line segment checks and a teleport experiment: **the port fires ALL THREE quads
  when Bond really stands on them** (`TELE 0x17/0x3C/0x19 = true` — [L probe02quads run
  2026-10-29]). The pokes were harness routing, not port gaps:
  - 0x17: the old leg approached on the west-wing-south diagonal (blocked at t=0.53 [L probe
    SEG]); the north line is clear [L probe SEG None]. The new leg walks north-of-plate then
    THROUGH the quad (target z 4.6, inside x −35.91..−34.52 / z 3.26..6.85 [L probe tris]).
  - 0x3C: the old arrive-1.2 stop at (−45.50, 5.85, 10.53) sat in a notch the volume's
    staircase slabs do not cover — no tri within the capsule's reach there, and the port's
    containment branch is bounded by the tris' own AABB (y ≤ 6.34) [L probe tris + run]. The new
    leg walks to the measured centre (−45.4, 5.9, 11.7), arrive 0.6, where standing raises the
    channel [L probe TELE]. `Trigger_Calc`'s clear-then-set loop is [M 0x1b1900 + the port's
    calc()], so the assert runs while Bond stands inside — it did not rise in the baseline run
    because Bond stood 1.2 u off the measured centre in that notch.
  - 0x19: the plate is IN the doorway (quad x −27.58..−26.19 [L probe tris]); the old arrive-1.2
    stop at x −28.09 stood 0.5 short of the quad's west edge [L base02.log]. The new leg walks
    THROUGH the doorway to x −26.3 (the west line is clear [L probe SEG None]).
- **`clear_captors` walks now**: it walks the nav graph to a vantage 3 u from the captor on the
  Bond→captor line, then fires — the same real fire path as before, no position write.
- **`top_up_health = false`** for this walk (and for the 07000003 hand-off walker). The walk takes
  the guards' fire through the real pipeline.
- **Real fight legs** (`clear_pursuers` + `loot`, on the branch): the ch-0x15 garden group (5
  guards [D docs/levels/07000002.md]) chases the crossing, the ch-0x81 ambush group (3 [D]) holds
  the bunker corridor — the walk engages them with real walk+fire, and on a dry gun walks to the
  nearest live pickup (a drop or a placed box) and takes it with Bond's own capsule, then cycles
  the weapon wheel. All real input.

## Where the walk stands (honest)

The baseline walk (all five stand-ins active) was green: 50.66 s / 32107 ticks (run 2026-10-29).
With the stand-ins removed the walk is **`#[ignore]`d with its exact blockers** — the REAL-health
run fails deterministically, and I am not re-adding stand-ins to fake it green:

1. **The south-hall-west nav-10 wedge is a guard body-block, not geometry.** The wedged spot
   (−29.4, 6.4, 18.4) has a CLEAR segment to the leg target [L fail_dump segment None]; the alarm
   trace shows guard g3 standing 0.8 u from Bond there, running 0x74 → 0x68 → **0x60** at
   t4137–4184 (NF_ALARM_TRACE, run06). ~1462 ticks into the wedge `level_exit` fires in EVERY
   real-health run (1836/1462/1462 — the auto-restart of the fail chain; `Drones::auto_restart`
   answers the fail menu the walker never does [M scene.rs 5600 → `fail_choose_restart` 0x216618]).
   The walker's stuck ladder sidesteps walls and opens doors but has **no drone-body-block
   response**. Fix options for the next window: a body-block sidestep in the ladder, or clearing
   g3 before the leg (he closes from beyond the 24 u sweep during it).
2. **The ammo economy cannot feed the fights.** 48 start rounds vs ~13 armed guards on the route;
   two kills emptied the gun. The drops exist but go untaken — one rests at **y 0.0, through the
   floor** (the looting line `looting (-21.4, 0.0, 29.3)`, run08 — a port drop-placement gap), and
   the reachable ones sit below the walk level where the arrive-0.4 take still did not grant
   (run07). This is the port gap to fix next: the drop placement/lifecycle.

The port-side machinery needed by the real legs was verified working (probe TELE); **no nf-game
source change was needed by the quads themselves**. The remaining work is the two blockers above,
then un-ignore.

## How it was verified

- `walkreal_07000002_live`: baseline (master tree) **1 passed / 0 failed, 50.66 s, 32107 ticks**;
  the branch tree = 0 passed / 0 failed / **1 ignored** (the blocker text above).
- `probe02quads` (new diagnostic, --nocapture): the three quads' tris + SEG/NAV/TELE lines quoted
  above; green.
- The four crates' release test binaries ALL build clean (`cargo test --no-run` over
  nf-game/nf-formats/nf-audio/nf-view, 0 errors, target-cache ~3.6 GB).
- **The full-suite total line: not available this window.** The fast runner (`tools/test/suite.py`)
  was started in the background per the task and was killed between tool calls (this sandbox kills
  background jobs — the level-02-every-objective report documented the same); the foreground
  re-builds each resumed incrementally, but the run phase did not fit after the setup overruns
  (no cargo in the sandbox → rustup install; alsa-sys needed a user-prefix ALSA dev extraction
  with a fixed alsa.pc prefix and a libasound.so symlink — the stock dev .pc points at /usr and
  its .so symlink is dangling). The touched binary (`walkreal_07000002_live`) and the new probe
  were run directly and are quoted above; the next window should run `tools/test/chunk_suite.py`
  to ALL DONE on the warm cache.

## Live checks for the main agent

- **The 0x3C notch.** Stand Bond at (−45.50, 5.85, 10.53) on a fresh 07000002 (the baseline
  arrival point) and watch `switch_channels[0x3C]` = 0x0026fcdd... (base 0x0026fc90 [M symtab] +
  0x3c), byte, write access. Expected per my port reading: NO write (the notch is outside every
  tri's reach) — if the EE writes 1 there, the game's contact is the glist BOUNDS (not the tris)
  and the port's containment branch needs widening to match.
- **Guard g3's fail chain.** Watch g3's state word during the south-hall-west approach; confirm
  what state 0x60 drives on the EE (the fail channel raise — the +1462 auto-restart's trigger) so
  the next fix cites the real chain rather than my inference.

## What's left

- Unblock the real-health walk (the two blockers above), un-ignore, and re-run.
- The full-suite total line via chunk_suite on the warm cache.
- The walk must end green with NO stand-ins — the legs on the branch are the real route; only the
  two blockers stand between it and green.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the only
proxies I can see are wall-clock timestamps in the shell — setup 21:59–22:45 UTC, the work runs
22:50–23:55 UTC — and each tool call's own duration).
