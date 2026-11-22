import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
const source=await readFile(new URL('../public/storage.js',import.meta.url),'utf8');
const helpers=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64'));
test('date arithmetic crosses month and year boundaries',()=>{assert.equal(helpers.shiftDate('2026-12-31',1),'2027-01-01');assert.equal(helpers.shiftDate('2028-03-01',-1),'2028-02-29');assert.equal(helpers.weekday('2026-11-22'),0);});
test('storage queues writes and propagates transaction failures',async()=>{
 let committed=[], active=0;
 globalThis.indexedDB={open(){let req={};queueMicrotask(()=>{req.result={transaction(){active++;assert.equal(active,1);let tx={objectStore(){return{put(v){tx.value=v;}}}};setTimeout(()=>{active--;if(tx.value==='bad'){tx.error=Error('quota');tx.onerror();}else{committed.push(tx.value);tx.oncomplete();}},5);return tx;}};req.onsuccess();});return req;}};
 await Promise.all([helpers.save('first'),helpers.save('second')]);
 await assert.rejects(helpers.save('bad'),/quota/);
 await helpers.save('after');assert.deepEqual(committed,['first','second','after']);
});
