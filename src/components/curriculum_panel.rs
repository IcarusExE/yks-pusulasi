use super::topic_planner::TopicPlanner;
use crate::{
    curriculum::*,
    learning::{self, topic_key, TopicStatus},
    model::{subjects, State, Task},
};
use leptos::prelude::*;
#[component]
pub fn CurriculumPanel(
    state: RwSignal<State>,
    track: Memo<String>,
    date: RwSignal<String>,
    on_add: Callback<Task>,
) -> impl IntoView {
    let course = RwSignal::new("math".to_string());
    let query = RwSignal::new(String::new());
    let offset = RwSignal::new(0usize);
    let lesson = RwSignal::new("TYT Matematik".to_string());
    let filter = RwSignal::new("all".to_string());
    let message = RwSignal::new(String::new());
    let choices = Memo::new(move |_| {
        let mut choices = subjects(&track.get())
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let extra = default_lesson(&course.get());
        if !choices.iter().any(|s| s == extra) {
            choices.push(extra.into());
        }
        let current = lesson.get();
        if !choices.contains(&current) {
            choices.push(current);
        }
        choices
    });
    let filtered = move || {
        let mode = filter.get();
        let snapshot = if mode == "all" {
            None
        } else {
            Some(state.get())
        };
        matching(&course.get(), &query.get())
            .into_iter()
            .filter(|topic| {
                snapshot
                    .as_ref()
                    .map(|s| learning::status(s, &topic_key(topic)).key() == mode)
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>()
    };
    let counts = Memo::new(move |_| {
        let topics = matching(&course.get(), &query.get());
        state.with(|snapshot| {
            let map = snapshot
                .topic_progress
                .iter()
                .map(|p| (p.id.as_str(), p.status))
                .collect::<std::collections::HashMap<_, _>>();
            let mut counts = TopicStatus::ALL.map(|status| (status, 0usize));
            for topic in topics {
                let key = topic_key(topic);
                let status = map.get(key.as_str()).copied().unwrap_or_default();
                if let Some((_, n)) = counts.iter_mut().find(|(s, _)| *s == status) {
                    *n += 1;
                }
            }
            counts
        })
    });
    // A status filter can remove its last page while another window updates progress.
    Effect::new(move |_| {
        let n = filtered().len();
        let current = offset.get_untracked();
        if current >= n && current != 0 {
            offset.set(n.saturating_sub(1) / 12 * 12);
        }
    });
    view! {<section class="panel form-panel curriculum-panel"><div class="section-head"><div><h2>"Konudan çalışma oluştur"</h2><p>"2026 MEB / TTKB resmî kapsamı · 18 ders"</p></div><span class="tag">{format!("Kaynak yılı: {}",catalogue().year)}</span></div><p class="hint">{catalogue().granularity.clone()}" Bu liste, 2027 kapsamı olarak doğrulanmamıştır."</p><div class="catalogue-filters"><label>"Resmî belgedeki ders"<select prop:value=move||course.get() on:change=move|event|{let next=event_target_value(&event);lesson.set(default_lesson(&next).into());course.set(next);query.set(String::new());offset.set(0);message.set(String::new());}>{catalogue().courses.iter().map(|c|view!{<option value=c.id.clone() selected=c.id=="math">{c.name.clone()}</option>}).collect_view()}</select></label><label>"Konu, kod veya sınıf ara"<input type="search" placeholder="Örn. türev, 11. sınıf, 9.3.3" prop:value=move||query.get() on:input=move|event|{query.set(event_target_value(&event));offset.set(0);}/></label><label>"Planındaki çalışma dersi"<select prop:value=move||lesson.get() on:change=move|event|lesson.set(event_target_value(&event))>{move||choices.get().into_iter().map(|s|{let value=s.clone();let text=s.clone();view!{<option value=value selected=move||lesson.get()==s>{text}</option>}}).collect_view()}</select></label></div><p class="hint">"Planındaki ders etiketi senin seçimin; resmî TYT/AYT sınıflandırması değildir."</p><section class="topic-progress"><h3>"Bu kapsamdaki ilerlemen"</h3><div class="topic-progress-counts">{TopicStatus::ALL.into_iter().map(|status|view!{<button class:selected=move||filter.get()==status.key() on:click=move|_|{filter.set(status.key().into());offset.set(0);} aria-pressed=move||filter.get()==status.key()><span>{status.label()}</span><strong>{move||counts.get().into_iter().find(|(key,_)|*key==status).map(|(_,n)|n).unwrap_or(0)}</strong></button>}).collect_view()}</div><progress aria-label="Tamamladım olarak işaretlenen konular" max=move||matching(&course.get(),&query.get()).len().max(1) value=move||counts.get()[3].1></progress><p class="hint">"Sayılar seçili ders ve arama kapsamındaki başlıklar içindir. Konu seviyesini sen belirlersin. Çalışmayı tiklemek konu seviyesini otomatik değiştirmez. Eski serbest çalışmalar otomatik olarak konularla eşleştirilmez."</p></section><label class="topic-filter">"İlerleme durumu filtresi"<select prop:value=move||filter.get() on:change=move|event|{filter.set(event_target_value(&event));offset.set(0);}><option value="all" selected=move||filter.get()=="all">"Tümü"</option>{TopicStatus::ALL.into_iter().map(|status|view!{<option value=status.key() selected=move||filter.get()==status.key()>{status.label()}</option>}).collect_view()}</select></label><Show when=move||!message.get().is_empty()><p class="notice" role="status">{move||message.get()}</p></Show><div class="topic-list">{move||{let matches=filtered();let start=offset.get().min(matches.len());matches.into_iter().skip(start).take(12).map(|topic|{let draft=topic.clone();let href=format!("{}#page={}",catalogue().source_url,topic.page);let select_key=topic_key(topic);let change_key=select_key.clone();let options_key=select_key.clone();let label=format!("{} için konu seviyesi",topic.title);view!{<article class="topic-row" id=topic.id.clone()><div><small>{format!("{} · {}",topic.level,topic.code)}</small><h3>{topic.title.clone()}</h3><a href=href target="_blank" rel="noopener noreferrer">{format!("Resmî belge · s. {}",topic.page)}</a></div><div class="topic-actions"><label>"Konu seviyesi"<select aria-label=label prop:value=move||state.with(|s|learning::status(s,&select_key)).key() on:change=move|event|{if let Some(status)=TopicStatus::parse(&event_target_value(&event)){let mut result=Ok(());state.update(|s|result=learning::set_status(s,&change_key,status));if let Err(error)=result{message.set(error.into());}}}>{TopicStatus::ALL.into_iter().map(|status|{let key=options_key.clone();view!{<option value=status.key() selected=move||state.with(|s|learning::status(s,&key))==status>{status.label()}</option>}}).collect_view()}</select></label><button class="secondary" on:click=move|_|on_add.run(Task{id:String::new(),title:learning::topic_title(&draft),subject:lesson.get(),date:date.get(),minutes:45,questions:20,weekly:false,done:vec![],skipped:vec![],topic_id:Some(topic_key(&draft))})>"Çalışma oluştur"</button></div></article>}}).collect_view()}}</div><Show when=move||filtered().is_empty()><p class="empty">"Aramana ve durum filtresine uygun konu bulunamadı."</p></Show><div class="topic-pagination"><button class="secondary" disabled=move||offset.get()==0 on:click=move|_|offset.update(|n|*n=n.saturating_sub(12))>"Önceki"</button><span>{move||{let n=filtered().len();if n==0{"0 konu".into()}else{format!("{}–{} / {}",offset.get().min(n-1)+1,(offset.get()+12).min(n),n)}}}</span><button class="secondary" disabled=move||offset.get()+12>=filtered().len() on:click=move|_|offset.update(|n|*n+=12)>"Sonraki"</button></div><TopicPlanner state=state course=course query=query filter=filter lesson=lesson date=date/><p class="hint">{format!("Kaynak kontrolü: {}. Konu seçimi ve ilerleme takibi çevrimdışı çalışır; resmî PDF bağlantısı için internet gerekir.",catalogue().checked_on)}</p></section>}
}
