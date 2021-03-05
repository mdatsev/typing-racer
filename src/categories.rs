const DEFAULT_TEXT: &str = include_str!("default_text");

pub struct Categories {
  texts_dir: String,
}

impl Categories {
  pub fn new(texts_dir: String) -> Categories {
    Categories { texts_dir }
  }

  pub fn get_categories(&self) -> std::vec::Vec<String> {
    let mut categories = vec![];
    if let Ok(entries) = std::fs::read_dir(&self.texts_dir) {
      for entry in entries {
        if let Ok(entry) = entry {
          if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
              categories.push(String::from(entry.file_name().to_string_lossy()));
            }
          }
        }
      }
    }
    return categories;
  }
  pub fn get_text(&self, category: &str) -> String {
    use std::path::Path;
    let category_path = Path::new(&self.texts_dir).join(Path::new(category));
    let mut file_paths = vec![];
    if let Ok(entries) = std::fs::read_dir(category_path.clone()) {
      for entry in entries {
        if let Ok(entry) = entry {
          if let Ok(file_type) = entry.file_type() {
            if file_type.is_file() {
              file_paths.push(entry.file_name());
            }
          }
        }
      }
    }

    use rand::seq::SliceRandom;
    if let Some(file_path) = file_paths.choose(&mut rand::thread_rng()) {
      if let Ok(text) = std::fs::read_to_string(category_path.join(Path::new(file_path))) {
        return text;
      }
    }

    return String::from(DEFAULT_TEXT);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn get_categories() {
    let dir = String::from("test_4u29034j0we"); // random
    std::fs::create_dir(dir.clone()).unwrap();
    std::fs::create_dir(dir.clone() + "/cat1").unwrap();
    std::fs::create_dir(dir.clone() + "/cat2").unwrap();

    let c = Categories::new(dir.clone());

    assert_eq!(c.get_categories(), vec!["cat1", "cat2"]);

    std::fs::remove_dir_all(dir).unwrap();
  }

  #[test]
  fn get_text() {
    let dir = String::from("test_239a3ef99"); // random
    std::fs::create_dir(dir.clone()).unwrap();
    std::fs::create_dir(dir.clone() + "/cat1").unwrap();
    std::fs::create_dir(dir.clone() + "/cat2").unwrap();

    let mut file = std::fs::OpenOptions::new()
      .write(true)
      .truncate(true)
      .create(true)
      .open(dir.clone() + "/cat2/test")
      .unwrap();
    use std::io::Write;
    file.write_all("TestContent".as_bytes()).unwrap();
    let c = Categories::new(dir.clone());

    assert_eq!(c.get_text("cat2"), "TestContent");

    std::fs::remove_dir_all(dir).unwrap();
  }
}
