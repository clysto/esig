pub fn human_readable_time(time: f64) -> String {
    let mut time = time;
    let mut units = vec!["ns", "us", "ms", "s"];
    let mut unit = units.pop().unwrap();
    while time < 1.0 && !units.is_empty() {
        time *= 1_000.0;
        unit = units.pop().unwrap();
    }
    format!("{}{}", (time * 1000000.).round() / 1000000., unit)
}
