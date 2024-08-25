use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::PathBuf;

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use chrono::Local;
use clap::{Parser, Subcommand};

mod models;
use models::strip_ansi_codes;
use models::vault_styling;
use models::Snippet;

const DATA_FILE: &str = "data/codevault.json";

#[derive(Parser)]
#[command(author, version, about = "Codevault", propagate_version = true, long_about = None, styles(vault_styling()))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, help = "Add a description to the provided code snippet")]
    description: Option<String>,

    #[arg(
        short,
        long,
        help = "Unique ID automatically assigned for identification of the snippets"
    )]
    id: Option<u32>,

    #[arg(
        short,
        long,
        help = "Select a programming language for syntax highlighting"
    )]
    language: Option<String>,

    #[arg(short, long, help = "Apply relevant tags to categorize the snippets")]
    tag: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Add a new code snippet to your collection")]
    Capture {
        #[arg(
            short = 'd',
            long = "description",
            help = "Add a description to the provided code snippet"
        )]
        description: String,

        #[arg(
            short = 'l',
            long = "language",
            help = "Select a programming language for syntax highlighting"
        )]
        language: String,

        #[arg(
            short = 't',
            long = "tag",
            help = "Apply relevant tags to categorize the snippets"
        )]
        tag: String,
    },
    #[command(about = "Show the code of a specified snippet using IDs")]
    Copy {
        #[arg(
            short = 'i',
            long = "id",
            help = "Unique ID automatically assigned for identification of the snippets"
        )]
        id: Option<u32>,
    },
    #[command(
        about = "Remove one or more code snippets by specifying their IDs, separated by commas"
    )]
    Delete {
        #[arg(
            short = 'i',
            long = "id",
            help = "Unique ID automatically assigned for identification of the snippets"
        )]
        id: Option<String>,
    },
    #[command(about = "Modify existing code snippet in your collection")]
    Edit {
        #[arg(
            short = 'i',
            long = "id",
            help = "The unique ID of the snippet to edit"
        )]
        id: Option<u32>,

        #[arg(short, long, help = "The tag of the snippet to edit")]
        tag: Option<String>,

        #[arg(
            short,
            long,
            help = "Select a programming language to syntax highlight the code snippet"
        )]
        language: Option<String>,
    },
    #[command(
        about = "Export code snippet or a batch by specifying their IDs, tags, or languages."
    )]
    Export {
        #[arg(
            short = 'i',
            long = "id",
            help = "Export code snippet by its unique ID"
        )]
        id: Option<u32>,

        #[arg(
            short = 'l',
            long = "language",
            help = "Export snippets based on the specified programming language (comma-separated)"
        )]
        language: Option<String>,

        #[arg(
            short = 't',
            long = "tag",
            help = "Export snippets by tag (comma-separated)"
        )]
        tag: Option<String>,

        #[arg(
            short = 'p',
            long = "path",
            help = "Specify the directory where the snippet should be exported"
        )]
        path: Option<PathBuf>, // This is now part of the `Export` command
    },
    #[command(about = "List of all programming languages supported for syntax highlighting")]
    Languages,
    #[command(
        about = "Display the code of a specified snippet or all captured snippets if none is specified"
    )]
    View {
        #[arg(
            short = 'i',
            long = "id",
            help = "Unique ID automatically assigned for identification of the snippets"
        )]
        id: Option<u32>,

        #[arg(short, long, help = "Search for snippets by keyword (comma-separated)")]
        keyword: Option<String>,

        #[arg(
            short,
            long,
            help = "Search for snippets by programming language (comma-separated)"
        )]
        language: Option<String>,

        #[arg(
            short = 's',
            long = "summary",
            help = "Display a summary of snippets instead of full content"
        )]
        summary: bool,

        #[arg(short, long, help = "Search for snippets by tag (comma-separated)")]
        tag: Option<String>,
    },
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let ps = SyntaxSet::load_defaults_newlines();
    let supported_languages: Vec<&str> = ps.syntaxes().iter().map(|s| s.name.as_str()).collect();

    match &cli.command {
        Commands::Capture {
            tag,
            description,
            language,
        } => {
            let code = capture_snippet();
            let new_snippet = Snippet {
                tag: tag.clone(),
                description: Some(description.clone()),
                code,
                timestamp: Local::now().to_string(),
                language: Some(language.clone()),
                id: generate_unique_id(DATA_FILE),
            };

            if let Err(err) = save_snippet(new_snippet, DATA_FILE) {
                println!("\x1b[1;31merror:\x1b[0m saving snippet {}", err);
            } else {
                println!("\n\x1b[1;32mSnippet captured successfully!\x1b[0m\n");
            }
        }
        Commands::Copy { id } => match copy_code(DATA_FILE, id) {
            Ok(snippet) => {
                println!("\n\x1b[1;38;5;201mCode:\x1b[0m\n");

                let highlighted_code = if let Some(lang) = &snippet.language {
                    highlight_code_snippets(&snippet.code, lang)
                } else {
                    snippet.code.clone()
                };

                println!("{}", highlighted_code);
            }
            Err(err) => {
                println!("\x1b[1m\x1b[31merror:\x1b[0m\x1b[1m  {}\x1b[0m", err);
            }
        },
        Commands::Delete { id } => {
            if let Some(id_str) = id {
                let ids: Vec<u32> = id_str
                    .split(',')
                    .map(|id| id.trim().parse::<u32>())
                    .collect::<Result<Vec<u32>, _>>()
                    .map_err(|err| format!("Invalid ID format: {}", err))?;

                match delete_snippet(DATA_FILE, &ids) {
                    Ok(_) => {}
                    Err(err) => println!("\x1b[31merror:\x1b[0m {}", err),
                }
            } else {
                match id.as_ref().and_then(|s| s.trim().parse::<u32>().ok()) {
                    Some(id) => match delete_snippet(DATA_FILE, &[id]) {
                        Ok(_) => {}
                        Err(err) => println!("\x1b[31merror:\x1b[0m {}", err),
                    },
                    None => {
                        println!("\x1b[31merror:\x1b[0m missing snippet ID.
                        \nPlease provide a snippet ID using the \x1b[1m\x1b[36m-i\x1b[0m or \x1b[1m\x1b[36m--id\x1b[0m flag.
                        \n\x1b[1m\x1b[32m\x1b[4mUsage:\x1b[0m \x1b[1m\x1b[36mcodevault delete\x1b[0m \x1b[1m\x1b[36m-i\x1b[0m \x1b[34m<ID>\x1b[0m
                        \n\x1b[1m\x1b[32m\x1b[4mExample:\x1b[0m \x1b[1m\x1b[36mcodevault delete\x1b[0m \x1b[1m\x1b[36m-i\x1b[0m \x1b[1m\x1b[34m7\x1b[0m
                        \nFor more information, try '\x1b[1m\x1b[36m--help\x1b[0m'");
                    }
                };
            }
        }
        Commands::Edit { id, tag, language } => {
            match edit_snippet(DATA_FILE, id, tag, language, &supported_languages) {
                Ok(_) => {
                    println!("\x1b[1;32mChanges have been applied to the snippet.\x1b[0m");
                }
                Err(err) => {
                    println!("\n\x1b[31merror:\x1b[0m{}\n", err);
                }
            }
        }
        Commands::Export {
            id,
            language,
            tag,
            path,
        } => match export_snippets(DATA_FILE, id, tag, language, path) {
            Ok(_) => {}
            Err(err) => println!("\x1b[31merror:\x1b[0m {}", err),
        },
        Commands::Languages => {
            println!("\n\x1b[38;5;201;1mSupported Languages:\x1b[0m\n");
            for language in &supported_languages {
                println!("\x1b[1;36m»\x1b[0m \x1b[1;33m{}\x1b[0m", language);
            }
        }
        Commands::View {
            id,
            tag,
            language,
            keyword,
            summary,
        } => {
            println!("\n\x1b[38;5;201;1mSnippets Collection:\x1b[0m\n");
            match view_snippets(DATA_FILE, id, tag, language, keyword, *summary) {
                Ok(snippets) => {
                    for snippet in snippets {
                        if *summary {
                            print_snippet_summary(&snippet);
                        } else {
                            print_snippet(&snippet);
                        }
                    }
                }
                Err(err) => println!("\x1b[31merror:\x1b[0m{}", err),
            }
        }
    }

    Ok(())
}

fn format_with_border(content: &str, width: usize) -> String {
    let stripped_content = strip_ansi_codes(content);
    let padding = width.saturating_sub(stripped_content.chars().count());
    return format!(
        "\x1b[34m║\x1b[0m{}{}\x1b[34m║\x1b[0m",
        content,
        " ".repeat(padding)
    );
}

fn print_formatted_code(code: &str, language: &Option<String>, width: usize) {
    let highlighted_code = if let Some(lang) = language {
        highlight_code_snippets(code, lang)
    } else {
        code.to_string()
    };

    println!(
        "{}",
        format_with_border(&format!("\x1b[33;1m  Code:\x1b[0m"), width)
    );
    for line in highlighted_code.lines() {
        let formatted_line = format!("  {}", line);
        println!("{}", format_with_border(&formatted_line, width));
    }
}

fn capture_snippet() -> String {
    let mut buffer = String::new();
    println!("\n\x1b[38;5;201;1mCapture snippet:\x1b[0m\n");
    println!("\x1b[1;36mEnter your code snippet (press \x1b[33mReturn\x1b[1;36m, then \x1b[33mCtrl+D\x1b[1;36m to finish):\x1b[0m");

    io::stdin()
        .read_to_string(&mut buffer)
        .expect("\x1b[1;31mfailed to read snippet from input\x1b[0m");
    buffer
}

fn save_snippet(snippet: Snippet, file_path: &str) -> Result<(), String> {
    let mut snippets: Vec<Snippet> = if let Ok(file) = File::open(file_path) {
        serde_json::from_reader(file).map_err(|err| {
            format!(
                "\x1b[31merror:\x1b[0m unable deserializing snippets {}",
                err
            )
        })?
    } else {
        Vec::new()
    };

    snippets.push(snippet);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .map_err(|err| format!("\x1b[1;33m{}\x1b[0m", err))?;

    serde_json::to_writer_pretty(&mut file, &snippets)
        .map_err(|err| format!(" serializing snippets: '\x1b[1;33m{}\x1b[0m'", err))?;

    Ok(())
}

fn load_snippets(file_path: &str) -> Result<Vec<Snippet>, String> {
    let file =
        File::open(file_path).map_err(|err| format!("\x1b[1;33m opening file {}\x1b[0m", err))?;
    let snippets: Vec<Snippet> = serde_json::from_reader(file)
        .map_err(|err| format!("\x1b[1;33m reading snippets{}\x1b[0m", err))?;
    Ok(snippets)
}

fn save_snippets_for_edit(snippets: Vec<Snippet>, file_path: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .map_err(|err| format!("\x1b[1;33m opening file{}\x1b[0m", err))?;

    serde_json::to_writer_pretty(&mut file, &snippets)
        .map_err(|err| format!(" serializing snippets: '\x1b[1;33m{}\x1b[0m'", err))?;
    Ok(())
}

fn view_snippets(
    file_path: &str,
    id: &Option<u32>,
    tag: &Option<String>,
    language: &Option<String>,
    keyword: &Option<String>,
    _summary: bool,
) -> Result<Vec<Snippet>, String> {
    let snippets = load_snippets(file_path)?;

    let mut filtered_snippets = snippets
        .into_iter()
        .filter(|snippet| {
            let tag_match = if let Some(tag) = tag {
                let tags: Vec<&str> = tag.split(',').map(|s| s.trim()).collect();
                tags.iter()
                    .any(|t| snippet.tag.to_lowercase().contains(&t.to_lowercase()))
            } else {
                true
            };

            let language_match = if let Some(language) = language {
                let langs: Vec<&str> = language.split(',').map(|s| s.trim()).collect();
                langs.iter().any(|l| {
                    snippet
                        .language
                        .as_ref()
                        .map(|lang| lang.to_lowercase().contains(&l.to_lowercase()))
                        .unwrap_or(false)
                })
            } else {
                true
            };

            let keyword_match = if let Some(keyword) = keyword {
                let keywords: Vec<&str> = keyword.split(',').map(|s| s.trim()).collect();
                keywords.iter().any(|k| {
                    snippet.tag.to_lowercase().contains(&k.to_lowercase())
                        || snippet
                            .description
                            .as_ref()
                            .map(|desc| desc.to_lowercase().contains(&k.to_lowercase()))
                            .unwrap_or(false)
                        || snippet.code.to_lowercase().contains(&k.to_lowercase())
                })
            } else {
                true
            };

            tag_match && language_match && keyword_match
        })
        .collect::<Vec<_>>();

    if let Some(id) = id {
        if let Some(index) = filtered_snippets.iter().position(|s| s.id == *id) {
            filtered_snippets = vec![filtered_snippets[index].clone()];
        } else {
            return Err(format!(
                " snippet ID '\x1b[1;33m{}\x1b[0m' does not exist in the collection",
                id
            ));
        }
    }

    Ok(filtered_snippets)
}

fn generate_unique_id(file_path: &str) -> u32 {
    let mut max_id = 0;
    if let Ok(mut file) = File::open(file_path) {
        let snippets: Vec<Snippet> = match serde_json::from_reader(&mut file) {
            Ok(s) => s,
            Err(_err) => {
                return 1;
            }
        };
        max_id = snippets.iter().map(|s| s.id).max().unwrap_or(0);
    }
    max_id + 1
}

fn edit_snippet(
    file_path: &str,
    id: &Option<u32>,
    tag: &Option<String>,
    _language: &Option<String>,
    _supported_languages: &Vec<&str>,
) -> Result<(), String> {
    let mut snippets = load_snippets(file_path)?;

    let mut snippet_to_edit = match (id, tag) {
        (Some(snippet_id), _) => {
            if let Some(index) = snippets.iter().position(|s| s.id == *snippet_id) {
                snippets.remove(index)
            } else {
                return Err(format!(
                    "Snippet ID '\x1b[1;33m{}\x1b[0m' does not exist in the collection",
                    snippet_id
                ));
            }
        }
        (None, Some(snippet_tag)) => {
            let matching_snippets: Vec<Snippet> =
                snippets.iter().cloned().filter(|s| s.tag == *snippet_tag).collect();

            if matching_snippets.len() > 1 {
                println!("\n\x1b[38;5;201;1mEdit snippet:\x1b[0m\n");
                println!("\x1b[1;36mMultiple matching tags found, choose an \x1b[1;33mID\x1b[1;36m to edit from list:\x1b[0m\n");
                for snippet in matching_snippets.iter() {
                    println!("\x1b[1;36m  »\x1b[0m \x1b[1;33mID {}\x1b[0m", snippet.id);
                }

                print!("\n\x1b[1;36mType the \x1b[1;33mID\x1b[0m\x1b[1;36m of the snippet you want to modify: \x1b[0m");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let chosen_id: u32 = match input.trim().parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err("Invalid input. Enter numbers only!".to_string());
                    }
                };

                if let Some(index) = snippets.iter().position(|s| s.id == chosen_id) {
                    snippets.remove(index)
                } else {
                    return Err(format!(
                        "Snippet '\x1b[1;33m{}\x1b[0m' ID doesn't exist. Enter a different ID",
                        chosen_id
                    ));
                }
            } else if let Some(index) = snippets.iter().position(|s| s.tag == *snippet_tag) {
                snippets.remove(index)
            } else {
                return Err(format!(
                    "Tag '\x1b[1;33m{}\x1b[0m' doesn't match any snippets. Try a different tag.",
                    snippet_tag
                ));
            }
        }
        (None, None) => {
            println!("\x1b[1;36mTo edit a snippet, use its \x1b[1;33mID\x1b[1;36m or \x1b[1;33mtag\x1b[0m\x1b[1;36m\x1b[0m");
            return Err("Enter a snippet ID or tag to proceed.".to_string());
        }
    };

    println!("\n\x1b[38;5;201;1mEdit snippet:\x1b[0m\n");
    println!(
        "\x1b[34;1m╔{}\x1b[0m",
        "═".repeat(62) + "╗\x1b[34;1m\x1b[0m"
    );
    let id_line = format!(
        "  \x1b[33;1mID:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet_to_edit.id
    );
    let created_line = format!(
        "  \x1b[33;1mCreated:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet_to_edit.timestamp
    );
    let tag_line = format!(
        "  \x1b[33;1mCurrent Snippet's Tag:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet_to_edit.tag
    );
    let description_line = if let Some(desc) = &snippet_to_edit.description {
        format!(
            "  \x1b[33;1mCurrent Description:\x1b[0m \x1b[35;1m{}\x1b[0m",
            desc
        )
    } else {
        String::new()
    };
    println!("{}", id_line);
    println!("{}", tag_line);
    println!("{}", created_line);
    if !description_line.is_empty() {
        println!("{}", description_line);
    }
    println!(
        "\x1b[34;1m╚{}\x1b[0m",
        "═".repeat(62) + "╝\x1b[34;1m\x1b[0m"
    );

    print!("\x1b[1m\x1b[36m  Enter new tag (\x1b[1;33mleave blank to keep current\x1b[0m\x1b[36m): \x1b[0m");
    io::stdout().flush().unwrap();
    let mut new_tag = String::new();
    io::stdin().read_line(&mut new_tag).unwrap();
    if !new_tag.trim().is_empty() {
        snippet_to_edit.tag = new_tag.trim().to_string();
    }

    print!("\x1b[1m\x1b[36m  Enter new description (\x1b[1;33mleave blank to keep current\x1b[0m\x1b[36m): \x1b[0m");
    io::stdout().flush().unwrap();
    let mut new_description = String::new();
    io::stdin().read_line(&mut new_description).unwrap();
    if !new_description.trim().is_empty() {
        snippet_to_edit.description = Some(new_description.trim().to_string());
    } else {
        snippet_to_edit.description = None;
    }

    print!("\x1b[1m\x1b[36m  Enter new language (\x1b[1;33mleave blank to keep current\x1b[0m\x1b[36m): \x1b[0m");
    io::stdout().flush().unwrap();
    let mut new_language = String::new();
    io::stdin().read_line(&mut new_language).unwrap();
    if !new_language.trim().is_empty() {
        snippet_to_edit.language = Some(new_language.trim().to_string());
    } else {
        snippet_to_edit.language = None;
    }

    println!("\n  \x1b[33;1mCurrent Code:\x1b[0m\n");
    let highlighted_code = if let Some(lang) = &snippet_to_edit.language {
        highlight_code_snippets(&snippet_to_edit.code, lang)
    } else {
        snippet_to_edit.code.clone()
    };
    println!("  {}", highlighted_code);

    println!(
        "\x1b[34;1m╔{}\x1b[0m",
        "═".repeat(62) + "╗\x1b[34;1m\x1b[0m"
    );
    println!("\x1b[1;36m Enter your code snippet (press \x1b[33mReturn\x1b[1;36m, then \x1b[33mCtrl+D\x1b[1;36m to finish):\x1b[0m");
    println!(
        "\x1b[34;1m╚{}\x1b[0m",
        "═".repeat(62) + "╝\x1b[34;1m\x1b[0m"
    );
    io::stdout().flush().unwrap();
    let mut new_code = String::new();
    io::stdin()
        .read_to_string(&mut new_code)
        .expect("failed to read snippet from input");

    snippet_to_edit.code = new_code;
    snippets.push(snippet_to_edit);
    save_snippets_for_edit(snippets, file_path)?;

    Ok(())
}

fn highlight_code_snippets(code: &str, language: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps
        .find_syntax_by_token(language)
        .or_else(|| ps.find_syntax_by_name(language))
        .unwrap_or(ps.find_syntax_plain_text());

    let mut output = String::new();
    for line in LinesWithEndings::from(code) {
        let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ps).unwrap();
        let escaped_line = format_terminal_snippets(&ranges);
        output.push_str(&escaped_line);
    }

    output
}

fn format_terminal_snippets(v: &[(Style, &str)]) -> String {
    let mut s = String::new();
    for &(ref style, text) in v.iter() {
        s.push_str(&format!(
            "\x1b[38;2;{};{};{}m{}",
            style.foreground.r, style.foreground.g, style.foreground.b, text
        ));
    }
    s.push_str("\x1b[0m");
    s
}

fn print_snippet(snippet: &Snippet) {
    let id_line = format!("  \x1b[33;1mID:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet.id);
    let created_line = format!(
        "  \x1b[33;1mCreated:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet.timestamp
    );
    let tag_line = format!(
        "  \x1b[33;1mSnippet's Tag:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet.tag
    );
    let description_line = if let Some(desc) = &snippet.description {
        format!("  \x1b[33;1mDescription:\x1b[0m \x1b[35;1m{}\x1b[0m", desc)
    } else {
        String::new()
    };

    let all_lines = vec![
        strip_ansi_codes(&id_line),
        strip_ansi_codes(&tag_line),
        strip_ansi_codes(&created_line),
        strip_ansi_codes(&description_line),
    ]
    .into_iter()
    .chain(snippet.code.lines().map(|line| strip_ansi_codes(line)))
    .collect::<Vec<_>>();

    let max_line_length = all_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let adjusted_width = max_line_length + 4;

    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╔\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╗\x1b[0m"
    );
    println!("{}", format_with_border(&id_line, adjusted_width));
    println!("{}", format_with_border(&tag_line, adjusted_width));
    println!("{}", format_with_border(&created_line, adjusted_width));
    if !description_line.is_empty() {
        println!("{}", format_with_border(&description_line, adjusted_width));
    }
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╟\x1b[0m".to_owned()
            + &"\x1b[34m─\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╢\x1b[0m"
    );
    print_formatted_code(&snippet.code, &snippet.language, adjusted_width);
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╚\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╝\x1b[0m\n"
    );
}

fn print_snippet_summary(snippet: &Snippet) {
    let id_line = format!("  \x1b[33;1mID:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet.id);
    let tag_line = format!("  \x1b[33;1mTag:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet.tag);
    let created_line = format!(
        "  \x1b[33;1mCreated:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet.timestamp
    );
    let description_line = if let Some(desc) = &snippet.description {
        format!("  \x1b[33;1mDescription:\x1b[0m \x1b[35;1m{}\x1b[0m", desc)
    } else {
        String::new()
    };
    let all_lines = vec![
        strip_ansi_codes(&id_line),
        strip_ansi_codes(&tag_line),
        strip_ansi_codes(&created_line),
        strip_ansi_codes(&description_line),
    ]
    .into_iter()
    .collect::<Vec<_>>();

    let max_line_length = all_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    let adjusted_width = max_line_length + 4;

    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╔\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╗\x1b[0m"
    );
    println!("{}", format_with_border(&id_line, adjusted_width));
    println!("{}", format_with_border(&tag_line, adjusted_width));
    println!("{}", format_with_border(&created_line, adjusted_width));
    if !description_line.is_empty() {
        println!("{}", format_with_border(&description_line, adjusted_width));
    }
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╚\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╝\x1b[0m\n"
    );
}

fn copy_code(file_path: &str, id: &Option<u32>) -> Result<Snippet, String> {
    let snippets = load_snippets(file_path)?;

    if id.is_none() {
        let error_message = "missing snippet ID
        \nPlease provide a snippet ID using the \x1b[1m\x1b[36m-i\x1b[0m or \x1b[1m\x1b[36m--id\x1b[0m flag.
        \n\x1b[1m\x1b[32m\x1b[4mUsage:\x1b[0m \x1b[1m\x1b[36mcodevault copy\x1b[0m \x1b[1m\x1b[36m-i\x1b[0m \x1b[34m<ID>\x1b[0m
        \n\x1b[1m\x1b[32m\x1b[4mExample:\x1b[0m \x1b[1m\x1b[36mcodevault copy\x1b[0m \x1b[1m\x1b[36m-i\x1b[0m \x1b[1m\x1b[34m22\x1b[0m
        \nFor more information, try '\x1b[1m\x1b[36m--help\x1b[0m'".to_string();
        return Err(error_message);
    }

    let id = id.unwrap();

    if let Some(snippet) = snippets.iter().find(|s| s.id == id) {
        return Ok(snippet.clone());
    }

    Err(format!(
        " snippet ID '\x1b[1;33m{}\x1b[0m' does not exist in the collection",
        id
    ))
}

fn delete_snippet(file_path: &str, ids: &[u32]) -> Result<(), String> {
    let mut snippets = load_snippets(file_path)?;

    let mut non_existent_ids: Vec<u32> = Vec::new();
    for id in ids {
        if !snippets.iter().any(|s| s.id == *id) {
            non_existent_ids.push(*id);
        }
    }

    if !non_existent_ids.is_empty() {
        let ids_str = non_existent_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            " snippet ID '\x1b[1;33m{}\x1b[0m' does not exist in the collection",
            ids_str
        ));
    }

    let ids_str = ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let mut plural = "snippet";
    if ids.len() > 1 {
        plural = "snippets";
    }
    println!("\n\x1b[38;5;201;1mDelete snippet:\x1b[0m\n");
    print!("\x1b[1m\x1b[36mAre you sure you want to permanently delete {} {} ? Please confirm (\x1b[33my/N\x1b[36m): \x1b[0m", plural, ids_str);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != "y" {
        print!("\n\x1b[91mSnippet deletion cancelled\x1b[0m\n");
        return Ok(());
    }

    let mut deleted_count = 0;
    for id in ids {
        if let Some(index) = snippets.iter().position(|s| s.id == *id) {
            snippets.remove(index);
            deleted_count += 1;
        }
    }

    if deleted_count > 0 {
        save_snippets_for_edit(snippets, file_path)?;
        println!("\n\x1b[32mdeleted successfully!\x1b[0m");
    }

    Ok(())
}

fn export_snippets(
    file_path: &str,
    id: &Option<u32>,
    tag: &Option<String>,
    language: &Option<String>,
    export_path: &Option<PathBuf>,
) -> Result<(), String> {
    let snippets = load_snippets(file_path)?;

    let mut filtered_snippets: Vec<Snippet> = snippets.clone();

    if let Some(id) = id {
        filtered_snippets = filtered_snippets
            .into_iter()
            .filter(|s| s.id == *id)
            .collect::<Vec<_>>();
    }

    if let Some(tag) = tag {
        let tags: Vec<&str> = tag.split(',').map(|s| s.trim()).collect();
        filtered_snippets = filtered_snippets
            .into_iter()
            .filter(|s| {
                tags.iter()
                    .any(|t| s.tag.to_lowercase().contains(&t.to_lowercase()))
            })
            .collect::<Vec<_>>();
    }

    if let Some(lang) = language {
        let langs: Vec<&str> = lang.split(',').map(|s| s.trim()).collect();
        filtered_snippets = filtered_snippets
            .into_iter()
            .filter(|s| {
                langs.iter().any(|l| {
                    s.language
                        .as_ref()
                        .map(|lang| lang.to_lowercase().contains(&l.to_lowercase()))
                        .unwrap_or(false)
                })
            })
            .collect::<Vec<_>>();
    }

    if filtered_snippets.is_empty() {
        if let Some(id) = id {
            return Err(format!(
                " snippet '\x1b[1;33m{}\x1b[0m' ID does not exist in the collection",
                id
            ));
        } else if let Some(tag) = tag {
            let tags: Vec<&str> = tag.split(',').map(|s| s.trim()).collect();
            if let Some(lang) = language {
                // Both tag and language are provided, but no match
                let langs: Vec<&str> = lang.split(',').map(|s| s.trim()).collect();
                let matching_html_count = snippets
                    .iter()
                    .filter(|s| {
                        s.language
                            .as_ref()
                            .map(|l| l.to_lowercase().contains(&"html".to_lowercase()))
                            .unwrap_or(false)
                    })
                    .count();

                return Err(format!(
                    "\x1b[1m\x1b[93m'{}'\x1b[0m\x1b[1m matching snippets found, but no snippets match both \x1b[93m'{}'\x1b[0m\x1b[1m tags and \x1b[93m'{}'\x1b[0m\x1b[1m language.\nPlease check your criteria. Both should match in order to export.\x1b[0m",
                    matching_html_count,
                    tags.join(", "),
                    langs.join(", ")
                ));
            } else {
                // Only tag is provided, but no match
                return Err(format!(
                    " snippet with '{}' tags does not exist in the collection",
                    tags.join(", ")
                ));
            }
        } else if let Some(lang) = language {
            // Only language is provided, but no match
            let langs: Vec<&str> = lang.split(',').map(|s| s.trim()).collect();
            return Err(format!(
                " snippet with '{}' language does not exist in the collection",
                langs.join(", ")
            ));
        }
    }

    // Show a warning and confirmation prompt before exporting
    // Show prompt only if multiple snippets are matched (more than one)
    if filtered_snippets.len() > 1 {
        println!("\n\x1b[38;5;201;1mExport Snippets:\x1b[0m\n");
        print!("\x1b[1m\x1b[36mExporting {} snippets in language-specific formats. Are you sure you want to continue? (\x1b[33my\x1b[36m/\x1b[33mN\x1b[36m): \x1b[0m", filtered_snippets.len());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() != "y" {
            println!("\x1b[1m\x1b[91m\nSnippet export cancelled\x1b[0m\x1b[0m");
            return Ok(());
        }
    }

    // Determine the export directory based on the --path flag or the default
    let export_dir = match export_path {
        Some(path) => path.clone(),
        None => {
            println!("\x1b[1m\x1b[36mNo export path specified. Exporting snippets to the default 'snippet_exports' directory. Please wait...\x1b[0m");
            PathBuf::from("snippet_exports")
        }
    };

    // Create the export directory if it doesn't exist
    std::fs::create_dir_all(&export_dir)
        .map_err(|err| format!("\x1b[31merror:\x1b[0m creating directory: {}\x1b[0m", err))?;

    for snippet in filtered_snippets {
        let _lowercase_lang = snippet.language.as_ref().map(|lang| lang.to_lowercase());

        // Map language to file extension
        let extension = match snippet.language.as_deref() {
            Some("AppleScript") => "applescript",
            Some("ASP") => "asp",
            Some("Batch File") => "bat",
            Some("BibTeX") => "bib",
            Some("Bourne Again Shell (bash)") => "sh",
            Some("C") => "c",
            Some("C#") => "cs",
            Some("C++") => "cpp",
            Some("Cargo Build Results") => "log",
            Some("Clojure") => "clj",
            Some("commands-builtin-shell-bash") => "sh",
            Some("CSS") => "css",
            Some("D") => "d",
            Some("Diff") => "diff",
            Some("Erlang") => "erl",
            Some("Go") => "go",
            Some("Graphviz (DOT)") => "dot",
            Some("Groovy") => "groovy",
            Some("Haml") => "haml",
            Some("Haskell") => "hs",
            Some("HTML") => "html",
            Some("Java") => "java",
            Some("Java Properties") => "properties",
            Some("JavaScript") => "js",
            Some("JSON") => "json",
            Some("LaTeX") => "tex",
            Some("LaTeX Log") => "log",
            Some("Lisp") => "lisp",
            Some("Lua") => "lua",
            Some("Make Output") => "mak",
            Some("Makefile") => "mak",
            Some("Markdown") => "md",
            Some("MATLAB") => "m",
            Some("MultiMarkdown") => "mmd",
            Some("NAnt Build File") => "build",
            Some("Objective-C") => "m",
            Some("Objective-C++") => "mm",
            Some("OCaml") => "ml",
            Some("OCamllex") => "mll",
            Some("OCamlyacc") => "mly",
            Some("Pascal") => "pas",
            Some("Perl") => "pl",
            Some("PHP") => "php",
            Some("Python") => "py",
            Some("R") => "R",
            Some("R Console") => "Rout",
            Some("Rd (R Documentation)") => "Rd",
            Some("Regular Expression") => "regex",
            Some("Regular Expressions (Javascript)") => "js",
            Some("Regular Expressions (Python)") => "py",
            Some("reStructuredText") => "rst",
            Some("Ruby") => "rb",
            Some("Ruby on Rails") => "rb",
            Some("Rust") => "rs",
            Some("Scala") => "scala",
            Some("Shell-Unix-Generic") => "sh",
            Some("SQL") => "sql",
            Some("Tcl") => "tcl",
            Some("TeX") => "tex",
            Some("Textile") => "textile",
            Some("XML") => "xml",
            Some("YAML") => "yaml",
            _ => "txt", // Default to .txt if no language specified
        };

        let filename = format!("{}/{}.{}", export_dir.display(), snippet.id, extension);

        // Check if the file already exists
        if std::fs::metadata(&filename).is_ok() {
            println!(
                "\n\x1b[1m\x1b[93mThe file has been already exported and is located at '{}'.\x1b[0m\x1b[0m",
                filename
            );
            continue; // Skip to the next snippet
        }

        let file = File::create(&filename).map_err(|err| {
            format!(
                "\x1b[31merror:\x1b[0m  creating file {}: {}\x1b[0m",
                filename, err
            )
        })?;
        let mut writer = BufWriter::new(file);

        write!(writer, "{}", snippet.code).map_err(|err| {
            format!(
                "\x1b[31merror:\x1b[0m  writing to file {}: {}\x1b[0m",
                filename, err
            )
        })?;

        println!(
            "\x1b[1;32m\nSuccessfully exported snippet to file '{}'.\x1b[0m",
            filename
        );
    }

    Ok(())
}
