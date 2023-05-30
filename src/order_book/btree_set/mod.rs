use std::collections::{BTreeMap, BTreeSet};

use ordered_float::OrderedFloat;

use crate::exchanges::Exchange;

use super::{error::OrderBookError, Order, OrderSide};

impl<T> OrderSide<T> for BTreeSet<T>
where
    T: Order,
{
    fn insert(&mut self, order: T) -> Result<(), OrderBookError> {
        self.insert(order);

        Ok(())
    }

    fn remove(&mut self, order: T) -> Result<(), OrderBookError> {
        self.remove(&order);

        Ok(())
    }
}
