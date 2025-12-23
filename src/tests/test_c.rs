#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    // use super::*;
    use crate::error as E;
    use crate::model as M;
    use crate::rules::c_rules as R;
    // use futures::executor;
    // use simple_logging;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn setup() {
        println!("XXXXXXXXXXXXXXXXXXXXX");
        crate::helpers::log::setup_logger(false, log::LevelFilter::Error).unwrap_or(())

        // INIT.call_once(|| {
        //     crate::helpers::log::setup_logger(false, log::LevelFilter::Error).unwrap_or(())
        // });
    }

    fn prepare_srcdir() -> PathBuf {
        let srcdir = TempDir::new("srcdir") // create a temporary directory for the source files
            .unwrap()
            .into_path()
            .canonicalize()
            .expect("canonicalize srcdir");
        for f1 in vec![
            "project_1/main.c",
            "project_1/add.c",
            "project_1/add.h",
            "project_1/wrapper.h",
        ] {
            let mut f = PathBuf::from("demo_projects");
            f.push(f1);
            let mut p = srcdir.clone();
            p.push(f1);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("create parent dir");
            std::fs::copy(f, p).expect("copy file");
        }
        srcdir
    }

    /// the nominal graph we use for the tests.
    /// test will alter this graph to check specific features
    async fn make_graph() -> Result<M::G, Box<dyn std::error::Error>> {
        let srcdir = prepare_srcdir();
        let sandbox = TempDir::new("example")
            .unwrap()
            .into_path()
            .canonicalize()
            .expect("canonicalize sandbox");

        let mut g = M::G::new(srcdir.clone(), sandbox.clone())?;
        let include_paths = vec![sandbox.clone()];
        let compile_flags = vec!["-Wall".into()];

        g.add_node(R::c_file::new(PathBuf::from("project_1/main.c"))?)?;
        g.add_node(R::c_file::new(PathBuf::from("project_1/add.c"))?)?;
        g.add_node(R::h_file::new(PathBuf::from("project_1/add.h"))?)?;
        g.add_node(R::h_file::new(PathBuf::from("project_1/wrapper.h"))?)?;
        g.add_node(R::o_file::new(
            PathBuf::from("project_1/main.o"),
            include_paths.clone(),
            compile_flags.clone(),
        )?)?;
        g.add_node(R::o_file::new(
            PathBuf::from("project_1/add.o"),
            include_paths.clone(),
            compile_flags.clone(),
        )?)?;

        g.add_edge(
            PathBuf::from("project_1/main.o"),
            PathBuf::from("project_1/main.c"),
        )?;
        g.add_edge(
            PathBuf::from("project_1/add.o"),
            PathBuf::from("project_1/add.c"),
        )?;

        let exe = PathBuf::from("project_1/demo");
        g.add_node(R::x_file::new(exe.clone(), vec!["-lm".into()])?)?;
        g.add_edge(exe.clone(), PathBuf::from("project_1/main.o"))?;
        g.add_edge(exe.clone(), PathBuf::from("project_1/add.o"))?;
        Ok(g)
    }

    /// nominal test, everything is ok
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_nominal() -> Result<(), Box<dyn std::error::Error>> {
        setup();
        let mut g = make_graph().await?;
        println!("sandbox is {:?}", g.sandbox);
        let ret: M::MakeReturn = g.make(false, 4).await?;
        log::info!("{:?}", ret);

        let exe = {
            let mut exe = g.sandbox.clone();
            exe.push("project_1/demo");
            exe
        };

        assert!(exe.exists());
        assert!(ret.success);

        Ok(())
    }

    /// forgot a source node, build should fail
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_forgot_header() -> Result<(), Box<dyn std::error::Error>> {
        setup();
        let mut g = make_graph().await?;
        let ni = g.ni_of_path("project_1/add.h".into())?;
        g.g.remove_node(ni).ok_or("x")?;
        let ret: M::MakeReturn = g.make(false, 4).await?;
        log::info!("{:?}", ret);

        let exe = {
            let mut exe = g.sandbox.clone();
            exe.push("project_1/demo");
            exe
        };

        assert!(!exe.exists());
        assert!(!ret.success);

        let check = vec![
            ("project_1/add.o", M::BuildType::Failed),
            ("project_1/main.o", M::BuildType::Failed),
        ];

        for (p, bt) in check {
            let bt_found = ret.nt.get(&g.ni_of_path(p.into())?).ok_or("xxxx")?;
            log::info!("{:?}", bt);
            assert_eq!(*bt_found, bt);
        }

        // assert_eq!(
        //     ret.nt.get("project_1/add.h").ok_or("x"),
        //     M::BuildType::Failed
        // );

        Ok(())
    }

    /// an o-file was inserted in the graph as a c-file, and therefore has no build rule
    /// build should fail with error that main.o could not be mounted
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_ofile_captured_as_cfile() -> Result<(), Box<dyn std::error::Error>> {
        setup();
        let mut g = make_graph().await?;
        println!("sandbox is {:?}", g.sandbox);
        let ni = g.ni_of_path("project_1/add.o".into())?;
        g.g.remove_node(ni).ok_or("x")?;
        g.add_node(R::c_file::new(PathBuf::from("project_1/add.o"))?)?;

        let ret = g.make(false, 4).await;
        assert!(ret.is_err_and(|e| e.is::<E::CouldNotMountFileError>()));

        Ok(())
    }

    /// test that if we rebuild the graph is untouched
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_rebuild_untouched() -> Result<(), Box<dyn std::error::Error>> {
        setup();
        let mut g = make_graph().await?;
        println!("sandbox is {:?}", g.sandbox);
        let ret = g.make(false, 4).await?;
        assert!(ret.success);
        log::info!("second run");
        let ret = g.make(false, 4).await?;
        assert!(ret.success);

        for (ni, v) in ret.nt.iter() {
            let node = g.g.node_weight(*ni).ok_or("x")?;
            match node.target().to_str().unwrap() {
                "project_1/add.h"
                | "project_1/add.c"
                | "project_1/main.c"
                | "project_1/main.h"
                | "project_1/wrapper.h" => {
                    assert_eq!(*v, M::BuildType::MountNotChanged(node.target()))
                }
                "project_1/main.o" | "project_1/demo" | "project_1/add.o" => {
                    assert_eq!(*v, M::BuildType::NotRebuilt(node.target()))
                }

                _ => {
                    log::error!("{:?}", node);
                    assert!(false)
                }
            };
        }

        Ok(())
    }

    /// test that if we delete a built node and remake, then this node has status r
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_rebuild_untouched2() -> Result<(), Box<dyn std::error::Error>> {
        // setup();
        crate::helpers::log::setup_logger(false, log::LevelFilter::Info)?;

        let mut g = make_graph().await?;
        println!("sandbox is {:?}", g.sandbox);
        let ret = g.make(false, 4).await?;
        assert!(ret.success);
        log::info!("delete add.o");
        let mut p = g.sandbox.clone();
        p.push("project_1");
        p.push("add.o");
        assert!(p.exists());
        let _ = std::fs::remove_file(&p);
        assert!(!p.exists());

        log::info!("second run");
        let ret = g.make(false, 4).await?;
        assert!(ret.success);
        for (ni, value) in &ret.nt {
            let node = g.g.node_weight(*ni).ok_or("x")?;
            // log::info!("xxx : {:?} ; {:?}",&ni,&value) ;
            match node.target().to_str().unwrap() {
                "project_1/add.o" => assert_eq!(*value, M::BuildType::Rebuilt(node.target())),
                "project_1/main.o" => assert_eq!(*value, M::BuildType::NotRebuilt(node.target())),
                "project_1/demo" => {
                    assert_eq!(*value, M::BuildType::RebuiltButUnchanged(node.target()))
                }
                _ => assert_eq!(*value, M::BuildType::MountNotChanged(node.target())),
            }
        }

        // for (ni, v) in ret.nt.iter() {
        //     let node = g.g.node_weight(*ni).ok_or("x")?;
        //     match node.target().to_str().unwrap() {
        //         "project_1/add.o" => assert_eq!(*v, M::BuildType::NotTouched(node.target())),
        //         _ => assert_eq!(*v, M::BuildType::NotTouched(node.target())),
        //      }
        // }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_node_state_1() -> Result<(), Box<dyn std::error::Error>> {
        // setup();
        crate::helpers::log::setup_logger(false, log::LevelFilter::Info)?;
        let mut g = make_graph().await?;

        let ni = g.ni_of_path("project_1/add.o".into()).unwrap();
        crate::actions::node_life::compute_node_state(&mut g, ni).await?;
        Ok(())
    }
}
