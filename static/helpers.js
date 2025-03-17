const cookie = (key) =>
  (new RegExp((key || "=") + "=(.*?); ", "gm").exec(document.cookie + "; ") || [
    "",
    null,
  ])[1];
const session = () => cookie("session");

const qrCodeStorage = (index) => `qrCodeSVG_${index}`;

function save_svg(event, index) {
  const qr = event.target;
  const svg = qr.firstChild;

  if (storageAvailable("localStorage")) {
    localStorage.setItem(qrCodeStorage(index), svg.outerHTML);
  }
}

function increment(id) {
  fetch(`tickets/${id}/inc`, { method: "POST" });
}

function clearLocalStorage() {
  if (storageAvailable("localStorage")) {
    localStorage.clear();
  }
}

function storageAvailable(type) {
  try {
    var storage = window[type],
      x = "__storage_test__";
    storage.setItem(x, x);
    storage.removeItem(x);
    return true;
  } catch (e) {
    return false;
  }
}
