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

  /* ----- Reveal-on-scroll ----- */
  var revealEls = doc.querySelectorAll('.reveal');
  if (reduced || !('IntersectionObserver' in window)) {
    revealEls.forEach(function (el) { el.classList.add('is-in'); });
  } else {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        var sibs = entry.target.parentElement ? entry.target.parentElement.querySelectorAll(':scope > .reveal') : [entry.target];
        var idx = Array.prototype.indexOf.call(sibs, entry.target);
        if (idx < 0) idx = 0;
        entry.target.style.transitionDelay = (idx * 60) + 'ms';
        entry.target.classList.add('is-in');
        io.unobserve(entry.target);
      });
    }, { threshold: 0.15, rootMargin: '0px 0px -40px 0px' });
    revealEls.forEach(function (el) { io.observe(el); });
  }

  /* ----- How-it-works dotted line ----- */
  var howLine = doc.querySelector('.how-line');
  if (howLine && 'IntersectionObserver' in window) {
    var io2 = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (entry.isIntersecting) { howLine.classList.add('is-on'); io2.unobserve(entry.target); }
      });
    }, { threshold: 0.4 });
    io2.observe(howLine);
  }

  /* ----- Hero parallax + mockup tilt ----- */
  if (!reduced) {
    var blobs = doc.querySelectorAll('.blob');
    var hero = doc.querySelector('.hero');
    var mockup = doc.getElementById('mockup');
    var targetX = 0, targetY = 0, curX = 0, curY = 0, mx = 0, my = 0;
    var rect = null;
    function recalc() { rect = hero ? hero.getBoundingClientRect() : null; }
    recalc();
    window.addEventListener('resize', recalc);
    if (hero) {
      hero.addEventListener('mousemove', function (e) {
        if (!rect) recalc();
        var cx = rect.left + rect.width / 2;
        var cy = rect.top + rect.height / 2;
        targetX = (e.clientX - cx);
        targetY = (e.clientY - cy);
        if (mockup) {
          var mr = mockup.getBoundingClientRect();
          var inside = e.clientX > mr.left - 40 && e.clientX < mr.right + 40 && e.clientY > mr.top - 40 && e.clientY < mr.bottom + 40;
          if (inside) {
            mockup.classList.add('is-tilt');
            var px = (e.clientX - (mr.left + mr.width / 2)) / (mr.width / 2);
            var py = (e.clientY - (mr.top + mr.height / 2)) / (mr.height / 2);
            mockup.style.setProperty('--tx', (px * 4).toFixed(2) + 'deg');
            mockup.style.setProperty('--ty', (-py * 4).toFixed(2) + 'deg');
          } else {
            mockup.classList.remove('is-tilt');
            mockup.style.removeProperty('--tx');
            mockup.style.removeProperty('--ty');
          }
        }
      });
      hero.addEventListener('mouseleave', function () {
        targetX = 0; targetY = 0;
        if (mockup) { mockup.classList.remove('is-tilt'); mockup.style.removeProperty('--tx'); mockup.style.removeProperty('--ty'); }
      });
    }
    function tick() {
      curX += (targetX - curX) * 0.06;
      curY += (targetY - curY) * 0.06;
      blobs.forEach(function (b) {
        var k = parseFloat(b.dataset.parallax || '0.02');
        var max = 24;
        var dx = Math.max(-max, Math.min(max, curX * k));
        var dy = Math.max(-max, Math.min(max, curY * k));
        b.style.transform = 'translate3d(' + dx.toFixed(2) + 'px,' + dy.toFixed(2) + 'px,0)';
      });
      requestAnimationFrame(tick);
    }
    requestAnimationFrame(tick);
  }

  /* ----- Copy on click (theme path button + code pane) ----- */
  function flashAck(el, label) {
    if (!el) return;
    var prev = el.textContent;
    el.textContent = label;
    el.classList.add('is-copied');
    setTimeout(function () { el.textContent = prev; el.classList.remove('is-copied'); }, 1400);
  }
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

  var themePath = doc.querySelector('.theme-path');
  if (themePath) {
    themePath.addEventListener('click', function () {
      copyText(themePath.dataset.copy || themePath.querySelector('code').textContent);
      var hint = themePath.querySelector('.copy-hint');
      flashAck(hint, 'Copied');
    });
  }

  var codePane = doc.getElementById('codePane');
  var codeAck = doc.getElementById('codeAck');
  if (codePane) {
    var pre = codePane.querySelector('pre.code');
    var doCopy = function () {
      copyText(pre.textContent);
      if (codeAck) {
        codeAck.hidden = false;
        clearTimeout(codePane._t);
        codePane._t = setTimeout(function () { codeAck.hidden = true; }, 1400);
      }
    };
    pre.addEventListener('click', doCopy);
    var tbCopy = codePane.querySelector('.tb-copy');
    if (tbCopy) tbCopy.addEventListener('click', function (e) { e.stopPropagation(); doCopy(); });
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
      if (e.key === 'ArrowRight') { rail.scrollBy({ left: 240, behavior: 'smooth' }); e.preventDefault(); }
      if (e.key === 'ArrowLeft')  { rail.scrollBy({ left: -240, behavior: 'smooth' }); e.preventDefault(); }
    });
  }
})();
