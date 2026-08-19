import urllib.request

doc_id = "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4"

urls = [
    f"https://docs.google.com/document/d/{doc_id}/edit?usp=sharing",
    f"https://docs.google.com/document/d/{doc_id}/preview",
    f"https://docs.google.com/document/d/{doc_id}/mobilebasic",
    f"https://docs.google.com/document/u/0/d/{doc_id}/mobilebasic",
]

for url in urls:
    try:
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'})
        with urllib.request.urlopen(req) as resp:
            data = resp.read().decode('utf-8', errors='ignore')
            print(f"[{url}] -> Status: {resp.status}, Length: {len(data)}, Final URL: {resp.url}")
            if "DOCS_modelChunk" in data:
                print("  Found DOCS_modelChunk in page!")
            if "google-site-verification" not in data and "ServiceLogin" not in resp.url:
                title_idx = data.find("<title>")
                if title_idx != -1:
                    print("  Title:", data[title_idx:title_idx+100].split('</title>')[0])
    except Exception as e:
        print(f"[{url}] -> FAILED: {e}")
