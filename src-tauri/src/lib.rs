use serde::{Deserialize, Serialize};

// --- Gemini structs ---

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct GeminiRequest {
    system_instruction: SystemInstruction,
    contents: Vec<Content>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
    #[serde(default)]
    thought: bool,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

// --- Groq (OpenAI-compatible) structs ---

#[derive(Serialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
}

#[derive(Deserialize)]
struct GroqResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqResponseMessage,
}

#[derive(Deserialize)]
struct GroqResponse {
    choices: Vec<GroqChoice>,
}

const GEMINI_SYSTEM_PROMPT: &str = r#"Converti prosa matematica italiana in Typst. Restituisci SOLO il documento Typst, niente altro.

BACKSLASH: MAI USARE.

\epsilon NO → epsilon SI
\lambda NO → lambda SI
\delta NO → delta SI
\sigma NO → sigma SI
\alpha NO → alpha SI
\pi NO → pi SI
\to NO → -> SI
\Rightarrow NO → => SI
\implies NO → => SI
\Leftrightarrow NO → <=> SI
\in NO → in SI
\forall NO → forall SI
\exists NO → exists SI
\leq NO → <= SI
\geq NO → >= SI
\infty NO → oo SI
\sup NO → sup SI
\subseteq NO → subset.eq SI
\supseteq NO → supset.eq SI
\sum_{n=0}^{\infty} NO → sum_(n=0)^oo SI
\mathbb{Z} NO → ZZ SI
\mathbb{R} NO → RR SI
\mathbb{C} NO → CC SI
\mathbb{Q} NO → QQ SI
\mathbb{N} NO → NN SI
\mathbb{F} NO → FF SI (es. FF_q per il campo finito)
\mathrm{Hom} NO → "Hom" SI
\frac{a}{b} NO → a/b SI

== ALTRE REGOLE ==

GRAFFE NEI PEDICI: MAI. x_{n+1} NO → x_(n+1) SI

TESTO IN MATH: virgolette doppie: "Hom" "Ext" "GL" "Ab" "weight" "coker"
  \mathrm{Hom} NO → "Hom" SI
  \text{Hom} NO → "Hom" SI (anche \text è sbagliato)

FUNZIONI BUILT-IN (no virgolette, no \text{}, no \mathrm{}): floor ceil sup inf min max abs norm det dim ker
  \text{floor} NO → floor SI
  "floor" NO → floor SI

MORFISMI: f #h(0pt)colon X -> Y (non f: X -> Y)

SPAZIO prima di parentesi dopo pedici/apici: H_n (X) non H_n(X), H^n (X; G) non H^n(X;G)

DOLLAR: ogni $ di apertura deve avere il $ di chiusura. Non spezzare mai un'espressione in due blocchi $ separati.
  SBAGLIATO: $"weight"(x P) = $"weight"(x)$
  CORRETTO:  $"weight"(x P) = "weight"(x)$
  SBAGLIATO: abbiamo "Ext"(L, G) = 0$
  CORRETTO:  abbiamo $"Ext"(L, G) = 0$

HASH: sempre \# in testo e math.

SEQUENZE ESATTE: blocco display unico $ 0 -> A -> B -> C -> 0 $

DIAGRAMMI COMMUTATIVI: pacchetto fletcher
  #import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge
  #align(center)[#diagram($
    A edge("r", f, ->) & B \
    edge("d", g, ->) & C edge("u", h, ->)
  $)]
  direzioni: "r" "l" "u" "d" "dr" "dl" "ur" "ul"

== ESEMPIO ==

Input: "Per ogni eps > 0 esiste N in NN tc per ogni n > N vale fn(x) - f(x) < eps.
Se X ha l componenti connesse, H0 di X con G è isomorfo a G alla l.
Per il UCT: successione esatta corta 0 a Ext di H0(X) G a H1(X;G) a Hom di H1(X) G a 0."

Output:
Per ogni $epsilon > 0$ esiste $N in NN$ tale che per ogni $n > N$ vale $abs(f_n (x) - f(x)) < epsilon$.

Se $X$ ha $l$ componenti connesse, allora $H^0 (X; G) tilde.equiv G^l$.

Per il Teorema dei Coefficienti Universali abbiamo la successione esatta corta
$ 0 -> "Ext"(H_0 (X), G) -> H^1 (X; G) -> "Hom"(H_1 (X), G) -> 0. $"#;

const GROQ_SYSTEM_PROMPT: &str = r#"Converti prosa matematica italiana in LaTeX. Restituisci SOLO il corpo del documento (senza \documentclass, \begin{document}, \end{document} o preamble), niente altro.

Usa LaTeX standard con amsmath, amssymb, amsthm. La prosa normale rimane prosa. La matematica va in $...$ (inline) o \[...\] (display).

== ESEMPIO ==

Input: "Per ogni eps > 0 esiste N naturale tc per ogni n > N vale fn(x) - f(x) < eps.
Se X ha l componenti connesse, H0 di X con G è isomorfo a G alla l.
Per il UCT: successione esatta corta 0 a Ext di H0(X) G a H1(X;G) a Hom di H1(X) G a 0."

Output:
Per ogni $\varepsilon > 0$ esiste $N \in \mathbb{N}$ tale che per ogni $n > N$ vale $|f_n(x) - f(x)| < \varepsilon$.

