import unittest
import urllib.request
import json
import sys

sys.stdout.reconfigure(encoding='utf-8')

USER_AGENT = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"

def fetch_json(url):
    req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(req, timeout=15) as resp:
        return json.loads(resp.read().decode('utf-8'))

class TestNovelArchiveSource(unittest.TestCase):
    def test_01_catalog_popular(self):
        """Test fetching popular novels on NovelArchive."""
        data = fetch_json("https://novelarchive.cc/api/novels?sort=popular&page=1&per_page=24")
        novels = data.get('novels', [])
        self.assertGreater(len(novels), 0)
        first = novels[0]
        self.assertIn('id', first)
        self.assertIn('title', first)
        print(f"\n✓ NovelArchive catalog verified: {len(novels)} novels returned. First: '{first.get('title')}' (ID: {first.get('id')})")

    def test_02_search(self):
        """Test searching for novels on NovelArchive."""
        data = fetch_json("https://novelarchive.cc/api/novels?search=psycho&page=1&per_page=24")
        novels = data.get('novels', [])
        self.assertGreater(len(novels), 0)
        print(f"✓ NovelArchive search verified: {len(novels)} results found for 'psycho'")

    def test_03_novel_details_and_chapters(self):
        """Test novel details and chapter names on NovelArchive."""
        data = fetch_json("https://novelarchive.cc/api/novels/6a82d83f16bf4c5dbcf25ebb")
        novel = data.get('novel', {})
        self.assertIsNotNone(novel)
        self.assertEqual(novel.get('id'), "6a82d83f16bf4c5dbcf25ebb")
        self.assertTrue(len(novel.get('chapter_names', [])) > 0)
        print(f"✓ NovelArchive details verified: Title='{novel.get('title')}', Author='{novel.get('author')}', Chapters={len(novel.get('chapter_names'))}")

    def test_04_chapter_content(self):
        """Test reading chapter content from NovelArchive."""
        data = fetch_json("https://novelarchive.cc/api/novels/6a82d83f16bf4c5dbcf25ebb/chapters/1")
        chap = data.get('chapter', {})
        content = chap.get('content', '')
        self.assertGreater(len(content), 100)
        print(f"✓ NovelArchive chapter content verified: {len(content)} characters parsed for Chapter 1. First 50 chars: '{content[:50]}'")

if __name__ == "__main__":
    unittest.main()
