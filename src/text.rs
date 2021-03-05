const LOGFILE_PATH: &str = ".typeracer-log";

struct DataPoint {
  cpm: f32,
  _time: u64,
  _accuracy: f32,
  _wpm: f32,
}

struct LetterInfo {
  duration: std::time::Duration,
  count: usize,
  errors: usize,
}

pub struct TextManager {
  current_text: String,
  typed_text: String,
  start_time: Option<std::time::Instant>,
  last_type: Option<std::time::Instant>,
  typed_chars: u32,
  typed_words: f32,
  accuracy: f32,
  log_file: Option<std::fs::File>,
  letters: std::collections::HashMap<char, LetterInfo>,
}

impl TextManager {
  pub fn new(current_text: String) -> Self {
    assert!(current_text.len() > 0);
    TextManager {
      current_text: String::from(current_text),
      typed_text: String::new(),
      start_time: None,
      last_type: None,
      typed_words: 0.,
      typed_chars: 0,
      accuracy: 0.,
      log_file: std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(LOGFILE_PATH)
        .ok(),
      letters: std::collections::HashMap::new(),
    }
  }

  pub fn type_char(&mut self, c: char) {
    if self.start_time.is_none() {
      self.start_time = Some(std::time::Instant::now());
      self.last_type = Some(std::time::Instant::now());
    }

    if self.typed_text.len() < self.current_text.len() {
      self.typed_text.push(c);
      self.update_stats(true, c);
    }
  }

  pub fn del_char(&mut self) {
    if let Some(c) = self.typed_text.pop() {
      self.update_stats(false, c);
    };
  }

  pub fn get_slowest_letters(&self) -> std::vec::Vec<(char, u128)> {
    let mut vec: std::vec::Vec<(char, u128)> = self
      .letters
      .iter()
      .map(|(c, info)| {
        (
          c.clone(),
          if info.count == 0 {
            0
          } else {
            (info.duration / info.count as u32).as_millis()
          },
        )
      })
      .collect();
    vec.sort_by_key(|(_c, v)| std::cmp::Reverse(v.clone()));
    return vec;
  }

  pub fn get_most_error_letters(&self) -> std::vec::Vec<(char, usize)> {
    let mut vec: std::vec::Vec<(char, usize)> = self
      .letters
      .iter()
      .map(|(c, info)| (c.clone(), info.errors))
      .collect();
    vec.sort_by_key(|(_c, v)| std::cmp::Reverse(v.clone()));
    return vec;
  }
  
  pub fn get_cpm(&self) -> Option<f32> {
    if let Some(start_time) = self.start_time {
      let mins = start_time.elapsed().as_millis() as f32 / 1000. / 60.;
      Some(self.typed_chars as f32 / mins)
    } else {
      None
    }
  }

  pub fn get_wpm(&self) -> Option<f32> {
    if let Some(start_time) = self.start_time {
      let mins = start_time.elapsed().as_millis() as f32 / 1000. / 60.;
      Some(self.typed_words as f32 / mins)
    } else {
      None
    }
  }

  pub fn get_accuracy(&self) -> Option<f32> {
    if self.accuracy.is_nan() {
      None
    } else {
      Some(self.accuracy)
    }
  }

