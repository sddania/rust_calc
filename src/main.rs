use std::collections::VecDeque;
use std::io::{self, Write};

/// Converte una stringa infissa (già tokenizzata) in RPN (postfissa).
///
/// * `tokens` – slice di token già separati (es. ["3", "+", "4", "*", "2"])
/// * ritorna `Ok(Vec<String>)` contenente la sequenza RPN oppure
///   `Err(String)` con un messaggio di errore.
fn shunting_yard(tokens: &[&str]) -> Result<Vec<String>, String> {
    // Output può essere una coda, ma per semplicità usiamo un Vec.
    // (VecDeque è mostrato qui solo per adempiere alla tua richiesta).
    let mut output: VecDeque<String> = VecDeque::new();
    let mut ops: Vec<&str> = Vec::new();

    // Priorità degli operatori
    fn prec(op: &str) -> u8 {
        match op {
            "+" | "-" => 1,
            "*" | "/" => 2,
            _ => 0,
        }
    }

    for token in tokens {
        // 1️⃣ Numero → lo aggiungiamo direttamente all'output
        if token.parse::<f64>().is_ok() {
            output.push_back(token.to_string());
            continue;
        }

        // 2️⃣ Operatore (+‑‑* /)
        if ["+", "-", "*", "/"].contains(token) {
            while let Some(&top) = ops.last() {
                if top != "(" && prec(top) >= prec(token) {
                    // Pop dall'op stack e push nell'output
                    output.push_back(ops.pop().unwrap().to_string());
                } else {
                    break;
                }
            }
            ops.push(token);
            continue;
        }

        // 3️⃣ Parentesi aperta
        if *token == "(" {
            ops.push(token);
            continue;
        }

        // 4️⃣ Parentesi chiusa
        if *token == ")" {
            while let Some(&top) = ops.last() {
                if top == "(" {
                    ops.pop(); // rimuove la '('
                    break;
                } else {
                    output.push_back(ops.pop().unwrap().to_string());
                }
            }
            continue;
        }

        if *token == " " {
            continue;
        }

        // 5️⃣ Qualcosa di sconosciuto
        return Err(format!("Token non riconosciuto: {}", token));
    }

    // 6️⃣ Svuota lo stack degli operatori residui
    while let Some(op) = ops.pop() {
        if op == "(" || op == ")" {
            return Err("Parentesi sbilanciate".into());
        }
        output.push_back(op.to_string());
    }

    // Convertiamo la VecDeque in un Vec<String> per restituirlo
    Ok(output.into_iter().collect())
}

/// Valuta una sequenza RPN (postfissa) composta solo da stringhe.
///
// * `rpn` – slice di token RPN (es. ["3","4","2","*","+"])
/// * ritorna `Ok(f64)` con il risultato oppure `Err(String)` con un messaggio.
fn eval_rpn(rpn: &[String]) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();

    for token in rpn {
        // Se è un numero, lo pushiamo sullo stack
        if let Ok(num) = token.parse::<f64>() {
            stack.push(num);
            continue;
        }

        // Altrimenti dovrebbe essere un operatore binario
        if stack.len() < 2 {
            return Err("Stack insufficiente per operatore".into());
        }
        let b = stack.pop().unwrap();
        let a = stack.pop().unwrap();

        let res = match token.as_str() {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" => {
                if b == 0.0 {
                    return Err("Divisione per zero".into());
                }
                a / b
            }
            _ => return Err(format!("Operatore sconosciuto: {}", token)),
        };
        stack.push(res);
    }

    if stack.len() != 1 {
        Err("Valutazione incompleta".into())
    } else {
        Ok(stack[0])
    }
}

/* --------------------------------------------------------------- */
/*  Helper: tokenizza una stringa arbitraria in &str separati da spazi */
/* --------------------------------------------------------------- */
fn split_into_tokens(expr: &str) -> Vec<&str> {
    // Rimuove spazi superflui e separa su whitespace.
    // Nota: questo semplice splitter non gestisce numeri con spazi interni
    // (es. "12 34") ma è sufficiente per dimostrazioni.
    expr.split_ascii_whitespace().collect()
}

fn main() {
    println!("=== Shunting‑Yard (solo stringhe) ===");
    println!("Digita un'espressione infissa oppure 'exit' per terminare.");

    loop {
        print!(">>> ");
        let line = match read_line() {
            Some(value) => value,
            None => continue,
        };

        if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
            println!("Arrivederci!");
            break;
        }

        // 1️⃣ Tokenizza l'input
        let tokens = split_into_tokens(line.as_str());

        // 2️⃣ Converte in RPN
        let rpn = match shunting_yard(&tokens) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Errore Shunting‑Yard: {}", e);
                continue;
            }
        };

        // 3️⃣ Valuta la RPN
        let result = match eval_rpn(&rpn) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Errore valutazione: {}", e);
                continue;
            }
        };

        // 4️⃣ Mostra il risultato
        println!("RPN : {}", rpn.join(" "));
        println!("Risultato = {}", result);
    }
}

fn read_line() -> Option<String> {
    io::stdout().flush().unwrap();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        eprintln!("Impossibile leggere da stdin");
        return None;
    }
    let line = line.trim().to_string();
    Some(line)
}
