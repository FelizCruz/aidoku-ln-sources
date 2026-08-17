import sys
import unittest
import urllib.request
import json
import re

sys.stdout.reconfigure(encoding='utf-8')

BASE_URL = "https://wetriedtls.com"
API_URL = "https://api.wetriedtls.com"
USER_AGENT = "Aidoku"

def fetch_json(url):
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT, "Accept": "application/json"})
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode('utf-8'))

def html_to_markdown(html):
    result = []
    in_tag = False
    current_tag = []
    i = 0
    chars = list(html)
    length = len(chars)

    while i < length:
        c = chars[i]
        if c == '<':
            in_tag = True
            current_tag = []
            i += 1
            continue
        if c == '>' and in_tag:
            in_tag = False
            tag_str = "".join(current_tag).strip().lower()
            if tag_str in ('/p', '/h1', '/h2', '/h3'):
                result.append('\n\n')
            elif tag_str in ('br', 'br/', 'br /'):
                result.append('\n')
            elif tag_str in ('strong', 'b', '/strong', '/b'):
                result.append('**')
            elif tag_str in ('em', 'i', '/em', '/i'):
                result.append('*')
            elif tag_str.startswith('h1'):
                result.append('# ')
            elif tag_str.startswith('h2'):
                result.append('## ')
            elif tag_str.startswith('h3'):
                result.append('### ')
            elif tag_str in ('hr', 'hr/', 'hr /'):
                result.append('\n\n---\n\n')
            i += 1
            continue
        if in_tag:
            current_tag.append(c)
            i += 1
            continue
        if c == '&':
            rest = "".join(chars[i:min(i+10, length)])
            if rest.startswith("&nbsp;"):
                result.append(' ')
                i += 6
                continue
            elif rest.startswith("&amp;"):
                result.append('&')
                i += 5
                continue
            elif rest.startswith("&lt;"):
                result.append('<')
                i += 4
                continue
            elif rest.startswith("&gt;"):
                result.append('>')
                i += 4
                continue
            elif rest.startswith("&quot;"):
                result.append('"')
                i += 6
                continue
            elif rest.startswith("&#39;") or rest.startswith("&apos;"):
                result.append('\'')
                i += 5 if rest.startswith("&#39;") else 6
                continue
        result.append(c)
        i += 1

    text = "".join(result).strip()
    return re.sub(r'\n{3,}', '\n\n', text)


class TestWeTriedTLsSource(unittest.TestCase):
    def test_01_catalog_listing(self):
        """Test fetching the novel catalog listing (get_manga_list / get_search_manga_list with query=None)."""
        url = f"{API_URL}/query?page=1&perPage=20"
        data = fetch_json(url)
        self.assertIn("data", data)
        self.assertIn("meta", data)
        novels = data["data"]
        self.assertGreater(len(novels), 0, "Catalog should have entries")
        
        # Verify first entry has required Manga fields
        first = novels[0]
        self.assertIsNotNone(first.get("series_slug"))
        self.assertIsNotNone(first.get("title"))
        self.assertTrue(first.get("thumbnail") is not None)
        print(f"✓ Catalog listing verified: {len(novels)} novels returned on page 1. First: '{first.get('title')}'")

    def test_02_search_query(self):
        """Test searching for novels by keyword (get_search_manga_list with query='cultivation')."""
        query = "cultivation"
        url = f"{API_URL}/query?adult=true&query_string={query}"
        data = fetch_json(url)
        self.assertIn("data", data)
        novels = data["data"]
        self.assertGreater(len(novels), 0, "Search for 'cultivation' should return results")
        slugs = [n.get("series_slug") for n in novels]
        self.assertIn("a-regressors-tale-of-cultivation", slugs)
        print(f"✓ Search query '{query}' verified: {len(novels)} results found, including 'a-regressors-tale-of-cultivation'.")

    def test_03_series_details(self):
        """Test fetching series details (get_manga_update with needs_details=true)."""
        series_slug = "a-regressors-tale-of-cultivation"
        url = f"{API_URL}/series/{series_slug}"
        details = fetch_json(url)
        self.assertEqual(details.get("series_slug"), series_slug)
        self.assertIn("title", details)
        self.assertIn("thumbnail", details)
        self.assertIn("description", details)
        self.assertIn("author", details)
        self.assertIn("status", details)
        self.assertIn("tags", details)
        
        # Test markdown conversion of description
        md_desc = html_to_markdown(details.get("description", ""))
        self.assertGreater(len(md_desc), 10)
        print(f"✓ Series details verified: Title='{details.get('title')}', Author='{details.get('author')}', Tags={len(details.get('tags', []))}")

    def test_04_chapter_list(self):
        """Test fetching chapter list (get_manga_update with needs_chapters=true)."""
        series_slug = "a-regressors-tale-of-cultivation"
        url = f"{API_URL}/chapters/{series_slug}?page=1&perPage=1000"
        chap_resp = fetch_json(url)
        self.assertIn("data", chap_resp)
        chapters = chap_resp["data"]
        self.assertGreater(len(chapters), 500, "Should have 500+ chapters")
        
        # Check chapter structure
        sample = chapters[0]
        self.assertIn("chapter_slug", sample)
        self.assertIn("chapter_name", sample)
        self.assertIn("index", sample)
        print(f"✓ Chapter list verified: Total {len(chapters)} chapters found. Sample: '{sample.get('chapter_name')}' ({sample.get('chapter_slug')})")

    def test_05_chapter_content_and_markdown(self):
        """Test fetching chapter content and converting to clean Markdown (get_page_list)."""
        series_slug = "a-regressors-tale-of-cultivation"
        chapter_slug = "chapter-1"
        url = f"{API_URL}/chapter/{series_slug}/{chapter_slug}"
        content_resp = fetch_json(url)
        self.assertIn("chapter", content_resp)
        chap = content_resp["chapter"]
        raw_html = chap.get("chapter_content", "")
        self.assertGreater(len(raw_html), 1000, "Chapter HTML content should be substantive")

        # Convert to Markdown
        markdown_text = html_to_markdown(raw_html)
        self.assertNotIn("<p", markdown_text, "HTML tags should be stripped")
        self.assertIn("Director Kim", markdown_text, "Novel content should be preserved")
        self.assertIn("workshop", markdown_text)
        print(f"✓ Chapter content verified: {len(markdown_text)} characters of clean Markdown parsed for {chapter_slug}.")


if __name__ == "__main__":
    unittest.main(verbosity=2)
