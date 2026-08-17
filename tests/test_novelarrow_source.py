import sys
import unittest
import urllib.request
import re

sys.stdout.reconfigure(encoding='utf-8')

BASE_URL = "https://novelarrow.com"
USER_AGENT = "Aidoku"

def fetch_html(url):
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req) as resp:
        return resp.read().decode('utf-8', errors='ignore')

def html_to_markdown(html):
    out = re.sub(r'</p>|</h1>|</h2>|</h3>|</h4>', '\n\n', html, flags=re.I)
    out = re.sub(r'<br\s*/?>', '\n', out, flags=re.I)
    out = re.sub(r'<[^>]+>', '', out)
    out = out.replace('&nbsp;', ' ').replace('&amp;', '&').replace('&quot;', '"').replace('&#39;', "'").replace('&#x27;', "'")
    return re.sub(r'\n{3,}', '\n\n', out).strip()

class TestNovelArrowSource(unittest.TestCase):
    def test_01_rankings_catalog(self):
        """Test fetching the /novel-ranking catalog on NovelArrow."""
        html = fetch_html(f"{BASE_URL}/novel-ranking")
        novels = re.findall(r'href=[\'"]/novel/([^\'"]+)[\'"]', html)
        self.assertGreater(len(novels), 10, "Should find 10+ novel links on rankings")
        self.assertIn("shadow-slave", novels)
        print(f"✓ NovelArrow rankings verified: {len(set(novels))} unique novels listed, including 'shadow-slave'")

    def test_02_search(self):
        """Test keyword search on NovelArrow /novels/search?keyword=shadow."""
        html = fetch_html(f"{BASE_URL}/novels/search?keyword=shadow")
        novels = re.findall(r'href=[\'"]/novel/([^\'"]+)[\'"]', html)
        self.assertGreater(len(novels), 0, "Search for 'shadow' should return results")
        print(f"✓ NovelArrow search query verified: {len(set(novels))} novels returned for keyword 'shadow'")

    def test_03_novel_details_and_chapters(self):
        """Test novel page and chapter extraction from /novel/shadow-slave."""
        html = fetch_html(f"{BASE_URL}/novel/shadow-slave")
        self.assertIn("Shadow Slave", html)
        
        chaps = re.findall(r'href=[\'"]/chapter/shadow-slave/([^\'"]+)[\'"]', html)
        self.assertGreater(len(chaps), 10, "Should find 10+ chapters on novel page")
        self.assertIn("chapter-1-nightmare-begins", chaps)
        print(f"✓ NovelArrow details & chapters verified: Title='Shadow Slave', Chapters={len(set(chaps))} listed")

    def test_04_chapter_content(self):
        """Test reading chapter content from /chapter/shadow-slave/chapter-1-nightmare-begins."""
        html = fetch_html(f"{BASE_URL}/chapter/shadow-slave/chapter-1-nightmare-begins")
        
        self.assertTrue("Nightmare Begins" in html or "frail-looking" in html)
        
        idx = html.find("\\u003ch4\\u003e")
        if idx != -1:
            end = html.find('",', idx)
            if end == -1:
                end = html.find('"\\]', idx)
            unescaped = html[idx:end].replace('\\u003c', '<').replace('\\u003e', '>').replace('\\"', '"')
            md = html_to_markdown(unescaped)
        else:
            md = html_to_markdown(html)
            
        self.assertIn("Nightmare", md)
        self.assertGreater(len(md), 500, "Should parse substantive Markdown")
        print(f"✓ NovelArrow chapter content verified: {len(md)} characters parsed for Chapter 1.")

if __name__ == "__main__":
    unittest.main(verbosity=2)
