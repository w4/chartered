// A quick little utility to fix the overscroll colour at the bottom
// of the page vs the top of the page. We don't have a footer so we
// just want to carry on the body background, whereas the header is
// white so we want to use that at the top of the page.

window.addEventListener("load", () => {
  let ticking;

  const backgroundFix = () => {
    if (!ticking) {
      ticking = true;

      window.requestAnimationFrame(() => {
        document.documentElement.style.backgroundColor =
          window.scrollY > 70 ? "var(--bs-primary)" : "#fff";
        ticking = false;
      });
    }
  };

  window.addEventListener("scroll", backgroundFix, false);
  backgroundFix();
});
