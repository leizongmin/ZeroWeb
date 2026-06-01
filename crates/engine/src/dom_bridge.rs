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
    /// 下一个可用的 JS 句柄 ID。
    next_handle: u64,
}

impl DomBridge {
    /// 创建新的 DOM 桥接器。
    pub fn new() -> Self {
        Self {
            handle_map: std::collections::HashMap::new(),
            next_handle: 1,
        }
    }

    /// 注册一个 DOM NodeId，返回 JS 可用的句柄 ID。
    ///
    /// 如果该 NodeId 已经注册过，返回已有句柄。
    pub fn register(&mut self, node_id: u64) -> u64 {
        // 检查是否已注册
        for (&handle, &nid) in &self.handle_map {
            if nid == node_id {
                return handle;
            }
        }
        let handle = self.next_handle;
        self.next_handle += 1;
        self.handle_map.insert(handle, node_id);
        handle
    }

    /// 通过句柄查找 DOM NodeId。
    pub fn resolve(&self, handle: u64) -> Option<u64> {
        self.handle_map.get(&handle).copied()
    }

    /// 移除句柄映射。
    pub fn unregister(&mut self, handle: u64) {
        self.handle_map.remove(&handle);
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
    let result = input[1..=end].to_string();

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
    if (selector.startsWith('#')) {
      return node.attributes.id === selector.substring(1);
    } else if (selector.startsWith('.')) {
      var cls = selector.substring(1);
      if (node.attributes['class']) {
        return node.attributes['class'].split(/\s+/).indexOf(cls) >= 0;
      }
      return false;
    } else {
      return node.tagName === selector.toUpperCase();
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

  globalThis.fetch = function(input, init) {
    var req = (input instanceof globalThis.Request) ? input : new globalThis.Request(input, init);
    // Stub: return empty 200 response. Real network handled by host runtime.
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
})();
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── DomBridge 测试 ──

    #[test]
    fn test_dom_bridge_new() {
        let bridge = DomBridge::new();
        assert!(bridge.is_empty());
        assert_eq!(bridge.len(), 0);
    }

    #[test]
    fn test_dom_bridge_register() {
        let mut bridge = DomBridge::new();
        let h1 = bridge.register(100);
        let h2 = bridge.register(200);
        assert_ne!(h1, h2, "Handles should be unique");
        assert_eq!(bridge.len(), 2);
    }

    #[test]
    fn test_dom_bridge_register_same_node_returns_same_handle() {
        let mut bridge = DomBridge::new();
        let h1 = bridge.register(100);
        let h2 = bridge.register(100);
        assert_eq!(h1, h2, "Same node should get same handle");
        assert_eq!(bridge.len(), 1);
    }

    #[test]
    fn test_dom_bridge_resolve() {
        let mut bridge = DomBridge::new();
        let h = bridge.register(42);
        assert_eq!(bridge.resolve(h), Some(42));
        assert_eq!(bridge.resolve(999), None);
    }

    #[test]
    fn test_dom_bridge_unregister() {
        let mut bridge = DomBridge::new();
        let h = bridge.register(42);
        bridge.unregister(h);
        assert_eq!(bridge.resolve(h), None);
        assert!(bridge.is_empty());
    }

    #[test]
    fn test_dom_bridge_clear() {
        let mut bridge = DomBridge::new();
        bridge.register(1);
        bridge.register(2);
        bridge.register(3);
        bridge.clear();
        assert!(bridge.is_empty());
    }

    #[test]
    fn test_dom_bridge_default() {
        let bridge = DomBridge::default();
        assert!(bridge.is_empty());
    }

    // ── DomCommand 解析测试 ──

    #[test]
    fn test_parse_get_element_by_id() {
        let cmd = DomBridge::parse_command(r#"document.getElementById("foo")"#);
        assert_eq!(cmd, Some(DomCommand::GetElementById { id: "foo".to_string() }));
    }

    #[test]
    fn test_parse_get_element_by_id_single_quotes() {
        let cmd = DomBridge::parse_command("document.getElementById('bar')");
        assert_eq!(cmd, Some(DomCommand::GetElementById { id: "bar".to_string() }));
    }

    #[test]
    fn test_parse_query_selector() {
        let cmd = DomBridge::parse_command(r#"document.querySelector("div.container")"#);
        assert_eq!(
            cmd,
            Some(DomCommand::QuerySelector {
                selector: "div.container".to_string()
            })
        );
    }

    #[test]
    fn test_parse_query_selector_all() {
        let cmd = DomBridge::parse_command(r#"document.querySelectorAll("li")"#);
        assert_eq!(
            cmd,
            Some(DomCommand::QuerySelectorAll {
                selector: "li".to_string()
            })
        );
    }

    #[test]
    fn test_parse_create_element() {
        let cmd = DomBridge::parse_command(r#"document.createElement("div")"#);
        assert_eq!(
            cmd,
            Some(DomCommand::CreateElement {
                tag_name: "div".to_string()
            })
        );
    }

    #[test]
    fn test_parse_create_text_node() {
        let cmd = DomBridge::parse_command(r#"document.createTextNode("Hello")"#);
        assert_eq!(
            cmd,
            Some(DomCommand::CreateTextNode {
                text: "Hello".to_string()
            })
        );
    }

    #[test]
    fn test_parse_get_elements_by_class_name() {
        let cmd = DomBridge::parse_command(r#"document.getElementsByClassName("active")"#);
        assert_eq!(
            cmd,
            Some(DomCommand::GetElementsByClassName {
                class_name: "active".to_string()
            })
        );
    }

    #[test]
    fn test_parse_get_elements_by_tag_name() {
        let cmd = DomBridge::parse_command(r#"document.getElementsByTagName("div")"#);
        assert_eq!(
            cmd,
            Some(DomCommand::GetElementsByTagName {
                tag_name: "div".to_string()
            })
        );
    }

    #[test]
    fn test_parse_unknown_command() {
        let cmd = DomBridge::parse_command("window.alert('hi')");
        assert_eq!(cmd, None);
    }

    #[test]
    fn test_parse_empty_input() {
        let cmd = DomBridge::parse_command("");
        assert_eq!(cmd, None);
    }

    #[test]
    fn test_parse_invalid_no_quotes() {
        let cmd = DomBridge::parse_command("document.getElementById(foo)");
        assert_eq!(cmd, None);
    }

    // ── DomResult 测试 ──

    #[test]
    fn test_dom_result_element() {
        let result = DomResult::Element(Some(42));
        assert_eq!(result, DomResult::Element(Some(42)));
    }

    #[test]
    fn test_dom_result_element_none() {
        let result = DomResult::Element(None);
        assert_eq!(result, DomResult::Element(None));
    }

    #[test]
    fn test_dom_result_element_list() {
        let result = DomResult::ElementList(vec![1, 2, 3]);
        assert_eq!(result, DomResult::ElementList(vec![1, 2, 3]));
    }

    #[test]
    fn test_dom_result_string() {
        let result = DomResult::String(Some("hello".to_string()));
        assert_eq!(result, DomResult::String(Some("hello".to_string())));
    }

    #[test]
    fn test_dom_result_bool() {
        assert_eq!(DomResult::Bool(true), DomResult::Bool(true));
        assert_eq!(DomResult::Bool(false), DomResult::Bool(false));
    }

    #[test]
    fn test_dom_result_void() {
        assert_eq!(DomResult::Void, DomResult::Void);
    }

    #[test]
    fn test_dom_result_error() {
        let result = DomResult::Error("not found".to_string());
        assert_eq!(result, DomResult::Error("not found".to_string()));
    }

    // ── Polyfill 生成测试 ──

    #[test]
    fn test_generate_dom_api_polyfill_not_empty() {
        let polyfill = generate_dom_api_polyfill();
        assert!(!polyfill.is_empty());
        assert!(polyfill.contains("document"));
        assert!(polyfill.contains("getElementById"));
        assert!(polyfill.contains("querySelector"));
        assert!(polyfill.contains("createElement"));
        assert!(polyfill.contains("appendChild"));
        assert!(polyfill.contains("setAttribute"));
        assert!(polyfill.contains("textContent"));
    }

    #[test]
    fn test_extract_string_arg_double_quotes() {
        let result = extract_string_arg(r#""hello")"#);
        assert_eq!(result, Some("hello".to_string()));
    }

    #[test]
    fn test_extract_string_arg_single_quotes() {
        let result = extract_string_arg("'world')");
        assert_eq!(result, Some("world".to_string()));
    }

    #[test]
    fn test_extract_string_arg_empty() {
        let result = extract_string_arg("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_string_arg_no_closing_paren() {
        let result = extract_string_arg(r#""hello""#);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_string_arg_no_quotes() {
        let result = extract_string_arg("hello)");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_string_arg_with_spaces() {
        let result = extract_string_arg(r#"  "hello"  )"#);
        assert_eq!(result, Some("hello".to_string()));
    }

    // ── 事件命令构造测试 ──

    #[test]
    fn test_add_event_listener_command_equality() {
        let cmd = DomCommand::AddEventListener {
            element_id: 42,
            event_type: "click".to_string(),
            capture: false,
        };
        assert_eq!(
            cmd,
            DomCommand::AddEventListener {
                element_id: 42,
                event_type: "click".to_string(),
                capture: false,
            }
        );
    }

    #[test]
    fn test_add_event_listener_capture() {
        let cmd = DomCommand::AddEventListener {
            element_id: 1,
            event_type: "keydown".to_string(),
            capture: true,
        };
        match cmd {
            DomCommand::AddEventListener { capture, .. } => assert!(capture),
            _ => panic!("Expected AddEventListener"),
        }
    }

    #[test]
    fn test_remove_event_listener_command() {
        let cmd = DomCommand::RemoveEventListener {
            element_id: 10,
            event_type: "input".to_string(),
        };
        assert_eq!(
            cmd,
            DomCommand::RemoveEventListener {
                element_id: 10,
                event_type: "input".to_string(),
            }
        );
    }

    #[test]
    fn test_dispatch_event_command() {
        let cmd = DomCommand::DispatchEvent {
            target_id: 5,
            event_type: "custom".to_string(),
            bubbles: true,
            cancelable: false,
        };
        assert_eq!(
            cmd,
            DomCommand::DispatchEvent {
                target_id: 5,
                event_type: "custom".to_string(),
                bubbles: true,
                cancelable: false,
            }
        );
    }

    #[test]
    fn test_dispatch_event_no_bubble() {
        let cmd = DomCommand::DispatchEvent {
            target_id: 1,
            event_type: "change".to_string(),
            bubbles: false,
            cancelable: true,
        };
        match cmd {
            DomCommand::DispatchEvent {
                bubbles, cancelable, ..
            } => {
                assert!(!bubbles);
                assert!(cancelable);
            }
            _ => panic!("Expected DispatchEvent"),
        }
    }

    // ── Polyfill 事件系统测试 ──

    #[test]
    fn test_polyfill_contains_event_system() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("addEventListener"));
        assert!(polyfill.contains("removeEventListener"));
        assert!(polyfill.contains("dispatchEvent"));
    }

    #[test]
    fn test_polyfill_contains_custom_event() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("CustomEvent"));
        assert!(polyfill.contains("preventDefault"));
        assert!(polyfill.contains("stopPropagation"));
        assert!(polyfill.contains("stopImmediatePropagation"));
    }

    #[test]
    fn test_polyfill_event_options_capture() {
        let polyfill = generate_dom_api_polyfill();
        // Verify the polyfill handles capture option
        assert!(polyfill.contains("capture"));
        assert!(polyfill.contains("_eventListeners"));
    }

    // ── Polyfill Fetch API 测试 ──

    #[test]
    fn test_polyfill_contains_fetch_api() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.fetch"));
        assert!(polyfill.contains("globalThis.Headers"));
        assert!(polyfill.contains("globalThis.Request"));
        assert!(polyfill.contains("globalThis.Response"));
    }

    #[test]
    fn test_polyfill_contains_response_methods() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("prototype.json"));
        assert!(polyfill.contains("prototype.text"));
        assert!(polyfill.contains("prototype.clone"));
        assert!(polyfill.contains("Response.error"));
    }

    #[test]
    fn test_polyfill_contains_headers_methods() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("prototype.append"));
        assert!(polyfill.contains("prototype.delete"));
        assert!(polyfill.contains("prototype.get"));
        assert!(polyfill.contains("prototype.has"));
        assert!(polyfill.contains("prototype.set"));
    }

    // ── Polyfill Console + Timer API 测试 ──

    #[test]
    fn test_polyfill_contains_console_api() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.console"));
        assert!(polyfill.contains("log: function"));
        assert!(polyfill.contains("warn: function"));
        assert!(polyfill.contains("error: function"));
        assert!(polyfill.contains("info: function"));
        assert!(polyfill.contains("time: function"));
        assert!(polyfill.contains("timeEnd: function"));
    }

    #[test]
    fn test_polyfill_contains_timer_api() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.setTimeout"));
        assert!(polyfill.contains("globalThis.setInterval"));
        assert!(polyfill.contains("globalThis.clearTimeout"));
        assert!(polyfill.contains("globalThis.clearInterval"));
    }

    // ── Polyfill Web Storage API 测试 ──

    #[test]
    fn test_polyfill_contains_storage_api() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.localStorage"));
        assert!(polyfill.contains("globalThis.sessionStorage"));
        assert!(polyfill.contains("getItem"));
        assert!(polyfill.contains("setItem"));
        assert!(polyfill.contains("removeItem"));
    }

    // ── Polyfill MutationObserver 测试 ──

    #[test]
    fn test_polyfill_contains_mutation_observer() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.MutationObserver"));
        assert!(polyfill.contains("prototype.observe"));
        assert!(polyfill.contains("prototype.disconnect"));
        assert!(polyfill.contains("prototype.takeRecords"));
    }

