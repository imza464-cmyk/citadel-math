// Copyright (c) 2026 Mekserina Karaosmanoğlu. Dual-licensed under GNU AGPL-3.0-only OR Commercial License (see LICENSE and COMMERCIAL_LICENSE.md).
// Matematik Çekirdeği — Sembolik İfade Ağacı (AST), Boyut Çıkarımı, Sembolik Türev,
// Cebirsel Denklem Tersleme (solve_for), Aralık Aritmetiği (Interval64) ve Otomatik Türev (Dual64)

use crate::dimensions::Dimension;
use crate::interval::{Dual64, Interval64};
use std::collections::HashMap;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Sembolik Matematiksel İfade Düğümü
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(f64),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    Ln(Box<Expr>),
    Exp(Box<Expr>),
    Neg(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Pow,
    LParen,
    RParen,
}

fn tokenize_expr(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'e'
                    || chars[i] == 'E'
                    || ((chars[i] == '+' || chars[i] == '-')
                        && i > start
                        && (chars[i - 1] == 'e' || chars[i - 1] == 'E')))
            {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let val = num_str
                .parse::<f64>()
                .map_err(|_| format!("Geçersiz sayı sabiti: '{}'", num_str))?;
            tokens.push(Token::Number(val));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            tokens.push(Token::Ident(ident));
        } else if c == '*' {
            if i + 1 < chars.len() && chars[i + 1] == '*' {
                tokens.push(Token::Pow);
                i += 2;
            } else {
                tokens.push(Token::Star);
                i += 1;
            }
        } else if c == '^' {
            tokens.push(Token::Pow);
            i += 1;
        } else if c == '+' {
            tokens.push(Token::Plus);
            i += 1;
        } else if c == '-' {
            tokens.push(Token::Minus);
            i += 1;
        } else if c == '/' {
            tokens.push(Token::Slash);
            i += 1;
        } else if c == '(' {
            tokens.push(Token::LParen);
            i += 1;
        } else if c == ')' {
            tokens.push(Token::RParen);
            i += 1;
        } else {
            return Err(format!("Beklenmeyen karakter: '{}'", c));
        }
    }

    Ok(tokens)
}

struct ExprParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ExprParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_term()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Plus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    node = Expr::add(node, rhs);
                }
                Token::Minus => {
                    self.next();
                    let rhs = self.parse_term()?;
                    node = Expr::sub(node, rhs);
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_unary()?;
        while let Some(tok) = self.peek() {
            match tok {
                Token::Star => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    node = Expr::mul(node, rhs);
                }
                Token::Slash => {
                    self.next();
                    let rhs = self.parse_unary()?;
                    node = Expr::div(node, rhs);
                }
                _ => break,
            }
        }
        Ok(node)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let Some(Token::Minus) = self.peek() {
            self.next();
            let inner = self.parse_unary()?;
            return Ok(Expr::Neg(Box::new(inner)));
        }
        if let Some(Token::Plus) = self.peek() {
            self.next();
            return self.parse_unary();
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_primary()?;
        if let Some(Token::Pow) = self.peek() {
            self.next();
            let exp = self.parse_unary()?;
            Ok(Expr::pow(base, exp))
        } else {
            Ok(base)
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.next() {
            Some(Token::Number(val)) => Ok(Expr::constant(*val)),
            Some(Token::Ident(name)) => {
                if let Some(Token::LParen) = self.peek() {
                    self.next(); // '('
                    let arg = self.parse_expr()?;
                    if self.next() != Some(&Token::RParen) {
                        return Err(format!("Fonksiyon çağrısı '{}' için ')' eksik!", name));
                    }
                    match name.as_str() {
                        "sqrt" => Ok(Expr::sqrt(arg)),
                        "log" | "ln" => Ok(Expr::ln(arg)),
                        "exp" => Ok(Expr::exp(arg)),
                        other => Err(format!("Desteklenmeyen fonksiyon: '{}'", other)),
                    }
                } else {
                    Ok(Expr::var(name))
                }
            }
            Some(Token::LParen) => {
                let inner = self.parse_expr()?;
                if self.next() != Some(&Token::RParen) {
                    return Err("Kapanmayan parantez '('!".to_string());
                }
                Ok(inner)
            }
            other => Err(format!("Beklenmeyen ifade başlangıcı: {:?}", other)),
        }
    }
}

impl Add for Expr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Expr::Add(Box::new(self), Box::new(rhs))
    }
}

impl Sub for Expr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Expr::Sub(Box::new(self), Box::new(rhs))
    }
}

impl Mul for Expr {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Expr::Mul(Box::new(self), Box::new(rhs))
    }
}

impl Div for Expr {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Expr::Div(Box::new(self), Box::new(rhs))
    }
}

impl Neg for Expr {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Expr::Neg(Box::new(self))
    }
}

impl Expr {
    pub fn constant(val: f64) -> Self { Expr::Constant(val) }
    pub fn var(name: &str) -> Self { Expr::Variable(name.to_string()) }

    pub fn pow(base: Expr, exp: Expr) -> Self { Expr::Pow(Box::new(base), Box::new(exp)) }
    pub fn sqrt(e: Expr) -> Self { Expr::Sqrt(Box::new(e)) }
    pub fn ln(e: Expr) -> Self { Expr::Ln(Box::new(e)) }
    pub fn exp(e: Expr) -> Self { Expr::Exp(Box::new(e)) }

    /// Metin tabanlı bir matematiksel ifadeyi (veya 'LHS = RHS' denkleminin sağ tarafını)
    /// özyinelemeli iniş (recursive-descent) ile doğrudan sembolik AST'ye ayrıştırır.
    pub fn parse_str(input: &str) -> Result<Self, String> {
        let rhs_str = if let Some((_, rhs)) = input.split_once('=') {
            rhs.trim()
        } else {
            input.trim()
        };
        let tokens = tokenize_expr(rhs_str)?;
        if tokens.is_empty() {
            return Err("Boş matematiksel ifade!".to_string());
        }
        let mut parser = ExprParser::new(&tokens);
        let expr = parser.parse_expr()?;
        if parser.pos < tokens.len() {
            return Err(format!("İfade sonunda fazladan token: {:?}", &tokens[parser.pos..]));
        }
        Ok(expr)
    }

