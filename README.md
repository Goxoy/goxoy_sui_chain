
# SUI Rust Library

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust Version](https://img.shields.io/badge/rustc-1.70%2B-blue)
![Build](https://img.shields.io/github/actions/workflow/status/yourusername/yourrepo/build.yml)

A Rust library designed to make it easier to interact with the [SUI Network](https://sui.io/). This library simplifies the development process by providing functions for various network interactions, allowing developers to focus on building applications without handling complex SUI protocol details.

## Features

- Simplified connection to the SUI network.
- Easy-to-use functions for common SUI network operations.
- Comprehensive error handling and logging.
- Lightweight and fast, optimized for performance.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) version 1.70 or higher.
- A SUI Network node or access to a SUI RPC endpoint.

### Installation

Add this library as a dependency in your `Cargo.toml`:

```toml
[dependencies]
sui-rust-lib = "0.1.0"
```

### Usage

```rust
use sui_rust_lib::SuiClient;

fn main() {
    let client = SuiClient::new("https://rpc.sui.io/");
    match client.get_balance("your_address") {
        Ok(balance) => println!("Balance: {}", balance),
        Err(e) => eprintln!("Error fetching balance: {}", e),
    }
}
```

### Examples

Refer to the `examples` directory for more usage examples, including:

- Retrieving account balances.
- Sending transactions.
- Querying smart contract states.

## Contributing

Contributions are welcome! Please see the [CONTRIBUTING.md](CONTRIBUTING.md) for more information.

## License

This library is licensed under the MIT License. See the [LICENSE](LICENSE) file for more details.

## Contact

Maintained by [Omer Goksoy](https://github.com/omergoksoy).

---

Happy coding with SUI!
