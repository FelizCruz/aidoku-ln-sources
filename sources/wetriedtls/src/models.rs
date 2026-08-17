#![allow(dead_code)]

use aidoku::alloc::{String, Vec};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryResponse {
	pub data: Option<Vec<SeriesItem>>,
	pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationMeta {
	pub total: Option<i32>,
	pub current_page: Option<i32>,
	pub last_page: Option<i32>,
	pub next_page_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesTag {
	pub id: Option<i64>,
	pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SeriesItem {
	pub id: Option<i64>,
	pub title: Option<String>,
	pub series_slug: Option<String>,
	pub thumbnail: Option<String>,
	pub description: Option<String>,
	pub status: Option<String>,
	pub author: Option<String>,
	pub tags: Option<Vec<SeriesTag>>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterListResponse {
	pub data: Option<Vec<ChapterItem>>,
	pub meta: Option<PaginationMeta>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterItem {
	pub id: Option<i64>,
	pub chapter_slug: Option<String>,
	pub chapter_name: Option<String>,
	pub chapter_title: Option<String>,
	pub index: Option<String>,
	pub price: Option<i32>,
	pub public: Option<bool>,
	pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterContentResponse {
	pub chapter: Option<ChapterDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterDetail {
	pub id: Option<i64>,
	pub chapter_name: Option<String>,
	pub chapter_title: Option<String>,
	pub chapter_content: Option<String>,
}
