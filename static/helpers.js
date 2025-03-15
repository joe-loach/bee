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

function try_load_svg(event, index) {
  if (storageAvailable("localStorage")) {
    const qrCode = localStorage.getItem(qrCodeStorage(index));
    if (qrCode === null) {
      return;
    }
    if (!event.detail.target.hasChildNodes()) {
      // set the HTML directly instead of fetching
      event.detail.target.innerHTML = qrCode;
    }
    event.preventDefault();
  }
}

function show_for_time(qr_id, timer_id, inc_id) {
  const timer = document.getElementById(timer_id);
  // start the timer animation
  timer.style.animationPlayState = "running";

  const TIME_TO_RESET = 10 * 1000; /* 10 seconds */
  setTimeout(() => {
    function reset_animation() {
      timer.style.animation = "none";
      timer.offsetHeight; /* trigger reflow */
      timer.style.animation = "";
    }
    reset_animation();

    // remove qr code svg
    const qr = document.getElementById(qr_id);
    qr.firstChild.remove();
    // increment counter
    htmx.trigger(`#${inc_id}`, "increment", {});
  }, TIME_TO_RESET);
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
