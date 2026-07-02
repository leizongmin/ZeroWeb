//! 表达式引擎实现（spec IF-005 `ExpressionEngine` / FR-008 / DC-6）。
//!
//! 三阶段管线：parse（Pratt 解析为 `Expression` AST）→ typecheck（推断 `ValueType`）→
//! eval（在 `EvalContext` 中确定性、无副作用求值）。sandbox：仅白名单纯函数可调用；
//! 禁用能力黑名单（eval/system/require/random/time/...）→ `ForbiddenCapability`，未注册 →
//! `UnknownFunction`；资源上限（节点数/递归深度/迭代数）→ `EvalResourceLimit`。
//!
//! M3 phase-1 覆盖：字面量 / `$path` / 算术·比较·布尔·空值合并 / 条件 `?:` /
//! 纯函数 count·contains·any·all·min·max·clamp·concat·starts_with·ends_with·format·field·index。
//! phase-3 加 `map`/`filter`（字段投影：`map($items, "field")` / `filter($items, "field")`，
//! 不引入 Lambda 变体，保持文法封闭 + sandbox 安全）。

use crate::diagnostics::DslError;
use crate::expression::{BinaryOp, Expression, PureFunctionId, UnaryOp};
use crate::loader::{EvalContext, ExpressionEngine};
use compact_str::CompactString;
use zero_ui_core::binding::{StatePath, Value, ValueType};

// ════════════════════════════════════════════════════════════════════════
// Tokenizer
// ════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(Value),
    Str(String),
    /// `$root.seg.seg`（至少含 root；其余可为空）。
    Path(Vec<String>),
    Ident(String),
    Punct(&'static str),
    Eof,
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Lexer {
            src: src.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, quote: u8) -> Result<String, DslError> {
        self.pos += 1; // 跳过开头引号
        let mut out = String::new();
        while let Some(c) = self.peek() {
            self.pos += 1;
            if c == quote {
                return Ok(out);
            }
            if c == b'\\' {
                let e = self
                    .peek()
                    .ok_or_else(|| DslError::Parse("unterminated escape".into()))?;
                self.pos += 1;
                out.push(match e {
                    b'"' => '"',
                    b'\'' => '\'',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    other => return Err(DslError::Parse(format!("bad escape \\{}", other as char))),
                });
            } else {
                out.push(c as char);
            }
        }
        Err(DslError::Parse("unterminated string".into()))
    }

    fn read_number(&mut self, first: u8) -> Result<Value, DslError> {
        let start = self.pos;
        let mut is_float = first == b'.';
        if first != b'.' {
            self.pos += 1;
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == b'.' {
                is_float = true;
                self.pos += 1;
            } else if c == b'e' || c == b'E' {
                is_float = true;
                self.pos += 1;
                if let Some(s) = self.peek()
                    && (s == b'+' || s == b'-')
                {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).map_err(|_| DslError::Parse("bad utf8".into()))?;
        if is_float {
            text.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| DslError::Parse(format!("bad number {text}")))
        } else {
            text.parse::<i64>()
                .map(Value::Int)
                .map_err(|_| DslError::Parse(format!("bad number {text}")))
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.src[start..self.pos])
            .unwrap_or("")
            .to_string()
    }

    fn read_path(&mut self) -> Result<Vec<String>, DslError> {
        self.pos += 1; // 跳过 $
        if self
            .peek()
            .map(|c| c.is_ascii_alphabetic() || c == b'_')
            .unwrap_or(false)
        {
            let root = self.read_ident();
            let mut segs = vec![root];
            while self.peek() == Some(b'.') {
                self.pos += 1;
                segs.push(self.read_ident());
            }
            Ok(segs)
        } else {
            Err(DslError::Parse("expected identifier after '$'".into()))
        }
    }

    fn next(&mut self) -> Result<Tok, DslError> {
        self.skip_ws();
        let c = match self.peek() {
            None => return Ok(Tok::Eof),
            Some(c) => c,
        };
        if c == b'"' || c == b'\'' {
            return Ok(Tok::Str(self.read_string(c)?));
        }
        if c == b'$' {
            return Ok(Tok::Path(self.read_path()?));
        }
        if c.is_ascii_digit() || (c == b'.' && self.src.get(self.pos + 1).map(|d| d.is_ascii_digit()).unwrap_or(false))
        {
            return Ok(Tok::Num(self.read_number(c)?));
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            return Ok(Tok::Ident(self.read_ident()));
        }
        // 多字符标点
        let two: [u8; 2] = [c, self.src.get(self.pos + 1).copied().unwrap_or(b' ')];
        let two_s = std::str::from_utf8(&two).unwrap_or("");
        for p in ["==", "!=", "<=", ">=", "&&", "||", "??", "->"] {
            if two_s == p {
                self.pos += 2;
                return Ok(Tok::Punct(p));
            }
        }
        // 单字符标点
        self.pos += 1;
        match c {
            b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'!' | b'?' | b':' | b'(' | b')' | b'[' | b']' | b'{' | b'}'
            | b',' | b'.' => Ok(Tok::Punct(single_punct(c))),
            _ => Err(DslError::Parse(format!("unexpected char '{}'", c as char))),
        }
    }
}

fn single_punct(c: u8) -> &'static str {
    match c {
        b'+' => "+",
        b'-' => "-",
        b'*' => "*",
        b'/' => "/",
        b'<' => "<",
        b'>' => ">",
        b'!' => "!",
        b'?' => "?",
        b':' => ":",
        b'(' => "(",
        b')' => ")",
        b'[' => "[",
        b']' => "]",
        b'{' => "{",
        b'}' => "}",
        b',' => ",",
        b'.' => ".",
        _ => "?",
    }
}

// ════════════════════════════════════════════════════════════════════════
// Parser (Pratt / precedence climbing)
// ════════════════════════════════════════════════════════════════════════

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    depth: usize,
}

/// 解析嵌套深度上限。每层嵌套经 ~12 个解析函数帧（parse_expr→ternary→…→primary），
/// 故取 64（≈780 帧）远低于线程栈；合法 UI 表达式嵌套极少超过 ~10 层。
const MAX_PARSE_DEPTH: usize = 64;

impl Parser {
    fn new(src: &str) -> Result<Self, DslError> {
        let mut lex = Lexer::new(src);
        let mut toks = Vec::new();
        loop {
            let t = lex.next()?;
            let eof = t == Tok::Eof;
            toks.push(t);
            if eof {
                break;
            }
        }
        Ok(Parser { toks, pos: 0, depth: 0 })
    }

