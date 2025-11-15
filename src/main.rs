use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::io::{self, Write};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Copy, Clone, PartialEq)]
enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Operator {
    pub fn get_priority(&self) -> u8 {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Pow => 3,
        }
    }

    /// true if operator is right-associative (e.g. exponentiation)
    pub fn is_right_associative(&self) -> bool {
        matches!(self, Operator::Pow)
    }
}

impl TryFrom<&str> for Operator {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            // ASCII
            "+" => Ok(Operator::Add),
            "-" => Ok(Operator::Sub),
            "*" => Ok(Operator::Mul),
            "/" => Ok(Operator::Div),
            "^" => Ok(Operator::Pow),
            // Unicode variants
            "×" => Ok(Operator::Mul),
            "÷" => Ok(Operator::Div),
            "−" => Ok(Operator::Sub),
            _ => Err(format!("Invalid operator: {}", value)),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use unicode symbols in output to match the expected test string
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "−"),
            Operator::Mul => write!(f, "×"),
            Operator::Div => write!(f, "÷"),
            Operator::Pow => write!(f, "^"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Operand(f64);

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print integer-like floats without trailing .0 for nicer output
        if (self.0 - (self.0 as i64) as f64).abs() < std::f64::EPSILON {
            write!(f, "{}", self.0 as i64)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl TryFrom<&str> for Operand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Accept only standard ASCII minus as part of numeric literal (tokenizer responsibility)
        match value.parse::<f64>() {
            Ok(v) => Ok(Operand(v)),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Add for Operand {
    type Output = Operand;
    fn add(self, other: Operand) -> Self::Output {
        Operand(self.0 + other.0)
    }
}
impl Sub for Operand {
    type Output = Operand;
    fn sub(self, other: Operand) -> Self::Output {
        Operand(self.0 - other.0)
    }
}
impl Mul for Operand {
    type Output = Operand;
    fn mul(self, other: Operand) -> Self::Output {
        Operand(self.0 * other.0)
    }
}
impl Div for Operand {
    type Output = Operand;
    fn div(self, other: Operand) -> Self::Output {
        Operand(self.0 / other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Operand(Operand),
    Operator(Operator),
    LeftParen,
    RightParen,
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Operand(op) => write!(f, "{}", op),
            Token::Operator(op) => write!(f, "{}", op),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
        }
    }
}

impl TryFrom<&str> for Token {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Try operand first
        if let Ok(val) = Operand::try_from(value) {
            return Ok(Token::Operand(val));
        }
        // Then operator
        if let Ok(op) = Operator::try_from(value) {
            return Ok(Token::Operator(op));
        }
        if value == "(" {
            return Ok(Token::LeftParen);
        }
        if value == ")" {
            return Ok(Token::RightParen);
        }
        Err(format!("cannot parse token: {}", value))
    }
}

/// TokenStack: a FIFO queue of Token. push_back appends, iteration consumes from front.
#[derive(Clone, Debug)]
struct TokenStack(VecDeque<Token>);

impl TokenStack {
    fn new() -> Self {
        TokenStack(VecDeque::new())
    }

    fn push_back(&mut self, t: Token) {
        self.0.push_back(t);
    }

    fn pop_front(&mut self) -> Option<Token> {
        self.0.pop_front()
    }

    fn join(&self, sep: &str) -> String {
        self.0
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join(sep)
    }
}

// Implement Iterator so owned TokenStack can be consumed with `for token in stack`
impl Iterator for TokenStack {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.pop_front()
    }
}

// Allow non-consuming iteration by reference: for token in &stack (returns &Token)
impl<'a> IntoIterator for &'a TokenStack {
    type Item = &'a Token;
    type IntoIter = std::collections::vec_deque::Iter<'a, Token>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Split an expression into tokens. For the user's provided test string, splitting on
/// whitespace is sufficient and preserves unicode operator symbols. Returns Vec<String>.
fn split_into_tokens(expr: &str) -> Vec<String> {
    expr.split_whitespace().map(|s| s.to_string()).collect()
}

/// Shunting‑yard: converte una lista di token in notazione postfissa (RPN).
///
/// Descrizione (in italiano):
/// - `tokens` è una slice di token in ordine infisso (es. ["3", "+", "4", "×", "2"]).
/// - `output` è la coda (risultato) che conterrà i token in ordine RPN.
/// - `ops` è lo stack temporaneo degli operatori e delle parentesi.
///
/// Algoritmo (sintesi):
/// 1. Per ogni token in input:
///    - Se è un operando: appendilo alla coda di output.
///    - Se è un operatore `op`:
///        * Confronta con l'operatore in cima allo stack `top_op` (se presente).
///        * Se `top_op` è un operatore e ha priorità maggiore (o uguale per
///          operatori left-associative), estrai `top_op` nello `output`.
///        * Ripeti finché la condizione di pop è vera, quindi metti `op` nello stack.
///      Nota: per operatori right-associative (es. `^`) la condizione di pop è
///            diversa (pop solo se `top_pr > cur_pr`).
///    - Se è `(`: push nello stack.
///    - Se è `)`: pop dallo stack fino a trovare `(`; se non trovi `(` -> errore.
///
/// 2. Alla fine, svuota lo stack degli operatori nella coda di output.
///    Se trovi ancora parentesi, allora le parentesi erano sbilanciate -> errore.
///
/// Il comportamento segue la versione classica dello Shunting‑Yard (gestione
/// di precedenza e associatività).
fn shunting_yard(tokens: &[String]) -> Result<TokenStack, String> {
    // output = coda (RPN)
    let mut output = TokenStack::new();
    // ops = stack temporaneo per operatori/parentesi
    let mut ops: Vec<Token> = Vec::new();

    for tok in tokens {
        // Converti la stringa token in Token (Operand/Operator/Paren)
        let token = Token::try_from(tok.as_str())?;
        match token {
            // Se è un valore/numero -> aggiungi all'output (RPN)
            Token::Operand(o) => output.push_back(Token::Operand(o)),

            // Se è un operatore:
            Token::Operator(op) => {
                // Confronta con l'operatore in cima allo stack finché conviene poppare.
                while let Some(top) = ops.last() {
                    if let Token::Operator(top_op) = top {
                        let top_pr = top_op.get_priority();
                        let cur_pr = op.get_priority();

                        // Se l'operatore corrente è right-associative (es. ^),
                        // dobbiamo poppare solo quando top_pr > cur_pr.
                        // Altrimenti (left-associative), poppiamo quando top_pr >= cur_pr.
                        let should_pop = if op.is_right_associative() {
                            top_pr > cur_pr
                        } else {
                            top_pr >= cur_pr
                        };

                        if should_pop {
                            // Sposta l'operatore dallo stack all'output
                            let popped = ops.pop().unwrap();
                            output.push_back(popped);
                            // continua a verificare il nuovo top
                            continue;
                        }
                    }
                    // Se non c'è un operatore in cima oppure non dobbiamo poppare -> esci
                    break;
                }
                // Metti l'operatore corrente nello stack
                ops.push(Token::Operator(op));
            }

            // Left paren: semplicemente mettila nello stack per segnare l'inizio di un gruppo
            Token::LeftParen => ops.push(Token::LeftParen),

            // Right paren: poppa fino alla left paren corrispondente
            Token::RightParen => {
                let mut found = false;
                while let Some(top) = ops.pop() {
                    match top {
                        Token::LeftParen => {
                            found = true;
                            break;
                        }
                        // Se non è left paren, spostalo nell'output
                        other => output.push_back(other),
                    }
                }
                // Se non abbiamo trovato la parentesi di apertura, è un errore
                if !found {
                    return Err("Parentesi sbilanciate".into());
                }
            }
        }
    }

    // Alla fine: svuota qualsiasi operatore residuo nello stack nell'output.
    while let Some(op) = ops.pop() {
        match op {
            Token::LeftParen | Token::RightParen => return Err("Parentesi sbilanciate".into()),
            other => output.push_back(other),
        }
    }

    Ok(output)
}

/// Valuta una sequenza RPN (notazione postfissa) contenuta in `TokenStack`.
///
/// Principio:
/// - Si usa uno stack (LIFO) di operandi.
/// - Per ogni token nella sequenza RPN:
///     * se è un operando, lo si mette nello stack;
///     * se è un operatore, si estraggono due operandii dallo stack (prima `b`, poi `a`)
///       e si applica `a op b` (attenzione all'ordine: l'operando estratto per secondo è il
///       sinistro dell'operatore).
/// - Alla fine lo stack deve contenere esattamente un valore (il risultato).
///
/// Errori gestiti:
/// - operatore senza abbastanza operandi -> errore (stack insufficiente)
/// - divisione per zero -> errore
/// - presenza di parentesi in RPN -> errore (non dovrebbero comparire)
fn eval_rpn(rpn: &TokenStack) -> Result<Operand, String> {
    let mut stack: Vec<Operand> = Vec::new();

    // Iteriamo consumando una copia della coda RPN (in ordine FIFO)
    for token in rpn.clone() {
        match token {
            // Operando: push nello stack
            Token::Operand(op) => stack.push(op),

            // Operatore: poppa due operandi e applica l'operazione
            Token::Operator(operator) => {
                if stack.len() < 2 {
                    // Non ci sono abbastanza valori per applicare l'operatore
                    return Err("Stack insufficiente per operatore".into());
                }
                // b è il secondo operando (destro), a è il primo (sinistro)
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();

                // Esegui l'operazione corrispondente
                let res = match operator {
                    Operator::Add => a + b,
                    Operator::Sub => a - b,
                    Operator::Mul => a * b,
                    Operator::Div => {
                        // Controllo divisione per zero (f64)
                        if b.0.abs() < std::f64::EPSILON {
                            return Err("Divisione per zero".into());
                        }
                        a / b
                    }
                    Operator::Pow => Operand(a.0.powf(b.0)),
                };
                // Inserisci il risultato nello stack per eventuali operazioni successive
                stack.push(res);
            }

            // Le parentesi non dovrebbero comparire in RPN: segnalazione d'errore
            Token::LeftParen | Token::RightParen => {
                return Err("Parentesi trovata nell'RPN".into());
            }
        }
    }

    // Alla fine ci deve essere esattamente un valore nello stack (il risultato)
    if stack.len() != 1 {
        Err("Valutazione incompleta".into())
    } else {
        Ok(stack[0])
    }
}

fn main() {
    println!("=== Shunting‑Yard & RPN evaluator ===");
    println!(
        "Digita un'espressione infissa (es. 3 + 4 * 2 / ( 1 − 5 ) ^ 2 ^ 3) oppure 'exit' per terminare."
    );

    loop {
        print!(">>> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            eprintln!("error reading from stdin");
            continue;
        }
        let line = line.trim();
        if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
            println!("done");
            break;
        }
        if line.is_empty() {
            continue;
        }

        let tokens = split_into_tokens(line);
        match shunting_yard(&tokens) {
            Ok(rpn) => {
                println!("RPN : {}", rpn.join(" "));
                match eval_rpn(&rpn) {
                    Ok(res) => println!("Res = {}", res),
                    Err(e) => eprintln!("Err = {}", e),
                }
            }
            Err(e) => eprintln!("Err Shunting‑Yard: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_given_expression_rpn_matches_expected() {
        let expr = "3 + 4 × 2 ÷ ( 1 − 5 ) ^ 2 ^ 3";
        let tokens = split_into_tokens(expr);
        assert!(!tokens.is_empty(), "Tokenization failed");

        let rpn = shunting_yard(&tokens).expect("shunting_yard failed");
        let out = rpn.join(" ");
        let expected = "3 4 2 × 1 5 − 2 3 ^ ^ ÷ +";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_eval_rpn_numeric_result() {
        // Same expression as above; numeric result should be 3.0001220703125
        let expr = "3 + 4 × 2 ÷ ( 1 − 5 ) ^ 2 ^ 3";
        let tokens = split_into_tokens(expr);
        let rpn = shunting_yard(&tokens).expect("shunting_yard failed");

        // Evaluate RPN and check numeric result within a small tolerance
        let result = eval_rpn(&rpn).expect("eval_rpn failed");
        let expected = 3.0001220703125_f64;
        let diff = (result.0 - expected).abs();
        assert!(
            diff < 1e-12,
            "Numeric result differs: got {}, expected {}, diff {}",
            result.0,
            expected,
            diff
        );
    }
}
