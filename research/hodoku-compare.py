"""Compares sumi's levels with HoDoKu 2.2.0's solving path (8 October 2026).

Inputs, in the working directory:
- all.txt: puzzles, one per line (the 1,000 bank puzzles, then
  `sumi gen -n 1000 --seed 5`)
- sumi.out: `sumi rate < all.txt`
- hodoku.out: `java -jar hodoku.jar /bs all.txt /vp /o hodoku.out`

Maps each HoDoKu step to sumi's level for the same technique, and reports
puzzles where HoDoKu needed a technique sumi doesn't have. Run with
`uv run hodoku-compare.py`. HoDoKu is GPL and used here as a tool only.
"""
import re
from collections import Counter, defaultdict
LV = {"Full House":0,"Hidden Single":0,"Naked Single":0,
      "Locked Candidates Type 1 (Pointing)":1,"Locked Candidates Type 2 (Claiming)":1,"Naked Pair":1,"Hidden Pair":1,"Locked Pair":1,
      "Naked Triple":2,"Hidden Triple":2,"Locked Triple":2,"Naked Quadruple":2,"Hidden Quadruple":2,"X-Wing":2,"Swordfish":2,
      "Skyscraper":3,"2-String Kite":3,"W-Wing":3,"XY-Wing":3,"XYZ-Wing":3,"Uniqueness Test 1":3,"Simple Colors Trap":3,"Simple Colors Wrap":3}
NAMES=["Easy","Medium","Hard","Expert","Beyond"]
sumi={}
for l in open("sumi.out"):
    w=l.split(); sumi[w[0]]=(w[1], " ".join(w[4:]))
cur=None; steps=defaultdict(list); hlevel={}
for l in open("hodoku.out"):
    m=re.match(r"^(\S{81}) #\d+ (\w+)", l)
    if m: cur=m.group(1); hlevel[cur]=m.group(2); continue
    if cur and l.startswith("   "):
        steps[cur].append(re.split(r":| =>", l.strip())[0].strip())
puzzles=[l.strip() for l in open("all.txt")]
agree=Counter(); outside=Counter(); conf=Counter(); hl=Counter()
for i,p in enumerate(puzzles):
    src = "bank" if i < 1000 else "gen"
    key = p.replace("0",".")
    s_level, s_tech = sumi[key]
    st=steps[key]
    out=[t for t in st if t not in LV]
    hl[(src, s_level, hlevel.get(key))]+=1
    if out:
        outside[(src, s_level)] += 1
        for t in set(out): conf[t]+=1
        continue
    h = NAMES[max((LV[t] for t in st), default=0)]
    agree[(src, s_level == h)] += 1
    if s_level != h: print("DIFF", src, key, s_level, s_tech, "| hodoku", h, sorted(set(st), key=lambda t: LV[t])[-1:])
print("same-technique-set puzzles, level agrees:", dict(agree))
print("used a technique outside sumi's set:", dict(outside))
print("outside techniques:", conf.most_common(12))
print("hodoku level by sumi level:")
for k,v in sorted(hl.items()): print("  ",k,v)
