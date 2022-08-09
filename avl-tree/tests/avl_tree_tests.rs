use bytemuck::Pod;
use bytemuck::Zeroable;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::{self, Rng};
use avl_tree::*;
use std::collections::BTreeMap;

const MAX_SIZE: usize = 20001;

#[repr(C)]
#[derive(Default, Copy, Clone, PartialEq)]
struct Widget {
    a: u128,
    b: u128,
    size: u64,
}

unsafe impl Zeroable for Widget {}
unsafe impl Pod for Widget {}

impl Widget {
    pub fn new_random(r: &mut ThreadRng) -> Self {
        Self {
            a: r.gen::<u128>(),
            b: r.gen::<u128>(),
            size: r.gen::<u64>(),
        }
    }
}

#[tokio::test(threaded_scheduler)]
async fn test_simulate() {
    type AVL = AVLTree<u128, Widget, MAX_SIZE>;
    let mut buf = vec![0u8; std::mem::size_of::<AVL>()];
    let avl = AVL::new_from_slice(buf.as_mut_slice());
    println!("AVLT Memory Size: {}", std::mem::size_of::<AVL>());
    println!("AVLT Capacity: {}", MAX_SIZE - 1);
    let mut rng = thread_rng();
    let mut keys = vec![];
    let mut map = Box::new(BTreeMap::new());
    let mut s = 0;
    let mut v;
    for _ in 0..(MAX_SIZE - 1) {
        let k = rng.gen::<u128>();
        v = Widget::new_random(&mut rng);
        assert!(avl.insert(k, v) != None);
        s += 1;
        assert!(s == avl.size());
        map.insert(k, v);
        keys.push(k);
    }

    let k = rng.gen::<u128>();
    let v = Widget::new_random(&mut rng);
    assert!(avl.insert(k, v) == None);

    for k in keys.iter() {
        assert!(avl.remove(k) != None);
        s -= 1;
        map.remove(k);
    }
    keys = vec![];

    for _i in 0..(MAX_SIZE >> 1) {
        let k = rng.gen::<u128>();
        let v = Widget::new_random(&mut rng);
        assert!(avl.insert(k, v) != None);
        s += 1;
        map.insert(k, v);
        keys.push(k);
    }

    for _ in 0..100000 {
        assert!(s == avl.size());
        let sample = rng.gen::<f64>();
        if sample < 0.33 {
            if avl.size() >= MAX_SIZE - 1 {
                continue;
            }
            let k = rng.gen::<u128>();
            let v = Widget::new_random(&mut rng);
            assert!(avl.insert(k, v) != None);
            s += 1;
            map.insert(k, v);
            keys.push(k);
        } else if sample < 0.66 {
            if keys.is_empty() {
                continue;
            }
            let j = rng.gen_range(0, keys.len());
            let key = keys[j];
            keys.swap_remove(j);
            assert!(avl[&key] == map[&key]);
            assert!(avl.remove(&key) != None);
            map.remove(&key);
            s -= 1;
        } else {
            if keys.is_empty() {
                continue;
            }
            let j = rng.gen_range(0, keys.len());
            let key = keys[j];
            let v = Widget::new_random(&mut rng);
            assert!(avl.insert(key, v) != None);
            map.insert(key, v);
        }
    }

    for ((k1, v1), (k2, v2)) in map.iter().zip(avl.iter()) {
        assert!(*k1 == *k2);
        assert!(*v1 == *v2);
    }
}