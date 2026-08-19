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

def extract_doc_text_and_styles(html):
    chunks_raw = re.findall(r'DOCS_modelChunk\s*=\s*(\{.*?\});\s*(?:DOCS_|var|</script>)', html, re.DOTALL)
    all_ops = []
    for c_str in chunks_raw:
        try:
            obj = json.loads(c_str)
            all_ops.extend(obj.get("chunk", []))
        except:
            pass

    is_ops = [op for op in all_ops if op.get("ty") == "is"]
    full_text = "".join([op.get("s", "") for op in is_ops])
    flags = bytearray(len(full_text))

    for op in all_ops:
        if op.get("ty") == "as" and op.get("st") == "text":
            si = op.get("si", 0)
            ei = op.get("ei", 0)
            sm = op.get("sm", {})
            if sm.get("ts_bd") is True:
                start = max(0, si - 1)
                end = min(len(full_text), ei)
                for k in range(start, end):
                    flags[k] |= 1
            if sm.get("ts_it") is True:
                start = max(0, si - 1)
                end = min(len(full_text), ei)
                for k in range(start, end):
                    flags[k] |= 2

    return full_text, flags

def find_chapter_boundaries(text):
    marker_positions = []
    lines = text.split('\n')
    cur_pos = 0
    for line in lines:
        stripped = line.strip().lstrip('\x0c')
        if re.match(r'^(?:Chapter|CHAPTER|chapter|Episode|EPISODE|Ch\.)\s+\d+', stripped):
            marker_positions.append(cur_pos)
        cur_pos += len(line) + 1

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

def build_chapter_markdown(text, flags, start, end):
    slice_text = text[start:end]
    lines = slice_text.split('\n')
    formatted_paragraphs = []
    
    offset = start
    for line in lines:
        stripped = line.strip()
        if not stripped:
            offset += len(line) + 1
            continue
            
        if stripped == '\x0c':
            formatted_paragraphs.append('---')
            offset += len(line) + 1
            continue
            
        clean_line = line.lstrip('\x0c')
        if line.startswith('\x0c') and formatted_paragraphs:
            formatted_paragraphs.append('---')
            
        if re.match(r'^(?:Chapter|CHAPTER|chapter|Episode|EPISODE|Ch\.)\s+\d+', clean_line.strip()):
            formatted_paragraphs.append(f"## {clean_line.strip()}")
            offset += len(line) + 1
            continue
            
        line_start = offset + (len(line) - len(clean_line))
        line_flags = flags[line_start:line_start + len(clean_line)]
        
        p_out = []
        in_bold = False
        in_italic = False
        
        for idx, ch in enumerate(clean_line):
            fl = line_flags[idx] if idx < len(line_flags) else 0
            is_b = bool(fl & 1)
            is_i = bool(fl & 2)
            
            if is_b != in_bold:
                p_out.append('**')
                in_bold = is_b
                
            if is_i != in_italic:
                p_out.append('*')
                in_italic = is_i
                
            p_out.append(ch)
            
        if in_italic:
            p_out.append('*')
        if in_bold:
            p_out.append('**')
            
        para_text = "".join(p_out).strip()
        if para_text:
            formatted_paragraphs.append(para_text)
            
        offset += len(line) + 1
        
    return "\n\n".join(formatted_paragraphs)

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

    def test_02_doc_text_and_styles(self):
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        text, flags = extract_doc_text_and_styles(html)
        self.assertGreater(len(text), 1000000)
        bold_count = sum(1 for f in flags if f & 1)
        italic_count = sum(1 for f in flags if f & 2)
        self.assertGreater(bold_count, 100)
        self.assertGreater(italic_count, 100)
        print(f"✓ Google Docs styles verified: {len(text)} chars, {bold_count} bold chars, {italic_count} italic chars.")

    def test_03_styled_chapter_markdown(self):
        doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"
        url = f"{BASE_URL}/document/d/{doc_id}/edit?usp=sharing"
        html = fetch_html(url)
        text, flags = extract_doc_text_and_styles(html)
        chapters = find_chapter_boundaries(text)
        ch1 = chapters[0]
        md = build_chapter_markdown(text, flags, ch1[2], ch1[3])
        self.assertIn("## Chapter 01", md)
        self.assertIn("\n\n", md) # Verified paragraph breaks
        self.assertIn("**", md) # Verified bold styling
        print(f"✓ Google Docs formatted markdown verified: {len(md)} chars with markdown headings, paragraph breaks, and inline styling.")

if __name__ == "__main__":
    unittest.main()
