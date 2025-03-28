# Examples
This directory contains examples of different DI patterns and integrations with external systems.

To run an example use e.g.:
```sh
cargo run -p example-axum
```

- [`axum`](./axum/)
  - Showcases using DI together with [`axum`](https://docs.rs/axum) crate
  - Provides an example of using `axum` layers to extract information from the request (e.g. auth token) and add it into request-scoped catalog
- [`cli`](./cli/)
  - Showcases using DI in a command line application to construct `Command` objects that combine injected dependencies with explicit command arguments
