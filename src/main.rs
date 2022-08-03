use std::{env, path::{PathBuf, Path}, fs};
use comment_parser::CommentParser;
use ignore::Walk;
use regex::Regex;

// TODO: Line number location breaks when multiple remainders exist with same content

#[derive(Debug)]
struct Reminder {
	file: PathBuf,
	row: u32,
	col: u32,
	contents: String
}


fn list_reminders<P>(path: P) -> Vec<Reminder>
where
	P: AsRef<Path>
{
	let mut reminders = vec![];
	let reminder_pattern = Regex::new(r"^[A-Z]+:").unwrap();

	for result in Walk::new(path) {
		if result.is_err() { continue; }

		let entry = result.unwrap();
		let path = entry.path();
		if !path.is_file() { continue; }

		let rules = comment_parser::get_syntax_from_path(path);
		if rules.is_err() { continue; }

		let rules = rules.unwrap();
		let file_contents = fs::read_to_string(path).unwrap();
		let parser = CommentParser::new(&file_contents, rules);
		for comment in parser {
			let text = comment.text().trim_start();
			if reminder_pattern.is_match(text) {
				let occurence = file_contents.find(text).unwrap();
				let row = (file_contents.chars().take(occurence).filter(|c| *c == '\n').count()+1) as u32;
				let col = (occurence - file_contents[..occurence].rfind('\n').unwrap_or(0)) as u32;

				reminders.push(Reminder {
					file: path.to_path_buf(),
					row,
					col,
					contents: text.to_string()
				})
			}
		}
	}

	reminders
}

fn main() {
	let search_dir;
	if let Some(path) = env::args().nth(1) {
		search_dir = path;
	} else {
		search_dir = ".".to_string();
	}

	for reminder in list_reminders(search_dir) {
		println!("{}:{}:{}:{}", reminder.file.display(), reminder.row, reminder.col, reminder.contents);
	}
}
