use crate::{
    error::{FinioError, Result},
    model::{Balance, DebitCredit, Entry, Statement},
    traits::{ReadFormat, WriteFormat},
};
use chrono::NaiveDate;
use quick_xml::{
    events::{BytesDecl, BytesStart, BytesText, Event},
    Reader, Writer,
};
use rust_decimal::Decimal;
use std::io::{BufRead, Write};

pub struct Camt053;

/* ------------------------------- WRITE ---------------------------------- */

impl WriteFormat for Camt053 {
    fn write<W: Write>(mut w: W, st: &Statement) -> Result<()> {
        let mut wr = Writer::new_with_indent(&mut w, b' ', 2);

        wr.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(xml)?;

        let mut doc = BytesStart::new("Document");
        doc.push_attribute(("xmlns", "urn:iso:std:iso:20022:tech:xsd:camt.053.001.02"));
        wr.write_event(Event::Start(doc)).map_err(xml)?;

        wr.write_event(Event::Start(BytesStart::new("BkToCstmrStmt")))
            .map_err(xml)?;
        wr.write_event(Event::Start(BytesStart::new("Stmt"))).map_err(xml)?;

        // <Id>
        wr.write_event(Event::Start(BytesStart::new("Id"))).map_err(xml)?;
        let id = st
            .statement_id
            .clone()
            .unwrap_or_else(|| "NOTPROVIDED".into());
        wr.write_event(Event::Text(BytesText::new(&id))).map_err(xml)?;
        wr.write_event(Event::End(BytesStart::new("Id").to_end()))
            .map_err(xml)?;

        // <Acct><Id><IBAN>
        wr.write_event(Event::Start(BytesStart::new("Acct"))).map_err(xml)?;
        wr.write_event(Event::Start(BytesStart::new("Id"))).map_err(xml)?;
        wr.write_event(Event::Start(BytesStart::new("IBAN"))).map_err(xml)?;
        wr.write_event(Event::Text(BytesText::new(&st.account_id)))
            .map_err(xml)?;
        wr.write_event(Event::End(BytesStart::new("IBAN").to_end()))
            .map_err(xml)?;
        wr.write_event(Event::End(BytesStart::new("Id").to_end()))
            .map_err(xml)?;
        wr.write_event(Event::End(BytesStart::new("Acct").to_end()))
            .map_err(xml)?;

        if let Some(b) = &st.opening_balance {
            write_bal(&mut wr, "OPBD", b).map_err(xml)?;
        }
        if let Some(b) = &st.closing_balance {
            write_bal(&mut wr, "CLBD", b).map_err(xml)?;
        }

        for e in &st.entries {
            write_entry(&mut wr, e).map_err(xml)?;
        }

        wr.write_event(Event::End(BytesStart::new("Stmt").to_end()))
            .map_err(xml)?;
        wr.write_event(Event::End(BytesStart::new("BkToCstmrStmt").to_end()))
            .map_err(xml)?;
        wr.write_event(Event::End(BytesStart::new("Document").to_end()))
            .map_err(xml)?;
        Ok(())
    }
}

fn write_bal<W: Write>(
    wr: &mut Writer<W>,
    tp: &str,
    b: &Balance,
) -> std::result::Result<(), quick_xml::Error> {
    wr.write_event(Event::Start(BytesStart::new("Bal")))?;
    wr.write_event(Event::Start(BytesStart::new("Tp")))?;
    wr.write_event(Event::Start(BytesStart::new("CdOrPrtry")))?;
    wr.write_event(Event::Start(BytesStart::new("Cd")))?;
    wr.write_event(Event::Text(BytesText::new(tp)))?;
    wr.write_event(Event::End(BytesStart::new("Cd").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("CdOrPrtry").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("Tp").to_end()))?;

    let amt_str = b.amount.to_string();
    wr.write_event(Event::Start(
        BytesStart::new("Amt").with_attributes([("Ccy", b.currency.as_str())]),
    ))?;
    wr.write_event(Event::Text(BytesText::new(&amt_str)))?;
    wr.write_event(Event::End(BytesStart::new("Amt").to_end()))?;

    let d = b.date.format("%Y-%m-%d").to_string();
    wr.write_event(Event::Start(BytesStart::new("Dt")))?;
    wr.write_event(Event::Start(BytesStart::new("Dt")))?;
    wr.write_event(Event::Text(BytesText::new(&d)))?;
    wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;

    wr.write_event(Event::End(BytesStart::new("Bal").to_end()))?;
    Ok(())
}

