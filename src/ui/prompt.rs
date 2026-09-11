use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, stdout, Write};

pub struct PromptOption {
    pub label: String,
    pub hint: Option<String>,
}

impl PromptOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hint: None,
        }
    }

    pub fn with_hint(label: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            hint: Some(hint.into()),
        }
    }
}

fn render_menu(
    title: &str,
    description: Option<&str>,
    options: &[PromptOption],
    selected_idx: usize,
    prev_lines: u16,
) -> io::Result<u16> {
    let mut out = stdout().lock();
    
    execute!(out, cursor::MoveToColumn(0))?;
    if prev_lines > 0 {
        execute!(out, cursor::MoveUp(prev_lines), Clear(ClearType::FromCursorDown))?;
    }

    let mut lines: u16 = 0;

    let title_line = format!("  {}  {}", "◆".bright_magenta().bold(), title.bold().bright_white());
    write!(out, "\r{}\r\n", title_line)?;
    lines += 1;

    if let Some(desc) = description {
        let desc_line = format!("  {}  {}", "│".dimmed(), desc.dimmed());
        write!(out, "\r{}\r\n", desc_line)?;
        lines += 1;
    }

    for (idx, opt) in options.iter().enumerate() {
        let is_sel = idx == selected_idx;
        let bullet = if is_sel { "●".bright_cyan().bold() } else { "○".dimmed() };
        let arrow = if is_sel { "❯".bright_cyan().bold() } else { " ".normal() };

        let label_colored = if is_sel {
            opt.label.bold().bright_cyan()
        } else {
            opt.label.dimmed()
        };

        let hint_str = if let Some(ref h) = opt.hint {
            format!(" {}", h.dimmed())
        } else {
            String::new()
        };

        let line = format!("  {}  {} {} {}{}", "│".dimmed(), arrow, bullet, label_colored, hint_str);
        write!(out, "\r{}\r\n", line)?;
        lines += 1;
    }

    write!(out, "\r  {}\r\n", "│".dimmed())?;
    lines += 1;

    out.flush()?;
    Ok(lines)
}

pub fn select_option(
    title: &str,
    description: Option<&str>,
    options: &[PromptOption],
    default_idx: usize,
) -> Option<usize> {
    if options.is_empty() {
        return None;
    }

    let mut selected = default_idx.min(options.len() - 1);
    let mut stdout = stdout();

    terminal::enable_raw_mode().ok()?;
    execute!(stdout, cursor::Hide).ok()?;

    let mut lines_rendered: u16 = render_menu(title, description, options, selected, 0).unwrap_or(0);

    let result = loop {
        if let Ok(Event::Key(KeyEvent { code, modifiers, .. })) = event::read() {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                break None;
            }

            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    } else {
                        selected = options.len() - 1;
                    }
                    lines_rendered = render_menu(title, description, options, selected, lines_rendered).unwrap_or(lines_rendered);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < options.len() {
                        selected += 1;
                    } else {
                        selected = 0;
                    }
                    lines_rendered = render_menu(title, description, options, selected, lines_rendered).unwrap_or(lines_rendered);
                }
                KeyCode::Enter => {
                    break Some(selected);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    break None;
                }
                _ => {}
            }
        }
    };

    let _ = execute!(stdout, cursor::Show);
    let _ = terminal::disable_raw_mode();

    if let Some(idx) = result {
        if lines_rendered > 0 {
            let mut out = stdout.lock();
            let _ = execute!(out, cursor::MoveToColumn(0), cursor::MoveUp(lines_rendered), Clear(ClearType::FromCursorDown));
            let check_icon = "◇".bright_green().bold();
            let chosen_label = options[idx].label.bright_white().bold();
            let collapsed_title = format!("  {}  {} {} {}", check_icon, title.dimmed(), "›".dimmed(), chosen_label);
            let _ = write!(out, "\r{}\r\n", collapsed_title);
            let _ = write!(out, "\r  {}\r\n", "│".dimmed());
            let _ = out.flush();
        }
    } else if lines_rendered > 0 {
        let mut out = stdout.lock();
        let _ = execute!(out, cursor::MoveToColumn(0), cursor::MoveUp(lines_rendered), Clear(ClearType::FromCursorDown));
        let _ = write!(out, "\r  {}  {} {}\r\n", "✖".bright_red(), title.dimmed(), "› Cancelled".bright_red());
        let _ = write!(out, "\r  {}\r\n", "│".dimmed());
        let _ = out.flush();
    }

    result
}

pub fn select_yes_no(
    title: &str,
    description: Option<&str>,
    default_yes: bool,
) -> Option<bool> {
    let opts = vec![
        PromptOption::new("No  - Cancel or keep current state"),
        PromptOption::new("Yes - Proceed"),
    ];
    let default_idx = if default_yes { 1 } else { 0 };
    select_option(title, description, &opts, default_idx).map(|idx| idx == 1)
}

pub fn input_text(
    title: &str,
    description: Option<&str>,
    default_value: Option<&str>,
) -> Option<String> {
    println!("  {}  {}", "◆".bright_magenta().bold(), title.bold().bright_white());
    if let Some(desc) = description {
        println!("  {}  {}", "│".dimmed(), desc.dimmed());
    }

    let default_hint = match default_value {
        Some(d) if !d.is_empty() => format!(" [{}]", d.dimmed()),
        _ => String::new(),
    };

    print!("  {}  {} › ", "│".dimmed(), "Enter path".cyan());
    if !default_hint.is_empty() {
        print!("{} ", default_hint);
    }
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let trimmed = input.trim().to_string();

    let final_val = if trimmed.is_empty() {
        default_value.map(|s| s.to_string())?
    } else {
        trimmed
    };

    println!("  {}  {} {} {}", "◇".bright_green().bold(), title.dimmed(), "›".dimmed(), final_val.bold().bright_white());
    println!("  {}", "│".dimmed());

    Some(final_val)
}
