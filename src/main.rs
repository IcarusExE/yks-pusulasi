mod model;
mod platform;
use leptos::prelude::*;
use model::*;
use platform::*;
use std::collections::BTreeMap;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;

fn fields(ev: web_sys::SubmitEvent) -> BTreeMap<String, String> {
    ev.prevent_default();
    let form: web_sys::HtmlFormElement = ev.target().unwrap().unchecked_into();
    let data = web_sys::FormData::new_with_form(&form).unwrap();
    js_sys::try_iter(&data)
        .unwrap()
        .unwrap()
        .filter_map(|v| {
            let a = js_sys::Array::from(&v.ok()?);
            Some((a.get(0).as_string()?, a.get(1).as_string()?))
        })
        .collect()
}
fn get(f: &BTreeMap<String, String>, k: &str) -> String {
    f.get(k).cloned().unwrap_or_default().trim().to_string()
}
fn num(f: &BTreeMap<String, String>, k: &str) -> u32 {
    get(f, k).parse().unwrap_or(0)
}
#[component]
fn App() -> impl IntoView {
    let state = RwSignal::new(State::default());
    let ready = RwSignal::new(false);
    let status = RwSignal::new("Kayıtlar açılıyor…".to_string());
    let page = RwSignal::new("today".to_string());
    let date = RwSignal::new(today(0));
    let notice = RwSignal::new(String::new());
    let kind = RwSignal::new("TYT".to_string());
    let flipped = RwSignal::new(false);
    let deck = RwSignal::new("Tümü".to_string());
    let tick = RwSignal::new(0u32);
    let editing = RwSignal::new(None::<String>);
    let restore = RwSignal::new(None::<State>);
    let active_track = Memo::new(move |_| state.get().track);
    // Draft fields are captured on opening, so unrelated remote updates do not
    // recreate the form while the student is typing.
    let editing_task = Memo::new(move |_| {
        let edit_id = editing.get().unwrap_or_default();
        let snapshot = state.get_untracked();
        snapshot
            .tasks
            .into_iter()
            .find(|task| task.id == edit_id)
            .unwrap_or(Task {
                id: String::new(),
                title: String::new(),
                subject: subjects(&snapshot.track)[0].into(),
                date: date.get_untracked(),
                minutes: 45,
                questions: 20,
                weekly: false,
                done: vec![],
            })
    });
    let persisted = StoredValue::new(String::new());
    let force_write = StoredValue::new(false);
    let change = StoredValue::new(0u64);
    let sync_callback =
        StoredValue::new_local(Closure::<dyn FnMut(String)>::new(move |raw: String| {
            if !ready.get_untracked() {
                return;
            }
            match serde_json::from_str::<State>(&raw) {
                Ok(incoming) if valid(&incoming) => {
                    let canonical = serde_json::to_string(&incoming).unwrap();
                    accept_snapshot(&canonical);
                    if canonical != persisted.get_value() {
                        change.set_value(change.get_value().wrapping_add(1));
                        persisted.set_value(canonical);
                        state.set(incoming);
                        status.set("Pencereler eşitlendi".into());
                    }
                }
                _ => {
                    status.set("Diğer penceredeki kayıt okunamadı. Mevcut görünüm korundu.".into())
                }
            }
        }));
    spawn_local(async move {
        match load().await {
            Ok(value) => {
                let raw = value.as_string().unwrap_or_default();
                if raw.is_empty() { accept_snapshot(&serde_json::to_string(&State::default()).unwrap()); ready.set(true); }
                else {
                    match serde_json::from_str::<State>(&raw) {
                        Ok(loaded) if valid(&loaded) => {
                            let canonical = serde_json::to_string(&loaded).unwrap();
                            accept_snapshot(&canonical);
                            persisted.set_value(canonical);
                            state.set(loaded);
                            ready.set(true);
                            status.set("Bu cihazda kayıtlı".into());
                        }
                        _ => status.set("Kayıt açılamadı. Mevcut verilerin üzerine yazılmadı.".into()),
                    }
                }
                if ready.get_untracked() {
                    sync_callback.with_value(|callback| subscribe(callback.as_ref().unchecked_ref()));
                }
            }
            Err(_) => status.set("Cihaz kayıt alanına erişilemiyor. Tarayıcı izinlerini kontrol edip sayfayı yenile.".into()),
        }
    });
    Effect::new(move |_| {
        if !ready.get() {
            return;
        }
        let snapshot = state.get();
        theme(&snapshot.palette, snapshot.dark);
        if !valid(&snapshot) {
            status.set(
                "Değişiklik kaydedilemedi. Girişlerini kontrol et; önceki kaydın korundu.".into(),
            );
            return;
        }
        let raw = serde_json::to_string(&snapshot).unwrap();
        if raw == persisted.get_value() && !force_write.get_value() {
            return;
        }
        force_write.set_value(false);
        persisted.set_value(raw.clone());
        let ticket = change.get_value().wrapping_add(1);
        change.set_value(ticket);
        status.set("Kaydediliyor…".into());
        spawn_local(async move {
            let result = save(&raw).await;
            if change.get_value() != ticket {
                return;
            }
            match result {
                Ok(value) => {
                    let merged = value.as_string().unwrap_or_default();
                    match serde_json::from_str::<State>(&merged) {
                        Ok(merged) if valid(&merged) => {
                            let canonical = serde_json::to_string(&merged).unwrap();
                            accept_snapshot(&canonical);
                            persisted.set_value(canonical.clone());
                            if canonical != raw {
                                state.set(merged);
                            }
                            status.set("Bu cihazda kayıtlı".into());
                        }
                        _ => status.set(
                            "Birleştirilen kayıt okunamadı. Yedek alıp sayfayı yenile.".into(),
                        ),
                    }
                }
                Err(_) => {
                    persisted.set_value(String::new());
                    status.set("Kayıt başarısız. Yedek al ve cihaz alanını kontrol et.".into());
                }
            }
        });
    });
    let interval = gloo_timers::callback::Interval::new(1000, move || {
        tick.update(|n| *n = n.wrapping_add(1));
        let t = state.get_untracked().timer;
        if t.running && t.mode == "Pomodoro" && elapsed(&t) >= 1500 {
            state.update(|s| {
                s.timer.running = false;
                s.timer.elapsed = 1500;
            });
            notify_done();
            notice.set("Odak oturumu tamamlandı. Çalışmanı kaydet, kısa bir mola ver.".into());
        }
    });
    let _interval = StoredValue::new_local(interval);
    let active_card = move || {
        let s = state.get();
        s.cards
            .into_iter()
            .find(|c| (deck.get() == "Tümü" || c.deck == deck.get()) && c.due <= today(0))
    };
    view! {
    <div class="app-shell">
    <aside class="sidebar"><a class="brand" href="#" on:click=move|e|{e.prevent_default();page.set("today".into());}><img src="icon.svg" alt=""/><span>"yks pusulası"<small>"KENDİ ROTANI ÇİZ"</small></span></a>
    <div class="nav-caption">"ÇALIŞMA ALANIN"</div><nav aria-label="Ana gezinme">{[("today","◈","Bugün"),("plan","▦","Programım"),("timer","◷","Odaklan"),("net","▥","Net hesaplayıcı"),("cards","▤","Hafıza kartları")].into_iter().map(|(key,icon,label)|view!{<button class:active=move||page.get()==key on:click=move|_|page.set(key.into())><span class="nav-icon">{icon}</span>{label}{(key=="today").then(||view!{<span class="nav-count">{move||state.get().tasks.iter().filter(|t|occurs(t,&today(0))&&!t.done.contains(&today(0))).count()}</span>})}</button>}).collect_view()}</nav>
    <div class="sidebar-bottom"><div class="local-note"><span>"⌁"</span><strong>"Senin verin, senin cihazın."</strong><p>"Çalışmaların bu tarayıcıda saklanır. Rotan hep sana ait."</p></div><button class="settings-nav" on:click=move|_|page.set("settings".into())>"⚙  Görünüm ve ayarlar"</button><div class="profile"><span class="avatar">{move||state.get().name.chars().next().unwrap_or('P').to_uppercase().to_string()}</span><div><strong>{move||if state.get().name.is_empty(){"YKS yolcusu".into()}else{state.get().name}}</strong><small>{move||format!("{} · TYT + {}",state.get().track,if state.get().track=="Dil"{"YDT"}else{"AYT"})}</small></div></div></div></aside>
    <div class="main-shell"><header class="topbar"><span class="breadcrumb">"Çalışma alanım / "<strong>{move||match page.get().as_str(){"plan"=>"Programım","timer"=>"Odaklan","net"=>"Net hesaplayıcı","cards"=>"Hafıza kartları","settings"=>"Ayarlar",_=>"Bugün"}}</strong></span><span class="save-status" role="status">{move||status.get()}</span><button class="icon-button" title="Açık / koyu tema" aria-label="Açık ve koyu tema arasında geçiş yap" on:click=move|_|state.update(|s|s.dark=!s.dark)>"◐"</button><button class="icon-button" title="Görünüm ve ayarlar" aria-label="Görünüm ve ayarlar" on:click=move|_|page.set("settings".into())>"⚙"</button></header>
    <main><Show when=move||ready.get() fallback=move||view!{<div class="panel empty"><h1>"Çalışma alanın hazırlanıyor"</h1><p>{move||status.get()}</p><button on:click=move|_|{spawn_local(async move{if let Ok(v)=raw_backup().await{if let Some(raw)=v.as_string(){download(&raw)}}});}>"Mevcut kaydı indir"</button></div>}>
    <Show when=move||!notice.get().is_empty()><div class="notice" role="status">{move||notice.get()}<button aria-label="Bildirimi kapat" on:click=move|_|notice.set(String::new())>"×"</button></div></Show>
    <Show when=move||page.get()=="today"||page.get()=="plan">
    <div class="page-heading"><div><div class="eyebrow">{move||if page.get()=="today"{"HER GÜN BİR ADIM"}else{"HEDEFİNE GİDEN YOL"}}</div><h1>{move||if page.get()=="today"{if state.get().name.is_empty(){"Bugün, kendin için.".into()}else{format!("Merhaba, {}.",state.get().name)}}else{"Rotanı birlikte çizelim.".into()}}</h1><p>"Büyük hedefler, tamamlanan küçük adımlarla başlar."</p></div><button class="primary" on:click=move|_|{page.set("plan".into());editing.set(Some(String::new()));}>"+ Çalışma ekle"</button></div>
    <Show when=move||page.get()=="today"><section class="hero"><div class="hero-copy"><span class="hero-label">"SENİN YOLCULUĞUN, SENİN RİTMİN"</span><h2>"Hedefin belli."<br/>"Sıradaki adım senin."</h2><p>"Her şeyi bir günde bitirmene gerek yok. Bugünün planına odaklan, gerisi adım adım gelir."</p><button class="hero-button" on:click=move|_|page.set("timer".into())>"Bir odak oturumu başlat"<span>"◷"</span></button></div><div class="compass-art" aria-hidden="true"><span class="north">"N"</span><div class="compass-ring"><div class="needle"></div><i></i></div><span class="compass-caption">"ROTAN SANA AİT"</span></div></section></Show>
    <div class="stats-grid"><div class="stat panel"><span class="stat-icon">"✓"</span><div><small>"Tamamlanan hedef"</small><strong>{move||{let s=state.get();let tasks:Vec<_>=s.tasks.iter().filter(|t|occurs(t,&date.get())).collect();format!("{} / {}",tasks.iter().filter(|t|t.done.contains(&date.get())).count(),tasks.len())}}</strong></div></div><div class="stat panel"><span class="stat-icon">"◷"</span><div><small>"Odak süresi"</small><strong>{move||format!("{} dk",state.get().sessions.iter().filter(|s|s.date==date.get()).map(|s|s.seconds).sum::<u32>()/60)}</strong></div></div><div class="stat panel"><span class="stat-icon">"▧"</span><div><small>"Tamamlanan soru hedefi"</small><strong>{move||state.get().tasks.iter().filter(|t|occurs(t,&date.get())&&t.done.contains(&date.get())).map(|t|t.questions).sum::<u32>()}<em>" soru"</em></strong></div></div></div>
    <div class="dashboard-grid"><section class="panel schedule"><div class="section-head"><div><h2>"Günün planı"</h2><p>{move||date_label(&date.get())}</p></div><input class="date-input" aria-label="Plan tarihi" type="date" min="2000-01-01" max="9999-12-31" prop:value=move||date.get() on:change=move|e|{let value=event_target_value(&e);if valid_date(&value){date.set(value)}else{date.set(date.get_untracked());notice.set("Geçerli bir tarih seç.".into());}}/></div><div class="week-strip">{(-2..5).map(|offset|{let key=today(offset);let key2=key.clone();let key3=key.clone();let label=date_label(&key);let parts:Vec<_>=label.split_whitespace().collect();let number=parts.first().unwrap_or(&"").to_string();let day=parts.last().unwrap_or(&"").chars().take(3).collect::<String>();view!{<button class:selected=move||date.get()==key2 on:click=move|_|date.set(key3.clone())><span>{day}</span><strong>{number}</strong><i class:has-tasks=move||state.get().tasks.iter().any(|t|occurs(t,&key))></i></button>}}).collect_view()}</div>
    <div class="task-list">{move||{let list:Vec<_>=state.get().tasks.into_iter().filter(|t|occurs(t,&date.get())).collect();if list.is_empty(){view!{<div class="empty"><span class="empty-icon">"✧"</span><h3>"Yeni bir güne yer aç."</h3><p>"Henüz bir çalışma yok. İlk küçük hedefini ekleyerek başla."</p><button class="secondary" on:click=move|_|{editing.set(Some(String::new()));page.set("plan".into());}>"İlk çalışmamı ekle"</button></div>}.into_any()}else{list.into_iter().map(|t|{let tid=t.id.clone();let edit_id=t.id.clone();let remove_id=t.id.clone();let done=t.done.contains(&date.get());view!{<article class="task" class:completed=done><button class="check" class:checked=done aria-label=format!("{} tamamlanma durumunu değiştir",t.title) aria-pressed=done on:click=move|_|state.update(|s|{if let Some(t)=s.tasks.iter_mut().find(|t|t.id==tid){let d=date.get();if t.done.contains(&d){t.done.retain(|x|x!=&d)}else{t.done.push(d)}}})>{if done{"✓"}else{""}}</button><div class="task-detail"><small>{t.subject}</small><h3>{t.title}</h3><p>{format!("{} dk  ·  {} soru{}",t.minutes,t.questions,if t.weekly{"  ·  Her hafta"}else{""})}</p></div><button class="text-button" aria-label="Çalışmayı düzenle" on:click=move|_|editing.set(Some(edit_id.clone()))>"Düzenle"</button><button class="delete-button" aria-label="Çalışmayı sil" on:click=move|_|state.update(|s|s.tasks.retain(|t|t.id!=remove_id))>"×"</button></article>}}).collect_view().into_any()}}}</div><div class="schedule-footer"><span>"Küçük adımlar da ilerlemedir."</span><button class="text-button" on:click=move|_|editing.set(Some(String::new()))>"+ Ekle"</button></div></section>
    <aside class="right-column"><section class="panel progress-panel"><span class="eyebrow">"GÜNLÜK İLERLEME"</span><div class="progress-ring" style=move||{let s=state.get();let total=s.tasks.iter().filter(|t|occurs(t,&date.get())).count();let done=s.tasks.iter().filter(|t|occurs(t,&date.get())&&t.done.contains(&date.get())).count();format!("--progress:{}%",if total==0{0}else{done*100/total})}><div><strong>{move||{let s=state.get();let all=s.tasks.iter().filter(|t|occurs(t,&date.get())).count();let n=s.tasks.iter().filter(|t|occurs(t,&date.get())&&t.done.contains(&date.get())).count();format!("%{}",if all==0{0}else{n*100/all})}}</strong><span>"tamamlandı"</span></div></div><h3>"Kendi hızında, ileriye."</h3><p>"Planını tamamladıkça burada ilerlemeni göreceksin."</p></section><section class="quick-card"><span>"KISA BİR TEKRAR?"</span><h3>"Bilgiyi taze tut."</h3><p>{move||format!("Tekrarını bekleyen {} kartın var.",state.get().cards.iter().filter(|c|c.due<=today(0)).count())}</p><button class="secondary" on:click=move|_|page.set("cards".into())>"Kartlarıma göz at"</button></section></aside></div>
    <Show when=move||page.get()=="plan"><section class="panel form-panel"><div class="section-head"><div><h2>"Haftana bir taslak hazırla"</h2><p>"Uygun günlerini seç; dersleri dengeli dağıtalım. Eklenen çalışmaları istediğin gibi düzenleyebilirsin."</p></div><span class="tag">"Kural tabanlı"</span></div><form on:submit=move|ev|{let f=fields(ev);let days:Vec<_>=(0..7).filter(|i|get(&f,&format!("day{i}"))=="on").collect();if days.is_empty(){notice.set("En az bir çalışma günü seç.".into());return;}let mins=num(&f,"minutes").clamp(30,360);let start=get(&f,"start");if !valid_date(&start)||!valid_date(&shift_date(&start,6)){notice.set("Geçerli bir başlangıç tarihi seç.".into());return;}let lessons=subjects(&active_track.get());let existing=state.get().tasks;let mut added=vec![];let mut index=0;for offset in 0..7{let d=shift_date(&start,offset);if !days.contains(&weekday(&d)){continue;}let blocks=mins.div_ceil(60);for b in 0..blocks{let subject=lessons[index%lessons.len()].to_string();index+=1;let title="Konu çalışması ve soru pratiği".to_string();if existing.iter().any(|t|t.date==d&&t.subject==subject&&t.title==title){continue;}added.push(Task{id:id(),title,subject,date:d.clone(),minutes:(mins-b*60).min(60),questions:20,weekly:false,done:vec![]});}}let count=added.len();state.update(|s|s.tasks.extend(added));notice.set(format!("{count} çalışma eklendi. Konu ve soru hedeflerini kendine göre düzenleyebilirsin."));}><div class="form-grid"><label>"Başlangıç"<input type="date" min="2000-01-01" max="9999-12-31" name="start" required prop:value=today(0)/></label><label>"Günlük süre (dk)"<input type="number" name="minutes" min="30" max="360" step="30" value="120" required/></label></div><fieldset class="days"><legend>"Uygun günlerim"</legend>{[(1,"Pzt"),(2,"Sal"),(3,"Çar"),(4,"Per"),(5,"Cum"),(6,"Cmt"),(0,"Paz")].into_iter().map(|(i,d)|view!{<label><input type="checkbox" name=format!("day{i}") checked=i!=0/>{d}</label>}).collect_view()}</fieldset><button class="primary" type="submit">"Haftalık taslak oluştur"</button><p class="hint">"Aynı tarihteki aynı taslak dersi yeniden eklenmez. Yapay zekâ bağlantısı bu sürümde bulunmuyor."</p></form></section></Show>
    </Show>
    <Show when=move||page.get()=="timer"><div class="page-heading"><div><div class="eyebrow">"ŞİMDİ SADECE BU AN"</div><h1>"Dikkatini bir yere topla."</h1><p>"Bir ders seç, zamanı başlat. Her dakika birikiyor."</p></div></div><section class="panel timer-panel"><div class="segmented">{["Kronometre","Pomodoro"].into_iter().map(|mode|view!{<button class:selected=move||state.get().timer.mode==mode on:click=move|_|{if state.get().timer.running||state.get().timer.elapsed>0{notice.set("Mod değiştirmek için mevcut oturumu kaydet veya sıfırla.".into());}else{state.update(|s|s.timer.mode=mode.into())}}>{mode}</button>}).collect_view()}</div><label class="timer-subject">"Çalıştığın ders"<select prop:disabled={move||state.get().timer.running||state.get().timer.elapsed>0} prop:value=move||state.get().timer.subject on:change=move|ev|state.update(|s|s.timer.subject=event_target_value(&ev))>{move||subjects(&active_track.get()).into_iter().map(|s|view!{<option value=s selected=move||state.get().timer.subject==s>{s}</option>}).collect_view()}</select></label><div class="timer-digits" aria-live="off">{move||{let _=tick.get();let t=state.get().timer;clock(if t.mode=="Pomodoro"{1500u32.saturating_sub(elapsed(&t))}else{elapsed(&t)})}}</div><p>{move||if state.get().timer.mode=="Pomodoro"{"25 dakika odaklan. Ardından 5 dakika mola ver."}else{"Acele etmeden, kendi ritminde."}}</p><div class="button-row"><button class="primary" on:click=move|_|state.update(|s|{if s.timer.running{s.timer.elapsed=elapsed(&s.timer);s.timer.running=false;}else if s.timer.mode!="Pomodoro"||s.timer.elapsed<1500{s.timer.started=js_sys::Date::now();s.timer.running=true;}})>{move||if state.get().timer.running{"Duraklat"}else{"Başlat"}}</button><button class="secondary" on:click=move|_|{let t=state.get().timer;let seconds=elapsed(&t).min(if t.mode=="Pomodoro"{1500}else{u32::MAX});if seconds==0{return;}state.update(|s|{s.sessions.push(Session{id:id(),date:today(0),subject:t.subject,seconds});s.timer=Timer{mode:t.mode,subject:s.timer.subject.clone(),..Default::default()};});notice.set("Çalışma süren kaydedildi.".into());}>"Bitir ve kaydet"</button><button class="text-button" on:click=move|_|state.update(|s|{s.timer.running=false;s.timer.elapsed=0;})>"Sıfırla"</button></div></section><section class="panel form-panel"><h2>"Son odak oturumların"</h2>{move||state.get().sessions.into_iter().rev().take(10).map(|s|view!{<div class="history-row"><strong>{s.subject}</strong><span>{date_label(&s.date)}</span><span>{clock(s.seconds)}</span></div>}).collect_view()}<Show when=move||state.get().sessions.is_empty()><p class="muted">"Kaydettiğin ilk çalışma burada görünecek."</p></Show></section></Show>
    <Show when=move||page.get()=="net"><div class="page-heading"><div><div class="eyebrow">"GELİŞİMİNİ GÖR"</div><h1>"Her deneme bir pusula."</h1><p>"Doğru ve yanlışlarını gir, netlerini kaydet. 4 yanlış 1 doğruyu götürür."</p></div></div><section class="panel form-panel"><div class="segmented">{["TYT","AYT / YDT"].into_iter().map(|k|view!{<button class:selected=move||kind.get()==k on:click=move|_|kind.set(k.into())>{k}</button>}).collect_view()}</div><form on:submit=move|ev|{let f=fields(ev);if !valid_date(&get(&f,"date")){notice.set("Geçerli bir deneme tarihi seç.".into());return;}let rows=exam_subjects(&kind.get(),&active_track.get());let mut result=vec![];for (i,(name,max)) in rows.iter().enumerate(){let c=num(&f,&format!("c{i}"));let w=num(&f,&format!("w{i}"));if c+w>*max{notice.set(format!("{name}: doğru ve yanlış toplamı {max} soruyu aşamaz."));return;}result.push((name.to_string(),c,w,*max));}let total=result.iter().map(|(_,c,w,_)|net(*c,*w)).sum::<f64>();state.update(|s|s.exams.push(Exam{id:id(),date:get(&f,"date"),kind:kind.get(),rows:result}));notice.set(format!("Denemen kaydedildi. Toplam net: {total:.2}"));}><label class="short-field">"Deneme tarihi"<input name="date" type="date" min="2000-01-01" max="9999-12-31" required prop:value=today(0)/></label><div class="table-scroll"><table><thead><tr><th>"Ders"</th><th>"Doğru"</th><th>"Yanlış"</th><th>"Soru"</th></tr></thead><tbody>{move||exam_subjects(&kind.get(),&active_track.get()).into_iter().enumerate().map(|(i,(name,max))|view!{<tr><th scope="row">{name}</th><td><input aria-label=format!("{name} doğru") type="number" min="0" max=max name=format!("c{i}") value="0" required/></td><td><input aria-label=format!("{name} yanlış") type="number" min="0" max=max name=format!("w{i}") value="0" required/></td><td>{max}</td></tr>}).collect_view()}</tbody></table></div><button class="primary" type="submit">"Netleri hesapla ve kaydet"</button><p class="hint">"Bu araç net hesaplar. Yerleştirme puanı veya sıralama tahmini yapmaz."</p></form></section><section class="panel form-panel"><h2>"Deneme geçmişi"</h2>{move||state.get().exams.into_iter().enumerate().rev().map(|(index,e)|{let total=e.rows.iter().map(|(_,c,w,_)|net(*c,*w)).sum::<f64>();view!{<details class="exam-record"><summary><span>{format!("{} · {}",date_label(&e.date),e.kind)}</span><strong>{format!("{total:.2} net")}</strong></summary>{e.rows.into_iter().map(|(name,c,w,max)|view!{<div class="history-row"><span>{name}</span><span>{format!("{c} D · {w} Y · {} B",max-c-w)}</span><strong>{format!("{:.2}",net(c,w))}</strong></div>}).collect_view()}<button class="text-button" on:click=move|_|state.update(|s|{s.exams.remove(index);})>"Kaydı sil"</button></details>}}).collect_view()}<Show when=move||state.get().exams.is_empty()><p class="muted">"İlk denemeni kaydettiğinde burada göreceksin."</p></Show></section></Show>
    <Show when=move||page.get()=="cards"><div class="page-heading"><div><div class="eyebrow">"HATIRLAMAYA BİR ADIM"</div><h1>"Öğrendiklerin sende kalsın."</h1><p>"Kendi kartlarını hazırla, aralıklı tekrarlarla bilgiyi pekiştir."</p></div></div><div class="cards-layout"><section class="panel review-panel"><label>"Deste"<select on:change=move|e|{deck.set(event_target_value(&e));flipped.set(false);} prop:value="Tümü"><option selected=move||deck.get()=="Tümü">"Tümü"</option>{move||{let mut decks=state.get().cards.iter().map(|c|c.deck.clone()).collect::<Vec<_>>();decks.sort();decks.dedup();decks.into_iter().map(|d|{let label=d.clone();view!{<option selected=move||deck.get()==d>{label}</option>}}).collect_view()}}</select></label>{move||if let Some(c)=active_card(){let card_id=c.id.clone();view!{<div class="flashcard"><small>{c.deck.clone()}</small><h2>{if flipped.get(){c.back}else{c.front}}</h2><span>{if flipped.get(){"CEVAP"}else{"SORU"}}</span></div><Show when=move||!flipped.get()><button class="primary" on:click=move|_|flipped.set(true)>"Cevabı göster"</button></Show><Show when=move||flipped.get()><p class="hint">"Ne kadar kolay hatırladın?"</p><div class="button-row">{[("Tekrar",0),("Zor",1),("İyi",3),("Kolay",7)].into_iter().map(|(label,days)|{let cid=card_id.clone();view!{<button class="secondary" on:click=move|_|{state.update(|s|if let Some(c)=s.cards.iter_mut().find(|c|c.id==cid){c.interval=if days==0{0}else{days.max(c.interval.saturating_mul(2)).min(365)};c.due=shift_date(&today(0),c.interval as i32);});flipped.set(false);if days==0{notice.set("Kart bugün tekrar listesinde kalacak.".into());}}>{label}</button>}}).collect_view()}</div></Show>}.into_any()}else{view!{<div class="empty"><span class="empty-icon">"✧"</span><h2>"Şimdilik tamam."</h2><p>"Bu destede bugün tekrar edilecek kart yok. Yeni kart ekleyebilir ya da yarın geri gelebilirsin."</p></div>}.into_any()}}</section><section class="panel form-panel"><h2>"Yeni hafıza kartı"</h2><form on:submit=move|ev|{let form: web_sys::HtmlFormElement=ev.target().unwrap().unchecked_into();let f=fields(ev);let front=get(&f,"front");let back=get(&f,"back");let d=get(&f,"deck");if front.is_empty()||back.is_empty()||d.is_empty(){return;}state.update(|s|s.cards.push(Card{id:id(),deck:d,front,back,due:today(0),interval:0}));form.reset();notice.set("Kart destene eklendi.".into());}><label>"Deste adı"<input name="deck" placeholder="Örn. Biyoloji · Hücre" maxlength="80" required/></label><label>"Ön yüz / Soru"<textarea name="front" rows="3" maxlength="2000" required placeholder="Hatırlamak istediğin soru…"></textarea></label><label>"Arka yüz / Cevap"<textarea name="back" rows="3" maxlength="4000" required placeholder="Kısa ve anlaşılır bir cevap…"></textarea></label><button class="primary" type="submit">"+ Kartı ekle"</button></form></section></div><section class="panel form-panel"><h2>"Kart kütüphanem"</h2>{move||state.get().cards.into_iter().map(|c|{let cid=c.id.clone();view!{<details class="exam-record"><summary><span>{format!("{} · {}",c.deck,c.front)}</span></summary><p>{c.back}</p><p class="hint">{format!("Sonraki tekrar: {}",date_label(&c.due))}</p><button class="text-button" on:click=move|_|state.update(|s|s.cards.retain(|c|c.id!=cid))>"Kartı sil"</button></details>}}).collect_view()}</section></Show>
    <Show when=move||page.get()=="settings"><div class="page-heading"><div><div class="eyebrow">"TAMAMEN SANA GÖRE"</div><h1>"Çalışma alanını kişiselleştir."</h1><p>"Renklerin, hedeflerin, kendi düzenin."</p></div></div><div class="settings-grid"><section class="panel form-panel"><h2>"Profil ve hedef"</h2><form on:submit=move|ev|{let f=fields(ev);state.update(|s|{s.name=get(&f,"name");s.track=get(&f,"track");});notice.set("Tercihlerin kaydedildi.".into());}><label>"Sana nasıl hitap edelim?"<input name="name" maxlength="40" prop:value=move||state.get().name placeholder="Adın"/></label><label>"Hazırlandığın alan"<select name="track" prop:value=move||state.get().track>{["Sayısal","Eşit ağırlık","Sözel","Dil"].into_iter().map(|t|view!{<option selected=move||state.get().track==t>{t}</option>}).collect_view()}</select></label><button class="primary" type="submit">"Tercihlerimi kaydet"</button></form></section><section class="panel form-panel"><h2>"Rengini bul"</h2><p class="muted">"Cam yüzeyler, sakin renkler. Sana iyi geleni seç."</p><div class="palette-grid">{[("forest","Orman","#225849"),("ocean","Okyanus","#285c85"),("terracotta","Kiremit","#9a4a35"),("plum","Mürdüm","#75466f")].into_iter().map(|(key,label,color)|view!{<button class="palette" class:selected=move||state.get().palette==key on:click=move|_|state.update(|s|s.palette=key.into())><span style=format!("background:{color}")></span>{label}</button>}).collect_view()}</div><label class="toggle"><input type="checkbox" prop:checked=move||state.get().dark on:change=move|e|state.update(|s|s.dark=event_target_checked(&e))/>"Koyu görünüm"</label></section><section class="panel form-panel"><h2>"Verilerin güvende kalsın"</h2><p>"Kayıtlar yalnızca bu tarayıcıda tutulur. Tarayıcı verilerini temizlemek kayıtlarını siler. Düzenli olarak yedek indir; başka cihaza da bu dosyayla taşıyabilirsin."</p><div class="button-row"><button class="primary" on:click=move|_|download(&serde_json::to_string_pretty(&state.get()).unwrap())>"Yedeğimi indir"</button><button class="secondary" on:click=move|_|spawn_local(async move{match read_backup().await{Ok(v)=>{let raw=v.as_string().unwrap_or_default();if raw.is_empty(){return;}match serde_json::from_str::<State>(&raw){Ok(s) if valid(&s)=>restore.set(Some(s)),_=>notice.set("Yedek geçerli değil veya sürümü desteklenmiyor. Mevcut kayıtların korundu.".into())}},Err(_)=>notice.set("Dosya okunamadı. Geçerli bir JSON yedeği seç.".into())}})>"Yedekten geri yükle"</button></div><Show when=move||restore.get().is_some()><div class="notice"><p>{move||restore.get().map(|s|format!("Yedekte {} çalışma, {} kart, {} deneme var. Mevcut kayıtların değiştirilecek.",s.tasks.len(),s.cards.len(),s.exams.len())).unwrap_or_default()}</p><button class="primary" on:click=move|_|{if confirm_restore(){if let Some(s)=restore.get(){force_write.set_value(true);replace_next();state.set(s);notice.set("Yedeğin geri yüklendi.".into());}restore.set(None);}}>"Geri yüklemeyi onayla"</button><button class="secondary" on:click=move|_|restore.set(None)>"Vazgeç"</button></div></Show></section><section class="panel form-panel"><h2>"Pusulan hep yanında"</h2><p>"Chrome veya Edge menüsünden “Uygulamayı yükle”, iPhone’da Safari paylaş menüsünden “Ana Ekrana Ekle” seçeneğini kullan."</p><p class="hint">"Sürüm 0.1.1 · İlk başarılı yüklemeden sonra çevrimdışı çalışır. Yeni sürüm geldiğinde uygulamayı kapatıp açarak güncelleyebilirsin."</p></section></div></Show>
    <button class="settings-mobile" on:click=move|_|page.set("settings".into())>"⚙ Görünüm ve ayarlar"</button><footer class="main-footer"><span>"yks pusulası"</span><span>"Her gün, kendine bir adım daha yakın."</span></footer>
    </Show></main></div>
    <Show when=move||editing.get().is_some()><div class="modal-overlay"><section class="panel modal" role="dialog" aria-modal="true" aria-labelledby="task-heading"><div class="section-head"><h2 id="task-heading">{move||if editing.get().unwrap_or_default().is_empty(){"Yeni çalışma"}else{"Çalışmanı düzenle"}}</h2><button class="icon-button" aria-label="Pencereyi kapat" on:click=move|_|editing.set(None)>"×"</button></div>{move||{let task=editing_task.get();let selected_subject=task.subject.clone();view!{<form on:submit=move|ev|{let f=fields(ev);let title=get(&f,"title");if title.is_empty()||!valid_date(&get(&f,"date")){notice.set("Çalışma adını ve tarihini kontrol et.".into());return;}let edit_id=editing.get().unwrap_or_default();if !edit_id.is_empty()&&!state.get_untracked().tasks.iter().any(|t|t.id==edit_id){editing.set(None);notice.set("Bu çalışma başka bir pencerede silinmiş. Değişiklik kaydedilmedi.".into());return;}state.update(|s|{let eid=editing.get().unwrap_or_default();let done=s.tasks.iter().find(|t|t.id==eid).map(|t|t.done.clone()).unwrap_or_default();let task=Task{id:if eid.is_empty(){id()}else{eid.clone()},title,subject:get(&f,"subject"),date:get(&f,"date"),minutes:num(&f,"minutes"),questions:num(&f,"questions"),weekly:get(&f,"weekly")=="on",done};if let Some(old)=s.tasks.iter_mut().find(|t|t.id==eid){*old=task;}else{s.tasks.push(task);}});editing.set(None);}>
    <label>"Ders"<select name="subject" prop:value=task.subject>{move||subjects(&active_track.get()).into_iter().map(|s|view!{<option selected=s==selected_subject>{s}</option>}).collect_view()}</select></label><label>"Konu / çalışma adı"<input name="title" required maxlength="160" value=task.title placeholder="Örn. Paragraf soru pratiği" autofocus/></label><label>"Tarih"<input type="date" min="2000-01-01" max="9999-12-31" name="date" required value=task.date/></label><div class="form-grid"><label>"Süre (dk)"<input type="number" name="minutes" min="1" max="600" required value=task.minutes/></label><label>"Soru hedefi"<input type="number" name="questions" min="0" max="1000" required value=task.questions/></label></div><label class="toggle"><input type="checkbox" name="weekly" checked=task.weekly/>"Her hafta aynı gün tekrarla"</label><button type="submit" class="primary full">"Çalışmayı kaydet"</button></form>}}}</section></div></Show>
    </div>
    }
}
fn main() {
    console_error_panic_hook::set_once();
    init();
    leptos::mount::mount_to_body(App);
}
