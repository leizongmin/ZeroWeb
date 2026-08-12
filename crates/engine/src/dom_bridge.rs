//! DOM Bridge — JS → DOM 操作桥接层。
//!
//! 将 DOM 操作请求建模为结构化的命令，供 JS 引擎回调时使用。
//! 支持 document.getElementById、querySelector、createElement 等核心 API。
//!
//! 设计说明：桥接层将 JS 调用转换为类型安全的 DOM 命令，
//! 与具体 JS 引擎（V8/QuickJS）解耦，便于测试。

/// DOM 操作命令 — JS 调用产生的 DOM 操作请求。
#[derive(Debug, Clone, PartialEq)]
pub enum DomCommand {
    /// document.getElementById(id) — 返回元素或 null。
    GetElementById {
        /// 元素 ID。
        id: String,
    },
    /// document.querySelector(selector) — 返回第一个匹配元素。
    QuerySelector {
        /// CSS 选择器。
        selector: String,
    },
    /// document.querySelectorAll(selector) — 返回所有匹配元素。
    QuerySelectorAll {
        /// CSS 选择器。
        selector: String,
    },
    /// document.createElement(tagName) — 创建新元素。
    CreateElement {
        /// 标签名。
        tag_name: String,
    },
    /// document.createTextNode(text) — 创建文本节点。
    CreateTextNode {
        /// 文本内容。
        text: String,
    },
    /// element.appendChild(child) — 添加子节点。
    AppendChild {
        /// 父节点 ID。
        parent_id: u64,
        /// 子节点 ID。
        child_id: u64,
    },
    /// element.removeChild(child) — 移除子节点。
    RemoveChild {
        /// 父节点 ID。
        parent_id: u64,
        /// 子节点 ID。
        child_id: u64,
    },
    /// element.setAttribute(name, value) — 设置属性。
    SetAttribute {
        /// 元素 ID。
        element_id: u64,
        /// 属性名。
        name: String,
        /// 属性值。
        value: String,
    },
    /// element.getAttribute(name) — 获取属性。
    GetAttribute {
        /// 元素 ID。
        element_id: u64,
        /// 属性名。
        name: String,
    },
    /// element.textContent — 获取/设置文本内容。
    TextContent {
        /// 元素 ID。
        element_id: u64,
    },
    /// element.textContent = value — 设置文本内容。
    SetTextContent {
        /// 元素 ID。
        element_id: u64,
        /// 新文本内容。
        text: String,
    },
    /// element.innerHTML — 获取/设置 innerHTML。
    InnerHtml {
        /// 元素 ID。
        element_id: u64,
    },
    /// element.className — 获取/设置 class。
    SetClassName {
        /// 元素 ID。
        element_id: u64,
        /// class 值。
        value: String,
    },
    /// document.getElementsByClassName(class) — 按类名查找。
    GetElementsByClassName {
        /// 类名。
        class_name: String,
    },
    /// document.getElementsByTagName(tag) — 按标签名查找。
    GetElementsByTagName {
        /// 标签名。
        tag_name: String,
    },
    /// element.addEventListener(type, listener, capture) — 添加事件监听器。
    AddEventListener {
        /// 元素 ID。
        element_id: u64,
        /// 事件类型（如 "click"、"input"、"keydown"）。
        event_type: String,
        /// 是否在捕获阶段触发。
        capture: bool,
    },
    /// element.removeEventListener(type, listener) — 移除事件监听器。
    RemoveEventListener {
        /// 元素 ID。
        element_id: u64,
        /// 事件类型。
        event_type: String,
    },
    /// 事件分发请求（从宿主运行时发往 DOM）。
    DispatchEvent {
        /// 目标元素 ID。
        target_id: u64,
        /// 事件类型。
        event_type: String,
        /// 是否冒泡。
        bubbles: bool,
        /// 是否可取消。
        cancelable: bool,
    },
    /// element.insertBefore(newChild, refChild) — 在参考节点前插入新节点。
    InsertBefore {
        /// 父节点 ID。
        parent_id: u64,
        /// 新子节点 ID。
        new_child_id: u64,
        /// 参考子节点 ID（None 表示插入末尾）。
        ref_child_id: Option<u64>,
    },
    /// element.replaceChild(newChild, oldChild) — 替换子节点。
    ReplaceChild {
        /// 父节点 ID。
        parent_id: u64,
        /// 新子节点 ID。
        new_child_id: u64,
        /// 旧子节点 ID。
        old_child_id: u64,
    },
    /// element.cloneNode(deep) — 克隆节点。
    CloneNode {
        /// 源元素 ID。
        element_id: u64,
        /// 是否深拷贝。
        deep: bool,
    },
    /// element.style — 获取元素的 inline style 字符串。
    GetStyle {
        /// 元素 ID。
        element_id: u64,
    },
    /// element.style = value — 设置 inline style。
    SetStyle {
        /// 元素 ID。
        element_id: u64,
        /// CSS 文本。
        value: String,
    },
    /// element.innerHTML = value — 设置 innerHTML。
    SetInnerHtml {
        /// 元素 ID。
        element_id: u64,
        /// HTML 内容。
        value: String,
    },
    /// element.parentNode — 获取父节点。
    GetParentNode {
        /// 元素 ID。
        element_id: u64,
    },
}

/// DOM 命令执行结果。
#[derive(Debug, Clone, PartialEq)]
pub enum DomResult {
    /// 返回元素 ID（找不到为 None）。
    Element(Option<u64>),
    /// 返回元素 ID 列表。
    ElementList(Vec<u64>),
    /// 返回字符串值。
    String(Option<String>),
    /// 返回布尔值。
    Bool(bool),
    /// 返回空（void 操作）。
    Void,
    /// 操作出错。
    Error(String),
}

/// DOM Bridge — 管理 JS 与 DOM 之间的命令/响应映射。
///
/// 维护一个 JS 可见的"虚拟 DOM"句柄映射，将 JS 侧的
/// 元素引用（句柄 ID）映射到实际 DOM NodeId。
pub struct DomBridge {
    /// JS 句柄 → DOM NodeId 的映射。
    handle_map: std::collections::HashMap<u64, u64>,
    /// DOM NodeId → JS 句柄的反向映射（O(1) 双向查找）。
    reverse_map: std::collections::HashMap<u64, u64>,
    /// 下一个可用的 JS 句柄 ID。
    next_handle: u64,
}

impl DomBridge {
    /// 创建新的 DOM 桥接器。
    pub fn new() -> Self {
        Self {
            handle_map: std::collections::HashMap::new(),
            reverse_map: std::collections::HashMap::new(),
            next_handle: 1,
        }
    }

    /// 注册一个 DOM NodeId，返回 JS 可用的句柄 ID。
    ///
    /// 如果该 NodeId 已经注册过，返回已有句柄。
    pub fn register(&mut self, node_id: u64) -> u64 {
        // O(1) 反向查找检查是否已注册
        if let Some(&handle) = self.reverse_map.get(&node_id) {
            return handle;
        }
        let handle = self.next_handle;
        self.next_handle += 1;
        self.handle_map.insert(handle, node_id);
        self.reverse_map.insert(node_id, handle);
        handle
    }

    /// 通过句柄查找 DOM NodeId。
    pub fn resolve(&self, handle: u64) -> Option<u64> {
        self.handle_map.get(&handle).copied()
    }

    /// 移除句柄映射。
    pub fn unregister(&mut self, handle: u64) {
        if let Some(node_id) = self.handle_map.remove(&handle) {
            self.reverse_map.remove(&node_id);
        }
    }

    /// 当前注册的句柄数量。
    pub fn len(&self) -> usize {
        self.handle_map.len()
    }

    /// 是否没有注册句柄。
    pub fn is_empty(&self) -> bool {
        self.handle_map.is_empty()
    }

    /// 清除所有映射。
    pub fn clear(&mut self) {
        self.handle_map.clear();
        self.reverse_map.clear();
    }

