use aidoku::alloc::{String, Vec};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NovelListItem {
	pub id: Option<String>,
	pub title: Option<String>,
	pub author: Option<String>,
	pub novel_image: Option<String>,
	pub image_url: Option<String>,
	pub genres: Option<String>,
	pub description: Option<String>,
	pub status: Option<String>,
	pub total_chapters: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationInfo {
	pub page: Option<i32>,
	pub total_pages: Option<i32>,
	pub has_next: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct NovelListResponse {
	pub novels: Option<Vec<NovelListItem>>,
	pub pagination: Option<PaginationInfo>,
}

#[derive(Debug, Deserialize)]
pub struct NovelDetailItem {
	pub id: Option<String>,
	pub title: Option<String>,
	pub author: Option<String>,
	pub novel_image: Option<String>,
	pub image_url: Option<String>,
	pub description: Option<String>,
	pub genres: Option<String>,
	pub status: Option<String>,
	pub total_chapters: Option<i32>,
	pub chapter_names: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct NovelDetailResponse {
	pub novel: Option<NovelDetailItem>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterDetail {
	pub number: Option<i32>,
	pub name: Option<String>,
	pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterResponse {
	pub chapter: Option<ChapterDetail>,
}
