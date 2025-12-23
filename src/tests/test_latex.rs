// #[cfg(test)]
// mod tests {
//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     // use super::*;
//     use crate::model as M;
//     use crate::rules::tex_rules::pdf_file::Pdffile;
//     use crate::rules::tex_rules::tex_file::Texfile;
//     use std::path::PathBuf;
//     use tempdir::TempDir;

//     fn prepare_srcdir() -> PathBuf {
//         let srcdir = TempDir::new("srcdir") // create a temporary directory for the source files
//             .unwrap()
//             .into_path()
//             .canonicalize()
//             .expect("canonicalize srcdir");
//         for f1 in vec!["project_latex/main.tex"] {
//             let mut f = PathBuf::from("../demo_projects");
//             f.push(f1);
//             let mut p = srcdir.clone();
//             p.push(f1);
//             std::fs::create_dir_all(p.parent().expect("parent")).expect("create parent dir");
//             std::fs::copy(f, p).expect("copy file");
//         }
//         srcdir
//     }

//     /// the nominal graph we use for the tests.
//     /// test will alter this graph to check specific features
//     async fn make_graph() -> Result<M::G, Box<dyn std::error::Error>> {
//         let srcdir = prepare_srcdir();
//         let sandbox = TempDir::new("example")
//             .unwrap()
//             .into_path()
//             .canonicalize()
//             .expect("canonicalize sandbox");

//         let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;
//         let include_paths = vec![sandbox.clone()];

//         g.add_node(Texfile::new(PathBuf::from("project_latex/main.tex"))?)?;
//         let pdf = PathBuf::from("project_latex/demo-pdf.pdf");
//         g.add_node(Pdffile::new(pdf.clone(), include_paths, vec![])?)?;
//         Ok(g)
//     }
// }
