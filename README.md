# Aidoku Light Novel Extensions

This repository contains custom extensions for the **[Aidoku](https://aidoku.app)** reader application, with first-class support for **Light Novels and Web Novel sources**.

---

## 1. How Light Novel Sources Work in Aidoku

While Aidoku is primarily known for manga/manhwa/comics, its official Rust SDK (`aidoku-rs`) natively supports text-based sources through the `PageContent::Text(String)` variant within its `Page` struct:

```rust
pub enum PageContent {
    /// A url to an image, with associated context.
    Url(String, Option<PageContext>),
    /// A markdown text string.
    Text(String),
    ...
}
```

### Chapter & Page Architecture
* **Source Trait Implementation**: Sources implement the standard Aidoku `Source` trait (`get_search_manga_list`, `get_manga_update`, `get_page_list`).
* **Content Delivery**: In `get_page_list()`, instead of returning image URLs, light novel extensions fetch the chapter's HTML content, convert it into clean Markdown (preserving paragraphs, headings, bold, italics, etc.), and wrap it inside `PageContent::Text(markdown_string)`.
* **Reader Rendering**: The Aidoku reader natively renders Markdown formatted text, providing seamless scrolling and reading for web novels.

---

## 2. Included Sources

### **WeTriedTLs (`sources/wetriedtls`)**
* **Target Site**: [WeTried Translations](https://wetriedtls.com)
* **Backend API**: `https://api.wetriedtls.com`
* **Features**:
  * **Novel Catalog & Pagination**: `GET /query?page={page}&perPage=20`
  * **Live Search**: `GET /query?adult=true&query_string={query}`
  * **Novel Metadata**: Title, Author, Cover image, Status, Description, and Tags from `GET /series/{slug}`
  * **Complete Chapter List**: Fetches all available free & premium chapters from `GET /chapters/{slug}?page=1&perPage=1000` with lock status indication.
  * **Chapter Reading**: Retrieves chapter text from `GET /chapter/{series_slug}/{chapter_slug}` and converts HTML `<p>`, `<strong>`, `<em>`, `<h1>`-`<h3>`, `<br>`, and HTML entities to clean Markdown.
  * **Deep Linking**: Opens `https://wetriedtls.com/series/{slug}` or specific chapters directly in Aidoku.

---

## 3. Project Structure

```
Aidoku_extensions/
├── .github/
│   └── workflows/
│       └── deploy.yml              # GitHub Actions CI/CD to deploy Pages automatically
├── Cargo.toml                      # Workspace configuration
├── build_public.py                 # 1-step build script for public repository
├── package_aix.py                  # Package builder (.aix zip format)
├── public/                         # Public source list repository for Aidoku
│   ├── icons/                      # Source icons
│   ├── sources/                    # Built .aix packages
│   ├── index.html                  # Landing page
│   ├── index.json                  # Source list manifest
│   └── index.min.json              # Minified manifest
├── sources/
│   └── wetriedtls/
│       ├── Cargo.toml              # cdylib package targeting wasm32-unknown-unknown
│       ├── res/
│       │   ├── source.json         # Extension metadata
│       │   └── icon.png            # Extension icon (128x128 opaque)
│       ├── src/
│       │   ├── lib.rs              # WeTriedTLs Source trait implementation
│       │   ├── models.rs           # JSON deserialization data models
│       │   └── parser.rs           # HTML to Markdown parser for novel text
│       └── wetriedtls.aix          # Packaged Aidoku extension
└── tests/
    └── test_wetriedtls_source.py   # Automated integration test suite
```

---

## 4. Building and Packaging

### Prerequisites
* Rust toolchain (1.80+)
* WebAssembly target: `rustup target add wasm32-unknown-unknown`
* Python 3.10+ (for tests and packaging)

### 1-Step Build for Public Repository
```bash
python build_public.py
```
This builds the WebAssembly binaries, packages `.aix` archives, and generates `public/index.min.json` + `public/sources/` ready to be served.

---

## 5. Automated Testing

Run the integration test suite to verify live API integration, metadata parsing, and content rendering:
```bash
python tests/test_wetriedtls_source.py
```

---

## 6. Public Source List & GitHub Pages Deployment

### Enabling GitHub Pages
1. Push this repository to GitHub on the `main` branch.
2. In your repository on GitHub:
   - Go to **Settings** ➔ **Pages**.
   - Under **Build and deployment** ➔ **Source**, select **GitHub Actions**.
3. The included [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml) will automatically build all extensions and deploy the public repository to GitHub Pages on every commit.

### Adding to Aidoku App
1. Open **Aidoku** on iOS/iPadOS.
2. Go to **Settings** ➔ **Source Lists** ➔ tap **+** (Add).
3. Enter your repository's GitHub Pages URL:
   ```
   https://<your-username>.github.io/<repo-name>/
   ```
4. Switch to the **Browse** tab to install **We Tried TLs** directly in Aidoku!
