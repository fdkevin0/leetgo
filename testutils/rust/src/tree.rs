use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use serde::de::SeqAccess;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize, Serializer};

// LeetCode use `Option<Rc<RefCell<TreeNode>>>` for tree links, but `Option<Box<TreeNode>>` should be enough.
// https://github.com/pretzelhammer/rust-blog/blob/master/posts/learning-rust-in-2020.md#leetcode
type TreeLink = Option<Rc<RefCell<TreeNode>>>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreeNode {
    pub val: i32,
    pub left: TreeLink,
    pub right: TreeLink,
}

impl TreeNode {
    #[inline]
    pub fn new(val: i32) -> Self {
        TreeNode {
            val,
            left: None,
            right: None,
        }
    }
}

#[macro_export]
macro_rules! tree {
    () => {
        None
    };
    ($e:expr) => {
        Some(Rc::new(RefCell::new(TreeNode::new($e))))
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryTree(TreeLink);

impl From<BinaryTree> for TreeLink {
    fn from(tree: BinaryTree) -> Self {
        tree.0
    }
}

impl From<TreeLink> for BinaryTree {
    fn from(link: TreeLink) -> Self {
        BinaryTree(link)
    }
}

impl Serialize for BinaryTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut queue = VecDeque::new();
        let mut values = Vec::new();

        queue.push_back(self.0.clone());
        while let Some(node) = queue.pop_front() {
            match node {
                Some(node) => {
                    let node = node.borrow();
                    values.push(Some(node.val));
                    queue.push_back(node.left.clone());
                    queue.push_back(node.right.clone());
                }
                None => values.push(None),
            }
        }

        while matches!(values.last(), Some(None)) {
            values.pop();
        }

        let mut seq = serializer.serialize_seq(Some(values.len()))?;
        for value in values {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

struct BinaryTreeVisitor;

impl BinaryTreeVisitor {
    fn from_level_order(nodes: &[TreeLink]) -> TreeLink {
        let Some(root) = nodes.first().and_then(|node| node.as_ref()).cloned() else {
            return None;
        };

        let (mut parent_index, mut child_index) = (0, 1);

        while parent_index < child_index && child_index < nodes.len() {
            if let Some(parent) = nodes[parent_index].as_ref() {
                let mut parent = parent.borrow_mut();
                parent.left = nodes[child_index].clone();
                child_index += 1;

                if child_index < nodes.len() {
                    parent.right = nodes[child_index].clone();
                    child_index += 1;
                }
            }
            parent_index += 1;
        }

        Some(root)
    }
}

impl<'de> serde::de::Visitor<'de> for BinaryTreeVisitor {
    type Value = BinaryTree;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of optional integers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut nodes: Vec<TreeLink> = Vec::new();

        while let Some(val) = seq.next_element::<Option<i32>>()? {
            nodes.push(val.map(|v| Rc::new(RefCell::new(TreeNode::new(v)))));
        }

        Ok(BinaryTree(Self::from_level_order(&nodes)))
    }
}

impl<'de> Deserialize<'de> for BinaryTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(BinaryTreeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_serialize() {
        let root = TreeNode {
            val: 1,
            left: Some(Rc::new(RefCell::new(TreeNode {
                val: 2,
                left: None,
                right: None,
            }))),
            right: Some(Rc::new(RefCell::new(TreeNode {
                val: 4,
                left: Some(Rc::new(RefCell::new(TreeNode {
                    val: 3,
                    left: None,
                    right: None,
                }))),
                right: None,
            }))),
        };
        let tree = BinaryTree(Some(Rc::new(RefCell::new(root))));
        let serialized = serde_json::to_string(&tree).unwrap();
        assert_eq!(serialized, "[1,2,4,null,null,3]");
    }

    #[test]
    fn test_tree_serialize_empty_tree() {
        let serialized = serde_json::to_string(&BinaryTree(None)).unwrap();
        assert_eq!(serialized, "[]");
    }

    #[test]
    fn test_tree_deserialize_empty_array() {
        let tree: BinaryTree = serde_json::from_str("[]").unwrap();
        assert_eq!(tree, BinaryTree(None));
    }

    #[test]
    fn test_tree_deserialize_null_root() {
        let tree: BinaryTree = serde_json::from_str("[null]").unwrap();
        let serialized = serde_json::to_string(&tree).unwrap();
        assert_eq!(serialized, "[]");
    }

    #[test]
    fn test_tree_deserialize() {
        let tree: BinaryTree = serde_json::from_str("[1,2,4,null,null,3]").unwrap();
        let serialized = serde_json::to_string(&tree).unwrap();
        assert_eq!(serialized, "[1,2,4,null,null,3]");
    }

    #[test]
    fn test_tree_deserialize_left_skewed_tree() {
        let tree: BinaryTree = serde_json::from_str("[1,2,null,3]").unwrap();
        let serialized = serde_json::to_string(&tree).unwrap();
        assert_eq!(serialized, "[1,2,null,3]");
    }

    #[test]
    fn test_tree_deserialize_right_skewed_tree() {
        let tree: BinaryTree = serde_json::from_str("[1,null,2,null,3]").unwrap();
        let serialized = serde_json::to_string(&tree).unwrap();
        assert_eq!(serialized, "[1,null,2,null,3]");
    }

    #[test]
    fn test_tree_deserialize_negative_values() {
        let tree: BinaryTree = serde_json::from_str("[-1,-2,3]").unwrap();
        let serialized = serde_json::to_string(&tree).unwrap();
        assert_eq!(serialized, "[-1,-2,3]");
    }
}
