# Codevault: Your Personal Code Snippet Manager

![Codevault is a simple and powerful CLI tool designed to manage your code snippets, helping you stay organized and productive. Whether you're a seasoned developer or just starting out, Codevault streamlines your code snippet management by offering features such as:](https://github.com/Quiflern/Codevault/blob/main/docs/codevault.png?raw=true)
Codevault is a simple and powerful CLI tool designed to manage your code snippets, helping you stay organized and productive. Whether you're a seasoned developer or just starting out, Codevault streamlines your code snippet management by offering features such as:

- **Capture and Store Snippets:** Capture code from the terminal, saving it alongside descriptions, tags, and the language for syntax highlighting by using **`capture`** command.
- **Organize Snippets with Tags:** Apply tags to categorize snippets, making them easily searchable later.
- **Search and Retrieve Snippets:** Effortlessly find snippets by using a combination of IDs, tags, keywords, and language filters, using **`view`** command.
- **Copy Snippets:** Copy the code view only the code for quick and easy copy pasting , by using **`copy`** command.
- **Edit and Update Snippets:** Modify descriptions, tags, and code of existing snippets with **`edit`** command.
- **Export Snippets:** Export individual or multiple snippets various language-specific, formats or .txt fomat for unsupported languages with the help of **`export`** command.
- **View Supported Languages:** Get a comprehensive list of languages that Codevault supports for syntax highlighting.

**Why CodeVault?**

- **Simplify Code Snippet Management:** Forget juggling various files or trying to remember where reuse the code. Codevault provides a central repository for all your code fragments.
- **Enhance Code Reusability:** Quickly find and reuse existing code snippets instead of rewriting the same logic again and again.
- **Boost Productivity:** Minimize distractions and maximize coding time by having all your snippets readily accessible in an organized manner.

#### Supported Languages for Syntax Highlighting:

| Plain Text         | ASP                          | HTML (ASP)                | ActionScript                     | AppleScript                 |
| ------------------ | ---------------------------- | ------------------------- | -------------------------------- | --------------------------- |
| Batch File         | NAnt Build File              | C#                        | C++                              | C                           |
| CSS                | Clojure                      | D                         | Diff                             | Erlang                      |
| HTML (Erlang)      | Go                           | Graphviz (DOT)            | Groovy                           | HTML                        |
| Haskell            | Literate Haskell             | Java Server Page (JSP)    | Java                             | JavaDoc                     |
| Java Properties    | JSON                         | JavaScript                | Regular Expressions (Javascript) | BibTeX                      |
| LaTeX Log          | LaTeX                        | TeX                       | Lisp                             | Lua                         |
| Make Output        | Makefile                     | Markdown                  | MultiMarkdown                    | MATLAB                      |
| OCaml              | OCamllex                     | OCamlyacc                 | camlp4                           | Objective-C++               |
| Objective-C        | PHP Source                   | PHP                       | Pascal                           | Perl                        |
| Python             | Regular Expressions (Python) | R Console                 | R                                | Rd (R Documentation)        |
| HTML (Rails)       | JavaScript (Rails)           | Ruby Haml                 | Ruby on Rails                    | SQL (Rails)                 |
| Regular Expression | reStructuredText             | Ruby                      | Cargo Build Results              | Rust                        |
| SQL                | Scala                        | Bourne Again Shell (bash) | Shell-Unix-Generic               | commands-builtin-shell-bash |
| HTML (Tcl)         | Tcl                          | Textile                   | XML                              | YAML                        |

### **Get Started: Building Your Snippet Sanctuary**

**1. Install Rust:**
**Prerequisites:**
You'll need Rust and Cargo installed on your system. - Check Rust's presence: `rustc --version`

**Install Rust:**
Install using the official installer: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
OR
Curl command(**Linux/Mac**):

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

**2. Clone CodeVault:**

    git clone https://github.com/quiflern/codevault.git
    cd codevault

**3. Build CodeVault:**

    cargo build

**4. Run CodeVault:**

    target/debug/codevault
    	or
    cargo run

## **Master the Codevault Commands:**

### Capture Command:

The **`capture`** command save your code and add it to snippet collection.

**Usage:**

    target/debug/codevault capture --description <DESCRIPTION> --language <LANGUAGE> --tag <TAG>

**Options:**

- **-d, --description `<description>`:** Add a descriptive label for your snippet.
  _eg:_ `-d sample description` | `--description "sample description"`.
- **-l, --language `<language>`:** Specify the programming language for accurate syntax highlighting.
  _eg:_ `-l Rust` | `--language"Rust"`.
- **-t, --tag `<tag>`:** Assign relevant tags to organize your snippet (`e.g., "algorithm," "api," "data"`). Multiple tags are supported it should be `,` comma separated.
  _eg:_ `-t algorithm,api ` | `--tag "algorithm"`.

**Example:**
**To save new snippet execute:**

    target/debug/codevault capture --description "This is short description for the code snippet" --language "JavaScript" --tag "sample javascript code"

**Demo:**

[capture-demo.webm](https://github.com/user-attachments/assets/d1831499-1920-4b5d-a7d4-a639be302064)

### Copy Command:

The **`copy`** command displays the code stored in snippet, and shows only the code of the snippet.
**Usage:**

    target/debug/codevault copy --id <id>

**Options:**

- **-i, --id `<id>`:** Unique ID automatically assigned for identification of the snippets.
  _eg:_ `-i 1 ` | `--id 1`.

**Examples:**
**To view the code of ID "1", execute:**

    target/debug/codevault copy --id 1

**Demo:**

[copy-demo.webm](https://github.com/user-attachments/assets/4d3c3b33-a555-4969-8f50-1cf03923840e)

### Delete Command:

The **`delete`** command remove snippets from your snippet collection.

**Usage:**

    target/debug/codevault delete --id <id>

**Options:**

- **-i, --id `<id>`:** Allows deleting with ID.
  _eg:_ `-i 1 ` | `--id 1`.

**Example:**
**To delete the snippet with ID "1", execute:**

    target/debug/codevault delete --id 1

**Demo:**

[delete-demo.webm](https://github.com/user-attachments/assets/ee23373f-fc53-49f2-b089-380faeecfd02)

### Edit Command:

The `edit` command allows you edit existing snippets.

**Usage:**

    target/debug/codevault edit [options]

**Options:**

- **-i, --id `<id>`:** Allows editing with a specified ID.
- **-t, --tag `<tag>`:** Allows editing with a specified tag.

**Examples:**

1. **Edit snippet with tags:**
   ```
   target/debug/codevault edit -t data
   ```
2. **Edit snippet with ID 1:**
   ```
   target/debug/codevault edit -i 1
   ```

**Demo:**

[edit-function-with-id.webm](https://github.com/user-attachments/assets/056e8352-d765-49ec-8608-05741fe58946)

### Export Command:

The **`export`** command exports saved snippets in language-specific format from your collection to default **`export_snippets`** directory, if path is not specified .

**Usage:**

    target/debug/codevault export [options]

**Options:**

- **-i, --id `<id>`:** Exports specified ID.
  eg.
- **-t, --tag `<tag>`:** Exports with a specified tag.
- **-l, --language `<language>`:** Export all snippets that matches specified language.
- **-p, --path `<language>`:** Export snippets specified path.

**Examples:**

- **Exporting all the snippets of `export_snippets` directory :**
  ```
  target/debug/codevault export
  ```
- **Exporting an snippet using id ;**
  ```
  target/debug/codevault export --id 1
  ```
- **Export an snippet using tags only ;**
  ```
  target/debug/codevault export --tags Rust
  ```
- **Export an snippet using languages only ;**
  ```
  target/debug/codevault export --language Rust
  ```
- **Advanced Exporting Options:**

  - **Exporting of snippets matches tags & languages to the specified path;**
    ```
    target/debug/codevault export --tag data --language Rust --path "export/html/"
    ```
  - **Multiple tags; **
    ```
    target/debug/codevault export --tag data,html
    ```
  - **Multiple Languages;**
    ```
    target/debug/codevault export --language Rust,C++
    ```
  - **Mutiple tags & Languages;**
    ```
    target/debug/codevault export --language Rust,C++
    ```

- **Path examples:**
  If the path is not specified it export to **`snippet_exports`** directory, which will created on the root directory (_i.e location where the program is opened)_.
  `    target/debug/codevault export --language Rust --path "codes/rust"`

**Demo :**
DEMO VIDEO

### View Command:

The **`view`** command allows you to list all snippets.
**Usage:**

    target/debug/codevault view [options]

**Options:**

- **-i, --id `<id>`:** View information for the snippet with a specific ID.
- **-k, --keyword `<keyword>`:** Search for snippets containing the specified keyword.
- **-t, --tag `<tag>`:** View snippets with the specified tag.
- **-l, --language `<language>`:** View snippets written in a specific language.
- **-s, --summary:** Show only **`id, tags, timestamp & description`** of the snippet.

**Examples:**

- **Viewing all the snippets :**
  ```
  target/debug/codevault view
  ```
- **Viewing an snippet using id:**
  ```
  target/debug/codevault view --id 1
  ```
- **Viewing an snippet using tags only:**
  ```
  target/debug/codevault view --tags Rust
  ```
- **Viewing an snippet using languages only :**
  ```
  target/debug/codevault view --language Rust
  ```
- **Advanced Viewing Options :**

  - **Multiple tags;**
    ```
    target/debug/codevault view --tag data,html
    ```
  - **Multiple Languages;**
    ```
    target/debug/codevault view --language Rust,C++
    ```

**Demo :**
DEMO VIDEO

**Embrace a Smoother, More Productive Coding Experience with Codevault.** Start capturing, organizing, and reusing your code snippets to level up your development experience.
