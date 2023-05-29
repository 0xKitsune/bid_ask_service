use std::cmp::Ord;
use std::cmp::Ordering;
pub mod ask_tree;
pub mod bid_tree;
use super::BidTree;
use super::PriceLevel;

#[derive(Debug, PartialEq)]
pub struct BinaryTree {
    root: Option<Box<Node>>,
}

#[derive(Debug, PartialEq)]
pub struct Node {
    value: PriceLevel,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    pub fn new(value: PriceLevel) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

impl From<Node> for Option<Box<Node>> {
    fn from(node: Node) -> Self {
        Some(Box::new(node))
    }
}

impl BinaryTree {
    pub fn new() -> Self {
        BinaryTree { root: None }
    }
}
