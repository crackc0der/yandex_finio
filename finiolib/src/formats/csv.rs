//! Простой CSV: заголовки:
//! booking_date,value_date,amount,currency,dc,description,reference,account_id,opening_amount,opening_currency,opening_date,closing_amount,closing_currency,closing_date

use crate::{error::{FinioError, Result}, model::{Balance, DebitCredit, Entry, Statement}};
use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};
use rust_decimal::Decimal;
use std::io::{BufRead, Write};

#[derive(serde::Deserialize)]
struct CsvRow {
    booking_date: String,
    value_date: Option<String>,
    amount: String,
    currency: String,
    dc: String,
    description: String,
    reference: Option<String>,
    account_id: String,

    opening_amount: Option<String>,
    opening_currency: Option<String>,
    opening_date: Option<String>,

    closing_amount: Option<String>,
    closing_currency: Option<String>,
    closing_date: Option<String>,
}

#[derive(serde::Serialize)]
struct CsvOutRow<'a> {
    booking_date: String,
    value_date: Option<String>,
    amount: String,
    currency: &'a str,
    dc: &'a str,
    description: &'a str,
    reference: &'a Option<String>,
    account_id: &'a str,

    opening_amount: Option<String>,
    opening_currency: Option<&'a str>,
    opening_date: Option<String>,

    closing_amount: Option<String>,
    closing_currency: Option<&'a str>,
    closing_date: Option<String>,
}

pub struct Csv;

impl crate::traits::ReadFormat for Csv {
    fn read<R: BufRead>(r: R) -> Result<Statement> {
        let mut rdr = ReaderBuilder::new().flexible(true).from_reader(r);
        let mut entries = Vec::new();
        let mut account_id = String::new();
        let mut opening: Option<Balance> = None;
        let mut closing: Option<Balance> = None;

        for rec in rdr.deserialize::<CsvRow>() {
            let row = rec?;
            if account_id.is_empty() {
                account_id = row.account_id.clone();
            }

            if opening.is_none() {
                if let (Some(a), Some(c), Some(d)) = (&row.opening_amount, &row.opening_currency, &row.opening_date) {
                    opening = Some(Balance {
                        amount: a.parse().map_err(|e| FinioError::Parse(format!("opening amount: {e}")))?,
                        currency: c.clone(),
                        date: NaiveDate::parse_from_str(d, "%Y-%m-%d")
                            .map_err(|e| FinioError::Parse(format!("opening date: {e}")))?,
                    });
                }
            }
            if closing.is_none() {
                if let (Some(a), Some(c), Some(d)) = (&row.closing_amount, &row.closing_currency, &row.closing_date) {
                    closing = Some(Balance {
                        amount: a.parse().map_err(|e| FinioError::Parse(format!("closing amount: {e}")))?,
                        currency: c.clone(),
                        date: NaiveDate::parse_from_str(d, "%Y-%m-%d")
                            .map_err(|e| FinioError::Parse(format!("closing date: {e}")))?,
                    });
                }
            }

            let dc = match row.dc.as_str() {
                "D" | "d" | "debit" => DebitCredit::Debit,
                "C" | "c" | "credit" => DebitCredit::Credit,
                other => return Err(FinioError::Parse(format!("unknown dc: {other}"))),
            };

            entries.push(Entry {
                booking_date: NaiveDate::parse_from_str(&row.booking_date, "%Y-%m-%d")
                    .map_err(|e| FinioError::Parse(format!("booking_date: {e}")))?,
                value_date: match row.value_date {
                    Some(v) => Some(NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                        .map_err(|e| FinioError::Parse(format!("value_date: {e}")))?),
                    None => None,
                },
                amount: row.amount.parse::<Decimal>()
                    .map_err(|e| FinioError::Parse(format!("amount: {e}")))?,
                currency: row.currency,
                dc,
                description: row.description,
                reference: row.reference,
            });
        }

        Ok(Statement {
            statement_id: None,
            account_id,
            opening_balance: opening,
            closing_balance: closing,
            entries,
        })
    }
}

impl crate::traits::WriteFormat for Csv {
    fn write<W: Write>(mut w: W, st: &Statement) -> Result<()> {
        let mut wrt = WriterBuilder::new().from_writer(&mut w);

        for e in &st.entries {
            let out = CsvOutRow {
                booking_date: e.booking_date.format("%Y-%m-%d").to_string(),
                value_date: e.value_date.map(|d| d.format("%Y-%m-%d").to_string()),
                amount: e.amount.to_string(),
                currency: &e.currency,
                dc: match e.dc { DebitCredit::Debit => "D", DebitCredit::Credit => "C" },
                description: &e.description,
                reference: &e.reference,
                account_id: &st.account_id,
                opening_amount: st.opening_balance.as_ref().map(|b| b.amount.to_string()),
                opening_currency: st.opening_balance.as_ref().map(|b| b.currency.as_str()),
                opening_date: st.opening_balance.as_ref().map(|b| b.date.format("%Y-%m-%d").to_string()),
                closing_amount: st.closing_balance.as_ref().map(|b| b.amount.to_string()),
                closing_currency: st.closing_balance.as_ref().map(|b| b.currency.as_str()),
                closing_date: st.closing_balance.as_ref().map(|b| b.date.format("%Y-%m-%d").to_string()),
            };
            wrt.serialize(out)?;
        }
        wrt.flush()?;
        Ok(())
    }
}
