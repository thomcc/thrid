# `thrid`: Fast Thread Identifier

This crate provides a very fast implementation of per-thread identifier. It is implemented generally by a small bit of inline assembly which returns a pointer to the thread control block, the thread environment block, the TLS base, or whatever the equivalent on your OS is. If we don't have an asm shim for it, we fall back to using either an OS thread ID value, or a pointer to a `thread_local!` variable (in which case we require `std`).

Please file bugs if it misbehaves, it's fairly new.

## Performance

In most cases (e.g. in cases where we have the asm shim) it's much faster than `std::thread::current().id()` (even if that value is cached in a `thread_local`), or access to the platform thread ID.

Compared to taking using a pointer to a `thread_local` this offers more consistent performance, and should never be slower. On ELF targets, access to a `thread_local` is often very fast, but that performance can drop drastically if done from within a dynamic library (dylib/cdylib crate), especially one loaded at runtime. Conversely, `thrid` is the same speed in these cases. On non-ELF targets `thrid` should be strictly faster in basically every case.

I need to make new benchmarks, and run them on different targets, but here are some preliminary numbers (from `aarch64-apple-darwin`).

```
ThrId::get()            time:   [321.86 ps 322.32 ps 322.81 ps]
thread_local! ptr       time:   [960.84 ps 962.03 ps 963.38 ps]
#[thread_local] ptr     time:   [961.74 ps 963.61 ps 966.35 ps]
thread::current().id()  time:   [9.6959 ns 9.7077 ns 9.7206 ns]
pthread_self            time:   [1.9381 ns 1.9410 ns 1.9438 ns]
pthread_getspecific     time:   [1.2926 ns 1.2950 ns 1.2975 ns]
```

## Use-cases

This is useful for cases like:

1. Memory allocators which want to detect cross-thread frees.
2. Recursive locks which use a thread ID to distinguish the owner.
3. Per-thread caches, so long as the caches can handle the case where the value has already been initialized due to accidental collision.

On tested platforms it's faster than:

1. The [`thread-id`](https://crates.io/crates/thread-id) crate. Unlike `thread-id`, the identifier returned by `thrid::ThrId::get()` is not a system thread ID. This allows a faster implementation on some targets, but the tradeoff is that it is very unlikely to be as useful for debugging, monitoring, and so on.

2. Getting the [`std::thread::ThreadId`](https://doc.rust-lang.org/nightly/std/thread/struct.ThreadId.html) for the current thread using something like [`std::thread::current().id()`](https://doc.rust-lang.org/nightly/std/thread/struct.Thread.html#method.id), even if the ID is cached.

    As mentioned, `std::thread::ThreadId` is guaranteed never to be reused, wheras the value from `thrid::ThrId::get()` may be reused after a thread terminates. On the other hand, `std::thread::current()` can panic during thread tear-down, whereas `thrid::ThrId::get()` will not.

## License

This is free and unencumbered software released into the public domain, as explained by [the Unlicense](./UNLICENSE), you may use it however you wish.

As a concession to the unfortunate reality that such licenses are not always usable, you may also use this code under either the terms of the [Apache License, Version 2.0](./LICENSE-APACHE) or the MIT license or the [MIT License](./LICENSE-MIT), at your option.