    /// İfade ağacının (AST) fiziksel boyut vektörünü Z^7 üzerinde cebirsel olarak çıkarır.
    pub fn infer_dimension(&self, var_dims: &HashMap<String, Dimension>) -> Result<Dimension, String> {
        match self {
            Expr::Constant(_) => Ok(Dimension::dimensionless()),
            Expr::Variable(name) => var_dims
                .get(name.as_str())
                .copied()
                .ok_or_else(|| format!("Değişken '{}' için boyut tanımı bulunamadı!", name)),
            Expr::Add(l, r) | Expr::Sub(l, r) => {
                let dl = l.infer_dimension(var_dims)?;
                let dr = r.infer_dimension(var_dims)?;
                if dl != dr {
                    Err(format!(
                        "Toplamsal boyut homojenliği ihlali ({}): Sol [{}] != Sağ [{}]",
                        self, dl, dr
                    ))
                } else {
                    Ok(dl)
                }
            }
            Expr::Mul(l, r) => {
                let dl = l.infer_dimension(var_dims)?;
                let dr = r.infer_dimension(var_dims)?;
                dl.checked_mul_dim(&dr)
            }
            Expr::Div(l, r) => {
                let dl = l.infer_dimension(var_dims)?;
                let dr = r.infer_dimension(var_dims)?;
                dl.checked_div_dim(&dr)
            }
            Expr::Pow(base, exp) => {
                let db = base.infer_dimension(var_dims)?;
                let de = exp.infer_dimension(var_dims)?;
                if !de.is_dimensionless() {
                    return Err(format!("Üs ifadesi ({}) boyutsuz olmalıdır, bulunan: [{}]", exp, de));
                }
                if db.is_dimensionless() {
                    return Ok(Dimension::dimensionless());
                }
                let simplified_exp = exp.simplify();
                match &simplified_exp {
                    Expr::Constant(p) if p.is_finite() => {
                        for q in 1_i32..=24_i32 {
                            let scaled = *p * f64::from(q);
                            if scaled.is_finite()
                                && (scaled - scaled.round()).abs() < 1e-9
                                && scaled.abs() <= 10_000.0
                            {
                                return db.rational_pow(scaled.round() as i32, q);
                            }
                        }
                        Err(format!(
                            "Boyutlu tabanın ({}) üssü sabit tamsayı/rasyonel olmalıdır: {}",
                            base, exp
                        ))
                    }
                    _ => Err(format!(
                        "Boyutlu tabanın ({}) üssü sabit tamsayı/rasyonel olmalıdır: {}",
                        base, exp
                    )),
                }
            }
            Expr::Sqrt(inner) => {
                let di = inner.infer_dimension(var_dims)?;
                di.sqrt_dim()
            }
            Expr::Ln(inner) => {
                let di = inner.infer_dimension(var_dims)?;
                if !di.is_dimensionless() {
                    return Err(format!("ln({}) argümanı boyutsuz olmalıdır, bulunan: [{}]", inner, di));
                }
                Ok(Dimension::dimensionless())
            }
            Expr::Exp(inner) => {
                let di = inner.infer_dimension(var_dims)?;
                if !di.is_dimensionless() {
                    return Err(format!("exp({}) argümanı boyutsuz olmalıdır, bulunan: [{}]", inner, di));
                }
                Ok(Dimension::dimensionless())
            }
            Expr::Neg(inner) => inner.infer_dimension(var_dims),
        }
    }

