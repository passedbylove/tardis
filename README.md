**Preview version, will not guarantee the stability of the API!
Do NOT use in production environment!**

---

**Elegant, Clean Rust development framework🛸**

---

[![Crate](https://img.shields.io/crates/v/tardis.svg)](https://crates.io/crates/tardis)
[![Docs](https://docs.rs/tardis/badge.svg)](https://docs.rs/tardis)
[![Build Status](https://github.com/ideal-world/tardis/actions/workflows/cicd.yml/badge.svg)](https://github.com/ideal-world/tardis/actions/workflows/cicd.yml)
[![Test Coverage](https://codecov.io/gh/ideal-world/tardis/branch/main/graph/badge.svg?token=L1LQ8DLUS2)](https://codecov.io/gh/ideal-world/tardis)
[![License](https://img.shields.io/github/license/ideal-world/tardis)](https://github.com/ideal-world/tardis/blob/main/LICENSE)

> TARDIS([tɑːrdɪs] "Time And Relative Dimension In Space") From "Doctor Who".

## 💖 Core functions

* Relational database client for MySQL, PostgresSQL
* Web service and web client for OpenAPI v3.x
* Distributed cache client for Redis protocol
* RabbitMQ client for AMQP protocol
* Search client for Elasticsearch
* Mainstream encryption algorithms and SM2/3/4 algorithms
* Containerized unit testing of mainstream middleware
* Multi-environment configuration
* Commonly used operations (E.g. uniform error handling, encryption and decryption, regular checksums)

## ⚙️Feature description

* ``trace`` tracing operation
* ``crypto`` Encryption, decryption and digest operations
* ``future`` asynchronous operations
* ``reldb`` relational database operations(based on [SeaORM](https://github.com/SeaQL/sea-orm))
* ``web-server`` web service operations(based on [Poem](https://github.com/poem-web/poem))
* ``web-client`` web client operations
* ``cache`` cache operations
* ``mq`` message queue operations
* ``test`` unit test operations

## 🚀 Quick start

The core operations of the framework all use ``TardisFuns`` as an entry point.
E.g.

```
TardisFuns::init(relative_path)      // Initialize the configuration
TardisFuns::field.x                  // Some field operations
TardisFuns::reldb().x                // Some relational database operations
TardisFuns::web_server().x           // Some web service operations
```

### Web service example

Dependency Configuration
```toml
[dependencies]
tardis = { version = "^0", features = ["web-server"] }
poem-openapi = { version = "^1"}
```

Processor Configuration
```rust
pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> TardisResult<String> {
        match name.0 {
            Some(name) => TardisResp::ok(format!("hello, {}!", name)),
            None => TardisResp::err(TardisError::NotFound("name does not exist".to_string())),
        }
    }
}
```

Startup class configuration
```rust
#[tokio::main]
async fn main() -> TardisResult<()> {
    // Initial configuration
    TardisFuns::init::<NoneConfig>("config").await?;
    // Register the processor and start the web service
    TardisFuns::web_server().add_module("", Api).start().await
}
```

### More examples

```
|-- examples
  |-- reldb         Relational database usage example
  |-- web-basic     Web service Usage Example
  |-- web-client    Web client Usage Example
  |-- webscoket     WebSocket Usage Example
  |-- cache         Cache Usage Example
  |-- mq            Message Queue Usage Example
  |-- todo          A complete project usage example
  |-- perf-test     Performance test case
```

----
Thanks to `Jetbrains` for the [Open Source License](https://www.jetbrains.com/community/opensource/)