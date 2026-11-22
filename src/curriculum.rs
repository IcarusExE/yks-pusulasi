use serde::Deserialize;
use std::sync::LazyLock;
#[derive(Clone, Debug, Deserialize)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub start_page: u32,
    pub end_page: u32,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Topic {
    pub id: String,
    pub course: String,
    pub title: String,
    pub page: u32,
    pub level: String,
    pub code: String,
}
#[derive(Debug, Deserialize)]
pub struct Catalogue {
    pub year: u32,
    pub checked_on: String,
    pub source_url: String,
    pub source_sha256: String,
    pub source_pages: u32,
    pub granularity: String,
    pub courses: Vec<Course>,
    pub topics: Vec<Topic>,
}
static CATALOGUE: LazyLock<Catalogue> = LazyLock::new(|| {
    let data: Catalogue = serde_json::from_str(include_str!("../data/curriculum-2026.json"))
        .expect("validated curriculum catalogue");
    assert_eq!(data.year, 2026);
    assert_eq!(data.source_pages, 581);
    assert_eq!(
        data.source_sha256,
        "f52502743b6372e5df7de42d741b02a0b18eec087eac785b50a1fe3dd4a6d536"
    );
    for course in &data.courses {
        assert!(course.start_page <= course.end_page && course.end_page <= data.source_pages);
    }
    data
});

pub fn catalogue() -> &'static Catalogue {
    &CATALOGUE
}
pub fn fold(value: &str) -> String {
    value.replace('İ', "i").replace('I', "ı").to_lowercase()
}
pub fn matching(course: &str, query: &str) -> Vec<&'static Topic> {
    let query = fold(query);
    catalogue()
        .topics
        .iter()
        .filter(|t| {
            t.course == course
                && fold(&format!("{} {} {}", t.title, t.level, t.code)).contains(&query)
        })
        .collect()
}
pub fn course_name(id: &str) -> String {
    catalogue()
        .courses
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.name.clone())
        .unwrap_or_default()
}
pub fn default_lesson(course: &str) -> &'static str {
    match course {
        "math" => "TYT Matematik",
        "literature" => "TYT Türkçe",
        "physics" | "chemistry" | "biology" => "TYT Fen",
        "english" | "german" | "french" | "arabic" | "russian" => "YDT Yabancı Dil",
        _ => "TYT Sosyal",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_and_all_eighteen_course_sections_are_preserved() {
        let c = catalogue();
        assert_eq!(c.year, 2026);
        assert_eq!(c.source_pages, 581);
        assert_eq!(c.courses.len(), 18);
        assert_eq!(c.topics.len(), 867);
        assert_eq!(c.source_sha256.len(), 64);
        let mut ids = std::collections::HashSet::new();
        for topic in &c.topics {
            assert!(ids.insert(&topic.id));
            let course = c.courses.iter().find(|x| x.id == topic.course).unwrap();
            assert!((course.start_page..=course.end_page).contains(&topic.page));
            assert!(!topic.title.chars().any(char::is_control));
            assert!(!topic.title.is_empty());
        }
        for course in &c.courses {
            assert!(c.topics.iter().any(|t| t.course == course.id));
        }
    }
    #[test]
    fn mathematical_topics_include_basic_level_and_calculus() {
        let math = matching("math", "");
        for code in ["9.3.3", "12.5.1", "12.6.2", "TD.11.1.1"] {
            assert!(math.iter().any(|t| t.code == code), "{code}");
        }
    }
    #[test]
    fn search_handles_turkish_case_and_does_not_cross_courses() {
        assert!(!matching("physics", "IŞIN").is_empty());
        assert!(!matching("chemistry", "İDEAL").is_empty());
        assert!(matching("math", "kimya").is_empty());
        assert_eq!(matching("english", "").len(), 40);
    }
}
