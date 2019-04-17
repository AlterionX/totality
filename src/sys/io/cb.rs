use super::e::*;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
    ops::{FnMut, DerefMut},
    result::Result,
};

pub trait CBFn: FnMut(&State, &V) + Send + 'static {}
impl <T> CBFn for T where T: FnMut(&State, &V) + Send + 'static {}
pub struct CB {
    c: C,
    cb: Weak<Mutex<CBFn>>,
}
impl CB {
    fn call(&self, s: &State, v: &V) -> Result<(), ()> {
        match self.cb.upgrade() {
            Some(cb_m) => match cb_m.lock() {
                Ok(mut cb_mg) => Result::Ok((cb_mg.deref_mut())(s, v)),
                Err(_) => Result::Ok(()),
            },
            None => Result::Err(()),
        }
    }
    pub fn new(c: C, cb: Weak<Mutex<CBFn>>) -> CB { CB { c: c, cb: cb } }
}
pub enum RegRequest {
    Register(C, Vec<CB>),
    Unregister(Vec<Weak<Mutex<CB>>>),
}
pub enum RegResponse {
    Register(C, Vec<Weak<Mutex<CB>>>),
    Unregister,
}

pub struct Manager {
    // TODO  see if there's a better way.
    occupied: Vec<C>,
    // TODO potentially change Vec into a linked list for O(1) removal
    buckets: HashMap<C, Vec<Arc<Mutex<CB>>>>,
}
impl Manager {
    pub fn new() -> Manager {
        Manager {
            occupied: vec![],
            buckets: HashMap::new(),
        }
    }
    fn fire_event(s: &State, v: &V, cb_m: &Arc<Mutex<CB>>) -> Result<(), ()> {
        match cb_m.lock() {
            Ok(cb_mg) => (*cb_mg).call(s, v),
            Err(_) => Result::Ok(()),
        }
    }
    fn fire_category_events(&self, s: &State, c: &C, v: &V) -> (C, Vec<Arc<Mutex<CB>>>) {
        let mut to_remove = vec![];
        if let Some(bucket) = self.buckets.get(&c) {
            for cb in bucket.iter() {
                match Manager::fire_event(s, v, cb) {
                    Ok(_) => (),
                    Err(_) => to_remove.push(cb.clone()),
                }
            }
        }
        (c.clone(), to_remove)
    }
    fn fire_all_events(&mut self, s: &State) -> Vec<(C, Vec<Arc<Mutex<CB>>>)> {
        let mut removal_stuff = Vec::with_capacity(self.occupied.len());
        for category in self.occupied.iter() {
            removal_stuff.push(self.fire_category_events(s, category, &s.get(&category)));
        };
        removal_stuff
    }
    fn remove_category(cc: &mut Vec<C>, c: &C) {
        let idx = cc.iter().position(|x| x == c)
            .expect("Bug in data structure maintenance. ");
        cc.swap_remove(idx);
    }
    fn remove_matching_arcs<T>(v: &mut Vec<Arc<T>>, removing: Vec<Arc<T>>) {
        v.retain(|x| !removing.iter().any(|remove| Arc::ptr_eq(remove, x)))
    }
    fn remove_matching(&mut self, removals: Vec<(C, Vec<Arc<Mutex<CB>>>)>) {
        for (c, removal) in removals {
            let bucket = self.buckets.get_mut(&c)
                .expect("Bug in data structure maintenance. Requested removal of nonexistent callbacks.");
            Self::remove_matching_arcs(bucket, removal);
            if bucket.len() == 0 { Self::remove_category(&mut self.occupied, &c); }
        }
    }
    pub fn register(&mut self, cb: CB) -> Weak<Mutex<CB>> {
        if !self.buckets.contains_key(&cb.c) { self.occupied.push(cb.c.clone()) }
        let v = self.buckets.entry(cb.c.clone()).or_insert(Vec::with_capacity(1));
        v.push(Arc::new(Mutex::new(cb)));
        Arc::downgrade(&v.last().unwrap())
    }
    fn unregister(&mut self, cb_m: Arc<Mutex<CB>>) {
        let c = match cb_m.lock() {
            Ok(cb_mg) => cb_mg.c,
            Err(poisoned) => poisoned.into_inner().c,
        }.clone();
        self.remove_matching(vec![(c, vec![cb_m.clone()])]);
    }
    pub fn fire_and_clean_listing(&mut self, s: &State, vv: &mut Vec<V>) {
        let mut deallocs = Vec::new();
        let mut dealloc_idx = HashMap::new();
        for v in vv.drain(..) {
            match v {
                V::Ignored => (),
                _ => {
                    let c = C::from(v.clone());
                    let (c, mut rr) = self.fire_category_events(s, &c, &v);
                    if !rr.is_empty() {
                        let idx = dealloc_idx.entry(c.clone()).or_insert_with(|| {
                            deallocs.push((c, Vec::with_capacity(1)));
                            deallocs.len() - 1
                        }).clone();
                        deallocs[idx].1.append(&mut rr);
                    }
                },
            }
        }
        self.remove_matching(deallocs);
    }
    pub fn fire_and_clean_all(&mut self, s: &State) {
        let rems = self.fire_all_events(s);
        self.remove_matching(rems);
    }
    pub fn handle_req(&mut self, req: RegRequest) -> RegResponse {
        match req {
            RegRequest::Register(c, cbs) => {
                RegResponse::Register(c, cbs.into_iter().map(|cb| self.register(cb)).collect())
            },
            RegRequest::Unregister(cbs) => {
                for cb in cbs {
                    match cb.upgrade() {
                        Some(cb) => self.unregister(cb),
                        None => ()
                    }
                }
                RegResponse::Unregister
            },
        }
    }
}
