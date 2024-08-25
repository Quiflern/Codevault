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

// data file stored in data dir
const DATA_FILE: &str = "data/codevault.json";

// Import the necessary libraries and macros
#[derive(Parser)]
#[command(
    author,                        // Specifies the author of the command-line tool
    version,                       // Specifies the version of the tool
    about = "Codevault",           // A brief description of the tool
    propagate_version = true,      // Automatically propagate the version number to subcommands
    long_about = None,             // No long description provided
    styles(vault_styling())        // Apply custom styling to the command-line output
)]
pub struct Cli {
    // This field specifies the command or subcommand to be executed
    #[command(subcommand)]
    pub command: Commands,

    // Optional description for the provided code snippet, accessible via short (-d) or long (--description) flag
    #[arg(short, long, help = "Add a description to the provided code snippet")]
    description: Option<String>,

    // Optional unique ID automatically assigned for identification of code snippets, accessible via short (-i) or long (--id) flag
    #[arg(
        short,
        long,
        help = "Unique ID automatically assigned for identification of the snippets"
    )]
    id: Option<u32>,

    // Optional programming language for syntax highlighting, accessible via short (-l) or long (--language) flag
    #[arg(
        short,
        long,
        help = "Select a programming language for syntax highlighting"
    )]
    language: Option<String>,

    // Optional tags for categorizing the code snippets, accessible via short (-t) or long (--tag) flag
    #[arg(short, long, help = "Apply relevant tags to categorize the snippets")]
    tag: Option<String>,
}

// Define a set of subcommands for the CLI using the Commands enum
#[derive(Subcommand)]
pub enum Commands {
    // Subcommand to add a new code snippet to the user's collection
    #[command(about = "Add a new code snippet to your collection")]
    Capture {
        // Argument to add a description to the provided code snippet, accessible with -d or --description
        #[arg(
            short = 'd',
            long = "description",
            help = "Add a description to the provided code snippet"
        )]
        description: String,

        // Argument to specify the programming language for syntax highlighting, accessible with -l or --language
        #[arg(
            short = 'l',
            long = "language",
            help = "Select a programming language for syntax highlighting"
        )]
        language: String,

        // Argument to apply relevant tags for categorizing the snippet, accessible with -t or --tag
        #[arg(
            short = 't',
            long = "tag",
            help = "Apply relevant tags to categorize the snippets"
        )]
        tag: String,
    },

    // Subcommand to show the code of a specified snippet using its ID
    #[command(about = "Show the code of a specified snippet using IDs")]
    Copy {
        // Argument to specify the unique ID of the snippet, accessible with -i or --id
        #[arg(
            short = 'i',
            long = "id",
            help = "Unique ID automatically assigned for identification of the snippets"
        )]
        id: Option<u32>,
    },

    // Subcommand to remove one or more code snippets by specifying their IDs
    #[command(
        about = "Remove one or more code snippets by specifying their IDs, separated by commas"
    )]
    Delete {
        // Argument to specify the unique ID(s) of the snippets to delete, accessible with -i or --id
        #[arg(
            short = 'i',
            long = "id",
            help = "Unique ID automatically assigned for identification of the snippets"
        )]
        id: Option<String>,
    },

    // Subcommand to modify an existing code snippet in the collection
    #[command(about = "Modify existing code snippet in your collection")]
    Edit {
        // Argument to specify the unique ID of the snippet to edit, accessible with -i or --id
        #[arg(
            short = 'i',
            long = "id",
            help = "The unique ID of the snippet to edit"
        )]
        id: Option<u32>,

        // Argument to specify the tag of the snippet to edit, accessible with -t or --tag
        #[arg(short, long, help = "The tag of the snippet to edit")]
        tag: Option<String>,

        // Argument to specify the programming language for syntax highlighting, accessible with -l or --language
        #[arg(
            short,
            long,
            help = "Select a programming language to syntax highlight the code snippet"
        )]
        language: Option<String>,
    },

    // Subcommand to export code snippets by specifying IDs, tags, or languages
    #[command(
        about = "Export code snippet or a batch by specifying their IDs, tags, or languages."
    )]
    Export {
        // Argument to specify the unique ID of the snippet to export, accessible with -i or --id
        #[arg(
            short = 'i',
            long = "id",
            help = "Export code snippet by its unique ID"
        )]
        id: Option<u32>,

        // Argument to export snippets based on the specified programming language, accessible with -l or --language
        #[arg(
            short = 'l',
            long = "language",
            help = "Export snippets based on the specified programming language (comma-separated)"
        )]
        language: Option<String>,

        // Argument to export snippets by tag, accessible with -t or --tag
        #[arg(
            short = 't',
            long = "tag",
            help = "Export snippets by tag (comma-separated)"
        )]
        tag: Option<String>,

        // Argument to specify the directory where the snippet should be exported, accessible with -p or --path
        #[arg(
            short = 'p',
            long = "path",
            help = "Specify the directory where the snippet should be exported"
        )]
        path: Option<PathBuf>,
    },

    // Subcommand to list all programming languages supported for syntax highlighting
    #[command(about = "List of all programming languages supported for syntax highlighting")]
    Languages,

    // Subcommand to display the code of a specified snippet or all captured snippets if none is specified
    #[command(
        about = "Display the code of a specified snippet or all captured snippets if none is specified"
    )]
    View {
        // Argument to specify the unique ID of the snippet to view, accessible with -i or --id
        #[arg(
            short = 'i',
            long = "id",
            help = "Unique ID automatically assigned for identification of the snippets"
        )]
        id: Option<u32>,

        // Argument to search for snippets by keyword, accessible with -k or --keyword
        #[arg(short, long, help = "Search for snippets by keyword (comma-separated)")]
        keyword: Option<String>,

        // Argument to search for snippets by programming language, accessible with -l or --language
        #[arg(
            short,
            long,
            help = "Search for snippets by programming language (comma-separated)"
        )]
        language: Option<String>,

        // Argument to display a summary of snippets instead of the full content, accessible with -s or --summary
        #[arg(
            short = 's',
            long = "summary",
            help = "Display a summary of snippets instead of full content"
        )]
        summary: bool,

        // Argument to search for snippets by tag, accessible with -t or --tag
        #[arg(short, long, help = "Search for snippets by tag (comma-separated)")]
        tag: Option<String>,
    },
}


