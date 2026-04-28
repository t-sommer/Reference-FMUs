use plotly::{Plot, Scatter};

#[test]
fn test_write_html() {
    let mut plot = Plot::new();
    plot.add_trace(Scatter::new(vec![1, 2, 3], vec![4, 5, 6]));
    plot.write_html("plot.html");
}