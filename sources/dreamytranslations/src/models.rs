#![allow(dead_code)]

use aidoku::alloc::{String, Vec};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProjectItem {
	pub id: Option<i64>,
	pub title: Option<String>,
	pub slug: Option<String>,
	pub synopsis: Option<String>,
	pub short_synopsis: Option<String>,
	pub author: Option<String>,
	pub genres: Option<Vec<String>>,
	pub tags: Option<Vec<String>>,
	pub status: Option<String>,
	pub completed: Option<bool>,
	pub total_chapters: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterItem {
	pub id: Option<i64>,
	pub title: Option<String>,
	pub index: Option<f32>,
	pub free: Option<bool>,
}
