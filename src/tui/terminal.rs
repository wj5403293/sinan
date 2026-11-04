use colored::Colorize;
use inquire::Text;
use std::io::{self, Write};
use terminal_size::terminal_size;

pub struct Terminal;
impl Terminal {
    fn width() -> usize {
        terminal_size().map_or(80, |(w, _)| w.0 as usize)
    }

    pub fn line() -> String {
        "=".repeat(Self::width()).green().to_string()
    }

    pub fn clear() {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
        let _ = io::stdout().flush();
    }

    pub fn center(text: &str) -> String {
        let w = Self::width();

        let max_text_width = w.saturating_sub(8);
        if max_text_width == 0 {
            return format!("{text:^w$}").trim_end().to_string();
        }

        let mut result = Vec::new();

        for line in text.lines() {
            let mut current_line = String::new();
            let mut current_width = 0;

            for c in line.chars() {
                if current_width >= max_text_width {
                    result.push(format!("{current_line:^w$}").trim_end().to_string());
                    current_line.clear();
                    current_width = 0;
                }
                current_line.push(c);
                current_width += 1;
            }

            result.push(format!("{current_line:^w$}").trim_end().to_string());
        }

        if result.is_empty() {
            return String::new();
        }

        result.join("\n")
    }

    pub fn wait_enter() {
        let _ = Text::new("按回车键继续...").prompt();
    }
}
