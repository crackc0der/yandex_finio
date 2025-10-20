use crate::{
    error::{FinioError, Result},
    model::{Balance, DebitCredit, Entry, Statement},
};
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use rust_decimal::Decimal;
use std::io::BufRead;

/// Минимальный набор тегов: :20:, :25:, :60F:, :61:, :86:, :62F:
pub struct Mt940;

impl crate::traits::ReadFormat for Mt940 {
    fn read<R: BufRead>(r: R) -> Result<Statement> {
        let mut account_id = String::new();
        let mut statement_id: Option<String> = None;
        let mut opening: Option<Balance> = None;
        let mut closing: Option<Balance> = None;
        let mut entries: Vec<Entry> = Vec::new();

        // :61: YYMMDD [MMDD] [C|D] [CCY]? amount[,~~] [TX]? [REF]...
        let re_61 = Regex::new(
            r"^:61:(?P<val>\d{6})(?P<book>\d{4})?(?P<dc>[CD])(?P<ccy>[A-Z]{3})?(?P<amt>\d+,\d{0,2})(?P<tx>[A-Z]{3,4})?(?P<ref>[^\r\n]*)?.*$",
        )
            .map_err(|e| FinioError::Parse(e.to_string()))?;

        let mut last_entry_has_86 = false;

        for line in r.lines() {
            let line = line?;
            if line.starts_with(":20:") {
                statement_id = Some(line[4..].trim().to_string());
            } else if line.starts_with(":25:") {
                account_id = line[4..].trim().to_string();
            } else if line.starts_with(":60F:") {
                opening = parse_balance(&line[5..])?;
            } else if line.starts_with(":62F:") {
                closing = parse_balance(&line[5..])?;
            } else if line.starts_with(":61:") {
                let caps = re_61
                    .captures(&line)
                    .ok_or_else(|| FinioError::Parse("bad :61:".into()))?;

                // маленький хелпер для обязательных групп
                let req = |name: &str| {
                    caps.name(name)
                        .map(|m| m.as_str())
                        .ok_or_else(|| FinioError::Parse(format!(":61: missing {name}")))
                };

                let val = req("val")?;
                let book_opt = caps.name("book").map(|m| m.as_str());

                let dc = match req("dc")? {
                    "C" => DebitCredit::Credit,
                    "D" => DebitCredit::Debit,
                    other => return Err(FinioError::Parse(format!(":61: dc {other}"))),
                };

                // сумма (в :61: — без валюты)
                let amt = req("amt")?.replace(',', ".");
                let amount: Decimal = amt
                    .parse()
                    .map_err(|e| FinioError::Parse(format!("amount: {e}")))?;

                // даты
                let value_date = parse_mt_date(val)?;
                let booking_date = match book_opt {
                    Some(b) => parse_mt_book_date(value_date.year(), b)?,
                    None => value_date,
                };

                // валюта: если указана прямо в :61:, берём её; иначе — из opening_balance; иначе XXX
                let currency = if let Some(m) = caps.name("ccy") {
                    m.as_str().to_string()
                } else {
                    opening
                        .as_ref()
                        .map(|b| b.currency.clone())
                        .unwrap_or_else(|| "XXX".into())
                };

                // reference: хвост после кода операции, игнорируем пустой и NONREF
                let reference = caps
                    .name("ref")
                    .map(|m| m.as_str().trim().to_string())
                    .filter(|s| !s.is_empty() && s != "NONREF");

                entries.push(Entry {
                    booking_date,
                    value_date: Some(value_date),
                    amount,
                    currency,
                    dc,
                    description: String::new(),
                    reference,
                });
                last_entry_has_86 = false;
            } else if line.starts_with(":86:") {
                if let Some(last) = entries.last_mut() {
                    let text = line[4..].to_string();
                    if last.description.is_empty() {
                        last.description = text;
                    } else {
                        last.description.push(' ');
                        last.description.push_str(&text);
                    }
                    last_entry_has_86 = true;
                }
            } else if last_entry_has_86 && !line.starts_with(':') {
                // продолжение описания без нового тега
                if let Some(last) = entries.last_mut() {
                    last.description.push(' ');
                    last.description.push_str(line.trim());
                }
            }
        }

        Ok(Statement {
            statement_id,
            account_id,
            opening_balance: opening,
            closing_balance: closing,
            entries,
        })
    }
}

