pub fn capitalise_first(s: &str) -> String {
  if s.is_empty() {
    return String::new();
  }

  // Capitalising a character can sometimes result in multiple characters
  let mut first: String = s
    .chars()
    .next()
    .expect("The first character should always be present")
    .to_uppercase()
    .to_string();

  let maybe_tail_index = s.char_indices().nth(1).map(|(n, _)| n);

  match maybe_tail_index {
    None => first,
    Some(tail_index) => {
      let tail = &s[tail_index..];
      first.push_str(tail);
      first
    }
  }
}

#[cfg(test)]
mod tests {
  use super::capitalise_first;

  #[test]
  fn capitalise_first_basic() {
    assert_eq!(capitalise_first(""), String::from(""));
    assert_eq!(capitalise_first("a"), String::from("A"));
    assert_eq!(capitalise_first("B"), String::from("B"));
    assert_eq!(capitalise_first("aa"), String::from("Aa"));
    assert_eq!(capitalise_first("äe"), String::from("Äe"));
    assert_eq!(capitalise_first("😎"), String::from("😎"));
  }
}
