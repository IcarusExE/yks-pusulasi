import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import vm from 'node:vm';
test('PWA caches nested WASM bridge and serves shell offline under repository path',async()=>{
 const handlers={},stored=new Map(),origin='https://example.test',base=origin+'/yks-pusulasi/';let offline=false;
 const bodies=new Map([[base,'<link href="style.css"><link href="app_bg.wasm"><script src="app.js"></script>'],[base+'app.js',"import { load } from './snippets/storage.js';"],[base+'snippets/storage.js','export function load() {}']]);
 const context={URL,Response,console,Set,Promise,self:{location:new URL(base+'sw.js'),clients:{claim:async()=>{}},addEventListener(k,fn){handlers[k]=fn;}},fetch:async req=>{if(offline)throw Error('offline');const url=typeof req==='string'?req:req.url;return new Response(bodies.get(url)||'asset');},caches:{open:async()=>({put:async(req,res)=>stored.set(typeof req==='string'?req:req.url,res)}),keys:async()=>['pusula-v1'],delete:async()=>{},match:async req=>{const hit=stored.get(typeof req==='string'?req:req.url);return hit?.clone();}}};
 vm.runInNewContext(await readFile(new URL('../public/sw.js',import.meta.url),'utf8'),context);
 let installing;handlers.install({waitUntil(p){installing=p;}});await installing;
 assert(stored.has(base+'app_bg.wasm'));assert(stored.has(base+'snippets/storage.js'));assert(stored.has(base+'icon-192.png'));
 offline=true;let result;handlers.fetch({request:{url:base,method:'GET',mode:'navigate'},respondWith(p){result=p;}});assert.match(await (await result).text(),/app.js/);
 handlers.fetch({request:{url:base+'snippets/storage.js',method:'GET',mode:'cors'},respondWith(p){result=p;}});assert.match(await (await result).text(),/export function/);
});
