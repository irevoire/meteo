use std::str::FromStr;

use meteo::Report;
use plotters::prelude::*;

fn main() {
    let input = std::env::args().nth(1).expect("Missing filename");
    println!("opening {input}");
    let output = format!("{input}.png");
    let input = std::fs::read_to_string(input).unwrap();

    let report = Report::from_str(&input).unwrap();

    let date = report.metadata.date;

    let root = BitMapBackend::new(&output, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("Température du mois de {}", date),
            ("sans-serif", 100).into_font(),
        )
        .margin(5)
        .x_label_area_size(80)
        .y_label_area_size(80)
        .build_cartesian_2d(
            chrono::NaiveDate::from_ymd_opt(date.year(), date.month() as u32, date.day() as u32)
                .unwrap()
                ..chrono::NaiveDate::from_ymd_opt(
                    date.year(),
                    date.month().next() as u32,
                    date.day() as u32,
                )
                .unwrap(),
            // report.range(|day| day.mean_temp, |l, r| l.total_cmp(r)),
            report.temperature_range(),
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
                    day.mean_temp,
                )
            }),
            GREEN,
        ))
        .unwrap()
        .label("Température moyenne")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN));
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
                    day.high_temp,
                )
            }),
            RED,
        ))
        .unwrap()
        .label("Température maximale")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));
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
                    day.low_temp,
                )
            }),
            BLUE,
        ))
        .unwrap()
        .label("Température minimale")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .unwrap();

    root.present().unwrap();
}
