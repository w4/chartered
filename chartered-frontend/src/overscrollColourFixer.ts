// A quick little utility to fix the overscroll colour at the bottom
// of the page vs the top of the page. We don't have a footer so we
// just want to carry on the body background, whereas the header is
// white so we want to use that at the top of the page.

export const backgroundFix = (body: HTMLBodyElement | null) => {
  if (body?.classList.contains("dark")) {
    document.documentElement.style.backgroundColor = window.scrollY > 70 ? "#0e1825" : "#2a3746";
  } else {
    document.documentElement.style.backgroundColor = window.scrollY > 70 ? "var(--bs-primary)" : "white";
  }
};

window.addEventListener("load", () => {
  const body = document.querySelector("body");
  let ticking = false;

  const tickBackgroundFix = () => {
    if (!ticking) {
      ticking = true;

      window.requestAnimationFrame(() => {
        backgroundFix(body);
        ticking = false;
      });
    }
  };

  window.addEventListener("scroll", tickBackgroundFix, false);
  tickBackgroundFix();
});
