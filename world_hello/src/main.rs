fn main() {
  let string1 = String::from("long string is long");
  let result = first_word(&string1);
  println!("The first word is: {}", result);
}

fn first_word(s: &str) -> &str {
  let bytes = s.as_bytes();

  for (i, &item) in bytes.iter().enumerate() {
      if item == b' ' {
          return &s[0..i];
      }
  }

  &s[..]
}