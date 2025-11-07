# QSpec Financial Agent

A comprehensive financial analysis and automation tool designed to work with Quicken data formats and provide AI-powered insights.

## Features

- **Quicken Integration**: Import and export QIF (Quicken Interchange Format) files
- **Financial Analysis**: Generate detailed reports, category analysis, and spending trends
- **Test-First Development**: Comprehensive test suite with high coverage
- **Modern Rust**: Built with async/await, error handling, and type safety
- **XML Processing**: Handle financial data in various XML formats
- **Database Support**: SQLite integration for data persistence
- **CLI Interface**: Command-line tool for batch processing and automation

## Quick Start

### Prerequisites

- Rust 1.70 or later
- WSL (Windows Subsystem for Linux) for Windows users
- Git

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/QSpecFinAgent.git
cd QSpecFinAgent
```

2. Build the project:
```bash
cargo build --release
```

3. Run tests:
```bash
cargo test
```

4. Run the application:
```bash
cargo run
```

## Usage

### Importing Quicken Data

```rust
use qspec_fin_agent::quicken::QifImporter;

// Import from QIF file
let financial_data = QifImporter::import_file("path/to/your/file.qif").await?;
```

### Generating Reports

```rust
use qspec_fin_agent::analysis::AnalysisEngine;

// Generate monthly report
let report = AnalysisEngine::generate_monthly_report(&data, 2024, 1)?;
println!("Net Income: {}", report.net_income);

// Analyze spending by categories
let categories = AnalysisEngine::analyze_categories(&data)?;
for category in categories {
    println!("{}: {}", category.category, category.total_amount);
}
```

### Exporting Data

```rust
use qspec_fin_agent::quicken::QifExporter;

// Export to QIF format
QifExporter::export_file(&financial_data, "output.qif").await?;
```

## Project Structure

```
src/
├── lib.rs          # Library entry point and public API
├── main.rs         # CLI application entry point
├── agent.rs        # Main financial agent orchestrator  
├── config.rs       # Configuration management
├── data.rs         # Core data structures (Account, Transaction, etc.)
├── quicken.rs      # QIF import/export functionality
├── analysis.rs     # Financial analysis and reporting
└── utils.rs        # Utility functions and helpers
```

## Data Models

### Account
Represents a financial account (checking, savings, credit card, etc.)

### Transaction  
Represents individual financial transactions with categories, payees, and metadata

### FinancialData
Container for all financial information including accounts and transactions

## Analysis Features

- **Monthly Reports**: Income, expenses, and net income by month
- **Category Analysis**: Spending breakdown by category with percentages
- **Trend Analysis**: Identify spending trends over time
- **Anomaly Detection**: Detect unusual transactions that may need attention

## Testing

The project follows test-first development principles with comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out html

# Run tests in watch mode (requires cargo-watch)  
cargo watch -x test

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench
```

## Configuration

The application uses a TOML configuration file located at:
- Linux/WSL: `~/.config/qspec-fin-agent/config.toml`
- Windows: `%APPDATA%\qspec\fin-agent\config\config.toml`

Example configuration:

```toml
[database]
path = "/home/user/.local/share/qspec-fin-agent/qspec_fin_agent.db"
max_connections = 5

[quicken]
watch_directory = "/home/user/Documents/Quicken"
file_patterns = ["*.qif", "*.QIF"]
auto_import = true

[ai]
enabled = false
api_endpoint = "https://api.example.com/ai"
api_key = "your-api-key-here"

[logging]
level = "info"
file_logging = true
log_file = "/home/user/.local/share/qspec-fin-agent/qspec_fin_agent.log"
```

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `serde`: Serialization/deserialization
- `anyhow`/`thiserror`: Error handling
- `tracing`: Logging and instrumentation
- `clap`: Command-line interface

### Financial Dependencies  
- `chrono`: Date/time handling
- `rust_decimal`: Precise decimal arithmetic for financial calculations
- `uuid`: Unique identifiers

### Data Processing
- `quick-xml`: XML processing for financial formats
- `sqlx`: Database integration
- `reqwest`: HTTP client for API integration

### Testing Dependencies
- `tokio-test`: Async testing utilities
- `mockall`: Mocking framework
- `wiremock`: HTTP mocking for integration tests
- `proptest`: Property-based testing
- `criterion`: Benchmarking

## Development

### Setting Up Development Environment

1. Install Rust via rustup
2. Install additional tools:
```bash
cargo install cargo-tarpaulin cargo-nextest cargo-watch cargo-edit
```

3. Install pre-commit hooks (optional):
```bash
# Install pre-commit
pip install pre-commit

# Set up hooks
pre-commit install
```

### Code Style

The project follows standard Rust conventions:
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write comprehensive tests for all public APIs
- Document public APIs with doc comments

### Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Write tests for your changes
4. Implement your changes
5. Ensure all tests pass: `cargo test`
6. Run formatting and linting: `cargo fmt && cargo clippy`
7. Commit your changes: `git commit -m 'Add amazing feature'`
8. Push to your branch: `git push origin feature/amazing-feature`
9. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Commercial Licensing

QSpecFinAgent is available under the MIT License for open source projects. For commercial applications that require proprietary licensing or additional features, please contact [daveboyd777@gmail.com](mailto:daveboyd777@gmail.com) for commercial licensing options.

**Commercial Use Cases:**
- Enterprise financial software integration
- Proprietary financial analysis tools
- White-label financial applications
- Custom development and support contracts

The MIT License allows commercial use, but we also offer:
- Extended support and maintenance
- Priority feature development
- Custom integrations and modifications
- Enterprise-grade SLA and support

## Acknowledgments

- Quicken and Intuit for the QIF format specification
- The Rust community for excellent crates and tools
- All contributors to this project

## Roadmap

### Version 0.2.0
- [ ] Database persistence with SQLite
- [ ] Web API interface
- [ ] Real-time file watching for auto-import
- [ ] Advanced reporting with charts and graphs

### Version 0.3.0
- [ ] AI-powered financial insights
- [ ] Machine learning for transaction categorization
- [ ] Budgeting and goal-setting features
- [ ] Multi-currency support

### Version 1.0.0
- [ ] Full Quicken compatibility
- [ ] Cloud synchronization
- [ ] Mobile companion app
- [ ] Advanced security features

## Support

- Documentation: [GitHub Wiki](https://github.com/yourusername/QSpecFinAgent/wiki)
- Issues: [GitHub Issues](https://github.com/yourusername/QSpecFinAgent/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/QSpecFinAgent/discussions)

---

**Note**: This is an open-source project and is not affiliated with Quicken or Intuit Inc.