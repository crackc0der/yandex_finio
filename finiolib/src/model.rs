//! Доменные модели — единый «нормализованный» слой между форматами.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DebitCredit {
    Debit,
    Credit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: Decimal,
    pub currency: String,
    pub dc: DebitCredit,
    pub description: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Balance {
    pub date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statement {
    pub statement_id: Option<String>,
    pub account_id: String,
    pub opening_balance: Option<Balance>,
    pub closing_balance: Option<Balance>,
    pub entries: Vec<Entry>,
}
