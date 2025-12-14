use argh::FromArgs;
use log;
use std::path::PathBuf;
use yamake::helpers::log::setup_logger;

use yamake::model as M;
use yamake::rules::lilypond_rules::ly_file::Lyfile;
use yamake::rules::lilypond_rules::lytex_file::Lyoutputfile;

use yamake::rules::tex_rules::pdf_file::Pdffile;
use yamake::rules::tex_rules::tex_file::Texfile;

/// arguments for make
#[derive(Debug, FromArgs)]
struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "8")]
    nb_workers: u32,
    /// force rebuild
    #[argh(switch, short = 'f')]
    force: bool,
    /// the rootdir of the sources
    #[argh(positional)]
    srcdir: String,
    /// the sandbox directory where the build will take place
    #[argh(positional)]
    sandbox: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    // ANCHOR: instanciate

    log::info!("Latex demo project");

    // process::exit(1);

    let cli: Cli = argh::from_env();

    let srcdir = PathBuf::from(cli.srcdir)
        .canonicalize()
        .expect("canonicalize srcdir");
    let sandbox = PathBuf::from(cli.sandbox)
        .canonicalize()
        .expect("canonicalize sandbox");

    let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;

    // don't use the srcdir ! we take the header files from the sandbox
    // take everything from sandbox, there might be generated files

    for f in vec![
        "project_latex/main.tex",
        "project_latex/data.tex",
        "project_latex/projectile.tikz",
    ] {
        g.add_node(Texfile::new(f.into())?)?;
    }

    g.add_node(Lyfile::new("project_latex/solo.ly".into())?)?;
    g.add_node(Lyoutputfile::new(
        "project_latex/solo.output/solo.tex".into(),
    )?)?;

    let mut p = sandbox.clone();
    p.push("project_latex");
    let include_paths = vec![p];
    let flags = vec![];
    g.add_node(Pdffile::new(
        "project_latex/main.pdf".into(),
        include_paths,
        flags,
    )?)?;

    g.add_edge(
        "project_latex/solo.output/solo.tex".into(),
        "project_latex/solo.ly".into(),
    )?;

    g.add_edge(
        "project_latex/main.pdf".into(),
        "project_latex/main.tex".into(),
    )?;

    g.add_edge(
        "project_latex/main.pdf".into(),
        "project_latex/solo.output/solo.tex".into(),
    )?;

    // ANCHOR_END: add_edges

    match g.make(cli.force, cli.nb_workers).await {
        Ok(ret) => {
            println!("success : {}", ret.success);
            // you can walk the graph and print status of each node
            // for (k, v) in ret.nt {
            //     println!("node {:?} : {:?}", k, v);
            // }
        }
        Err(e) => println!("{}", e.to_string()),
    };

    Ok(())
}
