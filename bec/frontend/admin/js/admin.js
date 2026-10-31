/* =====================================================================
   Админка БЕРЕГ — только UX поверх htmx.
   Разметку и данные рисует сервер (askama). Здесь: тосты, подтверждения,
   превью выбранных файлов, копирование AI-текста и allow-swap ошибок формы.
   ===================================================================== */
(function () {
  'use strict';

  const $ = (selector, root) => (root || document).querySelector(selector);

  /* ---------- тосты ---------- */
  let toastTimer = null;

  function showToast(node) {
    if (!node) return;
    node.classList.add('is-visible');
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => node.classList.remove('is-visible'), 5000);
  }

  function hideToast() {
    const toast = document.getElementById('adminToast');
    if (toast) toast.classList.remove('is-visible');
  }

  document.addEventListener('click', (event) => {
    if (event.target.closest('[data-toast-close]')) hideToast();

    const copy = event.target.closest('[data-copy-ai]');
    if (copy) {
      const textarea = document.querySelector('.a-ai-result textarea');
      if (!textarea) return;
      textarea.select();
      textarea.setSelectionRange(0, textarea.value.length);
      if (navigator.clipboard) {
        navigator.clipboard.writeText(textarea.value).then(() => {
          copy.textContent = 'Скопировано';
          setTimeout(() => (copy.textContent = 'Скопировать'), 1800);
        });
      }
    }
  });

  /* ---------- превью выбранных файлов ---------- */
  document.addEventListener('change', (event) => {
    const input = event.target.closest('input[type="file"]');
    if (!input) return;
    const form = input.closest('form');
    const old = form && form.querySelector('.a-file-preview');
    if (old) old.remove();
    if (!form || !input.files || !input.files.length) return;

    const wrap = document.createElement('div');
    wrap.className = 'a-file-preview';
    wrap.style.cssText = 'display:flex;gap:6px;flex-wrap:wrap;margin-top:8px';

    Array.from(input.files).slice(0, 8).forEach((file) => {
      if (!file.type.startsWith('image/')) return;
      const img = document.createElement('img');
      img.style.cssText = 'width:54px;height:40px;object-fit:cover;border-radius:3px;border:1px solid var(--line)';
      img.alt = file.name;
      img.src = URL.createObjectURL(file);
      wrap.appendChild(img);
    });

    form.appendChild(wrap);
  });

  /* ---------- ошибки формы 4xx: htmx по умолчанию их не вставляет ---------- */
  document.addEventListener('htmx:beforeSwap', (event) => {
    const status = event.detail.xhr.status;
    if (status >= 400 && status < 500) {
      event.detail.shouldSwap = true;
      event.detail.isError = false;
    }
  });

  /* ---------- после swap ---------- */
  function afterSwap() {
    showToast(document.getElementById('adminToast'));
  }

  document.addEventListener('htmx:afterSwap', afterSwap);
  document.addEventListener('htmx:afterSettle', afterSwap);

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', afterSwap);
  } else {
    afterSwap();
  }
})();
