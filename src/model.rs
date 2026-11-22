use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub subject: String,
    pub date: String,
    pub minutes: u32,
    pub questions: u32,
    pub weekly: bool,
    pub done: Vec<String>,
    #[serde(default)]
    pub skipped: Vec<String>,
    #[serde(default)]
    pub topic_id: Option<String>,
}
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
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
    #[serde(default)]
    pub id: String,
    pub date: String,
    pub kind: String,
    pub rows: Vec<(String, u32, u32, u32)>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Session {
    #[serde(default)]
    pub id: String,
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
    #[serde(default)]
    pub topic_progress: Vec<crate::learning::TopicProgress>,
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
            topic_progress: vec![],
        }
    }
}
pub fn occurs(t: &Task, date: &str) -> bool {
    valid_date(&t.date)
        && valid_date(date)
        && (t.done.iter().any(|day| day == date)
            || (!t.skipped.iter().any(|day| day == date)
                && (t.date == date
                    || (t.weekly
                        && date >= t.date.as_str()
                        && weekday_index(&t.date) == weekday_index(date)))))
}
/// Move one unfinished occurrence. Weekly recurrence and completion history stay intact.
pub fn move_task(
    tasks: &mut Vec<Task>,
    task_id: &str,
    from: &str,
    to: &str,
) -> Result<(), &'static str> {
    if !valid_date(from) || !valid_date(to) {
        return Err("Geçerli bir tarih seç.");
    }
    if from == to {
        return Err("Farklı bir gün seç.");
    }
    let index = tasks
        .iter()
        .position(|task| task.id == task_id)
        .ok_or("Çalışma artık mevcut değil.")?;
    let source = &tasks[index];
    if !occurs(source, from) {
        return Err("Bu gün için taşınabilecek çalışma kalmadı.");
    }
    if source.done.iter().any(|day| day == from) {
        return Err("Tamamlanan çalışma taşınamaz.");
    }
    if source.weekly {
        let new_id = format!("moved@{}@{}", source.id, from);
        if new_id.is_empty() || tasks.iter().any(|task| task.id == new_id) {
            return Err("Yeni çalışma oluşturulamadı.");
        }
        let mut moved = source.clone();
        moved.id = new_id;
        moved.date = to.into();
        moved.weekly = false;
        moved.done.clear();
        moved.skipped.clear();
        tasks[index].skipped.push(from.into());
        tasks.push(moved);
    } else {
        tasks[index].date = to.into();
    }
    Ok(())
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
pub fn edit_card(
    cards: &mut [Card],
    card_id: &str,
    deck: &str,
    front: &str,
    back: &str,
) -> Result<(), &'static str> {
    let (deck, front, back) = (deck.trim(), front.trim(), back.trim());
    if deck.is_empty() || front.is_empty() || back.is_empty() {
        return Err("Deste, soru ve cevap boş olamaz.");
    }
    if deck.chars().count() > 80 || front.chars().count() > 2000 || back.chars().count() > 4000 {
        return Err("Kart metni izin verilen uzunluğu aşıyor.");
    }
    let card = cards
        .iter_mut()
        .find(|c| c.id == card_id)
        .ok_or("Bu kart başka bir pencerede silinmiş.")?;
    card.deck = deck.into();
    card.front = front.into();
    card.back = back.into();
    Ok(())
}
#[derive(Default, Clone, Debug)]
pub struct SubjectSummary {
    pub planned: u32,
    pub completed: u32,
    pub focus_seconds: u64,
}
#[derive(Default, Clone, Debug)]
pub struct WeekSummary {
    pub planned: u32,
    pub completed: u32,
    pub questions: u64,
    pub focus_seconds: u64,
    pub subjects: std::collections::BTreeMap<String, SubjectSummary>,
}
pub fn summarize_week(state: &State, days: &[String]) -> WeekSummary {
    let mut result = WeekSummary::default();
    for day in days {
        for task in &state.tasks {
            if occurs(task, day) {
                result.planned += 1;
                let subject = result.subjects.entry(task.subject.clone()).or_default();
                subject.planned += 1;
                if task.done.contains(day) {
                    result.completed += 1;
                    subject.completed += 1;
                    result.questions += task.questions as u64;
                }
            }
        }
    }
    for session in &state.sessions {
        if days.contains(&session.date) {
            result.focus_seconds += session.seconds as u64;
            result
                .subjects
                .entry(session.subject.clone())
                .or_default()
                .focus_seconds += session.seconds as u64;
        }
    }
    result
}
pub fn exam_trend(state: &State, kind: &str, start: &str, end: &str) -> Option<(f64, Option<f64>)> {
    let mut exams = state
        .exams
        .iter()
        .enumerate()
        .filter(|(_, exam)| exam.kind == kind && exam.date.as_str() <= end)
        .collect::<Vec<_>>();
    exams.sort_by_key(|(i, e)| (&e.date, *i));
    let (_, latest) = exams.pop()?;
    if latest.date.as_str() < start {
        return None;
    }
    let total = |e: &Exam| e.rows.iter().map(|(_, c, w, _)| net(*c, *w)).sum::<f64>();
    let signature = |e: &Exam| {
        let mut rows = e
            .rows
            .iter()
            .map(|(name, _, _, max)| (name.clone(), *max))
            .collect::<Vec<_>>();
        rows.sort();
        rows
    };
    let previous = exams
        .into_iter()
        .rev()
        .find(|(_, e)| signature(e) == signature(latest));
    Some((
        total(latest),
        previous.map(|(_, e)| total(latest) - total(e)),
    ))
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
                && t.skipped.iter().all(|d| valid_date(d))
                && t.topic_id
                    .as_ref()
                    .map(|id| !id.is_empty() && id.len() <= 180)
                    .unwrap_or(true)
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
        && unique_ids(s.exams.iter().map(|e| e.id.as_str()))
        && unique_ids(s.sessions.iter().map(|s| s.id.as_str()))
        && s.topic_progress.len() <= 50000
        && s.topic_progress
            .iter()
            .all(|p| !p.id.is_empty() && p.id.len() <= 180)
        && unique_ids(s.topic_progress.iter().map(|p| p.id.as_str()))
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
    fn study(weekly: bool) -> Task {
        Task {
            id: "task".into(),
            title: "Çalışma".into(),
            subject: "TYT Matematik".into(),
            date: "2026-11-23".into(),
            minutes: 45,
            questions: 20,
            weekly,
            done: vec![],
            skipped: vec![],
            topic_id: None,
        }
    }
    #[test]
    fn empty_week_has_no_fabricated_progress_or_exam_score() {
        let state = State::default();
        let s = summarize_week(&state, &["2026-11-23".into()]);
        assert_eq!(
            (s.planned, s.completed, s.questions, s.focus_seconds),
            (0, 0, 0, 0)
        );
        assert!(s.subjects.is_empty());
        assert_eq!(exam_trend(&state, "TYT", "2026-11-23", "2026-11-29"), None);
    }
    #[test]
    fn editing_card_preserves_schedule_and_rejects_invalid_or_deleted_cards() {
        let original = Card {
            id: "card".into(),
            deck: "Fen".into(),
            front: "Soru".into(),
            back: "Cevap".into(),
            due: "2026-12-10".into(),
            interval: 14,
        };
        let mut cards = vec![original.clone()];
        edit_card(
            &mut cards,
            "card",
            "  Biyoloji ",
            " Yeni soru ",
            " Yeni cevap ",
        )
        .unwrap();
        assert_eq!(cards[0].deck, "Biyoloji");
        assert_eq!(cards[0].due, original.due);
        assert_eq!(cards[0].interval, 14);
        assert_eq!(cards[0].id, "card");
        let before = cards.clone();
        assert!(edit_card(&mut cards, "card", "", "Soru", "Cevap").is_err());
        assert!(edit_card(&mut cards, "card", "Fen", &"x".repeat(2001), "Cevap").is_err());
        assert!(edit_card(&mut cards, "deleted", "Fen", "Soru", "Cevap").is_err());
        assert_eq!(cards, before);
    }
    #[test]
    fn weekly_summary_counts_occurrences_goals_and_only_logged_time_in_window() {
        let mut state = State::default();
        let mut task = study(true);
        task.done = vec!["2026-11-23".into(), "2026-11-30".into()];
        state.tasks.push(task);
        state.sessions = vec![
            Session {
                id: "a".into(),
                date: "2026-11-23".into(),
                subject: "TYT Matematik".into(),
                seconds: 120,
            },
            Session {
                id: "b".into(),
                date: "2026-11-30".into(),
                subject: "TYT Matematik".into(),
                seconds: 999,
            },
        ];
        let days = (23..=29)
            .map(|d| format!("2026-11-{d}"))
            .collect::<Vec<_>>();
        let s = summarize_week(&state, &days);
        assert_eq!(
            (s.planned, s.completed, s.questions, s.focus_seconds),
            (1, 1, 20, 120)
        );
        assert_eq!(s.subjects["TYT Matematik"].completed, 1);
        state.tasks[0].done.retain(|d| d != "2026-11-30");
        move_task(&mut state.tasks, "task", "2026-11-30", "2026-12-01").unwrap();
        let s = summarize_week(&state, &["2026-11-30".into(), "2026-12-01".into()]);
        assert_eq!((s.planned, s.completed), (1, 0));
    }
    #[test]
    fn net_trend_excludes_future_dates_and_incompatible_exam_structures() {
        let make = |id: &str, date: &str, name: &str, correct: u32| Exam {
            id: id.into(),
            date: date.into(),
            kind: "TYT".into(),
            rows: vec![(name.into(), correct, 0, 40)],
        };
        let mut s = State::default();
        s.exams = vec![
            make("latest", "2026-11-25", "Türkçe", 35),
            make("future", "2026-12-01", "Türkçe", 40),
            make("old", "2026-11-20", "Türkçe", 30),
            make("different", "2026-11-24", "Matematik", 10),
        ];
        assert_eq!(
            exam_trend(&s, "TYT", "2026-11-23", "2026-11-29"),
            Some((35., Some(5.)))
        );
        assert_eq!(
            exam_trend(&s, "AYT / YDT", "2026-11-23", "2026-11-29"),
            None
        );
        assert_eq!(exam_trend(&s, "TYT", "2026-12-07", "2026-12-13"), None);
    }
    #[test]
    fn changing_schedule_does_not_hide_completed_history() {
        let mut task = study(true);
        task.done.push("2026-11-23".into());
        task.date = "2026-12-01".into();
        assert!(occurs(&task, "2026-11-23"));
        assert!(!occurs(&task, "2026-11-30"));
    }
    #[test]
    fn move_once_changes_date_without_duplicating_or_resetting_targets() {
        let mut tasks = vec![study(false)];
        move_task(&mut tasks, "task", "2026-11-23", "2026-11-24").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].questions, 20);
        assert!(occurs(&tasks[0], "2026-11-24"));
        assert!(!occurs(&tasks[0], "2026-11-23"));
    }
    #[test]
    fn move_weekly_only_skips_the_selected_occurrence_and_keeps_history() {
        let mut tasks = vec![study(true)];
        tasks[0].done.push("2026-11-23".into());
        move_task(&mut tasks, "task", "2026-11-30", "2026-12-01").unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(occurs(&tasks[0], "2026-11-23"));
        assert!(!occurs(&tasks[0], "2026-11-30"));
        assert!(occurs(&tasks[0], "2026-12-07"));
        assert!(occurs(&tasks[1], "2026-12-01"));
        assert!(!tasks[1].weekly);
        assert!(tasks[1].done.is_empty());
        let before = serde_json::to_string(&tasks).unwrap();
        assert!(move_task(&mut tasks, "task", "2026-11-30", "2026-12-02").is_err());
        assert_eq!(serde_json::to_string(&tasks).unwrap(), before);
    }
    #[test]
    fn completed_or_invalid_moves_leave_the_original_untouched() {
        let mut tasks = vec![study(false)];
        tasks[0].done.push("2026-11-23".into());
        let before = serde_json::to_string(&tasks).unwrap();
        for (from, to) in [
            ("2026-11-23", "2026-11-24"),
            ("2026-11-23", ""),
            ("2026-11-23", "2026-11-23"),
            ("2026-11-24", "2026-11-25"),
        ] {
            assert!(move_task(&mut tasks, "task", from, to).is_err());
            assert_eq!(serde_json::to_string(&tasks).unwrap(), before);
        }
    }
    #[test]
    fn schema_v1_tasks_without_exceptions_remain_readable() {
        let mut old = serde_json::to_value(study(true)).unwrap();
        old.as_object_mut().unwrap().remove("skipped");
        let loaded: Task = serde_json::from_value(old).unwrap();
        assert!(loaded.skipped.is_empty());
        assert!(occurs(&loaded, "2026-11-30"));
    }
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
            skipped: vec![],
            topic_id: None,
        };
        assert!(!occurs(&t, "2026-12-21"));
        assert!(occurs(&t, "2026-12-28"));
        assert!(occurs(&t, "2027-01-04"));
        assert!(!occurs(&t, "2027-01-05"));
        assert!(!occurs(&t, ""));
        assert!(!occurs(&t, "99999-01-01"));
        assert!(!occurs(&t, "2027-02-31"));
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
