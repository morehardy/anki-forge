type Cue = Omit<Keyframe, 'offset'> & { at: number };

/** One shared timeline: preview each card, keep it in the stack, then pack the deck. */
export function animateShowcase(studio: HTMLElement) {
  const cards = [...studio.querySelectorAll<HTMLElement>('.forge-card')];
  const secondsPerCard = 4.5;
  const packAt = cards.length * secondsPerCard + .4;
  const duration = packAt + 6;
  const packSpeed = 2;
  // Compress the packing cues together so every folding layer stays in sync.
  const playbackTime = (at: number) => at <= packAt ? at : packAt + (at - packAt) / packSpeed;
  const playbackDuration = playbackTime(duration);
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
  const ease = 'cubic-bezier(.22, .72, .24, 1)';
  const travel = 'cubic-bezier(.45, 0, .25, 1)';
  const preview = 'translate(-50%, -50%)';
  const stacked = `${preview} translate(var(--delivery-x), var(--delivery-y)) scale(.62)`;
  const bundled = `${preview} translate(var(--bundle-x), var(--bundle-y)) scale(.58)`;
  const packed = `${preview} translate(var(--bundle-x), calc(var(--bundle-y) + 5cqw)) scale(.5)`;
  let animations: Animation[] = [];
  let inView = true;

  const animate = (target: Element | string, cues: Cue[]) => {
    const element = typeof target === 'string' ? studio.querySelector(target) : target;
    if (!element) return;
    animations.push(element.animate(cues.map(({ at, ...frame }) => ({
      ...frame,
      offset: playbackTime(at) / playbackDuration,
    })), { duration: playbackDuration * 1000, iterations: Infinity, fill: 'both' }));
  };

  const updatePlayback = () => {
    const paused = document.hidden || !inView;
    const time = animations[0]?.currentTime ?? 0;
    const now = document.timeline.currentTime;
    for (const animation of animations) {
      if (paused) {
        animation.pause();
        animation.currentTime = time;
      } else {
        animation.play();
        if (typeof now === 'number' && typeof time === 'number') animation.startTime = now - time;
      }
    }
  };

  const start = () => {
    animations.forEach(animation => animation.cancel());
    animations = [];
    if (reducedMotion.matches) return;

    cards.forEach((card, index) => {
      const startAt = index * secondsPerCard;
      const enter = `${preview} translateX(3cqw) scale(.97)`;
      animate(card, [
        { at: 0, opacity: 0, transform: enter },
        { at: startAt, opacity: 0, transform: enter, easing: ease },
        { at: startAt + .35, opacity: 1, transform: preview },
        { at: startAt + 3.2, opacity: 1, transform: preview, easing: travel },
        { at: startAt + 4.35, opacity: 1, transform: stacked },
        { at: packAt + .2, opacity: 1, transform: stacked, easing: ease },
        { at: packAt + 1.25, opacity: 1, transform: bundled },
        { at: packAt + 1.55, opacity: 1, transform: bundled, easing: travel },
        { at: packAt + 2.3, opacity: 1, transform: packed },
        { at: packAt + 2.35, opacity: 0, transform: packed },
        { at: duration, opacity: 0, transform: packed },
      ]);
      animate(card.querySelector('.forge-card-flipper')!, [
        { at: 0, transform: 'rotateY(0)' },
        { at: startAt + 1.35, transform: 'rotateY(0)', easing: ease },
        { at: startAt + 2, transform: 'rotateY(-180deg)' },
        { at: duration, transform: 'rotateY(-180deg)' },
      ]);
    });

    animate('.forge-stack-label', [
      { at: 0, opacity: 0 },
      { at: 3.7, opacity: 0 },
      { at: 4.35, opacity: 1 },
      { at: packAt + .2, opacity: 1 },
      { at: packAt + .7, opacity: 0 },
      { at: duration, opacity: 0 },
    ]);
    animate('.forge-box', [
      { at: 0, opacity: 0, transform: 'translateY(0)' },
      { at: packAt + 1, opacity: 0, transform: 'translateY(0)' },
      { at: packAt + 1.15, opacity: 1, transform: 'translateY(0)' },
      { at: packAt + 3.1, opacity: 1, transform: 'translateY(0)', easing: ease },
      { at: packAt + 3.3, opacity: 1, transform: 'translateY(.6cqw)', easing: ease },
      { at: packAt + 3.55, opacity: 1, transform: 'translateY(0)' },
      { at: duration - .8, opacity: 1, transform: 'translateY(0)', easing: ease },
      { at: duration - .35, opacity: 0, transform: 'translateY(-1cqw)' },
      { at: duration, opacity: 0, transform: 'translateY(-1cqw)' },
    ]);
    animate('.forge-box-side', [
      { at: 0, transform: 'skewY(-45deg) rotateY(90deg)' },
      { at: packAt + 1.1, transform: 'skewY(-45deg) rotateY(90deg)', easing: ease },
      { at: packAt + 1.85, transform: 'skewY(-45deg) rotateY(0)' },
      { at: duration, transform: 'skewY(-45deg) rotateY(0)' },
    ]);
    animate('.forge-box-front', [
      { at: 0, transform: 'perspective(800px) rotateX(-90deg)' },
      { at: packAt + 1.35, transform: 'perspective(800px) rotateX(-90deg)', easing: ease },
      { at: packAt + 2.2, transform: 'perspective(800px) rotateX(0)' },
      { at: duration, transform: 'perspective(800px) rotateX(0)' },
    ]);
    animate('.forge-box-lid', [
      { at: 0, opacity: 0, transform: 'skewX(-45deg) rotateX(-115deg)' },
      { at: packAt + 1.8, opacity: 0, transform: 'skewX(-45deg) rotateX(-115deg)' },
      { at: packAt + 2.2, opacity: 1, transform: 'skewX(-45deg) rotateX(-115deg)', easing: ease },
      { at: packAt + 2.95, opacity: 1, transform: 'skewX(-45deg) rotateX(0)' },
      { at: duration, opacity: 1, transform: 'skewX(-45deg) rotateX(0)' },
    ]);
    studio.querySelectorAll('.forge-box-seal').forEach(seal => animate(seal, [
      { at: 0, opacity: 0, transform: 'scaleY(0)' },
      { at: packAt + 2.9, opacity: 0, transform: 'scaleY(0)', easing: ease },
      { at: packAt + 3.25, opacity: 1, transform: 'scaleY(1)' },
      { at: duration, opacity: 1, transform: 'scaleY(1)' },
    ]));
    animate('.forge-box-label', [
      { at: 0, opacity: 0, transform: 'translateY(1cqw)' },
      { at: packAt + 2.8, opacity: 0, transform: 'translateY(1cqw)', easing: ease },
      { at: packAt + 3.35, opacity: 1, transform: 'translateY(0)' },
      { at: duration, opacity: 1, transform: 'translateY(0)' },
    ]);
    animate('.forge-box-shadow', [
      { at: 0, opacity: 0, transform: 'scale(.7)' },
      { at: packAt + 1, opacity: 0, transform: 'scale(.7)', easing: ease },
      { at: packAt + 2.2, opacity: .6, transform: 'scale(1)' },
      { at: duration - .8, opacity: .6, transform: 'scale(1)' },
      { at: duration - .35, opacity: 0, transform: 'scale(.9)' },
      { at: duration, opacity: 0, transform: 'scale(.9)' },
    ]);

    // All tracks, including delayed cards, start and resume on the same clock.
    animations.forEach(animation => { animation.currentTime = 0; });
    updatePlayback();
  };

  const observer = new IntersectionObserver(([entry]) => {
    inView = entry.isIntersecting;
    updatePlayback();
  }, { threshold: .15 });
  observer.observe(studio);
  document.addEventListener('visibilitychange', updatePlayback);
  reducedMotion.addEventListener('change', start);
  start();
  document.addEventListener('astro:before-swap', () => {
    observer.disconnect();
    document.removeEventListener('visibilitychange', updatePlayback);
    reducedMotion.removeEventListener('change', start);
    animations.forEach(animation => animation.cancel());
  }, { once: true });
}