impl crate::traits::WriteFormat for Mt940 {
    fn write<W: std::io::Write>(mut w: W, st: &Statement) -> Result<()> {
        use std::fmt::Write as FmtWrite;
        let mut s = String::new();
        if let Some(id) = &st.statement_id {
            let _ = writeln!(s, ":20:{}", id);
        } else {
            let _ = writeln!(s, ":20:NOTPROVIDED");
        }
        let _ = writeln!(s, ":25:{}", st.account_id);

        if let Some(b) = &st.opening_balance {
            let _ = writeln!(
                s,
                ":60F:{}{}{}",
                if b.amount.is_sign_negative() { "D" } else { "C" },
                b.date.format("%y%m%d"),
                format_amount(&b.amount, &b.currency) // тут с валютой
            );
        }
        for e in &st.entries {
            let dc = match e.dc {
                DebitCredit::Debit => "D",
                DebitCredit::Credit => "C",
            };
            let val = e.value_date.unwrap_or(e.booking_date);

            // reference (если нет — пишем NONREF)
            let ref_str = e
                .reference
                .as_deref()
                .filter(|v| !v.is_empty())
                .unwrap_or("NONREF");

            // :61: YYMMDD MMDD D/C amount NTRF[REF]   — amount БЕЗ валюты
            let _ = writeln!(
                s,
                ":61:{}{}{}{}NTRF{}",
                val.format("%y%m%d"),
                e.booking_date.format("%m%d"),
                dc,
                format_amount_plain(&e.amount),
                ref_str
            );

            if !e.description.is_empty() {
                let _ = writeln!(s, ":86:{}", e.description);
            }
        }
        if let Some(b) = &st.closing_balance {
            let _ = writeln!(
                s,
                ":62F:{}{}{}",
                if b.amount.is_sign_negative() { "D" } else { "C" },
                b.date.format("%y%m%d"),
                format_amount(&b.amount, &b.currency) // тут с валютой
            );
        }
        w.write_all(s.as_bytes())?;
        Ok(())
    }
}

/// Парс баланса формата D/C + YYMMDD + CCY + amount
fn parse_balance(s: &str) -> Result<Option<Balance>> {
    if s.len() < 7 {
        return Ok(None);
    }
    let dc = &s[0..1];
    let date = &s[1..7];
    let rest = &s[7..];

    if rest.len() < 3 {
        return Ok(None);
    }
    let currency = &rest[0..3];
    let amt = &rest[3..].replace(',', ".");

    let amount: Decimal =
        amt.parse()
            .map_err(|e| FinioError::Parse(format!("balance amt: {e}")))?;
    let d = parse_mt_date(date)?;
    let signed = if dc == "D" { -amount } else { amount };
    Ok(Some(Balance {
        date: d,
        amount: signed,
        currency: currency.to_string(),
    }))
}

fn parse_mt_date(yy_mmdd: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(yy_mmdd, "%y%m%d").map_err(|e| FinioError::Parse(e.to_string()))
}

fn parse_mt_book_date(year: i32, mmdd: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(&format!("{year}{mmdd}"), "%Y%m%d")
        .map_err(|e| FinioError::Parse(e.to_string()))
}

fn format_amount(a: &rust_decimal::Decimal, ccy: &str) -> String {
    let mut s = a.abs().to_string();
    if let Some(dot) = s.find('.') {
        s.replace_range(dot..=dot, ",");
    }
    format!("{ccy}{s}")
}

fn format_amount_plain(a: &rust_decimal::Decimal) -> String {
    let mut s = a.abs().to_string();
    if let Some(dot) = s.find('.') {
        s.replace_range(dot..=dot, ",");
    }
    s
}
