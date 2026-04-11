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

function group_colour(gid) {
  let hash = Math.floor(gid / 1.61803398875 * 360);

  return `hsl(${hash}, 100%, 50%)`;
}

function sidebarGroups(groups, GROUP_ID) {
  console.log(groups);
  return Object.entries(groups)
    .map((a) => {
      let [id, name] = a;
      console.log(id, name, GROUP_ID);
      const isActive = id === GROUP_ID;
      return (
        '<a class="sitem' +
        (isActive ? " active" : "") +
        '" href="/group?id=' +
        id +
        '"><div class="gpip" style="background: ' +
        group_colour(id) +
        '"></div>' +
        name +
        "</a>"
      );
    })
    .join("");
}
