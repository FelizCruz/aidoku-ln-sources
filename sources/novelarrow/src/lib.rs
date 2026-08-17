#![no_std]

use aidoku::{
	alloc::{format, string::ToString, vec, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Listing, ListingProvider, Manga,
	MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

mod parser;

const BASE_URL: &str = "https://novelarrow.com";
const IMAGE_BASE: &str = "https://images.novelarrow.com/novel_328_490";

struct NovelArrow;

fn clean_tag(s: &str) -> String {
	let mut out = String::new();
	let mut in_tag = false;
	for c in s.chars() {
		if c == '<' {
			in_tag = true;
		} else if c == '>' {
			in_tag = false;
		} else if !in_tag {
			out.push(c);
		}
	}
	out.trim().to_string()
}

fn extract_novels_from_html(html: &str) -> Vec<Manga> {
	let mut entries = Vec::new();
	let mut search_pos = 0;

	while let Some(idx) = html[search_pos..].find("href=\"/novel/") {
		let abs_idx = search_pos + idx + 13;
		if let Some(end_href) = html[abs_idx..].find('"') {
			let slug = &html[abs_idx..abs_idx + end_href];
			if !slug.is_empty() && !slug.contains('/') {
				let sub_card = &html[abs_idx..abs_idx + 1200.min(html.len() - abs_idx)];
				let title = if let Some(h_start) = sub_card.find("<h2") {
					if let Some(h_end) = sub_card[h_start..].find("</h2>") {
						clean_tag(&sub_card[h_start..h_start + h_end])
					} else {
						slug.replace('-', " ")
					}
				} else if let Some(h_start) = sub_card.find("<h3") {
					if let Some(h_end) = sub_card[h_start..].find("</h3>") {
						clean_tag(&sub_card[h_start..h_start + h_end])
					} else {
						slug.replace('-', " ")
					}
				} else {
					slug.replace('-', " ")
				};

				let key = slug.to_string();
				if !entries.iter().any(|m: &Manga| m.key == key) {
					entries.push(Manga {
						key: key.clone(),
						title,
						cover: Some(format!("{IMAGE_BASE}/{key}.jpg")),
						status: MangaStatus::Ongoing,
						viewer: Viewer::Vertical,
						url: Some(format!("{BASE_URL}/novel/{key}")),
						..Default::default()
					});
				}
			}
			search_pos = abs_idx + end_href;
		} else {
			break;
		}
	}

	entries
}

impl Source for NovelArrow {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		_page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let url = match query {
			Some(ref q) if !q.trim().is_empty() => {
				format!("{BASE_URL}/novels/search?keyword={q}")
			}
			_ => {
				format!("{BASE_URL}/novel-ranking")
			}
		};

		let html = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.string()?;

		let entries = extract_novels_from_html(&html);
		let has_next_page = false;

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
			.header("User-Agent", "Aidoku")
			.string()?;

		if needs_details {
			if let Some(h1_start) = html.find("<h1") {
				if let Some(h1_end) = html[h1_start..].find("</h1>") {
					let title = clean_tag(&html[h1_start..h1_start + h1_end]);
					if !title.is_empty() {
						manga.title = title;
					}
				}
			}

			// Description
			if let Some(meta_idx) = html.find("name=\"description\" content=\"") {
				let start = meta_idx + 26;
				if let Some(end) = html[start..].find('"') {
					let desc = &html[start..start + end];
					manga.description = Some(desc.to_string());
				}
			}

			manga.cover = Some(format!("{IMAGE_BASE}/{series_key}.jpg"));
			manga.viewer = Viewer::Vertical;
			manga.url = Some(format!("{BASE_URL}/novel/{series_key}"));
		}

		if needs_chapters {
			let prefix = format!("/chapter/{series_key}/");
			let mut chapters = Vec::new();
			let mut search_pos = 0;

			while let Some(idx) = html[search_pos..].find(&prefix) {
				let abs_idx = search_pos + idx + prefix.len();
				if let Some(end_href) = html[abs_idx..].find('"') {
					let chap_slug = &html[abs_idx..abs_idx + end_href];
					if !chap_slug.is_empty() && !chap_slug.contains('/') {
						let sub = &html[abs_idx..abs_idx + 400.min(html.len() - abs_idx)];
						let title_str = if let Some(tag_end) = sub.find('>') {
							if let Some(a_end) = sub[tag_end..].find("</a>") {
								clean_tag(&sub[tag_end + 1..tag_end + a_end])
							} else {
								chap_slug.replace('-', " ")
							}
						} else {
							chap_slug.replace('-', " ")
						};

						let chapter_number = if let Some(c_idx) = chap_slug.find("chapter-") {
							let num_part = &chap_slug[c_idx + 8..];
							let end_num = num_part.find('-').unwrap_or(num_part.len());
							num_part[..end_num].parse::<f32>().ok()
						} else {
							None
						};

						let key = chap_slug.to_string();
						if !chapters.iter().any(|c: &Chapter| c.key == key) {
							chapters.push(Chapter {
								key: key.clone(),
								title: Some(title_str),
								chapter_number,
								url: Some(format!("{BASE_URL}/chapter/{series_key}/{key}")),
								locked: false,
								..Default::default()
							});
						}
					}
					search_pos = abs_idx + end_href;
				} else {
					break;
				}
			}

			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{BASE_URL}/chapter/{}/{}", manga.key, chapter.key);
		let html = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.string()?;

		// Extract chapter content from HTML or script payload
		let raw_content = if let Some(idx) = html.find("\\u003ch4\\u003e") {
			let end = html[idx..].find("\",").or_else(|| html[idx..].find("\"]")).unwrap_or(html.len() - idx);
			let unescaped = html[idx..idx + end]
				.replace("\\u003c", "<")
				.replace("\\u003e", ">")
				.replace("\\\"", "\"")
				.replace("\\'", "'");
			unescaped
		} else if let Some(idx) = html.find("<h4>") {
			let end = html[idx..].find("</div>").or_else(|| html[idx..].find("</article>")).unwrap_or(html.len() - idx);
			html[idx..idx + end].to_string()
		} else {
			html.clone()
		};

		let markdown_text = parser::html_to_markdown(&raw_content);

		Ok(vec![Page {
			content: PageContent::text(markdown_text),
			..Default::default()
		}])
	}
}

impl ListingProvider for NovelArrow {
	fn get_manga_list(&self, listing: Listing, _page: i32) -> Result<MangaPageResult> {
		let url = match listing.id.as_str() {
			"hot" => format!("{BASE_URL}/novels/hot"),
			"latest" => format!("{BASE_URL}/novels/latest"),
			_ => format!("{BASE_URL}/novel-ranking"),
		};

		let html = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.string()?;

		let entries = extract_novels_from_html(&html);

		Ok(MangaPageResult {
			entries,
			has_next_page: false,
		})
	}
}

impl DeepLinkHandler for NovelArrow {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let path = if let Some(stripped) = url.strip_prefix(BASE_URL) {
			stripped
		} else if let Some(idx) = url.find("/novel/") {
			&url[idx..]
		} else if let Some(idx) = url.find("/chapter/") {
			&url[idx..]
		} else {
			return Ok(None);
		};

		let parts: Vec<&str> = path
			.trim_matches('/')
			.split('/')
			.filter(|p| !p.is_empty())
			.collect();

		if parts.len() >= 3 && parts[0] == "chapter" {
			let manga_key = parts[1].to_string();
			let key = parts[2].to_string();
			Ok(Some(DeepLinkResult::Chapter { manga_key, key }))
		} else if parts.len() >= 2 && parts[0] == "novel" {
			let manga_key = parts[1].to_string();
			Ok(Some(DeepLinkResult::Manga { key: manga_key }))
		} else {
			Ok(None)
		}
	}
}

register_source!(NovelArrow, ListingProvider, DeepLinkHandler);
