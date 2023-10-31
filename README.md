# Block AI crawling in Actix

Crates link: https://crates.io/crates/actix_block_ai_crawling
Docs link: https://docs.rs/actix_block_ai_crawling/latest/actix_block_ai_crawling/

This blocks any HTTP requests coming from a Generative AI crawler. It works by blocking matching User Agents.

It's extremely simple to use. Just add `.wrap(actix_block_ai_crawling::BlockAi);` to your app.
```rust
let app = App::new()
.wrap(actix_block_ai_crawling::BlockAi);
```

Pull requests are welcome! Please hand-write your code. 
AI written code is not welcome.
