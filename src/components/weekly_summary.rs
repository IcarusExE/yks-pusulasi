use crate::{
    model::{exam_trend, occurs, summarize_week, State},
    platform::{date_label, shift_date, today, weekday},
};
use leptos::prelude::*;
#[component]
pub fn WeeklySummary(state: RwSignal<State>) -> impl IntoView {
    let anchor = RwSignal::new(today(0));
    let start =
        Memo::new(move |_| shift_date(&anchor.get(), -(((weekday(&anchor.get()) + 6) % 7) as i32)));
    let days = Memo::new(move |_| {
        (0..7)
            .map(|i| shift_date(&start.get(), i))
            .collect::<Vec<_>>()
    });
    let summary = move || summarize_week(&state.get(), &days.get());
    view! {<section class="panel form-panel weekly-summary"><div class="section-head"><div><h2>"Haftana bir bakış"</h2><p>{move||format!("{} — {}",date_label(&start.get()),date_label(&shift_date(&start.get(),6)))}</p></div><div class="week-controls"><button class="secondary" aria-label="Önceki hafta" on:click=move|_|anchor.update(|d|*d=shift_date(d,-7))>"Önceki"</button><button class="secondary" on:click=move|_|anchor.set(today(0))>"Bu hafta"</button><button class="secondary" aria-label="Sonraki hafta" on:click=move|_|anchor.update(|d|*d=shift_date(d,7))>"Sonraki"</button></div></div><div class="week-metrics"><div><small>"Tamamlanan çalışmalar"</small><strong>{move||{let s=summary();format!("{} / {}",s.completed,s.planned)}}</strong></div><div><small>"Kaydedilen odak süresi"</small><strong>{move||format!("{} dk",summary().focus_seconds/60)}</strong></div><div><small>"Tamamlanan soru hedefi"</small><strong>{move||format!("{} soru",summary().questions)}</strong></div></div><p class="hint">"Soru sayısı, tiklediğin çalışmaların hedef toplamıdır. Taşınan çalışmalar yeni gününde sayılır."</p><div class="week-bars">{move||days.get().into_iter().map(|day|{let snapshot=state.get();let tasks=snapshot.tasks.iter().filter(|t|occurs(t,&day)).collect::<Vec<_>>();let done=tasks.iter().filter(|t|t.done.contains(&day)).count();let total=tasks.len();let label=date_label(&day);let short=label.split_whitespace().last().unwrap_or("").chars().take(3).collect::<String>();let value=if total==0{0}else{done*100/total};view!{<div class="week-bar" aria-label=format!("{label}: {done}/{total} çalışma tamamlandı")><span>{short}</span><div class="week-bar-track"><div style=format!("height:{value}%")></div></div><small>{format!("{done}/{total}")}</small></div>}}).collect_view()}</div><Show when=move||!summary().subjects.is_empty()><h3>"Derslere göre"</h3><div class="table-scroll"><table><thead><tr><th>"Ders"</th><th>"Çalışma"</th><th>"Odak"</th></tr></thead><tbody>{move||summary().subjects.into_iter().map(|(name,s)|view!{<tr><th scope="row">{name}</th><td>{format!("{} / {}",s.completed,s.planned)}</td><td>{format!("{} dk",s.focus_seconds/60)}</td></tr>}).collect_view()}</tbody></table></div></Show><div class="weekly-exams">{["TYT","AYT / YDT"].into_iter().map(|kind|view!{<div><h3>{kind}</h3>{move||match exam_trend(&state.get(),kind,&start.get(),&shift_date(&start.get(),6)){Some((score,delta))=>view!{<strong>{format!("{score:.2} net")}</strong><p>{match delta{Some(change)=>format!("Önceki karşılaştırılabilir denemeye göre {change:+.2} net"),None=>"Henüz karşılaştırılabilir önceki deneme yok.".into()}}</p>}.into_any(),None=>view!{<p>"Bu hafta kaydedilmiş deneme yok."</p>}.into_any()}}</div>}).collect_view()}</div><p class="hint">"Net karşılaştırması aynı test ve ders/soru yapısındaki denemelerle yapılır. Bu bir puan veya sıralama tahmini değildir."</p></section>}
}
