use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use kbas::{
    exchanges::Exchange,
    order_book::{
        btree_set::BTreeSetOrderBook,
        price_level::{ask::Ask, bid::Bid},
        Order, OrderBook,
    },
};
use ordered_float::OrderedFloat;
use rand::Rng;

fn initialize_order_book() -> BTreeSetOrderBook {
    let mut order_book = BTreeSetOrderBook::new();
    let mut rng = rand::thread_rng();

    for _ in 0..20 {
        let price: f64 = rng.gen_range(80.0..600.0);
        let quantity: f64 = rng.gen_range(40.0..10000000000.0);
        let bid = Bid::new(price, quantity, Exchange::Binance);
        order_book.update_bids(bid);
    }

    for _ in 0..20 {
        let price: f64 = rng.gen_range(80.0..600.0);
        let quantity: f64 = rng.gen_range(40.0..10000000000.0);
        let ask = Ask::new(price, quantity, Exchange::Binance);
        order_book.update_asks(ask);
    }

    order_book
}

fn create_bid() -> Bid {
    let mut rng = rand::thread_rng();
    let price: f64 = rng.gen_range(80.0..120.0);
    let quantity: f64 = rng.gen_range(40.0..60.0);
    Bid::new(price, quantity, Exchange::Binance)
}

fn bench_insert_bid(c: &mut Criterion) {
    let mut order_book = initialize_order_book();

    c.bench_function("insert bid", |b| {
        b.iter_batched_ref(
            create_bid,
            |bid| order_book.update_bids(black_box(bid.clone())),
            BatchSize::SmallInput,
        )
    });
}

fn get_random_bid(order_book: &BTreeSetOrderBook) -> Bid {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..order_book.bids.len());
    order_book
        .bids
        .iter()
        .nth(index)
        .expect("could not get random bid")
        .clone()
}

fn bench_remove_bid(c: &mut Criterion) {
    c.bench_function("remove bid", |b| {
        b.iter_batched_ref(
            || {
                let order_book = initialize_order_book();
                (order_book.clone(), get_random_bid(&order_book))
            },
            |(ref mut order_book, bid)| {
                bid.set_quantity(OrderedFloat(0.0));
                order_book.update_bids(black_box(bid.clone()))
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_update_bid(c: &mut Criterion) {
    let order_book = initialize_order_book();

    c.bench_function("update bid", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                let mut bid = get_random_bid(&order_book.clone());
                let new_quantity: f64 = rng.gen_range(40.0..60.0);
                bid.set_quantity(OrderedFloat(new_quantity));
                (order_book.clone(), bid)
            },
            |(mut order_book, bid)| order_book.update_bids(black_box(bid.clone())),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_insert_bid,
    bench_remove_bid,
    bench_update_bid
);
criterion_main!(benches);
