use prettytable::{Cell, Row, Table};

use std::cell::RefCell;
use std::rc::Rc;

use std::collections::VecDeque;

use std::ops::{AddAssign, SubAssign};

use std::thread::sleep;
use std::time::Duration;

pub trait Behaviour {
    fn update(&mut self, bucket: Bucket, delta: u64);
}

#[derive(Default)]
pub struct BucketState {
    name: String,
    quantity: u64,
    behaviours: Vec<Rc<RefCell<Box<dyn Behaviour>>>>,
}

#[derive(Clone, Default)]
pub struct Bucket {
    state: Rc<RefCell<BucketState>>,
}

impl Bucket {
    fn new(name: &'_ str) -> Bucket {
        Bucket::default().with_name(name)
    }
    fn update(&mut self, ticks: u64) {
        let bs = { self.state.borrow_mut().behaviours.clone() };
        bs.iter()
            .for_each(|bs| bs.borrow_mut().update(self.clone(), ticks));
    }
    fn set_name(&mut self, name: &'_ str) {
        self.state.borrow_mut().name = name.to_owned();
    }
    fn with_name(self, name: &'_ str) -> Self {
        self.state.borrow_mut().name = name.to_owned();
        self
    }
    fn get(&self) -> u64 {
        self.state.borrow().quantity
    }
    fn name(&self) -> String {
        self.state.borrow().name.clone()
    }
    fn add(&mut self, behaviour: Box<dyn Behaviour>) {
        self.state
            .borrow_mut()
            .behaviours
            .push(Rc::new(RefCell::new(behaviour)));
    }
}

impl<T> AddAssign<T> for Bucket
where
    T: Into<i64>,
{
    fn add_assign(&mut self, rhs: T) {
        self.state.borrow_mut().quantity += rhs.into() as u64;
    }
}

impl<T> SubAssign<T> for Bucket
where
    T: Into<i64>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.state.borrow_mut().quantity -= rhs.into() as u64;
    }
}

pub struct Diffusion {
    target: Bucket,
    probability: f32,
}

impl Behaviour for Diffusion {
    fn update(&mut self, bucket: Bucket, delta: u64) {
        let c = bucket.get();
        let to_move = ((self.probability * c as f32).round() as u64 * delta) as i32;
        if c as i32 - to_move > 0 {
            self.target += to_move;
            let mut bucket = bucket;
            bucket -= to_move;
        }
    }
}

impl Diffusion {
    fn new(target: Bucket, probability: f32) -> Box<dyn Behaviour> {
        Box::new(Diffusion {
            target,
            probability,
        })
    }
}

pub struct Infection {
    target: Bucket,
    probability: f32,
}

impl Behaviour for Infection {
    fn update(&mut self, bucket: Bucket, delta: u64) {
        let to_move = ((self.probability * self.target.get() as f32).round() as u64 * delta) as i32;
        if self.target.get() as i32 - to_move > 0 {
            self.target += to_move;
            let mut bucket = bucket;
            bucket -= to_move;
        }
    }
}

impl Infection {
    fn new(target: Bucket, probability: f32) -> Box<dyn Behaviour> {
        Box::new(Diffusion {
            target,
            probability,
        })
    }
}

#[derive(Default)]
pub struct Model {
    buckets: Vec<Bucket>,
}

impl Model {
    fn new() -> Model {
        Model::default()
    }
    fn run(&mut self, speed: u64) {
        let names = self
            .buckets
            .iter()
            .map(|bucket| Cell::new(&bucket.name()))
            .collect::<Vec<Cell>>();

        let mut simulated: VecDeque<Vec<Cell>> = VecDeque::new();

        loop {
            let mut table = Table::new();
            table.add_row(Row::new(names.clone()));
            simulated.push_front(
                self.buckets
                    .iter()
                    .map(|bucket| Cell::new(&format!("{}", bucket.get())))
                    .collect(),
            );
            simulated.truncate(10);
            simulated.iter().for_each(|row| {
                table.add_row(Row::new(row.clone()));
            });
            table.printstd();
            print!("{}[2J", 27 as char);
            self.buckets
                .iter_mut()
                .for_each(|bucket| bucket.update(speed));
            sleep(Duration::from_millis(100));
        }
    }
    fn add(&mut self, bucket: Bucket) {
        self.buckets.push(bucket);
    }
}

fn main() {
    let mut model = Model::new();
    let mut s = Bucket::new("Susceptible");
    let mut i = Bucket::new("Infected");
    let mut r = Bucket::new("Recovered");
    let infection = Infection::new(i.clone(), 0.01);
    let recovery = Diffusion::new(r.clone(), 0.2);
    s.add(infection);
    i.add(recovery);
    s += 1000;
    i += 1;
    model.add(s);
    model.add(i);
    model.add(r);
    model.run(1);
}
