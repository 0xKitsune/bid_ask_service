use crate::order_book::{AskTree, BidTree, PriceLevel};

use super::{BinaryTree, Node};

impl AskTree for BinaryTree {
    fn insert_ask(&mut self, value: PriceLevel) {
        match self.root {
            None => self.root = Node::new(value).into(),
            Some(ref mut node) => BinaryTree::insert_ask_node(node, value),
        }
    }

    fn remove_ask(&mut self, value: PriceLevel) {
        todo!()
    }
}

impl BinaryTree {
    fn insert_ask_node(node: &mut Box<Node>, value: PriceLevel) {
        //If the value price already exists for a given exchange in the tree, update the corresponding node
        if value.price == node.value.price && value.exchange == node.value.exchange {
            node.value.quantity = value.quantity;
        } else if value >= node.value {
            match &mut node.right {
                None => node.right = Node::new(value).into(),
                Some(right_node) => BinaryTree::insert_ask_node(right_node, value),
            }
        } else {
            match &mut node.left {
                None => node.left = Node::new(value).into(),
                Some(left_node) => BinaryTree::insert_ask_node(left_node, value),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::order_book::AskTree;
    use ordered_float::OrderedFloat;
    use std::f32::consts::E;

    use crate::{
        exchanges::Exchange,
        order_book::{binary_tree::Node, PriceLevel},
    };

    use super::BinaryTree;

    #[test]
    fn test_insert_bid() {
        let mut tree = BinaryTree::new();

        tree.insert_ask(PriceLevel::new(100.00, 100.00, Exchange::Binance));
        tree.insert_ask(PriceLevel::new(101.00, 50.00, Exchange::Bitstamp));
        tree.insert_ask(PriceLevel::new(99.00, 100.00, Exchange::Binance));
        tree.insert_ask(PriceLevel::new(99.00, 100.00, Exchange::Bitstamp));
        tree.insert_ask(PriceLevel::new(100.00, 50.00, Exchange::Binance)); //Update the existing node's quantity at price 100
        tree.insert_ask(PriceLevel::new(300.00, 100.00, Exchange::Binance));

        let expected_tree = BinaryTree {
            root: Some(Box::new(Node {
                value: PriceLevel {
                    price: OrderedFloat(100.0),
                    quantity: OrderedFloat(100.0),
                    exchange: Exchange::Binance,
                },
                left: Some(Box::new(Node {
                    value: PriceLevel {
                        price: OrderedFloat(100.0),
                        quantity: OrderedFloat(50.0),
                        exchange: Exchange::Binance,
                    },
                    left: Some(Box::new(Node {
                        value: PriceLevel {
                            price: OrderedFloat(50.0),
                            quantity: OrderedFloat(100.0),
                            exchange: Exchange::Binance,
                        },
                        left: Some(Box::new(Node {
                            value: PriceLevel {
                                price: OrderedFloat(50.0),
                                quantity: OrderedFloat(50.0),
                                exchange: Exchange::Binance,
                            },
                            left: None,
                            right: None,
                        })),
                        right: None,
                    })),
                    right: None,
                })),
                right: Some(Box::new(Node {
                    value: PriceLevel {
                        price: OrderedFloat(300.0),
                        quantity: OrderedFloat(100.0),
                        exchange: Exchange::Binance,
                    },
                    left: None,
                    right: None,
                })),
            })),
        };

        dbg!(&tree);

        assert_eq!(tree, expected_tree);
    }
}
