use aidoku::alloc::String;

pub fn html_to_markdown(html: &str) -> String {
	let mut result = String::new();
	let mut in_tag = false;
	let mut current_tag = String::new();
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
			if tag_str == "/p" || tag_str == "/h1" || tag_str == "/h2" || tag_str == "/h3" {
				result.push_str("\n\n");
			} else if tag_str == "br" || tag_str == "br/" || tag_str == "br /" {
				result.push('\n');
			} else if tag_str == "strong" || tag_str == "b" || tag_str == "/strong" || tag_str == "/b" {
				result.push_str("**");
			} else if tag_str == "em" || tag_str == "i" || tag_str == "/em" || tag_str == "/i" {
				result.push('*');
			} else if tag_str.starts_with("h1") {
				result.push_str("# ");
			} else if tag_str.starts_with("h2") {
				result.push_str("## ");
			} else if tag_str.starts_with("h3") {
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
				"#39" | "apos" => result.push('\''),
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
