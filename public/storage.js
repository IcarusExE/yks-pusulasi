let dbPromise, queue = Promise.resolve(), viewBase = null, revision = 0;
let replacePending = false, failedBase = null;
const subscribers = new Set();
let channel;
function db() {
  return dbPromise ||= new Promise((resolve, reject) => {
    const req = indexedDB.open('yks-pusulasi', 1);
    req.onupgradeneeded = () => req.result.createObjectStore('state');
    req.onsuccess = () => {
      req.result.onversionchange = () => { req.result.close(); dbPromise = null; };
      resolve(req.result);
    };
    req.onerror = () => { dbPromise = null; reject(req.error); };
    req.onblocked = () => { dbPromise = null; reject(new Error('Diğer sekmeleri kapatıp tekrar dene.')); };
  });
}
const collections = ['tasks', 'cards', 'exams', 'sessions'];
const equal = (a, b) => canonical(a) === canonical(b);
function canonical(value) {
  if (value === undefined) return 'undefined';
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  return '{' + Object.keys(value).sort().map(k => JSON.stringify(k) + ':' + canonical(value[k])).join(',') + '}';
}
function legacyId(kind, item, index) {
  let hash = 2166136261;
  for (const c of canonical(item)) hash = Math.imul(hash ^ c.charCodeAt(0), 16777619) >>> 0;
  return `legacy-${kind}-${index}-${hash.toString(16)}`;
}
// Add stable identities to pre-sync exam/session records without changing schema v1.
export function normalizeState(value) {
  const state = structuredClone(value);
  if (!state || typeof state !== 'object' || Array.isArray(state)) throw new Error('Geçersiz kayıt');
  for (const kind of ['exams', 'sessions']) {
    if (!Array.isArray(state[kind])) continue;
    state[kind] = state[kind].map((item, index) => ({ ...item, id: item.id || legacyId(kind, item, index) }));
  }
  return state;
}
function mergeSet(base, local, remote) {
  const result = new Set(remote);
  for (const value of base) if (!local.includes(value)) result.delete(value);
  for (const value of local) if (!base.includes(value)) result.add(value);
  return [...result].sort();
}
function mergeEntity(base, local, remote, kind) {
  const result = structuredClone(remote);
  for (const key of Object.keys(local)) {
    if (kind === 'tasks' && key === 'done') result.done = mergeSet(base.done || [], local.done || [], remote.done || []);
    else if (!equal(base[key], local[key])) result[key] = structuredClone(local[key]);
  }
  return result;
}
export function mergeState(base, local, remote) {
  if (!remote) return structuredClone(local);
  const result = structuredClone(remote);
  for (const key of Object.keys(local)) {
    if (!collections.includes(key) && !equal(base[key], local[key])) result[key] = structuredClone(local[key]);
  }
  for (const kind of collections) {
    const old = new Map((base[kind] || []).map(item => [item.id, item]));
    const mine = new Map((local[kind] || []).map(item => [item.id, item]));
    const latest = new Map((remote[kind] || []).map(item => [item.id, item]));
    for (const id of old.keys()) if (!mine.has(id)) latest.delete(id);
    for (const [id, item] of mine) {
      if (!old.has(id)) latest.set(id, structuredClone(item));
      else if (latest.has(id)) latest.set(id, mergeEntity(old.get(id), item, latest.get(id), kind));
      // A concurrent deletion wins over an edit; stale windows cannot resurrect it.
    }
    result[kind] = [...latest.values()];
  }
  return result;
}
async function readRaw() {
  const d = await db();
  return new Promise((resolve, reject) => {
    const tx = d.transaction('state', 'readonly'), req = tx.objectStore('state').get('main');
    req.onsuccess = () => resolve(req.result || '');
    req.onerror = () => reject(req.error);
    tx.onabort = () => reject(tx.error);
  });
}
export function rawBackup() { return readRaw(); }
export async function load() {
  const raw = await readRaw();
  viewBase = raw ? normalizeState(JSON.parse(raw)) : null;
  return viewBase ? JSON.stringify(viewBase) : '';
}
export function acceptSnapshot(raw) { viewBase = normalizeState(JSON.parse(raw)); }
export function replaceNext() { replacePending = true; }
export function save(value) {
  const local = normalizeState(JSON.parse(value));
  const base = viewBase || { ...local, tasks: [], cards: [], exams: [], sessions: [] }, replace = replacePending;
  replacePending = false;
  viewBase = local;
  revision++;
  const next = queue.catch(() => {}).then(async () => {
    const effectiveBase = failedBase || base;
    try {
      const d = await db();
      const raw = await new Promise((resolve, reject) => {
        // Read + merge + write share a single transaction, serialized by IndexedDB
        // across windows. BroadcastChannel is only a notification transport.
        const tx = d.transaction('state', 'readwrite'), store = tx.objectStore('state');
        const req = store.get('main');
        let output;
        req.onsuccess = () => {
          try {
            const remote = req.result ? normalizeState(JSON.parse(req.result)) : null;
            output = JSON.stringify(replace ? local : mergeState(effectiveBase, local, remote));
            store.put(output, 'main');
          } catch (error) { tx.abort(); reject(error); }
        };
        tx.oncomplete = () => resolve(output);
        tx.onerror = () => reject(tx.error);
        tx.onabort = () => reject(tx.error);
      });
      failedBase = null;
      channel?.postMessage('changed');
      return raw;
    } catch (error) { failedBase = effectiveBase; throw error; }
  });
  queue = next;
  return next;
}
let refreshRunning = false, refreshAgain = false;
async function refresh() {
  if (refreshRunning) { refreshAgain = true; return; }
  refreshRunning = true;
  try {
    do {
      refreshAgain = false;
      const ticket = revision;
      await queue.catch(() => {});
      if (failedBase) break; // Keep unsaved local changes after a storage failure.
      const raw = await readRaw();
      if (ticket !== revision) { refreshAgain = true; continue; }
      if (raw) for (const callback of subscribers) callback(JSON.stringify(normalizeState(JSON.parse(raw))));
    } while (refreshAgain);
  } catch (error) { console.error('Eşitleme okunamadı', error); }
  finally { refreshRunning = false; }
}
export function unsubscribe(callback) {
  subscribers.delete(callback);
  if (!subscribers.size && channel) { channel.close(); channel = undefined; }
}
export function subscribe(callback) {
  subscribers.add(callback);
  if (!channel && typeof BroadcastChannel !== 'undefined') {
    channel = new BroadcastChannel('yks-pusulasi-state-v1');
    channel.onmessage = () => refresh();
  }
  // Also catches a change between initial load and subscription.
  refresh();
}
export function today(offset=0) {const d=new Date();d.setDate(d.getDate()+offset);return dateKey(d);}
function dateKey(d){return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`;}
export function shiftDate(key,offset){const d=new Date(key+'T12:00:00');d.setDate(d.getDate()+offset);return dateKey(d);}
export function dateLabel(key){return new Date(key+'T12:00:00').toLocaleDateString('tr-TR',{day:'numeric',month:'long',weekday:'long'});}
export function weekday(key){return new Date(key+'T12:00:00').getDay();}
export function theme(palette,dark){document.documentElement.dataset.palette=palette;document.documentElement.dataset.mode=dark?'dark':'light';}
export function download(value){const url=URL.createObjectURL(new Blob([value],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=`yks-pusulasi-${today()}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
export function readBackup(){return new Promise((resolve,reject)=>{const input=document.createElement('input');input.type='file';input.accept='.json,application/json';input.oncancel=()=>resolve('');input.onchange=async()=>{try{const file=input.files[0];if(file.size>10*1024*1024)throw Error('Dosya 10 MB sınırını aşıyor.');resolve(JSON.stringify(normalizeState(JSON.parse(await file.text()))));}catch(e){reject(e);}};input.click();});}
export function confirmRestore(){return confirm('Yedekteki veriler mevcut kayıtlarının yerini alacak. Devam edilsin mi?');}
export function init(){
 window.addEventListener("focus",()=>refresh());
 document.addEventListener("visibilitychange",()=>{if(document.visibilityState==="visible")refresh();});
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
