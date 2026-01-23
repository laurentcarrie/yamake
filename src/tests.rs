#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::model::{G, GNode, GRootNode, GraphError};

    struct SourceFile {
        name: String,
    }

    impl GRootNode for SourceFile {
        fn id(&self) -> String {
            self.name.clone()
        }

        fn tag(&self) -> String {
            "SourceFile".to_string()
        }

        fn pathbuf(&self) -> PathBuf {
            PathBuf::from(&self.name)
        }
    }

    struct TargetFile {
        name: String,
        path: PathBuf,
    }

    impl GNode for TargetFile {
        fn build(&self, _sandbox: &PathBuf, predecessors: &[&Box<dyn GNode + Send + Sync>]) -> bool {
            let _inputs: Vec<String> = predecessors.iter().map(|p| p.id()).collect();
            true
        }

        fn id(&self) -> String {
            self.name.clone()
        }

        fn tag(&self) -> String {
            "TargetFile".to_string()
        }

        fn pathbuf(&self) -> PathBuf {
            self.path.clone()
        }
    }

    #[test]
    fn test_construct_and_build_graph() {
        let mut g = G::new(
            PathBuf::from("src"),
            PathBuf::from("build"),
        );

        let src = g.add_root_node(SourceFile { name: "input.txt".to_string() }).unwrap();
        let target = g.add_node(TargetFile {
            name: "output.txt".to_string(),
            path: PathBuf::from("output.txt"),
        }).unwrap();

        g.add_edge(src, target);

        assert_eq!(g.g.node_count(), 2);
        assert_eq!(g.g.edge_count(), 1);

        g.build_graph(src);
    }

    #[test]
    fn test_duplicate_id_error() {
        let mut g = G::new(
            PathBuf::from("src"),
            PathBuf::from("build"),
        );

        g.add_root_node(SourceFile { name: "input.txt".to_string() }).unwrap();

        let result = g.add_root_node(SourceFile { name: "input.txt".to_string() });

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphError::DuplicateId(_)));
    }

    #[test]
    fn test_duplicate_pathbuf_error() {
        let mut g = G::new(
            PathBuf::from("src"),
            PathBuf::from("build"),
        );

        g.add_node(TargetFile {
            name: "node1".to_string(),
            path: PathBuf::from("same/path.txt"),
        }).unwrap();

        let result = g.add_node(TargetFile {
            name: "node2".to_string(),
            path: PathBuf::from("same/path.txt"),
        });

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphError::DuplicatePathBuf(_)));
    }
}
