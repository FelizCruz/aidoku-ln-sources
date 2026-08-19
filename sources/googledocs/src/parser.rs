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

pub struct DocModel {
	pub chars: Vec<char>,
	pub flags: Vec<u8>,
}

pub fn parse_google_doc_model(html: &str) -> DocModel {
	// 1. Collect all "is" string operations with their ibi
	let mut text_ops: Vec<(usize, Vec<char>)> = Vec::new();
	let mut max_char_pos = 0usize;

	let mut pos = 0;
	let is_pattern = "\"ty\":\"is\"";
	while let Some(idx) = html[pos..].find(is_pattern) {
		let match_start = pos + idx;
		let search_start = if match_start > 50 { match_start - 50 } else { 0 };
		let search_end = core::cmp::min(match_start + 40000, html.len());
		let block = &html[search_start..search_end];

		let mut ibi_val = None;
		if let Some(ibi_idx) = block.find("\"ibi\":") {
			let after = &block[ibi_idx + 6..];
			let num_str = after.trim_start().split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
			if let Ok(num) = num_str.parse::<usize>() {
				ibi_val = Some(num);
			}
		}

		if let (Some(ibi), Some(s_idx)) = (ibi_val, block.find("\"s\":\"")) {
			let sub = &block[s_idx + 5..];
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
			let char_vec: Vec<char> = unescaped.chars().collect();
			let end_pos = ibi.saturating_sub(1) + char_vec.len();
			if end_pos > max_char_pos {
				max_char_pos = end_pos;
			}
			text_ops.push((ibi, char_vec));
			pos = search_start + s_idx + 5 + end_idx + 1;
		} else {
			pos = match_start + is_pattern.len();
		}
	}

	// 2. Build character grid
	let mut chars = vec![' '; max_char_pos];
	for (ibi, char_vec) in text_ops {
		let start = ibi.saturating_sub(1);
		for (offset, c) in char_vec.into_iter().enumerate() {
			let p = start + offset;
			if p < chars.len() {
				chars[p] = c;
			}
		}
	}

	// 3. Extract styles
	let mut flags = vec![0u8; chars.len()];
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
				let start = si.saturating_sub(1);
				let end = core::cmp::min(ei, chars.len());
				let mask = (if is_bold { 1 } else { 0 }) | (if is_italic { 2 } else { 0 });
				for k in start..end {
					flags[k] |= mask;
				}
			}
		}

		pos = match_start + st_pattern.len();
	}

	DocModel { chars, flags }
}

pub fn find_chapter_boundaries(chars: &[char]) -> Vec<ChapterEntry> {
	let mut marker_positions = Vec::new();
	let len = chars.len();

	let mut i = 0;
	while i < len {
		let is_start = i == 0 || chars[i - 1] == '\n' || chars[i - 1] == '\r' || chars[i - 1] == '\u{000C}';
		if is_start {
			let remaining = &chars[i..];
			let is_chap = (remaining.len() >= 9
				&& remaining[0..8].iter().collect::<String>().to_lowercase() == "chapter "
				&& remaining[8].is_ascii_digit())
				|| (remaining.len() >= 9
					&& remaining[0..8].iter().collect::<String>().to_lowercase() == "episode "
					&& remaining[8].is_ascii_digit())
				|| (remaining.len() >= 5
					&& remaining[0..4].iter().collect::<String>().to_lowercase() == "ch. "
					&& remaining[4].is_ascii_digit());

			if is_chap {
				marker_positions.push(i);
			}
		}
		i += 1;
	}

	if marker_positions.is_empty() {
		return vec![ChapterEntry {
			index: 1,
			title: "Full Document".to_string(),
			start_pos: 0,
			end_pos: chars.len(),
		}];
	}

	let mut chapters = Vec::with_capacity(marker_positions.len());
	for (idx, &start) in marker_positions.iter().enumerate() {
		let end = if idx + 1 < marker_positions.len() {
			marker_positions[idx + 1]
		} else {
			chars.len()
		};

		let slice = &chars[start..end];
		let mut first_line = String::new();
		for &c in slice {
			if c == '\n' || c == '\r' {
				break;
			}
			first_line.push(c);
		}
		let trimmed_title = first_line.trim_start_matches('\u{000C}').trim();
		let title = if trimmed_title.is_empty() {
			format!("Chapter {}", idx + 1)
		} else {
			trimmed_title.to_string()
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

pub fn build_chapter_markdown(chars: &[char], flags: &[u8], start: usize, end: usize) -> String {
	let bounded_start = core::cmp::min(start, chars.len());
	let bounded_end = core::cmp::min(end, chars.len());
	if bounded_start >= bounded_end {
		return String::new();
	}

	let mut lines: Vec<(Vec<char>, Vec<u8>)> = Vec::new();
	let mut cur_line_chars = Vec::new();
	let mut cur_line_flags = Vec::new();

	for i in bounded_start..bounded_end {
		let c = chars[i];
		let fl = if i < flags.len() { flags[i] } else { 0 };

		if c == '\n' {
			lines.push((cur_line_chars, cur_line_flags));
			cur_line_chars = Vec::new();
			cur_line_flags = Vec::new();
		} else if c == '\r' {
			continue;
		} else {
			cur_line_chars.push(c);
			cur_line_flags.push(fl);
		}
	}
	if !cur_line_chars.is_empty() {
		lines.push((cur_line_chars, cur_line_flags));
	}

	let mut formatted_paragraphs = Vec::new();

	for (line_chars, line_flags) in lines {
		let line_str: String = line_chars.iter().collect();
		let trimmed = line_str.trim();
		if trimmed.is_empty() {
			continue;
		}

		if trimmed == "\u{000C}" || trimmed == "\u{000C}\u{000C}" {
			formatted_paragraphs.push("---".to_string());
			continue;
		}

		let lower = trimmed.to_lowercase();
		if (lower.starts_with("chapter ") && trimmed.len() > 8 && trimmed.as_bytes()[8].is_ascii_digit())
			|| (lower.starts_with("episode ") && trimmed.len() > 8 && trimmed.as_bytes()[8].is_ascii_digit())
			|| (lower.starts_with("ch. ") && trimmed.len() > 4 && trimmed.as_bytes()[4].is_ascii_digit())
		{
			formatted_paragraphs.push(format!("## {trimmed}"));
			continue;
		}

		let mut p_out = String::with_capacity(line_chars.len() + 16);
		let mut in_bold = false;
		let mut in_italic = false;

		for (&c, &fl) in line_chars.iter().zip(line_flags.iter()) {
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

			// Escape literal '*' so Markdown does not strip star symbols
			if c == '*' {
				p_out.push_str("\\*");
			} else {
				p_out.push(c);
			}
		}

		if in_italic {
			p_out.push('*');
		}
		if in_bold {
			p_out.push_str("**");
		}

		let mut res = p_out.trim().to_string();
		if res.starts_with("[**") && res.ends_with("]**") {
			res = format!("**[{}]**", &res[3..res.len() - 3]);
		}
		if res.starts_with("[**") && res.contains("]**") {
			res = res.replace("[**", "**[").replace("]**", "]**");
		}
		while res.contains("****") {
			res = res.replace("****", "");
		}

		let final_str = res.trim();
		if !final_str.is_empty() {
			formatted_paragraphs.push(final_str.to_string());
		}
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
