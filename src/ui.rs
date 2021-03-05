use crate::categories;
use crate::text::TextManager;
use pancurses;

const COLOR_NORMAL: i16 = 0;
const COLOR_RIGHT: i16 = 1;
const COLOR_WRONG: i16 = 2;
const COLOR_OPTION_SELECTED: i16 = 3;
const COLOR_CURRENT_CHAR: i16 = 3;

enum UIMode {
  TYPE,
  COMMAND,
}

pub struct UI {
  main_window: pancurses::Window,
  text_window: pancurses::Window,
  info_window: pancurses::Window,
  text_manager: TextManager,
  ui_mode: UIMode,
  is_first_update: bool,
  categories: categories::Categories,
  current_category: String,
}

impl UI {
  pub fn new(texts_dir: String) -> Self {
    std::env::set_var("ESCDELAY", "0");

    let main_window = pancurses::initscr();

    main_window.keypad(true);
    main_window.nodelay(true);

    pancurses::noecho();
    pancurses::start_color();
    pancurses::nl();
    pancurses::curs_set(0);

    pancurses::init_pair(COLOR_NORMAL, pancurses::COLOR_WHITE, pancurses::COLOR_BLACK);
    pancurses::init_pair(COLOR_RIGHT, pancurses::COLOR_GREEN, pancurses::COLOR_BLACK);
    pancurses::init_pair(COLOR_WRONG, pancurses::COLOR_WHITE, pancurses::COLOR_RED);
    pancurses::init_pair(
      COLOR_OPTION_SELECTED,
      pancurses::COLOR_BLACK,
      pancurses::COLOR_WHITE,
    );
    pancurses::init_pair(
      COLOR_CURRENT_CHAR,
      pancurses::COLOR_BLACK,
      pancurses::COLOR_WHITE,
    );

    let (text_window, info_window) = Self::create_subwindows(&main_window);
    let categories = categories::Categories::new(texts_dir);

    UI {
      main_window,
      text_window,
      info_window,
      text_manager: TextManager::new(categories.get_text("Basic")),
      ui_mode: UIMode::TYPE,
      is_first_update: true,
      categories,
      current_category: String::from("Basic"),
    }
  }

  pub fn run(&mut self) {
    loop {
      match self.ui_mode {
        UIMode::COMMAND => {
          if !self.command_loop() {
            break;
          }
          self.is_first_update = true;
        }
        UIMode::TYPE => {
          self.type_loop();
        }
      }
      self.common_loop();
    }

    pancurses::endwin();
  }

  fn end_run(&mut self) {
    self.text_manager.end_run();

    let (max_y, max_x) = self.main_window.get_max_yx();
    let stats_window = pancurses::newwin(max_y, max_x, 0, 0);
    stats_window.keypad(true);
    stats_window.nodelay(true);

    stats_window.mv(0, 0);
    stats_window.addstr("Run saved! Press q to go back to typing.\n");

    self.write_info_to_window(&stats_window);
    stats_window.refresh();
    loop {
      match stats_window.getch() {
        Some(pancurses::Input::Character('q')) => {
          self.ui_mode = UIMode::TYPE;
          break;
        }
        Some(pancurses::Input::KeyDown) => {}
        Some(pancurses::Input::KeyUp) => {}
        _ => (),
      }
    }
    stats_window.delwin();
    self.text_manager = TextManager::new(self.categories.get_text(&self.current_category));
  }

  fn show_improvement(&mut self) -> pancurses::Window {
    let (max_y, max_x) = self.main_window.get_max_yx();
    let stats_window = pancurses::newwin(max_y, max_x, 0, 0);
    stats_window.keypad(true);
    stats_window.nodelay(true);
    stats_window.mv(0, 0);

    let data = self
      .text_manager
      .get_improvement(max_x as usize, (max_y - 2) as usize);
    if let Some(data) = data {
      stats_window.addstr("Improvement: \n");
      let mut i = 0;
      for point in data {
        stats_window.mvaddch(max_y - 1 - point as i32, i, '*');
        i += 1;
      }
    } else {
      stats_window.addstr("No improvement data found\n");
    }
    stats_window.refresh();
    return stats_window;
  }

  fn command_loop(&mut self) -> bool {
    match self.main_window.getch() {
      Some(pancurses::Input::Character('i')) => {
        self.ui_mode = UIMode::TYPE;
      }
      Some(pancurses::Input::Character('q')) => {
        return false;
      }
      Some(pancurses::Input::Character('c')) => {
        let categories = self.categories.get_categories();
        if categories.len() > 0 {
          let idx = self.menu_choose(&categories);
          self.current_category = categories[idx].clone();
          self.text_manager = TextManager::new(self.categories.get_text(&self.current_category));
          self.ui_mode = UIMode::TYPE;
        }
      }
      Some(pancurses::Input::Character('t')) => {
        let mut improvement_win : Option<pancurses::Window> = None;
        let mut update_improvement = true;
        loop {
          match self.main_window.getch() {
            Some(pancurses::Input::Character('q')) => {
              self.ui_mode = UIMode::TYPE;
              break;
            }
            Some(pancurses::Input::KeyResize) => {
              update_improvement = true;
            }
            _ => {
              if update_improvement {
                if let Some(win) = improvement_win {
                  win.delwin();
                }
                improvement_win = Some(self.show_improvement());
                update_improvement = false;
              }
            },
          }
        }
        if let Some(win) = improvement_win {
          win.delwin();
        }
      }
      Some(pancurses::Input::Character('e')) => {
        self.end_run();
      }
      _ => (),
    }
    return true;
  }

