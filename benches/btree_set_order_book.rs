use std::collections::BTreeSet;

use bid_ask_service::{
    exchanges::Exchange,
    order_book::{
        price_level::{ask::Ask, bid::Bid},
        BuySide, Order, SellSide,
    },
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use ordered_float::OrderedFloat;
use rand::Rng;

fn initialize_bids() -> BTreeSet<Bid> {
    let mut order_book = BTreeSet::<Bid>::new();
    let mut rng = rand::thread_rng();

    for _ in 0..50 {
        let price: f64 = rng.gen_range(80.0..600.0);
        let quantity: f64 = rng.gen_range(40.0..10000000000.0);
        let bid = Bid::new(price, quantity, Exchange::Binance);
        order_book.update_bids(bid, 50);
    }

    order_book
}

fn initialize_asks() -> BTreeSet<Ask> {
    let mut order_book = BTreeSet::<Ask>::new();
    let mut rng = rand::thread_rng();

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
    let mut order_book = initialize_bids();

    c.bench_function("insert bid", |b| {
        b.iter_batched_ref(
            create_bid,
            |bid| order_book.update_bids(black_box(bid.clone()), 50),
            BatchSize::SmallInput,
        )
    });
}

fn get_random_bid(order_book: &BTreeSet<Bid>) -> Bid {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..order_book.len());
    order_book
        .iter()
        .nth(index)
        .expect("could not get random bid")
        .clone()
}

fn bench_remove_bid(c: &mut Criterion) {
    c.bench_function("remove bid", |b| {
        b.iter_batched_ref(
            || {
                let order_book = initialize_bids();
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
    let order_book = initialize_bids();

    c.bench_function("update bid", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                let mut bid = get_random_bid(&order_book.clone());
                let new_quantity: f64 = rng.gen_range(40.0..60.0);
                bid.set_quantity(OrderedFloat(new_quantity));
                (order_book.clone(), bid)
            },
            |(mut order_book, bid)| order_book.update_bids(black_box(bid), 50),
            BatchSize::SmallInput,
        )
    });
}

fn bench_get_best_bid(c: &mut Criterion) {
    let order_book = initialize_bids();

    c.bench_function("get best bid", |b| {
        b.iter_batched(
            || order_book.clone(),
            |order_book| {
                order_book
                    .get_best_bid()
                    .expect("Could not get best bid")
                    .clone()
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_get_best_n_bids(c: &mut Criterion) {
    let order_book = initialize_bids();

    c.bench_function("get best 'n' bids", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                let n = rng.gen_range(0..order_book.len());
                (n, order_book.clone())
            },
            |(n, order_book)| order_book.get_best_n_bids(n),
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
    let mut order_book = initialize_asks();

    c.bench_function("insert ask", |b| {
        b.iter_batched_ref(
            create_ask,
            |bid| order_book.update_asks(black_box(bid.clone()), 50),
            BatchSize::SmallInput,
        )
    });
}

fn get_random_ask(order_book: &BTreeSet<Ask>) -> Ask {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..order_book.len());
    order_book
        .iter()
        .nth(index)
        .expect("could not get random ask")
        .clone()
}

fn bench_remove_ask(c: &mut Criterion) {
    c.bench_function("remove ask", |b| {
        b.iter_batched_ref(
            || {
                let order_book = initialize_asks();
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
    let order_book = initialize_asks();

    c.bench_function("update ask", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                let mut ask = get_random_ask(&order_book.clone());
                let new_quantity: f64 = rng.gen_range(40.0..60.0);
                ask.set_quantity(OrderedFloat(new_quantity));
                (order_book.clone(), ask)
            },
            |(mut order_book, ask)| order_book.update_asks(black_box(ask), 50),
            BatchSize::SmallInput,
        )
    });
}

fn bench_get_best_ask(c: &mut Criterion) {
    let order_book = initialize_asks();

    c.bench_function("get best ask", |b| {
        b.iter_batched(
            || order_book.clone(),
            |order_book| {
                order_book
                    .get_best_ask()
                    .expect("Could not get best ask")
                    .clone()
            },
            BatchSize::SmallInput,
        )
    });
}

fn bench_get_best_n_asks(c: &mut Criterion) {
    let order_book = initialize_asks();

    c.bench_function("get best 'n' asks", |b| {
        b.iter_batched(
            || {
                let mut rng = rand::thread_rng();
                let n = rng.gen_range(0..order_book.len());
                (n, order_book.clone())
            },
            |(n, order_book)| order_book.get_best_n_asks(n),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_insert_bid,
    bench_remove_bid,
    bench_update_bid,
    bench_get_best_bid,
    bench_get_best_n_bids,
    bench_insert_ask,
    bench_remove_ask,
    bench_update_ask,
    bench_get_best_ask,
    bench_get_best_n_asks
);
criterion_main!(benches);
