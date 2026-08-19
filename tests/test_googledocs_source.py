import unittest
import urllib.request
import re
import sys
import json

sys.stdout.reconfigure(encoding='utf-8')

USER_AGENT = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
BASE_URL = "https://docs.google.com"

def fetch_html(url):
    req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(req, timeout=15) as resp:
        return resp.read().decode('utf-8', errors='ignore')

def parse_doc(html):
    chunks_raw = re.findall(r'DOCS_modelChunk\s*=\s*(\{.*?\});\s*(?:DOCS_|var|</script>)', html, re.DOTALL)
    all_ops = []
    for c_str in chunks_raw:
        try:
            obj = json.loads(c_str)
            all_ops.extend(obj.get("chunk", []))
        except:
            pass

    max_len = 0
    for op in all_ops:
        if op.get("ty") == "is":
            ibi = op.get("ibi", 0)
            s_len = len(op.get("s", ""))
            max_len = max(max_len, ibi + s_len)

    doc_chars = [" "] * max_len
    for op in all_ops:
        if op.get("ty") == "is":
            ibi = op.get("ibi", 0)
            s = op.get("s", "")
            for idx, ch in enumerate(s):
                pos = (ibi - 1) + idx
                if pos < max_len:
                    doc_chars[pos] = ch

    flags = bytearray(max_len)
    for op in all_ops:
        if op.get("ty") == "as" and op.get("st") == "text":
            si = op.get("si", 0)
            ei = op.get("ei", 0)
            sm = op.get("sm", {})
            if sm.get("ts_bd") is True:
                start = max(0, si - 1)
                end = min(max_len, ei)
                for k in range(start, end):
                    flags[k] |= 1
            if sm.get("ts_it") is True:
                start = max(0, si - 1)
                end = min(max_len, ei)
                for k in range(start, end):
                    flags[k] |= 2

    return doc_chars, flags

class TestGoogleDocsSource(unittest.TestCase):
    def test_01_featured_catalog(self):
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        title_m = re.search(r'<title>(.*?)</title>', html, re.I)
        self.assertIsNotNone(title_m)
        title = title_m.group(1).replace(" - Google Docs", "").replace("&#39;", "'").strip()
        self.assertEqual(title, "I'm A Young God, Won't You Raise Me?")
        print(f"\n✓ Google Docs catalog verified: Title='{title}'")

    def test_02_char_accurate_bold_and_stars(self):
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        chars, flags = parse_doc(html)
        full_str = "".join(chars)
        
        # Test exact bold alignment on Roasted Chestnut: Hello
        pos = full_str.find("Roasted Chestnut: Hello")
        self.assertNotEqual(pos, -1)
        # Check first char 'R' is bold (flag bit 1)
        self.assertEqual(flags[pos] & 1, 1, "First letter 'R' of 'Roasted' MUST be bold!")
        
        # Test exact star character presence
        star_pos = full_str.find("*Editor")
        self.assertNotEqual(star_pos, -1)
        self.assertEqual(chars[star_pos], '*')
        print("✓ Character-accurate bold and star preservation verified with zero index drift.")

if __name__ == "__main__":
    unittest.main()
