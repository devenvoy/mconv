use colored::*;

pub fn print_banner() {
    println!();

    let banner_lines = [
        "  ███╗   ███╗ ██████╗ ██████╗ ███╗   ██╗██╗   ██╗",
        "  ████╗ ████║██╔════╝██╔═══██╗████╗  ██║██║   ██║",
        "  ██╔████╔██║██║     ██║   ██║██╔██╗ ██║██║   ██║",
        "  ██║╚██╔╝██║██║     ██║   ██║██║╚██╗██║╚██╗ ██╔╝",
        "  ██║ ╚═╝ ██║╚██████╗╚██████╔╝██║ ╚████║ ╚████╔╝ ",
        "  ╚═╝     ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝  ╚═══╝  ",
    ];

    let colors = [
        (56, 189, 248),  // #38bdf8 sky blue
        (96, 165, 250),  // #60a5fa light blue
        (129, 140, 248), // #818cf8 indigo
        (167, 139, 250), // #a78bfa purple
        (192, 132, 252), // #c084fc violet
        (232, 121, 249), // #e879f9 fuchsia
    ];

    for (line, (r, g, b)) in banner_lines.iter().zip(colors.iter()) {
        println!("\x1b[38;2;{};{};{}m\x1b[1m{}\x1b[0m", r, g, b, line);
    }
    println!();

    let top = "  ╭──────────────────────────────────────────────────────────────╮";
    let bottom = "  ╰──────────────────────────────────────────────────────────────╯";
    let border = "│";

    println!("\x1b[38;2;96;165;250m\x1b[1m{}\x1b[0m", top);
    println!(
        "  \x1b[38;2;96;165;250m\x1b[1m{}\x1b[0m  {} {}  {}             {}  \x1b[38;2;96;165;250m\x1b[1m{}\x1b[0m",
        border,
        "●".bright_green(),
        "mconv".bold().bright_white(),
        "v2.0.0".cyan(),
        "High-Performance Media Engine".bright_yellow().bold(),
        border
    );
    println!(
        "  \x1b[38;2;96;165;250m\x1b[1m{}\x1b[0m  {: <58}  \x1b[38;2;96;165;250m\x1b[1m{}\x1b[0m",
        border,
        "Fast, defensive batch audio/video/image transcoder".dimmed(),
        border
    );
    println!("\x1b[38;2;96;165;250m\x1b[1m{}\x1b[0m", bottom);
    println!();
}

pub fn print_section_header(title: &str, description: Option<&str>) {
    println!();
    println!(
        "  {} {}",
        "◆".bright_magenta().bold(),
        title.bold().bright_white()
    );
    if let Some(desc) = description {
        println!("  {} {}", "│".dimmed(), desc.dimmed());
    }
}
