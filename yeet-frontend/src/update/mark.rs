use crate::model::mark::Marks;

pub fn print(marks: &Marks) -> Vec<String> {
    let mut marks: Vec<_> = marks
        .entries
        .iter()
        .map(|(key, path)| (key, path.to_string_lossy().to_string()))
        .map(|(key, path)| format!("{:<4} {}", key, path))
        .collect();

    marks.sort();

    let mut contents = vec![":marks".to_string(), "Char Content".to_string()];
    contents.extend(marks);

    contents
}
