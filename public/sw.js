const SCOPE = new URL('./', self.location);
const PREFIX = `yks-pusulasi:${SCOPE.pathname}:`;
const CACHE = `${PREFIX}v4`;
function inScope(url) {
  return url.origin === SCOPE.origin && url.pathname.startsWith(SCOPE.pathname);
}
self.addEventListener('install', event => {
  event.waitUntil((async () => {
    const response = await fetch(SCOPE.href, { cache: 'reload' });
    if (!response.ok) throw Error('Offline shell failed');
    const html = await response.clone().text(), cache = await caches.open(CACHE);
    const assets = new Set([...html.matchAll(/(?:href|src)="([^"]+\.(?:js|wasm|css|png|svg|webmanifest))"/g)]
      .map(m => new URL(m[1], SCOPE).href));
    assets.add(new URL('icon-192.png', SCOPE).href);
    assets.add(new URL('icon-512.png', SCOPE).href);
    const seen = new Set();
    async function cacheAsset(url) {
      if (seen.has(url) || !inScope(new URL(url))) return;
      seen.add(url);
      const res = await fetch(url, { cache: 'reload' });
      if (!res.ok) throw Error(`Asset failed: ${url}`);
      await cache.put(url, res.clone());
      if (new URL(url).pathname.endsWith('.js')) {
        const js = await res.text();
        for (const m of js.matchAll(/(?:from\s*|import\s*)["'](\.[^"']+\.js)["']/g)) {
          await cacheAsset(new URL(m[1], url).href);
        }
      }
    }
    await Promise.all([...assets].map(cacheAsset));
    await cache.put(SCOPE.href, response);
  })());
});
self.addEventListener('activate', event => {
  event.waitUntil((async () => {
    for (const key of await caches.keys()) {
      if (key.startsWith(PREFIX) && key !== CACHE) await caches.delete(key);
    }
    await self.clients.claim();
  })());
});
self.addEventListener('fetch', event => {
  if (event.request.method !== 'GET' || !inScope(new URL(event.request.url))) return;
  event.respondWith((async () => {
    const cache = await caches.open(CACHE);
    const cached = await cache.match(event.request);
    try {
      const response = await fetch(event.request);
      if (response.status >= 500 && cached) return cached;
      if (response.ok) await cache.put(event.request, response.clone());
      return response;
    } catch (error) {
      return cached || (event.request.mode === 'navigate' ? await cache.match(SCOPE.href) : undefined) || Response.error();
    }
  })());
});
