
## level-06-start-subtitles — 2026-11-01

Task: at the 07000006 start the PS2 shows the driver dialogue subtitles ("Can't I rest a
moment?...", "Then it's six hours back, old man!...") [L]; the port shows none. First check the
port's subtitle option default against the game's [M the settings init / save default]; if the
default is on, find where the dialogue's subtitle lines come from [D the script streams / text
bank entries for that conversation; M the subtitle post path] and port it. Pin: the lines post at
the script's ticks with the option on, none with it off. Run the subtitle/script binaries you
touch and nf-game; quote the totals.

Session 1 (2026-11-01): branch from origin/master (3f760e7d); sandbox kept from the previous
task (repo, work/, ISO, nf-queue worktree). WORKLOG started.

Session 1 continued (2026-11-01): the symptom chased to its source. The two lines are the
07000006 gate conversation's driver exchange — USATxt labels 0x0600006e/0x0600006f [D], mapped by
Snd2Lbl (0x2d25d8 idx 78/79) to sfx 0xac/0xae with the 0x80000000 always-show bit [D]. They are
played by the speakers' START-SCRIPT anim sound events (kind-5 anims: driver #494 0x0600011d
op-1 frame 5 = 0xac; escort #450 0x0600011b op-1 frame 295 = 0xae) via AnimProcessScriptCmds
0x1c1b38 case 1's Sound_Play3D, and subtitled by Sound_DoSubtitle 0x1dde58 (not ported before).
No briefing FMV exists (0x07100006 is off the UsingMovie whitelist [M 0x107eb8]) and the fresh
Subtitles default is OFF everywhere ([L] ee_start_*.bin @0x2a37c8 = 0; [M] bootup 0x1c7e78:34,
the def-codename template byte 0 [D]) — the port's default was already right; the lines show on
the PS2 because the always-show rows bypass the option. Ported: Snd2Lbl + SFXOutputData+0x10
durations parses (nf-formats sfx.rs), the DoSubtitle hook on the drone anim-sound starts
(drone.rs), push_always (subtitle.rs, the id-0xff AddMsg bypass), the host attach
(lib.rs/scene.rs). Pinned: level06_start_subtitles_live (2) — the whole gate conversation posts
in script order through the FIFO with the option off (opener tick ≤3, driver <300, reply FIFO-
chained, durations 196/211/183/115/219 = 0.92×f10×60), sister: option ON leaves them unchanged.
