use plotters::prelude::*;

pub fn draw(
    size: (u32, u32),
    title: &str,
    xlabel: &str,
    ylabel: &str,
    series: &[(&str, &[(f64, f64)])],
) -> String {
    let mut buf = String::new();
    let (minx, miny, maxx, maxy) =
        series
            .iter()
            .fold((0.0, 0.0, 0.0, 0.0), |rect, (_, samples)| {
                samples
                    .iter()
                    .fold(rect, |(minx, miny, maxx, maxy), &(x, y)| {
                        (
                            f64::min(minx, x),
                            f64::min(miny, y),
                            f64::max(maxx, x),
                            f64::max(maxy, y),
                        )
                    })
            });
    let root = SVGBackend::with_string(&mut buf, size).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut cc = ChartBuilder::on(&root)
        .margin(10)
        .caption(title, ("sans-serif", 30))
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(minx..maxx, miny..maxy)
        .unwrap();

    cc.configure_mesh()
        .x_label_formatter(&|x| format!("{}", x))
        .y_label_formatter(&|y| format!("{}", y))
        .x_labels(10)
        .y_labels(10)
        .x_desc(xlabel)
        .y_desc(ylabel)
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .unwrap();

    for (idx, (label, samples)) in series.iter().enumerate() {
        cc.draw_series(LineSeries::new(
            samples.iter().copied(),
            &Palette99::pick(idx),
        ))
        .unwrap()
        .label(label.to_string())
        .legend(move |(x, y)| {
            Rectangle::new([(x - 5, y - 5), (x + 5, y + 5)], &Palette99::pick(idx))
        });
    }

    cc.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();

    drop(cc);
    drop(root);

    buf
}
