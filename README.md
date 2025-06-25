<div align="center">
  <a href="https://ruxy.dev">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="assets/logo/light/ruxy.svg">
      <img alt="Next.js logo" src="assets/logo/dark/ruxy.svg" height="64">
    </picture>
  </a>
  <div align="center">
    <h3>
      Bringing React to Rust. 🎉 <br /><br />
    </h3>
  </div>
</div>

**Ruxy** is an emerging framework that combines the benefits of Rust with the flexibility of React.

It enables React server-side rendering without Node.js, V8, or any JavaScript runtime on the server — a fully
native backend serving pre-rendered React apps at incredible speed ([how it works](ARCHITECTURE.md)).

With Ruxy, you build your frontend in [React](https://react.dev) and your backend in [Rust](https://rust-lang.org).

---

### Performance

Thanks to its novel approach, Ruxy is easily the  [fastest](BENCHMARKS.md) React SSR framework out there.
Your pre-rendered React application is served by pure Rust.

How Ruxy optimizes for high performance:
1. There is no JS runtime on the server to slow your app down.
2. Ruxy renders your application from pre-rendered static chunks.
3. Despite its flexibility, routing doesn't allocate.
4. Ruxy is `async` – no expensive thread pre-emptions.
5. Ruxy's async model doesn't require synchronization ([read more](#Thread-per-core)).

---

### Project status

Ruxy is just getting started — under active development and open to contributions. 🧱

---

### Server-side rendering made easy

#### Client – `page.tsx`:

```tsx
import { useQuery } from 'ruxy/hooks';
import { usePageProps } from './+props';

export default function Homepage() {
  const query = useQuery();
  const name = query.get('name') ?? 'stranger';

  const { luckyNumber } = usePageProps(); // fully typed!

  return (
    <>
      <h1>Hello, {name}!</h1>
      <p>Your lucky number is {luckyNumber}.</p>
    </>
  );
}
```

#### Server – `page.rs`:

```rust
use ruxy::{loader, Props};

#[derive(Props)]
struct Props {
  lucky_number: u8,
}

#[loader]
async fn loader() -> Props {
  Props { lucky_number: 42 }
}
```

---

### Thread-per-core

Ruxy is a **painless-async** framework. What does that mean?

It adopts a **thread-per-core** model, where each core runs its own isolated async executor.

When a request comes in, Ruxy load-balances it internally across threads. Each request is then fully processed on its assigned thread, using async—but without hopping between threads.

This
[increasingly](https://maciej.codes/2022-06-09-local-async.html)
[popular](https://emschwartz.me/async-rust-can-be-a-pleasure-to-work-with-without-send-sync-static/)
model offers significant advantages:
- **Developer Experience**:
  - You're freed from `Send + 'static` constraints.
  - Async Rust feels much closer to writing synchronous Rust.
- **Performance**:
  - Eliminates synchronization overhead.
  - Improves CPU cache locality.
  - No task migration cost.

It’s worth noting that there is some [debate](https://without.boats/blog/thread-per-core/) about the suitability of the thread-per-core model for applications with uneven workload distribution.
However, this concern typically doesn’t apply to web applications.
In this context, Ruxy's thread-per-core model delivers both a great developer experience, and similar (or possibly even better) performance than work-stealing.

---

### Containerized Environments

Ruxy is a great fit whether you're using Kubernetes or another containerization platform.

As usual, the amount of containers vs. number of CPUs assigned to each container depends on whether
you want to optimize for throughput or the latency, and balancing these two is higly dependent on your
specific workload.

That said, Ruxy avoids synchronization overhead entirely, so single-threaded and multi-threaded
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
for inclusion in this project by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
