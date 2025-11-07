use anyhow::Result;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Datelike};
use rust_decimal::Decimal;
use crate::data::{FinancialData, Transaction, TransactionType};

/// Financial analysis engine
pub struct AnalysisEngine;

/// Monthly spending report
#[derive(Debug, Clone)]
pub struct MonthlyReport {
    pub year: i32,
    pub month: u32,
    pub total_income: Decimal,
    pub total_expenses: Decimal,
    pub net_income: Decimal,
    pub category_breakdown: HashMap<String, Decimal>,
    pub transaction_count: usize,
}

/// Category analysis
#[derive(Debug, Clone)]
pub struct CategoryAnalysis {
    pub category: String,
    pub total_amount: Decimal,
    pub transaction_count: usize,
    pub average_amount: Decimal,
    pub percentage_of_total: Decimal,
}

/// Spending trend analysis
#[derive(Debug, Clone)]
pub struct SpendingTrend {
    pub category: String,
    pub monthly_amounts: Vec<(String, Decimal)>, // (Month, Amount)
    pub trend_direction: TrendDirection,
    pub average_monthly: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

impl AnalysisEngine {
    /// Generate monthly report for a specific month
    pub fn generate_monthly_report(
        data: &FinancialData,
        year: i32,
        month: u32,
    ) -> Result<MonthlyReport> {
        let month_transactions: Vec<&Transaction> = data
            .transactions
            .iter()
            .filter(|t| {
                let date = t.date;
                date.year() == year && date.month() == month
            })
            .collect();

        let mut total_income = Decimal::ZERO;
        let mut total_expenses = Decimal::ZERO;
        let mut category_breakdown = HashMap::new();

        for transaction in &month_transactions {
            let amount = transaction.amount;
            
            match transaction.transaction_type {
                TransactionType::Credit => {
                    total_income += amount;
                }
                TransactionType::Debit => {
                    total_expenses += amount;
                }
                _ => {} // Handle other types as needed
            }

            // Add to category breakdown
            if let Some(ref category) = transaction.category {
                let current = category_breakdown.get(category).unwrap_or(&Decimal::ZERO);
                category_breakdown.insert(category.clone(), current + amount);
            }
        }

        let net_income = total_income - total_expenses;

        Ok(MonthlyReport {
            year,
            month,
            total_income,
            total_expenses,
            net_income,
            category_breakdown,
            transaction_count: month_transactions.len(),
        })
    }

    /// Analyze spending by categories
    pub fn analyze_categories(data: &FinancialData) -> Result<Vec<CategoryAnalysis>> {
        let mut category_totals: HashMap<String, (Decimal, usize)> = HashMap::new();
        let mut total_spending = Decimal::ZERO;

        // Calculate totals per category
        for transaction in &data.transactions {
            if matches!(transaction.transaction_type, TransactionType::Debit) {
                let category = transaction.category
                    .as_ref()
                    .unwrap_or(&"Uncategorized".to_string())
                    .clone();

                let (current_amount, current_count) = category_totals
                    .get(&category)
                    .unwrap_or(&(Decimal::ZERO, 0));

                category_totals.insert(
                    category,
                    (current_amount + transaction.amount, current_count + 1),
                );

                total_spending += transaction.amount;
            }
        }

        // Convert to analysis results
        let mut results = Vec::new();
        for (category, (total_amount, count)) in category_totals {
            let average_amount = if count > 0 {
                total_amount / Decimal::from(count)
            } else {
                Decimal::ZERO
            };

            let percentage_of_total = if total_spending > Decimal::ZERO {
                (total_amount / total_spending) * Decimal::from(100)
            } else {
                Decimal::ZERO
            };

            results.push(CategoryAnalysis {
                category,
                total_amount,
                transaction_count: count,
                average_amount,
                percentage_of_total,
            });
        }

        // Sort by total amount (descending)
        results.sort_by(|a, b| b.total_amount.cmp(&a.total_amount));

        Ok(results)
    }

