use std::str::FromStr;

use meteo::Report;
use plotters::prelude::*;

fn main() {
    let inputs = std::env::args().skip(1);
    let mut report: Option<Report> = None;
    for input in inputs {
        let input = std::fs::read_to_string(input).unwrap();

        let r = match Report::from_str(&input) {
            Ok(r) => r,
            Err(_) => continue,
        };
        match report {
            Some(ref mut report) => report.merge(r).unwrap(),
            None => report = Some(r),
        };
    }
    let report = report.unwrap();
    let output = format!("0.png");

    let first_date = report.first_date();
    let last_date = report.last_date();

    let root = BitMapBackend::new(&output, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Pluie du {} au {}", first_date, last_date),
            ("sans-serif", 100).into_font(),
        )
        .margin(5)
        .x_label_area_size(80)
        .y_label_area_size(80)
        .build_cartesian_2d(
            chrono::NaiveDate::from_ymd_opt(
                first_date.year(),
                first_date.month() as u32,
                first_date.day() as u32,
            )
            .unwrap()
                ..chrono::NaiveDate::from_ymd_opt(
                    last_date.year(),
                    last_date.month().next() as u32,
                    last_date.day() as u32,
                )
                .unwrap(),
            report.range(|day| day.rain, |l, r| l.total_cmp(r)),
        )
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            report.days.iter().map(|day| {
                (
                    chrono::NaiveDate::from_ymd_opt(
                        day.date.year(),
                        day.date.month() as u32,
                        day.date.day() as u32,
                    )
                    .expect(&format!("chrono is a piece of shit {:?}", day.date)),
                    day.rain,
                )
            }),
            BLUE,
        ))
        .unwrap()
        .label("Température moyenne")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();
}
