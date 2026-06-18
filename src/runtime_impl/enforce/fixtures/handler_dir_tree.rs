use crate::walk::DirTree;

/// Helper that builds a handler dir tree.
pub fn handler_dir_tree(handler_children_dirs: Vec<&str>, handler_files: Vec<&str>) -> DirTree {
    DirTree {
        name: "handler".to_string(),
        dirs: handler_children_dirs
            .into_iter()
            .map(|n| DirTree {
                name: n.to_string(),
                dirs: vec![],
                files: vec![],
            })
            .collect(),
        files: handler_files.into_iter().map(|f| f.to_string()).collect(),
    }
}
