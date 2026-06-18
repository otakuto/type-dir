/// Represents a tree that holds the immediate child directories (recursive) and child file names of a directory.
#[derive(Debug)]
pub struct DirTree {
    pub name: String,
    pub dirs: Vec<DirTree>,
    pub files: Vec<String>,
}