fn write_entry<W: Write>(
    wr: &mut Writer<W>,
    e: &Entry,
) -> std::result::Result<(), quick_xml::Error> {
    wr.write_event(Event::Start(BytesStart::new("Ntry")))?;

    // <NtryRef>REF...</NtryRef> — если есть reference
    if let Some(ref r) = e.reference {
        if !r.is_empty() {
            wr.write_event(Event::Start(BytesStart::new("NtryRef")))?;
            wr.write_event(Event::Text(BytesText::new(r)))?;
            wr.write_event(Event::End(BytesStart::new("NtryRef").to_end()))?;
        }
    }

    // <Amt Ccy="...">...</Amt>
    let amt = e.amount.to_string();
    wr.write_event(Event::Start(
        BytesStart::new("Amt").with_attributes([("Ccy", e.currency.as_str())]),
    ))?;
    wr.write_event(Event::Text(BytesText::new(&amt)))?;
    wr.write_event(Event::End(BytesStart::new("Amt").to_end()))?;

    // <CdtDbtInd>CRDT|DBIT</CdtDbtInd>
    let ind = match e.dc {
        DebitCredit::Credit => "CRDT",
        DebitCredit::Debit => "DBIT",
    };
    wr.write_event(Event::Start(BytesStart::new("CdtDbtInd")))?;
    wr.write_event(Event::Text(BytesText::new(ind)))?;
    wr.write_event(Event::End(BytesStart::new("CdtDbtInd").to_end()))?;

    // <ValDt><Dt>YYYY-MM-DD</Dt></ValDt>
    if let Some(vd) = e.value_date {
        let vd = vd.format("%Y-%m-%d").to_string();
        wr.write_event(Event::Start(BytesStart::new("ValDt")))?;
        wr.write_event(Event::Start(BytesStart::new("Dt")))?;
        wr.write_event(Event::Text(BytesText::new(&vd)))?;
        wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;
        wr.write_event(Event::End(BytesStart::new("ValDt").to_end()))?;
    }

    // <BookgDt><Dt>YYYY-MM-DD</Dt></BookgDt>
    let bd = e.booking_date.format("%Y-%m-%d").to_string();
    wr.write_event(Event::Start(BytesStart::new("BookgDt")))?;
    wr.write_event(Event::Start(BytesStart::new("Dt")))?;
    wr.write_event(Event::Text(BytesText::new(&bd)))?;
    wr.write_event(Event::End(BytesStart::new("Dt").to_end()))?;
    wr.write_event(Event::End(BytesStart::new("BookgDt").to_end()))?;

    // <AddtlNtryInf>...</AddtlNtryInf>
    if !e.description.is_empty() {
        wr.write_event(Event::Start(BytesStart::new("AddtlNtryInf")))?;
        wr.write_event(Event::Text(BytesText::new(&e.description)))?;
        wr.write_event(Event::End(BytesStart::new("AddtlNtryInf").to_end()))?;
    }

    wr.write_event(Event::End(BytesStart::new("Ntry").to_end()))?;
    Ok(())
}

fn xml<E: std::fmt::Display>(e: E) -> FinioError {
    FinioError::Xml(e.to_string())
}

/* ------------------------------- READ ----------------------------------- */