    #[test]
    fn test_polyfill_contains_mutation_record() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.MutationRecord"));
        assert!(polyfill.contains("addedNodes"));
        assert!(polyfill.contains("removedNodes"));
        assert!(polyfill.contains("attributeName"));
    }

    // ── Polyfill IntersectionObserver 测试 ──

    #[test]
    fn test_polyfill_contains_intersection_observer() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.IntersectionObserver"));
        assert!(polyfill.contains("prototype.observe"));
        assert!(polyfill.contains("prototype.unobserve"));
        assert!(polyfill.contains("prototype.disconnect"));
        assert!(polyfill.contains("prototype.takeRecords"));
    }

    #[test]
    fn test_polyfill_contains_intersection_observer_entry() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.IntersectionObserverEntry"));
        assert!(polyfill.contains("isIntersecting"));
        assert!(polyfill.contains("intersectionRatio"));
    }

    // ── Polyfill ResizeObserver 测试 ──

    #[test]
    fn test_polyfill_contains_resize_observer() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("globalThis.ResizeObserver"));
        assert!(polyfill.contains("globalThis.ResizeObserverEntry"));
        assert!(polyfill.contains("globalThis.DOMRectReadOnly"));
    }

    // ── 新增 DomCommand 变体测试 ──

    #[test]
    fn test_insert_before_command() {
        let cmd = DomCommand::InsertBefore {
            parent_id: 1,
            new_child_id: 2,
            ref_child_id: Some(3),
        };
        assert_eq!(
            cmd,
            DomCommand::InsertBefore {
                parent_id: 1,
                new_child_id: 2,
                ref_child_id: Some(3),
            }
        );
    }

    #[test]
    fn test_insert_before_no_ref() {
        let cmd = DomCommand::InsertBefore {
            parent_id: 1,
            new_child_id: 2,
            ref_child_id: None,
        };
        match cmd {
            DomCommand::InsertBefore { ref_child_id, .. } => assert!(ref_child_id.is_none()),
            _ => panic!("Expected InsertBefore"),
        }
    }

    #[test]
    fn test_replace_child_command() {
        let cmd = DomCommand::ReplaceChild {
            parent_id: 1,
            new_child_id: 2,
            old_child_id: 3,
        };
        assert_eq!(
            cmd,
            DomCommand::ReplaceChild {
                parent_id: 1,
                new_child_id: 2,
                old_child_id: 3,
            }
        );
    }

    #[test]
    fn test_clone_node_command() {
        let cmd = DomCommand::CloneNode {
            element_id: 42,
            deep: true,
        };
        assert_eq!(
            cmd,
            DomCommand::CloneNode {
                element_id: 42,
                deep: true,
            }
        );
    }

    #[test]
    fn test_clone_node_shallow() {
        let cmd = DomCommand::CloneNode {
            element_id: 5,
            deep: false,
        };
        match cmd {
            DomCommand::CloneNode { element_id, deep } => {
                assert_eq!(element_id, 5);
                assert!(!deep);
            }
            _ => panic!("Expected CloneNode"),
        }
    }

    #[test]
    fn test_get_style_command() {
        let cmd = DomCommand::GetStyle { element_id: 10 };
        assert_eq!(cmd, DomCommand::GetStyle { element_id: 10 });
    }

    #[test]
    fn test_set_style_command() {
        let cmd = DomCommand::SetStyle {
            element_id: 10,
            value: "color: red; font-size: 16px".to_string(),
        };
        assert_eq!(
            cmd,
            DomCommand::SetStyle {
                element_id: 10,
                value: "color: red; font-size: 16px".to_string(),
            }
        );
    }

    #[test]
    fn test_set_inner_html_command() {
        let cmd = DomCommand::SetInnerHtml {
            element_id: 7,
            value: "<p>Hello</p>".to_string(),
        };
        assert_eq!(
            cmd,
            DomCommand::SetInnerHtml {
                element_id: 7,
                value: "<p>Hello</p>".to_string(),
            }
        );
    }

    #[test]
    fn test_get_parent_node_command() {
        let cmd = DomCommand::GetParentNode { element_id: 3 };
        assert_eq!(cmd, DomCommand::GetParentNode { element_id: 3 });
    }

    // ── Polyfill 新增 API 验证测试 ──

    #[test]
    fn test_polyfill_contains_insert_before() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("insertBefore"));
    }

    #[test]
    fn test_polyfill_contains_replace_child() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("replaceChild"));
    }

    #[test]
    fn test_polyfill_contains_clone_node() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("cloneNode"));
    }

    #[test]
    fn test_polyfill_contains_navigation_properties() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("firstChild"));
        assert!(polyfill.contains("lastChild"));
        assert!(polyfill.contains("nextSibling"));
        assert!(polyfill.contains("previousSibling"));
        assert!(polyfill.contains("childElementCount"));
        assert!(polyfill.contains("hasChildNodes"));
    }

    #[test]
    fn test_polyfill_contains_style_api() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("_CSSStyleDeclaration"));
        assert!(polyfill.contains("getPropertyValue"));
        assert!(polyfill.contains("setProperty"));
        assert!(polyfill.contains("removeProperty"));
        assert!(polyfill.contains("cssText"));
    }

    #[test]
    fn test_polyfill_contains_classlist_api() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("_DOMTokenList"));
        assert!(polyfill.contains("classList"));
        assert!(polyfill.contains("contains"));
        assert!(polyfill.contains("toggle"));
    }

    #[test]
    fn test_polyfill_contains_inner_html_setter() {
        let polyfill = generate_dom_api_polyfill();
        // innerHTML getter + setter
        assert!(polyfill.contains("innerHTML"));
        assert!(polyfill.contains("outerHTML"));
    }

    #[test]
    fn test_polyfill_contains_text_content_property() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("textContent"));
        assert!(polyfill.contains("innerText"));
        assert!(polyfill.contains("getTextContent"));
        assert!(polyfill.contains("setTextContent"));
    }

    #[test]
    fn test_polyfill_contains_document_fragment() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("createDocumentFragment"));
    }

    #[test]
    fn test_polyfill_contains_id_classname() {
        let polyfill = generate_dom_api_polyfill();
        assert!(polyfill.contains("node.id"));
        assert!(polyfill.contains("node.className"));
    }
}