Se $X$ ha $l$ componenti connesse, allora $H^0(X; G) \cong G^l$.

Per il Teorema dei Coefficienti Universali abbiamo la successione esatta corta
\[ 0 \to \mathrm{Ext}(H_0(X), G) \to H^1(X; G) \to \mathrm{Hom}(H_1(X), G) \to 0. \]"#;

#[tauri::command]
async fn convert_to_typst(text: String, provider: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    match provider.as_str() {
        "groq" => {
            let api_key = std::env::var("GROQ_API_KEY")
                .map_err(|_| "GROQ_API_KEY non trovata nell'ambiente".to_string())?;

            let body = GroqRequest {
                model: "llama-3.3-70b-versatile".to_string(),
                messages: vec![
                    GroqMessage { role: "system".to_string(), content: GROQ_SYSTEM_PROMPT.to_string() },
                    GroqMessage { role: "user".to_string(), content: text },
                ],
            };

            let response = client
                .post("https://api.groq.com/openai/v1/chat/completions")
                .bearer_auth(api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Errore HTTP: {e}"))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error {status}: {body}"));
            }

            let parsed: GroqResponse = response
                .json()
                .await
                .map_err(|e| format!("Errore parsing risposta: {e}"))?;

            parsed
                .choices
                .into_iter()
                .next()
                .map(|c| c.message.content)
                .ok_or_else(|| "Risposta vuota dall'API".to_string())
        }

        _ => {
            let api_key = std::env::var("GEMINI_API_KEY")
                .map_err(|_| "GEMINI_API_KEY non trovata nell'ambiente".to_string())?;

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemma-4-31b-it:generateContent?key={}",
                api_key
            );

            let body = GeminiRequest {
                system_instruction: SystemInstruction {
                    parts: vec![Part { text: GEMINI_SYSTEM_PROMPT.to_string() }],
                },
                contents: vec![Content {
                    parts: vec![Part { text }],
                }],
            };

            let response = client
                .post(&url)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| format!("Errore HTTP: {e}"))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error {status}: {body}"));
            }

            let parsed: GeminiResponse = response
                .json()
                .await
                .map_err(|e| format!("Errore parsing risposta: {e}"))?;

            parsed
                .candidates
                .into_iter()
                .next()
                .and_then(|c| {
                    c.content.parts
                        .into_iter()
                        .filter(|p| !p.thought)
                        .map(|p| p.text)
                        .reduce(|a, b| a + &b)
                })
                .ok_or_else(|| "Risposta vuota dall'API".to_string())
        }
    }
}

#[tauri::command]
async fn compile_to_svg(content: String) -> Result<Vec<String>, String> {
    let tmp = std::env::temp_dir().join("natmat");
    tokio::fs::create_dir_all(&tmp).await.map_err(|e| e.to_string())?;

    let preamble = r#"\documentclass[12pt]{article}
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{amsmath,amssymb,amsthm}
\usepackage[margin=2cm]{geometry}
\pagestyle{empty}
\begin{document}
"#;
    let full_doc = format!("{}{}\n\\end{{document}}", preamble, content);

    let tex_file = tmp.join("doc.tex");
    tokio::fs::write(&tex_file, &full_doc).await.map_err(|e| e.to_string())?;

    // pdflatex
    let output = tokio::process::Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-output-directory")
        .arg(&tmp)
        .arg(&tex_file)
        .output()
        .await
        .map_err(|e| format!("pdflatex non trovato nel PATH: {e}"))?;

    if !output.status.success() {
        let log = tokio::fs::read_to_string(tmp.join("doc.log")).await.unwrap_or_default();
        let errors: String = log.lines()
            .filter(|l| l.starts_with('!'))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(format!("Errore LaTeX:\n{errors}"));
    }

    // dvisvgm — converte PDF in SVG pagina per pagina
    let pdf_file = tmp.join("doc.pdf");
    let output = tokio::process::Command::new("dvisvgm")
        .arg("--pdf")
        .arg("--page=1-")
        .arg("--output=%f-p%p.svg")
        .arg(&pdf_file)
        .current_dir(&tmp)
        .output()
        .await
        .map_err(|e| format!("dvisvgm non trovato nel PATH: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Errore dvisvgm: {stderr}"));
    }

    // raccoglie doc-p1.svg, doc-p2.svg, ...
    let mut pages = Vec::new();
    let mut i = 1;
    loop {
        let path = tmp.join(format!("doc-p{i}.svg"));
        if !path.exists() { break; }
        let svg = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
        pages.push(svg);
        i += 1;
    }

    if pages.is_empty() {
        return Err("Nessun SVG generato da dvisvgm".to_string());
    }

    Ok(pages)
}

#[tauri::command]
async fn save_session(
    italian: String,
    generated: String,
    corrected: String,
    provider: String,
) -> Result<String, String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "impossibile trovare la home directory".to_string())?;

    let dir = std::path::PathBuf::from(home)
        .join("Documents")
        .join("natmat")
        .join("natmat-sessions")
        .join(&provider)
        .join(ts.to_string());

    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("italian.txt"), &italian).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("generated.typ"), &generated).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("corrected.typ"), &corrected).map_err(|e| e.to_string())?;

    Ok(dir.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![convert_to_typst, compile_to_svg, save_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
