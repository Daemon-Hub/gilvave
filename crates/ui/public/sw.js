/**
 * Gilvave Service Worker
 * Transparently caches downloaded emoji pack assets (Fluent 3D, Apple, Google, Twemoji)
 * into CacheStorage ('gilvave-emojis-v1') so that every emoji is downloaded only once
 * and subsequent views load instantaneously (0ms) and work completely offline.
 */

const CACHE_NAME = 'gilvave-emojis-v1';
const FILES_CACHE_NAME = 'gilvave-files-v1';

self.addEventListener('install', () => {
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const keys = await caches.keys();
      await Promise.all(
        keys
          .filter(
            (k) =>
              (k.startsWith('gilvave-emojis-') && k !== CACHE_NAME) ||
              (k.startsWith('gilvave-files-') && k !== FILES_CACHE_NAME)
          )
          .map((k) => caches.delete(k))
      );
      await self.clients.claim();
    })()
  );
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  if (request.method !== 'GET') {
    return;
  }

  const url = new URL(request.url);

  // 1. Transparent file caching (/files/*) with HTTP Range support for media seeking
  if (url.pathname.startsWith('/files/')) {
    event.respondWith(
      caches.open(FILES_CACHE_NAME).then(async (cache) => {
        const rangeHeader = request.headers.get('range');
        const cached = await cache.match(request, { ignoreSearch: true });

        if (cached && rangeHeader) {
          const blob = await cached.blob();
          const total = blob.size;
          const parts = rangeHeader.replace(/bytes=/, '').split('-');
          const start = parseInt(parts[0], 10) || 0;
          const end = parts[1] ? parseInt(parts[1], 10) : total - 1;
          const chunk = blob.slice(start, end + 1);
          return new Response(chunk, {
            status: 206,
            statusText: 'Partial Content',
            headers: {
              'Content-Range': `bytes ${start}-${end}/${total}`,
              'Accept-Ranges': 'bytes',
              'Content-Length': String(chunk.size),
              'Content-Type': cached.headers.get('Content-Type') || 'video/mp4',
            },
          });
        }

        if (cached) {
          return cached;
        }

        try {
          const response = await fetch(request);
          if (response && response.ok && !rangeHeader) {
            cache.put(request, response.clone());
          }
          return response;
        } catch (_err) {
          return cached || new Response('Offline: file not found in cache', { status: 404 });
        }
      })
    );
    return;
  }

  // 2. Emoji assets caching

  const isEmojiAsset =
    (url.hostname === 'cdn.jsdelivr.net' &&
      (url.pathname.includes('/@lobehub/fluent-emoji') ||
        url.pathname.includes('/emoji-datasource') ||
        url.pathname.includes('/twemoji/'))) ||
    (url.hostname === 'fonts.gstatic.com' && url.pathname.includes('/notoemoji/'));

  if (!isEmojiAsset) {
    return;
  }

  event.respondWith(
    caches.open(CACHE_NAME).then(async (cache) => {
      const cached = await cache.match(request);
      if (cached) {
        return cached;
      }

      try {
        const response = await fetch(request);
        if (response && response.ok) {
          cache.put(request, response.clone());
        }
        return response;
      } catch (_err) {
        return cached || new Response(null, { status: 404, statusText: 'Emoji Offline Not Found' });
      }
    })
  );
});
