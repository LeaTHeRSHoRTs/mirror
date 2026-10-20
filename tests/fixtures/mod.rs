use crate::file_tree;

use tempfile::TempDir;

pub fn file_tree_nested() -> (TempDir, TempDir) {
    file_tree! {
        "subfolder/nested/deep.txt" => "Deep",
    }
}

pub fn file_tree_basic() -> (TempDir, TempDir) {
    file_tree! {
        "root.txt" => "Root",
        "subfolder/nested/deep.txt" => "Deep",
    }
}

pub fn file_tree_complex() -> (TempDir, TempDir) {
    file_tree! {
        "root.txt" => "Root",
        "root2.txt" => "Root2",
        "subfolder/nested/deep.txt" => "Deep",
        "subfolder/nested/deep2.txt" => "Deep 2",
        "subfolder/nested/deeper/deeper.txt" => "Deeper",
        "subfolder/nested/deeper/deeper2.txt" => "Deeper 2",
    }
}