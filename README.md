# reqwest-cross
Wrapper around [reqwest][reqwest-url] for ease of use in applications that target BOTH native and wasm but do not want to block in the calling task (eg. A UI thread). Inspired by [ehttp](https://docs.rs/ehttp/0.2.0/ehttp/) but uses [reqwest][reqwest-url] instead. Doesn't provide much value if you're only targeting one or the other because request does that pretty well, see their [wasm example](https://github.com/seanmonstar/reqwest/tree/master/examples/wasm_github_fetch).

## License

All code in this repository is dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are very good reasons to include both as noted in
this [issue](https://github.com/bevyengine/bevy/issues/2373) on [Bevy](https://bevyengine.org)'s repo.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.


[reqwest-url]: https://docs.rs/reqwest/latest/reqwest/
