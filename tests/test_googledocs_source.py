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

def extract_doc_text(html):
    pattern = '"s":"'
    pos = 0
    full_text = []
    while True:
        idx = html[pos:].find(pattern)
        if idx == -1:
            break
        abs_start = pos + idx + len(pattern)
        sub = html[abs_start:]
        in_escape = False
        end_idx = len(sub)
        for byte_idx, c in enumerate(sub):
            if in_escape:
                in_escape = False
            elif c == '\\':
                in_escape = True
            elif c == '"':
                end_idx = byte_idx
                break
        raw_val = sub[:end_idx]
        try:
            val = json.loads(f'"{raw_val}"')
            full_text.append(val)
        except Exception:
            full_text.append(raw_val.replace('\\n', '\n').replace('\\"', '"'))
        pos = abs_start + end_idx + 1
    return "".join(full_text)

def find_chapter_boundaries(text):
    marker_positions = []
    lines = text.split('\n')
    cur_pos = 0
    for line in lines:
        stripped = line.strip().lstrip('\x0c')
        if re.match(r'^(?:Chapter|CHAPTER|chapter|Episode|EPISODE|Ch\.)\s+\d+', stripped):
            marker_positions.append(cur_pos)
        cur_pos += len(line) + 1 # include newline

    if not marker_positions:
        return [(1, "Full Document", 0, len(text))]

    chapters = []
    for idx, start in enumerate(marker_positions):
        end = marker_positions[idx + 1] if idx + 1 < len(marker_positions) else len(text)
        slice_text = text[start:end]
        first_line = slice_text.strip().split('\n')[0].lstrip('\x0c').strip()
        title = first_line if first_line else f"Chapter {idx + 1}"
        chapters.append((idx + 1, title, start, end))
    return chapters

class TestGoogleDocsSource(unittest.TestCase):
    def test_01_featured_catalog(self):
        """Test featured Google Doc catalog entry."""
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        title_m = re.search(r'<title>(.*?)</title>', html, re.I)
        self.assertIsNotNone(title_m)
        title = title_m.group(1).replace(" - Google Docs", "").replace("&#39;", "'").strip()
        self.assertEqual(title, "I'm A Young God, Won't You Raise Me?")
        print(f"\n✓ Google Docs catalog verified: Title='{title}'")

    def test_02_doc_text_extraction(self):
        """Test extracting text model chunks from Google Doc."""
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        text = extract_doc_text(html)
        self.assertGreater(len(text), 1000000)
        self.assertIn("Han Goyo", text)
        print(f"✓ Google Docs text extraction verified: Total {len(text)} characters parsed.")

    def test_03_chapter_splitting(self):
        """Test splitting multi-chapter novel into chapters."""
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        text = extract_doc_text(html)
        chapters = find_chapter_boundaries(text)
        self.assertGreaterEqual(len(chapters), 100)
        print(f"✓ Google Docs chapter splitting verified: {len(chapters)} chapters identified. First='{chapters[0][1]}', Last='{chapters[-1][1]}'")

    def test_04_chapter_content(self):
        """Test extracting a specific chapter's content."""
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        text = extract_doc_text(html)
        chapters = find_chapter_boundaries(text)
        ch1 = chapters[0]
        content = text[ch1[2]:ch1[3]].replace('\x0c', '\n\n').strip()
        self.assertGreater(len(content), 5000)
        self.assertIn("Chapter 01", content)
        print(f"✓ Google Docs chapter content verified: {len(content)} characters in Chapter 1. First 60 chars: '{content[:60]}...'")

if __name__ == "__main__":
    unittest.main()
