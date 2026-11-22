import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
async function worker(){
 const handlers={},stores=new Map(),deleted=[],base='https://example.test/yks-pusulasi/';let mode='online';
 const bodies=new Map([[base,'<link href="style.css"><link href="app_bg.wasm"><script src="app.js"></script>'],[base+'app.js',"import { load } from './snippets/storage.js';"],[base+'snippets/storage.js','export function load() {}']]);
 const cache=name=>{if(!stores.has(name))stores.set(name,new Map());const store=stores.get(name);return{put:async(req,res)=>store.set(typeof req==='string'?req:req.url,res),match:async req=>store.get(typeof req==='string'?req:req.url)?.clone()};};
 const context={URL,Response,console,Set,Promise,self:{location:new URL(base+'sw.js'),clients:{claim:async()=>{}},addEventListener(k,fn){handlers[k]=fn;}},fetch:async req=>{if(mode==='offline')throw Error('offline');if(mode==='server-error')return new Response('unavailable',{status:503});return new Response(bodies.get(typeof req==='string'?req:req.url)||'asset');},caches:{open:async name=>cache(name),keys:async()=>[...stores.keys()],delete:async name=>{deleted.push(name);stores.delete(name);}}};
 vm.runInNewContext(await readFile(new URL('../public/sw.js',import.meta.url),'utf8'),context);
 async function lifecycle(name){let p;handlers[name]({waitUntil(value){p=value;}});await p;}
 async function fetchAsset(path,requestMode='cors'){let p;handlers.fetch({request:{url:new URL(path,base).href,method:'GET',mode:requestMode},respondWith(value){p=value;}});return p;}
 return{base,stores,deleted,cache,lifecycle,fetchAsset,setMode(value){mode=value;}};
}
test('PWA caches nested WASM bridge and serves shell and assets offline',async()=>{
 const w=await worker();await w.lifecycle('install');const store=[...w.stores.values()][0];assert(store.has(w.base+'app_bg.wasm'));assert(store.has(w.base+'snippets/storage.js'));assert(store.has(w.base+'icon-192.png'));
 w.setMode('offline');assert.match(await (await w.fetchAsset('./','navigate')).text(),/app.js/);assert.match(await (await w.fetchAsset('snippets/storage.js')).text(),/export function/);
});
test('PWA upgrade only deletes older caches from its own scope',async()=>{
 const w=await worker();w.cache('another-application-v1');w.cache('yks-pusulasi:/another-repo/:v1');w.cache('yks-pusulasi:/yks-pusulasi/:v1');await w.lifecycle('install');await w.lifecycle('activate');
 assert.deepEqual(w.deleted,['yks-pusulasi:/yks-pusulasi/:v1']);assert(w.stores.has('another-application-v1'));assert(w.stores.has('yks-pusulasi:/another-repo/:v1'));
 assert.equal(await w.fetchAsset('/another-repo/data.json'),undefined);
});
test('PWA falls back on server failure and never serves a different scope cache',async()=>{
 const w=await worker();await w.lifecycle('install');w.setMode('server-error');assert.match(await (await w.fetchAsset('./','navigate')).text(),/app.js/);
 await w.cache('other').put(w.base+'missing.js',new Response('wrong'));w.setMode('offline');assert.equal((await w.fetchAsset('missing.js')).type,'error');
});
