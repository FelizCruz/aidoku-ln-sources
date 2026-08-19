use aidoku::alloc::{format, string::ToString, String, Vec};

pub struct ChapterEntry {
	pub index: usize,
	pub title: String,
	pub start_pos: usize,
	pub end_pos: usize,
}

pub fn extract_doc_id(input: &str) -> String {
	let trimmed = input.trim();
	if let Some(idx) = trimmed.find("/document/d/") {
		let sub = &trimmed[idx + 12..];
		let id = sub.split('/').next().unwrap_or(sub).split('?').next().unwrap_or(sub).split('#').next().unwrap_or(sub);
		return id.to_string();
	}
	trimmed.split('?').next().unwrap_or(trimmed).split('#').next().unwrap_or(trimmed).to_string()
}

pub fn unescape_json_string(s: &str) -> String {
	let mut res = String::with_capacity(s.len());
	let mut chars = s.chars().peekable();

	while let Some(c) = chars.next() {
		if c == '\\' {
			if let Some(next) = chars.next() {
				match next {
					'n' => res.push('\n'),
					'r' => res.push('\r'),
					't' => res.push('\t'),
					'\"' => res.push('\"'),
					'\\' => res.push('\\'),
					'/' => res.push('/'),
					'b' => res.push('\u{0008}'),
					'f' => res.push('\u{000C}'),
					'u' => {
						// 4 hex digits
						let mut hex = String::with_capacity(4);
						for _ in 0..4 {
							if let Some(&h) = chars.peek() {
								if h.is_ascii_hexdigit() {
									hex.push(h);
									chars.next();
								} else {
									break;
								}
							}
						}
						if hex.len() == 4 {
							if let Ok(code) = u32::from_str_radix(&hex, 16) {
								if let Some(unicode_char) = core::char::from_u32(code) {
									res.push(unicode_char);
								}
							}
						}
					}
					_ => {
						res.push('\\');
						res.push(next);
					}
				}
			}
		} else {
			res.push(c);
		}
	}

	res
}

pub fn extract_doc_text_from_html(html: &str) -> String {
	let mut full_text = String::new();
	let pattern = "\"s\":\"";
	let mut pos = 0;

	while let Some(idx) = html[pos..].find(pattern) {
		let abs_start = pos + idx + pattern.len();
		let sub = &html[abs_start..];

		let mut in_escape = false;
		let mut end_idx = sub.len();

		for (byte_idx, b) in sub.bytes().enumerate() {
			if in_escape {
				in_escape = false;
			} else if b == b'\\' {
				in_escape = true;
			} else if b == b'\"' {
				end_idx = byte_idx;
				break;
			}
		}

		let raw_str = &sub[..end_idx];
		let unescaped = unescape_json_string(raw_str);
		full_text.push_str(&unescaped);

		pos = abs_start + end_idx + 1;
	}

	full_text
}

pub fn find_chapter_boundaries(text: &str) -> Vec<ChapterEntry> {
	let mut chapters = Vec::new();
	let mut marker_positions = Vec::new();

	// Scan for chapter markers (e.g., "\nChapter ", "\x0cChapter ", "\rChapter ")
	let bytes = text.as_bytes();
	let len = bytes.len();
	let mut i = 0;

	while i < len {
		let is_start = i == 0 || bytes[i - 1] == b'\n' || bytes[i - 1] == b'\r' || bytes[i - 1] == 0x0C;
		if is_start {
			let slice = &text[i..];
			let matches_chapter = slice.starts_with("Chapter ")
				|| slice.starts_with("CHAPTER ")
				|| slice.starts_with("chapter ")
				|| slice.starts_with("Episode ")
				|| slice.starts_with("EPISODE ")
				|| slice.starts_with("Ch. ")
				|| slice.starts_with("ch. ");

			if matches_chapter {
				// Verify following character is a digit
				let prefix_len = if slice.starts_with("Chapter ") || slice.starts_with("CHAPTER ") || slice.starts_with("chapter ") || slice.starts_with("Episode ") || slice.starts_with("EPISODE ") {
					8
				} else {
					4 // "Ch. "
				};
				if slice.len() > prefix_len && slice.as_bytes()[prefix_len].is_ascii_digit() {
					marker_positions.push(i);
				}
			}
		}
		i += 1;
	}

	if marker_positions.is_empty() {
		// Single continuous chapter
		chapters.push(ChapterEntry {
			index: 1,
			title: "Full Document".to_string(),
			start_pos: 0,
			end_pos: text.len(),
		});
		return chapters;
	}

	for (idx, &start) in marker_positions.iter().enumerate() {
		let end = if idx + 1 < marker_positions.len() {
			marker_positions[idx + 1]
		} else {
			text.len()
		};

		let slice = &text[start..end];
		let first_line = slice.lines().next().unwrap_or("").trim();
		let title = if first_line.is_empty() {
			format!("Chapter {}", idx + 1)
		} else {
			first_line.to_string()
		};

		chapters.push(ChapterEntry {
			index: idx + 1,
			title,
			start_pos: start,
			end_pos: end,
		});
	}

	chapters
}

pub fn get_chapter_content(text: &str, start: usize, end: usize) -> String {
	let bounded_start = core::cmp::min(start, text.len());
	let bounded_end = core::cmp::min(end, text.len());
	if bounded_start >= bounded_end {
		return String::new();
	}
	let raw = &text[bounded_start..bounded_end];
	raw.replace('\u{000C}', "\n\n").trim().to_string()
}
