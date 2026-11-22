let dbPromise, queue = Promise.resolve();
function db() {
  return dbPromise ||= new Promise((resolve,reject) => {
    const req = indexedDB.open('yks-pusulasi',1);
    req.onupgradeneeded = () => req.result.createObjectStore('state');
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
    req.onblocked = () => reject(new Error('Diğer sekmeleri kapatıp tekrar dene.'));
  });
}
export async function load() {
  const d = await db();
  return new Promise((resolve,reject) => {
    const tx=d.transaction('state','readonly'), req=tx.objectStore('state').get('main');
    req.onsuccess=()=>resolve(req.result || ''); req.onerror=()=>reject(req.error);
  });
}
export function save(value) {
  const next=queue.catch(()=>{}).then(async()=>{
    const d=await db();
    await new Promise((resolve,reject)=>{
      const tx=d.transaction('state','readwrite'); tx.objectStore('state').put(value,'main');
      tx.oncomplete=resolve; tx.onerror=()=>reject(tx.error); tx.onabort=()=>reject(tx.error);
    });
  });
  queue=next; return next;
}
export function today(offset=0) {const d=new Date();d.setDate(d.getDate()+offset);return dateKey(d);}
function dateKey(d){return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;}
export function shiftDate(key,offset){const d=new Date(key+'T12:00:00');d.setDate(d.getDate()+offset);return dateKey(d);}
export function dateLabel(key){return new Date(key+'T12:00:00').toLocaleDateString('tr-TR',{day:'numeric',month:'long',weekday:'long'});}
export function weekday(key){return new Date(key+'T12:00:00').getDay();}
export function theme(palette,dark){document.documentElement.dataset.palette=palette;document.documentElement.dataset.mode=dark?'dark':'light';}
export function download(value){const url=URL.createObjectURL(new Blob([value],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=`yks-pusulasi-${today()}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
export function readBackup(){return new Promise((resolve,reject)=>{const input=document.createElement('input');input.type='file';input.accept='.json,application/json';input.oncancel=()=>resolve('');input.onchange=async()=>{try{const file=input.files[0];if(file.size>10*1024*1024)throw Error('Dosya 10 MB sınırını aşıyor.');resolve(await file.text());}catch(e){reject(e);}};input.click();});}
export function confirmRestore(){return confirm('Yedekteki veriler mevcut kayıtlarının yerini alacak. Devam edilsin mi?');}
export function init(){
 const register=()=>navigator.serviceWorker.register('./sw.js').catch(console.error);
 if('serviceWorker' in navigator){if(document.readyState==='complete')register();else window.addEventListener('load',register,{once:true});}
 document.addEventListener('keydown',e=>{
  const modal=document.querySelector('[role=dialog]');if(!modal)return;
  const controls=[...modal.querySelectorAll('button,input,select,textarea')].filter(e=>!e.disabled);
  if(e.key==='Escape'){modal.querySelector('[aria-label="Pencereyi kapat"]').click();return;}
  if(e.key==='Tab'&&controls.length){const first=controls[0],last=controls.at(-1);if(e.shiftKey&&(document.activeElement===first||!modal.contains(document.activeElement))){e.preventDefault();last.focus();}else if(!e.shiftKey&&(document.activeElement===last||!modal.contains(document.activeElement))){e.preventDefault();first.focus();}}
 });
 let previous=null;
 new MutationObserver(()=>{const modal=document.querySelector('[role=dialog]');if(modal&&!previous){previous=document.activeElement;modal.querySelector('input[autofocus],input,button')?.focus();}else if(!modal&&previous){previous?.focus?.();previous=null;}}).observe(document.body,{childList:true,subtree:true});
}
export function notifyDone(){if(navigator.vibrate)navigator.vibrate([150,80,150]);}
