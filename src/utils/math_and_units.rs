use gtk::{prelude::*};
use meval;
use uom::si::f64::*;
use uom::si::length::{kilometer, meter, mile};

pub fn try_math_expression(query: &str) -> Option<(String, &'static str)> {
    if let Ok(result) = meval::eval_str(query) {
        Some((format!("{} = {}", query, result), "accessories-calculator"))
    } else {
        None
    }
}

pub fn try_unit_conversion(query: &str) -> Option<(String, &'static str)> {
    let lower = query.to_lowercase();
    if lower.contains("to") {
        let parts: Vec<&str> = lower.split("to").map(|s| s.trim()).collect();

        if parts.len() >= 2 {
            let value_parts: Vec<&str> = parts[0].split_whitespace().collect();
            if value_parts.len() != 2 {
                return None;
            }

            let mut value = match value_parts[0].parse::<f64>() {
                Ok(v) => v,
                Err(_) => return None,
            };
            let mut current_unit = value_parts[1];

            let mut result_str = format!("{:.4} {}", value, current_unit);

            for target_unit in &parts[1..] {
                let target_unit = target_unit.trim();
                value = match (current_unit, target_unit) {

                    // length
                    ("km", "mi") => Length::new::<kilometer>(value).get::<mile>(),
                    ("mi", "km") => Length::new::<mile>(value).get::<kilometer>(),
                    ("km", "m")  => Length::new::<kilometer>(value).get::<meter>(),
                    ("m", "km")  => Length::new::<meter>(value).get::<kilometer>(),

                    // temperature
                    ("c", "f") => (value * 9.0 / 5.0) + 32.0,
                    ("f", "c") => (value - 32.0) * 5.0 / 9.0,
                    ("c", "k") => value + 273.15,
                    ("k", "c") => value - 273.15,
                    ("f", "k") => ((value - 32.0) * 5.0 / 9.0) + 273.15,
                    ("k", "f") => ((value - 273.15) * 9.0 / 5.0) + 32.0,

                    // storage (binary units, 1024)
                    ("gb", "mb") => value * 1024.0,
                    ("mb", "gb") => value / 1024.0,
                    ("mb", "kb") => value * 1024.0,
                    ("kb", "mb") => value / 1024.0,
                    ("gb", "kb") => value * 1024.0 * 1024.0,
                    ("kb", "gb") => value / 1024.0 / 1024.0,

                    _ => return None,
                };
                current_unit = target_unit;
                result_str.push_str(&format!(" = {} {}", trim_trailing_zeros(value), current_unit));
            }

            return Some((result_str, "accessories-calculator"));
        }
    }
    None
}


pub fn copy_to_clipboard(text: &str) {
    if let Some(display) = gtk::gdk::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text(text);
    }
}

fn trim_trailing_zeros(num: f64) -> String {
    if num.fract() == 0.0 {
        format!("{}", num as i64)
    } else {
        format!("{:.4}", num).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

