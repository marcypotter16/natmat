use serde::{Deserialize, Serialize};

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

const SYSTEM_PROMPT: &str = r#"Sei un convertitore da prosa matematica italiana a Typst.
Ricevi appunti scritti in linguaggio naturale italiano e restituisci SOLO il documento Typst corrispondente,
senza spiegazioni, senza markdown, senza backtick.

== REGOLE FONDAMENTALI DI SINTASSI TYPST (non LaTeX) ==

1. NIENTE BACKSLASH. I simboli si scrivono per nome senza \:
   - `pi` non `\pi`, `in` non `\in`, `sum` non `\sum`, `partial` non `\partial`
   - `forall` non `\forall`, `exists` non `\exists`, `to` non `\to`
   - `times` non `\times`, `oplus` non `\oplus`, `otimes` non `\otimes`
   - `subset` non `\subset`, `supset` non `\supset`

2. GROUPING CON PARENTESI TONDE, non graffe:
   - `x_(n+1)` non `x_{n+1}`
   - `f^((n))` non `f^{(n)}`
   - `sum_(n=0)^oo` non `\sum_{n=0}^{\infty}`

3. TESTO IN MATH MODE con virgolette doppie:
   - `"Hom"` non `\mathrm{Hom}` o `\text{Hom}`
   - `"Ext"` non `\mathrm{Ext}`
   - `"Ab"` non `\mathrm{Ab}`

4. SIMBOLI CHE CAMBIANO NOME rispetto a LaTeX:
   - `ZZ` = \mathbb{Z}, `RR` = \mathbb{R}, `CC` = \mathbb{C}, `QQ` = \mathbb{Q}, `NN` = \mathbb{N}
   - `oo` = \infty
   - `->` = \to, `-->` = \longrightarrow, `<-` = \leftarrow
   - `=>` = \Rightarrow, `<=>` = \Leftrightarrow
   - `tilde.equiv` = \cong, `equiv` = \equiv
   - `!=` = \neq, `<=` = \leq, `>=` = \geq
   - `compose` = \circ
   - `plus.circle` = \oplus (somma diretta)
   - `colon` = \colon (nei morfismi)

5. FRAZIONI: usa `/` direttamente: `(a+b)/(c+d)` non `\frac{a+b}{c+d}`

6. SEQUENZE ESATTE: usa `->` tra i termini, tutto in un unico blocco display `$ ... $`

7. MORFISMI — il colon nei morfismi va scritto `#h(0pt)colon` per la spaziatura corretta:
   - `$f #h(0pt)colon X -> Y$` non `$f: X -> Y$` e non `$f colon X -> Y$`

8. DIAGRAMMI COMMUTATIVI — usa il pacchetto fletcher con questa sintassi:
   ```
   #import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge

   #align(center)[#diagram($
     A edge("r", f, ->) & B \
     edge("d", g, ->) & C edge("u", h, ->)
   $)]
   ```
   - La griglia usa `&` per separare colonne e `\` per andare a capo (come nelle matrici)
   - `edge("r", label, ->)` = freccia a destra; direzioni: "r","l","u","d","dr","dl","ur","ul"
   - Frecce: `->` = normale, `=>` = doppia, `<->` = bidirezionale, `"..>"` = tratteggiata
   - Esempio diagramma triangolare commutativo:
   ```
   #align(center)[#diagram($
     pi_1(X) edge(pi, ->) edge("d", f, ->) & H_1(X) edge("dl", f', ->) \
     G
   $)]
   ```

== ESEMPIO ==

Input: "Se X ha l componenti connesse allora H0 di X con coefficienti G è isomorfo a G alla l.
Per il UCT abbiamo la successione esatta corta 0 a Ext di H0X G a H1 X G a Hom H1X G a 0."

Output:
Se $X$ ha $l$ componenti connesse, allora $H^0(X; G) tilde.equiv G^l$.

Per il Teorema dei Coefficienti Universali abbiamo la successione esatta corta
$ 0 -> "Ext"(H_0(X), G) -> H^1(X; G) -> "Hom"(H_1(X), G) -> 0. $

== ALTRE REGOLE ==
- La prosa normale rimane prosa nel documento Typst.
- Inferisci indici e limiti dal contesto quando impliciti.
- Non inventare contenuto non presente nel testo originale.
- ATTENZIONE: ogni espressione matematica inline deve avere sia il `$` di apertura che quello di chiusura. Errore comune da evitare: scrivere `abbiamo "Ext"(L, G) = 0$` invece di `abbiamo $"Ext"(L, G) = 0$`.
- Il simbolo `#` in Typst avvia del codice, quindi va sempre escapato come `\#` quando appare in testo o in matematica.
- SPAZIO PRIMA DELLE PARENTESI dopo pedici/apici: in Typst `H_n(X)` viene parsato come H con subscript n(X), il che è sbagliato. Scrivi sempre `H_n (X)` con uno spazio. Regola generale: se dopo un pedice o apice seguono parentesi con argomenti, inserisci uno spazio. Esempi corretti: `H_n (X)`, `H^n (X; G)`, `"Hom"(H_n (X), G)`, `"Ext"(H_0 (X), G)`."#;

#[tauri::command]
async fn convert_to_typst(text: String) -> Result<String, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY non trovata nell'ambiente".to_string())?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemma-4-31b-it:generateContent?key={}",
        api_key
    );

    let client = reqwest::Client::new();

    let body = GeminiRequest {
        system_instruction: SystemInstruction {
            parts: vec![Part { text: SYSTEM_PROMPT.to_string() }],
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

#[tauri::command]
async fn compile_to_svg(content: String) -> Result<Vec<String>, String> {
    let tmp = std::env::temp_dir().join("natmat");
    tokio::fs::create_dir_all(&tmp).await.map_err(|e| e.to_string())?;

    let typ_file = tmp.join("doc.typ");
    tokio::fs::write(&typ_file, &content).await.map_err(|e| e.to_string())?;

    // clean up previous output
    let _ = tokio::fs::remove_file(tmp.join("doc.svg")).await;
    for i in 1..=50 {
        let _ = tokio::fs::remove_file(tmp.join(format!("doc_{i}.svg"))).await;
    }

    let output = tokio::process::Command::new("typst")
        .arg("compile")
        .arg(&typ_file)
        .arg(tmp.join("doc.svg"))
        .output()
        .await
        .map_err(|e| format!("typst non trovato nel PATH: {e}. Installalo da https://typst.app"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Errore Typst: {stderr}"));
    }

    // collect pages: typst outputs doc.svg (single) or doc_1.svg, doc_2.svg, ... (multi)
    let mut pages = Vec::new();

    let single = tmp.join("doc.svg");
    if single.exists() {
        let svg = tokio::fs::read_to_string(&single).await.map_err(|e| e.to_string())?;
        pages.push(svg);
    } else {
        let mut i = 1;
        loop {
            let path = tmp.join(format!("doc_{i}.svg"));
            if !path.exists() { break; }
            let svg = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
            pages.push(svg);
            i += 1;
        }
    }

    if pages.is_empty() {
        return Err("Nessun SVG generato da typst".to_string());
    }

    Ok(pages)
}

#[tauri::command]
async fn save_session(
    italian: String,
    generated: String,
    corrected: String,
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
        .join("natmat-sessions")
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
