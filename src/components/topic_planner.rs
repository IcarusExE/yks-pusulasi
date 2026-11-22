use crate::{
    curriculum::{course_name, matching},
    fields, get,
    learning::{self, topic_key, Proposal},
    model::{valid_date, State},
    num,
    platform::{date_label, shift_date, today, weekday},
};
use leptos::prelude::*;
#[component]
pub fn TopicPlanner(
    state: RwSignal<State>,
    course: RwSignal<String>,
    query: RwSignal<String>,
    filter: RwSignal<String>,
    lesson: RwSignal<String>,
    date: RwSignal<String>,
) -> impl IntoView {
    let draft = RwSignal::new(None::<Proposal>);
    let selected = RwSignal::new(Vec::<String>::new());
    let message = RwSignal::new(String::new());
    Effect::new(move |_| {
        let _ = (course.get(), query.get(), filter.get(), lesson.get());
        draft.set(None);
        selected.set(vec![]);
        message.set(String::new());
    });
    view! {<section class="topic-planner"><h3>"Eksik konularımdan program öner"</h3><p class="hint">"Seçili ders, arama ve durum filtresi kullanılır. Önce tekrar gereken, sonra çalıştığın, ardından başlamadığın konular ele alınır. Her grupta kaynak sırası izlenir."</p><p class="hint">"Tamamladığın veya tamamlanmamış bir çalışmaya bağlı konular önerilmez. Günlük bütçeden mevcut programın süresi düşülür."</p><form on:input=move|_|{draft.set(None);message.set(String::new());} on:submit=move|event|{let values=fields(event);let start=get(&values,"start");if !valid_date(&start)||start<today(0)||!valid_date(&shift_date(&start,6)){message.set("Bugün veya sonrasında geçerli bir başlangıç tarihi seç.".into());return;}let days=(0..7).map(|i|shift_date(&start,i)).filter(|day|get(&values,&format!("day{}",weekday(day)))=="on").collect::<Vec<_>>();let snapshot=state.get_untracked();let mode=filter.get_untracked();let keys=matching(&course.get_untracked(),&query.get_untracked()).into_iter().filter(|t|mode=="all"||learning::status(&snapshot,&topic_key(t)).key()==mode).map(topic_key).collect::<Vec<_>>();match learning::propose(&snapshot,&keys,&days,&lesson.get_untracked(),num(&values,"budget"),num(&values,"block"),num(&values,"questions")){Ok(proposal)=>{selected.set(proposal.items.iter().filter_map(|t|t.topic_id.clone()).collect());draft.set(Some(proposal));message.set(String::new());},Err(error)=>message.set(error.into())}}><div class="planner-grid"><label>"Başlangıç günü"<input name="start" type="date" min=today(0) max="9999-12-25" value=date.get_untracked().max(today(0)) required/></label><label>"Günlük toplam bütçe (dk)"<input name="budget" type="number" min="30" max="360" value="120" required/></label><label>"Bir çalışma oturumu (dk)"<input name="block" type="number" min="15" max="120" value="45" required/></label><label>"Oturum başına soru hedefi"<input name="questions" type="number" min="0" max="1000" value="20" required/></label></div><fieldset class="days"><legend>"Önümüzdeki 7 günde uygun günlerim"</legend>{[(1,"Pzt"),(2,"Sal"),(3,"Çar"),(4,"Per"),(5,"Cum"),(6,"Cmt"),(0,"Paz")].into_iter().map(|(i,name)|view!{<label><input type="checkbox" name=format!("day{i}") checked=i!=0/>{name}</label>}).collect_view()}</fieldset><button class="secondary" type="submit">"Öneri taslağını göster"</button></form><Show when=move||!message.get().is_empty()><p class="notice planner-message" role="status">{move||message.get()}</p></Show>{move||draft.get().map(|proposal|{let count=proposal.items.len();let budget=proposal.daily_budget;view!{<div class="proposal"><div class="section-head"><div><h3>{format!("{} · {} çalışma önerisi",course_name(&course.get()),count)}</h3><p>{format!("Günlük toplam bütçe: {budget} dk")}</p></div><span class="tag">"Kural tabanlı taslak"</span></div><p class="hint">{format!("{} tamamlanan, {} zaten planlanmış başlık dışarıda tutuldu. {} başlık süre bütçesine sığmadı.",proposal.completed_skipped,proposal.pending_skipped,proposal.unassigned)}</p><Show when=move||count==0><p>"Bu kapsamda önerilecek boş çalışma bulunamadı. Durum filtresini, uygun günleri veya günlük bütçeyi kontrol et."</p></Show><div class="proposal-list">{proposal.items.into_iter().map(|task|{let key=task.topic_id.clone().unwrap_or_default();let check_key=key.clone();let change_key=key.clone();let status_key=key;let label=format!("{} için öneriyi seç",task.title);let info=format!("{} · {} dk · {} soru",date_label(&task.date),task.minutes,task.questions);view!{<label class="proposal-row"><input type="checkbox" aria-label=label prop:checked=move||selected.get().contains(&check_key) on:change=move|event|{let checked=event_target_checked(&event);selected.update(|keys|{if checked{if !keys.contains(&change_key){keys.push(change_key.clone());}}else{keys.retain(|key|key!=&change_key);}});}/><span><strong>{task.title}</strong><small>{info}</small><em>{move||learning::status(&state.get(),&status_key).label()}</em></span></label>}}).collect_view()}</div><Show when={move||count>0}><p class="hint">"Süreler çalışma oturumu hedefidir; konunun tamamen öğrenilme süresi değildir. Eklemeden önce seçimini ve programını gözden geçir."</p><button class="primary" disabled=move||selected.get().is_empty() on:click=move|_|{if let Some(proposal)=draft.get_untracked(){let keys=selected.get_untracked();if keys.is_empty(){return;}let mut result=Err("Taslak eklenemedi.");state.update(|snapshot|result=learning::apply(snapshot,&proposal,&keys));match result{Ok(report)=>{message.set(format!("{} çalışma eklendi. Güncel ilerleme veya program nedeniyle {} öneri atlandı.",report.added,report.skipped));draft.set(None);selected.set(vec![]);},Err(error)=>message.set(error.into())}}}>{move||format!("Seçili {} çalışmayı programıma ekle",selected.get().len())}</button><p class="hint">"Onay sırasında tamamlanan konular, mevcut çalışmalar ve kalan süre yeniden kontrol edilir. Mevcut program değiştirilmez."</p></Show></div>}})}</section>}
}
