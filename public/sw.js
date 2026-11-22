const CACHE='pusula-v1';
self.addEventListener('install',event=>{event.waitUntil((async()=>{
 const base=new URL('./',self.location).href;
 const response=await fetch(base,{cache:'reload'});if(!response.ok)throw Error('Offline shell failed');
 const html=await response.clone().text(),cache=await caches.open(CACHE);
 const assets=new Set([...html.matchAll(/(?:href|src)="([^"]+\.(?:js|wasm|css|png|svg|webmanifest))"/g)].map(m=>new URL(m[1],base).href));
 assets.add(new URL('icon-192.png',base).href);assets.add(new URL('icon-512.png',base).href);
 const seen=new Set();
 async function cacheAsset(url){if(seen.has(url))return;seen.add(url);const res=await fetch(url,{cache:'reload'});if(!res.ok)throw Error(`Asset failed: ${url}`);await cache.put(url,res.clone());if(new URL(url).pathname.endsWith('.js')){const js=await res.text();for(const m of js.matchAll(/(?:from\s*|import\s*)["'](\.[^"']+\.js)["']/g)){await cacheAsset(new URL(m[1],url).href);}}}
 await Promise.all([...assets].map(cacheAsset));await cache.put(base,response);
})());});
self.addEventListener('activate',event=>{event.waitUntil((async()=>{for(const key of await caches.keys())if(key!==CACHE)await caches.delete(key);await self.clients.claim();})());});
self.addEventListener('fetch',event=>{if(event.request.method!=='GET'||new URL(event.request.url).origin!==self.location.origin)return;event.respondWith((async()=>{try{const response=await fetch(event.request);if(response.ok){const cache=await caches.open(CACHE);await cache.put(event.request,response.clone());}return response;}catch(error){return await caches.match(event.request)||(event.request.mode==='navigate'?await caches.match(new URL('./',self.location).href):undefined)||Response.error();}})());});
