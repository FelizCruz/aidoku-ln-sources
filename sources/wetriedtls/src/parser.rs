use aidoku::alloc::{string::ToString, String, Vec};

pub fn clean_html_tags(html: &str) -> String {
	let mut result = String::new();
	let mut in_tag = false;
	let mut current_tag = String::new();
	let mut skip_tag = false;
	let mut skip_tag_name = String::new();
	let mut chars = html.chars().peekable();

	while let Some(c) = chars.next() {
		if c == '<' {
			in_tag = true;
			current_tag.clear();
			continue;
		}

		if c == '>' && in_tag {
			in_tag = false;
			let tag_str = current_tag.trim().to_lowercase();

			if skip_tag {
				let end_name = "/".to_string() + &skip_tag_name;
				if tag_str.starts_with(&end_name) {
					skip_tag = false;
					skip_tag_name.clear();
				}
				continue;
			}

			if tag_str.starts_with("style")
				|| tag_str.starts_with("script")
				|| tag_str.starts_with("noscript")
				|| tag_str.starts_with("head")
				|| tag_str.starts_with("nav")
				|| tag_str.starts_with("header")
				|| tag_str.starts_with("footer")
			{
				skip_tag = true;
				let space_idx = tag_str.find(' ').unwrap_or(tag_str.len());
				skip_tag_name = tag_str[..space_idx].to_string();
				continue;
			}

			if tag_str == "/p"
				|| tag_str == "/h1"
				|| tag_str == "/h2"
				|| tag_str == "/h3"
				|| tag_str == "/h4"
			{
				result.push_str("\n\n");
			} else if tag_str == "br" || tag_str == "br/" || tag_str == "br /" {
				result.push('\n');
			} else if tag_str == "strong"
				|| tag_str == "b"
				|| tag_str == "/strong"
				|| tag_str == "/b"
			{
				result.push_str("**");
			} else if tag_str == "em" || tag_str == "i" || tag_str == "/em" || tag_str == "/i" {
				result.push('*');
			} else if tag_str.starts_with("h1") {
				result.push_str("# ");
			} else if tag_str.starts_with("h2") {
				result.push_str("## ");
			} else if tag_str.starts_with("h3") || tag_str.starts_with("h4") {
				result.push_str("### ");
			} else if tag_str == "hr" || tag_str == "hr/" || tag_str == "hr /" {
				result.push_str("\n\n---\n\n");
			}
			continue;
		}

		if in_tag {
			current_tag.push(c);
			continue;
		}

		if skip_tag {
			continue;
		}

		if c == '&' {
			let mut entity = String::new();
			while let Some(&next_c) = chars.peek() {
				if next_c == ';' || entity.len() >= 8 || next_c == ' ' || next_c == '<' {
					if next_c == ';' {
						chars.next();
					}
					break;
				}
				entity.push(chars.next().unwrap());
			}

			match entity.as_str() {
				"nbsp" => result.push(' '),
				"amp" => result.push('&'),
				"lt" => result.push('<'),
				"gt" => result.push('>'),
				"quot" => result.push('"'),
				"#39" | "apos" | "#x27" => result.push('\''),
				"mdash" => result.push_str("—"),
				"ndash" => result.push_str("–"),
				"hellip" => result.push_str("…"),
				_ => {
					result.push('&');
					result.push_str(&entity);
					result.push(';');
				}
			}
			continue;
		}

		result.push(c);
	}

	result
}

pub fn html_to_markdown(html: &str) -> String {
	let cleaned = clean_html_tags(html);
	let mut lines = Vec::new();
	for line in cleaned.split('\n') {
		let trimmed = line.trim();
		if !trimmed.is_empty() {
			lines.push(trimmed);
		}
	}
	lines.join("\n\n")
}
