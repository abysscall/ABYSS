/**
 * ABYSS — Book Page Transition Engine
 * Simulates a book page-flip between HTML pages.
 * Include this script in every page (bottom of <body>).
 */

(function () {
  /* ── INJECT STYLES ── */
  const css = `
    /* Page wrapper — every <body> gets this class on load */
    body.abyss-page {
      overflow-x: hidden;
      position: relative;
    }

    /* Overlay that covers the screen during flip */
    #abyss-flip-overlay {
      position: fixed;
      inset: 0;
      z-index: 9999;
      pointer-events: none;
      display: flex;
    }

    /* Left half (back cover) */
    #abyss-flip-left {
      width: 50%;
      height: 100%;
      background: #020408;
      transform-origin: right center;
      transform: scaleX(0);
      border-right: 1px solid rgba(0,229,255,0.15);
      position: relative;
      overflow: hidden;
    }
    #abyss-flip-left::after {
      content: '';
      position: absolute;
      inset: 0;
      background: linear-gradient(135deg,
        rgba(0,229,255,0.04) 0%,
        transparent 50%,
        rgba(123,97,255,0.04) 100%);
    }

    /* Right half (front cover) */
    #abyss-flip-right {
      width: 50%;
      height: 100%;
      background: #020408;
      transform-origin: left center;
      transform: scaleX(0);
      border-left: 1px solid rgba(0,229,255,0.15);
      position: relative;
      overflow: hidden;
    }
    #abyss-flip-right::after {
      content: '';
      position: absolute;
      inset: 0;
      background: linear-gradient(225deg,
        rgba(0,229,255,0.04) 0%,
        transparent 50%,
        rgba(123,97,255,0.04) 100%);
    }

    /* Grid lines on the covers */
    #abyss-flip-left::before, #abyss-flip-right::before {
      content: '';
      position: absolute;
      inset: 0;
      background-image:
        linear-gradient(rgba(0,229,255,0.03) 1px, transparent 1px),
        linear-gradient(90deg, rgba(0,229,255,0.03) 1px, transparent 1px);
      background-size: 60px 60px;
    }

    /* ABYSS logo in the center of the flip */
    #abyss-flip-logo {
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      z-index: 10000;
      font-family: 'Syne', sans-serif;
      font-weight: 800;
      font-size: clamp(1.5rem, 4vw, 2.5rem);
      letter-spacing: 0.3em;
      color: #00e5ff;
      text-shadow: 0 0 40px rgba(0,229,255,0.5);
      opacity: 0;
      pointer-events: none;
      white-space: nowrap;
      text-transform: uppercase;
    }

    /* Spine line in the center */
    #abyss-flip-spine {
      position: fixed;
      left: 50%;
      top: 0;
      bottom: 0;
      width: 2px;
      background: linear-gradient(to bottom,
        transparent,
        rgba(0,229,255,0.4) 20%,
        rgba(0,229,255,0.6) 50%,
        rgba(0,229,255,0.4) 80%,
        transparent);
      transform: translateX(-50%) scaleY(0);
      transform-origin: center top;
      z-index: 10000;
      pointer-events: none;
      box-shadow: 0 0 12px rgba(0,229,255,0.3);
    }

    /* Page number indicator */
    #abyss-page-indicator {
      position: fixed;
      bottom: 2rem;
      left: 50%;
      transform: translateX(-50%);
      z-index: 10001;
      display: flex;
      align-items: center;
      gap: 0.6rem;
      opacity: 0;
      pointer-events: none;
    }
    .abyss-pg-dot {
      width: 6px; height: 6px;
      border-radius: 50%;
      border: 1px solid rgba(0,229,255,0.4);
      transition: background 0.3s;
    }
    .abyss-pg-dot.current {
      background: #00e5ff;
      box-shadow: 0 0 8px rgba(0,229,255,0.6);
    }

    /* Entrance animation for incoming page */
    @keyframes abyssPageIn {
      from { opacity: 0; transform: translateY(8px); }
      to   { opacity: 1; transform: translateY(0); }
    }
    body.page-entering > *:not(#abyss-flip-overlay):not(#abyss-flip-logo):not(#abyss-flip-spine):not(#abyss-page-indicator) {
      animation: abyssPageIn 0.4s ease forwards;
    }

    /* Flip shadow on current page */
    body.page-leaving::before {
      content: '';
      position: fixed;
      inset: 0;
      background: rgba(2,4,8,0);
      z-index: 9998;
      transition: background 0.35s ease;
      pointer-events: none;
    }
    body.page-leaving-active::before {
      background: rgba(2,4,8,0.7);
    }
  `;

  const style = document.createElement('style');
  style.textContent = css;
  document.head.appendChild(style);

  /* ── BUILD OVERLAY DOM ── */
  document.body.classList.add('abyss-page');

  const overlay = document.createElement('div');
  overlay.id = 'abyss-flip-overlay';

  const left  = document.createElement('div'); left.id  = 'abyss-flip-left';
  const right = document.createElement('div'); right.id = 'abyss-flip-right';
  overlay.appendChild(left);
  overlay.appendChild(right);
  document.body.appendChild(overlay);

  const logo = document.createElement('div');
  logo.id = 'abyss-flip-logo';
  logo.textContent = 'ABYSS';
  document.body.appendChild(logo);

  const spine = document.createElement('div');
  spine.id = 'abyss-flip-spine';
  document.body.appendChild(spine);

  // Page dots
  const pages = [
    { href: 'index.html',  label: 'Main' },
    { href: 'invest.html', label: 'Invest' },
    { href: 'wallet.html', label: 'Wallet' },
  ];
  const currentPage = location.pathname.split('/').pop() || 'index.html';
  const currentIdx  = pages.findIndex(p => p.href === currentPage);

  const indicator = document.createElement('div');
  indicator.id = 'abyss-page-indicator';
  pages.forEach((p, i) => {
    const dot = document.createElement('div');
    dot.className = 'abyss-pg-dot' + (i === currentIdx ? ' current' : '');
    indicator.appendChild(dot);
  });
  document.body.appendChild(indicator);

  /* ── EASING UTIL ── */
  function ease(t) { return t < 0.5 ? 2*t*t : -1+(4-2*t)*t; }

  function animate(duration, onTick, onDone) {
    const start = performance.now();
    function frame(now) {
      const t = Math.min((now - start) / duration, 1);
      onTick(ease(t), t);
      if (t < 1) requestAnimationFrame(frame);
      else if (onDone) onDone();
    }
    requestAnimationFrame(frame);
  }

  /* ── FLIP ANIMATION ── */
  function flipTo(href, direction) {
    // direction: 'forward' | 'backward'
    const isForward = direction !== 'backward';

    // Dim current page
    document.body.classList.add('page-leaving');
    requestAnimationFrame(() => document.body.classList.add('page-leaving-active'));

    // Phase 1: close — covers slide in from outside to center
    const phase1 = 280;

    animate(phase1, (t) => {
      if (isForward) {
        // Right panel slides in from right
        right.style.transform = `scaleX(${t})`;
        left.style.transform  = `scaleX(${t * 0.3})`;
      } else {
        // Left panel slides in from left
        left.style.transform  = `scaleX(${t})`;
        right.style.transform = `scaleX(${t * 0.3})`;
      }
      // Spine grows
      spine.style.transform = `translateX(-50%) scaleY(${t})`;
      // Logo fades in
      logo.style.opacity = t > 0.5 ? ((t - 0.5) * 2).toString() : '0';
      // Indicator fades in
      indicator.style.opacity = t > 0.6 ? ((t - 0.6) * 2.5).toString() : '0';
    }, () => {
      // Phase 2: hold briefly, then navigate
      setTimeout(() => {
        window.location.href = href;
      }, 120);
    });
  }

  /* ── ENTRANCE on page load ── */
  window.addEventListener('load', () => {
    // Flash spine and logo briefly on enter
    spine.style.transition = 'none';
    spine.style.transform = 'translateX(-50%) scaleY(1)';
    logo.style.opacity = '0.8';
    indicator.style.opacity = '1';

    animate(400, (t) => {
      const inv = 1 - t;
      spine.style.transform = `translateX(-50%) scaleY(${inv})`;
      logo.style.opacity = (inv * 0.8).toString();
      indicator.style.opacity = inv.toString();
      // Panels open outward
      if (t < 0.5) {
        const p = t * 2;
        left.style.transform  = `scaleX(${1 - p})`;
        right.style.transform = `scaleX(${1 - p})`;
      } else {
        left.style.transform  = 'scaleX(0)';
        right.style.transform = 'scaleX(0)';
      }
    }, () => {
      spine.style.transform  = 'translateX(-50%) scaleY(0)';
      logo.style.opacity     = '0';
      indicator.style.opacity = '0';
      document.body.classList.add('page-entering');
      setTimeout(() => document.body.classList.remove('page-entering'), 500);
    });
  });

  /* ── INTERCEPT LINKS ── */
  const internalPages = ['index.html', 'invest.html', 'wallet.html'];

  document.addEventListener('click', function (e) {
    const a = e.target.closest('a[href]');
    if (!a) return;

    const href = a.getAttribute('href');
    if (!href || href.startsWith('#') || href.startsWith('http') || href.startsWith('mailto')) return;
    if (!internalPages.some(p => href.includes(p))) return;

    e.preventDefault();

    // Determine direction based on page order
    const targetIdx = pages.findIndex(p => href.includes(p.href));
    const direction = targetIdx >= currentIdx ? 'forward' : 'backward';

    flipTo(href, direction);
  }, true);

})();
