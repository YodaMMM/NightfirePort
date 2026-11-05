# Aim snap: finish on the corrected filter

Branch from `origin/s/aim-snap-placement`, push to `s/aim-snap-final`. Read docs/reports/aim-snap-objects.md first, but note its rocket and laser results are void: `aim_pick` read object flags, which never carry 0x8000, so every static surface was skipped and the snap never landed. `s/aim-snap-placement` reads `inst.placement.flags` instead.

Now green on that tip: `firing_along_the_walkway_breaks_no_panel`, with the ray stopping at x 6.958338, the game's own point. [L PCSX2 breakpoint at 0x1a4d90: head (16.741, -1.137, 4.62), hit (6.958, -1.137, 4.62), object 0.]

Red on that tip, each to be fixed by matching the game's ray, not by turning the snap off:
- rocket_glass08_live (exploded at z -96.706, must pass the window plane -96.75), rocket_stop_plane08_live if red. Check the cel walk through the 08 window and the glass mask.
- interactables_07000014_live and fusebox_geometry_07000014_live: the laser must cut. The game's ray hits object glists, so the wire needs to be in the ray.
- rocket_origin_live against the true snapped point.

Then check walkreal_0700000c_live, walkreal_07000014_live, walkreal_07000016_live, and walkreal_0700001b_live's consoles from spots a player would use. Strip the NF_AIMPROBE prints.

Use a PCSX2-backed number wherever one exists. Run each binary whole and quote the results.

---
claimed-by: sandbox5 at 2026-11-05T21:54Z
