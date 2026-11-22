import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {IDBFactory} from 'fake-indexeddb';
const source=await readFile(new URL('../public/storage.js',import.meta.url),'utf8');
let sequence=0;
async function client(){return import('data:text/javascript;base64,'+Buffer.from(source).toString('base64')+'#'+sequence++);}
const snapshot=()=>({version:1,name:'',track:'Sayısal',palette:'forest',dark:false,tasks:[],cards:[],exams:[],sessions:[],topic_progress:[],timer:{running:false,started:0,elapsed:0,mode:'Kronometre',subject:'TYT Matematik'}});
const task=(id,title=id)=>({id,title,subject:'TYT Matematik',date:'2026-11-22',minutes:45,questions:20,weekly:true,done:[]});
async function write(c,s){return JSON.parse(await c.save(JSON.stringify(s)));}
async function setup(initial=snapshot()){
 globalThis.indexedDB=new IDBFactory();
 const a=await client();await a.load();await write(a,initial);
 const b=await client();const [baseA,baseB]=await Promise.all([a.load(),b.load()]);
 return {a,b,baseA:JSON.parse(baseA),baseB:JSON.parse(baseB)};
}
test('date arithmetic crosses month and year boundaries',async()=>{
 const h=await client();assert.equal(h.shiftDate('2026-12-31',1),'2027-01-01');assert.equal(h.shiftDate('2028-03-01',-1),'2028-02-29');assert.equal(h.weekday('2026-11-22'),0);
});
test('independent concurrent task additions survive reload',async()=>{
 const {a,b,baseA,baseB}=await setup();baseA.tasks.push(task('a'));baseB.tasks.push(task('b'));
 await Promise.all([write(a,baseA),write(b,baseB)]);
 const reopened=await client();const saved=JSON.parse(await reopened.load());assert.deepEqual(saved.tasks.map(t=>t.id).sort(),['a','b']);
});
test('different fields and completion dates on the same task merge',async()=>{
 const initial=snapshot();initial.tasks=[task('a')];const {a,b,baseA,baseB}=await setup(initial);
 baseA.tasks[0].title='Yeni başlık';baseA.tasks[0].done=['2026-11-22'];baseB.tasks[0].questions=50;baseB.tasks[0].done=['2026-11-23'];
 await Promise.all([write(a,baseA),write(b,baseB)]);const saved=JSON.parse(await a.load());
 assert.equal(saved.tasks[0].title,'Yeni başlık');assert.equal(saved.tasks[0].questions,50);assert.deepEqual(saved.tasks[0].done,['2026-11-22','2026-11-23']);
});
test('same-field conflict uses last transaction and deletions do not resurrect',async()=>{
 const initial=snapshot();initial.tasks=[task('a')];const {a,b,baseA,baseB}=await setup(initial);
 baseA.tasks[0].title='İlk';baseB.tasks[0].title='Son';await write(a,baseA);await write(b,baseB);assert.equal(JSON.parse(await a.load()).tasks[0].title,'Son');
 const stale=JSON.parse(await b.load());const deleted=JSON.parse(await a.load());deleted.tasks=[];await write(a,deleted);stale.tasks[0].title='Eski pencere';await write(b,stale);assert.deepEqual(JSON.parse(await a.load()).tasks,[]);
});
test('queued rapid edits preserve remote changes and their local predecessors',async()=>{
 const {a,b,baseA,baseB}=await setup();baseB.cards.push({id:'card',deck:'Fen',front:'Q',back:'A',due:'2026-11-22',interval:0});await write(b,baseB);
 baseA.tasks.push(task('a'));const first=write(a,baseA);baseA.tasks[0].questions=60;const second=write(a,baseA);baseA.name='Ada';const third=write(a,baseA);
 await Promise.all([first,second,third]);const result=JSON.parse(await a.load());assert.equal(result.cards.length,1);assert.equal(result.tasks[0].questions,60);assert.equal(result.name,'Ada');
});
test('legacy record identities stay stable across clients and explicit restore replaces data',async()=>{
 const initial=snapshot();initial.exams=[{date:'2026-11-22',kind:'TYT',rows:[]}];initial.sessions=[{date:'2026-11-22',subject:'TYT',seconds:60}];
 const {a,b,baseA,baseB}=await setup(initial);assert.equal(baseA.exams[0].id,baseB.exams[0].id);assert.equal(baseA.sessions[0].id,baseB.sessions[0].id);
 baseA.exams=[];baseB.sessions=[];await Promise.all([write(a,baseA),write(b,baseB)]);const result=JSON.parse(await a.load());assert.equal(result.exams.length,0);assert.equal(result.sessions.length,0);
 a.replaceNext();await write(a,snapshot());assert.deepEqual(JSON.parse(await b.load()),snapshot());
});
test('failed write retains unsaved fields for next edit',async()=>{
 const {a,baseA}=await setup();const actual=globalThis.indexedDB;const realOpen=actual.open.bind(actual);
 // Abort the next read/write transaction, as a quota failure would.
 const request=realOpen('yks-pusulasi',1);const database=await new Promise((resolve,reject)=>{request.onsuccess=()=>resolve(request.result);request.onerror=reject;});
 const proto=Object.getPrototypeOf(database), original=proto.transaction;let abortNext=true;
 proto.transaction=function(...args){const tx=original.apply(this,args);if(args[1]==='readwrite'&&abortNext){abortNext=false;queueMicrotask(()=>tx.abort());}return tx;};
 try{baseA.name='Kaybolmamalı';await assert.rejects(write(a,baseA));baseA.dark=true;await write(a,baseA);const result=JSON.parse(await a.load());assert.equal(result.name,'Kaybolmamalı');assert.equal(result.dark,true);}finally{proto.transaction=original;database.close();}
});
test('broadcast updates another client without echo writes',async()=>{
 const native=globalThis.BroadcastChannel;
 const channels=new Set();
 globalThis.BroadcastChannel=class{constructor(){channels.add(this);}postMessage(data){for(const c of channels)if(c!==this)queueMicrotask(()=>c.onmessage?.({data}));}close(){channels.delete(this);}};
 const {a,b,baseA}=await setup();let resolveUpdate;const updated=new Promise(resolve=>resolveUpdate=resolve);
 const accept=raw=>{const s=JSON.parse(raw);b.acceptSnapshot(raw);if(s.tasks.some(t=>t.id==='broadcast'))resolveUpdate(s);};
 try{a.subscribe(()=>{});b.subscribe(accept);baseA.tasks.push(task('broadcast'));await write(a,baseA);const result=await Promise.race([updated,new Promise((_,reject)=>setTimeout(()=>reject(Error('No sync notification')),1000))]);assert.equal(result.tasks[0].id,'broadcast');}
 finally{b.unsubscribe(accept);globalThis.BroadcastChannel=native;for(const c of channels)c.close();}
});

