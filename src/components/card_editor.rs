use crate::{
    fields, get,
    model::{edit_card, Card, State},
};
use leptos::prelude::*;
#[component]
pub fn CardEditor(
    state: RwSignal<State>,
    editing: RwSignal<Option<String>>,
    notice: RwSignal<String>,
    flipped: RwSignal<bool>,
) -> impl IntoView {
    let error = RwSignal::new(String::new());
    Effect::new(move |_| {
        let _ = editing.get();
        error.set(String::new());
    });
    let draft = Memo::new(move |_| {
        let id = editing.get().unwrap_or_default();
        state
            .get_untracked()
            .cards
            .into_iter()
            .find(|c| c.id == id)
            .unwrap_or(Card {
                id: String::new(),
                deck: String::new(),
                front: String::new(),
                back: String::new(),
                due: String::new(),
                interval: 0,
            })
    });
    view! {<Show when=move||editing.get().is_some()><div class="modal-overlay"><section class="panel modal" role="dialog" aria-modal="true" aria-labelledby="card-edit-heading"><div class="section-head"><h2 id="card-edit-heading">"Hafıza kartını düzenle"</h2><button class="icon-button" aria-label="Pencereyi kapat" on:click=move|_|editing.set(None)>"×"</button></div><p class="hint">"Tekrar tarihi ve öğrenme aralığın korunur."</p><Show when=move||!error.get().is_empty()><p class="notice" role="alert">{move||error.get()}</p></Show>{move||{let card=draft.get();view!{<form on:submit=move|event|{let f=fields(event);let mut result=Err("Kart kaydedilemedi.");if let Some(id)=editing.get_untracked(){state.update(|s|result=edit_card(&mut s.cards,&id,&get(&f,"deck"),&get(&f,"front"),&get(&f,"back")));}match result{Ok(())=>{editing.set(None);flipped.set(false);notice.set("Kart güncellendi. Tekrar planın korundu.".into());},Err(message)=>error.set(message.into())}}><label>"Deste adı"<input name="deck" maxlength="80" required value=card.deck/></label><label>"Ön yüz / Soru"<textarea name="front" rows="4" maxlength="2000" required>{card.front}</textarea></label><label>"Arka yüz / Cevap"<textarea name="back" rows="5" maxlength="4000" required>{card.back}</textarea></label><button class="primary full" type="submit">"Değişiklikleri kaydet"</button></form>}}}</section></div></Show>}
}
