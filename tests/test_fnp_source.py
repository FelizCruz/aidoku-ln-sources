import unittest
import urllib.request
import re
import sys

sys.stdout.reconfigure(encoding='utf-8')

USER_AGENT = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"

def fetch_text(url):
    req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(req, timeout=15) as resp:
        return resp.read().decode('utf-8', errors='ignore')

def extract_images_from_html(html):
    imgs = []
    for m in re.finditer(r'https://img\.kfcok\.net/[^\s"\'<>]+', html):
        url = m.group(0)
        if url not in imgs:
            imgs.append(url)
    return imgs

class TestFuckNovelPiaSource(unittest.TestCase):
    def test_01_catalog_listing(self):
        """Test catalog listing on FUCKNOVELPIA."""
        html = fetch_text("https://fucknovelpia.com/index.php")
        self.assertIn("card", html)
        slugs = re.findall(r'href=[\'"]/novel/([^\'"]+)[\'"]', html)
        clean_slugs = [s for s in slugs if not s.isdigit() and s.strip()]
        self.assertGreater(len(clean_slugs), 0)
        print(f"\n✓ FUCKNOVELPIA catalog verified: {len(clean_slugs)} novel links found. Sample: '{clean_slugs[0]}'")

    def test_02_novel_details_and_chapters(self):
        """Test novel details and chapter list on FUCKNOVELPIA."""
        html = fetch_text("https://fucknovelpia.com/novel/i-raised-the-final-boss-weirdly")
        title_m = re.search(r'<h1[^>]*>(.*?)</h1>', html, re.I | re.DOTALL)
        self.assertIsNotNone(title_m)
        title = re.sub(r'<[^>]+>', '', title_m.group(1)).strip()
        self.assertEqual(title, "I Raised the Final Boss Weirdly")

        raw_chapters = re.findall(r'href=[\'"]/chapter\.php\?([^\'"]+)[\'"]', html)
        self.assertGreater(len(raw_chapters), 0)
        
        # Verify &amp; is replaced with &
        clean_chapters = [c.replace("&amp;", "&") for c in raw_chapters]
        self.assertNotIn("&amp;", clean_chapters[0])
        print(f"✓ FUCKNOVELPIA details & chapters verified: Title='{title}', Chapters={len(clean_chapters)} found. First='{clean_chapters[0]}'")

    def test_03_chapter_image_content(self):
        """Test reading chapter content from FUCKNOVELPIA."""
        html = fetch_text("https://fucknovelpia.com/chapter.php?hash=d895822a39260ea114119ec2e7e3de6b3f60bdc6&ch=0002")
        imgs = extract_images_from_html(html)
        self.assertGreater(len(imgs), 0)
        print(f"✓ FUCKNOVELPIA chapter content verified: {len(imgs)} images found. Sample: '{imgs[0]}'")

if __name__ == "__main__":
    unittest.main()