    /// 解析 DOM 命令字符串，返回结构化的 DomCommand。
    ///
    /// 支持简单的命令格式：`document.getElementById("foo")`
    /// 这是一个简化实现，用于桥接层测试。
    pub fn parse_command(input: &str) -> Option<DomCommand> {
        let input = input.trim();

        // document.getElementById("id")
        if let Some(rest) = input.strip_prefix("document.getElementById(") {
            let id = extract_string_arg(rest)?;
            return Some(DomCommand::GetElementById { id });
        }

        // document.querySelector("selector")
        if let Some(rest) = input.strip_prefix("document.querySelector(") {
            let selector = extract_string_arg(rest)?;
            return Some(DomCommand::QuerySelector { selector });
        }

        // document.querySelectorAll("selector")
        if let Some(rest) = input.strip_prefix("document.querySelectorAll(") {
            let selector = extract_string_arg(rest)?;
            return Some(DomCommand::QuerySelectorAll { selector });
        }

        // document.createElement("tag")
        if let Some(rest) = input.strip_prefix("document.createElement(") {
            let tag_name = extract_string_arg(rest)?;
            return Some(DomCommand::CreateElement { tag_name });
        }

        // document.createTextNode("text")
        if let Some(rest) = input.strip_prefix("document.createTextNode(") {
            let text = extract_string_arg(rest)?;
            return Some(DomCommand::CreateTextNode { text });
        }

        // document.getElementsByClassName("class")
        if let Some(rest) = input.strip_prefix("document.getElementsByClassName(") {
            let class_name = extract_string_arg(rest)?;
            return Some(DomCommand::GetElementsByClassName { class_name });
        }

        // document.getElementsByTagName("tag")
        if let Some(rest) = input.strip_prefix("document.getElementsByTagName(") {
            let tag_name = extract_string_arg(rest)?;
            return Some(DomCommand::GetElementsByTagName { tag_name });
        }

        None
    }
}

impl Default for DomBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// 从函数参数中提取引号包裹的字符串参数。
///
/// 支持单引号和双引号：`"foo")` → `foo`，`'bar')` → `bar`
fn extract_string_arg(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let quote_char = input.chars().next()?;
    if quote_char != '"' && quote_char != '\'' {
        return None;
    }

    // 找到结束引号
    let end = input[1..].find(quote_char)?;
    let result = input[1..1 + end].to_string();

    // 验证后面跟着 )
    let rest = input[end + 2..].trim();
    if !rest.starts_with(')') {
        return None;
    }

    Some(result)
}

