use clap::{Parser, ValueEnum};
use finiolib::{
    error::{FinioError, Result},
    formats::{csv::Csv, xml::SimpleXml, mt940::Mt940, camt053::Camt053},
    traits::{ReadFormat, WriteFormat},
};
use std::fs::File;
use std::io::{self, BufReader, Write};

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Fmt {
    Csv,
    Xml,
    Mt940,
    Camt053,
}

#[derive(Parser, Debug)]
#[command(name="finio", version, about="Конвертация финансовых данных")]
struct Cli {
    /// Входной файл (по умолчанию stdin)
    #[arg(short='i', long="input")]
    input: Option<String>,

    /// Выходной файл (по умолчанию stdout)
    #[arg(short='o', long="output")]
    output: Option<String>,

    /// Формат входа
    #[arg(long="in-format", value_enum)]
    in_format: Fmt,

    /// Формат выхода
    #[arg(long="out-format", value_enum)]
    out_format: Fmt,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // reader
    let reader: Box<dyn io::Read> = match cli.input {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };
    let br = BufReader::new(reader);

    let st = match cli.in_format {
        Fmt::Csv => Csv::read(br),
        Fmt::Xml => SimpleXml::read(br),
        Fmt::Mt940 => Mt940::read(br),
        Fmt::Camt053 => Camt053::read(br),
    }?;

    // writer
    let mut writer: Box<dyn Write> = match cli.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    match cli.out_format {
        Fmt::Csv => Csv::write(&mut writer, &st),
        Fmt::Xml => SimpleXml::write(&mut writer, &st),
        Fmt::Mt940 => Mt940::write(&mut writer, &st),
        Fmt::Camt053 => Camt053::write(&mut writer, &st),
    }?;

    writer.flush().map_err(FinioError::from)
}
