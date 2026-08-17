use aidoku::alloc::String;

/// Cleans and converts raw HTML chapter content to readable Markdown for Aidoku.
pub fn html_to_markdown(html: &str) -> String {
	let mut result = String::with_capacity(html.len());
	let mut in_tag = false;
	let mut current_tag = String::new();
	let mut i = 0;
	let chars: aidoku::alloc::Vec<char> = html.chars().collect();
	let len = chars.len();

	while i < len {
		let c = chars[i];

		if c == '<' {
			in_tag = true;
			current_tag.clear();
			i += 1;
			continue;
		}

		if c == '>' && in_tag {
			in_tag = false;
			let tag_lower = current_tag.trim().to_ascii_lowercase();

			if tag_lower == "p" || tag_lower.starts_with("p ") {
				// paragraph start
			} else if tag_lower == "/p" {
				result.push_str("\n\n");
			} else if tag_lower == "br" || tag_lower == "br/" || tag_lower == "br /" {
				result.push('\n');
			} else if tag_lower == "strong" || tag_lower == "b" {
				result.push_str("**");
			} else if tag_lower == "/strong" || tag_lower == "/b" {
				result.push_str("**");
			} else if tag_lower == "em" || tag_lower == "i" {
				result.push('*');
			} else if tag_lower == "/em" || tag_lower == "/i" {
				result.push('*');
			} else if tag_lower == "h1" || tag_lower.starts_with("h1 ") {
				result.push_str("# ");
			} else if tag_lower == "/h1" {
				result.push_str("\n\n");
			} else if tag_lower == "h2" || tag_lower.starts_with("h2 ") {
				result.push_str("## ");
			} else if tag_lower == "/h2" {
				result.push_str("\n\n");
			} else if tag_lower == "h3" || tag_lower.starts_with("h3 ") {
				result.push_str("### ");
			} else if tag_lower == "/h3" {
				result.push_str("\n\n");
			} else if tag_lower == "hr" || tag_lower == "hr/" || tag_lower == "hr /" {
				result.push_str("\n\n---\n\n");
			}

			i += 1;
			continue;
		}

		if in_tag {
			current_tag.push(c);
			i += 1;
			continue;
		}

		// Handle HTML entities
		if c == '&' {
			let rest: String = chars[i..core::cmp::min(i + 10, len)].iter().collect();
			if rest.starts_with("&nbsp;") {
				result.push(' ');
				i += 6;
				continue;
			} else if rest.starts_with("&amp;") {
				result.push('&');
				i += 5;
				continue;
			} else if rest.starts_with("&lt;") {
				result.push('<');
				i += 4;
				continue;
			} else if rest.starts_with("&gt;") {
				result.push('>');
				i += 4;
				continue;
			} else if rest.starts_with("&quot;") {
				result.push('"');
				i += 6;
				continue;
			} else if rest.starts_with("&#39;") || rest.starts_with("&apos;") {
				result.push('\'');
				i += if rest.starts_with("&#39;") { 5 } else { 6 };
				continue;
			}
		}

		result.push(c);
		i += 1;
	}

	clean_extra_blank_lines(result.trim())
}

fn clean_extra_blank_lines(text: &str) -> String {
	let mut cleaned = String::with_capacity(text.len());
	let mut consecutive_newlines = 0;

	for c in text.chars() {
		if c == '\n' {
			consecutive_newlines += 1;
			if consecutive_newlines <= 2 {
				cleaned.push(c);
			}
		} else {
			consecutive_newlines = 0;
			cleaned.push(c);
		}
	}

	cleaned
}
