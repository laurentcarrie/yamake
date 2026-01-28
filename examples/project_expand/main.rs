//! Example: Dynamic code generation with expand.
//!
//! This example demonstrates yamake's `expand` feature, which allows nodes
//! to dynamically generate additional nodes and edges during the build process.
//!
//! # How Expand Works
//!
//! 1. A YAML configuration file (`languages.yml`) defines available languages
//! 2. `YmlDesc` (root node) is mounted to the sandbox
//! 3. `JsonDesc` converts YAML to JSON and calls `expand()` after building
//! 4. `expand()` generates C source files, headers, and object files for each language
//! 5. These generated files are compiled into a static library (`liblangs.a`)
//! 6. The main executable links against this library
//!
//! # Project Structure
//!
//! ```text
//! project_expand/
//! ├── main.c           # Main source (uses generated headers)
//! ├── wrapper.h        # Includes generated languages.h
//! ├── languages.yml    # Configuration: list of languages
//! └── generated/       # Created by expand()
//!     ├── languages.h  # Generated: function pointer array
//!     ├── english.c    # Generated: "Hello, World!"
//!     ├── english.h
//!     ├── french.c     # Generated: "Bonjour!"
//!     ├── french.h
//!     └── liblangs.a   # Static library
//! ```
//!
//! # Build Graph (after expand)
//!
//! ```text
//! languages.yml ──► languages.json ──► [expand generates:]
//!                                       ├── english.c ──► english.o ──┐
//!                                       ├── french.c ──► french.o ────┼──► liblangs.a
//!                                       └── languages.h               │         │
//!                                                                     │         │
//! main.c ──────────────────────────────► main.o ──────────────────────┴─────────┴──► app
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example project_expand -- -s demo_projects -b /tmp/sandbox
//! ```

#[path = "../../tests/common/mod.rs"]
mod common;

use argh::FromArgs;
use common::{JsonDesc, YmlDesc};
use log::info;
use std::fs;
use std::path::PathBuf;
use yamake::c_nodes::{AFile, CFile, HFile, OFile, XFile};
use yamake::model::G;

/// Command-line arguments for the project_expand example.
#[derive(FromArgs)]
/// A C project build example demonstrating expand functionality
struct Args {
    /// source directory
    #[argh(option, short = 's')]
    src: PathBuf,

    /// sandbox directory
    #[argh(option, short = 'b')]
    sandbox: PathBuf,
}

fn main() {
    env_logger::init();

    let args: Args = argh::from_env();

    let srcdir = args.src;
    let sandbox = args.sandbox;

    let mut g = G::new(srcdir.clone(), sandbox);

    let main_c = g
        .add_root_node(CFile::new("project_expand/main.c"))
        .unwrap();
    let main_o = g
        .add_node(OFile::new("project_expand/main.o", vec![], vec![]))
        .unwrap();
    let _wrapper_h = g
        .add_root_node(HFile::new("project_expand/wrapper.h"))
        .unwrap();
    let app = g.add_node(XFile::new("project_expand/app")).unwrap();

    let languages_yml = g
        .add_root_node(YmlDesc::new("project_expand/languages.yml"))
        .unwrap();
    let languages_json = g
        .add_node(JsonDesc::new("project_expand/languages.json"))
        .unwrap();
    let liblangs = g
        .add_node(AFile::new("project_expand/generated/liblangs.a"))
        .unwrap();

    g.add_edge(main_c, main_o);
    g.add_edge(main_o, app);
    g.add_edge(languages_yml, languages_json);
    g.add_edge(languages_json, liblangs);
    g.add_edge(liblangs, app);

    info!(
        "Created graph with {} nodes and {} edges",
        g.g.node_count(),
        g.g.edge_count()
    );

    let success = g.make();
    info!("Build {}", if success { "succeeded" } else { "failed" });

    // Write mermaid graph to file
    let mermaid_path = srcdir.join("project_expand/graph.mermaid");
    let mermaid = g.to_mermaid();
    fs::write(&mermaid_path, &mermaid).expect("Failed to write graph.mermaid");
    info!("Wrote mermaid graph to {}", mermaid_path.display());
}
