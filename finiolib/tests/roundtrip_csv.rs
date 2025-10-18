use finiolib::{formats::csv::Csv, traits::{ReadFormat, WriteFormat}};
use std::io::Cursor;

#[test]
fn csv_roundtrip() {
    let input = r#"booking_date,value_date,amount,currency,dc,description,reference,account_id,opening_amount,opening_currency,opening_date,closing_amount,closing_currency,closing_date
2025-10-01,2025-10-01,100.00,EUR,C,Salary,REF1,DE0012345678,1000.00,EUR,2025-10-01,1100.00,EUR,2025-10-31
"#;
    let st = Csv::read(Cursor::new(input)).expect("read csv");
    let mut out = Vec::new();
    Csv::write(&mut out, &st).expect("write csv");
    assert!(!out.is_empty());
}
