use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::data::{Transaction, TransactionType};

/// Utility functions for financial calculations and data processing
///
/// Calculate compound interest
pub fn calculate_compound_interest(
    principal: Decimal,
    annual_rate: Decimal,
    compounds_per_year: u32,
    years: u32,
) -> Decimal {
    let rate_per_compound = annual_rate / Decimal::from(compounds_per_year);
    let total_compounds = compounds_per_year * years;
    
    let base = Decimal::ONE + rate_per_compound;
    
    // Manual exponentiation for Decimal
    let mut result = principal;
    for _ in 0..total_compounds {
        result *= base;
    }
    
    result
}

/// Calculate simple moving average for a series of values
pub fn simple_moving_average(values: &[Decimal], window_size: usize) -> Vec<Decimal> {
    if window_size == 0 || window_size > values.len() {
        return Vec::new();
    }
    
    let mut averages = Vec::new();
    
    for i in window_size - 1..values.len() {
        let sum: Decimal = values[i - (window_size - 1)..=i].iter().sum();
        averages.push(sum / Decimal::from(window_size));
    }
    
    averages
}

/// Format currency amount for display
pub fn format_currency(amount: Decimal, currency: &str) -> String {
    match currency {
        "USD" => format!("${:.2}", amount),
        "EUR" => format!("€{:.2}", amount),
        "GBP" => format!("£{:.2}", amount),
        _ => format!("{} {:.2}", currency, amount),
    }
}

/// Parse currency string to Decimal
pub fn parse_currency(input: &str) -> Result<Decimal> {
    let cleaned = input
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect::<String>();
    
    cleaned.parse::<Decimal>()
        .map_err(|e| anyhow::anyhow!("Failed to parse currency '{}': {}", input, e))
}

/// Calculate percentage change between two values
pub fn percentage_change(old_value: Decimal, new_value: Decimal) -> Decimal {
    if old_value.is_zero() {
        return Decimal::ZERO;
    }
    
    ((new_value - old_value) / old_value) * Decimal::from(100)
}

/// Round to nearest currency unit (typically 0.01)
pub fn round_currency(amount: Decimal) -> Decimal {
    amount.round_dp(2)
}

/// Calculate net worth from assets and liabilities
pub fn calculate_net_worth(assets: &[Decimal], liabilities: &[Decimal]) -> Decimal {
    let total_assets: Decimal = assets.iter().sum();
    let total_liabilities: Decimal = liabilities.iter().sum();
    total_assets - total_liabilities
}

/// Date utility functions
pub mod date_utils {
    use super::*;
    use chrono::{Datelike, Timelike};

    /// Get the start of the month for a given date
    pub fn start_of_month(date: DateTime<Utc>) -> DateTime<Utc> {
        date.with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
    }

    /// Get the end of the month for a given date
    pub fn end_of_month(date: DateTime<Utc>) -> DateTime<Utc> {
        let next_month = if date.month() == 12 {
            date.with_year(date.year() + 1).unwrap().with_month(1).unwrap()
        } else {
            date.with_month(date.month() + 1).unwrap()
        };
        
        next_month.with_day(1).unwrap() - Duration::seconds(1)
    }

    /// Check if two dates are in the same month
    pub fn same_month(date1: DateTime<Utc>, date2: DateTime<Utc>) -> bool {
        date1.year() == date2.year() && date1.month() == date2.month()
    }

    /// Get the number of days between two dates
    pub fn days_between(start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
        (end - start).num_days()
    }

    /// Get all months between two dates
    pub fn months_between(start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<(i32, u32)> {
        let mut months = Vec::new();
        let mut current = start_of_month(start);
        let end_month = start_of_month(end);
        
        while current <= end_month {
            months.push((current.year(), current.month()));
            
            if current.month() == 12 {
                current = current.with_year(current.year() + 1).unwrap().with_month(1).unwrap();
            } else {
                current = current.with_month(current.month() + 1).unwrap();
            }
        }
        
        months
    }
}

/// Transaction utility functions
pub mod transaction_utils {
    use super::*;
    use chrono::Datelike;

    /// Group transactions by month
    pub fn group_by_month(
        transactions: &[Transaction],
    ) -> HashMap<(i32, u32), Vec<&Transaction>> {
        let mut grouped = HashMap::new();
        
        for transaction in transactions {
            let key = (transaction.date.year(), transaction.date.month());
            grouped.entry(key).or_insert_with(Vec::new).push(transaction);
        }
        
        grouped
    }

