use std::cmp::Ordering;

use ordered_float::OrderedFloat;

use crate::{exchanges::Exchange, order_book::Order};

#[derive(Debug, Clone)]
pub struct Ask {
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub exchange: Exchange,
}

impl Ask {
    pub fn new(price: f64, quantity: f64, exchange: Exchange) -> Self {
        Ask {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            exchange,
        }
    }
}

impl Default for Ask {
    fn default() -> Self {
        Ask::new(f64::MAX, 0.0, Exchange::Binance)
    }
}
impl Order for Ask {
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

impl PartialEq for Ask {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
            && self.quantity == other.quantity
            && self.exchange == other.exchange
    }
}

impl Eq for Ask {}
impl PartialOrd for Ask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

//When ordering asks, we want the lowest price with the highest quantity to be the best
//so a price level with the same price but higher quantity should be considered less than a price level
//with the same price but lower quantity in order to ensure that the best price is considered the ask that is lesser than the other
impl Ord for Ask {
    fn cmp(&self, other: &Self) -> Ordering {
        //First check if the price is equal
        match self.price.cmp(&other.price) {
            //If the price is equal, check the exchange, this allows the order book structure to know to replace the quantity for this value
            Ordering::Equal => match self.exchange.cmp(&other.exchange).reverse() {
                Ordering::Equal => Ordering::Equal,

                //TODO: because the exchange is greater, it is returning {} when trying to find the key in the btree

                //If the price is the same but the exchange is different, compare the quantity
                _ => match self.quantity.cmp(&other.quantity).reverse() {
                    //TODO: add a note as to why we give strictly less ordering. ultimately, this is because when trying to check if a key is contained within an btree or btreemap/set, it uses the ord
                    //TODO: trait. This make it so that if the exchange has a higher order and it stops searching.
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

    use crate::{exchanges::Exchange, order_book::Ask};

    #[test]
    pub fn test_ask_less() {
        //the price is less but the quantity is the same
        let ask_0 = Ask::new(1.23, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_1.cmp(&ask_0).is_lt());

        //the price is the same but the quantity is less
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1300.56, Exchange::Bitstamp);

        assert!(ask_3.cmp(&ask_2).is_lt());

        //the price is less but the quantity is the same and the exchanges are different
        let ask_4 = Ask::new(1.25, 1200.56, Exchange::Binance);
        let ask_5 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_5.cmp(&ask_4).is_lt());

        //the price is the same but the quantity is less and the exchanges are different
        let ask_6 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        let ask_7 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert!(ask_7.cmp(&ask_6).is_lt());

        //the price and quantity are different
        let ask_8 = Ask::new(1.23, 1500.56, Exchange::Binance);
        let ask_9 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_9.cmp(&ask_8).is_lt());

        //the price and quantity are the same but the exchange is different
        let ask_10 = Ask::new(1.20, 1000.56, Exchange::Bitstamp);
        let ask_11 = Ask::new(1.20, 1000.56, Exchange::Binance);

        assert!(ask_11.cmp(&ask_10).is_lt());

        //the price and quantity are the same but the exchange is different
        //the price and quantity are the same but the exchange is different
        //Note that when the price and the quantity are the same but the exchange is different, the comparison is always less than.
        //For a more detailed explanation, visit the Ord implementation for Bid
        let ask_12 = Ask::new(1.20, 1000.56, Exchange::Binance);
        let ask_13 = Ask::new(1.20, 1000.56, Exchange::Bitstamp);

        assert!(ask_12.cmp(&ask_13).is_lt());
    }

    #[test]
    pub fn test_ask_greater() {
        //the price is greater but the quantity is the same
        let ask_0 = Ask::new(1.23, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_0.cmp(&ask_1).is_gt());

        //the price is the same but the quantity is greater
        let ask_2 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_3 = Ask::new(1.20, 1300.56, Exchange::Bitstamp);

        assert!(ask_2.cmp(&ask_3).is_gt());

        //the price is greater but the quantity is the same and the exchanges are different
        let ask_4 = Ask::new(1.25, 1200.56, Exchange::Binance);
        let ask_5: Ask = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_4.cmp(&ask_5).is_gt());

        //the price is the same but the quantity is greater and the exchanges are different
        let ask_6 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);
        let ask_7 = Ask::new(1.20, 1300.56, Exchange::Binance);

        assert!(ask_6.cmp(&ask_7).is_gt());
    }

    #[test]
    pub fn test_ask_equal() {
        //the price, quantity and the exchanges are the same
        let ask_0 = Ask::new(1.20, 1200.56, Exchange::Binance);
        let ask_1 = Ask::new(1.20, 1200.56, Exchange::Binance);

        assert!(ask_0 == ask_1);

        //the price and exchange are the same but the quantity is different
        let ask_2 = Ask::new(1.20, 234235.56, Exchange::Bitstamp);
        let ask_3 = Ask::new(1.20, 1200.56, Exchange::Bitstamp);

        assert!(ask_2.cmp(&ask_3).is_eq());
        assert!(ask_2 != ask_3);
    }
}
