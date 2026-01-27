use super::Language;
use std::fs;
use std::path::{Path, PathBuf};
use yamake::c_nodes::{AFile, CFile, HFile, OFile};
use yamake::model::{Edge, ExpandResult, GNode};

pub struct JsonDesc {
    pub name: String,
}

impl JsonDesc {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl GNode for JsonDesc {
    fn tag(&self) -> String {
        "JsonDesc".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }

    fn build(&self, sandbox: &Path, predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        assert!(
            predecessors.len() == 1,
            "JsonDesc expects exactly one predecessor, got {}",
            predecessors.len()
        );

        let predecessor = predecessors[0];
        assert!(
            predecessor.tag() == "YmlDesc",
            "JsonDesc predecessor must be YmlDesc, got {}",
            predecessor.tag()
        );

        let input_path = sandbox.join(predecessor.pathbuf());
        let yaml_content = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path.display(), e));

        let languages: Vec<Language> = serde_yaml::from_str(&yaml_content)
            .unwrap_or_else(|e| panic!("Failed to parse YAML {}: {}", input_path.display(), e));

        let json_content = serde_json::to_string_pretty(&languages)
            .unwrap_or_else(|e| panic!("Failed to serialize to JSON: {}", e));

        let output_path = sandbox.join(&self.name);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(&output_path, json_content)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", output_path.display(), e));

        true
    }

    fn expand(&self, sandbox: &Path, _predecessors: &[&(dyn GNode + Send + Sync)]) -> ExpandResult {
        let input_path = sandbox.join(&self.name);
        let json_content = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path.display(), e));

        let languages: Vec<Language> = serde_json::from_str(&json_content)
            .unwrap_or_else(|e| panic!("Failed to parse JSON {}: {}", input_path.display(), e));

        let generated_dir = sandbox.join("project_expand/generated");
        fs::create_dir_all(&generated_dir).ok();

        let mut nodes: Vec<Box<dyn GNode + Send + Sync>> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();

        let lib_path = "project_expand/generated/liblangs.a";

        for lang in &languages {
            // Generate C file
            let c_path = format!("project_expand/generated/{}.c", lang.language);
            let c_content = format!(
                "const char* helloworld_{}=\"{}\" ;\nconst char* get_{}() {{ return helloworld_{} ; }}\n",
                lang.language, lang.helloworld, lang.language, lang.language
            );
            fs::write(sandbox.join(&c_path), &c_content)
                .unwrap_or_else(|e| panic!("Failed to write {}: {}", c_path, e));
            nodes.push(Box::new(CFile::new(&c_path)));

            // Generate H file
            let h_path = format!("project_expand/generated/{}.h", lang.language);
            let h_content = format!("const char* get_{}() ;\n", lang.language);
            fs::write(sandbox.join(&h_path), &h_content)
                .unwrap_or_else(|e| panic!("Failed to write {}: {}", h_path, e));
            nodes.push(Box::new(HFile::new(&h_path)));

            // Create OFile for this language
            let o_path = format!("project_expand/generated/{}.o", lang.language);
            nodes.push(Box::new(OFile::new(&o_path, vec![], vec![])));

            // Add edges from JsonDesc to CFile and HFile (so they are not treated as root nodes)
            edges.push(Edge {
                nfrom: Box::new(JsonDesc::new(&self.name)),
                nto: Box::new(CFile::new(&c_path)),
            });
            edges.push(Edge {
                nfrom: Box::new(JsonDesc::new(&self.name)),
                nto: Box::new(HFile::new(&h_path)),
            });

            // Add edges: CFile -> OFile -> AFile
            edges.push(Edge {
                nfrom: Box::new(CFile::new(&c_path)),
                nto: Box::new(OFile::new(&o_path, vec![], vec![])),
            });
            edges.push(Edge {
                nfrom: Box::new(OFile::new(&o_path, vec![], vec![])),
                nto: Box::new(AFile::new(lib_path)),
            });
        }

        // Generate languages.h with array of function pointers
        let languages_h_path = PathBuf::from("project_expand/generated/languages.h");
        let mut h_content = String::new();
        h_content.push_str("#ifndef LANGUAGES_H\n");
        h_content.push_str("#define LANGUAGES_H\n\n");

        for lang in &languages {
            h_content.push_str(&format!(
                "#include \"project_expand/generated/{}.h\"\n",
                lang.language
            ));
        }
        h_content.push_str("\n");

        h_content.push_str(&format!("#define N_languages {}\n\n", languages.len()));

        h_content.push_str("typedef const char* (*get_language_fn)() ;\n\n");
        h_content.push_str("get_language_fn languages[] = {\n");
        for lang in &languages {
            h_content.push_str(&format!("    get_{},\n", lang.language));
        }
        h_content.push_str("} ;\n\n");

        h_content.push_str("#endif\n");

        fs::write(sandbox.join(&languages_h_path), &h_content)
            .unwrap_or_else(|e| panic!("Failed to write {}: {}", languages_h_path.display(), e));
        nodes.push(Box::new(HFile::new(&languages_h_path.to_string_lossy())));

        // Add edge from JsonDesc to languages.h
        edges.push(Edge {
            nfrom: Box::new(JsonDesc::new(&self.name)),
            nto: Box::new(HFile::new(&languages_h_path.to_string_lossy())),
        });

        (nodes, edges)
    }
}