    /// Analyze spending trends over time
    pub fn analyze_spending_trends(
        data: &FinancialData,
        months_back: usize,
    ) -> Result<Vec<SpendingTrend>> {
        let now = Utc::now();
        let mut trends = HashMap::new();

        // Get all unique categories
        let categories: std::collections::HashSet<String> = data
            .transactions
            .iter()
            .filter_map(|t| t.category.as_ref())
            .cloned()
            .collect();

        for category in categories {
            let mut monthly_amounts = Vec::new();

            // Calculate spending for each of the last N months
            for i in 0..months_back {
                let target_date = now - chrono::Duration::days((i * 30) as i64);
                let year = target_date.year();
                let month = target_date.month();

                let month_total: Decimal = data
                    .transactions
                    .iter()
                    .filter(|t| {
                        let t_date = t.date;
                        t_date.year() == year
                            && t_date.month() == month
                            && t.category.as_ref() == Some(&category)
                            && matches!(t.transaction_type, TransactionType::Debit)
                    })
                    .map(|t| t.amount)
                    .sum();

                monthly_amounts.push((
                    format!("{}-{:02}", year, month),
                    month_total,
                ));
            }

            // Reverse to get chronological order
            monthly_amounts.reverse();

            // Calculate trend direction
            let trend_direction = Self::calculate_trend_direction(&monthly_amounts);

            // Calculate average
            let total: Decimal = monthly_amounts.iter().map(|(_, amount)| amount).sum();
            let average_monthly = if !monthly_amounts.is_empty() {
                total / Decimal::from(monthly_amounts.len())
            } else {
                Decimal::ZERO
            };

            trends.insert(
                category.clone(),
                SpendingTrend {
                    category,
                    monthly_amounts,
                    trend_direction,
                    average_monthly,
                },
            );
        }

        Ok(trends.into_values().collect())
    }

