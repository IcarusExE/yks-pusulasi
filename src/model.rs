use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub date: String,
    pub minutes: u32,
    pub questions: u32,
    pub weekly: bool,
    pub done: Vec<String>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Card {
    pub id: String,
    pub deck: String,
    pub front: String,
    pub back: String,
    pub due: String,
    pub interval: u32,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Exam {
    pub date: String,
    pub kind: String,
    pub rows: Vec<(String, u32, u32, u32)>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Session {
    pub date: String,
    pub subject: String,
    pub seconds: u32,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Timer {
    pub running: bool,
    pub started: f64,
    pub elapsed: u32,
    pub mode: String,
    pub subject: String,
}
impl Default for Timer {
    fn default() -> Self {
        Self {
            running: false,
            started: 0.,
            elapsed: 0,
            mode: "Kronometre".into(),
            subject: "TYT Matematik".into(),
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct State {
    pub version: u32,
    pub name: String,
    pub track: String,
    pub palette: String,
    pub dark: bool,
    pub tasks: Vec<Task>,
    pub cards: Vec<Card>,
    pub exams: Vec<Exam>,
    pub sessions: Vec<Session>,
    pub timer: Timer,
}
impl Default for State {
    fn default() -> Self {
        Self {
            version: 1,
            name: String::new(),
            track: "Sayısal".into(),
            palette: "forest".into(),
            dark: false,
            tasks: vec![],
            cards: vec![],
            exams: vec![],
            sessions: vec![],
            timer: Timer::default(),
        }
    }
}
pub fn occurs(t: &Task, date: &str) -> bool {
    t.date == date
        || (t.weekly && date >= t.date.as_str() && weekday_index(&t.date) == weekday_index(date))
}
fn weekday_index(date: &str) -> u32 {
    let mut y = date[0..4].parse::<u32>().unwrap();
    let m = date[5..7].parse::<u32>().unwrap();
    let d = date[8..10].parse::<u32>().unwrap();
    let offsets = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    if m < 3 {
        y -= 1;
    }
    (y + y / 4 - y / 100 + y / 400 + offsets[(m - 1) as usize] + d) % 7
}
pub fn net(correct: u32, wrong: u32) -> f64 {
    correct as f64 - wrong as f64 / 4.
}
pub fn elapsed_at(t: &Timer, now: f64) -> u32 {
    t.elapsed.saturating_add(if t.running {
        ((now - t.started).max(0.) / 1000.) as u32
    } else {
        0
    })
}
pub fn clock(seconds: u32) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}
pub fn subjects(track: &str) -> Vec<&'static str> {
    let mut s = vec!["TYT Türkçe", "TYT Matematik", "TYT Sosyal", "TYT Fen"];
    s.extend(match track {
        "Eşit ağırlık" => vec![
            "AYT Matematik",
            "AYT Edebiyat",
            "AYT Tarih-1",
            "AYT Coğrafya-1",
        ],
        "Sözel" => vec![
            "AYT Edebiyat",
            "AYT Tarih",
            "AYT Coğrafya",
            "AYT Felsefe",
            "AYT Din",
        ],
        "Dil" => vec!["YDT Yabancı Dil"],
        _ => vec!["AYT Matematik", "AYT Fizik", "AYT Kimya", "AYT Biyoloji"],
    });
    s
}
pub fn exam_subjects(kind: &str, track: &str) -> Vec<(&'static str, u32)> {
    if kind == "TYT" {
        vec![
            ("Türkçe", 40),
            ("Sosyal Bilimler", 20),
            ("Temel Matematik", 40),
            ("Fen Bilimleri", 20),
        ]
    } else {
        match track {
            "Eşit ağırlık" => vec![
                ("Matematik", 40),
                ("Edebiyat", 24),
                ("Tarih-1", 10),
                ("Coğrafya-1", 6),
            ],
            "Sözel" => vec![
                ("Edebiyat", 24),
                ("Tarih-1", 10),
                ("Coğrafya-1", 6),
                ("Tarih-2", 11),
                ("Coğrafya-2", 11),
                ("Felsefe Grubu", 12),
                ("Din Kültürü", 6),
            ],
            "Dil" => vec![("Yabancı Dil", 80)],
            _ => vec![
                ("Matematik", 40),
                ("Fizik", 14),
                ("Kimya", 13),
                ("Biyoloji", 13),
            ],
        }
    }
}
pub fn valid(s: &State) -> bool {
    s.version == 1
        && ["Sayısal", "Eşit ağırlık", "Sözel", "Dil"].contains(&s.track.as_str())
        && ["forest", "ocean", "terracotta", "plum"].contains(&s.palette.as_str())
        && s.name.len() <= 160
        && s.tasks.len() <= 50000
        && s.cards.len() <= 50000
        && s.tasks.iter().all(|t| {
            valid_date(&t.date)
                && t.minutes > 0
                && t.minutes <= 600
                && t.questions <= 1000
                && !t.title.trim().is_empty()
                && t.title.len() <= 640
                && t.done.iter().all(|d| valid_date(d))
        })
        && s.cards.iter().all(|c| {
            valid_date(&c.due)
                && c.interval <= 365
                && !c.front.trim().is_empty()
                && !c.back.trim().is_empty()
        })
        && s.exams.iter().all(|e| {
            valid_date(&e.date)
                && e.rows
                    .iter()
                    .all(|(_, c, w, max)| *max <= 120 && c.saturating_add(*w) <= *max)
        })
        && s.timer.started.is_finite()
        && s.timer.started >= 0.
        && ["Kronometre", "Pomodoro"].contains(&s.timer.mode.as_str())
        && s.sessions.iter().all(|s| valid_date(&s.date))
        && unique_ids(s.tasks.iter().map(|t| t.id.as_str()))
        && unique_ids(s.cards.iter().map(|c| c.id.as_str()))
}
pub fn unique_ids<'a>(ids: impl Iterator<Item = &'a str>) -> bool {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter().all(|id| !id.is_empty() && seen.insert(id))
}
pub fn valid_date(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let y = s[0..4].parse::<u32>().unwrap_or(0);
    let m = s[5..7].parse::<u32>().unwrap_or(0);
    let d = s[8..10].parse::<u32>().unwrap_or(0);
    let max = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    y >= 2000 && d > 0 && d <= max
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weekly_tasks_do_not_appear_before_start() {
        let t = Task {
            id: "a".into(),
            title: "Test".into(),
            subject: "TYT".into(),
            date: "2026-12-28".into(),
            minutes: 45,
            questions: 20,
            weekly: true,
            done: vec![],
        };
        assert!(!occurs(&t, "2026-12-21"));
        assert!(occurs(&t, "2026-12-28"));
        assert!(occurs(&t, "2027-01-04"));
        assert!(!occurs(&t, "2027-01-05"));
    }
    #[test]
    fn timer_accounts_for_background_time_and_pause() {
        let t = Timer {
            running: true,
            started: 1000.,
            elapsed: 10,
            ..Default::default()
        };
        assert_eq!(elapsed_at(&t, 61000.), 70);
        assert_eq!(elapsed_at(&t, 0.), 10);
        let paused = Timer {
            running: false,
            ..t
        };
        assert_eq!(elapsed_at(&paused, 999999.), 10);
    }
    #[test]
    fn net_formula() {
        assert_eq!(net(32, 8), 30.);
        assert_eq!(net(0, 4), -1.);
        assert_eq!(net(40, 0), 40.);
    }
    #[test]
    fn exam_totals() {
        assert_eq!(
            exam_subjects("TYT", "Sayısal")
                .iter()
                .map(|(_, n)| n)
                .sum::<u32>(),
            120
        );
        for track in ["Sayısal", "Eşit ağırlık", "Sözel", "Dil"] {
            assert_eq!(
                exam_subjects("AYT", track)
                    .iter()
                    .map(|(_, n)| n)
                    .sum::<u32>(),
                80
            );
        }
    }
    #[test]
    fn backup_validation() {
        let mut s = State::default();
        assert!(valid(&s));
        s.version = 2;
        assert!(!valid(&s));
        assert!(!valid_date("2026-02-29"));
        assert!(valid_date("2028-02-29"));
        assert!(!valid_date("not a date"));
    }
    #[test]
    fn duration_display() {
        assert_eq!(clock(3661), "01:01:01");
    }
}
