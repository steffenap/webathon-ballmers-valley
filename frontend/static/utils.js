function toggleSidebar() {
  document.getElementById("sidebar").classList.toggle("open");
  document.getElementById("sidebar-overlay").classList.toggle("open");
}

function showToast(msg) {
  const t = document.getElementById("toast");
  t.textContent = msg;
  t.style.transform = "translateX(-50%) translateY(0)";
  clearTimeout(t._t);
  t._t = setTimeout(() => {
    t.style.transform = "translateX(-50%) translateY(200px)";
  }, 2200);
}
