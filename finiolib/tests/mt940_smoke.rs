use finiolib::{formats::mt940::Mt940, traits::ReadFormat, model::DebitCredit};
use rust_decimal::Decimal; // <— добавь
use std::io::Cursor;

#[test]
fn mt940_read_minimal() {
    let s = r#":20:STATEMENT1
:25:DE0012345678
:60F:C251001EUR1000,00
:61:2510011001C100,00NTRFNONREF
:86:Salary October
:62F:C251031EUR1100,00
"#;
    let st = Mt940::read(Cursor::new(s)).expect("mt940 read");
    assert_eq!(st.account_id, "DE0012345678");
    assert_eq!(st.entries.len(), 1);
    let e = &st.entries[0];
    assert_eq!(e.currency, "EUR");
    assert_eq!(e.dc, DebitCredit::Credit);
    assert_eq!(e.description, "Salary October");

    // ✅ сравниваем числовое значение, а не строку
    assert_eq!(e.amount, Decimal::new(100, 0));
    // или так:
    // assert_eq!(e.amount, Decimal::from_str_exact("100").unwrap());
    // Если хочется по строке — тогда ожидай "100.00":
    // assert_eq!(e.amount.to_string(), "100.00");
}