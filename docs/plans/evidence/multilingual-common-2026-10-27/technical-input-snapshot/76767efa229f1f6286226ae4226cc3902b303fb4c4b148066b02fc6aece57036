import { translatorFor } from '../i18n/format-message.mjs';
const clientMessages = document.querySelector<HTMLScriptElement>('script[data-anki-forge-messages]');
const t = translatorFor(JSON.parse(clientMessages?.textContent || '{}'), document.documentElement.lang);

const root = document.documentElement;
const darkPreference = window.matchMedia('(prefers-color-scheme: dark)');
const themeControls = [...document.querySelectorAll<HTMLElement>('[data-theme-control]')];
const themeOptions = [...document.querySelectorAll<HTMLInputElement>('[data-theme-option]')];
let theme = 'auto';
try { theme = localStorage.getItem('starlight-theme') || 'auto'; } catch { /* Storage is optional. */ }
if (!['light', 'dark', 'auto'].includes(theme)) theme = 'auto';
function applyTheme() {
  root.dataset.theme = theme === 'dark' || (theme === 'auto' && darkPreference.matches) ? 'dark' : 'light';
  themeControls.forEach(control => { control.dataset.themeChoice = theme; });
  themeOptions.forEach(option => { option.checked = option.value === theme; option.disabled = false; });
}
applyTheme();
darkPreference.addEventListener('change', applyTheme);
themeOptions.forEach(option => option.addEventListener('change', () => {
  theme = option.value;
  try { localStorage.setItem('starlight-theme', theme === 'auto' ? '' : theme); } catch { /* Respect storage restrictions. */ }
  applyTheme();
}));

document.querySelectorAll<HTMLElement>('[data-workbench]').forEach(workbench => {
  const tabs = [...workbench.querySelectorAll<HTMLButtonElement>('[data-tab]')];
  const panels = [...workbench.querySelectorAll<HTMLElement>('[data-example]')];
  function selectTab(button: HTMLButtonElement, focus = false) {
    tabs.forEach(tab => {
      const selected = tab === button;
      tab.setAttribute('aria-selected', String(selected));
      tab.tabIndex = selected ? 0 : -1;
    });
    panels.forEach(panel => {
      panel.hidden = panel.dataset.example !== button.dataset.tab;
      panel.querySelectorAll('audio').forEach(audio => audio.pause());
    });
    if (focus) button.focus();
  }
  tabs.forEach((button, index) => {
    button.addEventListener('click', () => selectTab(button));
    button.addEventListener('keydown', event => {
      let next: number | undefined;
      if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
      if (event.key === 'ArrowLeft') next = (index + tabs.length - 1) % tabs.length;
      if (event.key === 'Home') next = 0;
      if (event.key === 'End') next = tabs.length - 1;
      if (next !== undefined) {
        event.preventDefault();
        const target = tabs[next];
        if (target) selectTab(target, true);
      }
    });
  });
});

document.querySelectorAll<HTMLElement>('[data-card-preview]').forEach(preview => {
  const button = preview.querySelector<HTMLButtonElement>('[data-flip]');
  const card = preview.querySelector<HTMLElement>('[data-flashcard]');
  const front = preview.querySelector<HTMLElement>('[data-side="front"]');
  const back = preview.querySelector<HTMLElement>('[data-side="back"]');
  const status = preview.querySelector<HTMLElement>('[data-card-status]');
  if (!button || !card || !front || !back || !status) return;
  button.addEventListener('click', () => {
    const flipped = button.getAttribute('aria-pressed') !== 'true';
    button.setAttribute('aria-pressed', String(flipped));
    card.classList.toggle('is-flipped', flipped);
    front.setAttribute('aria-hidden', String(flipped));
    back.setAttribute('aria-hidden', String(!flipped));
    const label = button.querySelector('span:last-child');
    if (label) label.textContent = flipped ? t('card.showQuestion') : t('card.showAnswer');
    status.textContent = flipped ? t('card.answer', { answer: back.textContent?.trim() }) : t('card.questionShown');
  });
});

document.querySelectorAll<HTMLElement>('[data-update-demo]').forEach(demo => {
  const button = demo.querySelector<HTMLButtonElement>('[data-update]');
  const value = demo.querySelector<HTMLElement>('[data-update-value]');
  const status = demo.querySelector<HTMLElement>('[data-update-status]');
  if (!button || !value || !status) return;
  let updated = false;
  button.addEventListener('click', () => {
    updated = !updated;
    value.textContent = (updated ? demo.dataset.after : demo.dataset.before) || '';
    value.classList.toggle('is-updated', updated);
    const label = button.querySelector('span:first-child');
    if (label) label.textContent = updated ? t('update.reset') : t('update.change');
    status.textContent = updated ? t('update.changed') : t('update.try');
  });
});

