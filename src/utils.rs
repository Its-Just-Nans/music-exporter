use std::path::PathBuf;

pub fn input(txt: &str, env_name: &str) -> Option<String> {
    use std::io::{stdin, stdout, Write};
    if let Ok(val) = std::env::var(env_name) {
        return Some(val);
    }
    let mut s = String::new();
    print!("{}", txt);
    println!("({} not found in the env)", env_name);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Some(s)
}

pub fn write_to_file(filename: &PathBuf, data: Vec<crate::Music>) {
    use serde::Serialize;
    use std::fs::File;
    use std::io::{BufWriter, Write};
    let file = File::create(filename).expect("Could not create file");
    let mut writer = BufWriter::new(file);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut writer, formatter);
    data.serialize(&mut ser).unwrap();
    writer.flush().unwrap();
}

pub fn read_from_file(filename: &PathBuf, items: &mut Vec<crate::Music>) {
    use std::fs::File;
    use std::io::BufReader;
    let file = File::create(filename).expect("Could not create file");
    let reader = BufReader::new(file);
    let mut de = serde_json::Deserializer::from_reader(reader);
    items.clear();
    items.extend(serde::de::Deserialize::deserialize(&mut de).unwrap_or(vec![]));
}