fn main() -> Result<(), String> {
    // Parse the command-line arguments into the CLI struct
    let cli = Cli::parse();
    
    // Load the default syntax set for syntax highlighting with newlines
    let ps = SyntaxSet::load_defaults_newlines();
    
    // Collect all supported programming languages into a vector of strings
    let supported_languages: Vec<&str> = ps.syntaxes().iter().map(|s| s.name.as_str()).collect();

    // Match the parsed CLI command and execute the corresponding logic
    match &cli.command {
        // If the Capture command is selected
        Commands::Capture {
            tag,
            description,
            language,
        } => {
            // Capture the code snippet from user input
            let code = capture_snippet();
            
            // Create a new Snippet instance with the provided details
            let new_snippet = Snippet {
                tag: tag.clone(),
                description: Some(description.clone()),
                code,
                timestamp: Local::now().to_string(),
                language: Some(language.clone()),
                id: generate_unique_id(DATA_FILE),
            };

            // Save the snippet and handle any errors that may occur
            if let Err(err) = save_snippet(new_snippet, DATA_FILE) {
                println!("\x1b[1;31merror:\x1b[0m saving snippet {}", err);
            } else {
                println!("\n\x1b[1;32mSnippet captured successfully!\x1b[0m\n");
            }
        }
        
        // If the Copy command is selected
        Commands::Copy { id } => match copy_code(DATA_FILE, id) {
            Ok(snippet) => {
                println!("\n\x1b[1;38;5;201mCode:\x1b[0m\n");

                // Highlight the code snippet if a language is specified, otherwise print it as-is
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

        // If the Delete command is selected
        Commands::Delete { id } => {
            // If an ID string is provided, parse it into a vector of IDs and delete the corresponding snippets
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
                // Handle the case where no valid ID is provided
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

        // If the Edit command is selected
        Commands::Edit { id, tag, language } => {
            // Edit the snippet with the provided ID, tag, or language and update the data file
            match edit_snippet(DATA_FILE, id, tag, language, &supported_languages) {
                Ok(_) => {
                    println!("\n\x1b[1;32mChanges have been applied to the snippet.\x1b[0m");
                }
                Err(err) => {
                    println!("\n\x1b[31merror:\x1b[0m{}\n", err);
                }
            }
        }

        // If the Export command is selected
        Commands::Export {
            id,
            language,
            tag,
            path,
        } => match export_snippets(DATA_FILE, id, tag, language, path) {
            Ok(_) => {}
            Err(err) => println!("\x1b[31merror:\x1b[0m {}", err),
        },

        // If the Languages command is selected
        Commands::Languages => {
            // Display all supported programming languages
            println!("\n\x1b[38;5;201;1mSupported Languages:\x1b[0m\n");
            for language in &supported_languages {
                println!("\x1b[1;36m»\x1b[0m \x1b[1;33m{}\x1b[0m", language);
            }
        }

        // If the View command is selected
        Commands::View {
            id,
            tag,
            language,
            keyword,
            summary,
        } => {
            println!("\n\x1b[38;5;201;1mSnippets Collection:\x1b[0m\n");

            // View and print snippets based on the provided filters (ID, tag, language, keyword, or summary)
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

    // Return an OK result to indicate successful execution
    Ok(())
}

fn generate_unique_id(file_path: &str) -> u32 {
    let mut max_id = 0;
    
    // Open the file specified by file_path
    if let Ok(mut file) = File::open(file_path) {
        // Attempt to deserialize the file content into a Vec<Snippet>
        let snippets: Vec<Snippet> = match serde_json::from_reader(&mut file) {
            Ok(s) => s,
            Err(_err) => {
                // If deserialization fails, start with ID 1
                return 1;
            }
        };
        
        // Find the maximum ID from the existing snippets, default to 0 if no snippets are found
        max_id = snippets.iter().map(|s| s.id).max().unwrap_or(0);
    }
    
    // Return the next unique ID by incrementing the maximum ID found
    max_id + 1
}

fn format_with_border(content: &str, width: usize) -> String {
    // Remove ANSI color codes from content to calculate the width correctly
    let stripped_content = strip_ansi_codes(content);
    
    // Calculate the amount of padding needed to make the total width equal to 'width'
    let padding = width.saturating_sub(stripped_content.chars().count());
    
    // Format content with borders and padding to fit the specified width
    format!(
        "\x1b[34m║\x1b[0m{}{}\x1b[34m║\x1b[0m",
        content,
        " ".repeat(padding)
    )
}

fn print_formatted_code(code: &str, language: &Option<String>, width: usize) {
    // Highlight the code if a language is specified, otherwise use the plain code
    let highlighted_code = if let Some(lang) = language {
        highlight_code_snippets(code, lang)
    } else {
        code.to_string()
    };

    // Print the header for the code section with a border
    println!(
        "{}",
        format_with_border(&format!("\x1b[33;1m  Code:\x1b[0m"), width)
    );
    
    // Print each line of the highlighted code with a border
    for line in highlighted_code.lines() {
        let formatted_line = format!("  {}", line);
        println!("{}", format_with_border(&formatted_line, width));
    }
}

fn highlight_code_snippets(code: &str, language: &str) -> String {
    // Load default syntax settings and themes
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Find the syntax definition based on the provided language name or token
    let syntax = ps
        .find_syntax_by_token(language)
        .or_else(|| ps.find_syntax_by_name(language))
        .unwrap_or(ps.find_syntax_plain_text());

    let mut output = String::new();
    
    // Iterate through each line of the code with its endings
    for line in LinesWithEndings::from(code) {
        // Create a highlighter with the chosen syntax and theme
        let mut highlighter = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
        
        // Highlight the current line, collecting style and text tuples
        let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &ps).unwrap();
        
        // Format the highlighted line into terminal color codes
        let escaped_line = format_terminal_snippets(&ranges);
        
        // Append the formatted line to the output
        output.push_str(&escaped_line);
    }

    output
}

fn format_terminal_snippets(v: &[(Style, &str)]) -> String {
    let mut s = String::new();
    
    // Iterate through each style-text pair
    for &(ref style, text) in v.iter() {
        // Format text with ANSI color codes based on the style
        s.push_str(&format!(
            "\x1b[38;2;{};{};{}m{}",
            style.foreground.r, style.foreground.g, style.foreground.b, text
        ));
    }
    
    // Reset ANSI color codes to default
    s.push_str("\x1b[0m");
    
    s
}

fn print_snippet(snippet: &Snippet) {
    // Format the snippet ID line with ANSI color codes
    let id_line = format!("  \x1b[33;1mID:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet.id);
    
    // Format the creation timestamp line with ANSI color codes
    let created_line = format!(
        "  \x1b[33;1mCreated:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet.timestamp
    );
    
    // Format the tag line with ANSI color codes
    let tag_line = format!(
        "  \x1b[33;1mSnippet's Tag:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet.tag
    );
    
    // Format the description line if a description is available
    let description_line = if let Some(desc) = &snippet.description {
        format!("  \x1b[33;1mDescription:\x1b[0m \x1b[35;1m{}\x1b[0m", desc)
    } else {
        String::new()
    };

    // Collect all lines into a vector, stripping ANSI color codes for length calculation
    let all_lines = vec![
        strip_ansi_codes(&id_line),
        strip_ansi_codes(&tag_line),
        strip_ansi_codes(&created_line),
        strip_ansi_codes(&description_line),
    ]
    .into_iter()
    .chain(snippet.code.lines().map(|line| strip_ansi_codes(line)))
    .collect::<Vec<_>>();

    // Determine the maximum line length for formatting
    let max_line_length = all_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    // Set the width for the formatted output, adding extra space for borders
    let adjusted_width = max_line_length + 4;

    // Print the top border of the snippet box
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╔\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╗\x1b[0m"
    );
    
    // Print each formatted line within the border
    println!("{}", format_with_border(&id_line, adjusted_width));
    println!("{}", format_with_border(&tag_line, adjusted_width));
    println!("{}", format_with_border(&created_line, adjusted_width));
    if !description_line.is_empty() {
        println!("{}", format_with_border(&description_line, adjusted_width));
    }
    
    // Print a separator line within the snippet box
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╟\x1b[0m".to_owned()
            + &"\x1b[34m─\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╢\x1b[0m"
    );
    
    // Print the code inside the snippet box with formatting
    print_formatted_code(&snippet.code, &snippet.language, adjusted_width);
    
    // Print the bottom border of the snippet box
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╚\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╝\x1b[0m\n"
    );
}

fn print_snippet_summary(snippet: &Snippet) {
    // Format the snippet ID line with ANSI color codes
    let id_line = format!("  \x1b[33;1mID:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet.id);
    
    // Format the tag line with ANSI color codes
    let tag_line = format!("  \x1b[33;1mTag:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet.tag);
    
    // Format the creation timestamp line with ANSI color codes
    let created_line = format!(
        "  \x1b[33;1mCreated:\x1b[0m \x1b[35;1m{}\x1b[0m",
        snippet.timestamp
    );
    
    // Format the description line if a description is available
    let description_line = if let Some(desc) = &snippet.description {
        format!("  \x1b[33;1mDescription:\x1b[0m \x1b[35;1m{}\x1b[0m", desc)
    } else {
        String::new()
    };

    // Collect all lines into a vector, stripping ANSI color codes for length calculation
    let all_lines = vec![
        strip_ansi_codes(&id_line),
        strip_ansi_codes(&tag_line),
        strip_ansi_codes(&created_line),
        strip_ansi_codes(&description_line),
    ]
    .into_iter()
    .collect::<Vec<_>>();

    // Determine the maximum line length for formatting
    let max_line_length = all_lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0);

    // Set the width for the formatted output, adding extra space for borders
    let adjusted_width = max_line_length + 4;

    // Print the top border of the summary box
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╔\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╗\x1b[0m"
    );
    
    // Print each formatted line within the border
    println!("{}", format_with_border(&id_line, adjusted_width));
    println!("{}", format_with_border(&tag_line, adjusted_width));
    println!("{}", format_with_border(&created_line, adjusted_width));
    if !description_line.is_empty() {
        println!("{}", format_with_border(&description_line, adjusted_width));
    }
    
    // Print the bottom border of the summary box
    println!(
        "\x1b[34m{}\x1b[0m",
        "\x1b[34m╚\x1b[0m".to_owned()
            + &"\x1b[34m═\x1b[0m".repeat(adjusted_width)
            + "\x1b[34m╝\x1b[0m\n"
    );
}

// Function to capture a code snippet from standard input
fn capture_snippet() -> String {
    let mut buffer = String::new(); // Create a buffer to store the input
    println!("\n\x1b[38;5;201;1mCapture snippet:\x1b[0m\n");
    println!("\x1b[1;36m Enter your code snippet (press \x1b[33m'Return'\x1b[1;36m, then \x1b[33m'Ctrl+D'\x1b[1;36m to finish):\x1b[0m");
    println!("\x1b[1;31m Note:\x1b[0m \x1b[1;33m'Arrow Keys'\x1b[0m \x1b[1;36mare not captured, use \x1b[1;33m'Backspace'\x1b[0m \x1b[1;36mto erase inputs\x1b[0m");
    
    // Read the entire input into the buffer
    io::stdin()
        .read_to_string(&mut buffer)
        .expect("\x1b[1;31mfailed to read snippet from input\x1b[0m");
    buffer
}

// Function to save a snippet to a JSON file
fn save_snippet(snippet: Snippet, file_path: &str) -> Result<(), String> {
    // Attempt to open the file and deserialize existing snippets
    let mut snippets: Vec<Snippet> = if let Ok(file) = File::open(file_path) {
        serde_json::from_reader(file).map_err(|err| {
            format!(
                "\x1b[31merror:\x1b[0m unable deserializing snippets {}",
                err
            )
        })?
    } else {
        Vec::new() // If file does not exist, start with an empty vector
    };

    snippets.push(snippet); // Add the new snippet to the vector

    // Open the file for writing and truncate it to overwrite existing content
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .map_err(|err| format!("\x1b[1;33m{}\x1b[0m", err))?;

    // Serialize the snippets vector to the file
    serde_json::to_writer_pretty(&mut file, &snippets)
        .map_err(|err| format!(" serializing snippets: '\x1b[1;33m{}\x1b[0m'", err))?;

    Ok(())
}

// Function to load snippets from a JSON file
fn load_snippets(file_path: &str) -> Result<Vec<Snippet>, String> {
    // Open the file and read its content into a vector of snippets
    let file =
        File::open(file_path).map_err(|err| format!("\x1b[1;33m opening file {}\x1b[0m", err))?;
    let snippets: Vec<Snippet> = serde_json::from_reader(file)
        .map_err(|err| format!("\x1b[1;33m reading snippets{}\x1b[0m", err))?;
    Ok(snippets)
}

// Function to view snippets based on various filters like ID, tag, language, and keyword
fn view_snippets(
    file_path: &str,
    id: &Option<u32>,
    tag: &Option<String>,
    language: &Option<String>,
    keyword: &Option<String>,
    _summary: bool,
) -> Result<Vec<Snippet>, String> {
    // Load all snippets from the specified file
    let snippets = load_snippets(file_path)?;

    // Filter snippets based on provided criteria
    let mut filtered_snippets = snippets
        .into_iter()
        .filter(|snippet| {
            // Check if the snippet's tag matches any of the provided tags
            let tag_match = if let Some(tag) = tag {
                let tags: Vec<&str> = tag.split(',').map(|s| s.trim()).collect();
                tags.iter()
                    .any(|t| snippet.tag.to_lowercase().contains(&t.to_lowercase()))
            } else {
                true
            };

            // Check if the snippet's language matches any of the provided languages
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

            // Check if the snippet contains any of the provided keywords in its tag, description, or code
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
        .collect::<Vec<_>>(); // Collect the filtered snippets into a vector

    // If an ID is specified, filter to include only the snippet with that ID
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

    Ok(filtered_snippets) // Return the filtered snippets
}

fn edit_snippet(
    file_path: &str,
    id: &Option<u32>,
    tag: &Option<String>,
    _language: &Option<String>,
    _supported_languages: &Vec<&str>,
) -> Result<(), String> {
    // Load existing snippets from the file
    let mut snippets = load_snippets(file_path)?;

    // Determine which snippet to edit based on ID or tag
    let mut snippet_to_edit = match (id, tag) {
        // If an ID is provided, find the snippet with that ID
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
        // If no ID is provided but a tag is, find snippets with that tag
        (None, Some(snippet_tag)) => {
            let matching_snippets: Vec<&Snippet> = snippets
                .iter()
                .filter(|s| s.tag.to_lowercase().contains(&snippet_tag.to_lowercase()))
                .collect();

            // Handle cases where no snippets or multiple snippets match the tag
            if matching_snippets.is_empty() {
                return Err(format!(
                    "Tag '\x1b[1;33m{}\x1b[0m' doesn't match any snippets. Try a different tag.",
                    snippet_tag
                ));
            } else if matching_snippets.len() > 1 {
                println!("\n\x1b[38;5;201;1mEdit snippet:\x1b[0m\n");
                println!("\x1b[1;36mMultiple matching tags found, choose an \x1b[1;33mID\x1b[1;36m to edit from list:\x1b[0m\n");
                for snippet in matching_snippets.iter() {
                    println!("\x1b[1;36m  »\x1b[0m \x1b[1;33mID {}\x1b[0m", snippet.id);
                }

                // Prompt the user to select an ID to edit
                loop {
                    print!("\n\x1b[1;36mType the \x1b[1;33mID\x1b[0m\x1b[1;36m of the snippet you want to modify: \x1b[0m");
                    io::stdout().flush().unwrap(); 
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    let input_trimmed = input.trim();

                    if let Ok(chosen_id) = input_trimmed.parse::<u32>() {
                        if matching_snippets.iter().any(|s| s.id == chosen_id) {
                            if let Some(index) = snippets.iter().position(|s| s.id == chosen_id) {
                                break snippets.remove(index);
                            }
                        } else {
                            println!(
                                "\x1b[1;31mID '\x1b[1;33m{}\x1b[0m\x1b[1;31m' is not in the list. Please choose a valid ID.\x1b[0m",
                                chosen_id
                            );
                        }
                    } else {
                        println!(
                            "\x1b[1;31mInvalid input. Please enter a valid numeric ID from the list.\x1b[0m"
                        );
                    }
                }
            } else {
                matching_snippets[0].clone()
            }
        }
        // If neither ID nor tag is provided, return an error
        (None, None) => {
            println!("\x1b[1;36mTo edit a snippet, use its \x1b[1;33mID\x1b[1;36m or \x1b[1;33mtag\x1b[0m\x1b[1;36m\x1b[0m");
            return Err("Enter a snippet ID or tag to proceed.".to_string());
        }
    };

    // Display current snippet details to the user
    println!("\n\x1b[38;5;201;1mEdit snippet:\x1b[0m\n");

    let id_line = format!("  \x1b[33;1mID:\x1b[0m \x1b[35;1m{}\x1b[0m", snippet_to_edit.id);
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

    // Find the length of the longest line for formatting purposes
    let longest_line = [
        id_line.len(),
        created_line.len(),
        tag_line.len(),
        description_line.len(),
    ]
    .iter()
    .max()
    .unwrap()
    .clone(); 

    // Create a border string based on the longest line length
    let border_string = "═".repeat(longest_line);

    // Print the snippet details with a formatted border
    println!(
        "\x1b[34;1m╔{}\x1b[0m",
        border_string.clone() + "╗\x1b[34;1m\x1b[0m"
    );
    println!("{}", id_line);
    println!("{}", tag_line);
    println!("{}", created_line);
    if !description_line.is_empty() {
        println!("{}", description_line);
    }
    println!(
        "\x1b[34;1m╚{}\x1b[0m",
        border_string.clone() + "╝\x1b[34;1m\x1b[0m"
    );
    println!(
        "\x1b[34;1m╔{}\x1b[0m",
        border_string.clone() + "╗\x1b[34;1m\x1b[0m"
    );

    // Prompt user for new tag, description, and language
    print!("\x1b[1m\x1b[36m  Enter new tag (\x1b[1;33mleave blank to keep current, press 'Return'\x1b[0m\x1b[36m): \x1b[0m");
    io::stdout().flush().unwrap();
    let mut new_tag = String::new();
    io::stdin().read_line(&mut new_tag).unwrap();
    if !new_tag.trim().is_empty() {
        snippet_to_edit.tag = new_tag.trim().to_string();
    }

    print!("\x1b[1m\x1b[36m  Enter new description (\x1b[1;33mleave blank to keep current, press 'Return'\x1b[0m\x1b[36m): \x1b[0m");
    io::stdout().flush().unwrap();
    let mut new_description = String::new();
    io::stdin().read_line(&mut new_description).unwrap();
    if !new_description.trim().is_empty() {
        snippet_to_edit.description = Some(new_description.trim().to_string());
    } else {
        snippet_to_edit.description = None;
    }

    print!("\x1b[1m\x1b[36m  Enter new language (\x1b[1;33mleave blank to keep current, press 'Return'\x1b[0m\x1b[36m): \x1b[0m");
    io::stdout().flush().unwrap();
    let mut new_language = String::new();
    io::stdin().read_line(&mut new_language).unwrap();
    if !new_language.trim().is_empty() {
        snippet_to_edit.language = Some(new_language.trim().to_string());
    } else {
        snippet_to_edit.language = None;
    }

    println!(
        "\x1b[34;1m╚{}\x1b[0m",
        border_string.clone() + "╝\x1b[34;1m\x1b[0m"
    );
    println!("\n  \x1b[33;1mCurrent Code:\x1b[0m\n");

    // Print the current code with syntax highlighting
    let highlighted_code = if let Some(lang) = &snippet_to_edit.language {
        highlight_code_snippets(&snippet_to_edit.code, lang)
    } else {
        snippet_to_edit.code.clone()
    };
    println!("  {}", highlighted_code);

    println!(
        "\x1b[34;1m╔{}\x1b[0m",
        border_string.clone() + "╗\x1b[34;1m\x1b[0m"
    );
    println!("\x1b[1;36m Enter your code snippet (press \x1b[33m'Return'\x1b[1;36m, then \x1b[33m'Ctrl+D'\x1b[1;36m to finish):\x1b[0m");
    println!("\x1b[1;31m Note:\x1b[0m \x1b[1;33m'Arrow Keys'\x1b[0m \x1b[1;36mare not captured, use \x1b[1;33m'Backspace'\x1b[0m \x1b[1;36mto erase inputs\x1b[0m");
    println!(
        "\x1b[34;1m╚{}\x1b[0m",
        border_string.clone() + "╝\x1b[34;1m\x1b[0m"
    );
    io::stdout().flush().unwrap();

    // Read the new code snippet from the user input
    let mut new_code = String::new();
    io::stdin()
        .read_to_string(&mut new_code)
        .expect("failed to read snippet from input");

    // Update the snippet with the new code and save it
    snippet_to_edit.code = new_code;
    snippets.push(snippet_to_edit);
    save_snippets_for_edit(snippets, file_path)?;

    Ok(())
}

// Function to save the updated list of snippets to a file
fn save_snippets_for_edit(snippets: Vec<Snippet>, file_path: &str) -> Result<(), String> {
    // Open the file with write, truncate, and create options
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(file_path)
        .map_err(|err| format!("\x1b[1;33m opening file{}\x1b[0m", err))?;

    // Serialize the snippets and write to the file
    serde_json::to_writer_pretty(&mut file, &snippets)
        .map_err(|err| format!(" serializing snippets: '\x1b[1;33m{}\x1b[0m'", err))?;
    Ok(())
}

// Function to copy a snippet based on its ID
fn copy_code(file_path: &str, id: &Option<u32>) -> Result<Snippet, String> {
    // Load snippets from the file
    let snippets = load_snippets(file_path)?;

    // Check if ID is provided
    if id.is_none() {
        let error_message = "missing snippet ID
        \nPlease provide a snippet ID using the \x1b[1m\x1b[36m-i\x1b[0m or \x1b[1m\x1b[36m--id\x1b[0m flag.
        \n\x1b[1m\x1b[32m\x1b[4mUsage:\x1b[0m \x1b[1m\x1b[36mcodevault copy\x1b[0m \x1b[1m\x1b[36m-i\x1b[0m \x1b[34m<ID>\x1b[0m
        \n\x1b[1m\x1b[32m\x1b[4mExample:\x1b[0m \x1b[1m\x1b[36mcodevault copy\x1b[0m \x1b[1m\x1b[36m-i\x1b[0m \x1b[1m\x1b[34m22\x1b[0m
        \nFor more information, try '\x1b[1m\x1b[36m--help\x1b[0m'".to_string();
        return Err(error_message);
    }

    // Extract the ID from the option
    let id = id.unwrap();

    // Find and return the snippet with the given ID
    if let Some(snippet) = snippets.iter().find(|s| s.id == id) {
        return Ok(snippet.clone());
    }

    // Return an error if the snippet ID is not found
    Err(format!(
        " snippet ID '\x1b[1;33m{}\x1b[0m' does not exist in the collection",
        id
    ))
}

// Function to delete snippets based on their IDs
fn delete_snippet(file_path: &str, ids: &[u32]) -> Result<(), String> {
    // Load the existing snippets from the file
    let mut snippets = load_snippets(file_path)?;

    // Vector to hold IDs that do not exist in the current snippets
    let mut non_existent_ids: Vec<u32> = Vec::new();
    // Check which of the provided IDs do not exist
    for id in ids {
        if !snippets.iter().any(|s| s.id == *id) {
            non_existent_ids.push(*id);
        }
    }

    // If there are non-existent IDs, return an error with their details
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

    // Prepare a string of IDs for confirmation prompt
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
    // Prompt user for confirmation
    print!("\x1b[1m\x1b[36mAre you sure you want to permanently delete {} {} ? Please confirm (\x1b[33my/N\x1b[36m): \x1b[0m", plural, ids_str);
    io::stdout().flush().unwrap();

    // Read user input for confirmation
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().to_lowercase() != "y" {
        print!("\n\x1b[91mSnippet deletion cancelled\x1b[0m\n");
        return Ok(());
    }

    // Remove snippets with the specified IDs
    let mut deleted_count = 0;
    for id in ids {
        if let Some(index) = snippets.iter().position(|s| s.id == *id) {
            snippets.remove(index);
            deleted_count += 1;
        }
    }

    // Save the remaining snippets back to the file
    if deleted_count > 0 {
        save_snippets_for_edit(snippets, file_path)?;
        println!("\n\x1b[32mdeleted successfully!\x1b[0m");
    }

    Ok(())
}

// Function to export snippets based on filters
fn export_snippets(
    file_path: &str,
    id: &Option<u32>,
    tag: &Option<String>,
    language: &Option<String>,
    export_path: &Option<PathBuf>,
) -> Result<(), String> {
    // Load the existing snippets from the file
    let snippets = load_snippets(file_path)?;

    // Start with all snippets and apply filters
    let mut filtered_snippets: Vec<Snippet> = snippets.clone();

    // Filter by snippet ID if provided
    if let Some(id) = id {
        filtered_snippets = filtered_snippets
            .into_iter()
            .filter(|s| s.id == *id)
            .collect::<Vec<_>>();
    }

    // Filter by tag if provided
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

    // Filter by language if provided
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

    // Check if any snippets match the filter criteria
    if filtered_snippets.is_empty() {
        if let Some(id) = id {
            return Err(format!(
                " snippet '\x1b[1;33m{}\x1b[0m' ID does not exist in the collection",
                id
            ));
        } else if let Some(tag) = tag {
            let tags: Vec<&str> = tag.split(',').map(|s| s.trim()).collect();
            if let Some(lang) = language {
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
                return Err(format!(
                    " snippet with '{}' tags does not exist in the collection",
                    tags.join(", ")
                ));
            }
        } else if let Some(lang) = language {
            let langs: Vec<&str> = lang.split(',').map(|s| s.trim()).collect();
            return Err(format!(
                " snippet with '{}' language does not exist in the collection",
                langs.join(", ")
            ));
        }
    }

    // Confirm export if more than one snippet is being exported
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

    // Determine export directory
    let export_dir = match export_path {
        Some(path) => path.clone(),
        None => {
            println!("\x1b[1m\x1b[36mNo export path specified. Exporting snippets to the default 'snippet_exports' directory. Please wait...\x1b[0m");
            PathBuf::from("snippet_exports")
        }
    };

    // Create the export directory if it does not exist
    std::fs::create_dir_all(&export_dir)
        .map_err(|err| format!("\x1b[31merror:\x1b[0m creating directory: {}\x1b[0m", err))?;

    // Export each snippet to a file
    for snippet in filtered_snippets {
        let _lowercase_lang = snippet.language.as_ref().map(|lang| lang.to_lowercase());

        // Determine file extension based on snippet language
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
            _ => "txt", // Default extension for unknown languages
        };

        // Create the filename for the exported snippet
        let filename = format!("{}/{}.{}", export_dir.display(), snippet.id, extension);

        // Check if the file already exists
        if std::fs::metadata(&filename).is_ok() {
            println!(
                "\n\x1b[1m\x1b[93mThe file has been already exported and is located at '{}'.\x1b[0m\x1b[0m",
                filename
            );
            continue;
        }

        // Create the file and write the snippet code to it
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

        // Confirm successful export
        println!(
            "\x1b[1;32m\nSuccessfully exported snippet to file '{}'.\x1b[0m",
            filename
        );
    }

    Ok(())
}
