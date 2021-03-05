mod categories;
mod text;
mod ui;

fn main() {
  let args: Vec<_> = std::env::args().collect();

  ui::UI::new(if args.len() > 1 { args[1].clone() } else { String::from("./texts") }).run();
}