#!/usr/bin/env python3
"""Check or regenerate Studio-local maturity counts from its capability rows."""
import argparse
from collections import Counter
from pathlib import Path
import re
p = argparse.ArgumentParser()
p.add_argument('--write', action='store_true')
a = p.parse_args()
path = Path(__file__).resolve().parents[2] / 'studio/ROADMAP.md'
text = path.read_text()
rows = re.findall(r'^\| (S[A-Z]\d\d) \| ([🟢🟡🔴⚪]) (ESTABLISHED|PARTIAL|OPEN|LATER) \|', text, re.M)
assert rows and len({row[0] for row in rows}) == len(rows), 'Duplicate or absent capability IDs'
expected = {'🟢':'ESTABLISHED','🟡':'PARTIAL','🔴':'OPEN','⚪':'LATER'}
assert all(expected[mark] == status for _, mark, status in rows)
counts = Counter(status for _, _, status in rows)
body = 'Generated from the capability board: ' + ' · '.join(f'{mark} {status} **{counts[status]}**' for mark, status in expected.items()) + f' · **{len(rows)} properties**.\nRegenerate: `python3 tools/validation/check_studio_roadmap.py --write`.\n'
start, end = '<!-- maturity-counts:start -->', '<!-- maturity-counts:end -->'
new = text[:text.index(start)+len(start)] + '\n' + body + text[text.index(end):]
if a.write: path.write_text(new)
else: assert text == new, 'Studio roadmap counts are stale; regenerate them'
print(f'Studio capability counts: {dict(counts)}; total={len(rows)}; inventory_not_completion_percentage=true')
