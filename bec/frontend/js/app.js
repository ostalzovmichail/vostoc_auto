/* =====================================================================
   ВОСТОК АВТО — прогрессивные улучшения поверх htmx.

   Данные и разметку приносит htmx: он запрашивает HTML-фрагменты
   (/partials/hero, /partials/catalog, ...) и вставляет их в страницу.
   Этот файл отвечает только за то, что разметкой не выразить:
   появление блоков при скролле, счётчики, «шифр» заголовка и галерею.

   Всё навешивается делегированием и заново применяется после каждой
   подстановки htmx (событие htmx:afterSwap).
   ===================================================================== */
(function () {
  'use strict';

  const $ = (id) => document.getElementById(id);
  const RM = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* ---------- появление блоков при скролле ---------- */
  const REVEAL_SELECTOR = '.rv, .sec-h, .sold-card';
  let observer = null;

  function initReveal() {
    const targets = document.querySelectorAll(REVEAL_SELECTOR);
    if (RM || !('IntersectionObserver' in window)) {
      targets.forEach((el) => el.classList.add('in'));
      return;
    }
    if (!observer) {
      observer = new IntersectionObserver((entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('in');
            observer.unobserve(entry.target);
          }
        });
      }, { rootMargin: '0px 0px -8% 0px', threshold: 0.08 });
    }
    targets.forEach((el) => {
      if (!el.classList.contains('in')) observer.observe(el);
    });
  }

  /* ---------- анимированные счётчики ---------- */
  function runCounters() {
    document.querySelectorAll('.cnt[data-to]').forEach((el) => {
      if (el.dataset.done === '1') return;
      el.dataset.done = '1';

      const to = Number(el.dataset.to) || 0;
      if (RM) {
        el.textContent = to.toLocaleString('ru-RU');
        return;
      }

      const duration = 1100;
      const started = performance.now();
      (function step(now) {
        const progress = Math.min(1, (now - started) / duration);
        const eased = 1 - Math.pow(1 - progress, 3);
        el.textContent = Math.round(to * eased).toLocaleString('ru-RU');
        if (progress < 1) requestAnimationFrame(step);
      })(started);
    });
  }

  /* ---------- «шифр» заголовка главного акцента ---------- */
  const GLYPHS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789#/·';
  function scramble() {
    const el = $('scrambleTitle');
    if (!el || el.dataset.done === '1') return;
    el.dataset.done = '1';
    if (RM) return;

    const target = el.dataset.text || el.textContent;
    const frames = 26;
    let frame = 0;
    const timer = setInterval(() => {
      frame += 1;
      const progress = frame / frames;
      el.textContent = target
        .split('')
        .map((ch, index) => {
          if (ch === ' ') return ' ';
          if (index / target.length < progress) return ch;
          return GLYPHS[Math.floor(Math.random() * GLYPHS.length)];
        })
        .join('');
      if (frame >= frames) {
        clearInterval(timer);
        el.textContent = target;
      }
    }, 42);
  }

  /* ---------- галерея главного акцента (фрагмент приходит по htmx) ---------- */
  document.addEventListener('click', (event) => {
    const thumb = event.target.closest('.hero-thumb');
    if (!thumb) return;
    const src = thumb.dataset.src;
    const img = $('heroImg');
    if (img && src) img.src = src;
    document
      .querySelectorAll('#heroThumbs .hero-thumb')
      .forEach((btn) => btn.classList.toggle('on', btn === thumb));
  });

  /* ---------- прокрутка архива сделок ---------- */
  document.addEventListener('click', (event) => {
    const button = event.target.closest('#soldPrev, #soldNext');
    if (!button) return;
    const row = $('soldRow');
    if (!row) return;
    const step = Math.max(320, row.clientWidth * 0.8);
    row.scrollBy({
      left: button.id === 'soldNext' ? step : -step,
      behavior: RM ? 'auto' : 'smooth',
    });
  });

  /* ---------- шапка при скролле ---------- */
  function initHeader() {
    const header = $('hdr');
    if (!header) return;
    const onScroll = () => header.classList.toggle('scrolled', window.scrollY > 8);
    window.addEventListener('scroll', onScroll, { passive: true });
    onScroll();
  }

  /* ---------- форма обратного звонка ---------- */
  document.addEventListener('submit', (event) => {
    const form = event.target.closest('#cbForm');
    if (!form) return;
    event.preventDefault();
    form.style.display = 'none';
    const ok = $('formOk');
    if (ok) ok.style.display = 'block';
  });

  /* ---------- перезапуск после подстановок htmx ---------- */
  function refresh() {
    initReveal();
    runCounters();
    scramble();
  }

  document.addEventListener('htmx:afterSwap', refresh);
  document.addEventListener('htmx:afterSettle', refresh);

  initHeader();
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', refresh);
  } else {
    refresh();
  }
})();