const copyListeners = new AbortController();
const copyTimers = new Set<ReturnType<typeof setTimeout>>();
document.querySelectorAll<HTMLElement>('[data-ai-quickstart]').forEach(builder => {
  const templates: Record<string, { before: string; after: string; empty: string }> = JSON.parse(builder.querySelector<HTMLScriptElement>('script[data-ai-templates]')?.textContent || '{}');
  const tabs = [...builder.querySelectorAll<HTMLButtonElement>('[data-ai-language]')];
  const panels = [...builder.querySelectorAll<HTMLElement>('[data-ai-panel]')];
  const request = builder.querySelector<HTMLTextAreaElement>('[data-ai-request]');
  if (!tabs.length || !request) return;
  function selectLanguage(button: HTMLButtonElement, focus = false) {
    tabs.forEach(tab => {
      const selected = tab === button;
      tab.setAttribute('aria-selected', String(selected));
      tab.tabIndex = selected ? 0 : -1;
    });
    panels.forEach(panel => { panel.hidden = panel.dataset.aiPanel !== button.dataset.aiLanguage; });
    if (focus) button.focus();
  }
  tabs.forEach((button, index) => {
    button.addEventListener('click', () => selectLanguage(button), { signal: copyListeners.signal });
    button.addEventListener('keydown', event => {
      let next: number | undefined;
      if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
      if (event.key === 'ArrowLeft') next = (index + tabs.length - 1) % tabs.length;
      if (event.key === 'Home') next = 0;
      if (event.key === 'End') next = tabs.length - 1;
      if (next !== undefined) {
        event.preventDefault();
        selectLanguage(tabs[next]!, true);
      }
    }, { signal: copyListeners.signal });
  });
  panels.forEach(panel => {
    panel.setAttribute('role', 'tabpanel');
    panel.setAttribute('aria-labelledby', `ai-tab-${panel.dataset.aiPanel}`);
  });
  request.addEventListener('input', () => {
    panels.forEach(panel => {
      const prompt = panel.querySelector<HTMLElement>('[data-ai-prompt]');
      const template = templates[panel.dataset.aiPanel!];
      if (prompt && template) prompt.textContent = request.value.trim() ? `${template.before}${request.value}${template.after}` : template.empty;
      panel.querySelectorAll<HTMLElement>('[data-copy-status]').forEach(status => { status.textContent = ''; });
    });
  }, { signal: copyListeners.signal });
  selectLanguage(tabs[0]!);
  builder.classList.add('is-interactive');
  builder.querySelectorAll<HTMLElement>('[data-ai-enhancement]').forEach(element => { element.hidden = false; });
});

// Keep native Starlight rendering, but give every copy action the same visible
// success/failure contract. Capture prevents the native legacy-copy handler from
// clearing the manual selection when clipboard permission has been denied.
document.querySelectorAll<HTMLElement>('.expressive-code').forEach(block => {
  const button = block.querySelector<HTMLButtonElement>('.copy button');
  const content = block.querySelector('pre');
  if (!button || !content) return;
  block.dataset.copyBlock = '';
  block.dataset.copySuccess = t('copy.codeSuccess');
  block.dataset.copyFallback = t('copy.codeFallback');
  button.dataset.copy = '';
  content.dataset.copyContent = '';
  const status = document.createElement('p');
  status.dataset.copyStatus = '';
  status.setAttribute('role', 'status');
  status.setAttribute('aria-live', 'polite');
  block.append(status);
});

document.querySelectorAll<HTMLElement>('[data-copy-block]').forEach(block => {
  const button = block.querySelector<HTMLButtonElement>('[data-copy]');
  const content = block.querySelector<HTMLElement>('[data-copy-content]');
  const status = block.querySelector<HTMLElement>('[data-copy-status]');
  if (!button || !content || !status) return;
  button.disabled = false;
  const label = button.querySelector<HTMLElement>('[data-copy-label]');
  const originalLabel = label?.textContent || '';
  const copyIcon = button.querySelector<HTMLElement>('[data-copy-icon]');
  const checkIcon = button.querySelector<HTMLElement>('[data-copy-check]');
  let timer: ReturnType<typeof setTimeout> | undefined;
  function reset() {
    if (label) label.textContent = originalLabel;
    if (copyIcon) copyIcon.hidden = false;
    if (checkIcon) checkIcon.hidden = true;
  }
  button.addEventListener('click', async event => {
    event.stopImmediatePropagation();
    if (timer) { clearTimeout(timer); copyTimers.delete(timer); }
    reset();
    status.textContent = '';
    button.disabled = true;
    try {
      // Expressive Code serializes newlines as DEL in its public data-code attribute.
      const text = button.dataset.code?.replace(/\u007f/g, '\n') ?? content.textContent ?? '';
      await navigator.clipboard.writeText(text);
      if (copyListeners.signal.aborted) return;
      status.textContent = block.dataset.copySuccess || t('copy.success');
      if (label) label.textContent = t('copy.copied');
      if (copyIcon) copyIcon.hidden = true;
      if (checkIcon) checkIcon.hidden = false;
      timer = setTimeout(() => { reset(); if (timer) copyTimers.delete(timer); }, 2000);
      copyTimers.add(timer);
    } catch {
      if (copyListeners.signal.aborted) return;
      const selection = window.getSelection();
      const range = document.createRange();
      range.selectNodeContents(content);
      selection?.removeAllRanges();
      selection?.addRange(range);
      status.textContent = block.dataset.copyFallback || t('copy.fallback');
    } finally {
      button.disabled = false;
    }
  }, { signal: copyListeners.signal, capture: true });
});

window.addEventListener('pagehide', event => {
  // A page restored from the back/forward cache still needs its handlers.
  if (event.persisted) return;
  copyListeners.abort();
  copyTimers.forEach(timer => clearTimeout(timer));
  copyTimers.clear();
});

if (!matchMedia('(prefers-reduced-motion: reduce)').matches && 'IntersectionObserver' in window) {
  const observer = new IntersectionObserver(entries => {
    entries.forEach(entry => {
      if (entry.isIntersecting) { entry.target.classList.add('is-revealed'); observer.unobserve(entry.target); }
    });
  }, { threshold: 0.1 });
  document.querySelectorAll('[data-reveal]').forEach(element => observer.observe(element));
  window.addEventListener('pagehide', () => observer.disconnect(), { once: true });
}
