use finiolib::{formats::xml::SimpleXml, traits::{ReadFormat, WriteFormat}, model::{Statement, Entry, Balance, DebitCredit}};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::io::Cursor;

#[test]
fn simple_xml_roundtrip() {
    let st = Statement {
        statement_id: Some("S1".into()),
        account_id: "ACCT".into(),
        opening_balance: Some(Balance {
            date: NaiveDate::from_ymd_opt(2025,10,1).unwrap(),
            amount: Decimal::from_str_exact("1.00").unwrap(),
            currency: "EUR".into(),
        }),
        closing_balance: None,
        entries: vec![Entry{
            booking_date: NaiveDate::from_ymd_opt(2025,10,1).unwrap(),
            value_date: None,
            amount: Decimal::from_str_exact("2.50").unwrap(),
            currency: "EUR".into(),
            dc: DebitCredit::Debit,
            description: "Test".into(),
            reference: None,
        }],
    };

    let mut out = Vec::new();
    SimpleXml::write(&mut out, &st).expect("write simple xml");
    let st2 = SimpleXml::read(Cursor::new(out)).expect("read simple xml");
    assert_eq!(st2.account_id, "ACCT");
    assert_eq!(st2.entries.len(), 1);
    assert_eq!(st2.entries[0].description, "Test");
}
