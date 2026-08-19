use aidoku::alloc::{format, string::ToString, vec, String, Vec};

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

pub fn extract_doc_text_and_styles(html: &str) -> (String, Vec<u8>) {
	let mut full_text = String::new();
	let pattern = "\"s\":\"";
	let mut pos = 0;

	// 1. Extract all text strings
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

	let text_len = full_text.len();
	let mut flags = vec![0u8; text_len];

	// 2. Extract style spans: scan for `"st":"text"` or `"st": "text"`
	pos = 0;
	let st_pattern = "\"st\":\"text\"";
	while let Some(idx) = html[pos..].find(st_pattern) {
		let match_start = pos + idx;
		let search_start = if match_start > 150 { match_start - 150 } else { 0 };
		let search_end = core::cmp::min(match_start + 350, html.len());
		let block = &html[search_start..search_end];

		let is_bold = block.contains("\"ts_bd\":true") || block.contains("\"ts_bd\": true");
		let is_italic = block.contains("\"ts_it\":true") || block.contains("\"ts_it\": true");

		if is_bold || is_italic {
			let mut si_val = None;
			let mut ei_val = None;

			if let Some(si_idx) = block.find("\"si\":") {
				let after = &block[si_idx + 5..];
				let num_str = after.trim_start().split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
				if let Ok(num) = num_str.parse::<usize>() {
					si_val = Some(num);
				}
			}

			if let Some(ei_idx) = block.find("\"ei\":") {
				let after = &block[ei_idx + 5..];
				let num_str = after.trim_start().split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
				if let Ok(num) = num_str.parse::<usize>() {
					ei_val = Some(num);
				}
			}

			if let (Some(si), Some(ei)) = (si_val, ei_val) {
				let start = if si > 0 { si - 1 } else { 0 };
				let end = core::cmp::min(ei, text_len);
				let mask = (if is_bold { 1 } else { 0 }) | (if is_italic { 2 } else { 0 });
				for k in start..end {
					flags[k] |= mask;
				}
			}
		}

		pos = match_start + st_pattern.len();
	}

	(full_text, flags)
}

pub fn find_chapter_boundaries(text: &str) -> Vec<ChapterEntry> {
	let mut chapters = Vec::new();
	let mut marker_positions = Vec::new();

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

pub fn build_chapter_markdown(text: &str, flags: &[u8], start: usize, end: usize) -> String {
	let bounded_start = core::cmp::min(start, text.len());
	let bounded_end = core::cmp::min(end, text.len());
	if bounded_start >= bounded_end {
		return String::new();
	}

	let slice_text = &text[bounded_start..bounded_end];
	let mut formatted_paragraphs = Vec::new();
	let mut offset = bounded_start;

	for line in slice_text.lines() {
		let stripped = line.trim();
		if stripped.is_empty() {
			offset += line.len() + 1;
			continue;
		}

		// Handle form feed / page break
		if stripped == "\u{000C}" || stripped == "\u{000C}\u{000C}" {
			formatted_paragraphs.push("---".to_string());
			offset += line.len() + 1;
			continue;
		}

		let clean_line = line.trim_start_matches('\u{000C}');
		if line.starts_with('\u{000C}') && !formatted_paragraphs.is_empty() {
			formatted_paragraphs.push("---".to_string());
		}

		let trimmed_clean = clean_line.trim();

		// Chapter title heading
		let is_chapter_heading = trimmed_clean.starts_with("Chapter ")
			|| trimmed_clean.starts_with("CHAPTER ")
			|| trimmed_clean.starts_with("chapter ")
			|| trimmed_clean.starts_with("Episode ")
			|| trimmed_clean.starts_with("EPISODE ")
			|| trimmed_clean.starts_with("Ch. ");

		if is_chapter_heading {
			formatted_paragraphs.push(format!("## {trimmed_clean}"));
			offset += line.len() + 1;
			continue;
		}

		// Line flags
		let line_start = offset + (line.len() - clean_line.len());

		let mut p_out = String::with_capacity(clean_line.len() + 32);
		let mut in_bold = false;
		let mut in_italic = false;

		for (idx, ch) in clean_line.chars().enumerate() {
			let flag_idx = line_start + idx;
			let fl = if flag_idx < flags.len() { flags[flag_idx] } else { 0 };
			let is_b = (fl & 1) != 0;
			let is_i = (fl & 2) != 0;

			if is_b != in_bold {
				p_out.push_str("**");
				in_bold = is_b;
			}

			if is_i != in_italic {
				p_out.push('*');
				in_italic = is_i;
			}

			p_out.push(ch);
		}

		if in_italic {
			p_out.push('*');
		}
		if in_bold {
			p_out.push_str("**");
		}

		// Normalize bracketed formatting e.g. [** ... ]** -> **[ ... ]**
		let mut res = p_out;
		if res.starts_with("[**") && res.ends_with("]**") {
			res = format!("**[{}]**", &res[3..res.len() - 3]);
		}

		let trimmed_res = res.trim();
		if !trimmed_res.is_empty() {
			formatted_paragraphs.push(trimmed_res.to_string());
		}

		offset += line.len() + 1;
	}

	let mut result = String::new();
	for (idx, p) in formatted_paragraphs.iter().enumerate() {
		if idx > 0 {
			result.push_str("\n\n");
		}
		result.push_str(p);
	}

	result
}
