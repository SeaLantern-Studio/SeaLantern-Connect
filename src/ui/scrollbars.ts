const HIDE_DELAY_MS = 700;

export function enableAutoHidingScrollbars(): () => void {
  const hideTimers = new Map<Element, number>();

  function handleScroll(event: Event) {
    const element = event.target instanceof Element ? event.target : document.scrollingElement;
    if (!element) return;

    element.classList.add("scrollbar-active");
    const previousTimer = hideTimers.get(element);
    if (previousTimer != null) window.clearTimeout(previousTimer);
    hideTimers.set(
      element,
      window.setTimeout(() => {
        element.classList.remove("scrollbar-active");
        hideTimers.delete(element);
      }, HIDE_DELAY_MS),
    );
  }

  document.addEventListener("scroll", handleScroll, true);
  return () => {
    document.removeEventListener("scroll", handleScroll, true);
    for (const [element, timer] of hideTimers) {
      window.clearTimeout(timer);
      element.classList.remove("scrollbar-active");
    }
    hideTimers.clear();
  };
}
