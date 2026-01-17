const CACHE_NAME = 'inventory-v1';
const urlsToCache = [
  '/',
  '/index.html',
  '/styles.css',
  '/pkg/inventary_frontend.js',
  '/pkg/inventary_frontend_bg.wasm',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => cache.addAll(urlsToCache))
  );
});

self.addEventListener('fetch', (event) => {
  event.respondWith(
    caches.match(event.request)
      .then((response) => {
        if (response) {
          return response;
        }
        return fetch(event.request);
      })
  );
});