test('corrupted storage can still be exported without overwriting it',async()=>{
 globalThis.indexedDB=new IDBFactory();const c=await client();await c.load();
 const req=indexedDB.open('yks-pusulasi',1);const d=await new Promise(resolve=>req.onsuccess=()=>resolve(req.result));
 await new Promise((resolve,reject)=>{const tx=d.transaction('state','readwrite');tx.objectStore('state').put('{broken json','main');tx.oncomplete=resolve;tx.onerror=reject;});
 await assert.rejects(c.load());assert.equal(await c.rawBackup(),'{broken json');d.close();
});

test('fresh windows merge their first additions and default initialization does not erase them',async()=>{
 globalThis.indexedDB=new IDBFactory();const a=await client(),b=await client();await Promise.all([a.load(),b.load()]);
 a.acceptSnapshot(JSON.stringify(snapshot()));b.acceptSnapshot(JSON.stringify(snapshot()));
 const first=snapshot();first.tasks.push(task('first'));first.name='Ada';await write(a,first);await write(b,snapshot());
 const saved=JSON.parse(await b.load());assert.equal(saved.tasks[0].id,'first');assert.equal(saved.name,'Ada');
});

test('moving two different weekly occurrences preserves both exceptions and destinations',async()=>{
 const initial=snapshot();initial.tasks=[{...task('series'),skipped:[]}];const {a,b,baseA,baseB}=await setup(initial);
 for(const [local,day,target] of [[baseA,'2026-11-22','2026-11-23'],[baseB,'2026-11-29','2026-11-30']]){local.tasks[0].skipped=[day];local.tasks.push({...task(`moved@series@${day}`),date:target,weekly:false,skipped:[]});}
 await Promise.all([write(a,baseA),write(b,baseB)]);const saved=JSON.parse(await a.load());assert.equal(saved.tasks.length,3);assert.deepEqual(saved.tasks[0].skipped,['2026-11-22','2026-11-29']);
});
test('concurrent moves of the same occurrence use one stable destination identity',async()=>{
 const initial=snapshot();initial.tasks=[{...task('series'),skipped:[]}];const {a,b,baseA,baseB}=await setup(initial);
 for(const [local,target] of [[baseA,'2026-11-23'],[baseB,'2026-11-24']]){local.tasks[0].skipped=['2026-11-22'];local.tasks.push({...task('moved@series@2026-11-22'),date:target,weekly:false,skipped:[]});}
 await write(a,baseA);await write(b,baseB);const saved=JSON.parse(await a.load());assert.equal(saved.tasks.length,2);assert.equal(saved.tasks[1].date,'2026-11-24');
});

