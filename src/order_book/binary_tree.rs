use crate::order_book::PriceLevel;

use std::cmp::Ord;
use std::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub struct BinaryTree<T: Ord> {
    root: Option<Box<Node<T>>>,
}

#[derive(Debug, PartialEq)]
pub struct Node<T: Ord> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T>
where
    T: Ord,
{
    pub fn new(value: T) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

impl<T> From<Node<T>> for Option<Box<Node<T>>>
where
    T: Ord,
{
    fn from(node: Node<T>) -> Self {
        Some(Box::new(node))
    }
}

impl<T> BinaryTree<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        BinaryTree { root: None }
    }

    fn insert(&mut self, value: T) {
        match self.root {
            None => self.root = Node::new(value).into(),
            Some(ref mut node) => BinaryTree::insert_node(node, value),
        }
    }

    fn insert_node(node: &mut Box<Node<T>>, value: T) {
        // //If the value price already exists for a given exchange in the tree, update the corresponding node
        // if value.price == node.value.price && value.exchange == node.value.exchange {
        //     node.value.quantity = value.quantity;
        // } else if value >= node.value {
        //     match &mut node.right {
        //         None => node.right = Node::new(value).into(),
        //         Some(right_node) => BinaryTree::insert_node(right_node, value),
        //     }
        // } else {
        //     match &mut node.left {
        //         None => node.left = Node::new(value).into(),
        //         Some(left_node) => BinaryTree::insert_node(left_node, value),
        //     }
        // }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;
    use std::f32::consts::E;

    use crate::{
        exchanges::Exchange,
        order_book::{binary_tree::Node, price_level::OrderType, PriceLevel},
    };

    use super::BinaryTree;

    #[test]
    fn test_insert_bid() {
        // let mut tree = BinaryTree::new();

        // tree.insert(PriceLevel::new(
        //     100.00,
        //     100.00,
        //     Exchange::Binance,
        //     OrderType::Bid,
        // ));
        // tree.insert(PriceLevel::new(
        //     101.00,
        //     50.00,
        //     Exchange::Bitstamp,
        //     OrderType::Bid,
        // ));
        // tree.insert(PriceLevel::new(
        //     99.00,
        //     100.00,
        //     Exchange::Binance,
        //     OrderType::Bid,
        // ));
        // tree.insert(PriceLevel::new(
        //     99.00,
        //     100.00,
        //     Exchange::Bitstamp,
        //     OrderType::Bid,
        // ));
        // tree.insert(PriceLevel::new(
        //     100.00,
        //     50.00,
        //     Exchange::Binance,
        //     OrderType::Bid,
        // )); //Update the existing node's quantity at price 100
        // tree.insert(PriceLevel::new(
        //     300.00,
        //     100.00,
        //     Exchange::Binance,
        //     OrderType::Bid,
        // ));

        // let expected_tree = BinaryTree {
        //     root: Some(Box::new(Node {
        //         value: PriceLevel {
        //             price: OrderedFloat(100.0),
        //             quantity: OrderedFloat(100.0),
        //             exchange: Exchange::Binance,
        //             order_type: OrderType::Bid,
        //         },
        //         left: Some(Box::new(Node {
        //             value: PriceLevel {
        //                 price: OrderedFloat(100.0),
        //                 quantity: OrderedFloat(50.0),
        //                 exchange: Exchange::Binance,
        //                 order_type: OrderType::Bid,
        //             },
        //             left: Some(Box::new(Node {
        //                 value: PriceLevel {
        //                     price: OrderedFloat(50.0),
        //                     quantity: OrderedFloat(100.0),
        //                     exchange: Exchange::Binance,
        //                     order_type: OrderType::Bid,
        //                 },
        //                 left: Some(Box::new(Node {
        //                     value: PriceLevel {
        //                         price: OrderedFloat(50.0),
        //                         quantity: OrderedFloat(50.0),
        //                         exchange: Exchange::Binance,
        //                         order_type: OrderType::Bid,
        //                     },
        //                     left: None,
        //                     right: None,
        //                 })),
        //                 right: None,
        //             })),
        //             right: None,
        //         })),
        //         right: Some(Box::new(Node {
        //             value: PriceLevel {
        //                 price: OrderedFloat(300.0),
        //                 quantity: OrderedFloat(100.0),
        //                 exchange: Exchange::Binance,
        //                 order_type: OrderType::Bid,
        //             },
        //             left: None,
        //             right: None,
        //         })),
        //     })),
        // };

        // dbg!(&tree);

        // assert_eq!(tree, expected_tree);
    }
}