    /// İfade içindeki tüm serbest değişken adlarını döndürür.
    pub fn free_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_vars(&mut vars);
        vars
    }

    fn collect_vars(&self, out: &mut Vec<String>) {
        match self {
            Expr::Constant(_) => {}
            Expr::Variable(v) => {
                if !out.contains(v) {
                    out.push(v.clone());
                }
            }
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) | Expr::Pow(l, r) => {
                l.collect_vars(out);
                r.collect_vars(out);
            }
            Expr::Sqrt(e) | Expr::Ln(e) | Expr::Exp(e) | Expr::Neg(e) => e.collect_vars(out),
        }
    }

    /// Verilen değişkenin ifade içinde geçip geçmediğini döndürür.
    pub fn contains_var(&self, var_name: &str) -> bool {
        match self {
            Expr::Constant(_) => false,
            Expr::Variable(v) => v == var_name,
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) | Expr::Pow(l, r) => {
                l.contains_var(var_name) || r.contains_var(var_name)
            }
            Expr::Sqrt(e) | Expr::Ln(e) | Expr::Exp(e) | Expr::Neg(e) => e.contains_var(var_name),
        }
    }

    /// Verilen değişkenin ifade ağacında kaç kez yaprak olarak geçtiğini sayar.
    pub fn var_occurrences(&self, var_name: &str) -> usize {
        match self {
            Expr::Constant(_) => 0,
            Expr::Variable(v) => usize::from(v == var_name),
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) | Expr::Div(l, r) | Expr::Pow(l, r) => {
                l.var_occurrences(var_name) + r.var_occurrences(var_name)
            }
            Expr::Sqrt(e) | Expr::Ln(e) | Expr::Exp(e) | Expr::Neg(e) => e.var_occurrences(var_name),
        }
    }

    /// İfadenin tanım kümesi kısıtı (paydada değişken, karekök, logaritma, tam olmayan üs veya negatif üs)
    /// içerip içermediğini denetler.
    /// Yalnızca `is_domain_total() == true` olan ifadeler `X - X -> 0`, `X / X -> 1`, `0 * X -> 0` ile sadeleştirilebilir (§5.2).
    pub fn is_domain_total(&self) -> bool {
        match self {
            Expr::Constant(c) => c.is_finite(),
            Expr::Variable(_) => true,
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) => {
                l.is_domain_total() && r.is_domain_total()
            }
            Expr::Div(l, r) => {
                let sr = r.simplify();
                l.is_domain_total() && matches!(sr, Expr::Constant(c) if c.is_finite() && c.abs() > 1e-15)
            }
            Expr::Pow(b, e) => {
                let se = e.simplify();
                b.is_domain_total()
                    && e.is_domain_total()
                    && matches!(se, Expr::Constant(p) if p.is_finite() && p > 0.0 && (p - p.round()).abs() < 1e-12)
            }
            Expr::Exp(inner) | Expr::Neg(inner) => inner.is_domain_total(),
            Expr::Sqrt(_) | Expr::Ln(_) => false,
        }
    }

    /// §5.2: Sembolik ifade ağacındaki tüm tanım kümesi / yan koşulları (payda != 0, ln içi > 0, karekök içi >= 0,
    /// tam olmayan ve negatif üs kısıtları) özyinelemeli olarak çıkarır ve sabit ihlallerini derhal reddeder.
    pub fn extract_domain_conditions(&self) -> Result<Vec<String>, String> {
        let mut conds = Vec::new();
        self.collect_domain_conditions(&mut conds)?;
        Ok(conds)
    }

    fn push_unique_guard(out: &mut Vec<String>, guard: String) {
        if !out.contains(&guard) {
            out.push(guard);
        }
    }

    fn collect_domain_conditions(&self, out: &mut Vec<String>) -> Result<(), String> {
        match self {
            Expr::Constant(c) => {
                if !c.is_finite() {
                    return Err(format!("Tanım kümesi ihlali: sonlu olmayan sabit ({})", c));
                }
                Ok(())
            }
            Expr::Variable(_) => Ok(()),
            Expr::Add(l, r) | Expr::Sub(l, r) | Expr::Mul(l, r) => {
                l.collect_domain_conditions(out)?;
                r.collect_domain_conditions(out)
            }
            Expr::Pow(l, r) => {
                l.collect_domain_conditions(out)?;
                r.collect_domain_conditions(out)?;
                let sl = l.simplify();
                let sr = r.simplify();
                match &sr {
                    Expr::Constant(p) => {
                        if !p.is_finite() {
                            return Err(format!("Tanım kümesi ihlali: ({}) sonlu olmayan üs!", self));
                        }
                        let is_int = (*p - p.round()).abs() < 1e-12;
                        if is_int {
                            if *p < 0.0 {
                                if let Expr::Constant(c) = sl {
                                    if c.abs() < 1e-15 {
                                        return Err(format!(
                                            "Tanım kümesi ihlali: ({}) negatif üste sıfır taban!",
                                            self
                                        ));
                                    }
                                } else {
                                    Self::push_unique_guard(out, format!("{} != 0", sl));
                                }
                            } else if p.abs() <= 1e-12 {
                                if let Expr::Constant(c) = sl {
                                    if c.abs() < 1e-15 {
                                        return Err(format!(
                                            "Tanım kümesi ihlali: ({}) belirsiz 0^0 biçimi!",
                                            self
                                        ));
                                    }
                                }
                            }
                        } else if *p > 0.0 {
                            // Tam olmayan pozitif gerçel üs (örn: u^0.5, u^1.4) R üzerinde u >= 0 gerektirir
                            if let Expr::Constant(c) = sl {
                                if c < 0.0 {
                                    return Err(format!(
                                        "Tanım kümesi ihlali: ({}) negatif tabanın tam olmayan üssü R üzerinde tanımsızdır!",
                                        self
                                    ));
                                }
                            } else {
                                Self::push_unique_guard(out, format!("{} >= 0", sl));
                            }
                        } else {
                            // Tam olmayan negatif gerçel üs (örn: u^(-0.5)) R üzerinde u > 0 gerektirir
                            if let Expr::Constant(c) = sl {
                                if c <= 0.0 {
                                    return Err(format!(
                                        "Tanım kümesi ihlali: ({}) pozitif olmayan tabanın negatif kesirli üssü tanımsızdır!",
                                        self
                                    ));
                                }
                            } else {
                                Self::push_unique_guard(out, format!("{} > 0", sl));
                            }
                        }
                    }
                    _ => {
                        // Değişken üs: u^v = exp(v * ln(u)) R üzerinde u > 0 gerektirir
                        if let Expr::Constant(c) = sl {
                            if c <= 0.0 {
                                return Err(format!(
                                    "Tanım kümesi ihlali: ({}) değişken üslü ifadede taban > 0 olmalıdır!",
                                    self
                                ));
                            }
                        } else {
                            Self::push_unique_guard(out, format!("{} > 0", sl));
                        }
                    }
                }
                Ok(())
            }
            Expr::Div(l, r) => {
                l.collect_domain_conditions(out)?;
                r.collect_domain_conditions(out)?;
                let sr = r.simplify();
                if let Expr::Constant(c) = sr {
                    if c.abs() < 1e-15 {
                        return Err(format!("Tanım kümesi ihlali: ({}) ifadesinde sıfıra bölme!", self));
                    }
                } else {
                    Self::push_unique_guard(out, format!("{} != 0", sr));
                }
                Ok(())
            }
            Expr::Sqrt(inner) => {
                inner.collect_domain_conditions(out)?;
                let si = inner.simplify();
                if let Expr::Constant(c) = si {
                    if c < 0.0 {
                        return Err(format!("Tanım kümesi ihlali: sqrt({}) içinde negatif sabit!", c));
                    }
                } else {
                    Self::push_unique_guard(out, format!("{} >= 0", si));
                }
                Ok(())
            }
            Expr::Ln(inner) => {
                inner.collect_domain_conditions(out)?;
                let si = inner.simplify();
                if let Expr::Constant(c) = si {
                    if c <= 0.0 {
                        return Err(format!("Tanım kümesi ihlali: ln({}) argümanı pozitif olmalıdır!", c));
                    }
                } else {
                    Self::push_unique_guard(out, format!("{} > 0", si));
                }
                Ok(())
            }
            Expr::Exp(inner) | Expr::Neg(inner) => inner.collect_domain_conditions(out),
        }
    }

    /// Verilen değişken değerleriyle sembolik ifadeyi R üzerinde tam tanım kümesi ve sonluluk
    /// garantisiyle sayısal olarak değerlendirir (`Ok(NaN)` veya `Ok(±Inf)` sızıntısı imkânsızdır).
    pub fn evaluate(&self, bindings: &HashMap<&str, f64>) -> Result<f64, String> {
        let res = match self {
            Expr::Constant(val) => {
                if !val.is_finite() {
                    return Err(format!("Sonlu olmayan sabit değerlendirilemez: {}", val));
                }
                *val
            }
            Expr::Variable(name) => {
                let val = bindings
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| format!("Değişken değeri bulunamadı: {}", name))?;
                if !val.is_finite() {
                    return Err(format!("Değişken '{}' sonlu bir gerçel sayı olmalıdır: {}", name, val));
                }
                val
            }
            Expr::Add(l, r) => l.evaluate(bindings)? + r.evaluate(bindings)?,
            Expr::Sub(l, r) => l.evaluate(bindings)? - r.evaluate(bindings)?,
            Expr::Mul(l, r) => l.evaluate(bindings)? * r.evaluate(bindings)?,
            Expr::Div(l, r) => {
                let num = l.evaluate(bindings)?;
                let denom = r.evaluate(bindings)?;
                if denom == 0.0 {
                    return Err("Sıfıra bölme hatası!".to_string());
                }
                num / denom
            }
            Expr::Pow(base, exp) => {
                let b = base.evaluate(bindings)?;
                let e = exp.evaluate(bindings)?;
                if (e - e.round()).abs() == 0.0 && e.abs() <= f64::from(i32::MAX) {
                    let k = e.round() as i32;
                    if k == 0 {
                        if b == 0.0 {
                            return Err("Belirsiz biçim: 0^0 gerçel analizde tanımsızdır!".to_string());
                        }
                        1.0
                    } else if k < 0 && b == 0.0 {
                        return Err("Tanım kümesi ihlali: sıfırın negatif üssü (sıfıra bölme)!".to_string());
                    } else {
                        b.powi(k)
                    }
                } else if b < 0.0 {
                    return Err(format!(
                        "Tanım kümesi ihlali: negatif tabanın ({}) tam olmayan üssü ({}) gerçel sayılarda tanımsızdır!",
                        b, e
                    ));
                } else if b == 0.0 {
                    if e > 0.0 {
                        0.0
                    } else {
                        return Err(format!(
                            "Tanım kümesi ihlali: sıfırın pozitif olmayan üssü (0^{}) tanımsızdır!",
                            e
                        ));
                    }
                } else {
                    b.powf(e)
                }
            }
            Expr::Sqrt(e) => {
                let v = e.evaluate(bindings)?;
                if v < 0.0 {
                    return Err("Karekök içi negatif olamaz!".to_string());
                }
                v.sqrt()
            }
            Expr::Ln(e) => {
                let v = e.evaluate(bindings)?;
                if v <= 0.0 {
                    return Err("Logaritma argümanı pozitif olmalıdır!".to_string());
                }
                v.ln()
            }
            Expr::Exp(e) => e.evaluate(bindings)?.exp(),
            Expr::Neg(e) => -e.evaluate(bindings)?,
        };
        if !res.is_finite() {
            return Err(format!("Aritmetik taşma veya sonlu olmayan sonuç ({})!", self));
        }
        Ok(res)
    }

    /// Faz 3: IEEE-754 dışa yuvarlamalı (outward-rounded) doğrudan aralık değerlendirmesi.
    /// §5.2 Uyumu: `l == r` kısayolları yalnızca her iki alt ifade de tanım kümesi denetiminden geçtikten sonra uygulanır!
    pub fn eval_interval(&self, bindings: &HashMap<&str, Interval64>) -> Result<Interval64, String> {
        let iv = match self {
            Expr::Constant(val) => Interval64::point(*val)?,
            Expr::Variable(name) => {
                let b = bindings
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| format!("Aralık değişkeni bulunamadı: {}", name))?;
                Interval64::new(b.lo, b.hi)?
            }
            Expr::Add(l, r) => l.eval_interval(bindings)?.add(&r.eval_interval(bindings)?),
            Expr::Sub(l, r) => {
                let l_iv = l.eval_interval(bindings)?;
                let r_iv = r.eval_interval(bindings)?;
                if l == r {
                    return Interval64::point(0.0);
                }
                l_iv.sub(&r_iv)
            }
            Expr::Mul(l, r) => {
                let l_iv = l.eval_interval(bindings)?;
                let r_iv = r.eval_interval(bindings)?;
                if l == r {
                    l_iv.sqr()
                } else {
                    l_iv.mul(&r_iv)
                }
            }
            Expr::Div(l, r) => {
                let num = l.eval_interval(bindings)?;
                let denom = r.eval_interval(bindings)?;
                let q = num.div(&denom)?;
                if l == r {
                    return Interval64::point(1.0);
                }
                q
            }
            Expr::Pow(base, exp) => {
                let b_iv = base.eval_interval(bindings)?;
                let s_exp = exp.simplify();
                if let Expr::Constant(p) = &s_exp {
                    if p.is_finite() && (*p - p.round()).abs() < 1e-12 && p.abs() <= 64.0 {
                        return b_iv.powi(p.round() as i32);
                    }
                }
                let e_iv = exp.eval_interval(bindings)?;
                b_iv.pow(&e_iv)?
            }
            Expr::Sqrt(e) => e.eval_interval(bindings)?.sqrt()?,
            Expr::Ln(e) => e.eval_interval(bindings)?.ln()?,
            Expr::Exp(e) => e.eval_interval(bindings)?.exp()?,
            Expr::Neg(e) => e.eval_interval(bindings)?.neg(),
        };
        Interval64::new(iv.lo, iv.hi)
    }

    /// Faz 3: İleri Mod Otomatik Türev (Dual64) ile verilen değişkene göre tam makine hassasiyetinde
    /// fonksiyon değeri ve kısmi türev (f(x), ∂f/∂var) hesaplar.
    pub fn eval_dual(&self, bindings: &HashMap<&str, f64>, diff_var: &str) -> Result<Dual64, String> {
        let d = match self {
            Expr::Constant(c) => {
                if !c.is_finite() {
                    return Err(format!("Dual64 sonlu olmayan sabit: {}", c));
                }
                Dual64::constant(*c)
            }
            Expr::Variable(name) => {
                let val = bindings
                    .get(name.as_str())
                    .copied()
                    .ok_or_else(|| format!("Dual64 değişken değeri bulunamadı: {}", name))?;
                if !val.is_finite() {
                    return Err(format!("Dual64 değişken '{}' sonlu bir gerçel sayı olmalıdır!", name));
                }
                if name == diff_var {
                    Dual64::active_variable(val)
                } else {
                    Dual64::constant(val)
                }
            }
            Expr::Add(l, r) => l.eval_dual(bindings, diff_var)?.add(r.eval_dual(bindings, diff_var)?),
            Expr::Sub(l, r) => l.eval_dual(bindings, diff_var)?.sub(r.eval_dual(bindings, diff_var)?),
            Expr::Mul(l, r) => l.eval_dual(bindings, diff_var)?.mul(r.eval_dual(bindings, diff_var)?),
            Expr::Div(l, r) => l.eval_dual(bindings, diff_var)?.checked_div(r.eval_dual(bindings, diff_var)?)?,
            Expr::Pow(b, e) => b.eval_dual(bindings, diff_var)?.pow(e.eval_dual(bindings, diff_var)?)?,
            Expr::Sqrt(e) => e.eval_dual(bindings, diff_var)?.sqrt()?,
            Expr::Ln(e) => e.eval_dual(bindings, diff_var)?.ln()?,
            Expr::Exp(e) => e.eval_dual(bindings, diff_var)?.exp(),
            Expr::Neg(e) => e.eval_dual(bindings, diff_var)?.neg(),
        };
        if !d.val.is_finite() || !d.der.is_finite() {
            return Err(format!("Dual64 aritmetik taşma veya tanımsız türev ({})!", self));
        }
        Ok(d)
    }

    /// İfadenin tüm serbest değişkenlerine göre tam gradyan vektörünü (∇f) Dual64 ile hesaplar.
    pub fn gradient(&self, bindings: &HashMap<&str, f64>) -> Result<(f64, HashMap<String, f64>), String> {
        let vars = self.free_variables();
        let mut grad = HashMap::new();
        let mut f_val = self.evaluate(bindings)?;
        for v in vars {
            let d = self.eval_dual(bindings, &v)?;
            f_val = d.val;
            grad.insert(v, d.der);
        }
        Ok((f_val, grad))
    }

    /// Faz 3: 1. Derece Taylor / Ortalama Değer Biçimi (Mean-Value Form) ile daraltılmış
    /// bağımlılık-duyarlı garantili aralık kapanımı:
    /// f(X) ⊆ I_naive ∩ [ f(x_0) + ∑ (∂f/∂x_i)(X) · (X_i - x_{0,i}) ]
    pub fn eval_taylor_interval(&self, bindings: &HashMap<&str, Interval64>) -> Result<Interval64, String> {
        let naive_iv = self.eval_interval(bindings)?;
        let vars = self.free_variables();
        if vars.is_empty() {
            return Ok(naive_iv);
        }

        // Merkez noktası x_0'ı aralık olarak oluştur (yuvarlama hatasını kapsamak için [m, m])
        let mut mid_bindings: HashMap<&str, Interval64> = HashMap::new();
        for (&k, iv) in bindings {
            mid_bindings.insert(k, Interval64::point(iv.midpoint())?);
        }

        let mut mv_enclosure = self.eval_interval(&mid_bindings)?;
        for var in &vars {
            let sym_der = self.differentiate(var)?;
            let der_iv = sym_der.eval_interval(bindings)?;
            let x_iv = bindings
                .get(var.as_str())
                .ok_or_else(|| format!("Taylor aralığı için değişken '{}' eksik!", var))?;
            let x0_iv = mid_bindings[var.as_str()];
            let delta_x = x_iv.sub(&x0_iv);
            mv_enclosure = mv_enclosure.add(&der_iv.mul(&delta_x));
        }

        Ok(naive_iv.intersect(&mv_enclosure).unwrap_or(naive_iv))
    }

    /// Faz 3: Verilen değişkene göre tam sembolik kısmi türev (∂Expr / ∂var) alır ve sadeleştirir.
    pub fn differentiate(&self, var: &str) -> Result<Expr, String> {
        if !self.contains_var(var) {
            return Ok(Expr::Constant(0.0));
        }
        let raw_deriv = match self {
            Expr::Constant(_) => Expr::Constant(0.0),
            Expr::Variable(name) => {
                if name == var {
                    Expr::Constant(1.0)
                } else {
                    Expr::Constant(0.0)
                }
            }
            Expr::Add(u, v) => Expr::add(u.differentiate(var)?, v.differentiate(var)?),
            Expr::Sub(u, v) => Expr::sub(u.differentiate(var)?, v.differentiate(var)?),
            Expr::Mul(u, v) => {
                // d(u*v) = u'*v + u*v' (0 çarpanı üreterek yan koşulları yutmamak için sıfır dallarını doğrudan ayır)
                let du = u.differentiate(var)?;
                let dv = v.differentiate(var)?;
                match (du == Expr::Constant(0.0), dv == Expr::Constant(0.0)) {
                    (true, true) => Expr::Constant(0.0),
                    (true, false) => Expr::mul((**u).clone(), dv),
                    (false, true) => Expr::mul(du, (**v).clone()),
                    (false, false) => Expr::add(Expr::mul(du, (**v).clone()), Expr::mul((**u).clone(), dv)),
                }
            }
            Expr::Div(u, v) => {
                // d(u/v) = (u'*v - u*v') / v^2
                let du = u.differentiate(var)?;
                let dv = v.differentiate(var)?;
                match (du == Expr::Constant(0.0), dv == Expr::Constant(0.0)) {
                    (true, true) => Expr::Constant(0.0),
                    (false, true) => Expr::div(du, (**v).clone()),
                    (true, false) => {
                        let num = Expr::mul((**u).clone(), dv);
                        let den = Expr::pow((**v).clone(), Expr::Constant(2.0));
                        Expr::neg(Expr::div(num, den))
                    }
                    (false, false) => {
                        let num = Expr::sub(Expr::mul(du, (**v).clone()), Expr::mul((**u).clone(), dv));
                        let den = Expr::pow((**v).clone(), Expr::Constant(2.0));
                        Expr::div(num, den)
                    }
                }
            }
            Expr::Pow(u, v) => {
                let du = u.differentiate(var)?;
                let dv = v.differentiate(var)?;
                match (du == Expr::Constant(0.0), dv == Expr::Constant(0.0)) {
                    (true, true) => Expr::Constant(0.0),
                    (false, true) => {
                        // d(u^c) = c * u^(c - 1) * u'
                        let sv = (**v).simplify();
                        if matches!(sv, Expr::Constant(c) if (c - 1.0).abs() < 1e-15) {
                            du
                        } else {
                            let exp_minus_1 = Expr::sub(sv.clone(), Expr::Constant(1.0)).simplify();
                            Expr::mul(
                                Expr::mul(sv, Expr::pow((**u).clone(), exp_minus_1)),
                                du,
                            )
                        }
                    }
                    (true, false) => {
                        // d(c^v) = c^v * ln(c) * v'
                        Expr::mul(Expr::mul(self.clone(), Expr::ln((**u).clone())), dv)
                    }
                    (false, false) => {
                        // d(u^v) = u^v * (v' * ln(u) + v * u' / u)
                        let term1 = Expr::mul(dv, Expr::ln((**u).clone()));
                        let term2 = Expr::div(Expr::mul((**v).clone(), du), (**u).clone());
                        Expr::mul(self.clone(), Expr::add(term1, term2))
                    }
                }
            }
            Expr::Sqrt(u) => {
                // d(sqrt(u)) = u' / (2 * sqrt(u))
                let du = u.differentiate(var)?;
                Expr::div(du, Expr::mul(Expr::Constant(2.0), Expr::sqrt((**u).clone())))
            }
            Expr::Ln(u) => {
                // d(ln(u)) = u' / u
                let du = u.differentiate(var)?;
                Expr::div(du, (**u).clone())
            }
            Expr::Exp(u) => {
                // d(exp(u)) = exp(u) * u'
                let du = u.differentiate(var)?;
                Expr::mul(Expr::exp((**u).clone()), du)
            }
            Expr::Neg(u) => Expr::neg(u.differentiate(var)?),
        };
        Ok(raw_deriv.simplify())
    }

    /// Bir değişkenin yerine başka bir sembolik ifade koyar (Sembolik İkame / Substitution).
    pub fn substitute(&self, var_name: &str, replacement: &Expr) -> Expr {
        match self {
            Expr::Constant(c) => Expr::Constant(*c),
            Expr::Variable(name) => {
                if name == var_name {
                    replacement.clone()
                } else {
                    Expr::Variable(name.clone())
                }
            }
            Expr::Add(l, r) => Expr::Add(
                Box::new(l.substitute(var_name, replacement)),
                Box::new(r.substitute(var_name, replacement)),
            ),
            Expr::Sub(l, r) => Expr::Sub(
                Box::new(l.substitute(var_name, replacement)),
                Box::new(r.substitute(var_name, replacement)),
            ),
            Expr::Mul(l, r) => Expr::Mul(
                Box::new(l.substitute(var_name, replacement)),
                Box::new(r.substitute(var_name, replacement)),
            ),
            Expr::Div(l, r) => Expr::Div(
                Box::new(l.substitute(var_name, replacement)),
                Box::new(r.substitute(var_name, replacement)),
            ),
            Expr::Pow(b, e) => Expr::Pow(
                Box::new(b.substitute(var_name, replacement)),
                Box::new(e.substitute(var_name, replacement)),
            ),
            Expr::Sqrt(e) => Expr::Sqrt(Box::new(e.substitute(var_name, replacement))),
            Expr::Ln(e) => Expr::Ln(Box::new(e.substitute(var_name, replacement))),
            Expr::Exp(e) => Expr::Exp(Box::new(e.substitute(var_name, replacement))),
            Expr::Neg(e) => Expr::Neg(Box::new(e.substitute(var_name, replacement))),
        }
    }

    /// Sabit katlama ve kanonik cebirsel sadeleştirme (Multi-Pass Algebraic Simplifier).
    pub fn simplify(&self) -> Expr {
        let mut curr = self.simplify_once();
        for _ in 0..4 {
            let next = curr.simplify_once();
            if next == curr {
                break;
            }
            curr = next;
        }
        curr
    }

    fn fold_const_pow(a: f64, p: f64) -> Option<f64> {
        if !a.is_finite() || !p.is_finite() {
            return None;
        }
        let val = if (p - p.round()).abs() == 0.0 && p.abs() <= f64::from(i32::MAX) {
            let k = p.round() as i32;
            if k == 0 {
                if a == 0.0 {
                    return None;
                }
                1.0
            } else if k < 0 && a == 0.0 {
                return None;
            } else {
                a.powi(k)
            }
        } else if a < 0.0 || (a == 0.0 && p <= 0.0) {
            return None;
        } else if a == 0.0 {
            0.0
        } else {
            a.powf(p)
        };
        val.is_finite().then_some(val)
    }

    fn simplify_once(&self) -> Expr {
        match self {
            Expr::Add(l, r) => {
                let sl = l.simplify_once();
                let sr = r.simplify_once();
                match (&sl, &sr) {
                    (Expr::Constant(a), _) if a.is_finite() && a.abs() < 1e-15 => sr,
                    (_, Expr::Constant(b)) if b.is_finite() && b.abs() < 1e-15 => sl,
                    (Expr::Constant(a), Expr::Constant(b)) if (*a + *b).is_finite() => {
                        Expr::Constant(a + b)
                    }
                    (_, Expr::Neg(inner)) => Expr::sub(sl, (**inner).clone()),
                    _ if sl == sr => Expr::mul(Expr::Constant(2.0), sl),
                    _ => Expr::Add(Box::new(sl), Box::new(sr)),
                }
            }
            Expr::Mul(l, r) => {
                let sl = l.simplify_once();
                let sr = r.simplify_once();
                match (&sl, &sr) {
                    (Expr::Constant(a), _) if a.is_finite() && a.abs() < 1e-15 && sr.is_domain_total() => {
                        Expr::Constant(0.0)
                    }
                    (_, Expr::Constant(b)) if b.is_finite() && b.abs() < 1e-15 && sl.is_domain_total() => {
                        Expr::Constant(0.0)
                    }
                    (Expr::Constant(a), _) if a.is_finite() && (*a - 1.0).abs() < 1e-15 => sr,
                    (_, Expr::Constant(b)) if b.is_finite() && (*b - 1.0).abs() < 1e-15 => sl,
                    (Expr::Constant(a), _) if a.is_finite() && (*a + 1.0).abs() < 1e-15 => Expr::neg(sr),
                    (_, Expr::Constant(b)) if b.is_finite() && (*b + 1.0).abs() < 1e-15 => Expr::neg(sl),
                    (Expr::Constant(a), Expr::Constant(b)) if (*a * *b).is_finite() => {
                        Expr::Constant(a * b)
                    }
                    // (a * u) * u -> a * u^2
                    (Expr::Mul(a, u), v) if **u == *v => {
                        Expr::mul((**a).clone(), Expr::pow((**u).clone(), Expr::Constant(2.0)))
                    }
                    // (a * b) * c sabit birleştirme
                    (Expr::Constant(c1), Expr::Mul(c2, rest)) if matches!(**c2, Expr::Constant(_)) => {
                        if let Expr::Constant(v2) = **c2 {
                            if (c1 * v2).is_finite() {
                                Expr::mul(Expr::Constant(c1 * v2), (**rest).clone())
                            } else {
                                Expr::Mul(Box::new(sl), Box::new(sr))
                            }
                        } else {
                            Expr::Mul(Box::new(sl), Box::new(sr))
                        }
                    }
                    _ if sl == sr => Expr::pow(sl, Expr::Constant(2.0)),
                    _ => Expr::Mul(Box::new(sl), Box::new(sr)),
                }
            }
            Expr::Sub(l, r) => {
                let sl = l.simplify_once();
                let sr = r.simplify_once();
                match (&sl, &sr) {
                    (_, Expr::Constant(b)) if b.is_finite() && b.abs() < 1e-15 => sl,
                    (Expr::Constant(a), _) if a.is_finite() && a.abs() < 1e-15 => Expr::neg(sr),
                    (Expr::Constant(a), Expr::Constant(b)) if (*a - *b).is_finite() => {
                        Expr::Constant(a - b)
                    }
                    (_, Expr::Neg(inner)) => Expr::add(sl, (**inner).clone()),
                    _ if sl == sr && sl.is_domain_total() => Expr::Constant(0.0),
                    _ => Expr::Sub(Box::new(sl), Box::new(sr)),
                }
            }
            Expr::Div(l, r) => {
                let sl = l.simplify_once();
                let sr = r.simplify_once();
                match (&sl, &sr) {
                    (Expr::Constant(a), Expr::Constant(b))
                        if b.is_finite() && b.abs() > 1e-15 && (*a / *b).is_finite() =>
                    {
                        Expr::Constant(a / b)
                    }
                    (Expr::Constant(a), _)
                        if a.is_finite()
                            && a.abs() < 1e-15
                            && sr.is_domain_total()
                            && !matches!(sr, Expr::Constant(_)) =>
                    {
                        Expr::Div(Box::new(sl), Box::new(sr))
                    }
                    (_, Expr::Constant(b)) if b.is_finite() && (*b - 1.0).abs() < 1e-15 => sl,
                    // §5.2: Payda değişken içeriyorsa u / u -> 1 sadeleştirmesi yan koşulu (u != 0) sileceği için yasaktır!
                    _ => Expr::Div(Box::new(sl), Box::new(sr)),
                }
            }
            Expr::Pow(b, e) => {
                let sb = b.simplify_once();
                let se = e.simplify_once();
                match (&sb, &se) {
                    (Expr::Constant(a), Expr::Constant(p)) => {
                        if let Some(folded) = Self::fold_const_pow(*a, *p) {
                            Expr::Constant(folded)
                        } else {
                            Expr::Pow(Box::new(sb), Box::new(se))
                        }
                    }
                    (_, Expr::Constant(p)) if p.is_finite() && (*p - 1.0).abs() < 1e-15 => sb,
                    // §5.2: (u^p1)^p2 -> u^(p1*p2) YALNIZCA p1 ve p2 sıfır olmayan tamsayı olduğunda ve ikisi birden negatif
                    // olmadığında R üzerinde özdeştir! Aksi halde (x^2)^0.5 = |x| != x ve (x^0.5)^2 (x >= 0 yutulması) hatası oluşur!
                    (Expr::Pow(inner_b, inner_e), Expr::Constant(p2)) => {
                        if let Expr::Constant(p1) = **inner_e {
                            let p1_int = p1.is_finite() && (p1 - p1.round()).abs() < 1e-12 && p1.abs() > 0.5;
                            let p2_int = p2.is_finite() && (*p2 - p2.round()).abs() < 1e-12 && p2.abs() > 0.5;
                            let both_neg = p1 < 0.0 && *p2 < 0.0;
                            if p1_int && p2_int && !both_neg && (p1 * *p2).is_finite() {
                                Expr::pow((**inner_b).clone(), Expr::Constant(p1 * *p2))
                            } else {
                                Expr::Pow(Box::new(sb), Box::new(se))
                            }
                        } else {
                            Expr::Pow(Box::new(sb), Box::new(se))
                        }
                    }
                    // §5.2: sqrt(u)^2 -> u dönüşümü u >= 0 yan koşulunu sileceği için yasaktır!
                    _ => Expr::Pow(Box::new(sb), Box::new(se)),
                }
            }
            Expr::Sqrt(e) => {
                let se = e.simplify_once();
                if let Expr::Constant(v) = se {
                    if v.is_finite() && v >= 0.0 {
                        return Expr::Constant(v.sqrt());
                    }
                }
                Expr::Sqrt(Box::new(se))
            }
            Expr::Ln(e) => {
                let se = e.simplify_once();
                match &se {
                    Expr::Constant(v) if v.is_finite() && *v > 0.0 => Expr::Constant(v.ln()),
                    // ln(exp(u)) == u tüm gerçel u için geçerlidir (exp(u) > 0 daima sağlanır ve u kendi yan koşullarını korur)
                    Expr::Exp(inner) => (**inner).clone(),
                    _ => Expr::Ln(Box::new(se)),
                }
            }
            Expr::Exp(e) => {
                let se = e.simplify_once();
                match &se {
                    Expr::Constant(v) if v.is_finite() && v.exp().is_finite() => Expr::Constant(v.exp()),
                    // §5.2: exp(ln(u)) -> u dönüşümü u > 0 yan koşulunu sileceği için YASAKTIR!
                    _ => Expr::Exp(Box::new(se)),
                }
            }
            Expr::Neg(e) => {
                let se = e.simplify_once();
                match se {
                    Expr::Constant(v) if v.is_finite() => Expr::Constant(-v),
                    Expr::Neg(inner) => *inner,
                    other => Expr::Neg(Box::new(other)),
                }
            }
            other => other.clone(),
        }
    }

    /// Faz 3: Sembolik Denklem Çözücü / Ters Fonksiyon İzolatörü (Symbolic Equation Inverter).
    /// `lhs = rhs` denklemini verilen `target_var` değişkeni için cebirsel olarak tersine çevirir,
    /// `target_var = isolated_rhs` ifadesini ve gereken tüm tanım kümesi yan koşullarını döndürür.
    pub fn solve_for(lhs: &Expr, rhs: &Expr, target_var: &str) -> Result<(Expr, Vec<String>), String> {
        let mut guards = Vec::new();
        for g in lhs.extract_domain_conditions()? {
            if !guards.contains(&g) {
                guards.push(g);
            }
        }
        for g in rhs.extract_domain_conditions()? {
            if !guards.contains(&g) {
                guards.push(g);
            }
        }

        // 1. Önce her iki tarafı sadeleştir (u * u -> u^2 birleştirmesi dahil)
        let mut l = lhs.simplify();
        let mut r = rhs.simplify();

        // Hedef değişken sağ taraftaysa ve sol tarafta yoksa yer değiştir (l <-> r)
        if !l.contains_var(target_var) && r.contains_var(target_var) {
            std::mem::swap(&mut l, &mut r);
        }

        if !l.contains_var(target_var) {
            return Err(format!(
                "Değişken '{}' denklem içinde bulunamadı: {} = {}",
                target_var, lhs, rhs
            ));
        }

        // Eğer hedef değişken hem sol hem sağ tarafta veya toplam içinde birden fazla terimde doğrusal geçiyorsa,
        // l - r = 0 ifadesini A * target_var + B = 0 biçiminde çarpanlarına ayırmayı dene.
        if l.var_occurrences(target_var) + r.var_occurrences(target_var) > 1 {
            let diff = Expr::sub(l.clone(), r.clone()).simplify();
            if let Some((coeff_a, offset_b)) = diff.extract_linear_factors(target_var) {
                let sa = coeff_a.simplify();
                let sb = offset_b.simplify();
                if !sa.contains_var(target_var) && !sb.contains_var(target_var) && sa != Expr::Constant(0.0) {
                    l = Expr::mul(sa, Expr::var(target_var));
                    r = Expr::neg(sb).simplify();
                } else {
                    return Err(format!(
                        "Değişken '{}' denklemde doğrusal olmayan biçimde birden fazla kez geçiyor: {} = {}",
                        target_var, l, r
                    ));
                }
            } else {
                return Err(format!(
                    "Değişken '{}' denklemde izole edilemeyen çoklu terimde geçiyor: {} = {}",
                    target_var, l, r
                ));
            }
        }

        // 2. Tekil geçişli ağaçta kök düğümden yaprağa adım adım ters işlem uygula
        let mut step_guard = 0usize;
        while l != Expr::Variable(target_var.to_string()) {
            step_guard += 1;
            if step_guard > 64 {
                return Err("Sembolik denklem tersleme adım sınırı aşıldı!".to_string());
            }

            match l.clone() {
                Expr::Variable(v) if v == target_var => break,
                Expr::Neg(inner) => {
                    l = *inner;
                    r = Expr::neg(r);
                }
                Expr::Add(a, b) => {
                    if a.contains_var(target_var) {
                        l = *a;
                        r = Expr::sub(r, *b);
                    } else {
                        l = *b;
                        r = Expr::sub(r, *a);
                    }
                }
                Expr::Sub(a, b) => {
                    if a.contains_var(target_var) {
                        // a - b = r  ==>  a = r + b
                        l = *a;
                        r = Expr::add(r, *b);
                    } else {
                        // a - b = r  ==>  b = a - r
                        l = *b;
                        r = Expr::sub(*a, r);
                    }
                }
                Expr::Mul(a, b) => {
                    if a.contains_var(target_var) {
                        // a * b = r  ==>  a = r / b  (b != 0)
                        Self::push_nonzero_guard(&b, &mut guards);
                        l = *a;
                        r = Expr::div(r, *b);
                    } else {
                        // a * b = r  ==>  b = r / a  (a != 0)
                        Self::push_nonzero_guard(&a, &mut guards);
                        l = *b;
                        r = Expr::div(r, *a);
                    }
                }
                Expr::Div(a, b) => {
                    if a.contains_var(target_var) {
                        // a / b = r  ==>  a = r * b  (b != 0 zaten korundu)
                        Self::push_nonzero_guard(&b, &mut guards);
                        l = *a;
                        r = Expr::mul(r, *b);
                    } else {
                        // a / b = r  ==>  b = a / r  (r != 0)
                        Self::push_nonzero_guard(&r, &mut guards);
                        l = *b;
                        r = Expr::div(*a, r);
                    }
                }
                Expr::Pow(base, exp) => {
                    if base.contains_var(target_var) && !exp.contains_var(target_var) {
                        let s_exp = exp.simplify();
                        match &s_exp {
                            Expr::Constant(p) if (*p - 2.0).abs() < 1e-12 => {
                                // base^2 = r  ==>  base = sqrt(r)  (r >= 0 ve asal dal base >= 0)
                                Self::push_unique_guard(&mut guards, format!("{} >= 0", r.simplify()));
                                Self::push_unique_guard(&mut guards, format!("{} >= 0", base.simplify()));
                                l = *base;
                                r = Expr::sqrt(r);
                            }
                            Expr::Constant(p) if (*p - 0.5).abs() < 1e-12 => {
                                // base^0.5 = r  ==>  base = r^2  (yalancı kökleri engellemek için r >= 0 ve base >= 0!)
                                Self::push_unique_guard(&mut guards, format!("{} >= 0", r.simplify()));
                                Self::push_unique_guard(&mut guards, format!("{} >= 0", base.simplify()));
                                l = *base;
                                r = Expr::pow(r, Expr::Constant(2.0));
                            }
                            Expr::Constant(p) if (*p + 1.0).abs() < 1e-12 => {
                                // base^(-1) = r  ==>  base = 1 / r  (r != 0 ve base != 0)
                                Self::push_nonzero_guard(&r, &mut guards);
                                Self::push_nonzero_guard(&base, &mut guards);
                                l = *base;
                                r = Expr::div(Expr::Constant(1.0), r);
                            }
                            Expr::Constant(p) if p.is_finite() && p.abs() > 1e-12 => {
                                let is_int = (*p - p.round()).abs() < 1e-12;
                                if is_int {
                                    let k = p.round() as i64;
                                    if k % 2 == 0 {
                                        if k > 0 {
                                            Self::push_unique_guard(&mut guards, format!("{} >= 0", r.simplify()));
                                            Self::push_unique_guard(&mut guards, format!("{} >= 0", base.simplify()));
                                        } else {
                                            Self::push_unique_guard(&mut guards, format!("{} > 0", r.simplify()));
                                            Self::push_unique_guard(&mut guards, format!("{} > 0", base.simplify()));
                                        }
                                    } else if k < 0 {
                                        Self::push_nonzero_guard(&r, &mut guards);
                                        Self::push_nonzero_guard(&base, &mut guards);
                                    }
                                } else if *p > 0.0 {
                                    Self::push_unique_guard(&mut guards, format!("{} >= 0", r.simplify()));
                                    Self::push_unique_guard(&mut guards, format!("{} >= 0", base.simplify()));
                                } else {
                                    Self::push_unique_guard(&mut guards, format!("{} > 0", r.simplify()));
                                    Self::push_unique_guard(&mut guards, format!("{} > 0", base.simplify()));
                                }
                                l = *base;
                                r = Expr::pow(r, Expr::Constant(1.0 / *p));
                            }
                            Expr::Constant(_) => {
                                return Err(format!("Sıfır üslü ifade '{}' taban için terslenemez!", l));
                            }
                            _ => {
                                Self::push_nonzero_guard(&s_exp, &mut guards);
                                Self::push_unique_guard(&mut guards, format!("{} > 0", r.simplify()));
                                Self::push_unique_guard(&mut guards, format!("{} > 0", base.simplify()));
                                l = *base;
                                r = Expr::pow(r, Expr::div(Expr::Constant(1.0), s_exp));
                            }
                        }
                    } else if !base.contains_var(target_var) && exp.contains_var(target_var) {
                        // base^exp = r  ==>  exp = ln(r) / ln(base)
                        l = *exp;
                        r = Expr::div(Expr::ln(r), Expr::ln(*base));
                    } else {
                        return Err(format!("Üs ifadesi '{}' hem taban hem üste hedef değişken içeriyor!", l));
                    }
                }
                Expr::Sqrt(inner) => {
                    // sqrt(inner) = r  ==>  inner = r^2  (r >= 0 ve inner >= 0)
                    let g_r = format!("{} >= 0", r.simplify());
                    if !guards.contains(&g_r) {
                        guards.push(g_r);
                    }
                    let g_i = format!("{} >= 0", inner.simplify());
                    if !guards.contains(&g_i) {
                        guards.push(g_i);
                    }
                    l = *inner;
                    r = Expr::pow(r, Expr::Constant(2.0));
                }
                Expr::Ln(inner) => {
                    // ln(inner) = r  ==>  inner = exp(r)
                    l = *inner;
                    r = Expr::exp(r);
                }
                Expr::Exp(inner) => {
                    // exp(inner) = r  ==>  inner = ln(r)  (r > 0)
                    let g = format!("{} > 0", r.simplify());
                    if !guards.contains(&g) {
                        guards.push(g);
                    }
                    l = *inner;
                    r = Expr::ln(r);
                }
                other => {
                    return Err(format!("Terslenemeyen sembolik düğüm: {}", other));
                }
            }
        }

        let final_rhs = r.simplify();
        if final_rhs.contains_var(target_var) {
            return Err(format!(
                "Değişken '{}' sağ taraftan tamamen izole edilemedi: {} = {}",
                target_var, target_var, final_rhs
            ));
        }

        for g in final_rhs.extract_domain_conditions()? {
            if !guards.contains(&g) {
                guards.push(g);
            }
        }

        Ok((final_rhs, guards))
    }

    fn push_nonzero_guard(expr: &Expr, guards: &mut Vec<String>) {
        let s = expr.simplify();
        if !matches!(s, Expr::Constant(c) if c.abs() > 1e-15) {
            let g = format!("{} != 0", s);
            if !guards.contains(&g) {
                guards.push(g);
            }
        }
    }

    /// İfadenin `var` değişkenine göre doğrusal (A * var + B) ayrıştırmasını bulur.
    fn extract_linear_factors(&self, var: &str) -> Option<(Expr, Expr)> {
        if !self.contains_var(var) {
            return Some((Expr::Constant(0.0), self.clone()));
        }
        match self {
            Expr::Variable(v) if v == var => Some((Expr::Constant(1.0), Expr::Constant(0.0))),
            Expr::Neg(inner) => {
                let (a, b) = inner.extract_linear_factors(var)?;
                Some((Expr::neg(a), Expr::neg(b)))
            }
            Expr::Add(l, r) => {
                let (al, bl) = l.extract_linear_factors(var)?;
                let (ar, br) = r.extract_linear_factors(var)?;
                Some((Expr::add(al, ar), Expr::add(bl, br)))
            }
            Expr::Sub(l, r) => {
                let (al, bl) = l.extract_linear_factors(var)?;
                let (ar, br) = r.extract_linear_factors(var)?;
                Some((Expr::sub(al, ar), Expr::sub(bl, br)))
            }
            Expr::Mul(l, r) => {
                if !l.contains_var(var) {
                    let (ar, br) = r.extract_linear_factors(var)?;
                    Some((Expr::mul((**l).clone(), ar), Expr::mul((**l).clone(), br)))
                } else if !r.contains_var(var) {
                    let (al, bl) = l.extract_linear_factors(var)?;
                    Some((Expr::mul(al, (**r).clone()), Expr::mul(bl, (**r).clone())))
                } else {
                    None
                }
            }
            Expr::Div(l, r) => {
                if !r.contains_var(var) {
                    let (al, bl) = l.extract_linear_factors(var)?;
                    Some((Expr::div(al, (**r).clone()), Expr::div(bl, (**r).clone())))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Constant(v) => write!(f, "{}", v),
            Expr::Variable(name) => write!(f, "{}", name),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Sub(l, r) => write!(f, "({} - {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
            Expr::Div(l, r) => write!(f, "({} / {})", l, r),
            Expr::Pow(b, e) => {
                if matches!(b.as_ref(), Expr::Pow(_, _)) {
                    write!(f, "({})^{}", b, e)
                } else {
                    write!(f, "{}^{}", b, e)
                }
            }
            Expr::Sqrt(e) => write!(f, "sqrt({})", e),
            Expr::Ln(e) => write!(f, "ln({})", e),
            Expr::Exp(e) => write!(f, "exp({})", e),
            Expr::Neg(e) => write!(f, "(-{})", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_operator_precedence_and_errors() {
        let e1 = Expr::parse_str("2 + 3 * 4^2").unwrap();
        assert!((e1.evaluate(&HashMap::new()).unwrap() - 50.0).abs() < 1e-12);

        let e2 = Expr::parse_str("-(3^2) + (-3)^2").unwrap();
        assert!(e2.evaluate(&HashMap::new()).unwrap().abs() < 1e-12);

        assert!(Expr::parse_str("").is_err());
        assert!(Expr::parse_str("(1 + 2").is_err());
        assert!(Expr::parse_str("1 + * 2").is_err());
        assert!(Expr::parse_str("system(1)").is_err());
    }

    #[test]
    fn test_infer_dimension_rejects_heterogeneous_add_and_dimensional_transcendentals() {
        let mut dims = HashMap::new();
        dims.insert("L".to_string(), Dimension::length());
        dims.insert("t".to_string(), Dimension::time());
        dims.insert("v".to_string(), Dimension::velocity());

        let ok = Expr::parse_str("L / t + v").unwrap();
        assert_eq!(ok.infer_dimension(&dims).unwrap(), Dimension::velocity());

        let bad_add = Expr::parse_str("L + t").unwrap();
        assert!(bad_add.infer_dimension(&dims).is_err(), "L + t reddedilmeli!");

        let bad_exp = Expr::parse_str("exp(L)").unwrap();
        assert!(bad_exp.infer_dimension(&dims).is_err(), "exp(L) boyutsuz argüman zorunluluğu!");

        let bad_ln = Expr::parse_str("ln(v)").unwrap();
        assert!(bad_ln.infer_dimension(&dims).is_err(), "ln(v) reddedilmeli!");
    }

    #[test]
    fn test_interval_mean_value_tightening_reduces_dependency_overestimation() {
        // f(x) = x - x^2 on [0.4, 0.6]: exact range is [0.24, 0.25].
        // Naive interval gives [0.4,0.6] - [0.16,0.36] = [0.04, 0.44] (width 0.40).
        // Mean-Value / Taylor form dramatically tightens the enclosure while remaining sound!
        let expr = Expr::parse_str("x - x^2").unwrap();
        let mut env = HashMap::new();
        env.insert("x", Interval64::new(0.4, 0.6).unwrap());
        let naive = expr.eval_interval(&env).unwrap();
        let tight = expr.eval_taylor_interval(&env).unwrap();
        assert!(tight.contains(0.24) && tight.contains(0.25));
        assert!(tight.width() < naive.width(), "Mean-Value daraltması naif aralıktan dar olmalı!");
    }

    #[test]
    fn test_symbolic_differentiation_and_solve_for_roundtrip() {
        let expr = Expr::parse_str("v_e * ln(m_0 / m_f)").unwrap();
        let deriv = expr.differentiate("m_0").unwrap();
        let mut pt = HashMap::new();
        pt.insert("v_e", 3000.0);
        pt.insert("m_0", 10.0);
        pt.insert("m_f", 2.0);
        // d(v_e * ln(m_0 / m_f))/d(m_0) = v_e / m_0 = 300.0
        assert!((deriv.evaluate(&pt).unwrap() - 300.0).abs() < 1e-11);

        let (inv_m0, _) = Expr::solve_for(&Expr::Variable("dv".to_string()), &expr, "m_0").unwrap();
        pt.insert("dv", expr.evaluate(&pt).unwrap());
        assert!((inv_m0.evaluate(&pt).unwrap() - 10.0).abs() < 1e-10);
    }
}
