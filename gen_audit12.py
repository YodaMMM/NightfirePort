#!/usr/bin/env python3
"""Generate docs/levels/07000012-audit.md from census12.txt (dump_07000012_main_census --ignored).
One row per main-map placement, grouped by kind: the game's behaviour [M/D], the port's, GAP."""
import re
from collections import OrderedDict

census = open("census12.txt").read().splitlines()

# kind -> (game [M/D], port, gap). The game chains are the decompile's (ACTION.ELF), verified per
# address in this session or carried from docs/levels/07000012.md §7 + the 0c-audit kind map.
K = {
 0x0f: ("Drone_Create 0x136be0 (gated: Drone_bDisableSystem==0 and p2<=difficulty [M 0x136be0:49]) -> NDrone2_CreateObj; per-frame NDrone2 AI + SP idle links; death writes p12 [M]. Params: p0 skin, p1 start script, p3 wait/gate ch, p4 route, p6 alarm link, p9/p10 look, p12 death ch, p13 special-death script, p14 route id, p15 dtype, p16 mode [M 0x163c68]",
        "drone.rs: full NDrone2 AI; the wait/gate/alarm/death params via the drone record; difficulty gate p2",
        "the drone HITTEST (the walk-through-Bond gap) — see the report's large-gap section"),
 0x19: ("static path 0x1d1998 only (flags 0x8019: the 0x8000 cel bit — the dispatcher skips the row [M 0x1d1b68]); no object, no update",
        "parse_cel_graph draws every cel; collision tris via world.rs", "none"),
 0x20: ("Break_Create 0x12c230 / Break_Update 0x12bfd8 — bullets subtract damage; kind 2 glass shatters (sound 0x2e [M 0x12b970]); param 2 = the weapon-class filter [M]",
        "breakable.rs (bounds via the object transform); live shot test red (the level doc §7)", "the glass live shot on this level — a test gap, not a behaviour gap"),
 0x24: ("Player_AddNewStartPos(1) 0x195c00, MP-gated [M 0x1d04f0:91-98]; +0x22 kind 1; the 0xA0 level_tag copy keeps p0 (the gate channel) [M 0x195c00:20-65]",
        "level_starts gate-scan at load -> pick_start -> Bond::stand_at", "none"),
 0x27: ("no case in parsemap_create_dynamic_objects [M 0x1d04f0] — the briefcase display frames ride the static/geometry path only",
        "model-resolve only (the model sweep pins resolution)", "none by data"),
 0x28: ("Sensor_Create 0x1af010; param 6 = the monitor group (0xa here [D]); Monitor_Activate reads the watched sensor's param-8 channel [M 0x1b0058]",
        "sensor.rs; group 0xa wired to the monitor; proven live (the level doc §7)", "none"),
 0x2a: ("View_AddSkyObj 0x1e79a0: slot=p0 (<21), GfxList[slot] write, animated byte p1, +0x4a=p4 [M 0x1e79a0:19-36]; the sky renderer consumes it",
        "sky_instances -> the static sky pass scene.rs:7737-7739", "the +0x4a p4=1 renderer consumer unported ([S] no visible delta; the 05/08 open item)"),
 0x2e: ("Lock_Create 0x1ae1f8 / Lock_Update 0x1ae3d8 — the Q-Decryptor beam drains the bolt; p5 = the OUT channel written on unlock (0xa here [D]); kind 0 = the keypad pair; p6 = 1 auto-unlocks on Operative [M 0x1ae1f8 tail]",
        "lock.rs; the burn proven through the real G cycle (decryptor_07000012_live: the 425-contact unlock, ch 10 on, the sisters)", "none"),
 0x2f: ("case 0x2f in 0x1d04f0 — the helicopter dressing object [D the row]; no gameplay channel wiring on this level [D params]",
        "model-resolve (the sweep)", "none by data"),
 0x30: ("Monitor_Create 0x1b08b0 / Monitor_Activate 0x1b0058 — E enters the camera view (state 2), writes the watched sensor's param-8 channel per tick; p1 = 1 -> shot-immune [M]",
        "monitor.rs; proven live (E enters the view, the group-0xa sensor watched)", "none"),
 0x31: ("flags 0x2031: the 0x2000 bit trips the dispatcher's `flags & 0xa000` skip [M 0x1d1b68] — parsemap_create_dynamic_objects NEVER runs for these rows; nothing is created (no FuseBox_Create despite the kind byte)",
        "the port's WorldObjects::create matches on `flags & 0xff` without the 0xa000 skip — it would build 15 inert param-less FuseBox objects the game never builds (invisible either way: no script hash -> SP_Create 0 [M 0x1b0448:25])",
        "cosmetic divergence, no observable (see the report's small-gap note)"),
 0x33: ("hint/text plates — text ids [D params]; no dynamic case reads them beyond the draw",
        "draw-side text; not simulated", "none by data"),
 0x35: ("PCQWorm_Create 0x1b07a8 / PCQWorm_Activate 0x1b0840 — E starts the install; PCQWorm_Update 0x1aee20 writes ch 0x34 (p1) and shows the keycode text [M]",
        "useobj.rs; proven live on the real walk (the hack-computer leg)", "none"),
 0x3a: ("Grapple_Create 0x198298 (class 0x51) — the Q-Claw target [M; gadget.rs pick_grapple_point]",
        "gadget.rs; no live drive on this level", "a live Q-Claw ride test (test gap)"),
 0x41: ("InverterTrigger_Create 0x1b3040 (case 0x41 [M 0x1d04f0:265-268]): p0 -> +0xF4, p1 -> +0xF6; InverterTrigger_Update 0x1b30a8: `switch_channels[in] && out!=0 -> switch_channels[out]=0` (the raw OUT-FIELD gate [M 0x1b30a8:8-11])",
        "worldobj.rs Inverter (the exact tick); proven live: after the console E, 0x87 drops (the level doc §7)", "none"),
 0x47: ("static path only (flags 0xc047 — the 0x8000 bit skips the dispatcher [M 0x1d1b68]); room/ambient/env class cels",
        "world geometry + collision", "none"),
 0x71: ("case 0x71 [M 0x1d04f0:284-289]: rotor_init(pos, celglist, 0, *(u8*)(+0x2c), *(u8*)(+0x30)) — the ROTOR kind; no updater in the case",
        "worldobj.rs Rotor (spin + gate)", "none"),
 0xd9: ("SP_CreateScriptPlayer 0x1db9a8 (p0 or p1 != 0 [M :12-18]) + SP_Update 0x1dbda8; modes: 1 play-at-once, 8 looping, 0x40 preloaded; the trigger channel p6 arms it [M]",
        "nis.rs Players + drone.rs script mount; all four scripts decoded (this audit's script section)", "the intro's visual check needs a GPU (a render task, not behaviour)"),
 0xdb: ("Door_Create 0x136100: p0 0=swing/else spline, p2 OUT-ch, p3 lock-ch — RISING EDGE = 'Door Unlocked' (label 0x2000003 [D door-text-labels.md]) + Door_Activate(...,0) + the lock ch cleared + the 0x10 latch for good [M 0x1363c8:88-112]; p5/p6/p7 = open/close/locked sound ids (0xffff = none; #115 p7=0x140 = the 320 locked thud); Door_Update 0x1363c8; auto-open proximity + E arms",
        "door.rs (the unlock edge + labels queue per door-text-labels.md; the sounds)", "doors #103/104/105 (lock ch 0xc8), #544 (0xc5), #115 (0xc8): NO writer of ch 0xc8/0xc5 exists on this level [D the writer scan] — inert by data. #754 unlocks on ch 0xa (the keypad burn) — proven (decryptor_07000012_live)"),
 0xe1: ("Destroy_Create 0x12c318: health=p3, blast scale p5*0.01, IN-ch p1, OUT-ch p0, bullet mask ~p4, type p2; Destroy_Smash 0x12bb60 + Explode_Create; Update 0x12c4f8 [M]",
        "destroyable.rs (the smash + the blast)", "none"),
 0xe2: ("SS_Create 0x1a9c58 (out ch p3, operate sound p4, mask p2, state 0x40 armed, wrap p1, spline p6); SS_Operate 0x1a9df8; SS_Update 0x1a9948: the 0x40 arm plays Sound_Play3D(p4) then latches 1 [M 0x1a9948:86-89]",
        "ss.rs (KeyFrame_Interp 0x1e1140); proven live: the call button swaps both shutter pairs", "none"),
 0xe5: ("Drone_CoverCornerNode 0x136ff0: appended to CoverNodes [M 0x136ff0:44-88]; consumed by the NDrone2 cover logic; no updater",
        "covers.rs (the cover nodes)", "none"),
 0xe6: ("Drone_CoverLowNode 0x137340: type-1 low cover (p0==2 -> flags 1, pose bytes from tables) [M 0x137340:64-87]",
        "covers.rs", "none"),
 0xe8: ("Trigger_LoadLevelCreate 0x1b2950: mask forced 0xE; p0 next level, p1 key gate, p2 gate, p3 fallback, p4 success [M]; Trigger_Activate 0x1b2318:91-111",
        "triggers.rs LOAD_LEVEL + the exit-hold (the 300-frame push [M 0x1b2318:90-92])", "none"),
 0xe9: ("Drone_AIPoint 0x137550 [M 0x137550:16-46]; AI-only data",
        "ai_points", "none"),
 0xea: ("Trigger_TouchOnce 0x1b2b88: slot 0 = type 1, slot 1 = OUT, slot 9 = MASK, slots 10/11 = p7/p8, then Trigger_Create [M 0x1b2b88:12-30]; Trigger_Update 0x1b1a78 case 1; the one-shot latch",
        "triggers.rs TOUCH_ONCE", "none"),
 0xeb: ("Trigger_Touch 0x1b2aa8 (type 2): out |= hit every frame while inside [M 0x1b1a78 case 2]",
        "triggers.rs TOUCH", "all 20 rows carry only param 7 = 0 — no OUT channel [D]: the beams write nothing on this level (inert by data)"),
 0xed: ("Trigger_MultiplexSeqIn 0x1b2b18 (type 4 = SEQ_AND of the input words -> out) [M]",
        "triggers.rs SEQ_AND", "the outputs 0x41/0x44 have no consumers found on this level [D the level doc §Port gaps] — inert by data"),
 0xf0: ("Pickup_Create 0x18dda0: type p0, item p1, amount p2, ch2 p3, sound p4, respawn p5, label p6 [M 0x18dda0:123-128]; TYPE-6 gated on PlrStats_HasGoldMedal [M]; Pickup_Handler 0x18e650 take",
        "pickup.rs; the Scorpion take proven live", "none"),
 0xf1: ("DroneSpawner_Create 0x1378d0 (gated p0>1): mode p0, group p1, slots p2, quota p3, enable p4, disable p5, done p6, radius p8 [M 0x1378d0:16-56]; DroneSpawner_Control 0x137e68",
        "spawners.rs; driven by the drone-alert tests", "none"),
 0xf9: ("SoundTrigger_Create 0x1b2628 (in p0, sample p2, mode p3, stop p4, global p5, difficulty gate p6, keep_on p7 [M 0x1b2628]); SoundTrigger_Update 0x1b2790 play/stop [M]",
        "sound.rs SoundTriggers", "none"),
 0xfa: ("ThirdIcon_Create 0x197070 (class 0x42); ThirdIcon_Update 0x196e80 writes the nearest enabled icon [M]",
        "icons via the use system", "none"),
}

