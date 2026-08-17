#![no_std]

use aidoku::{
	alloc::{format, string::ToString, vec, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Listing, ListingProvider, Manga,
	MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

mod models;
mod parser;

use models::*;

const BASE_URL: &str = "https://wetriedtls.com";
const API_URL: &str = "https://api.wetriedtls.com";

struct WeTriedTLs;

impl Source for WeTriedTLs {
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
				format!("{API_URL}/query?adult=true&query_string={q}")
			}
			_ => {
				format!("{API_URL}/query?page={page}&perPage=20")
			}
		};

		let response: QueryResponse = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.json_owned()?;

		let entries = response
			.data
			.unwrap_or_default()
			.into_iter()
			.filter_map(|item| {
				let key = item.series_slug?;
				let title = item.title.unwrap_or_default();
				let cover = item.thumbnail;
				let authors = item.author.map(|a| vec![a]);
				let tags = item.tags.map(|t_list| {
					t_list.into_iter().filter_map(|t| t.name).collect::<Vec<_>>()
				});
				let status = match item.status.as_deref() {
					Some("Ongoing") => MangaStatus::Ongoing,
					Some("Completed") => MangaStatus::Completed,
					Some("Cancelled") => MangaStatus::Cancelled,
					Some("Hiatus") => MangaStatus::Hiatus,
					_ => MangaStatus::Unknown,
				};

				Some(Manga {
					key: key.clone(),
					title,
					cover,
					authors,
					tags,
					status,
					viewer: Viewer::Vertical,
					url: Some(format!("{BASE_URL}/series/{key}")),
					..Default::default()
				})
			})
			.collect::<Vec<_>>();

		let has_next_page = response
			.meta
			.and_then(|m| m.next_page_url)
			.is_some();

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

		if needs_details {
			let url = format!("{API_URL}/series/{series_key}");
			let details: SeriesItem = Request::get(&url)?
				.header("User-Agent", "Aidoku")
				.json_owned()?;

			if let Some(title) = details.title {
				manga.title = title;
			}
			if let Some(cover) = details.thumbnail {
				manga.cover = Some(cover);
			}
			if let Some(author) = details.author {
				manga.authors = Some(vec![author]);
			}
			if let Some(ref desc_html) = details.description {
				manga.description = Some(parser::html_to_markdown(desc_html));
			}
			if let Some(tags) = details.tags {
				manga.tags = Some(
					tags.into_iter()
						.filter_map(|t| t.name)
						.collect::<Vec<_>>(),
				);
			}
			manga.status = match details.status.as_deref() {
				Some("Ongoing") => MangaStatus::Ongoing,
				Some("Completed") => MangaStatus::Completed,
				Some("Cancelled") => MangaStatus::Cancelled,
				Some("Hiatus") => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			};
			manga.viewer = Viewer::Vertical;
			manga.url = Some(format!("{BASE_URL}/series/{series_key}"));
		}

		if needs_chapters {
			let url = format!("{API_URL}/chapters/{series_key}?page=1&perPage=1000");
			let chap_resp: ChapterListResponse = Request::get(&url)?
				.header("User-Agent", "Aidoku")
				.json_owned()?;

			let chapters = chap_resp
				.data
				.unwrap_or_default()
				.into_iter()
				.filter_map(|c| {
					let key = c.chapter_slug?;
					let name = c.chapter_name.unwrap_or_default();
					let title_str = c.chapter_title.unwrap_or_default();
					let title = if !title_str.is_empty() && !name.is_empty() {
						Some(format!("{name} - {title_str}"))
					} else if !title_str.is_empty() {
						Some(title_str)
					} else if !name.is_empty() {
						Some(name)
					} else {
						None
					};

					let chapter_number = c.index.as_deref().and_then(|idx| {
						idx.parse::<f32>().ok()
					});

					let locked = c.price.unwrap_or(0) > 0 || !c.public.unwrap_or(true);
					let chapter_url = format!("{BASE_URL}/series/{series_key}/{key}");

					Some(Chapter {
						key,
						title,
						chapter_number,
						url: Some(chapter_url),
						locked,
						..Default::default()
					})
				})
				.collect::<Vec<_>>();

			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{API_URL}/chapter/{}/{}", manga.key, chapter.key);
		let content_resp: ChapterContentResponse = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.json_owned()?;

		let chapter_data = content_resp.chapter.ok_or_else(|| error!("Chapter not found"))?;
		let raw_html = chapter_data.chapter_content.unwrap_or_default();
		let markdown_text = parser::html_to_markdown(&raw_html);

		Ok(vec![Page {
			content: PageContent::text(markdown_text),
			..Default::default()
		}])
	}
}

impl ListingProvider for WeTriedTLs {
	fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
		self.get_search_manga_list(None, page, Vec::new())
	}
}

impl DeepLinkHandler for WeTriedTLs {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let path = if let Some(stripped) = url.strip_prefix(BASE_URL) {
			stripped
		} else if let Some(idx) = url.find("/series/") {
			&url[idx..]
		} else {
			return Ok(None);
		};

		let parts: Vec<&str> = path
			.trim_matches('/')
			.split('/')
			.filter(|p| !p.is_empty())
			.collect();

		if parts.len() >= 3 && parts[0] == "series" {
			let manga_key = parts[1].to_string();
			let key = parts[2].to_string();
			Ok(Some(DeepLinkResult::Chapter {
				manga_key,
				key,
			}))
		} else if parts.len() >= 2 && parts[0] == "series" {
			let manga_key = parts[1].to_string();
			Ok(Some(DeepLinkResult::Manga { key: manga_key }))
		} else {
			Ok(None)
		}
	}
}

register_source!(WeTriedTLs, ListingProvider, DeepLinkHandler);
