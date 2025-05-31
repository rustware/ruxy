<div align="center">
  <a href="https://ruxy.dev">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="assets/logo/light/ruxy.svg">
      <img alt="Next.js logo" src="assets/logo/dark/ruxy.svg" height="64">
    </picture>
  </a>
  <div align="center">
    <h3>
      Rust speed. 
      React simplicity. 
      Ruxy. 🎉 <br /><br />
    </h3>
  </div>
</div>

**Ruxy** is an emerging framework that combines the performance of Rust with the flexibility of React.

It enables true server-side rendering without Node.js, V8, or any JavaScript runtime on the server — a fully
native backend serving pre-rendered React apps at incredible speed ([how it works](ARCHITECTURE.md)).

With Ruxy, you build your frontend in [React](https://react.dev) and your backend in [Rust](https://rust-lang.org) — seamlessly integrated for a smooth developer experience.

---

### Performance

Thanks to its novel approach, Ruxy is easily the  [fastest](BENCHMARKS.md) React SSR framework out there.
Ruxy makes you write your backend code in Rust, and your frontend code in React while preserving SSR.

This bridges the gap between fast backends and development experience on frontend.

How Ruxy optimizes for high performance:
1. There is no JS runtime on the server to slow your app down.
2. Ruxy renders your application without a single allocation.
3. Ruxy is `async` – no expensive thread pre-emptions.
4. Ruxy's async model doesn't require synchronization ([read more](#Runtime)).

---

### Project status

Ruxy is just getting started — under active development and open to contributions. 🧱

---

### Server-side rendering, reimagined.

#### Client

```tsx
import { useQuery, useServerValues } from '@ruxyjs/server';

export default function Homepage() {
  const { name } = useQuery();
  const { luckyNumber } = useServerValues();

  return (
    <>
      <h1>Hello, {query.name}!</h1>
      <p>Server's lucky number is {luckyNumber}</p>
    </>
  );
}
```

#### Server

```rust
#[ruxy::page]
async fn page(ctx: Context, name: QueryParam) {
  ruxy::value!("luckyNumber", 42);
}
```

---

### Runtime

Ruxy is a **painless-async** framework. What does that mean?

It adopts a **thread-per-core** model, where each core runs its own isolated async executor.

When a request comes in, Ruxy load-balances it internally across threads. Each request is then fully processed on its assigned thread, using async—but without hopping between threads.

This
[increasingly](https://maciej.codes/2022-06-09-local-async.html)
[popular](https://news.ycombinator.com/item?id=29500309)
[model](https://github.com/DataDog/glommio)
offers significant advantages:
- **Performance**:
  - Eliminates synchronization overhead.
  - Improves CPU cache locality.
  - No task migration cost.
- **Developer Experience**:
  - You’re freed from `Send + 'static` constraints.
  - Async Rust feels much closer to writing synchronous Rust.

It’s worth noting that there is some [debate](https://without.boats/blog/thread-per-core/) about the suitability of the thread-per-core model for applications with uneven workload distribution.
However, this concern typically doesn’t apply to web applications.
In this context, Ruxy’s thread-per-core model delivers both high performance and a great developer experience.

---

### Containerized Environments

Ruxy is a great fit whether you're using Kubernetes or another containerization platform.

For optimal performance, we slightly recommend using **fewer containers with more cores each**.
This allows Ruxy's internal thread-based load balancing to shine – reducing the latency and
overhead of external load balancers and extra network hops.

That said, there's **no disadvantage to assigning just a single core** to your container.
Ruxy avoids synchronization overhead entirely, so single-threaded and multi-threaded
deployments both perform efficiently, with no penalties either way.

---

### Caveats

(WIP)

- Some third-party libraries that provide APIs requiring to pass a Future will expect that Future to be `Send + 'static`, as the authors assume we're running under work-stealing executor, which isn't the case with Ruxy. Nothing to do about that (TODO: or maybe Ruxy can provide some kind of a helper wrapper using `unsafe` magic to make those values be accepted by the third-party lib).
- Conditional JSX/TSX rendering – all control logic must come from the server (TODO: elaborate).

---

### License


Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
