#![no_std]

use aidoku::{
	alloc::{format, string::ToString, vec, String, Vec},
	imports::net::Request,
	prelude::*,
	Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Listing, ListingProvider, Manga,
	MangaPageResult, MangaStatus, Page, PageContent, Result, Source, Viewer,
};

mod models;

use models::*;

const BASE_URL: &str = "https://novelarchive.cc";
const API_BASE: &str = "https://novelarchive.cc/api";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

struct NovelArchive;

fn format_cover_url(img: Option<String>) -> Option<String> {
	img.map(|i| {
		if i.starts_with('/') {
			format!("{BASE_URL}{i}")
		} else {
			i
		}
	})
}

fn map_novel_item(n: NovelListItem) -> Option<Manga> {
	let key = n.id?;
	let title = n.title.unwrap_or_default();
	let cover = format_cover_url(n.novel_image.or(n.image_url).or(n.cover_url));
	let authors = n.author.map(|a| vec![a]);
	let tags = n.genres.map(|g| {
		g.split(',')
			.map(|t| t.trim().to_string())
			.filter(|t| !t.is_empty())
			.collect::<Vec<_>>()
	});
	let status_str = n.release_status.as_deref().or(n.status.as_deref());
	let status = match status_str {
		Some("completed") | Some("Completed") => MangaStatus::Completed,
		Some("hiatus") | Some("Hiatus") => MangaStatus::Hiatus,
		Some("cancelled") | Some("Cancelled") => MangaStatus::Cancelled,
		_ => MangaStatus::Ongoing,
	};

	Some(Manga {
		key: key.clone(),
		title,
		cover,
		authors,
		description: n.description,
		tags,
		status,
		viewer: Viewer::Vertical,
		url: Some(format!("{BASE_URL}/novel?id={key}")),
		..Default::default()
	})
}

impl Source for NovelArchive {
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
				format!("{API_BASE}/novels?search={}&page={page}&per_page=24", q.trim())
			}
			_ => {
				format!("{API_BASE}/novels?sort=popular&page={page}&per_page=24")
			}
		};

		let raw_json = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		let response = serde_json::from_str::<NovelListResponse>(&raw_json)
			.map_err(|_| error!("Failed to parse novel list"))?;

		let entries = response
			.novels
			.unwrap_or_default()
			.into_iter()
			.filter_map(map_novel_item)
			.collect::<Vec<_>>();

		let has_next_page = response
			.pagination
			.and_then(|p| {
				if let (Some(page), Some(total)) = (p.page, p.total_pages) {
					Some(page < total)
				} else {
					p.has_next
				}
			})
			.unwrap_or(entries.len() >= 24);

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
		let url = format!("{API_BASE}/novels/{series_key}");

		let raw_json = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		let resp = serde_json::from_str::<NovelDetailResponse>(&raw_json)
			.map_err(|_| error!("Failed to parse novel details"))?;

		let detail = resp.novel.ok_or_else(|| error!("Novel not found"))?;

		if needs_details {
			if let Some(title) = detail.title {
				manga.title = title;
			}
			if let Some(author) = detail.author {
				manga.authors = Some(vec![author]);
			}
			if let Some(desc) = detail.description {
				manga.description = Some(desc);
			}
			if let Some(genres) = detail.genres {
				manga.tags = Some(
					genres
						.split(',')
						.map(|t| t.trim().to_string())
						.filter(|t| !t.is_empty())
						.collect::<Vec<_>>(),
				);
			}
			if let Some(cover) = format_cover_url(detail.novel_image.or(detail.image_url).or(detail.cover_url)) {
				manga.cover = Some(cover);
			}
			let status_str = detail.release_status.as_deref().or(detail.status.as_deref());
			manga.status = match status_str {
				Some("completed") | Some("Completed") => MangaStatus::Completed,
				Some("hiatus") | Some("Hiatus") => MangaStatus::Hiatus,
				Some("cancelled") | Some("Cancelled") => MangaStatus::Cancelled,
				_ => MangaStatus::Ongoing,
			};
			manga.viewer = Viewer::Vertical;
			manga.url = Some(format!("{BASE_URL}/novel?id={series_key}"));
		}

		if needs_chapters {
			if let Some(chapter_names) = detail.chapter_names {
				let chapters = chapter_names
					.into_iter()
					.enumerate()
					.map(|(i, name)| {
						let num = (i + 1) as i32;
						let key = format!("{num}");
						let chapter_number = Some(num as f32);
						let chapter_url = format!("{BASE_URL}/novel?id={series_key}&chapter={num}");

						Chapter {
							key,
							title: Some(name),
							chapter_number,
							url: Some(chapter_url),
							..Default::default()
						}
					})
					.collect::<Vec<_>>();

				manga.chapters = Some(chapters);
			}
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{API_BASE}/novels/{}/chapters/{}", manga.key, chapter.key);
		let raw_json = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		let resp = serde_json::from_str::<ChapterResponse>(&raw_json)
			.map_err(|_| error!("Failed to parse chapter content"))?;

		let chapter_data = resp.chapter.ok_or_else(|| error!("Chapter not found"))?;
		let content = chapter_data.content.unwrap_or_default();

		Ok(vec![Page {
			content: PageContent::text(content),
			..Default::default()
		}])
	}
}

impl ListingProvider for NovelArchive {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		let url = match listing.id.as_str() {
			"trending" => format!("{API_BASE}/novels/trending?limit=24"),
			"latest" => format!("{API_BASE}/novels/recently-updated?limit=24"),
			_ => format!("{API_BASE}/novels?sort=popular&page={page}&per_page=24"),
		};

		let raw_json = Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.string()?;

		if let Ok(resp) = serde_json::from_str::<NovelListResponse>(&raw_json) {
			let entries = resp
				.novels
				.unwrap_or_default()
				.into_iter()
				.filter_map(map_novel_item)
				.collect::<Vec<_>>();
			let has_next_page = resp
				.pagination
				.and_then(|p| {
					if let (Some(page), Some(total)) = (p.page, p.total_pages) {
						Some(page < total)
					} else {
						p.has_next
					}
				})
				.unwrap_or(entries.len() >= 24);
			Ok(MangaPageResult {
				entries,
				has_next_page,
			})
		} else if let Ok(items) = serde_json::from_str::<Vec<NovelListItem>>(&raw_json) {
			let entries = items.into_iter().filter_map(map_novel_item).collect::<Vec<_>>();
			Ok(MangaPageResult {
				entries,
				has_next_page: false,
			})
		} else {
			self.get_search_manga_list(None, page, Vec::new())
		}
	}
}

impl DeepLinkHandler for NovelArchive {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let path = if let Some(stripped) = url.strip_prefix(BASE_URL) {
			stripped
		} else {
			&url
		};

		if let Some(idx) = path.find("id=") {
			let sub = &path[idx + 3..];
			let id = sub.split('&').next().unwrap_or(sub);
			if let Some(chap_idx) = path.find("chapter=") {
				let chap_sub = &path[chap_idx + 8..];
				let chap_num = chap_sub.split('&').next().unwrap_or(chap_sub);
				return Ok(Some(DeepLinkResult::Chapter {
					manga_key: id.to_string(),
					key: chap_num.to_string(),
				}));
			}
			return Ok(Some(DeepLinkResult::Manga { key: id.to_string() }));
		}

		Ok(None)
	}
}

register_source!(NovelArchive, ListingProvider, DeepLinkHandler);
