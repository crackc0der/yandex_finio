//! Упрощённый XML (не CAMT!): <Statement><AccountId/><Entries>...</Entries></Statement>

use crate::{error::{FinioError, Result}, model::{Statement, Entry, Balance, DebitCredit}};
use chrono::NaiveDate;
use quick_xml::{de::from_reader, se::to_string};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

use rust_decimal::Decimal;

#[derive(Serialize, Deserialize, Debug)]
struct XmlEntry {
    booking_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_date: Option<String>,
    #[serde(with = "rust_decimal::serde::str")]
    amount: Decimal,
    currency: String,
    dc: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct XmlBalance {
    date: String,
    #[serde(with = "rust_decimal::serde::str")]
    amount: Decimal,
    currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct XmlStatement {
    #[serde(skip_serializing_if = "Option::is_none")]
    statement_id: Option<String>,
    account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    opening_balance: Option<XmlBalance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    closing_balance: Option<XmlBalance>,
    entries: Vec<XmlEntry>,
}


fn parse_dc(s: &str) -> Result<DebitCredit> {
    match s {
        "D" => Ok(DebitCredit::Debit),
        "C" => Ok(DebitCredit::Credit),
        _ => Err(FinioError::Parse(format!("dc {s}"))),
    }
}

pub struct SimpleXml;

// parse_dc оставить как было

impl crate::traits::ReadFormat for SimpleXml {
    fn read<R: BufRead>(r: R) -> Result<Statement> {
        use crate::error::FinioError;
        let x: XmlStatement = from_reader(r).map_err(|e| FinioError::Xml(format!("{e}")))?;

        // helper, чтобы не дублировать код
        fn parse_xml_balance(b: XmlBalance) -> Result<Balance> {
            Ok(Balance {
                date: chrono::NaiveDate::parse_from_str(&b.date, "%Y-%m-%d")
                    .map_err(|e| FinioError::Parse(format!("{e}")))?,
                amount: b.amount,
                currency: b.currency,
            })
        }

        // Option<Result<Balance>> -> Result<Option<Balance>>
        let opening: Option<Balance> = x.opening_balance
            .map(parse_xml_balance)
            .transpose()?;

        let closing: Option<Balance> = x.closing_balance
            .map(parse_xml_balance)
            .transpose()?;

        let mut entries = Vec::with_capacity(x.entries.len());
        for e in x.entries {
            let booking_date = NaiveDate::parse_from_str(&e.booking_date, "%Y-%m-%d")
                .map_err(|e| FinioError::Parse(format!("{e}")))?;
            let value_date = match e.value_date {
                Some(v) => Some(NaiveDate::parse_from_str(&v, "%Y-%m-%d")
                    .map_err(|e| FinioError::Parse(format!("{e}")))?),
                None => None,
            };
            let amount = e.amount;
            let dc = parse_dc(&e.dc)?;

            entries.push(Entry {
                booking_date,
                value_date,
                amount,
                currency: e.currency,
                dc,
                description: e.description,
                reference: e.reference,
            });
        }

        Ok(Statement {
            statement_id: x.statement_id,
            account_id: x.account_id,
            opening_balance: opening,
            closing_balance: closing,
            entries,
        })
    }
}


impl crate::traits::WriteFormat for SimpleXml {
    fn write<W: Write>(mut w: W, st: &Statement) -> Result<()> {
        let opening = st.opening_balance.as_ref().map(|b| XmlBalance {
            date: b.date.format("%Y-%m-%d").to_string(),
            amount: b.amount,                         // ⬅️ Decimal
            currency: b.currency.clone(),
        });

        let closing = st.closing_balance.as_ref().map(|b| XmlBalance {
            date: b.date.format("%Y-%m-%d").to_string(),
            amount: b.amount,                         // ⬅️ Decimal
            currency: b.currency.clone(),
        });

        let entries = st.entries.iter().map(|e| XmlEntry {
            booking_date: e.booking_date.format("%Y-%m-%d").to_string(),
            value_date: e.value_date.map(|d| d.format("%Y-%m-%d").to_string()),
            amount: e.amount,                         // ⬅️ Decimal
            currency: e.currency.clone(),
            dc: match e.dc { DebitCredit::Debit=>"D".into(), DebitCredit::Credit=>"C".into() },
            description: e.description.clone(),
            reference: e.reference.clone(),
        }).collect();

        let x = XmlStatement {
            statement_id: st.statement_id.clone(),
            account_id: st.account_id.clone(),
            opening_balance: opening,
            closing_balance: closing,
            entries,
        };

        // 🔧 сериализуем в String и пишем в io::Write
        let s = to_string(&x).map_err(|e| FinioError::Xml(format!("{e}")))?;
        w.write_all(s.as_bytes())?;
        Ok(())
    }

}

