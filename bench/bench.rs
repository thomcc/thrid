use criterion::*;

criterion::criterion_group!(benches, thread_id_benches);
criterion::criterion_main!(benches);

fn thread_id_benches(c: &mut Criterion) {
    c.bench_function("thrid::ThrId::get()", |b| {
        b.iter(|| thrid::ThrId::get());
    });
    c.bench_function("pointer to `thread_local!`", |b| {
        thread_local!(static BYTE: u8 = const { 0u8 });
        b.iter(|| BYTE.with(|b: &u8| b as *const u8 as usize));
    });
    c.bench_function("`std::thread::current().id()` (direct)", |b| {
        b.iter(|| std::thread::current().id());
    });
    c.bench_function("`std::thread::current().id()` (cached)", |b| {
        thread_local!(static CURRENT_ID: std::thread::ThreadId = std::thread::current().id());
        b.iter(|| CURRENT_ID.with(|tid| *tid));
    });
    c.bench_function("`thread_id::get()` (external crate)", |b| {
        b.iter(|| thread_id::get())
    });
    #[cfg(unix)]
    c.bench_function("`libc::pthread_self()`", |b| {
        extern "C" {
            fn pthread_self() -> usize;
        }
        b.iter(|| unsafe { pthread_self() })
    });
    // #[cfg(target_os = "linux")]
    // c.bench_function("gettid (linux)", |b| {
    //     b.iter(|| unsafe {
    //         libc::syscall(libc::SYS_gettid) as usize
    //         // libc::gettid() as usize
    //     })
    // });
}
