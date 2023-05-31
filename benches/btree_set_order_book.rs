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

    for _ in 0..50 {
        let price: f64 = rng.gen_range(80.0..600.0);
        let quantity: f64 = rng.gen_range(40.0..10000000000.0);
        let bid = Bid::new(price, quantity, Exchange::Binance);
        order_book.update_bids(bid, 50);
    }

    for _ in 0..50 {
        let price: f64 = rng.gen_range(80.0..600.0);
        let quantity: f64 = rng.gen_range(40.0..10000000000.0);
        let ask = Ask::new(price, quantity, Exchange::Binance);
        order_book.update_asks(ask, 50);
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
            |bid| order_book.update_bids(black_box(bid.clone()), 50),
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
                order_book.update_bids(black_box(bid.clone()), 50)
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
            |(mut order_book, bid)| order_book.update_bids(black_box(bid.clone()), 50),
            BatchSize::SmallInput,
        )
    });
}

fn create_ask() -> Ask {
    let mut rng = rand::thread_rng();
    let price: f64 = rng.gen_range(80.0..120.0);
    let quantity: f64 = rng.gen_range(40.0..60.0);
    Ask::new(price, quantity, Exchange::Binance)
}

fn bench_insert_ask(c: &mut Criterion) {
    let mut order_book = initialize_order_book();

    c.bench_function("insert ask", |b| {
        b.iter_batched_ref(
            create_ask,
            |bid| order_book.update_asks(black_box(bid.clone()), 50),
            BatchSize::SmallInput,
        )
    });
}

fn get_random_ask(order_book: &BTreeSetOrderBook) -> Ask {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..order_book.bids.len());
    order_book
        .asks
        .iter()
        .nth(index)
        .expect("could not get random ask")
        .clone()
}

fn bench_remove_ask(c: &mut Criterion) {
    c.bench_function("remove ask", |b| {
        b.iter_batched_ref(
            || {
                let order_book = initialize_order_book();
                (order_book.clone(), get_random_ask(&order_book))
            },
            |(ref mut order_book, ask)| {
                ask.set_quantity(OrderedFloat(0.0));
                order_book.update_asks(black_box(ask.clone()), 50)
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_update_ask(c: &mut Criterion) {
    let order_book = initialize_order_book();

    c.bench_function("update ask", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                let mut ask = get_random_ask(&order_book.clone());
                let new_quantity: f64 = rng.gen_range(40.0..60.0);
                ask.set_quantity(OrderedFloat(new_quantity));
                (order_book.clone(), ask)
            },
            |(mut order_book, ask)| order_book.update_asks(black_box(ask.clone()), 50),
            BatchSize::SmallInput,
        )
    });
}

//TODO: add bench for get best bid/get best ask

//TODO: also benches for get best n bids, get best n asks

criterion_group!(
    benches,
    bench_insert_bid,
    bench_remove_bid,
    bench_update_bid,
    bench_insert_ask,
    bench_remove_ask,
    bench_update_ask
);
criterion_main!(benches);
