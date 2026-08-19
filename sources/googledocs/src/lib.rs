#![no_std]

use aidoku::{
	alloc::{format, string::ToString, vec, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, ContentRating, DeepLinkHandler, DeepLinkResult, FilterValue, Listing, ListingProvider,
	Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

mod parser;

use parser::*;

const BASE_URL: &str = "https://docs.google.com";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

struct GoogleDocs;

fn get_featured_novels() -> Vec<Manga> {
	vec![
		Manga {
			key: "1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4".to_string(),
			title: "I'm A Young God, Won't You Raise Me?".to_string(),
			authors: Some(vec!["어린 신인데 키워주실래요?".to_string()]),
			description: Some("A former pro gamer who retired due to injury.\nAfter retiring, I spent my days just playing games…\nThen the game I used to play became reality.\n[You are the System of planet 'Earth'.]".to_string()),
			status: MangaStatus::Ongoing,
			content_rating: ContentRating::Safe,
			viewer: Viewer::Vertical,
			url: Some("https://docs.google.com/document/d/1wTp9NWX_5ALyzwCxW2IzpJnzIMgabw09qTIk-SR0wX4/edit?usp=sharing".to_string()),
			..Default::default()
		}
	]
}

fn clean_doc_title(html: &str) -> Option<String> {
	if let Some(start) = html.find("<title>") {
		let after = start + 7;
		if let Some(end) = html[after..].find("</title>") {
			let raw_title = &html[after..after + end];
			let cleaned = raw_title
				.replace(" - Google Docs", "")
				.replace("&#39;", "'")
				.replace("&quot;", "\"")
				.replace("&amp;", "&")
				.trim()
				.to_string();
			if !cleaned.is_empty() {
				return Some(cleaned);
			}
		}
	}
	None
}

impl Source for GoogleDocs {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		_page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		if let Some(ref q) = query {
			let trimmed = q.trim();
			if !trimmed.is_empty() {
				let doc_id = extract_doc_id(trimmed);

				// If query looks like a Google Doc ID or URL
				if doc_id.len() >= 20 && !doc_id.contains(' ') {
					let url = format!("{BASE_URL}/document/d/{doc_id}/edit?usp=sharing");
					let html = Request::get(&url)?
						.header("User-Agent", USER_AGENT)
						.string()?;

					let title = clean_doc_title(&html).unwrap_or_else(|| format!("Google Doc ({doc_id})"));

					let novel = Manga {
						key: doc_id.clone(),
						title,
						status: MangaStatus::Ongoing,
						content_rating: ContentRating::Safe,
						viewer: Viewer::Vertical,
						url: Some(url),
						..Default::default()
					};

					return Ok(MangaPageResult {
						entries: vec![novel],
						has_next_page: false,
					});
				}

				// Otherwise filter featured list
				let featured = get_featured_novels();
				let filtered = featured
					.into_iter()
					.filter(|m| {
						let q_lower = trimmed.to_lowercase();
						m.title.to_lowercase().contains(&q_lower) || m.key.contains(trimmed)
					})
					.collect::<Vec<_>>();

				return Ok(MangaPageResult {
					entries: filtered,
					has_next_page: false,
				});
			}
		}

		Ok(MangaPageResult {
			entries: get_featured_novels(),
			has_next_page: false,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let doc_id = extract_doc_id(&manga.key);
		let url = format!("{BASE_URL}/document/d/{doc_id}/edit?usp=sharing");

		let html = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		if needs_details {
			if let Some(title) = clean_doc_title(&html) {
				manga.title = title;
			}
			manga.content_rating = ContentRating::Safe;
			manga.viewer = Viewer::Vertical;
			manga.status = MangaStatus::Ongoing;
			manga.url = Some(url.clone());
		}

		if needs_chapters {
			let text = extract_doc_text_from_html(&html);
			let chapter_entries = find_chapter_boundaries(&text);

			let chapters = chapter_entries
				.into_iter()
				.map(|ch| {
					let key = format!("{}::{}::{}", doc_id, ch.start_pos, ch.end_pos);
					let chapter_number = Some(ch.index as f32);
					let chapter_url = format!("{BASE_URL}/document/d/{doc_id}/edit?usp=sharing#chapter={}", ch.index);

					Chapter {
						key,
						title: Some(ch.title),
						chapter_number,
						url: Some(chapter_url),
						..Default::default()
					}
				})
				.collect::<Vec<_>>();

			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let key_parts = chapter.key.split("::").collect::<Vec<_>>();
		let doc_id = key_parts.get(0).copied().unwrap_or(&chapter.key);
		let start_pos = key_parts.get(1).and_then(|s| s.parse::<usize>().ok());
		let end_pos = key_parts.get(2).and_then(|s| s.parse::<usize>().ok());

		let url = format!("{BASE_URL}/document/d/{doc_id}/edit?usp=sharing");
		let html = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		let full_text = extract_doc_text_from_html(&html);

		let markdown_text = match (start_pos, end_pos) {
			(Some(start), Some(end)) if start < full_text.len() => {
				get_chapter_content(&full_text, start, end)
			}
			_ => {
				full_text.replace('\u{000C}', "\n\n").trim().to_string()
			}
		};

		Ok(vec![Page {
			content: PageContent::text(markdown_text),
			..Default::default()
		}])
	}
}

impl ListingProvider for GoogleDocs {
	fn get_manga_list(&self, _listing: Listing, _page: i32) -> Result<MangaPageResult> {
		Ok(MangaPageResult {
			entries: get_featured_novels(),
			has_next_page: false,
		})
	}
}

impl DeepLinkHandler for GoogleDocs {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let doc_id = extract_doc_id(&url);
		if !doc_id.is_empty() && doc_id.len() >= 20 {
			return Ok(Some(DeepLinkResult::Manga { key: doc_id }));
		}
		Ok(None)
	}
}

register_source!(GoogleDocs, ListingProvider, DeepLinkHandler);
