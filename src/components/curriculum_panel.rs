use crate::{
    curriculum::*,
    model::{subjects, Task},
};
use leptos::prelude::*;
#[component]
pub fn CurriculumPanel(
    track: Memo<String>,
    date: RwSignal<String>,
    on_add: Callback<Task>,
) -> impl IntoView {
    let course = RwSignal::new("math".to_string());
    let query = RwSignal::new(String::new());
    let offset = RwSignal::new(0usize);
    let lesson = RwSignal::new("TYT Matematik".to_string());
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
    view! {<section class="panel form-panel curriculum-panel"><div class="section-head"><div><h2>"Konudan çalışma oluştur"</h2><p>"2026 MEB / TTKB resmî kapsamı · 18 ders"</p></div><span class="tag">{format!("Kaynak yılı: {}",catalogue().year)}</span></div><p class="hint">{catalogue().granularity.clone()}" Bu liste, 2027 kapsamı olarak doğrulanmamıştır."</p><div class="catalogue-filters"><label>"Resmî belgedeki ders"<select prop:value=move||course.get() on:change=move|e|{let c=event_target_value(&e);lesson.set(default_lesson(&c).into());course.set(c);query.set(String::new());offset.set(0);}>
    {catalogue().courses.iter().map(|c|view!{<option value=c.id.clone() selected=c.id=="math">{c.name.clone()}</option>}).collect_view()}</select></label><label>"Konu, kod veya sınıf ara"<input type="search" placeholder="Örn. türev, 11. sınıf, 9.3.3" prop:value=move||query.get() on:input=move|e|{query.set(event_target_value(&e));offset.set(0);}/></label><label>"Planındaki çalışma dersi"<select prop:value=move||lesson.get() on:change=move|e|lesson.set(event_target_value(&e))>{move||choices.get().into_iter().map(|s|{let value=s.clone();let text=s.clone();view!{<option value=value selected=move||lesson.get()==s>{text}</option>}}).collect_view()}</select></label></div><p class="hint">"Planındaki ders etiketi senin seçimin; resmî TYT/AYT sınıflandırması değildir."</p><div class="topic-list">{move||{let matches=matching(&course.get(),&query.get());let start=offset.get().min(matches.len());matches.into_iter().skip(start).take(12).map(|topic|{let draft=topic.clone();let href=format!("{}#page={}",catalogue().source_url,topic.page);view!{<article class="topic-row" id=topic.id.clone()><div><small>{format!("{} · {}",topic.level,topic.code)}</small><h3>{topic.title.clone()}</h3><a href=href target="_blank" rel="noopener noreferrer">{format!("Resmî belge · s. {}",topic.page)}</a></div><button class="secondary" on:click=move|_|{let full=format!("{} · {} ({})",course_name(&draft.course),draft.title,draft.level);let title=if full.chars().count()>160{full.chars().take(157).collect::<String>()+"…"}else{full};on_add.run(Task{id:String::new(),title,subject:lesson.get(),date:date.get(),minutes:45,questions:20,weekly:false,done:vec![],skipped:vec![]});}>"Çalışma oluştur"</button></article>}}).collect_view()}}</div><Show when=move||matching(&course.get(),&query.get()).is_empty()><p class="empty">"Aramana uygun konu bulunamadı."</p></Show><div class="topic-pagination"><button class="secondary" disabled=move||offset.get()==0 on:click=move|_|offset.update(|n|*n=n.saturating_sub(12))>"Önceki"</button><span>{move||{let n=matching(&course.get(),&query.get()).len();if n==0{"0 konu".into()}else{format!("{}–{} / {}",offset.get()+1,(offset.get()+12).min(n),n)}}}</span><button class="secondary" disabled=move||offset.get()+12>=matching(&course.get(),&query.get()).len() on:click=move|_|offset.update(|n|*n+=12)>"Sonraki"</button></div><p class="hint">{format!("Kaynak kontrolü: {}. Konu seçimi çevrimdışı çalışır; resmî PDF bağlantısı için internet gerekir.",catalogue().checked_on)}</p></section>}
}
