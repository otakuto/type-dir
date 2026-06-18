use crate::walk::DirTree;

/// Helper that builds a schema dir tree.
pub fn schema_dir_tree(op_files: Vec<&str>) -> DirTree {
    DirTree {
        name: "schema".to_string(),
        dirs: vec![],
        files: op_files.into_iter().map(|f| f.to_string()).collect(),
    }
}