def esc(s):
    return s.replace("|", "\\|")

out = []
out.append("# 07000012 — the behaviour audit (every main-map placement, script and record)\n")
out.append("Source rows: `dump_07000012_main_census` on the disc [D] (main map 38, 792 placements, 33 kinds; ")
out.append("the four ScriptPlayers' scripts decoded from the kind-7 entries). Game chains: ACTION.ELF decompile [M]. ")
out.append("Level context: docs/levels/07000012.md (the census §7 there is this audit's per-kind summary). ")
out.append("The dispatcher skip: any placement with `flags & 0xa000 != 0` never reaches ")
out.append("`parsemap_create_dynamic_objects` [M 0x1d1b68] — on this map that is kinds 0x19 (438), 0x47 (27) and ")
out.append("0x31 (15) only; every other row below is created by the game.\n")

cur = None
for line in census:
    if line.startswith("== KIND"):
        k = int(line.split()[2].rstrip(':'), 16)
        cur = k
        if k in K:
            g, p, gap = K[k]
            out.append(f"\n## Kind {k:#06x}\n")
            out.append(f"**Game [M/D]:** {g}  \n**Port:** {p}  \n**GAP:** {gap}\n")
            out.append("\n| # | name | @ | params | flags |")
            out.append("|---|---|---|---|---|")
        else:
            cur = None
            out.append(f"\n## Kind {k:#06x}\n\n(no audit row — see the header)\n")
    elif line.startswith("#") and cur in K:
        m = re.match(r"#(\d+) kind (0x[0-9a-f]{2,4})( CEL)? \"([^\"]*)\" @\(([-0-9.,]+)\) params \[([^\]]*)\] hash ([0-9a-f]+) flags (0x[0-9a-f]+)", line)
        if m:
            idx, name, at, params, flags = m.group(1), m.group(4), m.group(5), m.group(6), m.group(8)
            out.append(f"| {idx} | {esc(name)} | {at} | {esc(params)} | {flags} |")

open("docs/levels/07000012-audit.md", "w").write("\n".join(out) + "\n")
print("rows written:", sum(1 for l in out if l.startswith("| ") and not l.startswith("| #") and not l.startswith("|--")))
