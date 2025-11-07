use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a financial account (checking, savings, credit card, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub institution: Option<String>,
    pub account_number: Option<String>,
    pub balance: Decimal,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Types of financial accounts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    Checking,
    Savings,
    CreditCard,
    Investment,
    Cash,
    Liability,
    Asset,
    Other(String),
}

/// Represents a financial transaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub date: DateTime<Utc>,
    pub amount: Decimal,
    pub description: String,
    pub category: Option<String>,
    pub payee: Option<String>,
    pub memo: Option<String>,
    pub cleared: bool,
    pub reconciled: bool,
    pub transaction_type: TransactionType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Types of transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Debit,
    Credit,
    Transfer,
    Fee,
    Interest,
    Dividend,
    Other(String),
}

/// Container for all financial data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialData {
    pub accounts: Vec<Account>,
    pub transactions: Vec<Transaction>,
    pub categories: Vec<String>,
    pub payees: Vec<String>,
}

impl Account {
    /// Create a new account
    pub fn new(
        name: String,
        account_type: AccountType,
        balance: Decimal,
        currency: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            account_type,
            institution: None,
            account_number: None,
            balance,
            currency,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update account balance
    pub fn update_balance(&mut self, new_balance: Decimal) {
        self.balance = new_balance;
        self.updated_at = Utc::now();
    }
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        account_id: Uuid,
        date: DateTime<Utc>,
        amount: Decimal,
        description: String,
        transaction_type: TransactionType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            account_id,
            date,
            amount,
            description,
            category: None,
            payee: None,
            memo: None,
            cleared: false,
            reconciled: false,
            transaction_type,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark transaction as cleared
    pub fn mark_cleared(&mut self) {
        self.cleared = true;
        self.updated_at = Utc::now();
    }

    /// Mark transaction as reconciled
    pub fn mark_reconciled(&mut self) {
        self.reconciled = true;
        self.cleared = true; // Reconciled implies cleared
        self.updated_at = Utc::now();
    }
}

impl FinancialData {
    /// Create new empty financial data container
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            transactions: Vec::new(),
            categories: Vec::new(),
            payees: Vec::new(),
        }
    }

    /// Add an account
    pub fn add_account(&mut self, account: Account) {
        self.accounts.push(account);
    }

    /// Add a transaction
    pub fn add_transaction(&mut self, transaction: Transaction) {
        // Add category if not exists
        if let Some(ref category) = transaction.category {
            if !self.categories.contains(category) {
                self.categories.push(category.clone());
            }
        }

        // Add payee if not exists
        if let Some(ref payee) = transaction.payee {
            if !self.payees.contains(payee) {
                self.payees.push(payee.clone());
            }
        }

        self.transactions.push(transaction);
    }

    /// Get transactions for a specific account
    pub fn get_account_transactions(&self, account_id: &Uuid) -> Vec<&Transaction> {
        self.transactions
            .iter()
            .filter(|t| &t.account_id == account_id)
            .collect()
    }

    /// Calculate account balance from transactions
    pub fn calculate_account_balance(&self, account_id: &Uuid) -> Decimal {
        self.get_account_transactions(account_id)
            .iter()
            .map(|t| match t.transaction_type {
                TransactionType::Credit => t.amount,
                TransactionType::Debit => -t.amount,
                TransactionType::Transfer => t.amount,
                _ => t.amount,
            })
            .sum()
    }
}

impl Default for FinancialData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_account_creation() {
        let account = Account::new(
            "Test Checking".to_string(),
            AccountType::Checking,
            dec!(1000.00),
            "USD".to_string(),
        );

        assert_eq!(account.name, "Test Checking");
        assert_eq!(account.account_type, AccountType::Checking);
        assert_eq!(account.balance, dec!(1000.00));
        assert_eq!(account.currency, "USD");
    }

    #[test]
    fn test_transaction_creation() {
        let account_id = Uuid::new_v4();
        let transaction = Transaction::new(
            account_id,
            Utc::now(),
            dec!(50.00),
            "Test purchase".to_string(),
            TransactionType::Debit,
        );

        assert_eq!(transaction.account_id, account_id);
        assert_eq!(transaction.amount, dec!(50.00));
        assert_eq!(transaction.description, "Test purchase");
        assert_eq!(transaction.transaction_type, TransactionType::Debit);
        assert!(!transaction.cleared);
        assert!(!transaction.reconciled);
    }

    #[test]
    fn test_financial_data() {
        let mut data = FinancialData::new();

        let account = Account::new(
            "Test Account".to_string(),
            AccountType::Checking,
            dec!(1000.00),
            "USD".to_string(),
        );
        let account_id = account.id;

        data.add_account(account);

        let mut transaction = Transaction::new(
            account_id,
            Utc::now(),
            dec!(100.00),
            "Test transaction".to_string(),
            TransactionType::Debit,
        );
        transaction.category = Some("Groceries".to_string());
        transaction.payee = Some("Store ABC".to_string());

        data.add_transaction(transaction);

        assert_eq!(data.accounts.len(), 1);
        assert_eq!(data.transactions.len(), 1);
        assert_eq!(data.categories.len(), 1);
        assert_eq!(data.payees.len(), 1);
        assert!(data.categories.contains(&"Groceries".to_string()));
        assert!(data.payees.contains(&"Store ABC".to_string()));
    }

    #[test]
    fn test_account_balance_calculation() {
        let mut data = FinancialData::new();
        let account_id = Uuid::new_v4();

        // Add some transactions
        let credit = Transaction::new(
            account_id,
            Utc::now(),
            dec!(1000.00),
            "Initial deposit".to_string(),
            TransactionType::Credit,
        );

        let debit = Transaction::new(
            account_id,
            Utc::now(),
            dec!(250.00),
            "Purchase".to_string(),
            TransactionType::Debit,
        );

        data.add_transaction(credit);
        data.add_transaction(debit);

        let balance = data.calculate_account_balance(&account_id);
        assert_eq!(balance, dec!(750.00)); // 1000 - 250
    }

    #[test]
    fn test_transaction_state_changes() {
        let mut transaction = Transaction::new(
            Uuid::new_v4(),
            Utc::now(),
            dec!(100.00),
            "Test".to_string(),
            TransactionType::Debit,
        );

        assert!(!transaction.cleared);
        assert!(!transaction.reconciled);

        transaction.mark_cleared();
        assert!(transaction.cleared);
        assert!(!transaction.reconciled);

        transaction.mark_reconciled();
        assert!(transaction.cleared);
        assert!(transaction.reconciled);
    }
}
