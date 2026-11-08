#!/usr/bin/env python3
"""Build a Unicode 17 catalogue from the bundled font cmaps and UnicodeData.txt.

Run: python3 scripts/unicode-candidates.py /path/to/UnicodeData.txt
Requires fontTools. UnicodeData 17: https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt
"""
import struct, sys
from pathlib import Path
from fontTools.ttLib import TTFont
root = Path(__file__).resolve().parent.parent
low = TTFont(root / 'assets/Unifont-17.0.05.otf').getBestCmap()
high = TTFont(root / 'assets/UnifontUpper-17.0.05.otf').getBestCmap()
unifont = {p for p in low if p <= 0xffff} | {p for p in high if p >= 0x10000}
han = set(TTFont(root / 'assets/BabelStoneHan.ttf').getBestCmap())
extra_fonts = ['NotoSansArabic-Regular.ttf', 'NotoSansArmenian-Regular.ttf', 'NotoSansBamum-Regular.ttf', 'NotoSansBengali-Regular.ttf', 'NotoSansCanadianAboriginal-Regular.ttf', 'NotoSansCherokee-Regular.ttf', 'NotoSansCuneiform-Regular.ttf', 'NotoSansDevanagari-Regular.ttf', 'NotoSansEgyptianHieroglyphs-Regular.ttf', 'NotoSansEthiopic-Regular.ttf', 'NotoSansGeorgian-Regular.ttf', 'NotoSansGlagolitic-Regular.ttf', 'NotoSansGothic-Regular.ttf', 'NotoSansGujarati-Regular.ttf', 'NotoSansGurmukhi-Regular.ttf', 'NotoSansHebrew-Regular.ttf', 'NotoSansKhmer-Regular.ttf', 'NotoSansLao-Regular.ttf', 'NotoSansLinearB-Regular.ttf', 'NotoSansMalayalam-Regular.ttf', 'NotoSansMongolian-Regular.ttf', 'NotoSansMyanmar-Regular.ttf', 'NotoSansOldItalic-Regular.ttf', 'NotoSansOldPersian-Regular.ttf', 'NotoSansOsmanya-Regular.ttf', 'NotoSansRunic-Regular.ttf', 'NotoSansSinhala-Regular.ttf', 'NotoSansSyriac-Regular.ttf', 'NotoSansTamil-Regular.ttf', 'NotoSansThaana-Regular.ttf', 'NotoSansThai-Regular.ttf', 'NotoSansUgaritic-Regular.ttf', 'NotoSansYi-Regular.ttf']
extra_cmaps = [TTFont(root / 'assets' / filename).getBestCmap() for filename in extra_fonts]
covered = unifont | han | set().union(*(set(cmap) for cmap in extra_cmaps))
excluded = {'Cc','Cf','Cs','Co','Cn','Zs','Zl','Zp','Mn','Mc','Me'}
assigned = set()
start = None
for line in Path(sys.argv[1]).read_text(encoding='utf-8').splitlines():
    fields = line.split(';'); cp = int(fields[0], 16); name = fields[1]; category = fields[2]
    if name.endswith(', First>'):
        start = (cp, category)
    elif name.endswith(', Last>'):
        if start and start[1] not in excluded:
            assigned.update(range(start[0], cp + 1))
        start = None
    elif category not in excluded:
        assigned.add(cp)
points = sorted(p for p in (covered & assigned) if not 0xfff0 <= p <= 0xffff)
(root / 'assets/unifont-cmap.bin').write_bytes(b''.join(struct.pack('<I',p) for p in points))
print(f'{len(points)} covered assigned standalone code points')

# Record preferred outline fonts, including BabelStone Han; Unifont is fallback.
# The numeric font choice is read by the browser frontend.
font_names = ['NotoSans-Regular.ttf', 'NotoSansSymbols-Regular.ttf', 'NotoSansSymbols2-Regular.ttf', 'NotoSansMath-Regular.ttf'] + extra_fonts + ['BabelStoneHan.ttf']
choice = {}
for index, filename in enumerate(font_names, 1):
    for cp in TTFont(root / 'assets' / filename).getBestCmap():
        if cp in covered and cp in assigned and not 0xfff0 <= cp <= 0xffff:
            choice.setdefault(cp, index)
(root / 'assets/noto-cmap.bin').write_bytes(b''.join(struct.pack('<IB',cp,i) for cp,i in sorted(choice.items())))
print(f'{len(choice)} have preferred outline fonts; {sum(i == len(font_names) for i in choice.values())} use BabelStone Han')
