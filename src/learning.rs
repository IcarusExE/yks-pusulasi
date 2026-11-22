use crate::{
    curriculum::{catalogue, course_name, Topic},
    model::{occurs, valid_date, State, Task},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TopicStatus {
    #[default]
    NotStarted,
    InProgress,
    NeedsReview,
    Completed,
}
impl TopicStatus {
    pub const ALL: [Self; 4] = [
        Self::NotStarted,
        Self::InProgress,
        Self::NeedsReview,
        Self::Completed,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::InProgress => "in_progress",
            Self::NeedsReview => "needs_review",
            Self::Completed => "completed",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Başlamadım",
            Self::InProgress => "Çalışıyorum",
            Self::NeedsReview => "Tekrar gerekli",
            Self::Completed => "Tamamladım",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|status| status.key() == s)
    }
    fn priority(self) -> u8 {
        match self {
            Self::NeedsReview => 0,
            Self::InProgress => 1,
            Self::NotStarted => 2,
            Self::Completed => 3,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicProgress {
    pub id: String,
    pub status: TopicStatus,
}
pub fn topic_key(topic: &Topic) -> String {
    format!("{}/{}", catalogue().year, topic.id)
}
pub fn find_topic(key: &str) -> Option<&'static Topic> {
    let prefix = format!("{}/", catalogue().year);
    let id = key.strip_prefix(&prefix)?;
    catalogue().topics.iter().find(|t| t.id == id)
}
pub fn status(state: &State, key: &str) -> TopicStatus {
    state
        .topic_progress
        .iter()
        .find(|p| p.id == key)
        .map(|p| p.status)
        .unwrap_or_default()
}
pub fn set_status(state: &mut State, key: &str, value: TopicStatus) -> Result<(), &'static str> {
    if find_topic(key).is_none() {
        return Err("Konu, mevcut kaynak kataloğunda bulunamadı.");
    }
    if let Some(progress) = state.topic_progress.iter_mut().find(|p| p.id == key) {
        progress.status = value;
    } else {
        state.topic_progress.push(TopicProgress {
            id: key.into(),
            status: value,
        });
    }
    Ok(())
}
pub fn topic_title(topic: &Topic) -> String {
    let title = format!(
        "{} · {} ({})",
        course_name(&topic.course),
        topic.title,
        topic.level
    );
    if title.chars().count() > 160 {
        title.chars().take(157).collect::<String>() + "…"
    } else {
        title
    }
}
pub fn is_pending(task: &Task) -> bool {
    task.weekly || (!task.done.contains(&task.date) && occurs(task, &task.date))
}
pub fn pending_topic(state: &State, key: &str) -> bool {
    state
        .tasks
        .iter()
        .any(|t| t.topic_id.as_deref() == Some(key) && is_pending(t))
}
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub items: Vec<Task>,
    pub daily_budget: u32,
    pub completed_skipped: usize,
    pub pending_skipped: usize,
    pub unassigned: usize,
}
pub fn propose(
    state: &State,
    keys: &[String],
    days: &[String],
    lesson: &str,
    daily_budget: u32,
    block: u32,
    questions: u32,
) -> Result<Proposal, &'static str> {
    if !(30..=360).contains(&daily_budget)
        || !(15..=120).contains(&block)
        || block > daily_budget
        || questions > 1000
        || lesson.trim().is_empty()
    {
        return Err("Süre ve soru hedeflerini kontrol et.");
    }
    if days.is_empty() || days.len() > 7 || days.iter().any(|d| !valid_date(d)) {
        return Err("En az bir geçerli çalışma günü seç.");
    }
    let mut days = days.to_vec();
    days.sort();
    days.dedup();
    let mut seen = std::collections::HashSet::new();
    let mut candidates = vec![];
    let mut result = Proposal {
        items: vec![],
        daily_budget,
        completed_skipped: 0,
        pending_skipped: 0,
        unassigned: 0,
    };
    for key in keys {
        if !seen.insert(key.clone()) {
            continue;
        }
        let topic = find_topic(key).ok_or("Konu kaynağı bulunamadı.")?;
        let progress = status(state, key);
        if progress == TopicStatus::Completed {
            result.completed_skipped += 1;
            continue;
        }
        if pending_topic(state, key) {
            result.pending_skipped += 1;
            continue;
        }
        candidates.push((topic, progress));
    }
    candidates.sort_by_cached_key(|(topic, progress)| {
        (
            progress.priority(),
            catalogue()
                .topics
                .iter()
                .position(|t| t.id == topic.id)
                .unwrap_or(usize::MAX),
        )
    });
    let eligible = candidates.len();
    let mut next = candidates.into_iter();
    for day in days {
        let occupied = state
            .tasks
            .iter()
            .filter(|t| occurs(t, &day))
            .map(|t| t.minutes as u64)
            .sum::<u64>();
        let mut remaining = (daily_budget as u64).saturating_sub(occupied);
        while remaining >= block as u64 {
            let Some((topic, _)) = next.next() else {
                break;
            };
            result.items.push(Task {
                id: String::new(),
                title: topic_title(topic),
                subject: lesson.trim().into(),
                date: day.clone(),
                minutes: block,
                questions,
                weekly: false,
                done: vec![],
                skipped: vec![],
                topic_id: Some(topic_key(topic)),
            });
            remaining -= block as u64;
        }
    }
    result.unassigned = eligible - result.items.len();
    Ok(result)
}
#[derive(Default, Debug)]
pub struct ApplyReport {
    pub added: usize,
    pub skipped: usize,
}
/// Recheck the latest UI state at confirmation. No schedule is replaced.
pub fn apply(
    state: &mut State,
    proposal: &Proposal,
    selected: &[String],
) -> Result<ApplyReport, &'static str> {
    if !(30..=360).contains(&proposal.daily_budget) {
        return Err("Taslağı yeniden oluştur.");
    }
    let mut staged: Vec<Task> = vec![];
    let mut report = ApplyReport::default();
    for item in &proposal.items {
        let Some(key) = item.topic_id.as_deref() else {
            return Err("Taslak konu bağlantısı eksik.");
        };
        if !selected.iter().any(|s| s == key) {
            continue;
        }
        if find_topic(key).is_none()
            || !valid_date(&item.date)
            || item.weekly
            || !(15..=120).contains(&item.minutes)
            || item.questions > 1000
            || item.title.trim().is_empty()
            || item.subject.trim().is_empty()
            || item.title.chars().count() > 160
            || item.subject.chars().count() > 160
            || !item.done.is_empty()
            || !item.skipped.is_empty()
        {
            return Err("Taslağı yeniden oluştur.");
        }
        if status(state, key) == TopicStatus::Completed
            || pending_topic(state, key)
            || staged.iter().any(|t| t.topic_id.as_deref() == Some(key))
        {
            report.skipped += 1;
            continue;
        }
        let occupied = state
            .tasks
            .iter()
            .chain(staged.iter())
            .filter(|t| occurs(t, &item.date))
            .map(|t| t.minutes as u64)
            .sum::<u64>();
        if occupied + item.minutes as u64 > proposal.daily_budget as u64 {
            report.skipped += 1;
            continue;
        }
        // Concurrent windows accepting the same learning cycle use one identity,
        // even if they selected different dates. Completed cycles get a new ID.
        let cycles = state
            .tasks
            .iter()
            .filter(|t| t.topic_id.as_deref() == Some(key) && !is_pending(t))
            .count();
        let mut task = item.clone();
        task.id = format!("topic-plan@{key}@{cycles}");
        if state.tasks.iter().any(|t| t.id == task.id) || staged.iter().any(|t| t.id == task.id) {
            report.skipped += 1;
            continue;
        }
        staged.push(task);
        report.added += 1;
    }
    state.tasks.extend(staged);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn keys(n: usize) -> Vec<String> {
        crate::curriculum::matching("math", "")
            .into_iter()
            .take(n)
            .map(topic_key)
            .collect()
    }
    fn existing(id: &str, date: &str, minutes: u32, key: Option<String>) -> Task {
        Task {
            id: id.into(),
            title: "Çalışma".into(),
            subject: "TYT Matematik".into(),
            date: date.into(),
            minutes,
            questions: 20,
            weekly: false,
            done: vec![],
            skipped: vec![],
            topic_id: key,
        }
    }
    fn proposal(state: &State, keys: &[String]) -> Proposal {
        propose(
            state,
            keys,
            &["2026-11-23".into()],
            "TYT Matematik",
            180,
            45,
            20,
        )
        .unwrap()
    }
    #[test]
    fn linked_weekly_moves_keep_the_learning_reference() {
        let key = keys(1)[0].clone();
        let mut s = State::default();
        let mut task = existing("weekly", "2026-11-23", 45, Some(key.clone()));
        task.weekly = true;
        s.tasks.push(task);
        crate::model::move_task(&mut s.tasks, "weekly", "2026-11-23", "2026-11-24").unwrap();
        assert_eq!(s.tasks[1].topic_id.as_deref(), Some(key.as_str()));
        assert!(pending_topic(&s, &key));
        assert_eq!(status(&s, &key), TopicStatus::NotStarted);
    }
    #[test]
    fn malformed_or_duplicate_progress_is_rejected_without_coercion() {
        let key = keys(1)[0].clone();
        let mut s = State::default();
        s.topic_progress = vec![
            TopicProgress {
                id: key.clone(),
                status: TopicStatus::Completed,
            },
            TopicProgress {
                id: key,
                status: TopicStatus::NotStarted,
            },
        ];
        assert!(!crate::model::valid(&s));
        assert!(serde_json::from_str::<TopicProgress>(r#"{"id":"a","status":"unknown"}"#).is_err());
    }
    #[test]
    fn manual_progress_is_independent_and_reset_is_explicit() {
        let key = keys(1)[0].clone();
        let mut s = State::default();
        assert_eq!(status(&s, &key), TopicStatus::NotStarted);
        let mut t = existing("a", "2026-11-23", 45, Some(key.clone()));
        t.done.push(t.date.clone());
        s.tasks.push(t);
        assert_eq!(status(&s, &key), TopicStatus::NotStarted);
        set_status(&mut s, &key, TopicStatus::NeedsReview).unwrap();
        set_status(&mut s, &key, TopicStatus::NotStarted).unwrap();
        assert_eq!(s.topic_progress.len(), 1);
        assert_eq!(status(&s, &key), TopicStatus::NotStarted);
        assert!(set_status(&mut s, "missing", TopicStatus::Completed).is_err());
        assert!(crate::model::valid(&s));
    }
    #[test]
    fn priority_excludes_completed_and_pending_topics_without_mutation() {
        let k = keys(5);
        let mut s = State::default();
        set_status(&mut s, &k[1], TopicStatus::InProgress).unwrap();
        set_status(&mut s, &k[2], TopicStatus::NeedsReview).unwrap();
        set_status(&mut s, &k[3], TopicStatus::Completed).unwrap();
        s.tasks
            .push(existing("pending", "2026-11-22", 45, Some(k[4].clone())));
        let before = serde_json::to_string(&s).unwrap();
        let p = proposal(
            &s,
            &[
                k[0].clone(),
                k[4].clone(),
                k[3].clone(),
                k[1].clone(),
                k[2].clone(),
            ],
        );
        assert_eq!(
            p.items
                .iter()
                .map(|t| t.topic_id.clone().unwrap())
                .collect::<Vec<_>>(),
            vec![k[2].clone(), k[1].clone(), k[0].clone()]
        );
        assert_eq!((p.completed_skipped, p.pending_skipped), (1, 1));
        assert_eq!(before, serde_json::to_string(&s).unwrap());
    }
    #[test]
    fn existing_weekly_and_daily_work_reduce_capacity() {
        let k = keys(3);
        let mut s = State::default();
        s.tasks.push(existing("daily", "2026-11-23", 90, None));
        let mut weekly = existing("weekly", "2026-11-24", 30, None);
        weekly.weekly = true;
        s.tasks.push(weekly);
        let p = propose(
            &s,
            &k,
            &["2026-11-23".into(), "2026-11-24".into()],
            "TYT Matematik",
            120,
            45,
            20,
        )
        .unwrap();
        assert_eq!(p.items.len(), 2);
        assert!(p.items.iter().all(|t| t.date == "2026-11-24"));
        assert_eq!(p.unassigned, 1);
    }
    #[test]
    fn confirmation_rechecks_progress_pending_work_and_capacity() {
        let k = keys(3);
        let mut s = State::default();
        let p = proposal(&s, &k);
        set_status(&mut s, &k[0], TopicStatus::Completed).unwrap();
        s.tasks
            .push(existing("pending", "2026-11-25", 45, Some(k[1].clone())));
        s.tasks.push(existing("busy", "2026-11-23", 150, None));
        let report = apply(&mut s, &p, &k).unwrap();
        assert_eq!((report.added, report.skipped), (0, 3));
        assert_eq!(s.tasks.len(), 2);
    }
    #[test]
    fn selection_double_confirmation_and_repeat_cycles_preserve_history() {
        let k = keys(2);
        let mut s = State::default();
        let p = proposal(&s, &k);
        assert_eq!(apply(&mut s, &p, &k[..1]).unwrap().added, 1);
        let original = s.tasks[0].id.clone();
        assert_eq!(apply(&mut s, &p, &k[..1]).unwrap().added, 0);
        let date = s.tasks[0].date.clone();
        s.tasks[0].done.push(date);
        set_status(&mut s, &k[0], TopicStatus::NeedsReview).unwrap();
        let next = proposal(&s, &k[..1]);
        assert_eq!(apply(&mut s, &next, &k[..1]).unwrap().added, 1);
        assert_ne!(s.tasks[1].id, original);
        assert!(s.tasks[0].done.len() == 1);
        assert_eq!(status(&s, &k[0]), TopicStatus::NeedsReview);
    }
    #[test]
    fn concurrent_proposals_use_one_identity_even_for_different_dates() {
        let k = keys(1);
        let s = State::default();
        let a = proposal(&s, &k);
        let b = propose(&s, &k, &["2026-11-24".into()], "TYT Matematik", 180, 45, 20).unwrap();
        let mut x = s.clone();
        let mut y = s;
        apply(&mut x, &a, &k).unwrap();
        apply(&mut y, &b, &k).unwrap();
        assert_eq!(x.tasks[0].id, y.tasks[0].id);
    }
    #[test]
    fn invalid_or_empty_requests_do_not_create_fake_work() {
        let k = keys(1);
        let s = State::default();
        assert!(propose(&s, &k, &[], "TYT", 120, 45, 20).is_err());
        assert!(propose(&s, &k, &["bad".into()], "TYT", 120, 45, 20).is_err());
        assert!(propose(&s, &k, &["2026-11-23".into()], "TYT", 30, 45, 20).is_err());
        assert!(proposal(&s, &[]).items.is_empty());
        let p = propose(
            &s,
            &[k[0].clone(), k[0].clone()],
            &["2026-11-23".into(), "2026-11-23".into()],
            "TYT",
            120,
            45,
            20,
        )
        .unwrap();
        assert_eq!(p.items.len(), 1);
    }
    #[test]
    fn old_snapshots_default_to_unlinked_tasks_and_empty_progress() {
        let mut old = serde_json::to_value(State::default()).unwrap();
        old.as_object_mut().unwrap().remove("topic_progress");
        let mut task = serde_json::to_value(existing("old", "2026-11-23", 45, None)).unwrap();
        task.as_object_mut().unwrap().remove("topic_id");
        old["tasks"] = serde_json::json!([task]);
        let restored: State = serde_json::from_value(old).unwrap();
        assert!(restored.topic_progress.is_empty());
        assert_eq!(restored.tasks[0].topic_id, None);
        assert!(crate::model::valid(&restored));
    }
    #[test]
    fn invalid_preview_is_atomic() {
        let k = keys(2);
        let mut s = State::default();
        let mut p = proposal(&s, &k);
        p.items[1].date = "invalid".into();
        assert!(apply(&mut s, &p, &k).is_err());
        assert!(s.tasks.is_empty());
    }
}
