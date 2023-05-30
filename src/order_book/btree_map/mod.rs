// use std::collections::{BTreeMap, BTreeSet};

// use ordered_float::OrderedFloat;

// use crate::exchanges::Exchange;

// use super::{error::OrderBookError, Order, OrderSide};

// pub struct OrderQuantity {
//     quantity: OrderedFloat<f64>,
//     exchange: Exchange,
// }

// impl<T> OrderSide<T> for BTreeMap<OrderedFloat<f64>, BTreeMap<Exchange, >>
// where
//     T: Order,
// {
//     fn insert(&mut self, order: T) -> Result<(), OrderBookError> {
//         let price = order.get_price();

//         if let Some(quantities) = self.get(price) {
//             if let Some(quantity) = quantities.get(&order) {}
//         }

//         self.insert(*order.get_price(), order);

//         self.get(key)
//         Ok(())
//     }

//     fn remove(&mut self, order: T) -> Result<(), OrderBookError> {
//         self.remove(*order.get_price());

//         Ok(())
//     }
// }
