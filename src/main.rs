use std::{env, path::Path, fs, collections::HashSet};
use comment_parser::CommentParser;
use ignore::Walk;
use regex::Regex;

// TODO: Add language support for tsx, jsx, lua

#[derive(Debug)]
struct Reminder {
	row: u32,
	col: u32,
	contents: String
}

static ALLOWED_VERBS: [&str; 3] = ["TODO", "FIXME", "BUG"];

fn get_row_and_column(contents: &str, substr: &str, from: usize) -> (u32, u32) {
	let occurence = contents[from..].find(substr).unwrap() + from;
	let row = (contents.chars().take(occurence).filter(|c| *c == '\n').count()+1) as u32;
	let col = (occurence - contents[..occurence].rfind('\n').unwrap_or(0)) as u32;

	(row, col)
}

fn list_reminders<P: AsRef<Path> + ?Sized>(path: &P) -> Option<Vec<Reminder>>
{
	let mut reminders = vec![];
	let reminder_pattern: Regex = Regex::new(r"^[A-Z]+.*:").unwrap();

	let rules = comment_parser::get_syntax_from_path(path);
	if rules.is_err() { return None; }

	let rules = rules.unwrap();
	let file_contents = fs::read_to_string(path).unwrap();
	let parser = CommentParser::new(&file_contents, rules);
	let mut search_from = 0;
	for comment in parser {
		let text = comment.text().trim_start();
		for line in text.lines() {
			if !reminder_pattern.is_match(line) { continue; }

			let is_allowed = ALLOWED_VERBS.into_iter().any(|v| line.starts_with(v));
			if !is_allowed { continue; }

			let (row, col) = get_row_and_column(&file_contents, line, search_from);
			reminders.push(Reminder {
				row,
				col,
				contents: line.into()
			});

			search_from += file_contents[search_from..].find(line).unwrap()+1;
		}
	}

	Some(reminders)
}

fn main() {
	let search_dir = if env::args().count() > 1 {
		env::args().skip(1).collect()
	} else {
		vec![".".into()]
	};

	let mut showed_files = HashSet::new();
	for dir in search_dir {
		for result in Walk::new(dir) {
			if result.is_err() { continue; }

			let entry = result.unwrap();
			let path = entry.path();
			if !path.is_file() { continue; }

			let canonical = path.canonicalize().unwrap();
			if showed_files.contains(&canonical) { continue; }

			let reminders = list_reminders(path);
			if let Some(reminders) = reminders {
				for reminder in reminders{
					println!("{}:{}:{}:{}", path.display(), reminder.row, reminder.col, reminder.contents);
				}
			}

			showed_files.insert(canonical);
		}
	}
}
