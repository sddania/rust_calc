// src/main.rs
use std::io::{self, Write};

/// Operazioni supportate dalla calcolatrice
enum Operazione {
    Add,
    Sottrai,
    Moltiplica,
    Dividi,
}

/// Calcola il risultato di a (op) b
fn calcola(a: f64, b: f64, op: Operazione) -> Result<f64, String> {
    match op {
        Operazione::Add => Ok(a + b),
        Operazione::Sottrai => Ok(a - b),
        Operazione::Moltiplica => Ok(a * b),
        Operazione::Dividi => {
            if b == 0.0 {
                Err("Divisione per zero".into())
            } else {
                Ok(a / b)
            }
        }
    }
}

/// Legge una riga dall'input standard
fn leggi_riga() -> String {
    print!("Inserisci espressione (es. 3 + 4) o 'exit' per terminare: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Errore nella lettura da stdin");
    input.trim().to_string()
}

/// Converte la stringa in (a, operazione, b)
fn parse_input(input: &str) -> Result<(f64, Operazione, f64), String> {
    let parti: Vec<&str> = input.split_whitespace().collect();
    if parti.len() != 3 {
        return Err(
            "Formato errato: devi inserire 'numero operatore numero' (es. 2 + 3)".into(),
        );
    }

    let a: f64 = parti[0].parse().map_err(|_| "Primo numero non valido")?;
    let op = match parti[1] {
        "+" => Operazione::Add,
        "-" => Operazione::Sottrai,
        "*" => Operazione::Moltiplica,
        "/" => Operazione::Dividi,
        _ => return Err("Operatore non riconosciuto".into()),
    };
    let b: f64 = parti[2].parse().map_err(|_| "Secondo numero non valido")?;

    Ok((a, op, b))
}

fn main() {
    loop {
        let riga = leggi_riga();

        if riga.eq_ignore_ascii_case("exit") || riga.eq_ignore_ascii_case("quit") {
            println!("Arrivederci!");
            break;
        }

        match parse_input(&riga) {
        Ok((a, op, b)) => match calcola(a, b, op) {
                Ok(ris) => println!("Risultato: {}", ris),
                Err(e) => eprintln!("Errore di calcolo: {}", e),
            },
            Err(e) => eprintln!("Errore di parsing: {}", e),
        }
    }
}