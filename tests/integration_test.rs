// Simple integration test to verify basic functionality
use qspec_fin_agent::*;

#[tokio::test]
async fn test_basic_functionality() {
    // Test that we can create basic data structures
    let mut data = FinancialData::new();

    // Create a test account
    let account = Account::new(
        "Test Account".to_string(),
        data::AccountType::Checking,
        rust_decimal::Decimal::new(1000, 2), // $10.00
        "USD".to_string(),
    );

    data.add_account(account);
    assert_eq!(data.accounts.len(), 1);
}
