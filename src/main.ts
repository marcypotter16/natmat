import "./style.css";
import { EditorView, basicSetup } from "codemirror";
import { EditorState } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import { invoke } from "@tauri-apps/api/core";

const statusEl = document.getElementById("status")!;
const previewStatus = document.getElementById("preview-status")!;
const svgContainer = document.getElementById("svg-container")!;

let lastGenerated = "";

function setStatus(el: HTMLElement, state: "idle" | "loading" | "done" | "error", text: string) {
  el.className = state;
  el.textContent = text;
}

function setTypstContent(content: string) {
  typstView.dispatch({
    changes: { from: 0, to: typstView.state.doc.length, insert: content },
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
    setStatus(previewStatus, "done", `${pages.length} ${pages.length === 1 ? "pagina" : "pagine"}`);
  } catch (e) {
    setStatus(previewStatus, "error", String(e));
  }
}

async function convertAndCompile() {
  const text = italianView.state.doc.toString();
  if (!text.trim()) {
    setTypstContent("");
    svgContainer.innerHTML = "";
    setStatus(statusEl, "idle", "Ctrl+S per convertire");
    setStatus(previewStatus, "idle", "");
    return;
  }

  setStatus(statusEl, "loading", "conversione...");
  setStatus(previewStatus, "idle", "");
  try {
    const typst = await invoke<string>("convert_to_typst", { text });
    lastGenerated = typst;
    setTypstContent(typst);
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
  parent: document.getElementById("editor-pane")!,
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