    fn calculate_trend_direction(monthly_amounts: &[(String, Decimal)]) -> TrendDirection {
        if monthly_amounts.len() < 2 {
            return TrendDirection::Stable;
        }

        let first_half_avg = Self::calculate_average(&monthly_amounts[..monthly_amounts.len() / 2]);
        let second_half_avg = Self::calculate_average(&monthly_amounts[monthly_amounts.len() / 2..]);

        let difference = second_half_avg - first_half_avg;
        let threshold = first_half_avg * Decimal::from_f32(0.1).unwrap_or(Decimal::ZERO); // 10% threshold

        if difference > threshold {
            TrendDirection::Increasing
        } else if difference < -threshold {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_average(amounts: &[(String, Decimal)]) -> Decimal {
        if amounts.is_empty() {
            return Decimal::ZERO;
        }

        let sum: Decimal = amounts.iter().map(|(_, amount)| amount).sum();
        sum / Decimal::from(amounts.len())
    }

    /// Detect unusual spending patterns
    pub fn detect_anomalies(data: &FinancialData) -> Result<Vec<&Transaction>> {
        let mut anomalies = Vec::new();

        // Calculate average transaction amount per category
        let mut category_stats: HashMap<String, (Decimal, usize)> = HashMap::new();

        for transaction in &data.transactions {
            if matches!(transaction.transaction_type, TransactionType::Debit) {
                let category = transaction.category
                    .as_ref()
                    .unwrap_or(&"Uncategorized".to_string());

                let (total, count) = category_stats.get(category).unwrap_or(&(Decimal::ZERO, 0));
                category_stats.insert(category.clone(), (total + transaction.amount, count + 1));
            }
        }

        // Find transactions that are significantly above average for their category
        for transaction in &data.transactions {
            if matches!(transaction.transaction_type, TransactionType::Debit) {
                let category = transaction.category
                    .as_ref()
                    .unwrap_or(&"Uncategorized".to_string());

                if let Some((total, count)) = category_stats.get(category) {
                    if *count > 0 {
                        let average = total / Decimal::from(*count);
                        let threshold = average * Decimal::from(3); // 3x average threshold

                        if transaction.amount > threshold {
                            anomalies.push(transaction);
                        }
                    }
                }
            }
        }

        Ok(anomalies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{Account, AccountType, FinancialData};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    fn create_test_data() -> FinancialData {
        let mut data = FinancialData::new();

        let account = Account::new(
            "Test Account".to_string(),
            AccountType::Checking,
            dec!(1000.00),
            "USD".to_string(),
        );
        let account_id = account.id;
        data.add_account(account);

        // Add some test transactions
        let transactions = vec![
            // January transactions
            (
                chrono::Utc
                    .with_ymd_and_hms(2024, 1, 15, 0, 0, 0)
                    .unwrap(),
                dec!(500.00),
                TransactionType::Debit,
                "Groceries",
            ),
            (
                chrono::Utc
                    .with_ymd_and_hms(2024, 1, 20, 0, 0, 0)
                    .unwrap(),
                dec!(200.00),
                TransactionType::Debit,
                "Gas",
            ),
            (
                chrono::Utc
                    .with_ymd_and_hms(2024, 1, 25, 0, 0, 0)
                    .unwrap(),
                dec!(3000.00),
                TransactionType::Credit,
                "Salary",
            ),
            // February transactions
            (
                chrono::Utc
                    .with_ymd_and_hms(2024, 2, 10, 0, 0, 0)
                    .unwrap(),
                dec!(600.00),
                TransactionType::Debit,
                "Groceries",
            ),
            (
                chrono::Utc
                    .with_ymd_and_hms(2024, 2, 15, 0, 0, 0)
                    .unwrap(),
                dec!(250.00),
                TransactionType::Debit,
                "Gas",
            ),
        ];

        for (date, amount, tx_type, category) in transactions {
            let mut transaction = Transaction::new(account_id, date, amount, "Test".to_string(), tx_type);
            transaction.category = Some(category.to_string());
            data.add_transaction(transaction);
        }

        data
    }

    #[test]
    fn test_monthly_report() {
        let data = create_test_data();
        let report = AnalysisEngine::generate_monthly_report(&data, 2024, 1).unwrap();

        assert_eq!(report.year, 2024);
        assert_eq!(report.month, 1);
        assert_eq!(report.total_income, dec!(3000.00));
        assert_eq!(report.total_expenses, dec!(700.00)); // 500 + 200
        assert_eq!(report.net_income, dec!(2300.00)); // 3000 - 700
        assert_eq!(report.transaction_count, 3);

        // Check category breakdown
        assert_eq!(report.category_breakdown.get("Groceries"), Some(&dec!(500.00)));
        assert_eq!(report.category_breakdown.get("Gas"), Some(&dec!(200.00)));
    }

    #[test]
    fn test_category_analysis() {
        let data = create_test_data();
        let analysis = AnalysisEngine::analyze_categories(&data).unwrap();

        // Should have 2 categories (Groceries and Gas)
        assert_eq!(analysis.len(), 2);

        // Groceries should be first (higher total: 500 + 600 = 1100)
        let groceries = &analysis[0];
        assert_eq!(groceries.category, "Groceries");
        assert_eq!(groceries.total_amount, dec!(1100.00));
        assert_eq!(groceries.transaction_count, 2);
        assert_eq!(groceries.average_amount, dec!(550.00));

        // Gas should be second (450 total)
        let gas = &analysis[1];
        assert_eq!(gas.category, "Gas");
        assert_eq!(gas.total_amount, dec!(450.00));
        assert_eq!(gas.transaction_count, 2);
        assert_eq!(gas.average_amount, dec!(225.00));
    }

    #[test]
    fn test_trend_direction_calculation() {
        let increasing = vec![
            ("2024-01".to_string(), dec!(100.00)),
            ("2024-02".to_string(), dec!(150.00)),
            ("2024-03".to_string(), dec!(200.00)),
            ("2024-04".to_string(), dec!(250.00)),
        ];
        assert_eq!(
            AnalysisEngine::calculate_trend_direction(&increasing),
            TrendDirection::Increasing
        );

        let decreasing = vec![
            ("2024-01".to_string(), dec!(250.00)),
            ("2024-02".to_string(), dec!(200.00)),
            ("2024-03".to_string(), dec!(150.00)),
            ("2024-04".to_string(), dec!(100.00)),
        ];
        assert_eq!(
            AnalysisEngine::calculate_trend_direction(&decreasing),
            TrendDirection::Decreasing
        );

        let stable = vec![
            ("2024-01".to_string(), dec!(200.00)),
            ("2024-02".to_string(), dec!(190.00)),
            ("2024-03".to_string(), dec!(210.00)),
            ("2024-04".to_string(), dec!(200.00)),
        ];
        assert_eq!(
            AnalysisEngine::calculate_trend_direction(&stable),
            TrendDirection::Stable
        );
    }

    #[test]
    fn test_anomaly_detection() {
        let mut data = create_test_data();

        // Add an unusually large transaction
        let account_id = data.accounts[0].id;
        let mut large_transaction = Transaction::new(
            account_id,
            Utc::now(),
            dec!(5000.00), // Much larger than normal groceries
            "Huge grocery bill".to_string(),
            TransactionType::Debit,
        );
        large_transaction.category = Some("Groceries".to_string());
        data.add_transaction(large_transaction);

        let anomalies = AnalysisEngine::detect_anomalies(&data).unwrap();

        // Should detect the large transaction as an anomaly
        assert_eq!(anomalies.len(), 1);
        assert_eq!(anomalies[0].amount, dec!(5000.00));
    }
}