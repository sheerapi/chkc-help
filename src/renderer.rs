//! takes a [`HelpPage`] and paints it with termimad, scrolling when the text is too long.

use std::io::{stdout, Write};

use termimad::crossterm::event::KeyCode::*;
use termimad::crossterm::event::{self, Event};
use termimad::crossterm::style::{Color, Stylize};
use termimad::crossterm::{queue, QueueableCommand};
use termimad::MadView;

use crate::help_page::HelpPage;
use crate::theme::HelpTheme;

/// render a help page, falling back to a scrollable view if it doesn't fit on screen.
pub fn render_command_help(theme: &HelpTheme, page: &HelpPage) {
    let mut md = String::new();

    render_header(&mut md, page);
    render_usage(&mut md, page);
    render_subcommands(&mut md, page);
    render_positionals(&mut md, page);
    render_options(&mut md, page);
    render_examples(&mut md, page);
    render_notes(&mut md, page);

    let (_, rows) = termimad::crossterm::terminal::size().unwrap();
    if md.lines().count() > rows.into() {
        let _ = run_scrollable_help(theme, &page.app_name, md);
    } else {
        let mut doc_skin = theme.skin.clone();
        doc_skin.headers[0].align = termimad::Alignment::Left;
        print!("{}", doc_skin.term_text(&md));
    }
}

fn render_header(md: &mut String, page: &HelpPage) {
    if !page.path.is_empty() {
        md.push_str(&format!("# {} {}\n", page.app_name, page.path))
    } else if let Some(version) = &page.version {
        md.push_str(&format!("# {} v{}\n", page.app_name, version));
    } else {
        md.push_str(&format!("# {}\n", page.app_name));
    }

    if let Some(summary) = &page.summary {
        if !summary.is_empty() {
            md.push_str(&format!("{}\n", summary));
            md.push_str("\n");
        }
    }

    if let Some(desc) = &page.description {
        if !desc.is_empty() {
            md.push_str(&format!("{}\n", desc));
            md.push_str("\n");
        }
    }
}

fn render_usage(md: &mut String, page: &HelpPage) {
    let mut usage = page.usage.replace("Usage:", "").trim().to_owned();
    let binding = std::env::current_exe().expect("Failed to get executable path");
    let exec_name = binding
        .file_name()
        .expect("Failed to get executable name")
        .to_str()
        .unwrap();

    if !usage.starts_with(exec_name) {
        usage.insert_str(0, &format!("{} ", exec_name));
    }

    md.push_str(&format!("**Usage:** `{}`\n", usage));
}

fn render_subcommands(md: &mut String, page: &HelpPage) {
    if page.subcommands.is_empty() {
        return;
    }

    md.push_str("**Subcommands:**\n");

    md.push_str("|:-|:-\n");
    md.push_str("| command | description |\n");
    md.push_str("|:-|:-\n");

    for sc in &page.subcommands {
        md.push_str(&format!(
            "| {} | {} |\n",
            sc.name,
            sc.summary.as_deref().unwrap_or("")
        ));
    }

    md.push_str("|-\n");
    md.push_str("\n");
}

fn render_positionals(md: &mut String, page: &HelpPage) {
    if page.positionals.is_empty() {
        return;
    }

    md.push_str("**Arguments:**\n");

    for arg in &page.positionals {
        md.push_str(&format!(
            "* `{}`: {} *({}{})*\n",
            arg.name,
            arg.description.as_deref().unwrap_or(""),
            if arg.required {
                "~~required~~"
            } else {
                "~~optional~~"
            },
            if arg.multiple { ", ~~multiple~~" } else { "" }
        ));
    }

    md.push_str("\n");
}

fn render_options(md: &mut String, page: &HelpPage) {
    if page.options.is_empty() {
        return;
    }

    md.push_str("**Options:**\n");

    md.push_str("|:-:|:-:|-\n");
    md.push_str("|short|long|description|\n");
    md.push_str("|:-:|:-|-\n");

    for opt in &page.options {
        let mut name_short = String::new();
        let mut name_long = String::new();
        let mut desc = opt.description.clone();

        if let Some(short) = opt.short {
            name_short.push_str(&format!("-{}", short));
        }

        if let Some(long) = &opt.long {
            name_long.push_str(&format!("--{}", long));
        }

        if let Some(val) = &opt.value {
            if !name_short.is_empty() {
                name_short.push_str(&format!(" ~~<{}>~~", val.to_ascii_lowercase()));
            }

            name_long.push_str(&format!(" ~~<{}>~~", val.to_ascii_lowercase()));
        }

        if !opt.default.is_empty() {
            desc.push_str(&format!(" *(defaults to {})*", opt.default));
        }

        md.push_str(&format!("| {} | {} | {}\n", name_short, name_long, desc));
    }

    md.push_str("|-\n");

    md.push_str("\n");
}

fn render_examples(md: &mut String, page: &HelpPage) {
    if page.examples.is_empty() {
        return;
    }

    md.push_str("**Examples:**\n");

    let mark = page
        .examples
        .iter()
        .enumerate()
        .map(|(i, e)| format!("~~{})~~ {}", i + 1, e))
        .collect::<Vec<_>>()
        .join("\n");

    md.push_str(&mark);
    md.push_str("\n\n");
}

fn render_notes(md: &mut String, page: &HelpPage) {
    if page.notes.is_empty() {
        return;
    }

    md.push_str("**Notes:**\n");

    let mark = page
        .notes
        .iter()
        .map(|n| format!("- {}", n))
        .collect::<Vec<_>>()
        .join("\n");

    md.push_str(&mark);
}