    fn peek(&self) -> &Tok {
        &self.toks[self.pos]
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.pos].clone();
        if !matches!(t, Tok::Eof) {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, p: &str) -> bool {
        if let Tok::Punct(q) = self.peek()
            && *q == p
        {
            self.pos += 1;
            return true;
        }
        false
    }

    fn expect(&mut self, p: &str) -> Result<(), DslError> {
        if self.eat(p) {
            Ok(())
        } else {
            Err(DslError::Parse(format!("expected '{p}'")))
        }
    }

    fn parse_expr(&mut self) -> Result<Expression, DslError> {
        self.depth += 1;
        if self.depth > MAX_PARSE_DEPTH {
            self.depth -= 1;
            return Err(DslError::EvalResourceLimit(format!("parse depth > {MAX_PARSE_DEPTH}")));
        }
        let r = self.ternary();
        self.depth -= 1;
        r
    }

    fn ternary(&mut self) -> Result<Expression, DslError> {
        let cond = self.coalesce()?;
        if self.eat("?") {
            let then_e = self.parse_expr()?;
            self.expect(":")?;
            let else_e = self.parse_expr()?;
            Ok(Expression::Conditional {
                condition: Box::new(cond),
                then_expr: Box::new(then_e),
                else_expr: Box::new(else_e),
            })
        } else {
            Ok(cond)
        }
    }

