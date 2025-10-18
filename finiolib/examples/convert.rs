use finiolib::{
    formats::{csv::Csv, xml::SimpleXml},
    traits::{ReadFormat, WriteFormat},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Пример: конвертируем CSV -> XML (stdin -> stdout)
    let st = Csv::read(std::io::BufReader::new(std::io::stdin()))?;
    SimpleXml::write(std::io::stdout(), &st)?;
    Ok(())
}
