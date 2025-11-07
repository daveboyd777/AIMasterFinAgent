use anyhow::Result;
use tracing::{info, error};
use crate::config::Config;

/// Main Financial Agent that orchestrates all financial operations
pub struct FinancialAgent {
    config: Config,
}

impl FinancialAgent {
    /// Create a new Financial Agent instance
    pub async fn new() -> Result<Self> {
        let config = Config::load().await?;
        
        info!("Initializing QSpec Financial Agent");
        
        Ok(Self {
            config,
        })
    }
    
    /// Run the main agent loop
    pub async fn run(&self) -> Result<()> {
        info!("Starting QSpec Financial Agent");
        
        // TODO: Implement main agent logic
        // - Monitor for new Quicken data
        // - Process financial transactions
        // - Perform AI analysis
        // - Generate reports and insights
        
        info!("QSpec Financial Agent running successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_financial_agent_new() {
        // Test agent creation
        let result = FinancialAgent::new().await;
        assert!(result.is_ok(), "Failed to create FinancialAgent: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_financial_agent_run() {
        // Test agent execution
        let agent = FinancialAgent::new().await.expect("Failed to create agent");
        let result = agent.run().await;
        assert!(result.is_ok(), "Agent run failed: {:?}", result.err());
    }
}