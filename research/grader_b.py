import sys, grader
from grader import *
grader.ORDER[:] = [
    ("Full House", 0, full_house), ("Naked Single", 0, naked_single), ("Hidden Single", 0, hidden_single),
    ("Locked Candidates", 1, locked), ("Naked Pair", 1, naked(2)), ("Hidden Pair", 1, hidden(2)),
    ("Naked Triple", 2, naked(3)), ("Hidden Triple", 2, hidden(3)), ("X-Wing", 2, fish(2)),
    ("Naked Quad", 2, naked(4)), ("Swordfish", 2, fish(3)), ("Hidden Quad", 2, hidden(4)),
    ("Skyscraper", 3, skyscraper), ("2-String Kite", 3, kite), ("W-Wing", 3, wwing), ("XY-Wing", 3, xywing),
    ("XYZ-Wing", 3, xyzwing), ("UR Type 1", 3, ur1), ("Simple Colouring", 3, colouring),
]
grader.IDX.clear(); grader.IDX.update({n: k for k, (n, _, _) in enumerate(grader.ORDER)})
if __name__ == "__main__":
    sys.argv = ["grader.py"] + sys.argv[1:]
    exec(open("grader.py").read().split('if __name__ == "__main__":')[1].replace("\n    ", "\n"))
