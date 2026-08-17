#![no_std]

use aidoku::{
	alloc::{format, string::ToString, vec, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, ContentRating, DeepLinkHandler, DeepLinkResult, FilterValue, Listing, ListingProvider,
	Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

mod parser;

const BASE_URL: &str = "https://fucknovelpia.com";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

struct FuckNovelPia;

fn extract_novels_from_html(html: &str) -> Vec<Manga> {
	let mut novels = Vec::new();
	let mut seen_keys = Vec::new();

	// Match card links
	let mut pos = 0;
	while let Some(idx) = html[pos..].find("href=\"/novel/") {
		let abs_start = pos + idx + 13;
		if let Some(end) = html[abs_start..].find('\"') {
			let href = &html[abs_start..abs_start + end];
			let slug = href.split('?').next().unwrap_or(href).split('#').next().unwrap_or(href);

			if !slug.is_empty() && !seen_keys.contains(&slug.to_string()) && slug.chars().any(|c| c.is_alphabetic()) {
				seen_keys.push(slug.to_string());

				// Extract cover and title from surrounding HTML
				let search_start = if abs_start > 500 { abs_start - 500 } else { 0 };
				let search_end = core::cmp::min(abs_start + 500, html.len());
				let surrounding = &html[search_start..search_end];

				let mut cover = None;
				let mut title = slug.to_string();

				if let Some(img_idx) = surrounding.find("<img") {
					let img_sub = &surrounding[img_idx..];
					if let Some(img_end) = img_sub.find('>') {
						let img_tag = &img_sub[..img_end];
						
						if let Some(src_idx) = img_tag.find("src=\"") {
							let src_sub = &img_tag[src_idx + 5..];
							if let Some(src_end) = src_sub.find('\"') {
								cover = Some(src_sub[..src_end].to_string());
							}
						}

						if let Some(alt_idx) = img_tag.find("alt=\"") {
							let alt_sub = &img_tag[alt_idx + 5..];
							if let Some(alt_end) = alt_sub.find('\"') {
								let raw_title = &alt_sub[..alt_end];
								if !raw_title.is_empty() {
									title = parser::clean_html_tags(raw_title).trim().to_string();
								}
							}
						}
					}
				}

				novels.push(Manga {
					key: slug.to_string(),
					title,
					cover,
					status: MangaStatus::Ongoing,
					content_rating: ContentRating::NSFW,
					viewer: Viewer::Vertical,
					url: Some(format!("{BASE_URL}/novel/{slug}")),
					..Default::default()
				});
			}
			pos = abs_start + end;
		} else {
			break;
		}
	}

	novels
}

impl Source for FuckNovelPia {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let url = match query {
			Some(ref q) if !q.trim().is_empty() => {
				format!("{BASE_URL}/search.php?q={}", q.trim())
			}
			_ => {
				if page == 1 {
					format!("{BASE_URL}/index.php")
				} else {
					format!("{BASE_URL}/?page={page}")
				}
			}
		};

		let html = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		let entries = extract_novels_from_html(&html);
		let has_next_page = html.contains(&format!("page={}", page + 1)) || entries.len() >= 10;

		Ok(MangaPageResult {
			entries,
			has_next_page,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let series_key = &manga.key;
		let url = format!("{BASE_URL}/novel/{series_key}");

		let html = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		if needs_details {
			// Title from <h1>
			if let Some(h1_start) = html.find("<h1") {
				if let Some(tag_end) = html[h1_start..].find('>') {
					let after_tag = h1_start + tag_end + 1;
					if let Some(h1_end) = html[after_tag..].find("</h1>") {
						let raw_title = &html[after_tag..after_tag + h1_end];
						manga.title = parser::clean_html_tags(raw_title).trim().to_string();
					}
				}
			}

			// Cover from og:image or cover image
			if let Some(og_idx) = html.find("property=\"og:image\" content=\"") {
				let after_og = og_idx + 29;
				if let Some(og_end) = html[after_og..].find('\"') {
					manga.cover = Some(html[after_og..after_og + og_end].to_string());
				}
			}

			// Description from meta description
			if let Some(desc_idx) = html.find("name=\"description\" content=\"") {
				let after_desc = desc_idx + 28;
				if let Some(desc_end) = html[after_desc..].find('\"') {
					let raw_desc = &html[after_desc..after_desc + desc_end];
					manga.description = Some(parser::clean_html_tags(raw_desc).trim().to_string());
				}
			}

			manga.content_rating = ContentRating::NSFW;
			manga.viewer = Viewer::Vertical;
			manga.status = MangaStatus::Ongoing;
			manga.url = Some(format!("{BASE_URL}/novel/{series_key}"));
		}

		if needs_chapters {
			let mut chapters = Vec::new();
			let mut seen_keys = Vec::new();
			let mut pos = 0;

			while let Some(idx) = html[pos..].find("href=\"/chapter.php?") {
				let abs_start = pos + idx + 18;
				if let Some(end) = html[abs_start..].find('\"') {
					let query_str = &html[abs_start..abs_start + end];
					
					if !seen_keys.contains(&query_str.to_string()) {
						seen_keys.push(query_str.to_string());

						// Extract ch= number for chapter ordering
						let mut chapter_number = None;
						if let Some(ch_idx) = query_str.find("ch=") {
							let ch_sub = &query_str[ch_idx + 3..];
							let ch_val_str = ch_sub.split('&').next().unwrap_or(ch_sub);
							if let Ok(num) = ch_val_str.parse::<f32>() {
								chapter_number = Some(num);
							}
						}

						// Extract chapter title snippet
						let search_end = core::cmp::min(abs_start + end + 300, html.len());
						let sub = &html[abs_start + end..search_end];
						let mut title = None;
						if let Some(title_start) = sub.find("<span class=\"chapter-item-main\">") {
							let after_span = title_start + 32;
							if let Some(span_end) = sub[after_span..].find("</span>") {
								let t = parser::clean_html_tags(&sub[after_span..after_span + span_end]);
								title = Some(t.trim().to_string());
							}
						}

						let chapter_url = format!("{BASE_URL}/chapter.php?{query_str}");

						chapters.push(Chapter {
							key: query_str.to_string(),
							title,
							chapter_number,
							url: Some(chapter_url),
							..Default::default()
						});
					}
					pos = abs_start + end;
				} else {
					break;
				}
			}

			// Sort by chapter number
			chapters.sort_by(|a, b| {
				let na = a.chapter_number.unwrap_or(0.0);
				let nb = b.chapter_number.unwrap_or(0.0);
				na.partial_cmp(&nb).unwrap_or(core::cmp::Ordering::Equal)
			});

			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = if chapter.key.starts_with("http") {
			chapter.key
		} else {
			format!("{BASE_URL}/chapter.php?{}", chapter.key)
		};

		let html = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		// 1. Check if chapter is an image/scan page
		let mut img_urls = Vec::new();
		let mut pos = 0;
		while let Some(idx) = html[pos..].find("src=\"https://img.kfcok.net/books/") {
			let abs_start = pos + idx + 5;
			if let Some(end) = html[abs_start..].find('\"') {
				let img_url = &html[abs_start..abs_start + end];
				if !img_urls.contains(&img_url.to_string()) {
					img_urls.push(img_url.to_string());
				}
				pos = abs_start + end;
			} else {
				break;
			}
		}

		if !img_urls.is_empty() {
			let pages = img_urls
				.into_iter()
				.map(|u| Page {
					content: PageContent::url(u),
					..Default::default()
				})
				.collect::<Vec<_>>();
			return Ok(pages);
		}

		// 2. Otherwise extract and clean chapter text
		let markdown_text = if let Some(reader_start) = html.find("<div class=\"reader\">") {
			let after_start = reader_start + 20;
			if let Some(reader_end) = html[after_start..].find("</div>") {
				parser::html_to_markdown(&html[after_start..after_start + reader_end])
			} else {
				parser::html_to_markdown(&html)
			}
		} else {
			parser::html_to_markdown(&html)
		};

		Ok(vec![Page {
			content: PageContent::text(markdown_text),
			..Default::default()
		}])
	}
}

impl ListingProvider for FuckNovelPia {
	fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
		self.get_search_manga_list(None, page, Vec::new())
	}
}

impl DeepLinkHandler for FuckNovelPia {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let path = if let Some(stripped) = url.strip_prefix(BASE_URL) {
			stripped
		} else if let Some(idx) = url.find("/novel/") {
			&url[idx..]
		} else if let Some(idx) = url.find("/chapter.php?") {
			&url[idx..]
		} else {
			return Ok(None);
		};

		if path.starts_with("/novel/") {
			let slug = path.trim_start_matches("/novel/").split('?').next().unwrap_or("").split('#').next().unwrap_or("");
			if !slug.is_empty() {
				return Ok(Some(DeepLinkResult::Manga { key: slug.to_string() }));
			}
		} else if path.starts_with("/chapter.php?") {
			let query = path.trim_start_matches("/chapter.php?");
			return Ok(Some(DeepLinkResult::Chapter {
				manga_key: String::new(),
				key: query.to_string(),
			}));
		}

		Ok(None)
	}
}

register_source!(FuckNovelPia, ListingProvider, DeepLinkHandler);
