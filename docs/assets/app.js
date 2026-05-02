(function () {
  'use strict';

  var doc = document;
  var html = doc.documentElement;
  var reduced = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* ----- Theme toggle ----- */
  var themeBtn = doc.getElementById('themeToggle');
  function applyTheme(t) {
    html.setAttribute('data-theme', t);
    if (themeBtn) themeBtn.setAttribute('aria-pressed', t === 'light' ? 'true' : 'false');
    try { localStorage.setItem('mural-theme', t); } catch (e) {}
  }
  if (themeBtn) {
    themeBtn.setAttribute('aria-pressed', html.getAttribute('data-theme') === 'light' ? 'true' : 'false');
    themeBtn.addEventListener('click', function () {
      applyTheme(html.getAttribute('data-theme') === 'light' ? 'dark' : 'light');
    });
  }

  /* ----- Sticky nav reveal ----- */
  var nav = doc.getElementById('nav');
  function onScroll() {
    if (!nav) return;
    if (window.scrollY > 80) nav.classList.add('is-visible');
    else nav.classList.remove('is-visible');
  }
  window.addEventListener('scroll', onScroll, { passive: true });
  onScroll();

  /* ----- Hamburger drawer ----- */
  var burger = doc.getElementById('hamburger');
  var drawer = doc.getElementById('navDrawer');
  if (burger && drawer) {
    burger.addEventListener('click', function () {
      var open = drawer.classList.toggle('is-open');
      drawer.hidden = !open;
      burger.setAttribute('aria-expanded', open ? 'true' : 'false');
    });
    drawer.addEventListener('click', function (e) {
      if (e.target.tagName === 'A') {
        drawer.classList.remove('is-open');
        drawer.hidden = true;
        burger.setAttribute('aria-expanded', 'false');
      }
    });
  }

  /* ----- Reveal-on-scroll (sparingly applied) ----- */
  var revealEls = doc.querySelectorAll('.reveal');
  if (reduced || !('IntersectionObserver' in window)) {
    revealEls.forEach(function (el) { el.classList.add('is-in'); });
  } else {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        entry.target.classList.add('is-in');
        io.unobserve(entry.target);
      });
    }, { threshold: 0.15, rootMargin: '0px 0px -40px 0px' });
    revealEls.forEach(function (el) { io.observe(el); });
  }

  /* ----- Copy on click ----- */
  function copyText(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      return navigator.clipboard.writeText(text);
    }
    var ta = doc.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    doc.body.appendChild(ta);
    ta.select();
    try { doc.execCommand('copy'); } catch (e) {}
    doc.body.removeChild(ta);
    return Promise.resolve();
  }
  function flashAck(el, label) {
    if (!el) return;
    var prev = el.textContent;
    el.textContent = label;
    el.classList.add('is-copied');
    setTimeout(function () { el.textContent = prev; el.classList.remove('is-copied'); }, 1400);
  }

  var themePath = doc.querySelector('.theme-path');
  if (themePath) {
    themePath.addEventListener('click', function () {
      copyText(themePath.dataset.copy || themePath.querySelector('code').textContent);
      flashAck(themePath.querySelector('.copy-hint'), 'Copied');
      themePath.classList.add('is-copied');
      setTimeout(function () { themePath.classList.remove('is-copied'); }, 1400);
    });
  }

  var codePane = doc.getElementById('codePane');
  var codeAck = doc.getElementById('codeAck');
  if (codePane) {
    var pre = codePane.querySelector('pre.code');
    pre.addEventListener('click', function () {
      copyText(pre.textContent);
      if (codeAck) {
        codeAck.hidden = false;
        clearTimeout(codePane._t);
        codePane._t = setTimeout(function () { codeAck.hidden = true; }, 1400);
      }
    });
  }

  /* ----- Theme rail drag scroll ----- */
  var rail = doc.getElementById('themeRail');
  if (rail) {
    var down = false, sx = 0, sl = 0;
    rail.addEventListener('mousedown', function (e) {
      down = true; sx = e.pageX - rail.offsetLeft; sl = rail.scrollLeft;
      rail.style.cursor = 'grabbing';
    });
    var stop = function () { down = false; rail.style.cursor = ''; };
    rail.addEventListener('mouseleave', stop);
    rail.addEventListener('mouseup', stop);
    rail.addEventListener('mousemove', function (e) {
      if (!down) return;
      e.preventDefault();
      rail.scrollLeft = sl - (e.pageX - rail.offsetLeft - sx);
    });
    rail.addEventListener('keydown', function (e) {
      if (e.key === 'ArrowRight') { rail.scrollBy({ left: 220, behavior: 'smooth' }); e.preventDefault(); }
      if (e.key === 'ArrowLeft')  { rail.scrollBy({ left: -220, behavior: 'smooth' }); e.preventDefault(); }
    });
  }
})();