impl ReadFormat for Camt053 {
    fn read<R: BufRead>(r: R) -> Result<Statement> {
        let mut reader = Reader::from_reader(r);
        reader.trim_text(true);

        let mut st = Statement {
            statement_id: None,
            account_id: String::new(),
            opening_balance: None,
            closing_balance: None,
            entries: Vec::new(),
        };

        let mut buf = Vec::new();
        let mut text_buf = String::new();

        // «флаги» текущего положения курсора
        let mut in_iban = false;
        let mut in_id = false;       // Id внутри Stmt (не путать с Acct/Id/IBAN)
        let mut in_amt = false;
        let mut amt_ccy = String::new();
        let mut in_cdt_dbt = false;
        let mut in_book_dt = false;
        let mut in_val_dt = false;
        let mut in_addtl = false;
        let mut in_ntry_ref = false;

        let mut pending: Option<Entry> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.local_name().as_ref() {
                        b"IBAN" => in_iban = true,
                        b"Id" => in_id = true,
                        b"Amt" => {
                            in_amt = true;
                            amt_ccy.clear();
                            for a in e.attributes().flatten() {
                                if a.key.as_ref() == b"Ccy" {
                                    if let Ok(v) = String::from_utf8(a.value.into_owned()) {
                                        amt_ccy = v;
                                    }
                                }
                            }
                        }
                        b"CdtDbtInd" => in_cdt_dbt = true,
                        b"BookgDt" => in_book_dt = true,
                        b"ValDt" => in_val_dt = true,
                        b"AddtlNtryInf" => in_addtl = true,
                        b"NtryRef" => in_ntry_ref = true,
                        b"Ntry" => {
                            let booking_date = NaiveDate::from_ymd_opt(1970, 1, 1)
                                .ok_or_else(|| FinioError::Parse("invalid default booking date 1970-01-01".into()))?;
                            pending = Some(Entry {
                                booking_date,
                                value_date: None,
                                amount: Decimal::ZERO,
                                currency: "XXX".into(),
                                dc: DebitCredit::Credit,
                                description: String::new(),
                                reference: None,
                            });
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(t)) => {
                    text_buf.clear();
                    text_buf.push_str(
                        &t.unescape()
                            .map_err(|e| FinioError::Xml(e.to_string()))?
                            .to_string(),
                    );

                    if in_iban {
                        st.account_id = text_buf.clone();
                    } else if in_id && st.statement_id.is_none() {
                        // первый <Id> внутри Stmt — это идентификатор выписки
                        st.statement_id = Some(text_buf.clone());
                    } else if in_amt {
                        if let Some(ref mut e) = pending {
                            // сумма + валюта из атрибута Amt Ccy
                            e.amount = Decimal::from_str_exact(&text_buf)
                                .or_else(|_| text_buf.parse())
                                .map_err(|e| FinioError::Parse(format!("camt amount: {e}")))?;
                            if !amt_ccy.is_empty() {
                                e.currency = amt_ccy.clone();
                            }
                        }
                    } else if in_cdt_dbt {
                        if let Some(ref mut e) = pending {
                            e.dc = match text_buf.as_str() {
                                "CRDT" => DebitCredit::Credit,
                                "DBIT" => DebitCredit::Debit,
                                other => {
                                    return Err(FinioError::Parse(format!(
                                        "CdtDbtInd {}",
                                        other
                                    )))
                                }
                            }
                        }
                    } else if in_book_dt {
                        if let Some(ref mut e) = pending {
                            e.booking_date = NaiveDate::parse_from_str(&text_buf, "%Y-%m-%d")
                                .map_err(|e| FinioError::Parse(format!("{e}")))?;
                        }
                    } else if in_val_dt {
                        if let Some(ref mut e) = pending {
                            e.value_date = Some(
                                NaiveDate::parse_from_str(&text_buf, "%Y-%m-%d")
                                    .map_err(|e| FinioError::Parse(format!("{e}")))?,
                            );
                        }
                    } else if in_addtl {
                        if let Some(ref mut e) = pending {
                            e.description = text_buf.clone();
                        }
                    } else if in_ntry_ref {
                        if let Some(ref mut e) = pending {
                            e.reference = Some(text_buf.clone());
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    match e.local_name().as_ref() {
                        b"IBAN" => in_iban = false,
                        b"Id" => in_id = false,
                        b"Amt" => in_amt = false,
                        b"CdtDbtInd" => in_cdt_dbt = false,
                        b"BookgDt" => in_book_dt = false,
                        b"ValDt" => in_val_dt = false,
                        b"AddtlNtryInf" => in_addtl = false,
                        b"NtryRef" => in_ntry_ref = false,
                        b"Ntry" => {
                            if let Some(e) = pending.take() {
                                st.entries.push(e);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(FinioError::Xml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }
        Ok(st)
    }
}
