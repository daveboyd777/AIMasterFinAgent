use anyhow::{Result, Context, bail};
use std::path::Path;
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;
use uuid::Uuid;
use crate::data::{Account, Transaction, FinancialData, AccountType, TransactionType};

/// QIF (Quicken Interchange Format) importer
pub struct QifImporter;

/// QIF exporter for creating Quicken-compatible files
pub struct QifExporter;

impl QifImporter {
    /// Import QIF file and return financial data
    pub async fn import_file<P: AsRef<Path>>(path: P) -> Result<FinancialData> {
        let content = tokio::fs::read_to_string(path.as_ref()).await
            .context("Failed to read QIF file")?;
        
        Self::parse_qif_content(&content)
    }
    
    /// Parse QIF content from string
    pub fn parse_qif_content(content: &str) -> Result<FinancialData> {
        let mut data = FinancialData::new();
        let mut current_account: Option<Account> = None;
        let mut account_map: HashMap<String, Uuid> = HashMap::new();
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.starts_with("!Account") {
                // Parse account header
                i += 1;
                current_account = Some(Self::parse_account_section(&lines, &mut i)?);
                if let Some(ref account) = current_account {
                    account_map.insert(account.name.clone(), account.id);
                    data.add_account(account.clone());
                }
            } else if line.starts_with("!Type:") {
                // Parse transaction section
                let account_id = current_account
                    .as_ref()
                    .map(|a| a.id)
                    .context("No current account for transactions")?;
                
                i += 1;
                let transactions = Self::parse_transaction_section(&lines, &mut i, account_id)?;
                for transaction in transactions {
                    data.add_transaction(transaction);
                }
            } else {
                i += 1;
            }
        }
        
        Ok(data)
    }
    
    fn parse_account_section(lines: &[&str], index: &mut usize) -> Result<Account> {
        let mut name = "Unknown Account".to_string();
        let mut account_type = AccountType::Other("Unknown".to_string());
        let mut description = None;
        let mut balance = Decimal::ZERO;
        
        while *index < lines.len() {
            let line = lines[*index].trim();
            
            if line.is_empty() || line.starts_with("!") {
                break;
            }
            
            if let Some(content) = line.strip_prefix('N') {
                name = content.to_string();
            } else if let Some(content) = line.strip_prefix('T') {
                account_type = Self::parse_account_type(content);
            } else if let Some(content) = line.strip_prefix('D') {
                description = Some(content.to_string());
            } else if line == "^" {
                *index += 1;
                break;
            }
            
            *index += 1;
        }
        
        let mut account = Account::new(name, account_type, balance, "USD".to_string());
        if let Some(desc) = description {
            // Store description in institution field for now
            account.institution = Some(desc);
        }
        
        Ok(account)
    }
    
    fn parse_transaction_section(
        lines: &[&str], 
        index: &mut usize, 
        account_id: Uuid
    ) -> Result<Vec<Transaction>> {
        let mut transactions = Vec::new();
        
        while *index < lines.len() {
            let line = lines[*index].trim();
            
            if line.is_empty() || line.starts_with("!") {
                break;
            }
            
            if let Ok(transaction) = Self::parse_single_transaction(lines, index, account_id) {
                transactions.push(transaction);
            } else {
                *index += 1;
            }
        }
        
        Ok(transactions)
    }
    
    fn parse_single_transaction(
        lines: &[&str], 
        index: &mut usize, 
        account_id: Uuid
    ) -> Result<Transaction> {
        let mut date = None;
        let mut amount = Decimal::ZERO;
        let mut description = "Unknown".to_string();
        let mut payee = None;
        let mut category = None;
        let mut memo = None;
        let mut cleared = false;
        
        while *index < lines.len() {
            let line = lines[*index].trim();
            
            if line == "^" {
                *index += 1;
                break;
            }
            
            if let Some(content) = line.strip_prefix('D') {
                date = Some(Self::parse_qif_date(content)?);
            } else if let Some(content) = line.strip_prefix('T') {
                amount = content.trim().parse::<Decimal>()
                    .context("Failed to parse transaction amount")?;
            } else if let Some(content) = line.strip_prefix('P') {
                payee = Some(content.to_string());
                description = content.to_string(); // Use payee as description if no memo
            } else if let Some(content) = line.strip_prefix('L') {
                category = Some(content.to_string());
            } else if let Some(content) = line.strip_prefix('M') {
                memo = Some(content.to_string());
                description = content.to_string(); // Use memo as description
            } else if let Some(content) = line.strip_prefix('C') {
                cleared = content == "*" || content.to_lowercase() == "x";
            }
            
            *index += 1;
        }
        
        let transaction_date = date.unwrap_or_else(Utc::now);
        let transaction_type = if amount >= Decimal::ZERO {
            TransactionType::Credit
        } else {
            TransactionType::Debit
        };
        
        let mut transaction = Transaction::new(
            account_id,
            transaction_date,
            amount.abs(),
            description,
            transaction_type,
        );
        
        transaction.payee = payee;
        transaction.category = category;
        transaction.memo = memo;
        transaction.cleared = cleared;
        
        Ok(transaction)
    }
    
    fn parse_qif_date(date_str: &str) -> Result<DateTime<Utc>> {
        // QIF dates can be in various formats: M/D/YY, MM/DD/YYYY, etc.
        let cleaned = date_str.trim().replace('\'', "");
        
        // Try different date formats
        let formats = [
            "%m/%d/%Y",
            "%m/%d/%y", 
            "%m-%d-%Y",
            "%m-%d-%y",
            "%Y-%m-%d",
        ];
        
        for format in &formats {
            if let Ok(naive_date) = NaiveDate::parse_from_str(&cleaned, format) {
                return Ok(naive_date.and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc());
            }
        }
        
        bail!("Could not parse date: {}", date_str)
    }
    
    fn parse_account_type(type_str: &str) -> AccountType {
        match type_str.to_lowercase().as_str() {
            "bank" | "checking" => AccountType::Checking,
            "savings" => AccountType::Savings,
            "ccard" | "credit" => AccountType::CreditCard,
            "invst" | "investment" => AccountType::Investment,
            "cash" => AccountType::Cash,
            "liability" => AccountType::Liability,
            "asset" => AccountType::Asset,
            other => AccountType::Other(other.to_string()),
        }
    }
}

