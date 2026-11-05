# guard-damage-since-1017 — Bond loses far more health standing still in some spots

Standing 10 seconds on 07000006 costs 188 health and on 07000011 costs 180, against 6.6 and 10 on a 10-17 build; other stands barely moved. Bisect master from 10-17 for the commit or commits that changed it, using the same stand at default difficulty. Read each against the decompile [M]: guard hit roll, range falloff, weapon damage, difficulty scale. Fix anything that differs and pin the numbers. If the current behaviour is right, say so with the addresses.
