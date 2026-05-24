import "./style.css";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import { invoke } from "@tauri-apps/api/core";

const statusEl = document.getElementById("status")!;
const previewStatus = document.getElementById("preview-status")!;
const svgContainer = document.getElementById("svg-container")!;
const outputPane = document.getElementById("output-pane")!;
const previewPane = document.getElementById("preview-pane")!;
const divider1 = document.getElementById("divider-1")!;
const divider2 = document.getElementById("divider-2")!;
const toggleTypstBtn = document.getElementById("toggle-typst")!;
const togglePreviewBtn = document.getElementById("toggle-preview")!;
const providerGemmaBtn = document.getElementById("provider-gemma")!;
const providerGroqBtn = document.getElementById("provider-groq")!;

// provider
let provider = "gemma";

providerGemmaBtn.addEventListener("click", () => {
  provider = "gemma";
  providerGemmaBtn.classList.add("active");
  providerGroqBtn.classList.remove("active");
});

providerGroqBtn.addEventListener("click", () => {
  provider = "groq";
  providerGroqBtn.classList.add("active");
  providerGemmaBtn.classList.remove("active");
});

// zoom
let zoomLevel = 1.0;
const ZOOM_STEP = 0.15;
const ZOOM_MIN = 0.3;
const ZOOM_MAX = 4.0;

function applyZoom() {
  document.querySelectorAll<HTMLElement>("#svg-container .page").forEach(p => {
    p.style.zoom = String(zoomLevel);
  });
}

function changeZoom(delta: number) {
  zoomLevel = Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, zoomLevel + delta));
  applyZoom();
}

svgContainer.addEventListener("wheel", e => {
  if (!e.ctrlKey && !e.metaKey) return;
  e.preventDefault();
  changeZoom(e.deltaY < 0 ? ZOOM_STEP : -ZOOM_STEP);
}, { passive: false });

document.addEventListener("keydown", e => {
  if (!e.ctrlKey && !e.metaKey) return;
  if (e.key === "=" || e.key === "+") { e.preventDefault(); changeZoom(ZOOM_STEP); }
  else if (e.key === "-") { e.preventDefault(); changeZoom(-ZOOM_STEP); }
  else if (e.key === "0") { e.preventDefault(); zoomLevel = 1.0; applyZoom(); }
});

// toggle panels
function setPaneVisible(pane: HTMLElement, divider: HTMLElement, btn: HTMLElement, visible: boolean) {
  pane.style.display = visible ? "" : "none";
  divider.style.display = visible ? "" : "none";
  btn.classList.toggle("active", visible);
}

toggleTypstBtn.addEventListener("click", () => {
  setPaneVisible(outputPane, divider1, toggleTypstBtn, outputPane.style.display === "none");
});

togglePreviewBtn.addEventListener("click", () => {
  setPaneVisible(previewPane, divider2, togglePreviewBtn, previewPane.style.display === "none");
});

let lastGenerated = "";
let lastParsedEnd = 0;

function setStatus(el: HTMLElement, state: "idle" | "loading" | "done" | "error", text: string) {
  el.className = state;
  el.textContent = text;
}

function setTypstContent(content: string) {
  typstView.dispatch({
    changes: { from: 0, to: typstView.state.doc.length, insert: content },
  });
}

function appendTypstContent(chunk: string) {
  const current = typstView.state.doc.toString();
  const sep = current.trim() ? "\n\n" : "";
  typstView.dispatch({
    changes: { from: typstView.state.doc.length, insert: sep + chunk },
  });
}

async function compile() {
  const content = typstView.state.doc.toString();
  if (!content.trim()) {
    svgContainer.innerHTML = "";
    setStatus(previewStatus, "idle", "");
    return;
  }
  setStatus(previewStatus, "loading", "compilazione...");
  try {
    const pages = await invoke<string[]>("compile_to_svg", { content });
    svgContainer.innerHTML = pages.map(svg => `<div class="page">${svg}</div>`).join("");
    applyZoom();
    setStatus(previewStatus, "done", `${pages.length} ${pages.length === 1 ? "pagina" : "pagine"}`);
  } catch (e) {
    setStatus(previewStatus, "error", String(e));
  }
}

async function convertAndCompile() {
  const fullText = italianView.state.doc.toString();

  // se l'utente ha cancellato testo prima di lastParsedEnd, reset completo
  if (lastParsedEnd > fullText.length) {
    lastParsedEnd = 0;
    lastGenerated = "";
    setTypstContent("");
  }

  const newText = fullText.slice(lastParsedEnd);

  if (!newText.trim()) {
    if (!fullText.trim()) {
      setTypstContent("");
      svgContainer.innerHTML = "";
      setStatus(statusEl, "idle", "Ctrl+S per convertire");
      setStatus(previewStatus, "idle", "");
    }
    return;
  }

  setStatus(statusEl, "loading", "conversione...");
  setStatus(previewStatus, "idle", "");
  try {
    const typst = await invoke<string>("convert_to_typst", { text: newText, provider });
    lastGenerated += (lastGenerated ? "\n\n" : "") + typst;
    lastParsedEnd = fullText.length;
    appendTypstContent(typst);
    setStatus(statusEl, "done", "ok");
  } catch (e) {
    setStatus(statusEl, "error", String(e));
    return;
  }

  await compile();
}

async function saveSession() {
  const italian = italianView.state.doc.toString();
  const corrected = typstView.state.doc.toString();
  if (!italian.trim() && !corrected.trim()) return;
  try {
    const path = await invoke<string>("save_session", {
      italian,
      generated: lastGenerated,
      corrected,
      provider,
    });
    setStatus(statusEl, "done", `salvato → ${path}`);
  } catch (e) {
    setStatus(statusEl, "error", String(e));
  }
}

const editorTheme = EditorView.theme({
  "&": { height: "100%" },
  ".cm-content": { padding: "16px" },
  ".cm-scroller": { overflow: "auto" },
});

// pannello sinistro: prosa italiana — Ctrl+S converte + compila
const italianView = new EditorView({
  state: EditorState.create({
    doc: "",
    extensions: [
      basicSetup,
      oneDark,
      EditorView.lineWrapping,
      editorTheme,
      EditorView.domEventHandlers({
        keydown(e) {
          if (e.key.toLowerCase() === "s" && (e.ctrlKey || e.metaKey) && e.shiftKey) {
            e.preventDefault();
            saveSession();
          } else if (e.key.toLowerCase() === "s" && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            convertAndCompile();
          }
        },
      }),
    ],
  }),
  parent: document.getElementById("editor-inner")!,
});

// pannello centrale: Typst editabile — Ctrl+S compila direttamente
const typstView = new EditorView({
  state: EditorState.create({
    doc: "",
    extensions: [
      basicSetup,
      oneDark,
      EditorView.lineWrapping,
      editorTheme,
      EditorView.domEventHandlers({
        keydown(e) {
          if (e.key.toLowerCase() === "s" && (e.ctrlKey || e.metaKey) && e.shiftKey) {
            e.preventDefault();
            saveSession();
          } else if (e.key.toLowerCase() === "s" && (e.ctrlKey || e.metaKey)) {
            e.preventDefault();
            compile();
          }
        },
      }),
    ],
  }),
  parent: document.getElementById("typst-editor")!,
});

setStatus(statusEl, "idle", "Ctrl+S per convertire");
