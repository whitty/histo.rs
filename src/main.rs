use asciigraph::Graph;

fn main() {
    println!("Hello, world!");
    let d = Graph::new(36, 18)  // (width, height)
    .set_1d_data(
        vec![256, 128, 192, 160, 176, 168, 172, 170]
    )
    .draw();
    println!("{d}");

    let mut g1 = Graph::default();

    g1.set_1d_data(vec![0, 1, 1, 0, 2, 0, 1, 2, 0, 0, 0, 1, 0, 1000])
//    .set_y_min(0)
//    .set_y_max(4)
//    .set_plot_height(20)
    .set_block_width(3)
//    .set_y_label_margin(1)
//    .set_title(String::from("HEllo 23123123"))
    .set_paddings([1;4])
    .set_big_title(true)
    .set_x_axis_label(String::from("x_axis_label\nxz"))
        .set_y_axis_label(String::from("y_axis_label\nyy"));
    let x = g1.draw();

    println!("{x}");
}