  fn menu_choose(&self, list: &std::vec::Vec<String>) -> usize {
    let (max_y, max_x) = self.main_window.get_max_yx();
    let menu_window = pancurses::newwin(max_y, max_x, 0, 0);
    menu_window.keypad(true);
    menu_window.nodelay(true);
    let mut curr = 0;
    loop {
      match menu_window.getch() {
        Some(pancurses::Input::Character('\n')) => {
          break;
        }
        Some(pancurses::Input::KeyDown) => {
          curr = (curr + 1) % list.len();
        }
        Some(pancurses::Input::KeyUp) => {
          curr = if curr == 0 { list.len() - 1 } else { curr - 1 };
        }
        _ => (),
      }
      menu_window.mv(0, 0);
      menu_window
        .addstr("Press Up and Down to choose an option. Press Enter to make a selection.\n");
      for i in 0..list.len() {
        if i == curr {
          menu_window.color_set(COLOR_OPTION_SELECTED);
        }
        menu_window.addstr(&list[i][..]);
        menu_window.addch('\n');
        menu_window.color_set(COLOR_NORMAL);
      }
    }
    menu_window.delwin();

    //self.main_window.touch();
    //self.main_window.refresh();

    return curr;
  }

  fn create_subwindows(main_window: &pancurses::Window) -> (pancurses::Window, pancurses::Window) {
    let (max_y, max_x) = main_window.get_max_yx();

    let text_w = max_x / 3 * 2;

    let text_window = main_window.subwin(max_y, text_w, 0, 0).unwrap();
    text_window.setscrreg(0, max_y);

    let info_window = main_window
      .subwin(max_y, max_x - text_w, 0, text_w)
      .unwrap();

    return (text_window, info_window);
  }

  fn recreate_subwindows(&mut self) {
    let (new_text_window, new_info_window) = Self::create_subwindows(&self.main_window);
    std::mem::replace(&mut self.text_window, new_text_window).delwin();
    std::mem::replace(&mut self.info_window, new_info_window).delwin();
    self.main_window.clear();
    self.text_window.clear();
    self.info_window.clear();
    self.main_window.refresh();
    self.text_window.refresh();
    self.info_window.refresh();
    self.is_first_update = true;
  }

  fn type_loop(&mut self) {
    let mut need_to_update_text = self.is_first_update;
    match self.main_window.getch() {
      Some(pancurses::Input::Character('\u{1b}')) => {
        self.ui_mode = UIMode::COMMAND;
      }
      Some(pancurses::Input::Character(c)) => {
        self.text_manager.type_char(c);
        need_to_update_text = true;
      }
      Some(pancurses::Input::KeyBackspace) => {
        self.text_manager.del_char();
        need_to_update_text = true;
      }
      Some(pancurses::Input::KeyResize) => {
        self.recreate_subwindows();
      }
      _ => (),
    }
    if need_to_update_text {
      let parts = self.text_manager.get_text_parts();
      let (h, _) = self.text_window.get_max_yx();
      self.text_window.scrollok(true);
      self.text_window.mv(0, 0);
      let mut current_right = true;
      for _ in 0..h / 2 {
        self.text_window.addch('\n');
      }
      for i in 0..parts.len() {
        let current_rest = i == parts.len() - 1;
        if current_rest {
          self.text_window.color_set(COLOR_NORMAL);
        } else if current_right {
          self.text_window.color_set(COLOR_RIGHT);
          current_right = false;
        } else {
          self.text_window.color_set(COLOR_WRONG);
          current_right = true;
        }
        let mut lines = 0;
        let mut last_x = 0;

        for (i, c) in parts[i].char_indices() {
          let (_, x) = self.text_window.get_cur_yx();
          if x == 0 && last_x != 0 {
            lines += 1;
          }
          if current_rest && lines >= h / 2 {
            break;
          }
          if current_rest && i == 0 {
            self.text_window.color_set(COLOR_CURRENT_CHAR);
          }

          // self.text_window.addch(c);
          self.text_window.addstr(c.to_string()); // workaround for unicode characters

          last_x = x;
          if current_rest {
            self.text_window.color_set(COLOR_NORMAL);
          }
        }
      }
      self.text_window.refresh();
      self.is_first_update = false;
    }
  }

  fn write_info_to_window(&self, window: &pancurses::Window) {
    window.addstr(format!(
      "  Accuracy: {:.2}%\n",
      self.text_manager.get_accuracy().unwrap_or(0.) * 100.
    ));
    window.addstr(format!(
      "  WPM: {:.2}\n",
      self.text_manager.get_wpm().unwrap_or(0.)
    ));
    window.addstr(format!(
      "  CPM: {:.2}\n",
      self.text_manager.get_cpm().unwrap_or(0.)
    ));
    window.addstr("  Slowest letters:  Most error letters:\n");

    let slowest_letters = self.text_manager.get_slowest_letters();
    let most_error_letters = self.text_manager.get_most_error_letters();

    let (h, _) = window.get_max_yx();

    for ((slow_letter, ms), (error_letter, count)) in
      slowest_letters.into_iter().zip(most_error_letters)
    {
      window.addstr(format!(
        "{: <18}",
        format!("  {:?} - {:?} ms", slow_letter, ms)
      ));
      window.addstr(format!(
        "{: <18}",
        format!("  {:?} - {:?}", error_letter, count)
      ));
      window.addch('\n');
      let (y, _) = window.get_cur_yx();
      if y >= h - 1 {
        break;
      }
    }

    window.refresh();
  }

  fn common_loop(&mut self) {
    self.info_window.mv(0, 0);
    self.write_info_to_window(&self.info_window);
  }
}