    /// Group transactions by category
    pub fn group_by_category(
        transactions: &[Transaction],
    ) -> HashMap<String, Vec<&Transaction>> {
        let mut grouped = HashMap::new();
        
        for transaction in transactions {
            let category = transaction
                .category
                .as_ref()
                .unwrap_or(&"Uncategorized".to_string())
                .clone();
            grouped.entry(category).or_insert_with(Vec::new).push(transaction);
        }
        
        grouped
    }

    /// Filter transactions by date range
    pub fn filter_by_date_range(
        transactions: &[Transaction],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&Transaction> {
        transactions
            .iter()
            .filter(|t| t.date >= start && t.date <= end)
            .collect()
    }

    /// Calculate total for transaction type
    pub fn total_by_type(
        transactions: &[Transaction],
        transaction_type: TransactionType,
    ) -> Decimal {
        transactions
            .iter()
            .filter(|t| std::mem::discriminant(&t.transaction_type) == std::mem::discriminant(&transaction_type))
            .map(|t| t.amount)
            .sum()
    }
}

/// Validation utilities
pub mod validation {
    use super::*;
    use regex::Regex;

    /// Validate email address
    pub fn is_valid_email(email: &str) -> bool {
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        email_regex.is_match(email)
    }

    /// Validate account number (basic check)
    pub fn is_valid_account_number(account_number: &str) -> bool {
        !account_number.trim().is_empty() && account_number.len() >= 4
    }

    /// Validate currency code (ISO 4217-like)
    pub fn is_valid_currency_code(currency: &str) -> bool {
        currency.len() == 3 && currency.chars().all(|c| c.is_ascii_uppercase())
    }

    /// Validate transaction amount (must be positive)
    pub fn is_valid_amount(amount: Decimal) -> bool {
        amount > Decimal::ZERO
    }
}

/// File utilities
pub mod file_utils {
    use super::*;
    use std::path::{Path, PathBuf};

    /// Get file extension
    pub fn get_file_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Check if file is a QIF file
    pub fn is_qif_file(path: &Path) -> bool {
        matches!(get_file_extension(path).as_deref(), Some("qif"))
    }

    /// Generate backup filename
    pub fn generate_backup_filename(original: &Path) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let mut backup = original.to_path_buf();
        
        if let Some(stem) = original.file_stem() {
            if let Some(ext) = original.extension() {
                backup.set_file_name(format!(
                    "{}_{}.{}",
                    stem.to_string_lossy(),
                    timestamp,
                    ext.to_string_lossy()
                ));
            } else {
                backup.set_file_name(format!("{}_{}", stem.to_string_lossy(), timestamp));
            }
        }
        
        backup
    }

    /// Ensure directory exists
    pub async fn ensure_dir_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use chrono::{TimeZone, Datelike, Timelike};
    use uuid::Uuid;

    #[test]
    fn test_compound_interest() {
        let principal = dec!(1000.00);
        let annual_rate = dec!(0.05); // 5%
        let compounds_per_year = 12; // Monthly
        let years = 1;
        
        let result = calculate_compound_interest(principal, annual_rate, compounds_per_year, years);
        
        // Should be approximately 1051.16 for 5% compounded monthly
        assert!(result > dec!(1050.00) && result < dec!(1055.00));
    }

    #[test]
    fn test_simple_moving_average() {
        let values = vec![
            dec!(100), dec!(110), dec!(105), dec!(120), dec!(115)
        ];
        
        let averages = simple_moving_average(&values, 3);
        
        assert_eq!(averages.len(), 3);
        assert_eq!(averages[0], dec!(105.00)); // (100 + 110 + 105) / 3
        assert_eq!(averages[1].round_dp(2), dec!(111.67)); // (110 + 105 + 120) / 3
        assert_eq!(averages[2].round_dp(2), dec!(113.33)); // (105 + 120 + 115) / 3
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(dec!(123.45), "USD"), "$123.45");
        assert_eq!(format_currency(dec!(67.89), "EUR"), "€67.89");
        assert_eq!(format_currency(dec!(100.00), "GBP"), "£100.00");
        assert_eq!(format_currency(dec!(50.00), "CAD"), "CAD 50.00");
    }

