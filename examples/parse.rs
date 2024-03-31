use std::str::FromStr;

use meteo::Report;

fn main() {
    let file = std::env::args().nth(1).expect("Missing filename");
    println!("opening {file}");
    let file = std::fs::read_to_string(file).unwrap();

    let report = Report::from_str(&file).unwrap();

    println!(
        "Mean temp of the month: {:.1}",
        report.days.iter().map(|day| day.mean_temp).sum::<f32>() / report.days.len() as f32
    );
}
