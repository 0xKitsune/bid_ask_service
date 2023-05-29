use std::cmp::Ord;
use std::cmp::Ordering;

// use super::PriceLevel;

// // pub struct BinaryTree<T> {
// //     root: Option<Box<Node>>,
// // }
// pub struct Node {
//     value: PriceLevel,
//     left: Option<Box<Node>>,
//     right: Option<Box<Node>>,
// }

// impl<T: Ord> BinaryTree<T> {
//     pub fn new() -> Self {
//         BinaryTree { root: None }
//     }

//     // pub fn insert(&mut self, value: T) {
//     //     let new_node = Box::new(Node {
//     //         value,
//     //         left: None,
//     //         right: None,
//     //     });

//     //     match self.root {
//     //         None => {
//     //             self.root = Some(new_node);
//     //         }
//     //         Some(ref mut node) => {
//     //             self.insert_node(node, new_node);
//     //         }
//     //     }
//     // }

//     // fn insert_node(node: &mut Box<Node>, value: PriceLevel) {
//     //     match node.value.cmp(&new_node.value) {
//     //         Ordering::Less => {
//     //             if let Some(ref mut right) = node.right {
//     //                 self.insert_node(right, new_node);
//     //             } else {
//     //                 node.right = Some(new_node);
//     //             }
//     //         }
//     //         _ => {
//     //             if let Some(ref mut left) = node.left {
//     //                 self.insert_node(left, new_node);
//     //             } else {
//     //                 node.left = Some(new_node);
//     //             }
//     //         }
//     //     }
//     // }
// }
