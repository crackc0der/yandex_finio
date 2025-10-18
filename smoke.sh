#!/usr/bin/env bash
set -euo pipefail

# 0) Сборка
cargo build --workspace >/dev/null

# 1) Временная папка и фикстуры
TMP="${TMPDIR:-/tmp}/finio_smoke"
rm -rf "$TMP" && mkdir -p "$TMP"
cat > "$TMP/sample.csv" <<'CSV'
booking_date,value_date,amount,currency,dc,description,reference,account_id,opening_amount,opening_currency,opening_date,closing_amount,closing_currency,closing_date
2025-10-01,2025-10-01,100.00,EUR,C,Salary Oct,REF1,DE0012345678,1000.00,EUR,2025-10-01,1100.00,EUR,2025-10-31
2025-10-02,2025-10-02,25.50,EUR,D,Groceries,REF2,DE0012345678,1000.00,EUR,2025-10-01,1100.00,EUR,2025-10-31
CSV

cat > "$TMP/sample.mt940" <<'MT'
:20:STATEMENT1
:25:DE0012345678
:60F:C251001EUR1000,00
:61:2510011001C100,00NTRFNONREF
:86:Salary Oct
:61:2510021002D25,50NTRFNONREF
:86:Groceries
:62F:C251031EUR1100,00
MT

# 2) Конверсии
# CSV -> MT940 -> CSV
cargo run -q -p finio -- -i "$TMP/sample.csv"   --in-format csv    -o "$TMP/a.mt940" --out-format mt940
cargo run -q -p finio -- -i "$TMP/a.mt940"      --in-format mt940  -o "$TMP/a.csv"   --out-format csv

# CSV -> CAMT -> CSV
cargo run -q -p finio -- -i "$TMP/sample.csv"   --in-format csv    -o "$TMP/b.camt.xml" --out-format camt053
cargo run -q -p finio -- -i "$TMP/b.camt.xml"   --in-format camt053 -o "$TMP/b.csv"     --out-format csv

# CSV -> simple XML -> CSV
cargo run -q -p finio -- -i "$TMP/sample.csv"   --in-format csv    -o "$TMP/c.xml"   --out-format xml
cargo run -q -p finio -- -i "$TMP/c.xml"        --in-format xml    -o "$TMP/c.csv"   --out-format csv

# 3) Быстрые sanity-проверки
test "$(grep -o '<Ntry>' "$TMP/b.camt.xml" | wc -l | tr -d ' ')" -eq 2 || {
  echo "FAIL: CAMT должен содержать 2 <Ntry>"
  head -c 400 "$TMP/b.camt.xml"; echo
  exit 1
}
test "$(tail -n +2 "$TMP/a.csv" | wc -l | tr -d ' ')" = "2" || { echo "FAIL: roundtrip MT940 дал неверное кол-во строк"; exit 1; }

# 4) Сравнение «по смыслу», а не по форматированию
python3 - "$TMP/sample.csv" "$TMP/a.csv" <<"PY"
import sys, csv, decimal
orig, roundtrip = sys.argv[1], sys.argv[2]
def rows(path):
    with open(path, newline="") as f:
        r = csv.DictReader(f)
        out=[]
        for row in r:
            tup = (
                row["booking_date"],
                row.get("value_date") or "",
                decimal.Decimal(row["amount"]),
                row["currency"],
                row["dc"].upper(),
                row["description"],
                row.get("reference") or "",
            )
            out.append(tup)
        return sorted(out)
a,b = rows(orig), rows(roundtrip)
if a!=b:
    print("FAIL: CSV -> MT940 -> CSV изменил данные.")
    print("ORIG:", a)
    print("RND : ", b)
    sys.exit(1)
print("OK: CSV -> MT940 -> CSV совпали по полям.")
PY

python3 - "$TMP/sample.csv" "$TMP/b.csv" <<"PY"
import sys, csv, decimal
orig, roundtrip = sys.argv[1], sys.argv[2]
def rows(path):
    with open(path, newline="") as f:
        r = csv.DictReader(f)
        out=[]
        for row in r:
            tup = (
                row["booking_date"],
                row.get("value_date") or "",
                decimal.Decimal(row["amount"]),
                row["currency"],
                row["dc"].upper(),
                row["description"],
                row.get("reference") or "",
            )
            out.append(tup)
        return sorted(out)
a,b = rows(orig), rows(roundtrip)
if a!=b:
    print("FAIL: CSV -> CAMT -> CSV изменил данные.")
    print("ORIG:", a)
    print("RND : ", b)
    sys.exit(1)
print("OK: CSV -> CAMT -> CSV совпали по полям.")
PY

echo "Все отлично.  Результаты в $TMP"
