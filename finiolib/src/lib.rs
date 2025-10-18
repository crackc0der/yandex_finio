//! finiolib — библиотека для чтения/записи финансовых данных (CSV, XML, MT940, CAMT.053)

pub mod error;
pub mod model;
pub mod traits;
pub mod convert;

pub mod formats {
    pub mod csv;
    pub mod xml;
    pub mod mt940;
    pub mod camt053;
}
