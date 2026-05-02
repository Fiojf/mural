(function () {
  'use strict';

  var doc = document;
  var html = doc.documentElement;
  var reduced = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  /* ----- Summon: bind trigger FIRST so nothing later can break it ----- */
  doc.addEventListener('click', function (e) {
    var t = e.target.closest && e.target.closest('#summonTrigger, [data-summon-open]');
    if (!t) return;
    e.preventDefault();
    var s = doc.getElementById('summon');
    if (!s) return;
    s.hidden = false;
    s.setAttribute('aria-hidden', 'false');
    requestAnimationFrame(function () { s.classList.add('is-open'); });
    doc.body.style.overflow = 'hidden';
  });

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

  /* ----- applyPageTheme: live recolor via CSS custom props ----- */
  var resetBtn = doc.getElementById('themeReset');
  var activeSwatch = null;
  var activeSummon = null;

  function applyPageTheme(p) {
    var r = html;
    if (!p) {
      r.style.removeProperty('--bg');
      r.style.removeProperty('--bg-rgb');
      r.style.removeProperty('--surface');
      r.style.removeProperty('--surface-2');
      r.style.removeProperty('--accent');
      r.style.removeProperty('--text');
      r.style.removeProperty('--border');
      r.style.removeProperty('--border-soft');
      r.style.removeProperty('--text-mute');
      r.style.removeProperty('--text-dim');
      if (resetBtn) resetBtn.hidden = true;
      if (activeSwatch) { activeSwatch.classList.remove('is-active'); activeSwatch = null; }
      if (activeSummon) { activeSummon.classList.remove('is-active'); activeSummon = null; }
      return;
    }
    function hexToRgb(h) {
      h = h.replace('#','');
      if (h.length === 3) h = h.split('').map(function(c){return c+c;}).join('');
      var n = parseInt(h, 16);
      return [(n>>16)&255, (n>>8)&255, n&255].join(' ');
    }
    r.style.setProperty('--bg', p.bg);
    r.style.setProperty('--bg-rgb', hexToRgb(p.bg));
    r.style.setProperty('--surface', p.surface || p.bg);
    r.style.setProperty('--surface-2', p.surface || p.bg);
    r.style.setProperty('--accent', p.accent);
    r.style.setProperty('--text', p.text);
    r.style.setProperty('--text-mute', 'color-mix(in oklab, ' + p.text + ' 65%, transparent)');
    r.style.setProperty('--text-dim', 'color-mix(in oklab, ' + p.text + ' 38%, transparent)');
    r.style.setProperty('--border', 'color-mix(in oklab, ' + p.text + ' 14%, ' + p.bg + ')');
    r.style.setProperty('--border-soft', 'color-mix(in oklab, ' + p.text + ' 6%, transparent)');
    if (resetBtn) resetBtn.hidden = false;
  }

  if (resetBtn) {
    resetBtn.addEventListener('click', function () { applyPageTheme(null); });
  }

  /* Wire existing rail swatches */
  doc.querySelectorAll('.swatch').forEach(function (sw) {
    sw.addEventListener('click', function () {
      var bg = sw.style.getPropertyValue('--bg').trim();
      var surf = sw.style.getPropertyValue('--surf').trim();
      var ac = sw.style.getPropertyValue('--ac').trim();
      var tx = sw.style.getPropertyValue('--tx').trim();
      if (!bg || !ac || !tx) return;
      if (activeSwatch === sw) {
        applyPageTheme(null);
        return;
      }
      if (activeSwatch) activeSwatch.classList.remove('is-active');
      sw.classList.add('is-active');
      activeSwatch = sw;
      if (activeSummon) { activeSummon.classList.remove('is-active'); activeSummon = null; }
      applyPageTheme({ bg: bg, surface: surf, accent: ac, text: tx });
    });
  });

  /* ----- Summon overlay (hotkey demo) ----- */
  var summon = doc.getElementById('summon');
  function openSummon() {
    if (!summon) return;
    summon.hidden = false;
    summon.setAttribute('aria-hidden', 'false');
    requestAnimationFrame(function () { summon.classList.add('is-open'); });
    doc.body.style.overflow = 'hidden';
  }
  function closeSummon() {
    if (!summon) return;
    summon.classList.remove('is-open');
    doc.body.style.overflow = '';
    setTimeout(function () {
      if (!summon.classList.contains('is-open')) {
        summon.hidden = true;
        summon.setAttribute('aria-hidden', 'true');
      }
    }, 240);
  }
  var summonTrigger = doc.getElementById('summonTrigger');
  if (summonTrigger) {
    summonTrigger.addEventListener('click', function () {
      if (summon && summon.classList.contains('is-open')) closeSummon();
      else openSummon();
    });
  }

  if (summon) {
    summon.querySelectorAll('[data-summon-close]').forEach(function (el) {
      el.addEventListener('click', closeSummon);
    });
    summon.querySelectorAll('.summon-thumb').forEach(function (t) {
      t.addEventListener('click', function () {
        if (activeSummon === t) {
          applyPageTheme(null);
          return;
        }
        if (activeSummon) activeSummon.classList.remove('is-active');
        t.classList.add('is-active');
        activeSummon = t;
        if (activeSwatch) { activeSwatch.classList.remove('is-active'); activeSwatch = null; }
        applyPageTheme({
          bg: t.dataset.bg,
          surface: t.dataset.surface,
          accent: t.dataset.accent,
          text: t.dataset.text
        });
      });
    });
  }

  function isTypingTarget(t) {
    if (!t) return false;
    var tag = t.tagName;
    return tag === 'INPUT' || tag === 'TEXTAREA' || t.isContentEditable;
  }
  window.addEventListener('keydown', function (e) {
    var k = (e.key || '').toLowerCase();
    if (k === 'escape') {
      if (summon && summon.classList.contains('is-open')) closeSummon();
      return;
    }
    if (isTypingTarget(e.target)) return;
    var meta = e.metaKey || e.ctrlKey;
    // Try ⌘⇧W (some browsers reserve this — we still attempt preventDefault).
    if (meta && e.shiftKey && k === 'w') {
      e.preventDefault();
      e.stopPropagation();
      if (summon && summon.classList.contains('is-open')) closeSummon();
      else openSummon();
      return;
    }
    // Browser-safe fallback: plain "/" opens the demo (Linear / GitHub style).
    if (!meta && !e.altKey && k === '/') {
      e.preventDefault();
      if (summon && summon.classList.contains('is-open')) closeSummon();
      else openSummon();
    }
  });

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