    fn coalesce(&mut self) -> Result<Expression, DslError> {
        let mut left = self.logic_or()?;
        while self.eat("??") {
            let right = self.logic_or()?;
            left = Expression::Binary {
                op: BinaryOp::NullCoalesce,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn logic_or(&mut self) -> Result<Expression, DslError> {
        let mut left = self.logic_and()?;
        while self.eat("||") {
            let right = self.logic_and()?;
            left = Expression::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn logic_and(&mut self) -> Result<Expression, DslError> {
        let mut left = self.equality()?;
        while self.eat("&&") {
            let right = self.equality()?;
            left = Expression::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn equality(&mut self) -> Result<Expression, DslError> {
        let mut left = self.compare()?;
        loop {
            let op = if self.eat("==") {
                BinaryOp::Eq
            } else if self.eat("!=") {
                BinaryOp::Ne
            } else {
                break;
            };
            let right = self.compare()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn compare(&mut self) -> Result<Expression, DslError> {
        let mut left = self.additive()?;
        loop {
            let op = if self.eat("<") {
                BinaryOp::Lt
            } else if self.eat("<=") {
                BinaryOp::Le
            } else if self.eat(">") {
                BinaryOp::Gt
            } else if self.eat(">=") {
                BinaryOp::Ge
            } else {
                break;
            };
            let right = self.additive()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn additive(&mut self) -> Result<Expression, DslError> {
        let mut left = self.mul()?;
        loop {
            let op = if self.eat("+") {
                BinaryOp::Add
            } else if self.eat("-") {
                BinaryOp::Sub
            } else {
                break;
            };
            let right = self.mul()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn mul(&mut self) -> Result<Expression, DslError> {
        let mut left = self.unary()?;
        loop {
            let op = if self.eat("*") {
                BinaryOp::Mul
            } else if self.eat("/") {
                BinaryOp::Div
            } else {
                break;
            };
            let right = self.unary()?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expression, DslError> {
        if self.eat("!") {
            let e = self.unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Not,
                expr: Box::new(e),
            });
        }
        if self.eat("-") {
            let e = self.unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(e),
            });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expression, DslError> {
        let mut e = self.primary()?;
        loop {
            if self.eat(".") {
                // 字段访问 → Call("field", [e, "name"])
                let name = match self.bump() {
                    Tok::Ident(s) => s,
                    other => return Err(DslError::Parse(format!("expected field name after '.', got {other:?}"))),
                };
                e = Expression::Call {
                    function: PureFunctionId::new("field"),
                    args: vec![e, Expression::Literal(Value::Text(name))],
                };
            } else if self.eat("[") {
                let idx = self.parse_expr()?;
                self.expect("]")?;
                e = Expression::Call {
                    function: PureFunctionId::new("index"),
                    args: vec![e, idx],
                };
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> Result<Expression, DslError> {
        match self.bump() {
            Tok::Num(v) => Ok(Expression::Literal(v)),
            Tok::Str(s) => Ok(Expression::Literal(Value::Text(s))),
            Tok::Path(segs) => {
                let compact = segs.into_iter().map(CompactString::new).collect();
                Ok(Expression::Path(StatePath(compact)))
            }
            Tok::Punct("(") => {
                let e = self.parse_expr()?;
                self.expect(")")?;
                Ok(e)
            }
            Tok::Punct("[") => {
                let mut items = Vec::new();
                if !matches!(self.peek(), Tok::Punct("]")) {
                    items.push(self.parse_expr()?);
                    while self.eat(",") {
                        if matches!(self.peek(), Tok::Punct("]")) {
                            break;
                        }
                        items.push(self.parse_expr()?);
                    }
                }
                self.expect("]")?;
                Ok(Expression::Array(items))
            }
            Tok::Punct("{") => {
                let mut entries = Vec::new();
                if !matches!(self.peek(), Tok::Punct("}")) {
                    entries.push(self.obj_entry()?);
                    while self.eat(",") {
                        if matches!(self.peek(), Tok::Punct("}")) {
                            break;
                        }
                        entries.push(self.obj_entry()?);
                    }
                }
                self.expect("}")?;
                Ok(Expression::Object(entries))
            }
            Tok::Ident(name) => {
                match name.as_str() {
                    "true" => Ok(Expression::Literal(Value::Bool(true))),
                    "false" => Ok(Expression::Literal(Value::Bool(false))),
                    "null" => Ok(Expression::Literal(Value::Null)),
                    _ => {
                        // 函数调用
                        if self.eat("(") {
                            let mut args = Vec::new();
                            if !matches!(self.peek(), Tok::Punct(")")) {
                                args.push(self.parse_expr()?);
                                while self.eat(",") {
                                    args.push(self.parse_expr()?);
                                }
                            }
                            self.expect(")")?;
                            Ok(Expression::Call {
                                function: PureFunctionId::new(&name),
                                args,
                            })
                        } else {
                            Err(DslError::Parse(format!(
                                "unknown identifier '{name}'（变量须用 $path 引用）"
                            )))
                        }
                    }
                }
            }
            other => Err(DslError::Parse(format!("unexpected token {other:?}"))),
        }
    }

    fn obj_entry(&mut self) -> Result<(CompactString, Expression), DslError> {
        let key = match self.bump() {
            Tok::Ident(s) => s,
            Tok::Str(s) => s,
            other => return Err(DslError::Parse(format!("expected object key, got {other:?}"))),
        };
        self.expect(":")?;
        let v = self.parse_expr()?;
        Ok((CompactString::new(key), v))
    }
}

// ════════════════════════════════════════════════════════════════════════
// Typecheck（推断 ValueType）
// ════════════════════════════════════════════════════════════════════════

fn type_of(expr: &Expression) -> ValueType {
    match expr {
        Expression::Literal(v) => v.value_type(),
        Expression::Path(_) => ValueType::Any,
        Expression::Unary { op, .. } => match op {
            UnaryOp::Neg => ValueType::Number,
            UnaryOp::Not => ValueType::Bool,
        },
        Expression::Binary { op, .. } => match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => ValueType::Number,
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => ValueType::Bool,
            BinaryOp::And | BinaryOp::Or => ValueType::Bool,
            BinaryOp::NullCoalesce => ValueType::Any,
        },
        Expression::Conditional {
            then_expr, else_expr, ..
        } => {
            let a = type_of(then_expr);
            let b = type_of(else_expr);
            if a == b { a } else { ValueType::Any }
        }
        Expression::Call { function, args } => call_return_type(function.0.as_str(), args),
        Expression::Array(_) => ValueType::Array,
        Expression::Object(_) => ValueType::Object,
    }
}

fn call_return_type(name: &str, args: &[Expression]) -> ValueType {
    match name {
        "count" => ValueType::Int,
        "contains" | "starts_with" | "ends_with" | "any" | "all" => ValueType::Bool,
        "min" | "max" | "clamp" => ValueType::Number,
        "concat" | "format" => ValueType::Text,
        "field" | "index" => args.first().map(type_of).unwrap_or(ValueType::Any),
        // map/filter 经字段投影返回数组（element 类型由运行时决定）。
        "map" | "filter" => ValueType::Array,
        _ => ValueType::Any,
    }
}

// ════════════════════════════════════════════════════════════════════════
// Evaluator
// ════════════════════════════════════════════════════════════════════════

/// 显式禁用的能力（系统/IO/时钟/随机/动态代码）—— 与「未注册」区分。
const FORBIDDEN_FN: &[&str] = &[
    "eval", "system", "require", "import", "load", "exec", "spawn", "fetch", "open", "read", "write", "random", "rand",
    "time", "now", "date", "clock", "while", "loop", "recurse",
];

/// 白名单纯函数。
const PURE_FN: &[&str] = &[
    "count",
    "contains",
    "any",
    "all",
    "min",
    "max",
    "clamp",
    "concat",
    "starts_with",
    "ends_with",
    "format",
    "field",
    "index",
    "map",
    "filter",
];

struct EvalState {
    depth: usize,
    iters: usize,
    max_depth: usize,
    max_iters: usize,
}

fn eval(expr: &Expression, ctx: &EvalContext, st: &mut EvalState) -> Result<Value, DslError> {
    st.depth += 1;
    if st.depth > st.max_depth {
        return Err(DslError::EvalResourceLimit(format!("eval depth > {}", st.max_depth)));
    }
    let result = eval_inner(expr, ctx, st);
    st.depth -= 1;
    result
}

fn eval_inner(expr: &Expression, ctx: &EvalContext, st: &mut EvalState) -> Result<Value, DslError> {
    match expr {
        Expression::Literal(v) => Ok(v.clone()),
        Expression::Path(p) => Ok(resolve_path(p, ctx)),
        Expression::Unary { op, expr } => {
            let v = eval(expr, ctx, st)?;
            Ok(match op {
                UnaryOp::Neg => match v {
                    // checked_neg：i64::MIN 无 i64 相反数（|MIN| = MAX+1）→ 直接 `-i` 在 debug
                    // panic（overflow）、release wrap 成 MIN（静默错误）。sandbox 不得对任意 host
                    // 状态值 panic，故 None 时安全退化为 Float（正 9.22e18），与 binary 算术经
                    // f64 的安全路径一致（lei-deep-review 修复）。
                    Value::Int(i) => match i.checked_neg() {
                        Some(n) => Value::Int(n),
                        None => Value::Float(-(i as f64)),
                    },
                    Value::Float(f) => Value::Float(-f),
                    _ => return Err(DslError::Typecheck("unary '-' on non-number".into())),
                },
                UnaryOp::Not => Value::Bool(!v.is_truthy()),
            })
        }
        Expression::Binary { op, left, right } => {
            // 短路布尔。
            if matches!(op, BinaryOp::And) {
                let l = eval(left, ctx, st)?;
                if !l.is_truthy() {
                    return Ok(Value::Bool(false));
                }
                return Ok(Value::Bool(eval(right, ctx, st)?.is_truthy()));
            }
            if matches!(op, BinaryOp::Or) {
                let l = eval(left, ctx, st)?;
                if l.is_truthy() {
                    return Ok(Value::Bool(true));
                }
                return Ok(Value::Bool(eval(right, ctx, st)?.is_truthy()));
            }
            let l = eval(left, ctx, st)?;
            if matches!(op, BinaryOp::NullCoalesce) {
                if l == Value::Null {
                    return eval(right, ctx, st);
                }
                return Ok(l);
            }
            let r = eval(right, ctx, st)?;
            Ok(binary_op(*op, l, r)?)
        }
        Expression::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            let c = eval(condition, ctx, st)?;
            if c.is_truthy() {
                eval(then_expr, ctx, st)
            } else {
                eval(else_expr, ctx, st)
            }
        }
        Expression::Call { function, args } => call_fn(function, args, ctx, st),
        Expression::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for e in items {
                out.push(eval(e, ctx, st)?);
            }
            Ok(Value::Array(out))
        }
        Expression::Object(entries) => {
            let mut obj = hashbrown::HashMap::with_capacity(entries.len());
            for (k, e) in entries {
                obj.insert(k.to_string(), eval(e, ctx, st)?);
            }
            Ok(Value::Object(obj))
        }
    }
}

fn resolve_path(p: &StatePath, ctx: &EvalContext) -> Value {
    let mut segs = p.0.iter();
    let Some(root) = segs.next() else {
        return Value::Null;
    };
    let mut cur = match ctx.vars.get(root.as_str()) {
        Some(v) => v.clone(),
        None => return Value::Null,
    };
    for seg in segs {
        cur = match &cur {
            Value::Object(o) => o.get(seg.as_str()).cloned().unwrap_or(Value::Null),
            Value::Array(a) => seg
                .as_str()
                .parse::<usize>()
                .ok()
                .and_then(|i| a.get(i).cloned())
                .unwrap_or(Value::Null),
            _ => Value::Null,
        };
    }
    cur
}

fn as_number(v: &Value) -> Option<f64> {
    match v {
        Value::Int(i) => Some(*i as f64),
        Value::Float(f) => Some(*f),
        _ => None,
    }
}

fn binary_op(op: BinaryOp, l: Value, r: Value) -> Result<Value, DslError> {
    use BinaryOp::*;
    Ok(match op {
        Add => {
            // Text + Text → concat；Number + Number → add。
            match (l.clone(), r.clone()) {
                (Value::Text(a), Value::Text(b)) => Value::Text(a + &b),
                (a, b) => {
                    let (la, lb) = (as_number(&a), as_number(&b));
                    match (la, lb) {
                        (Some(x), Some(y)) => num(x + y),
                        _ => return Err(DslError::Typecheck("'+' needs numbers or texts".into())),
                    }
                }
            }
        }
        Sub | Mul | Div => {
            let (x, y) = (as_number(&l), as_number(&r));
            match (x, y, op) {
                (Some(x), Some(y), Sub) => num(x - y),
                (Some(x), Some(y), Mul) => num(x * y),
                (Some(x), Some(y), Div) => {
                    if y == 0.0 {
                        Value::Float(f64::NAN)
                    } else {
                        num(x / y)
                    }
                }
                _ => return Err(DslError::Typecheck("arithmetic on non-number".into())),
            }
        }
        Eq => Value::Bool(l == r),
        Ne => Value::Bool(l != r),
        Lt | Le | Gt | Ge => {
            // 数字或文本字典序。
            let ord = match (&l, &r) {
                (Value::Text(a), Value::Text(b)) => a.cmp(b),
                _ => {
                    let (x, y) = (
                        as_number(&l).ok_or_else(|| DslError::Typecheck("compare on non-number".into()))?,
                        as_number(&r).ok_or_else(|| DslError::Typecheck("compare on non-number".into()))?,
                    );
                    x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
                }
            };
            Value::Bool(match op {
                Lt => ord == std::cmp::Ordering::Less,
                Le => ord != std::cmp::Ordering::Greater,
                Gt => ord == std::cmp::Ordering::Greater,
                Ge => ord != std::cmp::Ordering::Less,
                _ => unreachable!(),
            })
        }
        And | Or | NullCoalesce => unreachable!("short-circuit handled earlier"),
    })
}

/// f64 → Int（若整值）或 Float。
fn num(x: f64) -> Value {
    if x.fract() == 0.0 && x.is_finite() && x.abs() < 9.0072e15 {
        Value::Int(x as i64)
    } else {
        Value::Float(x)
    }
}

fn call_fn(
    function: &PureFunctionId,
    args: &[Expression],
    ctx: &EvalContext,
    st: &mut EvalState,
) -> Result<Value, DslError> {
    let name = function.0.as_str();
    if !ctx.allow_functions {
        return Err(DslError::ForbiddenCapability(format!(
            "functions disabled (call '{name}')"
        )));
    }
    if FORBIDDEN_FN.contains(&name) {
        return Err(DslError::ForbiddenCapability(format!(
            "'{name}' is a forbidden capability"
        )));
    }
    if !PURE_FN.contains(&name) {
        return Err(DslError::UnknownFunction(name.to_string()));
    }
    // 求值参数（受迭代预算约束）。
    let argv: Result<Vec<Value>, DslError> = args.iter().map(|a| eval(a, ctx, st)).collect();
    let argv = argv?;
    let mut bump = |n: usize| -> Result<(), DslError> {
        st.iters += n;
        if st.iters > st.max_iters {
            Err(DslError::EvalResourceLimit(format!(
                "iteration count > {}",
                st.max_iters
            )))
        } else {
            Ok(())
        }
    };
    match name {
        "count" => {
            expect_argc(name, &argv, 1)?;
            let n = match &argv[0] {
                Value::Array(a) => a.len(),
                Value::Text(s) => s.chars().count(),
                _ => return Err(DslError::Typecheck("count() needs array/text".into())),
            };
            bump(n)?;
            Ok(Value::Int(n as i64))
        }
        "contains" => {
            expect_argc(name, &argv, 2)?;
            match (&argv[0], &argv[1]) {
                (Value::Array(a), needle) => {
                    bump(a.len())?;
                    Ok(Value::Bool(a.contains(needle)))
                }
                (Value::Text(h), Value::Text(n)) => Ok(Value::Bool(h.contains(n.as_str()))),
                _ => Err(DslError::Typecheck("contains() needs array/text + needle".into())),
            }
        }
        "any" => {
            expect_argc(name, &argv, 1)?;
            let a = as_array(&argv[0])?;
            bump(a.len())?;
            Ok(Value::Bool(a.iter().any(|v| v.is_truthy())))
        }
        "all" => {
            expect_argc(name, &argv, 1)?;
            let a = as_array(&argv[0])?;
            bump(a.len())?;
            Ok(Value::Bool(a.iter().all(|v| v.is_truthy())))
        }
        "min" | "max" => {
            expect_argc(name, &argv, 2)?;
            let (x, y) = (num_arg(&argv[0], name)?, num_arg(&argv[1], name)?);
            Ok(num(if name == "min" { x.min(y) } else { x.max(y) }))
        }
        "clamp" => {
            expect_argc(name, &argv, 3)?;
            let (v, lo, hi) = (
                num_arg(&argv[0], name)?,
                num_arg(&argv[1], name)?,
                num_arg(&argv[2], name)?,
            );
            Ok(num(v.max(lo).min(hi)))
        }
        "concat" => {
            let mut s = String::new();
            for v in &argv {
                match v {
                    Value::Text(t) => s.push_str(t.as_str()),
                    _ => return Err(DslError::Typecheck("concat() needs texts".into())),
                }
            }
            Ok(Value::Text(s))
        }
        "starts_with" => {
            expect_argc(name, &argv, 2)?;
            let (h, n) = (text_arg(&argv[0], name)?, text_arg(&argv[1], name)?);
            Ok(Value::Bool(h.starts_with(n.as_str())))
        }
        "ends_with" => {
            expect_argc(name, &argv, 2)?;
            let (h, n) = (text_arg(&argv[0], name)?, text_arg(&argv[1], name)?);
            Ok(Value::Bool(h.ends_with(n.as_str())))
        }
        "format" => {
            // format("{}-{}", 1, 2) —— 顺序占位 {}（配对跳过 `{}`）。
            expect_argc_min(name, &argv, 1)?;
            let tmpl = text_arg(&argv[0], name)?;
            let rest = &argv[1..];
            let chars: Vec<char> = tmpl.chars().collect();
            let mut out = String::new();
            let mut i = 0;
            let mut arg_idx = 0;
            while i < chars.len() {
                if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '}' {
                    if arg_idx < rest.len() {
                        out.push_str(&value_to_text(&rest[arg_idx]));
                        arg_idx += 1;
                    }
                    i += 2; // 跳过 {}
                } else {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            Ok(Value::Text(out))
        }
        "field" => {
            expect_argc(name, &argv, 2)?;
            match (&argv[0], &argv[1]) {
                (Value::Object(o), Value::Text(k)) => Ok(o.get(k.as_str()).cloned().unwrap_or(Value::Null)),
                _ => Err(DslError::Typecheck("field() needs object + text key".into())),
            }
        }
        "index" => {
            expect_argc(name, &argv, 2)?;
            match (&argv[0], &argv[1]) {
                (Value::Array(a), Value::Int(i)) => {
                    let idx = *i as isize;
                    if idx < 0 || idx as usize >= a.len() {
                        Ok(Value::Null)
                    } else {
                        Ok(a[idx as usize].clone())
                    }
                }
                _ => Err(DslError::Typecheck("index() needs array + int".into())),
            }
        }
        // map/filter 采用**字段投影**（非 lambda）：第二参数为字段路径（Text，支持点分嵌套 TBD-10）。
        // map 对每个 object 元素按路径取值 → 新数组；filter 保留路径 truthy 的元素。
        // 避免给 Expression 加 Lambda 变体（保持文法封闭、sandbox 安全；每元素仅字段查找，
        // 计算量受数组大小 + 迭代预算约束；谓词过滤 `filter($items, field>x)` 仍需 lambda，超当前受控计算层范围）。
        "map" => {
            expect_argc(name, &argv, 2)?;
            let a = as_array(&argv[0])?;
            let field = text_arg(&argv[1], name)?;
            bump(a.len())?;
            let out: Vec<Value> = a.iter().map(|v| project_field(v, field.as_str())).collect();
            Ok(Value::Array(out))
        }
        "filter" => {
            expect_argc(name, &argv, 2)?;
            let a = as_array(&argv[0])?;
            let field = text_arg(&argv[1], name)?;
            bump(a.len())?;
            let out: Vec<Value> = a
                .iter()
                .filter(|v| project_field(v, field.as_str()).is_truthy())
                .cloned()
                .collect();
            Ok(Value::Array(out))
        }
        _ => Err(DslError::UnknownFunction(name.to_string())),
    }
}

/// 从一个值投影指定字段路径（map/filter 共用）。
///
/// 支持点分嵌套路径：`"a.b.c"` 逐段下钻 object（TBD-10）；任一非 object 中段 → `Null`。
/// 单段路径（`"title"`）行为与历史一致（向后兼容）。
fn project_field(v: &Value, path: &str) -> Value {
    let mut cur = v;
    for seg in path.split('.') {
        cur = match cur {
            Value::Object(o) => o.get(seg).unwrap_or(&Value::Null),
            _ => &Value::Null,
        };
    }
    cur.clone()
}

fn value_to_text(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Text(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => "[value]".into(),
    }
}

fn expect_argc(name: &str, argv: &[Value], n: usize) -> Result<(), DslError> {
    if argv.len() == n {
        Ok(())
    } else {
        Err(DslError::Typecheck(format!(
            "{name}() expects {n} args, got {}",
            argv.len()
        )))
    }
}

fn expect_argc_min(name: &str, argv: &[Value], n: usize) -> Result<(), DslError> {
    if argv.len() >= n {
        Ok(())
    } else {
        Err(DslError::Typecheck(format!(
            "{name}() expects >= {n} args, got {}",
            argv.len()
        )))
    }
}

fn as_array(v: &Value) -> Result<&Vec<Value>, DslError> {
    match v {
        Value::Array(a) => Ok(a),
        _ => Err(DslError::Typecheck("expected array".into())),
    }
}

fn num_arg(v: &Value, fn_name: &str) -> Result<f64, DslError> {
    as_number(v).ok_or_else(|| DslError::Typecheck(format!("{fn_name}() needs numbers")))
}

fn text_arg(v: &Value, fn_name: &str) -> Result<String, DslError> {
    match v {
        Value::Text(s) => Ok(s.clone()),
        _ => Err(DslError::Typecheck(format!("{fn_name}() needs text"))),
    }
}

// ════════════════════════════════════════════════════════════════════════
// Engine（ExpressionEngine impl）
// ════════════════════════════════════════════════════════════════════════

/// 默认表达式引擎（spec IF-005 `ExpressionEngine`）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Engine;

impl ExpressionEngine for Engine {
    fn parse(&self, source: &str) -> Result<Expression, DslError> {
        let mut p = Parser::new(source)?;
        let e = p.parse_expr()?;
        if !matches!(p.peek(), Tok::Eof) {
            return Err(DslError::Parse(format!("trailing tokens at {}", p.pos)));
        }
        Ok(e)
    }

    fn typecheck(
        &self,
        expr: &Expression,
        _schema: &zero_ui_core::binding::BindingSchema,
    ) -> Result<ValueType, DslError> {
        Ok(type_of(expr))
    }

    fn eval(&self, expr: &Expression, ctx: &EvalContext) -> Result<Value, DslError> {
        let nodes = expr.node_count();
        if nodes > ctx.max_nodes {
            return Err(DslError::EvalResourceLimit(format!(
                "node count {nodes} > max_nodes {}",
                ctx.max_nodes
            )));
        }
        let mut st = EvalState {
            depth: 0,
            iters: 0,
            max_depth: ctx.max_depth,
            max_iters: ctx.max_iterations,
        };
        eval(expr, ctx, &mut st)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::EvalContext;

    fn eng() -> Engine {
        Engine
    }

    fn eval_str(src: &str, ctx: &EvalContext) -> Value {
        let e = eng();
        let expr = e.parse(src).unwrap_or_else(|err| panic!("parse {src}: {err:?}"));
        e.eval(&expr, ctx).unwrap_or_else(|err| panic!("eval {src}: {err:?}"))
    }

    fn parse_err(src: &str) -> DslError {
        eng().parse(src).unwrap_err()
    }

    fn eval_err(src: &str, ctx: &EvalContext) -> DslError {
        let e = eng();
        let expr = e.parse(src).expect("should parse");
        e.eval(&expr, ctx).unwrap_err()
    }

    fn state_ctx() -> EvalContext {
        let mut tabs = hashbrown::HashMap::new();
        tabs.insert("count".to_string(), Value::Int(3));
        tabs.insert("active".to_string(), Value::Text("zero.example".into()));
        let mut state = hashbrown::HashMap::new();
        state.insert("tabs".to_string(), Value::Object(tabs));
        state.insert("can_go_back".to_string(), Value::Bool(true));
        EvalContext::default().with_var("state", Value::Object(state))
    }

    // ── literals / arithmetic / precedence ─────────────────────────────
    #[test]
    fn literals_and_arith() {
        let ctx = EvalContext::default();
        assert_eq!(eval_str("1 + 2 * 3", &ctx), Value::Int(7));
        assert_eq!(eval_str("(1 + 2) * 3", &ctx), Value::Int(9));
        assert_eq!(eval_str("10 / 4", &ctx), Value::Float(2.5));
        assert_eq!(eval_str("-5 + 2", &ctx), Value::Int(-3));
        assert_eq!(eval_str("\"hi\"", &ctx), Value::Text("hi".into()));
        assert_eq!(eval_str("true", &ctx), Value::Bool(true));
        assert_eq!(eval_str("null", &ctx), Value::Null);
    }

    #[test]
    fn text_concat_with_plus() {
        let ctx = EvalContext::default();
        assert_eq!(eval_str(r#""a" + "b" + "c""#, &ctx), Value::Text("abc".into()));
    }

    // ── comparison / boolean / null-coalesce / ternary ─────────────────
    #[test]
    fn compare_bool_coalesce_ternary() {
        let ctx = EvalContext::default();
        assert_eq!(eval_str("1 < 2 && 3 > 1", &ctx), Value::Bool(true));
        assert_eq!(eval_str("2 <= 2 || false", &ctx), Value::Bool(true));
        assert_eq!(eval_str("!false", &ctx), Value::Bool(true));
        assert_eq!(eval_str("null ?? 5", &ctx), Value::Int(5));
        assert_eq!(eval_str("7 ?? 5", &ctx), Value::Int(7));
        assert_eq!(eval_str("1 > 2 ? 'a' : 'b'", &ctx), Value::Text("b".into()));
        assert_eq!(eval_str("1 == 1", &ctx), Value::Bool(true));
        assert_eq!(eval_str("1 != 2", &ctx), Value::Bool(true));
    }

    // ── path reads ─────────────────────────────────────────────────────
    #[test]
    fn path_reads_state() {
        let ctx = state_ctx();
        assert_eq!(eval_str("$state.can_go_back", &ctx), Value::Bool(true));
        assert_eq!(eval_str("$state.tabs.count", &ctx), Value::Int(3));
        assert_eq!(eval_str("$state.tabs.active", &ctx), Value::Text("zero.example".into()));
        // 缺失路径 → null（确定性，不报错）。
        assert_eq!(eval_str("$state.missing", &ctx), Value::Null);
        assert_eq!(eval_str("$unknown.root", &ctx), Value::Null);
    }

    // ── 纯函数白名单 ───────────────────────────────────────────────────
    #[test]
    fn pure_functions() {
        let ctx = EvalContext::default();
        assert_eq!(eval_str("count([1, 2, 3])", &ctx), Value::Int(3));
        assert_eq!(eval_str("count('hello')", &ctx), Value::Int(5));
        assert_eq!(eval_str("contains([1, 2, 3], 2)", &ctx), Value::Bool(true));
        assert_eq!(eval_str("contains('hello', 'ell')", &ctx), Value::Bool(true));
        assert_eq!(eval_str("any([false, true])", &ctx), Value::Bool(true));
        assert_eq!(eval_str("all([true, false])", &ctx), Value::Bool(false));
        assert_eq!(eval_str("min(3, 5)", &ctx), Value::Int(3));
        assert_eq!(eval_str("max(3, 5)", &ctx), Value::Int(5));
        assert_eq!(eval_str("clamp(10, 0, 5)", &ctx), Value::Int(5));
        assert_eq!(eval_str("clamp(-1, 0, 5)", &ctx), Value::Int(0));
        assert_eq!(eval_str(r#"concat("a", "b", "c")"#, &ctx), Value::Text("abc".into()));
        assert_eq!(eval_str(r#"starts_with("hello", "he")"#, &ctx), Value::Bool(true));
        assert_eq!(eval_str(r#"ends_with("hello", "lo")"#, &ctx), Value::Bool(true));
        assert_eq!(eval_str(r#"format("{}-{}", 1, 2)"#, &ctx), Value::Text("1-2".into()));
        assert_eq!(eval_str("index([10, 20, 30], 1)", &ctx), Value::Int(20));
        assert_eq!(eval_str("index([10, 20], 9)", &ctx), Value::Null); // 越界 → null
    }

    fn items_ctx() -> EvalContext {
        let mk = |title: &str, active: bool| {
            let mut o = hashbrown::HashMap::new();
            o.insert("title".to_string(), Value::Text(title.into()));
            o.insert("active".to_string(), Value::Bool(active));
            Value::Object(o)
        };
        let items = vec![mk("zero", true), mk("one", false), mk("two", true)];
        EvalContext::default().with_var("items", Value::Array(items))
    }

    #[test]
    fn map_filter_field_projection() {
        // DC-6 phase-3：map/filter 用字段投影（无 lambda）。
        let ctx = items_ctx();
        // map 投影 title。
        assert_eq!(
            eval_str(r#"map($items, "title")"#, &ctx),
            Value::Array(vec![
                Value::Text("zero".into()),
                Value::Text("one".into()),
                Value::Text("two".into()),
            ])
        );
        // filter 保留 active=true → 2 项。
        match eval_str(r#"filter($items, "active")"#, &ctx) {
            Value::Array(a) => assert_eq!(a.len(), 2),
            other => panic!("filter should return array, got {other:?}"),
        }
        // 组合：map + count / filter + count。
        assert_eq!(eval_str(r#"count(map($items, "title"))"#, &ctx), Value::Int(3));
        assert_eq!(eval_str(r#"count(filter($items, "active"))"#, &ctx), Value::Int(2));
        // 缺失字段 → Null（map）/ falsy 丢弃（filter）。
        assert_eq!(
            eval_str(r#"map($items, "missing")"#, &ctx),
            Value::Array(vec![Value::Null, Value::Null, Value::Null])
        );
        assert_eq!(eval_str(r#"count(filter($items, "missing"))"#, &ctx), Value::Int(0));
        // 字段名可由表达式提供（求值为 Text）。
        let key_ctx = EvalContext::default().with_var("items", {
            let mut o = hashbrown::HashMap::new();
            o.insert("title".to_string(), Value::Text("a".into()));
            Value::Array(vec![Value::Object(o)])
        });
        // 非 array → typecheck 错。
        assert!(matches!(
            eval_err(r#"map("notarray", "x")"#, &ctx),
            DslError::Typecheck(_)
        ));
        let _ = key_ctx;
    }

    #[test]
    fn map_filter_nested_path_projection() {
        // DC-6 TBD-10：map/filter 支持点分嵌套路径投影（无 lambda）。
        let mk = |title: &str, count: i64, active: bool| {
            let mut meta = hashbrown::HashMap::new();
            meta.insert("title".to_string(), Value::Text(title.into()));
            meta.insert("count".to_string(), Value::Int(count));
            meta.insert("active".to_string(), Value::Bool(active));
            let mut o = hashbrown::HashMap::new();
            o.insert("meta".to_string(), Value::Object(meta));
            Value::Object(o)
        };
        let ctx =
            EvalContext::default().with_var("items", Value::Array(vec![mk("zero", 1, true), mk("one", 2, false)]));
        // 嵌套路径 map：取 meta.title / meta.count。
        assert_eq!(
            eval_str(r#"map($items, "meta.title")"#, &ctx),
            Value::Array(vec![Value::Text("zero".into()), Value::Text("one".into())])
        );
        assert_eq!(
            eval_str(r#"map($items, "meta.count")"#, &ctx),
            Value::Array(vec![Value::Int(1), Value::Int(2)])
        );
        // 嵌套路径 filter：保留 meta.active=true → 1 项。
        match eval_str(r#"filter($items, "meta.active")"#, &ctx) {
            Value::Array(a) => assert_eq!(a.len(), 1),
            other => panic!("filter nested should return array, got {other:?}"),
        }
        // 缺失叶子 → Null（map）/ falsy 丢弃（filter）。
        assert_eq!(
            eval_str(r#"map($items, "meta.missing")"#, &ctx),
            Value::Array(vec![Value::Null, Value::Null])
        );
        assert_eq!(
            eval_str(r#"count(filter($items, "meta.missing"))"#, &ctx),
            Value::Int(0)
        );
        // 中段缺失 → Null。
        assert_eq!(
            eval_str(r#"map($items, "nope.x")"#, &ctx),
            Value::Array(vec![Value::Null, Value::Null])
        );
        // 路径穿过非 object 叶子（meta.title 是 Text，无 .deep）→ Null。
        assert_eq!(
            eval_str(r#"map($items, "meta.title.deep")"#, &ctx),
            Value::Array(vec![Value::Null, Value::Null])
        );
        // 单段路径仍正常（向后兼容）：map 取 meta → [Object, Object]。
        match eval_str(r#"map($items, "meta")"#, &ctx) {
            Value::Array(a) => assert_eq!(a.len(), 2),
            other => panic!("map single-segment should return array, got {other:?}"),
        }
    }

    #[test]
    fn map_filter_iter_budget() {
        // 大数组触发迭代预算上限（default max_iterations=10_000）。
        let big: Vec<Value> = (0..20_000).map(|_| Value::Int(1)).collect();
        let ctx = EvalContext::default().with_var("big", Value::Array(big));
        assert!(matches!(eval_err("count($big)", &ctx), DslError::EvalResourceLimit(_)));
        assert!(matches!(
            eval_err(r#"map($big, "x")"#, &ctx),
            DslError::EvalResourceLimit(_)
        ));
    }

    #[test]
    fn postfix_field_and_index() {
        let ctx = EvalContext::default();
        assert_eq!(eval_str("{a: 1, b: 2}.b", &ctx), Value::Int(2));
        assert_eq!(eval_str("[10, 20, 30][2]", &ctx), Value::Int(30));
    }

    // ── typecheck ──────────────────────────────────────────────────────
    #[test]
    fn typecheck_infers() {
        let e = eng();
        let schema = zero_ui_core::binding::BindingSchema::default();
        assert_eq!(e.typecheck(&e.parse("1").unwrap(), &schema).unwrap(), ValueType::Int);
        assert_eq!(
            e.typecheck(&e.parse("1 < 2").unwrap(), &schema).unwrap(),
            ValueType::Bool
        );
        assert_eq!(
            e.typecheck(&e.parse("1 + 2").unwrap(), &schema).unwrap(),
            ValueType::Number
        );
        assert_eq!(
            e.typecheck(&e.parse("count([1])").unwrap(), &schema).unwrap(),
            ValueType::Int
        );
        assert_eq!(
            e.typecheck(&e.parse("[1, 2]").unwrap(), &schema).unwrap(),
            ValueType::Array
        );
    }

    // ── sandbox negative tests（DC-6 关键）──────────────────────────────
    #[test]
    fn forbidden_capabilities_rejected() {
        let ctx = EvalContext::default();
        // 禁用能力（IO/系统/时钟/随机/动态代码）→ ForbiddenCapability。
        for src in [
            "eval('x')",
            "system('ls')",
            "require('fs')",
            "random()",
            "time()",
            "while()",
            "fetch('http://x')",
        ] {
            match eval_err(src, &ctx) {
                DslError::ForbiddenCapability(_) => {}
                other => panic!("{src} 应被 ForbiddenCapability 拒绝，got {other:?}"),
            }
        }
    }

    #[test]
    fn unknown_function_rejected() {
        let ctx = EvalContext::default();
        assert!(matches!(eval_err("foo(1)", &ctx), DslError::UnknownFunction(_)));
        assert!(matches!(eval_err("not_a_fn()", &ctx), DslError::UnknownFunction(_)));
    }

    #[test]
    fn functions_disabled_by_context() {
        let ctx = EvalContext {
            allow_functions: false,
            ..EvalContext::default()
        };
        assert!(matches!(eval_err("count([1])", &ctx), DslError::ForbiddenCapability(_)));
    }

    #[test]
    fn bare_identifier_is_parse_error() {
        // 变量必须用 $path；裸标识符（非 true/false/null/函数）→ parse error。
        assert!(matches!(parse_err("foo"), DslError::Parse(_)));
    }

    // ── 资源上限（DC-6 关键）──────────────────────────────────────────
    #[test]
    fn eval_depth_limit() {
        // 深度嵌套 unary（每层一个 AST 节点；括号不产生节点）→ eval 递归超过 max_depth。
        let deep = "!".repeat(20) + "true";
        let ctx = EvalContext {
            max_depth: 5,
            ..EvalContext::default()
        };
        assert!(matches!(eval_err(&deep, &ctx), DslError::EvalResourceLimit(_)));
    }

    #[test]
    fn eval_node_count_limit() {
        // 200 元素数组 → 节点数超 max_nodes。
        let mut src = String::from("[");
        for i in 0..200 {
            if i > 0 {
                src.push(',');
            }
            src.push_str(&i.to_string());
        }
        src.push(']');
        let ctx = EvalContext {
            max_nodes: 50,
            ..EvalContext::default()
        };
        assert!(matches!(eval_err(&src, &ctx), DslError::EvalResourceLimit(_)));
    }

    #[test]
    fn eval_iteration_limit() {
        // 大数组 count → 迭代数超 max_iterations。
        let mut src = String::from("[");
        for i in 0..500 {
            if i > 0 {
                src.push(',');
            }
            src.push('0');
        }
        src.push(']');
        let expr_src = format!("count({src})");
        let ctx = EvalContext {
            max_iterations: 100,
            ..EvalContext::default()
        };
        assert!(matches!(eval_err(&expr_src, &ctx), DslError::EvalResourceLimit(_)));
    }

    #[test]
    fn parse_depth_limit_guard() {
        // 极深嵌套表达式 → parser 深度守卫触发。
        let deep = "(".repeat(MAX_PARSE_DEPTH + 10) + "1" + &")".repeat(MAX_PARSE_DEPTH + 10);
        assert!(matches!(parse_err(&deep), DslError::EvalResourceLimit(_)));
    }

    #[test]
    fn trailing_tokens_rejected() {
        assert!(matches!(parse_err("1 2"), DslError::Parse(_)));
        assert!(matches!(parse_err("1 + "), DslError::Parse(_)));
    }

    #[test]
    fn end_to_end_visible_when() {
        // 模拟 visible_when：tabs.count > 0。
        let ctx = state_ctx();
        assert_eq!(eval_str("$state.tabs.count > 0", &ctx), Value::Bool(true));
    }

    // ── error paths / edge branches（补覆盖率）──────────────────────────
    #[test]
    fn parse_error_paths() {
        assert!(matches!(parse_str_err(r#""abc"#), DslError::Parse(_))); // 未闭合串
        assert!(matches!(parse_str_err(r#""a\q""#), DslError::Parse(_))); // 非法转义
        assert!(matches!(parse_str_err("@"), DslError::Parse(_))); // 非法字符
        assert!(matches!(parse_str_err("$"), DslError::Parse(_))); // $ 后无 ident
        assert!(matches!(parse_str_err("1 +"), DslError::Parse(_))); // 不完整
    }

    fn parse_str_err(src: &str) -> DslError {
        eng().parse(src).unwrap_err()
    }

    #[test]
    fn type_and_argc_errors() {
        let ctx = EvalContext::default();
        // 参数数量错误。
        assert!(matches!(eval_err("count(1, 2)", &ctx), DslError::Typecheck(_)));
        assert!(matches!(eval_err("min(1)", &ctx), DslError::Typecheck(_)));
        // 类型错误。
        assert!(matches!(eval_err("count(1)", &ctx), DslError::Typecheck(_))); // count 非数组
        assert!(matches!(eval_err("1 + true", &ctx), DslError::Typecheck(_))); // + 混类型
        assert!(matches!(eval_err("1 < true", &ctx), DslError::Typecheck(_))); // < 非数字
        assert!(matches!(eval_err("index(1, 2)", &ctx), DslError::Typecheck(_))); // index 非数组
        assert!(matches!(eval_err("field(1, 'x')", &ctx), DslError::Typecheck(_))); // field 非对象
    }

    #[test]
    fn text_comparison_and_format_branches() {
        let ctx = EvalContext::default();
        // 文本字典序比较。
        assert_eq!(eval_str(r#""a" < "b""#, &ctx), Value::Bool(true));
        assert_eq!(eval_str(r#""b" >= "a""#, &ctx), Value::Bool(true));
        // format 把 bool/float 转 text（value_to_text 分支）。
        assert_eq!(eval_str(r#"format("{}", true)"#, &ctx), Value::Text("true".into()));
        assert_eq!(eval_str(r#"format("{}", 1.5)"#, &ctx), Value::Text("1.5".into()));
    }

    #[test]
    fn float_arith_and_div_by_zero() {
        let ctx = EvalContext::default();
        // 分数结果 → Float。
        assert_eq!(eval_str("1.5 + 0.25", &ctx), Value::Float(1.75));
        // 整值结果归一化为 Int（num() 把 5.0 → Int(5)）。
        assert_eq!(eval_str("2.5 * 2", &ctx), Value::Int(5));
        // 除零 → NaN（不 panic）。
        let v = eval_str("1 / 0", &ctx);
        assert!(matches!(v, Value::Float(x) if x.is_nan()));
    }

    // ── 深度审查（lei-deep-review）：unary 取负整数溢出边界 ────────────────
    #[test]
    fn unary_neg_of_int_min_does_not_panic() {
        // i64::MIN 的相反数无 i64 表示（|MIN| = MAX+1）。
        // 此前 `Value::Int(i) => Value::Int(-i)` 在 debug 构建会 panic（attempt to negate with overflow），
        // release 构建会 wrap 成 i64::MIN（静默错误）。sandbox 不得对任意 host 状态值 panic。
        // 经 $path / field / index 解析到含 i64::MIN 的状态即可触发（host 控制状态，非 DSL 作者）。
        // 修复后：checked_neg 对 i64::MIN 返回 None → 安全退化为 Float（正 9.22e18），不 panic、不 wrap。
        let ctx = EvalContext::default().with_var("n", Value::Int(i64::MIN));
        let v = eval_str("-($n)", &ctx);
        match v {
            Value::Float(f) => assert!(f > 0.0 && f.is_finite(), "expected +9.22e18 Float, got {f}"),
            other => panic!("expected Float for -i64::MIN (no exact i64 negation), got {other:?}"),
        }
    }

    #[test]
    fn unary_neg_preserves_int_when_representable() {
        // 可表示的相反数保持 Int（checked_neg → Some）。
        let ctx = EvalContext::default();
        assert_eq!(eval_str("-5", &ctx), Value::Int(-5));
        assert_eq!(eval_str("-0", &ctx), Value::Int(0));
        // i64::MAX 的相反数 = i64::MIN + 1，仍可表示 → 保持 Int。
        let ctx_max = EvalContext::default().with_var("mx", Value::Int(i64::MAX));
        assert_eq!(eval_str("-($mx)", &ctx_max), Value::Int(i64::MIN + 1));
        // 经 field/index 取出的 i64::MIN 同样安全（不只字面状态路径）。
        let arr_ctx = EvalContext::default().with_var("arr", Value::Array(vec![Value::Int(i64::MIN)]));
        match eval_str("-index($arr, 0)", &arr_ctx) {
            Value::Float(f) => assert!(f > 0.0 && f.is_finite()),
            other => panic!("expected Float for -index(..)=i64::MIN, got {other:?}"),
        }
    }
}