test('editing a card in one window preserves review progress recorded by another',async()=>{
 const initial=snapshot();initial.cards=[{id:'c',deck:'Fen',front:'Soru',back:'Cevap',due:'2026-11-22',interval:0}];const {a,b,baseA,baseB}=await setup(initial);
 baseA.cards[0].due='2026-11-29';baseA.cards[0].interval=7;baseB.cards[0].deck='Biyoloji';baseB.cards[0].front='Yeni soru';await write(a,baseA);await write(b,baseB);
 const card=JSON.parse(await a.load()).cards[0];assert.equal(card.interval,7);assert.equal(card.due,'2026-11-29');assert.equal(card.front,'Yeni soru');assert.equal(card.deck,'Biyoloji');
});

test('topic progress for different topics merges across windows and survives reload',async()=>{
 const {a,b,baseA,baseB}=await setup();baseA.topic_progress=[{id:'2026/math-a',status:'in_progress'}];baseB.topic_progress=[{id:'2026/math-b',status:'completed'}];await Promise.all([write(a,baseA),write(b,baseB)]);const c=await client();const loaded=JSON.parse(await c.load());assert.deepEqual(loaded.topic_progress.map(p=>p.id).sort(),['2026/math-a','2026/math-b']);
});
test('a progress reset is retained as an explicit record and same-topic conflicts use last write',async()=>{
 const initial=snapshot();initial.topic_progress=[{id:'2026/math-a',status:'needs_review'}];const {a,b,baseA,baseB}=await setup(initial);baseA.topic_progress[0].status='completed';baseB.topic_progress[0].status='not_started';await write(a,baseA);await write(b,baseB);assert.deepEqual(JSON.parse(await a.load()).topic_progress,[{id:'2026/math-a',status:'not_started'}]);
});
test('legacy snapshots gain empty progress and linked tasks remain linked after moving dates',async()=>{
 const {a,b,baseA}=await setup();delete baseA.topic_progress;baseA.tasks=[{...task('linked'),topic_id:'2026/math-a'}];await write(a,baseA);const loaded=JSON.parse(await b.load());assert.deepEqual(loaded.topic_progress,[]);loaded.tasks[0].date='2026-11-23';await write(b,loaded);assert.equal(JSON.parse(await a.load()).tasks[0].topic_id,'2026/math-a');
});
test('concurrent acceptance of the same recommendation cycle creates one task',async()=>{
 const {a,b,baseA,baseB}=await setup();baseA.tasks=[{...task('topic-plan@2026/math-a@0'),weekly:false,topic_id:'2026/math-a'}];baseB.tasks=[{...task('topic-plan@2026/math-a@0'),date:'2026-11-23',weekly:false,topic_id:'2026/math-a'}];await write(a,baseA);await write(b,baseB);const loaded=JSON.parse(await a.load());assert.equal(loaded.tasks.length,1);assert.equal(loaded.tasks[0].date,'2026-11-23');
});