fn view_area() -> termimad::Area {
    let mut area = termimad::Area::full_screen();
    if area.width <= 120 {
        area.height -= 1;
    }
    area.pad_for_max_width(80);
    area
}

fn draw_vertical_legend<W: Write>(
    out: &mut W,
    app_name: &str,
    accent: Color,
) -> anyhow::Result<()> {
    let legend = [
        format!("{}", format!("{app_name} Help").with(accent).bold()),
        format!(
            "{} {} {} {}",
            "↑".with(accent).bold(),
            "/".dark_grey(),
            "↓".with(accent).bold(),
            "Scroll".dark_grey()
        ),
        format!(
            "{}{}{} {}",
            "PgUp".with(accent).bold(),
            "/".dark_grey(),
            "Dn".with(accent).bold(),
            "Page".dark_grey()
        ),
        format!("{} {}", "Mouse".with(accent).bold(), "Scroll".dark_grey()),
        format!(
            "{} {} {} {}",
            "q".red().bold(),
            "/".dark_grey(),
            "Esc".red().bold(),
            "Quit".dark_grey()
        ),
    ];

    let x = 0;
    let y = 0;

    for (i, line) in legend.iter().enumerate() {
        out.queue(termimad::crossterm::style::SetBackgroundColor(
            termimad::crossterm::style::Color::Black,
        ))?;

        out.queue(termimad::crossterm::cursor::MoveTo(x, y + i as u16))?;
        out.queue(termimad::crossterm::style::Print(line))?;
    }

    out.queue(termimad::crossterm::style::ResetColor)?;

    Ok(())
}

fn visible_width(s: &str) -> usize {
    let stripped = strip_ansi_escapes::strip(s);

    let stripped = std::str::from_utf8(&stripped).unwrap_or("");
    unicode_width::UnicodeWidthStr::width(stripped)
}

fn join_justify_between(items: &[String], width: u16) -> String {
    let visible_total: usize = items.iter().map(|s| visible_width(s)).sum();
    let gaps = items.len().saturating_sub(1);

    if gaps == 0 || visible_total >= width as usize {
        return items.join(" ");
    }

    let remaining = width as usize - visible_total;
    let space = remaining / gaps;
    let extra = remaining % gaps;

    let mut out = String::new();

    for (i, item) in items.iter().enumerate() {
        out.push_str(item);

        if i < gaps {
            let pad = space + if i < extra { 1 } else { 0 };
            out.push_str(&" ".repeat(pad));
        }
    }

    out
}

fn draw_horizontal_legend<W: Write>(
    out: &mut W,
    area: &termimad::Area,
    app_name: &str,
    accent: Color,
) -> anyhow::Result<()> {
    let items = [
        format!("{}", format!("{app_name} Help").with(accent).bold()),
        format!(
            "{} {} {} {}",
            "↑".with(accent).bold(),
            "/".dark_grey(),
            "↓".with(accent).bold(),
            "Scroll".dark_grey()
        ),
        format!(
            "{} {} {} {}",
            "PgUp".with(accent).bold(),
            "/".dark_grey(),
            "Dn".with(accent).bold(),
            "Page".dark_grey()
        ),
        format!("{} {}", "Mouse".with(accent).bold(), "Scroll".dark_grey()),
        format!(
            "{} {} {} {}",
            "q".red().bold(),
            "/".dark_grey(),
            "Esc".red().bold(),
            "Quit".dark_grey()
        ),
    ];

    let line = join_justify_between(&items, area.width);

    out.queue(termimad::crossterm::cursor::MoveTo(0, area.height - 1))?;
    out.queue(termimad::crossterm::style::Print(line))?;

    Ok(())
}

fn draw_legend<W: Write>(
    out: &mut W,
    area: &termimad::Area,
    app_name: &str,
    accent: Color,
) -> anyhow::Result<()> {
    if area.width > 120 {
        draw_vertical_legend(out, app_name, accent)
    } else {
        draw_horizontal_legend(out, area, app_name, accent)
    }
}

/// open a scrollable markdown view with keyboard and mouse shortcuts.
pub fn run_scrollable_help(
    theme: &HelpTheme,
    app_name: &str,
    markdown: String,
) -> anyhow::Result<()> {
    let mut w = stdout();
    queue!(w, termimad::crossterm::terminal::EnterAlternateScreen)?;
    termimad::crossterm::terminal::enable_raw_mode()?;
    queue!(w, termimad::crossterm::cursor::Hide)?;
    let mut view = MadView::from(markdown, view_area(), theme.skin.clone());

    let mut term_area = termimad::Area::full_screen();

    loop {
        view.write_on(&mut w)?;
        draw_legend(&mut stdout(), &term_area, app_name, theme.accent)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(event::KeyEvent { code, .. })) => match code {
                Up => view.try_scroll_lines(-1),
                Down => view.try_scroll_lines(1),
                Char('j') => view.try_scroll_lines(-1),
                Char('k') => view.try_scroll_lines(1),
                Char('q') => break,
                Esc => break,
                PageUp => view.try_scroll_pages(-1),
                PageDown => view.try_scroll_pages(1),
                _ => {}
            },
            Ok(Event::Resize(..)) => {
                term_area = termimad::Area::full_screen();

                queue!(
                    w,
                    termimad::crossterm::terminal::Clear(
                        termimad::crossterm::terminal::ClearType::All
                    )
                )?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }

    termimad::crossterm::terminal::disable_raw_mode()?;
    queue!(w, termimad::crossterm::cursor::Show)?;
    queue!(w, termimad::crossterm::terminal::LeaveAlternateScreen)?;
    w.flush()?;
    Ok(())
}
