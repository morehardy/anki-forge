const root = document.documentElement;
const darkPreference = window.matchMedia('(prefers-color-scheme: dark)');
const themePicker = document.querySelector<HTMLSelectElement>('[data-theme-picker]');
let theme = 'auto';
try { theme = localStorage.getItem('starlight-theme') || 'auto'; } catch { /* Storage is optional. */ }
if (!['light', 'dark', 'auto'].includes(theme)) theme = 'auto';
function applyTheme() {
  root.dataset.theme = theme === 'dark' || (theme === 'auto' && darkPreference.matches) ? 'dark' : 'light';
  if (themePicker) themePicker.value = theme;
}
applyTheme();
darkPreference.addEventListener('change', applyTheme);
themePicker?.addEventListener('change', () => {
  theme = themePicker.value;
  try { localStorage.setItem('starlight-theme', theme); } catch { /* Respect storage restrictions. */ }
  applyTheme();
});

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
    if (label) label.textContent = flipped ? 'Show question' : 'Show answer';
    status.textContent = flipped ? `Answer: ${back.textContent?.trim()}` : 'Question shown.';
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
    if (label) label.textContent = updated ? 'Reset example' : 'Update the answer';
    status.textContent = updated ? 'Content changed. Same note ID.' : 'Try a content change.';
  });
});

document.querySelectorAll<HTMLElement>('[data-copy-block]').forEach(block => {
  const button = block.querySelector<HTMLButtonElement>('[data-copy]');
  const content = block.querySelector<HTMLElement>('[data-copy-content]');
  const status = block.querySelector<HTMLElement>('[data-copy-status]');
  if (!button || !content || !status) return;
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(content.textContent || '');
      status.textContent = 'Command copied.';
    } catch {
      const selection = window.getSelection();
      const range = document.createRange();
      range.selectNodeContents(content);
      selection?.removeAllRanges();
      selection?.addRange(range);
      status.textContent = 'Command selected. Copy it with your keyboard.';
    }
  });
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
