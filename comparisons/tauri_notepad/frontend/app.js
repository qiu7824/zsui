const invoke = window.__TAURI__.core.invoke;
const editor = document.querySelector("#editor");
const errorBar = document.querySelector("#error");
const statusBar = document.querySelector("#status");
const cursor = document.querySelector("#cursor");
const characters = document.querySelector("#characters");

const state = {
  path: null,
  displayName: "Untitled",
  dirty: false,
};

editor.value = "Tauri 2 Notepad baseline\n\nA WebView2-backed text editing benchmark.\n";

async function refreshTitle() {
  const title = `${state.dirty ? "*" : ""}${state.displayName} - Tauri Notepad baseline`;
  await invoke("set_window_title", { title });
}

function refreshStatus() {
  const caret = editor.selectionStart;
  const prefix = editor.value.slice(0, caret);
  const lines = prefix.split("\n");
  cursor.textContent = `Ln ${lines.length}, Col ${lines.at(-1).length + 1}`;
  characters.textContent = `${[...editor.value].length} chars`;
}

function showError(error) {
  errorBar.textContent = `Error: ${String(error)}`;
  errorBar.hidden = false;
}

function clearError() {
  errorBar.hidden = true;
  errorBar.textContent = "";
}

function applyDocument(document) {
  state.path = document.path;
  state.displayName = document.displayName;
  state.dirty = false;
  editor.value = document.text;
  clearError();
  refreshStatus();
  refreshTitle();
  editor.focus();
}

async function save(forcePicker) {
  try {
    const document = await invoke("save_document", {
      path: state.path,
      text: editor.value,
      forcePicker,
    });
    if (document) applyDocument(document);
  } catch (error) {
    showError(error);
  }
}

document.querySelector("#new").addEventListener("click", () => {
  state.path = null;
  state.displayName = "Untitled";
  state.dirty = false;
  editor.value = "";
  clearError();
  refreshStatus();
  refreshTitle();
  editor.focus();
});

document.querySelector("#open").addEventListener("click", async () => {
  try {
    const document = await invoke("open_document");
    if (document) applyDocument(document);
  } catch (error) {
    showError(error);
  }
});

document.querySelector("#save").addEventListener("click", () => save(false));
document.querySelector("#save-as").addEventListener("click", () => save(true));
document.querySelector("#word-wrap").addEventListener("change", (event) => {
  editor.wrap = event.target.checked ? "soft" : "off";
});
document.querySelector("#show-status").addEventListener("change", (event) => {
  statusBar.hidden = !event.target.checked;
});

editor.addEventListener("input", () => {
  state.dirty = true;
  clearError();
  refreshStatus();
  refreshTitle();
});
editor.addEventListener("click", refreshStatus);
editor.addEventListener("keyup", refreshStatus);

window.addEventListener("keydown", (event) => {
  if (!event.ctrlKey && !event.metaKey) return;
  const key = event.key.toLowerCase();
  if (key === "n") document.querySelector("#new").click();
  if (key === "o") document.querySelector("#open").click();
  if (key === "s") save(event.shiftKey);
  if (["n", "o", "s"].includes(key)) event.preventDefault();
});

refreshStatus();
refreshTitle();
