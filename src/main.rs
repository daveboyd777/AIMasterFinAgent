use qspec_fin_agent::FinancialAgent;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Create and start the financial agent
    let agent = FinancialAgent::new().await?;
    agent.run().await?;

    Ok(())
}
