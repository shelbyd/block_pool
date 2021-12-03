# block_pool

A simple object pool that blocks when taking an item out.

```rust
use block_pool::Pool;

let pool = Pool::new(vec![1, 2, 3]);
let mut item = pool.take();
*item += 1;
drop(item);
```

License: MIT
