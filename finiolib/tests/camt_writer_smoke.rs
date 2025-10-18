use finiolib::{
    formats::camt053::Camt053,
    model::{Statement, Entry, Balance, DebitCredit},
    traits::{ReadFormat, WriteFormat},
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::io::Cursor;

#[test]
fn camt_write_then_read_back() {
    let st = Statement {
        statement_id: Some("S1".into()),
        account_id: "DE0012345678".into(),
        opening_balance: Some(Balance {
            date: NaiveDate::from_ymd_opt(2025,10,1).unwrap(),
            amount: Decimal::from_str_exact("1000.00").unwrap(),
            currency: "EUR".into(),
        }),
        closing_balance: Some(Balance {
            date: NaiveDate::from_ymd_opt(2025,10,31).unwrap(),
            amount: Decimal::from_str_exact("1100.00").unwrap(),
            currency: "EUR".into(),
        }),
        entries: vec![Entry{
            booking_date: NaiveDate::from_ymd_opt(2025,10,1).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2025,10,1).unwrap()),
            amount: Decimal::from_str_exact("100.00").unwrap(),
            currency: "EUR".into(),
            dc: DebitCredit::Credit,
            description: "Salary October".into(),
            reference: None,
        }],
    };

    // write to XML
    let mut buf = Vec::new();
    Camt053::write(&mut buf, &st).expect("write camt");

    // read back
    let st2 = Camt053::read(Cursor::new(buf)).expect("read camt");
    assert_eq!(st2.account_id, st.account_id);
    assert_eq!(st2.entries.len(), 1);
    assert_eq!(st2.entries[0].dc, DebitCredit::Credit);
    assert_eq!(st2.entries[0].amount, Decimal::new(100, 0));
}
