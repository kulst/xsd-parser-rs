extern crate inflector;
use inflector::cases::pascalcase::to_pascal_case;
use std::str;
use self::inflector::cases::snakecase::to_snake_case;

fn split_comment_line(s: &str) -> String {
    s.as_bytes()
    .chunks(60)
    .map(|ch| format!("// {}\n", str::from_utf8(ch).unwrap()))
    .collect::<Vec<String>>()
    .join("")
}

pub fn get_structure_comment(doc: Option<&str>) -> String {
    doc.
        unwrap_or("").
        lines().
        map(|s| s.trim()).
        filter(|s| s.len() > 2).
        map(|s| split_comment_line(s)).
        fold(String::new(), |x , y| (x+&y))
}

pub fn get_field_comment(doc: Option<&str>) -> String {
    doc.
        unwrap_or("").
        lines().
        map(|s| s.trim()).
        filter(|s| s.len() > 1).
        map(|s| format!("// {}  ", s)).
        fold(String::new(), |x , y| (x+&y))
}

pub fn get_type_name(name: &str) -> String {
    to_pascal_case(name)
}

pub fn get_field_name(name: &str) -> String {
    to_snake_case(name)
}

pub(crate) fn yaserde_derive() -> String {
    "#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]\n\
        #[yaserde(\n\
          prefix = \"unknown\",\n\
          namespace = \"unknown: unknown\"\n\
        )\n".to_string()
}