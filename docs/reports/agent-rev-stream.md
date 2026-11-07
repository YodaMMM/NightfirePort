# agent-rev-stream

Two small repairs found in review, stacked on origin/master + s/guard-alt-voice-set + s/aim-15-agent. This branch moves the shared random stream on 14/15/16 through the guard-alt-voice-set merge.

- `walk_anim`: the game adds 1 to the +0x108 serial before the %3 [M 0x13cc70, disasm 0x13ccd0]. New test `walk_variant_adds_one_to_the_serial`.
- `Door::bullet_pass`: reads the glist word `f0 & 0x60` [M 0x12cf18]. The placement word is the door kind 0xdb, whose 0x40 bit made every door pass rounds. New test `door_object_pass_word_live`.
- Comment addresses corrected: the serial counter is 0x299090, the drone list 0x298e14.

Open: the walkreal_07000016 legs and walkreal_07000014's delta-exit real-damage leg are red on this stack. See the review files.
