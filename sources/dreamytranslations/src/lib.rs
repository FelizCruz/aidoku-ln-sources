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

const BASE_URL: &str = "https://dreamy-translations.com";

struct DreamyTranslations;

fn extract_projects_from_rsc(rsc: &str) -> Vec<ProjectItem> {
	for line in rsc.split('\n') {
		if let Some(idx) = line.find("\"projects\":") {
			if let Some(start) = line[idx..].find('[') {
				let abs_start = idx + start;
				if let Some(end) = line[abs_start..].find(",\"genres\":") {
					let json_str = &line[abs_start..abs_start + end];
					if let Ok(projects) = serde_json::from_str::<Vec<ProjectItem>>(json_str) {
						return projects;
					}
				}
			}
		}
	}
	Vec::new()
}

fn extract_cover_from_html(html: &str) -> Option<String> {
	if let Some(idx) = html.find("https://supabase.dreamy-translations.com/storage/v1/object/public/covers/") {
		let sub = &html[idx..];
		if let Some(end) = sub.find('"').or_else(|| sub.find('\'')) {
			return Some(sub[..end].to_string());
		}
	}
	None
}

impl Source for DreamyTranslations {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let url = format!("{BASE_URL}/series");
		let rsc_data = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.header("RSC", "1")
			.string()?;

		let mut all_projects = extract_projects_from_rsc(&rsc_data);

		if let Some(q) = query {
			let q_lower = q.trim().to_lowercase();
			if !q_lower.is_empty() {
				all_projects.retain(|p| {
					let title_match = p.title.as_ref().map(|t| t.to_lowercase().contains(&q_lower)).unwrap_or(false);
					let tag_match = p.tags.as_ref().map(|tags| tags.iter().any(|t| t.to_lowercase().contains(&q_lower))).unwrap_or(false);
					let genre_match = p.genres.as_ref().map(|genres| genres.iter().any(|g| g.to_lowercase().contains(&q_lower))).unwrap_or(false);
					title_match || tag_match || genre_match
				});
			}
		}

		let per_page = 20;
		let start_idx = ((page - 1) * per_page) as usize;
		let has_next_page = start_idx + (per_page as usize) < all_projects.len();

		let entries = all_projects
			.into_iter()
			.skip(start_idx)
			.take(per_page as usize)
			.filter_map(|p| {
				let key = p.slug?;
				let title = p.title.unwrap_or_default();
				let description = p.synopsis.or(p.short_synopsis);
				let authors = p.author.map(|a| vec![a]);
				let tags = p.tags.or(p.genres);
				let status = if p.completed == Some(true) {
					MangaStatus::Completed
				} else {
					MangaStatus::Ongoing
				};

				Some(Manga {
					key: key.clone(),
					title,
					description,
					authors,
					tags,
					status,
					viewer: Viewer::Vertical,
					url: Some(format!("{BASE_URL}/novel/{key}")),
					..Default::default()
				})
			})
			.collect::<Vec<_>>();

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

		let rsc_data = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.header("RSC", "1")
			.string()?;

		if needs_details {
			if let Some(idx) = rsc_data.find("{\"project\":") {
				if let Some(start) = rsc_data[idx..].find('{') {
					let abs_start = idx + start + 10;
					if let Some(end) = rsc_data[abs_start..].find(",\"chapters\":") {
						let proj_json = &rsc_data[abs_start..abs_start + end];
						if let Ok(proj) = serde_json::from_str::<ProjectItem>(proj_json) {
							if let Some(title) = proj.title {
								manga.title = title;
							}
							if let Some(author) = proj.author {
								manga.authors = Some(vec![author]);
							}
							if let Some(desc) = proj.synopsis.or(proj.short_synopsis) {
								manga.description = Some(desc);
							}
							if let Some(tags) = proj.tags.or(proj.genres) {
								manga.tags = Some(tags);
							}
							manga.status = if proj.completed == Some(true) {
								MangaStatus::Completed
							} else {
								MangaStatus::Ongoing
							};
						}
					}
				}
			}

			// Extract cover image from novel HTML
			if let Ok(html) = Request::get(&url)?.header("User-Agent", "Aidoku").string() {
				if let Some(cover_url) = extract_cover_from_html(&html) {
					manga.cover = Some(cover_url);
				}
			}

			manga.viewer = Viewer::Vertical;
			manga.url = Some(format!("{BASE_URL}/novel/{series_key}"));
		}

		if needs_chapters {
			if let Some(idx) = rsc_data.find("\"chapters\":") {
				if let Some(start) = rsc_data[idx..].find('[') {
					let abs_start = idx + start;
					if let Some(end) = rsc_data[abs_start..].find("]}") {
						let chaps_json = &rsc_data[abs_start..=abs_start + end];
						if let Ok(chaps) = serde_json::from_str::<Vec<ChapterItem>>(chaps_json) {
							let chapters = chaps
								.into_iter()
								.filter_map(|c| {
									let chapter_number = c.index;
									let index_val = chapter_number.unwrap_or(0.0);
									let key = if (index_val as i32 as f32) == index_val {
										format!("{}", index_val as i32)
									} else {
										format!("{}", index_val)
									};

									let title = c.title;
									let locked = !c.free.unwrap_or(true);
									let chapter_url = format!("{BASE_URL}/novel/{series_key}/chapter/{key}");

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
					}
				}
			}
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{BASE_URL}/novel/{}/chapter/{}", manga.key, chapter.key);
		let html = Request::get(&url)?
			.header("User-Agent", "Aidoku")
			.string()?;

		let markdown_text = parser::html_to_markdown(&html);

		Ok(vec![Page {
			content: PageContent::text(markdown_text),
			..Default::default()
		}])
	}
}

impl ListingProvider for DreamyTranslations {
	fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
		self.get_search_manga_list(None, page, Vec::new())
	}
}

impl DeepLinkHandler for DreamyTranslations {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let path = if let Some(stripped) = url.strip_prefix(BASE_URL) {
			stripped
		} else if let Some(idx) = url.find("/novel/") {
			&url[idx..]
		} else {
			return Ok(None);
		};

		let parts: Vec<&str> = path
			.trim_matches('/')
			.split('/')
			.filter(|p| !p.is_empty())
			.collect();

		if parts.len() >= 4 && parts[0] == "novel" && parts[2] == "chapter" {
			let manga_key = parts[1].to_string();
			let key = parts[3].to_string();
			Ok(Some(DeepLinkResult::Chapter { manga_key, key }))
		} else if parts.len() >= 2 && parts[0] == "novel" {
			let manga_key = parts[1].to_string();
			Ok(Some(DeepLinkResult::Manga { key: manga_key }))
		} else {
			Ok(None)
		}
	}
}

register_source!(DreamyTranslations, ListingProvider, DeepLinkHandler);
