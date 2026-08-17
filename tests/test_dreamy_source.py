import sys
import unittest
import urllib.request
import json
import re

sys.stdout.reconfigure(encoding='utf-8')

BASE_URL = "https://dreamy-translations.com"
USER_AGENT = "Aidoku"

def fetch_rsc(url):
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "RSC": "1"})
    with urllib.request.urlopen(req) as resp:
        return resp.read().decode('utf-8', errors='ignore')

def fetch_html(url):
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req) as resp:
        return resp.read().decode('utf-8', errors='ignore')

def html_to_markdown(html):
    out = re.sub(r'</p>|</h1>|</h2>|</h3>', '\n\n', html, flags=re.I)
    out = re.sub(r'<br\s*/?>', '\n', out, flags=re.I)
    out = re.sub(r'<[^>]+>', '', out)
    out = out.replace('&nbsp;', ' ').replace('&amp;', '&').replace('&quot;', '"').replace('&#39;', "'")
    return re.sub(r'\n{3,}', '\n\n', out).strip()

class TestDreamyTranslationsSource(unittest.TestCase):
    def test_01_catalog_and_search(self):
        """Test fetching the /series catalog and filtering for search."""
        rsc = fetch_rsc(f"{BASE_URL}/series")
        projects = []
        for line in rsc.split('\n'):
            if '"projects":' in line:
                colon = line.find(':')
                payload = line[colon+1:]
                parsed = json.loads(payload)
                projects = parsed[3]['projects']
                break

        self.assertGreater(len(projects), 50, "Should have 50+ projects")
        
        # Test search
        filtered = [p for p in projects if "wilderness" in p.get("title", "").lower()]
        self.assertGreater(len(filtered), 0, "Search for 'wilderness' should match")
        self.assertEqual(filtered[0].get("slug"), "twimc")
        print(f"✓ Dreamy catalog & search verified: {len(projects)} total projects, search matched '{filtered[0].get('title')}'")

    def test_02_novel_details(self):
        """Test fetching novel metadata from /novel/twimc."""
        rsc = fetch_rsc(f"{BASE_URL}/novel/twimc")
        proj = None
        for line in rsc.split('\n'):
            if '{"project":' in line:
                colon = line.find(':')
                payload = line[colon+1:]
                parsed = json.loads(payload)
                proj = parsed[3]['project']
                break
        
        self.assertIsNotNone(proj)
        self.assertEqual(proj.get("slug"), "twimc")
        self.assertIn("title", proj)
        self.assertIn("author", proj)
        print(f"✓ Dreamy novel details verified: Title='{proj.get('title')}', Author='{proj.get('author')}'")

    def test_03_chapter_list(self):
        """Test fetching chapter list for /novel/twimc."""
        rsc = fetch_rsc(f"{BASE_URL}/novel/twimc")
        chaps = []
        for line in rsc.split('\n'):
            if '"chapters":' in line:
                colon = line.find(':')
                payload = line[colon+1:]
                parsed = json.loads(payload)
                chaps = parsed[3]['chapters']
                break

        self.assertGreater(len(chaps), 10, "Should have at least 10 chapters")
        sample = chaps[0]
        self.assertIn("title", sample)
        self.assertIn("index", sample)
        print(f"✓ Dreamy chapter list verified: {len(chaps)} chapters found. First: '{sample.get('title')}' (Index: {sample.get('index')})")

    def test_04_chapter_content(self):
        """Test reading chapter content from /novel/twimc/chapter/1."""
        html = fetch_html(f"{BASE_URL}/novel/twimc/chapter/1")
        self.assertIn("paragraph", html)
        
        md = html_to_markdown(html)
        self.assertIn("Interview", md)
        self.assertGreater(len(md), 500, "Markdown text should be substantive")
        print(f"✓ Dreamy chapter content verified: {len(md)} characters parsed for Chapter 1.")

if __name__ == "__main__":
    unittest.main(verbosity=2)