  pub fn end_run(&mut self) -> Option<()> {
    let now = std::time::SystemTime::now()
      .duration_since(std::time::SystemTime::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs();
    let acc = self.get_accuracy();
    let wpm = self.get_wpm();
    let cpm = self.get_cpm();

    if let Some(log_file) = &mut self.log_file {
      use std::io::Write;
      if let (Some(acc), Some(wpm), Some(cpm)) = (acc, wpm, cpm) {
        log_file
          .write_all(format!("{:?} {:?} {:?} {:?}\n", now, acc, wpm, cpm).as_bytes())
          .ok()
      } else {
        None
      }
    } else {
      None
    }
  }

  pub fn get_improvement(&self, scale_x: usize, scale_y: usize) -> Option<std::vec::Vec<usize>> {
    if let Some(raw_data) = self.get_raw_improvement() {
      let mut result = vec![];

      let mut max = 0.;
      for point in &raw_data {
        if point.cpm > max {
          max = point.cpm;
        }
      }

      let data_len = raw_data.len();
      for i in 0..scale_x {
        if data_len == scale_x {
          // exact
          let cpm = raw_data[i].cpm;
          result.push((cpm / max * (scale_y as f32)) as usize);
        } else if data_len < scale_x {
          // interpolate
          let f_idx = i as f32 / (scale_x - 1) as f32 * (data_len - 1) as f32;
          let c1 = raw_data[f_idx.floor() as usize].cpm;
          let c2 = raw_data[f_idx.ceil() as usize].cpm;

          let dist = f_idx - f_idx.floor();
          let cpm = (1. - dist) * c1 + dist * c2;

          result.push((cpm / max * (scale_y as f32)) as usize);
        } else {
          // average
          let idx1 = (std::cmp::max(0, i as i32 - 1) as f32 / (scale_x - 1) as f32
            * (data_len - 1) as f32) as usize;
          let mut idx2 = (std::cmp::min(scale_x - 1, i + 1) as f32 / (scale_x - 1) as f32
            * (data_len - 1) as f32)
            .ceil() as usize;

          if idx2 >= data_len {
            idx2 = data_len - 1;
          }

          let mut sum = 0.;
          for i in idx1..idx2 + 1 {
            sum += raw_data[i].cpm;
          }

          let cpm = sum / (idx2 + 1 - idx1) as f32;

          result.push((cpm / max * (scale_y as f32)) as usize);
        }
      }

      Some(result)
    } else {
      None
    }
  }

  pub fn get_text_parts(&self) -> std::vec::Vec<&str> {
    let mut result = vec![];

    let mut current_right = true;
    let mut start = 0;
    let mut text_iter = self.current_text.char_indices();
    let mut last_idx = -1;
    for (_, c) in self.typed_text.char_indices() {
      let next = text_iter.next();
      if let Some((text_i, text_char)) = next {
        last_idx = text_i as i32;
        if current_right && c != text_char {
          let bound = Self::get_next_boundary(&self.current_text, text_i);
          result.push(&self.current_text[start..bound]);
          start = bound;
          current_right = false;
        }

        if !current_right && c == text_char {
          let bound = Self::get_next_boundary(&self.current_text, text_i);
          result.push(&self.current_text[start..bound]);
          start = bound;
          current_right = true;
        }
      }
    }
    let end = Self::get_next_boundary(&self.current_text, (last_idx + 1) as usize);
    result.push(&self.current_text[start..end]);
    result.push(&self.current_text[end..]);
    result
  }

  fn update_stats(&mut self, has_inserted: bool, last_typed: char) {
    self.typed_chars = 0;
    self.typed_words = 0.;
    let mut in_word = false;
    let mut text_iter = self.current_text.chars();
    let mut curr_word_chars = 0;
    let mut curr_word_correct = 0;
    let mut total_correct = 0;
    let mut last_typed_real = '\0';
    for typed in self.typed_text.chars() {
      let text_next = text_iter.next();
      if let Some(text_char) = text_next {
        curr_word_chars += 1;
        if typed == text_char {
          self.typed_chars += 1;
          curr_word_correct += 1;
          total_correct += 1;
        }
        if in_word && !text_char.is_alphanumeric() {
          self.typed_words += curr_word_correct as f32 / curr_word_chars as f32;
          in_word = false;
        }
        if !in_word && text_char.is_alphanumeric() {
          in_word = true;
        }
        last_typed_real = text_char;
      }
    }

    if has_inserted {
      assert_ne!(last_typed_real, '\0');
      let info = self.letters.entry(last_typed_real).or_insert(LetterInfo {
        duration: std::time::Duration::from_secs(0),
        count: 0,
        errors: 0,
      });
      if last_typed == last_typed_real {
        info.count += 1;
        info.duration += std::time::Instant::now().duration_since(self.last_type.unwrap());
      } else {
        info.errors += 1;
      }
      self.last_type = Some(std::time::Instant::now());
    } else {
      last_typed_real = text_iter.next().unwrap();

      if last_typed == last_typed_real {
        let info = self.letters.get_mut(&last_typed_real).unwrap();
        info.count -= 1;
      }
    }

    self.accuracy = total_correct as f32 / self.typed_text.len() as f32;
  }

  fn get_raw_improvement(&self) -> Option<std::vec::Vec<DataPoint>> {
    let mut result = vec![];

    let log_file = std::fs::File::open(LOGFILE_PATH);
    if let Ok(log_file) = log_file {
      let reader = std::io::BufReader::new(log_file);
      use std::io::BufRead;
      for line in reader.lines() {
        if let Ok(line) = line {
          let vec: std::vec::Vec<&str> = line.split(' ').collect();

          if let (Some(time), Some(acc), Some(wpm), Some(cpm)) =
            (vec.get(0), vec.get(1), vec.get(2), vec.get(3))
          {
            if let (Ok(time), Ok(acc), Ok(wpm), Ok(cpm)) =
              (time.parse(), acc.parse(), wpm.parse(), cpm.parse())
            {
              result.push(DataPoint {
                cpm: cpm,
                _time: time,
                _accuracy: acc,
                _wpm: wpm,
              });
            } else {
              return None;
            }
          } else {
            return None;
          }
        } else {
          return None;
        }
      }
      Some(result)
    } else {
      None
    }
  }

  fn get_next_boundary(text: &str, i: usize) -> usize {
    let mut end = i;
    while !text.is_char_boundary(end) {
      end += 1;
    }
    return end;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn basic_parts() {
    let mut t = TextManager::new(String::from("Hello, world!"));

    t.type_char('H');
    t.type_char('e');
    t.type_char('l');

    t.type_char('x');

    t.type_char('o');

    assert_eq!(t.get_text_parts(), vec!["Hel", "l", "o", ", world!"]);
  }

  #[test]
  fn basic_del() {
    let mut t = TextManager::new(String::from("Hello, world!"));

    t.type_char('H');
    t.type_char('e');

    t.type_char('l');
    t.del_char();

    t.type_char('l');

    t.type_char('x');
    t.del_char();

    assert_eq!(t.get_text_parts(), vec!["Hel", "lo, world!"]);
  }

  #[test]
  fn unicode_parts() {
    let mut t = TextManager::new(String::from("Здравей, свят!"));

    t.type_char('З');
    t.type_char('д');
    t.type_char('р');

    t.type_char('ь');

    t.type_char('в');

    assert_eq!(t.get_text_parts(), vec!["Здр", "а", "в", "ей, свят!"]);
  }

  #[test]
  fn unicode_parts_mixed() {
    let mut t = TextManager::new(String::from("Здравей, свят!"));

    t.type_char('З');

    t.type_char('d');
    t.type_char('r');
    t.type_char('ь');

    t.type_char('в');

    let parts = t.get_text_parts();
    assert_eq!(parts, vec!["З", "дра", "в", "ей, свят!"]);
  }

  #[test]
  fn unicode_parts_mixed_reverse() {
    let mut t = TextManager::new(String::from("Hello, world!"));

    t.type_char('H');

    t.type_char('б');
    t.type_char('l');
    t.type_char('l');

    t.type_char('ь');
    t.type_char(',');

    let parts = t.get_text_parts();
    assert_eq!(parts, vec!["H", "e", "ll", "o", ",", " world!"]);
  }

  #[test]
  fn parts_empty() {
    let t = TextManager::new(String::from("Hello"));

    assert_eq!(t.get_text_parts(), vec!["", "Hello"]);
  }

  #[test]
  fn error_letters() {
    let mut t = TextManager::new(String::from("Hello world!"));

    t.type_char('H');
    t.type_char('x');
    t.type_char('l');
    t.type_char('l');
    t.type_char('x');
    t.type_char(' ');
    t.type_char('w');
    t.type_char('x');

    assert_eq!(t.get_most_error_letters()[..2], vec![('o', 2), ('e', 1)]);
  }

  #[test]
  fn slowest_letters() {
    let mut t = TextManager::new(String::from("Hello world!"));

    t.type_char('H');
    std::thread::sleep(std::time::Duration::from_millis(100));
    t.type_char('e');
    std::thread::sleep(std::time::Duration::from_millis(100));
    t.type_char('l');
    std::thread::sleep(std::time::Duration::from_millis(300));
    t.type_char('l');
    std::thread::sleep(std::time::Duration::from_millis(250));
    t.type_char('o');
    t.type_char(' ');
    t.type_char('w');
    std::thread::sleep(std::time::Duration::from_millis(250));
    t.type_char('o');

    let letters = t.get_slowest_letters();

    assert_eq!(letters[0].0, 'o');
    assert_eq!(letters[1].0, 'l');
    assert_eq!(letters[2].0, 'e');
  }

  #[test]
  fn slowest_letters_retype() {
    let mut t = TextManager::new(String::from("Hello"));

    t.type_char('H');

    std::thread::sleep(std::time::Duration::from_millis(100));
    t.type_char('e');
    t.del_char();
    t.type_char('e');
    t.del_char();
    t.type_char('e');
    t.del_char();
    t.type_char('e');
    t.del_char();
    t.type_char('e');

    std::thread::sleep(std::time::Duration::from_millis(50));
    t.type_char('l');

    let letters = t.get_slowest_letters();

    assert_eq!(letters[0].0, 'e');
    assert_eq!(letters[1].0, 'l');
    assert_eq!(letters[2].0, 'H');
  }

  #[test]
  fn accuracy() {
    let mut t = TextManager::new(String::from("Hello"));

    t.type_char('H');
    t.type_char('x');
    t.del_char();
    t.type_char('e');
    t.type_char('l');
    t.type_char('x');
    t.type_char('x');

    let acc = t.get_accuracy().unwrap();
    assert!((acc - 0.6).abs() < 0.0001);
  }
}
