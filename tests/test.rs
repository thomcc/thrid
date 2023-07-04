use std::collections::HashMap;

#[test]
fn same_thread_eq() {
    let a = thrid::ThrId::get();
    let b = thrid::ThrId::get();
    assert_eq!(a, b);
}

#[test]
fn diff_thread_ne() {
    let a = thrid::ThrId::get();
    let b = std::thread::spawn(|| thrid::ThrId::get()).join().unwrap();
    assert_ne!(a, b);
}

#[test]
fn many_threads_all_ne() {
    use std::thread::{current, ThreadId};
    let nthreads = 100usize;
    let pair = |i: usize| (thrid::ThrId::get(), current().id(), i);
    let mtx = &std::sync::Mutex::new(vec![pair(0)]);
    mtx.lock().unwrap().reserve(nthreads);
    let barrier = &std::sync::Barrier::new(nthreads);
    std::thread::scope(|scope| {
        let mut v = Vec::with_capacity(nthreads);
        for t in 0..nthreads {
            v.push(scope.spawn(move || {
                let to_add = pair(t + 1);
                mtx.lock().unwrap().push(to_add);
                barrier.wait();
            }));
        }
        for (i, handle) in v.into_iter().enumerate() {
            handle.join().unwrap_or_else(|e| {
                let msg = e
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| e.downcast_ref::<&'static str>().copied())
                    .unwrap_or("no clue");
                panic!("damn, thread {} panicked: {msg:?}", i + 1);
            });
        }
    });
    let ids = mtx.lock().unwrap();
    let mut hm: HashMap<thrid::ThrId, (ThreadId, usize)> = HashMap::with_capacity(ids.len());
    for &(mine, std, num) in ids.iter() {
        if let Some((old_std, old_num)) = hm.insert(mine, (std, num)) {
            panic!(
                "`thrid` {mine:?} duplicated between {:?} and {:?}",
                (std, num),
                (old_std, old_num),
            );
        }
    }
}

#[test]
fn many_threads_all_same_consistently() {
    std::thread::scope(|scope| {
        for _ in 0..100 {
            scope.spawn(|| {
                let a = thrid::ThrId::get();
                std::thread::sleep(std::time::Duration::from_millis(300));
                let b = thrid::ThrId::get();
                assert_eq!(a, b);
            });
        }
    });
}
