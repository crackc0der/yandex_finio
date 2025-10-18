cargo build --workspace
cargo test --workspace
# CSV -> MT940
cat examples/sample.csv | cargo run -p finio -- --in-format csv --out-format mt940
# MT940 -> CSV
cargo run -p finio -- -i examples/sample.mt940 --in-format mt940 -o out.csv --out-format csv
# CSV -> simple XML
cargo run -p finio -- -i examples/sample.csv --in-format csv -o out.xml --out-format xml
# CSV -> CAMT.053
cargo run -p finio -- -i examples/sample.csv --in-format csv -o out.camt.xml --out-format camt053
