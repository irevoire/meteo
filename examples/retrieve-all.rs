use std::str::FromStr;

use logos::Logos;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime};

fn main() {
    let base_url = "http://meteo.lyc-chamson-levigan.ac-montpellier.fr/meteo/releve/fichiersbrut/sauvegardes/fichiersMensuels";

    for year in 2006..=2024 {
        for month in 1..=12 {
            // http://meteo.lyc-chamson-levigan.ac-montpellier.fr/meteo/releve/fichiersbrut/sauvegardes/fichiersMensuels/2006_06.txt
            let url = format!("{base_url}/{year}_{month:02}.txt");
            let response = match ureq::get(&url).call() {
                Ok(response) => response,
                Err(e) => {
                    eprintln!("Could not fetch report for {year}/{month} with url: `{url}`. {e}");
                    continue;
                }
            };

            let report = response.into_string().unwrap();
            std::fs::write(format!("{year}_{month:02}.txt"), &report).unwrap();

            println!("Wrote report of {year}/{month}");
        }
    }
}
