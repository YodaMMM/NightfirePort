
## level-0a-low-beam-ride-body-2 (2026-11-03)

Task: the standing beam fires (live-confirmed) without stalling 05's truck. [L 2026-11-02
fresh boot] on 0700000a Bond walking STANDING across low beam #621's line ((-12.4, 0.608,
-31.0) south to z -33.78) fires ch 0x24 in the real game — walkreal_0700000a_live::
low_beam_621_fires_standing's expectation is the game's; the port's standing toucher (the
measured box pos-0.05..pos+1.26, r 0.55) is too high. Both earlier attempts stalled
fix_truck_live when swapping in the game's standing body (Player_Collision 0x194df0 default
arm + Intersect_CylGeom 0x109da0) — 07000005's ch-0x70 park plate fired early / the truck
chain never started. Do: (1) find Bond's substate/attachment during the 05 truck ride and
which Player_Collision arm (or none — the cut-scene's disabled window) runs then [M]; build
the trigger toucher per substate; (2) both green together (low_beam_621_fires_standing
un-ignored + both fix_truck_live tests) and every interactables_*_live + walkreal_*_live;
(3) the full suite. Shared helper changes opt-in per level, every walkreal re-run.

Branch cut from origin/master 6f16146e. Sandbox carried from the previous session; the
pending level-12 finish steps (queue attach + ntfy-failed report) closed first.

Session result (02:25 UTC): the law work landed, the code did not — both prior attempts'
blocker is now precisely named and the parity is green on the revert. [M] The arms table is
settled from 00194df0_Player_Collision: the default arm (the walk substate AND the vehicle
take-control) is r 0.55, A = pos+up·0.275, B = pos+up·(0.55−cc) [:315-333] with cc = the anim
parent's obj+0xcc ROOT HEIGHT [:86] — standing 1.0347183 [L trace], crouched 0.55 — so the
standing capsule bottoms AT THE FEET (pos−1.0347): 0a's #621 band y 0.13-0.39 [D] is inside a
standing body's reach (pos 1.13 → bottom 0.095) and the firing needs no sphere; the old test
doc's "r 0.275" misread 0x3e8ccccd (A's offset) for the radius. [M] The game's trigger contact
gate read from 001eae80_Collide_PickObj: by contact type — 0x200 the toucher SEGMENT vs the
object's world sphere (obj+0x80 centre / obj+0x8c radius) [:93-107], 0x800 the same at r×1.5 +
the toucher radius [:152-166], 0x100/0x400/0x1000 sphere-vs-sphere sums [:226-228]; the world
sphere = the glist's LOCAL sphere through the obj matrix [M Control_BuildWorldSph 0x133070:33-49
— radius × |obj+0xe8|], the glist-less default (translation, 0.1) [:23-30]. Both were
IMPLEMENTED (the Trigger.sphere field fed from the parsed entity-params bounds in build() and
a bbox-sphere stand-in at the script-trigger site, plus the standing-toucher swap to
player.body) and MEASURED: the sphere gate alone → fix_truck 0/2 (the SCRIPT-made triggers —
the ch-0x70 park plate is script stream 6 [D] — fire the standing plate through their bbox
spheres); the arm swap + the spheres → fix_truck 0/2 (the standing plate fires through the
feet-level capsule — the plate 0.46 above the bed floor is inside the arm's reach — and the
ride's chain opener needs the old box's upper reach). Reconcile gap, now precise: the game's
contact must fire #621 standing (the feet-level capsule), keep #607 silent crouched (the
crouch arm's volume [0.99, 2.29] contains #607's 1.49-1.76 band yet the game stays silent
[L 2026-11-01]) and (per the not-live-verified sister) keep the standing ride off the park
plate — no capsule/AND/OR/sphere combination satisfies all three, so the refine step between
the sphere pick and Trigger_Hit's contact list is the missing law; three sharpened live checks
filed in the report. Reverted to parity: fix_truck 2/2, walkreal_0700000a_live 10 passed /
3 ignored; the sphere dump probe committed (probe_sphere12.rs: #621/#607 r 2.63, the 05 spawn
plate 36 r 1.56, the ch-0x22 gatehouse volume r 11.01).