impl QifExporter {
    /// Export financial data to QIF format
    pub fn export_to_string(data: &FinancialData) -> Result<String> {
        let mut output = String::new();
        
        for account in &data.accounts {
            output.push_str(&Self::export_account(account)?);
            output.push('\n');
            
            let transactions = data.get_account_transactions(&account.id);
            if !transactions.is_empty() {
                output.push_str(&Self::export_transactions(&transactions, account)?);
                output.push('\n');
            }
        }
        
        Ok(output)
    }
    
    /// Export to QIF file
    pub async fn export_file<P: AsRef<Path>>(data: &FinancialData, path: P) -> Result<()> {
        let content = Self::export_to_string(data)?;
        tokio::fs::write(path.as_ref(), content).await
            .context("Failed to write QIF file")?;
        Ok(())
    }
    
    fn export_account(account: &Account) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("!Account\n");
        output.push_str(&format!("N{}\n", account.name));
        output.push_str(&format!("T{}\n", Self::account_type_to_qif(&account.account_type)));
        
        if let Some(ref institution) = account.institution {
            output.push_str(&format!("D{}\n", institution));
        }
        
        output.push_str("^\n");
        
        Ok(output)
    }
    
    fn export_transactions(transactions: &[&Transaction], account: &Account) -> Result<String> {
        let mut output = String::new();
        
        output.push_str(&format!("!Type:{}\n", Self::account_type_to_qif(&account.account_type)));
        
        for transaction in transactions {
            output.push_str(&Self::export_transaction(transaction)?);
        }
        
        Ok(output)
    }
    
    fn export_transaction(transaction: &Transaction) -> Result<String> {
        let mut output = String::new();
        
        // Date
        output.push_str(&format!("D{}\n", transaction.date.format("%m/%d/%Y")));
        
        // Amount (negative for debits in QIF)
        let amount = match transaction.transaction_type {
            TransactionType::Debit => -transaction.amount,
            _ => transaction.amount,
        };
        output.push_str(&format!("T{}\n", amount));
        
        // Payee
        if let Some(ref payee) = transaction.payee {
            output.push_str(&format!("P{}\n", payee));
        }
        
        // Category
        if let Some(ref category) = transaction.category {
            output.push_str(&format!("L{}\n", category));
        }
        
        // Memo
        if let Some(ref memo) = transaction.memo {
            output.push_str(&format!("M{}\n", memo));
        }
        
        // Cleared status
        if transaction.cleared {
            output.push_str("C*\n");
        }
        
        output.push_str("^\n");
        
        Ok(output)
    }
    
    fn account_type_to_qif(account_type: &AccountType) -> &str {
        match account_type {
            AccountType::Checking => "Bank",
            AccountType::Savings => "Bank",
            AccountType::CreditCard => "CCard",
            AccountType::Investment => "Invst",
            AccountType::Cash => "Cash",
            AccountType::Liability => "Liability", 
            AccountType::Asset => "Asset",
            AccountType::Other(_) => "Bank",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const SAMPLE_QIF: &str = r#"!Account
NChecking Account
TBank
^
!Type:Bank
D12/1/2023
T-50.00
PGrocery Store
LGroceries
MWeekly shopping
C*
^
D12/2/2023
T1000.00
PPaycheck
LSalary
MMonthly salary
^
"#;

    #[test]
    fn test_qif_import() {
        let result = QifImporter::parse_qif_content(SAMPLE_QIF);
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.accounts.len(), 1);
        assert_eq!(data.transactions.len(), 2);
        
        let account = &data.accounts[0];
        assert_eq!(account.name, "Checking Account");
        assert_eq!(account.account_type, AccountType::Checking);
        
        // Check transactions
        let transactions = data.get_account_transactions(&account.id);
        assert_eq!(transactions.len(), 2);
        
        // First transaction (debit)
        let debit_tx = transactions.iter()
            .find(|t| t.amount == dec!(50.00) && matches!(t.transaction_type, TransactionType::Debit))
            .expect("Debit transaction not found");
        
        assert_eq!(debit_tx.payee, Some("Grocery Store".to_string()));
        assert_eq!(debit_tx.category, Some("Groceries".to_string()));
        assert_eq!(debit_tx.memo, Some("Weekly shopping".to_string()));
        assert!(debit_tx.cleared);
        
        // Second transaction (credit)
        let credit_tx = transactions.iter()
            .find(|t| t.amount == dec!(1000.00) && matches!(t.transaction_type, TransactionType::Credit))
            .expect("Credit transaction not found");
        
        assert_eq!(credit_tx.payee, Some("Paycheck".to_string()));
        assert_eq!(credit_tx.category, Some("Salary".to_string()));
        assert!(!credit_tx.cleared);
    }

    #[test]
    fn test_qif_export() {
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
            chrono::Utc::now(),
            dec!(100.00),
            "Test Transaction".to_string(),
            TransactionType::Debit,
        );
        transaction.payee = Some("Test Payee".to_string());
        transaction.category = Some("Test Category".to_string());
        transaction.cleared = true;
        
        data.add_transaction(transaction);
        
        let result = QifExporter::export_to_string(&data);
        assert!(result.is_ok());
        
        let qif_content = result.unwrap();
        assert!(qif_content.contains("!Account"));
        assert!(qif_content.contains("NTest Account"));
        assert!(qif_content.contains("TBank"));
        assert!(qif_content.contains("!Type:Bank"));
        assert!(qif_content.contains("PTest Payee"));
        assert!(qif_content.contains("LTest Category"));
        assert!(qif_content.contains("C*"));
    }

    #[test]
    fn test_qif_date_parsing() {
        let test_dates = [
            ("12/1/2023", true),
            ("1/15/23", true),
            ("12-01-2023", true),
            ("2023-12-01", true),
            ("invalid", false),
        ];
        
        for (date_str, should_succeed) in test_dates {
            let result = QifImporter::parse_qif_date(date_str);
            assert_eq!(result.is_ok(), should_succeed, "Date: {}", date_str);
        }
    }

    #[test]
    fn test_round_trip() {
        // Import QIF, then export, then import again
        let original_data = QifImporter::parse_qif_content(SAMPLE_QIF).unwrap();
        let exported_qif = QifExporter::export_to_string(&original_data).unwrap();
        let reimported_data = QifImporter::parse_qif_content(&exported_qif).unwrap();
        
        assert_eq!(original_data.accounts.len(), reimported_data.accounts.len());
        assert_eq!(original_data.transactions.len(), reimported_data.transactions.len());
    }
}