/// 生成注入到 JS 环境中的 DOM API polyfill 代码。
///
/// 此代码在 JS 引擎中创建 `document` 和基本 DOM API 的桩实现，
/// 实际操作通过桥接层转发到 Rust DOM。
pub fn generate_dom_api_polyfill() -> String {
    r#"(function() {
  // DOM API polyfill for ZeroBrowser
  // Elements are represented as plain objects with __nodeId
  var _nodeIdCounter = 1;
  var _nodeMap = {};

  function _createNode(type) {
    var id = _nodeIdCounter++;
    var node = { __nodeId: id, nodeType: type, children: [], attributes: {}, parentNode: null };
    _nodeMap[id] = node;
    return node;
  }

  globalThis.document = {
    createElement: function(tag) {
      var node = _createNode(1);
      node.tagName = tag.toUpperCase();
      node.nodeName = tag.toUpperCase();
      node.nodeType = 1;
      return node;
    },
    createTextNode: function(text) {
      var node = _createNode(3);
      node.textContent = text;
      node.nodeType = 3;
      return node;
    },
    createComment: function(text) {
      var node = _createNode(8);
      node.textContent = text;
      node.nodeType = 8;
      return node;
    },
    getElementById: function(id) {
      for (var nid in _nodeMap) {
        var node = _nodeMap[nid];
        if (node.attributes && node.attributes.id === id) return node;
      }
      return null;
    },
    querySelector: function(selector) {
      // Simplified: only supports tag, .class, #id selectors
      for (var nid in _nodeMap) {
        var node = _nodeMap[nid];
        if (_matchesSelector(node, selector)) return node;
      }
      return null;
    },
    querySelectorAll: function(selector) {
      var results = [];
      for (var nid in _nodeMap) {
        var node = _nodeMap[nid];
        if (_matchesSelector(node, selector)) results.push(node);
      }
      return results;
    },
    getElementsByClassName: function(className) {
      var results = [];
      for (var nid in _nodeMap) {
        var node = _nodeMap[nid];
        if (node.attributes && node.attributes['class']) {
          var classes = node.attributes['class'].split(/\s+/);
          if (classes.indexOf(className) >= 0) results.push(node);
        }
      }
      return results;
    },
    getElementsByTagName: function(tagName) {
      var results = [];
      var tag = tagName.toUpperCase();
      for (var nid in _nodeMap) {
        var node = _nodeMap[nid];
        if (node.tagName === tag) results.push(node);
      }
      return results;
    },
    createDocumentFragment: function() {
      var node = _createNode(11);
      node.nodeType = 11;
      return node;
    },
    title: '',
    body: _createNode(1),
    head: _createNode(1),
    documentElement: _createNode(1)
  };
  document.body.tagName = 'BODY';
  document.head.tagName = 'HEAD';
  document.documentElement.tagName = 'HTML';

  // Element.prototype methods (simplified)
  var _elementProto = {
    appendChild: function(child) {
      if (child.parentNode) {
        child.parentNode.children = child.parentNode.children.filter(function(c) { return c !== child; });
      }
      this.children.push(child);
      child.parentNode = this;
      return child;
    },
    removeChild: function(child) {
      this.children = this.children.filter(function(c) { return c !== child; });
      child.parentNode = null;
      return child;
    },
    insertBefore: function(newChild, refChild) {
      if (newChild.parentNode) {
        newChild.parentNode.children = newChild.parentNode.children.filter(function(c) { return c !== newChild; });
      }
      if (!refChild) {
        this.children.push(newChild);
      } else {
        var idx = this.children.indexOf(refChild);
        if (idx === -1) throw new Error('NotFoundError');
        this.children.splice(idx, 0, newChild);
      }
      newChild.parentNode = this;
      return newChild;
    },
    replaceChild: function(newChild, oldChild) {
      var idx = this.children.indexOf(oldChild);
      if (idx === -1) throw new Error('NotFoundError');
      if (newChild.parentNode) {
        newChild.parentNode.children = newChild.parentNode.children.filter(function(c) { return c !== newChild; });
      }
      this.children[idx] = newChild;
      newChild.parentNode = this;
      oldChild.parentNode = null;
      return oldChild;
    },
    cloneNode: function(deep) {
      var clone = document.createElement(this.tagName ? this.tagName.toLowerCase() : 'div');
      // 复制属性
      for (var key in this.attributes) {
        if (this.attributes.hasOwnProperty(key)) {
          clone.attributes[key] = this.attributes[key];
        }
      }
      clone.id = this.id || '';
      if (deep) {
        for (var i = 0; i < this.children.length; i++) {
          var child = this.children[i];
          if (child.nodeType === 1) {
            clone.appendChild(child.cloneNode(true));
          } else if (child.nodeType === 3) {
            clone.appendChild(document.createTextNode(child.textContent || ''));
          }
        }
      }
      return clone;
    },
    setAttribute: function(name, value) {
      this.attributes[name] = String(value);
      if (name === 'id') this.id = String(value);
      if (name === 'class') this.className = String(value);
    },
    getAttribute: function(name) {
      return this.attributes[name] || null;
    },
    removeAttribute: function(name) {
      delete this.attributes[name];
      if (name === 'id') this.id = '';
      if (name === 'class') this.className = '';
    },
    hasAttribute: function(name) {
      return name in this.attributes;
    },
    hasChildNodes: function() { return this.children.length > 0; },
    // ── matches / closest ──
    matches: function(selector) {
      return _matchesSelector(this, selector);
    },
    closest: function(selector) {
      var node = this;
      while (node) {
        if (_matchesSelector(node, selector)) return node;
        node = node.parentNode;
      }
      return null;
    },
    // ── querySelector / querySelectorAll on element ──
    querySelector: function(selector) {
      function _find(node) {
        if (node.nodeType === 1 && _matchesSelector(node, selector) && node !== this) return node;
        for (var i = 0; i < node.children.length; i++) {
          var result = _find.call(this, node.children[i]);
          if (result) return result;
        }
        return null;
      }
      for (var i = 0; i < this.children.length; i++) {
        if (_matchesSelector(this.children[i], selector)) return this.children[i];
        var result = this.querySelector.call(this.children[i], selector);
        if (result) return result;
      }
      return null;
    },
    querySelectorAll: function(selector) {
      var results = [];
      function _collect(node) {
        if (node.nodeType === 1 && _matchesSelector(node, selector) && node !== this) results.push(node);
        for (var i = 0; i < node.children.length; i++) {
          _collect(node.children[i]);
        }
      }
      for (var i = 0; i < this.children.length; i++) {
        if (_matchesSelector(this.children[i], selector)) results.push(this.children[i]);
        for (var j = 0; j < this.children[i].children.length; j++) {
          _collect(this.children[i].children[j]);
        }
      }
      return results;
    },
    // ── getBoundingClientRect stub ──
    getBoundingClientRect: function() {
      return { top: 0, right: 0, bottom: 0, left: 0, width: 0, height: 0, x: 0, y: 0 };
    },
    // ── textContent getter/setter ──
    getTextContent: function() {
      if (this.nodeType === 3) return this.textContent || '';
      var result = '';
      for (var i = 0; i < this.children.length; i++) {
        var child = this.children[i];
        if (child.nodeType === 3) {
          result += child.textContent || '';
        } else if (child.nodeType === 1) {
          result += child.getTextContent();
        }
      }
      return result;
    },
    setTextContent: function(value) {
      this.children = [];
      if (value) {
        var textNode = document.createTextNode(value);
        Object.assign(textNode, { parentNode: this });
        this.children.push(textNode);
      }
    },
    // ── Event System ──
    addEventListener: function(type, listener, options) {
      if (typeof listener !== 'function') return;
      if (!this._eventListeners) this._eventListeners = {};
      if (!this._eventListeners[type]) this._eventListeners[type] = [];
      var capture = (options === true) || (options && options.capture === true);
      this._eventListeners[type].push({ fn: listener, capture: capture });
    },
    removeEventListener: function(type, listener) {
      if (!this._eventListeners || !this._eventListeners[type]) return;
      this._eventListeners[type] = this._eventListeners[type].filter(function(l) {
        return l.fn !== listener;
      });
    },
    dispatchEvent: function(event) {
      if (!this._eventListeners) return true;
      var listeners = this._eventListeners[event.type];
      if (!listeners) return true;
      event.target = this;
      event.currentTarget = this;
      // Capture phase (simplified: just call capture listeners first)
      for (var i = 0; i < listeners.length; i++) {
        if (listeners[i].capture) {
          listeners[i].fn.call(this, event);
          if (event._immediatePropagationStopped) return !event._defaultPrevented;
        }
      }
      // Target + bubble phase
      for (var i = 0; i < listeners.length; i++) {
        if (!listeners[i].capture) {
          listeners[i].fn.call(this, event);
          if (event._immediatePropagationStopped) return !event._defaultPrevented;
        }
      }
      return !event._defaultPrevented;
    }
  };

  // CustomEvent constructor
  globalThis.CustomEvent = function(type, options) {
    this.type = type;
    this.bubbles = (options && options.bubbles) || false;
    this.cancelable = (options && options.cancelable) || false;
    this.detail = (options && options.detail) || null;
    this.target = null;
    this.currentTarget = null;
    this._defaultPrevented = false;
    this._propagationStopped = false;
    this._immediatePropagationStopped = false;
    this.preventDefault = function() { if (this.cancelable) this._defaultPrevented = true; };
    this.stopPropagation = function() { this._propagationStopped = true; };
    this.stopImmediatePropagation = function() { this._immediatePropagationStopped = true; this._propagationStopped = true; };
  };

  function _matchesSelector(node, selector) {
    if (!node.attributes) return false;
    // 支持逗号分隔的多选择器
    var parts = selector.split(',');
    for (var p = 0; p < parts.length; p++) {
      if (_matchesSingleSelector(node, parts[p].trim())) return true;
    }
    return false;
  }

  // 扫描复合选择器链的组合器边界（` `/`>`/`+`/`~`，忽略 `[]`/引号内），返
  // { segs: [compound-str...], combs: [comb...] }——`combs[i]` 连接 `segs[i]`↔`segs[i+1]`，
  // 取值 `' '`/`'>'`/`'+'`/`'~'`。无组合器（单 compound）返 null（交回单选择器分支）。
  // 显式符号（>`+`~`）覆盖空白触发的后代边界（`h1 + p` 中 `+` 前空白不误记为后代）。
  // 与 R3285/R3286 四组合器一致化系列对齐（CSS Selectors L3 §14）。
  function _splitCombinatorChain(text) {
    var segs = [], combs = [];
    var cur = '', depth = 0, quote = null;
    var lastSegChar = false, pendingExplicit = null;
    var flush = function (comb) {
      if (cur) { segs.push(cur); combs.push(comb); cur = ''; }
      lastSegChar = false;
    };
    for (var i = 0; i < text.length; i++) {
      var ch = text[i];
      if (quote) { cur += ch; if (ch === quote) quote = null; lastSegChar = true; continue; }
      if (ch === '"' || ch === "'") { quote = ch; cur += ch; lastSegChar = true; continue; }
      if (ch === '[') { depth++; cur += ch; lastSegChar = true; continue; }
      if (ch === ']') { depth--; cur += ch; lastSegChar = true; continue; }
      // 括号 `(``)` 亦计入深度——`:nth-child(2n+1)` / `:not(.a)` / `:is(a, b)` 内的 `+`/`,`/` `
      // 非组合器边界（R3288 修复：旧仅计 `[]` 致 nth 公式 an+b 的 `+` 误判为相邻兄弟组合器）。
      if (ch === '(') { depth++; cur += ch; lastSegChar = true; continue; }
      if (ch === ')') { depth--; cur += ch; lastSegChar = true; continue; }
      if (depth === 0 && (ch === '>' || ch === '+' || ch === '~')) {
        if (!lastSegChar && combs.length > 0 && combs[combs.length - 1] === ' ') {
          combs[combs.length - 1] = ch;
        } else {
          pendingExplicit = ch;
        }
        lastSegChar = false;
        continue;
      }
      if (depth === 0 && /\s/.test(ch)) {
        if (lastSegChar && pendingExplicit === null) { flush(' '); }
        continue;
      }
      if (pendingExplicit !== null) { flush(pendingExplicit); pendingExplicit = null; }
      cur += ch;
      lastSegChar = true;
    }
    if (cur) segs.push(cur);
    if (segs.length < 2) return null;
    return { segs: segs, combs: combs.slice(0, segs.length - 1) };
  }

  // 从右起回溯求值复合链：节点 cur 须匹配 segs[idx]，再按 combs[idx-1] 回溯到 segs[idx-1] 候选。
  // `' '`（后代）与 `'~'`（通用兄弟）须回溯——选定目标须能继续匹配左侧链（如 `h1 + p ~ p`）。
  // `'>'`（子代）取 parentNode、`'+'`（相邻兄弟）取紧邻 previousSibling，均无回溯。
  function _matchChainSeg(cur, segs, combs, idx) {
    if (!cur || !_matchesSingleSelector(cur, segs[idx])) return false;
    if (idx === 0) return true;
    var comb = combs[idx - 1], left = idx - 1;
    if (comb === ' ') {
      // 后代：沿 parentNode 链找任一匹配祖先（回溯）。
      var a = cur.parentNode;
      while (a) {
        if (_matchChainSeg(a, segs, combs, left)) return true;
        a = a.parentNode;
      }
      return false;
    } else if (comb === '>') {
      var p = cur.parentNode;
      return p && _matchChainSeg(p, segs, combs, left);
    } else if (comb === '+') {
      var ps = cur.previousSibling;
      return ps && _matchChainSeg(ps, segs, combs, left);
    } else if (comb === '~') {
      // 通用兄弟：沿 previousSibling 链找任一匹配（回溯）。
      var s = cur.previousSibling;
      while (s) {
        if (_matchChainSeg(s, segs, combs, left)) return true;
        s = s.previousSibling;
      }
      return false;
    }
    return false;
  }

  function _matchesSingleSelector(node, selector) {
    if (!selector || !node.attributes) return false;

    // 复合选择器链（含组合器 ` `/`>`/`+`/`~`）。先扫描组合器边界（忽略 `[]`/引号内），按四组合器
    // 从右起回溯求值——与 R3285（DOM crate）/ R3286（B 代 shim）四组合器一致化系列对齐。
    // 旧实现按 `if indexOf(' ')` / `indexOf('>')` 顺序短路，致 `h1 + p`（含空格）误入后代分支、
    // `a > b`（含空格）误入后代分支——组合器检测须先于空白后代判定。
    var chain = _splitCombinatorChain(selector);
    if (chain) {
      var segs = chain.segs, combs = chain.combs;
      return _matchChainSeg(node, segs, combs, segs.length - 1);
    }

    // 单复合选择器（无组合器）：解析为 { tag, ids, classes, attrs, pseudos } 后逐部分求值。
    // 旧实现按 `#`/`.`/`[`/`:`/tag 顺序短路，致复合选择器（如 `div.foo:first-child`、`input:checked`、
    // `a:not(.x)`）无法工作——tag 分支先吞掉复合串。R3288 重构为真复合匹配，与 DOM crate
    // parse_simple_selector + B 代 shim _parseCompoundOf 能力对齐（伪类面一致化）。
    return _matchCompound(node, selector);

  // 解析单复合选择器（无组合器）为 { tag, ids, classes, attrs, pseudos }。
  // tag 为 null（`*` / 无）或大写 tag；ids/classes/attrs/pseudos 数组。遇非法段返 null（不匹配）。
  function _parseCompound(text) {
    var c = { tag: null, ids: [], classes: [], attrs: [], pseudos: [] };
    var i = 0, n = text.length, seenTag = false;
    while (i < n) {
      var ch = text[i];
      if (ch === '.') {
        var j = i + 1;
        while (j < n && !'.#[:'.includes(text[j])) j++;
        if (j === i + 1) return null;
        c.classes.push(text.substring(i + 1, j)); i = j;
      } else if (ch === '#') {
        var j2 = i + 1;
        while (j2 < n && !'.#[:'.includes(text[j2])) j2++;
        if (j2 === i + 1) return null;
        c.ids.push(text.substring(i + 1, j2)); i = j2;
      } else if (ch === '[') {
        var end = text.indexOf(']', i);
        if (end < 0) return null;
        var inner = text.substring(i + 1, end);
        var am = inner.match(/^([\w:-]+)\s*(?:([~|^$*]?=)\s*(.*?))?\s*$/);
        if (!am) return null;
        var val = am[3];
        if (val != null) val = String(val).replace(/^['"]|['"]$/g, '');
        c.attrs.push({ name: am[1], op: am[2] || null, val: val == null ? '' : val });
        i = end + 1;
      } else if (ch === ':') {
        // 伪类名（含括号参数，括号内不切分）。`::` 伪元素视作伪类名前缀（保守不匹配）。
        var j3 = i + 1;
        var depthP = 0;
        while (j3 < n) {
          var cj = text[j3];
          if (cj === '(') depthP++;
          else if (cj === ')') depthP--;
          else if (depthP === 0 && '.#[:'.includes(cj)) break;
          j3++;
        }
        if (j3 === i + 1) return null;
        c.pseudos.push(text.substring(i + 1, j3)); i = j3;
      } else {
        // 裸 token = tag（首个），后续裸 token 非法（复合选择器不可有两个 tag）。
        var jt = i;
        while (jt < n && !'.#[:'.includes(text[jt])) jt++;
        var tg = text.substring(i, jt);
        if (!tg) return null;
        if (!seenTag) { c.tag = tg === '*' ? null : tg.toUpperCase(); seenTag = true; }
        else return null;
        i = jt;
      }
    }
    return c;
  }

  // 单复合选择器匹配：节点须满足全部部分（tag/ids/classes/attrs/pseudos AND 语义）。
  function _matchCompound(node, selector) {
    var c = _parseCompound(selector);
    if (!c) return false;
    if (c.tag && node.tagName !== c.tag) return false;
    // A-gen id 经 .id= 或 setAttribute('id',) 设置——两处不同步，id 选择器须兼容读取。
    var idVal = node.attributes.id != null ? node.attributes.id
      : (typeof node.id === 'string' ? node.id : '');
    for (var k = 0; k < c.ids.length; k++) {
      if (idVal !== c.ids[k]) return false;
    }
    if (c.classes.length) {
      // A-gen className 为普通属性，与 attributes['class'] 不同步（经 .className= 或 setAttribute('class',)
      // 任一设置）；class 选择器须两处都查（与 getElementsByClassName 同源：仅读 attributes['class']，
      // 但 .className= 高频路径无对应属性 → 兼容读取，避免 class 选择器对 .className= 元素恒不匹配）。
      var raw = node.attributes['class'] != null ? node.attributes['class']
        : (typeof node.className === 'string' ? node.className : '');
      var cls = raw ? raw.split(/\s+/) : [];
      for (var k2 = 0; k2 < c.classes.length; k2++) {
        if (cls.indexOf(c.classes[k2]) < 0) return false;
      }
    }
    for (var k3 = 0; k3 < c.attrs.length; k3++) {
      if (!_matchAttr(node, c.attrs[k3])) return false;
    }
    for (var k4 = 0; k4 < c.pseudos.length; k4++) {
      if (!_matchPseudo(node, c.pseudos[k4])) return false;
    }
    return true;
  }

  // 属性选择器匹配（= / ~= / |= / ^= / $= / *= / 存在性）。
  function _matchAttr(node, a) {
    var av = node.attributes[a.name];
    if (av == null) return a.op === null && false; // 缺属性 → 除存在性（也无）外均 false
    if (a.op === null) return true;
    var v = String(av);
    switch (a.op) {
      case '=': return v === a.val;
      case '~=': return a.val !== '' && v.split(/\s+/).indexOf(a.val) >= 0;
      case '|=': return v === a.val || v.indexOf(a.val + '-') === 0;
      case '^=': return a.val !== '' && v.indexOf(a.val) === 0;
      case '$=': return a.val !== '' && v.lastIndexOf(a.val) === v.length - a.val.length;
      case '*=': return a.val !== '' && v.indexOf(a.val) >= 0;
    }
    return false;
  }

  // 静态可判定伪类匹配（与 DOM crate R3277-R3284 系列 + B 代 shim 对齐）。交互态伪类
  //（`:hover`/`:focus`/`:active` 等）headless 无交互态 → 保守 false。`:visited` 隐私安全恒 false。
  function _matchPseudo(node, name) {
    var lc = name.toLowerCase();
    if (lc === 'first-child') {
      return !!node.parentNode && node.parentNode.children[0] === node;
    }
    if (lc === 'last-child') {
      return !!node.parentNode && node.parentNode.children[node.parentNode.children.length - 1] === node;
    }
    if (lc === 'only-child') {
      return !!node.parentNode && node.parentNode.children.length === 1 && node.parentNode.children[0] === node;
    }
    if (lc === 'empty') {
      // 无元素子且无非空文本子（A-gen 节点 .children 含 appendChild 的节点；text/comment 经 createTextNode/
      // createComment 产生但本 polyfill 不挂 .children，故 empty ≈ 无 .children 子）。spec：无子节点（含文本）。
      return node.children.length === 0;
    }
    if (lc === 'root') {
      // 文档根元素：无元素父（parentNode 为 document 或 null）。
      return !node.parentNode || !node.parentNode.attributes;
    }
    if (lc === 'checked') {
      // checkbox/radio checked 属性存在、option selected 属性存在（静态 HTML 语义）。
      if ('checked' in node.attributes) return true;
      return 'selected' in node.attributes;
    }
    if (lc === 'disabled') return 'disabled' in node.attributes;
    if (lc === 'enabled') return !('disabled' in node.attributes);
    if (lc === 'required') return 'required' in node.attributes;
    if (lc === 'optional') return !('required' in node.attributes);
    if (lc === 'readonly') return 'readonly' in node.attributes || 'disabled' in node.attributes;
    if (lc === 'read-write') return !('readonly' in node.attributes) && !('disabled' in node.attributes);
    if (lc === 'visited' || lc === 'hover' || lc === 'focus' || lc === 'active'
        || lc === 'focus-within' || lc === 'focus-visible') {
      return false; // 交互态 / 隐私 → 保守 false
    }
    var nth = lc.match(/^nth-child\((.+)\)$/);
    if (nth && node.parentNode) {
      var sibs = node.parentNode.children;
      var pos = sibs.indexOf(node) + 1; // 1-based
      return _matchNth(nth[1], pos);
    }
    var nthLast = lc.match(/^nth-last-child\((.+)\)$/);
    if (nthLast && node.parentNode) {
      var sibs2 = node.parentNode.children;
      var pos2 = sibs2.length - sibs2.indexOf(node); // 从末尾 1-based
      return _matchNth(nthLast[1], pos2);
    }
    // :not(simple)——否定（内嵌经 _matchesSingleSelector 递归，可含复合，不含组合器——组合器须外层链）。
    var notM = lc.match(/^not\((.+)\)$/);
    if (notM) {
      // 内嵌不含逗号列表的简化（spec 允许选择器列表，此处单 simple）。
      return !_matchesSingleSelector(node, notM[1].trim());
    }
    return false; // 未识别伪类 → 保守不匹配
  }

  // `:nth-child(an+b)` 求值：支持 odd/even/纯整数/n/an+b。pos 为 1-based 位置。
  function _matchNth(expr, pos) {
    var s = String(expr).trim().toLowerCase();
    if (s === 'odd') return pos % 2 === 1;
    if (s === 'even') return pos % 2 === 0;
    var m = s.match(/^(?:(-?\d*)n)?\s*([+-]?\d+)?$/);
    if (m) {
      var a = m[1];
      var aVal;
      if (a === undefined) {
        // 无 `n`（纯整数 b，如 "3"）—— a=0，匹配 pos===b。
        aVal = 0;
      } else if (a === '' || a === '+') {
        aVal = 1; // 裸 `n` / `+n` → a=1
      } else if (a === '-') {
        aVal = -1; // `-n` → a=-1
      } else {
        aVal = parseInt(a, 10); // `2n` / `-3n` → a=系数
      }
      var b = m[2] ? parseInt(m[2], 10) : 0;
      // pos = aVal*k + b，k ≥ 0 整数 → (pos - b) / aVal ≥ 0 且整除（aVal=0 时 pos===b）。
      if (aVal === 0) return pos === b;
      var k = (pos - b) / aVal;
      return k >= 0 && k === Math.floor(k);
    }
    // 纯整数
    var pure = parseInt(s, 10);
    if (!isNaN(pure)) return pos === pure;
    return false;
  }
  }

  // Mix in element methods to all created nodes
  var origCreateElement = document.createElement.bind(document);
  document.createElement = function(tag) {
    var node = origCreateElement(tag);
    Object.assign(node, _elementProto);
    // ── 导航属性（getter 需要在 Object.assign 之后定义，否则赋值时触发 getter） ──
    Object.defineProperty(node, 'firstChild', {
      get: function() { return this.children.length > 0 ? this.children[0] : null; },
      configurable: true
    });
    Object.defineProperty(node, 'lastChild', {
      get: function() { return this.children.length > 0 ? this.children[this.children.length - 1] : null; },
      configurable: true
    });
    Object.defineProperty(node, 'nextSibling', {
      get: function() {
        if (!this.parentNode) return null;
        var idx = this.parentNode.children.indexOf(this);
        return idx >= 0 && idx < this.parentNode.children.length - 1 ? this.parentNode.children[idx + 1] : null;
      },
      configurable: true
    });
    Object.defineProperty(node, 'previousSibling', {
      get: function() {
        if (!this.parentNode) return null;
        var idx = this.parentNode.children.indexOf(this);
        return idx > 0 ? this.parentNode.children[idx - 1] : null;
      },
      configurable: true
    });
    Object.defineProperty(node, 'childElementCount', {
      get: function() { return this.children.length; },
      configurable: true
    });
    // ── style 属性（CSSStyleDeclaration 简化实现） ──
    node.style = new _CSSStyleDeclaration();
    // ── classList 属性（DOMTokenList 简化实现） ──
    node.classList = new _DOMTokenList(node, 'class');
    // ── id / className getter+setter ──
    node.id = '';
    node.className = '';
    // ── innerHTML setter ──
    Object.defineProperty(node, 'innerHTML', {
      get: function() {
        var html = '';
        for (var i = 0; i < this.children.length; i++) {
          var child = this.children[i];
          if (child.nodeType === 3) {
            html += child.textContent;
          } else if (child.nodeType === 1) {
            html += '<' + child.tagName.toLowerCase();
            for (var attr in child.attributes) {
              if (child.attributes.hasOwnProperty(attr)) {
                html += ' ' + attr + '="' + child.attributes[attr] + '"';
              }
            }
            html += '>';
            if (child.innerHTML !== undefined) html += child.innerHTML;
            html += '</' + child.tagName.toLowerCase() + '>';
          }
        }
        return html;
      },
      set: function(value) {
        this.children = [];
        // 简化实现：将 innerHTML 内容作为单个文本节点
        if (value) {
          var textNode = document.createTextNode(value);
          Object.assign(textNode, { parentNode: this });
          this.children.push(textNode);
        }
      },
      configurable: true
    });
    // ── outerHTML getter ──
    Object.defineProperty(node, 'outerHTML', {
      get: function() {
        var tag = this.tagName ? this.tagName.toLowerCase() : 'div';
        var attrs = '';
        for (var attr in this.attributes) {
          if (this.attributes.hasOwnProperty(attr)) {
            attrs += ' ' + attr + '="' + this.attributes[attr] + '"';
          }
        }
        return '<' + tag + attrs + '>' + this.innerHTML + '</' + tag + '>';
      },
      configurable: true
    });
    // ── textContent getter+setter ──
    Object.defineProperty(node, 'textContent', {
      get: function() { return this.getTextContent(); },
      set: function(value) { this.setTextContent(value); },
      configurable: true
    });
    // ── innerText getter+setter（简化：等同 textContent） ──
    Object.defineProperty(node, 'innerText', {
      get: function() { return this.getTextContent(); },
      set: function(value) { this.setTextContent(value); },
      configurable: true
    });
    return node;
  };

  // ── CSSStyleDeclaration 简化实现 ──
  // 支持 cssText getter/setter 和按属性名读写。
  function _CSSStyleDeclaration() {
    this._props = {};
  }
  _CSSStyleDeclaration.prototype.getPropertyValue = function(prop) {
    return this._props[prop] || '';
  };
  _CSSStyleDeclaration.prototype.setProperty = function(prop, value) {
    this._props[prop] = value;
  };
  _CSSStyleDeclaration.prototype.removeProperty = function(prop) {
    var v = this._props[prop] || '';
    delete this._props[prop];
    return v;
  };
  Object.defineProperty(_CSSStyleDeclaration.prototype, 'cssText', {
    get: function() {
      var parts = [];
      for (var k in this._props) {
        if (this._props.hasOwnProperty(k)) parts.push(k + ': ' + this._props[k]);
      }
      return parts.join('; ');
    },
    set: function(text) {
      this._props = {};
      if (!text) return;
      var decls = text.split(';');
      for (var i = 0; i < decls.length; i++) {
        var decl = decls[i].trim();
        if (!decl) continue;
        var colon = decl.indexOf(':');
        if (colon === -1) continue;
        var prop = decl.substring(0, colon).trim();
        var val = decl.substring(colon + 1).trim();
        if (prop) this._props[prop] = val;
      }
    },
    configurable: true
  });

  // ── DOMTokenList 简化实现（用于 classList） ──
  function _DOMTokenList(element, attrName) {
    this._element = element;
    this._attrName = attrName;
  }
  _DOMTokenList.prototype._tokens = function() {
    var v = this._element.getAttribute(this._attrName) || '';
    return v ? v.split(/\s+/).filter(function(t) { return t; }) : [];
  };
  _DOMTokenList.prototype._sync = function(tokens) {
    var val = tokens.join(' ');
    this._element.setAttribute(this._attrName, val);
    if (this._attrName === 'class') this._element.className = val;
  };
  Object.defineProperty(_DOMTokenList.prototype, 'length', {
    get: function() { return this._tokens().length; }
  });
  _DOMTokenList.prototype.item = function(index) { return this._tokens()[index] || null; };
  _DOMTokenList.prototype.contains = function(token) { return this._tokens().indexOf(token) !== -1; };
  _DOMTokenList.prototype.add = function() {
    var tokens = this._tokens();
    for (var i = 0; i < arguments.length; i++) {
      if (tokens.indexOf(arguments[i]) === -1) tokens.push(arguments[i]);
    }
    this._sync(tokens);
  };
  _DOMTokenList.prototype.remove = function() {
    var tokens = this._tokens();
    for (var i = 0; i < arguments.length; i++) {
      var idx = tokens.indexOf(arguments[i]);
      if (idx !== -1) tokens.splice(idx, 1);
    }
    this._sync(tokens);
  };
  _DOMTokenList.prototype.toggle = function(token) {
    if (this.contains(token)) { this.remove(token); return false; }
    else { this.add(token); return true; }
  };
  _DOMTokenList.prototype.replace = function(oldToken, newToken) {
    var tokens = this._tokens();
    var idx = tokens.indexOf(oldToken);
    if (idx === -1) return false;
    tokens[idx] = newToken;
    this._sync(tokens);
    return true;
  };

  // ── 为预创建的 document 节点混入元素方法 ──
  // document.body/head/documentElement 在 _elementProto/CSSStyleDeclaration/DOMTokenList
  // 定义之前创建，需要在这里补齐方法。
  function _mixinElementMethods(node) {
    Object.assign(node, _elementProto);
    node.style = new _CSSStyleDeclaration();
    node.classList = new _DOMTokenList(node, 'class');
    node.id = '';
    node.className = '';
  }
  _mixinElementMethods(document.body);
  _mixinElementMethods(document.head);
  _mixinElementMethods(document.documentElement);

  // ── Fetch API Stub ──
  // Provides globalThis.fetch, Headers, Request, Response constructors.
  // fetch() returns a stub Response (status 200, empty body) since real
  // network access is handled by the host runtime.

  globalThis.Headers = function(init) {
    this._headers = {};
    if (init) {
      if (typeof init === 'object') {
        for (var key in init) {
          if (init.hasOwnProperty(key)) this._headers[key.toLowerCase()] = String(init[key]);
        }
      }
    }
  };
  globalThis.Headers.prototype.append = function(name, value) { this._headers[name.toLowerCase()] = String(value); };
  globalThis.Headers.prototype.delete = function(name) { delete this._headers[name.toLowerCase()]; };
  globalThis.Headers.prototype.get = function(name) { return this._headers[name.toLowerCase()] || null; };
  globalThis.Headers.prototype.has = function(name) { return name.toLowerCase() in this._headers; };
  globalThis.Headers.prototype.set = function(name, value) { this._headers[name.toLowerCase()] = String(value); };

  globalThis.Request = function(input, init) {
    init = init || {};
    this.url = typeof input === 'string' ? input : (input && input.url || '');
    this.method = init.method || 'GET';
    this.headers = new globalThis.Headers(init.headers || {});
    this.body = init.body || null;
    this._signal = init.signal || null;
  };

  globalThis.Response = function(body, init) {
    init = init || {};
    this.body = body;
    this.status = init.status || 200;
    this.statusText = init.statusText || 'OK';
    this.headers = new globalThis.Headers(init.headers || {});
    this.ok = this.status >= 200 && this.status < 300;
    this.type = 'default';
    this.url = init.url || '';
  };
  globalThis.Response.prototype.json = function() {
    return Promise.resolve(JSON.parse(this.body));
  };
  globalThis.Response.prototype.text = function() {
    return Promise.resolve(this.body || '');
  };
  globalThis.Response.prototype.clone = function() {
    return new globalThis.Response(this.body, {
      status: this.status,
      statusText: this.statusText,
      headers: this.headers._headers,
      url: this.url
    });
  };
  globalThis.Response.error = function() {
    var r = new globalThis.Response(null, { status: 0, statusText: '' });
    r.type = 'error';
    return r;
  };

  // ── P1a fetch 切片：data: URL 同步真实解码 ──
  // data: URL 无需 host 网络/事件循环即可在 polyfill 内同步解码为真实 body，
  // 是 fetch「真实化」首个可独立落地的切片（http(s)/blob 仍 stub，须事件循环 + net）。

  // 纯 JS base64 解码（polyfill 无 atob）→ Latin-1 字符串（ASCII 文本正确；
  // 多字节 UTF-8 base64 为已知限制，binary body 后续 ArrayBuffer 切片处理）。
  function _b64decode(str) {
    var A = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
    var lut = {};
    for (var i = 0; i < 64; i++) lut[A[i]] = i;
    var s = String(str).replace(/=+$/, '');
    var out = '';
    for (var i = 0; i < s.length; i += 4) {
      var c0 = s[i], c1 = s[i + 1], c2 = s[i + 2], c3 = s[i + 3];
      var b0 = lut[c0] || 0, b1 = lut[c1] || 0, b2 = lut[c2], b3 = lut[c3];
      out += String.fromCharCode((b0 << 2) | (b1 >> 4));
      if (c2 !== undefined) out += String.fromCharCode(((b1 & 15) << 4) | ((b2 || 0) >> 2));
      if (c3 !== undefined) out += String.fromCharCode(((b2 || 0) & 3) << 6 | (b3 || 0));
    }
    return out;
  }

  // 解析 data: URL → { body, contentType } 或 null（非 data: URL）。
  // 格式：data:[<mediatype>][;base64],<data>；空 mediatype 默认 text/plain;charset=US-ASCII。
  function _decodeDataUrl(url) {
    if (typeof url !== 'string' || url.substring(0, 5) !== 'data:') return null;
    var commaIdx = url.indexOf(',');
    if (commaIdx < 0) return null;
    var meta = url.substring(5, commaIdx);
    var data = url.substring(commaIdx + 1);
    var isBase64 = meta.slice(-7) === ';base64';
    var contentType = isBase64 ? meta.slice(0, -7) : meta;
    if (contentType === '') contentType = 'text/plain;charset=US-ASCII';
    var body;
    if (isBase64) {
      body = _b64decode(data);
    } else {
      try { body = decodeURIComponent(data); } catch (e) { body = data; }
    }
    return { body: body, contentType: contentType };
  }

  // 同步构造 data: URL 的真实 Response（供 fetch 与测试共用）。
  function _fetchDataUrlSync(url) {
    var d = _decodeDataUrl(url);
    if (!d) return null;
    return new globalThis.Response(d.body, {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': d.contentType },
      url: url
    });
  }
  // 暴露同步测试钩子（fetch 返回 Promise，V8 桩不保证 .then 同步执行）。
  globalThis.__fetchDataUrlSync = _fetchDataUrlSync;

  globalThis.fetch = function(input, init) {
    var req = (input instanceof globalThis.Request) ? input : new globalThis.Request(input, init);
    // P1a 切片：data: URL 同步真实解码；非 data: URL 仍 stub（真网络须事件循环 + net）。
    var dataResp = _fetchDataUrlSync(req.url);
    if (dataResp) return Promise.resolve(dataResp);
    return Promise.resolve(new globalThis.Response(null, {
      status: 200,
      statusText: 'OK',
      headers: { 'content-type': 'text/plain' },
      url: req.url
    }));
  };

  // ── Console API Stub ──
  // Provides console.log/warn/error/info/debug/trace/time/timeEnd.

  var _consoleTimers = {};
  globalThis.console = {
    log: function() { /* stub: output handled by host runtime */ },
    warn: function() {},
    error: function() {},
    info: function() {},
    debug: function() {},
    trace: function() {},
    time: function(label) { _consoleTimers[label || 'default'] = Date.now(); },
    timeEnd: function(label) { var key = label || 'default'; delete _consoleTimers[key]; },
    assert: function(condition) { if (!condition) { /* stub */ } },
    clear: function() {},
    count: function() {},
    group: function() {},
    groupEnd: function() {},
    table: function() {}
  };

  // ── Timer API Stub ──
  // Provides setTimeout/setInterval/clearTimeout/clearInterval.
  // Real timing handled by host runtime event loop.

  globalThis.setTimeout = function(fn, delay) {
    if (typeof fn === 'function') fn();
    return 0;
  };
  globalThis.setInterval = function(fn, delay) {
    if (typeof fn === 'function') fn();
    return 0;
  };
  globalThis.clearTimeout = function(id) {};
  globalThis.clearInterval = function(id) {};

  // ── Web Storage API Stub ──
  // Provides localStorage and sessionStorage with full Storage interface.
  // Data is session-scoped (not persisted to disk); real persistence by host runtime.

  function _createStorage() {
    var _data = {};
    return {
      getItem: function(key) { return _data.hasOwnProperty(key) ? _data[key] : null; },
      setItem: function(key, value) { _data[String(key)] = String(value); },
      removeItem: function(key) { delete _data[String(key)]; },
      clear: function() { _data = {}; },
      key: function(index) {
        var keys = Object.keys(_data);
        return index >= 0 && index < keys.length ? keys[index] : null;
      },
      get length() { return Object.keys(_data).length; }
    };
  }

  globalThis.localStorage = _createStorage();
  globalThis.sessionStorage = _createStorage();

  // ── MutationObserver Stub ──
  // Provides MutationObserver with observe/disconnect/takeRecords.
  // Real observation by host runtime; stub records mutations for testing.

  globalThis.MutationObserver = function(callback) {
    this._callback = callback;
    this._records = [];
    this._observing = false;
  };
  globalThis.MutationObserver.prototype.observe = function(target, options) {
    this._observing = true;
    this._target = target;
    this._options = options || {};
  };
  globalThis.MutationObserver.prototype.disconnect = function() {
    this._observing = false;
    this._target = null;
    this._options = null;
  };
  globalThis.MutationObserver.prototype.takeRecords = function() {
    var records = this._records;
    this._records = [];
    return records;
  };

  // MutationRecord constructor
  globalThis.MutationRecord = function(type, target) {
    this.type = type;
    this.target = target;
    this.addedNodes = [];
    this.removedNodes = [];
    this.previousSibling = null;
    this.nextSibling = null;
    this.attributeName = null;
    this.attributeNamespace = null;
    this.oldValue = null;
  };

  // ── IntersectionObserver Stub ──
  // Provides IntersectionObserver with observe/unobserve/disconnect.
  // Real intersection computation by host runtime.

  globalThis.IntersectionObserver = function(callback, options) {
    this._callback = callback;
    this._options = options || {};
    this._observing = [];
  };
  globalThis.IntersectionObserver.prototype.observe = function(target) {
    if (this._observing.indexOf(target) === -1) {
      this._observing.push(target);
    }
  };
  globalThis.IntersectionObserver.prototype.unobserve = function(target) {
    var idx = this._observing.indexOf(target);
    if (idx !== -1) this._observing.splice(idx, 1);
  };
  globalThis.IntersectionObserver.prototype.disconnect = function() {
    this._observing = [];
  };
  globalThis.IntersectionObserver.prototype.takeRecords = function() {
    return [];
  };

  // IntersectionObserverEntry constructor
  globalThis.IntersectionObserverEntry = function(init) {
    this.time = init.time || 0;
    this.rootBounds = init.rootBounds || null;
    this.boundingClientRect = init.boundingClientRect || null;
    this.intersectionRect = init.intersectionRect || null;
    this.isIntersecting = init.isIntersecting || false;
    this.target = init.target || null;
    this.intersectionRatio = init.intersectionRatio || 0;
  };

  // ── ResizeObserver Stub ──
  // Provides ResizeObserver with observe/unobserve/disconnect.
  // Real resize detection by host runtime.

  globalThis.ResizeObserver = function(callback) {
    this._callback = callback;
    this._observing = [];
  };
  globalThis.ResizeObserver.prototype.observe = function(target) {
    if (this._observing.indexOf(target) === -1) {
      this._observing.push(target);
    }
  };
  globalThis.ResizeObserver.prototype.unobserve = function(target) {
    var idx = this._observing.indexOf(target);
    if (idx !== -1) this._observing.splice(idx, 1);
  };
  globalThis.ResizeObserver.prototype.disconnect = function() {
    this._observing = [];
  };

  // ResizeObserverEntry constructor
  globalThis.ResizeObserverEntry = function(target, contentRect) {
    this.target = target;
    this.contentRect = contentRect || { x: 0, y: 0, width: 0, height: 0, top: 0, right: 0, bottom: 0, left: 0 };
  };

  // DOMRectReadOnly stub (used by ResizeObserver)
  globalThis.DOMRectReadOnly = function(x, y, width, height) {
    this.x = x || 0;
    this.y = y || 0;
    this.width = width || 0;
    this.height = height || 0;
    this.top = this.y;
    this.right = this.x + this.width;
    this.bottom = this.y + this.height;
    this.left = this.x;
  };

  // ── PerformanceObserver Stub ──
  // Provides PerformanceObserver for observing performance entries.
  // Real observation by host runtime; stub records registrations.

  globalThis.PerformanceObserver = function(callback) {
    this._callback = callback;
    this._observing = [];
    this._supportedEntryTypes = ['mark', 'measure', 'navigation', 'resource', 'paint'];
  };
  globalThis.PerformanceObserver.prototype.observe = function(options) {
    var types = (options && options.type) ? [options.type] : (options && options.entryTypes) || [];
    for (var i = 0; i < types.length; i++) {
      if (this._observing.indexOf(types[i]) === -1) {
        this._observing.push(types[i]);
      }
    }
  };
  globalThis.PerformanceObserver.prototype.disconnect = function() {
    this._observing = [];
  };
  globalThis.PerformanceObserver.prototype.takeRecords = function() {
    return [];
  };
  globalThis.PerformanceObserver.supportedEntryTypes = ['mark', 'measure', 'navigation', 'resource', 'paint'];

  // ── Performance API Stub ──
  // Basic performance.now() and performance.mark/measure.

  globalThis.performance = {
    now: function() { return Date.now(); },
    mark: function(name) { return { name: name, entryType: 'mark', startTime: Date.now(), duration: 0 }; },
    measure: function(name, startMark, endMark) { return { name: name, entryType: 'measure', startTime: 0, duration: 0 }; },
    getEntries: function() { return []; },
    getEntriesByType: function(type) { return []; },
    getEntriesByName: function(name) { return []; },
    clearMarks: function() {},
    clearMeasures: function() {},
    timeOrigin: Date.now()
  };

  // ── WebAssembly API with Host Auto-Bridge ──
  // Full WebAssembly JavaScript API surface with automatic bridge to the
  // host WASM runtime (zero-wasm-sandbox / wasmi or wasmtime).
  //
  // Bridge protocol (instantiation):
  //   JS emits: __WASM_BRIDGE__:{"id":N,"bytes":"base64...","importKeys":[...]}
  //   Host compiles, instantiates, executes _start/_initialize, injects results.
  //
  // Bridge protocol (export calls):
  //   JS queues calls in WebAssembly._callQueue
  //   Host reads queue, executes via wasm-sandbox, injects results into
  //   __wasm_call_results__ for the JS side to consume on next tick.

  // Minimal base64 encoder for WASM bytes
  var __wasm_b64__ = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  function __wasmToBase64(bytes) {
    var r = '';
    for (var i = 0; i < bytes.length; i += 3) {
      var a = bytes[i], b = i+1 < bytes.length ? bytes[i+1] : 0, c = i+2 < bytes.length ? bytes[i+2] : 0;
      r += __wasm_b64__[a >> 2] + __wasm_b64__[((a & 3) << 4) | (b >> 4)];
      r += (i+1 < bytes.length) ? __wasm_b64__[((b & 15) << 2) | (c >> 6)] : '=';
      r += (i+2 < bytes.length) ? __wasm_b64__[c & 63] : '=';
    }
    return r;
  }

  // 从 ArrayBuffer/TypedArray 提取字节
  function __wasmToBytes(bufferSource) {
    if (bufferSource instanceof ArrayBuffer) return new Uint8Array(bufferSource);
    if (bufferSource instanceof Uint8Array) return bufferSource;
    if (ArrayBuffer.isView(bufferSource)) return new Uint8Array(bufferSource.buffer, bufferSource.byteOffset, bufferSource.byteLength);
    return null;
  }

  globalThis.WebAssembly = {
    _modules: {},
    _instances: {},
    _nextId: 1,
    _pendingBridge: null,
    // 导出函数调用队列 — 每次调用存储 {instanceId, name, args, callId}
    _callQueue: [],
    _nextCallId: 1,
    // 调用结果缓存 — host 执行后注入 {callId: result}
    _callResults: {},

    compile: function(bufferSource) {
      var bytes = __wasmToBytes(bufferSource);
      if (!bytes) return Promise.reject(new TypeError('WebAssembly.compile(): Argument 0 must be a buffer source'));
      var id = this._nextId++;
      this._modules[id] = bytes;
      // 发送编译桥接命令
      var b64 = __wasmToBase64(bytes);
      this._pendingBridge = '__WASM_COMPILE__:' + JSON.stringify({id: id, bytes: b64});
      // 如果 host 已经预编译并注入了结果，直接返回
      if (globalThis.__wasm_compiled__ && globalThis.__wasm_compiled__[id]) {
        var compiled = globalThis.__wasm_compiled__[id];
        delete globalThis.__wasm_compiled__[id];
        return Promise.resolve(compiled);
      }
      return Promise.resolve({
        _id: id,
        _bytes: bytes,
        _compiled: true,
        exports: function() { return []; }
      });
    },

    instantiate: function(bufferSourceOrModule, importObject) {
      var self = this;
      var bytes;
      if (bufferSourceOrModule && bufferSourceOrModule._id !== undefined && bufferSourceOrModule._compiled) {
        // 第二种形式: instantiate(Module, imports)
        bytes = bufferSourceOrModule._bytes;
      } else {
        bytes = __wasmToBytes(bufferSourceOrModule);
        if (!bytes) return Promise.reject(new TypeError('WebAssembly.instantiate(): Argument 0 must be a buffer source or Module'));
      }
      var moduleId = self._nextId++;
      self._modules[moduleId] = bytes;
      var instanceId = self._nextId++;

      // 收集 importObject 的键名传给 host
      var importKeys = [];
      if (importObject) {
        try {
          var moduleKeys = Object.keys(importObject);
          for (var mi = 0; mi < moduleKeys.length; mi++) {
            var modName = moduleKeys[mi];
            var modVal = importObject[modName];
            if (modVal && typeof modVal === 'object') {
              var fnKeys = Object.keys(modVal);
              for (var fi = 0; fi < fnKeys.length; fi++) {
                importKeys.push(modName + '.' + fnKeys[fi]);
              }
            }
          }
        } catch(e) {}
      }

      // 发送实例化桥接命令
      var b64 = __wasmToBase64(bytes);
      self._pendingBridge = '__WASM_BRIDGE__:' + JSON.stringify({id: instanceId, bytes: b64, importKeys: importKeys});

      // 如果 host 已经预解析了此实例（第二次 instantiate 同一模块），直接返回缓存
      if (globalThis.__wasm_results__ && globalThis.__wasm_results__[instanceId]) {
        var resolved = globalThis.__wasm_results__[instanceId];
        delete globalThis.__wasm_results__[instanceId];
        return Promise.resolve({
          module: { _id: moduleId, _bytes: bytes, _compiled: true },
          instance: resolved
        });
      }

      // 创建带有可调用导出的桩实例
      var stub = self._createInstance(instanceId, bytes, importObject);
      self._instances[instanceId] = { moduleId: moduleId, imports: importObject || {}, stub: stub };
      return Promise.resolve({
        module: { _id: moduleId, _bytes: bytes, _compiled: true },
        instance: stub
      });
    },

    // WebAssembly.instantiateStreaming() — 从 Response 流式编译
    instantiateStreaming: function(source, importObject) {
      var self = this;
      // 在无头环境中 Response 可能不可用，回退到 ArrayBuffer 路径
      if (source && typeof source.arrayBuffer === 'function') {
        return source.arrayBuffer().then(function(buffer) {
          return self.instantiate(new Uint8Array(buffer), importObject);
        });
      }
      // 如果 source 已经是 ArrayBuffer/Uint8Array，直接使用
      if (source instanceof ArrayBuffer || source instanceof Uint8Array) {
        return self.instantiate(source, importObject);
      }
      return Promise.reject(new TypeError('WebAssembly.instantiateStreaming(): source must be a Response or buffer source'));
    },

    _createInstance: function(instanceId, bytes, importObject) {
      var self = this;
      return {
        _id: instanceId,
        exports: {
          memory: {
            buffer: new ArrayBuffer(65536),
            grow: function(delta) { return 1; },
            byteLength: 65536
          },
          __wasm_export_names__: [],
          // 占位：host 注入后会被真实可调用函数替换
          __host_backed__: false
        }
      };
    },

    validate: function(bufferSource) {
      if (!bufferSource) return false;
      var bytes = __wasmToBytes(bufferSource);
      if (!bytes || bytes.length < 8) return false;
      // WASM 魔术字节: 0x00 0x61 0x73 0x6D (即 \0asm)
      return bytes[0] === 0x00 && bytes[1] === 0x61 && bytes[2] === 0x73 && bytes[3] === 0x6D;
    }
  };

  // ── navigator.serviceWorker API Stub ──
  // Provides navigator.serviceWorker.register() for JS-based SW registration.
  // The host runtime (WebView) processes registrations via DomCommand.

  if (!globalThis.navigator) globalThis.navigator = {};
  globalThis.navigator.serviceWorker = {
    _registrations: [],
    _controller: null,
    _ready: Promise.resolve(null),

    register: function(scriptURL, options) {
      if (!scriptURL || typeof scriptURL !== 'string') {
        return Promise.reject(new TypeError('ServiceWorker.register: scriptURL is required'));
      }
      var scope = (options && options.scope) || scriptURL.substring(0, scriptURL.lastIndexOf('/') + 1);
      var reg = {
        _scriptURL: scriptURL,
        _scope: scope,
        installing: null,
        waiting: null,
        active: null,
        scope: scope,
        unregister: function() { return Promise.resolve(true); },
        update: function() { return Promise.resolve(); }
      };
      this._registrations.push(reg);
      // Simulate install → activate lifecycle
      reg.installing = { scriptURL: scriptURL, state: 'installing' };
      var self = this;
      // Simulate async lifecycle transitions
      setTimeout(function() {
        reg.waiting = { scriptURL: scriptURL, state: 'installed' };
        reg.installing = null;
      }, 0);
      setTimeout(function() {
        reg.active = { scriptURL: scriptURL, state: 'activated' };
        reg.waiting = null;
        self._controller = reg.active;
      }, 0);
      return Promise.resolve(reg);
    },

    getRegistration: function(scope) {
      for (var i = 0; i < this._registrations.length; i++) {
        if (!scope || this._registrations[i].scope === scope) {
          return Promise.resolve(this._registrations[i]);
        }
      }
      return Promise.resolve(undefined);
    },

    getRegistrations: function() {
      return Promise.resolve(this._registrations.slice());
    },

    ready: Promise.resolve(null),

    oncontrollerchange: null,
    onmessage: null
  };

  // ── Web Worker API Stub ──
  // Provides the Dedicated Worker constructor (new Worker()).
  // In this polyfill environment, Workers execute synchronously in the same thread.
  // Real implementations would use separate V8 isolates or processes.

  function Worker(scriptURL, options) {
    if (!scriptURL || typeof scriptURL !== 'string') {
      throw new TypeError('Worker: scriptURL is required and must be a string');
    }
    this._scriptURL = scriptURL;
    this._options = options || {};
    this._terminated = false;
    this._listeners = {};
    this.onerror = null;
    this.onmessage = null;

    // Simulate worker script loading
    var self = this;
    setTimeout(function() {
      if (self._terminated) return;
      // In a real implementation, the script would be loaded and executed
      // in a separate context. Here we just simulate the ready state.
    }, 0);
  }

  Worker.prototype.postMessage = function(message, transfer) {
    if (this._terminated) {
      throw new Error('Cannot postMessage to a terminated Worker');
    }
    // In a real implementation, this would serialize the message and
    // send it to the worker thread. Here we store it for testing.
    this._lastMessage = message;
    this._lastTransfer = transfer;

    // Trigger onmessage if set (simulating echo behavior for testing)
    var self = this;
    if (self.onmessage) {
      setTimeout(function() {
        if (!self._terminated && self.onmessage) {
          self.onmessage({ data: message });
        }
      }, 0);
    }
    // Dispatch to listeners
    if (this._listeners['message']) {
      var listeners = this._listeners['message'].slice();
      setTimeout(function() {
        if (!self._terminated) {
          for (var i = 0; i < listeners.length; i++) {
            listeners[i]({ data: message });
          }
        }
      }, 0);
    }
  };

  Worker.prototype.terminate = function() {
    this._terminated = true;
    this._listeners = {};
    this.onmessage = null;
    this.onerror = null;
  };

  Worker.prototype.addEventListener = function(type, listener) {
    if (!this._listeners[type]) this._listeners[type] = [];
    this._listeners[type].push(listener);
  };

  Worker.prototype.removeEventListener = function(type, listener) {
    if (!this._listeners[type]) return;
    var idx = this._listeners[type].indexOf(listener);
    if (idx >= 0) this._listeners[type].splice(idx, 1);
  };

  Worker.prototype.dispatchEvent = function(event) {
    // Dispatch to on-type handler first, then listeners
    var type = event.type || event;
    var handler = this['on' + type];
    if (typeof handler === 'function') handler(event);
    if (this._listeners[type]) {
      var listeners = this._listeners[type].slice();
      for (var i = 0; i < listeners.length; i++) listeners[i](event);
    }
    return true;
  };

  globalThis.Worker = Worker;

  // ── ES Module Support Stub ──
  // Provides import() dynamic import and module-related globals.
  // Real ES Module loading requires the network stack and module resolution.

  // Dynamic import() — returns a Promise that resolves to a module namespace.
  // In polyfill mode, returns an empty module namespace object.
  globalThis.import = function(specifier) {
    if (typeof specifier !== 'string') {
      return Promise.reject(new TypeError('import() requires a module specifier string'));
    }
    // Simulate async module loading
    return new Promise(function(resolve, reject) {
      setTimeout(function() {
        // In a real implementation, this would:
        // 1. Resolve the specifier to a URL
        // 2. Fetch the module source
        // 3. Parse and compile the module
        // 4. Execute the module and return its namespace
        //
        // For the polyfill, return a namespace-like object
        // that records the import for testing purposes.
        resolve({
          __esModule: true,
          __importedFrom: specifier,
          default: undefined
        });
      }, 0);
    });
  };

  // import.meta — available in ES Module context.
  // In polyfill mode, provide a basic stub.
  // Note: import.meta is only available inside <script type="module">
  // and cannot be polyfilled in classic scripts, but we provide it
  // for feature detection purposes.
  if (typeof globalThis.importMeta === 'undefined') {
    Object.defineProperty(globalThis, 'importMeta', {
      get: function() {
        return {
          url: (typeof globalThis.location !== 'undefined' && globalThis.location.href)
            || 'about:blank',
          resolve: function(specifier) {
            // Basic URL resolution stub
            return specifier;
          }
        };
      },
      configurable: true
    });
  }

})();
"#
        .to_string()
}

#[cfg(test)]
#[path = "dom_bridge_tests.rs"]
mod dom_bridge_tests;

#[cfg(test)]
#[path = "dom_bridge_extended_tests.rs"]
mod dom_bridge_extended_tests;
