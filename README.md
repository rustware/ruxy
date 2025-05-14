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

Ruxy is an emerging framework that brings together the performance of Rust and the flexibility of React.

It enables true server-side rendering without Node.js, V8, or JavaScript runtimes on the server — giving you
a fully native, type-safe backend that serves pre-hydrated React apps at incredible speed.

With Ruxy, you write your frontend in React, your backend in Rust, and deploy a single,
unified application — no compromise, no JS bloat, no V8.

#### Project status

Ruxy is just getting started — under active development and open to contributions. 🧱

---

### Server-side rendering, reimagined.

#### Client

```tsx
import { useQuery, useServerValues } from '@ruxyjs/server';

export default function Homepage() {
  const { name } = useQuery();
  const { message, luckyNumber } = useServerValues();

  return (
    <>
      <h1>Hello, {query.name}!</h1>
      <p>Server message is: {message}</p>
      <p>Lucky number: {luckyNumber}</p>
    </>
  );
}
```

#### Server

```rust
#[ruxy::cache(1 hour)]
async fn page(ctx: Context) {
  ruxy::add_value!("message", "Hi there!");
  ruxy::add_value!("luckyNumber", 42);
}
```

Pretty simple, huh?

---

...
