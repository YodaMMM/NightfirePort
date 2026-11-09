# 0700001b: console 1 on the 00 Agent walk

walk_arrival_to_drakes_defeat_on_real_health_00agent (walkreal_0700001b_live, ignored) fails 'console 1 latched by the real shot'. Find why the shot does not latch console 1 on 00 Agent (aim, range, the console's hit test, difficulty-gated scripts [D the console UI scripts]) and fix the port or the route.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-1b-console-latch-00agent`, report in docs/reports/level-1b-console-latch-00agent.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T02:23Z

---
## Report (done, 2026-11-09T03:05Z)


---
## Report from the worker (done, 2026-11-09T03:05Z)

# level-1b-console-latch-00agent

## What was done

Task: `walk_arrival_to_drakes_defeat_on_real_health_00agent` (walkreal_0700001b_live, ignored)
failed `console 1 latched by the real shot`. Find why the shot does not latch console 1 on
00 Agent and fix the port or the route. Branch `s/level-1b-console-latch-00agent` from
origin/master (316a14fa0; the sandbox was reset at session start, so the clone, the ISO, the
Rust toolchain and the target/ were all rebuilt from zero — the build identity is this session's).

### The red, reproduced on the real level through real input (run1)

The walk loaded 07000001b at difficulty 3 the menu way, fought the first wave from the hub, and
died at tick 6251 at (32.00,-1.70,0.00) — mid-console-phase. Consoles 5/4/3 had latched
(3093/4317/5511); the death read as `console 1 latched by the real shot` because the per-console
assert (line 941) scans k 1..8 and ch 1 was the first channel still off. The kill record: 18
hits for 100.0, **0 return-fire fights** — the ch-19 launch order (frame 1775 [D spawner #163
p6 = 194, AND #202]) fires MID-WAVE, five skin-0x5000078 astronauts (hp 40/37) keep their posts
at 6-19 u, and their 0x6a rounds (5.0 [D weapon_data +0x0c] × pain Plr_DMod_Hard 1.0 [M
Player_HandlePain 0x1902c8; 0x1d3a80 case 0x700001b]) land ~1 hit per 250 ticks through the
whole console phase. The wave defense in `shoot_consoles` only guards the tail group (k 1/2/7),
so the mid group ran undefended.

### The port-rule check (the decompile side)

The latch mechanism itself is difficulty-blind and matches the game: the console UI scripts
(gate 30+k, hit_points 20 [D SP#213-220]) hold with three 20-damage holds; the SP_Update damage
tail adds the tick's bullet damage into SP+0x18 and fires SP_UnPause at the threshold [M nis.rs
1957-1980 = 0x1dc1c0-0x1dc330 / SP_GetHitDamage 0x1dc5f8; SP_UnPause 0x1dca58]; the channel
write is stream 4 ev0x0d [D]. The bullet targets are the held script's glist spheres [M
held_hit_targets / sp_bullet_targets, nis.rs 2150/2207]. No difficulty gate anywhere on the
path — so the fix is the route, as the measured shape said.

### The route fix (all inside walkreal_0700001b_live.rs, d3-gated through a `hard` flag)

1. **Exposure-hedged station picks** — `level_station` gains `hard`; the candidate score adds
   the number of live astronauts whose muzzle ray reaches the stance's eye (×2.0 over the walk
   distance), and the default anchor is only taken when at most ONE shooter lines it and the
   stance is hurt-free. A player fighting through a hub whose gate has already fired takes the
   stance the fewest shooters can line.
2. **The bounded mid-phase defense** — when 2+ shooters line Bond (checked per tick), a
   `fight_opts` wave fight at the current position (waves_first cover discipline, 700-tick cap,
   1200-tick cooldown) breaks off and kills the nearest threat before the console work resumes.
3. **The hurt-free stance guard** — `hurt_free` re-runs the game's own contact law read-only
   (the feet..+1.8 r 0.55 capsule [M Player_DealWithObjHit 0x194458; Hurt_Update 0x17e5d0's
   disabled bit]) against the level's armed hurt volumes, so a hedged stance never trades lined
   muzzles for a 20.0 plate. Plus a one-time dump of the level's hurt volumes (the ledger below).
4. **The latch-stop** — the console fire gate stops the taps the tick the console's channel is
   on (d3-gated). Run5 proved it trajectory-neutral (byte-identical latches and death tick), so
   it is kept as documentation of the finding, not as a fix.

### Result: the console phase is GREEN

run3/run5 [L]: all 8 consoles latch in deadline order — k5 3667, k4 4322, k3 5531, k6 7085,
k8 7409, k1 9264, k2 9401, k7 10079 — health 39.3 at the k7 latch, the `console {k} latched by
the real shot` assert passes for every k, the astronaut fire reduced from 18 hits/100.0 to 2
hits/5.0.

## What's left (the walk is still red, downstream)

The disarm wait dies at tick 10424 at (46.01,-8.08,0.00) — the same zone as the old d2/d3
ledger deaths (10695). The killer is NOT the waves (5.0 total) and NOT the hurt volumes (the
dump: only two exist — kind 2 damage 50 at (-34.4,5.5,-0.9)..(-32.5,13.8,0.9) far west; kind 1
damage 50 gate 18 disabled at (8.5,9.0,-0.4)..(81.4,9.9,0.4); neither touches the stations).
It is **kind-107 debris blasts** (damage 20.0, radius 20.0, owner -2 [L 1b-run4.log
NF_DMG_TRACE]) detonating AT the screens Bond shoots: 8502 exactly on the x20 station
(20.2,-1.9,-0.2), 10424 exactly on the x46 nest (46.1,-7.8,-0.2), two point-blank deliveries
each; the fire ticks are exactly 1922 apart (4658 → 6580 → 8502 → 10424) and byte-identical
across run3/run4/run5. The next lever is the decompile-compare on the blast source: which
script fires the kind-107 explosion on the panel resumes (GAPS G-047: "the 0700001b solar
panels (param 7 = 20) resume and their stream 1-10 debris plays") and whether the game's blast
schedule/damage matches the port's. The full ledger is in the test's ignore message (updated
this session; the stale "dies in the Drake fight at tick 8426" note is replaced).

## How it was verified

Every run is the ignored test itself, whole, on the real ISO (`NF_ISO` at
Nightfire-onfoot.iso, `NF_SILENT=1`, `--test-threads=1`, `--nocapture`), real input from the
load — no teleports, no health/ammo/channel writes. run1 = the unfixed red; run3 = the fixed
console phase; run4 = NF_DMG_TRACE identifying the blast sources; run5 = the latch-stop
control. The changed file compiles clean and the binary is committed; no `crates/*/src/` file
changed, so no other level's walkreal binary is affected by this branch.

## Live checks for the main agent

1. The blast law: in PCSX2 at 00 Agent, shoot console 1's screen from the x20 station and watch
   the health trace for a 20.0 blast delivered at the screen (the expected tick shape: ~3
   resumes per console, some detonating). If the game's player takes the blast too, the route
   must dodge the 1922-tick fire ticks; if the game's player takes nothing, the port's
   debris-blast delivery is the port gap and the fix belongs in the debris/panel chain.
2. The kind-107 source: gdbwatch on the explosion object's spawn site while a screen resumes —
   expected: the creating script is the console UI script's post-latch stream (or the panel
   script's debris stream), which settles which [D] stream owns the blast.

## Usage

- Context window / tokens / cost: **not visible to me** — no token, credit or context figures
  are exposed anywhere in this environment; I cannot quote numbers I was never shown.
- Wall clock (from the session's own `date -u`): the driver message arrived ~00:53 UTC; the
  sandbox reset was found at 02:25 UTC (the previous session's tree was lost); setup redo
  02:25-02:42 (clone, downloads, rustup, ALSA prefix, warm build 54 s); run1 02:44; the fix
  landed 02:50-03:30; run3 03:04, run4 03:14, run5 03:33; this report written ~03:40 UTC,
  inside the ~04:24 UTC expiry.
