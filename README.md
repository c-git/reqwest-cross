# reqwest-cross
Wrapper around [reqwest][reqwest-url] for ease of use in applications that target BOTH native and wasm but do not want to block in the calling task (eg. A UI thread). Inspired by [ehttp](https://docs.rs/ehttp/0.2.0/ehttp/) but uses [reqwest][reqwest-url] instead. Doesn't provide much value if you're only targeting one or the other because request does that pretty well, see their [wasm example](https://github.com/seanmonstar/reqwest/tree/master/examples/wasm_github_fetch).

[reqwest-url]: https://docs.rs/reqwest/latest/reqwest/