    #[test]
    fn test_parse_currency() {
        assert_eq!(parse_currency("$123.45").unwrap(), dec!(123.45));
        assert_eq!(parse_currency("€67.89").unwrap(), dec!(67.89));
        assert_eq!(parse_currency("1,234.56").unwrap(), dec!(1234.56));
        assert_eq!(parse_currency("-100.00").unwrap(), dec!(-100.00));
        assert!(parse_currency("invalid").is_err());
    }

    #[test]
    fn test_percentage_change() {
        assert_eq!(percentage_change(dec!(100), dec!(110)), dec!(10));
        assert_eq!(percentage_change(dec!(100), dec!(90)), dec!(-10));
        assert_eq!(percentage_change(dec!(0), dec!(100)), dec!(0)); // Avoid division by zero
    }

    #[test]
    fn test_net_worth_calculation() {
        let assets = vec![dec!(100000), dec!(50000), dec!(25000)]; // House, savings, car
        let liabilities = vec![dec!(80000), dec!(15000)]; // Mortgage, car loan
        
        let net_worth = calculate_net_worth(&assets, &liabilities);
        assert_eq!(net_worth, dec!(80000)); // 175000 - 95000
    }

    #[test]
    fn test_date_utils() {
        use date_utils::*;
        
        let date = Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 45).unwrap();
        
        let month_start = start_of_month(date);
        assert_eq!(month_start.day(), 1);
        assert_eq!(month_start.hour(), 0);
        assert_eq!(month_start.minute(), 0);
        
        let same_month_date = Utc.with_ymd_and_hms(2024, 6, 20, 10, 0, 0).unwrap();
        let different_month_date = Utc.with_ymd_and_hms(2024, 7, 1, 10, 0, 0).unwrap();
        
        assert!(same_month(date, same_month_date));
        assert!(!same_month(date, different_month_date));
    }

    #[test]
    fn test_transaction_utils() {
        use transaction_utils::*;
        
        let account_id = Uuid::new_v4();
        
        let transactions = vec![
            Transaction::new(
                account_id,
                Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap(),
                dec!(100),
                "Test 1".to_string(),
                TransactionType::Debit,
            ),
            Transaction::new(
                account_id,
                Utc.with_ymd_and_hms(2024, 2, 10, 0, 0, 0).unwrap(),
                dec!(200),
                "Test 2".to_string(),
                TransactionType::Credit,
            ),
        ];
        
        let grouped = group_by_month(&transactions);
        assert_eq!(grouped.len(), 2);
        assert!(grouped.contains_key(&(2024, 1)));
        assert!(grouped.contains_key(&(2024, 2)));
        
        let debit_total = total_by_type(&transactions, TransactionType::Debit);
        assert_eq!(debit_total, dec!(100));
        
        let credit_total = total_by_type(&transactions, TransactionType::Credit);
        assert_eq!(credit_total, dec!(200));
    }

    #[test]
    fn test_validation() {
        use validation::*;
        
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid-email"));
        
        assert!(is_valid_account_number("1234567890"));
        assert!(!is_valid_account_number("123")); // Too short
        assert!(!is_valid_account_number("")); // Empty
        
        assert!(is_valid_currency_code("USD"));
        assert!(is_valid_currency_code("EUR"));
        assert!(!is_valid_currency_code("usd")); // Lowercase
        assert!(!is_valid_currency_code("US")); // Too short
        
        assert!(is_valid_amount(dec!(100.00)));
        assert!(!is_valid_amount(dec!(0.00)));
        assert!(!is_valid_amount(dec!(-50.00)));
    }

    #[test]
    fn test_file_utils() {
        use file_utils::*;
        use std::path::Path;
        
        let qif_path = Path::new("test.qif");
        let txt_path = Path::new("test.txt");
        
        assert_eq!(get_file_extension(qif_path), Some("qif".to_string()));
        assert_eq!(get_file_extension(txt_path), Some("txt".to_string()));
        
        assert!(is_qif_file(qif_path));
        assert!(!is_qif_file(txt_path));
        
        let backup = generate_backup_filename(qif_path);
        assert!(backup.file_name().unwrap().to_string_lossy().starts_with("test_"));
        assert!(backup.file_name().unwrap().to_string_lossy().ends_with(".qif"));
    }
}