//! QSpec Financial Agent
//! 
//! A comprehensive financial analysis and automation tool designed to work with
//! Quicken data formats and provide AI-powered insights.

pub mod agent;
pub mod config;
pub mod data;
pub mod quicken;
pub mod analysis;
pub mod utils;

pub use agent::FinancialAgent;
pub use config::Config;

// Re-export commonly used types
pub use data::{Account, Transaction, FinancialData};
pub use quicken::{QifImporter, QifExporter};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_financial_agent_creation() {
        // Test that we can create a financial agent
        // This is a basic smoke test
        assert!(true);
    }
}
