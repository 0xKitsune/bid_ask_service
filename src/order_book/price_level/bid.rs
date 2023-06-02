use std::cmp::Ordering;

use ordered_float::OrderedFloat;

use crate::{exchanges::Exchange, order_book::Order};

#[derive(Debug, Clone)]
pub struct Bid {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

impl Bid {
    pub fn new(price: f64, quantity: f64, exchange: Exchange) -> Self {
        Bid {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            exchange,
        }
    }
}

impl Default for Bid {
    fn default() -> Self {
        Bid::new(0.0, 0.0, Exchange::Binance)
    }
}

impl Order for Bid {
    fn get_price(&self) -> &OrderedFloat<f64> {
        &self.price
    }
    fn get_quantity(&self) -> &OrderedFloat<f64> {
        &self.quantity
    }
    fn set_quantity(&mut self, quantity: OrderedFloat<f64>) {
        self.quantity = quantity;
    }
    fn get_exchange(&self) -> &Exchange {
        &self.exchange
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
            && self.quantity == other.quantity
            && self.exchange == other.exchange
    }
}

impl Eq for Bid {}
impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        //First check if the price is equal
        match self.price.cmp(&other.price) {
            //If the price is equal, check the exchange, this allows the order book structure to know to replace the quantity for this value
            Ordering::Equal => match self.exchange.cmp(&other.exchange) {
                Ordering::Equal => Ordering::Equal,
                //If the price is the same but the exchange is different, compare the quantity
                _ => match self.quantity.cmp(&other.quantity) {
                    //When using Rust's std btree or other std lib data types that use the Ord implementation to sort a key, when checking if a structure
                    //contains a value, it compares the key to each value and checks if it is greater than, less than or equal to the current value.
                    //In the case that the price is the same, and the quantity is the same but the exchange is different, we need to return Ordering::Less, otherwise the contains function will exit early

                    //For example in the search_node function of a BTreeSet:
                    // match key.cmp(k.borrow()) {
                    //     Ordering::Greater => {}
                    //     Ordering::Equal => return IndexResult::KV(start_index + offset),
                    //     Ordering::Less => return IndexResult::Edge(start_index + offset),
                    // }

                    //Lets say a new bid is being added with a price of 1.0, quantity of 1.0 and the exchange is Binance.
                    //Lets also say that there is currently a price level with a price of 1.0, quantity of 5.0 and the exchange is Binance.
                    //If we compare it to an order that has a price of 1.0, quantity of 1.0 and the exchange is Bitstamp, the new order will be considered greater
                    // and the function will exit saying that the order is not in the data structure, however it would be the next value in this case.
                    //For this reason, if the price and quantity are exactly the same, we return less so that the function continues to search the structure
                    Ordering::Equal => Ordering::Less,
                    other => other,
                },
            },
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{exchanges::Exchange, order_book::Bid};

    #[test]
    pub fn test_bid_greater() {
        //the price is greater but the quantity is the same
        let bid_0 = Bid::new(1.23, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_0.cmp(&bid_1).is_gt());

        //the price is greater but the quantity is the same and the exchanges are different
        let bid_4 = Bid::new(1.24, 1200.56, Exchange::Bitstamp);
        let bid_5 = Bid::new(1.20, 1200.56, Exchange::Binance);
        assert!(bid_4.cmp(&bid_5).is_gt());

        //the price is the same but the quantity is greater and the exchanges are different
        let bid_6 = Bid::new(1.20, 1300.56, Exchange::Binance);
        let bid_7 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(bid_6.cmp(&bid_7).is_gt());

        //the price and quantity are different
        let bid_8 = Bid::new(1.23, 1000.56, Exchange::Binance);
        let bid_9 = Bid::new(1.20, 1200.56, Exchange::Bitstamp);
        assert!(bid_8.cmp(&bid_9).is_gt());

        //the price and quantity are the same but the exchange is different
        let bid_10 = Bid::new(1.20, 1000.56, Exchange::Binance);
        let bid_11 = Bid::new(1.20, 1000.56, Exchange::Bitstamp);

        assert!(bid_10.cmp(&bid_11).is_gt());
    }

    #[test]
    pub fn test_bid_less() {
        //the price is less but the quantity is the same
        let bid_0 = Bid::new(1.23, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_1.cmp(&bid_0).is_lt());

        //the price is less but the quantity is the same and the exchanges are different
        let bid_4 = Bid::new(1.25, 1200.56, Exchange::Bitstamp);
        let bid_5 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_5.cmp(&bid_4).is_lt());

        //the price is the same but the quantity is less and the exchanges are different
        let bid_6 = Bid::new(1.20, 1300.56, Exchange::Bitstamp);
        let bid_7 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_7.cmp(&bid_6).is_lt());

        //the price and quantity are different
        let bid_8 = Bid::new(1.23, 1000.56, Exchange::Bitstamp);
        let bid_9 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_9.cmp(&bid_8).is_lt());

        //the price and quantity are the same but the exchange is different
        let bid_10 = Bid::new(1.20, 1000.56, Exchange::Binance);
        let bid_11 = Bid::new(1.20, 1000.56, Exchange::Bitstamp);

        assert!(bid_11.cmp(&bid_10).is_lt());
    }
    #[test]
    pub fn test_bid_equal() {
        //the price, quantity and the exchanges are the same
        let bid_0 = Bid::new(1.20, 1200.56, Exchange::Binance);
        let bid_1 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_0 == bid_1);

        //the price and exchange are the same but the quantity is different
        let bid_2 = Bid::new(1.20, 12309.56, Exchange::Binance);
        let bid_3 = Bid::new(1.20, 1200.56, Exchange::Binance);

        assert!(bid_2.cmp(&bid_3).is_eq());
        assert!(bid_2 != bid_3);
    }
}
