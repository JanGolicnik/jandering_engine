use std::{
    cell::RefCell,
    sync::{Arc},
};

struct Item<T> {
    r: T,
    count: usize,
}

type ItemRef<T> = Arc<RefCell<Option<Item<T>>>>;

pub struct RcVec<T> {
    items: Vec<ItemRef<T>>,
}

impl<T> RcVec<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) -> RcVecHandle<T> {
        let r = Arc::new(RefCell::new(Some(Item { r: item, count: 1 })));
        self.items.push(r.clone());
        RcVecHandle { item: r }
    }

    pub fn get(&self, handle: &RcVecHandle<T>) -> &T {
        handle.item.
    }
}

#[derive(Clone)]
struct RcVecHandle<T> {
    item: ItemRef<T>,
}

impl<T> Drop for RcVecHandle<T> {
    fn drop(&mut self) {
        let mut item = self.item.lock().unwrap();

        match &mut *item {
            Some(inner_item) => {
                if inner_item.count > 1 {
                    inner_item.count -= 1;
                } else {
                    *item = None;
                }
            }
            None => panic!("Somehow a handle to a removed object was held"),
        }
    }
}
